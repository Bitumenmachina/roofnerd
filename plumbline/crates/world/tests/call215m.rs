//! Call 215: a cabinet by rule. The starter rules make the starter boxes; a code the chest has
//! not is made by its rule when a run asks for it; a width not stocked is made and said so; a
//! code no rule reads is refused in a cabinet maker's words; where a box stands comes from its
//! rule; the rules are a table the person owns, brought in and written out, and the job carries
//! them only once they are changed. Made-up houses, no real job.

use plumbline_world::building::cabinet;
use plumbline_world::database::{self, Given};
use plumbline_world::editing as ed;
use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up plan" }
}

fn rows_of(text: &str) -> Vec<Vec<String>> {
    plumbline_world::building::shape::csv_rows(text)
}

/// A 24 ft by 16 ft house, 9 ft walls, a Kitchen, and the starter items: the east wall's face is
/// clear from 0.25 ft to 15.75 ft along it.
fn kitchen() -> (Job, String, String, String) {
    let mut job = Job::new("A kitchen (not a real job)", "test", "2026-11-26");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let room = ed::room(&mut job, &level, "Kitchen", [12.0, 8.0], known()).unwrap();
    tables::items_in(&mut job, &rows_of(tables::STARTER_ITEMS), None).unwrap();
    (job, level, outer[1].clone(), room)
}

fn codes(text: &str) -> Vec<String> {
    text.split(' ').map(str::to_string).collect()
}

fn item_of(job: &mut Job, name: &str) -> Item {
    let (path, _) = ed::box_codes(job).into_iter().find(|(_, n)| n == name).unwrap_or_else(|| panic!("{name} is not in the chest"));
    let e = job.find(&path).unwrap();
    job.world.get::<Item>(e).unwrap().clone()
}

fn inches(v: &value::Val<units::Len>) -> f64 {
    v.value.unwrap().feet(&v.unit).unwrap() * 12.0
}

/// Step 1. The starter chest has twenty-seven cabinet codes (the contract said thirty-five: the
/// other twenty-one of its forty-eight rows are items like a range or a vanity, with no code).
/// Each is, size for size and shape for shape, what its rule makes from its code.
#[test]
fn every_starter_box_is_what_its_rule_makes() {
    let (mut job, ..) = kitchen();
    let chest = ed::box_codes(&mut job);
    assert_eq!(chest.len(), 27);
    let rules = cabinet::effective(&[]);
    assert_eq!(rules.len(), 8);
    for (_, name) in &chest {
        let held = item_of(&mut job, name);
        let (letters, _) = cabinet::read_code(name).unwrap();
        let rule = cabinet::rule_for(&rules, &letters).unwrap_or_else(|| panic!("no rule for {name}"));
        let made = rule.make(name).unwrap();
        assert_eq!(made.name, *name);
        assert!(!made.not_stocked, "{name} is a stocked width of its rule");
        for (what, a, b) in [("width", &held.width, &made.item.width), ("depth", &held.depth, &made.item.depth), ("height", &held.height, &made.item.height)] {
            assert_eq!(inches(a), inches(b), "{name}: {what}");
        }
        assert_eq!(held.form, made.item.form, "{name}: drawn as");
        assert_eq!((held.category, held.division), (made.item.category, made.item.division), "{name}");
        assert!(made.item.width.status == Status::Assumed && made.item.depth.status == Status::Assumed && made.item.height.status == Status::Assumed, "{name}");
    }
    // A few by hand: B12 to B36 are 24 deep and 34 1/2 high; W1230 is 12 wide, 12 deep, 30 high;
    // W3018 is 30 wide and 18 high; T1884 is 18 wide, 24 deep, 84 high; LS36 is 36 by 36; F3 is 3 wide.
    let sizes = |name: &str, job: &mut Job| {
        let it = item_of(job, name);
        (inches(&it.width), inches(&it.depth), inches(&it.height))
    };
    assert_eq!(sizes("B30", &mut job), (30.0, 24.0, 34.5));
    assert_eq!(sizes("W1230", &mut job), (12.0, 12.0, 30.0));
    assert_eq!(sizes("W3018", &mut job), (30.0, 12.0, 18.0));
    assert_eq!(sizes("T1884", &mut job), (18.0, 24.0, 84.0));
    assert_eq!(sizes("LS36", &mut job), (36.0, 36.0, 34.5));
    assert_eq!(sizes("F3", &mut job), (3.0, 24.0, 34.5));
}

