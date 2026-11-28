use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{measure, edit, attrs, figures, child, hip_roof, measure_of, drawn, clashes_of, addition_layout};

// ---------------------------------------------------------------- the fourteenth judgment

#[test]
fn j14_2_3_a_scale_set_again_says_what_it_left_and_keeps_a_drawing_lined_up() {
    use plumbline_world::editing as ed;
    use plumbline_world::plans;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let read = ed::Entered { status: Status::Assumed, source: "read off the drawing" };
    let (mut job, s, wt, ..) = drawn();
    let d = plans::add(&mut job, &s, "A-1", plans::Read { file: "a-1.pdf", page: 1, bytes: 10, checksum: "0123456789abcdef", size: [792.0, 612.0] }, how).unwrap();
    // A dimension misread as 15 ft over 540 points: 36 to the foot. The house's corner,
    // (126, 126) on the page, lined up to the plan's (0, 0): the sheet's own corner is
    // then at -126 / 36 = -3.5 each way.
    let first = plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "15 ft", read).unwrap();
    assert!(first.left_at_earlier_scale.is_empty() && !first.lined_up_again);
    plans::line_up(&mut job, &d, [126.0, 126.0], [0.0, 0.0], how).unwrap();
    let drawing = |job: &mut Job| job.find(&d).and_then(|e| job.world.get::<plans::Drawing>(e).cloned()).unwrap();
    assert_eq!(drawing(&mut job).at.value, Some([-3.5, -3.5]));
    // A wall traced along the outline's south side, 540 points: 15 ft at that scale.
    let source = plans::traced_from(&mut job, &d).unwrap();
    let traced = ed::Entered { status: Status::Assumed, source: &source };
    let (a, b) = (drawing(&mut job).to_plan([126.0, 126.0]).unwrap(), drawing(&mut job).to_plan([666.0, 126.0]).unwrap());
    let wall = ed::walls(&mut job, &s, &wt, &[a, b], false, ed::Top::Height("8 ft"), traced).unwrap();
    assert_eq!(measure_of(&mut job, &wall[0], "plan_length").value, Some(15.0));
    assert_eq!(plans::traced_over(&mut job, &d).unwrap(), wall);
    // The dimension read again, rightly, as 30 ft: 18 to the foot. The wall is where it
    // was, 15 ft long, and is named as left at the earlier scale; the corner the drawing
    // was lined up by is still the plan's (0, 0), so the sheet's own corner is at -7.
    let again = plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", read).unwrap();
    assert!((again.feet_per_unit - 1.0 / 18.0).abs() < 1.0e-12);
    assert_eq!((again.left_at_earlier_scale.clone(), again.lined_up_again), (wall.clone(), true));
    assert_eq!(drawing(&mut job).at.value, Some([-7.0, -7.0]));
    let corner = drawing(&mut job).to_plan([126.0, 126.0]).unwrap();
    assert!(corner[0].abs() < 1.0e-9 && corner[1].abs() < 1.0e-9, "{corner:?}");
    assert_eq!(measure_of(&mut job, &wall[0], "plan_length").value, Some(15.0));
    // It stays said: by the drawing, after a save and a load, until the wall is gone.
    let (mut back, _) = load(&save(&mut job)).unwrap();
    let earlier = plans::at_earlier_scales(&mut back, &d).unwrap();
    assert_eq!(earlier.len(), 1);
    assert!((earlier[0].0 - 1.0 / 36.0).abs() < 1.0e-12 && earlier[0].1 == wall);
    ed::remove(&mut back, &wall[0]).unwrap();
    assert!(plans::at_earlier_scales(&mut back, &d).unwrap().is_empty());
    // The same scale given again leaves nothing behind: nothing changed.
    let same = plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", read).unwrap();
    assert!(same.left_at_earlier_scale.is_empty());
    // Two points a thousandth of a unit apart, or off the page, are no scale.
    let before = save(&mut job);
    assert!(plans::calibrate(&mut job, &d, [126.0, 90.0], [126.001, 90.0], "30 ft", read).unwrap_err().contains("its two points are one point"));
    assert!(plans::calibrate(&mut job, &d, [-5000.0, 90.0], [666.0, 90.0], "30 ft", read).unwrap_err().contains("not both on the page"));
    assert_eq!(save(&mut job), before);
}

#[test]
fn j14_5_6_7_walls_at_an_angle_are_walls_and_a_lateral_is_a_joint() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    // Walls crossing at 25, 20 and 10 degrees, and one meeting another's end at 10: each
    // is a wall. (Tangents: 0.4663, 0.3640, 0.1763.)
    let (mut job, s, wt, ..) = drawn();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [40.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    for (x, tangent) in [(5.0, 0.4663), (15.0, 0.3640), (25.0, 0.1763)] {
        ed::walls(&mut job, &s, &wt, &[[x - 4.0, -4.0 * tangent], [x + 4.0, 4.0 * tangent]], false, ed::Top::Height("9 ft"), how).unwrap();
    }
    ed::walls(&mut job, &s, &wt, &[[40.0, 0.0], [50.0, 1.763]], false, ed::Top::Height("9 ft"), how).unwrap();
    // One a degree off the first, crossing it along its length, is the first drawn again.
    let before = save(&mut job);
    assert!(ed::walls(&mut job, &s, &wt, &[[0.0, -0.1], [40.0, 0.6]], false, ed::Top::Height("9 ft"), how).unwrap_err().contains("lies along"));
    assert_eq!(save(&mut job), before);

    // Two walls an inch apart for 20 ft, of a type whose thickness nobody knows: the
    // second is refused all the same.
    let (mut job, s, ..) = drawn();
    let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).unwrap();
    let unsized_type = job.add(&root, Kind::Type, "wall, thickness not known").unwrap();
    job.put(&unsized_type, WallType { thickness: Val::unknown("in", "not on the drawings"), layers: Vec::new(), footing: false }).unwrap();
    ed::walls(&mut job, &s, &unsized_type, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    assert!(ed::walls(&mut job, &s, &unsized_type, &[[0.0, 1.0 / 12.0], [20.0, 1.0 / 12.0]], false, ed::Top::Height("9 ft"), how).unwrap_err().contains("lies along"));

    // A ground-floor wall held up to the roof's level, 16 ft up, and a wall on the floor
    // above on the same line: the upper one stands inside the lower. Refused.
    let (mut job, s, wt, ..) = drawn();
    let upper = ed::storey(&mut job, "upper", "8 ft", "8 ft", how).unwrap();
    let roof = ed::storey(&mut job, "roof", "16 ft", "1 ft", how).unwrap();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Level(&roof), how).unwrap();
    assert!(ed::walls(&mut job, &upper, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("8 ft"), how).unwrap_err().contains("lies along"));
    // One on the floor above over a wall that stops at that floor is two walls.
    let (mut job, s, wt, ..) = drawn();
    let upper = ed::storey(&mut job, "upper", "9 ft", "8 ft", how).unwrap();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Level(&upper), how).unwrap();
    ed::walls(&mut job, &upper, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("8 ft"), how).unwrap();

    // Two 4 in pipes, the branch leaving the main's side at 30 degrees: a lateral. The
    // branch is inside the main for (2 + 2) in / sin 30 = 8 in, and that is the joint.
    let (mut job, s, ..) = drawn();
    let pipe = ed::segment_type(&mut job, "4 in", Kind::Pipe, "4 in", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 0.0], [20.0, 0.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[10.0, 0.0], [10.0 + 6.0 * 0.866_025_4, 3.0]], "8 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty(), "{:?}", clashes_of(&mut job, None));
    ed::fitting(&mut job, &s, [10.0, 0.0], None, None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty(), "{:?}", clashes_of(&mut job, None));
    // A branch only 5 degrees off the main, 10 ft of it, lies along the main: a clash.
    let (mut job, s, ..) = drawn();
    let pipe = ed::segment_type(&mut job, "4 in", Kind::Pipe, "4 in", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 0.0], [20.0, 0.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[10.0, 0.0], [10.0 + 10.0 * 0.996_194_7, 0.871_557]], "8 ft", None, how).unwrap();
    assert_eq!(clashes_of(&mut job, None).len(), 1);
}

