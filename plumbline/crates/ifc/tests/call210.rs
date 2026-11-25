//! Call 210 (contract 210M): another trade's model brought into a job that has a building, only its runs and what they serve.
//! The sample whole house is the job; the designer's model is made here, with the program's own changes, and written out as a
//! model. Everything is made up. The figures are worked by hand in STATUS.md ("Another trade's model into my job").
//!
//! The model: a job of its own, one level "Level 1" at 0 ft (10 ft), four walls on the sample's outside line (40 ft by 28 ft:
//! they lie over the sample's own), one floor of the same outline, one duct 10 ft long, 14 in by 8 in, from (20, 20) to
//! (30, 20) at 8 ft (through the sample's partition at 24, which runs (24, 16) to (24, 28) and is 9 ft high), on a supply-air
//! system, and one air handler 2 ft by 2.5 ft by 5 ft with its middle at (30, 8), clear of every wall.

use plumbline_ifc::{import, import_choosing, write::export, Bring};
use plumbline_world::building::{self, clash};
use plumbline_world::editing::{self as ed, Entered, FloorBy};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const MADE: Entered = Entered { status: Status::Assumed, source: "made-up designer" };
const HOW: Entered = Entered { status: Status::Assumed, source: "the designer's model, made up" };

fn designer() -> String {
    let mut job = Job::new("Designer (not a real job)", "test", "2026-11-25");
    let l1 = ed::storey(&mut job, "Level 1", "0 ft", "10 ft", MADE).unwrap();
    let w6 = ed::wall_type(&mut job, "Outside wall, 6 in", "6 in", MADE).unwrap();
    ed::walls(&mut job, &l1, &w6, &[[0.0, 0.0], [40.0, 0.0], [40.0, 28.0], [0.0, 28.0]], true, ed::Top::Height("10 ft"), MADE).unwrap();
    let slab = ed::floor_type(&mut job, "Slab", "4 in", MADE).unwrap();
    ed::floor(&mut job, &l1, &slab, FloorBy::Points(&[[0.0, 0.0], [40.0, 0.0], [40.0, 28.0], [0.0, 28.0]]), MADE).unwrap();
    let t = ed::segment_type(&mut job, "Duct 14 by 8", Kind::Duct, "14 in", Some("8 in"), MADE).unwrap();
    let sys = ed::system(&mut job, "Supply air, designer", "Supply Air").unwrap();
    ed::runs(&mut job, &l1, Kind::Duct, &t, &[[20.0, 20.0], [30.0, 20.0]], "8 ft", Some(&sys), MADE).unwrap();
    ed::place(&mut job, &l1, Kind::Equipment, "air handler", [30.0, 8.0], "0 ft", Some([2.0, 2.5, 5.0]), None, MADE).unwrap();
    export(&mut job, "designer.ifc", WHEN).unwrap().text
}

fn count(job: &mut Job, kind: Kind) -> usize {
    let mut q = job.world.query::<&Kind>();
    q.iter(&job.world).filter(|k| **k == kind).count()
}

fn levels(job: &mut Job) -> Vec<String> {
    let mut q = job.world.query::<(&Node, &building::Level)>();
    let mut v: Vec<String> = q.iter(&job.world).map(|(n, _)| n.name.clone()).collect();
    v.sort();
    v
}

fn path_of(job: &mut Job, level: &str) -> Option<String> {
    let mut q = job.world.query::<(&Node, &building::Level)>();
    q.iter(&job.world).find(|(n, _)| n.name == level).map(|(n, _)| n.path.clone())
}

fn price(job: &mut Job) -> f64 {
    (view::estimate(job).price.value.unwrap() * 100.0).round() / 100.0
}

fn counts(job: &mut Job) -> plumbline_ifc::ClashCounts {
    let found = clash::clashes(job, None).unwrap();
    plumbline_ifc::ClashCounts { same_space: found.in_the_same_space().len(), set_against: found.set_against().len(), holes: found.hole_rows().len(), laid_in: found.laid_in().len() }
}

