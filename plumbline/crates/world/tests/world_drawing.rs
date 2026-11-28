use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{measure, measure_of, room, figures_of, drawn, ducted, joined_of, fittings_of};

/// Casing is taken off by the opening's own size: two legs and a head. A height and a
/// width are lengths and work in a formula as any length does (M21-8: they were in
/// "ft" and a formula refused them). A door 3 ft by 7 ft: 2 x 7 + 3 = 17 LF. Corner
/// bead up both ends of a 9 ft wall: 18 LF. A volume still has no place in a formula,
/// and the refusal says what does.
#[test]
fn casing_is_taken_off_by_the_openings_own_size() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, _) = drawn();
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap().remove(0);
    let door = ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset("5 ft"), "0 ft", how).unwrap();
    job.add_line(&door, "Casing, one side", "2 * height + width", Some(Class::Material), false).unwrap();
    job.add_line(&wall, "Corner bead", "2 * height", Some(Class::Material), false).unwrap();
    job.derive();
    let quantity = |job: &mut Job, path: &str, line: &str| {
        let e = job.find(path).unwrap();
        job.world.get::<plumbline_world::LineQuantities>(e).unwrap().0[line].quantity.clone()
    };
    let casing = quantity(&mut job, &door, "Casing, one side");
    assert_eq!((casing.value, casing.unit.as_str(), casing.status), (Some(17.0), "LF", Status::Known), "{casing:?}");
    let bead = quantity(&mut job, &wall, "Corner bead");
    assert_eq!((bead.value, bead.unit.as_str()), (Some(18.0), "LF"), "{bead:?}");
    // A footing's concrete is a volume: not in a formula, and the refusal says what is.
    // A footing's type is made as a wall's is: its thickness is the footing's width.
    let ft = ed::footing_type(&mut job, "footing 24 x 12", "24 in", how).unwrap();
    let footing = ed::footings(&mut job, &s, &ft, &[[30.0, 0.0], [40.0, 0.0]], false, "12 in", how).unwrap().remove(0);
    let refused = job.add_line(&footing, "Forms", "volume * 2", Some(Class::Material), false).unwrap_err();
    assert!(refused.contains("a formula works in lengths, areas and counts") && !refused.contains("the figure it comes from"), "{refused}");
}

/// Foundation work (calls 130 and 133). A wall says how thick it is, and a floor how
/// thick and how far round; a formula whose lengths make a volume comes out in CY with
/// the ÷ 27 written in; and a formula works from a figure as it was worked, so forms on a
/// footing 10 in deep are 66.6667 SF, not the 66.664 its written 0.8333 ft would give.
/// Figures by hand: a wall 20 ft by 9 ft, 6 in thick: 180 SF, 180 x 0.5 / 27 = 3.3333 CY.
/// A slab 20 by 10, 4 in: perimeter 60 LF, 200 x 4/12 / 27 = 2.4691 CY. A footing 40 ft
/// by 24 in by 10 in: forms 2 x 40 x 10/12 = 66.6667 SF.
#[test]
fn a_foundation_is_taken_off_in_cubic_yards_from_figures_as_worked() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, _, _) = drawn();
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap().remove(0);
    let st = ed::floor_type(&mut job, "slab 4 in", "4 in", how).unwrap();
    let slab = ed::floor(&mut job, &s, &st, ed::FloorBy::Points(&[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]]), how).unwrap().path;
    let ft = ed::footing_type(&mut job, "footing 24 x 10", "24 in", how).unwrap();
    let footing = ed::footings(&mut job, &s, &ft, &[[30.0, 0.0], [70.0, 0.0]], false, "10 in", how).unwrap().remove(0);
    job.add_line(&wall, "Wall concrete", "net_area * thickness", Some(Class::Material), false).unwrap();
    job.add_line(&slab, "Slab concrete", "plan_area * thickness", Some(Class::Material), false).unwrap();
    job.add_line(&slab, "Edge form", "perimeter", Some(Class::Material), false).unwrap();
    job.add_line(&footing, "Footing forms", "2 * plan_length * depth", Some(Class::Material), false).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &wall, "thickness").value, Some(0.5));
    assert_eq!(measure(&mut job, &slab, "perimeter").value, Some(60.0));
    assert_eq!(measure(&mut job, &slab, "thickness").value, Some(0.3333));
    let quantity = |job: &mut Job, path: &str, line: &str| {
        let e = job.find(path).unwrap();
        job.world.get::<plumbline_world::LineQuantities>(e).unwrap().0[line].quantity.clone()
    };
    let concrete = quantity(&mut job, &wall, "Wall concrete");
    assert_eq!((concrete.value, concrete.unit.as_str(), concrete.formula.as_str()), (Some(3.3333), "CY", "net_area × thickness ÷ 27"), "{concrete:?}");
    let slab_cy = quantity(&mut job, &slab, "Slab concrete");
    assert_eq!((slab_cy.value, slab_cy.unit.as_str()), (Some(2.4691), "CY"), "{slab_cy:?}");
    assert_eq!(quantity(&mut job, &slab, "Edge form").value, Some(60.0));
    // The ÷ 27 is the program's: typed out of habit, it would divide twice.
    let twice = job.add_line(&footing, "Footing concrete by hand", "plan_length * width * depth / 27", Some(Class::Material), false).unwrap_err();
    assert!(twice.contains("take the ÷ 27 out"), "{twice}");
    // The footing's depth is written 0.8333 ft and worked as 10/12.
    assert_eq!(measure(&mut job, &footing, "depth").value, Some(0.8333));
    assert_eq!(quantity(&mut job, &footing, "Footing forms").value, Some(66.6667));
    // A height typed on the line in feet and inches works as typed: 20 x 3 ft 4 in is
    // 66.6667 SF. (A line waiting on a height nobody has given, Unknown in the starter
    // library, has no figure at all: demos/foundation.sh.)
    job.set_formula(&wall, Some("Wall concrete"), "plan_length * below_grade", &[("below_grade".into(), "3 ft 4 in".into())], Status::Known, "made-up grading plan").unwrap();
    job.derive();
    assert_eq!(quantity(&mut job, &wall, "Wall concrete").value, Some(66.6667));
}

