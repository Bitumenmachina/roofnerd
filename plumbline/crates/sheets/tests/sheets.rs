use plumbline_sheets::{csv, read, sort_of, Sort};

fn file(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/../../demos/tables/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn csv_comes_back_as_it_went_whatever_is_in_its_cells() {
    let rows: Vec<Vec<String>> = vec![
        vec!["Material".into(), "Where from".into()],
        vec!["Copper, 16 oz".into(), "he said \"about nine\"".into()],
        vec!["Two\nlines".into(), String::new(), "a cell after an empty one".into()],
        vec!["  kept spaces  ".into(), "é ü ¼ in".into()],
    ];
    let text = csv(&rows);
    // A comma, a quote and a line break are each inside quotes; a quote is doubled.
    assert!(text.contains("\"Copper, 16 oz\",\"he said \"\"about nine\"\"\"\n"));
    let back = read(text.as_bytes(), None).unwrap();
    assert_eq!((back.sort, back.rows, back.lost.len()), (Sort::Csv, rows, 0));
    // A mark at the start that says the text is UTF-8 is not part of the first cell.
    let marked = [b"\xEF\xBB\xBF".as_slice(), text.as_bytes()].concat();
    assert_eq!(read(&marked, None).unwrap().rows[0][0], "Material");
    // Rows with nothing in them, and empty cells at a row's end, are not rows or cells.
    let ragged = read(b"a,b,,\n,,\n\nc\n", None).unwrap();
    assert_eq!(ragged.rows, vec![vec!["a".to_string(), "b".to_string()], vec!["c".to_string()]]);
}

#[test]
fn what_a_csv_file_is_read_as_when_it_is_not_plain_is_said() {
    // Semicolons, as a spreadsheet set for another country writes them.
    let semi = read(b"Item;UOM;Price\nCopper;lb;9,05\n", None).unwrap();
    assert_eq!(semi.rows[1], ["Copper", "lb", "9,05"]);
    assert!(semi.lost.iter().any(|l| l.contains("semicolons")), "{:?}", semi.lost);
    // Not UTF-8: read as Western European, and that is said.
    let latin = read(b"Item,Price\nCaf\xE9,2\n", None).unwrap();
    assert_eq!(latin.rows[1][0], "Café");
    assert!(latin.lost.iter().any(|l| l.contains("not UTF-8")), "{:?}", latin.lost);
    // A CSV file has no sheets; a file that is neither is refused.
    assert!(read(b"a,b\n", Some("Prices")).unwrap_err().contains("no sheets"));
    assert_eq!(sort_of(b"\x00\x01\x02 binary"), None);
    assert!(read(b"\x00\x01\x02 binary", None).unwrap_err().contains("not a CSV file or an XLSX workbook"));
    assert!(read(b"", None).is_err());
}

#[test]
fn a_workbook_gives_its_first_sheets_cells_and_says_what_it_did_not_give() {
    let bytes = file("tinners-yard.xlsx");
    assert_eq!(sort_of(&bytes), Some(Sort::Xlsx));
    let sheet = read(&bytes, None).unwrap();
    assert_eq!((sheet.sort, sheet.name.as_str(), sheet.rows.len()), (Sort::Xlsx, "Prices", 8));
    assert_eq!(sheet.rows[2], ["Item", "UOM", "Price", "Lead time"]);
    // A figure is the figure: 9.05, not 9.0500000000000007; 95, not 95.0. The formula's
    // cell is what it last worked out to, 9.4.
    assert_eq!(sheet.rows[3], ["Copper, 16 oz", "lb", "9.05", "2 weeks"]);
    assert_eq!(sheet.rows[4], ["copper,20 OZ", "LB", "9.4", "3 weeks"]);
    assert_eq!(sheet.rows[6][2], "95");
    assert_eq!(sheet.rows[7], ["Lead-coated copper", "lb", "call"]);
    let lost = sheet.lost.join(" | ");
    assert!(lost.contains("1 other sheet was not: \"Terms\""), "{lost}");
    assert!(lost.contains("1 cell holds a formula"), "{lost}");
    assert!(lost.contains("how its cells are shown"), "{lost}");
    // The other sheet by name, in any capitals; a sheet that is not there is refused
    // with the names of those that are.
    let terms = read(&bytes, Some("terms")).unwrap();
    assert_eq!((terms.name.as_str(), terms.rows[0][0].as_str()), ("Terms", "Terms"));
    assert_eq!(read(&bytes, Some("Nope")).unwrap_err(), "the workbook has no sheet \"Nope\"; it has: Prices, Terms");
    // A zip that is no workbook is refused, not a panic.
    assert!(read(b"PK\x03\x04 and then nothing a workbook has", None).is_err());
    // The same list as CSV and as a workbook: the same cells, but that a workbook keeps
    // 9.40 as the figure 9.4 and $9.05 as 9.05.
    let as_csv = read(&file("tinners-yard.csv"), None).unwrap();
    assert_eq!(as_csv.rows.len(), sheet.rows.len());
    assert_eq!((as_csv.rows[3][2].as_str(), as_csv.rows[4][2].as_str()), ("$9.05", "9.40"));
}
