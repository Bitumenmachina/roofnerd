//! Call 167: what the judge of calls 159 to 166 found, set right. The houses are made up;
//! the figures are worked by hand, as in STATUS.md ("The judge's findings on calls 159 to
//! 166 set right").

use plumbline_world::editing as ed;
use plumbline_world::file::{load, save};
use plumbline_world::units;
use plumbline_world::value::Status;
use plumbline_world::view::{estimate, lump_lines};
use plumbline_world::vif::vif;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up layout" }
}

/// A 24 ft by 16 ft house, 9 ft walls, and a partition at east 12: the level, the outer walls
/// (south, east, north, west) and the partition.
fn house() -> (Job, String, Vec<String>, String) {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let part = ed::walls(&mut job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap();
    (job, level, outer, part[0].clone())
}

fn amount(job: &mut Job, path: &str, what: &str) -> Option<f64> {
    estimate(job).lines.iter().find(|l| l.path == path && l.description == what).and_then(|l| l.amount.value)
}

// 1. A typed measure that is not Known is a field question.

#[test]
fn a_typed_measure_not_known_is_a_field_question_with_its_lines_dollars_riding() {
    let (mut job, _, outer, _) = house();
    let south = &outer[0];
    job.add_line_measured(south, "Patch and match siding", "60 SF", Some(Class::Subcontract), false, Status::Assumed, "walked it").unwrap();
    job.set_on_line(south, "unit_cost", Some("Patch and match siding"), Some("$14/SF"), Status::Known, "made-up price").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, south, "Patch and match siding"), Some(840.0));
    let list = vif(&mut job);
    assert_eq!(list.questions.len(), 1);
    let q = &list.questions[0];
    assert_eq!((q.path.as_str(), q.dollars, q.lines), (south.as_str(), Some(840.0), 1));
    assert_eq!(q.values, vec![("\"Patch and match siding\" (60 SF)".to_string(), Status::Assumed, true)]);
    assert_eq!(q.to_do(), "Measure \"Patch and match siding\" (60 SF).");
    // Measured Known, it leaves.
    ed::measure_on_line(&mut job, south, "Patch and match siding", "measured", ed::Reading::Same, "made-up tape").unwrap();
    job.derive();
    assert!(vif(&mut job).questions.is_empty());
}

#[test]
fn an_assemblys_rule_value_is_not_a_question_of_the_site() {
    let (mut job, _, outer, _) = house();
    let south = &outer[0];
    job.add_line(south, "Fill", "plan_length", Some(Class::Subcontract), false).unwrap();
    // A strip's width is in feet, an allowance and not a measure of the building.
    job.set_formula(south, Some("Fill"), "plan_length * strip", &[("strip".to_string(), "2 ft".to_string())], Status::Assumed, "made-up allowance").unwrap();
    job.derive();
    assert!(vif(&mut job).questions.is_empty());
}

#[test]
fn a_typed_measure_with_no_figure_is_asked_too() {
    let (mut job, _, outer, _) = house();
    let south = &outer[0];
    job.add_line_measured(south, "Patch", "60 SF", Some(Class::Subcontract), false, Status::Assumed, "walked it").unwrap();
    job.set_on_line(south, "measured", Some("Patch"), None, Status::Unknown, "not walked yet").unwrap();
    job.derive();
    let list = vif(&mut job);
    assert_eq!(list.questions.len(), 1);
    assert_eq!(list.questions[0].values, vec![("\"Patch\" (no figure yet)".to_string(), Status::Unknown, true)]);
    assert_eq!(list.questions[0].dollars, None, "nothing priced rides on it: no dollars, not $0.00");
}

// 6. Measures as estimators write them.