/// A wall's frame, counted (call 137), from the line that names its framing. A 24 ft wall
/// at 16 in: studs centred on 16, 32 ... 272 in (17) and one at each end, 19. An opening's
/// offset is to its centre: a 3 ft door at 6 ft runs 54 to 90 in and takes the studs on
/// 64 and 80; a 4 ft window at 15 ft 6 in runs 162 to 210 in and takes those on 176, 192
/// and 208, and the one on 160 stands where its king and jack go: 13 left. A king and a
/// jack each side of each opening: 8. Cripples on the stations inside the openings: 2
/// over the door, 3 over the window and 3 under it: 8. 29 in all. Plates 3 x 24 = 72 LF;
/// headers jack to jack 3.25 + 4.25 = 7.5 LF; a rough sill under the window, 4 LF. (The
/// first count by hand took the offset to the opening's near edge and got 30.) A wall
/// nothing names a framing for has none of the four.
#[test]
fn a_walls_frame_is_counted_from_the_line_that_names_its_framing() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    // `drawn` gives a door 3 ft by 7 ft and a window 4 ft by 3 ft.
    let (mut job, s, wt, dt, nt) = drawn();
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [24.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap().remove(0);
    ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset("6 ft"), "0 ft", how).unwrap();
    ed::opening(&mut job, &wall, Kind::Window, &nt, ed::OnWall::Offset("15 ft 6 in"), "3 ft", how).unwrap();
    job.derive();
    let e = job.find(&wall).unwrap();
    assert!(!job.world.get::<Measure>(e).unwrap().0.contains_key("studs"), "nothing names its framing yet");
    job.add_line(&wall, "Studs, 2x6 at 16 in", "studs", Some(Class::Material), false).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &wall, "studs").value, Some(29.0));
    assert_eq!(measure(&mut job, &wall, "plate_length").value, Some(72.0));
    assert_eq!(measure(&mut job, &wall, "header_length").value, Some(7.5));
    assert_eq!(measure(&mut job, &wall, "sill_length").value, Some(4.0));
    // The count rests on a rule (the spacing the line names; corners left to the
    // allowance): Assumed, though the wall is Known.
    assert_eq!(measure(&mut job, &wall, "studs").status, Status::Assumed);
    // A window whose head is in the top plates has no room for a header: the frame is
    // refused, and says why, as is one that does not fit its wall (judged 2026-11-18).
    let high = ed::walls(&mut job, &s, &wt, &[[0.0, 10.0], [12.0, 10.0]], false, ed::Top::Height("9 ft"), how).unwrap().remove(0);
    ed::opening(&mut job, &high, Kind::Window, &nt, ed::OnWall::Offset("6 ft"), "6 ft", how).unwrap();
    job.add_line(&high, "Studs, 2x6 at 16 in", "studs", Some(Class::Material), false).unwrap();
    job.derive();
    let refused = measure(&mut job, &high, "header_length");
    assert!(refused.value.is_none() && refused.note.contains("top plates"), "{refused:?}");
}

