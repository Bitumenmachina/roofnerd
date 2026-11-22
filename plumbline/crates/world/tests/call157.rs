//! Call 157: a room's run of wall, and the kitchen and the layouts set right, each to the
//! figures worked by hand. The houses are made up (no real job).

use plumbline_world::building::clash::clashes;
use plumbline_world::editing as ed;
use plumbline_world::export::{lens_out, Lens, Out};
use plumbline_world::file::save;
use plumbline_world::value::Status;
use plumbline_world::view::{estimate, options, OptionFigure};
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up addition plan" }
}
fn measure(job: &mut Job, path: &str, name: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0[name].clone()
}

/// The starter library's rows, the way a file reads them.
fn starter_rows(text: &str) -> Vec<Vec<String>> {
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

/// A house 24 ft by 16 ft, 9 ft walls, 6 in thick (faces 0.25 ft in): Known.
fn house() -> (Job, String, String, Vec<String>) {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    (job, level, w6, outer)
}

/// Call 157, point 1. The roomline house (east wall to demolish, a 12 ft addition, a room line
/// at x = 24): Living 63 LF and Kitchen 39 LF of wall, their perimeters 78.5 and 54.5 as they
/// were (the line is 15.5 LF in each: 78.5 - 15.5 = 63, 54.5 - 15.5 = 39). The drywall house (a
/// partition at x = 12 with a 3 ft door): each room 54 LF round, less the door, 51 LF.
#[test]
fn a_rooms_run_of_wall_is_its_floor_line_along_walls_less_its_doorways() {
    let (mut job, level, w6, outer) = house();
    ed::mark(&mut job, &[outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    let living = ed::room(&mut job, &level, "Living", [6.0, 8.0], known()).unwrap();
    let kitchen = ed::room(&mut job, &level, "Kitchen", [30.0, 8.0], known()).unwrap();
    ed::room_lines(&mut job, &level, &[[24.0, 0.0], [24.0, 16.0]], assumed()).unwrap();
    job.derive();
    for (room, round, run) in [(&living, 78.5, 63.0), (&kitchen, 54.5, 39.0)] {
        assert_eq!(measure(&mut job, room, "perimeter").value, Some(round), "{room}");
        let d = measure(&mut job, room, "wall_run");
        assert_eq!((d.value, d.unit.as_str()), (Some(run), "LF"), "{room}: {d:?}");
        // As sure as the walls it is along (the addition is Assumed), and it says how and from what.
        assert_eq!(d.status, Status::Assumed, "{room}");
        assert!(d.formula.contains("less the doorways") && d.formula.contains("room line"), "{d:?}");
        assert!(d.inputs.iter().any(|i| i.starts_with("roomline-")) && d.inputs.iter().any(|i| i.starts_with("wall-")), "{d:?}");
    }
    // The floor line `receptacles` walks is the same: 63 / 12 is 5.25, 6 outlets; 39 / 12 is 3.25, 4.
    assert_eq!((measure(&mut job, &living, "receptacles").value, measure(&mut job, &kitchen, "receptacles").value), (Some(6.0), Some(4.0)));
    // Taken away, the room line goes and the run is the whole way round: the Living room is the whole inside.
    let line = job.order.iter().filter_map(|&e| job.world.get::<Node>(e).map(|n| (n.path.clone(), job.world.get::<Kind>(e).copied()))).find(|(_, k)| *k == Some(Kind::RoomLine)).unwrap().0;
    ed::remove(&mut job, &line).unwrap();
    job.derive();
    // 35.5 by 15.5 inside the faces: 2 x (35.5 + 15.5) = 102 LF, all of it along walls.
    assert_eq!((measure(&mut job, &living, "perimeter").value, measure(&mut job, &living, "wall_run").value), (Some(102.0), Some(102.0)));

    // The drywall house: a partition with a 3 ft door at 8 ft.
    let (mut job, level, w6, _) = house();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap()[0].clone();
    let door = ed::opening_type(&mut job, "door 3 ft", Kind::Door, "3 ft", "7 ft", known()).unwrap();
    let d = ed::opening(&mut job, &part, Kind::Door, &door, ed::OnWall::Offset("8 ft"), "0 ft", known()).unwrap();
    let west = ed::room(&mut job, &level, "West room", [6.0, 8.0], known()).unwrap();
    let east = ed::room(&mut job, &level, "East room", [18.0, 8.0], known()).unwrap();
    job.derive();
    for room in [&west, &east] {
        assert_eq!(measure(&mut job, room, "perimeter").value, Some(54.0), "{room}");
        assert_eq!(measure(&mut job, room, "wall_run").value, Some(51.0), "{room}");
        assert_eq!(measure(&mut job, room, "wall_run").status, Status::Known, "{room}");
    }
    // A door to demolish is filled in and is no doorway: 54 LF.
    ed::mark(&mut job, &[d.clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &west, "wall_run").value, Some(54.0));
}

/// The judge's room: a 24 x 16 house, rooms A and B, a room line at x = 8. A's perimeter is
/// 7.75 + 15.5 + 7.75 + 15.5 = 46.5; its wall is 31. Base priced from the starter library
/// (`Room, hardwood floor`) reads 31 LF, and not 46.5. Eight lines of the library are priced by
/// the run of wall, and none by the perimeter.
#[test]
fn base_from_the_starter_library_is_priced_by_the_run_of_wall() {
    let lib = tables::STARTER_LIBRARY;
    let base: Vec<&str> = lib.lines().filter(|l| l.contains(",Base and trim,") || l.contains(",Install base and trim,")).collect();
    assert_eq!(base.len(), 8);
    assert!(base.iter().all(|l| l.contains(",wall_run,") && !l.contains(",perimeter,")), "{base:?}");

    let (mut job, level, _, _) = house();
    let a = ed::room(&mut job, &level, "A", [4.0, 8.0], known()).unwrap();
    ed::room(&mut job, &level, "B", [16.0, 8.0], known()).unwrap();
    ed::room_lines(&mut job, &level, &[[8.0, 0.0], [8.0, 16.0]], known()).unwrap();
    tables::assemblies_in(&mut job, &starter_rows(lib), None).unwrap();
    job.apply(&a, "Room, hardwood floor", "made-up").unwrap();
    job.set_on_line(&a, "unit_cost", Some("Base and trim"), Some("$2.00/LF"), Status::Assumed, "made-up price").unwrap();
    job.derive();
    assert_eq!(measure(&mut job, &a, "perimeter").value, Some(46.5));
    assert_eq!(measure(&mut job, &a, "wall_run").value, Some(31.0));
    let est = estimate(&mut job);
    let line = est.lines.iter().find(|l| l.element == "A" && l.description == "Base and trim" || l.path == a && l.description == "Base and trim").unwrap();
    // 31 LF at $2.00: $62.00, and not 46.5 LF, $93.00.
    assert_eq!((line.quantity.value, line.amount.value), (Some(31.0), Some(62.0)));
}

/// A kitchen of the house of the room line: the Kitchen's east wall (36,0 to 36,16), its south
/// wall (24,0 to 36,0) and its north wall (36,16 to 24,16, running west), each 12 ft or 16 ft
/// with its face 0.25 ft in. The tool chest is brought in.
fn kitchen() -> (Job, String, String, [String; 3]) {
    let (mut job, level, w6, outer) = house();
    ed::mark(&mut job, &[outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    let new = ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [36.0, 0.0], [36.0, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    ed::room(&mut job, &level, "Living", [6.0, 8.0], known()).unwrap();
    let kitchen = ed::room(&mut job, &level, "Kitchen", [30.0, 8.0], known()).unwrap();
    ed::room_lines(&mut job, &level, &[[24.0, 0.0], [24.0, 16.0]], assumed()).unwrap();
    tables::items_in(&mut job, &starter_rows(tables::STARTER_ITEMS), None).unwrap();
    (job, level, kitchen, [new[0].clone(), new[1].clone(), new[2].clone()])
}

fn codes(text: &str) -> Vec<String> {
    ed::box_list(text).unwrap()
}

/// Point 2 of the kitchen: a box that would stand in another thing set in place is refused,
/// naming it and by how much, and the job is as it was. The east run B18 SB36 from 1 ft fills
/// x = 33.75 to 35.75, y = 1 to 5.
#[test]
fn a_box_that_would_stand_in_another_thing_is_refused_and_the_job_is_as_it_was() {
    let (mut job, level, kitchen, [south, east, _]) = kitchen();
    assert_eq!(ed::box_codes(&mut job).len(), 27);
    ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &codes("B18 SB36"), false, assumed()).unwrap();
    let before = save(&mut job);
    // A second run, the same boxes from the same place: over the first.
    let err = ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &codes("B18 SB36"), false, assumed()).unwrap_err();
    assert!(err.contains("B18 would stand in B18 (furnishing-") && err.contains(" in: set the run clear of it") && err.contains(" by "), "{err}");
    assert_eq!(save(&mut job), before);
    // Two runs meeting in an inside corner: the south wall's run to the corner, B36 from 8.75 ft
    // (x = 32.75 to 35.75, y = 0.25 to 2.25), stands in the B18 of the east run (y from 1).
    let err = ed::cabinets(&mut job, &level, &south, &kitchen, "8.75 ft", &codes("B36"), false, assumed()).unwrap_err();
    assert!(err.contains("B36 would stand in B18 (furnishing-") && err.contains("24 in for base boxes"), "{err}");
    assert_eq!(save(&mut job), before);
    // Off the corner by what the east run fills (24 in): it ends at 9.75 ft, x = 33.75: taken, and clash free.
    let clashes_before = clashes(&mut job, None).unwrap().found.len();
    let run = ed::cabinets(&mut job, &level, &south, &kitchen, "6.75 ft", &codes("B36"), false, assumed()).unwrap();
    assert_eq!((run.from, run.run, run.left), (6.75, 3.0, 2.0));
    assert_eq!(clashes(&mut job, None).unwrap().found.len(), clashes_before);
}

/// Windows. The north wall, 12 ft, its face clear 0.25 to 12 ft along (it runs west from x = 36).
/// Window 1 from 2 to 5 ft, sill 3 ft (36 in), head 7 ft; window 2 from 8 to 11 ft, sill 2 ft
/// (24 in), head 6 ft. A wall box (54 to 84 in) and a tall box (0 to 84 in) are across window 1;
/// a base box (0 to 34 1/2 in) is not under window 1, whose sill is above its top, and is under
/// window 2, whose sill is below it.
#[test]
fn a_box_across_a_window_is_refused_and_a_base_box_under_a_high_sill_is_taken() {
    let (mut job, level, kitchen, [_, _, north]) = kitchen();
    let win = ed::opening_type(&mut job, "window 3 x 4", Kind::Window, "3 ft", "4 ft", known()).unwrap();
    ed::opening(&mut job, &north, Kind::Window, &win, ed::OnWall::Offset("3.5 ft"), "3 ft", known()).unwrap();
    ed::opening(&mut job, &north, Kind::Window, &win, ed::OnWall::Offset("9.5 ft"), "2 ft", known()).unwrap();
    let before = save(&mut job);
    let err = ed::cabinets(&mut job, &level, &north, &kitchen, "2.5 ft", &codes("W3030"), false, assumed()).unwrap_err();
    assert!(err.contains("W3030 hangs from 2' 6\" to 5' 0\" along") && err.contains("across window 1 ("), "{err}");
    let err = ed::cabinets(&mut job, &level, &north, &kitchen, "2.5 ft", &codes("T2484"), false, assumed()).unwrap_err();
    assert!(err.contains("T2484 stands from") && err.contains("across window 1 ("), "{err}");
    let err = ed::cabinets(&mut job, &level, &north, &kitchen, "8.5 ft", &codes("B30"), false, assumed()).unwrap_err();
    assert!(err.contains("across window 2 (") && err.contains("sill is at or above its top"), "{err}");
    assert_eq!(save(&mut job), before);
    // Under window 1: its sill, 36 in, is above the box's 34 1/2 in top.
    ed::cabinets(&mut job, &level, &north, &kitchen, "2.25 ft", &codes("B30"), false, assumed()).unwrap();
    // A box beside a window, not across it, is taken: W1230 from 5 ft to 6 ft.
    ed::cabinets(&mut job, &level, &north, &kitchen, "5 ft", &codes("W1230"), false, assumed()).unwrap();
}

/// Names. `B18 [Dishwasher] SB36 [Range, 30 in]`: 18 + 24 + 36 + 30 = 108 in, 9 ft. The
/// countertop goes over what stands no taller than 34 1/2 in: B18 (34 1/2), the dishwasher (34),
/// SB36 (34 1/2): 18 + 24 + 36 = 78 in = 6.5 ft, not over the range (36 in). 6.5 ft x 25 1/2 in
/// (2.125 ft) = 13.8125 SF. The supplier's list says Assumed for boxes set Assumed; a price by
/// its code goes on every box of it.
#[test]
fn appliances_go_in_a_run_by_name_and_the_countertop_is_not_laid_over_a_range() {
    assert_eq!(codes("B18 [Dishwasher] SB36 [Range, 30 in]"), ["B18", "Dishwasher", "SB36", "Range, 30 in"]);
    assert!(ed::box_list("B18 [Dishwasher").unwrap_err().contains("not closed"));
    assert!(ed::box_list("B18 []").unwrap_err().contains("no name"));
    let (mut job, level, kitchen, [_, _, north]) = kitchen();
    let run = ed::cabinets(&mut job, &level, &north, &kitchen, "1 ft", &codes("B18 [Dishwasher] SB36 [Range, 30 in]"), true, assumed()).unwrap();
    assert_eq!((run.run, run.from, run.left), (9.0, 1.0, 2.0));
    assert_eq!(run.boxes.iter().map(|b| b.1.as_str()).collect::<Vec<_>>(), ["B18", "Dishwasher", "SB36", "Range, 30 in"]);
    let (top, mid) = run.countertop.clone().unwrap();
    assert_eq!(measure(&mut job, &top, "width").value, Some(6.5));
    assert_eq!(measure(&mut job, &top, "plan_area").value, Some(13.8125));
    // Its middle is half of 6.5 ft from where the base boxes start (1 ft), along the wall to the west of
    // x = 36: x = 36 - 1 - 3.25 = 31.75, and 15.75 - 1.0625 = 14.6875 on the plan.
    assert_eq!(mid, [31.75, 14.6875]);
    // Each stands on the floor, back to the wall; the dishwasher carries its line in Division 11.
    let dish = run.boxes[1].0.clone();
    let e = job.find(&dish).unwrap();
    assert_eq!(job.world.get::<Cost>(e).unwrap().lines[0].division.map(|d| d.number()), Some(11));
    assert_eq!(job.world.get::<Placed>(e).unwrap().offset.feet(), Some(0.0));
    // A tall thing in the middle of the base boxes: the countertop is not laid over it.
    let (mut job2, level2, kitchen2, [_, _, north2]) = self::kitchen();
    let err = ed::cabinets(&mut job2, &level2, &north2, &kitchen2, "1 ft", &codes("B18 [Refrigerator] B18"), true, assumed()).unwrap_err();
    assert!(err.contains("Refrigerator stands taller than a countertop in the middle of the base boxes"), "{err}");

    // The supplier's list: Assumed, for what is set Assumed (it read Known).
    let sheet = &lens_out(&mut job, Lens::Supplier)[0];
    let sure_of = |what: &str| -> String {
        let row = sheet.rows.iter().find(|r| matches!(r.first(), Some(Out::Text(t)) if t == what)).unwrap_or_else(|| panic!("{what} is not on the list"));
        match &row[4] {
            Out::Text(t) => t.clone(),
            other => panic!("{other:?}"),
        }
    };
    for what in ["B18", "Dishwasher", "SB36", "Range, 30 in"] {
        assert_eq!(sure_of(what), "Assumed", "{what}");
    }

    // A price by its code: B18 is set in two runs, and one command prices both.
    let (mut job, level, kitchen, [south, east, _]) = self::kitchen();
    ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &codes("B18 B24"), false, assumed()).unwrap();
    ed::cabinets(&mut job, &level, &south, &kitchen, "2 ft", &codes("B18"), false, assumed()).unwrap();
    let before = save(&mut job);
    let none = ed::price_item(&mut job, "B21", "$180", Status::Assumed, "made-up quote").unwrap();
    assert_eq!((none.priced.len(), none.without.len()), (0, 0));
    assert_eq!(save(&mut job), before);
    assert!(ed::price_item(&mut job, "B99", "$180", Status::Assumed, "made-up quote").unwrap_err().contains("is not an item of this job"));
    assert_eq!(save(&mut job), before);
    let done = ed::price_item(&mut job, "B18", "$180", Status::Assumed, "made-up quote").unwrap();
    assert_eq!((done.item.as_str(), done.priced.len(), done.without.len()), ("B18", 2, 0));
    let est = estimate(&mut job);
    let b18: Vec<_> = est.lines.iter().filter(|l| l.description == "B18").collect();
    assert_eq!(b18.len(), 2);
    assert!(b18.iter().all(|l| l.amount.value == Some(180.0)), "{b18:?}");
    // B24 was not touched.
    let b24 = est.lines.iter().find(|l| l.description == "B24").unwrap();
    assert!(b24.unit_cost.is_none() || b24.unit_cost.as_ref().unwrap().value.is_none());
}

/// A run on a wall that is to demolish is refused for that reason; a wall marked to demolish, or
/// removed, says what is set against it and still stands, and moves and removes nothing.
#[test]
fn a_wall_to_demolish_is_no_wall_for_a_run_and_says_what_stands_against_it() {
    let (mut job, level, kitchen, [south, east, _]) = kitchen();
    ed::cabinets(&mut job, &level, &east, &kitchen, "1 ft", &codes("B18 B18 B18"), false, assumed()).unwrap();
    ed::cabinets(&mut job, &level, &east, &kitchen, "8 ft", &codes("W1830"), false, assumed()).unwrap();
    assert!(ed::set_against_said(&mut job, &south).is_none(), "nothing stands against the south wall");
    let said = ed::set_against_said(&mut job, &east).unwrap();
    assert!(said.starts_with("4 things set against it still stand: B18 (furnishing-") && said.ends_with("; move or remove them"), "{said}");
    // Marked to demolish: it says so, the boxes stand and keep their lines.
    let said = ed::mark(&mut job, &[east.clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    assert!(said.iter().any(|l| l.contains("4 things set against it still stand: ") && l.contains("move or remove them")), "{said:?}");
    assert_eq!(ed::box_codes(&mut job).len(), 27);
    let boxes = job.order.iter().filter(|&&e| job.world.get::<Placed>(e).is_some()).count();
    assert_eq!(boxes, 4);
    // A run on it is refused for that reason, the job as it was.
    let before = save(&mut job);
    let err = ed::cabinets(&mut job, &level, &east, &kitchen, "10 ft", &codes("B18"), false, assumed()).unwrap_err();
    assert!(err.contains("is to demolish: boxes set against a wall that is coming down") && !err.contains("no face"), "{err}");
    assert_eq!(save(&mut job), before);
}

/// A room line that reaches no wall and no other room line at an end is taken and said.
#[test]
fn a_room_line_that_reaches_nothing_is_said() {
    let (mut job, level, _, _) = house();
    ed::room(&mut job, &level, "A", [6.0, 8.0], known()).unwrap();
    // A stub from outside the house to inside the room: neither end meets anything.
    let made = ed::room_lines(&mut job, &level, &[[12.0, -5.0], [12.0, 5.0]], known()).unwrap();
    let said = ed::room_lines_unreached_said(&mut job, &made);
    assert_eq!(said.len(), 2, "{said:?}");
    assert!(said[0].ends_with("its end at 12,-5 meets no wall and no room line: it parts nothing until it does"), "{said:?}");
    assert!(said[1].ends_with("its end at 12,5 meets no wall and no room line: it parts nothing until it does"), "{said:?}");
    // A line across the house meets the south wall and the north wall: nothing to say.
    let made = ed::room_lines(&mut job, &level, &[[12.0, 0.0], [12.0, 16.0]], known()).unwrap();
    assert!(ed::room_lines_unreached_said(&mut job, &made).is_empty());
    // A line ending on another room line meets it.
    let made = ed::room_lines(&mut job, &level, &[[6.0, 8.0], [12.0, 8.0]], known()).unwrap();
    let said = ed::room_lines_unreached_said(&mut job, &made);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("its end at 6,8 meets no wall"), "{said:?}");
}

/// `options`: the "left out of the job cost" row, with each layout's count of lines the estimate
/// leaves out; none reads none; the last less the first.
#[test]
fn options_say_what_each_job_cost_leaves_out() {
    let make = |priced: bool| {
        let (mut job, level, _, _) = house();
        let room = ed::room(&mut job, &level, "Living", [6.0, 8.0], known()).unwrap();
        job.add_line(&room, "Flooring", "floor_area", Some(Class::Material), false).unwrap();
        job.set(&room, "unit_cost", if priced { Some("$8.00/SF") } else { None }, if priced { Status::Known } else { Status::Unknown }, "made-up price").unwrap();
        job.derive();
        job
    };
    let (mut a, mut b) = (make(true), make(false));
    let table = options(&mut [&mut a, &mut b]);
    let row = table.rows.iter().find(|r| r.label == "left out of the job cost").unwrap();
    assert_eq!(row.figures, [OptionFigure::Is(0.0, Status::Known), OptionFigure::Is(1.0, Status::Known)]);
    assert_eq!(row.figures.iter().map(|f| f.said(row.unit)).collect::<Vec<_>>(), ["none", "1 line"]);
    assert_eq!(row.difference_said(), "+1 line");
}