#[test]
fn a_formula_can_be_typed_as_its_words() {
    use plumbline_world::formula::{names, parse};
    // Words in a row are one name: `plan length` is `plan_length`.
    let typed = parse("plan length * stretch out").unwrap();
    assert_eq!(names(&typed), ["plan_length", "stretch_out"]);
    assert_eq!(names(&parse("plan_length*stretch_out").unwrap()), ["plan_length", "stretch_out"]);
    assert_eq!(names(&parse("( face area - opening area ) * 2").unwrap()), ["face_area", "opening_area"]);
    // A number and a name side by side are still two things with nothing between them.
    assert!(parse("2 height").is_err());
    assert!(parse("height 2").is_err());
}

// ---- The fifteenth judgment, part A: a sheet moved or rescaled after things were traced.

#[test]
fn j15_4_15_a_sheet_lined_up_again_says_what_it_left_behind_and_stops_saying_it_when_it_is_back() {
    use plumbline_world::editing as ed;
    use plumbline_world::plans;
    let how = ed::Entered { status: Status::Assumed, source: "made-up sheet" };
    let read = ed::Entered { status: Status::Assumed, source: "read off the drawing" };
    let mut job = Job::new("Traced (not a real job)", "test", "test");
    let s = ed::storey(&mut job, "ground", "0 ft", "9 ft", how).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", how).unwrap();
    let d = plans::add(&mut job, &s, "A-1", plans::Read { file: "a-1.pdf", page: 1, bytes: 10, checksum: "0123456789abcdef", size: [792.0, 612.0] }, how).unwrap();
    // 540 points are 30 ft: 18 to the foot. The house's corner, (126, 126), at (0, 0).
    plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", read).unwrap();
    assert!(plans::line_up(&mut job, &d, [126.0, 126.0], [0.0, 0.0], how).unwrap().is_empty());
    let drawing = |job: &mut Job| job.find(&d).and_then(|e| job.world.get::<plans::Drawing>(e).cloned()).unwrap();
    let trace = |job: &mut Job| {
        let source = plans::traced_from(job, &d).unwrap();
        let (a, b) = (drawing(job).to_plan([126.0, 126.0]).unwrap(), drawing(job).to_plan([666.0, 126.0]).unwrap());
        ed::walls(job, &s, &wt, &[a, b], false, ed::Top::Height("8 ft"), ed::Entered { status: Status::Assumed, source: &source })
    };
    let wall = trace(&mut job).unwrap();
    assert!(plans::stale(&mut job, &d).unwrap().is_empty());
    // Lined up again, 10 ft east and 5 ft north: the sheet moves, the wall does not, and
    // that is what comes back.
    let left = plans::line_up(&mut job, &d, [126.0, 126.0], [10.0, 5.0], how).unwrap();
    assert_eq!(left, wall);
    let stale = plans::stale(&mut job, &d).unwrap();
    assert_eq!(stale.len(), 1);
    assert_eq!((stale[0].rescaled, stale[0].moved, stale[0].traced.clone()), (false, true, wall.clone()));
    // It is said among the drawing's own values, flagged, by the wall's name.
    let rows = view::elements(&mut job).into_iter().find(|e| e.path == d).unwrap().rows;
    let row = rows.iter().find(|r| r.label == "traced before it was rescaled or moved").expect("the drawing says what was left behind");
    assert_eq!((row.text.as_str(), row.status), ("1 thing: wall 1", Status::Assumed));
    // After a save and a load too.
    let (mut back, _) = load(&save(&mut job)).unwrap();
    assert_eq!(plans::stale(&mut back, &d).unwrap(), stale);
    // The same printed line traced again is a second wall, 10 ft east and 5 ft north of
    // the first: not refused (it does not lie on the first), and the first is still listed.
    let second = trace(&mut job).unwrap();
    assert_ne!(second, wall);
    assert_eq!(plans::stale(&mut job, &d).unwrap()[0].traced, wall);
    // The sheet put back where it was: the first wall is where the sheet shows it again
    // and is no longer listed; the second, traced while the sheet lay elsewhere, now is.
    let left = plans::line_up(&mut job, &d, [126.0, 126.0], [0.0, 0.0], how).unwrap();
    assert_eq!(left, second);
    let stale = plans::stale(&mut job, &d).unwrap();
    assert_eq!(stale.iter().flat_map(|s| s.traced.clone()).collect::<Vec<_>>(), second);
    // A scale changed and set back (J15-15): nothing stays listed for it.
    ed::remove(&mut job, &second[0]).unwrap();
    assert!(plans::stale(&mut job, &d).unwrap().is_empty());
    let changed = plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "15 ft", read).unwrap();
    assert_eq!(changed.left_at_earlier_scale, wall);
    assert_eq!(plans::stale(&mut job, &d).unwrap().len(), 1);
    plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", read).unwrap();
    assert!(plans::stale(&mut job, &d).unwrap().is_empty(), "{:?}", plans::stale(&mut job, &d));
    assert!(drawing(&mut job).earlier.is_empty());
}