#[test]
fn a_house_is_drawn_from_nothing_and_its_figures_are_the_ones_worked_by_hand() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, nt) = drawn();
    // Four walls round 20 ft by 10 ft, 9 ft tall, closed back to the start.
    let made = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]], true, ed::Top::Height("9 ft"), how).unwrap();
    assert_eq!(made.len(), 4);
    // Each wall is named for its kind, numbered from 1 on its storey (call 52).
    let names: Vec<String> = made.iter().map(|p| job.find(p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap()).collect();
    assert_eq!(names, ["wall 1", "wall 2", "wall 3", "wall 4"]);
    // South wall: 20 LF, 9 ft, face 180 SF.
    assert_eq!(measure_of(&mut job, &made[0], "plan_length").value, Some(20.0));
    assert_eq!(measure_of(&mut job, &made[0], "face_area").value, Some(180.0));
    // A door 5 ft along the south wall, and a window where a point falls along the east wall:
    // (20, 6) is 6 ft from that wall's first end at (20, 0).
    let door = ed::opening(&mut job, &made[0], Kind::Door, &dt, ed::OnWall::Offset("5 ft"), "0 ft", how).unwrap();
    let window = ed::opening(&mut job, &made[1], Kind::Window, &nt, ed::OnWall::Point([20.0, 6.0]), "3 ft", how).unwrap();
    assert_eq!(job.find(&window).and_then(|e| job.world.get::<plumbline_world::Opening>(e).map(|o| o.offset.feet())).unwrap(), Some(6.0));
    // South wall net of the door: 180 − 3 × 7 = 159. East wall, 10 × 9 = 90, less 4 × 3 = 78.
    assert_eq!(measure_of(&mut job, &made[0], "net_area").value, Some(159.0));
    assert_eq!(measure_of(&mut job, &made[1], "net_area").value, Some(78.0));
    assert_eq!(job.kind(&door), Some(Kind::Door));
    // A room in it: 19.5 by 9.5 inside the 6 in walls, 185.25 SF (the box of the room tests).
    let r = room(&mut job, &s, "room", [5.0, 5.0]);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(185.25), Some(200.0), Some(58.0)]);
    // A supply duct, 6 in round, two straight pieces at 8 ft, on a system: 12 + 5 = 17 LF.
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    let supply = ed::system(&mut job, "Supply air", "supply air").unwrap();
    let pieces = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[2.0, 2.0], [14.0, 2.0], [14.0, 7.0]], "8 ft", Some(&supply), how).unwrap();
    assert_eq!(pieces.len(), 2);
    assert_eq!(measure_of(&mut job, &pieces[0], "true_length").value, Some(12.0));
    assert_eq!(measure_of(&mut job, &pieces[1], "true_length").value, Some(5.0));
    // A register stood on it.
    let reg = ed::place(&mut job, &s, Kind::Fixture, "Ceiling register", [14.0, 7.0], "8 ft", Some([1.0, 1.0, 0.25]), Some(&supply), how).unwrap();
    assert_eq!(job.kind(&reg), Some(Kind::Fixture));
    // It saves, loads to the same bytes, and the takeoff has the walls and the duct.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
    let totals = plumbline_world::building::storey_totals(&mut job);
    assert_eq!(totals[0].walls.value, Some(60.0)); // 20 + 10 + 20 + 10
    // Everything a verb wrote carries the status and source it was given.
    let how = ed::Entered { status: Status::Assumed, source: "set in the 3D view" };
    let more = ed::walls(&mut job, &s, &wt, &[[10.0, 0.0], [10.0, 10.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let w = job.find(&more[0]).and_then(|e| job.world.get::<plumbline_world::Wall>(e).cloned()).unwrap();
    assert_eq!((w.line.status, w.line.source.as_str(), w.height.unwrap().status, w.type_name.status), (Status::Assumed, "set in the 3D view", Status::Assumed, Status::Assumed));
    // The room is now the west half, and no surer than the wall that made it so:
    // 9.5 by 9.5 = 90.25 SF, Assumed.
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(90.25), Status::Assumed));
}

