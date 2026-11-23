//! Call 179, part C: the wrong figures a judge found. The houses are made up; every figure is
//! worked by hand beside it.

use plumbline_world::editing as ed;
use plumbline_world::file;
use plumbline_world::value::Val;
use plumbline_world::{Len, Opening, SegmentType};
use plumbline_world::value::Status;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up plan" }
}

/// One 40 ft wall, 10 ft high, with the given doors and windows ((width, height, offset, sill)),
/// a stone line priced by `net_length` and a siding line that takes the stone's 3 ft off.
fn wall_with(doors: &[(&str, &str, &str, &str)], windows: &[(&str, &str, &str, &str)], high: &str) -> (Job, String) {
    let mut job = Job::new("A wall (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", high, known()).unwrap();
    let wt = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let wall = ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [40.0, 0.0]], false, ed::Top::Height(high), known()).unwrap().remove(0);
    for (i, (w, h, at, sill)) in doors.iter().enumerate() {
        let t = ed::opening_type(&mut job, &format!("door {i}"), Kind::Door, w, h, known()).unwrap();
        ed::opening(&mut job, &wall, Kind::Door, &t, ed::OnWall::Offset(at), sill, known()).unwrap();
    }
    for (i, (w, h, at, sill)) in windows.iter().enumerate() {
        let t = ed::opening_type(&mut job, &format!("window {i}"), Kind::Window, w, h, known()).unwrap();
        ed::opening(&mut job, &wall, Kind::Window, &t, ed::OnWall::Offset(at), sill, known()).unwrap();
    }
    job.add_line(&wall, "Stone veneer wainscot, 3 ft", "net_length", Some(Class::Subcontract), false).unwrap();
    job.add_line(&wall, "Lap siding", "net_area", Some(Class::Material), false).unwrap();
    job.set_formula(&wall, Some("Lap siding"), "net_area - net_area_below_wainscot", &[("wainscot".to_string(), "3 ft".to_string())], Status::Assumed, "the 3 ft in the stone line's name").unwrap();
    (job, wall)
}

fn figure(job: &mut Job, path: &str, name: &str) -> Option<f64> {
    job.derive();
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).and_then(|m| m.0.get(name)).and_then(|d| d.value).map(|x| (x * 100.0).round() / 100.0)
}

/// What the sheet says the siding is: its quantity, and whether it is flagged Assumed.
fn siding(job: &mut Job, _wall: &str) -> Option<f64> {
    job.derive();
    let est = view::estimate(job);
    let line = est.lines.iter().find(|l| l.description == "Lap siding").unwrap();
    line.quantity.value.map(|x| (x * 100.0).round() / 100.0)
}

/// C2. The door 3 ft by 6 ft 8 in at the floor and a window 5 ft by 5 ft: face 400, door 20,
/// window 25, net 355. Under 3 ft: 3 x 40 = 120 less the door's 3 x 3 = 9. The window's part:
/// sill at 1 ft, 5 x 2 = 10, so under 3 ft 101 and the siding 355 - 101 = 254; sill at 3 ft,
/// nothing, 111 and 244; sill at 0.5 ft, 5 x 2.5 = 12.5, 98.5 and 256.5 (355 - 98.5). The stone
/// stays 37 LF.
#[test]
fn siding_is_the_net_area_less_the_net_area_below_the_stone_so_a_low_sill_is_taken_off_once() {
    for (sill, below, by_hand) in [("1 ft", 101.0, 254.0), ("3 ft", 111.0, 244.0), ("0.5 ft", 98.5, 256.5), ("4 ft", 111.0, 244.0)] {
        let (mut job, wall) = wall_with(&[("3 ft", "6 ft 8 in", "6 ft", "0 ft")], &[("5 ft", "5 ft", "24 ft", sill)], "10 ft");
        assert_eq!(figure(&mut job, &wall, "net_length"), Some(37.0), "{sill}");
        let net = figure(&mut job, &wall, "net_area").unwrap();
        // The window at 4 ft is the same 25 SF; its sill is above the stone: nothing below.
        assert_eq!(net, 355.0, "{sill}");
        assert_eq!(figure(&mut job, &wall, "net_area_below_wainscot"), Some(below), "{sill}");
        assert_eq!(siding(&mut job, &wall), Some(by_hand), "{sill}");
    }
}

