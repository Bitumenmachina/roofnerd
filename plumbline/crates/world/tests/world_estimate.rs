use plumbline_world::file::{load, save};
use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{figures, measure_of, path_named, ft, walled, wall, room, figures_of, priced_of, extension_of, boxed, class_total, priced_known};

#[test]
fn the_estimate_puts_every_line_in_one_class_and_adds_up_what_is_shown() {
    let mut job = sample::priced();
    let est = view::estimate(&mut job);
    let total = |c: Class| class_total(&est, c).total.clone();
    assert_eq!((total(Class::Material).value, total(Class::Material).status, total(Class::Material).members), (Some(8628.10), Status::Assumed, 6));
    assert_eq!(total(Class::Material).assumed, 1383.24);
    assert_eq!((total(Class::Labor).value, total(Class::Labor).status), (Some(2147.97), Status::Assumed));
    assert_eq!(class_total(&est, Class::Labor).hours, Some((29.8328, Status::Assumed)));
    assert_eq!((total(Class::Fabrication).value, total(Class::Fabrication).status), (Some(402.50), Status::Known));
    assert_eq!(class_total(&est, Class::Fabrication).hours, Some((6.7082, Status::Known)));
    assert_eq!(total(Class::Supervision).value, Some(3800.00));
    assert_eq!(class_total(&est, Class::Supervision).hours, Some((40.0, Status::Known)));
    assert_eq!(total(Class::Subcontract).value, Some(10313.86));
    assert_eq!(class_total(&est, Class::Subcontract).hours, None);
    // Nothing is in general conditions, and that is not $0.00.
    assert_eq!(total(Class::General).value, None);
    // The three columns: material; labor with fabrication and supervision; the rest.
    let column = |c: Column| est.columns.iter().find(|x| x.0 == c).unwrap().1.value;
    assert_eq!(column(Column::Material), Some(8628.10));
    assert_eq!(column(Column::Labor), Some(6350.47)); // 2,147.97 + 402.50 + 3,800.00
    assert_eq!(column(Column::General), Some(10313.86));
    // The job cost is the classes added up, and it names the line that is in none.
    assert_eq!((est.job_cost.value, est.job_cost.status, est.job_cost.members), (Some(25292.43), Status::Assumed, 13));
    assert_eq!(est.job_cost.left_out.len(), 1);
    assert!(est.job_cost.left_out[0].0.starts_with("plane-0005") && est.job_cost.left_out[0].1.contains("no cost type"), "{:?}", est.job_cost.left_out);
    assert_eq!(est.not_classed, [("plane-0005".to_string(), "Copper standing seam, installed".to_string())]);
    // Every line that is on, classed and has a figure is in exactly one class.
    let in_classes: usize = est.classes.iter().map(|c| c.total.members).sum();
    assert_eq!(in_classes, est.lines.iter().filter(|l| !l.off && l.class.is_some() && l.amount.value.is_some()).count());
    // Overhead and profit are not entered: there is no price, and the recap says why.
    assert_eq!(est.price.value, None);
    assert!(est.price.note.contains("overhead has not been entered"), "{}", est.price.note);
    // It saves, and loads to the same bytes.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
}

