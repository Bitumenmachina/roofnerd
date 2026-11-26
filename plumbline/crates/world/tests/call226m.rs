//! Call 226, the model's side (contract 226M): a length in a sentence is said as a builder says it, a cabinet rule can be taken away, and a
//! height the rule does not stock is flagged as a width is. Written before the code. Everything is made up; the figures are worked by hand
//! beside each test.
//!
//! The job is that of `call219m.rs`: one level, Ground, 9 ft high; four walls 6 in thick round 20 ft by 12 ft; a door 3 ft by 6 ft 8 in in
//! the south wall, its middle 4 ft from the wall's first end.

use plumbline_world::building::cabinet;
use plumbline_world::editing::{self as ed, Change, Entered, HowTall};
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

fn job_with_walls() -> (Job, String, String, Vec<String>) {
    let mut job = Job::new("Sentences (not a real job)", "test", "2026-11-26");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "Wall 6", "6 in", KNOWN).unwrap();
    let door_type = ed::opening_type(&mut job, "Door 3068", Kind::Door, "3 ft", "6 ft 8 in", KNOWN).unwrap();
    let walls = ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [20.0, 0.0], [20.0, 12.0], [0.0, 12.0]], true, ed::Top::Height("9 ft"), KNOWN).unwrap();
    ed::opening(&mut job, &walls[0], Kind::Door, &door_type, ed::OnWall::Offset("4 ft"), "0 ft", KNOWN).unwrap();
    job.derive();
    (job, level, wall_type, walls)
}

// ------------------------------------------------------------------ point 2: a length in a sentence

/// How a length is said: whole feet as feet, feet and inches as both, under a foot as inches, to the sixteenth; never with two places of
/// decimals. A length that is not a sixteenth (0.4 in: a 3/8 in pipe is 0.375 in; a measured 0.4) is said as it is, not rounded to a
/// sixteenth it is not.
#[test]
fn a_length_is_said_as_a_builder_says_it() {
    // By hand: 10 ft; 7.5 ft = 7 ft 6 in; 14 in = 1 ft 2 in; 0.5 ft = 6 in; 4 13/16 in = 0.40104 ft; 0.4 in = 0.03333 ft.
    assert_eq!(units::length_said(10.0), "10 ft");
    assert_eq!(units::length_said(7.5), "7 ft 6 in");
    assert_eq!(units::length_said(14.0 / 12.0), "1 ft 2 in");
    assert_eq!(units::length_said(0.5), "6 in");
    assert_eq!(units::length_said(4.8125 / 12.0), "4 13/16 in");
    assert_eq!(units::length_said(0.4 / 12.0), "0.4 in");
    assert_eq!(units::length_said(0.0), "0 in");
    assert_eq!(units::length_said(1000.0), "1,000 ft", "a thousand separates as the estimate's figures do");
    assert_eq!(units::length_said(f64::NAN), "unknown");
}

/// The refusal for a wall drawn over another: "lies along wall 1 for 10 ft", not "10.00 ft". (call 219 point 6: the older words the new verbs pass on.)
#[test]
fn a_wall_over_a_wall_says_how_far_as_a_builder_says_it() {
    let (mut job, level, wall_type, _) = job_with_walls();
    let change = Change::Walls { storey: level.clone(), type_path: wall_type.clone(), points: vec![[2.0, 0.0], [12.0, 0.0]], closed: false, tall: HowTall::Height("9 ft".into()) };
    let tried = ed::try_change(&mut job, &change, KNOWN);
    let why = tried.refused.expect("refused");
    assert!(why.contains("lies along wall 1 for 10 ft: one wall drawn over another"), "{why}");
    assert!(!why.contains("10.00"), "{why}");
    // 20 ft over the whole south wall; 7.5 ft is 7 ft 6 in.
    let over_all = Change::Walls { storey: level.clone(), type_path: wall_type.clone(), points: vec![[0.0, 0.0], [20.0, 0.0]], closed: false, tall: HowTall::Height("9 ft".into()) };
    assert!(ed::try_change(&mut job, &over_all, KNOWN).refused.unwrap().contains("for 20 ft"));
    let part = Change::Walls { storey: level, type_path: wall_type, points: vec![[2.0, 0.0], [9.5, 0.0]], closed: false, tall: HowTall::Height("9 ft".into()) };
    assert!(ed::try_change(&mut job, &part, KNOWN).refused.unwrap().contains("for 7 ft 6 in"));
}

