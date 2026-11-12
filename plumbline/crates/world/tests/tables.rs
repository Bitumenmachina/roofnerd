//! Tables brought in and written out: a price list, an assembly library, a prior
//! estimate. Every figure here is made up.

use plumbline_world::editing::Entered;
use plumbline_world::file::{load, save};
use plumbline_world::tables::{self, Prior};
use plumbline_world::value::Status;
use plumbline_world::*;

fn rows(text: &[&[&str]]) -> Vec<Vec<String>> {
    text.iter().map(|r| r.iter().map(|c| c.to_string()).collect()).collect()
}

fn supplies(job: &mut Job) -> Vec<(String, Supply)> {
    let mut q = job.world.query::<(&Node, &Supply)>();
    let mut all: Vec<(String, Supply)> = q.iter(&job.world).map(|(n, s)| (n.name.clone(), s.clone())).collect();
    all.sort_by(|a, b| a.0.cmp(&b.0));
    all
}

#[test]
fn a_price_list_goes_out_and_comes_back_the_same() {
    let mut job = sample::priced();
    let out = tables::prices_out(&mut job);
    // Two coppers: each its own row, then a row for each supplier that has quoted it.
    assert_eq!(out.len(), 1 + (1 + 3) + (1 + 2));
    assert_eq!(out[0], tables::PRICE_HEADINGS);
    assert_eq!(out[1], ["Copper, 16 oz", "lb", "9", "", "Known", "made-up price", "1 lb/SF", "no", "Known", "its weight is its name"]);
    assert_eq!(out[2], ["Copper, 16 oz", "lb", "8.8", "Harbor Metals", "Known", "made-up quote"]);
    // Into a job with nothing in it, adding the materials: the same price list, value for
    // value, and the same table written again.
    let mut empty = Job::new("empty (not a real job)", "test", "test");
    let brought = tables::prices_in(&mut empty, &out, None, None, true, None).unwrap();
    assert_eq!((brought.rows, brought.not_used.len(), brought.same), (7, 0, 0));
    assert_eq!(supplies(&mut empty), supplies(&mut job));
    assert_eq!(tables::prices_out(&mut empty), out);
    // Brought into the job it came from, every row says what the job already has.
    let before = save(&mut job);
    let again = tables::prices_in(&mut job, &out, None, None, false, None).unwrap();
    assert_eq!((again.done.len(), again.same, again.not_used.len()), (0, 7, 0));
    assert_eq!(save(&mut job), before);
    // And the job it was brought into loads.
    assert!(load(&save(&mut empty)).is_ok());
}