#[test]
fn a_size_is_written_in_inches_as_the_trade_writes_it() {
    use plumbline_world::units::size_said;
    assert_eq!(size_said([16.0 / 12.0, 16.0 / 12.0, 8.0 / 12.0]), "16 × 16 × 8 in");
    assert_eq!(size_said([2.0, 2.5, 5.0]), "24 × 30 × 60 in");
    assert_eq!(size_said([1.75 / 12.0, 0.5 / 12.0, 30.5 / 12.0]), "1 3/4 × 1/2 × 30 1/2 in");
    // A side of 8 ft or more: feet and inches, as a tape reads.
    assert_eq!(size_said([10.0, 2.0, 7.0]), "10' 0\" × 2' 0\" × 7' 0\"");
}

#[test]
fn a_quantity_is_written_as_a_takeoff_sheet_writes_it() {
    use formula::{names, parse};
    // "face_area x 0.10" is times (call 124): the same reading as with the star, and
    // the one name it uses is the area. A thing named x on its own is still a name.
    assert_eq!(format!("{:?}", parse("face_area x 0.10").unwrap()), format!("{:?}", parse("face_area * 0.10").unwrap()));
    assert_eq!(names(&parse("face_area x 0.10").unwrap()), vec!["face_area".to_string()]);
    assert_eq!(format!("{:?}", parse("2 X 3").unwrap()), format!("{:?}", parse("2 * 3").unwrap()));
    assert_eq!(names(&parse("x").unwrap()), vec!["x".to_string()]);
    // Priced each: one.
    assert_eq!(format!("{:?}", parse("each").unwrap()), format!("{:?}", parse("1").unwrap()));
    assert_eq!(format!("{:?}", parse("EA").unwrap()), format!("{:?}", parse("1").unwrap()));
    // Two figures side by side say what goes between them.
    assert!(parse("face_area 0.10").unwrap_err().ends_with("put +, -, x or / between them"));
}

/// Call 139: a low-slope area's drains and its tapered insulation, laid out by roofnerd's
/// solver on NRCA's four-way layout. Fig. 4-11's square roof, worked by hand in STATUS.md.
#[test]
fn a_low_slope_areas_taper_is_laid_out_to_its_drains() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Assumed, source: "made-up roof plan" };
    let mut job = Job::new("Flat roof", "test", "2026-11-19");
    let area = ed::low_slope(&mut job, "Main roof", &[[0.0, 0.0], [48.0, 0.0], [48.0, 48.0], [0.0, 48.0]], "1/4:12", "11 ft", how()).unwrap();
    job.derive();
    // No drains: no taper, and nothing flagged for it.
    for name in plumbline_world::taper::FIGURES {
        assert!(job.find(&area).and_then(|e| job.world.get::<Measure>(e)).is_some_and(|m| !m.0.contains_key(name)), "{name}");
    }
    ed::drains(&mut job, &area, &[[24.0, 24.0]], "1/2 in", how()).unwrap();
    job.derive();
    let got = |job: &mut Job, n: &str| measure(job, &area, n).value;
    assert_eq!(got(&mut job, "drains"), Some(1.0));
    assert_eq!(got(&mut job, "thinnest"), Some(0.5));
    assert_eq!(got(&mut job, "thickest"), Some(6.5));
    assert_eq!(got(&mut job, "average_thickness"), Some(4.5));
    assert_eq!(got(&mut job, "board_feet"), Some(10368.0));
    assert_eq!(got(&mut job, "valley_length"), Some(135.7645));
    assert_eq!(measure(&mut job, &area, "board_feet").status, Status::Assumed);
    // Two drains: two regions 24 by 48.
    ed::drains(&mut job, &area, &[[12.0, 24.0], [36.0, 24.0]], "1/2 in", how()).unwrap();
    job.derive();
    assert_eq!(got(&mut job, "average_thickness"), Some(3.75));
    assert_eq!(got(&mut job, "board_feet"), Some(8640.0));
    // A drain off the area is refused and the job keeps what it had.
    let before = save(&mut job);
    let why = ed::drains(&mut job, &area, &[[60.0, 24.0]], "1/2 in", how()).unwrap_err();
    assert!(why.contains("is not on"), "{why}");
    assert_eq!(save(&mut job), before);
    // Read back: the same figures.
    let (mut loaded, _) = load(&before).unwrap();
    assert_eq!(measure(&mut loaded, &area, "board_feet").value, Some(8640.0));
    // A drain moved off the area by hand in the file: refused on the figures, not silent.
    let moved = edit(&before, |v| attrs(v, &area)["pl::taper"]["drains"]["value"][1] = serde_json::json!([70.0, 24.0]));
    let (mut loaded, _) = load(&moved).unwrap();
    let bf = measure(&mut loaded, &area, "board_feet");
    assert_eq!((bf.value, bf.status), (None, Status::Unknown));
    assert!(bf.note.contains("the drain at 70,24 is not on the area"), "{bf:?}");
    // The record on a roof plane is refused when the file is read.
    let (mut hip, roof) = hip_roof();
    let plane = child(&mut hip, &roof, "plane 1");
    let text = save(&mut hip);
    let put = edit(&text, |v| attrs(v, &plane)["pl::taper"] = serde_json::json!({"drains": {"value": [[5.0, 5.0]], "unit": "ft", "status": "Known", "source": "x"}, "at_drain": {"value": 0.5, "unit": "in", "status": "Known", "source": "x"}}));
    let why = load(&put).err().expect("refused");
    assert!(why.contains("only a low-slope area can"), "{why}");
}