/// Step 2. B15, B30 and B42 in a run: B42 is not in the chest and is made by the rule for B.
#[test]
fn a_code_the_chest_lacks_is_made_by_its_rule() {
    let (mut job, level, east, room) = kitchen();
    assert_eq!(ed::box_codes(&mut job).len(), 27);
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B15 B30 B42"), false, assumed()).unwrap();
    // 15 + 30 + 42 = 87 in = 7.25 ft.
    assert_eq!(run.boxes.len(), 3);
    assert!((run.run - 7.25).abs() < 1e-9, "{}", run.run);
    assert_eq!(ed::box_codes(&mut job).len(), 28);
    // What was said: the rule, the sizes, and that 42 in is not stocked.
    assert_eq!(run.made.len(), 1, "{:?}", run.made);
    let said = &run.made[0];
    assert!(said.contains("B42 was made by the rule for B: 42 in wide, 24 in deep and 34 1/2 in high") && said.contains("Assumed"), "{said}");
    assert!(said.contains("42 in is not a stocked width"), "{said}");
    // The item is an item like any other: Assumed, saying the rule and how the number was read.
    let it = item_of(&mut job, "B42");
    assert_eq!((inches(&it.width), inches(&it.depth), inches(&it.height)), (42.0, 24.0, 34.5));
    assert_eq!(it.form.as_deref(), Some("base cabinet"));
    for v in [&it.width, &it.depth, &it.height] {
        assert_eq!(v.status, Status::Assumed);
        assert!(v.source.contains("the cabinet rule for B"), "{}", v.source);
    }
    assert!(it.width.source.contains("the number 42 is the width in inches") && it.width.source.contains("not a stocked width"), "{}", it.width.source);
    // Listed, changed, written out, brought back: it is in the items table with the rest.
    let rows = tables::items_out(&mut job);
    assert!(rows.iter().any(|r| r[0] == "B42"));
    // A made code asked for again is the item that is there, and nothing is made.
    let again = ed::cabinets(&mut job, &level, &east, &room, "9 ft", &codes("B42"), false, assumed()).unwrap();
    assert!(again.made.is_empty());
    assert_eq!(ed::box_codes(&mut job).len(), 28);
}

/// Step 3. W3342 and T2490.
#[test]
fn a_wall_box_and_a_tall_box_are_read_from_their_numbers() {
    let (mut job, level, east, room) = kitchen();
    let wall = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("W3342"), false, assumed()).unwrap();
    let it = item_of(&mut job, "W3342");
    assert_eq!((inches(&it.width), inches(&it.depth), inches(&it.height)), (33.0, 12.0, 42.0));
    // Hung where the starter wall boxes hang: 54 in, 4.5 ft.
    assert_eq!(wall.hung, Some(4.5));
    // 33 is not a stocked width of W (12, 15, 18, 24, 30, 36): said, and made.
    assert!(wall.made[0].contains("33 in is not a stocked width"), "{:?}", wall.made);
    let (mut job, level, east, room) = kitchen();
    let tall = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("T2490"), false, assumed()).unwrap();
    let it = item_of(&mut job, "T2490");
    assert_eq!((inches(&it.width), inches(&it.depth), inches(&it.height)), (24.0, 24.0, 90.0));
    assert_eq!(tall.hung, None);
    assert!(!tall.made[0].contains("not a stocked width"), "{:?}", tall.made);
    // A lower-case code is made under its capitals.
    let (mut job, level, east, room) = kitchen();
    ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("b42"), false, assumed()).unwrap();
    assert!(ed::box_codes(&mut job).iter().any(|(_, n)| n == "B42"));
}

