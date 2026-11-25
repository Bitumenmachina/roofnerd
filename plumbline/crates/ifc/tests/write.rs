//! The building written as an IFC4 model, and read back by this crate's own reader.
//! The house is the made-up whole house; its figures are worked by hand in
//! `demos/house.sh` and, for the model written, in STATUS.md ("Exports and lenses").

use std::collections::BTreeSet;

use plumbline_ifc::write::export;
use plumbline_world::building::storey_totals;
use plumbline_world::editing::Entered;
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";

#[test]
fn the_house_is_written_whole_and_what_has_no_place_in_a_model_is_counted() {
    let mut job = sample::house();
    let written = export(&mut job, "house.ifc", WHEN).unwrap();
    let made = |what: &str| written.made.get(what).copied().unwrap_or(0);
    assert_eq!((made("levels"), made("walls"), made("doors"), made("windows"), made("floors"), made("rooms"), made("roofs"), made("roof planes")), (2, 16, 9, 17, 2, 10, 1, 4));
    // The kitchen's hot branch has no size: its three pieces are written as the runs they are, their line and
    // no body (call 200); its two elbows are written with their place and no body (call 204, point 6), so none is said
    // to have no shape, and all 41 fittings are in the file.
    let no_shape: Vec<&(String, usize, String)> = written.not_written.iter().filter(|n| n.2.contains("size is not known")).collect();
    assert_eq!(no_shape.len(), 0, "{:#?}", written.not_written);
    assert_eq!(made("fittings"), 41);
    assert_eq!(written.text.matches("'hot to the kitchen sink, piece ").count(), 3);
    assert!(written.text.starts_with("ISO-10303-21;\nHEADER;\n"));
    assert!(written.text.contains("FILE_SCHEMA(('IFC4'));"));
    // Every thing's identifier is 22 characters and no two are the same.
    let ids: Vec<&str> = written.text.lines().filter_map(|l| l.split_once("=IFC")).filter_map(|(_, rest)| rest.split_once("('")).map(|(_, rest)| &rest[..rest.find('\'').unwrap_or(0)]).filter(|id| id.len() == 22 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')).collect();
    assert!(ids.len() > 300, "{}", ids.len());
    assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), ids.len(), "two things have one identifier");
    // Written again, it is the same file to the byte: a model sent twice can be set
    // against itself.
    assert_eq!(export(&mut sample::house(), "house.ifc", WHEN).unwrap().text, written.text);
    // How sure a value is goes with it.
    assert!(written.text.contains("Plumbline_Values"));
    assert!(written.text.contains("(Assumed; "), "an Assumed value is not said to be");
}

#[test]
fn read_back_the_levels_walls_floors_and_rooms_are_what_was_written() {
    let mut job = sample::house();
    let written = export(&mut job, "house.ifc", WHEN).unwrap();
    let mut back = Job::new("read back (not a real job)", "test", "test");
    let brought = plumbline_ifc::import(&mut back, written.text.as_bytes(), "house.ifc", Entered { status: Status::Assumed, source: "its own model" }).unwrap();
    assert_eq!(brought.schema, "IFC4");
    assert!((brought.feet_per_unit - 1.0).abs() < 1.0e-9, "{}", brought.feet_per_unit);
    back.derive();
    let (was, is) = (storey_totals(&mut job), storey_totals(&mut back));
    assert_eq!(was.len(), 2);
    assert_eq!(is.len(), 2);
    let cents = |x: Option<f64>| x.map(|v| (v * 100.0).round() / 100.0);
    for (a, b) in was.iter().zip(&is) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.elevation, b.elevation);
        // 210.00 LF of wall a level; 1,120.00 SF of floor; the rooms' floor inside the walls.
        assert_eq!((a.walls.members, cents(a.walls.value)), (b.walls.members, cents(b.walls.value)), "{}: walls", a.name);
        assert_eq!(cents(a.walls.value), Some(210.00));
        assert_eq!(cents(a.wall_net.value), cents(b.wall_net.value), "{}: wall faces less openings", a.name);
        assert_eq!((a.floors.members, cents(a.floors.value)), (b.floors.members, cents(b.floors.value)), "{}: floors", a.name);
        assert_eq!(cents(a.floors.value), Some(1120.00));
        assert_eq!((a.rooms.members, cents(a.rooms.value)), (b.rooms.members, cents(b.rooms.value)), "{}: rooms", a.name);
        assert_eq!((a.doors, a.windows), (b.doors, b.windows), "{}: doors and windows", a.name);
        // What was Known in the job comes back no surer than whoever brought it in says.
        assert_eq!(b.walls.status, Status::Assumed);
    }
}

