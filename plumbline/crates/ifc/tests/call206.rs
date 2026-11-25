//! Call 206 point 3: a model brought into a job that has a building. A level of the model at the height of one the job
//! has (within an eighth of an inch) is that level, under the job's name, and the reply says so; a level at a new height is
//! added and said; and the reply names every room of the job whose figures moved, worked by the model. Everything is made up.
//!
//! By hand. The job: "first floor" at 0 ft, 10 ft high, a ring 20 ft by 12 ft of 6 in walls, one room with its point at
//! (5, 6). Inside the faces: 19.5 ft by 11.5 ft = 224.25 SF. The model: its level "L1" at 0 ft (9 ft high) with one wall 12 ft
//! long at x = 10, 6 in thick, which cuts the room at 9.75 and 10.25 ft: the room's part holding (5, 6) is 9.5 ft by 11.5 ft
//! = 109.25 SF. The model's level "L2" at 10 ft is at no height the job has, and is added.

use plumbline_ifc::{import, write::export};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const MADE: Entered = Entered { status: Status::Known, source: "made-up survey" };
const HOW: Entered = Entered { status: Status::Assumed, source: "a made-up model" };

fn house() -> Job {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "first floor", "0 ft", "10 ft", MADE).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", MADE).unwrap();
    ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0], [20.0, 12.0], [0.0, 12.0]], true, ed::Top::Height("10 ft"), MADE).unwrap();
    ed::room(&mut job, &level, "living", [5.0, 6.0], MADE).unwrap();
    job.derive();
    job
}

/// Another trade's model: "L1" at 0 ft, one wall at x = 10; and, if asked, "L2" at 10 ft with a wall.
fn model(second: bool) -> String {
    let mut job = Job::new("Another trade (not a real job)", "test", "2026-11-25");
    let l1 = ed::storey(&mut job, "L1", "0 ft", "9 ft", MADE).unwrap();
    let w6 = ed::wall_type(&mut job, "W", "6 in", MADE).unwrap();
    ed::walls(&mut job, &l1, &w6, &[[10.0, 0.0], [10.0, 12.0]], false, ed::Top::Height("9 ft"), MADE).unwrap();
    if second {
        let l2 = ed::storey(&mut job, "L2", "10 ft", "9 ft", MADE).unwrap();
        ed::walls(&mut job, &l2, &w6, &[[3.0, 3.0], [9.0, 3.0]], false, ed::Top::Height("9 ft"), MADE).unwrap();
    }
    export(&mut job, "other.ifc", WHEN).unwrap().text
}