#[test]
fn a_suppliers_own_list_sets_what_it_can_and_says_what_it_could_not() {
    let mut job = sample::priced();
    let list = rows(&[
        &["Tinners Yard (a made-up supplier)"],
        &["Price list"],
        &["Item", "UOM", "Price", "Lead time"],
        &["Copper, 16 oz", "lb", "$9.05", "2 weeks"],
        &["copper,20 OZ", "LB", "9.40", "3 weeks"],
        &["Zinc, 0.7 mm", "lb", "3.10", "stock"],
        &["Copper, 16 oz", "sheet", "$95.00", "2 weeks"],
        &["Lead-coated copper", "lb", "call"],
    ]);
    let how = Entered { status: Status::Known, source: "Tinners Yard price list (made up)" };
    // With no word on how sure the prices are, the list is refused and the job untouched.
    let before = save(&mut job);
    assert!(tables::prices_in(&mut job, &list, None, Some("Tinners Yard"), false, None).unwrap_err().contains("how sure"));
    assert_eq!(save(&mut job), before);
    let b = tables::prices_in(&mut job, &list, None, Some("tinners yard"), false, Some(how)).unwrap();
    assert_eq!((b.rows, b.above, b.same), (5, 2, 0));
    assert_eq!(b.done, ["Copper, 16 oz from Tinners Yard: $9.10/lb becomes $9.05/lb", "Copper, 20 oz from Tinners Yard: $9.40/lb, which Tinners Yard had not quoted"]);
    assert_eq!(
        b.not_used,
        [
            (6, "the job has no material called \"Zinc, 0.7 mm\"".to_string()),
            (7, "Copper, 16 oz: priced by the sheet; the job buys Copper, 16 oz by the lb".to_string()),
            (8, "Lead-coated copper: its price, \"call\", is not a figure".to_string()),
        ]
    );
    assert_eq!(b.columns_not_used, ["Lead time"]);
    // Every row is accounted for: set, the same, or not used.
    assert_eq!(b.done.len() + b.same + b.not_used.len(), b.rows);
    let all = supplies(&mut job);
    let quote = |material: &str| all.iter().find(|(n, _)| n == material).and_then(|(_, s)| s.quotes.get("Tinners Yard").cloned()).unwrap();
    assert_eq!((quote("Copper, 16 oz").value, quote("Copper, 20 oz").value), (Some(9.05), Some(9.4)));
    assert_eq!((quote("Copper, 20 oz").status, quote("Copper, 20 oz").source.as_str()), (Status::Known, "Tinners Yard price list (made up)"));
    // The 20 oz copper's lines are now priced at that supplier: the headwall's 30 lb at
    // $9.40 is $282.00.
    let est = view::estimate(&mut job);
    let headwall = est.lines.iter().find(|l| l.description == "Headwall flashing").unwrap();
    assert_eq!(headwall.quotes.get("Tinners Yard").and_then(|d| d.value), Some(282.0));
    // The same list again: nothing to set.
    let again = tables::prices_in(&mut job, &list, None, Some("Tinners Yard"), false, Some(how)).unwrap();
    assert_eq!((again.done.len(), again.same, again.not_used.len()), (0, 2, 3));

    // What is refused whole: no headings, no price column, no unit column.
    let mut job = sample::priced();
    let before = save(&mut job);
    for (bad, says) in [
        (rows(&[&["just", "some", "cells"], &["and", "more"]]), "not a price list"),
        (rows(&[&["Item", "UOM"], &["Copper, 16 oz", "lb"]]), "not a price list"),
        (rows(&[&["Item", "Price"], &["Copper, 16 oz", "9.05"]]), "no column for what each thing is bought by"),
    ] {
        let why = tables::prices_in(&mut job, &bad, None, None, false, Some(how)).unwrap_err();
        assert!(why.contains(says), "{why}");
        assert_eq!(save(&mut job), before);
    }
}

#[test]
fn an_assembly_library_goes_out_and_comes_back_the_same() {
    let mut job = sample::priced();
    let out = tables::assemblies_out(&mut job);
    let recipes = |job: &mut Job| -> Vec<(String, Recipe)> {
        let mut q = job.world.query::<(&Node, &Recipe)>();
        q.iter(&job.world).map(|(n, r)| (n.name.clone(), r.clone())).collect()
    };
    let had = recipes(&mut job);
    assert!(!had.is_empty());
    let mut empty = Job::new("empty (not a real job)", "test", "test");
    let brought = tables::assemblies_in(&mut empty, &out, None).unwrap();
    assert_eq!((brought.not_used.len(), brought.same, brought.done.len()), (0, 0, had.len()), "{:?}", brought.not_used);
    assert_eq!(recipes(&mut empty), had);
    assert_eq!(tables::assemblies_out(&mut empty), out);
    assert!(load(&save(&mut empty)).is_ok());
    // Into the job they came from: every row is the same, and nothing changes.
    let before = save(&mut job);
    let again = tables::assemblies_in(&mut job, &out, None).unwrap();
    assert_eq!((again.done.len(), again.same, again.not_used.len()), (0, out.len() - 1, 0));
    assert_eq!(save(&mut job), before);
    // One whose name the job has, with a line changed: left as it is, and said.
    let mut changed = out.clone();
    let line = 1 + changed.iter().skip(1).position(|r| r.len() > 9 && !r[9].is_empty()).unwrap();
    changed[line][9] = format!("({}) * 2", changed[line][9]);
    let said = tables::assemblies_in(&mut job, &changed, None).unwrap();
    assert!(said.not_used.iter().any(|(_, why)| why.contains("already has an assembly called") && why.contains("left as it is")), "{:?}", said.not_used);
    assert_eq!(save(&mut job), before);
    // A table that is not one of these is refused.
    assert!(tables::assemblies_in(&mut job, &rows(&[&["Item", "Price"], &["x", "1"]]), None).unwrap_err().contains("not an assembly library this program wrote"));
}

