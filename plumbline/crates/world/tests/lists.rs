//! Call 158: the takeoff says what is built, what stays and what comes down; the three
//! add up to the line. The house is made up (no real job). Figures worked by hand in
//! STATUS.md ("Lists by what is built, kept and taken down, and by trade").

use plumbline_world::building::storey_totals;
use plumbline_world::editing as ed;
use plumbline_world::value::Status;
use plumbline_world::view::Sum;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up addition plan" }
}

/// The house of the renovation: 24 ft by 16 ft round and a partition, 6 in walls, 9 ft
/// tall, all five existing, the partition and the east wall to demolish, three new walls
/// 24,0 36,0 36,16 24,16.
fn house() -> (Job, Vec<String>) {
    let mut job = Job::new("A renovation (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
    let mut all = outer.clone();
    all.extend(part.clone());
    ed::mark(&mut job, &all, WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    ed::mark(&mut job, &[part[0].clone(), outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    let new = ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    assert_eq!(new.len(), 3);
    // A footing under each new wall, 24 in wide and 12 in deep; one under the south wall
    // as it stands, existing.
    let ft = ed::footing_type(&mut job, "footing 24 in", "24 in", known()).unwrap();
    ed::footings(&mut job, &level, &ft, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, "12 in", assumed()).unwrap();
    let under = ed::footings(&mut job, &level, &ft, &[[0.0, 0.0], [24.0, 0.0]], false, "12 in", known()).unwrap();
    ed::mark(&mut job, &under, WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    job.derive();
    (job, vec![level])
}

fn said(s: &Sum) -> f64 {
    s.value.unwrap()
}

/// Step 1 and 2: 8 walls, 136 LF as they stand; to build 3 walls 40 LF, existing to
/// remain 3 walls 64 LF, to demolish 2 walls 32 LF; the faces 1,224, 360, 576, 288 SF;
/// the footings 64 LF as they stand, 40 to build and 24 existing; each way adds up.
#[test]
fn the_takeoff_gives_the_walls_three_ways_that_add_up_to_the_line() {
    let (mut job, _) = house();
    let level = storey_totals(&mut job).remove(0);
    assert_eq!((level.walls.members, said(&level.walls)), (8, 136.0));
    let by = level.by_work.walls.clone().unwrap();
    assert_eq!([(by.to_build.members, said(&by.to_build)), (by.existing.members, said(&by.existing)), (by.demolish.members, said(&by.demolish))], [(3, 40.0), (3, 64.0), (2, 32.0)]);
    assert_eq!(said(&by.to_build) + said(&by.existing) + said(&by.demolish), said(&level.walls));
    let face = level.by_work.wall_face.clone().unwrap();
    assert_eq!(said(&level.wall_face), 1224.0);
    assert_eq!([said(&face.to_build), said(&face.existing), said(&face.demolish)], [360.0, 576.0, 288.0]);
    assert_eq!(said(&face.to_build) + said(&face.existing) + said(&face.demolish), said(&level.wall_face));
    let net = level.by_work.wall_net.clone().unwrap();
    assert_eq!(said(&net.to_build) + said(&net.existing) + said(&net.demolish), said(&level.wall_net));
    // Footings: 12 + 16 + 12 = 40 LF new, 24 LF existing; concrete 40 x 2 x 1 = 80 CF,
    // 2.963 CY; 24 x 2 x 1 = 48 CF, 1.778 CY.
    assert_eq!((level.footings.members, said(&level.footings)), (4, 64.0));
    let f = level.by_work.footings.clone().unwrap();
    assert_eq!([(f.to_build.members, said(&f.to_build)), (f.existing.members, said(&f.existing)), (f.demolish.members, f.demolish.value.map_or(0.0, |x| x))], [(3, 40.0), (1, 24.0), (0, 0.0)]);
    assert_eq!(f.demolish.value, None, "nothing to demolish is none, not a zero");
    let v = level.by_work.footing_volume.clone().unwrap();
    assert!((said(&v.to_build) - 2.963).abs() < 0.0005, "{v:?}");
    assert!((said(&v.existing) - 1.7778).abs() < 0.0005, "{v:?}");
    assert!((said(&v.to_build) + said(&v.existing) - said(&level.footing_volume)).abs() < 0.00015);
    // Nothing else on the level is marked: floors and openings are not given three ways.
    assert!(level.by_work.floors.is_none() && level.by_work.doors.is_none() && level.by_work.windows.is_none());
}

/// A job with no mark anywhere gives no split on any line.
#[test]
fn a_job_with_no_mark_is_given_no_split() {
    let mut job = Job::new("Plain (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    job.derive();
    let level = storey_totals(&mut job).remove(0);
    assert!(level.by_work.walls.is_none() && level.by_work.wall_face.is_none() && level.by_work.wall_net.is_none() && level.by_work.footings.is_none());
    // A thing marked new, or temporary, is to build, and is no reason to give a split.
    let all: Vec<String> = job.order.iter().filter_map(|&e| job.world.get::<Node>(e).filter(|n| n.path.starts_with("wall-")).map(|n| n.path.clone())).collect();
    ed::mark(&mut job, &all, WorkStatus::Temporary, ed::ItsLines::Ask, known()).unwrap();
    job.derive();
    assert!(storey_totals(&mut job).remove(0).by_work.walls.is_none());
}