#[test]
fn overhead_and_profit_are_entered_and_neither_has_a_default() {
    let mut job = sample::priced();
    let known = Status::Known;
    // Overhead alone: 25,292.43 x 10 % = 2,529.243 -> 2,529.24. Still no price: no profit.
    job.set("job-0001", "overhead", Some("10"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!(est.overhead.value, Some(2529.24));
    assert_eq!(est.profit.value, None);
    assert!(est.price.note.contains("profit is not known (not entered)"), "{}", est.price.note);
    // Profit is on the job cost and the overhead together:
    // (25,292.43 + 2,529.24) x 8 % = 27,821.67 x 0.08 = 2,225.7336 -> 2,225.73.
    job.set("job-0001", "profit", Some("8 %"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!(est.profit.value, Some(2225.73));
    // The price is the three figures as shown: 25,292.43 + 2,529.24 + 2,225.73 = 30,047.40.
    // It rests on the shed's assumed pitch, so it is Assumed.
    assert_eq!((est.price.value, est.price.status), (Some(30047.40), Status::Assumed));
    // Changed: 12 % is 3,035.0916 -> 3,035.09; profit 28,327.52 x 0.08 = 2,266.2016 -> 2,266.20.
    job.set("job-0001", "overhead", Some("12"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!((est.overhead.value, est.profit.value, est.price.value), (Some(3035.09), Some(2266.20), Some(30593.72)));
    // Nought entered is nought, and a price: it is an answer. Unknown is not.
    job.set("job-0001", "overhead", Some("0"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    // Profit 25,292.43 x 0.08 = 2,023.3944 -> 2,023.39; price 27,315.82.
    assert_eq!((est.overhead.value, est.profit.value, est.price.value), (Some(0.0), Some(2023.39), Some(27315.82)));
    job.set("job-0001", "overhead", None, Status::Unknown, "not settled with the owner").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!((est.overhead.value, est.profit.value, est.price.value), (None, None, None));
    assert!(est.profit.note.contains("the overhead has no figure"), "{}", est.profit.note);
    // An assumed percentage makes an assumed price even on a job that is all Known.
    // Refused: not a number; below nought; on something that is not the job.
    assert!(job.set("job-0001", "overhead", Some("ten"), known, "x").is_err());
    assert!(job.set("job-0001", "profit", Some("-5"), known, "x").unwrap_err().contains("percentage from 0"));
    assert!(job.set("plane-0003", "profit", Some("5"), known, "x").unwrap_err().contains("the job's"));
}

/// A labor line with its hours taken off and no price yet is in neither the dollars
/// nor the hours beside them, and its hours are held apart to be said (call 82; M21-3:
/// they were in nothing, and nothing said so).
#[test]
fn hours_on_a_labor_line_with_no_price_are_held_apart_and_said() {
    let mut job = sample::priced();
    job.derive();
    let before = view::estimate(&mut job);
    assert_eq!(class_total(&before, Class::Labor).hours, Some((29.8328, Status::Assumed)));
    assert_eq!(class_total(&before, Class::Labor).hours_unpriced, None);
    // "Install panels" on the south plane is 16.7705 hr. Its price is marked unknown.
    job.set_on_line("plane-0003", "unit_cost", Some("Install panels"), None, Status::Unknown, "made-up: not priced yet").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let labor = class_total(&est, Class::Labor);
    // The hours beside the dollars: 29.8328 - 16.7705 = 13.0623. The rest wait, said.
    assert_eq!(labor.hours, Some((13.0623, Status::Assumed)));
    assert_eq!(labor.hours_unpriced, Some((16.7705, 1)));
    assert_eq!(labor.hours_unpriced_said().as_deref(), Some("16.77 hr more on 1 line with no price"));
    // A cost type with no hours in it says nothing of the kind.
    assert_eq!(class_total(&est, Class::Material).hours_unpriced, None);
}

/// A thing's material is found by its name as names are compared everywhere else:
/// capitals, spacing and a full stop aside (M21-12: named "copper, 16 OZ" in a job file,
/// it priced nothing, while a price list and the keyboard took it).
#[test]
fn a_material_is_found_by_its_name_however_it_is_capitalised() {
    let mut job = sample::priced();
    let before = priced_of(&mut job, "plane-0003", "detail").value;
    assert_eq!(before, Some(5987.08));
    job.set("plane-0003", "material", Some("copper,16 OZ."), Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, before);
    let est = view::estimate(&mut job);
    assert!(est.lines.iter().filter(|l| l.path == "plane-0003" && l.material.is_some()).all(|l| l.amount.value.is_some()), "a line made of the material has no figure");
}

#[test]
fn a_labor_line_is_switched_off_and_leaves_every_total_by_its_amount() {
    let mut job = sample::priced();
    // The south plane's detail: 4,150.70 + 377.34 + 251.56 + 1,207.48 = 5,987.08.
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(5987.08));
    job.set_off("plane-0003", Some("Install panels"), true).unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    // Labor: 2,147.97 - 1,207.48 = 940.49. Hours: 29.8328 - 16.7705 = 13.0623.
    assert_eq!(class_total(&est, Class::Labor).total.value, Some(940.49));
    assert_eq!(class_total(&est, Class::Labor).hours, Some((13.0623, Status::Assumed)));
    assert_eq!(est.job_cost.value, Some(24084.95)); // 25,292.43 - 1,207.48
    assert_eq!(est.job_cost.left_out.len(), 1, "a line switched off is not a line left out");
    // The element's detail leaves it too: 5,987.08 - 1,207.48 = 4,779.60.
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(4779.60));
    // It is still on the sheet, marked off, with what it would come to.
    let line = est.lines.iter().find(|l| l.path == "plane-0003" && l.description == "Install panels").unwrap();
    assert!(line.off);
    assert_eq!(line.amount.value, Some(1207.48));
    // The file says it is off, and loads so.
    let text = save(&mut job);
    assert!(text.contains("\"off\": true"));
    let (mut again, _) = load(&text).unwrap();
    assert_eq!(view::estimate(&mut again).job_cost.value, Some(24084.95));
    // On again: back to the cent.
    job.set_off("plane-0003", Some("Install panels"), false).unwrap();
    job.derive();
    assert_eq!(view::estimate(&mut job).job_cost.value, Some(25292.43));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(5987.08));
    assert!(!save(&mut job).contains("\"off\""));
    // Only a Labor line has the switch.
    for line in ["Standing seam panels", "Underlayment", "Fabricate panels"] {
        assert!(job.set_off("plane-0003", Some(line), true).unwrap_err().contains("only a Labor line"), "{line}");
    }
    assert!(job.set_off("plane-0006", None, true).unwrap_err().contains("only a Labor line"));
}

#[test]
fn in_house_work_is_a_production_rate_and_a_subcontract_is_a_price() {
    let mut job = sample::priced();
    // 419.2627 SF at 25 SF to the hour is 16.7705 hours, not rounded up to 17.
    let f = figures(&mut job, "plane-0003", "Install panels");
    assert_eq!((f.order.as_ref().unwrap().value, f.order.as_ref().unwrap().unit.as_str()), (Some(16.7705), "hr"));
    assert_eq!(f.extension.unwrap().value, Some(1207.48));
    // Stated the other way up: 12 LF at a quarter of an hour each is 3 hours.
    assert_eq!(figures(&mut job, "headwall-0021", "Install headwall flashing").order.unwrap().value, Some(3.0));
    // A rate typed on the command line is not rounded up to whole hours either:
    // 419.2627 / 40 = 10.4816 hr.
    job.set_on_line("plane-0003", "order", Some("Install panels"), Some("40 SF/hr"), Status::Known, "made-up rate").unwrap();
    job.derive();
    assert_eq!(figures(&mut job, "plane-0003", "Install panels").order.unwrap().value, Some(10.4816));
    // A subcontract line is refused a production rate, and in-house work is refused
    // without one.
    assert!(job.set_class("plane-0003", Some("Install panels"), Class::Subcontract).unwrap_err().contains("Subcontract and carries a production rate"));
    assert!(job.set_class("plane-0003", Some("Install panels"), Class::Material).unwrap_err().contains("is in hours"));
    assert!(job.set_class("plane-0006", None, Class::Labor).unwrap_err().contains("is not in hours"));
    assert!(job.set_class("plane-0003", Some("Underlayment"), Class::Fabrication).unwrap_err().contains("is not in hours"));
    // Fabrication, labor and supervision are each a class of in-house work, and a line
    // in hours can be moved between them.
    job.set_class("plane-0003", Some("Install panels"), Class::Supervision).unwrap();
    job.set_class("plane-0003", Some("Install panels"), Class::Labor).unwrap();
    // A class is typed in a few plain ways and nothing else.
    assert_eq!(Class::parse("Sub"), Ok(Class::Subcontract));
    assert_eq!(Class::parse(" general "), Ok(Class::General));
    assert!(Class::parse("overhead").is_err());
}

#[test]
fn changing_what_an_element_is_made_of_reprices_the_lines_made_of_it_and_no_others() {
    let mut job = sample::priced();
    let before = view::estimate(&mut job);
    // South plane, 16 oz to 20 oz copper: 461.1890 SF x 1.25 lb/SF = 576.48625 -> 576.4863 lb
    // (it was 461.1890 lb), x 9.00 = 5,188.3767 -> 5,188.38 (it was 4,150.70).
    job.set("plane-0003", "material", Some("Copper, 20 oz"), Status::Known, "made-up revision").unwrap();
    job.derive();
    let f = figures(&mut job, "plane-0003", "Standing seam panels");
    assert_eq!(f.order.as_ref().unwrap().value, Some(576.4863));
    assert_eq!(f.extension.as_ref().unwrap().value, Some(5188.38));
    // Its quotes are the new material's: Harbor Metals 576.4863 x 8.90 = 5,130.72807 -> 5,130.73;
    // Tinners Yard has not quoted 20 oz.
    assert_eq!(f.quotes["Harbor Metals"].value, Some(5130.73));
    assert!(!f.quotes.contains_key("Tinners Yard"));
    // The lines not made of it have not moved.
    for (line, amount) in [("Underlayment", 377.34), ("Fabricate panels", 251.56), ("Install panels", 1207.48)] {
        assert_eq!(extension_of(&mut job, "plane-0003", line).value, Some(amount), "{line}");
    }
    // The recap moved by the difference and nothing else: 5,188.38 - 4,150.70 = 1,037.68.
    let after = view::estimate(&mut job);
    assert_eq!(class_total(&after, Class::Material).total.value, Some(9665.78)); // 8,628.10 + 1,037.68
    assert_eq!(after.job_cost.value, Some(26330.11)); // 25,292.43 + 1,037.68
    for class in [Class::Labor, Class::Fabrication, Class::Supervision, Class::Subcontract] {
        assert_eq!(class_total(&after, class).total, class_total(&before, class).total, "{class:?}");
    }
    // An assumed material makes assumed lines.
    job.set("plane-0003", "material", Some("Copper, 20 oz"), Status::Assumed, "not confirmed with the architect").unwrap();
    job.derive();
    assert_eq!(extension_of(&mut job, "plane-0003", "Standing seam panels").status, Status::Assumed);
    assert_eq!(extension_of(&mut job, "plane-0003", "Underlayment").status, Status::Known);
    // A material the job does not have: the line has no figure and says why; it is not
    // priced at the old material, and not at nothing.
    job.set("plane-0003", "material", Some("Zinc, 0.7 mm"), Status::Known, "made-up revision").unwrap();
    job.derive();
    let d = extension_of(&mut job, "plane-0003", "Standing seam panels");
    assert_eq!(d.value, None);
    assert!(d.note.contains("the job has no material of that name"), "{}", d.note);
    let est = view::estimate(&mut job);
    assert!(est.job_cost.left_out.iter().any(|(who, why)| who.starts_with("plane-0003") && why.contains("no material of that name")), "{:?}", est.job_cost.left_out);
    // Not known what it is made of: the same, with that reason.
    job.set("plane-0003", "material", None, Status::Unknown, "finish schedule not issued").unwrap();
    job.derive();
    assert!(extension_of(&mut job, "plane-0003", "Standing seam panels").note.contains("is not known (finish schedule not issued)"));
    // Changing a material's price reprices every element made of it. East plane, 16 oz at
    // 9.50: 276.7134 x 9.50 = 2,628.7773 -> 2,628.78.
    let copper = job.index().into_iter().map(|(p, _)| p).find(|p| p.starts_with("material-") && job.find(p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name == "Copper, 16 oz")).unwrap_or(false)).unwrap();
    job.set(&copper, "price", Some("$9.50/lb"), Status::Known, "made-up price").unwrap();
    job.derive();
    assert_eq!(extension_of(&mut job, "plane-0004", "Standing seam panels").value, Some(2628.78));
    // A material's price is for the unit it is bought by.
    assert!(job.set(&copper, "price", Some("$9.50/SF"), Status::Known, "x").unwrap_err().contains("does not carry from one unit to another"));
    // A line made of the element's material carries no price or order of its own.
    assert!(job.set_on_line("plane-0004", "unit_cost", Some("Standing seam panels"), Some("9"), Status::Known, "x").unwrap_err().contains("its price is the material's"));
    assert!(job.set_on_line("plane-0004", "order", Some("Standing seam panels"), Some("1 lb/SF"), Status::Known, "x").unwrap_err().contains("is the material's"));
    // A thing that is not made of a material cannot be given one.
    assert!(job.set("job-0001", "material", Some("Copper, 16 oz"), Status::Known, "x").unwrap_err().contains("is not made of a material"));
}

#[test]
fn suppliers_stand_side_by_side_and_a_total_names_what_it_leaves_out() {
    let mut job = sample::priced();
    let est = view::estimate(&mut job);
    assert_eq!(est.suppliers, ["Harbor Metals", "Ridge Supply", "Tinners Yard"]);
    // Generic, over the four lines made of a material: 4,150.70 + 2,490.42 + 1,113.24 + 270.00.
    assert_eq!((est.at_generic.value, est.at_generic.members), (Some(8024.36), 4));
    let at = |s: &str| est.at_supplier.iter().find(|x| x.0 == s).unwrap().1.clone();
    // Harbor Metals: 461.1890 x 8.80 = 4,058.46; 276.7134 x 8.80 = 2,435.08;
    // 123.6931 x 8.90 = 1,100.87; 30 x 8.90 = 267.00. Total 7,861.41.
    assert_eq!((at("Harbor Metals").value, at("Harbor Metals").members), (Some(7861.41), 4));
    // Ridge Supply: 4,266.00 + 2,559.60 + 1,150.35 + 279.00 = 8,254.95.
    assert_eq!(at("Ridge Supply").value, Some(8254.95));
    // Tinners Yard quoted 16 oz only: 461.1890 x 9.10 = 4,196.82; 276.7134 x 9.10 = 2,518.09.
    // Its total is over two lines, and it names the two it leaves out.
    let tinners = at("Tinners Yard");
    assert_eq!((tinners.value, tinners.members, tinners.left_out.len()), (Some(6714.91), 2, 2));
    assert!(tinners.left_out.iter().all(|(_, why)| why.contains("has not quoted \"Copper, 20 oz\"")), "{:?}", tinners.left_out);
    // A quote is entered for the unit the material is bought by, and no other.
    let m20 = job.index().into_iter().map(|(p, _)| p).find(|p| job.find(p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name == "Copper, 20 oz")).unwrap_or(false)).unwrap();
    // (Since call 208 a quote goes by the price list's road, `pricelist::set_price`, the only road; its words for the two refusals.)
    let how = |status, source| plumbline_world::editing::Entered { status, source };
    assert!(plumbline_world::pricelist::set_price(&mut job, &m20, Some("Tinners Yard"), Some("$9.40/SF"), how(Status::Known, "x")).unwrap_err().contains("priced by the SF"));
    assert!(plumbline_world::pricelist::set_price(&mut job, "plane-0003", Some("Tinners Yard"), Some("9.40"), how(Status::Known, "x")).unwrap_err().contains("neither a material nor an item"));
    // Entered: 123.6931 x 9.40 = 1,162.71514 -> 1,162.72; 30 x 9.40 = 282.00.
    // Tinners Yard is now over all four: 6,714.91 + 1,162.72 + 282.00 = 8,159.63.
    plumbline_world::pricelist::set_price(&mut job, &m20, Some("Tinners Yard"), Some("$9.40/lb"), how(Status::Assumed, "made-up quote, by telephone")).unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let tinners = est.at_supplier.iter().find(|x| x.0 == "Tinners Yard").unwrap().1.clone();
    assert_eq!((tinners.value, tinners.members, tinners.left_out.len()), (Some(8159.63), 4, 0));
    // A quote asked for and not come is a quote with no figure, named: not a price of nothing.
    plumbline_world::pricelist::set_price(&mut job, &m20, Some("Tinners Yard"), None, how(Status::Unknown, "asked for, not received")).unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let tinners = est.at_supplier.iter().find(|x| x.0 == "Tinners Yard").unwrap().1.clone();
    assert_eq!((tinners.value, tinners.members), (Some(6714.91), 2));
    assert!(tinners.left_out.iter().all(|(_, why)| why.contains("has no price (asked for, not received)")), "{:?}", tinners.left_out);
}

#[test]
fn a_line_typed_by_hand_onto_the_job_lands_in_general_conditions() {
    let mut job = sample::priced();
    job.add_line("job-0001", "Dumpster", "2", Some(Class::General), false).unwrap();
    job.derive();
    // Typed and not yet priced: it has no figure and is named. General conditions are not $0.00.
    let est = view::estimate(&mut job);
    assert_eq!(class_total(&est, Class::General).total.value, None);
    assert!(est.job_cost.left_out.iter().any(|(who, why)| who.contains("Dumpster") && why.contains("has no price")), "{:?}", est.job_cost.left_out);
    // 2 x 650.00 = 1,300.00; a permit, 1 x 1,850.00.
    job.set_on_line("job-0001", "unit_cost", Some("Dumpster"), Some("650"), Status::Known, "made-up price").unwrap();
    job.add_line("job-0001", "Permit", "1", Some(Class::General), false).unwrap();
    job.set_on_line("job-0001", "unit_cost", Some("Permit"), Some("$1,850"), Status::Known, "made-up price").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!((class_total(&est, Class::General).total.value, class_total(&est, Class::General).total.members), (Some(3150.00), 2));
    assert_eq!(est.job_cost.value, Some(28442.43)); // 25,292.43 + 3,150.00
    // The general column is the subcontract and the general conditions: 10,313.86 + 3,150.00.
    assert_eq!(est.columns.iter().find(|x| x.0 == Column::General).unwrap().1.value, Some(13463.86));
    // A line with no class can be typed; it is named and is in no row until it has one.
    job.add_line("job-0001", "Mobilization", "1", None, false).unwrap();
    job.set_on_line("job-0001", "unit_cost", Some("Mobilization"), Some("500"), Status::Known, "made-up price").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(est.job_cost.value, Some(28442.43));
    assert!(est.not_classed.contains(&("job-0001".to_string(), "Mobilization".to_string())));
    job.set_class("job-0001", Some("Mobilization"), Class::General).unwrap();
    job.derive();
    assert_eq!(view::estimate(&mut job).job_cost.value, Some(28942.43));
    // Two lines on one element are not described the same.
    assert!(job.add_line("job-0001", "dumpster", "1", Some(Class::General), false).is_err());
    // A line made of the element's material is Material, and on an element that does not
    // say what it is made of it has no figure and says so.
    assert!(job.add_line("plane-0006", "Trim", "surface_area", Some(Class::General), true).unwrap_err().contains("its cost type is Material"));
    job.add_line("plane-0006", "Trim", "surface_area", None, true).unwrap();
    job.derive();
    assert!(extension_of(&mut job, "plane-0006", "Trim").note.contains("does not say what it is made of"));
}

// ---- The tenth judgment: each finding, as a test that fails without its fix ----

#[test]
fn j10_1_an_assumed_or_unknown_condition_shows_in_what_its_lines_come_to() {
    let mut job = sample();
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Known, "made-up quote").unwrap();
    job.set("plane-0003", "allowance", Some("28"), Status::Known, "made-up budget rate").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").status, Status::Known);
    // Only assumed to be that condition: everything its lines come to is assumed.
    job.set("plane-0003", "condition", Some("Standing seam copper, 16 oz"), Status::Assumed, "not on the finish schedule yet").unwrap();
    job.derive();
    for name in ["detail", "detail_rate", "difference", "difference_percent"] {
        let d = priced_of(&mut job, "plane-0003", name);
        assert_eq!((d.value.is_some(), d.status), (true, Status::Assumed), "{name}");
    }
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(13206.78));
    // The allowance is a rate on its area and does not rest on the condition.
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").status, Status::Known);
    let p = view::pricing(&mut job);
    assert_eq!((p.detail.status, p.detail.assumed), (Status::Assumed, 13206.78));
    // Not known: its lines come to no figure, with the reason; nothing is priced as if it were.
    job.set("plane-0003", "condition", None, Status::Unknown, "nobody knows").unwrap();
    job.derive();
    let d = priced_of(&mut job, "plane-0003", "detail");
    assert_eq!(d.value, None);
    assert!(d.note.contains("the element's assembly is not known (nobody knows)"), "{}", d.note);
}

#[test]
fn j10_3_a_line_whose_quantity_is_nothing_says_so() {
    let mut job = sample();
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Known, "made-up quote").unwrap();
    job.set_formula("plane-0003", None, "surface_area * 0", &[], Status::Known, "made-up").unwrap();
    job.derive();
    let d = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!(d.value, Some(0.0));
    assert_eq!(d.note, "the line's quantity is 0, so it comes to nothing");
    let detail = priced_of(&mut job, "plane-0003", "detail");
    assert_eq!(detail.value, Some(0.0));
    assert!(detail.note.contains("1 of its 1 lines comes to nothing because the quantity is 0"), "{}", detail.note);
    // A price of $0 that somebody entered is an answer, and is named on the line (call 206 point 13: it was said to carry no note).
    let mut job = sample();
    job.set("plane-0003", "unit_cost", Some("0"), Status::Known, "included in the panel price").unwrap();
    job.derive();
    let d = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!((d.value, d.note.as_str()), (Some(0.0), "its price is $0.00 as entered: nothing is charged for it"));
}