/// Judged 2026-11-19 (call 140): eight drains on a 140 by 80 field gave 50,587.5 BF in
/// the order typed and 49,469.5 or 48,682 in others; one region's faces lost 45 SF when
/// a cut met a corner the solver had left as two. The order the drains are typed in
/// changes nothing, and a drain typed twice is refused.
#[test]
fn a_taper_does_not_hang_on_the_order_its_drains_are_typed_in() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Known, source: "made-up roof plan" };
    let bf = |ring: &[[f64; 2]], drains: &[[f64; 2]]| -> Derived {
        let mut job = Job::new("Flat roof", "test", "2026-11-19");
        let area = ed::low_slope(&mut job, "Main roof", ring, "1/4:12", "11 ft", how()).unwrap();
        ed::drains(&mut job, &area, drains, "1 in", how()).unwrap();
        job.derive();
        measure(&mut job, &area, "board_feet")
    };
    // A shuffle that is the same every run.
    let shuffled = |pts: &[[f64; 2]], seed: u64| -> Vec<[f64; 2]> {
        let mut v = pts.to_vec();
        let mut x = seed;
        for i in (1..v.len()).rev() {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            v.swap(i, (x >> 33) as usize % (i + 1));
        }
        v
    };
    let field = [[0.0, 0.0], [140.0, 0.0], [140.0, 80.0], [0.0, 80.0]];
    let eight: Vec<[f64; 2]> = [20.0, 50.0].iter().flat_map(|y| [20.0, 50.0, 80.0, 110.0].iter().map(move |x| [*x, *y])).collect();
    let first = bf(&field, &eight);
    assert_eq!(first.status, Status::Assumed, "it rests on the uncapped build-up: {first:?}");
    assert!(first.note.contains("nothing caps the build-up yet"), "{first:?}");
    let mut reversed = eight.clone();
    reversed.reverse();
    assert_eq!(bf(&field, &reversed).value, first.value);
    for seed in 1..6 {
        assert_eq!(bf(&field, &shuffled(&eight, seed)).value, first.value, "seed {seed}");
    }
    let square = [[0.0, 0.0], [200.0, 0.0], [200.0, 200.0], [0.0, 200.0]];
    let grid: Vec<[f64; 2]> = (0..6).flat_map(|j| (0..6).map(move |i| [25.0 + 30.0 * i as f64, 25.0 + 30.0 * j as f64])).collect();
    let in_order = bf(&square, &grid).value;
    assert!(in_order.is_some());
    for seed in 1..4 {
        let got = bf(&square, &shuffled(&grid, seed));
        assert_eq!(got.value, in_order, "seed {seed}: {got:?}");
    }
    // The same drain twice is one drain typed twice.
    let mut job = Job::new("Flat roof", "test", "2026-11-19");
    let area = ed::low_slope(&mut job, "Main roof", &field, "1/4:12", "11 ft", how()).unwrap();
    let why = ed::drains(&mut job, &area, &[[20.0, 10.0], [20.0, 10.0]], "1 in", how()).unwrap_err();
    assert!(why.contains("is given twice"), "{why}");
}

/// Judged 2026-11-19, after call 140: a drain on the edge was taken on two sides and
/// refused on the other two; drains exactly an inch apart were refused; a roof drawn far
/// from the plan's origin drifted; and where two sides tie for longest, the corner the
/// outline began at changed the figures.
#[test]
fn a_taper_is_the_same_wherever_and_however_its_roof_is_traced() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Known, source: "made-up roof plan" };
    let made = |ring: &[[f64; 2]]| -> (Job, String) {
        let mut job = Job::new("Flat roof", "test", "2026-11-19");
        let area = ed::low_slope(&mut job, "Main roof", ring, "1/4:12", "11 ft", how()).unwrap();
        (job, area)
    };
    let bf = |ring: &[[f64; 2]], drains: &[[f64; 2]]| -> Option<f64> {
        let (mut job, area) = made(ring);
        ed::drains(&mut job, &area, drains, "1 in", how()).unwrap();
        job.derive();
        measure(&mut job, &area, "board_feet").value
    };
    // Every edge and corner alike.
    let square = [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]];
    for d in [[0.0, 20.0], [20.0, 0.0], [40.0, 20.0], [20.0, 40.0], [0.0, 0.0], [40.0, 40.0]] {
        let (mut job, area) = made(&square);
        let why = ed::drains(&mut job, &area, &[d], "1 in", how()).unwrap_err();
        assert!(why.contains("is on the edge"), "{d:?}: {why}");
    }
    // Exactly an inch apart is two drains.
    let (mut job, area) = made(&square);
    ed::drains(&mut job, &area, &[[20.0, 20.0], [20.0 + 1.0 / 12.0, 20.0]], "1 in", how()).unwrap();
    // Far from the origin, the same.
    let roof = [[0.0, 0.0], [80.0, 0.0], [80.0, 50.0], [0.0, 50.0]];
    let seven = [[10.0, 10.0], [30.0, 14.0], [55.0, 12.0], [12.0, 38.0], [33.0, 41.0], [65.0, 37.0], [70.0, 25.0]];
    let here = bf(&roof, &seven);
    for far in [1.0e5, 5.0e5, 9.0e5] {
        let moved = |p: &[[f64; 2]]| p.iter().map(|q| [q[0] + far, q[1] + far]).collect::<Vec<_>>();
        assert_eq!(bf(&moved(&roof), &moved(&seven)), here, "{far}");
    }
    // A rhombus, every side 50 ft, begun at each corner and run either way.
    let (c, s) = (0.3f64.cos(), 0.3f64.sin());
    let rhombus: Vec<[f64; 2]> = [[0.0, 0.0], [50.0, 0.0], [80.0, 40.0], [30.0, 40.0]].iter().map(|p| [100.0 + p[0] * c - p[1] * s, 100.0 + p[0] * s + p[1] * c]).collect();
    let middle = [rhombus.iter().map(|p| p[0]).sum::<f64>() / 4.0, rhombus.iter().map(|p| p[1]).sum::<f64>() / 4.0];
    let first = bf(&rhombus, &[middle]);
    assert!(first.is_some());
    for start in 0..4 {
        let mut r: Vec<[f64; 2]> = (0..4).map(|k| rhombus[(start + k) % 4]).collect();
        assert_eq!(bf(&r, &[middle]), first, "begun at {start}");
        r.reverse();
        assert_eq!(bf(&r, &[middle]), first, "begun at {start}, run the other way");
    }
}