#[test]
fn measures_take_thousands_and_the_common_spellings() {
    for (typed, want) in [
        ("1,200 SF", (1200.0, "SF")),
        ("60 sq ft", (60.0, "SF")),
        ("60 sqft", (60.0, "SF")),
        ("60 Sq. Ft.", (60.0, "SF")),
        ("24 lin ft", (24.0, "LF")),
        ("3 sq", (3.0, "SQ")),
        ("3 Square", (3.0, "SQ")),
        ("3 squares", (3.0, "SQ")),
        ("2 cu yd", (2.0, "CY")),
        ("4 each", (4.0, "EA")),
        ("12,500.5 lf", (12500.5, "LF")),
    ] {
        assert_eq!(units::parse_measure(typed).unwrap(), want, "{typed}");
    }
    // A length typed on a line is LF; on its own, "ft" is a length and not a measure.
    assert_eq!(units::parse_measure_on_line("24 ft").unwrap(), (24.0, "LF"));
    assert!(units::parse_measure("24 ft").is_err());
    // Below zero is said as that; what is not a measure is refused with the units it takes.
    let below = units::parse_measure("-3 SF").unwrap_err();
    assert!(below.contains("below zero"), "{below}");
    let boards = units::parse_measure("60 boards").unwrap_err();
    assert!(boards.contains("LF, SF, SQ, CY, EA"), "{boards}");
    let comma = units::parse_measure("1,25 SF").unwrap_err();
    assert!(comma.contains("comma"), "{comma}");
    assert!(units::looks_like_measure("60 boards") && units::looks_like_measure("-3 SF") && units::looks_like_measure("1,200 sq ft"));
    assert!(!units::looks_like_measure("1 + 1") && !units::looks_like_measure("plan_length") && !units::looks_like_measure("4"));
}

#[test]
fn a_typed_measure_is_set_with_the_spellings_and_refused_in_words_leaving_the_job() {
    let (mut job, _, outer, _) = house();
    let south = &outer[0];
    job.add_line_measured(south, "Patch", "60 SF", Some(Class::Subcontract), false, Status::Assumed, "walked it").unwrap();
    job.set_on_line(south, "unit_cost", Some("Patch"), Some("$14/SF"), Status::Known, "made-up price").unwrap();
    job.set_on_line(south, "measured", Some("Patch"), Some("1,200 SF"), Status::Assumed, "walked it again").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, south, "Patch"), Some(16800.0));
    job.set_on_line(south, "measured", Some("Patch"), Some("60 sq ft"), Status::Assumed, "walked it again").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, south, "Patch"), Some(840.0));
    let before = save(&mut job);
    let why = job.set_on_line(south, "measured", Some("Patch"), Some("-3 SF"), Status::Assumed, "x").unwrap_err();
    assert!(why.contains("below zero"), "{why}");
    let why = job.set_on_line(south, "measured", Some("Patch"), Some("60 boards"), Status::Assumed, "x").unwrap_err();
    assert!(why.contains("LF, SF, SQ, CY, EA"), "{why}");
    assert_eq!(save(&mut job), before, "a refusal leaves the job as it was");
    // Measured with a tape, the same words.
    let why = ed::measure_on_line(&mut job, south, "Patch", "measured", ed::Reading::Length("60 boards"), "tape").unwrap_err();
    assert!(why.contains("LF, SF, SQ, CY, EA"), "{why}");
    let done = ed::measure_on_line(&mut job, south, "Patch", "measured", ed::Reading::Length("1,200 sq ft"), "tape").unwrap();
    assert!(done.said.contains("1200 SF"), "{}", done.said);
}

#[test]
fn a_length_typed_on_a_line_is_lf_and_a_line_by_its_own_figure_reads_in_lf() {
    let (mut job, _, outer, _) = house();
    let south = &outer[0];
    job.add_line_measured(south, "Edge", "24 ft", Some(Class::Subcontract), false, Status::Assumed, "walked it").unwrap();
    job.derive();
    let line = estimate(&mut job).lines.iter().find(|l| l.description == "Edge").cloned().unwrap();
    assert_eq!((line.quantity.value, line.quantity.unit.as_str()), (Some(24.0), "LF"));
}

#[test]
fn bearing_takes_the_words_a_trade_writes_and_says_its_two() {
    use plumbline_world::Bearing;
    for typed in ["bearing", "Bearing", "load bearing", "Load-Bearing", " LOAD  BEARING "] {
        assert_eq!(Bearing::read(typed).unwrap(), Bearing::Bearing, "{typed}");
    }
    for typed in ["not bearing", "NOT BEARING", "non-bearing", "Non-Bearing", "nonbearing", "NonBearing"] {
        assert_eq!(Bearing::read(typed).unwrap(), Bearing::NotBearing, "{typed}");
    }
    let why = Bearing::read("bearing wall").unwrap_err();
    assert!(why.contains("\"bearing\" or \"not bearing\""), "{why}");
    let (mut job, _, _, part) = house();
    job.set(&part, "bearing", Some("non-bearing"), Status::Known, "seen").unwrap();
    assert_eq!(job.shown(&part, "bearing", None).as_deref(), Some("not bearing"));
    // "in the wall" is taken in any capitals, joined or not.
    job.set(&part, "In_The_Wall", Some("one 2 in drain"), Status::Known, "opened").unwrap();
    job.set(&part, "inthewall", Some("nothing"), Status::Known, "opened").unwrap();
    assert_eq!(job.shown(&part, "in_the_wall", None).as_deref(), Some("nothing"));
    // The list of what a thing takes names both words of a wall.
    let level = job.find(&part).and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone()).unwrap();
    let why = job.set(&level, "colour", Some("red"), Status::Known, "x").unwrap_err();
    assert!(why.contains("bearing") && why.contains("in the wall"), "{why}");
}