#[test]
fn j10_5_a_comma_in_a_price_is_a_thousands_mark_or_it_is_refused() {
    use plumbline_world::units::parse_price;
    assert_eq!(parse_price("1,250", "$/SF"), Ok(1250.0));
    assert_eq!(parse_price("$12,500.75/SF", "$/SF"), Ok(12500.75));
    assert_eq!(parse_price("1,234,567", "$/SF"), Ok(1234567.0));
    for wrong in ["1,25", ",5", "12,34.5", "1,2345", "1234,567", "1,,250", "1,250,", "1.250,5", "1,25.0"] {
        let why = parse_price(wrong, "$/SF").unwrap_err();
        assert!(why.contains("comma") || why.contains("not a price"), "{wrong}: {why}");
    }
}

#[test]
fn j10_6_a_rate_under_a_cent_is_not_written_as_nothing() {
    assert_eq!(view::rate(28.0), "28.00");
    assert_eq!(view::rate(31.5), "31.50");
    assert_eq!(view::rate(0.004), "0.004");
    assert_eq!(view::rate(1.015), "1.015");
    assert_eq!(view::rate(1250.0), "1,250.00");
    assert_eq!(view::rate(0.00305), "0.00305");
    let mut job = sample();
    job.set("plane-0004", "allowance", Some("0.004"), Status::Known, "made-up").unwrap();
    job.derive();
    let rows = view::elements(&mut job);
    let row = rows.iter().find(|e| e.path == "plane-0004").unwrap().rows.iter().find(|r| r.label == "allowance").unwrap().clone();
    assert!(row.text.starts_with("0.004"), "{}", row.text);
}

