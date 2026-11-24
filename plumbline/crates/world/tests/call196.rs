//! Call 196: an item carries its cost ("Priced as"). The starter assemblies and the starter tool
//! chest, a level, "Water heater, 50 gal" set down twice. Every price is made up. The figures are
//! worked by hand in STATUS.md ("The bench, step B"):
//!
//! 1. Set down with nothing more done: no line, and two pieces of equipment said to be set down and
//!    not priced.
//! 2. "Priced as" set to "Bought and set": 2 follow; each has "Water heater, 50 gal" (Material) and
//!    "Set in place" (Labor); the job cost has no figure and names the four lines.
//! 3. The price list has the heater at $1,850.00 and a second supplier's $1,790.00: each line is
//!    $1,850.00; side by side the two heaters come to $3,700.00 and $3,580.00.
//! 4. "Set in place" at 4 hours each, $85.00/hr: $340.00 each; job cost 2 x (1,850 + 340) = $4,380.00.
//! 5. Changed to "Mechanical equipment" (one subcontract line): 2 follow, the job cost is refused
//!    naming two lines; changed back, each material line is priced again from the price list.
//! 6. The chest written out has the heater's row at "Bought and set" and $1,850.00; brought into a
//!    second job it is the same, one price set, and written out again it is the same to the byte.
//! 7. A job saved before opens as it was, every item priced as nothing.

use plumbline_world::editing as ed;
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::view::{estimate, pricing};
use plumbline_world::*;

const HEATER: &str = "Water heater, 50 gal";
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

fn text_of(rows: &[Vec<String>]) -> String {
    let cell = |c: &String| if c.contains(',') || c.contains('"') { format!("\"{}\"", c.replace('"', "\"\"")) } else { c.clone() };
    rows.iter().map(|r| r.iter().map(cell).collect::<Vec<_>>().join(",")).collect::<Vec<_>>().join("\n")
}

fn priced_as(job: &mut Job, name: &str) -> Option<String> {
    let path = item_path(job, name);
    let e = job.find(&path).unwrap();
    job.world.get::<Item>(e).unwrap().priced_as.clone()
}