// 4. The place given for a thing in the way of a wall that comes down.

#[test]
fn the_place_given_for_a_pipe_in_a_wall_is_on_the_pipe_and_inside_the_wall() {
    let (mut job, level, _, part) = house();
    let pipe = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, known()).unwrap();
    ed::mark(&mut job, std::slice::from_ref(&part), WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    // Three pipes meeting at east 12, north 14, 4 ft up: A east to 15,14 through the partition (6 in
    // thick about east 12: 11.75 to 12.25), B from the west, C down the partition to 12,12.
    let a = ed::runs(&mut job, &level, Kind::Pipe, &pipe, &[[12.0, 14.0], [15.0, 14.0]], "4 ft", None, known()).unwrap().remove(0);
    let b = ed::runs(&mut job, &level, Kind::Pipe, &pipe, &[[6.0, 14.0], [12.0, 14.0]], "4 ft", None, known()).unwrap().remove(0);
    let c = ed::runs(&mut job, &level, Kind::Pipe, &pipe, &[[12.0, 14.0], [12.0, 12.0]], "4 ft", None, known()).unwrap().remove(0);
    let found = plumbline_world::building::clash::in_the_way(&mut job).unwrap();
    assert_eq!(found.len(), 3);
    let at = |path: &str| found.iter().find(|w| w.thing == path).map(|w| w.at).unwrap();
    // A: from east 12 to 12.25 is in the wall, its middle 12.125.
    assert_eq!(at(&a), [12.125, 14.0, 4.0]);
    // B: from east 11.75 to 12 is in the wall, its middle 11.875.
    assert_eq!(at(&b), [11.875, 14.0, 4.0]);
    // C: all of it is in the wall, north 14 to 12: its middle.
    assert_eq!(at(&c), [12.0, 13.0, 4.0]);
}

#[test]
fn a_pipe_only_its_body_reaches_into_the_wall_is_given_a_place_in_the_wall_still() {
    let (mut job, level, _, part) = house();
    let pipe = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, known()).unwrap();
    ed::mark(&mut job, std::slice::from_ref(&part), WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    // Its centre line at east 11.70 is 0.05 ft outside the wall's face at 11.75; its 2 in body is not.
    let p = ed::runs(&mut job, &level, Kind::Pipe, &pipe, &[[11.7, 4.0], [11.7, 10.0]], "4 ft", None, known()).unwrap().remove(0);
    let found = plumbline_world::building::clash::in_the_way(&mut job).unwrap();
    let at = found.iter().find(|w| w.thing == p).map(|w| w.at).unwrap();
    assert!((11.75..=12.25).contains(&at[0]) && (4.0..=10.0).contains(&at[1]) && at[2] == 4.0, "{at:?}");
}

// 5. An assembly less a line.

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

/// The kitchen of call 162: 6 in walls 9 ft tall round 0,0 to 12,16, a 3 ft door in the west
/// wall, the room "Kitchen", the starter assemblies and items, the Kitchen on the room.
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

