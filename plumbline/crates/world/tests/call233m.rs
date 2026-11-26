//! Call 233 (contract 233M): the tool chest's appliances and small devices as shapes. A shape is rows of a table
//! (`library/shapes.csv`); these tests hold the fourteen new starter shapes to their items' sizes, the seven items that were
//! boxes to the sizes they had, the nine new items to what they are, and the two tables out and in again to the byte.
//! Every figure is made up or a typical size, and Assumed.

use plumbline_world::building::forms::Form;
use plumbline_world::building::shape::{self, csv_rows};
use plumbline_world::file::{load, save};
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::*;

/// The fourteen new shapes, in the order of the forms after the thirteen.
const NEW_SHAPES: [&str; 14] = [
    "range", "refrigerator", "dishwasher", "laundry machine", "air handler", "panelboard", "receptacle", "switch", "ceiling light", "recessed light", "smoke alarm", "thermostat", "register", "shower base",
];

/// The seven starter items that were boxes: name, the shape they are drawn as now, width, depth and height in inches (what they were).
const SEVEN: [(&str, &str, f64, f64, f64); 7] = [
    ("Range, 30 in", "range", 30.0, 26.0, 36.0),
    ("Refrigerator", "refrigerator", 36.0, 30.0, 70.0),
    ("Dishwasher", "dishwasher", 24.0, 24.0, 34.0),
    ("Washer", "laundry machine", 27.0, 30.0, 38.0),
    ("Dryer", "laundry machine", 27.0, 30.0, 38.0),
    ("Air handler", "air handler", 24.0, 22.0, 48.0),
    ("Panelboard, 200 A", "panelboard", 15.0, 4.0, 30.0),
];

/// The nine new items: name, trade, kind, the shape, width, depth and height in inches.
const NINE: [(&str, u8, &str, &str, f64, f64, f64); 9] = [
    ("Duplex receptacle", 16, "fixture", "receptacle", 2.75, 0.5, 4.5),
    ("Switch", 16, "fixture", "switch", 2.75, 0.5, 4.5),
    ("Ceiling light", 16, "fixture", "ceiling light", 6.0, 6.0, 2.0),
    ("Recessed light, 6 in", 16, "fixture", "recessed light", 7.5, 7.5, 6.0),
    ("Smoke alarm", 16, "fixture", "smoke alarm", 5.625, 5.625, 2.0),
    ("Thermostat", 15, "fixture", "thermostat", 3.3125, 1.5, 4.8125),
    ("Supply register, 4 x 10", 15, "fixture", "register", 11.625, 0.75, 5.625),
    ("Return grille, 20 x 20", 15, "fixture", "register", 21.75, 1.0, 21.75),
    ("Shower base, 60 x 32", 15, "fixture", "shower base", 60.0, 32.0, 2.625),
];

fn starter_job() -> Job {
    let mut job = Job::new("the chest (not a real job)", "test", "test");
    let rows = csv_rows(tables::STARTER_ITEMS);
    let brought = tables::items_in(&mut job, &rows, None).unwrap();
    assert!(brought.not_used.is_empty(), "{:?}", brought.not_used);
    job
}

fn items(job: &mut Job) -> Vec<(String, Item)> {
    let mut q = job.world.query::<(&Node, &Item)>();
    q.iter(&job.world).map(|(n, it)| (n.name.clone(), it.clone())).collect()
}

fn feet(inches: f64) -> f64 {
    inches / 12.0
}

/// The starter chest is 57 items and 27 shapes, the new ones after the thirteen in the order of the forms.
#[test]
fn the_chest_has_fifty_seven_items_and_twenty_seven_shapes() {
    let names: Vec<&str> = shape::starter().iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names.len(), 27, "{names:?}");
    assert_eq!(&names[13..], NEW_SHAPES);
    assert_eq!(Form::ALL.len(), 27);
    assert_eq!(Form::ALL.map(Form::name)[13..], NEW_SHAPES);
    let mut job = starter_job();
    assert_eq!(items(&mut job).len(), 57);
}

