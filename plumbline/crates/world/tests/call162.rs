//! Call 162: a kitchen's base, its lump lines and its countertop. The room is made up: 6 in
//! walls 9 ft tall round 0,0 to 12,16 (faces 0.25 ft in), a 3 ft door in the west wall, the
//! room "Kitchen": 54 LF round, a wall run of 51 LF. Hand figures as in STATUS.md.

use plumbline_world::editing as ed;
use plumbline_world::value::Status;
use plumbline_world::view::{lump_lines, pricing};
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up layout" }
}
fn rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .map(|line| {
            let (mut out, mut cell, mut quoted) = (Vec::new(), String::new(), false);
            for c in line.chars() {
                match (c, quoted) {
                    ('"', _) => quoted = !quoted,
                    (',', false) => out.push(std::mem::take(&mut cell)),
                    (c, _) => cell.push(c),
                }
            }
            out.push(cell);
            out
        })
        .collect()
}
fn figure(job: &mut Job, path: &str, name: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0[name].clone()
}

/// The kitchen, the starter assemblies and items, the starter Kitchen on the room.
fn kitchen() -> (Job, String, String, [String; 4]) {
    let mut job = Job::new("A kitchen (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let d3 = ed::opening_type(&mut job, "door 3 ft", Kind::Door, "3 ft", "7 ft", known()).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [12.0, 0.0], [12.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    ed::opening(&mut job, &walls[3], Kind::Door, &d3, ed::OnWall::Offset("6.5 ft"), "0 ft", known()).unwrap();
    let room = ed::room(&mut job, &level, "Kitchen", [6.0, 8.0], known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    job.apply(&room, "Kitchen", "made-up").unwrap();
    job.derive();
    (job, level, room, [walls[0].clone(), walls[1].clone(), walls[2].clone(), walls[3].clone()])
}

fn codes(text: &str) -> Vec<String> {
    ed::box_list(text).unwrap()
}

#[test]
fn base_is_the_wall_run_less_what_stands_against_the_walls_on_the_floor() {
    let (mut job, level, room, [south, east, ..]) = kitchen();
    // Nothing set: the base is the wall run, to the digit, and Known.
    let (run, base) = (figure(&mut job, &room, "wall_run"), figure(&mut job, &room, "base_run"));
    assert_eq!((run.value, base.value, base.status, base.note.as_str()), (Some(51.0), Some(51.0), Status::Known, ""));
    // Run A in the corner along the east wall (8 ft from 3 in), run B along the south wall: LS36
    // covers 3 ft of it, B18 DB18 another 3 ft. 8 + 3 + 3 = 14 ft behind 5 things.
    ed::cabinets(&mut job, &level, &east, &room, "3 in", &codes("LS36 B24 SB36"), true, assumed()).unwrap();
    ed::cabinets(&mut job, &level, &south, &room, "5 ft 9 in", &codes("B18 DB18"), true, assumed()).unwrap();
    job.derive();
    let (run, base) = (figure(&mut job, &room, "wall_run"), figure(&mut job, &room, "base_run"));
    assert_eq!((run.value, base.value, base.status), (Some(51.0), Some(37.0), Status::Assumed));
    assert_eq!(base.note, "less 14 ft behind 5 things set against the walls");
    // Wall boxes hung 54 in up cover nothing.
    ed::cabinets(&mut job, &level, &east, &room, "3 in", &codes("W3030 W2430 W3030"), false, assumed()).unwrap();
    job.derive();
    assert_eq!(figure(&mut job, &room, "base_run").value, Some(37.0));
    // A thing marked to demolish covers nothing: run B comes down, LS36 still covers 3 ft of the south wall.
    let b: Vec<String> = {
        let mut q = job.world.query::<(&Node, &Placed)>();
        q.iter(&job.world).filter(|(n, _)| n.name == "B18" || n.name == "DB18").map(|(n, _)| n.path.clone()).collect()
    };
    assert_eq!(b.len(), 2);
    ed::mark(&mut job, &b, WorkStatus::Demolish, ed::ItsLines::Clear, known()).unwrap();
    job.derive();
    assert_eq!(figure(&mut job, &room, "base_run").value, Some(40.0));
    assert_eq!(figure(&mut job, &room, "wall_run").value, Some(51.0));
}

#[test]
fn two_things_over_the_same_length_count_it_once() {
    let (mut job, level, room, [south, ..]) = kitchen();
    // A box that stands away from the wall (a base box in the middle of the room) covers none.
    ed::place(&mut job, &level, Kind::Furnishing, "Island", [6.0, 8.0], "0 ft", Some([4.0, 2.0, 3.0]), None, assumed()).unwrap();
    job.derive();
    assert_eq!(figure(&mut job, &room, "base_run").value, Some(51.0), "an island stands against no wall");
    // Two boxes on the same stretch of the south wall: 4 ft by 2 ft, and 2 ft by 2 ft inside it.
    ed::place(&mut job, &level, Kind::Furnishing, "Pantry", [3.0, 1.25], "0 ft", Some([4.0, 2.0, 7.0]), None, assumed()).unwrap();
    ed::place(&mut job, &level, Kind::Furnishing, "Hamper", [3.0, 1.25], "0 ft", Some([2.0, 2.0, 2.0]), None, assumed()).unwrap();
    job.derive();
    let base = figure(&mut job, &room, "base_run");
    assert_eq!(base.value, Some(47.0), "4 ft covered once: 51 - 4");
    assert_eq!(base.note, "less 4 ft behind 2 things set against the walls");
    let _ = south;
}

#[test]
fn a_lump_line_and_the_things_it_stands_for_are_both_said() {
    let (mut job, level, room, [south, east, ..]) = kitchen();
    assert!(lump_lines(&mut job).is_empty());
    ed::cabinets(&mut job, &level, &east, &room, "3 in", &codes("LS36 B24 SB36"), true, assumed()).unwrap();
    ed::cabinets(&mut job, &level, &south, &room, "5 ft 9 in", &codes("B18 DB18"), true, assumed()).unwrap();
    job.derive();
    let said: Vec<String> = pricing(&mut job).lumps.iter().map(|l| l.said()).collect();
    assert_eq!(said.len(), 2, "{said:?}");
    assert_eq!(said[0], "Kitchen's line \"Cabinets\" (1 EA, 12 Furnishings) is in the price beside the lines of 7 things set in Kitchen in that division: if it stands for them, take the line off.");
    assert_eq!(said[1], said[0].replace("Cabinets", "Countertops"));
    // Nothing of Division 11 is set: nothing is said of "Kitchen appliances".
    assert!(!said.iter().any(|s| s.contains("Kitchen appliances")));
    // A dishwasher (Division 11) set in a run says it of that line too, in the singular.
    let north = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let mut walls: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Wall).map(|(n, _)| n.path.clone()).collect();
        walls.sort();
        walls[2].clone()
    };
    ed::cabinets(&mut job, &level, &north, &room, "3 in", &codes("[Dishwasher]"), false, assumed()).unwrap();
    job.derive();
    let said: Vec<String> = lump_lines(&mut job).iter().map(|l| l.said()).collect();
    assert!(said.iter().any(|s| s == "Kitchen's line \"Kitchen appliances\" (1 EA, 11 Equipment) is in the price beside the line of 1 thing set in Kitchen in that division: if it stands for it, take the line off."), "{said:?}");
}

