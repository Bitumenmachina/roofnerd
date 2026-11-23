//! Call 179 part B, the moved wall, tested as a unit (call 180: the builder wrote demonstration steps and no
//! unit test, and its work had three defects a judge found). Every house is made up and every figure is worked by
//! hand beside its assert. Prices are made up: floor $10.00/SF, footing $20.00/LF, wall face $3.00/SF, roofing
//! $8.00/SF of surface.

use plumbline_world::editing as ed;
use plumbline_world::value::Status;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn measure(job: &mut Job, path: &str, name: &str) -> Option<f64> {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).and_then(|m| m.0.get(name).and_then(|d| d.worked()))
}
fn total(job: &mut Job, paths: &[String], name: &str) -> f64 {
    paths.iter().filter_map(|p| measure(job, p, name)).sum()
}
fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}
fn roof_of(job: &mut Job, name: &str, ring: &[[f64; 2]], eave: &str) -> (String, Vec<String>) {
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length(eave).unwrap();
    let v = |x| value::Val { value: Some(x), unit: units::PITCH.into(), status: Status::Known, source: "made-up survey".into() };
    let footprint = Footprint {
        edges: ring.iter().map(|_| v(Rise::Pitch(rise))).collect(),
        points: value::Val { value: Some(ring.to_vec()), unit: units::FT.into(), status: Status::Known, source: "made-up survey".into() },
        eave_height: value::Val { value: Some(len), unit: unit.into(), status: Status::Known, source: "made-up survey".into() },
    };
    let roof = solve::new_roof(job, name, footprint).unwrap().0;
    let all: Vec<String> = job.children().get(&roof).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let planes = all.into_iter().filter(|p| job.kind(p) == Some(Kind::RoofPlane)).collect();
    (roof, planes)
}

/// A house of `levels` levels, each 24 by 16 with 6 in walls 9 ft high, a floor on each, a footing under the
/// ground floor and a roof over the top level, with prices on all of it; a room on the ground floor.
struct House {
    job: Job,
    walls: Vec<Vec<String>>, // by level: south, east, north, west
    floors: Vec<String>,
    room: String,
    planes: Vec<String>,
}

