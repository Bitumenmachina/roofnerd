//! Call 223 at the command line (contract 223M): "my database", every table of the job written out as one workbook (or a folder of
//! tables) and brought back whole, all of it or none of it. The seven steps of STATUS.md ("The database in one hand"), driven through
//! the real program. Everything is made up.

use plumbline_sheets as sheets;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_plumbline")).current_dir(dir).args(args).output().unwrap();
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn ok(dir: &Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(good, "{args:?}: {said}");
    said
}

fn refused(dir: &Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(!good, "{args:?} should have been refused: {said}");
    said
}

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-223-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// call 225: the cabinet rules joined the database (call 223 as built, point 6)
const TABLES: [&str; 6] = ["shapes", "items", "cabinet-rules", "connections", "assemblies", "prices"];
/// The file a table has in a folder of the database: the tab's name (the cabinet rules' tab is "Cabinet codes").
// call 225: the cabinet rules joined the database (call 223 as built, point 6)
fn folder_file(t: &str) -> String {
    format!("{}.csv", if t == "cabinet-rules" { "cabinet codes" } else { t })
}

/// A job with the starter tables, a made-up price list of three suppliers, one shape and one item of its own.
fn made(dir: &Path, job: &str) {
    ok(dir, &["new", job, "Database (not a real job)"]);
    for t in ["items", "connections", "assemblies"] {
        ok(dir, &["import", t, job, "starter"]);
    }
    std::fs::write(
        dir.join("list.csv"),
        "Material,Bought by,Price,Supplier,How sure,Where from\nFraming stud,EA,3.10,Supplier A,Assumed,a made-up quote\nFraming stud,EA,3.25,Supplier B,Assumed,a made-up quote\nFraming stud,EA,3.05,Supplier C,Assumed,a made-up quote\nDrywall sheet,EA,14.00,Supplier A,Assumed,a made-up quote\nDrywall sheet,EA,13.50,Supplier B,Assumed,a made-up quote\nDrywall sheet,EA,14.25,Supplier C,Assumed,a made-up quote\n",
    )
    .unwrap();
    ok(dir, &["import", "prices", job, "list.csv", "--add"]);
    std::fs::write(dir.join("shape.csv"), "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to\nwall-hung closet,porcelain,box,-width / 2,width / 2,0,0.25 * depth,0.4 * height,height\n").unwrap();
    std::fs::write(dir.join("item.csv"), "Item,Trade,Kind,Drawn as,Width,Depth,Height,How sure,Where from\nWall-hung closet,15 Mechanical,fixture,wall-hung closet,16 in,26 in,30 in,Assumed,a made-up size\n").unwrap();
    ok(dir, &["import", "shapes", job, "shape.csv"]);
    ok(dir, &["import", "items", job, "item.csv"]);
}

fn tabs_of(path: &Path) -> Vec<(String, Vec<Vec<String>>)> {
    sheets::read_tabs(&std::fs::read(path).unwrap()).unwrap().into_iter().map(|s| (s.name, s.rows)).collect()
}