/// An opening that runs past an end of its wall, a point off a wall, a top below its base: each says its lengths as a builder says them.
#[test]
fn the_other_sentences_about_a_wall_say_lengths_as_a_builder_does() {
    let (mut job, _, _, walls) = job_with_walls();
    // A window 3 ft wide with its middle 21 ft along the 20 ft south wall, put in by hand as a job saved before holds it: it ends at 22 ft 6 in.
    let window_type = ed::opening_type(&mut job, "Window 3030", Kind::Window, "3 ft", "3 ft", KNOWN).unwrap();
    let len = |x: f64| value::Val::known(units::Len::One(x), "ft", "test");
    let w = job.add(&walls[0], Kind::Window, "too far").unwrap();
    job.put(&w, Opening { type_name: value::Val::known(window_type, "", "test"), offset: len(21.0), sill: len(3.0) }).unwrap();
    job.derive();
    let e = job.find(&walls[0]).unwrap();
    let note = job.world.get::<Measure>(e).unwrap().0.get("net_area").unwrap().note.clone();
    assert!(note.contains("ends 22 ft 6 in along a wall 20 ft long") && !note.contains(".00"), "{note}");
    let door_type = ed::opening_type(&mut job, "Door 3070", Kind::Door, "3 ft", "7 ft", KNOWN).unwrap();
    // A point 500 ft off a wall: "is 500 ft off the middle of wall 1, which is 6 in thick".
    let at = ed::opening(&mut job, &walls[0], Kind::Door, &door_type, ed::OnWall::Point([5.0, 500.0]), "0 ft", KNOWN).unwrap_err();
    assert!(at.contains("is 500 ft off the middle of") && at.contains("which is 6 in thick: it is not on the wall") && !at.contains(".00"), "{at}");
}

/// A hole to cut, in a sentence the try gives: "14 in by 8 in", and a round one "4 in round"; the clash list (a list) keeps "14.00 in by 8.00 in".
#[test]
fn a_hole_in_a_sentence_says_its_size_as_a_builder_does_and_the_list_does_not_move() {
    let (mut job, level, _, _) = job_with_walls();
    let duct_type = ed::segment_type(&mut job, "Duct 14 x 8", Kind::Duct, "14 in", Some("8 in"), KNOWN).unwrap();
    let change = Change::Runs { storey: level, kind: Kind::Duct, type_path: duct_type, points: vec![[-2.0, 6.0], [22.0, 6.0]], height: "8 ft".into(), system: None };
    let tried = ed::try_change(&mut job, &change, KNOWN);
    let said = tried.said().join("\n");
    assert_eq!(said.matches("14 in by 8 in").count(), 2, "{said}");
    assert!(!said.contains("14.00"), "{said}");
    assert!(said.contains("24 ft long"), "{said}");
    // The list: the hole's own size text is what it was.
    ed::apply(&mut job, &change, KNOWN).unwrap();
    let found = building::clash::clashes(&mut job, None).unwrap();
    assert!(found.holes.iter().all(|h| h.size == "14.00 in by 8.00 in"), "{:?}", found.holes.iter().map(|h| &h.size).collect::<Vec<_>>());
}

// ------------------------------------------------------------------ point 4: a rule taken away

fn rows_of(text: &str) -> Vec<Vec<String>> {
    plumbline_world::building::shape::csv_rows(text)
}

const OLD_HEADINGS: &str = "Code,What it is,Number reads,Width,Depth,Height,Stands,Drawn as,Stocked widths,How sure,Where from";

