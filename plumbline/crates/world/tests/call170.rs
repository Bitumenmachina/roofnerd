//! Call 170: a wall moved, and the walls that meet it follow. The house is made up (no
//! real job); every figure is worked by hand in STATUS.md ("A wall moved, and the walls
//! that meet it follow").

use plumbline_world::editing as ed;
use plumbline_world::file::save;
use plumbline_world::value::Status;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn design() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "design" }
}
fn measure(job: &mut Job, path: &str, name: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0[name].clone()
}
fn line(job: &mut Job, path: &str) -> [[f64; 2]; 2] {
    let e = job.find(path).unwrap();
    job.world.get::<Wall>(e).unwrap().line_ft().unwrap()
}
fn long(job: &mut Job, path: &str) -> f64 {
    let [a, b] = line(job, path);
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
}
fn offset(job: &mut Job, path: &str) -> f64 {
    let e = job.find(path).unwrap();
    job.world.get::<Opening>(e).unwrap().offset.feet().unwrap()
}

struct House {
    job: Job,
    level: String,
    south: String,
    east: String,
    north: String,
    west: String,
    part: String,
    door: String,
    window: String,
    rooms: [String; 2],
}

/// 6 in walls 9 ft tall: south 0,0 to 24,0; east 24,0 to 24,16; north 24,16 to 0,16; west
/// 0,16 to 0,0; a partition 12,0 to 12,16. A 3 ft door in the south wall spanning east 18 to
/// 21, a 3 ft window in the north wall spanning east 18 down to 15. Rooms West and East.
fn house() -> House {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap()[0].clone();
    let dt = ed::opening_type(&mut job, "door 3 ft", Kind::Door, "3 ft", "7 ft", known()).unwrap();
    let wt = ed::opening_type(&mut job, "window 3 ft", Kind::Window, "3 ft", "5 ft", known()).unwrap();
    let door = ed::opening(&mut job, &outer[0], Kind::Door, &dt, ed::OnWall::Offset("19.5 ft"), "0 ft", known()).unwrap();
    let window = ed::opening(&mut job, &outer[2], Kind::Window, &wt, ed::OnWall::Offset("7.5 ft"), "3 ft", known()).unwrap();
    let west = ed::room(&mut job, &level, "West", [6.0, 8.0], known()).unwrap();
    let east = ed::room(&mut job, &level, "East", [18.0, 8.0], known()).unwrap();
    job.derive();
    House { job, level, south: outer[0].clone(), east: outer[1].clone(), north: outer[2].clone(), west: outer[3].clone(), part, door, window, rooms: [west, east] }
}

fn areas(h: &mut House) -> (f64, f64) {
    h.job.derive();
    let (a, b) = (h.rooms[0].clone(), h.rooms[1].clone());
    (measure(&mut h.job, &a, "floor_area").value.unwrap(), measure(&mut h.job, &b, "floor_area").value.unwrap())
}
fn walls_lf(h: &mut House) -> f64 {
    [h.south.clone(), h.east.clone(), h.north.clone(), h.west.clone(), h.part.clone()].iter().map(|w| long(&mut h.job, w)).sum()
}

