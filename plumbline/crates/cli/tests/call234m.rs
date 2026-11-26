//! Call 234 at the command line (contract 234M): `plumbline export field-sheet` and `plumbline import field-sheet`. The job is
//! the made-up field job of `demos/field.sh` (`demos/field-job.sh`); the figures are that demonstration's own.

use std::collections::BTreeMap;
use std::process::Command;

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_plumbline")
}

fn run(dir: &std::path::Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(exe()).current_dir(dir).args(args).output().unwrap();
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn ok(dir: &std::path::Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(good, "{args:?}: {said}");
    said
}

fn refused(dir: &std::path::Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(!good, "{args:?} should have been refused: {said}");
    said
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-234-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The field job, made by the demonstration's own script; what it names, by name.
fn field_job(dir: &std::path::Path) -> BTreeMap<String, String> {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../demos/field-job.sh");
    let bin = std::path::Path::new(exe()).parent().unwrap().to_string_lossy().to_string();
    let out = Command::new("bash").arg(script).arg(dir.join("job.json")).env("PLUMBLINE_BIN", bin).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).split_whitespace().filter_map(|w| w.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

const FOUND: usize = 6;
const BY: usize = 7;
const ON: usize = 8;
const REFERENCE: usize = 9;

fn read_table(path: &std::path::Path) -> Vec<Vec<String>> {
    plumbline_sheets::read(&std::fs::read(path).unwrap(), None).unwrap().rows
}

/// Fill a row in the table of a sheet read back (rows are cut after their last cell, so they are made whole first).
fn fill(table: &mut [Vec<String>], reference: &str, found: &str) {
    let row = table.iter_mut().find(|r| r.get(REFERENCE).is_some_and(|x| x == reference)).unwrap_or_else(|| panic!("no row {reference}"));
    row[FOUND] = found.to_string();
    row[BY] = "JM".to_string();
    row[ON] = "2026-11-26".to_string();
}

fn whole(table: &mut [Vec<String>]) {
    for row in table.iter_mut() {
        while row.len() < 10 {
            row.push(String::new());
        }
    }
}

fn write_csv(path: &std::path::Path, table: &[Vec<String>]) {
    std::fs::write(path, plumbline_sheets::csv(table)).unwrap();
}

fn write_xlsx(path: &std::path::Path, table: &[Vec<String>]) {
    use plumbline_sheets::Cell;
    let rows = table.iter().map(|r| r.iter().map(|c| if c.is_empty() { Cell::Empty } else { Cell::Text(c.clone()) }).collect()).collect();
    std::fs::write(path, plumbline_sheets::xlsx(&[plumbline_sheets::Tab { name: "Field sheet".into(), rows, head: 0, widths: vec![] }]).unwrap()).unwrap();
}

fn fill_the_field(table: &mut [Vec<String>], n: &BTreeMap<String, String>) {
    fill(table, &format!("{} elevation", n["GROUND"]), "same");
    fill(table, &format!("{} level height", n["GROUND"]), "8'10\"1/2");
    fill(table, &format!("{} line on the plan", n["DUCT"]), "same");
    fill(table, &format!("{} type", n["DUCT"]), "same");
    fill(table, &format!("{} height above its level", n["DUCT"]), "bottom 7'10\"3/4");
}

#[test]
fn the_sheet_is_written_as_csv_and_as_a_workbook_and_says_what_is_not_a_row() {
    let dir = folder("write");
    let n = field_job(&dir);
    let said = ok(&dir, &["export", "field-sheet", "job.json", "sheet.csv"]);
    assert!(said.contains("a sheet to fill on site") && said.contains("to measure, the most dollars first"), "{said}");
    assert!(said.contains("2 of the 5 questions on the list to verify are not rows"), "{said}");
    let table = read_table(&dir.join("sheet.csv"));
    assert_eq!(table[0][FOUND], "As found");
    assert_eq!(table[0][REFERENCE], "Reference");
    assert!(table.iter().any(|r| r.get(REFERENCE).is_some_and(|x| x == &format!("{} level height", n["GROUND"]))));
    ok(&dir, &["export", "field-sheet", "job.json", "sheet.xlsx"]);
    let book = plumbline_sheets::read(&std::fs::read(dir.join("sheet.xlsx")).unwrap(), None).unwrap();
    assert_eq!(book.rows[0][REFERENCE], "Reference");
    // The same rows in both, bar how dollars are written.
    assert_eq!(book.rows.len(), table.len());
    for (a, b) in book.rows.iter().zip(&table) {
        assert_eq!(a.get(REFERENCE), b.get(REFERENCE));
    }
    // A sheet is not written over a job.
    let why = refused(&dir, &["export", "field-sheet", "job.json", "job.json"]);
    assert!(why.contains("is the job itself"), "{why}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_sheet_brought_straight_back_is_a_job_byte_for_byte_what_it_was() {
    let dir = folder("blank");
    field_job(&dir);
    ok(&dir, &["export", "field-sheet", "job.json", "sheet.csv"]);
    let before = std::fs::read(dir.join("job.json")).unwrap();
    let said = ok(&dir, &["import", "field-sheet", "job.json", "sheet.csv"]);
    assert!(said.contains("no value was measured"), "{said}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_field_measurements_filled_in_a_workbook_and_brought_back_come_to_what_typing_them_comes_to() {
    let dir = folder("fill");
    let n = field_job(&dir);
    std::fs::copy(dir.join("job.json"), dir.join("typed.json")).unwrap();
    ok(&dir, &["export", "field-sheet", "job.json", "sheet.csv"]);
    let mut table = read_table(&dir.join("sheet.csv"));
    whole(&mut table);
    fill_the_field(&mut table, &n);
    write_xlsx(&dir.join("filled.xlsx"), &table);
    let said = ok(&dir, &["import", "field-sheet", "job.json", "filled.xlsx"]);
    // The way typing does it.
    for (what, reading) in [("elevation", "same"), ("level height", "8'10\"1/2")] {
        ok(&dir, &["measure", "typed.json", &n["GROUND"], what, reading, "--source", "JM"]);
    }
    ok(&dir, &["measure", "typed.json", &n["DUCT"], "line on the plan", "same", "--source", "JM"]);
    ok(&dir, &["measure", "typed.json", &n["DUCT"], "type", "same", "--source", "JM"]);
    ok(&dir, &["measure", "typed.json", &n["DUCT"], "height above its level", "7'10\"3/4", "--to", "bottom", "--source", "JM"]);
    assert!(said.contains("the price was $615.86 and is $607.31"), "{said}");
    assert!(said.contains("clash cleared") && said.contains("clash confirmed, now a finding"), "{said}");
    assert!(said.contains("5 values were measured"), "{said}");
    assert!(said.contains("1 question is left to verify in the field, and nothing priced rides on it."), "{said}");
    for verb in ["estimate", "clashes", "vif"] {
        let a = ok(&dir, &[verb, "job.json"]);
        let b = ok(&dir, &[verb, "typed.json"]);
        assert_eq!(a, b, "{verb}");
    }
    let show = ok(&dir, &["show", "job.json"]);
    assert!(show.contains("level height     8' 10 1/2\""), "{show}");
    // Each value's source, as the job file keeps it, names the sheet, its row, who and when.
    let saved = std::fs::read_to_string(dir.join("job.json")).unwrap();
    assert!(saved.contains(r#"measured in the field by JM on 2026-11-26: field sheet \"filled.xlsx\", row "#), "{saved}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn one_row_that_cannot_be_read_brings_nothing_in_and_is_named() {
    let dir = folder("refuse");
    let n = field_job(&dir);
    ok(&dir, &["export", "field-sheet", "job.json", "sheet.csv"]);
    let before = std::fs::read(dir.join("job.json")).unwrap();
    let mut table = read_table(&dir.join("sheet.csv"));
    whole(&mut table);
    fill(&mut table, &format!("{} elevation", n["GROUND"]), "same");
    fill(&mut table, &format!("{} level height", n["GROUND"]), "about nine feet");
    write_csv(&dir.join("bad.csv"), &table);
    let why = refused(&dir, &["import", "field-sheet", "job.json", "bad.csv"]);
    assert!(why.contains("row ") && why.contains("level height") && why.contains("nothing was brought in"), "{why}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before);
    // A figure from a workbook with no unit is refused as a typed one is.
    fill(&mut table, &format!("{} level height", n["GROUND"]), "9");
    write_csv(&dir.join("bare.csv"), &table);
    let why = refused(&dir, &["import", "field-sheet", "job.json", "bare.csv"]);
    assert!(why.contains("has no unit"), "{why}");
    // Not a field sheet.
    std::fs::write(dir.join("other.csv"), "a,b\n1,2\n").unwrap();
    let why = refused(&dir, &["import", "field-sheet", "job.json", "other.csv"]);
    assert!(why.contains("not a field sheet"), "{why}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_value_changed_since_the_sheet_went_out_needs_anyway() {
    let dir = folder("stale");
    let n = field_job(&dir);
    ok(&dir, &["export", "field-sheet", "job.json", "sheet.csv"]);
    ok(&dir, &["measure", "job.json", &n["GROUND"], "level height", "8'11\"", "--source", "KT with the tape"]);
    let mut table = read_table(&dir.join("sheet.csv"));
    whole(&mut table);
    fill(&mut table, &format!("{} level height", n["GROUND"]), "8'10\"1/2");
    write_csv(&dir.join("late.csv"), &table);
    let kept = std::fs::read(dir.join("job.json")).unwrap();
    let why = refused(&dir, &["import", "field-sheet", "job.json", "late.csv"]);
    assert!(why.contains("the sheet had 9 ft, Assumed and the job has 8' 11\", Known (KT with the tape)") && why.contains("--anyway"), "{why}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), kept);
    let said = ok(&dir, &["import", "field-sheet", "job.json", "late.csv", "--anyway"]);
    assert!(said.contains("1 value was measured"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_made_up_whole_house_has_a_sheet_that_says_what_it_could_not_make_rows_of() {
    let dir = folder("house");
    ok(&dir, &["sixteen", "house.json"]);
    let said = ok(&dir, &["export", "field-sheet", "house.json", "sheet.csv"]);
    assert!(said.contains("85 questions"), "{said}");
    let table = read_table(&dir.join("sheet.csv"));
    // Every row that has a reference names a value; none is Known.
    for row in table.iter().skip(1).filter(|r| r.get(REFERENCE).is_some_and(|x| !x.is_empty())) {
        assert!(row[4] == "Assumed" || row[4] == "Unknown", "{row:?}");
    }
    let before = std::fs::read(dir.join("house.json")).unwrap();
    ok(&dir, &["import", "field-sheet", "house.json", "sheet.csv"]);
    assert_eq!(std::fs::read(dir.join("house.json")).unwrap(), before);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_workbook_that_turned_what_was_typed_into_a_number_is_refused_by_row() {
    use plumbline_sheets::Cell;
    let dir = folder("number");
    let n = field_job(&dir);
    ok(&dir, &["export", "field-sheet", "job.json", "sheet.xlsx"]);
    let mut table = read_table(&dir.join("sheet.xlsx"));
    whole(&mut table);
    fill(&mut table, &format!("{} level height", n["GROUND"]), "9");
    let rows: Vec<Vec<Cell>> = table
        .iter()
        .map(|r| r.iter().enumerate().map(|(c, t)| if t.is_empty() { Cell::Empty } else if c == FOUND && t == "9" { Cell::Number(9.0) } else { Cell::Text(t.clone()) }).collect())
        .collect();
    std::fs::write(dir.join("number.xlsx"), plumbline_sheets::xlsx(&[plumbline_sheets::Tab { name: "Field sheet".into(), rows, head: 0, widths: vec![] }]).unwrap()).unwrap();
    let before = std::fs::read(dir.join("job.json")).unwrap();
    let why = refused(&dir, &["import", "field-sheet", "job.json", "number.xlsx"]);
    assert!(why.contains("row 3") && why.contains("level height") && why.contains("has no unit"), "{why}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before);
    let _ = std::fs::remove_dir_all(&dir);
}
