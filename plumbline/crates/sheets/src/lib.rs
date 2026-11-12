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
//! CSV comes back out as it went in: `csv` of the rows `read` gave is the same table,
//! but for one thing: a cell that begins with `=`, `+`, `-` or `@` and is not a figure is
//! written with an apostrophe in front, so that a spreadsheet does not run it as a
//! formula, and the apostrophe comes off when it is read. A
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
    /// The number each of `rows` has in its file, counted from 1 as a spreadsheet counts
    /// them: a row left out because it is empty still has its number.
    pub numbers: Vec<usize>,
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
/// in it. Cells keep their place across: an empty cell between two full ones stays. Each
/// row keeps the number it has in its file.
fn tidy(rows: Vec<(usize, Vec<String>)>) -> (Vec<Vec<String>>, Vec<usize>) {
    let mut kept = Vec::new();
    let mut numbers = Vec::new();
    for (number, mut row) in rows {
        while row.last().is_some_and(|c| c.trim().is_empty()) {
            row.pop();
        }
        if !row.is_empty() {
            kept.push(row);
            numbers.push(number);
        }
    }
    (kept, numbers)
}

/// A cell a spreadsheet would run as a formula if it were opened there: one that begins
/// with `=`, `+`, `-` or `@` and is not a plain figure. Written with an apostrophe in
/// front, which a spreadsheet takes as "this is text" and does not show.
fn runs_as_formula(cell: &str) -> bool {
    cell.starts_with(['=', '+', '-', '@']) && cell.trim().parse::<f64>().is_err()
}