fn kitchen() -> (Job, String, String, String) {
    let mut job = Job::new("A kitchen (not a real job)", "test", "2026-11-26");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", KNOWN).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), KNOWN).unwrap();
    let room = ed::room(&mut job, &level, "Kitchen", [12.0, 8.0], KNOWN).unwrap();
    tables::items_in(&mut job, &rows_of(tables::STARTER_ITEMS), None).unwrap();
    (job, level, outer[1].clone(), room)
}

fn codes(text: &str) -> Vec<String> {
    text.split(' ').map(str::to_string).collect()
}

fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up plan" }
}

fn item_of(job: &mut Job, name: &str) -> Item {
    let (path, _) = ed::box_codes(job).into_iter().find(|(_, n)| n == name).unwrap_or_else(|| panic!("{name} is not in the chest"));
    let e = job.find(&path).unwrap();
    job.world.get::<Item>(e).unwrap().clone()
}

fn inches(v: &value::Val<units::Len>) -> f64 {
    v.value.unwrap().feet(&v.unit).unwrap() * 12.0
}

fn x_rule() -> Vec<Vec<String>> {
    rows_of(&format!("{OLD_HEADINGS}\nX,base,the width,from the number,24 in,30 in,on the floor,base cabinet,,Assumed,a made-up rule\n"))
}

/// Case 1: a rule of the job's own goes. X30 was made by it and stays in the chest, still saying the rule for X made it; X33 is now refused
/// (no rule reads X); the job carries no rules again and is saved as a job that never touched them is.
#[test]
fn a_rule_of_the_jobs_own_is_taken_away_and_the_boxes_it_made_stay() {
    let (mut job, level, east, room) = kitchen();
    let plain = file::save(&mut job);
    tables::cabinet_rules_in(&mut job, &x_rule(), None).unwrap();
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("X30"), false, assumed()).unwrap();
    assert!(run.made[0].contains("X30 was made by the rule for X: 30 in wide, 24 in deep and 30 in high"), "{:?}", run.made);
    let (changed, said) = tables::cabinet_rule_away(&mut job, "X").unwrap();
    assert!(changed);
    assert!(said.contains("the rule for X was taken away") && said.contains("no rule reads X now") && said.contains("X30") && said.contains("stay in the tool chest"), "{said}");
    assert!(job.cabinet_rules.is_empty());
    let refused = ed::cabinets(&mut job, &level, &east, &room, "9 ft", &codes("X33"), false, assumed()).unwrap_err();
    assert!(refused.contains("no cabinet rule reads X"), "{refused}");
    let it = item_of(&mut job, "X30");
    assert_eq!((inches(&it.width), inches(&it.depth), inches(&it.height)), (30.0, 24.0, 30.0));
    assert!(it.width.source.contains("the cabinet rule for X"), "{}", it.width.source);
    // Said of the same job saved again with nothing in it of rules: the key is gone.
    assert!(!file::save(&mut job).contains("cabinetRules"));
    // And a job that never had the rule is what it was (no box of X in it, but the text of the other parts).
    assert!(plain.len() < file::save(&mut job).len());
}

/// Case 2: B changed to 21 in deep, then taken away: B45 is made 24 in deep again (the program's own), and the reply says B is the program's own.
#[test]
fn a_rule_the_job_changed_goes_back_to_the_programs_own() {
    let (mut job, level, east, room) = kitchen();
    let mut rows = rows_of(tables::STARTER_CABINET_RULES);
    for r in rows.iter_mut().skip(1) {
        if r[0] == "B" {
            r[4] = "21 in".into();
        }
    }
    tables::cabinet_rules_in(&mut job, &rows, None).unwrap();
    assert_eq!(job.cabinet_rules.len(), 1);
    let (changed, said) = tables::cabinet_rule_away(&mut job, "B").unwrap();
    assert!(changed);
    assert!(said.contains("B goes back to the program's own") && said.contains("the job's rule for B was taken away"), "{said}");
    assert!(job.cabinet_rules.is_empty());
    ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B45"), false, assumed()).unwrap();
    assert_eq!(inches(&item_of(&mut job, "B45").depth), 24.0);
}

