//! Call 166: one cost line taken off a thing.

use plumbline_world::{file, view, Job};

fn wall_with_two_lines() -> (Job, String) {
    let mut job = plumbline_world::sample::house();
    job.derive();
    let wall = view::elements(&mut job).into_iter().find(|e| e.kind == plumbline_world::Kind::Wall).expect("the sample house has a wall").path;
    job.add_line(&wall, "Made-up line A", "2", None, false).unwrap();
    job.add_line(&wall, "Made-up line B", "3", None, false).unwrap();
    job.derive();
    (job, wall)
}

#[test]
fn one_line_is_taken_off_and_the_rest_of_the_thing_stays() {
    let (mut job, wall) = wall_with_two_lines();
    let before: Vec<String> = view::estimate(&mut job).lines.iter().filter(|l| l.path == wall).map(|l| l.description.clone()).collect();
    assert!(before.contains(&"Made-up line A".to_string()) && before.contains(&"Made-up line B".to_string()));
    let gone = job.remove_line(&wall, "Made-up line A").unwrap();
    assert_eq!(gone.description, "Made-up line A");
    job.derive();
    let after: Vec<String> = view::estimate(&mut job).lines.iter().filter(|l| l.path == wall).map(|l| l.description.clone()).collect();
    let expected: Vec<String> = before.iter().filter(|d| *d != "Made-up line A").cloned().collect();
    assert_eq!(after, expected, "only that line went, and the others kept their order");
    // The job still saves and opens to the same text.
    let text = file::save(&mut job);
    let (mut again, _) = file::load(&text).unwrap();
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn a_line_the_thing_does_not_have_is_refused_and_nothing_changes() {
    let (mut job, wall) = wall_with_two_lines();
    let text = file::save(&mut job);
    let why = job.remove_line(&wall, "No such line").unwrap_err();
    assert!(why.contains("has no cost line \"No such line\"; it has") && why.contains("\"Made-up line B\""), "{why}");
    assert_eq!(file::save(&mut job), text);
}