#[test]
fn the_east_wall_moved_four_feet_east_takes_the_south_and_north_walls_ends() {
    let mut h = house();
    assert_eq!(areas(&mut h), (178.25, 178.25));
    assert_eq!(walls_lf(&mut h), 96.0);
    let east = h.east.clone();
    let moved = ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    assert_eq!(line(&mut h.job, &h.east), [[28.0, 0.0], [28.0, 16.0]]);
    assert_eq!(line(&mut h.job, &h.south), [[0.0, 0.0], [28.0, 0.0]]);
    assert_eq!(line(&mut h.job, &h.north), [[28.0, 16.0], [0.0, 16.0]]);
    assert_eq!((long(&mut h.job, &h.south), long(&mut h.job, &h.north)), (28.0, 28.0));
    assert_eq!(line(&mut h.job, &h.west), [[0.0, 16.0], [0.0, 0.0]]);
    assert_eq!(line(&mut h.job, &h.part), [[12.0, 0.0], [12.0, 16.0]]);
    // The window stays where it is in the house, east 18 down to 15: its centre 16.5, so 11.5 along from 28.
    assert_eq!(offset(&mut h.job, &h.window), 11.5);
    assert_eq!(offset(&mut h.job, &h.door), 19.5);
    assert_eq!(areas(&mut h), (178.25, 240.25));
    assert_eq!(walls_lf(&mut h), 104.0);
    // It names the two walls and by how much.
    assert_eq!(moved.followed.len(), 2);
    assert!(moved.followed.iter().all(|f| (f.grew_by - 4.0).abs() < 1e-9 && !f.turned));
    let paths: Vec<&str> = moved.followed.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, [h.south.as_str(), h.north.as_str()]);
    let said = moved.said().join("\n");
    assert!(said.contains("wall 1 followed: 4 ft longer") && said.contains("wall 3 followed: 4 ft longer"), "{said}");
}

#[test]
fn the_partition_slid_sideways_keeps_its_ends_on_the_walls_it_meets() {
    let mut h = house();
    let (east, part) = (h.east.clone(), h.part.clone());
    ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    let before = (line(&mut h.job, &h.south), line(&mut h.job, &h.north), line(&mut h.job, &h.east), line(&mut h.job, &h.west));
    let moved = ed::move_wall(&mut h.job, &part, -2.0, 0.0, known(), false).unwrap();
    assert_eq!(line(&mut h.job, &h.part), [[10.0, 0.0], [10.0, 16.0]]);
    assert_eq!(long(&mut h.job, &h.part), 16.0);
    assert_eq!(areas(&mut h), (147.25, 271.25));
    // No other wall changes.
    assert_eq!(before, (line(&mut h.job, &h.south), line(&mut h.job, &h.north), line(&mut h.job, &h.east), line(&mut h.job, &h.west)));
    assert!(moved.followed.is_empty());
    assert_eq!(moved.held_on, [h.south.clone(), h.north.clone()]);
}

#[test]
fn a_partition_pushed_along_its_own_length_does_not_move_and_is_refused() {
    let mut h = house();
    let part = h.part.clone();
    let before = save(&mut h.job);
    let why = ed::move_wall(&mut h.job, &part, 0.0, 3.0, known(), false).unwrap_err();
    assert!(why.contains("wall 5 does not move") && why.contains("held by the walls they meet (wall 1 and wall 3)"), "{why}");
    assert_eq!(save(&mut h.job), before, "the refusal left the file as it was");
    // Alone, it is moved as it always was (and is then off the walls: its ends are where it put them).
    ed::move_alone(&mut h.job, &part, 0.0, 3.0, known()).unwrap();
    assert_eq!(line(&mut h.job, &h.part), [[12.0, 3.0], [12.0, 19.0]]);
}

#[test]
fn the_south_wall_moved_two_feet_south_takes_the_walls_ends_and_the_partitions_square() {
    let mut h = house();
    let (east, part, south) = (h.east.clone(), h.part.clone(), h.south.clone());
    ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    ed::move_wall(&mut h.job, &part, -2.0, 0.0, known(), false).unwrap();
    let moved = ed::move_wall(&mut h.job, &south, 0.0, -2.0, known(), false).unwrap();
    assert_eq!(line(&mut h.job, &h.west), [[0.0, 16.0], [0.0, -2.0]]);
    assert_eq!(line(&mut h.job, &h.east), [[28.0, -2.0], [28.0, 16.0]]);
    assert_eq!(line(&mut h.job, &h.part), [[10.0, -2.0], [10.0, 16.0]]);
    assert_eq!((long(&mut h.job, &h.west), long(&mut h.job, &h.east), long(&mut h.job, &h.part)), (18.0, 18.0, 18.0));
    assert_eq!(offset(&mut h.job, &h.door), 19.5, "the door goes with its wall: still 18 ft to its edge");
    assert_eq!(areas(&mut h), (166.25, 306.25));
    assert_eq!(walls_lf(&mut h), 110.0);
    assert_eq!(moved.followed.len(), 3);
}