/// Step 4. SB42 is made by the rule for SB, not the rule for B.
#[test]
fn the_rule_is_the_one_with_the_codes_letters() {
    let (mut job, level, east, room) = kitchen();
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("SB42"), false, assumed()).unwrap();
    assert!(run.made[0].contains("SB42 was made by the rule for SB:"), "{:?}", run.made);
    assert!(item_of(&mut job, "SB42").width.source.contains("the cabinet rule for SB"));
    // 42 is not among SB's stocked widths (30, 33, 36).
    assert!(run.made[0].contains("42 in is not a stocked width of SB"), "{:?}", run.made);
}

/// Step 5. A code no rule reads is refused, and the refusal says how to write it.
#[test]
fn a_code_no_rule_reads_is_refused_with_how_to_write_it() {
    let (mut job, level, east, room) = kitchen();
    let before = save(&mut job);
    let say = |job: &mut Job, code: &str| ed::cabinets(job, &level, &east, &room, "1 ft", &[code.to_string()], false, assumed()).unwrap_err();
    let w33 = say(&mut job, "W33");
    assert!(w33.contains("W reads four digits, the width then the height: W3342 is 33 in wide and 42 in high"), "{w33}");
    let b = say(&mut job, "B");
    assert!(b.contains("B has no number") && b.contains("B reads the width in inches: B42 is 42 in wide"), "{b}");
    let x = say(&mut job, "X42");
    assert!(x.contains("X42 is not in the tool chest") && x.contains("no cabinet rule reads X") && x.contains("The letters the rules read are B, SB, DB, BLB, LS, W, T, F"), "{x}");
    // The window's way of keeping the list of what there is out of its sentence still works.
    assert!(x.contains(". It has: B12"), "{x}");
    // No software words, no decimals.
    for said in [&w33, &b, &x] {
        for word in ["None", "Err", "unwrap", "f64", "0.0"] {
            assert!(!said.contains(word), "{said}");
        }
    }
    // A name that is not a code at all is refused as it always was.
    let dish = say(&mut job, "Toaster");
    assert!(dish.starts_with("Toaster is not in the tool chest. It has: "), "{dish}");
    assert_eq!(save(&mut job), before);
    // A code that a rule reads is not left half made when the run is refused.
    let big = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B96 B96 B96 B96"), false, assumed()).unwrap_err();
    assert!(big.contains("past the corner"), "{big}");
    assert_eq!(save(&mut job), before);
}

/// Step 6. What was made is in `items` and comes back, row for row, in a new job.
#[test]
fn made_boxes_are_written_out_and_brought_back() {
    for code in ["B42", "W3342", "T2490", "SB42"] {
        let (mut job, level, east, room) = kitchen();
        ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes(code), false, assumed()).unwrap();
        let out = tables::items_out(&mut job);
        assert!(out.iter().any(|r| r[0] == code), "{code}");
        let mut fresh = Job::new("Another (not a real job)", "test", "2026-11-26");
        tables::items_in(&mut fresh, &out, None).unwrap();
        assert_eq!(tables::items_out(&mut fresh), out, "{code}");
        assert!(ed::box_codes(&mut fresh).iter().any(|(_, n)| n == code), "{code}");
    }
}