/// Step 1, as the code does it (not as the design first read): brought in whole, the model is not refused: the duct and the air
/// handler come in, the four walls and the floor are left out as lying over the job's own, and the reply says that the runs and
/// the equipment can be brought in alone.
#[test]
fn step_1_whole_leaves_what_lies_over_and_says_only_the_runs_can_be_chosen() {
    let mut job = sample::house_priced();
    let b = import_choosing(&mut job, designer().as_bytes(), "designer.ifc", HOW, Bring::Whole).unwrap();
    assert_eq!((b.made.get("ducts"), b.made.get("pieces of equipment")), (Some(&1), Some(&1)), "{:#?}", b.made);
    let said = b.left_out_said().join("\n");
    assert_eq!(said.matches("1 wall: ").count(), 4, "{said}");
    assert_eq!(said.matches("one wall drawn over another would be counted twice").count(), 4, "{said}");
    assert!(said.contains("1 floor slab") && said.contains("already on this level"), "{said}");
    let notes = b.notes.join("\n");
    assert!(notes.contains("4 walls and 1 floor of the model lie over what the job has and were left out"), "{notes}");
    assert!(notes.contains("choose only its runs and the equipment they serve"), "{notes}");
    assert!(!notes.contains('`') && !notes.contains("--"), "{notes}");
    assert!(b.not_asked.is_empty());
}

/// Step 2: only its runs and what they serve: 1 duct and 1 piece of equipment come in, 4 walls and 1 floor are counted as not
/// asked for, the model's level is the job's first floor, no level is added.
#[test]
fn step_2_only_runs_brings_the_duct_and_the_air_handler_and_counts_the_rest_as_not_asked() {
    let mut job = sample::house_priced();
    let levels_before = levels(&mut job);
    let b = import_choosing(&mut job, designer().as_bytes(), "designer.ifc", HOW, Bring::RunsOnly).unwrap();
    assert_eq!(b.made.get("ducts"), Some(&1), "{:#?}", b.made);
    assert_eq!(b.made.get("pieces of equipment"), Some(&1), "{:#?}", b.made);
    assert_eq!(b.made.len(), 2, "nothing else was made: {:#?}", b.made);
    assert_eq!(b.not_asked, vec![("floors".to_string(), 1), ("walls".to_string(), 4)], "{:#?}", b.not_asked);
    assert!(b.left_out.is_empty(), "nothing was asked for and refused: {:#?}", b.left_out);
    assert_eq!(levels(&mut job), levels_before, "no level is added");
    let notes = b.notes.join("\n");
    assert!(notes.contains("the model's level \"Level 1\" has its floor at 0 ft, the height of the job's \"first floor\": it is that level"), "{notes}");
    assert!(!notes.contains("was added as a new level"), "{notes}");
    let said = b.not_asked_said().unwrap();
    assert_eq!(said, "not asked for: 1 floor and 4 walls", "{said}");
}

/// Step 3: the sample's own figures do not move: walls, rooms, floors, roof, and the price to the cent; the two things carry no
/// cost line and the estimate says two more things are priced on nothing.
#[test]
fn step_3_the_samples_figures_and_price_do_not_move() {
    let mut job = sample::house_priced();
    let before = (count(&mut job, Kind::Wall), count(&mut job, Kind::Room), count(&mut job, Kind::Floor), count(&mut job, Kind::Roof), price(&mut job));
    assert_eq!(before.4, 468_050.22);
    import_choosing(&mut job, designer().as_bytes(), "designer.ifc", HOW, Bring::RunsOnly).unwrap();
    let after = (count(&mut job, Kind::Wall), count(&mut job, Kind::Room), count(&mut job, Kind::Floor), count(&mut job, Kind::Roof), price(&mut job));
    assert_eq!(after, before);
    let said = view::not_priced_said(&view::pricing(&mut job).not_priced).unwrap();
    assert!(said.contains("1 duct, 1 piece of equipment"), "{said}");
}

