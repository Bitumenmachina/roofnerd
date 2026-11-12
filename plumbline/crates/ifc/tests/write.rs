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
    // The kitchen's hot branch has no size: its three pieces and its two elbows have no
    // shape, and each is said.
    let no_shape: Vec<&(String, usize, String)> = written.not_written.iter().filter(|n| n.2.contains("size is not known")).collect();
    assert_eq!(no_shape.len(), 5, "{:#?}", written.not_written);
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