fn levels(job: &mut Job) -> Vec<(String, Option<f64>)> {
    let mut q = job.world.query::<(&Node, &building::Level)>();
    let mut v: Vec<(String, Option<f64>)> = q.iter(&job.world).map(|(n, l)| (n.name.clone(), l.elevation.feet())).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

#[test]
fn a_level_of_the_model_at_the_height_of_the_jobs_is_that_level_and_the_reply_says_so() {
    let mut job = house();
    let b = import(&mut job, model(false).as_bytes(), "other.ifc", HOW).unwrap();
    assert_eq!(levels(&mut job), vec![("first floor".to_string(), Some(0.0))], "no second level at 0 ft");
    let notes = b.notes.join("\n");
    assert!(notes.contains("the model's level \"L1\" has its floor at 0 ft, the height of the job's \"first floor\": it is that level"), "{notes}");
    assert!(notes.contains("the model gives it as 9 ft high and the job has it 10 ft high, and the job's stays"), "{notes}");
    // The wall stands on the job's level.
    let mut q = job.world.query::<(&Node, &Kind)>();
    let on_first = q.iter(&job.world).filter(|(n, k)| **k == Kind::Wall && n.parent.as_deref() == Some("level-0002")).count();
    assert_eq!(on_first, 5, "the ring's four and the model's one");
}

#[test]
fn the_reply_names_the_room_whose_figures_moved_with_its_area_before_and_after() {
    let mut job = house();
    let before = ed::room_figures(&mut job);
    assert!((before[0].area.unwrap() - 224.25).abs() < 1.0e-6, "{before:?}");
    let b = import(&mut job, model(false).as_bytes(), "other.ifc", HOW).unwrap();
    assert_eq!(b.rooms_moved.len(), 1, "{:?}", b.rooms_moved);
    let m = &b.rooms_moved[0];
    assert_eq!((m.name.as_str(), m.level.as_str()), ("living", "first floor"));
    assert!((m.area.0.unwrap() - 224.25).abs() < 1.0e-6 && (m.area.1.unwrap() - 109.25).abs() < 1.0e-6, "{m:?}");
    let notes = b.notes.join("\n");
    assert!(notes.contains("1 room of the job has other figures now, because walls brought in from the model stand in it: living (first floor): floor area 224.25 SF before, 109.25 SF now"), "{notes}");
}

#[test]
fn a_level_at_a_new_height_is_added_and_said_and_a_level_within_an_eighth_of_an_inch_is_the_same() {
    let mut job = house();
    let b = import(&mut job, model(true).as_bytes(), "other.ifc", HOW).unwrap();
    assert_eq!(levels(&mut job).len(), 2, "{:?}", levels(&mut job));
    let notes = b.notes.join("\n");
    assert!(notes.contains("the model's level \"L2\" has its floor at 10 ft, which is not the height of any level the job has: it was added as a new level"), "{notes}");
    // The job's level 0.1 in off the model's: the same level (1/8 in is 0.0104 ft; 0.1 in is 0.0083 ft); 0.2 in off: not.
    for (up, same) in [("0.1 in", true), ("0.2 in", false)] {
        let mut job = Job::new("A house (not a real job)", "test", "2026-11-25");
        ed::storey(&mut job, "ground", up, "10 ft", MADE).unwrap();
        let b = import(&mut job, model(false).as_bytes(), "other.ifc", HOW).unwrap();
        assert_eq!(levels(&mut job).len(), if same { 1 } else { 2 }, "{up}: {:?} {:?}", levels(&mut job), b.notes);
    }
}

#[test]
fn point_9_a_model_brought_in_twice_says_it_is_in_the_job_by_counts_and_in_a_builders_words() {
    let mut job = house();
    import(&mut job, model(false).as_bytes(), "other.ifc", HOW).unwrap();
    let why = import(&mut job, model(false).as_bytes(), "other.ifc", HOW).unwrap_err();
    assert_eq!(why, "this model is already in the job: nothing was brought in. 1 of its 1 thing is in the job already, in the same place, so a second would be counted twice (1 wall). To bring it in again, bring it into a job of its own, or take what it brought in out of this job first. A model comes in whole: bringing in only what the job lacks is not there yet");
    // The whole sample house twice: counted, one sentence, no word of the file's own, no clause for each fitting.
    let mut sample = plumbline_world::sample::house_priced();
    let text = export(&mut sample, "house.ifc", WHEN).unwrap().text;
    let mut again = Job::new("Again (not a real job)", "test", "2026-11-25");
    import(&mut again, text.as_bytes(), "house.ifc", HOW).unwrap();
    let why = import(&mut again, text.as_bytes(), "house.ifc", HOW).unwrap_err();
    assert!(why.starts_with("this model is already in the job: nothing was brought in. "), "{why}");
    assert!(!why.contains("distributionflowelement") && !why.contains("as the model calls it") && !why.contains("a equipment"), "{why}");
    assert!(!why.contains("fitting-0"), "no clause for each fitting: {why}");
    // The counts stay under 600 characters, as they were; what a person can do about it (call 209 point 4) comes after them.
    let counts = why.split(". To bring it in again").next().unwrap();
    assert!(counts.len() < 600, "{} characters: {why}", counts.len());
    assert!(why.contains("To bring it in again, bring it into a job of its own"), "{why}");
}

#[test]
fn point_10_a_connection_left_loose_is_named_with_its_thing_and_where() {
    let mut sample = plumbline_world::sample::house_priced();
    let text = export(&mut sample, "house.ifc", WHEN).unwrap().text;
    let mut whole = Job::new("Whole (not a real job)", "test", "2026-11-25");
    let b = import(&mut whole, text.as_bytes(), "house.ifc", HOW).unwrap();
    assert!(!b.notes.iter().any(|n| n.contains("not tied to the end of a run")), "all tied: {:?}", b.notes);
    // One tie taken out of the file.
    let at = text.lines().position(|l| l.contains("=IFCRELCONNECTSPORTS(")).unwrap();
    let cut: String = text.lines().enumerate().filter(|(i, _)| *i != at).map(|(_, l)| format!("{l}\n")).collect();
    let mut loose = Job::new("Loose (not a real job)", "test", "2026-11-25");
    let b = import(&mut loose, cut.as_bytes(), "house.ifc", HOW).unwrap();
    let said = b.notes.iter().find(|n| n.contains("not tied to the end of a run")).unwrap_or_else(|| panic!("{:?}", b.notes));
    assert!(said.starts_with("1 of the 46 connections is not tied to the end of a run in the file, and is left loose: "), "{said}");
    assert!(said.contains(" at ("), "{said}");
}

#[test]
fn point_8_a_roof_with_no_outline_of_this_programs_says_that() {
    let mut sample = plumbline_world::sample::house_priced();
    let text = export(&mut sample, "house.ifc", WHEN).unwrap().text;
    let other = text.replace("'Plumbline_Roof'", "'Somebody_Elses'");
    let mut job = Job::new("Other (not a real job)", "test", "2026-11-25");
    let b = import(&mut job, other.as_bytes(), "house.ifc", HOW).unwrap();
    assert!(b.left_out.iter().any(|(c, n, why)| c == "roof slab" && *n == 4 && why == "a roof with no outline of this program's in the file is not brought in: draw it from its footprint and its pitch"), "{:#?}", b.left_out);
    assert!(b.left_out.iter().all(|(_, _, why)| !why.contains("not brought in from a model yet")), "{:#?}", b.left_out);
}
