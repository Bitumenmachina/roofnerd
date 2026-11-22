//! Call 160: what a wall the work opens may be hiding. The house is made up.

use plumbline_world::editing as ed;
use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::vif::vif;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}

/// A 24 ft by 16 ft house, 9 ft walls, a partition at x = 12: the partition's path.
fn house() -> (Job, String, Vec<String>) {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
    (job, part[0].clone(), outer)
}

#[test]
fn a_job_that_says_nothing_of_a_wall_saves_as_it_always_did() {
    let (mut job, _, _) = house();
    let text = save(&mut job);
    assert!(!text.contains("pl::hidden"), "a job with nothing said of a wall has no pl::hidden, not even its schema");
    let (mut back, _) = load(&text).unwrap();
    assert_eq!(save(&mut back), text, "opened and saved, byte for byte");
    // Marks that ask nothing are no change to the file either.
    let mut marked = back;
    ed::mark(&mut marked, &["wall-0004".to_string()], WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    let text = save(&mut marked);
    assert!(!text.contains("pl::hidden"));
    let (mut again, _) = load(&text).unwrap();
    assert_eq!(save(&mut again), text);
}

#[test]
fn what_is_said_of_a_wall_is_saved_and_read_back_the_same() {
    let (mut job, part, _) = house();
    job.set(&part, "bearing", Some("Not  bearing"), Status::Known, "joists run with it, seen from the attic").unwrap();
    job.set(&part, "in_the_wall", Some("one 2 in  drain"), Status::Assumed, "an old drain stack is usual").unwrap();
    let text = save(&mut job);
    assert!(text.contains("pl::hidden") && text.contains("\"not bearing\"") && text.contains("one 2 in drain"));
    let (mut back, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut back), text);
    // Put back to not known.
    back.set(&part, "bearing", None, Status::Unknown, "not seen again").unwrap();
    let e = back.find(&part).unwrap();
    let h = back.world.get::<Hidden>(e).unwrap();
    assert_eq!(h.bearing.as_ref().unwrap().value, None);
    assert_eq!(h.in_the_wall.as_ref().unwrap().value.as_deref(), Some("one 2 in drain"));
}

#[test]
fn any_other_word_for_bearing_is_refused_and_it_is_said_of_a_wall_only() {
    let (mut job, part, outer) = house();
    let before = save(&mut job);
    let why = job.set(&part, "bearing", Some("bearing wall"), Status::Known, "seen").unwrap_err();
    assert!(why.contains("\"bearing\" or \"not bearing\""), "{why}");
    assert!(job.set(&part, "bearing", Some("bearing"), Status::Known, " ").is_err());
    assert!(job.set(&part, "in_the_wall", Some("   "), Status::Known, "seen").is_err());
    assert!(job.set(&part, "bearing", None, Status::Known, "seen").is_err(), "unknown carrying a status of Known");
    assert_eq!(save(&mut job), before);
    let level = job.find(&part).and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone()).unwrap();
    assert!(job.set(&level, "bearing", Some("bearing"), Status::Known, "seen").is_err());
    let _ = outer;
}

#[test]
fn a_wall_is_asked_only_where_the_work_opens_it() {
    let (mut job, part, outer) = house();
    // Nothing marked: nothing asked, and no row.
    assert!(hidden::opened_walls(&mut job.world).is_empty());
    assert!(vif(&mut job).questions.is_empty());
    let all: Vec<String> = outer.iter().cloned().chain([part.clone()]).collect();
    ed::mark(&mut job, &all, WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    assert!(hidden::opened_walls(&mut job.world).is_empty(), "existing and opened by nothing");
    let said = ed::mark(&mut job, std::slice::from_ref(&part), WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    assert_eq!(said.last().unwrap(), "Is it bearing, and what is in it? Both are on the field list.");
    assert_eq!(hidden::opened_walls(&mut job.world).get(&part), Some(&hidden::Opened::Down));
    let list = vif(&mut job);
    assert_eq!(list.questions.len(), 1);
    let q = &list.questions[0];
    assert_eq!((q.path.as_str(), q.dollars), (part.as_str(), None), "nothing priced on it: no dollars, which is not $0.00");
    assert_eq!(q.to_do(), "Check whether it is bearing and what is in the wall.");
    // A word said of a wall the work does not open is kept, and asks nothing.
    job.set(&outer[0], "bearing", Some("bearing"), Status::Assumed, "guess").unwrap();
    assert_eq!(vif(&mut job).questions.len(), 1);
    // Said Known, the partition leaves the list.
    job.set(&part, "bearing", Some("not bearing"), Status::Known, "seen").unwrap();
    job.set(&part, "in_the_wall", Some("nothing"), Status::Known, "opened").unwrap();
    assert!(vif(&mut job).questions.is_empty());
}
