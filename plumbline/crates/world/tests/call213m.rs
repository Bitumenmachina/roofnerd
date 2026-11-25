//! Call 213 (contract 213M), the model's side: what the second command-line judge found that is the model's. Everything is made up.
//! The command-line side of each point is in `crates/cli/tests/call213m.rs`.

use plumbline_world::building::forms::Made;
use plumbline_world::building::shape::{Piece, Shape};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::formula::{self, Operand};
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

// ------------------------------------------------------------------ point 1 (F16)

/// An item cannot be renamed to the name of anything else in its place: a material, say. Refused; the job is byte for byte what it was and
/// still loads. (A job writes each element's children as name and path: two children of one name lose one, and the file cannot be read.)
#[test]
fn point_1_an_item_cannot_be_renamed_to_the_name_of_a_material() {
    let mut job = Job::new("Names (not a real job)", "test", "test");
    let oven = item_of(&mut job, "Wall oven, 27 in", 27.0);
    job.add_material("Plywood", "sheet").unwrap();
    let before = file::save(&mut job);
    let why = ed::rename_item(&mut job, &oven, "Plywood").unwrap_err();
    assert!(why.contains("there is already something called \"Plywood\"") && why.contains("two things in one place cannot go by one name"), "{why}");
    assert_eq!(file::save(&mut job), before, "the job is what it was");
    assert!(file::load(&before).is_ok());
    // Every verb of the window and the line goes by this one function.
    let why = ed::rename_item_counted(&mut job, &oven, "Plywood").unwrap_err();
    assert!(why.contains("two things in one place"), "{why}");
    // A name nothing has is taken, and the job still loads.
    assert_eq!(ed::rename_item(&mut job, &oven, "Wall oven, single"), Ok(0));
    let text = file::save(&mut job);
    assert!(file::load(&text).is_ok());
}

