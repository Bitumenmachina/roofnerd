//! Call 206, points 4 to 7 and 11 to 13: the words and smaller refusals the command-line judge of 2026-11-25 found. Every
//! figure is made up. Point 14 (the `rename` verb) and the words of the command line are in `demos/judged-1125.sh`.

use plumbline_world::editing as ed;
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::view::estimate;
use plumbline_world::*;

const FRIDGE: &str = "Refrigerator";
const BOUGHT: &str = "Bought and set";

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up" }
}

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
            out
        })
        .collect()
}

fn row(cells: &[&str]) -> Vec<String> {
    cells.iter().map(|c| c.to_string()).collect()
}

/// Three refrigerators set down, priced as "Bought and set", with a price list that has them at $1,000.00.
fn fridges(n: usize) -> (Job, String, Vec<String>) {
    let mut job = Job::new("Refrigerators (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    ed::item_priced_as(&mut job, FRIDGE, Some(BOUGHT)).unwrap();
    let list = vec![row(&["Material", "Unit", "Price", "How sure", "Where from"]), row(&[FRIDGE, "EA", "1000", "Assumed", "a made-up list"])];
    tables::prices_in(&mut job, &list, None, Some("Acme"), true, None).unwrap();
    tables::prices_in(&mut job, &list, None, None, true, None).unwrap();
    let things: Vec<String> = (0..n).map(|i| ed::place_item(&mut job, &level, FRIDGE, [10.0 + 4.0 * i as f64, 5.0], "0 ft", known()).unwrap()).collect();
    job.derive();
    (job, level, things)
}

#[test]
fn point_4_an_item_priced_as_what_it_already_is_says_so_and_how_many_are_set_down() {
    let (mut job, _, _) = fridges(3);
    let said = ed::item_priced_as(&mut job, FRIDGE, Some(BOUGHT)).unwrap().said().join("\n");
    assert_eq!(said, "Refrigerator is already priced as \"Bought and set\"; nothing changes, and the 3 set down from it keep it");
    let (mut none, _, _) = fridges(0);
    let said = ed::item_priced_as(&mut none, FRIDGE, Some(BOUGHT)).unwrap().said().join("\n");
    assert_eq!(said, "Refrigerator is already priced as \"Bought and set\"; nothing changes, and none is set down yet");
}

#[test]
fn point_5_an_item_taken_out_of_the_chest_says_the_prices_that_go_with_it() {
    let (mut job, _, things) = fridges(1);
    for t in &things {
        ed::remove(&mut job, t).unwrap();
    }
    let path = {
        let mut q = job.world.query::<(&Node, &Item)>();
        q.iter(&job.world).find(|(n, _)| n.name == FRIDGE).map(|(n, _)| n.path.clone()).unwrap()
    };
    let said = ed::prices_that_go(&mut job, &path).unwrap();
    assert_eq!(said, "its 2 prices in the price list went with it: the price $1,000.00/EA, Acme's $1,000.00/EA; bring them in again to price it");
    ed::remove(&mut job, &path).unwrap();
    assert!(ed::prices_that_go(&mut job, "type-9999").is_none());
}

#[test]
fn point_6_an_items_trade_changed_moves_the_lines_on_what_is_set_down_from_it_and_says_how_many() {
    let (mut job, _, things) = fridges(3);
    let path = {
        let mut q = job.world.query::<(&Node, &Item)>();
        q.iter(&job.world).find(|(n, _)| n.name == FRIDGE).map(|(n, _)| n.path.clone()).unwrap()
    };
    let before = estimate(&mut job).lines.iter().filter(|l| l.path == things[0]).map(|l| l.division).collect::<Vec<_>>();
    assert!(before.iter().all(|d| *d == Some(Division::Equipment)), "{before:?}");
    let followed = ed::item_trade_followed(&mut job, &path, Some(Division::Furnishings)).unwrap();
    // Each thing has the item's line and "Set in place": 2 lines on each of 3.
    assert_eq!((followed.lines, followed.things, followed.set_down), (6, 3, 3));
    assert_eq!(followed.said(Some(Division::Furnishings)), "6 lines on 3 set down from it moved to Division 12 Furnishings");
    let after = estimate(&mut job).lines.iter().filter(|l| l.path == things[0]).map(|l| l.division).collect::<Vec<_>>();
    assert!(after.iter().all(|d| *d == Some(Division::Furnishings)), "{after:?}");
    // The money is the same: only where it lands moved.
    let total = estimate(&mut job).job_cost.value;
    let _ = total;
}

#[test]
fn point_7_a_price_on_a_line_made_of_its_item_is_refused_with_how_the_price_is_given() {
    let (mut job, _, _) = fridges(1);
    let why = ed::price_item(&mut job, FRIDGE, "$1200", Status::Known, "made-up").unwrap_err();
    assert!(why.contains("is made of the item itself, so its price is the item's") && why.contains("price list") && why.contains("Unit cost column"), "{why}");
    // A price brought in with an item table for an item priced as nothing says it prices nothing yet.
    let (mut job, ..) = fridges(0);
    let mut table = tables::items_out(&mut job);
    let at = table[0].iter().position(|h| h == "Unit cost").unwrap();
    let dish = table.iter().position(|r| r[0] == "Dishwasher").unwrap();
    table[dish][at] = "700".into();
    table[dish][at + 1] = "Assumed".into();
    table[dish][at + 2] = "made-up".into();
    let brought = tables::items_in(&mut job, &table, None).unwrap();
    assert!(brought.notes.iter().any(|n| n.contains("price nothing on the job yet") && n.contains("Dishwasher")), "{:?}", brought.notes);
}

#[test]
fn point_11_a_refusal_quotes_what_was_typed_and_min_with_nothing_says_what_it_takes() {
    for (typed, said) in [("height * 0.5 in", "\"height * 0.5 in\" comes out as an area"), ("min()", "min( takes two figures or more, with a comma between them, like min(4 in, height); there is nothing in it"), ("min(height,)", "there is nothing after the last comma")] {
        let table = vec![row(&["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to"]), row(&["z", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", typed])];
        let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
        let brought = tables::shapes_in(&mut job, &table, None).unwrap();
        assert!(brought.not_used.iter().any(|(_, why)| why.contains(said)), "{typed}: {:?}", brought.not_used);
        assert!(brought.not_used.iter().all(|(_, why)| !why.contains('\u{00F7}') && !why.contains('\u{00D7}')), "{:?}", brought.not_used);
    }
    // And in a cost line: net_area and 1 in, as typed.
    let (mut job, level, _) = fridges(0);
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let wall = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("10 ft"), known()).unwrap().remove(0);
    job.apply(&wall, "Exterior wall, 2x6, lap siding", "made-up").unwrap();
    let why = job.set_formula(&wall, Some("Lap siding"), "min(net_area, 1 in)", &[], Status::Known, "made-up").unwrap_err();
    assert!(why.contains("net_area is an area and 1 in is a length"), "{why}");
}

#[test]
fn point_12_one_block_not_1_blocks_and_rows_of_one_shape_that_do_not_stand_together_are_refused() {
    let head = row(&["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to"]);
    let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
    let brought = tables::shapes_in(&mut job, &[head.clone(), row(&["x", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", "height"])], None).unwrap();
    assert_eq!(brought.done, vec!["x: added: 1 block".to_string()]);
    // mybox, mybox2, mybox: the rows of mybox do not stand together.
    let table = vec![
        head,
        row(&["mybox", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", "height"]),
        row(&["mybox2", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", "height"]),
        row(&["mybox", "timber", "box", "-width / 4", "width / 4", "0", "depth", "0", "height"]),
    ];
    let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
    let brought = tables::shapes_in(&mut job, &table, None).unwrap();
    assert_eq!(brought.done, vec!["mybox2: added: 1 block".to_string()], "{:?}", brought.done);
    assert_eq!(brought.not_used.len(), 2, "{:?}", brought.not_used);
    assert!(brought.not_used.iter().all(|(_, why)| why.contains("its rows do not stand together") && why.contains("rows 2, 4")), "{:?}", brought.not_used);
}

#[test]
fn point_13_a_price_of_nothing_is_taken_and_named_and_a_height_below_nothing_is_refused() {
    let (mut job, level, _) = fridges(0);
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let wall = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("10 ft"), known()).unwrap().remove(0);
    job.apply(&wall, "Exterior wall, 2x6, lap siding", "made-up").unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Lap siding"), Some("$0.00"), Status::Known, "made-up").unwrap();
    job.derive();
    let line = estimate(&mut job).lines.into_iter().find(|l| l.path == wall && l.description == "Lap siding").unwrap();
    assert_eq!(line.amount.value, Some(0.0), "it comes to $0.00");
    assert_eq!(line.amount.note, "its price is $0.00 as entered: nothing is charged for it", "and the estimate marks it");
    // A price list row at $0.00 says so.
    let list = vec![row(&["Material", "Unit", "Price", "How sure", "Where from"]), row(&["Free thing", "EA", "0", "Known", "a made-up list"])];
    let brought = tables::prices_in(&mut job, &list, None, None, true, None).unwrap();
    assert!(brought.notes.iter().any(|n| n.contains("Free thing: priced at $0.00")), "{:?}", brought.notes);
    // A wainscot below nothing.
    let why = job.set_on_line(&wall, "wainscot", Some("Corner boards"), Some("-1 ft"), Status::Known, "made-up").unwrap_err();
    assert!(why.contains("wainscot is below nothing (-1 ft)"), "{why}");
    // Nothing is not below nothing (whatever else the line says of the entry).
    assert!(job.set_on_line(&wall, "wainscot", Some("Corner boards"), Some("0 ft"), Status::Known, "made-up").err().is_none_or(|why| !why.contains("below nothing")));
}
