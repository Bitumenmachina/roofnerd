//! Call 229, the model's side (contract 229M): the house as its solids, the one source a mesh writer takes. Written before the code was
//! held to it. Everything is made up; the figures are worked by hand beside each test.
//!
//! The job: one level, Ground at 0, 9 ft high; four walls 6 in thick round 20 ft by 12 ft drawn south, east, north, west (so each is
//! drawn after the one before it); a door 3 ft by 6 ft 8 in in the south wall, its middle 4 ft along; a window 4 ft by 3 ft, sill 3 ft,
//! in the north wall, its middle 10 ft along.

use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::solids::{self, Body, Mesh};
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

fn job() -> Job {
    let mut job = Job::new("Solids (not a real job)", "test", "2026-11-26");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "Wall 6", "6 in", KNOWN).unwrap();
    let door = ed::opening_type(&mut job, "Door 3068", Kind::Door, "3 ft", "6 ft 8 in", KNOWN).unwrap();
    let window = ed::opening_type(&mut job, "Window 4030", Kind::Window, "4 ft", "3 ft", KNOWN).unwrap();
    let walls = ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [20.0, 0.0], [20.0, 12.0], [0.0, 12.0]], true, ed::Top::Height("9 ft"), KNOWN).unwrap();
    ed::opening(&mut job, &walls[0], Kind::Door, &door, ed::OnWall::Offset("4 ft"), "0 ft", KNOWN).unwrap();
    ed::opening(&mut job, &walls[2], Kind::Window, &window, ed::OnWall::Offset("10 ft"), "3 ft", KNOWN).unwrap();
    job.derive();
    job
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn bounds(b: &Body) -> [[f64; 3]; 2] {
    b.bounds().unwrap()
}

fn volume(b: &Body) -> f64 {
    match b {
        Body::Solid(m) => m.volume(),
        _ => panic!("not a solid"),
    }
}

#[test]
fn four_walls_a_door_and_a_window_are_six_parts_on_the_one_level_with_nothing_left_out() {
    let mut job = job();
    let s = solids::solids(&mut job).unwrap();
    assert_eq!(s.parts.len(), 6, "4 walls, 1 door, 1 window");
    assert_eq!(s.parts.iter().filter(|p| p.kind == Kind::Wall).count(), 4);
    assert_eq!(s.parts.iter().filter(|p| p.kind == Kind::Door).count(), 1);
    assert_eq!(s.parts.iter().filter(|p| p.kind == Kind::Window).count(), 1);
    assert_eq!(s.levels.len(), 1);
    assert_eq!(s.levels[0].name, "Ground");
    assert!(near(s.levels[0].elevation, 0.0));
    assert!(s.parts.iter().all(|p| p.level.as_deref() == Some(s.levels[0].path.as_str())));
    assert!(s.roofs.is_empty());
    assert!(s.not_carried.is_empty(), "{:?}", s.not_carried);
    assert_eq!(s.title, "Solids (not a real job)");
}

#[test]
fn the_walls_run_to_the_outside_corner_one_way_round_and_stop_at_the_face_the_other() {
    let mut job = job();
    let s = solids::solids(&mut job).unwrap();
    let walls: Vec<_> = s.parts.iter().filter(|p| p.kind == Kind::Wall).collect();
    // By hand (J20-6): the one drawn first runs through, the one drawn after stops at its face. South (drawn first) runs 0.25 ft past
    // each end: x -0.25 to 20.25, y -0.25 to 0.25. East (from y = 0 to 12) starts 0.25 short and ends 0.25 long: y 0.25 to 12.25, and
    // is x 19.75 to 20.25 across.
    let by_y = |lo: f64| walls.iter().find(|w| near(bounds(&w.body)[0][1], lo) && near(bounds(&w.body)[1][1], lo + 0.5)).unwrap();
    let south = by_y(-0.25);
    let b = bounds(&south.body);
    assert!(near(b[0][0], -0.25) && near(b[1][0], 20.25) && near(b[0][2], 0.0) && near(b[1][2], 9.0), "{b:?}");
    let east = walls.iter().find(|w| near(bounds(&w.body)[0][0], 19.75)).unwrap();
    let b = bounds(&east.body);
    assert!(near(b[0][1], 0.25) && near(b[1][1], 12.25) && near(b[1][0], 20.25), "{b:?}");
    // The ring: outside 20.5 ft by 12.5 ft less inside 19.5 ft by 11.5 ft is 32 ft2; as high as 9 ft is 288 ft3, held once; the door's
    // 3 x 0.5 x 6.6667 = 10 ft3 and the window's 4 x 0.5 x 3 = 6 ft3 are cut out of it.
    let total: f64 = walls.iter().map(|w| volume(&w.body)).sum();
    assert!(near(total, 288.0 - 10.0 - 6.0), "{total}");
    // The south wall: 20.5 x 0.5 x 9 = 92.25 less the door's 10 = 82.25.
    assert!(near(volume(&south.body), 82.25), "{}", volume(&south.body));
}