/// Call 142: the field list asks about the drains and the thickness at them, `measure`
/// takes both, and the uncapped caution travels to the line priced by the board feet.
#[test]
fn the_field_list_asks_about_a_tapers_drains_and_its_caution_reaches_the_line() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Assumed, source: "made-up roof plan" };
    let mut job = Job::new("Flat roof", "test", "2026-11-19");
    let area = ed::low_slope(&mut job, "Main roof", &[[0.0, 0.0], [48.0, 0.0], [48.0, 48.0], [0.0, 48.0]], "1/4:12", "11 ft", ed::Entered { status: Status::Known, source: "made-up roof plan" }).unwrap();
    ed::drains(&mut job, &area, &[[12.0, 12.0], [36.0, 36.0], [12.0, 36.0], [36.0, 12.0]], "1/2 in", how()).unwrap();
    job.put(&area, Cost { code: value::Val::unknown("", "not coded"), lines: vec![CostLine::of("Tapered polyiso", "board_feet", value::Val::known(1.2, "$/BF", "made-up price"))] }).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &area, "board_feet").value, Some(5760.0));
    let q = &figures(&mut job, &area, "Tapered polyiso").quantity;
    assert!(q.note.contains("nothing caps the build-up yet"), "{q:?}");
    let list = plumbline_world::vif::vif(&mut job);
    let asked: Vec<String> = list.questions.iter().filter(|x| x.path == area).flat_map(|x| x.values.iter().map(|v| v.0.clone())).collect();
    assert!(asked.contains(&"drains".to_string()) && asked.contains(&"thickness at the drain".to_string()), "{asked:?}");
    ed::measure::measure(&mut job, &area, "thickness at the drain", ed::measure::Reading::Length("5/8 in"), None, "tape at the drain").unwrap();
    ed::measure::measure(&mut job, &area, "drains", ed::measure::Reading::Same, None, "walked the roof").unwrap();
    job.derive();
    // 2,304 SF × 1/8 in more.
    assert_eq!(measure(&mut job, &area, "board_feet").value, Some(6048.0));
    assert!(plumbline_world::vif::vif(&mut job).questions.iter().all(|x| x.path != area));
}

/// Call 146: a room's wall area goes round what stands inside it, is refused where an
/// opening does not fit its wall, and board priced by the room and on its walls is said.
#[test]
fn a_rooms_wall_area_takes_in_what_stands_inside_it_and_board_twice_is_said() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("Rooms", "test", "2026-11-19");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", how()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", how()).unwrap();
    let w4 = ed::wall_type(&mut job, "w4", "4 in", how()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), how()).unwrap();
    ed::walls(&mut job, &level, &w4, &[[8.0, 5.0], [14.0, 5.0], [14.0, 11.0], [8.0, 11.0]], true, ed::Top::Height("9 ft"), how()).unwrap();
    let big = ed::room(&mut job, &level, "Big", [3.0, 3.0], how()).unwrap();
    job.derive();
    // 103.3333 LF round, the inner room's faces with it, × 9 ft.
    assert_eq!(measure(&mut job, &big, "perimeter").value, Some(103.3333));
    assert_eq!(measure(&mut job, &big, "wall_area").value, Some(930.0));
    // Priced by its wall area, and board on one of its walls as well: said.
    job.put(&big, Cost { code: value::Val::unknown("", "not coded"), lines: vec![CostLine::of("Gypsum board, walls", "wall_area", value::Val::unknown("$/SF", "no price entered"))] }).unwrap();
    job.derive();
    assert!(plumbline_world::view::pricing(&mut job).board_twice.is_empty());
    job.put(&outer[0], Cost { code: value::Val::unknown("", "not coded"), lines: vec![CostLine::of("Gypsum board, hung and finished", "net_area", value::Val::unknown("$/SF", "no price entered"))] }).unwrap();
    job.derive();
    let said = plumbline_world::view::board_twice_said(&plumbline_world::view::pricing(&mut job).board_twice).expect("said");
    assert!(said.contains("Big's walls are priced by its wall area") && said.contains("in the price twice"), "{said}");
}

/// Call 147: the fewest receptacles a room's walls need, NEC 210.52(A)(1): runs of wall
/// between doors, one for each 12 ft or part; Assumed, saying what it leaves out.
#[test]
fn a_room_counts_the_fewest_receptacles_its_walls_need() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("Rooms", "test", "2026-11-19");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", how()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", how()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [10.0, 0.0], [10.0, 12.0], [0.0, 12.0]], true, ed::Top::Height("9 ft"), how()).unwrap();
    let shut = ed::room(&mut job, &level, "Closed room", [5.0, 6.0], how()).unwrap();
    job.derive();
    // 42 LF round its faces, one closed run: 3.5, so 4.
    let d = measure(&mut job, &shut, "receptacles");
    assert_eq!((d.value, d.status), (Some(4.0), Status::Assumed), "{d:?}");
    assert!(d.note.contains("not drawn") && d.formula.contains("210.52(A)(1)"), "{d:?}");
}

/// Call 148: existing, to demolish, new. A demolished partition bounds no room and does
/// not trim the walls it met; a new wall may stand where it stood; a priced thing marked
/// existing asks what to do with its lines; the mark is saved, read back, and refused on
/// what is not built.
#[test]
fn a_renovation_marks_what_stands_what_comes_down_and_what_is_new() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Known, source: "made-up survey" };
    let mut job = Job::new("Renovation", "test", "2026-11-21");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", how()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", how()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), how()).unwrap();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), how()).unwrap().remove(0);
    let room = ed::room(&mut job, &level, "Living", [6.0, 8.0], how()).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &room, "floor_area").value, Some(178.25));
    // A wall over the standing partition is one wall drawn over another.
    let over = |job: &mut Job| ed::walls(job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), ed::Entered { status: Status::Assumed, source: "made-up plan" });
    assert!(over(&mut job).unwrap_err().contains("drawn over another"));
    // Priced, then marked existing: asked which; cleared.
    job.put(&part, Cost { code: value::Val::unknown("", "not coded"), lines: vec![CostLine::of("Gypsum board, both sides", "2 * net_area", value::Val::unknown("$/SF", "no price entered"))] }).unwrap();
    let why = ed::mark(&mut job, &[part.clone()], WorkStatus::Existing, ed::ItsLines::Ask, how()).unwrap_err();
    assert!(why.contains("Say which: take its lines off, or keep them priced"), "{why}");
    ed::mark(&mut job, &[part.clone()], WorkStatus::Existing, ed::ItsLines::Clear, how()).unwrap();
    assert!(job.find(&part).and_then(|e| job.world.get::<Cost>(e)).is_none());
    // To demolish: one room of two, to the faces, with no notch where it met the walls.
    ed::mark(&mut job, &[part.clone()], WorkStatus::Demolish, ed::ItsLines::Ask, how()).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &room, "floor_area").value, Some(364.25));
    // The addition past a demolished east wall.
    ed::mark(&mut job, &[outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, how()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), how()).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &room, "floor_area").value, Some(550.25));
    // A new wall where the partition stood is taken now.
    let text = save(&mut job);
    over(&mut job).unwrap();
    // Saved and read back; a room is not built.
    let (mut loaded, _) = load(&text).unwrap();
    assert_eq!(measure(&mut loaded, &room, "floor_area").value, Some(550.25));
    assert_eq!(loaded.find(&part).and_then(|e| loaded.world.get::<Work>(e)).and_then(|w| w.status.value), Some(WorkStatus::Demolish));
    assert!(ed::mark(&mut loaded, &[room.clone()], WorkStatus::Existing, ed::ItsLines::Ask, how()).unwrap_err().contains("not a thing that is built"));
}

