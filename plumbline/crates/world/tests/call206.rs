//! Call 206: what the command-line judge of 2026-11-25 found. Part 1: points 1 to 3. Every price is made up; the
//! figures are worked by hand in STATUS.md ("The command line judged (2026-11-25)").
//!
//! Point 1: a line taken off a thing by a change is named with everything a person entered on it.
//!   Refrigerator set down once, priced as "Bought and set" at $1,000.00; 7 typed as its quantity: $7,000.00.

use plumbline_world::editing as ed;
use plumbline_world::entered::{entered_on, FIELDS};
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::view::estimate;
use plumbline_world::*;

const FRIDGE: &str = "Refrigerator";
const BOUGHT: &str = "Bought and set";

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up" }
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

fn fridge() -> (Job, String) {
    let mut job = Job::new("A refrigerator (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    ed::item_priced_as(&mut job, FRIDGE, Some(BOUGHT)).unwrap();
    let list = vec![vec!["Material".to_string(), "Unit".into(), "Price".into(), "How sure".into(), "Where from".into()], vec![FRIDGE.into(), "EA".into(), "1000".into(), "Assumed".into(), "a made-up list".into()]];
    tables::prices_in(&mut job, &list, None, None, true, None).unwrap();
    let thing = ed::place_item(&mut job, &level, FRIDGE, [10.0, 5.0], "0 ft", known()).unwrap();
    job.derive();
    (job, thing)
}

fn amount(job: &mut Job, thing: &str, line: &str) -> Option<f64> {
    estimate(job).lines.into_iter().find(|l| l.path == thing && l.description == line).and_then(|l| l.amount.value)
}

#[test]
fn point_1_a_quantity_typed_on_an_items_line_is_named_when_the_item_is_priced_as_another() {
    let (mut job, thing) = fridge();
    assert_eq!(amount(&mut job, &thing, FRIDGE), Some(1000.0));
    job.set_on_line(&thing, "quantity", Some(FRIDGE), Some("7"), Status::Assumed, "mine").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, &thing, FRIDGE), Some(7000.0));
    let said = ed::item_priced_as(&mut job, FRIDGE, Some("Mechanical equipment")).unwrap().said().join("\n");
    assert!(said.contains("\"Refrigerator\" came off") && said.contains("a quantity of 7.00 EA"), "{said}");
    assert!(said.contains("where the assembly writes \"1\""), "{said}");
    assert!(said.contains("(it came to $7,000.00)"), "{said}");
    // Back again: 1 EA and $1,000.00, as the judge saw, and the reply for this step says what the lines it took had on them.
    ed::item_priced_as(&mut job, FRIDGE, Some(BOUGHT)).unwrap();
    job.derive();
    assert_eq!(amount(&mut job, &thing, FRIDGE), Some(1000.0));
}

#[test]
fn point_1_and_a_line_priced_as_nothing_and_a_line_nobody_changed_say_what_they_carried() {
    let (mut job, thing) = fridge();
    job.set_on_line(&thing, "waste", Some(FRIDGE), Some("12"), Status::Assumed, "mine").unwrap();
    let said = ed::item_priced_as(&mut job, FRIDGE, None).unwrap().said().join("\n");
    assert!(said.contains("12 % waste entered on it"), "{said}");
    assert!(said.contains("(it came to $1,000.00)"), "{said}");
    // The program's own writing is not said to be entered: the "Set in place" line had nothing typed on it.
    assert!(!said.contains("\"Set in place\" came off equipment-") || !said.lines().any(|l| l.contains("\"Set in place\"") && l.contains("entered on it")), "{said}");
}