/// Each new shape, set at its item's own sizes and at half again as wide: every block inside the item's box, and together reaching each of
/// its six faces (a shape true to its sizes). The two sizes of a thing a person would set down.
#[test]
fn each_new_shape_is_true_to_its_items_sizes_at_two_sizes() {
    let mut job = starter_job();
    let all = items(&mut job);
    let mut tried = 0;
    for name in NEW_SHAPES {
        let drawn: Vec<&(String, Item)> = all.iter().filter(|(_, it)| it.form.as_deref() == Some(name)).collect();
        assert!(!drawn.is_empty(), "no starter item is drawn as {name}");
        for (item, it) in drawn {
            let (size, status, _) = it.size_ft().unwrap_or_else(|why| panic!("{item}: {why}"));
            assert_eq!(status, Status::Assumed, "{item}");
            for wide in [size, [size[0] * 1.5, size[1], size[2]]] {
                let blocks = shape::starter_blocks(name, wide).unwrap_or_else(|| panic!("no rows for {name}"));
                assert!(blocks.len() >= 2, "{name}: a shape is a few blocks, not a box: {}", blocks.len());
                let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
                for b in &blocks {
                    for i in 0..3 {
                        assert!(b.lo[i] < b.hi[i] + 1.0e-12, "{name}: a block inside out");
                        (lo[i], hi[i]) = (lo[i].min(b.lo[i]), hi[i].max(b.hi[i]));
                    }
                }
                let want = [[-wide[0] / 2.0, wide[0] / 2.0], [-wide[1] / 2.0, wide[1] / 2.0], [0.0, wide[2]]];
                for i in 0..3 {
                    assert!((lo[i] - want[i][0]).abs() < 1.0e-9 && (hi[i] - want[i][1]).abs() < 1.0e-9, "{name} on {item} at {wide:?}: does not fill the box on axis {i}: {lo:?} to {hi:?}");
                }
                // And the model itself holds the shape to the item's box at that size.
                shape::starter_named(name).unwrap().check_at(item, wide).unwrap_or_else(|why| panic!("{why}"));
                tried += 1;
            }
        }
    }
    assert_eq!(tried, 32, "16 items drawn as a new shape, at two sizes each");
}

/// The seven that were boxes: drawn as their shapes, their sizes, status and sources what they were.
#[test]
fn the_seven_that_were_boxes_are_drawn_as_their_shapes_at_the_sizes_they_had() {
    let mut job = starter_job();
    let all = items(&mut job);
    for (name, shape_name, w, d, h) in SEVEN {
        let (_, it) = all.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("{name} is not in the chest"));
        assert_eq!(it.form.as_deref(), Some(shape_name), "{name}");
        let (size, status, source) = it.size_ft().unwrap();
        assert_eq!(size, [feet(w), feet(d), feet(h)], "{name}");
        assert_eq!(status, Status::Assumed, "{name}");
        assert_eq!(source, "a typical size; the product is not chosen", "{name}");
        assert_eq!(it.maker.status, Status::Unknown, "{name}");
    }
}