/// Step 4: the reply says how many clashes the two things make with the house, by kind, from the model's own clash list before
/// and after; the duct through the partition at 24 is the one hole to cut; and the figures are what `clashes` gives.
#[test]
fn step_4_the_reply_counts_the_clashes_made_by_the_clash_list_before_and_after() {
    let mut job = sample::house_priced();
    let b = import_choosing(&mut job, designer().as_bytes(), "designer.ifc", HOW, Bring::RunsOnly).unwrap();
    let c = b.clashes.clone().expect("the reply says what it was done for");
    // The sample alone: 11 in the same space, 10 set against, 47 holes. With the two things: one more hole.
    assert_eq!(c.before, plumbline_ifc::ClashCounts { same_space: 11, set_against: 10, holes: 47, laid_in: c.before.laid_in });
    assert_eq!(c.after, plumbline_ifc::ClashCounts { same_space: 11, set_against: 10, holes: 48, laid_in: c.before.laid_in });
    assert_eq!(c.made, plumbline_ifc::ClashCounts { same_space: 0, set_against: 0, holes: 1, laid_in: 0 });
    assert_eq!(c.after, counts(&mut job), "the same function the clashes verb calls");
    assert_eq!(c.listed.len(), 1, "{:#?}", c.listed);
    assert!(c.listed[0].contains("duct 1") && c.listed[0].contains("partition at 24") && c.listed[0].contains("14.00 in by 8.00 in") && c.listed[0].contains("(24, 20, 8)"), "{:#?}", c.listed);
    let said = b.clashes_said().join("\n");
    assert!(said.contains("1 hole to cut") && said.contains("0 in the same space") && said.contains("0 set against a wall"), "{said}");
    assert!(said.contains("the job had 47 holes to cut and now has 48"), "{said}");
}

/// Step 5: the same file again is refused, nothing doubled.
#[test]
fn step_5_the_same_file_again_is_refused_and_nothing_is_doubled() {
    let mut job = sample::house_priced();
    let model = designer();
    import_choosing(&mut job, model.as_bytes(), "designer.ifc", HOW, Bring::RunsOnly).unwrap();
    let before = file::save(&mut job);
    let why = import_choosing(&mut job, model.as_bytes(), "designer.ifc", HOW, Bring::RunsOnly).unwrap_err();
    assert!(why.contains("this model is already in the job: nothing was brought in"), "{why}");
    assert!(why.contains("2 of its 2 things"), "{why}");
    assert!(why.contains("It holds 1 floor and 4 walls, which were not asked for"), "{why}");
    assert!(why.contains("(1 duct, 1 piece of equipment)"), "{why}");
    assert_eq!(file::save(&mut job), before);
    // The sample's 23 ducts and 5 pieces of equipment, and the one each: not doubled.
    assert_eq!((count(&mut job, Kind::Duct), count(&mut job, Kind::Equipment)), (24, 6));
}

/// Step 6: everything brought in reads Assumed and says it came from the file.
#[test]
fn step_6_everything_brought_in_is_assumed_and_sourced_to_the_file() {
    let mut job = sample::house_priced();
    // Said Known: the reader never takes a model's say-so.
    let b = import_choosing(&mut job, designer().as_bytes(), "designer.ifc", Entered { status: Status::Known, source: "the designer, by phone" }, Bring::RunsOnly).unwrap();
    assert!(b.notes.iter().any(|n| n.contains("was brought in Assumed, not Known as was said of it")), "{:#?}", b.notes);
    let mut q = job.world.query::<(&Node, &Kind, &building::Segment)>();
    let ducts: Vec<_> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Duct && n.name == "duct 1").collect();
    assert_eq!(ducts.len(), 1);
    let s = ducts[0].2;
    assert_eq!((s.line.status, s.offset.status), (Status::Assumed, Status::Assumed));
    assert!(s.line.source.contains("designer.ifc") && s.line.source.contains("duct"), "{}", s.line.source);
    let mut q = job.world.query::<(&Node, &Kind, &Placed)>();
    let placed: Vec<_> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Equipment && n.name.starts_with("air handler")).collect();
    let new: Vec<_> = placed.iter().filter(|(_, _, p)| p.at_ft().is_some_and(|a| (a[0] - 30.0).abs() < 1e-6 && (a[1] - 8.0).abs() < 1e-6)).collect();
    assert_eq!(new.len(), 1);
    let p = new[0].2;
    assert_eq!((p.at.status, p.size.status), (Status::Assumed, Status::Assumed));
    assert!(p.at.source.contains("designer.ifc"), "{}", p.at.source);
}

