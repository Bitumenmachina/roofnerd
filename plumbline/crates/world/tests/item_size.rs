//! Contract 200P, part 2: an item's size as typed, one change in the model (`editing::item_size`), for the
//! window and the command line to say alike. What the window did with three steps (the measuring change, then
//! the item set back to Assumed, then the things set down put right) is one change here, and the saved job is
//! the same for the same typing. Made-up sizes.

use plumbline_world::editing::{self as ed, ItemSizeDone, HowSized};
use plumbline_world::units;
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const TYPED: &str = "typed in the window; not yet checked against its maker's sheet";

/// A level, a vanity 36 x 21 x 34 1/2 in as an item with two of it set down, and a water heater as an item
/// never set down (as the window's own test builds it).
fn job_with_items() -> (Job, String, String) {
    let mut job = Job::new("items (not a real job)", "test", "test");
    let how = ed::Entered { status: Status::Assumed, source: "made-up plan" };
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", how).expect("a level");
    let size = |typed: &str| {
        let (len, unit) = units::parse_length(typed).expect("a length");
        Val { value: Some(len), unit: unit.to_string(), status: Status::Assumed, source: "a typical size; the product is not chosen".to_string() }
    };
    let not_chosen = || Val::<String>::unknown("", "not chosen");
    let item = |kind, w: &str, d: &str, h: &str| Item { category: kind, maker: not_chosen(), model: not_chosen(), width: size(w), depth: size(d), height: size(h), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    let vanity = ed::item(&mut job, "Vanity, 36 in", item(Kind::Furnishing, "36 in", "21 in", "34 1/2 in")).expect("the vanity");
    let heater = ed::item(&mut job, "Water heater, 50 gal", item(Kind::Equipment, "22 in", "22 in", "60 in")).expect("the heater");
    for at in [[5.0, 5.0], [9.0, 5.0]] {
        ed::place_item(&mut job, &level, &vanity, at, "0 ft", ed::Entered { status: Status::Assumed, source: "set down by hand in the model" }).expect("set down");
    }
    job.derive();
    (job, vanity, heater)
}

fn size_of(job: &mut Job, path: &str, what: &str) -> (Option<String>, Status, String) {
    let e = job.find(path).expect("there");
    let item = job.world.get::<Item>(e).cloned().expect("an item");
    let v = match what {
        "width" => item.width,
        "depth" => item.depth,
        _ => item.height,
    };
    (v.value.map(|l| units::show_length(&l, &v.unit)), v.status, v.source)
}

fn followers(job: &mut Job) -> Vec<([f64; 3], Status)> {
    let mut q = job.world.query::<&Placed>();
    q.iter(&job.world).filter(|p| p.item.is_some()).map(|p| (p.size_ft().expect("a size"), p.size.status)).collect()
}

/// What the window does today, step by step, as `tools.rs` `item_size` does it: the measuring change (which makes
/// the value Known and moves what follows), the item's value set back to Assumed, and the things set down put
/// right to the status the item's sizes together are.
fn the_windows_steps(job: &mut Job, path: &str, what: &str, reading: &str) {
    let e = job.find(path).expect("there");
    let name = job.world.get::<Node>(e).map(|n| n.name.clone()).expect("a name");
    let done = ed::measure(job, path, what, ed::Reading::Length(reading), None, TYPED).expect("measured");
    let mut it = job.world.get::<Item>(e).cloned().expect("an item");
    match what {
        "width" => it.width.status = Status::Assumed,
        "depth" => it.depth.status = Status::Assumed,
        _ => it.height.status = Status::Assumed,
    }
    job.put(path, it.clone()).expect("put");
    let again: Vec<String> = done.also.iter().filter_map(|a| a.strip_suffix(" is one of it and is now that size")).map(str::to_string).collect();
    if let Ok((feet, status, from)) = it.size_ft() {
        for at in &again {
            let mut placed = job.find(at).and_then(|f| job.world.get::<Placed>(f)).cloned().expect("placed");
            if placed.size.status != status {
                placed.size = Val { value: Some(feet), unit: "ft".into(), status, source: format!("{name}: {from}") };
                job.put(at, placed).expect("put");
            }
        }
    }
    job.derive();
}

/// A size that differs from the one the item has is the item changed: Assumed, in the words New item uses; the
/// two set down follow, and are Assumed; and it says what it was, what it is, and how many followed.
#[test]
fn a_size_that_differs_is_the_item_changed_and_what_is_set_down_from_it_follows() {
    let (mut job, vanity, _) = job_with_items();
    let done: ItemSizeDone = ed::item_size(&mut job, &vanity, "width", "48 in", "").expect("changed");
    assert_eq!(done.how, HowSized::Changed);
    assert_eq!((done.old.as_deref(), done.new.as_str()), (Some("36 in"), "48 in"));
    assert_eq!(done.what, "width");
    assert_eq!(done.followed.len(), 2, "{done:?}");
    assert!(done.name_carries_old, "Vanity, 36 in");
    let (said, status, source) = size_of(&mut job, &vanity, "width");
    assert_eq!((said.as_deref(), status, source.as_str()), (Some("48 in"), Status::Assumed, TYPED));
    assert_eq!(ed::ITEM_SIZE_TYPED, TYPED);
    for (size, status) in followers(&mut job) {
        assert!((size[0] - 4.0).abs() < 1e-9, "{size:?}");
        assert_eq!(status, Status::Assumed);
    }
}

/// The same size again is the check: Known, "checked on site"; the one typed after a check is Assumed again.
#[test]
fn the_same_size_again_is_the_check_and_known() {
    let (mut job, vanity, _) = job_with_items();
    ed::item_size(&mut job, &vanity, "width", "48 in", "").expect("changed");
    let done = ed::item_size(&mut job, &vanity, "width", "48 in", "").expect("checked");
    assert_eq!(done.how, HowSized::Checked);
    assert_eq!((done.old.as_deref(), done.new.as_str()), (Some("48 in"), "48 in"));
    let (_, status, source) = size_of(&mut job, &vanity, "width");
    assert_eq!(status, Status::Known);
    assert!(source.contains("checked on site"), "{source}");
    // Said by whom: the checker's name goes on the source.
    let done = ed::item_size(&mut job, &vanity, "depth", "same", "J. Smith, tape").expect("checked");
    assert_eq!(done.how, HowSized::Checked);
    assert!(size_of(&mut job, &vanity, "depth").2.contains("checked on site: J. Smith, tape"));
    // A depth typed after is Assumed again and the width stays Known.
    ed::item_size(&mut job, &vanity, "depth", "22 in", "").expect("changed");
    assert_eq!(size_of(&mut job, &vanity, "depth").1, Status::Assumed);
    assert_eq!(size_of(&mut job, &vanity, "width").1, Status::Known);
}

/// An item with nothing set down: nothing follows, and it says so; an item with two set down says two.
#[test]
fn an_item_with_nothing_set_down_has_nothing_follow_and_one_with_two_has_two() {
    let (mut job, vanity, heater) = job_with_items();
    let none = ed::item_size(&mut job, &heater, "height", "62 in", "").expect("changed");
    assert_eq!((none.followed.len(), none.how), (0, HowSized::Changed));
    assert_eq!(none.old.as_deref(), Some("60 in"));
    // "Water heater, 50 gal" does not carry "60 in".
    assert!(!none.name_carries_old);
    let two = ed::item_size(&mut job, &vanity, "height", "36 in", "").expect("changed");
    assert_eq!(two.followed.len(), 2);
    assert!(!two.name_carries_old, "its name carries 36 in, but the height it had was 34 1/2 in");
}

/// A size that is not a length is refused and nothing changes.
#[test]
fn a_size_that_is_not_a_length_is_refused_and_nothing_changes() {
    let (mut job, vanity, _) = job_with_items();
    let before = file::save(&mut job);
    for typed in ["wide", "", "48", "4 hands"] {
        let why = ed::item_size(&mut job, &vanity, "width", typed, "").expect_err(typed);
        assert!(!why.is_empty());
    }
    assert!(ed::item_size(&mut job, &vanity, "weight", "40 in", "").is_err());
    assert!(ed::item_size(&mut job, "no-such-thing", "width", "40 in", "").is_err());
    assert_eq!(file::save(&mut job), before, "nothing was changed");
}

/// An item whose name carries the size it had ("Vanity, 36 in" to 48 in) says so; one whose name does not
/// carry it does not.
#[test]
fn a_name_that_carries_the_old_size_is_said_and_one_that_does_not_is_not() {
    let (mut job, vanity, heater) = job_with_items();
    assert!(ed::item_size(&mut job, &vanity, "width", "48 in", "").unwrap().name_carries_old);
    assert!(!ed::item_size(&mut job, &heater, "width", "24 in", "").unwrap().name_carries_old);
}

/// The window's steps, and the one change, make the same saved job for the same typing, byte for byte.
#[test]
fn the_one_change_makes_the_same_saved_job_as_the_windows_steps() {
    for (what, typed) in [("width", "48 in"), ("depth", "22 in"), ("height", "36 in")] {
        let (mut a, vanity, _) = job_with_items();
        let (mut b, _, _) = job_with_items();
        the_windows_steps(&mut a, &vanity, what, typed);
        ed::item_size(&mut b, &vanity, what, typed, "").expect("changed");
        assert_eq!(file::save(&mut a), file::save(&mut b), "{what} {typed}");
    }
    // After all three checked on site, and then one typed over: what is set down is Assumed.
    let (mut a, vanity, _) = job_with_items();
    let (mut b, _, _) = job_with_items();
    for what in ["width", "depth", "height"] {
        ed::measure(&mut a, &vanity, what, ed::Reading::Same, None, "checked on site").unwrap();
        ed::item_size(&mut b, &vanity, what, "same", "").unwrap();
    }
    assert!(followers(&mut b).iter().all(|(_, s)| *s == Status::Known));
    the_windows_steps(&mut a, &vanity, "width", "40 in");
    ed::item_size(&mut b, &vanity, "width", "40 in", "").unwrap();
    assert!(followers(&mut b).iter().all(|(_, s)| *s == Status::Assumed));
    assert_eq!(file::save(&mut a), file::save(&mut b));
}