/// Read a table. `sheet` names the workbook's sheet to read; with none, its first.
pub fn read(bytes: &[u8], sheet: Option<&str>) -> Result<Sheet, String> {
    if bytes.iter().all(|b| b.is_ascii_whitespace()) {
        return Err("the file is empty".into());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        return Err("it is text saved as UTF-16 (\"Unicode text\"), which this does not read: save it as CSV, or as CSV UTF-8".into());
    }
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
        lost.push("its cells are parted by semicolons, not commas: read that way. A list written like this often has a comma for its decimal mark (9,05): a figure with a comma is read only where the comma can part thousands, so check any figure in it that has one".to_string());
        b';'
    } else if count(',') == 0 && count('\t') > 0 {
        lost.push("its cells are parted by tabs, not commas: read that way".to_string());
        b'\t'
    } else {
        b','
    };
    let mut reader = csv::ReaderBuilder::new().has_headers(false).flexible(true).delimiter(delimiter).from_reader(text.as_bytes());
    let mut rows: Vec<(usize, Vec<String>)> = Vec::new();
    let mut over = 0usize;
    for (i, record) in reader.records().enumerate() {
        let record = record.map_err(|_| "the CSV could not be read: a row of it is not laid out as CSV is (a quote that is never closed, most often)".to_string())?;
        if rows.len() >= MOST_ROWS {
            over += 1;
            continue;
        }
        // The line the row starts on: a row left out because it is empty, and a cell
        // with a line break in it, both keep the count right.
        // The reader's own place for a row is where it began looking, which is before
        // any empty lines above the row: those are stepped over here.
        let number = match record.position() {
            Some(p) => {
                let mut at = (p.byte() as usize).min(text.len());
                while text.as_bytes().get(at).is_some_and(|b| matches!(b, b'\n' | b'\r')) {
                    at += 1;
                }
                1 + text.as_bytes()[..at].iter().filter(|b| **b == b'\n').count()
            }
            None => i + 1,
        };
        // An apostrophe put in front of a cell so a spreadsheet does not run it (see
        // `csv`) comes off again.
        rows.push((number, record.iter().map(|cell| cell.strip_prefix('\'').filter(|rest| runs_as_formula(rest)).unwrap_or(cell).to_string()).collect()));
    }
    if over > 0 {
        lost.push(format!("{over} rows after the first {MOST_ROWS} were not read"));
    }
    let (rows, numbers) = tidy(rows);
    Ok(Sheet { sort: Sort::Csv, name: String::new(), rows, numbers, lost })
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
    let mut book: Xlsx<_> = Xlsx::new(Cursor::new(bytes)).map_err(|_| "the workbook could not be read: it is damaged, locked with a password, or not a workbook (an .xlsx file) at all".to_string())?;
    let names = book.sheet_names();
    // A sheet the workbook hides is not what the person sees when they open it: with no
    // sheet named, the first one on show is read.
    let hidden: Vec<String> = book.sheets_metadata().iter().filter(|m| m.visible != calamine::SheetVisible::Visible).map(|m| m.name.clone()).collect();
    let name = match sheet {
        Some(asked) => names.iter().find(|n| n.eq_ignore_ascii_case(asked)).cloned().ok_or_else(|| format!("the workbook has no sheet \"{asked}\"; it has: {}", names.join(", ")))?,
        None => names.iter().find(|n| !hidden.contains(n)).or(names.first()).cloned().ok_or("the workbook has no sheets")?,
    };
    let mut lost = Vec::new();
    if hidden.contains(&name) {
        lost.push(format!("the sheet \"{name}\" is hidden in the workbook: it was read because {}", if sheet.is_some() { "it was asked for" } else { "the workbook shows no sheet" }));
    }
    let others: Vec<&String> = names.iter().filter(|n| **n != name).collect();
    if !others.is_empty() {
        lost.push(format!(
            "the sheet \"{name}\" was read; {} not: {}",
            if others.len() == 1 { "1 other sheet was".to_string() } else { format!("{} other sheets were", others.len()) },
            others.iter().map(|n| if hidden.contains(n) { format!("\"{n}\" (hidden)") } else { format!("\"{n}\"") }).collect::<Vec<_>>().join(", ")
        ));
    }
    // Cells shown as a percentage hold the fraction (9.05% is 0.0905): they are read as
    // they are shown, so that a percentage is never taken for a price or a quantity.
    let percent: std::collections::HashSet<(u32, u32)> = book
        .worksheet_style(&name)
        .map(|styles| styles.cells().filter(|(_, _, style)| style.number_format.as_ref().is_some_and(|f| f.format_code.contains('%'))).map(|(r, c, _)| (r, c)).collect())
        .unwrap_or_default();
    let range = book.worksheet_range(&name).map_err(|_| format!("the sheet \"{name}\" could not be read: the workbook is damaged, or is not laid out as a workbook is"))?;
    let (mut errors, mut dates, mut percents) = (0usize, 0usize, 0usize);
    let mut rows: Vec<(usize, Vec<String>)> = Vec::new();
    let mut over = 0usize;
    // A sheet's used cells may start below its first row or right of its first column;
    // the rows keep their cells' places from the first used column, and each row the
    // number the workbook gives it.
    let (top, left) = range.start().unwrap_or((0, 0));
    for (i, row) in range.rows().enumerate() {
        if rows.len() >= MOST_ROWS {
            over += 1;
            continue;
        }
        let at = top + i as u32;
        rows.push((
            at as usize + 1,
            row.iter()
                .enumerate()
                .map(|(j, cell)| match cell {
                    Data::Empty => String::new(),
                    Data::String(s) => s.clone(),
                    Data::Int(_) | Data::Float(_) if percent.contains(&(at, left + j as u32)) => {
                        percents += 1;
                        let x = match cell {
                            Data::Int(i) => *i as f64,
                            Data::Float(f) => *f,
                            _ => 0.0,
                        };
                        format!("{}%", number(x * 100.0))
                    }
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => number(*f),
                    Data::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                    // A date is not a figure: it is read as words, with the day number
                    // the workbook keeps, so nothing takes it for a price.
                    Data::DateTime(d) => {
                        dates += 1;
                        format!("a date or a time (day {})", number(d.as_f64()))
                    }
                    Data::DateTimeIso(s) | Data::DurationIso(s) => {
                        dates += 1;
                        format!("a date or a time ({s})")
                    }
                    Data::Error(_) => {
                        errors += 1;
                        String::new()
                    }
                })
                .collect(),
        ));
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
        lost.push(format!("{} a date or a time, which is read as those words and not as a figure", if dates == 1 { "1 cell holds".to_string() } else { format!("{dates} cells hold") }));
    }
    if percents > 0 {
        lost.push(format!("{} shown as a percentage and {} read as shown (9.05%), not as the fraction the workbook keeps", if percents == 1 { "1 cell is".to_string() } else { format!("{percents} cells are") }, if percents == 1 { "is" } else { "are" }));
    }
    if let Ok(merged) = book.merge_cells_by_sheet_name(&name) {
        if !merged.is_empty() {
            lost.push(format!("{}: what is in each is in its first cell only", plural(merged.len(), "run of cells is merged into one", "runs of cells are merged into one")));
        }
    }
    lost.push("how its cells are shown (dollar signs, decimals, color, bold) was not read: a cell is the figure in it".to_string());
    let (rows, numbers) = tidy(rows);
    Ok(Sheet { sort: Sort::Xlsx, name, rows, numbers, lost })
}

/// Rows written as CSV: a cell with a comma, a quote or a line break in it is quoted,
/// lines end with a line feed, and the text is UTF-8. Reading it back gives the rows.
pub fn csv(rows: &[Vec<String>]) -> String {
    let mut writer = csv::WriterBuilder::new().flexible(true).terminator(csv::Terminator::Any(b'\n')).from_writer(Vec::new());
    for row in rows {
        // A row of one empty cell would be written as an empty line and read back as
        // no row at all; `tidy` drops such rows on the way in, so none is written.
        // A cell a spreadsheet would run as a formula goes out with an apostrophe in
        // front; `read` takes it off again.
        let _ = writer.write_record(row.iter().map(|cell| if runs_as_formula(cell) { format!("'{cell}") } else { cell.clone() }));
    }
    String::from_utf8(writer.into_inner().unwrap_or_default()).unwrap_or_default()
}