/// An empty job with the starter assemblies and the starter tool chest, one level.
fn empty() -> (Job, String) {
    let mut job = Job::new("A heater (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    (job, level)
}

/// And two heaters set down, nothing more done.
fn two() -> (Job, String, [String; 2]) {
    let (mut job, level) = empty();
    let a = ed::place_item(&mut job, &level, HEATER, [10.0, 5.0], "0 ft", known()).unwrap();
    let b = ed::place_item(&mut job, &level, HEATER, [14.0, 5.0], "0 ft", known()).unwrap();
    job.derive();
    (job, level, [a, b])
}

fn price_list(job: &mut Job, supplier: Option<&str>, price: &str) -> tables::Brought {
    let list = vec![
        vec!["Material".to_string(), "Unit".into(), "Price".into(), "How sure".into(), "Where from".into()],
        vec![ed::item_material_name(HEATER), "EA".into(), price.into(), "Assumed".into(), "a made-up list".into()],
    ];
    tables::prices_in(job, &list, None, supplier, true, None).unwrap()
}

fn item_path(job: &mut Job, name: &str) -> String {
    let mut q = job.world.query::<(&Node, &Item)>();
    q.iter(&job.world).find(|(n, _)| n.name == name).map(|(n, _)| n.path.clone()).unwrap()
}

fn amount(job: &mut Job, thing: &str, line: &str) -> Option<f64> {
    estimate(job).lines.into_iter().find(|l| l.path == thing && l.description == line).and_then(|l| l.amount.value)
}

fn lines_of(job: &mut Job, thing: &str) -> Vec<String> {
    let e = job.find(thing).unwrap();
    job.world.get::<Cost>(e).map(|c| c.lines.iter().map(|l| l.description.clone()).collect()).unwrap_or_default()
}

#[test]
fn step_1_a_thing_set_down_from_an_item_with_no_priced_as_has_no_line_and_is_said_not_priced() {
    let (mut job, _, [a, b]) = two();
    assert!(lines_of(&mut job, &a).is_empty() && lines_of(&mut job, &b).is_empty());
    let p = pricing(&mut job);
    assert_eq!(p.not_priced, vec![(Kind::Equipment, 2)], "two pieces of equipment are set down and not priced");
    let est = estimate(&mut job);
    assert!(est.job_cost.value.is_none(), "nothing is priced at nothing: no figure, not $0.00");
}

#[test]
fn step_2_priced_as_is_followed_by_the_things_already_set_down() {
    let (mut job, _, [a, b]) = two();
    let followed = ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    assert_eq!(followed.followed.len(), 2, "{followed:?}");
    assert!(followed.said().iter().any(|s| s.contains("2 set down from it follow")), "{:?}", followed.said());
    for t in [&a, &b] {
        assert_eq!(lines_of(&mut job, t), vec![HEATER.to_string(), "Set in place".to_string()]);
    }
    let est = estimate(&mut job);
    let item_line = est.lines.iter().find(|l| l.path == a && l.description == HEATER).unwrap();
    assert_eq!(item_line.class, Some(Class::Material));
    assert_eq!(item_line.quantity.value, Some(1.0));
    assert_eq!(item_line.quantity.unit, "EA");
    assert!(item_line.amount.value.is_none(), "no price entered: no figure");
    // The job cost is refused, and names the four lines.
    assert!(est.job_cost.value.is_none());
    assert_eq!(est.job_cost.left_out.len(), 4, "{:?}", est.job_cost.left_out);
    // They are priced now, so nothing is said to be set down and not priced.
    assert!(pricing(&mut job).not_priced.is_empty());
}

#[test]
fn steps_3_and_4_the_price_list_and_the_hours() {
    let (mut job, _, [a, b]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let first = price_list(&mut job, None, "1850");
    assert!(first.done.iter().any(|d| d.contains("$1,850.00/EA")) && first.notes.is_empty(), "the entry is added and priced, and is not said to price nothing: {first:?}");
    let second = price_list(&mut job, Some("Second Supply"), "1790");
    assert!(second.done.iter().any(|d| d.contains("$1,790.00/EA")), "{second:?}");
    job.derive();
    for t in [&a, &b] {
        assert_eq!(amount(&mut job, t, HEATER), Some(1850.0));
    }
    let est = estimate(&mut job);
    assert_eq!(est.at_generic.value, Some(3700.0));
    assert_eq!(est.at_supplier.iter().map(|(s, t)| (s.as_str(), t.value)).collect::<Vec<_>>(), vec![("Second Supply", Some(3580.0))]);
    // 4. Hours: 4 hr each at $85.00/hr is $340.00 each.
    for t in [&a, &b] {
        job.set_on_line(t, "order", Some("Set in place"), Some("4 hr/EA"), Status::Assumed, "made-up crew").unwrap();
        job.set_on_line(t, "unit_cost", Some("Set in place"), Some("$85.00/hr"), Status::Assumed, "made-up crew").unwrap();
    }
    job.derive();
    for t in [&a, &b] {
        assert_eq!(amount(&mut job, t, "Set in place"), Some(340.0));
    }
    assert_eq!(estimate(&mut job).job_cost.value, Some(4380.0), "2 x (1,850.00 + 340.00)");
}

#[test]
fn step_5_changed_to_another_assembly_and_back() {
    let (mut job, _, [a, b]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    price_list(&mut job, None, "1850");
    for t in [&a, &b] {
        job.set_on_line(t, "order", Some("Set in place"), Some("4 hr/EA"), Status::Assumed, "made-up crew").unwrap();
        job.set_on_line(t, "unit_cost", Some("Set in place"), Some("$85.00/hr"), Status::Assumed, "made-up crew").unwrap();
    }
    job.derive();
    assert_eq!(estimate(&mut job).job_cost.value, Some(4380.0));
    let to = ed::item_priced_as(&mut job, HEATER, Some("Mechanical equipment")).unwrap();
    assert_eq!(to.followed.len(), 2);
    // The hours entered on the lines that went are named when they went.
    let said = to.said().join(" | ");
    assert!(said.contains("Set in place") && said.contains("4 hr/EA") && said.contains("$85.00/hr"), "{said}");
    for t in [&a, &b] {
        assert_eq!(lines_of(&mut job, t), vec!["Equipment, set and connected".to_string()]);
    }
    let est = estimate(&mut job);
    assert!(est.job_cost.value.is_none());
    assert_eq!(est.job_cost.left_out.len(), 2, "{:?}", est.job_cost.left_out);
    // Changed back: the price list still has the heater, so each material line is priced again; the hours are gone.
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    job.derive();
    for t in [&a, &b] {
        assert_eq!(lines_of(&mut job, t), vec!["Equipment, set and connected".to_string().replace("Equipment, set and connected", HEATER), "Set in place".to_string()]);
        assert_eq!(amount(&mut job, t, HEATER), Some(1850.0));
        assert_eq!(amount(&mut job, t, "Set in place"), None);
    }
}

#[test]
fn clearing_priced_as_takes_its_lines_off_and_says_so_and_a_thing_with_no_line_is_not_priced_at_nothing() {
    let (mut job, _, [a, _]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let cleared = ed::item_priced_as(&mut job, HEATER, None).unwrap();
    assert_eq!(cleared.followed.len(), 2);
    assert!(lines_of(&mut job, &a).is_empty());
    assert_eq!(pricing(&mut job).not_priced, vec![(Kind::Equipment, 2)]);
}

#[test]
fn a_thing_set_down_after_takes_the_assembly_and_a_job_with_none_to_name_says_so() {
    let (mut job, level) = empty();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let (c, notes) = ed::place_item_said(&mut job, &level, HEATER, [20.0, 5.0], "0 ft", known()).unwrap();
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(lines_of(&mut job, &c), vec![HEATER.to_string(), "Set in place".to_string()]);
}

#[test]
fn priced_as_has_to_name_an_assembly_the_job_has_that_goes_on_the_items_kind() {
    let (mut job, _, _) = two();
    let no = ed::item_priced_as(&mut job, HEATER, Some("No such assembly")).unwrap_err();
    assert!(no.contains("No such assembly"), "{no}");
    // A wall's assembly does not go on equipment.
    let wall = ed::item_priced_as(&mut job, HEATER, Some("Exterior wall, 2x6, lap siding")).unwrap_err();
    assert!(wall.contains("goes on"), "{wall}");
    // The item is as it was.
    assert_eq!(priced_as(&mut job, HEATER), None);
}

#[test]
fn an_assembly_an_item_is_priced_as_cannot_be_removed() {
    let (mut job, _, _) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let path = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).find(|(n, k)| **k == Kind::Condition && n.name == BOUGHT).map(|(n, _)| n.path.clone()).unwrap()
    };
    let refused = ed::remove(&mut job, &path).unwrap_err();
    assert!(refused.contains(HEATER), "{refused}");
}

#[test]
fn a_line_a_person_put_on_a_thing_stays_and_an_assembly_a_person_chose_is_kept_and_both_are_named() {
    let (mut job, _, [a, b]) = two();
    // `a` is equipment: the estimator picks the equipment assembly by hand on it.
    job.apply(&a, "Mechanical equipment", "made-up: the estimator's own choice").unwrap();
    job.add_line(&b, "Crane day", "1", Some(Class::General), false).unwrap();
    let followed = ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let said = followed.said().join(" | ");
    assert_eq!(followed.followed, vec![b.clone()], "{said}");
    assert!(said.contains(&a) && said.contains("assembly of its own"), "{said}");
    assert!(said.contains("Crane day"), "a line entered on the thing itself is named: {said}");
    assert_eq!(lines_of(&mut job, &b), vec!["Crane day".to_string(), HEATER.to_string(), "Set in place".to_string()]);
}

#[test]
fn a_box_set_in_a_run_is_priced_by_its_own_line_and_never_twice() {
    // A kitchen's wall: B18 set in a run carries one line named for its item.
    let mut job = Job::new("A run (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [12.0, 0.0], [12.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let room = ed::room(&mut job, &level, "Kitchen", [6.0, 8.0], known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    ed::item_priced_as(&mut job, "B18", Some(BOUGHT)).unwrap();
    let run = ed::cabinets(&mut job, &level, &walls[0], &room, "1 ft", &ed::box_list("B18").unwrap(), false, known()).unwrap();
    let thing = run.boxes[0].0.clone();
    assert_eq!(lines_of(&mut job, &thing), vec!["B18".to_string()], "the run's own line only: the box is not priced twice");
    // Priced as nothing and then as the assembly again: the box is named, not touched.
    ed::item_priced_as(&mut job, "B18", None).unwrap();
    let followed = ed::item_priced_as(&mut job, "B18", Some(BOUGHT)).unwrap();
    assert!(followed.followed.is_empty(), "{followed:?}");
    assert!(followed.said().join(" | ").contains("set in a run"), "{:?}", followed.said());
    assert_eq!(lines_of(&mut job, &thing), vec!["B18".to_string()]);
}

#[test]
fn a_line_made_of_the_material_is_left_as_it_was_on_a_wall_and_on_a_thing_with_no_item() {
    let mut job = Job::new("A wall (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let wall = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0);
    job.add_line(&wall, "Siding", "net_area", Some(Class::Material), true).unwrap();
    let loose = ed::place(&mut job, &level, Kind::Fixture, "Basin", [4.0, 4.0], "0 ft", None, None, known()).unwrap();
    job.add_line(&loose, "Basin", "1", Some(Class::Material), true).unwrap();
    job.derive();
    let est = estimate(&mut job);
    for (who, line) in [(&wall, "Siding"), (&loose, "Basin")] {
        let l = est.lines.iter().find(|l| &l.path == who && l.description == line).unwrap();
        assert!(l.amount.value.is_none());
        assert!(est.job_cost.left_out.iter().any(|(w, why)| w.contains(who.as_str()) && why.contains("does not say what it is made of")), "{:?}", est.job_cost.left_out);
    }
}

#[test]
fn a_thing_set_down_from_an_item_made_of_its_item_with_no_price_in_the_list_says_where_to_put_it() {
    let (mut job, _, [a, _]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let est = estimate(&mut job);
    let why = est.job_cost.left_out.iter().find(|(w, _)| w.contains(&a) && w.contains(HEATER)).map(|(_, why)| why.clone()).unwrap();
    assert!(why.contains(&ed::item_material_name(HEATER)) && why.contains("price list"), "{why}");
}

#[test]
fn renaming_an_item_takes_its_price_list_entry_with_it() {
    let (mut job, _, [a, _]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    price_list(&mut job, None, "1850");
    job.derive();
    assert_eq!(amount(&mut job, &a, HEATER), Some(1850.0));
    let path = item_path(&mut job, HEATER);
    ed::rename_item(&mut job, &path, "Water heater, 40 gal").unwrap();
    job.derive();
    assert_eq!(amount(&mut job, &a, "Water heater, 40 gal"), Some(1850.0), "{:?}", lines_of(&mut job, &a));
}

// ---------------------------------------------------------------- the chest's table

#[test]
fn step_6_the_chest_written_out_is_priced_and_comes_back_to_the_byte() {
    let (mut job, _, _) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    price_list(&mut job, None, "1850");
    let out = tables::items_out(&mut job);
    let heads = &out[0];
    assert_eq!(heads.len(), 26);
    assert_eq!(&heads[22..], ["Priced as", "Unit cost", "Unit cost: how sure", "Unit cost: where from"]);
    let row = out.iter().find(|r| r[0] == HEATER).unwrap();
    assert_eq!(&row[22..], [BOUGHT, "$1,850.00/EA", "Assumed", "a made-up list"]);
    // A row of an item priced as nothing: blank, not said.
    let other = out.iter().find(|r| r[0] == "Refrigerator").unwrap();
    assert_eq!(&other[22..], ["", "", "Unknown", "not said"]);
    let first = text_of(&out);
    // A second empty job that has the starter assemblies.
    let mut second = Job::new("A second (not a real job)", "test", "2026-11-24");
    tables::assemblies_in(&mut second, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let brought = tables::items_in(&mut second, &rows(&first), None).unwrap();
    assert!(brought.not_used.is_empty(), "{:?}", brought.not_used);
    assert!(brought.notes.iter().chain(brought.done.iter()).any(|s| s.contains("1 price was set")), "{:?} {:?}", brought.notes, brought.done);
    assert_eq!(priced_as(&mut second, HEATER).as_deref(), Some(BOUGHT));
    assert_eq!(text_of(&tables::items_out(&mut second)), first, "the table written out again from the second job is the first, to the byte");
    // And the price list has it.
    let prices = tables::prices_out(&mut second);
    assert!(prices.iter().any(|r| r[0] == ed::item_material_name(HEATER) && r[2] == "1850"), "{prices:?}");
}

#[test]
fn a_chest_with_none_of_the_new_columns_is_read_as_it_was() {
    let (mut job, _, [a, _]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    price_list(&mut job, None, "1850");
    // The chest as written before: the first 22 columns.
    let old: Vec<Vec<String>> = tables::items_out(&mut job).into_iter().map(|r| r[..22].to_vec()).collect();
    let before = text_of(&tables::prices_out(&mut job));
    let said = tables::items_in(&mut job, &old, None).unwrap();
    assert!(said.not_used.is_empty(), "{:?}", said.not_used);
    assert_eq!(said.same, 48, "every item is as the job has it: {:?}", said.done);
    assert_eq!(text_of(&tables::prices_out(&mut job)), before, "the price list is not touched");
    assert_eq!(priced_as(&mut job, HEATER).as_deref(), Some(BOUGHT), "an item keeps what it is priced as");
    assert_eq!(lines_of(&mut job, &a).len(), 2);
}

#[test]
fn a_table_with_the_column_and_a_blank_cell_clears_priced_as_and_one_that_names_no_assembly_is_not_used() {
    let (mut job, _, [a, _]) = two();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let mut out = tables::items_out(&mut job);
    let row = out.iter_mut().find(|r| r[0] == HEATER).unwrap();
    row[22] = String::new();
    let said = tables::items_in(&mut job, &out, None).unwrap();
    assert!(said.not_used.is_empty(), "{:?}", said.not_used);
    assert!(said.done.iter().any(|d| d.contains(HEATER) && d.contains("priced as")), "{:?}", said.done);
    assert!(lines_of(&mut job, &a).is_empty());
    let mut out = tables::items_out(&mut job);
    out.iter_mut().find(|r| r[0] == HEATER).unwrap()[22] = "No such assembly".into();
    let said = tables::items_in(&mut job, &out, None).unwrap();
    assert_eq!(said.not_used.len(), 1, "{:?}", said.not_used);
    assert!(said.not_used[0].1.contains("No such assembly"), "{:?}", said.not_used);
}

// ---------------------------------------------------------------- the saved job

#[test]
fn step_7_the_saved_job_has_one_entry_on_an_item_and_an_old_job_opens_as_it_was() {
    let (mut job, _, _) = two();
    let before = file::save(&mut job);
    assert!(!before.contains("priced_as"), "an item priced as nothing writes nothing");
    let est_before = estimate(&mut job).job_cost.left_out.len();
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    let after = file::save(&mut job);
    assert_eq!(after.matches("\"priced_as\": \"Bought and set\"").count(), 1, "one entry, on the one item");
    // A job saved before opens, and every item is priced as nothing.
    let (mut old, _) = file::load(&before).unwrap();
    for e in old.order.clone() {
        if let Some(it) = old.world.get::<Item>(e) {
            assert_eq!(it.priced_as, None);
        }
    }
    assert_eq!(estimate(&mut old).job_cost.left_out.len(), est_before);
    // The new one opens as it was saved.
    let (mut again, _) = file::load(&after).unwrap();
    assert_eq!(file::save(&mut again), after);
}

#[test]
fn a_price_list_row_that_names_an_item_priced_as_nothing_sets_its_price_and_says_it_prices_nothing_yet() {
    let (mut job, _, _) = two();
    let said = price_list(&mut job, None, "1850");
    assert!(said.done.iter().any(|d| d.contains("$1,850.00/EA")), "{said:?}");
    assert!(said.notes.iter().any(|n| n.contains("prices nothing on the job yet") && n.contains(HEATER)), "{:?}", said.notes);
    // No material was made for it: the item is its own entry, and the price list has one row for it.
    let mut q = job.world.query::<(&Node, &Kind)>();
    assert!(!q.iter(&job.world).any(|(_, k)| *k == Kind::Material));
    let prices = tables::prices_out(&mut job);
    assert_eq!(prices.iter().filter(|r| r[0] == HEATER).count(), 1, "{prices:?}");
    // Priced as something afterwards, it is priced from that entry with no further word.
    let a = {
        let mut q = job.world.query::<(&Node, &Placed)>();
        q.iter(&job.world).map(|(n, _)| n.path.clone()).min().unwrap()
    };
    ed::item_priced_as(&mut job, HEATER, Some(BOUGHT)).unwrap();
    job.derive();
    assert_eq!(amount(&mut job, &a, HEATER), Some(1850.0));
}
