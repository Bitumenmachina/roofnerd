//! Call 223 (contract 223M), the model's side: "my database", every table of the program written out as one and brought back whole,
//! all of it or none of it. Everything is made up.

use plumbline_world::database::{self, Given};
use plumbline_world::editing::Entered;
use plumbline_world::value::Status;
use plumbline_world::*;

fn rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .map(|line| {
            let (mut out, mut cell, mut quoted) = (Vec::new(), String::new(), false);
            for c in line.chars() {
                match (c, quoted) {
                    ('"', _) => quoted = !quoted,
                    (',', false) => out.push(std::mem::take(&mut cell)),
                    (c, _) => cell.push(c),
                }
            }
            out.push(cell);
            // A cell a spreadsheet would run as a formula is written with an apostrophe in front, which reading takes off.
            out.into_iter().map(|c| match c.strip_prefix('\'') { Some(rest) if rest.starts_with(['=', '+', '-', '@']) => rest.to_string(), _ => c }).collect()
        })
        .collect()
}

fn given(name: &str, rows: Vec<Vec<String>>) -> Given {
    let numbers = (1..=rows.len()).collect();
    Given { name: name.to_string(), rows, numbers, lost: Vec::new() }
}

fn given_all(job: &mut Job) -> Vec<Given> {
    database::out(job).into_iter().map(|(name, rows)| given(name, rows)).collect()
}

/// A job with the starter tables, a made-up price list of three suppliers, one shape and one item of its own.
fn full_job() -> Job {
    let mut job = Job::new("Database (not a real job)", "test", "test");
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    tables::connections_in(&mut job, &rows(tables::STARTER_CONNECTIONS), None).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let list = rows(
        "Material,Bought by,Price,Supplier,How sure,Where from\n\
         Framing stud,EA,3.10,Supplier A,Assumed,a made-up quote\n\
         Framing stud,EA,3.25,Supplier B,Assumed,a made-up quote\n\
         Framing stud,EA,3.05,Supplier C,Assumed,a made-up quote\n\
         Drywall sheet,EA,14.00,Supplier A,Assumed,a made-up quote\n\
         Drywall sheet,EA,13.50,Supplier B,Assumed,a made-up quote\n\
         Drywall sheet,EA,14.25,Supplier C,Assumed,a made-up quote",
    );
    tables::prices_in(&mut job, &list, None, None, true, None).unwrap();
    let shape = rows("Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to\nwall-hung closet,porcelain,box,-width / 2,width / 2,0,0.25 * depth,0.4 * height,height");
    tables::shapes_in(&mut job, &shape, None).unwrap();
    let item = rows("Item,Trade,Kind,Drawn as,Width,Depth,Height,How sure,Where from\nWall-hung closet,15 Mechanical,fixture,wall-hung closet,16 in,26 in,30 in,Assumed,a made-up size");
    tables::items_in(&mut job, &item, None).unwrap();
    job
}

fn tab<'a>(w: &'a database::Whole, name: &str) -> &'a database::TabIn {
    w.tabs.iter().find(|t| t.name == name).unwrap_or_else(|| panic!("no tab {name} in the reply"))
}

