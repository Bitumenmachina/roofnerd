//! Call 197 point 10, contract 205M part 2: what is set down on a level, counted by the model
//! (`StoreyTotals::set_down`), where the sheet window grouped the model's placements itself.
//!
//! The window's rule, from `crates/app/src/sheet.rs` `set_down` (read, not edited): the things of the model's
//! `placements()` list (`snap.drawn`) whose storey is the level and whose kind is placed, grouped by kind in
//! the order each kind first appears; each group says `kind.so_many(n)`, the worst status of its things, and the
//! names of its things in the list's order joined by ", ". Two things differ, and are said: the window's
//! `Kind::is_placed` includes fittings (the model's count is equipment, fixtures and furnishings: a fitting is
//! joined to a run, not set down), and the window's list holds only what has a shape, so a thing set down with
//! no size is missing from its count (the model counts it and says it is not drawn).
//!
//! Made-up figures, worked by hand: a level with a "Vanity, 36 in" set down twice and a "Water heater, 50 gal"
//! once. Nothing priced: 2 furnishings and 1 piece of equipment, all 3 not priced. The heater priced as
//! "Bought and set": 2 not priced (the two vanities).

use plumbline_world::building::{placements, storey_totals, SetDownKind};
use plumbline_world::editing as ed;
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::view::pricing;
use plumbline_world::*;

const HEATER: &str = "Water heater, 50 gal";
const VANITY: &str = "Vanity, 36 in";

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up" }
}

fn rows(text: &str) -> Vec<Vec<String>> {
    plumbline_world::building::shape::csv_rows(text)
}

