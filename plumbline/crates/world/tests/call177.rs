//! What a run, a beam, a post and a footing are called by their size (call 177): worked in
//! the model, once, and only written by a window. Every tag is worked by hand here.

use plumbline_world::building::{self, Shape};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::sample::house;
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

/// A section and what it carries, the way the trade says it: a round one by its diameter
/// in inches, a rectangular one by its two sides; and the system's word after.
#[test]
fn a_run_is_called_by_its_size_and_what_it_carries() {
    // 3 in round on a drain: 3 in waste. 3/4 in on cold water. 1 1/2 in is 1 1/2.
    assert_eq!(building::run_tag(Some("Pipe 3 in round"), Some(0.25), None, Some("Sanitary")).as_deref(), Some("3 in waste"));
    assert_eq!(building::run_tag(Some("Pipe 3/4 in round"), Some(0.75 / 12.0), None, Some("Domestic Cold Water")).as_deref(), Some("3/4 in cold water"));
    assert_eq!(building::run_tag(Some("Pipe 1 1/2 in round"), Some(1.5 / 12.0), None, Some("Vent")).as_deref(), Some("1 1/2 in vent"));
    // 16 in by 8 in duct: 16 x 8 supply; a 6 in round duct: 6 in supply.
    assert_eq!(building::run_tag(Some("Duct 16 in x 8 in"), None, Some((16.0 / 12.0, 8.0 / 12.0)), Some("Supply Air")).as_deref(), Some("16 x 8 supply"));
    assert_eq!(building::run_tag(Some("Duct 6 in round"), Some(0.5), None, Some("Return Air")).as_deref(), Some("6 in return"));
    // A size that is no sixteenth is said as it is: 0.4 in.
    assert_eq!(building::run_tag(Some("Cable 0.4 in round"), Some(0.4 / 12.0), None, Some("Power")).as_deref(), Some("0.4 in power"));
    // On no system, its type's own name; no type, the size alone; no size, it says so.
    assert_eq!(building::run_tag(Some("Pipe 3 in round"), Some(0.25), None, None).as_deref(), Some("Pipe 3 in round"));
    assert_eq!(building::run_tag(None, Some(0.25), None, None).as_deref(), Some("3 in"));
    assert_eq!(building::run_tag(None, None, None, None).as_deref(), Some("size not known"));
    // A run whose size is not known says so, and says what it carries.
    assert_eq!(building::run_tag(None, None, None, Some("Domestic Hot Water")).as_deref(), Some("hot water, size not known"));
}

#[test]
fn the_made_up_house_names_its_runs_footings_and_walls_by_their_size() {
    let mut job = house();
    let (drawn, _) = building::placements(&mut job);
    let runs: Vec<_> = drawn.iter().filter(|p| p.kind.is_segment() && p.system.is_some()).collect();
    assert!(!runs.is_empty());
    for p in &runs {
        let tag = p.tag.as_deref().unwrap_or_else(|| panic!("{} has no tag", p.path));
        // Every run on a system says what it carries, in the trade's word.
        assert!(["supply", "return", "exhaust", "refrigerant", "cold water", "hot water", "waste", "vent", "power", "lighting"].iter().any(|w| tag.contains(w)), "{}: {tag}", p.path);
        // A stack is a vertical drain of 3 in and over, and says so; nothing else does.
        let upright = matches!(&p.shape, Shape::Run { a, b, .. } if (a[0] - b[0]).hypot(a[1] - b[1]) < 1.0e-6);
        assert_eq!(p.stack, tag.ends_with(" stack"), "{}: {tag}", p.path);
        if p.stack {
            assert!(upright && tag.contains("waste"), "{tag}");
        }
    }
    // A wall is called by its type.
    assert!(drawn.iter().filter(|p| p.kind == Kind::Wall).all(|p| p.tag.is_some()));
}

