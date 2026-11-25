//! Call 209, points 3 and 4 (contract 209M): what a model written out and brought in says, in a builder's words, and what the
//! two refusals tell a person to do. Everything is made up.

use plumbline_ifc::{import, write::export};
use plumbline_world::editing::{self as ed, Entered, FloorBy};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const MADE: Entered = Entered { status: Status::Known, source: "made-up survey" };
const HOW: Entered = Entered { status: Status::Assumed, source: "a made-up model" };

/// The whole made-up house written out: no line says "x", names a command, or uses the file's word for a joint.
#[test]
fn point_3_what_is_written_out_is_said_in_a_builders_words() {
    let mut house = plumbline_world::sample::house_priced();
    let written = export(&mut house, "house.ifc", WHEN).unwrap();
    assert_eq!(written.made.get("joints"), Some(&147));
    assert_eq!(written.made.get("connections"), Some(&332));
    assert!(written.made.keys().all(|k| !k.contains("connection point")), "{:?}", written.made.keys().collect::<Vec<_>>());
    let said = written.not_written_said();
    assert!(said.iter().any(|l| l.starts_with("432 cost lines: the estimate is not part of a building model: it is written out as a workbook")), "{said:#?}");
    assert!(said.iter().any(|l| l.starts_with("31 wall, door, window or run types: each wall, door, window and run is written with its own size")), "{said:#?}");
    for line in &said {
        assert!(!line.contains(" x ") && !line.contains('`') && !line.contains("export estimate"), "{line}");
    }
}

/// What a model says it left out: one line each, "n kind: why", with no word about this program's taking.
#[test]
fn point_3_what_is_left_out_of_a_model_is_said_plainly() {
    let mut house = plumbline_world::sample::house_priced();
    let text = export(&mut house, "house.ifc", WHEN).unwrap().text;
    let mut job = Job::new("Empty (not a real job)", "test", "2026-11-25");
    let b = import(&mut job, text.as_bytes(), "house.ifc", HOW).unwrap();
    let said = b.left_out_said();
    assert_eq!(said.len(), b.left_out.len());
    for line in &said {
        assert!(line.chars().next().is_some_and(|c| c.is_ascii_digit()) && !line.contains(" x "), "{line}");
    }
    let all = format!("{said:#?}\n{:#?}", b.notes);
    for word in ["would not take it", "stands upright", "lists of what the model says", "the very place", "come in with its size"] {
        assert!(!all.contains(word), "{word}: {all}");
    }
    assert!(b.notes.iter().any(|n| n.starts_with("not read: the notes the model carries on its things (")), "{:#?}", b.notes);
    assert!(b.notes.iter().all(|n| !n.contains("0 materials")), "{:#?}", b.notes);
}

fn plain_model(floor: Option<&[[f64; 2]]>) -> String {
    let mut job = Job::new("Another trade (not a real job)", "test", "2026-11-25");
    let l1 = ed::storey(&mut job, "L1", "0 ft", "9 ft", MADE).unwrap();
    let w6 = ed::wall_type(&mut job, "W", "6 in", MADE).unwrap();
    ed::walls(&mut job, &l1, &w6, &[[10.0, 0.0], [10.0, 12.0]], false, ed::Top::Height("9 ft"), MADE).unwrap();
    if let Some(outline) = floor {
        let ft = ed::floor_type(&mut job, "slab", "4 in", MADE).unwrap();
        ed::floor(&mut job, &l1, &ft, FloorBy::Points(outline), MADE).unwrap();
    }
    export(&mut job, "other.ifc", WHEN).unwrap().text
}

/// Point 4: the same model twice is refused, and says what stands in the way and what can be done.
#[test]
fn point_4_a_model_brought_in_twice_says_what_to_do() {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-25");
    ed::storey(&mut job, "first floor", "0 ft", "10 ft", MADE).unwrap();
    import(&mut job, plain_model(None).as_bytes(), "other.ifc", HOW).unwrap();
    let before = file::save(&mut job);
    let why = import(&mut job, plain_model(None).as_bytes(), "other.ifc", HOW).unwrap_err();
    assert!(why.contains("this model is already in the job: nothing was brought in"), "{why}");
    assert!(why.contains("bring it into a job of its own") && why.contains("take what it brought in out of this job first"), "{why}");
    assert!(why.contains("bringing in only what the job lacks is not there yet"), "{why}");
    assert!(!why.contains('`') && !why.contains("very place"), "{why}");
    assert_eq!(file::save(&mut job), before, "nothing was brought in");
}

/// Point 4: a model whose floor lies over a floor the job has is refused whole, and says what to take out or where to bring it.
#[test]
fn point_4_a_floor_over_a_floor_says_what_to_do() {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-25");
    let l = ed::storey(&mut job, "first floor", "0 ft", "9 ft", MADE).unwrap();
    let ft = ed::floor_type(&mut job, "slab", "4 in", MADE).unwrap();
    ed::floor(&mut job, &l, &ft, FloorBy::Points(&[[0.0, 0.0], [20.0, 0.0], [20.0, 12.0], [0.0, 12.0]]), MADE).unwrap();
    let before = file::save(&mut job);
    let model = plain_model(Some(&[[2.0, 2.0], [10.0, 2.0], [10.0, 8.0], [2.0, 8.0]]));
    let why = import(&mut job, model.as_bytes(), "other.ifc", HOW).unwrap_err();
    assert!(why.contains("cover the same ground on one level: two floors in one place would be counted twice"), "{why}");
    assert!(why.contains("Nothing was brought in"), "{why}");
    assert!(why.contains("take out of the job what is already there on that ground (here, the floor), or bring the model into a job of its own"), "{why}");
    assert!(!why.contains("lies in") && !why.contains('`'), "{why}");
    assert_eq!(file::save(&mut job), before, "nothing was brought in");
}