#[test]
fn a_job_with_no_level_has_no_building_to_write() {
    let mut job = Job::new("empty (not a real job)", "test", "test");
    assert!(export(&mut job, "empty.ifc", WHEN).unwrap_err().contains("no level"));
}

/// The seventeenth judgment, J17B-16 and 17: a model this program wrote says value by
/// value how sure each is. Read back as "Known", nothing in it comes in surer than
/// Assumed where any value was marked so, and the report says so; and the property sets
/// it does not read are counted, not passed over.
#[test]
fn a_model_this_program_wrote_keeps_its_doubt_when_it_is_read_back_as_known() {
    use plumbline_world::editing::{self as ed, Top};
    let unsure = Entered { status: Status::Assumed, source: "guessed from a photograph" };
    let mut job = Job::new("unsure (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", unsure).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", unsure).unwrap();
    ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]], true, Top::Height("9 ft"), unsure).unwrap();
    job.derive();
    let written = export(&mut job, "unsure.ifc", WHEN).unwrap();
    assert!(written.text.contains("(Assumed; "), "the model does not carry its marks");
    let mut back = Job::new("read back (not a real job)", "test", "test");
    let brought = plumbline_ifc::import(&mut back, written.text.as_bytes(), "unsure.ifc", Entered { status: Status::Known, source: "their model" }).unwrap();
    assert!(brought.notes.iter().any(|n| n.contains("was written by BuildOS and marks") && n.contains("everything from it was brought in Assumed, not Known as was said of it")), "{:#?}", brought.notes);
    assert!(brought.notes.iter().any(|n| n.starts_with("not read: ") && n.contains("the notes the model carries on its things")), "{:#?}", brought.notes);
    back.derive();
    let levels = storey_totals(&mut back);
    assert_eq!(levels.len(), 1);
    // 60 LF of wall, and no surer than the model that held it.
    assert_eq!((levels[0].walls.value.map(|v| (v * 100.0).round() / 100.0), levels[0].walls.status), (Some(60.0), Status::Assumed));
}

/// Call 179, C1. A wall this program wrote comes back on the line it was drawn on, to the
/// same length: the body runs past the line at a corner (the first of two walls to the
/// outside corner), and that was read back as the wall (a 20 ft wall came back 20 5/12 ft,
/// a 12 ft wall 11 1/2 ft). By hand: a ring 20 ft by 12 ft of 6 in walls, one 4 in, so the
/// four walls are 20, 12, 20 and 12 ft on the lines they were drawn on, each way; a door at
/// 5 ft on the first and a window at 8 ft on the second keep their places along the line.
#[test]
fn a_wall_this_program_wrote_is_read_back_on_its_own_line_to_its_own_length() {
    use plumbline_world::building::{placements, Shape};
    use plumbline_world::editing::{self as ed, OnWall, Top};
    let known = Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("ring (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", known).unwrap();
    let six = ed::wall_type(&mut job, "wall 6 in", "6 in", known).unwrap();
    let four = ed::wall_type(&mut job, "wall 4 in", "4 in", known).unwrap();
    let ring = [[0.0, 0.0], [20.0, 0.0], [20.0, 12.0], [0.0, 12.0]];
    let made = ed::walls(&mut job, &level, &six, &ring, true, Top::Height("9 ft"), known).unwrap();
    assert_eq!(made.len(), 4);
    let e = job.find(&made[3]).unwrap();
    let mut w = job.world.get::<plumbline_world::building::Wall>(e).cloned().unwrap();
    w.type_name.value = Some(four.clone());
    job.put(&made[3], w).unwrap();
    let door_type = ed::opening_type(&mut job, "door 3 ft", Kind::Door, "3 ft", "7 ft", known).unwrap();
    let window_type = ed::opening_type(&mut job, "window 4 ft", Kind::Window, "4 ft", "4 ft", known).unwrap();
    ed::opening(&mut job, &made[0], Kind::Door, &door_type, OnWall::Offset("5 ft"), "0 ft", known).unwrap();
    ed::opening(&mut job, &made[1], Kind::Window, &window_type, OnWall::Offset("8 ft"), "3 ft", known).unwrap();
    job.derive();
    let lines = |job: &mut Job| -> Vec<([f64; 2], [f64; 2], Vec<[f64; 4]>)> {
        let mut v: Vec<_> = placements(job).0.into_iter().filter_map(|p| match p.shape { Shape::Wall { a, b, holes, .. } if p.kind == Kind::Wall => Some((a, b, holes.iter().map(|h| h.2).collect())), _ => None }).collect();
        v.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        v
    };
    let was = lines(&mut job);
    let written = export(&mut job, "ring.ifc", WHEN).unwrap();
    let mut back = Job::new("read back (not a real job)", "test", "test");
    plumbline_ifc::import(&mut back, written.text.as_bytes(), "ring.ifc", Entered { status: Status::Assumed, source: "its own model" }).unwrap();
    back.derive();
    let is = lines(&mut back);
    assert_eq!(was.len(), 4);
    assert_eq!(is.len(), 4, "{is:?}");
    for (x, y) in was.iter().zip(&is) {
        let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() < 1.0e-3 && (p[1] - q[1]).abs() < 1.0e-3;
        assert!(near(x.0, y.0) && near(x.1, y.1), "a wall drawn {:?} to {:?} came back {:?} to {:?}", x.0, x.1, y.0, y.1);
        assert_eq!(x.2.len(), y.2.len());
        for (p, q) in x.2.iter().zip(&y.2) {
            assert!(p.iter().zip(q).all(|(m, n)| (m - n).abs() < 1.0e-3), "an opening {p:?} came back {q:?}");
        }
    }
    let lengths: Vec<f64> = is.iter().map(|l| (l.1[0] - l.0[0]).hypot(l.1[1] - l.0[1])).collect();
    let mut sorted = lengths.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for (got, by_hand) in sorted.iter().zip([12.0, 12.0, 20.0, 20.0]) {
        assert!((got - by_hand).abs() < 1.0e-3, "{sorted:?}");
    }
}