#[test]
fn a_move_that_would_leave_a_window_outside_its_wall_is_refused_naming_it() {
    let mut h = house();
    let east = h.east.clone();
    ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    let before = save(&mut h.job);
    let why = ed::move_wall(&mut h.job, &east, -12.0, 0.0, known(), false).unwrap_err();
    assert!(why.contains(&format!("window 1 [{}]", h.window)) && why.contains("no longer lie within wall 3"), "{why}");
    assert!(why.contains(&format!("door 1 [{}]", h.door)), "the door in the south wall, which would be 16 ft, is named too: {why}");
    assert_eq!(save(&mut h.job), before);
}

#[test]
fn alone_moves_the_wall_and_nothing_follows() {
    let mut h = house();
    let east = h.east.clone();
    let moved = ed::move_wall(&mut h.job, &east, 1.0, 0.0, known(), true).unwrap();
    assert_eq!(line(&mut h.job, &h.east), [[25.0, 0.0], [25.0, 16.0]]);
    assert_eq!(long(&mut h.job, &h.south), 24.0);
    assert_eq!(long(&mut h.job, &h.north), 24.0);
    assert!(moved.followed.is_empty() && moved.said().iter().any(|l| l.contains("nothing followed")));
}

#[test]
fn what_stands_against_the_moved_wall_is_told_and_not_moved() {
    let mut h = house();
    let level = h.level.clone();
    let boxed = ed::place(&mut h.job, &level, Kind::Furnishing, "Box", [22.75, 8.0], "0 ft", Some([2.0, 2.0, 3.0]), None, known()).unwrap();
    let east = h.east.clone();
    let moved = ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    assert_eq!(moved.against.len(), 1);
    assert!(moved.said().iter().any(|l| l.starts_with("1 thing stands against wall 2 and was not moved")), "{:?}", moved.said());
    let be = h.job.find(&boxed).unwrap();
    let at = h.job.world.get::<Placed>(be).unwrap().at.value.unwrap();
    assert_eq!(at, [22.75, 8.0]);
}