fn house(levels: usize) -> House {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-23");
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let slab = ed::floor_type(&mut job, "slab", "10 in", known()).unwrap();
    let ft = ed::footing_type(&mut job, "f24", "24 in", known()).unwrap();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
    let (mut walls, mut floors, mut first) = (Vec::new(), Vec::new(), String::new());
    for i in 0..levels {
        let level = ed::storey(&mut job, if i == 0 { "ground" } else { "second" }, &format!("{} ft", 9 * i), "9 ft", known()).unwrap();
        if i == 0 {
            first = level.clone();
        }
        walls.push(ed::walls(&mut job, &level, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap());
        floors.push(ed::floor(&mut job, &level, &slab, ed::FloorBy::Points(&ring), known()).unwrap().path);
    }
    let footing = ed::footings(&mut job, &first, &ft, &ring, true, "12 in", known()).unwrap();
    let room = ed::room(&mut job, &first, "Inside", [12.0, 8.0], known()).unwrap();
    let (_, planes) = roof_of(&mut job, "R", &ring, &format!("{} ft", 9 * levels));
    for f in &floors {
        job.add_line(f, "Floor", "plan_area", None, false).unwrap();
        job.set_on_line(f, "unit_cost", Some("Floor"), Some("$10/SF"), Status::Known, "made-up price").unwrap();
    }
    for f in &footing {
        job.add_line(f, "Footing", "plan_length", None, false).unwrap();
        job.set_on_line(f, "unit_cost", Some("Footing"), Some("$20/LF"), Status::Known, "made-up price").unwrap();
    }
    for w in walls.iter().flatten() {
        job.add_line(w, "Wall", "face_area", None, false).unwrap();
        job.set_on_line(w, "unit_cost", Some("Wall"), Some("$3/SF"), Status::Known, "made-up price").unwrap();
    }
    for p in &planes {
        job.add_line(p, "Roofing", "surface_area", None, false).unwrap();
        job.set_on_line(p, "unit_cost", Some("Roofing"), Some("$8/SF"), Status::Known, "made-up price").unwrap();
    }
    job.derive();
    House { job, walls, floors, room, planes }
}

fn price(job: &mut Job) -> f64 {
    job.derive();
    let est = view::estimate(job);
    est.lines.iter().filter(|l| !l.off).filter_map(|l| l.amount.value).sum()
}

struct Figures {
    floor: f64,
    footings: f64,
    walls: f64,
    room: f64,
    plan: f64,
    surface: f64,
    price: f64,
}

fn figures(h: &mut House) -> Figures {
    h.job.derive();
    let footings: Vec<String> = h.job.children().values().flat_map(|c| c.values().cloned()).filter(|p| p.starts_with("footing-")).collect();
    let walls: Vec<String> = h.walls.iter().flatten().cloned().collect();
    let floors = h.floors.clone();
    let planes = h.planes.clone();
    let room = h.room.clone();
    Figures {
        floor: floors.first().map(|f| measure(&mut h.job, f, "plan_area").unwrap()).unwrap(),
        footings: total(&mut h.job, &footings, "plan_length"),
        walls: total(&mut h.job, &walls, "plan_length"),
        room: measure(&mut h.job, &room, "floor_area").unwrap(),
        plan: total(&mut h.job, &planes, "plan_area"),
        surface: total(&mut h.job, &planes, "surface_area"),
        price: price(&mut h.job),
    }
}

/// The box, one level. By hand (STATUS.md, "Call 179 part B built"): floor 24 x 16 = 384 SF, footings 2 x 24 + 2 x 16 =
/// 80 LF, walls 80 LF, the room 23.5 x 15.5 = 364.25 SF, the roof's plan 384 SF and its surface 384 x 1.118034 = 429.33 SF;
/// priced 3,840 + 1,600 + 80 x 9 x 3 (2,160) + 429.325 x 8 (3,434.60) = $11,034.60. The east wall 4 ft east: floor 28 x 16 =
/// 448, footings 88, walls 88, the room 27.5 x 15.5 = 426.25, plan 448, surface 500.88, and the price 4,480 + 1,760 + 2,376
/// + 4,007.03 = $12,623.03: the rise is 640 + 160 + 216 + 572.43 = $1,588.43.
#[test]
fn the_box_east_wall_four_feet_east_takes_floor_footings_room_and_roof_and_back_restores_every_figure() {
    let mut h = house(1);
    let before = figures(&mut h);
    assert!(near(before.floor, 384.0, 1e-6) && near(before.footings, 80.0, 1e-6) && near(before.walls, 80.0, 1e-6) && near(before.room, 364.25, 1e-6), "{:?}", (before.floor, before.footings, before.walls, before.room));
    assert!(near(before.plan, 384.0, 1e-6) && near(before.surface, 429.325, 0.01) && near(before.price, 11034.60, 0.02), "{:?}", (before.plan, before.surface, before.price));
    let east = h.walls[0][1].clone();
    let moved = ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    let after = figures(&mut h);
    assert!(near(after.floor, 448.0, 1e-6) && near(after.footings, 88.0, 1e-6) && near(after.walls, 88.0, 1e-6) && near(after.room, 426.25, 1e-6), "{:?}", (after.floor, after.footings, after.walls, after.room));
    assert!(near(after.plan, 448.0, 1e-6) && near(after.surface, 500.879, 0.01) && near(after.price, 12623.03, 0.02), "{:?}", (after.plan, after.surface, after.price));
    let said = moved.said().join("\n");
    for part in ["wall 1 followed: 4 ft longer", "wall 3 followed: 4 ft longer", "floor 1 followed", "footing", "R followed: its edge over wall 2 went with it"] {
        assert!(said.contains(part), "{part}: {said}");
    }
    assert!(!said.contains("left where"), "{said}");
    // Back: every figure as it was.
    ed::move_wall(&mut h.job, &east, -4.0, 0.0, known(), false).unwrap();
    let back = figures(&mut h);
    assert!(near(back.floor, before.floor, 1e-6) && near(back.footings, before.footings, 1e-6) && near(back.walls, before.walls, 1e-6) && near(back.room, before.room, 1e-6));
    assert!(near(back.plan, before.plan, 1e-6) && near(back.surface, before.surface, 1e-6) && near(back.price, before.price, 0.005), "{:?} {:?}", (back.price, back.surface), (before.price, before.surface));
}

/// The south wall 2 ft south (the box grows to 24 x 18) and then the west wall 3 ft west (27 x 18): floor 432 then 486 SF,
/// footings 2 x (24 + 18) = 84 then 2 x (27 + 18) = 90 LF; the room 23.5 x 17.5 = 411.25 SF, then 26.5 x 17.5 = 463.75 SF.
#[test]
fn other_walls_of_the_box_move_the_same_way() {
    let mut h = house(1);
    let south = h.walls[0][0].clone();
    ed::move_wall(&mut h.job, &south, 0.0, -2.0, known(), false).unwrap();
    let a = figures(&mut h);
    assert!(near(a.floor, 24.0 * 18.0, 1e-6) && near(a.footings, 84.0, 1e-6) && near(a.walls, 84.0, 1e-6) && near(a.room, 23.5 * 17.5, 1e-6), "{:?}", (a.floor, a.footings, a.walls, a.room));
    assert!(near(a.plan, 432.0, 1e-6), "{}", a.plan);
    let west = h.walls[0][3].clone();
    ed::move_wall(&mut h.job, &west, -3.0, 0.0, known(), false).unwrap();
    let b = figures(&mut h);
    assert!(near(b.floor, 27.0 * 18.0, 1e-6) && near(b.footings, 90.0, 1e-6) && near(b.room, 26.5 * 17.5, 1e-6), "{:?}", (b.floor, b.footings, b.room));
    assert!(near(b.plan, 486.0, 1e-6), "{}", b.plan);
}

/// An L: 0,0 30,0 30,12 18,12 18,24 0,24 (30 x 12 and 18 x 12: 576 SF, 108 LF). The east wall of the low arm 3 ft east:
/// 33 x 12 + 18 x 12 = 612 SF, 108 + 3 + 3 = 114 LF of walls and of footing; the stem's east wall (18,12 to 18,24) 2 ft
/// east: the notch wall shortens 2 and the north wall lengthens 2: 612 + 24 = 636 SF? no, from the first figures: 576 + 2 x 12 =
/// 600 SF, 108 LF. Each by hand, and each back.
#[test]
fn an_l_house_moved_out_and_back_matches_the_hand_figures() {
    let mut job = Job::new("An L (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let slab = ed::floor_type(&mut job, "slab", "10 in", known()).unwrap();
    let ft = ed::footing_type(&mut job, "f24", "24 in", known()).unwrap();
    let ring = [[0.0, 0.0], [30.0, 0.0], [30.0, 12.0], [18.0, 12.0], [18.0, 24.0], [0.0, 24.0]];
    let walls = ed::walls(&mut job, &level, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap();
    let floor = ed::floor(&mut job, &level, &slab, ed::FloorBy::Points(&ring), known()).unwrap().path;
    let footing = ed::footings(&mut job, &level, &ft, &ring, true, "12 in", known()).unwrap();
    job.derive();
    let lf = |job: &mut Job, paths: &[String]| total(job, paths, "plan_length");
    assert!(near(measure(&mut job, &floor, "plan_area").unwrap(), 576.0, 1e-6) && near(lf(&mut job, &walls), 108.0, 1e-6) && near(lf(&mut job, &footing), 108.0, 1e-6));
    // The east wall of the low arm: walls[1] runs (30,0) to (30,12).
    ed::move_wall(&mut job, &walls[1], 3.0, 0.0, known(), false).unwrap();
    job.derive();
    assert!(near(measure(&mut job, &floor, "plan_area").unwrap(), 612.0, 1e-6), "{:?}", measure(&mut job, &floor, "plan_area"));
    assert!(near(lf(&mut job, &walls), 114.0, 1e-6) && near(lf(&mut job, &footing), 114.0, 1e-6), "{} {}", lf(&mut job, &walls), lf(&mut job, &footing));
    ed::move_wall(&mut job, &walls[1], -3.0, 0.0, known(), false).unwrap();
    job.derive();
    assert!(near(measure(&mut job, &floor, "plan_area").unwrap(), 576.0, 1e-6) && near(lf(&mut job, &walls), 108.0, 1e-6) && near(lf(&mut job, &footing), 108.0, 1e-6));
    // The stem's east wall, walls[3] (18,12 to 18,24), 2 ft east: 576 + 2 x 12 = 600 SF, 108 LF as it was.
    ed::move_wall(&mut job, &walls[3], 2.0, 0.0, known(), false).unwrap();
    job.derive();
    assert!(near(measure(&mut job, &floor, "plan_area").unwrap(), 600.0, 1e-6), "{:?}", measure(&mut job, &floor, "plan_area"));
    assert!(near(lf(&mut job, &walls), 108.0, 1e-6) && near(lf(&mut job, &footing), 108.0, 1e-6));
    ed::move_wall(&mut job, &walls[3], -2.0, 0.0, known(), false).unwrap();
    job.derive();
    assert!(near(measure(&mut job, &floor, "plan_area").unwrap(), 576.0, 1e-6));
}

/// Two storeys: the ground east wall 4 ft east takes the wall over it, both floors, the footing and the roof. By hand
/// (the judge's): before 2 x 3,840 + 1,600 + 2 x 2,160 + 429.325 x 8 = $17,034.60; after 2 x 4,480 + 1,760 + 2 x 2,376 +
/// 500.879 x 8 = 8,960 + 1,760 + 4,752 + 4,007.03 = $19,479.03.
#[test]
fn a_two_storey_house_moves_its_upper_wall_both_floors_the_footing_and_the_roof_with_the_wall() {
    let mut h = house(2);
    let before = figures(&mut h);
    assert!(near(before.price, 17034.60, 0.02), "{}", before.price);
    let east = h.walls[0][1].clone();
    let moved = ed::move_wall(&mut h.job, &east, 4.0, 0.0, known(), false).unwrap();
    let after = figures(&mut h);
    assert!(near(after.walls, 176.0, 1e-6), "88 LF on each level: {}", after.walls);
    assert!(near(after.floor, 448.0, 1e-6) && near(after.footings, 88.0, 1e-6) && near(after.plan, 448.0, 1e-6) && near(after.surface, 500.879, 0.01), "{:?}", (after.floor, after.footings, after.plan, after.surface));
    assert!(near(after.price, 19479.03, 0.02), "{}", after.price);
    let upper_floor = h.floors[1].clone();
    assert!(near(measure(&mut h.job, &upper_floor, "plan_area").unwrap(), 448.0, 1e-6));
    let said = moved.said().join("\n");
    assert!(said.contains("went with it: it stands over") && said.contains("followed"), "{said}");
    // The upper wall moved alone: the ground stays (a cantilever), the upper floor and the roof go with it.
    let mut h = house(2);
    let up = h.walls[1][1].clone();
    ed::move_wall(&mut h.job, &up, 4.0, 0.0, known(), false).unwrap();
    let a = figures(&mut h);
    assert!(near(a.floor, 384.0, 1e-6), "{}", a.floor);
    let upper_floor = h.floors[1].clone();
    assert!(near(measure(&mut h.job, &upper_floor, "plan_area").unwrap(), 448.0, 1e-6));
    assert!(near(a.plan, 448.0, 1e-6), "{}", a.plan);
    // And back.
    ed::move_wall(&mut h.job, &up, -4.0, 0.0, known(), false).unwrap();
    let b = figures(&mut h);
    assert!(near(b.price, 17034.60, 0.02), "{}", b.price);
}

/// A partition at x = 12: the east wall moved west is refused where its body would meet the partition's, naming both and how
/// far it can go: 24 - 12 = 12 ft between the lines, less half of each 6 in wall (3 in + 3 in) is 11 ft 6 in; moved that far it
/// touches and is accepted. A partition pushed along its own length does not move, and is refused with a sentence. Nothing
/// is changed by a refusal.
#[test]
fn a_move_across_a_partition_is_refused_naming_both_and_how_far_it_can_go() {
    let mut h = house(1);
    let level = {
        let mut q = h.job.world.query::<(&Node, &Kind)>();
        q.iter(&h.job.world).find(|(_, k)| **k == Kind::Storey).map(|(n, _)| n.path.clone()).unwrap()
    };
    let w6 = {
        let mut q = h.job.world.query::<(&Node, &Kind)>();
        q.iter(&h.job.world).find(|(n, k)| **k == Kind::Type && n.name == "w6").map(|(n, _)| n.path.clone()).unwrap()
    };
    let part = ed::walls(&mut h.job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0);
    h.job.derive();
    let before = plumbline_world::file::save(&mut h.job);
    let east = h.walls[0][1].clone();
    let why = ed::move_wall(&mut h.job, &east, -12.0, 0.0, known(), false).unwrap_err();
    assert!(why.contains("wall 2") && why.contains("wall 5") && why.contains("does not meet") && why.contains("it can go 11 ft 6 in west"), "{why}");
    assert_eq!(plumbline_world::file::save(&mut h.job), before, "nothing changed");
    ed::move_wall(&mut h.job, &east, -11.5, 0.0, known(), false).unwrap();
    // The partition pushed along its own length: both its ends are held on the walls it meets.
    let why = ed::move_wall(&mut h.job, &part, 0.0, 3.0, known(), false).unwrap_err();
    assert!(why.contains("does not move") && why.contains("held"), "{why}");
}