/// A model of walls alone, brought in as runs only: nothing comes in, and it says so, and what it holds that was not asked for.
#[test]
fn a_model_of_walls_alone_brings_nothing_as_runs_only_and_says_so() {
    let mut other = Job::new("Walls only (not a real job)", "test", "2026-11-25");
    let l = ed::storey(&mut other, "L1", "0 ft", "9 ft", MADE).unwrap();
    let w = ed::wall_type(&mut other, "W", "6 in", MADE).unwrap();
    ed::walls(&mut other, &l, &w, &[[0.0, 40.0], [12.0, 40.0], [12.0, 52.0]], false, ed::Top::Height("9 ft"), MADE).unwrap();
    let text = export(&mut other, "walls.ifc", WHEN).unwrap().text;
    let mut job = sample::house_priced();
    let before = file::save(&mut job);
    let why = import_choosing(&mut job, text.as_bytes(), "walls.ifc", HOW, Bring::RunsOnly).unwrap_err();
    assert!(why.contains("this model has no runs and no equipment: nothing was brought in"), "{why}");
    assert!(why.contains("It holds 2 walls, which were not asked for"), "{why}");
    assert_eq!(file::save(&mut job), before, "nothing was brought in");
    // Brought in whole, the same model comes in as walls, as it always did.
    let b = import_choosing(&mut job, text.as_bytes(), "walls.ifc", HOW, Bring::Whole).unwrap();
    assert_eq!(b.made.get("walls"), Some(&2));
}