#[test]
fn j10_7_money_is_rounded_to_the_cent_as_it_would_be_on_paper() {
    use plumbline_world::lines::cents;
    // 1.015 is held as 1.01499999999999990230...; on paper it is half a cent and goes up.
    assert_eq!(cents(1.015), 1.02);
    assert_eq!(cents(4.0 * 0.25375), 1.02);
    assert_eq!(cents(2.675), 2.68);
    assert_eq!(cents(1.0149), 1.01);
    assert_eq!(cents(1.014999), 1.01);
    assert_eq!(cents(-1.015), -1.02);
    assert_eq!(cents(0.0), 0.0);
    let mut job = sample();
    job.set_formula("plane-0004", None, "n", &[("n".to_string(), "1".to_string())], Status::Known, "made-up").unwrap();
    job.set("plane-0004", "unit_cost", Some("1.015"), Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(extension_of(&mut job, "plane-0004", "Copper standing seam, installed").value, Some(1.02));
}

#[test]
fn j10_8_what_is_bought_by_the_each_is_counted_when_it_is_not_priced() {
    let mut job = sample::house();
    let p = view::pricing(&mut job);
    let count = |k: Kind| p.not_priced.iter().find(|x| x.0 == k).map(|x| x.1).unwrap_or(0);
    // The sample house has nine doors and seventeen windows, and none has a price.
    assert_eq!((count(Kind::Door), count(Kind::Window)), (9, 17));
    assert!(count(Kind::Fitting) > 0 && count(Kind::Fixture) > 0 && count(Kind::Equipment) > 0);
    // What is not a thing to price is not counted: a storey, a system, a type.
    assert_eq!((count(Kind::Storey), count(Kind::System), count(Kind::Type)), (0, 0, 0));
}

#[test]
fn j10_11_money_past_what_holds_to_the_cent_is_refused() {
    let d = plumbline_world::lines::dollars(Some(999_999_999_989_999_861_760.0), &[Status::Known], "", &[]);
    assert_eq!(d.value, None);
    assert!(d.note.contains("more than can be held to the cent"));
    assert_eq!(plumbline_world::lines::dollars(Some(9.9e12), &[Status::Known], "", &[]).value, Some(9.9e12));
}

#[test]
fn j10_2_a_room_is_no_surer_than_a_wall_its_walls_are_cut_to() {
    use plumbline_world::{Wall, WallType};
    // The box of J9-4: N across the middle, not bounding rooms, and a partition P from N's
    // line east to the east wall. P is cut to N's face, so the room rests on N's thickness
    // and line though N bounds nothing.
    let (mut job, s, t, r) = boxed();
    let thick = job.add("job-0001", Kind::Type, "N's type").unwrap();
    job.put(&thick, WallType { thickness: Val::known(Len::One(6.0), "in", "test"), layers: Vec::new(), footing: false }).unwrap();
    let n = wall(&mut job, &s, &thick, "N", [10.0, 0.0], [10.0, 10.0], 10.0);
    let e = job.find(&n).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().room_bounding = false;
    wall(&mut job, &s, &t, "P", [10.0, 5.0], [20.0, 5.0], 10.0);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(180.5), Some(200.0), Some(77.0)]);
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Known);
    // N's thickness assumed, 12 in: P stops at x = 10.5, so it is 9.25 ft of a 6 in wall,
    // 4.625 SF. Floor 185.25 − 4.625 = 180.625; round 58 + 18.5 + 0.5 − 0.5 = 76.5.
    let te = job.find(&thick).unwrap();
    job.world.get_mut::<WallType>(te).unwrap().thickness = Val::assumed(Len::One(12.0), "in", "not on the wall schedule");
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(180.625), Some(200.0), Some(76.5)]);
    for name in ["floor_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!(d.status, Status::Assumed, "{name}");
        assert!(d.note.contains(&format!("is cut to N ({n}), whose line or thickness is assumed")), "{name}: {}", d.note);
        assert!(d.inputs.contains(&format!("{n}#pl::wall")), "{name}: {:?}", d.inputs);
    }
    // N's line assumed instead, at x = 11: P is cut at x = 11.25, 8.5 ft of it, 4.25 SF.
    job.world.get_mut::<WallType>(te).unwrap().thickness = Val::known(Len::One(6.0), "in", "test");
    job.world.get_mut::<Wall>(e).unwrap().line = Val::assumed([[11.0, 0.0], [11.0, 10.0]], "ft", "scaled off a sketch");
    let p = path_named(&mut job, "P");
    let pe = job.find(&p).unwrap();
    job.world.get_mut::<Wall>(pe).unwrap().line = Val::known([[11.0, 5.0], [20.0, 5.0]], "ft", "test");
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(181.0)); // 185.25 − 4.25
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Assumed);
}