/// The judge's own case: no door, the window 5 by 5 with its sill at 1 ft. Face 400, net 375;
/// under 3 ft 120 less 5 x 2 = 110; siding 375 - 110 = 265 (400 - 120 band - 15 above it): 265 is right.
#[test]
fn the_judges_case_comes_to_265() {
    let (mut job, wall) = wall_with(&[], &[("5 ft", "5 ft", "24 ft", "1 ft")], "10 ft");
    assert_eq!(figure(&mut job, &wall, "net_area"), Some(375.0));
    assert_eq!(figure(&mut job, &wall, "net_length"), Some(40.0));
    assert_eq!(figure(&mut job, &wall, "net_area_below_wainscot"), Some(110.0));
    assert_eq!(siding(&mut job, &wall), Some(265.0));
}

/// Two more of the same kind. A window 5 ft by 2 ft at the floor, wholly under the stone: the
/// stone line is 35 LF (the old figure bought 285 SF, taking 3 ft off a window 2 ft tall); face
/// 400 less 10 = 390 net; under 3 ft 120 - 10 = 110; siding 280 (all of the wall above the 3 ft).
/// And a window 5 ft by 5 ft with its sill at 2 ft, its top at 7 ft: 5 x 1 under the stone, 120 - 5
/// = 115; net 375; siding 260 (400 - 120 band - 5 x 4 above it = 260).
#[test]
fn an_opening_lower_than_the_stone_and_one_straddling_its_top_are_taken_off_once() {
    let (mut job, wall) = wall_with(&[], &[("5 ft", "2 ft", "24 ft", "0 ft")], "10 ft");
    assert_eq!((figure(&mut job, &wall, "net_length"), figure(&mut job, &wall, "net_area"), figure(&mut job, &wall, "net_area_below_wainscot")), (Some(35.0), Some(390.0), Some(110.0)));
    assert_eq!(siding(&mut job, &wall), Some(280.0));
    let (mut job, wall) = wall_with(&[], &[("5 ft", "5 ft", "24 ft", "2 ft")], "10 ft");
    assert_eq!((figure(&mut job, &wall, "net_area"), figure(&mut job, &wall, "net_area_below_wainscot")), (Some(375.0), Some(115.0)));
    assert_eq!(siding(&mut job, &wall), Some(260.0));
}

/// The 3 ft is the line's own entry: change it and the figure follows; it is Assumed, and so is
/// the siding; a wall lower than the stone has no siding (a quantity below zero is not given).
#[test]
fn the_height_is_the_lines_own_and_what_rests_on_it_is_assumed() {
    let (mut job, wall) = wall_with(&[], &[("5 ft", "5 ft", "24 ft", "1 ft")], "10 ft");
    job.set_formula(&wall, Some("Lap siding"), "net_area - net_area_below_wainscot", &[("wainscot".to_string(), "4 ft".to_string())], Status::Assumed, "the stone is 4 ft").unwrap();
    // 4 x 40 = 160 less 5 x 3 = 15: 145; 375 - 145 = 230.
    assert_eq!(figure(&mut job, &wall, "net_area_below_wainscot"), Some(145.0));
    assert_eq!(siding(&mut job, &wall), Some(230.0));
    job.derive();
    let e = job.find(&wall).unwrap();
    assert_eq!(job.world.get::<Measure>(e).unwrap().0["net_area_below_wainscot"].status, Status::Assumed);
    // A wall 2 ft high under a 3 ft stone: 80 - 80 (its two ft) less the window's... no window here:
    let (mut low, wall) = wall_with(&[], &[], "2 ft");
    assert_eq!(figure(&mut low, &wall, "net_area_below_wainscot"), Some(120.0));
    assert_eq!(siding(&mut low, &wall), None, "80 SF of wall less 120 SF of stone is not a quantity");
}