#[test]
fn a_countertop_carries_its_line_and_says_what_its_area_leaves_out() {
    let (mut job, level, room, [south, east, ..]) = kitchen();
    let a = ed::cabinets(&mut job, &level, &east, &room, "3 in", &codes("LS36 B24 SB36"), true, assumed()).unwrap().countertop.unwrap().0;
    let b = ed::cabinets(&mut job, &level, &south, &room, "5 ft 9 in", &codes("B18 DB18"), true, assumed()).unwrap().countertop.unwrap().0;
    job.derive();
    let (area_a, area_b) = (figure(&mut job, &a, "plan_area"), figure(&mut job, &b, "plan_area"));
    // 8 ft by 25 1/2 in = 17 SF, and 3 ft by 25 1/2 in = 6.375 SF: the area does not move.
    assert_eq!((area_a.value, area_b.value), (Some(17.0), Some(6.375)));
    assert_eq!(area_a.note, "the leg that turns the corner over LS36, 10 1/2 in by 25 1/2 in (1.86 SF), is not in this area: add it where the top is priced by area.");
    assert_eq!(area_b.note, "");
    // Each carries a line "Countertop" by plan_area, in SF, Division 12, with no price.
    for top in [&a, &b] {
        let e = job.find(top).unwrap();
        let cost = job.world.get::<Cost>(e).unwrap();
        assert_eq!(cost.lines.len(), 1);
        let l = &cost.lines[0];
        assert_eq!((l.description.as_str(), l.quantity.as_str(), l.division), ("Countertop", "plan_area", Division::of_number(12)));
        assert_eq!(l.unit_cost.value, None);
    }
    // One price for both, found by what they are: 17 x 62 = 1,054.00 and 6.375 x 62 = 395.25.
    let done = ed::price_item(&mut job, "Countertop", "$62/SF", Status::Assumed, "made-up quote").unwrap();
    assert_eq!(done.priced.len(), 2);
    job.derive();
    let est = plumbline_world::view::estimate(&mut job);
    let amounts: Vec<f64> = est.lines.iter().filter(|l| l.description == "Countertop").filter_map(|l| l.amount.value).collect();
    assert_eq!(amounts, [1054.0, 395.25]);
    // The caution travels with the figure to its line.
    let line = est.lines.iter().find(|l| l.path == a && l.description == "Countertop").unwrap();
    assert!(line.quantity.note.contains("the leg that turns the corner over LS36"), "{:?}", line.quantity);
    // An item of the tool chest named the same is the item's; a name that is neither is refused.
    assert!(ed::price_item(&mut job, "Granite", "$62/SF", Status::Assumed, "made-up quote").is_err());
}
