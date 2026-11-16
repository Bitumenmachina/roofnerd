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

/// J18-8. A level's walls are also added up by the type of wall: a 6 in outside wall and
/// a 4 in partition are not one number to an estimator. Outside walls 24 ft by 16 ft
/// round, 9 ft high: 2 x (24 + 16) = 80 LF and 80 x 9 = 720 SF. One partition 16 ft
/// long: 16 LF and 144 SF. Together 96 LF and 864 SF, which is what the level says.
#[test]
fn a_levels_walls_are_added_up_by_their_type() {
    use plumbline_world::building::storey_totals;
    use plumbline_world::editing::{self as ed, Entered, Top};
    use plumbline_world::value::Status;
    let known = Entered { status: Status::Known, source: "made-up plan" };
    let mut job = plumbline_world::Job::new("Walls by type (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", known).unwrap();
    let outside = ed::wall_type(&mut job, "wall 6 in", "6 in", known).unwrap();
    let partition = ed::wall_type(&mut job, "partition 4 in", "4 in", known).unwrap();
    ed::walls(&mut job, &level, &outside, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, Top::Height("9 ft"), known).unwrap();
    ed::walls(&mut job, &level, &partition, &[[12.0, 0.0], [12.0, 16.0]], false, Top::Height("9 ft"), known).unwrap();
    job.derive();
    let totals = storey_totals(&mut job);
    let ground = &totals[0];
    assert_eq!((ground.walls.value, ground.wall_face.value), (Some(96.0), Some(864.0)));
    let by: Vec<(&str, usize, Option<f64>, Option<f64>)> = ground.walls_by_type.iter().map(|t| (t.name.as_str(), t.length.members, t.length.value, t.face.value)).collect();
    assert_eq!(by, [("wall 6 in", 4, Some(80.0), Some(720.0)), ("partition 4 in", 1, Some(16.0), Some(144.0))]);
    // The parts are the whole: nothing is in two types, and nothing in none.
    assert_eq!(by.iter().map(|t| t.2.unwrap()).sum::<f64>(), 96.0);
}