#[test]
fn j10_10_a_wall_that_starts_above_a_room_does_not_flag_it_whatever_its_top() {
    use plumbline_world::{Level, Wall};
    // The box on the ground, and a storey above at 10 ft with a wall across the middle of
    // the plan whose height is assumed. Its base is Known at 10 ft: however tall it turns
    // out, it does not come down to the 4 ft the ground-floor room is found at.
    let (mut job, _s, t, r) = boxed();
    let upper = job.add("job-0001", Kind::Storey, "upper").unwrap();
    job.put(&upper, Level { elevation: ft(10.0), height: ft(9.0), computation_height: None }).unwrap();
    let above = wall(&mut job, &upper, &t, "above", [10.0, 0.0], [10.0, 10.0], 8.0);
    let e = job.find(&above).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().height = Some(Val::assumed(Len::One(8.0), "ft", "not dimensioned"));
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status, d.note.as_str()), (Some(185.25), Status::Known, ""));
    // If where it starts is itself unsure, it might come down, and the room says so,
    // naming the wall by its path as well as its name.
    job.world.get_mut::<Wall>(e).unwrap().base_offset = Some(Val::assumed(Len::One(0.0), "ft", "not dimensioned"));
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!(d.status, Status::Assumed);
    assert!(d.note.contains(&format!("its outline counts on above ({above}), whose height is")), "{}", d.note);
}

