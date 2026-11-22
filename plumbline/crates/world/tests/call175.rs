//! Call 175: what the sweep found wrong in dollars. The house is made up; every figure is
//! worked by hand beside it (STATUS.md, "Call 175's fixes").

use plumbline_world::building::clash::clashes;
use plumbline_world::editing as ed;
use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up plan" }
}

/// One 40 ft wall, 10 ft high, a 3 ft by 6 ft 8 in door with its sill at the floor and a
/// 5 ft by 5 ft window with its sill at 3 ft.
fn wall_with_openings() -> (Job, String) {
    let mut job = Job::new("A wall (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "10 ft", known()).unwrap();
    let wt = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let dt = ed::opening_type(&mut job, "door", Kind::Door, "3 ft", "6 ft 8 in", known()).unwrap();
    let nt = ed::opening_type(&mut job, "window", Kind::Window, "5 ft", "5 ft", known()).unwrap();
    let wall = ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [40.0, 0.0]], false, ed::Top::Height("10 ft"), known()).unwrap().remove(0);
    ed::opening(&mut job, &wall, Kind::Door, &dt, ed::OnWall::Offset("6 ft"), "0 ft", known()).unwrap();
    ed::opening(&mut job, &wall, Kind::Window, &nt, ed::OnWall::Offset("24 ft"), "3 ft", known()).unwrap();
    (job, wall)
}

fn figure(job: &mut Job, path: &str, name: &str) -> Option<f64> {
    job.derive();
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).and_then(|m| m.0.get(name)).and_then(|d| d.value)
}

#[test]
fn a_wall_comes_down_to_its_floor_only_where_a_door_or_a_sill_at_the_floor_is() {
    let (mut job, wall) = wall_with_openings();
    // Nothing prices it by net_length: the wall has no such figure, and its file says nothing new.
    assert_eq!(figure(&mut job, &wall, "net_length"), None);
    let plain = save(&mut job);
    assert!(!plain.contains("net_length"));
    let (mut back, _) = load(&plain).unwrap();
    assert_eq!(save(&mut back), plain, "a job saved before opens and saves byte for byte");
    // Priced by it: the door takes 3 ft, the window (sill at 3 ft) nothing. 40 - 3 = 37.
    job.add_line(&wall, "Stone veneer wainscot, 3 ft", "net_length", Some(Class::Subcontract), false).unwrap();
    job.add_line(&wall, "Lap siding", "net_area", Some(Class::Material), false).unwrap();
    job.set_formula(&wall, Some("Lap siding"), "net_area - wainscot * net_length", &[("wainscot".to_string(), "3 ft".to_string())], Status::Assumed, "the 3 ft in the stone line's name").unwrap();
    assert_eq!(figure(&mut job, &wall, "net_length"), Some(37.0));
    // 400 - 20 - 25 = 355; 355 - 3 x 37 = 244.
    assert_eq!(figure(&mut job, &wall, "net_area").map(|x| (x * 100.0).round() / 100.0), Some(355.0));
    let est = view::estimate(&mut job);
    let said = format!("{est:?}");
    assert!(said.contains("244"), "the siding is bought on 244 SF: {said}");
    // And it is saved and opened again, byte for byte.
    let text = save(&mut job);
    let (mut again, _) = load(&text).unwrap();
    assert_eq!(save(&mut again), text);
}

/// A room 12 ft by 10 ft, 6 in walls, 9 ft high.
fn room() -> (Job, String, String) {
    let mut job = Job::new("A room (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let wt = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [12.0, 0.0], [12.0, 10.0], [0.0, 10.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    (job, level, wt)
}

#[test]
fn a_tub_against_its_wall_is_set_against_and_past_two_inches_it_is_in_the_wall() {
    let (mut job, level, _) = room();
    // The south wall's inside face is at y = 0.25: a tub 2 1/2 ft deep with its south edge 0.45 in into it.
    let tub = ed::place(&mut job, &level, Kind::Fixture, "tub", [6.0, 1.4625], "0 ft", Some([5.0, 2.5, 1.5]), None, known()).unwrap();
    let found = clashes(&mut job, None).unwrap();
    assert_eq!(found.found.iter().filter(|c| c.how == building::clash::ClashKind::Hard && !c.through && !c.set_against).count(), 0);
    let against: Vec<_> = found.found.iter().filter(|c| c.set_against).collect();
    assert_eq!(against.len(), 1);
    assert!((against[0].by.unwrap() * 12.0 - 0.45).abs() < 0.01, "{:?}", against[0].by);
    // Six inches in: in the wall.
    ed::move_by(&mut job, &tub, 0.0, -0.4625, known()).unwrap();
    let found = clashes(&mut job, None).unwrap();
    assert_eq!(found.found.iter().filter(|c| c.how == building::clash::ClashKind::Hard && !c.through && !c.set_against).count(), 1);
    assert_eq!(found.found.iter().filter(|c| c.set_against).count(), 0);
}

#[test]
fn a_riser_of_three_pieces_through_one_floor_is_one_hole_and_through_two_floors_two() {
    let mut job = Job::new("A riser (not a real job)", "test", "2026-11-22");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let second = ed::storey(&mut job, "second", "9 ft", "9 ft", known()).unwrap();
    let third = ed::storey(&mut job, "third", "18 ft", "9 ft", known()).unwrap();
    let ft = ed::floor_type(&mut job, "slab", "10 in", known()).unwrap();
    let pt = ed::segment_type(&mut job, "pipe", Kind::Pipe, "4 in", None, known()).unwrap();
    let square = vec![[0.0, 0.0], [12.0, 0.0], [12.0, 10.0], [0.0, 10.0]];
    ed::floor(&mut job, &second, &ft, ed::FloorBy::Points(&square), known()).unwrap();
    for (from, to) in [("8 ft", "8.4 ft"), ("8.4 ft", "8.8 ft"), ("8.8 ft", "9.5 ft")] {
        ed::riser(&mut job, &ground, Kind::Pipe, &pt, [6.0, 5.0], from, to, None, known()).unwrap();
    }
    let found = clashes(&mut job, None).unwrap();
    assert_eq!(found.found.iter().filter(|c| c.through).count(), 3, "three pieces are in the slab");
    assert_eq!(found.holes.len(), 1, "one hole to cut");
    assert_eq!(found.holes[0].pieces.len(), 3);
    assert_eq!(found.holes[0].size, "4.00 in round");
    ed::floor(&mut job, &third, &ft, ed::FloorBy::Points(&square), known()).unwrap();
    for (from, to) in [("9.5 ft", "17.3 ft"), ("17.3 ft", "17.9 ft"), ("17.9 ft", "18.6 ft")] {
        ed::riser(&mut job, &ground, Kind::Pipe, &pt, [6.0, 5.0], from, to, None, known()).unwrap();
    }
    assert_eq!(clashes(&mut job, None).unwrap().holes.len(), 2);
}