#[test]
fn a_beam_a_post_and_a_pad_footing_are_called_by_their_types_name() {
    let mut job = Job::new("Tags (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let beam = ed::framing_type(&mut job, "LVL 3 1/2 x 11 7/8", Kind::Beam, "3.5 in", Some("11.875 in"), None, KNOWN).unwrap();
    let post = ed::framing_type(&mut job, "6x6 post", Kind::Post, "5.5 in", Some("5.5 in"), None, KNOWN).unwrap();
    let pad = ed::framing_type(&mut job, "Pad 36 x 36", Kind::PadFooting, "3 ft", Some("3 ft"), None, KNOWN).unwrap();
    ed::beams(&mut job, &level, &beam, &[[12.0, 0.0], [12.0, 16.0]], "9 ft", KNOWN).unwrap();
    ed::post(&mut job, &level, &post, [12.0, 0.0], Some("8 ft"), None, KNOWN).unwrap();
    ed::pad(&mut job, &level, &pad, [12.0, 0.0], "18 in", "-1 ft", KNOWN).unwrap();
    let (drawn, _) = building::placements(&mut job);
    let tag = |k: Kind| drawn.iter().find(|p| p.kind == k).and_then(|p| p.tag.clone());
    assert_eq!(tag(Kind::Beam).as_deref(), Some("LVL 3 1/2 x 11 7/8"));
    assert_eq!(tag(Kind::Post).as_deref(), Some("6x6 post"));
    assert_eq!(tag(Kind::PadFooting).as_deref(), Some("Pad 36 x 36"));
}

// ---- The frame, placed ----

fn figure(job: &mut Job, path: &str, name: &str) -> Option<Derived> {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0.get(name).cloned()
}

fn plan_length(m: &[[[f64; 2]; 2]]) -> f64 {
    m.iter().map(|[a, b]| (b[0] - a[0]).hypot(b[1] - a[1])).sum()
}

/// A floor 24 ft by 16 ft, joists named 2x10 at 16 in. Not said which way they run: the
/// count takes the short way, Assumed, and the frame says no direction was said and lays none.
/// Said the short way (Assumed): the 19 joists of 16 ft, 304 LF, and the rim 2 x 24 = 48 LF,
/// are the very figures counted (`joists`, `joist_length`, `rim_length`). Said the long way: 13
/// of 24 ft, 312 LF, rim 32 LF.
#[test]
fn a_floors_joists_are_laid_as_they_are_counted_and_none_where_the_way_is_not_said() {
    let mut job = Job::new("Joists (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let ft = ed::floor_type(&mut job, "floor 10 in", "10 in", KNOWN).unwrap();
    let bare = ed::floor(&mut job, &level, &ft, ed::FloorBy::Points(&[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]]), KNOWN).unwrap().path;
    // No line, no frame.
    assert!(building::frames::frames(&mut job).is_empty());
    job.add_line(&bare, "Floor joists, 2x10 at 16 in", "joists", None, false).unwrap();
    job.derive();
    let laid = building::frames::frames(&mut job);
    assert_eq!(laid.len(), 1);
    assert!(!laid[0].said && laid[0].members.is_empty() && laid[0].rim.is_empty());
    // 2x10 is 9 1/4 in deep: 0.7708 ft; 16 in is 1.3333 ft.
    assert!((laid[0].depth - 9.25 / 12.0).abs() < 1.0e-9 && (laid[0].spacing - 16.0 / 12.0).abs() < 1.0e-9);
    job.set_on_line(&bare, "joists_span", None, Some("short way"), Status::Assumed, "usual").unwrap();
    job.derive();
    let laid = building::frames::frames(&mut job).remove(0);
    assert!(laid.said);
    assert_eq!(laid.members.len() as f64, figure(&mut job, &bare, "joists").unwrap().value.unwrap());
    assert_eq!(laid.members.len(), 19);
    assert!((plan_length(&laid.members) - 304.0).abs() < 1.0e-9);
    assert!((plan_length(&laid.rim) - figure(&mut job, &bare, "rim_length").unwrap().value.unwrap()).abs() < 1.0e-9);
    assert!((plan_length(&laid.rim) - 48.0).abs() < 1.0e-9);
    // Each joist runs the short way: north-south, 16 ft, and inside the floor.
    assert!(laid.members.iter().all(|[a, b]| (a[0] - b[0]).abs() < 1.0e-9 && ((b[1] - a[1]).abs() - 16.0).abs() < 1.0e-9 && a[0] > 0.0 && a[0] < 24.0));
    job.set_on_line(&bare, "joists_span", None, Some("the long way"), Status::Known, "the framing sheet, made up").unwrap();
    job.derive();
    let laid = building::frames::frames(&mut job).remove(0);
    assert_eq!(laid.members.len(), 13);
    assert!((plan_length(&laid.members) - 312.0).abs() < 1.0e-9 && (plan_length(&laid.rim) - 32.0).abs() < 1.0e-9);
}

/// The roof of tests/members.rs: a hip roof of 24 ft by 16 ft, rafters named on the roof.
/// 36 rafters in four planes (11, 11, 7, 7), laid where they are counted.
#[test]
fn a_roof_planes_rafters_are_laid_as_they_are_counted() {
    let mut job = Job::new("Rafters (not a real job)", "test", "2026-11-22");
    let footprint = Footprint {
        edges: (0..4).map(|_| Val { value: Some(Rise::Pitch(6.0)), unit: "in/12".into(), status: Status::Known, source: "made-up plan".into() }).collect(),
        points: Val::known(vec![[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], "ft", "made-up plan"),
        eave_height: Val::known(Len::One(9.0), "ft", "made-up plan"),
    };
    let (roof, _) = solve::new_roof(&mut job, "Main roof", footprint).unwrap();
    assert!(building::frames::frames(&mut job).is_empty());
    job.add_line(&roof, "Rafters, 2x8 at 24 in", "rafters", None, false).unwrap();
    job.derive();
    let laid = building::frames::frames(&mut job);
    let mut counts: Vec<usize> = laid.iter().map(|f| f.members.len()).collect();
    counts.sort();
    assert_eq!(counts, vec![7, 7, 11, 11]);
    for f in &laid {
        assert_eq!(f.members.len() as f64, figure(&mut job, &f.path, "rafters").unwrap().value.unwrap());
        // 2x8 is 7 1/4 in deep; 24 in apart.
        assert!((f.depth - 7.25 / 12.0).abs() < 1.0e-9 && (f.spacing - 2.0).abs() < 1.0e-9 && f.kind == Kind::RoofPlane);
    }
    // The long planes' rafters on plan add to 63.9375 ft each.
    let long: Vec<_> = laid.iter().filter(|f| f.members.len() == 11).collect();
    assert!(long.iter().all(|f| (plan_length(&f.members) - 63.9375).abs() < 1.0e-6));
}

// ---- A clash says where ----

/// The made-up house: what the window counts is what the model counts, apart from what is set
/// against a wall and what is a hole, and a hole is one row. And each clash is in a room, or
/// outside any room.
#[test]
fn the_window_counts_clashes_as_the_model_does_and_each_says_its_room() {
    use plumbline_world::building::clash;
    let mut job = plumbline_world::sample::house_priced();
    let found = clash::clashes(&mut job, None).unwrap();
    let (same, against, holes) = (found.in_the_same_space(), found.set_against(), found.hole_rows());
    // The three lists are the whole of what is Hard, and share nothing.
    let hard = found.found.iter().filter(|c| c.how == clash::ClashKind::Hard).count();
    let through = found.found.iter().filter(|c| c.how == clash::ClashKind::Hard && c.through).count();
    assert_eq!(same.len() + against.len() + through, hard);
    assert!(same.iter().all(|c| !c.set_against && !c.through));
    assert!(against.iter().all(|c| c.set_against && !c.through));
    // A hole is one row, however many pieces of the run pass through it.
    assert_eq!(holes.len(), found.holes.len());
    assert!(!same.is_empty() && !against.is_empty() && !holes.is_empty());
    let (rooms, hole_rooms) = clash::rooms_of(&mut job, &found);
    assert_eq!((rooms.len(), hole_rooms.len()), (found.found.len(), found.holes.len()));
    let names: Vec<String> = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).filter(|(_, k)| **k == Kind::Room).map(|(n, _)| n.name.clone()).collect()
    };
    assert!(rooms.iter().chain(&hole_rooms).all(|r| r == "outside any room" || names.contains(r)), "{rooms:?}");
    // The water service meeting the kitchen sink's waste at (11, 27.5, 0.7) is in the kitchen, whose
    // outline holds that place; the water closet against the wet wall at (18.5, 21.8) is in the wall,
    // and so in no room.
    let in_the_kitchen = found.found.iter().position(|c| c.a_name == "water service, piece 4 of 4" && c.b_name == "waste from the kitchen sink, piece 2 of 3").unwrap();
    assert_eq!(rooms[in_the_kitchen], "kitchen");
    let in_the_wall = found.found.iter().position(|c| c.a_name == "wet wall" && c.b_name.starts_with("Water closet (water closet 1)")).unwrap();
    assert_eq!(rooms[in_the_wall], "outside any room");
}