#[test]
fn point_1_an_assembly_changed_names_what_was_typed_on_each_line_that_comes_off() {
    let mut job = Job::new("A wall (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "ground", "0 ft", "10 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let wall = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0], [20.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("10 ft"), known()).unwrap().remove(0);
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    job.apply(&wall, "Exterior wall, 2x6, lap siding", "made-up").unwrap();
    job.set_on_line(&wall, "quantity", Some("Corner boards"), Some("40 LF"), Status::Assumed, "mine").unwrap();
    job.set_on_line(&wall, "waste", Some("Corner boards"), Some("12"), Status::Assumed, "mine").unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Corner boards"), Some("$6.25"), Status::Assumed, "mine").unwrap();
    job.derive();
    let changed = job.change_assembly_said(&wall, "Take down a partition", Status::Known, "x").unwrap();
    let boards = changed.took.iter().find(|t| t.line == "Corner boards").expect("the line came off and is named");
    let said = boards.entered.join(" | ");
    assert!(said.contains("a quantity of 40.00 LF"), "{said}");
    assert!(said.contains("12 % waste") && said.contains("$6.25/LF"), "{said}");
    // A line nobody touched has nothing entered on it.
    let studs = changed.took.iter().find(|t| t.line == "Studs, 2x6 at 16 in").unwrap();
    assert!(studs.entered.is_empty(), "{:?}", studs.entered);
}

/// A cost line with every optional field set, to read the file's own names for its fields.
fn full() -> CostLine {
    let mut l = CostLine::of("L", "1", Val::known(5.0, "$/EA", "p"));
    l.stretch_out = Some(Val::known(Len::One(1.0), "ft", "p"));
    l.inputs.insert("a".into(), Val::known(Len::One(1.0), "ft", "p"));
    l.waste = Some(Val::known(10.0, "%", "p"));
    l.order = Some(Order { unit: "box".into(), factor: Val::known(2.0, "EA/box", "p"), round_up: true });
    l.class = Some(Class::Labor);
    l.division = Some(Division::Concrete);
    l.off = true;
    l.of_material = true;
    l
}

#[test]
fn point_1_every_field_of_a_cost_line_is_covered_and_a_field_added_later_fails_this() {
    // The fields as the saved file names them: each is in the list `entered` keeps, and nothing in the list is missing from the line.
    let json = serde_json::to_value(full()).unwrap();
    let mut file: Vec<String> = json.as_object().unwrap().keys().cloned().collect();
    file.sort();
    let mut listed: Vec<String> = FIELDS.iter().map(|f| f.0.to_string()).collect();
    listed.sort();
    assert_eq!(file, listed, "a field of a cost line that `entered::FIELDS` does not say how it is named (or one it names that the line no longer has)");
    // Each field, changed alone from the assembly's line, is named.
    let base = CostLine::of("L", "1", Val::known(5.0, "$/EA", "p"));
    for (name, _) in FIELDS {
        let mut l = base.clone();
        let mut theirs = base.clone();
        match *name {
            "description" => continue,
            "quantity" => l.quantity = "2".into(),
            "stretch_out" => l.stretch_out = Some(Val::known(Len::One(1.0), "ft", "p")),
            "inputs" => {
                l.inputs.insert("a".into(), Val::known(Len::One(1.0), "ft", "p"));
            }
            "waste" => l.waste = Some(Val::known(10.0, "%", "p")),
            "order" => l.order = Some(Order { unit: "box".into(), factor: Val::known(2.0, "EA/box", "p"), round_up: true }),
            "unit_cost" => l.unit_cost = Val::known(9.0, "$/EA", "p"),
            "class" => l.class = Some(Class::Labor),
            "division" => l.division = Some(Division::Concrete),
            "off" => l.off = true,
            "of_material" => {
                l.of_material = true;
                theirs.of_material = false;
            }
            other => panic!("a field of a cost line, {other:?}, has no change in this test: add it here and to `entered_on`"),
        }
        let said = entered_on(&l, Some(&theirs), Some("2.00 EA"));
        assert!(!said.is_empty(), "{name} entered on a line is not named");
        // And the line as its assembly has it names nothing.
        assert!(entered_on(&theirs, Some(&theirs), None).is_empty());
    }
    // A price marked unknown on a line that had one, and a waste taken away, are named too.
    let mut l = base.clone();
    l.unit_cost = Val { value: None, unit: "$/EA".into(), status: Status::Unknown, source: "mine".into() };
    assert_eq!(entered_on(&l, Some(&base), None), vec!["a price marked unknown".to_string()]);
}
