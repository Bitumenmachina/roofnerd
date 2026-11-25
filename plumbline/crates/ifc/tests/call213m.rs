//! Call 213, points 10 (the words of a refusal), 11 and 12 (contract 213M): a model brought in at a height the job has not got says what
//! it did of the things, and says so where a thing stands above the job's top or below its bottom; a model already in the job is
//! said in the words a reply uses everywhere. Everything is made up; the figures are worked by hand in STATUS.md.
//! The sample whole house is the job: its first floor at 0 ft and its second floor at 10 ft, each 9 ft high, so its top is at 19 ft.

use plumbline_ifc::{import, import_choosing, write::export, Bring};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const MADE: Entered = Entered { status: Status::Assumed, source: "made-up designer" };
const HOW: Entered = Entered { status: Status::Assumed, source: "the designer's model, made up" };

/// A designer's model: one level at `floor` and a 12 in round duct 10 ft long, 3 ft above it.
fn designer(floor: &str) -> String {
    let mut job = Job::new("Designer (not a real job)", "test", "2026-11-25");
    let l = ed::storey(&mut job, "Mechanical", floor, "5 ft", MADE).unwrap();
    let t = ed::segment_type(&mut job, "Duct round 12", Kind::Duct, "12 in", None, MADE).unwrap();
    ed::runs(&mut job, &l, Kind::Duct, &t, &[[5.0, 5.0], [15.0, 5.0]], "3 ft", None, MADE).unwrap();
    export(&mut job, "designer.ifc", WHEN).unwrap().text
}

fn notes_of(floor: &str) -> String {
    let mut job = sample::house_priced();
    import_choosing(&mut job, designer(floor).as_bytes(), "designer.ifc", HOW, Bring::RunsOnly).unwrap().notes.join("\n")
}

/// F13, the demonstration's step 10: a level at 25 ft and a duct 3 ft above it is 28 ft up, 9 ft above the top of the second floor (10 ft
/// and 9 ft high: 19 ft). The note speaks of the things, and the reply flags the height and says to check the model's heights.
#[test]
fn point_11_a_thing_above_the_top_of_the_highest_level_is_flagged() {
    let notes = notes_of("25 ft");
    assert!(notes.contains("each was put on the level it stands within"), "{notes}");
    assert!(!notes.contains("the level it stands within, and no level was added"), "{notes}");
    assert!(notes.contains("stands above the top of the job's highest level"), "{notes}");
    assert!(notes.contains("\"second floor\"") && notes.contains("19 ft") && notes.contains("9 ft above"), "{notes}");
    assert!(notes.contains("check the model's heights against the job's"), "{notes}");
}

/// The 9.5 ft case: the level is inside the job's first floor and the duct 3 ft above it is at 12.5 ft, on the second floor, within it: no flag.
#[test]
fn point_11_a_thing_within_the_levels_is_not_flagged() {
    let notes = notes_of("9.5 ft");
    assert!(notes.contains("each was put on the level it stands within"), "{notes}");
    assert!(!notes.contains("stands above the top") && !notes.contains("stand above the top") && !notes.contains("below the job's lowest level"), "{notes}");
}

/// Below the lowest level: -10 ft, the duct 3 ft above it, is at -7 ft: 3 ft below the foundation's floor (-4 ft, the job's lowest).
#[test]
fn point_11_a_thing_below_the_lowest_level_is_flagged_and_said_with_its_grammar() {
    let notes = notes_of("-10 ft");
    assert!(notes.contains("stands below the job's lowest level") && notes.contains("\"foundation\"") && notes.contains("3 ft below"), "{notes}");
    assert!(notes.contains("check the model's heights against the job's"), "{notes}");
    assert!(!notes.contains("of them stand below"), "{notes}");
}

/// A model already in the job: each kind with its plural, one word for a kind everywhere, no "other thing" for a piece of equipment.
#[test]
fn point_10_a_model_already_in_is_said_with_plurals_and_one_word_for_a_thing() {
    let mut job = sample::house_priced();
    let text = export(&mut job, "house.ifc", WHEN).unwrap().text;
    let why = import(&mut job, text.as_bytes(), "house.ifc", HOW).unwrap_err();
    assert!(why.contains("this model is already in the job"), "{why}");
    assert!(why.contains(" walls") && why.contains(" ducts") && why.contains(" pipes") && why.contains(" fittings"), "{why}");
    for word in ["fixture or outlet", "plumbing fixture", "other thing", " 20 wall,", " 20 wall)"] {
        assert!(!why.contains(word), "{word}: {why}");
    }
    assert!(why.contains(" fixtures") && why.contains("pieces of equipment"), "{why}");
}