#[test]
fn a_prior_estimate_is_set_against_the_job_and_is_no_part_of_its_price() {
    let mut job = sample::priced();
    let cost_before = view::estimate(&mut job).job_cost.value;
    let prior = rows(&[
        &["Description", "Quantity", "Unit", "Unit cost", "Amount", "Notes"],
        &["Standing seam panels", "700", "lb", "8.50", "5950.00"],
        &["Underlayment", "650", "SF", "0.90", "585.00"],
        &["Gutters", "120", "LF", "14.00"],
        &["Dumpster", "1", "EA", "450.00", "$450.00"],
        &["ROOFING"],
    ]);
    let how = Entered { status: Status::Assumed, source: "a made-up earlier estimate" };
    let (path, b) = tables::prior_in(&mut job, "Earlier estimate", &prior, None, how).unwrap();
    assert_eq!((b.rows, b.columns_not_used.clone(), b.not_used.clone()), (5, Vec::<String>::new(), vec![(6, "ROOFING: it has no figure at all (a heading or a note)".to_string())]));
    // (Its Notes column is kept with each line since the seventeenth judgment, J17B-25.)
    // It is in the job, under the job, flagged as it was brought; the job's cost is what
    // it was, and the file loads and saves to the same bytes.
    assert_eq!(job.kind(&path), Some(Kind::Prior));
    assert_eq!(view::estimate(&mut job).job_cost.value, cost_before);
    let text = save(&mut job);
    let (mut back, _) = load(&text).unwrap();
    assert_eq!(save(&mut back), text);
    let e = back.find(&path).unwrap();
    assert_eq!(back.world.get::<Prior>(e).unwrap().lines.status, Status::Assumed);

    let all = tables::compare(&mut job);
    assert_eq!(all.len(), 1);
    let c = &all[0];
    assert_eq!((c.name.as_str(), c.status, c.lines.len()), ("Earlier estimate", Status::Assumed, 4));
    // Its lines add to 5,950 + 585 + 120 x 14 (worked: it gave no amount) + 450 = 8,665.
    assert_eq!((c.prior_total, c.prior_left_out), (8665.0, 0));
    assert!(c.lines[2].worked && c.lines[2].prior_amount == Some(1680.0));
    // Standing seam panels: the job has three lines called that, on the south, the east
    // and the shed: $4,150.70 + $2,490.42 + $1,113.24 = $7,754.36, in pounds, and no
    // surer than the shed's, which is Assumed.
    let panels = c.lines[0].job.as_ref().unwrap();
    assert_eq!((panels.on.len(), panels.unit.as_str(), panels.amount, panels.status), (3, "lb", Some(7754.36), Status::Assumed));
    let est = view::estimate(&mut job);
    let pounds: f64 = est.lines.iter().filter(|l| l.description == "Standing seam panels").filter_map(|l| l.priced.value).sum();
    assert_eq!(panels.quantity, Some(pounds));
    // Underlayment: two lines, $377.34 + $226.40 = $603.74.
    let under = c.lines[1].job.as_ref().unwrap();
    assert_eq!((under.on.len(), under.unit.as_str(), under.amount, under.status), (2, "SF", Some(603.74), Status::Known));
    // Gutters and the dumpster are in no line of the job.
    assert!(c.lines[2].job.is_none() && c.lines[3].job.is_none());
    // What the job has that the prior estimate does not name: seven kinds of line.
    let only: Vec<&str> = c.only_in_job.iter().map(|(d, ..)| d.as_str()).collect();
    assert_eq!(only, ["Foreman, days on site", "Fabricate panels", "Install panels", "Copper standing seam, installed", "Standing seam roofing, by subcontractor", "Headwall flashing", "Install headwall flashing"]);
    let install = c.only_in_job.iter().find(|(d, ..)| d == "Install panels").unwrap();
    assert_eq!((install.1.len(), install.2), (2, Some(1931.97)));
    // The line with no price has no figure, and that is not a zero.
    assert_eq!(c.only_in_job.iter().find(|(d, ..)| d == "Copper standing seam, installed").unwrap().2, None);

    // Refused: an estimate with no amounts and no way to work them, and one with no source.
    let before = save(&mut job);
    assert!(tables::prior_in(&mut job, "x", &rows(&[&["Description", "Quantity"], &["Gutters", "120"]]), None, how).unwrap_err().contains("no column for what each line comes to"));
    assert!(tables::prior_in(&mut job, "x", &prior, None, Entered { status: Status::Assumed, source: " " }).is_err());
    assert!(tables::prior_in(&mut job, "Earlier estimate", &prior, None, how).is_err(), "a second of the same name");
    assert_eq!(save(&mut job), before);
    // A prior estimate cannot carry cost lines of the job's.
    assert!(job.add_line(&path, "X", "5", Some(Class::General), false).is_err());
}

