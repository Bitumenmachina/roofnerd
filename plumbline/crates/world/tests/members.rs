//! Joists and rafters counted across a floor and a roof plane (call 165), from the line that
//! names them. Every figure is worked by hand in `src/building/members.rs` and in STATUS.md
//! ("Joists and rafters counted").

use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::file::{load, save};
use plumbline_world::solve;
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

fn figure(job: &mut Job, path: &str, name: &str) -> Option<Derived> {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0.get(name).cloned()
}

fn floor_job(points: &[[f64; 2]]) -> (Job, String) {
    let mut job = Job::new("Joists (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let ft = ed::floor_type(&mut job, "floor 10 in", "10 in", KNOWN).unwrap();
    let floor = ed::floor(&mut job, &level, &ft, ed::FloorBy::Points(points), KNOWN).unwrap().path;
    (job, floor)
}

#[test]
fn a_floors_joists_are_counted_where_a_line_names_them_and_nowhere_else() {
    let (mut job, floor) = floor_job(&[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]]);
    // No line, no joists; a line that names no lumber and spacing counts none.
    assert!(figure(&mut job, &floor, "joists").is_none());
    job.add_line(&floor, "Rim joist", "rim_length", None, false).unwrap();
    assert!(figure(&mut job, &floor, "joists").is_none());
    // A line that is not in the work names nothing either.
    job.add_line(&floor, "Floor joists, 2x10 at 16 in, by others", "perimeter", None, false).unwrap();
    assert!(figure(&mut job, &floor, "joists").is_none());
    job.add_line(&floor, "Floor joists, 2x10 at 16 in", "joists", None, false).unwrap();
    job.derive();
    // 24 / 1.3333 + 1 = 19 joists of 16 ft: 304 LF; the rim 2 x 24 = 48 LF. Assumed: the way
    // they run is not said, and the figure says so.
    let joists = figure(&mut job, &floor, "joists").unwrap();
    assert_eq!((joists.value, joists.unit.as_str(), joists.status), (Some(19.0), "EA", Status::Assumed));
    assert!(joists.note.contains("joists usually span the short way: say which way these run"), "{}", joists.note);
    assert!(joists.note.contains("blocking, hangers and doubled members are not in this count"), "{}", joists.note);
    assert_eq!(figure(&mut job, &floor, "joist_length").unwrap().value, Some(304.0));
    assert_eq!(figure(&mut job, &floor, "joist_longest").unwrap().value, Some(16.0));
    assert_eq!(figure(&mut job, &floor, "rim_length").unwrap().value, Some(48.0));
    // Said the long way, Known: 13 joists of 24 ft, 312 LF; the rim 2 x 16 = 32 LF.
    job.set_on_line(&floor, "joists_span", None, Some("the long way"), Status::Known, "the framing sheet, made up").unwrap();
    job.derive();
    let joists = figure(&mut job, &floor, "joists").unwrap();
    assert_eq!((joists.value, joists.status), (Some(13.0), Status::Known));
    assert_eq!(figure(&mut job, &floor, "joist_length").unwrap().value, Some(312.0));
    assert_eq!(figure(&mut job, &floor, "rim_length").unwrap().value, Some(32.0));
    // Only the two ways are words for it.
    assert!(job.set_on_line(&floor, "joists_span", None, Some("diagonally"), Status::Known, "x").unwrap_err().contains("is not a way for joists to run"));
}

#[test]
fn an_l_shaped_floor_has_20_joists_and_a_floor_with_a_line_to_save_comes_back_the_same() {
    let (mut job, floor) = floor_job(&[[0.0, 0.0], [24.0, 0.0], [24.0, 10.0], [12.0, 10.0], [12.0, 16.0], [0.0, 16.0]]);
    job.add_line(&floor, "Floor joists, 2x10 at 16 in", "joists", None, false).unwrap();
    job.derive();
    assert_eq!(figure(&mut job, &floor, "joists").unwrap().value, Some(20.0));
    assert_eq!(figure(&mut job, &floor, "joist_length").unwrap().value, Some(250.0));
    assert_eq!(figure(&mut job, &floor, "rim_length").unwrap().value, Some(48.0));
    job.set_on_line(&floor, "joists_span", None, Some("short way"), Status::Assumed, "usual").unwrap();
    job.derive();
    let text = save(&mut job);
    assert!(text.contains("pl::joists_span"));
    let (mut back, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut back), text);
}