/// Step 7. The rules are a table, written out and brought back; changed there, the next code is
/// read as changed, and what was made before keeps its sizes.
#[test]
fn the_rules_are_the_persons_table() {
    let (mut job, level, east, room) = kitchen();
    ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B42"), false, assumed()).unwrap();
    let out = tables::cabinet_rules_out(&mut job);
    assert_eq!(out[0], cabinet::HEADINGS.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    assert_eq!(out.len(), 9);
    assert_eq!(out[1][0], "B");
    // Nothing touched: the job carries none of them.
    assert!(job.cabinet_rules.is_empty());
    // The same table brought back is the same, and the job still carries none.
    let before = save(&mut job);
    let said = tables::cabinet_rules_in(&mut job, &out, None).unwrap();
    assert_eq!((said.same, said.done.len(), said.not_used.len()), (8, 0, 0), "{said:?}");
    assert_eq!(save(&mut job), before);
    // The rule for B changed to 21 in deep.
    let mut changed = out.clone();
    assert_eq!(changed[1][4], "24 in");
    changed[1][4] = "21 in".to_string();
    let said = tables::cabinet_rules_in(&mut job, &changed, None).unwrap();
    assert_eq!(said.done, vec!["B: changed: its depth".to_string()], "{said:?}");
    assert_eq!(job.cabinet_rules.len(), 1);
    let after_change = save(&mut job);
    assert!(after_change.contains("\"cabinetRules\""));
    // B45 is 21 deep. B42, made before, is still 24 deep.
    ed::cabinets(&mut job, &level, &east, &room, "6 ft", &codes("B45"), false, assumed()).unwrap();
    assert_eq!(inches(&item_of(&mut job, "B45").depth), 21.0);
    assert_eq!(inches(&item_of(&mut job, "B42").depth), 24.0);
    // The job's rules are saved with it and come back whole.
    let (mut again, _) = load(&save(&mut job)).unwrap();
    assert_eq!(tables::cabinet_rules_out(&mut again)[1][4], "21 in");
    assert_eq!(save(&mut again), save(&mut job));
    // Written out and brought into a new job, they are the same rows.
    let written = tables::cabinet_rules_out(&mut job);
    let mut fresh = Job::new("Another (not a real job)", "test", "2026-11-26");
    tables::cabinet_rules_in(&mut fresh, &written, None).unwrap();
    assert_eq!(tables::cabinet_rules_out(&mut fresh), written);
    // Changed back to the program's own: the job carries nothing again.
    let mut back = written.clone();
    back[1][4] = "24 in".to_string();
    tables::cabinet_rules_in(&mut job, &back, None).unwrap();
    assert!(job.cabinet_rules.is_empty());
}

/// A rule the person adds is read for its letters, and one that cannot be used is named and
/// refused whole, with the table's own reasons.
#[test]
fn a_rule_of_ones_own_and_the_rows_that_cannot_be_used() {
    let (mut job, level, east, room) = kitchen();
    let mut rows = tables::cabinet_rules_out(&mut job);
    // A pantry: tall, 24 deep, number reads the width then the height.
    rows.push(["P", "tall box", "the width then the height", "from the number", "24 in", "from the number", "on the floor", "wardrobe", "18, 24 in", "Assumed", "made-up shop habit"].iter().map(|s| s.to_string()).collect());
    // Not usable: a wall box that stands on the floor; Known; a width given as a length; a code twice.
    rows.push(["Q", "wall box", "the width", "from the number", "12 in", "30 in", "on the floor", "box", "", "Assumed", "x"].iter().map(|s| s.to_string()).collect());
    rows.push(["R", "base", "the width", "from the number", "24 in", "34 1/2 in", "on the floor", "base cabinet", "", "Known", "x"].iter().map(|s| s.to_string()).collect());
    rows.push(["S", "base", "the width", "30 in", "24 in", "34 1/2 in", "on the floor", "base cabinet", "", "Assumed", "x"].iter().map(|s| s.to_string()).collect());
    rows.push(["B", "base", "the width", "from the number", "20 in", "34 1/2 in", "on the floor", "base cabinet", "", "Assumed", "x"].iter().map(|s| s.to_string()).collect());
    let said = tables::cabinet_rules_in(&mut job, &rows, None).unwrap();
    assert_eq!(said.done, vec!["P: added".to_string()], "{said:?}");
    assert_eq!(said.not_used.len(), 4, "{said:?}");
    let why: Vec<&str> = said.not_used.iter().map(|(_, w)| w.as_str()).collect();
    assert!(why.iter().any(|w| w.starts_with("Q: ") && w.contains("a wall box hangs")), "{why:?}");
    assert!(why.iter().any(|w| w.starts_with("R: ") && w.contains("Assumed")), "{why:?}");
    assert!(why.iter().any(|w| w.starts_with("S: ") && w.contains("the number after the letters gives it")), "{why:?}");
    assert!(why.iter().any(|w| w.starts_with("B: ") && w.contains("same code")), "{why:?}");
    // P2490 is made by the new rule.
    ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("P2490"), false, assumed()).unwrap();
    assert_eq!(inches(&item_of(&mut job, "P2490").height), 90.0);
    // Not a table of rules at all: refused whole.
    let err = tables::cabinet_rules_in(&mut job, &[vec!["Item".into(), "Trade".into()]], None).unwrap_err();
    assert!(err.contains("not a table of cabinet rules"), "{err}");
    // A shape that is not there is not used.
    let mut bad = tables::cabinet_rules_out(&mut job);
    bad[1][7] = "no such shape".into();
    let said = tables::cabinet_rules_in(&mut job, &bad, None).unwrap();
    assert!(said.not_used.iter().any(|(_, w)| w.starts_with("B: drawn as: ")), "{said:?}");
}