/// Case 3: a rule the job does not carry is said to be the program's own, and nothing changes; a code nothing reads is refused.
#[test]
fn a_rule_the_job_does_not_carry_is_said_to_be_the_programs_own_and_is_left() {
    let (mut job, ..) = kitchen();
    let text = file::save(&mut job);
    let (changed, said) = tables::cabinet_rule_away(&mut job, "b").unwrap();
    assert!(!changed);
    assert!(said.contains("B is the program's own rule and the job carries nothing of it: nothing was changed"), "{said}");
    assert_eq!(file::save(&mut job), text);
    let why = tables::cabinet_rule_away(&mut job, "Q").unwrap_err();
    assert!(why.contains("no cabinet rule reads Q") && why.contains("The letters the rules read are"), "{why}");
    assert_eq!(file::save(&mut job), text);
}

// ------------------------------------------------------------------ point 5: a height the rule does not stock

/// The table's heading gains "Stocked heights" after "Stocked widths"; the starter rows that read a height from the number (W and T) take the
/// heights the chest has for their code: W 12, 18 and 30 in (W3612, W3018, the W--30 boxes), T 84 in (T1884, T2484); a rule with a height of its
/// own says nothing there. The starter's twenty-seven boxes are still exactly what their rules make.
#[test]
fn the_table_has_stocked_heights_and_the_starter_rows_take_the_chests() {
    assert_eq!(cabinet::HEADINGS.len(), 12);
    assert_eq!(cabinet::HEADINGS[8], "Stocked widths");
    assert_eq!(cabinet::HEADINGS[11], "Stocked heights", "last, so every table and test written before it reads as it did");
    let w = cabinet::starter().iter().find(|r| r.code == "W").unwrap();
    assert_eq!(w.stocked_heights, vec![12, 18, 30]);
    let t = cabinet::starter().iter().find(|r| r.code == "T").unwrap();
    assert_eq!(t.stocked_heights, vec![84]);
    for r in cabinet::starter().iter().filter(|r| r.code != "W" && r.code != "T") {
        assert!(r.stocked_heights.is_empty(), "{}: it has a height of its own", r.code);
    }
    let (mut job, ..) = kitchen();
    let out = tables::cabinet_rules_out(&mut job);
    assert_eq!(out[0][11], "Stocked heights");
    assert_eq!(out[0].len(), 12);
    let wrow = out.iter().find(|r| r[0] == "W").unwrap();
    assert_eq!(wrow[11], "12, 18, 30 in");
    // The chest's twenty-seven are what their rules make (the test of call 215), and the starter's heights are the chest's own.
    for (path, name) in ed::box_codes(&mut job) {
        let (letters, digits) = cabinet::read_code(&name).unwrap();
        let rule = cabinet::starter().iter().find(|r| r.code == letters).unwrap();
        let made = rule.make(&name).unwrap();
        let e = job.find(&path).unwrap();
        let held = job.world.get::<Item>(e).unwrap().clone();
        assert_eq!((inches(&made.item.width), inches(&made.item.depth), inches(&made.item.height)), (inches(&held.width), inches(&held.depth), inches(&held.height)), "{name}");
        assert!(!made.not_stocked && !made.not_stocked_height, "{name} ({digits}) is a stocked box of its rule");
    }
}

/// A table without the column reads as before (no height is flagged); one with it reads it back and writes it out unchanged.
#[test]
fn a_table_without_the_column_flags_no_height_and_one_with_it_comes_back_whole() {
    let (mut job, level, east, room) = kitchen();
    // Eleven headings, as a table written before this call: W's rule says nothing of heights.
    let mut old: Vec<Vec<String>> = rows_of(tables::STARTER_CABINET_RULES).into_iter().map(|mut r| {
        r.remove(11);
        r
    }).collect();
    assert_eq!(old[0].len(), 11, "{:?}", old[0]);
    old[0][8] = "Stocked widths".into();
    tables::cabinet_rules_in(&mut job, &old, None).unwrap();
    // Nothing changed from the program's own for W by this table? It said no heights, and the program's own say 12, 18, 30: that is a change.
    // What matters is the flag: with the heights empty in the job's own W, no height is flagged.
    let w = cabinet::effective(&job.cabinet_rules).into_iter().find(|r| r.code == "W").unwrap();
    let made = w.make("W3342").unwrap();
    assert!(made.not_stocked && !made.not_stocked_height, "33 in is no stocked width; no height is flagged where the rule stocks none: {}", made.said);
    let _ = (level, east, room);
}

