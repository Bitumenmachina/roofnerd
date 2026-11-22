//! Call 159: a measure typed outright on a line ("60 SF"), held as a value entered on the
//! line, and the starter assembly "Take down a partition". The house is made up.

use plumbline_world::editing as ed;
use plumbline_world::file::{load, save};
use plumbline_world::units::{self, Len};
use plumbline_world::value::Status;
use plumbline_world::view::estimate;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}

/// A 24 ft by 16 ft house, 9 ft walls, and the path of its first wall.
fn house() -> (Job, String) {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    (job, outer[0].clone())
}

fn amount(job: &mut Job, what: &str) -> Option<f64> {
    estimate(job).lines.iter().find(|l| l.description == what).and_then(|l| l.amount.value)
}

#[test]
fn a_measure_is_a_number_and_one_of_five_units() {
    assert_eq!(units::parse_measure("60 SF").unwrap(), (60.0, "SF"));
    assert_eq!(units::parse_measure(" 2.5 cy ").unwrap(), (2.5, "CY"));
    assert_eq!(units::parse_measure("24LF").unwrap(), (24.0, "LF"));
    assert_eq!(units::parse_measure("3 SQ").unwrap(), (3.0, "SQ"));
    assert_eq!(units::parse_measure("4 EA").unwrap(), (4.0, "EA"));
    for bad in ["60", "60 ft", "60 CF", "SF", "-5 SF", "6 0 SF"] {
        assert!(units::parse_measure(bad).is_err(), "{bad}");
    }
}

#[test]
fn a_typed_measure_is_an_entered_value_priced_and_saved_without_loss() {
    let (mut job, wall) = house();
    // It says how sure it is: not Unknown.
    assert!(job.add_line_measured(&wall, "Patch and match siding", "60 SF", Some(Class::Subcontract), false, Status::Unknown, "walked it").is_err());
    assert!(job.add_line_measured(&wall, "Patch and match siding", "60 SF", Some(Class::Subcontract), false, Status::Assumed, " ").is_err());
    assert!(job.add_line_measured(&wall, "Patch and match siding", "60", Some(Class::Subcontract), false, Status::Assumed, "walked it").is_err());
    assert!(job.find(&wall).and_then(|e| job.world.get::<Cost>(e)).is_none(), "a refusal leaves the wall as it was");
    job.add_line_measured(&wall, "Patch and match siding", "60 SF", Some(Class::Subcontract), false, Status::Assumed, "walked it").unwrap();
    job.set(&wall, "unit_cost", Some("$14/SF"), Status::Known, "made-up price").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, "Patch and match siding"), Some(840.0));
    let e = job.find(&wall).unwrap();
    let line = job.world.get::<Cost>(e).unwrap().lines[0].clone();
    assert_eq!(line.quantity, "measured");
    let held = &line.inputs["measured"];
    assert_eq!((held.value, held.unit.as_str(), held.status, held.source.as_str()), (Some(Len::One(60.0)), "SF", Status::Assumed, "walked it"));
    // Changed, then measured: Known on the tape's word.
    job.set_on_line(&wall, "measured", None, Some("72 SF"), Status::Assumed, "walked it again").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, "Patch and match siding"), Some(1008.0));
    let done = ed::measure_on_line(&mut job, &wall, "Patch and match siding", "measured", ed::Reading::Same, "made-up tape").unwrap();
    assert!(done.said.contains("is 72 SF, found as drawn"), "{}", done.said);
    let e = job.find(&wall).unwrap();
    let held = job.world.get::<Cost>(e).unwrap().lines[0].inputs["measured"].clone();
    assert_eq!(held.status, Status::Known);
    // Saved and read back the same, to the byte.
    let text = save(&mut job);
    let (mut back, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut back), text);
}

#[test]
fn a_measure_is_what_its_unit_says_in_a_formula() {
    let (mut job, wall) = house();
    job.add_line(&wall, "Twice", "1", Some(Class::Subcontract), false).unwrap();
    for (typed, unit, formula, want) in [("72 SF", "SF", "measured * 2", 144.0), ("2 CY", "CY", "measured", 2.0), ("3 SQ", "SF", "measured", 300.0), ("24 LF", "LF", "measured", 24.0), ("4 EA", "EA", "measured", 4.0)] {
        job.set_formula(&wall, Some("Twice"), formula, &[("measured".to_string(), typed.to_string())], Status::Assumed, "made-up walk").unwrap();
        job.derive();
        let figures = estimate(&mut job);
        let line = figures.lines.iter().find(|l| l.description == "Twice").unwrap();
        assert_eq!((line.quantity.value, line.quantity.unit.as_str()), (Some(want), unit), "{typed} in {formula}");
    }
}

#[test]
fn a_file_with_a_measure_in_a_bad_unit_is_refused() {
    let (mut job, wall) = house();
    job.add_line_measured(&wall, "Patch", "60 SF", Some(Class::Subcontract), false, Status::Assumed, "walked it").unwrap();
    let text = save(&mut job);
    assert!(load(&text.replace("\"SF\"", "\"CF\"")).is_err());
}