// ---- Drawing and editing (HANDOFF_DRAW.md) ----

#[test]
fn any_step_taken_back_leaves_the_job_byte_for_byte_what_it_was() {
    use plumbline_world::editing::{History, UNDO_LIMIT};
    let mut job = sample::priced();
    let mut history = History::default();
    let start = save(&mut job);
    // Nothing done: nothing to undo, and it says so.
    assert_eq!(history.undo(&mut job).unwrap_err(), "nothing to undo");
    assert_eq!(history.redo(&mut job).unwrap_err(), "nothing to redo");
    // Three steps.
    let step = |job: &mut Job, history: &mut History, label: &str, edit: &dyn Fn(&mut Job)| {
        let before = History::before(job);
        edit(job);
        job.derive();
        history.record(label, before, job)
    };
    assert!(step(&mut job, &mut history, "overhead", &|j| j.set("job-0001", "overhead", Some("12"), Status::Known, "made-up").unwrap()));
    let after_one = save(&mut job);
    assert!(step(&mut job, &mut history, "material", &|j| j.set("plane-0003", "material", Some("Copper, 20 oz"), Status::Known, "made-up").unwrap()));
    let after_two = save(&mut job);
    assert!(step(&mut job, &mut history, "labor off", &|j| j.set_off("plane-0003", Some("Install panels"), true).unwrap()));
    let after_three = save(&mut job);
    // A step that changes nothing is not a step.
    assert!(!step(&mut job, &mut history, "nothing", &|_| {}));
    assert_eq!(history.labels(false), ["overhead", "material", "labor off"]);
    // Back, one at a time: each time the job is the text it was.
    assert_eq!(history.undo(&mut job).unwrap(), "labor off");
    assert_eq!(save(&mut job), after_two);
    assert_eq!(history.undo(&mut job).unwrap(), "material");
    assert_eq!(save(&mut job), after_one);
    assert_eq!(history.labels(true), ["labor off", "material"]);
    // Forward again.
    assert_eq!(history.redo(&mut job).unwrap(), "material");
    assert_eq!(save(&mut job), after_two);
    // A new step after an undo empties what could be redone.
    assert!(step(&mut job, &mut history, "profit", &|j| j.set("job-0001", "profit", Some("8"), Status::Known, "made-up").unwrap()));
    assert_eq!(history.redo(&mut job).unwrap_err(), "nothing to redo");
    assert_ne!(save(&mut job), after_three);
    // All the way back: the job as it started, to the byte, and its figures with it.
    while history.undo(&mut job).is_ok() {}
    assert_eq!(save(&mut job), start);
    assert_eq!(view::estimate(&mut job).job_cost.value, Some(25292.43));
    // The last 200 steps are kept and the oldest dropped.
    let mut history = History::default();
    for i in 0..UNDO_LIMIT + 5 {
        let percent = format!("{}", i % 50);
        let before = History::before(&mut job);
        job.set("job-0001", "overhead", Some(&percent), Status::Known, &format!("step {i}")).unwrap();
        assert!(history.record(&format!("step {i}"), before, &mut job));
    }
    assert_eq!(history.labels(false).len(), UNDO_LIMIT);
    assert_eq!(history.labels(false)[0], "step 5");
}