#[test]
fn what_cannot_be_drawn_is_refused_and_leaves_the_job_as_it_was() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, nt) = drawn();
    let made = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let before = save(&mut job);
    let refused = |job: &mut Job, got: Result<String, String>, want: &str| {
        let why = got.expect_err(want);
        assert!(why.contains(want), "wanted {want:?}, got {why:?}");
        assert_eq!(save(job), before, "a refusal changed the job: {want}");
    };
    let none = |r: Result<Vec<String>, String>| r.map(|v| v.join(","));
    // One point is not a wall; a wall of no length; a height that is not a length; a
    // wall with a type that is not a wall type; on something that is not a storey.
    let got = none(ed::walls(&mut job, &s, &wt, &[[0.0, 0.0]], false, ed::Top::Height("9 ft"), how));
    refused(&mut job, got, "at least 2 points");
    // The second of three walls has no length: the first is not left behind.
    let got = none(ed::walls(&mut job, &s, &wt, &[[0.0, 20.0], [5.0, 20.0], [5.0, 20.0], [9.0, 20.0]], false, ed::Top::Height("9 ft"), how));
    refused(&mut job, got, "wall line");
    let got = none(ed::walls(&mut job, &s, &wt, &[[0.0, 20.0], [5.0, 20.0]], false, ed::Top::Height("9"), how));
    refused(&mut job, got, "9");
    let got = none(ed::walls(&mut job, &s, &dt, &[[0.0, 20.0], [5.0, 20.0]], false, ed::Top::Height("9 ft"), how));
    refused(&mut job, got, "type");
    let got = none(ed::walls(&mut job, &wt, &wt, &[[0.0, 20.0], [5.0, 20.0]], false, ed::Top::Height("9 ft"), how));
    refused(&mut job, got, "not a level");
    let got = none(ed::walls(&mut job, &s, &wt, &[[0.0, 20.0], [5.0, 20.0]], false, ed::Top::Level("storey-9999"), how));
    refused(&mut job, got, "storey-9999");
    // A point that is not along the wall; a door of a window's type; on what is not a wall.
    let got = ed::opening(&mut job, &made[0], Kind::Door, &dt, ed::OnWall::Point([25.0, 0.0]), "0 ft", how);
    refused(&mut job, got, "it is not on the wall");
    let got = ed::opening(&mut job, &made[0], Kind::Door, &nt, ed::OnWall::Offset("5 ft"), "0 ft", how);
    refused(&mut job, got, "type");
    let got = ed::opening(&mut job, &s, Kind::Door, &dt, ed::OnWall::Offset("5 ft"), "0 ft", how);
    refused(&mut job, got, "is not a wall");
    // A run of a kind that is not a run; on a system that is not there.
    let got = none(ed::runs(&mut job, &s, Kind::Wall, &wt, &[[0.0, 0.0], [5.0, 0.0]], "8 ft", None, how));
    refused(&mut job, got, "is not a duct");
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    let before = save(&mut job);
    let refused = |job: &mut Job, got: Result<String, String>, want: &str| {
        let why = got.expect_err(want);
        assert!(why.contains(want), "wanted {want:?}, got {why:?}");
        assert_eq!(save(job), before, "a refusal changed the job: {want}");
    };
    let got = none(ed::runs(&mut job, &s, Kind::Duct, &duct, &[[0.0, 0.0], [5.0, 0.0]], "8 ft", Some("system-9999"), how));
    refused(&mut job, got, "system-9999");
    let got = ed::system(&mut job, "Steam", "steam");
    refused(&mut job, got, "is not a kind of system");
    // A type in use cannot be removed; a storey with walls on it cannot.
    let got = none(ed::remove(&mut job, &wt));
    refused(&mut job, got, "type");
    let got = none(ed::remove(&mut job, &s));
    refused(&mut job, got, "under it: delete those first");
}