#[test]
fn an_assembly_less_a_line_says_so_and_is_made_whole_touching_no_other_line() {
    let (mut job, _, room, _) = kitchen();
    let all = |job: &mut Job| -> Vec<String> {
        let e = job.find(&room).unwrap();
        job.world.get::<Cost>(e).unwrap().lines.iter().map(|l| l.description.clone()).collect()
    };
    let whole = all(&mut job);
    assert!(whole.iter().any(|l| l == "Cabinets"));
    // Whole, it says nothing of what it lacks, and putting it on again is refused in words.
    let before = save(&mut job);
    let why = job.apply(&room, "Kitchen", "made-up").unwrap_err();
    assert!(why.contains("has every line of it"), "{why}");
    assert_eq!(save(&mut job), before);
    // A price entered on another line stays through it all.
    job.set_on_line(&room, "unit_cost", Some("Base and trim"), Some("$4/LF"), Status::Known, "made-up price").unwrap();
    job.remove_line(&room, "Cabinets").unwrap();
    let rows = plumbline_world::view::elements(&mut job).into_iter().find(|e| e.path == room).unwrap().rows;
    let condition = rows.iter().find(|r| r.label == "condition").unwrap();
    assert_eq!((condition.text.as_str(), condition.note.as_str()), ("Kitchen", "without its line \"Cabinets\""));
    let less = all(&mut job);
    assert_eq!(less.len(), whole.len() - 1);
    // Put on again: that one line back, said, and no other touched.
    let added = job.apply(&room, "Kitchen", "made-up").unwrap();
    assert_eq!(added, vec!["Cabinets".to_string()]);
    let mut back = all(&mut job);
    assert_eq!(back.pop().as_deref(), Some("Cabinets"));
    assert_eq!(back, less);
    let room_entity = job.find(&room).unwrap();
    let base = job.world.get::<Cost>(room_entity).unwrap().lines.iter().find(|l| l.description == "Base and trim").unwrap().unit_cost.value;
    assert_eq!(base, Some(4.0), "the price on another line is as it was");
    let rows = plumbline_world::view::elements(&mut job).into_iter().find(|e| e.path == room).unwrap().rows;
    assert_eq!(rows.iter().find(|r| r.label == "condition").unwrap().note, "");
    // Taken off and put on with the change verb: the same.
    job.remove_line(&room, "Countertops").unwrap();
    let (added, taken) = job.change_assembly(&room, "Kitchen", Status::Known, "made-up").unwrap();
    assert_eq!((added, taken), (vec!["Countertops".to_string()], Vec::<String>::new()));
    let why = job.change_assembly(&room, "Kitchen", Status::Known, "made-up").unwrap_err();
    assert!(why.contains("has every line of it"), "{why}");
}

// 7. A lump line names no figure and is more than nothing.

#[test]
fn a_lump_line_is_a_number_numbers_together_or_a_count_typed_and_never_a_zero() {
    let (mut job, level, room, [south, ..]) = kitchen();
    let codes = ed::box_list("B18 B24").unwrap();
    ed::cabinets(&mut job, &level, &south, &room, "3 in", &codes, false, assumed()).unwrap();
    job.derive();
    let say = |job: &mut Job| lump_lines(job).into_iter().map(|l| (l.line, l.quantity)).collect::<Vec<_>>();
    let before = say(&mut job);
    assert!(before.iter().all(|(l, _)| l != "Hardware"));
    let twelve = Division::of_number(12).unwrap();
    job.add_line(&room, "Hardware", "1 + 1", Some(Class::Material), false).unwrap();
    job.set_division(&room, Some("Hardware"), twelve).unwrap();
    job.add_line_measured(&room, "Handles", "4 EA", Some(Class::Material), false, Status::Assumed, "made-up count").unwrap();
    job.set_division(&room, Some("Handles"), twelve).unwrap();
    job.add_line(&room, "Nothing at all", "0", Some(Class::Material), false).unwrap();
    job.set_division(&room, Some("Nothing at all"), twelve).unwrap();
    job.add_line(&room, "By the floor", "floor_area", Some(Class::Material), false).unwrap();
    job.set_division(&room, Some("By the floor"), twelve).unwrap();
    job.derive();
    let after = say(&mut job);
    assert!(after.contains(&("Hardware".to_string(), 2.0)), "{after:?}");
    assert!(after.contains(&("Handles".to_string(), 4.0)), "{after:?}");
    assert!(after.iter().all(|(l, _)| l != "Nothing at all" && l != "By the floor"), "{after:?}");
    // The ones that were lumps still are.
    for was in &before {
        assert!(after.contains(was));
    }
}

// 8. `place` takes an item by its name.

#[test]
fn place_takes_an_item_by_its_name_in_any_capitals_or_by_its_number() {
    let (mut job, level, _, _) = kitchen();
    let item = {
        let mut q = job.world.query::<(&Node, &Item)>();
        q.iter(&job.world).find(|(n, _)| n.name == "B36").map(|(n, _)| n.path.clone()).unwrap()
    };
    let by_name = ed::place_item(&mut job, &level, "B36", [6.0, 8.0], "0 ft", assumed()).unwrap();
    let by_capitals = ed::place_item(&mut job, &level, "b36", [6.0, 10.0], "0 ft", assumed()).unwrap();
    let by_number = ed::place_item(&mut job, &level, &item, [6.0, 12.0], "0 ft", assumed()).unwrap();
    for path in [by_name, by_capitals, by_number] {
        let e = job.find(&path).unwrap();
        assert_eq!(job.world.get::<Placed>(e).unwrap().item.as_deref(), Some(item.as_str()));
    }
    // What names no item is refused, and the job is as it was.
    let before = save(&mut job);
    let why = ed::place_item(&mut job, &level, "B999", [6.0, 8.0], "0 ft", assumed()).unwrap_err();
    assert!(why.contains("B999"), "{why}");
    assert_eq!(save(&mut job), before);
}