fn roof(job_pitch: &str) -> (Job, String) {
    let mut job = Job::new("Rafters (not a real job)", "test", "2026-11-22");
    let _ = job_pitch;
    let footprint = Footprint {
        edges: (0..4).map(|_| Val { value: Some(Rise::Pitch(6.0)), unit: "in/12".into(), status: Status::Known, source: "made-up plan".into() }).collect(),
        points: Val::known(vec![[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], "ft", "made-up plan"),
        eave_height: Val::known(Len::One(9.0), "ft", "made-up plan"),
    };
    let (roof, _) = solve::new_roof(&mut job, "Main roof", footprint).unwrap();
    (job, roof)
}

#[test]
fn a_hip_roof_counts_36_rafters_and_a_gable_roof_26_where_a_line_of_the_roof_names_them() {
    let (mut job, roof) = roof("6:12");
    let planes = |job: &mut Job| -> Vec<String> {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let mut v: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::RoofPlane).map(|(n, _)| n.path.clone()).collect();
        v.sort();
        v
    };
    // Nothing named, nothing counted.
    assert!(figure(&mut job, &roof, "rafters").is_none());
    job.add_line(&roof, "Rafters, 2x8 at 24 in", "rafters", None, false).unwrap();
    job.derive();
    // Two long planes of 11 (63.9375 ft on plan, 71.4843 LF) and two short of 7 (31.9375 ft,
    // 35.7072 LF): 36 rafters, 214.3837 LF.
    let r = figure(&mut job, &roof, "rafters").unwrap();
    assert_eq!((r.value, r.unit.as_str()), (Some(36.0), "EA"));
    assert!((figure(&mut job, &roof, "rafter_length").unwrap().value.unwrap() - 191.75 * 1.118033988749895).abs() < 1.0e-3);
    let mut counts = Vec::new();
    for p in planes(&mut job) {
        counts.push(figure(&mut job, &p, "rafters").unwrap().value.unwrap() as u32);
    }
    counts.sort();
    assert_eq!(counts, vec![7, 7, 11, 11]);
    // The long plane's sliver at the corner is left out, and said.
    let long = planes(&mut job).into_iter().find(|p| figure(&mut job, p, "rafters").unwrap().value == Some(11.0)).unwrap();
    assert!(figure(&mut job, &long, "rafters").unwrap().note.contains("1 piece under 6 in left out"));
    // The two short edges made gables: two planes of 13 rafters of 8 ft on plan, 8.944272 ft along the slope.
    for edge in [2, 4] {
        solve::edit(&mut job, &roof, solve::Change::Rise { edge, rise: Val { value: Some(Rise::Gable), unit: String::new(), status: Status::Known, source: "made-up plan".into() } }).unwrap();
    }
    job.derive();
    assert_eq!(figure(&mut job, &roof, "rafters").unwrap().value, Some(26.0));
    assert!((figure(&mut job, &roof, "rafter_length").unwrap().value.unwrap() - 2.0 * 13.0 * 8.0 * 1.118033988749895).abs() < 1.0e-3);
    for p in planes(&mut job) {
        assert_eq!(figure(&mut job, &p, "rafters").unwrap().value, Some(13.0));
        assert!((figure(&mut job, &p, "rafter_longest").unwrap().value.unwrap() - 8.944272).abs() < 1.0e-4);
    }
}

#[test]
fn a_job_with_no_line_that_names_joists_or_rafters_saves_as_it_did() {
    let mut old = plumbline_world::sample::house();
    let text = save(&mut old);
    assert!(!text.contains("pl::joists_span"));
    let (mut again, _) = load(&text).unwrap();
    assert_eq!(save(&mut again), text);
}