/// C4. A 20 ft wall of 6 in on the line y = 0, a 4 in pipe at 4 ft from (6, -3) to (6, 3) crosses it: one
/// hole, at (6, 0, 4). Cut the pipe at (6, -1), at (6, 0.1) and again at (6, -0.2) (across the wall), and
/// the hole is still where the run passes through it, (6, 0, 4): it was (5.97, -1.375, 4.04) once cut.
/// A diagonal wall, the line from (0, 0) to (10, 10), crossed by a pipe along y = 4 from (2, 4) to (9, 4),
/// at 5 ft: the walls cross where x = 4, the hole at (4, 4, 5).
#[test]
fn a_hole_is_where_the_run_passes_through_however_the_run_is_split() {
    let mut job = Job::new("A pipe through a wall (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let wt = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let pt = ed::segment_type(&mut job, "pipe", Kind::Pipe, "4 in", None, known()).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
    let run = ed::runs(&mut job, &level, Kind::Pipe, &pt, &[[6.0, -3.0], [6.0, 3.0]], "4 ft", None, known()).unwrap().remove(0);
    let hole = |job: &mut Job| {
        let found = building::clash::clashes(job, None).unwrap();
        assert_eq!(found.holes.len(), 1, "one hole");
        found.holes[0].at
    };
    let near = |at: [f64; 3], want: [f64; 3]| at.iter().zip(want).all(|(a, b)| (a - b).abs() < 1.0e-3);
    assert!(near(hole(&mut job), [6.0, 0.0, 4.0]));
    let second = ed::split(&mut job, &run, [6.0, -1.0], known()).unwrap();
    let at = hole(&mut job);
    assert!(near(at, [6.0, 0.0, 4.0]), "{at:?}");
    let third = ed::split(&mut job, &second, [6.0, 0.1], known()).unwrap();
    let at = hole(&mut job);
    assert!(near(at, [6.0, 0.0, 4.0]), "{at:?}");
    ed::split(&mut job, &second, [6.0, -0.2], known()).unwrap();
    let at = hole(&mut job);
    assert!(near(at, [6.0, 0.0, 4.0]), "{at:?}");
    let _ = third;
    // A diagonal wall.
    let mut job = Job::new("A diagonal (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let wt = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let pt = ed::segment_type(&mut job, "pipe", Kind::Pipe, "4 in", None, known()).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [10.0, 10.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
    let run = ed::runs(&mut job, &level, Kind::Pipe, &pt, &[[2.0, 4.0], [9.0, 4.0]], "5 ft", None, known()).unwrap().remove(0);
    let at = hole(&mut job);
    assert!(near(at, [4.0, 4.0, 5.0]), "{at:?}");
    ed::split(&mut job, &run, [3.0, 4.0], known()).unwrap();
    let at = hole(&mut job);
    assert!(near(at, [4.0, 4.0, 5.0]), "{at:?}");
}

/// C5. A floor 10 in thick with its top at 9 ft (its depth 8.1667 to 9 ft): a 1 in pipe from (2, 5) to (10, 5)
/// at 8.6 ft lies in it along its whole length: laid in, 1, and no hole. The same pipe rising from 8 ft to
/// 9.5 ft comes out of it at the top: a hole, and nothing laid in. A pipe in a 6 in wall (the wall on y = 0, the
/// pipe along y = 0.1 at 4 ft from x = 3 to x = 15, 4 in round: its axis in the 6 in): laid in; the same
/// pipe along y = 0.4 (axis 0.15 ft outside the wall's 0.25 ft half-thickness, its skin 0.0167 ft into the
/// wall): not laid in; the program still counts it as one hole, as it did (a run that meets a wall is a
/// hole whatever the angle: not changed here, said in the report).
#[test]
fn a_run_lying_in_a_floor_or_a_wall_along_its_length_is_laid_in_and_no_hole() {
    let mut job = Job::new("Laid in (not a real job)", "test", "2026-11-23");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let second = ed::storey(&mut job, "second", "9 ft", "9 ft", known()).unwrap();
    let ft = ed::floor_type(&mut job, "slab", "10 in", known()).unwrap();
    let pt = ed::segment_type(&mut job, "pipe", Kind::Pipe, "1 in", None, known()).unwrap();
    ed::floor(&mut job, &second, &ft, ed::FloorBy::Points(&[[0.0, 0.0], [12.0, 0.0], [12.0, 10.0], [0.0, 10.0]]), known()).unwrap();
    let laid = ed::runs(&mut job, &ground, Kind::Pipe, &pt, &[[2.0, 5.0], [10.0, 5.0]], "8.6 ft", None, known()).unwrap().remove(0);
    let found = building::clash::clashes(&mut job, None).unwrap();
    assert_eq!((found.holes.len(), found.laid_in().len()), (0, 1), "{found:#?}");
    assert!(found.hole_rows().is_empty());
    ed::remove(&mut job, &laid).unwrap();
    ed::riser(&mut job, &ground, Kind::Pipe, &pt, [6.0, 5.0], "8 ft", "9.5 ft", None, known()).unwrap();
    let found = building::clash::clashes(&mut job, None).unwrap();
    assert_eq!((found.holes.len(), found.laid_in().len()), (1, 0));
    // A wall.
    for (y, laid_in, holes) in [(0.1, 1, 0), (0.4, 0, 1)] {
        let mut job = Job::new("Laid in a wall (not a real job)", "test", "2026-11-23");
        let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
        let wt = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
        let pt = ed::segment_type(&mut job, "pipe", Kind::Pipe, "4 in", None, known()).unwrap();
        ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
        ed::runs(&mut job, &level, Kind::Pipe, &pt, &[[3.0, y], [15.0, y]], "4 ft", None, known()).unwrap();
        let found = building::clash::clashes(&mut job, None).unwrap();
        assert_eq!((found.holes.len(), found.laid_in().len()), (holes, laid_in), "y = {y}");
    }
}

fn refused(r: Result<impl std::fmt::Debug, String>) -> String {
    r.expect_err("this was taken")
}

/// B5. A 40 ft wall; a door 3 ft wide hung with its middle at 0 ft (it starts 1.50 ft before the wall), at 40 ft
/// (it ends 1.5 ft past it) and at 39 ft (it ends at 40.5) is refused; at 1.5 ft and at 38.5 ft it fits (it ends
/// at 40.0). A window 5 ft wide at 20 ft (17.5 to 22.5) and another at 20.5 over it: refused; at 25 ft (22.5 to
/// 27.5), touching, they fit; a door under a window at another height over the same stretch is not over it (the
/// door 0 to 6 ft 8 in, the window from 7 ft). A door moved (measured) past the end, or onto a window, is refused.
#[test]
fn an_opening_lies_within_its_wall_and_clear_of_every_other_opening_in_it() {
    let (mut job, wall) = wall_with(&[], &[], "10 ft");
    let dt = ed::opening_type(&mut job, "door 3", Kind::Door, "3 ft", "6 ft 8 in", known()).unwrap();
    let nt = ed::opening_type(&mut job, "window 5", Kind::Window, "5 ft", "5 ft", known()).unwrap();
    for at in ["0 ft", "40 ft", "39 ft", "50 ft"] {
        let said = refused(ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset(at), "0 ft", known()));
        assert!(said.contains("an opening lies within its wall"), "{at}: {said}");
    }
    let before = file::save(&mut job);
    ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset("1.5 ft"), "0 ft", known()).unwrap();
    ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset("38.5 ft"), "0 ft", known()).unwrap();
    assert_ne!(file::save(&mut job), before);
    let first = ed::opening(&mut job, &wall, Kind::Window, &nt, ed::OnWall::Offset("20 ft"), "3 ft", known()).unwrap();
    let said = refused(ed::opening(&mut job, &wall, Kind::Window, &nt, ed::OnWall::Offset("20.5 ft"), "3 ft", known()));
    assert!(said.contains("lie over each other"), "{said}");
    // Touching is clear; a door under the window's sill, over the same stretch, is clear of it.
    let touching = ed::opening(&mut job, &wall, Kind::Window, &nt, ed::OnWall::Offset("25 ft"), "3 ft", known()).unwrap();
    ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset("10 ft"), "0 ft", known()).unwrap();
    let high = ed::opening_type(&mut job, "window high", Kind::Window, "5 ft", "2 ft", known()).unwrap();
    ed::opening(&mut job, &wall, Kind::Window, &high, ed::OnWall::Offset("10 ft"), "7 ft", known()).unwrap();
    // Moved: past the end, and onto the other window.
    let said = refused(ed::measure(&mut job, &first, "center, along the wall", ed::Reading::Length("41 ft"), None, "made-up tape"));
    assert!(said.contains("an opening lies within its wall"), "{said}");
    let said = refused(ed::measure(&mut job, &touching, "center, along the wall", ed::Reading::Length("21 ft"), None, "made-up tape"));
    assert!(said.contains("lie over each other"), "{said}");
    ed::measure(&mut job, &touching, "center, along the wall", ed::Reading::Length("30 ft"), None, "made-up tape").unwrap();
    assert!(building::drawn_faults(&mut job).is_empty());
}