#[test]
fn j11_1_a_price_over_a_job_cost_that_leaves_a_line_out_does_not_read_as_clean() {
    let mut job = priced_known();
    // The north plane's line has no class (and no price): the job cost leaves it out.
    let est = view::estimate(&mut job);
    assert_eq!(est.job_cost.left_out.len(), 1);
    assert_eq!((est.job_cost.value, est.job_cost.status), (Some(25292.43), Status::Assumed));
    assert_eq!((est.price.value, est.price.status), (Some(30047.40), Status::Assumed));
    assert!(est.price.note.contains("it leaves out 1 line") && est.price.note.contains("not of the job"), "{}", est.price.note);
    // Priced, it is still in no class, and the price still says so: $12,577.88 of roof is
    // not in it.
    job.set("plane-0005", "unit_cost", Some("30"), Status::Known, "made-up price").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!((est.price.value, est.price.status), (Some(30047.40), Status::Assumed));
    assert!(est.price.note.contains("it leaves out 1 line"));
    // A class's own total is flagged the same way when a line of the class has no figure.
    job.set_class("plane-0005", None, Class::Material).unwrap();
    job.set("plane-0005", "unit_cost", None, Status::Unknown, "not quoted").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(class_total(&est, Class::Material).total.status, Status::Assumed);
    assert_eq!(class_total(&est, Class::Fabrication).total.status, Status::Known);
    // With every line in a class and priced, the price is Known and carries no note:
    // 419.2627 x 30 = 12,577.88; job cost 37,870.31; overhead 3,787.03; profit
    // 41,657.34 x 0.08 = 3,332.5872 -> 3,332.59; price 44,989.93.
    job.set("plane-0005", "unit_cost", Some("30"), Status::Known, "made-up price").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert!(est.job_cost.left_out.is_empty());
    assert_eq!((est.price.value, est.price.status, est.price.note.as_str()), (Some(44989.93), Status::Known, ""));
}

#[test]
fn j11_2_a_half_cent_is_restored_and_0_4999_of_a_cent_is_not_made_into_one() {
    use plumbline_world::lines::cents;
    assert_eq!(cents(300_000_000.004999), 300_000_000.00);
    assert_eq!(cents(30_000_000.0049999), 30_000_000.00);
    assert_eq!(cents(300_000_000.005), 300_000_000.01);
    assert_eq!(cents(1.015), 1.02);
    assert_eq!(cents(2.675), 2.68);
    assert_eq!(cents(0.004999), 0.0);
}

#[test]
fn j11_3_a_quantity_typed_as_a_number_is_what_was_typed() {
    let mut job = sample::priced();
    // Two and a half months of a dumpster at 650.00 is 1,625.00, not three months.
    job.add_line("job-0001", "Dumpster, months", "2.5", Some(Class::General), false).unwrap();
    job.set_on_line("job-0001", "unit_cost", Some("Dumpster, months"), Some("650"), Status::Known, "made-up price").unwrap();
    job.derive();
    let f = figures(&mut job, "job-0001", "Dumpster, months");
    assert_eq!((f.quantity.value, f.quantity.note.as_str()), (Some(2.5), ""));
    assert_eq!(f.extension.unwrap().value, Some(1625.00));
    // A count worked out by a formula is still whole, and now says what it was rounded from.
    job.add_line("job-0001", "Anchors", "n / spacing", Some(Class::General), false).unwrap_err();
    job.add_line("job-0001", "Anchors", "5", Some(Class::General), false).unwrap();
    job.set_formula("job-0001", Some("Anchors"), "n / 2", &[("n".to_string(), "5".to_string())], Status::Known, "made-up").unwrap();
    job.derive();
    let f = figures(&mut job, "job-0001", "Anchors");
    assert_eq!(f.quantity.value, Some(3.0));
    assert_eq!(f.quantity.note, "2.5 rounded up to 3: a count is whole");
}

#[test]
fn j11_5_a_line_typed_onto_a_door_leaves_a_file_that_still_checks() {
    let mut job = sample::house();
    let door = job.index().into_iter().map(|(p, _)| p).filter(|p| p.starts_with("door-")).min().unwrap();
    job.add_line(&door, "Hang door", "1", None, false).unwrap();
    job.derive();
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "stale after a line on {door}: {:?}", report.stale);
    assert_eq!(save(&mut again), text);
}

#[test]
fn j11_6_a_rate_is_written_as_it_was_entered_wherever_it_is_written() {
    assert_eq!(view::rate(0.00004), "0.00004");
    assert_eq!(view::rate(0.0000004), "0.0000004");
    assert_eq!(view::rate(0.12345678), "0.12345678");
    assert_eq!(view::rate(31.5), "31.50");
    assert_eq!(view::rate(1250.0), "1,250.00");
    assert_eq!(view::rate(12.0), "12.00");
    // `show` writes a unit cost the same way.
    let mut job = sample();
    job.set("plane-0005", "unit_cost", Some("0.00004"), Status::Known, "made-up").unwrap();
    job.derive();
    let rows = view::elements(&mut job);
    let row = rows.iter().find(|e| e.path == "plane-0005").unwrap().rows.iter().find(|r| r.label.starts_with("unit cost")).unwrap().clone();
    assert!(row.text.starts_with("0.00004"), "{}", row.text);
}

#[test]
fn j11_7_a_supplier_column_gives_the_true_reason_a_line_is_not_in_it() {
    let mut job = sample::priced();
    job.set("plane-0003", "condition", None, Status::Unknown, "nobody knows").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let harbor = est.at_supplier.iter().find(|x| x.0 == "Harbor Metals").unwrap().1.clone();
    let why = harbor.left_out.iter().find(|(who, _)| who.starts_with("plane-0003")).map(|x| x.1.clone()).unwrap();
    assert!(why.contains("the element's assembly is not known") && !why.contains("has not quoted"), "{why}");
    // The line is still measured and ordered: 461.189 lb. It is what it comes to that has no figure.
    let f = figures(&mut job, "plane-0003", "Standing seam panels");
    assert_eq!((f.order.as_ref().unwrap().value, f.order.as_ref().unwrap().unit.as_str()), (Some(461.189), "lb"));
    assert_eq!(f.extension.unwrap().value, None);
    // A material named by its path is shown by its name.
    let m20 = job.index().into_iter().map(|(p, _)| p).find(|p| job.find(p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name == "Copper, 20 oz")).unwrap_or(false)).unwrap();
    job.set("plane-0004", "material", Some(&m20), Status::Known, "made-up").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let line = est.lines.iter().find(|l| l.path == "plane-0004" && l.description == "Standing seam panels").unwrap();
    assert_eq!(line.material.as_deref(), Some("Copper, 20 oz"));
}