/// Point 1: each tab is exactly what its table's own writing-out gives, in the order the tables depend on one another.
#[test]
fn point_1_each_tab_is_its_tables_own_rows() {
    let mut job = full_job();
    let all = database::out(&mut job);
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert_eq!(all.iter().map(|(n, _)| *n).collect::<Vec<_>>(), ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    assert_eq!(all[0].1, tables::shapes_out(&mut job));
    assert_eq!(all[1].1, tables::items_out(&mut job));
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert_eq!(all[2].1, tables::cabinet_rules_out(&mut job));
    assert_eq!(all[3].1, tables::connections_out(&mut job));
    assert_eq!(all[4].1, tables::assemblies_out(&mut job));
    assert_eq!(all[5].1, tables::prices_out(&mut job));
    // The counts the program holds: 49 items, 50 rows of shapes (14 shapes), 5 rows of connections, 202 rows of assemblies (42 assemblies), 8 price rows.
    assert_eq!(all.iter().map(|(_, r)| r.len() - 1).collect::<Vec<_>>(), [50, 49, 8, 5, 202, 8], "rows under each tab's headings");
}

/// Points 2 and 3: brought into an empty job, each tab by its own reader, with counts; written out again it is the same.
#[test]
fn points_2_and_3_out_in_and_out_again_is_the_same() {
    let mut first = full_job();
    let sent = given_all(&mut first);
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let reply = database::bring_in(&mut second, &sent, false).unwrap();
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert_eq!(reply.tabs.iter().map(|t| t.name).collect::<Vec<_>>(), ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"], "in the order they depend on one another");
    // The 13 starter shapes are in every job: they are the same; the job's own is added.
    assert_eq!(tab(&reply, "Shapes").brought.same, 13);
    assert_eq!(tab(&reply, "Shapes").brought.done.len(), 1);
    assert_eq!(tab(&reply, "Items").brought.done.len(), 49);
    assert_eq!(tab(&reply, "Connections").brought.done.len(), 2);
    assert_eq!(tab(&reply, "Assemblies").brought.done.len(), 42);
    assert!(tab(&reply, "Prices").brought.done.len() >= 8);
    for t in &reply.tabs {
        assert!(t.brought.not_used.is_empty(), "{}: {:?}", t.name, t.brought.not_used);
    }
    assert_eq!(database::out(&mut second), database::out(&mut first), "the same tables, row for row");
}

/// Point 2 and the model's own rule: a job that has all of it takes it again and is what it was, byte for byte.
#[test]
fn point_2_a_job_that_has_it_all_is_not_changed() {
    let mut job = full_job();
    let sent = given_all(&mut job);
    let before = file::save(&mut job);
    let reply = database::bring_in(&mut job, &sent, false).unwrap();
    for t in &reply.tabs {
        assert!(t.brought.done.is_empty(), "{}: {:?}", t.name, t.brought.done);
        assert!(t.brought.not_used.is_empty());
        // A shape is several rows (a block each) and is counted once.
        let same = match t.name {
            "Shapes" => 14,
            "Connections" => 2, // a row for each connection, counted once for each item
            _ => t.brought.rows,
        };
        assert_eq!(t.brought.same, same, "{}: everything in it is the same as the job has", t.name);
    }
    assert_eq!(file::save(&mut job), before, "byte for byte");
}

/// The order of the tabs in the file does not matter: an item of the job's own shape comes in after its shape.
#[test]
fn point_2_tabs_are_brought_in_by_what_they_depend_on_not_by_where_they_lie() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    sent.reverse();
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let reply = database::bring_in(&mut second, &sent, false).unwrap();
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert_eq!(reply.tabs.iter().map(|t| t.name).collect::<Vec<_>>(), ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    assert_eq!(database::out(&mut second), database::out(&mut first));
}

/// Point 4: a tab it does not know and a column it does not know are named and left alone; the rest comes in.
#[test]
fn point_4_what_it_cannot_read_is_said_and_the_rest_comes_in() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    let items = sent.iter_mut().find(|g| g.name == "Items").unwrap();
    items.rows[0].push("My code".to_string());
    for (i, r) in items.rows.iter_mut().enumerate().skip(1) {
        r.resize(items_width(i), String::new());
        r.push(format!("Z-{i}"));
    }
    sent.push(given("Notes", vec![vec!["Remember".to_string()], vec!["call the supplier".to_string()]]));
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let reply = database::bring_in(&mut second, &sent, false).unwrap();
    assert_eq!(reply.not_read, ["Notes"]);
    assert_eq!(tab(&reply, "Items").brought.columns_not_used, ["My code"]);
    assert_eq!(tab(&reply, "Items").brought.done.len(), 49);
    assert_eq!(database::out(&mut second), database::out(&mut first));
}

fn items_width(_row: usize) -> usize {
    tables::ITEM_HEADINGS.len()
}

/// Point 2: one bad row anywhere and nothing at all comes in; the reply names the tab, the row and why, in a builder's words.
#[test]
fn point_2_one_bad_row_and_nothing_comes_in() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    // The 5th row of Items gets a width that is not a length.
    let items = sent.iter_mut().find(|g| g.name == "Items").unwrap();
    let width = items.rows[0].iter().position(|h| h == "Width").unwrap();
    items.rows[5][width] = "sixteen-ish".to_string();
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let before = file::save(&mut second);
    let said = database::bring_in(&mut second, &sent, false).unwrap_err();
    assert!(said.contains("Items"), "{said}");
    assert!(said.contains("row 6"), "{said}");
    assert!(said.contains("sixteen-ish"), "{said}");
    assert_eq!(file::save(&mut second), before, "the job is as it was: no shape, no item, no price came in");
    for bad in ["schema", "record", "field", "null", "panic"] {
        assert!(!said.to_lowercase().contains(bad), "{said}");
    }
}

/// The same, with the bad row in the last tab: what the earlier tabs brought is taken back.
#[test]
fn point_2_a_bad_row_in_the_last_tab_takes_back_the_first() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    let prices = sent.iter_mut().find(|g| g.name == "Prices").unwrap();
    let price = prices.rows[0].iter().position(|h| h == "Price").unwrap();
    prices.rows[3][price] = "cheap".to_string();
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let before = file::save(&mut second);
    let said = database::bring_in(&mut second, &sent, false).unwrap_err();
    assert!(said.contains("Prices") && said.contains("row 4") && said.contains("cheap"), "{said}");
    assert_eq!(file::save(&mut second), before);
}