/// A job saved before this holds a door hung past the end of its wall: it opens, `check` says it (the same
/// sentence), it can still be edited, and what it cannot have is one more.
#[test]
fn a_job_that_already_holds_such_a_thing_still_opens_says_so_and_can_be_changed() {
    let (mut job, wall) = wall_with(&[], &[], "10 ft");
    // (The door was drawn before the refusal: it is put in as the file had it, by hand.)
    let dt = ed::opening_type(&mut job, "door 3", Kind::Door, "3 ft", "6 ft 8 in", known()).unwrap();
    let door = job.add(&wall, Kind::Door, "door 1").unwrap();
    let val = |x: Len| Val { value: Some(x), unit: "ft".to_string(), status: Status::Known, source: "made-up plan".into() };
    let opening = Opening { type_name: Val { value: Some(dt.clone()), unit: String::new(), status: Status::Known, source: "made-up plan".into() }, offset: val(Len::One(0.0)), sill: val(Len::One(0.0)) };
    job.put(&door, opening).unwrap();
    job.derive();
    let text = file::save(&mut job);
    let (mut back, _) = file::load(&text).expect("a job saved before opens");
    assert_eq!(file::save(&mut back), text, "byte for byte");
    let said = building::drawn_faults(&mut back);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].1.contains("starts 1.50 ft before the start of its wall"), "{said:?}");
    // Another fault is refused; a change that brings none is taken.
    assert!(refused(ed::opening(&mut back, &wall, Kind::Door, &dt, ed::OnWall::Offset("40 ft"), "0 ft", known())).contains("an opening lies within its wall"));
    ed::opening(&mut back, &wall, Kind::Door, &dt, ed::OnWall::Offset("20 ft"), "0 ft", known()).unwrap();
}