/// Each kind of thing that can be renamed holds the rule: an item (above); the things set down from it, which keep their name where another
/// thing on their level has the one wanted, and the job still loads.
#[test]
fn point_1_a_thing_set_down_keeps_its_name_where_the_new_one_is_taken_on_its_level() {
    let mut job = Job::new("Names (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let oven = item_of(&mut job, "Wall oven, 27 in", 27.0);
    let placed = ed::place_item(&mut job, &level, &oven, [4.0, 4.0], "0 ft", KNOWN).unwrap();
    ed::place(&mut job, &level, Kind::Fixture, "Hood", [9.0, 9.0], "0 ft", Some([2.0, 2.0, 1.0]), None, KNOWN).unwrap();
    let done = ed::rename_item_counted(&mut job, &oven, "Hood").unwrap();
    assert_eq!((done.set_down, done.renamed, done.kept.len()), (1, 0, 1), "{done:?}");
    let e = job.find(&placed).unwrap();
    assert_eq!(job.world.get::<Node>(e).unwrap().name, "Wall oven, 27 in");
    let text = file::save(&mut job);
    assert!(file::load(&text).is_ok());
}

// ------------------------------------------------------------------ point 8 (F8, F20)

fn read(text: &str) -> Option<f64> {
    let e = formula::parse(text).ok()?;
    let w = formula::work(&e, &|n| Err(format!("no {n}"))).ok()?;
    assert_eq!(w.dim, 1, "{text} is a length");
    w.value
}

/// A number, a slash, a number and a unit is a fraction of that unit; a whole number before it is added; feet and inches side by side add;
/// marks for feet and inches are those units.
#[test]
fn point_8_a_length_is_written_as_a_length_is_written_anywhere() {
    let near = |text: &str, inches: f64| {
        let got = read(text).unwrap_or_else(|| panic!("{text} is not read"));
        assert!((got * 12.0 - inches).abs() < 1.0e-9, "{text}: {} in", got * 12.0);
    };
    near("1/2 in", 0.5);
    near("1 / 2 in", 0.5);
    near("5 1/2 in", 5.5);
    near("3 ft 6 in", 42.0);
    near("3 ft 6 1/2 in", 42.5);
    near("3'6\"", 42.0);
    near("3' 6 1/2\"", 42.5);
    near("3'", 36.0);
    near("6\"", 6.0);
    near("3/4 ft", 9.0);
    near("12 in + 1/2 in", 12.5);
    near("min(3 1/2 in, 4 in)", 3.5);
    near("-1/2 in", -0.5);
    // A minus is a minus.
    near("3 ft - 6 in", 30.0);
    near("3 ft - 1/2 in", 35.5);
}

#[test]
fn point_8_a_minus_is_always_a_minus_and_the_old_readings_stand() {
    // `1/2 in` was one over two inches and is not a length; a plain 1/2 is still a half; a division by a length still divides.
    let w = formula::work(&formula::parse("plan_length * 1 / 2 in").unwrap(), &|_| Ok(Operand { value: Some(10.0), dim: 1, status: Status::Known, shown: "plan_length".into() })).unwrap();
    assert_eq!(w.dim, 0, "a length times 1, over 2 in, as before: a plain number");
    assert!((w.value.unwrap() - 10.0 / (2.0 / 12.0)).abs() < 1.0e-9);
    assert!((read("1 / 2 * 4 in").unwrap() * 12.0 - 2.0).abs() < 1.0e-9);
}

/// A refusal of a mark it cannot hold says how to write a length, with no stray marks.
#[test]
fn point_8_a_refusal_says_how_to_write_a_length_and_shows_no_stray_marks() {
    for text in ["3#", "3 @ 4", "{3 in}", "3 in;"] {
        let why = formula::parse(text).unwrap_err();
        assert!(!why.contains("'\\''") && !why.contains("\\\""), "{text}: {why}");
        assert!(why.contains("3 ft 6 in") && why.contains("1/2 in"), "{text}: {why}");
    }
}

// ------------------------------------------------------------------ point 15 (F22), point 14 words

fn shape_of(blocks: Vec<Piece>) -> Shape {
    Shape { name: "My panel".into(), blocks }
}

fn block(across: [&str; 2], back: [&str; 2], up: [&str; 2]) -> Piece {
    Piece { made: Made::Timber, round: false, across: across.map(String::from), back: back.map(String::from), up: up.map(String::from), when: None }
}

#[test]
fn point_15_a_block_with_no_thickness_or_written_backwards_is_refused_where_a_shape_is_changed() {
    let good = block(["-width / 2", "width / 2"], ["0", "depth"], ["0", "height"]);
    let vanity = [3.0, 1.75, 2.875];
    assert!(shape_of(vec![good.clone()]).check_for(&[("Vanity, 36 in".into(), vanity)]).is_ok());
    assert!(shape_of(vec![good.clone()]).check_for(&[]).is_ok());
    // Flat: Up from and Up to are both height. At the item's own size, both ends are named, in inches.
    let mut flat = good.clone();
    flat.up = ["height".into(), "height".into()];
    let why = shape_of(vec![flat.clone()]).check_for(&[("Vanity, 36 in".into(), vanity)]).unwrap_err();
    assert!(why.contains("block 1") && why.contains("Vanity, 36 in") && why.contains("Up") && why.contains("no thickness") && why.contains("34 1/2 in to 34 1/2 in"), "{why}");
    // With no item drawn as it, at every size a shape is tried at.
    let why = shape_of(vec![flat]).check_for(&[]).unwrap_err();
    assert!(why.contains("block 1") && why.contains("Up") && why.contains("no thickness") && why.contains("every size"), "{why}");
    // Backwards.
    let mut back = good.clone();
    back.across = ["width / 2".into(), "-width / 2".into()];
    let why = shape_of(vec![back.clone()]).check_for(&[("Vanity, 36 in".into(), vanity)]).unwrap_err();
    assert!(why.contains("Across") && why.contains("backwards") && why.contains("18 in to -18 in"), "{why}");
    assert!(shape_of(vec![back]).check_for(&[]).unwrap_err().contains("backwards"));
    // A block that runs the wrong way only below the smallest size a shape is tried at is left (call 205's panel: up from 4 in to the height less 6 in).
    let mut panel = good.clone();
    panel.up = ["4 in".into(), "height - 6 in".into()];
    assert!(shape_of(vec![panel]).check_for(&[("Vanity, 36 in".into(), vanity)]).is_ok());
    // A block that is not there at a size is not held to it.
    let mut there = good;
    there.up = ["height".into(), "height".into()];
    there.when = Some("height > 100 ft".into());
    assert!(shape_of(vec![there]).check_for(&[]).is_ok());
}

// ------------------------------------------------------------------ point 10 (F17)

#[test]
fn point_10_a_price_is_dollars_and_1e3_is_not_a_price() {
    let mut job = Job::new("Prices (not a real job)", "test", "test");
    item_of(&mut job, "Wall oven, 27 in", 27.0);
    let how = Entered { status: Status::Assumed, source: "made-up list" };
    for text in ["1e3", "$1e3", "1E3", "$1.5e2"] {
        let why = plumbline_world::pricelist::set_price(&mut job, "Wall oven, 27 in", None, Some(text), how).unwrap_err();
        assert!(why.contains("is not a figure") && why.contains("$180/EA"), "{text}: {why}");
    }
    // A price to a fraction of a cent, and a thousands comma, are taken as before.
    assert!(plumbline_world::pricelist::set_price(&mut job, "Wall oven, 27 in", None, Some("$1,850.456"), how).is_ok());
    let why = plumbline_world::pricelist::set_price(&mut job, "Wall oven, 27 in", None, Some("1850 each"), how).unwrap_err();
    assert!(why.contains("is not a figure") && why.contains("$180/EA"), "{why}");
}