/// Step 1: one workbook, a tab for each table, each tab row for row what that table's own writing-out gives.
#[test]
fn step_1_one_workbook_each_tab_is_its_table() {
    let dir = folder("one");
    made(&dir, "a.json");
    let said = ok(&dir, &["export", "database", "a.json", "my.xlsx"]);
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    // call 233: the starter chest grew
    assert!(said.contains("a tab for each table") && said.contains("Shapes 102, Items 58, Cabinet codes 8, Connections 5, Assemblies 202, Prices 8"), "{said}");
    let tabs = tabs_of(&dir.join("my.xlsx"));
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert_eq!(tabs.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    for (t, (_, rows)) in TABLES.iter().zip(&tabs) {
        ok(&dir, &["export", t, "a.json", &format!("{t}.csv")]);
        let single = sheets::read(&std::fs::read(dir.join(format!("{t}.csv"))).unwrap(), None).unwrap().rows;
        assert_eq!(rows, &single, "the {t} tab is the {t} table");
    }
    // The folder: each file is, byte for byte, the table written out alone.
    ok(&dir, &["export", "database", "a.json", "my folder"]);
    for t in TABLES {
        assert_eq!(std::fs::read(dir.join("my folder").join(folder_file(t))).unwrap(), std::fs::read(dir.join(format!("{t}.csv"))).unwrap(), "{t}");
    }
}

/// Steps 2 and 3: brought into a new empty job, the counts are the counts written; written out again, the same workbook cell for cell and the same folder byte for byte.
#[test]
fn steps_2_and_3_in_to_an_empty_job_and_out_again() {
    let dir = folder("two");
    made(&dir, "a.json");
    ok(&dir, &["export", "database", "a.json", "my.xlsx"]);
    ok(&dir, &["export", "database", "a.json", "my"]);
    ok(&dir, &["new", "b.json", "Second (not a real job)"]);
    let said = ok(&dir, &["import", "database", "b.json", "my.xlsx"]);
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    // call 233: the starter chest grew
    for line in ["(a workbook, 6 tabs): brought in whole", "the tab Shapes: 102 rows under its headings", "the tab Items: 58 rows under its headings", "the tab Connections: 5 rows under its headings", "the tab Assemblies: 202 rows under its headings", "the tab Prices: 8 rows under its headings"] {
        assert!(said.contains(line), "{line}\n{said}");
    }
    // call 233: the starter chest grew
    assert!(said.contains("  set: 58\n"), "58 items added:\n{said}");
    assert!(!said.contains("not used: 1"), "{said}");
    ok(&dir, &["export", "database", "b.json", "my2.xlsx"]);
    assert_eq!(tabs_of(&dir.join("my.xlsx")), tabs_of(&dir.join("my2.xlsx")), "cell for cell");
    ok(&dir, &["export", "database", "b.json", "my2"]);
    for t in TABLES {
        assert_eq!(std::fs::read(dir.join("my").join(folder_file(t))).unwrap(), std::fs::read(dir.join("my2").join(folder_file(t))).unwrap(), "{t}: byte for byte");
    }
    // The folder comes in too, and is the same again.
    ok(&dir, &["new", "c.json", "Third (not a real job)"]);
    let said = ok(&dir, &["import", "database", "c.json", "my"]);
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert!(said.contains("(a folder, 6 files): brought in whole"), "{said}");
    ok(&dir, &["export", "database", "c.json", "my3"]);
    for t in TABLES {
        assert_eq!(std::fs::read(dir.join("my").join(folder_file(t))).unwrap(), std::fs::read(dir.join("my3").join(folder_file(t))).unwrap(), "{t}: byte for byte");
    }
}

/// Step 4: into a job that has it all, every row is the same as the job has, nothing is set, and the job is byte for byte what it was.
#[test]
fn step_4_a_job_that_has_it_all_is_what_it_was() {
    let dir = folder("four");
    made(&dir, "a.json");
    ok(&dir, &["export", "database", "a.json", "my.xlsx"]);
    let before = std::fs::read(dir.join("a.json")).unwrap();
    let said = ok(&dir, &["import", "database", "a.json", "my.xlsx"]);
    assert!(said.contains("in all: 0 set; no row refused"), "{said}");
    // call 233: the starter chest grew
    assert!(said.contains("the same as the job already has: 58"), "{said}");
    assert_eq!(std::fs::read(dir.join("a.json")).unwrap(), before);
}

/// Step 5: a tab added by hand and a column added to Items are named as not read; everything else comes in.
#[test]
fn step_5_what_it_cannot_read_is_said() {
    let dir = folder("five");
    made(&dir, "a.json");
    // A workbook as a person might have edited it: Items with a column of their own, and a tab of notes.
    let mut tabs = tabs_of(&{
        ok(&dir, &["export", "database", "a.json", "my.xlsx"]);
        dir.join("my.xlsx")
    });
    let items = tabs.iter_mut().find(|(n, _)| n == "Items").unwrap();
    items.1[0].push("My code".into());
    for (i, r) in items.1.iter_mut().enumerate().skip(1) {
        r.resize(26, String::new());
        r.push(format!("Z-{i}"));
    }
    tabs.push(("Notes".into(), vec![vec!["Remember".into()], vec!["call the supplier".into()]]));
    let tabs: Vec<sheets::Tab> = tabs
        .into_iter()
        .map(|(name, rows)| sheets::Tab { name, rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { sheets::Cell::Empty } else { sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 0, widths: vec![] })
        .collect();
    std::fs::write(dir.join("edited.xlsx"), sheets::xlsx(&tabs).unwrap()).unwrap();
    ok(&dir, &["new", "b.json", "Second (not a real job)"]);
    let said = ok(&dir, &["import", "database", "b.json", "edited.xlsx"]);
    assert!(said.contains("not read: the tab \"Notes\""), "{said}");
    assert!(said.contains("not read: the column \"My code\""), "{said}");
    // call 233: the starter chest grew
    assert!(said.contains("  set: 58\n"), "{said}");
    ok(&dir, &["export", "database", "b.json", "my2.xlsx"]);
    assert_eq!(tabs_of(&dir.join("my.xlsx")), tabs_of(&dir.join("my2.xlsx")));
}

/// Step 6: one bad row (an item with a width that is not a length): nothing at all comes in, and the reply names the tab, the row and why.
#[test]
fn step_6_one_bad_row_and_nothing_comes_in() {
    let dir = folder("six");
    made(&dir, "a.json");
    ok(&dir, &["export", "database", "a.json", "my"]);
    let items = std::fs::read_to_string(dir.join("my/items.csv")).unwrap();
    let bad = items.replacen(",16 in,", ",sixteen-ish,", 1);
    assert_ne!(bad, items);
    std::fs::write(dir.join("my/items.csv"), bad).unwrap();
    ok(&dir, &["new", "b.json", "Second (not a real job)"]);
    let before = std::fs::read(dir.join("b.json")).unwrap();
    let said = refused(&dir, &["import", "database", "b.json", "my"]);
    assert!(said.contains("nothing was brought in") && said.contains("tab Items, row ") && said.contains("sixteen-ish"), "{said}");
    assert_eq!(std::fs::read(dir.join("b.json")).unwrap(), before, "the job is not touched: no shape, no price came in");
    // The same through a workbook.
    let mut tabs = tabs_of(&{
        ok(&dir, &["export", "database", "a.json", "good.xlsx"]);
        dir.join("good.xlsx")
    });
    let prices = tabs.iter_mut().find(|(n, _)| n == "Prices").unwrap();
    prices.1[2][2] = "cheap".into();
    let tabs: Vec<sheets::Tab> = tabs.into_iter().map(|(name, rows)| sheets::Tab { name, rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { sheets::Cell::Empty } else { sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 0, widths: vec![] }).collect();
    std::fs::write(dir.join("bad.xlsx"), sheets::xlsx(&tabs).unwrap()).unwrap();
    let said = refused(&dir, &["import", "database", "b.json", "bad.xlsx"]);
    assert!(said.contains("tab Prices, row 3") && said.contains("cheap"), "{said}");
    assert_eq!(std::fs::read(dir.join("b.json")).unwrap(), before);
}

/// Step 7: the starter database in one bringing-in gives the same job as the starter tables one by one, byte for byte.
#[test]
fn step_7_the_starter_database_is_the_starter_tables() {
    let dir = folder("seven");
    ok(&dir, &["new", "one.json", "Starter (not a real job)"]);
    for t in ["items", "connections", "assemblies"] {
        ok(&dir, &["import", t, "one.json", "starter"]);
    }
    ok(&dir, &["new", "whole.json", "Starter (not a real job)"]);
    let said = ok(&dir, &["import", "database", "whole.json", "starter"]);
    assert!(said.contains("the starter database: brought in whole") && said.contains("the tab Assemblies: 202 rows"), "{said}");
    assert!(!said.contains("no tab for Prices"), "there is no starter price list, and that is not a lack: {said}");
    let (one, whole) = (std::fs::read_to_string(dir.join("one.json")).unwrap(), std::fs::read_to_string(dir.join("whole.json")).unwrap());
    assert_eq!(one, whole, "byte for byte");
}

/// A single CSV file, a workbook that is not a database, and a flag the verb does not take are each refused in a builder's words.
#[test]
fn what_is_not_a_database_is_refused() {
    let dir = folder("not");
    made(&dir, "a.json");
    ok(&dir, &["export", "items", "a.json", "items.csv"]);
    let said = refused(&dir, &["import", "database", "a.json", "items.csv"]);
    assert!(said.contains("one table"), "{said}");
    let book = sheets::xlsx(&[sheets::Tab { name: "Budget".into(), rows: vec![vec![sheets::Cell::Text("x".into())]], head: 0, widths: vec![] }]).unwrap();
    std::fs::write(dir.join("budget.xlsx"), book).unwrap();
    let said = refused(&dir, &["import", "database", "a.json", "budget.xlsx"]);
    assert!(said.contains("\"Budget\"") && said.contains("Items"), "{said}");
    let said = refused(&dir, &["export", "database", "a.json", "a.json"]);
    assert!(said.contains("job"), "{said}");
    let said = refused(&dir, &["export", "database", "a.json", "items.csv"]);
    assert!(said.contains("is a file"), "{said}");
    let said = refused(&dir, &["import", "database", "a.json", "starter", "--sheet", "Items"]);
    assert!(said.contains("does not take --sheet"), "{said}");
}