/// Point 6. Where a box stands is its rule's: a rule that hangs a box of other letters hangs it, and
/// a rule that stands W on the floor stands it. A box whose code no rule reads stands on the floor.
#[test]
fn where_a_box_stands_comes_from_its_rule() {
    // A job's own rule: V hangs 60 in up.
    let (mut job, level, east, room) = kitchen();
    let mut rows = tables::cabinet_rules_out(&mut job);
    rows.push(["V", "wall box", "the width then the height", "from the number", "12 in", "from the number", "hung 60 in up", "box", "", "Assumed", "made-up shop habit"].iter().map(|s| s.to_string()).collect());
    tables::cabinet_rules_in(&mut job, &rows, None).unwrap();
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("V3030"), false, assumed()).unwrap();
    assert_eq!(run.hung, Some(5.0));
    // And with a base box in the same run: wall boxes and base boxes are two runs.
    let err = ed::cabinets(&mut job, &level, &east, &room, "5 ft", &codes("V3030 B18"), false, assumed()).unwrap_err();
    assert!(err.contains("two runs"), "{err}");
    // Two wall boxes that hang at different heights are two runs too.
    let err = ed::cabinets(&mut job, &level, &east, &room, "5 ft", &codes("V3030 W3030"), false, assumed()).unwrap_err();
    assert!(err.contains("one height") || err.contains("two runs"), "{err}");
    // A job's rule that stands W on the floor: W is no longer hung. (A wall box has to hang: it is a base now.)
    let (mut job, level, east, room) = kitchen();
    let mut rows = tables::cabinet_rules_out(&mut job);
    let w = rows.iter().position(|r| r[0] == "W").unwrap();
    rows[w][1] = "base".into();
    rows[w][6] = "on the floor".into();
    tables::cabinet_rules_in(&mut job, &rows, None).unwrap();
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("W1230"), false, assumed()).unwrap();
    assert_eq!(run.hung, None);
    // A box named by a code no rule reads (an item of the chest the person named so) stands on the floor.
    let (mut job, level, east, room) = kitchen();
    ed::item(&mut job, "Z12", Item {
        category: Kind::Furnishing,
        maker: value::Val { value: None, unit: String::new(), status: Status::Unknown, source: "not chosen".into() },
        model: value::Val { value: None, unit: String::new(), status: Status::Unknown, source: "not chosen".into() },
        width: value::Val { value: Some(units::Len::One(12.0)), unit: "in".into(), status: Status::Assumed, source: "made-up".into() },
        depth: value::Val { value: Some(units::Len::One(24.0)), unit: "in".into(), status: Status::Assumed, source: "made-up".into() },
        height: value::Val { value: Some(units::Len::One(34.5)), unit: "in".into(), status: Status::Assumed, source: "made-up".into() },
        weight: None,
        sheet: String::new(),
        division: components::Division::of_number(12),
        form: None,
        priced_as: None,
    })
    .unwrap();
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("Z12"), false, assumed()).unwrap();
    assert_eq!(run.hung, None);
}

