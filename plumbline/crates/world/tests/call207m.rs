//! Call 207, points 4 and 5: a price set or cleared by name for a material or an item, and the price list as rows.
//! Every figure and name is made up. Worked by hand:
//!   Two "Water heater, 50 gal" set down, priced as "Bought and set": each has the item's line (1 EA, Material) and
//!   "Set in place" (Labor, no rate, so no figure). At $1,850.00 each, the item lines come to 2 x 1,850.00 = $3,700.00;
//!   at "Northside Supply"'s $1,790.00, 2 x 1,790.00 = $3,580.00.

use plumbline_world::editing as ed;
use plumbline_world::file::save;
use plumbline_world::pricelist::{price_list, set_price, PriceRow};
use plumbline_world::value::Status;
use plumbline_world::view::estimate;
use plumbline_world::*;

const HEATER: &str = "Water heater, 50 gal";

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up plan" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up quote" }
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

/// An empty job, the starter assemblies and chest, the heater set down twice and priced as "Bought and set".
fn heaters() -> Job {
    let mut job = Job::new("Heaters (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    tables::items_in(&mut job, &rows(tables::STARTER_ITEMS), None).unwrap();
    ed::place_item(&mut job, &level, HEATER, [10.0, 5.0], "0 ft", known()).unwrap();
    ed::place_item(&mut job, &level, HEATER, [14.0, 5.0], "0 ft", known()).unwrap();
    ed::item_priced_as(&mut job, HEATER, Some("Bought and set")).unwrap();
    job
}

fn row(job: &mut Job, name: &str) -> PriceRow {
    price_list(job).rows.into_iter().find(|r| r.name == name).unwrap_or_else(|| panic!("no row for {name}"))
}
fn item_amounts(job: &mut Job) -> Vec<Option<f64>> {
    estimate(job).lines.iter().filter(|l| l.description == HEATER).map(|l| l.amount.value).collect()
}

/// Step 1: the heater is at the head, no price, 2 lines waiting on 2 things, and no dollars.
#[test]
fn step_1_an_item_nothing_prices_yet_heads_the_list_with_its_lines_waiting() {
    let mut job = heaters();
    let list = price_list(&mut job);
    let first = &list.rows[0];
    assert_eq!((first.name.as_str(), first.item, first.unit.as_str()), (HEATER, true, "EA"));
    assert_eq!(first.price.value, None);
    assert_eq!((first.lines, first.things, first.waiting), (2, 2, 2));
    assert_eq!(first.price.riding, None);
    assert!(list.suppliers.is_empty());
    assert!(list.not_found.is_empty(), "{:?}", list.not_found);
}

/// Steps 2 and 3: $1,850.00: 2 lines on 2 things, the job cost from none to $3,700.00; a supplier new to the job, its column $3,580.00.
#[test]
fn steps_2_and_3_a_price_then_a_supplier_new_to_the_job() {
    let mut job = heaters();
    let done = set_price(&mut job, HEATER, None, Some("$1,850.00"), assumed()).unwrap();
    assert_eq!((done.lines, done.things), (2, 2));
    assert_eq!((done.cost_before, done.cost_after), (None, Some(3700.0)));
    assert_eq!(done.supplier_added, None);
    assert!(done.said().iter().any(|s| s.contains("2 lines on 2 things")), "{:?}", done.said());
    assert!(done.said().iter().any(|s| s.contains("$3,700.00")), "{:?}", done.said());
    let r = row(&mut job, HEATER);
    assert_eq!((r.price.value, r.price.status, r.lines, r.things, r.waiting), (Some(1850.0), Status::Assumed, 2, 2, 0));
    assert_eq!(r.price.riding, Some(3700.0));
    // The labor lines still wait, and the job cost is only what has a figure.
    assert!(estimate(&mut job).job_cost.left_out.len() >= 2);

    let done = set_price(&mut job, HEATER, Some("Northside Supply"), Some("$1,790.00"), assumed()).unwrap();
    assert_eq!(done.supplier_added.as_deref(), Some("Northside Supply"));
    assert!(done.said().iter().any(|s| s.contains("Northside Supply") && s.contains("new to this job")), "{:?}", done.said());
    let list = price_list(&mut job);
    assert_eq!(list.suppliers, ["Northside Supply"]);
    let r = &list.rows[0];
    assert_eq!(r.price.value, Some(1850.0));
    let at = r.suppliers[0].as_ref().unwrap();
    assert_eq!((at.value, at.riding), (Some(1790.0), Some(3580.0)));
    assert_eq!(estimate(&mut job).at_supplier[0].1.value, Some(3580.0));
}

/// Step 4: cleared is no price, never $0.00; set again.
#[test]
fn step_4_a_price_cleared_is_no_price_and_the_lines_read_no_price() {
    let mut job = heaters();
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    let done = set_price(&mut job, HEATER, None, None, assumed()).unwrap();
    assert!(done.cleared);
    assert_eq!((done.cost_before, done.cost_after), (Some(3700.0), None));
    let r = row(&mut job, HEATER);
    assert_eq!((r.price.value, r.price.at_nothing, r.waiting), (None, false, 2));
    assert_eq!(item_amounts(&mut job), [None, None]);
    assert_eq!(price_list(&mut job).rows[0].name, HEATER);
    for said in done.said() {
        assert!(!said.contains("$0.00"), "{said}");
    }
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    assert_eq!(item_amounts(&mut job), [Some(1850.0), Some(1850.0)]);
}

/// Step 5: $0.00 is taken, marked on the row and in the reply, and the lines read $0.00.
#[test]
fn step_5_nothing_typed_is_taken_and_marked() {
    let mut job = heaters();
    let done = set_price(&mut job, HEATER, None, Some("$0.00"), assumed()).unwrap();
    assert!(done.at_nothing);
    assert!(done.said().iter().any(|s| s.contains("$0.00") && s.contains("nothing is charged")), "{:?}", done.said());
    let r = row(&mut job, HEATER);
    assert_eq!((r.price.value, r.price.at_nothing, r.price.riding), (Some(0.0), true, Some(0.0)));
    assert_eq!(item_amounts(&mut job), [Some(0.0), Some(0.0)]);
}

/// Step 6: a name that is neither is refused, naming what is near; abc and a price below nothing are refused; the job is as it was.
#[test]
fn step_6_what_is_refused_leaves_the_job_as_it_was() {
    let mut job = heaters();
    let before = save(&mut job);
    let err = set_price(&mut job, "Water heater, 40 gal", None, Some("900"), assumed()).unwrap_err();
    assert!(err.contains("neither a material nor an item") && err.contains(HEATER), "{err}");
    let err = set_price(&mut job, HEATER, None, Some("abc"), assumed()).unwrap_err();
    assert!(err.contains("abc") && err.contains("not a figure"), "{err}");
    let err = set_price(&mut job, HEATER, None, Some("-5"), assumed()).unwrap_err();
    assert!(err.contains("less than nothing"), "{err}");
    let err = set_price(&mut job, HEATER, Some("  "), Some("5"), assumed()).unwrap_err();
    assert!(err.contains("supplier"), "{err}");
    let err = set_price(&mut job, HEATER, None, Some("900"), ed::Entered { status: Status::Unknown, source: "x" }).unwrap_err();
    assert!(err.contains("Unknown"), "{err}");
    let err = set_price(&mut job, HEATER, None, Some("900/SF"), assumed()).unwrap_err();
    assert!(err.contains("SF"), "{err}");
    assert_eq!(save(&mut job), before);
}

/// Step 7: a material a line is made of, priced by the same change: copper at $9.00/lb (made up) to $10.00/lb. The 16 oz
/// copper is made of on two planes: 461.1890 lb + 276.7134 lb. At $10.00/lb: $4,611.89 + $2,767.13 (cents each).
#[test]
fn step_7_a_material_by_the_same_change() {
    let mut job = sample::priced();
    let lines_before: Vec<_> = estimate(&mut job).lines.iter().filter(|l| l.material.as_deref() == Some("Copper, 16 oz")).map(|l| l.amount.value).collect();
    assert_eq!(lines_before.len(), 2);
    let done = set_price(&mut job, "copper, 16 OZ", None, Some("$10.00"), assumed()).unwrap();
    assert_eq!((done.lines, done.things), (2, 2));
    assert_eq!(done.name, "Copper, 16 oz");
    let r = row(&mut job, "Copper, 16 oz");
    assert_eq!((r.price.value, r.unit.as_str(), r.lines), (Some(10.0), "lb", 2));
    let now: Vec<f64> = estimate(&mut job).lines.iter().filter(|l| l.material.as_deref() == Some("Copper, 16 oz")).filter_map(|l| l.amount.value).collect();
    assert_eq!(now, [4611.89, 2767.13]);
    assert_eq!(r.price.riding, Some(7379.02));
    // By hand: at $9.00/lb the two came to 4,150.70 + 2,490.42 = 6,641.12; at $10.00 to 7,379.02: the job cost is 737.90 more.
    assert_eq!(lines_before, [Some(4150.70), Some(2490.42)]);
    assert_eq!(((done.cost_after.unwrap() - done.cost_before.unwrap()) * 100.0).round() / 100.0, 737.90);
}

/// Step 8: the price list written out and brought into a second job with the same things: the same, row for row.
#[test]
fn step_8_the_list_written_out_comes_back_the_same() {
    let mut job = heaters();
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    set_price(&mut job, HEATER, Some("Northside Supply"), Some("1790"), assumed()).unwrap();
    let out = tables::prices_out(&mut job);
    let mut second = heaters();
    tables::prices_in(&mut second, &out, None, None, false, None).unwrap();
    assert_eq!(tables::prices_out(&mut second), out);
    assert_eq!(price_list(&mut second), price_list(&mut job));
}

/// A price for a material nothing is made of is taken, and the reply says it prices nothing yet.
#[test]
fn a_price_for_a_material_nothing_is_made_of_is_taken_and_says_it_prices_nothing() {
    let mut job = sample::priced();
    job.add_material("Zinc sheet (made up)", "sheet").unwrap();
    let done = set_price(&mut job, "Zinc sheet (made up)", None, Some("$41.00"), assumed()).unwrap();
    assert_eq!((done.lines, done.things), (0, 0));
    assert!(done.said().iter().any(|s| s.contains("prices nothing")), "{:?}", done.said());
    let r = row(&mut job, "Zinc sheet (made up)");
    assert_eq!((r.price.value, r.lines, r.price.riding), (Some(41.0), 0, None));
    assert_eq!(r.unit, "sheet");
}

/// Two suppliers, then a third and a fourth: the model has no limit of three; the list and the estimate give them all.
#[test]
fn four_suppliers_side_by_side() {
    let mut job = heaters();
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    for (n, who) in ["Northside Supply", "Eastside Supply", "Southside Supply", "Westside Supply"].iter().enumerate() {
        let done = set_price(&mut job, HEATER, Some(who), Some(&format!("{}", 1800 - 10 * n)), assumed()).unwrap();
        assert_eq!(done.supplier_added.as_deref(), Some(*who));
    }
    let list = price_list(&mut job);
    assert_eq!(list.suppliers, ["Eastside Supply", "Northside Supply", "Southside Supply", "Westside Supply"]);
    let r = &list.rows[0];
    let by: Vec<Option<f64>> = r.suppliers.iter().map(|c| c.as_ref().and_then(|c| c.value)).collect();
    assert_eq!(by, [Some(1790.0), Some(1800.0), Some(1780.0), Some(1770.0)]);
    let est = estimate(&mut job);
    assert_eq!(est.suppliers.len(), 4);
    // A supplier that has not quoted what a row is made of has no cell.
    let mut job2 = sample::priced();
    set_price(&mut job2, "Copper, 20 oz", Some("Corner Supply"), Some("9.50"), assumed()).unwrap();
    let list = price_list(&mut job2);
    let r16 = list.rows.iter().find(|r| r.name == "Copper, 16 oz").unwrap();
    let at = list.suppliers.iter().position(|s| s == "Corner Supply").unwrap();
    assert!(r16.suppliers[at].is_none());
    assert_eq!(list.suppliers.len(), 4);
}

/// The same price again says nothing changed, and the job is as it was.
#[test]
fn the_same_price_again_says_nothing_changed() {
    let mut job = heaters();
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    let before = save(&mut job);
    let again = set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    assert!(again.same);
    assert!(again.said().iter().any(|s| s.contains("nothing changed")), "{:?}", again.said());
    assert_eq!(save(&mut job), before);
}

/// A price set, the item renamed: the price is still its, under the new name.
#[test]
fn a_price_set_and_the_item_renamed_is_still_its() {
    let mut job = heaters();
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    let path = job.world.query::<(&Node, &Item)>().iter(&job.world).find(|(n, _)| n.name == HEATER).map(|(n, _)| n.path.clone()).unwrap();
    ed::rename_item(&mut job, &path, "Tank heater, 50 gal").unwrap();
    let list = price_list(&mut job);
    assert!(list.rows.iter().all(|r| r.name != HEATER));
    let r = row(&mut job, "Tank heater, 50 gal");
    assert_eq!((r.price.value, r.lines, r.price.riding), (Some(1850.0), 2, Some(3700.0)));
}

/// A price set, then `import prices` of a list with the same name: the later wins, and the import says it did.
#[test]
fn a_price_set_then_a_list_with_the_same_name_the_list_wins_and_says_so() {
    let mut job = heaters();
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    let list: Vec<Vec<String>> = vec![vec!["Material".into(), "Bought by".into(), "Price".into()], vec![HEATER.into(), "EA".into(), "1900".into()]];
    let how = ed::Entered { status: Status::Assumed, source: "made-up list" };
    let brought = tables::prices_in(&mut job, &list, None, None, false, Some(how)).unwrap();
    assert!(brought.done.iter().any(|d| d.contains("$1,850.00/EA becomes $1,900.00/EA")), "{:?}", brought.done);
    assert_eq!(row(&mut job, HEATER).price.value, Some(1900.0));
    // And the other way round: the typed price over the list's, said the same way.
    let done = set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    assert!(done.said().iter().any(|s| s.contains("$1,900.00/EA becomes $1,850.00/EA")), "{:?}", done.said());
}

/// Clearing what was never set changes nothing and gives the item no entry it had not; a supplier that never quoted
/// has nothing to clear.
#[test]
fn clearing_what_was_never_set_changes_nothing() {
    let mut job = heaters();
    let before = save(&mut job);
    let done = set_price(&mut job, HEATER, None, None, assumed()).unwrap();
    assert!(done.same && done.cleared);
    assert!(done.said().iter().any(|s| s.contains("nothing changed")), "{:?}", done.said());
    let done = set_price(&mut job, HEATER, Some("Northside Supply"), None, assumed()).unwrap();
    assert!(done.same && done.supplier_added.is_none());
    assert_eq!(save(&mut job), before);
}

/// The head of the list: what has no price and lines waiting, then by dollars riding; a priced thing nothing is made of
/// comes after, with no lines.
#[test]
fn the_list_is_in_the_order_the_estimate_waits_on_it() {
    let mut job = heaters();
    job.add_material("Zinc sheet (made up)", "sheet").unwrap();
    set_price(&mut job, "Zinc sheet (made up)", None, Some("41"), assumed()).unwrap();
    let names: Vec<String> = price_list(&mut job).rows.into_iter().map(|r| r.name).collect();
    assert_eq!(names, [HEATER, "Zinc sheet (made up)"]);
    set_price(&mut job, HEATER, None, Some("1850"), assumed()).unwrap();
    let names: Vec<String> = price_list(&mut job).rows.into_iter().map(|r| r.name).collect();
    assert_eq!(names, [HEATER, "Zinc sheet (made up)"], "$3,700.00 riding is ahead of nothing riding");
}

/// A name written another way is found, and so is a path. (The job holds no material and item of one name: it
/// refuses the second, so a name is never two things; `find` still refuses a name that were.)
#[test]
fn a_name_written_another_way_or_a_path_is_found() {
    let mut job = heaters();
    set_price(&mut job, "water heater 50 GAL", None, Some("1850"), assumed()).unwrap();
    assert_eq!(row(&mut job, HEATER).price.value, Some(1850.0));
    let path = row(&mut job, HEATER).path;
    let done = set_price(&mut job, &path, None, Some("1900"), assumed()).unwrap();
    assert_eq!(done.name, HEATER);
    let err = job.add_material(HEATER, "EA").unwrap_err();
    assert!(err.contains("already something called"), "{err}");
}

/// Call 207 ruling 1: where the price list would store what a plain price does not, it says so (and the command line keeps the
/// older road there); everywhere else the two store the same.
#[test]
fn where_the_price_list_would_store_more_than_a_plain_price_it_says_so() {
    use plumbline_world::pricelist::stores_more_than_a_plain_price as more;
    let mut job = sample::priced();
    assert_eq!(more(&mut job, "Copper, 16 oz", None), None);
    assert_eq!(more(&mut job, "Copper, 16 oz", Some("Corner Supply")), None);
    assert_eq!(more(&mut job, "Copper, 16 oz", Some("harbor  metals")).map(|s| s.contains("another spelling")), Some(true));
    job.add_material("Zinc sheet (made up)", "sheet").unwrap();
    assert_eq!(more(&mut job, "Zinc sheet (made up)", None), None);
    assert!(more(&mut job, "Zinc sheet (made up)", Some("Corner Supply")).unwrap().contains("call 108"));
    // A plain price and the price list store the same figure, unit, status and source on a material that has none of the three differences.
    let mut a = sample::priced();
    let mut b = sample::priced();
    a.set_on_line("material-0001", "price", None, Some("$10.00"), Status::Assumed, "made-up quote").ok();
    let path = b.world.query::<(&Node, &Supply)>().iter(&b.world).find(|(n, _)| n.name == "Copper, 16 oz").map(|(n, _)| n.path.clone()).unwrap();
    let a_path = a.world.query::<(&Node, &Supply)>().iter(&a.world).find(|(n, _)| n.name == "Copper, 16 oz").map(|(n, _)| n.path.clone()).unwrap();
    a.set_on_line(&a_path, "price", None, Some("$10.00"), Status::Assumed, "made-up quote").unwrap();
    set_price(&mut b, &path, None, Some("$10.00"), assumed()).unwrap();
    let supply = |j: &mut Job| j.world.query::<&Supply>().iter(&j.world).filter(|s| s.price.value == Some(10.0)).cloned().collect::<Vec<_>>();
    assert_eq!(supply(&mut a), supply(&mut b));
    assert_eq!(supply(&mut a).len(), 1);
}