/// A run on a level at a height the job does not have is put on the job's level it stands within, and the reply says so; a level is
/// not added. By hand: the job's levels are the first floor at 0 ft and the second floor at 10 ft. The model's "Mezzanine" has its
/// floor at 4 ft and a duct 3 ft above it: 7 ft up, which is within the first floor, 7 ft above its floor. Its "Loft" has its
/// floor at 13 ft and an air handler standing on it 2 ft up: 15 ft up, within the second floor, 5 ft above its floor.
#[test]
fn a_run_on_a_level_at_a_height_the_job_lacks_is_put_on_the_level_it_stands_within() {
    let mut other = Job::new("Mezzanine (not a real job)", "test", "2026-11-25");
    let mez = ed::storey(&mut other, "Mezzanine", "4 ft", "5 ft", MADE).unwrap();
    let loft = ed::storey(&mut other, "Loft", "13 ft", "6 ft", MADE).unwrap();
    let t = ed::segment_type(&mut other, "Duct 12 round", Kind::Duct, "12 in", None, MADE).unwrap();
    ed::runs(&mut other, &mez, Kind::Duct, &t, &[[5.0, 5.0], [15.0, 5.0]], "3 ft", None, MADE).unwrap();
    ed::place(&mut other, &loft, Kind::Equipment, "unit heater", [8.0, 8.0], "2 ft", Some([2.0, 2.0, 2.0]), None, MADE).unwrap();
    let text = export(&mut other, "mezz.ifc", WHEN).unwrap().text;
    let mut job = Job::new("House (not a real job)", "test", "2026-11-25");
    ed::storey(&mut job, "first floor", "0 ft", "10 ft", MADE).unwrap();
    ed::storey(&mut job, "second floor", "10 ft", "9 ft", MADE).unwrap();
    let b = import_choosing(&mut job, text.as_bytes(), "mezz.ifc", HOW, Bring::RunsOnly).unwrap();
    assert_eq!(levels(&mut job), vec!["first floor".to_string(), "second floor".to_string()], "no level is added");
    assert_eq!((b.made.get("ducts"), b.made.get("pieces of equipment"), b.made.get("levels")), (Some(&1), Some(&1), None), "{:#?}", b.made);
    let notes = b.notes.join("\n");
    assert!(
        notes.contains("the model's level \"Mezzanine\" has its floor at 4 ft, which is not the height of any level the job has: it holds 1 thing, and each was put on the level it stands within (the job's \"first floor\"); no level was added"),
        "{notes}"
    );
    assert!(notes.contains("the model's level \"Loft\" has its floor at 13 ft") && notes.contains("the job's \"second floor\""), "{notes}");
    // Where each stands now.
    let (first, second) = (path_of(&mut job, "first floor"), path_of(&mut job, "second floor"));
    let mut q = job.world.query::<(&Node, &Kind, &building::Segment)>();
    let (n, _, s) = q.iter(&job.world).find(|(_, k, _)| **k == Kind::Duct).unwrap();
    assert_eq!(n.parent, first);
    assert_eq!(s.offset.feet(), Some(7.0), "3 ft above the model's 4 ft is 7 ft above the first floor");
    let mut q = job.world.query::<(&Node, &Kind, &Placed)>();
    let (n, _, p) = q.iter(&job.world).find(|(_, k, _)| **k == Kind::Equipment).unwrap();
    assert_eq!(n.parent, second);
    assert_eq!(p.offset.feet(), Some(5.0), "2 ft above the model's 13 ft is 15 ft: 5 ft above the second floor");
    // Brought in whole, the same model adds its two levels, as it always did.
    let mut whole = Job::new("House (not a real job)", "test", "2026-11-25");
    ed::storey(&mut whole, "first floor", "0 ft", "10 ft", MADE).unwrap();
    ed::storey(&mut whole, "second floor", "10 ft", "9 ft", MADE).unwrap();
    import_choosing(&mut whole, text.as_bytes(), "mezz.ifc", HOW, Bring::Whole).unwrap();
    assert_eq!(levels(&mut whole).len(), 4);
}

/// A fixture connected to a run in the file comes in connected: the sample whole house is the mechanical trade's file; its walls,
/// floors and roof are not asked for, and the open ends by system are what they are when it is brought in whole into an empty job.
#[test]
fn a_fixture_connected_to_a_run_in_the_file_comes_in_connected() {
    let mut first = sample::house_priced();
    let text = export(&mut first, "house.ifc", WHEN).unwrap().text;
    // The job: the same levels, nothing on them.
    let mut job = Job::new("Bare house (not a real job)", "test", "2026-11-25");
    let mut have: Vec<(String, f64, f64)> = {
        let mut q = first.world.query::<(&Node, &building::Level)>();
        q.iter(&first.world).map(|(n, l)| (n.name.clone(), l.elevation.feet().unwrap(), l.height.feet().unwrap_or(9.0))).collect()
    };
    have.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (name, at, high) in &have {
        ed::storey(&mut job, name, &format!("{at} ft"), &format!("{high} ft"), MADE).unwrap();
    }
    let b = import_choosing(&mut job, text.as_bytes(), "house.ifc", HOW, Bring::RunsOnly).unwrap();
    assert_eq!(b.made.get("fixtures"), Some(&22), "{:#?}", b.made);
    assert_eq!(b.made.get("pieces of equipment"), Some(&5), "{:#?}", b.made);
    assert!(b.made.get("walls").is_none() && b.made.get("floors").is_none() && b.made.get("roofs").is_none(), "{:#?}", b.made);
    assert!(b.not_asked.iter().any(|(k, n)| k == "walls" && *n == 20), "{:#?}", b.not_asked);
    let notes = b.notes.join("\n");
    assert!(notes.contains("46 connections on fixtures and equipment were brought in from the file's connection points, each Assumed; 46 were tied to the end of a run"), "{notes}");
    let whole = {
        let mut empty = Job::new("Second (not a real job)", "test", "2026-11-25");
        import(&mut empty, text.as_bytes(), "house.ifc", HOW).unwrap();
        empty
    };
    let mut whole = whole;
    let open = |job: &mut Job| {
        job.derive();
        let mut out = std::collections::BTreeMap::new();
        for t in building::takeoff(job).into_iter().filter(|t| !t.classification.is_empty()) {
            *out.entry(t.classification.clone()).or_insert(0usize) += t.joined.open.len();
        }
        out
    };
    assert_eq!(open(&mut job), open(&mut whole));
}

