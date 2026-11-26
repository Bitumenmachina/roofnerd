//! Call 219, the model's side (contract 219M): trying a change and taking it back, a door or a window moved, a copy and a row of copies,
//! a wall copied with its openings, and one search by name. Written before the code. Everything is made up; the figures are worked by hand
//! beside each test (the lead's figures are in STATUS.md, "How tools feel, the model's side"; where the program's own way differs, the test
//! holds the program's and says so).
//!
//! The job: one level, Ground, floor at 0 ft, 9 ft high; four walls 6 in thick round 20 ft by 12 ft, drawn 0,0 to 20,0 to 20,12 to 0,12
//! (wall 1 south, from its west end; wall 2 east; wall 3 north, from its EAST end; wall 4 west). A door 3 ft wide and 6 ft 8 in high in
//! wall 1, its MIDDLE 4 ft from the wall's first end (an opening's place is its middle: the lead's "4 ft from the west end" and "4 + 15 + 3"
//! take it as its near edge, and are 1 ft 6 in out). A face is 20 x 9 = 180 SF; less the door's 3 x 6 2/3 = 20 SF, 160 SF.

use plumbline_world::building::clash;
use plumbline_world::editing::{self as ed, AtOpening, Change, Entered, HowTall, OpeningTo};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const ASSUMED: Entered = Entered { status: Status::Assumed, source: "made-up plan" };

struct House {
    job: Job,
    level: String,
    wall_type: String,
    door_type: String,
    window_type: String,
    walls: Vec<String>,
    door: String,
    lav: String,
    duct_type: String,
}

fn house() -> House {
    let mut job = Job::new("Tools (not a real job)", "test", "2026-11-26");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "Wall 6", "6 in", KNOWN).unwrap();
    let door_type = ed::opening_type(&mut job, "Door 3068", Kind::Door, "3 ft", "6 ft 8 in", KNOWN).unwrap();
    let window_type = ed::opening_type(&mut job, "Window 3030", Kind::Window, "3 ft", "3 ft", KNOWN).unwrap();
    let walls = ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [20.0, 0.0], [20.0, 12.0], [0.0, 12.0]], true, ed::Top::Height("9 ft"), KNOWN).unwrap();
    let door = ed::opening(&mut job, &walls[0], Kind::Door, &door_type, ed::OnWall::Offset("4 ft"), "0 ft", KNOWN).unwrap();
    let inches = |x: f64| {
        let (l, unit) = units::parse_length(&format!("{x} in")).unwrap();
        Val { value: Some(l), unit: unit.to_string(), status: Status::Assumed, source: "a made-up catalog".to_string() }
    };
    let not_chosen = || Val::<String>::unknown("", "not chosen");
    let item = Item { category: Kind::Fixture, maker: not_chosen(), model: not_chosen(), width: inches(20.0), depth: inches(18.0), height: inches(34.0), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    let lav = ed::item(&mut job, "Lavatory", item).unwrap();
    let duct_type = ed::segment_type(&mut job, "Duct 14 x 8", Kind::Duct, "14 in", Some("8 in"), KNOWN).unwrap();
    job.derive();
    House { job, level, wall_type, door_type, window_type, walls, door, lav, duct_type }
}

fn next_id(job: &mut Job) -> u32 {
    let root = job.root();
    job.world.get::<JobInfo>(root).unwrap().next_id
}

fn net(job: &mut Job, wall: &str) -> f64 {
    let e = job.find(wall).unwrap();
    job.world.get::<Measure>(e).unwrap().0.get("net_area").unwrap().worked().unwrap()
}

fn placed(job: &mut Job, path: &str) -> Placed {
    let e = job.find(path).unwrap();
    job.world.get::<Placed>(e).unwrap().clone()
}

fn partition() -> impl Fn(&House) -> Change {
    |h: &House| Change::Walls { storey: h.level.clone(), type_path: h.wall_type.clone(), points: vec![[10.0, 0.0], [10.0, 12.0]], closed: false, tall: HowTall::Height("9 ft".into()) }
}

fn duct(h: &House) -> Change {
    Change::Runs { storey: h.level.clone(), kind: Kind::Duct, type_path: h.duct_type.clone(), points: vec![[-2.0, 6.0], [22.0, 6.0]], height: "8 ft".into(), system: None }
}