#[test]
fn a_wall_moved_takes_its_openings_and_a_wall_removed_takes_them_too() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, _nt) = drawn();
    let made = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let door = ed::opening(&mut job, &made[0], Kind::Door, &dt, ed::OnWall::Offset("5 ft"), "0 ft", how).unwrap();
    // Moved 3 ft east and 2 ft north by somebody who is not sure: the wall's line is where
    // it was plus that, and Assumed; its door is still 5 ft along it; its figures hold.
    let moving = ed::Entered { status: Status::Assumed, source: "set in the 3D view" };
    ed::move_by(&mut job, &made[0], 3.0, 2.0, moving).unwrap();
    let w = job.find(&made[0]).and_then(|e| job.world.get::<plumbline_world::Wall>(e).cloned()).unwrap();
    assert_eq!(w.line.value, Some([[3.0, 2.0], [23.0, 2.0]]));
    assert_eq!((w.line.status, w.line.source.as_str()), (Status::Assumed, "made-up plan; moved (set in the 3D view)"));
    assert_eq!(job.find(&door).and_then(|e| job.world.get::<plumbline_world::Opening>(e).map(|o| o.offset.feet())).unwrap(), Some(5.0));
    assert_eq!(measure_of(&mut job, &made[0], "net_area").value, Some(159.0));
    // A move by a Known amount of something Known keeps its status and adds to its source.
    ed::walls(&mut job, &s, &wt, &[[0.0, 20.0], [20.0, 20.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    // What is not moved this way says so.
    assert!(ed::move_by(&mut job, &wt, 1.0, 0.0, how).unwrap_err().contains("is not moved this way"));
    assert!(ed::move_by(&mut job, &made[0], f64::NAN, 0.0, how).unwrap_err().contains("is not a figure"));
    // Removed: the wall and its door, and nothing else.
    let before = job.index().len();
    let gone = ed::remove(&mut job, &made[0]).unwrap();
    assert_eq!(gone, [door.clone(), made[0].clone()]);
    assert_eq!(job.index().len(), before - 2);
    assert!(job.find(&door).is_none());
}

#[test]
fn a_point_picked_on_the_plan_snaps_to_what_is_there() {
    use plumbline_world::editing::snap::{corner, scene, snap, Reach, Scene, SnapKind, Snaps};
    // A plan written out by hand: wall A along the east axis from (0, 0) to (20, 0); wall B
    // across it from (10, -5) to (10, 5); wall C from (30, 0) to (40, 0). Each 6 in thick.
    let mut plan = Scene::default();
    for (name, a, b) in [("A", [0.0, 0.0], [20.0, 0.0]), ("B", [10.0, -5.0], [10.0, 5.0]), ("C", [30.0, 0.0], [40.0, 0.0])] {
        plan.segment(a, b, Some(name.to_string()));
        plan.walls.push((a, b, 0.5, name.to_string()));
    }
    let reach = Reach { point: 0.5, line: 0.5 };
    let all = Snaps::default();
    let at = |p: [f64; 2], from: &[[f64; 2]], square: bool, o: &Snaps| snap(&plan, p, reach, from, square, o);
    // Near an end: the end, and whose.
    let f = at([0.2, 0.1], &[], false, &all);
    assert_eq!((f.at, f.kind, f.source.as_deref()), ([0.0, 0.0], SnapKind::Endpoint, Some("A")));
    // Where A and B cross is also the middle of each: a crossing ranks before a middle.
    let f = at([10.1, 0.1], &[], false, &all);
    assert_eq!((f.at, f.kind), ([10.0, 0.0], SnapKind::Intersection));
    // The middle of C, where nothing crosses.
    let f = at([35.2, 0.1], &[], false, &all);
    assert_eq!((f.at, f.kind, f.source.as_deref()), ([35.0, 0.0], SnapKind::Midpoint, Some("C")));
    // The foot of the perpendicular from the last point, (5, 4), onto A is (5, 0): straight down.
    let f = at([5.1, 0.2], &[[5.0, 4.0]], false, &all);
    assert_eq!((f.at, f.kind), ([5.0, 0.0], SnapKind::Perpendicular));
    assert_eq!(f.guide.map(|g| g.angle), Some(270.0));
    // Away from everything, a direction within 4 degrees of a guide goes onto it: from
    // (0, 10) towards (8, 10.3) is 2.1 degrees off east, so the point is 8 ft due east.
    let f = at([8.0, 10.3], &[[0.0, 10.0]], false, &all);
    assert_eq!((f.at, f.kind, f.guide.map(|g| g.angle)), ([8.0, 10.0], SnapKind::Angle, Some(0.0)));
    // Too far off any guide (7.1 degrees), or with no point before it: the nearest inch.
    // 8.02 ft is 96.24 in, so 96 in; 10.31 ft is 123.72 in, so 124 in, 10.3333 ft.
    let f = at([8.0, 11.0], &[[0.0, 10.0]], false, &all);
    assert_eq!((f.at, f.kind), ([8.0, 11.0], SnapKind::Free));
    let f = at([8.02, 10.31], &[], false, &all);
    assert_eq!((f.at, f.kind), ([8.0, 10.3333], SnapKind::Free));
    // Beside a wall and near none of its points: the nearest inch along its line.
    // (3.03, 0.4) is 0.15 ft off A's face; 3.03 ft along is 36.36 in, so 36 in.
    let f = at([3.03, 0.4], &[], false, &all);
    assert_eq!((f.at, f.kind, f.source.as_deref()), ([3.0, 0.0], SnapKind::Nearest, Some("A")));
    // Farther than the reach from its face: not the wall's.
    assert_eq!(at([3.03, 0.9], &[], false, &all).kind, SnapKind::Free);
    // A square step keeps to east or north of the last point, whichever is nearer.
    let f = at([9.0, 10.4], &[[2.0, 10.0]], true, &Snaps::none());
    assert_eq!((f.at, f.kind), ([9.0, 10.0], SnapKind::Free));
    let f = at([2.3, 14.0], &[[2.0, 10.0]], true, &Snaps::none());
    assert_eq!((f.at, f.kind), ([2.0, 14.0], SnapKind::Free));
    // With the snaps off, an end is not snapped to.
    let f = at([0.2, 0.1], &[], false, &Snaps::none());
    assert_eq!((f.at, f.kind), ([0.1667, 0.0833], SnapKind::Free));
    // A guide ends where it crosses a wall: from (15, 5) due south it meets A at (15, 0).
    let f = at([15.05, 0.2], &[[15.0, 5.0]], false, &Snaps { perpendicular: false, ..all });
    assert_eq!((f.at, f.kind, f.source.as_deref()), ([15.0, 0.0], SnapKind::Intersection, Some("A")));
    // The points picked so far are ends too: back to where the run began.
    let f = at([2.1, 9.9], &[[2.0, 10.0], [6.0, 10.0]], false, &all);
    assert_eq!((f.at, f.kind, f.source), ([2.0, 10.0], SnapKind::Endpoint, None));
    // The corner a new line makes with the one before: a right angle, and straight on.
    assert_eq!(corner([0.0, 0.0], [10.0, 0.0], [10.0, 5.0]), Some(90.0));
    assert_eq!(corner([0.0, 0.0], [10.0, 0.0], [15.0, 0.0]), Some(180.0));
    assert_eq!(corner([0.0, 0.0], [0.0, 0.0], [15.0, 0.0]), None);

    // The scene of a job: the walls and runs entered on the storey, and where things stand.
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, _dt, _nt) = drawn();
    let made = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    let run = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[2.0, 2.0], [14.0, 2.0]], "8 ft", None, how).unwrap();
    let thing = ed::place(&mut job, &s, Kind::Fixture, "Register", [14.0, 7.0], "8 ft", None, None, how).unwrap();
    let plan = scene(&mut job, &s, None);
    assert_eq!(plan.walls.len(), 2);
    assert_eq!(plan.walls[0], ([0.0, 0.0], [20.0, 0.0], 0.5, made[0].clone()));
    let f = snap(&plan, [19.8, 0.2], reach, &[], false, &all);
    assert_eq!((f.at, f.kind), ([20.0, 0.0], SnapKind::Endpoint));
    let f = snap(&plan, [8.1, 2.1], reach, &[], false, &all);
    assert_eq!((f.at, f.kind, f.source), ([8.0, 2.0], SnapKind::Midpoint, Some(run[0].clone())));
    let f = snap(&plan, [14.1, 7.1], reach, &[], false, &all);
    assert_eq!((f.at, f.kind, f.source), ([14.0, 7.0], SnapKind::Centre, Some(thing)));
    // What is being moved does not snap to where it was.
    let without = scene(&mut job, &s, Some(&made[0]));
    assert_eq!(without.walls.len(), 1);
    assert_eq!(snap(&without, [0.2, 0.1], reach, &[], false, &all).kind, SnapKind::Free);
}