/// A model with two systems: each run goes on the system of its own kind: the job's own where it has one, a new one named as the
/// file names it where it has not.
#[test]
fn a_model_with_two_systems_puts_each_run_on_its_own() {
    let mut other = Job::new("Two systems (not a real job)", "test", "2026-11-25");
    let l = ed::storey(&mut other, "L1", "0 ft", "9 ft", MADE).unwrap();
    let air = ed::system(&mut other, "Supply air, designer", "Supply Air").unwrap();
    let cold = ed::system(&mut other, "Cold water, designer", "Domestic Cold Water").unwrap();
    let d = ed::segment_type(&mut other, "Duct 10 round", Kind::Duct, "10 in", None, MADE).unwrap();
    let p = ed::segment_type(&mut other, "Pipe 1 in", Kind::Pipe, "1 in", None, MADE).unwrap();
    ed::runs(&mut other, &l, Kind::Duct, &d, &[[3.0, 3.0], [13.0, 3.0]], "8 ft", Some(&air), MADE).unwrap();
    ed::runs(&mut other, &l, Kind::Pipe, &p, &[[3.0, 6.0], [13.0, 6.0]], "2 ft", Some(&cold), MADE).unwrap();
    let text = export(&mut other, "two.ifc", WHEN).unwrap().text;
    let mut job = Job::new("House (not a real job)", "test", "2026-11-25");
    ed::storey(&mut job, "first floor", "0 ft", "10 ft", MADE).unwrap();
    let own = ed::system(&mut job, "Supply air", "Supply Air").unwrap();
    let b = import_choosing(&mut job, text.as_bytes(), "two.ifc", HOW, Bring::RunsOnly).unwrap();
    assert_eq!((b.made.get("ducts"), b.made.get("pipes")), (Some(&1), Some(&1)), "{:#?}", b.made);
    let mut q = job.world.query::<(&Node, &Kind, &building::InSystem)>();
    let on: Vec<(String, String)> = q.iter(&job.world).map(|(n, _, s)| (n.name.clone(), s.0.value.clone().unwrap_or_default())).collect();
    assert_eq!(on.len(), 2, "{on:?}");
    assert!(on.iter().any(|(n, s)| n.starts_with("duct") && *s == own), "the duct is on the job's own supply air: {on:?}");
    let mut q = job.world.query::<(&Node, &building::SystemInfo)>();
    let systems: Vec<(String, String)> = q.iter(&job.world).map(|(n, s)| (n.name.clone(), s.classification.clone())).collect();
    assert_eq!(systems.len(), 2, "one the job had, one new: {systems:?}");
    assert!(systems.iter().any(|(n, c)| n == "Cold water, designer" && c == "Domestic Cold Water"), "{systems:?}");
}