#[test]
fn what_is_moved_is_no_surer_than_the_move_and_an_existing_wall_is_named() {
    let mut h = house();
    ed::mark(&mut h.job, &[h.south.clone()], WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    let east = h.east.clone();
    let moved = ed::move_wall(&mut h.job, &east, 4.0, 0.0, design(), false).unwrap();
    let wall = |j: &mut Job, p: &str| {
        let e = j.find(p).unwrap();
        j.world.get::<Wall>(e).unwrap().line.clone()
    };
    for p in [&h.east, &h.south, &h.north] {
        let l = wall(&mut h.job, p);
        assert_eq!(l.status, Status::Assumed, "{p}");
        assert!(l.source.contains("moved"), "{p}: {}", l.source);
    }
    assert!(wall(&mut h.job, &h.south).source.contains("moved with wall 2"));
    for p in [&h.west, &h.part] {
        assert_eq!(wall(&mut h.job, p).status, Status::Known, "{p}");
    }
    assert_eq!(moved.existing, [h.south.clone()]);
    assert!(moved.said().iter().any(|l| l == "wall 1 is marked existing: as moved, it is not as it stands."), "{:?}", moved.said());
}

#[test]
fn a_wall_no_wall_meets_moves_exactly_as_before() {
    let mut job = Job::new("A lone wall (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let w = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap()[0].clone();
    let w2 = ed::walls(&mut job, &level, &w6, &[[0.0, 5.0], [10.0, 5.0]], false, ed::Top::Height("9 ft"), known()).unwrap()[0].clone();
    let text = save(&mut job);
    let (mut a, mut b) = (plumbline_world::file::load(&text).unwrap().0, plumbline_world::file::load(&text).unwrap().0);
    let moved = ed::move_wall(&mut a, &w, 1.5, 2.0, design(), false).unwrap();
    ed::move_alone(&mut b, &w, 1.5, 2.0, design()).unwrap();
    assert_eq!(save(&mut a), save(&mut b), "the same file, byte for byte");
    assert!(moved.followed.is_empty() && moved.said().len() == 1);
    assert_eq!(line(&mut a, &w2), [[0.0, 5.0], [10.0, 5.0]]);
}

#[test]
fn a_partition_pushed_past_the_end_of_the_wall_it_meets_is_refused() {
    let mut h = house();
    let part = h.part.clone();
    let before = save(&mut h.job);
    let why = ed::move_wall(&mut h.job, &part, 20.0, 0.0, known(), false).unwrap_err();
    assert!(why.contains("would no longer meet wall 1"), "{why}");
    assert_eq!(save(&mut h.job), before);
}

#[test]
fn a_wall_that_does_not_join_follows_nobody_and_is_followed_by_nobody() {
    let mut h = house();
    for p in [&h.south] {
        let e = h.job.find(p).unwrap();
        let mut w = h.job.world.get::<Wall>(e).unwrap().clone();
        w.join = false;
        h.job.put(p, w).unwrap();
    }
    let east = h.east.clone();
    let moved = ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    assert_eq!(long(&mut h.job, &h.south), 24.0);
    assert_eq!(moved.followed.len(), 1);
}

#[test]
fn a_wall_moved_across_slides_its_neighbours_end_along_it_and_a_turn_is_told() {
    // An end on the moved wall's span moves by what is square to it: a partition that meets
    // the east wall mid-way keeps its place along and stays butted.
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let cross = ed::walls(&mut job, &level, &w6, &[[24.0, 8.0], [30.0, 8.0]], false, ed::Top::Height("9 ft"), known()).unwrap()[0].clone();
    let moved = ed::move_wall(&mut job, &outer[1], -3.0, 0.0, known(), false).unwrap();
    assert_eq!(line(&mut job, &cross), [[21.0, 8.0], [30.0, 8.0]]);
    assert_eq!(moved.followed.iter().find(|f| f.path == cross).map(|f| f.grew_by), Some(3.0));
    // Moved along itself by 2: the end of each wall that meets it goes with the end, and the
    // cross wall's end, on the span, goes by the square part, which is nothing.
    let moved = ed::move_wall(&mut job, &outer[1], 0.0, 2.0, known(), false).unwrap();
    assert_eq!(line(&mut job, &cross), [[21.0, 8.0], [30.0, 8.0]]);
    assert_eq!(line(&mut job, &outer[1]), [[21.0, 2.0], [21.0, 18.0]]);
    assert!(moved.followed.iter().all(|f| !f.turned || f.path != cross));
}

#[test]
fn a_wall_moved_onto_the_wall_across_from_it_is_refused_with_its_neighbours_left_nothing() {
    // The old test (world.rs, a wall moved onto another) moves the north wall of a closed
    // 20 by 10 ft rectangle 10 ft south, alone, and is refused as counted twice. Not alone,
    // the east and west walls would shrink to nothing: refused, in those words.
    let mut job = Job::new("A box (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let made = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let before = save(&mut job);
    let why = ed::move_wall(&mut job, &made[2], 0.0, -10.0, known(), false).unwrap_err();
    assert!(why.contains("would shrink to nothing"), "{why}");
    assert!(ed::move_alone(&mut job, &made[2], 0.0, -10.0, known()).unwrap_err().contains("counted twice"));
    assert_eq!(save(&mut job), before);
}