/// One cell of a workbook to be written.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    /// A quantity, a rate or a count, shown to as many places as it has, up to four.
    Number(f64),
    Dollars(f64),
    /// A formula, with what it comes to as it is written, so a reader that does not
    /// work formulas out still shows the figure. Shown as dollars.
    Formula { text: String, value: Option<f64> },
    /// A formula that, as written, comes to a word or to nothing ("none" for a cost type
    /// with no priced line yet; nothing for an amount with no price yet): it turns to a
    /// figure when what it rests on is filled in.
    FormulaText { text: String, shown: String },
}

/// One sheet of a workbook to be written. `head` is its row of headings, from 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub name: String,
    pub rows: Vec<Vec<Cell>>,
    pub head: usize,
    pub widths: Vec<f64>,
}

/// A workbook written as XLSX: each tab a sheet, its headings and its first row in
/// bold, dollars as dollars, and every formula live.
pub fn xlsx(tabs: &[Tab]) -> Result<Vec<u8>, String> {
    use rust_xlsxwriter::{Format, Formula, Workbook};
    let fault = |e: rust_xlsxwriter::XlsxError| format!("the workbook could not be written: {e}");
    let mut book = Workbook::new();
    let bold = Format::new().set_bold();
    let dollars = Format::new().set_num_format("$#,##0.00");
    let figure = Format::new().set_num_format("#,##0.####");
    for tab in tabs {
        let sheet = book.add_worksheet();
        // A formula that comes to nothing is written with nothing, not with 0 (J17C-5).
        sheet.set_formula_result_default("");
        sheet.set_name(&tab.name).map_err(fault)?;
        for (c, width) in tab.widths.iter().enumerate() {
            sheet.set_column_width(c as u16, *width).map_err(fault)?;
        }
        for (r, row) in tab.rows.iter().enumerate() {
            let strong = r == 0 || r == tab.head;
            for (c, cell) in row.iter().enumerate() {
                let (r, c) = (r as u32, c as u16);
                match cell {
                    Cell::Empty => {}
                    Cell::Text(s) if s.is_empty() => {}
                    Cell::Text(s) if strong => {
                        sheet.write_string_with_format(r, c, s, &bold).map_err(fault)?;
                    }
                    Cell::Text(s) => {
                        sheet.write_string(r, c, s).map_err(fault)?;
                    }
                    Cell::Number(x) => {
                        sheet.write_number_with_format(r, c, *x, &figure).map_err(fault)?;
                    }
                    Cell::Dollars(x) => {
                        sheet.write_number_with_format(r, c, *x, &dollars).map_err(fault)?;
                    }
                    Cell::Formula { text, value } => {
                        let formula = match value {
                            Some(v) => Formula::new(text).set_result(format!("{v}")),
                            None => Formula::new(text),
                        };
                        sheet.write_formula_with_format(r, c, formula, &dollars).map_err(fault)?;
                    }
                    Cell::FormulaText { text, shown } => {
                        sheet.write_formula_with_format(r, c, Formula::new(text).set_result(shown.clone()), &dollars).map_err(fault)?;
                    }
                }
            }
        }
    }
    book.save_to_buffer().map_err(fault)
}

/// The estimate, or a lens on it, as a workbook: each sheet the model laid out, its
/// figures written as the formulas they are. Returns the file and how many of its
/// cells are formulas. One way of writing it, for the command line and the window.
pub fn workbook(sheets: &[plumbline_world::export::OutSheet]) -> Result<(Vec<u8>, usize), String> {
    use plumbline_world::export::Out;
    let tabs: Vec<Tab> = sheets
        .iter()
        .map(|s| Tab {
            name: s.name.clone(),
            head: s.head,
            widths: s.widths.clone(),
            rows: s
                .rows
                .iter()
                .map(|r| {
                    r.iter()
                        .map(|c| match c {
                            Out::None => Cell::Empty,
                            Out::Text(t) => Cell::Text(t.clone()),
                            Out::Number(x) => Cell::Number(*x),
                            Out::Dollars(x) => Cell::Dollars(*x),
                            Out::Formula { text, value } => Cell::Formula { text: text.clone(), value: *value },
                            Out::FormulaText { text, shown } => Cell::FormulaText { text: text.clone(), shown: shown.clone() },
                        })
                        .collect()
                })
                .collect(),
        })
        .collect();
    let formulas = sheets.iter().flat_map(|s| s.rows.iter().flatten()).filter(|c| matches!(c, Out::Formula { .. } | Out::FormulaText { .. })).count();
    Ok((xlsx(&tabs)?, formulas))
}