/// The nine new: each there, in its trade, Assumed, saying where its size is from (and no maker named).
#[test]
fn the_nine_new_items_are_there_assumed_and_say_where_their_sizes_are_from() {
    let mut job = starter_job();
    let all = items(&mut job);
    for (name, trade, kind, shape_name, w, d, h) in NINE {
        let (_, it) = all.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("{name} is not in the chest"));
        assert_eq!(it.division.map(|d| d.number()), Some(trade), "{name}");
        assert_eq!(format!("{:?}", it.category).to_lowercase(), kind, "{name}");
        assert_eq!(it.form.as_deref(), Some(shape_name), "{name}");
        for (what, v, want) in [("width", &it.width, w), ("depth", &it.depth, d), ("height", &it.height, h)] {
            assert_eq!(v.status, Status::Assumed, "{name} {what}");
            assert!((v.feet().unwrap() - feet(want)).abs() < 1.0e-9, "{name} {what}: {:?}", v.feet());
            // call 235: a size no published page gave says it is an assumption; the six such are named, and no other may say so.
            let assumed_only = matches!((name, what), ("Duplex receptacle", "depth") | ("Switch", "depth") | ("Ceiling light", "height") | ("Recessed light, 6 in", "height") | ("Supply register, 4 x 10", "depth") | ("Return grille, 20 x 20", "depth"));
            let said = if assumed_only { "an assumption: no published size was found" } else { "a typical size; the product is not chosen" };
            assert_eq!(v.source, said, "{name} {what}: {}", v.source);
            assert!(!v.source.to_lowercase().contains("leviton") && !v.source.to_lowercase().contains("kohler"), "no maker is named");
        }
        assert_eq!(it.maker.status, Status::Unknown, "{name}");
    }
}

/// The tables: the items as the file is kept, out and in again to the byte; the shapes the items are drawn as, out and in again to the byte;
/// every starter shape and item name read back by the table's own readers.
#[test]
fn the_tables_go_out_and_come_back_the_same_to_the_byte() {
    let mut job = starter_job();
    let out = tables::items_out(&mut job);
    assert_eq!(out, csv_rows(tables::STARTER_ITEMS), "the items written out are the starter file, cell for cell");
    assert_eq!(out.len(), 1 + 57);
    let (shapes, unused) = tables::shapes_out_counted(&mut job);
    assert!(unused.is_empty());
    let drawn: std::collections::BTreeSet<&str> = shapes[1..].iter().map(|r| r[0].as_str()).collect();
    for name in NEW_SHAPES {
        assert!(drawn.contains(name), "{name} is drawn by an item and so is written out");
    }
    let mut second = Job::new("second (not a real job)", "test", "test");
    let brought = tables::shapes_in(&mut second, &shapes, None).unwrap();
    assert!(brought.not_used.is_empty(), "{:?}", brought.not_used);
    let brought = tables::items_in(&mut second, &out, None).unwrap();
    assert_eq!((brought.done.len(), brought.not_used.len()), (57, 0));
    assert_eq!(tables::items_out(&mut second), out);
    assert_eq!(tables::shapes_out(&mut second), tables::shapes_out(&mut job));
    // The job as saved opens and saves again the same.
    let text = save(&mut second);
    assert_eq!(save(&mut load(&text).unwrap().0), text);
    // A job carries no starter shape in its own list: they are in the program.
    assert!(second.shapes.is_empty());
}

/// Nothing already in a job moves: a job that holds the seven as boxes (a row whose Drawn as says box) keeps them boxes when the starter
/// library is brought in again; the starter rows do not overwrite it.
#[test]
fn a_job_that_has_the_seven_as_boxes_keeps_them_as_boxes() {
    let mut job = Job::new("a job saved before (not a real job)", "test", "test");
    let mut rows = csv_rows(tables::STARTER_ITEMS);
    for r in rows.iter_mut().skip(1) {
        if SEVEN.iter().any(|s| s.0 == r[0]) {
            r[3] = "box".into();
        }
    }
    tables::items_in(&mut job, &rows, None).unwrap();
    let before = save(&mut job);
    // Brought in again as the starter says: the rows differ in "Drawn as", so they are named as changes only because a person brought them;
    // a job that is only opened and saved is untouched.
    assert_eq!(save(&mut load(&before).unwrap().0), before);
    let all = items(&mut job);
    for (name, _, ..) in SEVEN {
        let (_, it) = all.iter().find(|(n, _)| n == name).unwrap();
        assert_eq!(it.form.as_deref(), Some("box"), "{name}");
    }
}