// ---- The fifteenth judgment, part A: what the judge broke, held here.

#[test]
fn j15_1_a_comma_is_a_thousands_mark_or_the_figure_is_not_read() {
    let mut job = sample::priced();
    let how = Entered { status: Status::Assumed, source: "a made-up list" };
    // Nine and a bit, as a list from another country writes it; twelve hundred, the same.
    let list = rows(&[&["Item", "UOM", "Price"], &["Copper, 16 oz", "lb", "9,05"], &["Copper, 20 oz", "lb", "1.204,50"], &["Copper, 16 oz", "lb", "1,204.50"]]);
    let before = supplies(&mut job);
    let b = tables::prices_in(&mut job, &list, None, Some("Tinners Yard"), false, Some(how)).unwrap();
    assert_eq!(b.not_used.len(), 2, "{:?}", b.not_used);
    assert!(b.not_used[0].1.contains("\"9,05\" has a comma that does not part thousands") && b.not_used[0].1.contains("write a point"), "{:?}", b.not_used);
    assert!(b.not_used[1].1.contains("\"1.204,50\""), "{:?}", b.not_used);
    // The one written with a thousands mark is read as twelve hundred and four fifty.
    assert_eq!(b.done, ["Copper, 16 oz from Tinners Yard: $9.10/lb becomes $1,204.50/lb"]);
    let after = supplies(&mut job);
    assert_eq!(after.iter().filter(|(n, _)| n == "Copper, 20 oz").collect::<Vec<_>>(), before.iter().filter(|(n, _)| n == "Copper, 20 oz").collect::<Vec<_>>());
    // What a comma may and may not be.
    for (typed, ok) in [("1,204", true), ("12,345,678.9", true), ("$1,204.50", true), ("1,20", false), ("1,2045", false), ("9,05", false), (",5", false), ("1,204,5", false), ("1,204.5,0", false), ("1.204,50", false)] {
        let list = rows(&[&["Item", "UOM", "Price"], &["Copper, 16 oz", "lb", typed]]);
        let b = tables::prices_in(&mut sample::priced(), &list, None, None, false, Some(how)).unwrap();
        assert_eq!(b.not_used.is_empty(), ok, "{typed}: {:?}", b.not_used);
    }
}

