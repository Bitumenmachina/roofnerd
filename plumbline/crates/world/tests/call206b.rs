//! Call 206 point 2: a shape stays inside its item's box at the item's own sizes, wherever a shape meets an item.
//! A shape of the job's own, `x`, has a block up to `height + max(0 in, width - 10 ft)`: inside the box at the three
//! sizes a table is tried at, and 24 in above it on an item 144 in wide (12 ft is 2 ft over 10). Made-up items.

use plumbline_world::editing as ed;
use plumbline_world::editing::shapes as sh;
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up" }
}

fn row(cells: &[&str]) -> Vec<String> {
    cells.iter().map(|c| c.to_string()).collect()
}

const X_UP: &str = "height + max(0 in, width - 10 ft)";

fn shape_table() -> Vec<Vec<String>> {
    vec![row(&["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to", "There when"]), row(&["x", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", X_UP, ""])]
}

fn item_of(width_in: f64, form: Option<&str>) -> Item {
    let len = |inches: f64| Val::<Len>::known(Len::One(inches / 12.0), "ft", "made-up");
    Item { category: Kind::Furnishing, maker: Val::unknown("", "not said"), model: Val::unknown("", "not said"), width: len(width_in), depth: len(24.0), height: len(96.0), weight: None, sheet: String::new(), division: None, form: form.map(str::to_string), priced_as: None }
}

fn job_with_x() -> Job {
    let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
    let brought = tables::shapes_in(&mut job, &shape_table(), None).unwrap();
    assert!(brought.not_used.is_empty(), "{:?}", brought.not_used);
    job
}

fn path_of(job: &mut Job, name: &str) -> String {
    let mut q = job.world.query::<(&Node, &Item)>();
    q.iter(&job.world).find(|(n, _)| n.name == name).map(|(n, _)| n.path.clone()).unwrap()
}

#[test]
fn an_item_drawn_as_the_shape_at_its_own_sizes_is_refused_naming_the_item_the_block_and_how_far() {
    let mut job = job_with_x();
    // 100 in wide: inside. 144 in wide: 24 in above the box.
    ed::item(&mut job, "Narrow thing", item_of(100.0, Some("x"))).unwrap();
    let why = ed::item(&mut job, "Wide thing", item_of(144.0, Some("x"))).unwrap_err();
    assert!(why.contains("on Wide thing (") && why.contains("block 1") && why.contains("24 in above the top"), "{why}");
    assert!(job.find("type-0002").is_none() || path_of_opt(&mut job, "Wide thing").is_none(), "nothing was added");
}

fn path_of_opt(job: &mut Job, name: &str) -> Option<String> {
    let mut q = job.world.query::<(&Node, &Item)>();
    q.iter(&job.world).find(|(n, _)| n.name == name).map(|(n, _)| n.path.clone())
}

#[test]
fn an_item_drawn_as_the_shape_and_then_sized_to_144_in_is_refused_and_left_as_it_was() {
    let mut job = job_with_x();
    ed::item(&mut job, "Narrow thing", item_of(100.0, Some("x"))).unwrap();
    let path = path_of(&mut job, "Narrow thing");
    let why = ed::item_size(&mut job, &path, "width", "144 in", "me").unwrap_err();
    assert!(why.contains("on Narrow thing (") && why.contains("24 in above the top"), "{why}");
    let e = job.find(&path).unwrap();
    assert_eq!(job.world.get::<Item>(e).unwrap().width.feet(), Some(100.0 / 12.0), "the size is as it was");
}

#[test]
fn an_item_drawn_as_a_box_and_then_drawn_as_the_shape_is_refused() {
    let mut job = job_with_x();
    ed::item(&mut job, "Wide thing", item_of(144.0, None)).unwrap();
    let path = path_of(&mut job, "Wide thing");
    let why = ed::item_drawn_as(&mut job, &path, Some("x")).unwrap_err();
    assert!(why.contains("on Wide thing (") && why.contains("24 in above the top"), "{why}");
}

#[test]
fn a_block_of_the_jobs_own_shape_set_so_that_an_item_stands_outside_is_refused_naming_the_item() {
    let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
    tables::shapes_in(&mut job, &[row(&["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to", "There when"]), row(&["y", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", "height", ""])], None).unwrap();
    ed::item(&mut job, "Wide thing", item_of(144.0, Some("y"))).unwrap();
    let why = sh::shape_block_set(&mut job, "y", 1, sh::BlockSet::Figure(5, X_UP.into())).unwrap_err();
    assert!(why.contains("on Wide thing (") && why.contains("24 in above the top"), "{why}");
    assert_eq!(job.shapes[0].blocks[0].up[1], "height", "the shape is as it was");
    // A block added is the whole box and stays inside.
    sh::shape_block_add(&mut job, "y", None).unwrap();
}

#[test]
fn a_shape_table_that_changes_a_shape_an_item_is_drawn_as_is_not_used_where_the_item_would_stand_outside() {
    let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
    let table = |up: &str| vec![row(&["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to", "There when"]), row(&["y", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", up, ""])];
    tables::shapes_in(&mut job, &table("height"), None).unwrap();
    ed::item(&mut job, "Wide thing", item_of(144.0, Some("y"))).unwrap();
    let brought = tables::shapes_in(&mut job, &table(X_UP), None).unwrap();
    assert_eq!(brought.not_used.len(), 1, "{:?}", brought.not_used);
    assert!(brought.not_used[0].1.contains("on Wide thing (") && brought.not_used[0].1.contains("24 in above the top"), "{:?}", brought.not_used);
    assert_eq!(job.shapes[0].blocks[0].up[1], "height");
}

#[test]
fn an_items_table_row_drawn_as_the_shape_is_not_used_where_its_sizes_put_a_block_outside() {
    let mut job = job_with_x();
    let items = vec![
        row(&["Item", "Trade", "Kind", "Drawn as", "Width", "Width: how sure", "Width: where from", "Depth", "Depth: how sure", "Depth: where from", "Height", "Height: how sure", "Height: where from"]),
        row(&["Wide thing", "", "furnishing", "x", "144 in", "Assumed", "made-up", "24 in", "Assumed", "made-up", "96 in", "Assumed", "made-up"]),
        row(&["Narrow thing", "", "furnishing", "x", "100 in", "Assumed", "made-up", "24 in", "Assumed", "made-up", "96 in", "Assumed", "made-up"]),
    ];
    let brought = tables::items_in(&mut job, &items, None).unwrap();
    assert_eq!(brought.not_used.len(), 1, "{:?}", brought.not_used);
    assert!(brought.not_used[0].1.contains("on Wide thing (") && brought.not_used[0].1.contains("24 in above the top"), "{:?}", brought.not_used);
    assert!(path_of_opt(&mut job, "Narrow thing").is_some() && path_of_opt(&mut job, "Wide thing").is_none());
}

#[test]
fn a_job_that_already_holds_one_says_so_and_still_opens() {
    let mut job = job_with_x();
    ed::item(&mut job, "Narrow thing", item_of(100.0, Some("x"))).unwrap();
    // As a job saved by a program before this one could hold it: the size put on the item with nothing checked.
    let path = path_of(&mut job, "Narrow thing");
    let e = job.find(&path).unwrap();
    let mut it = job.world.get::<Item>(e).unwrap().clone();
    it.width = Val::known(Len::One(12.0), "ft", "made-up");
    job.put(&path, it).unwrap();
    let said = sh::items_outside_their_boxes(&mut job);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].1.contains("on Narrow thing (") && said[0].1.contains("24 in above the top"), "{said:?}");
}

#[test]
fn the_starter_shapes_hold_at_the_odd_sizes_a_judge_tried() {
    // The thirteen are held at three sizes when they are made; at these, an item drawn as each is not refused.
    let mut fails = Vec::new();
    for shape in plumbline_world::building::shape::starter() {
        for (w, d, h) in [(6.0, 4.0, 5.0), (240.0, 120.0, 144.0), (200.0, 7.0, 60.0), (36.0, 21.0, 34.5), (1.0, 100.0, 1.0), (300.0, 2.0, 200.0)] {
            let len = |inches: f64| Val::<Len>::known(Len::One(inches / 12.0), "ft", "made-up");
            let it = Item { width: len(w), depth: len(d), height: len(h), ..item_of(10.0, Some(&shape.name)) };
            if let Err(why) = sh::item_fits(&[], &shape.name, &it) {
                fails.push(format!("{} at {w} x {d} x {h}: {why}", shape.name));
            }
        }
    }
    assert!(fails.is_empty(), "{fails:#?}");
}