/// The job as it was, three ways: the saved text, the next path it would give out, and the undo history.
fn unchanged(job: &mut Job, text: &str, next: u32, history: &ed::History, labels: &[String]) {
    assert_eq!(file::save(job), text, "the saved job is byte for byte what it was");
    assert_eq!(next_id(job), next, "the next path the job would give out is what it was");
    assert_eq!(history.labels(false).iter().map(|s| s.to_string()).collect::<Vec<_>>(), labels, "the undo history is what it was");
}

// ------------------------------------------------------------------ 1. tried and taken back

/// A partition from (10, 0) to (10, 12): 1 wall, 12 ft long as `takeoff` gives it, 12 x 9 = 108 SF of face; it is in the same space as
/// nothing (it meets the south and the north wall's faces, which a wall does not clash with); the job is what it was.
#[test]
fn step_1_a_partition_is_tried_and_the_job_is_what_it_was() {
    let mut h = house();
    let mut history = ed::History::default();
    let before = ed::History::before(&mut h.job);
    ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Along(1.0), ASSUMED).unwrap();
    assert!(history.record("moved the door", before, &mut h.job));
    let labels: Vec<String> = history.labels(false).iter().map(|s| s.to_string()).collect();
    let (text, next) = (file::save(&mut h.job), next_id(&mut h.job));
    let change = partition()(&h);
    let tried = ed::try_change(&mut h.job, &change, KNOWN);
    assert_eq!(tried.refused, None, "{tried:?}");
    assert_eq!(tried.made.len(), 1, "{tried:?}");
    let wall = &tried.made[0];
    assert_eq!(wall.kind, Kind::Wall);
    assert_eq!(wall.name, "wall 5");
    let length = wall.figures.iter().find(|f| f.0 == "plan_length").expect("its length");
    assert!((length.1 - 12.0).abs() < 1.0e-9, "{length:?}");
    let face = wall.figures.iter().find(|f| f.0 == "net_area").expect("its face less openings");
    assert!((face.1 - 108.0).abs() < 1.0e-9, "{face:?}");
    assert!(tried.fits(), "{:?}", tried.said());
    assert!(tried.said().iter().any(|l| l.contains("wall 5, 12 ft long")), "{:?}", tried.said());
    unchanged(&mut h.job, &text, next, &history, &labels);
    // The real change makes what the try said it would: the same path, the same figures.
    let made = ed::apply(&mut h.job, &change, KNOWN).unwrap();
    assert_eq!(made.made, vec![wall.path.clone()], "the path it would have been given is the path it is given");
    assert!((net(&mut h.job, &wall.path) - 108.0).abs() < 1.0e-9);
}

// ------------------------------------------------------------------ 2. where it would clash

/// A duct 14 in by 8 in from (-2, 6) to (22, 6), 8 ft up: it passes through the west and the east wall: 2 holes, each naming its wall; the
/// reply says the duct is 24 ft long; nothing is made.
#[test]
fn step_2_a_duct_through_two_walls_says_two_holes() {
    let mut h = house();
    let (text, next) = (file::save(&mut h.job), next_id(&mut h.job));
    let history = ed::History::default();
    let change = duct(&h);
    let tried = ed::try_change(&mut h.job, &change, KNOWN);
    assert_eq!(tried.refused, None, "{tried:?}");
    assert_eq!(tried.holes.len(), 2, "{:?}", tried.said());
    let said = tried.said().join("\n");
    assert!(said.contains("2 holes to cut"), "{said}");
    assert!(said.contains("wall 2") && said.contains("wall 4"), "{said}");
    assert!(said.contains("24 ft long"), "{said}");
    assert!(!said.contains("-0"), "no path in it: {said}");
    assert!(tried.same_space.is_empty(), "a hole is not a clash: {said}");
    unchanged(&mut h.job, &text, next, &history, &[]);
    // What the try says is what `clashes` says once it is made.
    ed::apply(&mut h.job, &change, KNOWN).unwrap();
    let found = clash::clashes(&mut h.job, None).unwrap();
    assert_eq!(found.hole_rows().len(), 2);
}

// ------------------------------------------------------------------ 3. where it would be refused

