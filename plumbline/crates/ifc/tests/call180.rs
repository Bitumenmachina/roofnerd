//! Call 180, part C3: the BIM file read back. A level's height comes back what it was,
//! everything read from a model is Assumed, a wall is never dropped for its type's name,
//! and a short stub of wall comes back as drawn. Everything here is made up.

use plumbline_ifc::{import, write::export};
use plumbline_world::building::{Level, Wall};
use plumbline_world::editing::{self as ed, Entered, Top};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const KNOWN: Entered = Entered { status: Status::Known, source: "made-up survey" };

fn heights(job: &mut Job) -> Vec<(String, Option<f64>, Status)> {
    let mut q = job.world.query::<(&Node, &Level)>();
    let mut v: Vec<(f64, (String, Option<f64>, Status))> = q.iter(&job.world).map(|(n, l)| (l.elevation.feet().unwrap_or(0.0), (n.name.clone(), l.height.feet(), l.height.status))).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v.into_iter().map(|x| x.1).collect()
}

/// By hand: a basement 8 ft 4 in high (8.3333 ft), a main floor 9 ft 1 1/8 in high
/// (9 + 1/8 / 12 + 1/12 = 9.09375 ft) with its floor at 0, and an upper floor at 9 ft 7 1/2 in
/// (9.625 ft) 8 ft high. The distance from the main floor to the upper is 9.625 ft, which is
/// 6 3/8 in (0.53125 ft) more than the main floor's height: that is what came back before.
#[test]
fn a_level_comes_back_as_high_as_it_was_and_a_model_read_in_is_assumed() {
    let mut job = Job::new("levels (not a real job)", "test", "test");
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    for (name, at, high) in [("basement", "-8 ft 4 in", "8 ft 4 in"), ("main floor", "0 ft", "9 ft 1 1/8 in"), ("upper floor", "9 ft 7 1/2 in", "8 ft")] {
        let level = ed::storey(&mut job, name, at, high, KNOWN).unwrap();
        ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    }
    job.derive();
    let written = export(&mut job, "levels.ifc", WHEN).unwrap();
    let mut back = Job::new("read back (not a real job)", "test", "test");
    import(&mut back, written.text.as_bytes(), "levels.ifc", KNOWN).unwrap();
    let got = heights(&mut back);
    let want = [("basement", 8.0 + 4.0 / 12.0), ("main floor", 9.09375), ("upper floor", 8.0)];
    assert_eq!(got.len(), 3, "{got:?}");
    for ((name, height, status), (wname, wheight)) in got.iter().zip(want) {
        assert_eq!(name, wname);
        assert!((height.expect(name) - wheight).abs() < 1.0e-6, "{name}: {height:?}, by hand {wheight}");
        assert_eq!(*status, Status::Assumed, "{name}: read from a model, it is Assumed");
    }
}

/// The same, on a model with walls: nothing from a model is Known, whatever was said.
#[test]
fn nothing_read_from_a_model_is_known_whatever_was_said() {
    let mut job = Job::new("walls (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]], true, Top::Height("9 ft"), KNOWN).unwrap();
    job.derive();
    let written = export(&mut job, "w.ifc", WHEN).unwrap();
    let mut back = Job::new("read back (not a real job)", "test", "test");
    let brought = import(&mut back, written.text.as_bytes(), "w.ifc", KNOWN).unwrap();
    back.derive();
    let totals = plumbline_world::building::storey_totals(&mut back);
    assert_eq!(totals[0].walls.status, Status::Assumed);
    assert!(brought.notes.iter().any(|n| n.contains("Assumed") && n.contains("Known")), "{:#?}", brought.notes);
}

/// By hand: a stub 4 in long and 6 in thick at (10, 10) and a wall 20 ft long. Both come back; the
/// stub is 4 in (0.3333 ft) long and 6 in thick: 0.3333 x 9 = 3.00 SF of face; the wall is
/// 20 x 9 = 180.00 SF.
#[test]
fn a_four_inch_stub_and_a_wall_both_come_back_as_drawn() {
    let mut job = Job::new("stub (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    ed::walls(&mut job, &level, &wt, &[[10.0, 10.0], [10.0 + 1.0 / 3.0, 10.0]], false, Top::Height("9 ft"), KNOWN).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, Top::Height("9 ft"), KNOWN).unwrap();
    job.derive();
    let written = export(&mut job, "stub.ifc", WHEN).unwrap();
    let mut back = Job::new("read back (not a real job)", "test", "test");
    let brought = import(&mut back, written.text.as_bytes(), "stub.ifc", KNOWN).unwrap();
    back.derive();
    assert!(!brought.notes.iter().any(|n| n.contains("would not take it")), "{:#?}", brought.notes);
    let mut q = back.world.query::<&Wall>();
    let mut lines: Vec<f64> = q.iter(&back.world).map(|w| { let [a, b] = w.line.value.unwrap(); (b[0] - a[0]).hypot(b[1] - a[1]) }).collect();
    lines.sort_by(|a, b| a.total_cmp(b));
    assert_eq!(lines.len(), 2, "a wall was dropped: {lines:?}");
    assert!((lines[0] - 1.0 / 3.0).abs() < 1.0e-3 && (lines[1] - 20.0).abs() < 1.0e-3, "{lines:?}");
    let totals = plumbline_world::building::storey_totals(&mut back);
    let face = totals[0].wall_face.value.unwrap();
    assert!((face - 183.0).abs() < 0.01, "face {face}, by hand 180 + 3");
}

/// The other road to the same loss: a designer's file that calls a 4 in wall's type "wall 6 in"
/// and then a 6 in wall's. Neither wall may be dropped for the name.
#[test]
fn a_type_name_the_job_has_at_another_thickness_drops_no_wall() {
    let mut job = Job::new("names (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, Top::Height("9 ft"), KNOWN).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 5.0], [20.0, 5.0]], false, Top::Height("9 ft"), KNOWN).unwrap();
    job.derive();
    let written = export(&mut job, "names.ifc", WHEN).unwrap();
    // Make the second wall's type name be the first's while it is 4 in thick in the body.
    let text = written.text.clone();
    let mut other = Job::new("read back (not a real job)", "test", "test");
    // The job already has a "wall 6 in" type of 6 in; a model's 4 in wall called "wall 6 in".
    let wt4 = ed::wall_type(&mut other, "wall 6 in", "4 in", KNOWN).unwrap();
    let _ = wt4;
    let brought = import(&mut other, text.as_bytes(), "names.ifc", KNOWN).unwrap();
    other.derive();
    let mut q = other.world.query::<&Wall>();
    assert_eq!(q.iter(&other.world).count(), 2, "{:#?}", brought.notes);
}