/// B6. A floor 0,0 24,0 24,16 0,16 (384 SF) on a level. A floor 6,4 12,4 12,8 6,8 inside it: refused (it was 24 SF
/// more); one overlapping its edge (20,0 30,0 30,10 20,10): refused; one beside it sharing an edge
/// (24,0 34,0 34,16 24,16): taken; the same outline on the level above: taken. An L (0,0 24,0 24,10 12,10 12,16
/// 0,16) takes a floor in its notch (12,10 24,10 24,16 12,16), which only touches it, and refuses one that lies
/// 1 ft into it (11,9 24,9 24,16 11,16).
#[test]
fn a_floor_is_not_drawn_in_another_floor_of_its_level() {
    let mut job = Job::new("Floors (not a real job)", "test", "2026-11-23");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let second = ed::storey(&mut job, "second", "9 ft", "9 ft", known()).unwrap();
    let ft = ed::floor_type(&mut job, "slab", "6 in", known()).unwrap();
    let square = |x: f64, y: f64, w: f64, h: f64| vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
    ed::floor(&mut job, &ground, &ft, ed::FloorBy::Points(&square(0.0, 0.0, 24.0, 16.0)), known()).unwrap();
    for (x, y, w, h) in [(6.0, 4.0, 6.0, 4.0), (20.0, 0.0, 10.0, 10.0), (-2.0, -2.0, 30.0, 20.0), (0.0, 0.0, 24.0, 8.0)] {
        let said = refused(ed::floor(&mut job, &ground, &ft, ed::FloorBy::Points(&square(x, y, w, h)), known()));
        assert!(said.contains("in") && said.contains("would be counted twice"), "{x},{y}: {said}");
    }
    ed::floor(&mut job, &ground, &ft, ed::FloorBy::Points(&square(24.0, 0.0, 10.0, 16.0)), known()).unwrap();
    ed::floor(&mut job, &second, &ft, ed::FloorBy::Points(&square(0.0, 0.0, 24.0, 16.0)), known()).unwrap();
    let mut job = Job::new("L (not a real job)", "test", "2026-11-23");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let ft = ed::floor_type(&mut job, "slab", "6 in", known()).unwrap();
    ed::floor(&mut job, &ground, &ft, ed::FloorBy::Points(&[[0.0, 0.0], [24.0, 0.0], [24.0, 10.0], [12.0, 10.0], [12.0, 16.0], [0.0, 16.0]]), known()).unwrap();
    let said = refused(ed::floor(&mut job, &ground, &ft, ed::FloorBy::Points(&[[11.0, 9.0], [24.0, 9.0], [24.0, 16.0], [11.0, 16.0]]), known()));
    assert!(said.contains("would be counted twice"), "{said}");
    ed::floor(&mut job, &ground, &ft, ed::FloorBy::Points(&[[12.0, 10.0], [24.0, 10.0], [24.0, 16.0], [12.0, 16.0]]), known()).unwrap();
}