fn empty() -> (Job, String) {
    let mut job = Job::new("A level (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    (job, level)
}

fn three() -> (Job, String) {
    let (mut job, level) = empty();
    ed::place_item(&mut job, &level, VANITY, [5.0, 5.0], "0 ft", known()).unwrap();
    ed::place_item(&mut job, &level, HEATER, [10.0, 5.0], "0 ft", known()).unwrap();
    ed::place_item(&mut job, &level, VANITY, [9.0, 5.0], "0 ft", known()).unwrap();
    job.derive();
    (job, level)
}

fn the_level(job: &mut Job) -> plumbline_world::building::StoreyTotals {
    storey_totals(job).remove(0)
}

/// The sheet window's grouping, as its code does it: from the model's placements.
fn windows_grouping(job: &mut Job, level: &str) -> Vec<(Kind, usize, Status, String)> {
    let (drawn, _) = placements(job);
    let mut kinds: Vec<Kind> = Vec::new();
    for p in drawn.iter().filter(|p| p.kind.is_placed() && p.storey.as_deref() == Some(level)) {
        if !kinds.contains(&p.kind) {
            kinds.push(p.kind);
        }
    }
    kinds
        .into_iter()
        .map(|kind| {
            let here: Vec<&plumbline_world::building::Placement> = drawn.iter().filter(|p| p.kind == kind && p.storey.as_deref() == Some(level)).collect();
            let worst = here.iter().map(|p| p.status).max().unwrap_or(Status::Unknown);
            (kind, here.len(), worst, here.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "))
        })
        .collect()
}

fn models(k: &SetDownKind) -> (Kind, usize, Status, String) {
    (k.kind, k.count, k.status, k.names.join(", "))
}

#[test]
fn a_level_with_nothing_set_down_says_none() {
    let (mut job, _) = empty();
    let t = the_level(&mut job);
    assert!(t.set_down.kinds.is_empty());
    assert_eq!((t.set_down.count(), t.set_down.not_priced(), t.set_down.not_drawn()), (0, 0, 0));
    assert_eq!(t.set_down.said(), vec!["Nothing is set down on this level.".to_string()]);
}

#[test]
fn two_vanities_and_a_water_heater_are_two_furnishings_and_a_piece_of_equipment_with_their_names_and_nothing_priced_on_any() {
    let (mut job, level) = three();
    let t = the_level(&mut job);
    let d = &t.set_down;
    // In the order each kind first appears: the vanity was set down first.
    assert_eq!(d.kinds.iter().map(|k| (k.kind, k.count)).collect::<Vec<_>>(), vec![(Kind::Furnishing, 2), (Kind::Equipment, 1)]);
    // The second is called as the job numbers a second of a name.
    assert_eq!(d.kinds[0].names, vec![VANITY.to_string(), format!("{VANITY} 2")]);
    assert_eq!(d.kinds[1].names, vec![HEATER.to_string()]);
    assert_eq!(d.kinds[0].paths.len(), 2);
    assert_eq!((d.count(), d.not_priced(), d.not_drawn()), (3, 3, 0));
    assert_eq!((d.kinds[0].not_priced, d.kinds[1].not_priced), (2, 1));
    // Every one is drawn from sizes the item says are Assumed (a typical size): as sure as the window shows it.
    assert!(d.kinds.iter().all(|k| k.status == Status::Assumed), "{:?}", d.kinds.iter().map(|k| k.status).collect::<Vec<_>>());
    assert_eq!(d.said(), vec![
        format!("2 furnishings: {VANITY}, {VANITY} 2"),
        format!("1 piece of equipment: {HEATER}"),
        "2 furnishings, 1 piece of equipment are set down and not priced.".to_string(),
    ]);
    // The same as what the window works out for itself.
    assert_eq!(d.kinds.iter().map(models).collect::<Vec<_>>(), windows_grouping(&mut job, &level));
    // And the whole job's own figure for what is not priced says the same three.
    let p = pricing(&mut job);
    let all: usize = p.not_priced.iter().filter(|(k, _)| k.is_placed()).map(|x| x.1).sum();
    assert_eq!(all, d.not_priced());
}

#[test]
fn what_is_priced_is_not_counted_as_priced_on_nothing() {
    let (mut job, _) = three();
    ed::item_priced_as(&mut job, HEATER, Some("Bought and set")).unwrap();
    let d = the_level(&mut job).set_down;
    assert_eq!((d.kinds[0].not_priced, d.kinds[1].not_priced, d.not_priced()), (2, 0, 2));
    assert_eq!(d.said().last().unwrap(), "2 furnishings are set down and not priced.");
    ed::item_priced_as(&mut job, VANITY, Some("Bought and set")).unwrap();
    let d = the_level(&mut job).set_down;
    assert_eq!(d.not_priced(), 0);
    assert_eq!(d.said().len(), 2, "nothing left to say is not priced: {:?}", d.said());
    // The whole job's figure agrees.
    assert!(pricing(&mut job).not_priced.iter().all(|(k, _)| !k.is_placed()));
}

#[test]
fn a_box_in_a_cabinet_run_is_one_thing_counted_once() {
    let mut job = Job::new("A run (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [12.0, 0.0], [12.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let room = ed::room(&mut job, &level, "Kitchen", [6.0, 8.0], known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    ed::item_priced_as(&mut job, "B18", Some("Bought and set")).unwrap();
    let run = ed::cabinets(&mut job, &level, &walls[0], &room, "1 ft", &ed::box_list("B18 B24 B18").unwrap(), true, known()).unwrap();
    assert_eq!(run.boxes.len(), 3);
    let t = the_level(&mut job);
    let furnishings = t.set_down.kinds.iter().find(|k| k.kind == Kind::Furnishing).unwrap();
    // Three boxes and the countertop: four things, each once, each path once.
    assert_eq!(furnishings.count, 4, "{:?}", furnishings.names);
    let mut paths = furnishings.paths.clone();
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), 4);
    for (path, ..) in &run.boxes {
        assert!(furnishings.paths.contains(path), "{path}");
    }
    assert_eq!(furnishings.names.iter().filter(|n| n.starts_with("B18")).count(), 2);
    // The model's count is what the window's grouping would give.
    assert_eq!(t.set_down.kinds.iter().map(models).collect::<Vec<_>>(), windows_grouping(&mut job, &level));
}

#[test]
fn a_thing_with_no_size_is_counted_and_said_not_drawn_where_the_windows_own_list_would_leave_it_out() {
    let (mut job, level) = three();
    // An item whose sizes are not all known is set down and has no shape (editing.rs, `place_item`).
    let item = |kind, w: &str| {
        let size = |t: &str| {
            let (len, unit) = units::parse_length(t).unwrap();
            Val { value: Some(len), unit: unit.to_string(), status: Status::Assumed, source: "made-up".to_string() }
        };
        let unknown = || Val::<String>::unknown("", "not chosen");
        Item { category: kind, maker: unknown(), model: unknown(), width: size(w), depth: Val::unknown("ft", "not measured"), height: size("30 in"), weight: None, sheet: String::new(), division: None, form: None, priced_as: None }
    };
    ed::item(&mut job, "Oddment", item(Kind::Furnishing, "20 in")).unwrap();
    ed::place_item(&mut job, &level, "Oddment", [14.0, 5.0], "0 ft", known()).unwrap();
    job.derive();
    let t = the_level(&mut job);
    let f = t.set_down.kinds.iter().find(|k| k.kind == Kind::Furnishing).unwrap();
    assert_eq!((f.count, f.not_drawn), (3, 1), "{f:?}");
    assert_eq!(f.status, Status::Unknown, "a thing with no size is as sure as its size: not at all");
    assert!(f.names.contains(&"Oddment".to_string()));
    assert!(t.set_down.said().iter().any(|l| l.contains("1 furnishing cannot be drawn") && l.contains("its size is not known")), "{:?}", t.set_down.said());
    // The window's grouping leaves it out: 2 furnishings, which is the difference the model's count makes.
    let theirs = windows_grouping(&mut job, &level);
    assert_eq!(theirs.iter().find(|k| k.0 == Kind::Furnishing).unwrap().1, 2);
}

#[test]
fn things_on_two_levels_are_counted_each_on_its_own() {
    let (mut job, level) = three();
    let upper = ed::storey(&mut job, "upper", "10 ft", "9 ft", known()).unwrap();
    ed::place_item(&mut job, &upper, VANITY, [5.0, 5.0], "0 ft", known()).unwrap();
    job.derive();
    let totals = storey_totals(&mut job);
    let (ground, up) = (totals.iter().find(|t| t.path == level).unwrap(), totals.iter().find(|t| t.path == upper).unwrap());
    assert_eq!((ground.set_down.count(), up.set_down.count()), (3, 1));
    assert_eq!(up.set_down.kinds.iter().map(models).collect::<Vec<_>>(), windows_grouping(&mut job, &upper));
    assert_eq!(ground.set_down.kinds.iter().map(models).collect::<Vec<_>>(), windows_grouping(&mut job, &level));
}