/// Point 7. A job that never touched the rules is byte for byte what it was: laying the starter
/// codes gives the job the program gave before, and a job with no rule of its own writes no key.
#[test]
fn a_job_that_never_touched_the_rules_saves_as_it_did() {
    let (mut job, level, east, room) = kitchen();
    ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B15 B30 SB36"), true, assumed()).unwrap();
    assert!(job.cabinet_rules.is_empty());
    let text = save(&mut job);
    assert!(!text.contains("cabinetRules"));
    // Bringing the program's own rules in is no change.
    tables::cabinet_rules_in(&mut job, &rows_of(tables::STARTER_CABINET_RULES), None).unwrap();
    assert_eq!(save(&mut job), text);
}

/// The database workbook has the rules as a tab, after Items, and brings it back whole.
#[test]
fn the_database_has_the_rules_as_a_tab_after_the_items() {
    assert_eq!(database::TABS, ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    let (mut job, level, east, room) = kitchen();
    let mut rows = tables::cabinet_rules_out(&mut job);
    rows[1][4] = "21 in".to_string();
    tables::cabinet_rules_in(&mut job, &rows, None).unwrap();
    ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B42"), false, assumed()).unwrap();
    let out = database::out(&mut job);
    let names: Vec<&str> = out.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    // Into an empty job, tabs given in any order: the same tables come out.
    let mut given: Vec<Given> = out.iter().rev().map(|(n, r)| Given { name: n.to_string(), rows: r.clone(), numbers: Vec::new(), lost: Vec::new() }).collect();
    let mut fresh = Job::new("Another (not a real job)", "test", "2026-11-26");
    let whole = database::bring_in(&mut fresh, &given, false).unwrap();
    assert!(whole.absent.is_empty());
    assert_eq!(whole.tabs.iter().map(|t| t.name).collect::<Vec<_>>(), ["Shapes", "Items", "Cabinet codes", "Connections", "Assemblies", "Prices"]);
    assert_eq!(database::out(&mut fresh), out);
    // One bad row in the rules tab and nothing at all comes in, the tab named.
    let bad = given.iter_mut().find(|g| g.name == "Cabinet codes").unwrap();
    bad.rows[1][1] = "a cupboard".to_string();
    let mut other = Job::new("Another (not a real job)", "test", "2026-11-26");
    let err = database::bring_in(&mut other, &given, false).unwrap_err();
    assert!(err.contains("tab Cabinet codes, row 2") && err.contains("a base, a wall box, a tall box or a filler"), "{err}");
    assert!(other.cabinet_rules.is_empty());
    // The starter database has no tab for them: the program's own rules are always in force, and are kept as the job has them.
    let mut starter = Job::new("Another (not a real job)", "test", "2026-11-26");
    let g: Vec<Given> = database::starter_files().iter().map(|(n, t)| Given { name: n.to_string(), rows: rows_of(t), numbers: Vec::new(), lost: Vec::new() }).collect();
    let whole = database::bring_in(&mut starter, &g, true).unwrap();
    assert!(!whole.tabs.iter().any(|t| t.name == "Cabinet codes") && whole.absent.is_empty());
    assert!(starter.cabinet_rules.is_empty());
}

/// Call 225: no box is made wider than 96 in (a sheet's length). 96 is made; wider is refused, saying what the code read and the limit.
#[test]
fn no_box_is_made_wider_than_96_in() {
    let (mut job, level, east, room) = kitchen();
    let before = save(&mut job);
    for (code, wide) in [("B999", 999), ("B97", 97), ("W9942", 99)] {
        let err = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &[code.to_string()], false, assumed()).unwrap_err();
        assert!(err.contains(&format!("{code} reads {wide} in wide: no box is made wider than 96 in, a sheet's length")), "{err}");
        assert!(err.contains(". It has: B12"), "{err}");
    }
    assert_eq!(save(&mut job), before);
    // 96 itself is made (8 ft along a 14.5 ft clear face), flagged as not stocked.
    let run = ed::cabinets(&mut job, &level, &east, &room, "1 ft", &codes("B96"), false, assumed()).unwrap();
    assert!(run.made[0].contains("B96 was made by the rule for B: 96 in wide") && run.made[0].contains("not a stocked width"), "{:?}", run.made);
}