#[test]
fn a_door_and_a_window_are_the_space_they_take_up_in_their_wall() {
    let mut job = job();
    let s = solids::solids(&mut job).unwrap();
    // The door: 3 ft wide from 2.5 to 5.5 ft along the south wall's line (middle 4 ft along the wall from its first end), 6 ft 8 in
    // high from the floor, as thick as the wall (0.5 ft, centred on the line).
    let door = s.parts.iter().find(|p| p.kind == Kind::Door).unwrap();
    let b = bounds(&door.body);
    assert!(near(b[0][0], 2.5) && near(b[1][0], 5.5), "{b:?}");
    assert!(near(b[0][1], -0.25) && near(b[1][1], 0.25) && near(b[0][2], 0.0) && near(b[1][2], 6.0 + 8.0 / 12.0), "{b:?}");
    assert!(near(volume(&door.body), 3.0 * 0.5 * (6.0 + 8.0 / 12.0)));
    // The window: 4 ft by 3 ft, its sill at 3 ft, in the north wall; middle 10 ft along from its first end at x = 20, so x 8 to 12
    // (the north wall is drawn from east to west).
    let window = s.parts.iter().find(|p| p.kind == Kind::Window).unwrap();
    let b = bounds(&window.body);
    assert!(near(b[0][0], 8.0) && near(b[1][0], 12.0) && near(b[0][2], 3.0) && near(b[1][2], 6.0), "{b:?}");
    assert!(near(b[0][1], 11.75) && near(b[1][1], 12.25), "{b:?}");
}

#[test]
fn a_mesh_knows_its_box_and_its_volume() {
    // A unit cube by hand: eight places, twelve triangles facing out: volume 1, box 0..1.
    let p = |x: f64, y: f64, z: f64| [x, y, z];
    let m = Mesh {
        points: vec![p(0., 0., 0.), p(1., 0., 0.), p(1., 1., 0.), p(0., 1., 0.), p(0., 0., 1.), p(1., 0., 1.), p(1., 1., 1.), p(0., 1., 1.)],
        faces: vec![[4, 5, 6], [4, 6, 7], [0, 2, 1], [0, 3, 2], [0, 1, 5], [0, 5, 4], [1, 2, 6], [1, 6, 5], [2, 3, 7], [2, 7, 6], [3, 0, 4], [3, 4, 7]],
    };
    assert!(near(m.volume(), 1.0));
    assert_eq!(m.bounds(), Some([[0.0; 3], [1.0; 3]]));
    assert_eq!(Mesh::default().bounds(), None);
}

#[test]
fn along_a_run_is_the_unit_way_a_level_way_across_and_the_length() {
    let (way, across, length) = solids::along([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]).unwrap();
    assert!(near(length, 5.0) && near(way[0], 0.6) && near(way[1], 0.8) && near(way[2], 0.0));
    // Across is square to the run on plan and level: the run turned a quarter to the left.
    assert!(near(across[0], -0.8) && near(across[1], 0.6) && near(across[2], 0.0));
    // A riser's across is east; a run of no length has none.
    assert_eq!(solids::along([1.0, 1.0, 0.0], [1.0, 1.0, 5.0]).unwrap().1, [1.0, 0.0, 0.0]);
    assert!(solids::along([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]).is_none());
}

#[test]
fn a_job_with_no_level_has_no_solids_and_says_so() {
    let mut job = Job::new("Nothing (not a real job)", "test", "2026-11-26");
    let said = solids::solids(&mut job).unwrap_err();
    assert!(said.contains("no level"), "{said}");
}