/// Two beams on one line (either way round, the same top) are refused; one with another top, another line, or
/// only part of the same line is a beam of its own. Two posts at one point are refused, one stood above the
/// other (base 8 ft, to 16 ft) is not; two pad footings at one point are refused, one 2 ft away is not.
#[test]
fn two_beams_posts_or_pads_are_not_drawn_in_one_place() {
    let mut job = Job::new("Members (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let wide = ed::segment_type(&mut job, "beam", Kind::Beam, "3 1/2 in", Some("11 7/8 in"), known()).unwrap();
    let post_type = ed::segment_type(&mut job, "post", Kind::Post, "3 1/2 in", Some("3 1/2 in"), known()).unwrap();
    let pad_type = ed::segment_type(&mut job, "pad", Kind::PadFooting, "24 in", Some("24 in"), known()).unwrap();
    ed::beams(&mut job, &level, &wide, &[[0.0, 0.0], [10.0, 0.0]], "9 ft", known()).unwrap();
    for points in [vec![[0.0, 0.0], [10.0, 0.0]], vec![[10.0, 0.0], [0.0, 0.0]]] {
        let said = refused(ed::beams(&mut job, &level, &wide, &points, "9 ft", known()));
        assert!(said.contains("two beams in one place would be counted twice"), "{said}");
    }
    ed::beams(&mut job, &level, &wide, &[[0.0, 0.0], [10.0, 0.0]], "8 ft", known()).unwrap();
    ed::beams(&mut job, &level, &wide, &[[0.0, 4.0], [10.0, 4.0]], "9 ft", known()).unwrap();
    ed::beams(&mut job, &level, &wide, &[[0.0, 0.0], [5.0, 0.0]], "9 ft", known()).unwrap();
    ed::post(&mut job, &level, &post_type, [3.0, 3.0], Some("8 ft"), None, known()).unwrap();
    let said = refused(ed::post(&mut job, &level, &post_type, [3.0, 3.0], Some("8 ft"), None, known()));
    assert!(said.contains("two posts in one place would be counted twice"), "{said}");
    ed::post(&mut job, &level, &post_type, [3.0, 3.0], Some("8 ft"), Some("8 ft"), known()).unwrap();
    ed::post(&mut job, &level, &post_type, [6.0, 3.0], Some("8 ft"), None, known()).unwrap();
    ed::pad(&mut job, &level, &pad_type, [3.0, 3.0], "18 in", "-1 ft", known()).unwrap();
    let said = refused(ed::pad(&mut job, &level, &pad_type, [3.0, 3.0], "18 in", "-1 ft", known()));
    assert!(said.contains("two pad footings in one place would be counted twice"), "{said}");
    ed::pad(&mut job, &level, &pad_type, [5.0, 3.0], "18 in", "-1 ft", known()).unwrap();
}

/// C8. A beam's weight typed with the program's own word for a foot of length is read: "18 lb/LF", "18 lbs/lf",
/// "18 lb / LF" are "18 lb/ft", 18 pounds a foot; a number with no unit is refused and the refusal gives both ways.
#[test]
fn a_weight_per_foot_typed_in_lf_is_read() {
    for typed in ["18 lb/LF", "18 lbs/lf", "18 lb / LF", "18 lb/ft", "18 plf"] {
        let mut job = Job::new("Weight (not a real job)", "test", "2026-11-23");
        let t = ed::framing_type(&mut job, "W8x18", Kind::Beam, "5.25 in", Some("8.14 in"), Some(typed), known()).unwrap_or_else(|e| panic!("{typed}: {e}"));
        let e = job.find(&t).unwrap();
        let w = job.world.get::<SegmentType>(e).unwrap().weight_per_foot.clone().unwrap();
        assert_eq!((w.value, w.unit.as_str()), (Some(18.0), "lb/ft"), "{typed}");
    }
    let mut job = Job::new("Weight (not a real job)", "test", "2026-11-23");
    let said = ed::framing_type(&mut job, "W8x18", Kind::Beam, "5.25 in", Some("8.14 in"), Some("18"), known()).unwrap_err();
    assert!(said.contains("with no unit") && said.contains("18 lb/LF"), "{said}");
}