/// Every refusal is told, not only the first.
#[test]
fn point_2_every_refusal_is_named() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    let items = sent.iter_mut().find(|g| g.name == "Items").unwrap();
    let width = items.rows[0].iter().position(|h| h == "Width").unwrap();
    items.rows[2][width] = "wide".to_string();
    let prices = sent.iter_mut().find(|g| g.name == "Prices").unwrap();
    let price = prices.rows[0].iter().position(|h| h == "Price").unwrap();
    prices.rows[3][price] = "cheap".to_string();
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let said = database::bring_in(&mut second, &sent, false).unwrap_err();
    assert!(said.contains("Items") && said.contains("row 3") && said.contains("Prices") && said.contains("row 4"), "{said}");
}

/// A tab that is not a table at all (no headings the table reads) refuses the whole.
#[test]
fn a_tab_with_no_headings_refuses_the_whole() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    let prices = sent.iter_mut().find(|g| g.name == "Prices").unwrap();
    prices.rows = vec![vec!["nothing".into(), "here".into()]];
    prices.numbers = vec![1];
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let before = file::save(&mut second);
    let said = database::bring_in(&mut second, &sent, false).unwrap_err();
    assert!(said.contains("Prices"), "{said}");
    assert_eq!(file::save(&mut second), before);
}

/// A workbook with none of the tabs is not a database: it is refused, and what it has is named.
#[test]
fn a_workbook_with_no_tab_it_knows_is_refused_and_says_what_it_has() {
    let mut job = Job::new("Second (not a real job)", "test", "test");
    let said = database::bring_in(&mut job, &[given("Notes", vec![vec!["x".into()]]), given("Budget", vec![vec!["y".into()]])], false).unwrap_err();
    assert!(said.contains("Notes") && said.contains("Budget") && said.contains("Items"), "{said}");
    let said = database::bring_in(&mut job, &[], false).unwrap_err();
    assert!(!said.is_empty());
}

/// A tab named twice cannot be told apart: refused.
#[test]
fn a_tab_named_twice_is_refused() {
    let mut first = full_job();
    let mut sent = given_all(&mut first);
    let again = sent[1].clone();
    sent.push(again);
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let said = database::bring_in(&mut second, &sent, false).unwrap_err();
    assert!(said.contains("Items") && said.contains("twice"), "{said}");
}

/// A tab is found by its name however it is cased; a database with only some of its tabs brings those and says which it did not have.
#[test]
fn a_part_of_a_database_comes_in_and_the_missing_tabs_are_said() {
    let mut first = Job::new("First (not a real job)", "test", "test");
    tables::items_in(&mut first, &rows(tables::STARTER_ITEMS), None).unwrap();
    let sent: Vec<Given> = given_all(&mut first).into_iter().filter(|g| g.name == "Items").map(|mut g| {
        g.name = "items".into();
        g
    }).collect();
    let mut second = Job::new("Second (not a real job)", "test", "test");
    let reply = database::bring_in(&mut second, &sent, false).unwrap();
    assert_eq!(reply.tabs.len(), 1);
    // call 225: the cabinet rules joined the database (call 223 as built, point 6)
    assert_eq!(reply.absent, ["Shapes", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    assert_eq!(tab(&reply, "Items").brought.done.len(), 48);
}

/// Point 5: the starter database in one bringing-in is the starter tables one by one, byte for byte.
#[test]
fn point_5_the_starter_database_is_the_starter_tables_one_by_one() {
    let mut one = Job::new("Starter (not a real job)", "test", "test");
    tables::items_in_missing(&mut one, &rows(tables::STARTER_ITEMS), None).unwrap();
    tables::connections_in_missing(&mut one, &rows(tables::STARTER_CONNECTIONS), None).unwrap();
    tables::assemblies_in(&mut one, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let mut whole = Job::new("Starter (not a real job)", "test", "test");
    let files = database::starter_files();
    assert_eq!(files.iter().map(|(n, _)| *n).collect::<Vec<_>>(), ["Shapes", "Items", "Connections", "Assemblies"]);
    let sent: Vec<Given> = files.iter().map(|(n, text)| given(n, rows(text))).collect();
    let reply = database::bring_in(&mut whole, &sent, true).unwrap();
    assert_eq!(tab(&reply, "Items").brought.done.len(), 48);
    assert_eq!(tab(&reply, "Assemblies").brought.done.len(), 42);
    assert_eq!(file::save(&mut whole), file::save(&mut one), "byte for byte");
}

/// Point 6: a value keeps how sure it is and where it came from through the whole; a price Assumed stays Assumed and never becomes Known on a file's say-so.
#[test]
fn point_6_how_sure_and_where_from_survive() {
    let mut first = full_job();
    let sent = given_all(&mut first);
    let mut second = Job::new("Second (not a real job)", "test", "test");
    database::bring_in(&mut second, &sent, false).unwrap();
    let prices = tables::prices_out(&mut second);
    assert!(prices.len() == 9 && prices.iter().skip(1).all(|r| r[4] == "Assumed" && r[5].contains("a made-up quote")), "{prices:?}");
    let _ = (Status::Known, Entered { status: Status::Known, source: "x" });
}