// ---- The eleventh judgment: each finding, as a test that fails without its fix ----

#[test]
fn a_fitting_is_worked_out_from_the_runs_that_meet_and_a_run_is_joined_to_a_thing() {
    use plumbline_world::editing as ed;
    use plumbline_world::{ConnectorKind, Connectors, Fitting, FittingKind, Placed};
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, duct, air, pieces) = ducted();
    // Two pieces end to end: one network, open at its two far ends, nothing met mid-run.
    assert_eq!(joined_of(&mut job, "Supply air"), (1, 2, 0));
    // Asked for as a tee, the corner is refused: two ends at a right angle make a bend.
    let before = save(&mut job);
    let why = ed::fitting(&mut job, &s, [10.0, 2.0], None, Some(FittingKind::Tee), how).unwrap_err();
    assert_eq!(why, "what meets at (10, 2) makes a bend, not a tee: 2 runs' ends meet there");
    assert_eq!(save(&mut job), before);
    let bend = ed::fitting(&mut job, &s, [10.0, 2.0], None, None, how).unwrap();
    assert_eq!((bend.kind, bend.cut.len()), (FittingKind::Bend, 0));
    let e = job.find(&bend.path).unwrap();
    let placed = job.world.get::<Placed>(e).cloned().unwrap();
    // It is its run's size, 6 in each way. Its middle is on the joint, 8 ft up, so its
    // base is 3 in under that: 7.75 ft.
    assert_eq!(placed.what.value.as_deref(), Some("6 in round elbow"));
    assert_eq!((placed.at.value, placed.offset.feet(), placed.size.value), (Some([10.0, 2.0]), Some(7.75), Some([6.0, 6.0, 6.0])));
    // It is as sure as the runs, which are Known. The size it is drawn at is Assumed, and
    // its source says the part is not selected.
    assert_eq!((placed.what.status, placed.size.status), (Status::Known, Status::Assumed));
    // (Call 52: "not specified", the word a specification uses.)
    assert!(placed.what.source.contains("which elbow is bought is not specified"), "{}", placed.what.source);
    assert_eq!(job.world.get::<Connectors>(e).unwrap().0.len(), 2);
    assert_eq!(job.world.get::<Fitting>(e).unwrap().kind, FittingKind::Bend);
    // The joint has its fitting: a second is refused. An open end takes none.
    assert!(ed::fitting(&mut job, &s, [10.0, 2.0], None, None, how).unwrap_err().contains("is already the fitting"));
    assert_eq!(ed::fitting(&mut job, &s, [10.0, 14.0], None, None, how).unwrap_err(), format!("only {} ends at (10, 14): a fitting joins two, three or four runs' ends", pieces[1]));
    assert_eq!(ed::fitting(&mut job, &s, [20.0, 20.0], None, None, how).unwrap_err(), "nothing meets at (20, 20): a fitting goes where runs meet");

    // A branch, drawn in the window and so Assumed, from the middle of the first piece:
    // (6, 2) to (6, 10). It meets that piece along its length.
    let seen = ed::Entered { status: Status::Assumed, source: "set in the 3D view" };
    let branch = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[6.0, 2.0], [6.0, 10.0]], "8 ft", Some(&air), seen).unwrap();
    assert_eq!(joined_of(&mut job, "Supply air"), (1, 3, 1));
    // A tee there cuts the first piece in two at (6, 2): its 8 LF is 4 and 4.
    let tee = ed::fitting(&mut job, &s, [6.0, 2.0], None, None, how).unwrap();
    assert_eq!(tee.kind, FittingKind::Tee);
    assert_eq!(tee.cut.iter().map(|c| c.0.as_str()).collect::<Vec<_>>(), [pieces[0].as_str()]);
    assert_eq!(measure_of(&mut job, &pieces[0], "true_length").value, Some(4.0));
    assert_eq!(measure_of(&mut job, &tee.cut[0].1, "true_length").value, Some(4.0));
    assert_eq!(joined_of(&mut job, "Supply air"), (1, 3, 0));
    let e = job.find(&tee.path).unwrap();
    assert_eq!(job.world.get::<Connectors>(e).unwrap().0.len(), 3);
    // The tee joins an Assumed run, so it is Assumed, though it was asked for as Known.
    assert_eq!(job.world.get::<Placed>(e).unwrap().what.status, Status::Assumed);
    // The takeoff: 8 + 12 + 8 = 28 LF of duct, as before the cut; one elbow, one tee.
    let t = plumbline_world::building::takeoff(&mut job);
    assert_eq!(t.iter().find(|x| x.name == "Supply air").unwrap().runs.value, Some(28.0));
    assert_eq!(fittings_of(&mut job, "Supply air"), [("Bend".to_string(), "6 in round elbow".to_string(), 1, Status::Known), ("Tee".to_string(), "6 in round tee".to_string(), 1, Status::Assumed)]);

    // An air handler 2 ft square and 4 ft tall, its base 6 ft up, stands over the duct's
    // first end at (2, 2), 8 ft up: the end is inside it.
    let ahu = ed::place(&mut job, &s, Kind::Equipment, "Air handler", [2.0, 2.0], "6 ft", Some([2.0, 2.0, 4.0]), Some(&air), how).unwrap();
    assert_eq!(ed::connect(&mut job, &ahu, &pieces[0], how).unwrap(), 1);
    // Its connector is where the duct ends, in the handler's own axes: on its middle,
    // 2 ft (24 in) above its base.
    let c = job.find(&ahu).and_then(|e| job.world.get::<Connectors>(e).cloned()).unwrap();
    assert_eq!((c.0[0].at.value, c.0[0].at.unit.as_str(), c.0[0].kind, c.0[0].domain), (Some([0.0, 0.0, 24.0]), "in", ConnectorKind::Duct, None));
    // One open end fewer: the far end of the second piece and the end of the branch.
    assert_eq!(joined_of(&mut job, "Supply air"), (1, 2, 0));
    assert!(ed::connect(&mut job, &ahu, &pieces[0], how).unwrap_err().contains("already has a connection where"));
    assert!(ed::connect(&mut job, &ahu, &branch[0], how).unwrap_err().starts_with(&format!("neither end of {} is at {ahu}", branch[0])));
    assert!(ed::connect(&mut job, &bend.path, &pieces[1], how).unwrap_err().contains("is a fitting"));
    // It saves and loads to the same bytes.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
}