/// A wall drawn over the south wall: the reply is the refusal the real change gives, in the names of things, and nothing is made.
#[test]
fn step_3_a_wall_over_a_wall_is_refused_in_the_words_the_real_change_gives() {
    let mut h = house();
    let (text, next) = (file::save(&mut h.job), next_id(&mut h.job));
    let change = Change::Walls { storey: h.level.clone(), type_path: h.wall_type.clone(), points: vec![[2.0, 0.0], [12.0, 0.0]], closed: false, tall: HowTall::Height("9 ft".into()) };
    let tried = ed::try_change(&mut h.job, &change, KNOWN);
    let why = tried.refused.clone().expect("refused");
    assert!(why.contains("lies along wall 1 for 10 ft") && why.contains("one wall drawn over another would be counted twice"), "{why}");  // call 226: a length in a sentence is said as a builder says it
    assert!(tried.made.is_empty());
    assert_eq!(tried.said(), vec![format!("it would be refused: {why}")]);
    assert_eq!(file::save(&mut h.job), text);
    assert_eq!(next_id(&mut h.job), next);
    // The real change is refused in the same words, but for the paths it would say.
    let real = ed::apply(&mut h.job, &change, KNOWN).unwrap_err();
    assert!(real.contains("lies along") && real.contains("one wall drawn over another would be counted twice"), "{real}");
    assert_eq!(file::save(&mut h.job), text);
}

// ------------------------------------------------------------------ 4. a door along its wall

/// The door's middle 4 ft from the wall's first end; moved 5 ft east: 9 ft, clear 7 ft 6 in of the west end and 9 ft 6 in of the east
/// end, the wall's face less openings 160 SF as it was. Moved 15 ft east from where it began: its middle would be 19 ft along, its east edge
/// 20 ft 6 in: refused, it would run **6 in** past the east end of a wall 20 ft long. (The lead's 2 ft takes the 4 ft as the door's near
/// edge; the model holds a door by its middle.) The job is what it was after each.
#[test]
fn step_4_a_door_moved_along_its_wall_and_refused_where_it_would_overrun() {
    let mut h = house();
    let (text, next) = (file::save(&mut h.job), next_id(&mut h.job));
    let history = ed::History::default();
    let tried = ed::try_change(&mut h.job, &Change::MoveOpening { path: h.door.clone(), to: OpeningTo::Along(5.0) }, ASSUMED);
    assert_eq!(tried.refused, None, "{:?}", tried.said());
    assert!(tried.said().iter().any(|l| l.contains("9 ft from the west end") && l.contains("7 ft 6 in clear of the west end") && l.contains("9 ft 6 in clear of the east end")), "{:?}", tried.said());
    assert_eq!(tried.faces.len(), 1);
    assert!((tried.faces[0].was.unwrap() - 160.0).abs() < 1.0e-9 && (tried.faces[0].now.unwrap() - 160.0).abs() < 1.0e-9, "{:?}", tried.faces);
    unchanged(&mut h.job, &text, next, &history, &[]);
    // Too far: refused, saying by how much, with no coordinate in it.
    let far = Change::MoveOpening { path: h.door.clone(), to: OpeningTo::Along(15.0) };
    let tried = ed::try_change(&mut h.job, &far, ASSUMED);
    let why = tried.refused.expect("refused");
    assert!(why.contains("door 1 would run 6 in past the east end of wall 1"), "{why}");
    assert!(why.contains("the wall is 20 ft long") && why.contains("3 ft wide") && why.contains("19 ft from the west end"), "{why}");
    assert!(!why.contains('(') && !why.contains("door-"), "no coordinate, no path: {why}");
    unchanged(&mut h.job, &text, next, &history, &[]);
    let real = ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Along(15.0), ASSUMED).unwrap_err();
    assert_eq!(real, why, "the try says what the real change says");
    assert_eq!(file::save(&mut h.job), text);
    // And the other way: past the west end.
    let why = ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Along(-3.0), ASSUMED).unwrap_err();
    assert!(why.contains("door 1 would run 6 in past the west end of wall 1"), "{why}");
    // Moved for real: the door is where it was moved to, and as sure as the move.
    let done = ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Along(5.0), ASSUMED).unwrap();
    assert_eq!((done.offset_ft, done.clear_ft), (9.0, (7.5, 9.5)));
    let e = h.job.find(&h.door).unwrap();
    let o = h.job.world.get::<Opening>(e).unwrap();
    assert_eq!(o.offset.feet(), Some(9.0));
    assert_eq!(o.offset.status, Status::Assumed, "what was moved is no surer than the move");
    assert!(o.offset.source.contains("moved (made-up plan)"), "{}", o.offset.source);
}