fn one_duct_on(level: &str, at: &str, run_at: &str) -> String {
    let mut other = Job::new("One duct (not a real job)", "test", "2026-11-25");
    let l = ed::storey(&mut other, level, at, "9 ft", MADE).unwrap();
    let t = ed::segment_type(&mut other, "Duct 12 round", Kind::Duct, "12 in", None, MADE).unwrap();
    ed::runs(&mut other, &l, Kind::Duct, &t, &[[5.0, 5.0], [15.0, 5.0]], run_at, None, MADE).unwrap();
    export(&mut other, "one.ifc", WHEN).unwrap().text
}

/// A whole model refused for what has nothing to do with its runs (a level of the job's own name at another height) says that its
/// runs alone can be brought in, with what they are, because a trial of it was worked and taken back; and bringing them in alone works.
#[test]
fn a_whole_refusal_says_what_could_be_brought_in_alone_and_it_can() {
    let text = one_duct_on("first floor", "5 ft", "2 ft");
    let mut job = Job::new("House (not a real job)", "test", "2026-11-25");
    ed::storey(&mut job, "first floor", "0 ft", "10 ft", MADE).unwrap();
    let before = file::save(&mut job);
    let why = import_choosing(&mut job, text.as_bytes(), "one.ifc", HOW, Bring::Whole).unwrap_err();
    assert!(why.contains("they cannot both be right"), "{why}");
    assert!(why.contains("Its runs and the equipment they serve (1 duct) can be brought in alone, without its walls and floors"), "{why}");
    assert_eq!(file::save(&mut job), before, "the trial was taken back");
    // What the window and `import` say is as it was: no new words.
    let old = import(&mut job, text.as_bytes(), "one.ifc", HOW).unwrap_err();
    assert!(!old.contains("brought in alone"), "{old}");
    let b = import_choosing(&mut job, text.as_bytes(), "one.ifc", HOW, Bring::RunsOnly).unwrap();
    assert_eq!(b.made.get("ducts"), Some(&1));
    assert_eq!(levels(&mut job), vec!["first floor".to_string()], "no level was added");
    // 2 ft above the model's 5 ft floor is 7 ft up, within the job's first floor.
    let mut q = job.world.query::<(&Kind, &building::Segment)>();
    assert_eq!(q.iter(&job.world).find(|(k, _)| **k == Kind::Duct).unwrap().1.offset.feet(), Some(7.0));
}

/// A job with no level has nothing to put runs on, and says so; and a run below the job's lowest level is put on that level, said.
#[test]
fn no_level_to_stand_on_is_refused_and_a_run_below_the_lowest_is_put_on_it_and_said() {
    let text = one_duct_on("Basement", "-9 ft", "2 ft");
    let mut empty = Job::new("Empty (not a real job)", "test", "2026-11-25");
    let before = file::save(&mut empty);
    let why = import_choosing(&mut empty, text.as_bytes(), "one.ifc", HOW, Bring::RunsOnly).unwrap_err();
    assert!(why.contains("the job has no level with a height to put the model's runs on"), "{why}");
    assert_eq!(file::save(&mut empty), before);
    let mut job = Job::new("House (not a real job)", "test", "2026-11-25");
    ed::storey(&mut job, "first floor", "0 ft", "10 ft", MADE).unwrap();
    let b = import_choosing(&mut job, text.as_bytes(), "one.ifc", HOW, Bring::RunsOnly).unwrap();
    // 2 ft above -9 ft is 7 ft below the first floor, which holds it.
    let mut q = job.world.query::<(&Kind, &building::Segment)>();
    assert_eq!(q.iter(&job.world).find(|(k, _)| **k == Kind::Duct).unwrap().1.offset.feet(), Some(-7.0));
    let notes = b.notes.join("\n");
    assert!(notes.contains("the model's level \"Basement\" has its floor at -9 ft") && notes.contains("1 thing stands below the job's lowest level, \"first floor\" (its floor at 0 ft): ") && notes.contains("7 ft below it; check the model's heights against the job's"), "{notes}");
}