#[test]
fn every_joint_of_a_system_is_fitted_and_what_is_passed_over_is_said() {
    use plumbline_world::editing as ed;
    use plumbline_world::{Connectors, Domain, FittingKind};
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    let big = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    let small = ed::segment_type(&mut job, "1/2 in", Kind::Pipe, "1/2 in", None, how).unwrap();
    let cold = ed::system(&mut job, "Cold water", "domestic cold water").unwrap();
    // A 3/4 in main east along y = 1, drawn in two pieces that meet in line at (5, 1). A
    // 1/2 in pipe carries on in line from (9, 1) to (12, 1) and turns north to (12, 6).
    let main = ed::runs(&mut job, &s, Kind::Pipe, &big, &[[0.0, 1.0], [5.0, 1.0], [9.0, 1.0]], "1 ft", Some(&cold), how).unwrap();
    let rest = ed::runs(&mut job, &s, Kind::Pipe, &small, &[[9.0, 1.0], [12.0, 1.0], [12.0, 6.0]], "1 ft", Some(&cold), how).unwrap();
    let (made, passed) = ed::fit_all(&mut job, Some(&cold), how).unwrap();
    // At (9, 1) two sizes meet in line: a transition. At (12, 1) a bend. At (5, 1) two
    // pieces of one size meet in line, which is one run drawn in two: passed over.
    let mut kinds: Vec<&str> = made.iter().map(|f| f.kind.title()).collect();
    kinds.sort();
    assert_eq!(kinds, ["Bend", "Transition"]);
    assert_eq!(passed.len(), 1);
    assert_eq!((passed[0].storey.as_str(), passed[0].at), (s.as_str(), [5.0, 1.0, 1.0]));
    assert!(passed[0].why.contains("a coupling is put only where it is asked for"), "{}", passed[0].why);
    assert_eq!(fittings_of(&mut job, "Cold water"), [("Bend".to_string(), "1/2 in elbow".to_string(), 1, Status::Known), ("Transition".to_string(), "3/4 in to 1/2 in transition".to_string(), 1, Status::Known)]);
    // Asked for by name, the coupling goes in; and then there is nothing left to fit.
    let coupling = ed::fitting(&mut job, &s, [5.0, 1.0], None, Some(FittingKind::Coupling), how).unwrap();
    assert_eq!(coupling.kind, FittingKind::Coupling);
    assert_eq!(ed::fit_all(&mut job, Some(&cold), how).unwrap(), (Vec::new(), Vec::new()));
    assert_eq!(fittings_of(&mut job, "Cold water").len(), 3);
    // A lavatory 2 ft by 1.5 ft and 3 ft tall on the floor at (12, 6): the pipe's end, a
    // foot up, is inside it. Joined, its connector says what the pipe's system carries.
    let lav = ed::place(&mut job, &s, Kind::Fixture, "Lavatory", [12.0, 6.0], "0 ft", Some([2.0, 1.5, 3.0]), Some(&cold), how).unwrap();
    assert_eq!(ed::connect(&mut job, &lav, &rest[1], how).unwrap(), 1);
    let c = job.find(&lav).and_then(|e| job.world.get::<Connectors>(e).cloned()).unwrap();
    assert_eq!((c.0[0].domain, c.0[0].name.as_str(), c.0[0].at.value), (Some(Domain::Cold), "Domestic Cold Water", Some([0.0, 0.0, 12.0])));
    // One network, open only where the main starts.
    assert_eq!(joined_of(&mut job, "Cold water"), (1, 1, 0));
    let _ = main;
}