/// A window beside the door in the same wall: the door is refused where it would lie over it, by how far.
#[test]
fn step_4_a_door_is_refused_where_it_would_lie_over_a_window() {
    let mut h = house();
    // A window 3 ft wide, sill 3 ft, middle 12 ft along the south wall: it spans 10 ft 6 in to 13 ft 6 in.
    let window = ed::opening(&mut h.job, &h.walls[0].clone(), Kind::Window, &h.window_type.clone(), ed::OnWall::Offset("12 ft"), "3 ft", KNOWN).unwrap();
    let text = file::save(&mut h.job);
    // The door moved 7 ft: its middle at 11 ft, spanning 9 ft 6 in to 12 ft 6 in: 2 ft over the window.
    let why = ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Along(7.0), ASSUMED).unwrap_err();
    assert!(why.contains("door 1 would lie over window 1 in wall 1 by 2 ft"), "{why}");
    assert_eq!(file::save(&mut h.job), text);
    let _ = window;
}

// ------------------------------------------------------------------ 5. a door to another wall

/// To the north wall, its middle 6 ft from that wall's first end: the south wall's face less openings grows by 20.00 SF (160 to 180) and the
/// north wall's shrinks by it (180 to 160); the door is the same door (its path, its type) and is under the north wall. The job still loads.
#[test]
fn step_5_a_door_moved_to_another_wall_moves_the_faces() {
    let mut h = house();
    let (north, south) = (h.walls[2].clone(), h.walls[0].clone());
    assert!((net(&mut h.job, &south) - 160.0).abs() < 1.0e-9 && (net(&mut h.job, &north) - 180.0).abs() < 1.0e-9);
    let text = file::save(&mut h.job);
    let to = OpeningTo::Offset { wall: Some(north.clone()), offset: "6 ft".into() };
    let tried = ed::try_change(&mut h.job, &Change::MoveOpening { path: h.door.clone(), to: to.clone() }, ASSUMED);
    assert_eq!(tried.refused, None, "{:?}", tried.said());
    let said = tried.said().join("\n");
    assert!(said.contains("wall 1: face less openings was 160.00 SF, now 180.00 SF"), "{said}");
    assert!(said.contains("wall 3: face less openings was 180.00 SF, now 160.00 SF"), "{said}");
    assert!(said.contains("6 ft from the east end"), "wall 3 is drawn from its east end: {said}");
    assert_eq!(file::save(&mut h.job), text, "the try changed nothing");
    let done = ed::move_opening(&mut h.job, &h.door.clone(), &to, ASSUMED).unwrap();
    assert_eq!((done.from_wall.as_str(), done.to_wall.as_str()), (south.as_str(), north.as_str()));
    assert!((net(&mut h.job, &south) - 180.0).abs() < 1.0e-9 && (net(&mut h.job, &north) - 160.0).abs() < 1.0e-9);
    let e = h.job.find(&h.door).unwrap();
    assert_eq!(h.job.world.get::<Node>(e).unwrap().parent.as_deref(), Some(north.as_str()));
    assert_eq!(h.job.world.get::<Opening>(e).unwrap().type_name.value.as_deref(), Some(h.door_type.as_str()));
    let saved = file::save(&mut h.job);
    let (mut again, _) = file::load(&saved).expect("the job loads");
    assert_eq!(file::save(&mut again), saved);
    // To a point on the plan, nearest which it goes: (5, 12) is 15 ft along the north wall from its east end.
    let by_point = ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Point { wall: None, point: [5.0, 12.0] }, ASSUMED).unwrap();
    assert!((by_point.offset_ft - 15.0).abs() < 1.0e-9, "{by_point:?}");
    // Not to a thing that is not a wall, and not to a wall that is coming down.
    let why = ed::move_opening(&mut h.job, &h.door.clone(), &OpeningTo::Offset { wall: Some(h.level.clone()), offset: "6 ft".into() }, ASSUMED).unwrap_err();
    assert!(why.contains("is not a wall"), "{why}");
    let why = ed::move_opening(&mut h.job, &h.walls[1].clone(), &OpeningTo::Along(1.0), ASSUMED).unwrap_err();
    assert!(why.contains("is not a door or a window"), "{why}");
}