/// Call 150: a doorway widened. The old door to demolish is filled in and is no hole;
/// the wider door is the wall's one hole; no door is set in a wall coming down.
#[test]
fn a_door_to_demolish_is_filled_in() {
    use plumbline_world::editing as ed;
    let how = || ed::Entered { status: Status::Known, source: "made-up survey" };
    let mut job = Job::new("Renovation", "test", "2026-11-21");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", how()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", how()).unwrap();
    let d3 = ed::opening_type(&mut job, "d3", Kind::Door, "3 ft", "7 ft", how()).unwrap();
    let d5 = ed::opening_type(&mut job, "d5", Kind::Door, "5 ft", "7 ft", how()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), how()).unwrap();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), how()).unwrap().remove(0);
    let old = ed::opening(&mut job, &part, Kind::Door, &d3, ed::OnWall::Offset("8 ft"), "0 ft", how()).unwrap();
    let room = ed::room(&mut job, &level, "West", [6.0, 8.0], how()).unwrap();
    ed::mark(&mut job, &[old.clone()], WorkStatus::Demolish, ed::ItsLines::Ask, how()).unwrap();
    job.derive();
    // Filled in: the partition is whole, and the room's floor line is one run of 54 ft.
    assert_eq!(measure(&mut job, &part, "net_area").value, Some(144.0));
    assert_eq!(measure(&mut job, &room, "receptacles").value, Some(5.0));
    ed::opening(&mut job, &part, Kind::Door, &d5, ed::OnWall::Offset("8 ft"), "0 ft", how()).unwrap();
    job.derive();
    // 144 less the 5 ft door's 35; the room's 486 less the same.
    assert_eq!(measure(&mut job, &part, "net_area").value, Some(109.0));
    assert_eq!(measure(&mut job, &room, "wall_area").value, Some(451.0));
    // Marking the wall leaves the door that is coming out as it is.
    let said = ed::mark(&mut job, &[part.clone()], WorkStatus::Existing, ed::ItsLines::Ask, how()).unwrap().join("\n");
    assert!(said.contains(&format!("{old}, in it: stays to demolish")), "{said}");
    // No door in a wall that is coming down.
    ed::mark(&mut job, &[part.clone()], WorkStatus::Demolish, ed::ItsLines::Ask, how()).unwrap();
    assert!(ed::opening(&mut job, &part, Kind::Door, &d3, ed::OnWall::Offset("3 ft"), "0 ft", how()).unwrap_err().contains("is to demolish"));
}