#[test]
fn j15_2_13_20_whose_word_stands_and_what_a_row_cannot_say() {
    let how = Entered { status: Status::Assumed, source: "their flyer, as I read it" };
    // The list calls its own price Known. Whoever brings it in says Assumed: that stands,
    // and the result says the list's columns were not read.
    let mut job = sample::priced();
    let list = rows(&[&["Material", "Bought by", "Price", "How sure", "Where from"], &["Copper, 16 oz", "lb", "7.77", "Known", "their flyer"]]);
    let b = tables::prices_in(&mut job, &list, None, None, false, Some(how)).unwrap();
    assert!(b.notes.iter().any(|n| n.contains("were not read") && n.contains("Assumed, their flyer, as I read it")), "{:?}", b.notes);
    let copper = supplies(&mut job).into_iter().find(|(n, _)| n == "Copper, 16 oz").unwrap().1;
    assert_eq!((copper.price.value, copper.price.status, copper.price.source.as_str()), (Some(7.77), Status::Assumed, "their flyer, as I read it"));
    // With nobody saying, the list's own words are read, as before.
    let mut job = sample::priced();
    tables::prices_in(&mut job, &list, None, None, false, None).unwrap();
    let copper = supplies(&mut job).into_iter().find(|(n, _)| n == "Copper, 16 oz").unwrap().1;
    assert_eq!((copper.price.status, copper.price.source.as_str()), (Status::Known, "their flyer"));
    // A supplier's row that says how the material is ordered: its price is read, and
    // the rest is said not to have been.
    let mut job = sample::priced();
    let list = rows(&[&["Material", "Bought by", "Price", "Supplier", "Ordered at", "Whole ones only"], &["Copper, 16 oz", "lb", "8.80", "Harbor Metals", "1.5 lb/SF", "yes"]]);
    let b = tables::prices_in(&mut job, &list, None, None, false, Some(how)).unwrap();
    assert!(b.notes.iter().any(|n| n.starts_with("row 2: Copper, 16 oz: how it is ordered (1.5 lb/SF) was not read")), "{:?}", b.notes);
    // A part number beside a description: the description is the material.
    let mut job = sample::priced();
    let list = rows(&[&["Item", "Description", "UOM", "Price"], &["10045", "Copper, 16 oz", "lb", "9.05"]]);
    let b = tables::prices_in(&mut job, &list, None, Some("Tinners Yard"), true, Some(how)).unwrap();
    assert_eq!(b.columns_not_used, ["Item"]);
    assert_eq!((b.done.len(), b.not_used.len()), (1, 0), "{:?} {:?}", b.done, b.not_used);
    assert!(b.done[0].starts_with("Copper, 16 oz from Tinners Yard"), "{:?}", b.done);
    // One thing priced twice in one list at two prices: the second is not used.
    let mut job = sample::priced();
    let list = rows(&[&["Item", "UOM", "Price"], &["Copper, 16 oz", "lb", "9.05"], &["copper 16 oz", "LBS", "9.50"], &["Copper, 16 oz", "lb.", "9.05"]]);
    let b = tables::prices_in(&mut job, &list, None, Some("Tinners Yard"), false, Some(how)).unwrap();
    assert_eq!(b.not_used, [(3, "copper 16 oz: row 2 of this list already prices it, at $9.05/lb; this row says $9.50/lb, and was not used".to_string())]);
    assert_eq!((b.done.len(), b.same), (1, 1));
}

#[test]
fn j15_10_a_row_is_called_by_its_number_in_the_file() {
    // The file's rows 1, 4, 6 and 9: the rest were blank, or above the sheet's first cell.
    let list = rows(&[&["Item", "UOM", "Price"], &["Copper, 16 oz", "lb", "call"], &["Nothing the job has", "lb", "1"], &["", "lb", "1"]]);
    let how = Entered { status: Status::Assumed, source: "a made-up list" };
    let b = tables::prices_in(&mut sample::priced(), &list, Some(&[1, 4, 6, 9]), None, false, Some(how)).unwrap();
    assert_eq!(b.not_used.iter().map(|n| n.0).collect::<Vec<_>>(), [4, 6, 9]);
}