// ------------------------------------------------------------------ 6. a copy is Assumed

/// A Lavatory set down at (5, 5), its place Known and a line on it; copied 3 ft east: two lavatories; the copy's place and size are Assumed
/// and say what they were copied from; it carries the same item and cost line; its own name and path; the original is as it was.
#[test]
fn step_6_a_copy_is_assumed_and_carries_what_its_original_carries() {
    let mut h = house();
    let first = ed::place_item(&mut h.job, &h.level.clone(), &h.lav.clone(), [5.0, 5.0], "0 ft", KNOWN).unwrap();
    h.job.add_line(&first, "Set lavatory", "1", Some(Class::Material), false).unwrap();
    h.job.derive();
    assert_eq!(placed(&mut h.job, &first).at.status, Status::Known);
    let was = placed(&mut h.job, &first);
    let copy = ed::copy_by(&mut h.job, &first, 3.0, 0.0, KNOWN).unwrap();
    assert_eq!(copy.made.len(), 1);
    assert_eq!(copy.names, vec!["Lavatory 2".to_string()]);
    assert_eq!(placed(&mut h.job, &first), was, "the original is as it was");
    let p = placed(&mut h.job, &copy.made[0]);
    assert_eq!(p.at.value, Some([8.0, 5.0]));
    for (what, status, source) in [("at", p.at.status, &p.at.source), ("size", p.size.status, &p.size.source), ("offset", p.offset.status, &p.offset.source)] {
        assert_eq!(status, Status::Assumed, "a copy is never Known: {what}");
        assert!(source.contains("copied from Lavatory"), "{what}: {source}");
    }
    assert_eq!(p.size.value, was.size.value, "its sizes are its original's");
    assert_eq!(p.item, was.item, "it is one of the same item");
    let e = h.job.find(&copy.made[0]).unwrap();
    let lines = h.job.world.get::<Cost>(e).map(|c| c.lines.iter().map(|l| l.description.clone()).collect::<Vec<_>>()).unwrap_or_default();
    assert_eq!(lines, vec!["Set lavatory".to_string()], "the same cost lines");
    let sheet = view::estimate(&mut h.job);
    assert_eq!(sheet.lines.iter().filter(|l| l.description == "Set lavatory").count(), 2, "the estimate counts both");
    // Copied from a copy: still Assumed, still says where from.
    let again = ed::copy_by(&mut h.job, &copy.made[0], 3.0, 0.0, KNOWN).unwrap();
    assert_eq!(placed(&mut h.job, &again.made[0]).at.status, Status::Assumed);
    // Not copied: a thing that is only a fitting, a door alone, a floor.
    assert!(ed::copy_by(&mut h.job, &h.door.clone(), 1.0, 0.0, KNOWN).unwrap_err().contains("copied with its wall"));
    assert!(ed::copy_by(&mut h.job, &first, 0.0, 0.0, KNOWN).unwrap_err().contains("say how far"));
}

// ------------------------------------------------------------------ 7. a row of copies