#[test]
fn a_room_closed_at_its_walls_faces_and_not_on_their_lines_has_no_area_to_wall_centers() {
    // A house 30 ft by 20 ft in 6 in walls. A room in its north-east corner, 10 ft by
    // 10 ft to wall lines, behind two partitions that were traced to stop 2 1/2 in
    // short of each other: one down from the north wall at 20 ft east, ending at 10 ft
    // north; one in from the east wall at 10 ft north, ending at 20 ft 2 1/2 in east.
    // The second's end is inside the first's thickness, so the faces close the room.
    let (mut job, s, wt, _) = start();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]], true, Top::Height("8 ft"), KNOWN).unwrap();
    ed::walls(&mut job, &s, &wt, &[[20.0, 20.0], [20.0, 10.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    ed::walls(&mut job, &s, &wt, &[[30.0, 10.0], [20.0 + 2.5 / 12.0, 10.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    let room = ed::room(&mut job, &s, "bedroom", [25.0, 15.0], KNOWN).unwrap();
    job.derive();
    let e = job.find(&room).unwrap();
    let m = job.world.get::<Measure>(e).unwrap().0.clone();
    // Inside the faces: 10 − 0.25 − 0.25 = 9.5 ft each way, 90.25 SF.
    assert_eq!(m["floor_area"].value, Some(90.25), "{}", m["floor_area"].note);
    // To the wall centers the room is not closed: the figure would be the whole house
    // less nothing (600 SF), six times the room. It is refused, and says what to do.
    assert_eq!(m["gross_area"].value, None);
    assert!(m["gross_area"].note.contains("stop short of each other") && m["gross_area"].note.contains("Draw them to meet"), "{}", m["gross_area"].note);
    // Drawn to meet, it has both: 100 SF to wall centers.
    let (mut job, s, wt, _) = start();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]], true, Top::Height("8 ft"), KNOWN).unwrap();
    ed::walls(&mut job, &s, &wt, &[[20.0, 20.0], [20.0, 10.0], [30.0, 10.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    let room = ed::room(&mut job, &s, "bedroom", [25.0, 15.0], KNOWN).unwrap();
    job.derive();
    let e = job.find(&room).unwrap();
    let m = job.world.get::<Measure>(e).unwrap().0.clone();
    assert_eq!((m["floor_area"].value, m["gross_area"].value), (Some(90.25), Some(100.0)));
}

#[test]
fn a_placed_item_is_drawn_as_its_item_says_or_as_its_name_says_or_as_its_box() {
    use plumbline_world::building::forms::Form;
    use plumbline_world::building::placements;
    use plumbline_world::units::Len;
    use plumbline_world::value::Val;
    let mut job = Job::new("items placed (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let inches = |x: f64| Val { value: Some(Len::One(x)), unit: "in".to_string(), status: Status::Assumed, source: "a made-up catalog".to_string() };
    let unknown = || Val::<String>::unknown("", "not chosen");
    let mut make = |name: &str, form: Option<&str>| {
        let item = Item { category: Kind::Fixture, maker: unknown(), model: unknown(), width: inches(24.0), depth: inches(24.0), height: inches(36.0), weight: None, sheet: String::new(), division: None, form: form.map(str::to_string) };
        ed::item(&mut job, name, item).unwrap()
    };
    // What it is drawn as is the item's to say: a thing whose name says nothing, drawn
    // as a tank; a sink told to be its box; a lavatory left to its name.
    let (thing, sink, lavatory) = (make("Unit 7", Some("tank")), make("Kitchen sink", Some("box")), make("Lavatory", None));
    let mut drawn_as = |item: &str, at: [f64; 2]| {
        let placed = ed::place_item(&mut job, &level, item, at, "0 ft", KNOWN).unwrap();
        placements(&mut job).0.into_iter().find(|p| p.path == placed).unwrap().form
    };
    assert_eq!(drawn_as(&thing, [4.0, 4.0]), Some(Form::Tank));
    assert_eq!(drawn_as(&sink, [10.0, 4.0]), None);
    assert_eq!(drawn_as(&lavatory, [16.0, 4.0]), Some(Form::Washbasin));
    // An item drawn as nothing the program draws is refused where it is made.
    let bad = Item { category: Kind::Fixture, maker: unknown(), model: unknown(), width: inches(24.0), depth: inches(24.0), height: inches(36.0), weight: None, sheet: String::new(), division: None, form: Some("gazebo".into()) };
    assert!(ed::item(&mut job, "Gazebo", bad).unwrap_err().contains("drawn as: \"gazebo\" is not something a thing can be drawn as"));
}

#[test]
fn a_thing_set_down_is_turned_a_quarter_at_a_time_and_a_fitting_is_not() {
    // J25A-3: a panelboard set against a wall that runs north could not face the room.
    use plumbline_world::building::{placements, Shape};
    use plumbline_world::units::Len;
    use plumbline_world::value::Val;
    let mut job = Job::new("a thing turned (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let inches = |x: f64| Val { value: Some(Len::One(x)), unit: "in".to_string(), status: Status::Assumed, source: "a made-up catalog".to_string() };
    let unknown = || Val::<String>::unknown("", "not chosen");
    let item = Item { category: Kind::Equipment, maker: unknown(), model: unknown(), width: inches(15.0), depth: inches(4.0), height: inches(30.0), weight: None, sheet: String::new(), division: None, form: None };
    let panel = ed::item(&mut job, "Panelboard", item).unwrap();
    let placed = ed::place_item(&mut job, &level, &panel, [10.0, 6.0], "0 ft", KNOWN).unwrap();
    let mut turned = |job: &mut Job| placements(job).0.into_iter().find(|p| p.path == placed).map(|p| match p.shape { Shape::Box { at, size, angle } => (at, size, angle), _ => panic!("not a box") }).unwrap();
    let (at, size, angle) = turned(&mut job);
    assert_eq!((angle, size[0], size[1]), (0.0, 1.25, 4.0 / 12.0));
    let by_hand = || ed::Entered { status: Status::Assumed, source: "turned by hand" };
    // A quarter: 90 degrees counter-clockwise. Where it stands and its size are as they were.
    assert_eq!(ed::turn(&mut job, &placed, 90.0, by_hand()).unwrap(), 90.0);
    assert_eq!(turned(&mut job), (at, size, 90.0));
    // Four quarters come round: 180, then -90 (270 kept between -180 and 180), then 0.
    assert_eq!(ed::turn(&mut job, &placed, 90.0, by_hand()).unwrap(), 180.0);
    assert_eq!(ed::turn(&mut job, &placed, 90.0, by_hand()).unwrap(), -90.0);
    assert_eq!(ed::turn(&mut job, &placed, 90.0, by_hand()).unwrap(), 0.0);
    // How it is turned is as sure as the hand that turned it, and says so.
    let e = job.find(&placed).unwrap();
    let a = job.world.get::<plumbline_world::Placed>(e).unwrap().angle.clone().unwrap();
    assert_eq!((a.status, a.source.as_str()), (Status::Assumed, "turned (turned by hand)"));
    // Nothing and nonsense are refused; so is a wall.
    assert!(ed::turn(&mut job, &placed, 0.0, by_hand()).is_err());
    assert!(ed::turn(&mut job, &placed, f64::NAN, by_hand()).is_err());
    assert!(ed::turn(&mut job, &level, 90.0, by_hand()).unwrap_err().contains("is not turned this way"));
    // The job still reads back as it was saved.
    let text = plumbline_world::file::save(&mut job);
    assert_eq!(plumbline_world::file::save(&mut plumbline_world::file::load(&text).unwrap().0), text);
}
