//! Floors by the outline of their walls, and what the verb refuses. Every figure here is
//! made up, and worked by hand before it is compared.

use plumbline_world::building;
use plumbline_world::editing::{self as ed, Entered, FloorBy, Top};
use plumbline_world::file::save;
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const TRACED: Entered = Entered { status: Status::Assumed, source: "traced off a made-up plan" };

fn measure(job: &mut Job, path: &str, name: &str) -> (Option<f64>, Status) {
    let e = job.find(path).unwrap();
    let d = job.world.get::<Measure>(e).unwrap().0[name].clone();
    (d.value.map(|x| (x * 100.0).round() / 100.0), d.status)
}

/// A storey, a 6 in wall type and a 4 in floor type.
fn start() -> (Job, String, String, String) {
    let mut job = Job::new("Built (not a real job)", "test", "test");
    let s = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let ft = ed::floor_type(&mut job, "slab 4 in", "4 in", KNOWN).unwrap();
    (job, s, wt, ft)
}

#[test]
fn a_floor_is_the_outline_its_walls_enclose_and_no_surer_than_they_are() {
    let (mut job, s, wt, ft) = start();
    // No walls yet: nothing is enclosed.
    let before = save(&mut job);
    let why = ed::floor(&mut job, &s, &ft, FloorBy::Walls, KNOWN).unwrap_err();
    assert!(why.contains("no walls enclose an area on ground"), "{why}");
    assert_eq!(save(&mut job), before);
    // A house 30 ft by 20 ft, traced, with a partition 12 ft from its west side.
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]], true, Top::Height("8 ft"), TRACED).unwrap();
    ed::walls(&mut job, &s, &wt, &[[12.0, 0.0], [12.0, 20.0]], false, Top::Height("8 ft"), TRACED).unwrap();
    let outline = building::floor_outline(&mut job, &s).unwrap().unwrap();
    assert_eq!(outline.boundary, [[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]], "its corners only, from the south-west, the partition's ends not among them");
    assert_eq!((outline.courtyards, outline.others, outline.status), (0, 0, Status::Assumed));
    // The floor: 30 by 20, 600 SF. Said to be Known by whoever asks; the walls are traced,
    // so it is Assumed, and its source says it is their outline.
    let made = ed::floor(&mut job, &s, &ft, FloorBy::Walls, KNOWN).unwrap();
    assert!(made.notes.is_empty(), "{:?}", made.notes);
    assert_eq!(measure(&mut job, &made.path, "plan_area"), (Some(600.0), Status::Assumed));
    let f = job.find(&made.path).and_then(|e| job.world.get::<Floor>(e).cloned()).unwrap();
    assert_eq!((f.boundary.status, f.boundary.source.as_str()), (Status::Assumed, "the outline of the walls on ground (made-up plan)"));
    let name = job.find(&made.path).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap();
    assert_eq!(name, "floor 1");
    // A second floor on that outline, by its walls or by the same corners in another
    // order, would be counted twice.
    let before = save(&mut job);
    for by in [FloorBy::Walls, FloorBy::Points(&[[30.0, 20.0], [0.0, 20.0], [0.0, 0.0], [30.0, 0.0]])] {
        let why = ed::floor(&mut job, &s, &ft, by, KNOWN).unwrap_err();
        assert!(why.contains("floor 1 is already on this outline on ground"), "{why}");
        assert_eq!(save(&mut job), before);
    }
    // Two rooms: inside the faces, 11.5 by 19.5 and 17.5 by 19.5.
    let west = ed::room(&mut job, &s, "West room", [6.0, 10.0], TRACED).unwrap();
    let east = ed::room(&mut job, &s, "East room", [21.0, 10.0], TRACED).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &west, "floor_area"), (Some(224.25), Status::Assumed));
    assert_eq!(measure(&mut job, &east, "floor_area"), (Some(341.25), Status::Assumed));
    // Saved and read back, the same.
    let text = save(&mut job);
    let (mut back, _) = file::load(&text).unwrap();
    assert_eq!(save(&mut back), text);
}

#[test]
fn a_floor_by_its_corners_is_as_sure_as_whoever_gives_it_says() {
    let (mut job, s, _, ft) = start();
    let made = ed::floor(&mut job, &s, &ft, FloorBy::Points(&[[0.0, 0.0], [10.0, 0.0], [10.0, 4.0], [4.0, 4.0], [4.0, 10.0], [0.0, 10.0]]), KNOWN).unwrap();
    // An L: 10 by 10 less 6 by 6.
    assert_eq!(measure(&mut job, &made.path, "plan_area"), (Some(64.0), Status::Known));
    let before = save(&mut job);
    assert!(ed::floor(&mut job, &s, &ft, FloorBy::Points(&[[0.0, 0.0], [1.0, 1.0]]), KNOWN).unwrap_err().contains("at least three corners"));
    assert!(ed::floor(&mut job, &s, &s, FloorBy::Points(&[[20.0, 0.0], [30.0, 0.0], [30.0, 4.0]]), KNOWN).unwrap_err().contains("is not a floor type"));
    assert!(ed::floor(&mut job, "storey-9999", &ft, FloorBy::Walls, KNOWN).is_err());
    assert_eq!(save(&mut job), before);
}

#[test]
fn walls_that_do_not_close_a_courtyard_and_a_second_house() {
    let (mut job, s, wt, ft) = start();
    // Three sides of a house: nothing is enclosed.
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    assert!(ed::floor(&mut job, &s, &ft, FloorBy::Walls, KNOWN).unwrap_err().contains("no walls enclose an area"));
    // The fourth side closes it.
    ed::walls(&mut job, &s, &wt, &[[0.0, 20.0], [0.0, 0.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    // A courtyard standing free in the middle, 6 ft square: a floor here has no hole, so
    // the floor by its walls is refused and says to give its corners.
    ed::walls(&mut job, &s, &wt, &[[12.0, 7.0], [18.0, 7.0], [18.0, 13.0], [12.0, 13.0]], true, Top::Height("8 ft"), KNOWN).unwrap();
    let before = save(&mut job);
    let why = ed::floor(&mut job, &s, &ft, FloorBy::Walls, KNOWN).unwrap_err();
    assert!(why.contains("enclose a courtyard inside the floor's outline") && why.contains("give the floor by its corners"), "{why}");
    assert_eq!(save(&mut job), before);
    // A room in that loop makes it a room, not a courtyard: the floor runs under it.
    ed::room(&mut job, &s, "Core", [15.0, 10.0], KNOWN).unwrap();
    let made = ed::floor(&mut job, &s, &ft, FloorBy::Walls, KNOWN).unwrap();
    assert_eq!(measure(&mut job, &made.path, "plan_area"), (Some(600.0), Status::Known));
    // A second house on the same storey, 10 ft square, 20 ft east of the first: the floor
    // by the walls is the larger, and the other is said.
    let (mut job, s, wt, ft) = start();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]], true, Top::Height("8 ft"), KNOWN).unwrap();
    ed::walls(&mut job, &s, &wt, &[[50.0, 0.0], [60.0, 0.0], [60.0, 10.0], [50.0, 10.0]], true, Top::Height("8 ft"), KNOWN).unwrap();
    let made = ed::floor(&mut job, &s, &ft, FloorBy::Walls, KNOWN).unwrap();
    assert_eq!(measure(&mut job, &made.path, "plan_area"), (Some(600.0), Status::Known));
    assert_eq!(made.notes.len(), 1);
    assert!(made.notes[0].contains("enclose another outline apart from this one"), "{:?}", made.notes);
}