/// Four copies of the lavatory at (5, 5), 3 ft apart eastward: at 8, 11, 14 and 17: five lavatories, each copy Assumed. Six copies: the
/// fifth stands at 20, on the east wall's line, 20 in wide: the reply (tried) says it is in the same space as wall 2; the sixth at 23 is
/// outside the house and is made.
#[test]
fn step_7_a_row_of_copies() {
    let mut h = house();
    let first = ed::place_item(&mut h.job, &h.level.clone(), &h.lav.clone(), [5.0, 5.0], "0 ft", KNOWN).unwrap();
    let text = file::save(&mut h.job);
    let all = ed::copy_row(&mut h.job, &first, 4, 3.0, 0.0, KNOWN).unwrap();
    let xs: Vec<f64> = all.iter().map(|c| placed(&mut h.job, &c.made[0]).at.value.unwrap()[0]).collect();
    assert_eq!(xs, vec![8.0, 11.0, 14.0, 17.0]);
    for c in &all {
        assert_eq!(placed(&mut h.job, &c.made[0]).at.status, Status::Assumed);
    }
    let count = |job: &mut Job| {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).filter(|(_, k)| **k == Kind::Fixture).count()
    };
    assert_eq!(count(&mut h.job), 5);
    // Six, tried from the first alone.
    let (mut again, _) = file::load(&text).unwrap();
    let tried = ed::try_change(&mut again, &Change::Row { path: first.clone(), times: 6, east: 3.0, north: 0.0 }, KNOWN);
    assert_eq!(tried.refused, None, "{:?}", tried.said());
    assert_eq!(tried.made.len(), 6, "all six would be made, the sixth outside the house");
    let touching: Vec<&str> = tried.same_space.iter().map(|t| t.said.as_str()).collect();
    assert_eq!(touching.len(), 1, "{touching:?}");
    assert!(touching[0].contains("Lavatory 6 would be in the same space as wall 2"), "the fifth copy is the sixth lavatory: {touching:?}");
    assert_eq!(file::save(&mut again), text, "tried and taken back");
    assert_eq!(count(&mut again), 1);
    // No copies, and too many.
    assert!(ed::copy_row(&mut again, &first, 0, 3.0, 0.0, KNOWN).unwrap_err().contains("say how many"));
}

/// All or nothing: a row of a wall whose second copy would lie over the east wall makes none, and says which copy.
#[test]
fn step_7_a_row_is_all_or_nothing() {
    let mut h = house();
    let made = ed::walls(&mut h.job, &h.level.clone(), &h.wall_type.clone(), &[[10.0, 0.0], [10.0, 12.0]], false, ed::Top::Height("9 ft"), KNOWN).unwrap();
    let text = file::save(&mut h.job);
    let why = ed::copy_row(&mut h.job, &made[0], 3, 5.0, 0.0, KNOWN).unwrap_err();
    assert!(why.starts_with("copy 2 of 3: "), "{why}");
    assert!(why.contains("lies along wall 2"), "{why}");
    assert_eq!(file::save(&mut h.job), text, "none were made");
}

// ------------------------------------------------------------------ 8. a wall with its openings

/// The south wall copied 12 ft north, onto the north wall's line: refused. Copied 6 ft north: a wall 20 ft long with a door 4 ft along it
/// (its middle, as the original's), Assumed; the original and its door are as they were.
#[test]
fn step_8_a_wall_copied_with_its_door() {
    let mut h = house();
    let south = h.walls[0].clone();
    let text = file::save(&mut h.job);
    let why = ed::copy_by(&mut h.job, &south, 0.0, 12.0, KNOWN).unwrap_err();
    assert!(why.contains("lies along wall 3 for 20 ft"), "{why}");  // call 226: a length in a sentence is said as a builder says it
    assert_eq!(file::save(&mut h.job), text);
    let done = ed::copy_by(&mut h.job, &south, 0.0, 6.0, KNOWN).unwrap();
    assert_eq!(done.made.len(), 2, "the wall and its door");
    let (wall, door) = (done.made[0].clone(), done.made[1].clone());
    let e = h.job.find(&wall).unwrap();
    let w = h.job.world.get::<Wall>(e).unwrap();
    assert_eq!(w.line_ft(), Some([[0.0, 6.0], [20.0, 6.0]]));
    assert_eq!(w.line.status, Status::Assumed);
    assert!(w.line.source.contains("copied from wall 1"), "{}", w.line.source);
    let d = h.job.find(&door).unwrap();
    let o = h.job.world.get::<Opening>(d).unwrap();
    assert_eq!(o.offset.feet(), Some(4.0));
    assert_eq!(o.type_name.value.as_deref(), Some(h.door_type.as_str()));
    assert_eq!(o.offset.status, Status::Assumed);
    assert_eq!(h.job.world.get::<Node>(d).unwrap().parent.as_deref(), Some(wall.as_str()));
    assert!((net(&mut h.job, &wall) - 160.0).abs() < 1.0e-9, "its face less its door");
    assert!(done.said().contains("with its 1 door or window"), "{}", done.said());
    let saved = file::save(&mut h.job);
    assert!(file::load(&saved).is_ok());
    assert!(!text.is_empty());
}

// ------------------------------------------------------------------ 9. find