/// W3342: made 33 by 12 by 42 and flagged twice; W3030: no flag; T2490: 90 in is not a stocked height of T.
#[test]
fn a_height_the_rule_does_not_stock_is_flagged_as_a_width_is() {
    let (mut job, level, east, room) = kitchen();
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("W3342"), false, assumed()).unwrap();
    let said = &run.made[0];
    assert!(said.contains("W3342 was made by the rule for W: 33 in wide, 12 in deep and 42 in high; Assumed"), "{said}");
    assert!(said.contains("33 in is not a stocked width of W") && said.contains("42 in is not a stocked height of W"), "{said}");
    assert!(said.ends_with("so it is a made-to-order box"), "{said}");
    let it = item_of(&mut job, "W3342");
    assert_eq!((inches(&it.width), inches(&it.depth), inches(&it.height)), (33.0, 12.0, 42.0));
    assert!(it.height.source.contains("42 in is not a stocked height (stocked: 12, 18, 30)"), "{}", it.height.source);
    // W3030 is a box the chest has: made by nothing, so nothing is said; asked of its rule it has no flag.
    let w = cabinet::starter().iter().find(|r| r.code == "W").unwrap();
    let m = w.make("W3030").unwrap();
    assert!(!m.not_stocked && !m.not_stocked_height, "{}", m.said);
    assert!(!m.said.contains("not a stocked"), "{}", m.said);
    // A width flagged alone says what it said before, word for word.
    let m = w.make("W3312").unwrap();
    assert!(m.not_stocked && !m.not_stocked_height);
    assert!(m.said.ends_with("; 33 in is not a stocked width of W, so it is a made-to-order box"), "{}", m.said);
    // A height flagged alone.
    let m = w.make("W3042").unwrap();
    assert!(!m.not_stocked && m.not_stocked_height);
    assert!(m.said.ends_with("; 42 in is not a stocked height of W, so it is a made-to-order box"), "{}", m.said);
    // A box whose rule gives it a height of its own is never flagged for a height.
    let b = cabinet::starter().iter().find(|r| r.code == "B").unwrap().make("B36").unwrap();
    assert!(!b.not_stocked_height);
}

/// The new column is read and written like the old ones, whole: out, in to an empty job, out again.
#[test]
fn stocked_heights_are_written_out_and_brought_back_whole() {
    let (mut job, ..) = kitchen();
    let mut rows = tables::cabinet_rules_out(&mut job);
    for r in rows.iter_mut().skip(1) {
        if r[0] == "W" {
            r[11] = "12, 18, 30, 36 in".into();
        }
    }
    tables::cabinet_rules_in(&mut job, &rows, None).unwrap();
    assert_eq!(job.cabinet_rules.len(), 1);
    assert_eq!(job.cabinet_rules[0].stocked_heights, vec![12, 18, 30, 36]);
    let out = tables::cabinet_rules_out(&mut job);
    assert_eq!(out, rows);
    let (mut other, ..) = kitchen();
    tables::cabinet_rules_in(&mut other, &out, None).unwrap();
    assert_eq!(tables::cabinet_rules_out(&mut other), out);
    // The job file round-trips it.
    let text = file::save(&mut job);
    let (mut back, _) = file::load(&text).unwrap();
    assert_eq!(file::save(&mut back), text);
    // A height of 36 is now stocked; 42 still is not.
    assert!(!cabinet::effective(&job.cabinet_rules).into_iter().find(|r| r.code == "W").unwrap().make("W3036").unwrap().not_stocked_height);
}
