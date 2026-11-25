//! Call 213 points 5, 6 and 7 (contract 213M), the model's side: who typed a size is said by who typed it, the same size typed again at
//! the command line is not a check, and a turn by nothing changes nothing. Everything is made up.

use plumbline_world::editing::{self as ed, Asked, Entered, HowSized};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

fn item_of(job: &mut Job, name: &str, width: f64) -> String {
    use plumbline_world::units::Len;
    let inches = |x: f64| Val { value: Some(Len::One(x)), unit: "in".to_string(), status: Status::Assumed, source: "a made-up catalog".to_string() };
    let unknown = || Val::<String>::unknown("", "not chosen");
    let item = Item { category: Kind::Equipment, maker: unknown(), model: unknown(), width: inches(width), depth: inches(24.0), height: inches(29.0), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    ed::item(job, name, item).unwrap()
}

// ------------------------------------------------------------------ point 5 (F4), point 6 (F5)

fn sized_item(job: &mut Job) -> String {
    item_of(job, "Wall oven, 27 in", 27.0)
}

/// The window's own way is unchanged: `item_size` is told the window asks. Typed, it says "in the window"; the same size again is the check.
#[test]
fn point_5_the_window_asks_and_is_told_so() {
    let mut job = Job::new("Sizes (not a real job)", "test", "test");
    let oven = sized_item(&mut job);
    let done = ed::item_size(&mut job, &oven, "width", "30 in", "").unwrap();
    assert!(done.said().contains("Assumed: typed in the window; not yet checked against its maker's sheet"), "{}", done.said());
    let again = ed::item_size(&mut job, &oven, "width", "30 in", "").unwrap();
    assert_eq!(again.how, HowSized::Checked);
    assert!(again.said().contains("is checked: Known, checked on site."), "{}", again.said());
    // The same call, told the window asks.
    let mut other = Job::new("Sizes (not a real job)", "test", "test");
    let oven2 = sized_item(&mut other);
    ed::item_size_asked(&mut other, &oven2, "width", "30 in", "", Asked::Window).unwrap();
    let twice = ed::item_size_asked(&mut other, &oven2, "width", "30 in", "", Asked::Window).unwrap();
    assert_eq!(twice.how, HowSized::Checked);
    assert_eq!(file::save(&mut job), file::save(&mut other), "the same job, to the byte");
}

/// At the command line: said as typed there; the same size again changes nothing and says so; Known only with how sure and where from.
#[test]
fn point_6_at_the_command_line_the_same_size_again_is_not_a_check() {
    let mut job = Job::new("Sizes (not a real job)", "test", "test");
    let oven = sized_item(&mut job);
    let done = ed::item_size_asked(&mut job, &oven, "width", "30 in", "", Asked::CommandLine).unwrap();
    assert_eq!(done.how, HowSized::Changed);
    let said = done.said();
    assert!(said.contains("Assumed: typed at the command line; not yet checked against its maker's sheet") && !said.contains("in the window"), "{said}");
    assert!(said.contains("--status Known --source"), "{said}");
    let before = file::save(&mut job);
    let again = ed::item_size_asked(&mut job, &oven, "width", "30 in", "", Asked::CommandLine).unwrap();
    assert_eq!(again.how, HowSized::Unchanged);
    assert!(again.said().contains("width is 30 in already: nothing changed"), "{}", again.said());
    assert_eq!(file::save(&mut job), before, "nothing changed");
    // The word "same" is a reading, and a reading is how sure and where from: with no source it is not taken.
    let again = ed::item_size_asked(&mut job, &oven, "width", "same", "", Asked::CommandLine).unwrap();
    assert_eq!(again.how, HowSized::Unchanged);
    assert_eq!(file::save(&mut job), before);
    // Known, the way every value is: measured, with where from.
    let known = ed::item_size_known(&mut job, &oven, "width", "30 in", "J. Smith, tape").unwrap();
    assert_eq!(known.how, HowSized::Checked);
    assert!(known.said().contains("width 30 in is Known: J. Smith, tape"), "{}", known.said());
    let e = job.find(&oven).unwrap();
    let w = &job.world.get::<Item>(e).unwrap().width;
    assert_eq!((w.status, w.source.as_str()), (Status::Known, "J. Smith, tape"));
    // A different size, measured, is the item changed to what was measured, Known.
    let known = ed::item_size_known(&mut job, &oven, "depth", "26 in", "J. Smith, tape").unwrap();
    assert!(known.said().contains("depth 26 in is Known: J. Smith, tape"), "{}", known.said());
    assert!(ed::item_size_known(&mut job, &oven, "depth", "26 in", "").is_err(), "where from is needed");
}

// ------------------------------------------------------------------ point 7 (F6)

#[test]
fn point_7_a_turn_by_nothing_says_it_changes_nothing() {
    let mut job = Job::new("Turns (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let oven = item_of(&mut job, "Wall oven, 27 in", 27.0);
    let placed = ed::place_item(&mut job, &level, &oven, [4.0, 4.0], "0 ft", KNOWN).unwrap();
    let how = || Entered { status: Status::Assumed, source: "turned by hand" };
    for by in [0.0, 360.0, -720.0] {
        let why = ed::turn(&mut job, &placed, by, how()).unwrap_err();
        assert!(why.contains("changes nothing"), "{by}: {why}");
        assert!(!why.contains("not a figure"), "{by}: {why}");
    }
    let why = ed::turn(&mut job, &placed, f64::NAN, how()).unwrap_err();
    assert!(why.contains("not a figure"), "{why}");
    assert_eq!(ed::turn(&mut job, &placed, 450.0, how()).unwrap(), 90.0);
    assert_eq!(ed::turned_said(90.0), 90.0);
    assert_eq!(ed::turned_said(-90.0), 270.0);
    assert_eq!(ed::turned_said(180.0), 180.0);
    assert_eq!(ed::turned_said(-180.0), 180.0);
    assert_eq!(ed::turned_said(0.0), 0.0);
}