/// `lav` finds the item Lavatory and each lavatory set down, with its level; `south` finds nothing here (no wall is so named) and says so;
/// a level's name finds the level; a word of an estimate line finds the line with its amount; exact names come first.
#[test]
fn step_9_find_by_name() {
    let mut h = house();
    let a = ed::place_item(&mut h.job, &h.level.clone(), &h.lav.clone(), [5.0, 5.0], "0 ft", KNOWN).unwrap();
    h.job.add_line(&a, "Set lavatory", "1", Some(Class::Material), false).unwrap();
    ed::copy_by(&mut h.job, &a, 3.0, 0.0, KNOWN).unwrap();
    h.job.derive();
    let found = ed::find_in_job(&mut h.job, "lav");
    let names: Vec<(String, String, Option<String>)> = found.iter().filter(|f| f.line.is_none()).map(|f| (f.name.clone(), f.is.clone(), f.level.clone())).collect();
    assert_eq!(names, vec![
        ("Lavatory".to_string(), "fixture".to_string(), Some("Ground".to_string())),
        ("Lavatory 2".to_string(), "fixture".to_string(), Some("Ground".to_string())),
        ("Lavatory".to_string(), "item in the tool chest".to_string(), None),
    ], "names that begin with it: a thing set down before an item, in the job's order");
    assert!(found.iter().any(|f| f.line.as_deref() == Some("Set lavatory")), "the estimate line");
    assert!(found.iter().all(|f| !f.exact), "`lav` is no whole name");
    // The whole name first: both of the name Lavatory (the thing, then the item), then the one that only begins with it.
    let whole = ed::find_names(&mut h.job, "lavatory");
    assert_eq!(whole.iter().map(|f| (f.name.as_str(), f.exact)).collect::<Vec<_>>(), vec![("Lavatory", true), ("Lavatory", true), ("Lavatory 2", false)]);
    assert_eq!((whole[0].is.as_str(), whole[1].is.as_str()), ("fixture", "item in the tool chest"));
    // Capitals do not matter; every word must be in the name.
    assert_eq!(ed::find_names(&mut h.job, "LAVATORY 2").len(), 1);
    assert_eq!(ed::find_names(&mut h.job, "2 lavatory").len(), 1);
    // Nothing is so named.
    assert!(ed::find_in_job(&mut h.job, "south").is_empty());
    assert_eq!(ed::nothing_said("south"), "nothing in the job is called \"south\", and no line of the estimate has it");
    assert_eq!(ed::nothing_said("  "), "type a name, or a word of one");
    // A level; a type.
    let level = ed::find_names(&mut h.job, "ground");
    assert_eq!((level.len(), level[0].is.as_str(), level[0].level.clone()), (1, "level", None));
    let types = ed::find_names(&mut h.job, "wall 6");
    assert!(types.iter().any(|f| f.is == "wall type"), "{types:?}");
    // A line, with what it comes to, and where.
    let lines = ed::find_lines(&mut h.job, "set lav");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].level.as_deref(), Some("Ground"));
    assert!(lines[0].said().starts_with("Set lavatory: estimate line of Lavatory"), "{}", lines[0].said());
    // Nothing typed finds nothing.
    assert!(ed::find_in_job(&mut h.job, "").is_empty());
}

// ------------------------------------------------------------------ 10. the sample whole house

