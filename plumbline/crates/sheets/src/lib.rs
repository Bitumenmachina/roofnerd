//! Tables people already have, read for bringing in: a supplier's price list, a prior
//! estimate, an assembly library, as a CSV file or as a sheet of an XLSX workbook.
//!
//! What comes out is rows of cells as text, and a list of what was in the file and is
//! not in those rows. What the rows mean is not decided here: `plumbline_world::tables`
//! reads a price list or an estimate out of them. Nothing here touches a disk.
//!
//! Ours, on two libraries: `csv` (MIT or Unlicense) reads and writes CSV as RFC 4180 has
//! it, and `calamine` (MIT) reads a workbook's cells. BIMCraft writes one sheet of XLSX
//! by hand and reads none back (`xlsx.rs`: "Nothing is read back").
//!
//! CSV comes back out as it went in: `csv` of the rows `read` gave is the same table. A
//! workbook does not, and says what it lost: its other sheets, its formulas (a cell is
//! read as what the formula last worked out to), its merged cells, its formats.

use std::io::Cursor;

use calamine::{Data, Reader, Xlsx};

/// What sort of file a table came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Csv,
    Xlsx,
}

impl Sort {
    pub fn said(self) -> &'static str {
        match self {
            Sort::Csv => "CSV",
            Sort::Xlsx => "XLSX",
        }
    }
}

/// One table as read.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    pub sort: Sort,
    /// The workbook's sheet that was read; empty for a CSV file.
    pub name: String,
    /// Every row that holds anything, each cut after its last cell that holds anything.
    pub rows: Vec<Vec<String>>,
    /// What the file held that is not in the rows, and what was guessed in reading it.
    pub lost: Vec<String>,
}

/// The most rows read from one table. A longer one is cut there, and that is said.
pub const MOST_ROWS: usize = 200_000;

/// What sort of file this is, from what is in it: a workbook is a zip, a CSV file is
/// text. None for anything else.
pub fn sort_of(bytes: &[u8]) -> Option<Sort> {
    if bytes.starts_with(b"PK\x03\x04") {
        return Some(Sort::Xlsx);
    }
    // Text: no byte below a space but a tab, a line feed or a carriage return.
    let text = !bytes.is_empty() && !bytes.iter().any(|b| *b < 0x20 && !matches!(*b, b'\t' | b'\n' | b'\r'));
    text.then_some(Sort::Csv)
}

/// Rows with nothing in them dropped, and each row cut after its last cell with anything
/// in it. Cells keep their place across: an empty cell between two full ones stays.
fn tidy(rows: Vec<Vec<String>>) -> Vec<Vec<String>> {
    rows.into_iter()
        .map(|mut row| {
            while row.last().is_some_and(|c| c.trim().is_empty()) {
                row.pop();
            }
            row
        })
        .filter(|row| !row.is_empty())
        .collect()
}

/// Read a table. `sheet` names the workbook's sheet to read; with none, its first.
pub fn read(bytes: &[u8], sheet: Option<&str>) -> Result<Sheet, String> {
    match sort_of(bytes) {
        None => Err("it is not a CSV file or an XLSX workbook".into()),
        Some(Sort::Csv) => {
            if sheet.is_some() {
                return Err("a CSV file is one table and has no sheets to name".into());
            }
            read_csv(bytes)
        }
        Some(Sort::Xlsx) => read_xlsx(bytes, sheet),
    }
}

fn read_csv(bytes: &[u8]) -> Result<Sheet, String> {
    let mut lost = Vec::new();
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    // Text as UTF-8; if it is not, as the Western European of older spreadsheets, which
    // is said, because a letter may have been read wrong.
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => {
            lost.push("the file is not UTF-8 text: it was read as Western European (Latin-1), and an accented letter or a symbol may be wrong".to_string());
            bytes.iter().map(|b| char::from(*b)).collect()
        }
    };
    // What parts its cells: a comma, unless its first line has none and has semicolons
    // or tabs, as a spreadsheet set for another country writes it.
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let count = |c: char| first.matches(c).count();
    let delimiter = if count(',') == 0 && count(';') > 0 {
        lost.push("its cells are parted by semicolons, not commas: read that way".to_string());
        b';'
    } else if count(',') == 0 && count('\t') > 0 {
        lost.push("its cells are parted by tabs, not commas: read that way".to_string());
        b'\t'
    } else {
        b','
    };
    let mut reader = csv::ReaderBuilder::new().has_headers(false).flexible(true).delimiter(delimiter).from_reader(text.as_bytes());
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut over = 0usize;
    for record in reader.records() {
        let record = record.map_err(|e| format!("the CSV could not be read ({e})"))?;
        if rows.len() >= MOST_ROWS {
            over += 1;
            continue;
        }
        rows.push(record.iter().map(str::to_string).collect());
    }
    if over > 0 {
        lost.push(format!("{over} rows after the first {MOST_ROWS} were not read"));
    }
    Ok(Sheet { sort: Sort::Csv, name: String::new(), rows: tidy(rows), lost })
}