/// Call 179, C6 and C8. A door in a wall marked to demolish (the door marked to demolish too) is written, and is read back
/// in its wall, the wall and the door each to demolish: it was refused ("a door is not set in a wall that is coming
/// down") because the wall's mark was put on before its door. What is said of the model is in a builder's words: no
/// "property sets", no "LoadBearing"; and the figures a quantity was worked from are named in words, not as the program
/// holds them (plan length, not plan_length), and a volume's formula is not divided by 27 and multiplied by 27.
#[test]
fn a_door_in_a_wall_to_demolish_is_read_back_in_it_and_what_is_said_is_in_words() {
    use plumbline_world::components::WorkStatus;
    use plumbline_world::editing::{self as ed, ItsLines, OnWall, Top};
    let known = Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("demolition (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", known).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", known).unwrap();
    let dt = ed::opening_type(&mut job, "door 3 ft", Kind::Door, "3 ft", "7 ft", known).unwrap();
    let wall = ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, Top::Height("9 ft"), known).unwrap().remove(0);
    let door = ed::opening(&mut job, &wall, Kind::Door, &dt, OnWall::Offset("5 ft"), "0 ft", known).unwrap();
    ed::mark(&mut job, &[door.clone()], WorkStatus::Demolish, ItsLines::Ask, known).unwrap();
    ed::mark(&mut job, &[wall.clone()], WorkStatus::Demolish, ItsLines::Ask, known).unwrap();
    job.derive();
    let written = export(&mut job, "demolition.ifc", WHEN).unwrap();
    // Figures are named in words.
    assert!(!written.text.contains("plan_length") && !written.text.contains("face_area"), "a figure is named as the program holds it");
    assert!(written.text.contains("plan length"));
    let mut back = Job::new("read back (not a real job)", "test", "test");
    let brought = plumbline_ifc::import(&mut back, written.text.as_bytes(), "demolition.ifc", Entered { status: Status::Assumed, source: "its own model" }).unwrap();
    assert_eq!((brought.made.get("walls").copied(), brought.made.get("doors").copied()), (Some(1), Some(1)), "{:#?}", brought.notes);
    assert!(!brought.notes.iter().any(|n| n.contains("property set") || n.contains("LoadBearing") || n.contains("Status")), "{:#?}", brought.notes);
    let gone = plumbline_world::components::demolished(&mut back.world);
    assert_eq!(gone.len(), 2, "the wall and its door are each to demolish: {gone:?}");
}