// 9. `mark` asks only of what has not been said.

#[test]
fn mark_asks_only_what_has_not_been_said() {
    let (mut job, _, outer, part) = house();
    let all: Vec<String> = outer.iter().cloned().chain([part.clone()]).collect();
    ed::mark(&mut job, &all, WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    let ask = |job: &mut Job| ed::mark(job, std::slice::from_ref(&part), WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap().last().cloned().unwrap();
    let existing = |job: &mut Job| ed::mark(job, std::slice::from_ref(&part), WorkStatus::Existing, ed::ItsLines::Ask, known()).unwrap();
    assert_eq!(ask(&mut job), "Is it bearing, and what is in it? Both are on the field list.");
    existing(&mut job);
    // Bearing said: only the contents are asked.
    job.set(&part, "bearing", Some("not bearing"), Status::Known, "seen").unwrap();
    assert_eq!(ask(&mut job), "What is in it? That is on the field list.");
    existing(&mut job);
    // Both said: nothing is asked.
    job.set(&part, "in_the_wall", Some("nothing"), Status::Known, "opened").unwrap();
    let said = ed::mark(&mut job, std::slice::from_ref(&part), WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    assert_eq!(said, vec![format!("{part}: to demolish")]);
    existing(&mut job);
    // Contents said, bearing not.
    job.set(&part, "bearing", None, Status::Unknown, "not seen again").unwrap();
    assert_eq!(ask(&mut job), "Is it bearing? That is on the field list.");
}

// 2. A line bought by a length reads in LF (the countertop).

#[test]
fn a_countertop_is_priced_by_the_lf_of_its_front_edge_or_the_sf_of_its_top() {
    let (mut job, level, room, [south, east, ..]) = kitchen();
    // Run A: LS36 B24 SB36 from 3 in along the east wall, 8 ft with its top; run B: B18 DB18 from
    // 5 ft 9 in along the south wall, 3 ft with its top.
    let a = ed::cabinets(&mut job, &level, &east, &room, "3 in", &ed::box_list("LS36 B24 SB36").unwrap(), true, assumed()).unwrap().countertop.unwrap().0;
    let b = ed::cabinets(&mut job, &level, &south, &room, "5 ft 9 in", &ed::box_list("B18 DB18").unwrap(), true, assumed()).unwrap().countertop.unwrap().0;
    job.derive();
    let before = save(&mut job);
    // Any other unit is refused with the two it takes, and the job is as it was.
    let why = ed::price_item(&mut job, "Countertop", "$5/EA", Status::Assumed, "made-up quote").unwrap_err();
    assert!(why.contains("LF of its front edge or by the SF of its top"), "{why}");
    assert_eq!(save(&mut job), before);
    // By the LF: A is 8 ft of front edge, B 3 ft: 8 x 80 = 640 and 3 x 80 = 240, each read in LF.
    let done = ed::price_item(&mut job, "Countertop", "$80/LF", Status::Assumed, "made-up quote").unwrap();
    assert_eq!(done.priced, vec![a.clone(), b.clone()]);
    job.derive();
    for (path, quantity, money) in [(&a, 8.0, 640.0), (&b, 3.0, 240.0)] {
        let line = estimate(&mut job).lines.iter().find(|l| &l.path == path && l.description == "Countertop").cloned().unwrap();
        assert_eq!((line.quantity.value, line.quantity.unit.as_str(), line.amount.value), (Some(quantity), "LF", Some(money)), "{path}");
        assert_eq!(line.unit_cost.as_ref().map(|u| (u.value, u.unit.as_str())), Some((Some(80.0), "$/LF")));
    }
    // Back to the area: 17 x 62 = 1,054 and 6.375 x 62 = 395.25.
    ed::price_item(&mut job, "Countertop", "$62/SF", Status::Assumed, "made-up quote").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, &a, "Countertop"), Some(1054.0));
    assert_eq!(amount(&mut job, &b, "Countertop"), Some(395.25));
    let line = estimate(&mut job).lines.iter().find(|l| l.path == a && l.description == "Countertop").cloned().unwrap();
    assert_eq!(line.quantity.unit, "SF");
    // And saved and read back the same.
    let text = save(&mut job);
    let (mut back, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut back), text);
}