/// The sample whole house (`sixteen`): every kind of try leaves it byte for byte what it was, with its next path and its undo history; and
/// what each costs is printed (`cargo test -p plumbline-world --test call219m sample -- --nocapture`).
#[test]
fn step_10_the_sample_house_is_what_it_was_after_every_try() {
    let mut job = sample::house_priced();
    job.derive();
    let (text, next) = (file::save(&mut job), next_id(&mut job));
    let storey = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let mut s: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Storey).map(|(n, _)| n.path.clone()).collect();
        s.sort();
        s.remove(0)
    };
    let (wall, door, thing, wtype) = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let mut walls: Vec<String> = q.iter(&job.world).filter(|(n, k)| **k == Kind::Wall && n.parent.as_deref() == Some(storey.as_str())).map(|(n, _)| n.path.clone()).collect();
        walls.sort();
        let door = q.iter(&job.world).find(|(_, k)| **k == Kind::Door).map(|(n, _)| n.path.clone()).expect("a door");
        let thing = q.iter(&job.world).find(|(_, k)| **k == Kind::Fixture).map(|(n, _)| n.path.clone());
        let w = job.find(&walls[0]).unwrap();
        let wtype = job.world.get::<Wall>(w).unwrap().type_name.value.clone().unwrap();
        (walls[0].clone(), door, thing, wtype)
    };
    let history = ed::History::default();
    let mut changes = vec![
        ("a wall", Change::Walls { storey: storey.clone(), type_path: wtype.clone(), points: vec![[1.0, 1.0], [1.0, 3.0]], closed: false, tall: HowTall::Height("8 ft".into()) }),
        ("a wall moved", Change::Move { path: wall.clone(), east: 1.0, north: 0.0, alone: false }),
        ("a door slid", Change::MoveOpening { path: door.clone(), to: OpeningTo::Along(0.5) }),
        ("a wall copied", Change::Copy { path: wall.clone(), east: 0.0, north: -40.0 }),
    ];
    if let Some(t) = &thing {
        changes.push(("a thing copied", Change::Row { path: t.clone(), times: 3, east: 2.0, north: 0.0 }));
        changes.push(("a thing moved", Change::Move { path: t.clone(), east: 1.0, north: 1.0, alone: false }));
    }
    for (what, change) in &changes {
        let started = std::time::Instant::now();
        let tried = ed::try_change(&mut job, change, KNOWN);
        let took = started.elapsed();
        println!("a try of {what}: {took:?}: {}", tried.said().first().cloned().unwrap_or_default());
        unchanged(&mut job, &text, next, &history, &[]);
        assert!(took.as_secs() < 30, "{what} took {took:?}");
    }
}

/// What a try is made of, on the sample whole house: saving the job's text, loading it back, and the clash test (printed, not asserted).
#[test]
fn step_10_what_a_try_is_made_of() {
    let mut job = sample::house_priced();
    job.derive();
    let t = std::time::Instant::now();
    let text = file::save(&mut job);
    let save = t.elapsed();
    let t = std::time::Instant::now();
    let _ = file::load(&text).unwrap();
    let load = t.elapsed();
    let t = std::time::Instant::now();
    let found = clash::clashes(&mut job, None).unwrap();
    let clashes = t.elapsed();
    println!("sample whole house, {} bytes saved: save {save:?}, load {load:?}, clash test {clashes:?} ({} things tested)", text.len(), found.tested);
}

/// A try that is refused leaves the job as it was too; a change that fails part way (the second wall of two) leaves none of it.
#[test]
fn a_refused_change_leaves_the_job_as_it_was() {
    let mut h = house();
    let text = file::save(&mut h.job);
    let bad = Change::Walls { storey: h.level.clone(), type_path: "type-9999".into(), points: vec![[1.0, 1.0], [5.0, 1.0]], closed: false, tall: HowTall::Height("9 ft".into()) };
    let tried = ed::try_change(&mut h.job, &bad, KNOWN);
    assert!(tried.refused.is_some(), "{tried:?}");
    assert_eq!(file::save(&mut h.job), text);
    let tried = ed::try_change(&mut h.job, &Change::Item { storey: h.level.clone(), item: "No such item".into(), at: [5.0, 5.0], offset: "0 ft".into() }, KNOWN);
    assert!(tried.refused.is_some());
    assert_eq!(file::save(&mut h.job), text);
    // A door tried in a wall: where it fits and where it would not.
    let tried = ed::try_change(&mut h.job, &Change::Opening { wall: h.walls[1].clone(), kind: Kind::Door, type_path: h.door_type.clone(), at: AtOpening::Offset("6 ft".into()), sill: "0 ft".into() }, KNOWN);
    assert_eq!(tried.refused, None, "{:?}", tried.said());
    assert!(tried.said()[0].contains("door 2 in wall 2"), "{:?}", tried.said());
    let tried = ed::try_change(&mut h.job, &Change::Opening { wall: h.walls[1].clone(), kind: Kind::Door, type_path: h.door_type.clone(), at: AtOpening::Offset("11 ft".into()), sill: "0 ft".into() }, KNOWN);
    assert!(tried.refused.is_some(), "a door 3 ft wide, its middle 11 ft along a wall 12 ft long, would overrun it");
    assert_eq!(file::save(&mut h.job), text);
}