#[test]
fn a_run_is_cut_in_two_and_a_cut_that_would_count_a_line_twice_is_refused() {
    use plumbline_world::editing as ed;
    use plumbline_world::{InSystem, Segment};
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, duct, air, pieces) = ducted();
    // A point off the run, and a point at its end: refused, and the job is as it was.
    let before = save(&mut job);
    assert_eq!(ed::split(&mut job, &pieces[0], [6.0, 3.0], how).unwrap_err(), format!("(6, 3) is not on {}", pieces[0]));
    assert_eq!(ed::split(&mut job, &pieces[0], [2.0, 2.0], how).unwrap_err(), format!("cut {} between its ends", pieces[0]));
    assert_eq!(save(&mut job), before);
    // The second piece, 12 LF from (10, 2) to (10, 14), cut at (10, 6) by someone looking
    // at the window: 4 LF and 8 LF, both on the system, both lines now Assumed.
    let seen = ed::Entered { status: Status::Assumed, source: "set in the 3D view" };
    let new = ed::split(&mut job, &pieces[1], [10.0, 6.0], seen).unwrap();
    assert_eq!(measure_of(&mut job, &pieces[1], "true_length").value, Some(4.0));
    assert_eq!(measure_of(&mut job, &new, "true_length").value, Some(8.0));
    let part = |job: &mut Job, p: &str| job.find(p).map(|e| (job.world.get::<Segment>(e).cloned().unwrap(), job.world.get::<InSystem>(e).and_then(|s| s.0.value.clone()))).unwrap();
    let (first, second) = (part(&mut job, &pieces[1]), part(&mut job, &new));
    assert_eq!((first.0.line.value, second.0.line.value), (Some([[10.0, 2.0], [10.0, 6.0]]), Some([[10.0, 6.0], [10.0, 14.0]])));
    assert_eq!((first.0.line.status, second.0.line.status, first.0.line.source.as_str()), (Status::Assumed, Status::Assumed, "made-up plan; cut (set in the 3D view)"));
    assert_eq!((first.1.as_deref(), second.1.as_deref()), (Some(air.as_str()), Some(air.as_str())));
    assert_eq!(first.0.type_name, second.0.type_name);
    // A run that climbs from 8 ft to 9 ft over 10 ft, cut 4 ft along: 8.4 ft at the cut.
    let climbing = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[0.0, 20.0], [10.0, 20.0]], "8 ft", Some(&air), how).unwrap();
    let e = job.find(&climbing[0]).unwrap();
    let mut seg = job.world.get::<Segment>(e).cloned().unwrap();
    seg.end_offset = Some(Val::known(Len::One(9.0), "ft", "made-up plan"));
    job.put(&climbing[0], seg).unwrap();
    job.derive();
    let upper = ed::split(&mut job, &climbing[0], [4.0, 20.0], how).unwrap();
    let (lower, upper) = (part(&mut job, &climbing[0]).0, part(&mut job, &upper).0);
    assert_eq!((lower.offsets_ft(), upper.offsets_ft()), (Some((8.0, 8.4)), Some((8.4, 9.0))));
    assert_eq!(lower.end_offset.as_ref().map(|v| v.status), Some(Status::Known));
    // A cost line with a typed quantity: four hangers on the whole run. Cut in two, each
    // piece would carry four. Refused, and the reason names the line.
    job.add_line(&new, "hangers", "4", None, false).unwrap();
    job.derive();
    let before = save(&mut job);
    let why = ed::split(&mut job, &new, [10.0, 10.0], how).unwrap_err();
    assert!(why.contains("\"hangers\"") && why.contains("typed number (4)") && why.contains("counted on both pieces"), "{why}");
    assert_eq!(save(&mut job), before);
}