#[test]
fn j15_5_6_7_a_prior_estimates_totals_extensions_and_lines_of_one_name() {
    let mut job = sample::priced();
    let how = Entered { status: Status::Assumed, source: "a made-up estimate" };
    let prior = rows(&[
        &["Description", "Qty", "Unit", "Unit cost", "Amount"],
        &["Standing seam panels", "300", "lb", "9.00", "2,700.00"],
        &["Standing seam panels", "400", "lb", "9.00", "3,600.00"],
        &["Gutters", "700", "LF", "8.50", "9,999.00"],
        &["Subtotal", "", "", "", "16,299.00"],
        &["Total station hire", "2", "day", "150", "300.00"],
        &["TOTAL:", "", "", "", "16,599.00"],
        &["Total of the above", "", "", "", "16,599.00"],
    ]);
    let (_, b) = tables::prior_in(&mut job, "Earlier estimate", &prior, None, how).unwrap();
    // The three totals are not lines; a line that begins with the word and has a
    // quantity is one.
    assert_eq!(b.not_used.iter().map(|n| n.0).collect::<Vec<_>>(), [5, 7, 8], "{:?}", b.not_used);
    assert!(b.not_used.iter().all(|n| n.1.contains("a total of rows above it")), "{:?}", b.not_used);
    // 700 at $8.50 is $5,950.00, not $9,999.00: kept as written, and said.
    assert_eq!(b.notes, ["row 4: Gutters: its amount, $9,999.00, is not its quantity times its unit cost (700.00 × $8.50 = $5,950.00); the amount was kept as written"]);
    let c = tables::compare(&mut job).remove(0);
    // 2,700 + 3,600 + 9,999 + 300.
    assert_eq!((c.prior_total, c.prior_left_out), (16_599.0, 0));
    // The two lines of one name are one line here: 700 lb, $6,300.00, set once against
    // the job's lines of that name.
    assert_eq!(c.lines.len(), 3);
    let panels = &c.lines[0];
    assert_eq!((panels.of, panels.prior.quantity, panels.prior_amount, panels.prior.unit_cost), (2, Some(700.0), Some(6_300.0), Some(9.0)));
    assert!(panels.job.is_some());
}

#[test]
fn j15_14_two_assemblies_under_one_name_are_not_made_one() {
    let mut job = sample::priced();
    let out = tables::assemblies_out(&mut job);
    let name = out[1][0].clone();
    let mine: Vec<Vec<String>> = out.iter().filter(|r| r[0] == name || r[0] == "Assembly").cloned().collect();
    assert!(mine.len() >= 2, "the sample has an assembly to try this on");
    // A second row of the same name that goes on something else.
    let mut two = mine.clone();
    let mut other = mine[1].clone();
    other[1] = if other[1].contains("Eave") { "Rake".to_string() } else { "Eave".to_string() };
    two.push(other);
    let mut empty = Job::new("empty (not a real job)", "test", "test");
    let b = tables::assemblies_in(&mut empty, &two, None).unwrap();
    assert!(b.done.is_empty(), "{:?}", b.done);
    assert!(b.not_used.iter().all(|n| n.1.contains("do not agree on what it goes on")), "{:?}", b.not_used);
    assert_eq!(b.not_used.len(), two.len() - 1);
}

#[test]
fn j15_23_no_spelling_of_time_is_what_a_material_is_ordered_by() {
    for unit in ["workhour", "crew-day", "labor hour", "crew hr", "LH", "manweek", "MH", "man-hours", "hr", "day", "crewday", "work hours", "wk", "month", "person-hours", "min"] {
        assert!(check::is_hours(unit), "{unit} is a measure of time");
    }
    for unit in ["lb", "sheet", "SF", "LF", "EA", "SQ", "roll", "box", "M", "bundle", "pail", "gal", "tube", "each", "inch", "board", "third", "piece", "length", "cwt", "CY", "ton"] {
        assert!(!check::is_hours(unit), "{unit} is not a measure of time");
    }
}