#[test]
fn j11_8_cost_lines_go_on_what_is_built_or_on_the_job_and_nowhere_else() {
    let mut job = sample::priced();
    for kind in ["material-", "condition-"] {
        let path = job.index().into_iter().map(|(p, _)| p).filter(|p| p.starts_with(kind)).min().unwrap();
        let why = job.add_line(&path, "Freight", "1", Some(Class::General), false).unwrap_err();
        assert!(why.contains("cannot carry cost lines"), "{path}: {why}");
    }
    let mut house = sample::house();
    let t = house.index().into_iter().map(|(p, _)| p).filter(|p| p.starts_with("type-")).min().unwrap();
    assert!(house.add_line(&t, "Freight", "1", Some(Class::General), false).unwrap_err().contains("cannot carry cost lines"));
}

#[test]
fn j11_9_a_room_is_not_flagged_for_a_wall_met_at_the_far_end_of_one_of_its_walls() {
    use plumbline_world::WallType;
    // The 20 by 10 box with a partition at x = 10, the room its west half (9.5 by 9.5 =
    // 90.25 SF). Only the east wall's thickness is assumed. The south wall is cut to the
    // east wall at x = 20: ten feet from the room.
    let (mut job, s, t) = walled();
    let east_type = job.add("job-0001", Kind::Type, "east's type").unwrap();
    job.put(&east_type, WallType { thickness: Val::assumed(Len::One(6.0), "in", "not on the wall schedule"), layers: Vec::new(), footing: false }).unwrap();
    wall(&mut job, &s, &t, "south", [0.0, 0.0], [20.0, 0.0], 10.0);
    wall(&mut job, &s, &east_type, "east", [20.0, 0.0], [20.0, 10.0], 10.0);
    wall(&mut job, &s, &t, "north", [20.0, 10.0], [0.0, 10.0], 10.0);
    wall(&mut job, &s, &t, "west", [0.0, 10.0], [0.0, 0.0], 10.0);
    wall(&mut job, &s, &t, "partition", [10.0, 0.0], [10.0, 10.0], 10.0);
    let west = room(&mut job, &s, "west half", [5.0, 5.0]);
    let east = room(&mut job, &s, "east half", [15.0, 5.0]);
    job.derive();
    let d = measure_of(&mut job, &west, "floor_area");
    assert_eq!((d.value, d.status, d.note.as_str()), (Some(90.25), Status::Known, ""));
    // The east half is measured to the east wall, and is as sure as it.
    assert_eq!(measure_of(&mut job, &east, "floor_area").status, Status::Assumed);
}

#[test]
fn j11_10_to_14_the_smaller_findings() {
    // 10: an element whose only line is switched off has no detail, and is known as that.
    let mut job = sample::priced();
    job.add_line("eave-0007", "Install eave trim", "true_length", None, false).unwrap();
    job.set_on_line("eave-0007", "order", None, Some("0.1 hr/LF"), Status::Known, "made-up rate").unwrap();
    job.set_on_line("eave-0007", "unit_cost", None, Some("72"), Status::Known, "made-up rate").unwrap();
    job.set_class("eave-0007", None, Class::Labor).unwrap();
    job.set_off("eave-0007", None, true).unwrap();
    job.derive();
    let p = view::pricing(&mut job);
    let eave = p.elements.iter().find(|e| e.path == "eave-0007").unwrap();
    assert!(eave.all_off && eave.detail.is_none() && eave.rate.is_none());
    // 11: a total past what holds to the cent is not given.
    let mut job = sample::priced();
    for i in 0..4 {
        let name = format!("Big {i}");
        job.add_line("job-0001", &name, "9999", Some(Class::General), false).unwrap();
        job.set_on_line("job-0001", "unit_cost", Some(&name), Some("999999999.99"), Status::Known, "made-up").unwrap();
    }
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(class_total(&est, Class::General).total.value, None);
    assert!(class_total(&est, Class::General).total.left_out.iter().any(|(_, why)| why.contains("more than can be held to the cent")));
    assert_eq!(est.job_cost.value, None);
    // 12: one supplier, one name; one material, one name.
    let mut job = sample::priced();
    // (Call 208 point 1(b): the two refusals of a supplier written another way, held here (item 12 of `j11_10_to_14_the_smaller_findings`) by the older
    // road `Job::set_quote`, are gone with it: a supplier written another way is taken as the job's spelling, and said; that is held in
    // crates/cli/tests/call208.rs `a_suppliers_name_written_another_way_is_taken_as_the_jobs_and_said`.)
    assert!(job.add_material("copper, 16 OZ", "lb").unwrap_err().contains("already has a material called \"Copper, 16 oz\""));
    // 13: hours are not rounded up to whole hours.
    assert!(job.set_on_line("plane-0003", "order", Some("Install panels"), Some("25 SF/hr whole"), Status::Known, "x").unwrap_err().contains("hours are not bought whole"));
    // 14: a material is not bought by the hour.
    assert!(job.add_material("Zinc", "hr").unwrap_err().contains("not bought by the hour"));
    // 14: with an assumed condition, what a line comes to is assumed; how much of it
    // there is, in SF and in pounds, is not.
    job.set("plane-0003", "condition", Some("Standing seam copper, 16 oz"), Status::Assumed, "not on the schedule").unwrap();
    job.derive();
    let f = figures(&mut job, "plane-0003", "Standing seam panels");
    assert_eq!((f.quantity.status, f.order.as_ref().unwrap().status, f.extension.as_ref().unwrap().status), (Status::Known, Status::Known, Status::Assumed));
    let f = figures(&mut job, "plane-0003", "Install panels");
    assert_eq!((f.order.as_ref().unwrap().status, f.extension.as_ref().unwrap().status), (Status::Known, Status::Assumed));
}