/// A number as a spreadsheet shows it with no format: 4.1, not 4.0999999999999996; 12,
/// not 12.0.
fn number(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1.0e15 {
        format!("{x:.0}")
    } else {
        // Fifteen figures is what a spreadsheet keeps; the sixteenth is noise.
        let s = format!("{x:.15e}");
        let parsed: f64 = s.parse().unwrap_or(x);
        let short = format!("{}", (parsed * 1.0e12).round() / 1.0e12);
        if short.parse::<f64>().is_ok_and(|y| (y - x).abs() <= x.abs() * 1.0e-12) {
            short
        } else {
            format!("{x}")
        }
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

fn read_xlsx(bytes: &[u8], sheet: Option<&str>) -> Result<Sheet, String> {
    let mut book: Xlsx<_> = Xlsx::new(Cursor::new(bytes)).map_err(|e| format!("the workbook could not be read ({e})"))?;
    let names = book.sheet_names();
    let name = match sheet {
        Some(asked) => names.iter().find(|n| n.eq_ignore_ascii_case(asked)).cloned().ok_or_else(|| format!("the workbook has no sheet {asked:?}; it has: {}", names.join(", ")))?,
        None => names.first().cloned().ok_or("the workbook has no sheets")?,
    };
    let mut lost = Vec::new();
    let others: Vec<&String> = names.iter().filter(|n| **n != name).collect();
    if !others.is_empty() {
        lost.push(format!(
            "the sheet {name:?} was read; {} not: {}",
            if others.len() == 1 { "1 other sheet was".to_string() } else { format!("{} other sheets were", others.len()) },
            others.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join(", ")
        ));
    }
    let range = book.worksheet_range(&name).map_err(|e| format!("the sheet {name:?} could not be read ({e})"))?;
    let (mut errors, mut dates) = (0usize, 0usize);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut over = 0usize;
    // A sheet's used cells may start below its first row or right of its first column;
    // the rows keep their cells' places from the first used column.
    for row in range.rows() {
        if rows.len() >= MOST_ROWS {
            over += 1;
            continue;
        }
        rows.push(
            row.iter()
                .map(|cell| match cell {
                    Data::Empty => String::new(),
                    Data::String(s) => s.clone(),
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => number(*f),
                    Data::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                    Data::DateTime(d) => {
                        dates += 1;
                        number(d.as_f64())
                    }
                    Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
                    Data::Error(_) => {
                        errors += 1;
                        String::new()
                    }
                })
                .collect(),
        );
    }
    if over > 0 {
        lost.push(format!("{over} rows after the first {MOST_ROWS} were not read"));
    }
    if let Ok(formulas) = book.worksheet_formula(&name) {
        let n = formulas.rows().flat_map(|r| r.iter()).filter(|f| !f.is_empty()).count();
        if n > 0 {
            lost.push(format!("{} a formula: what it last worked out to was read, not the formula", if n == 1 { "1 cell holds".to_string() } else { format!("{n} cells hold") }));
        }
    }
    if errors > 0 {
        lost.push(format!("{} an error (like #DIV/0!) and {} read as empty", if errors == 1 { "1 cell holds".to_string() } else { format!("{errors} cells hold") }, if errors == 1 { "was" } else { "were" }));
    }
    if dates > 0 {
        lost.push(format!("{} a date or a time, read as the day number the workbook keeps", if dates == 1 { "1 cell holds".to_string() } else { format!("{dates} cells hold") }));
    }
    if let Ok(merged) = book.merge_cells_by_sheet_name(&name) {
        if !merged.is_empty() {
            lost.push(format!("{}: what is in each is in its first cell only", plural(merged.len(), "run of cells is merged into one", "runs of cells are merged into one")));
        }
    }
    lost.push("how its cells are shown (dollar signs, decimals, colour, bold) was not read: a cell is the figure in it".to_string());
    Ok(Sheet { sort: Sort::Xlsx, name, rows: tidy(rows), lost })
}

/// Rows written as CSV: a cell with a comma, a quote or a line break in it is quoted,
/// lines end with a line feed, and the text is UTF-8. Reading it back gives the rows.
pub fn csv(rows: &[Vec<String>]) -> String {
    let mut writer = csv::WriterBuilder::new().flexible(true).terminator(csv::Terminator::Any(b'\n')).from_writer(Vec::new());
    for row in rows {
        // A row of one empty cell would be written as an empty line and read back as
        // no row at all; `tidy` drops such rows on the way in, so none is written.
        let _ = writer.write_record(row);
    }
    String::from_utf8(writer.into_inner().unwrap_or_default()).unwrap_or_default()
}