/// Call 152: a room line parts two rooms where no wall stands. It runs the rooms of its
/// level to it, is no wall anywhere, takes no price and no mark, is no surer than it says,
/// is saved and read back, and when it is removed the rooms are as they were.
#[test]
fn a_room_line_parts_two_rooms_where_no_wall_stands() {
    use plumbline_world::editing as ed;
    let known = || ed::Entered { status: Status::Known, source: "made-up survey" };
    let mut job = Job::new("Open plan", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    ed::mark(&mut job, &[outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
    let living = ed::room(&mut job, &level, "Living", [6.0, 8.0], known()).unwrap();
    let kitchen = ed::room(&mut job, &level, "Kitchen", [30.0, 8.0], known()).unwrap();
    job.derive();
    // One room is the whole inside; the second has no figures: the first has the space.
    assert_eq!(measure(&mut job, &living, "floor_area").value, Some(550.25));
    let (k, status) = (measure(&mut job, &kitchen, "floor_area"), measure(&mut job, &living, "floor_area").status);
    assert!(k.value.is_none() && k.note.contains("its point is in the space Living has"), "{k:?}");
    assert_eq!(status, Status::Known);
    let clashes_before = clashes_of(&mut job, None);
    let walls_before = plumbline_world::building::storey_totals(&mut job)[0].walls.clone();

    // A room line is under a level and nothing else.
    let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).unwrap();
    assert!(ed::room_lines(&mut job, &root, &[[24.0, 0.0], [24.0, 16.0]], known()).unwrap_err().contains("not a level"));
    let loose = job.add(&root, Kind::RoomLine, "loose").unwrap();
    job.put(&loose, RoomLine { line: value::Val::known([[24.0, 0.0], [24.0, 16.0]], "ft", known().source) }).unwrap();
    assert!(plumbline_world::building::check_whole(&mut job).unwrap_err().contains("has to be directly under"));
    job.remove(&loose).unwrap();

    // The line, Assumed: the two rooms, to the digit.
    let made = ed::room_lines(&mut job, &level, &[[24.0, 0.0], [24.0, 16.0]], ed::Entered { status: Status::Assumed, source: "made-up addition plan" }).unwrap();
    assert_eq!(made.len(), 1);
    job.derive();
    let figure = |job: &mut Job, room: &str, name: &str| measure(job, room, name).value;
    for (room, floor, round, gross, walls, outlets) in [(&living, 368.125, 78.5, 384.0, 567.0, 6.0), (&kitchen, 182.125, 54.5, 192.0, 351.0, 4.0)] {
        assert_eq!(figure(&mut job, room, "floor_area"), Some(floor), "{room}");
        assert_eq!(figure(&mut job, room, "perimeter"), Some(round), "{room}");
        assert_eq!(figure(&mut job, room, "gross_area"), Some(gross), "{room}");
        assert_eq!(figure(&mut job, room, "wall_area"), Some(walls), "{room}");
        assert_eq!(figure(&mut job, room, "receptacles"), Some(outlets), "{room}");
        // No surer than the line: it is cited, and the room is Assumed.
        let d = measure(&mut job, room, "floor_area");
        assert_eq!(d.status, Status::Assumed, "{room}");
        assert!(d.inputs.iter().any(|i| i == &format!("{}#pl::room_line", made[0])), "{d:?}");
    }
    assert_eq!(figure(&mut job, &living, "floor_area").unwrap() + figure(&mut job, &kitchen, "floor_area").unwrap(), 550.25);

    // It is no wall: not counted, not tested, not priced, not marked.
    let totals = plumbline_world::building::storey_totals(&mut job);
    assert_eq!((totals[0].walls.members, totals[0].walls.value), (walls_before.members, walls_before.value));
    assert_eq!(totals[0].walls.members, 7);
    assert_eq!(clashes_of(&mut job, None), clashes_before);
    let why = job.add_line(&made[0], "Partition", "1", None, false).unwrap_err();
    assert!(why.contains("a room line cannot carry cost lines"), "{why}");
    let why = job.set(&made[0], "allowance", Some("$2.00/SF"), Status::Known, "made-up").unwrap_err();
    assert!(why.contains("a room line cannot carry an allowance"), "{why}");
    let why = ed::mark(&mut job, &[made[0].clone()], WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap_err();
    assert!(why.contains("a room line, which is not a thing that is built") || why.contains("not a thing that is built"), "{why}");

    // Saved and read back the same.
    let text = save(&mut job);
    assert!(text.contains("pl::room_line"));
    let (mut loaded, _) = load(&text).unwrap();
    assert_eq!(save(&mut loaded), text);
    assert_eq!(measure(&mut loaded, &living, "floor_area").value, Some(368.125));
    assert_eq!(measure(&mut loaded, &kitchen, "floor_area").value, Some(182.125));

    // Removed, the rooms are as they were.
    ed::remove(&mut job, &made[0]).unwrap();
    job.derive();
    assert_eq!(figure(&mut job, &living, "floor_area"), Some(550.25));
    assert_eq!(measure(&mut job, &living, "floor_area").status, Status::Known);
    assert!(figure(&mut job, &kitchen, "floor_area").is_none());
}

#[test]
fn two_layouts_are_set_side_by_side_from_their_own_takeoff_and_estimate() {
    use plumbline_world::view::{options, OptionFigure::*};
    let (mut a, mut b, mut c) = (addition_layout("Addition 12 ft", 36.0), addition_layout("Addition 16 ft", 40.0), Job::new("Nothing drawn yet", "test", "2026-11-22"));
    c.derive();
    let before = (save(&mut a), save(&mut b), save(&mut c));
    let two = options(&mut [&mut a, &mut b]);
    assert_eq!(two.titles, ["Addition 12 ft", "Addition 16 ft"]);
    let row = |t: &plumbline_world::view::Options, label: &str| t.rows.iter().find(|r| r.label.contains(label)).unwrap_or_else(|| panic!("no row {label}: {:?}", t.rows.iter().map(|r| &r.label).collect::<Vec<_>>())).clone();
    // To the figures worked by hand in STATUS.md.
    for (label, x, y, said) in [
        ("Living", 368.125, 368.125, "same"),
        ("Kitchen", 182.125, 244.125, "+62 SF"),
        ("all rooms", 550.25, 612.25, "+62 SF"),
        ("walls, new", 40.0, 48.0, "+8 LF"),
        ("job cost", 5777.0, 7137.0, "+$1,360.00"),
        ("Wood and Plastics", 4320.0, 5184.0, "+$864.00"),
        ("Finishes", 1457.0, 1953.0, "+$496.00"),
    ] {
        let r = row(&two, label);
        assert_eq!((&r.figures[0], &r.figures[1]), (&Is(x, Status::Assumed), &Is(y, Status::Assumed)), "{label}");
        assert_eq!(r.difference_said(), said, "{label}");
    }
    // What stands and what comes down: Known (the survey), the same in both.
    assert_eq!(row(&two, "existing").figures, [Is(64.0, Status::Known), Is(64.0, Status::Known)]);
    assert_eq!(row(&two, "to demolish").figures, [Is(16.0, Status::Known), Is(16.0, Status::Known)]);
    assert_eq!(row(&two, "existing").difference_said(), "same");
    // Divisions with no work in either are not rows.
    assert!(two.rows.iter().all(|r| !r.label.contains("Masonry")));
    // The other way round, a decrease has its minus.
    let back = options(&mut [&mut b, &mut a]);
    assert_eq!(row(&back, "Kitchen").difference_said(), "-62 SF");
    assert_eq!(row(&back, "walls, new").difference_said(), "-8 LF");
    assert_eq!(row(&back, "job cost").difference_said(), "-$1,360.00");
    // A third, empty: none, no figure, and no difference worked.
    let three = options(&mut [&mut a, &mut b, &mut c]);
    assert_eq!(three.titles[2], "Nothing drawn yet");
    for label in ["Living", "all rooms", "walls, new", "Wood and Plastics"] {
        let r = row(&three, label);
        assert_eq!(r.figures[2], None, "{label}");
        assert_eq!(r.difference, Option::None, "{label}");
        assert_eq!(r.difference_said(), "not worked", "{label}");
    }
    assert_eq!(row(&three, "job cost").figures[2], NoFigure);
    assert_eq!(row(&three, "job cost").difference, Option::None);
    // Nothing in any of the three was changed.
    assert_eq!((save(&mut a), save(&mut b), save(&mut c)), before);
}

/// Call 155: a run of cabinet boxes along a wall's face on a room's side, by the trade's
/// codes. The house of `a_room_line_parts_two_rooms_where_no_wall_stands`: the Kitchen's
/// east wall runs 36,0 to 36,16, its face on the Kitchen's side is at x = 35.75, clear from
/// y = 0.25 to 15.75. B18 SB36 B24 DB18 B36 from 1 ft: 18 + 36 + 24 + 18 + 36 = 132 in, 11 ft
/// from y = 1 to y = 12, 3.75 ft left. Each box is 2 ft deep: its middle is 1 ft off the
/// face, x = 34.75, at y = 1.75, 4, 6.5, 8.25, 10.5. The countertop: 11 ft by 25 1/2 in
/// (2.125 ft) is 23.375 SF, its middle 1.0625 ft off the face (34.6875, 6.5), 34 1/2 in up;
/// at $62/SF, 23.375 x 62 = $1,449.25. Wall boxes W3030 W3630 W1830 from 1 ft: 7 ft, 1 ft
/// deep, middles 0.5 ft off the face (35.25) at y = 2.25, 5, 7.25, 54 in up.
#[test]
fn a_run_of_cabinet_boxes_stands_along_a_wall_to_the_figures_worked_by_hand() {
    use plumbline_world::editing as ed;
    let known = || ed::Entered { status: Status::Known, source: "made-up survey" };
    let assumed = || ed::Entered { status: Status::Assumed, source: "made-up addition plan" };
    let mut job = Job::new("A kitchen", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    ed::mark(&mut job, &[outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    let new = ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    let east = new[1].clone();
    ed::room(&mut job, &level, "Living", [6.0, 8.0], known()).unwrap();
    let kitchen = ed::room(&mut job, &level, "Kitchen", [30.0, 8.0], known()).unwrap();
    ed::room_lines(&mut job, &level, &[[24.0, 0.0], [24.0, 16.0]], assumed()).unwrap();
    // The tool chest: 27 boxes in Division 12, by their codes.
    let rows: Vec<Vec<String>> = plumbline_world::tables::STARTER_ITEMS
        .lines()
        .map(|line| {
            let (mut out, mut cell, mut quoted) = (Vec::new(), String::new(), false);
            for c in line.chars() {
                match (c, quoted) {
                    ('"', _) => quoted = !quoted,
                    (',', false) => out.push(std::mem::take(&mut cell)),
                    (c, _) => cell.push(c),
                }
            }
            out.push(cell);
            out
        })
        .collect();
    plumbline_world::tables::items_in(&mut job, &rows, None).unwrap();
    assert_eq!(ed::box_codes(&mut job).len(), 27);
    let clashes_before = clashes_of(&mut job, None);

    // Refused, and the job is as it was.
    let before = save(&mut job);
    // call 225: B99 is now a box made by the rule for B (call 215); a code no rule reads is Q99
    let err = ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &["Q99".to_string()], false, assumed()).unwrap_err();
    assert!(err.contains("Q99 is not in the tool chest, and no cabinet rule reads Q") && err.contains("B18") && err.contains("T2484"), "{err}");
    let six: Vec<String> = ["B36"; 6].iter().map(|c| c.to_string()).collect();
    let err = ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &six, false, assumed()).unwrap_err();
    assert!(err.contains("18' 0\" from 1' 0\" reaches 19' 0\" along") && err.contains("past the corner"), "{err}");
    let err = ed::cabinets(&mut job, &level, &east, &kitchen, "0 ft", &["B18".to_string()], false, assumed()).unwrap_err();
    assert!(err.contains("clear face"), "{err}");
    assert_eq!(save(&mut job), before);

    // The run, with its countertop.
    let codes: Vec<String> = "B18 SB36 B24 DB18 B36".split(' ').map(str::to_string).collect();
    let run = ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &codes, true, assumed()).unwrap();
    assert_eq!((run.from, run.run, run.left, run.hung), (1.0, 11.0, 3.75, None));
    let places: Vec<(&str, [f64; 2])> = run.boxes.iter().map(|b| (b.1.as_str(), b.2)).collect();
    assert_eq!(places, [("B18", [34.75, 1.75]), ("SB36", [34.75, 4.0]), ("B24", [34.75, 6.5]), ("DB18", [34.75, 8.25]), ("B36", [34.75, 10.5])]);
    // Each box is one of its item, as sure as the item, backs to the wall (turned a quarter
    // the wrong way round from east: -90 degrees puts its back, +y, on the east), with one
    // line: its item's name, 1 EA, Division 12, no price.
    for (path, code, _, width) in &run.boxes {
        let e = job.find(path).unwrap();
        let p = job.world.get::<Placed>(e).unwrap().clone();
        assert!(p.item.is_some() && p.size.status == Status::Assumed && p.at.status == Status::Assumed && p.offset.value == Some(units::Len::One(0.0)), "{p:?}");
        assert_eq!(p.angle_deg(), Some(-90.0));
        assert_eq!(p.size_ft().map(|s| (s[0], s[1])), Some((*width, 2.0)));
        let cost = job.world.get::<Cost>(e).unwrap();
        assert_eq!(cost.lines.len(), 1);
        assert_eq!((cost.lines[0].description.as_str(), cost.lines[0].quantity.as_str(), cost.lines[0].division.map(|d| d.number()), cost.lines[0].unit_cost.value.is_none()), (code.as_str(), "1", Some(12), true));
    }
    // Its figures are as sure as its size, each with its formula and inputs.
    let (top, at) = run.countertop.clone().unwrap();
    assert_eq!(at, [34.6875, 6.5]);
    for (name, want) in [("width", 11.0), ("depth", 2.125), ("plan_area", 23.375)] {
        let d = measure(&mut job, &top, name);
        assert_eq!(d.value, Some(want), "{name}: {d:?}");
        assert_eq!(d.status, Status::Assumed);
        assert!(!d.formula.is_empty() && d.inputs == [format!("{top}#pl::placed")], "{d:?}");
    }
    assert_eq!(measure(&mut job, &top, "height").value, Some(0.125));
    let e = job.find(&top).unwrap();
    let p = job.world.get::<Placed>(e).unwrap().clone();
    assert_eq!(p.offset.feet(), Some(2.875));
    // Priced by plan_area at $62/SF: 23.375 x 62 = 1,449.25. (Call 162: the countertop carries its
    // own line, "Countertop", by its area, with no price; it is priced where it stands.)
    job.set_on_line(&top, "unit_cost", Some("Countertop"), Some("$62.00/SF"), Status::Assumed, "made-up price").unwrap();
    job.derive();
    let est = plumbline_world::view::estimate(&mut job);
    let line = est.lines.iter().find(|l| l.path == top && l.description == "Countertop").unwrap();
    assert!((line.amount.value.unwrap() - 1449.25).abs() < 1.0e-9, "{:?}", line.amount);
    // The wall boxes, hung 54 in up.
    let wall_codes: Vec<String> = "W3030 W3630 W1830".split(' ').map(str::to_string).collect();
    let hung = ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &wall_codes, false, assumed()).unwrap();
    assert_eq!((hung.run, hung.hung), (7.0, Some(4.5)));
    let places: Vec<[f64; 2]> = hung.boxes.iter().map(|b| b.2).collect();
    assert_eq!(places, [[35.25, 2.25], [35.25, 5.0], [35.25, 7.25]]);
    let e = job.find(&hung.boxes[0].0).unwrap();
    let p = job.world.get::<Placed>(e).unwrap().clone();
    assert_eq!((p.offset.feet(), p.offset.status), (Some(4.5), Status::Assumed));
    assert!(p.offset.source.contains("54 in up"));
    // No box clashes with the wall it backs onto or with the boxes beside it.
    assert_eq!(clashes_of(&mut job, None), clashes_before);
    // The test is live: a box pushed 0.3 ft into the wall does clash with it.
    ed::move_by(&mut job, &run.boxes[0].0, 0.3, 0.0, assumed()).unwrap();
    assert_eq!(clashes_of(&mut job, None).len(), clashes_before.len() + 1);
}
