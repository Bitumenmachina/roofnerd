//! Call 234 (contract 234M): the field's measurements brought back by file. The job is the field job of `demos/field.sh`
//! (made up): two levels scaled off a plan, four Known walls, the south one carrying gypsum board at $2.40/SF, a traced
//! duct and two Known pipes across it. Figures worked by hand in STATUS.md ("Field entry"): the wall is 216 SF, $518.40,
//! price $615.86 before; with the ground level's height at 8 ft 10 1/2 in it is 213 SF, $511.20, price $607.31; with the
//! duct's underside at 7 ft 10 3/4 in pipe A clears (3/4 in) and pipe B is in the duct by 2 1/4 in, a finding.

use plumbline_world::editing::{self as ed, Entered, Reading, Top};
use plumbline_world::fieldsheet::{self, HEADINGS};
use plumbline_world::value::Status;
use plumbline_world::vif::vif;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const TRACED: Entered = Entered { status: Status::Assumed, source: "scaled off a made-up plan" };

struct Made {
    job: Job,
    ground: String,
    upper: String,
    duct: String,
}

fn made() -> Made {
    let mut job = Job::new("Verified in the field (not a real job)", "test", "test");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", TRACED).unwrap();
    let upper = ed::storey(&mut job, "upper", "9 ft", "8 ft", TRACED).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let duct_type = ed::segment_type(&mut job, "12 x 8", Kind::Duct, "12 in", Some("8 in"), KNOWN).unwrap();
    let pipe_type = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, KNOWN).unwrap();
    let walls = ed::walls(&mut job, &ground, &wall_type, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, Top::Level(&upper), KNOWN).unwrap();
    let duct = ed::runs(&mut job, &ground, Kind::Duct, &duct_type, &[[2.0, 8.0], [22.0, 8.0]], "8 ft", None, TRACED).unwrap().remove(0);
    ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[12.0, 2.0], [12.0, 14.0]], "7 ft 9 in", None, KNOWN).unwrap();
    ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[18.0, 2.0], [18.0, 14.0]], "8 ft", None, KNOWN).unwrap();
    let wall = walls[0].clone();
    job.add_line(&wall, "Gypsum board, hung and finished", "net_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Gypsum board, hung and finished"), Some("$2.40/SF"), Status::Known, "made-up plan").unwrap();
    job.set("job-0001", "overhead", Some("10"), Status::Known, "made-up plan").unwrap();
    job.set("job-0001", "profit", Some("8"), Status::Known, "made-up plan").unwrap();
    job.derive();
    Made { job, ground, upper, duct }
}

/// Columns of the sheet.
const FOUND: usize = 6;
const BY: usize = 7;
const ON: usize = 8;
const REFERENCE: usize = 9;

fn numbers(table: &[Vec<String>]) -> Vec<usize> {
    (1..=table.len()).collect()
}

fn fill(table: &mut [Vec<String>], reference: &str, found: &str) {
    let row = table.iter_mut().find(|r| r[REFERENCE] == reference).unwrap_or_else(|| panic!("no row {reference}"));
    row[FOUND] = found.to_string();
    row[BY] = "JM".to_string();
    row[ON] = "2026-11-26".to_string();
}

fn price(job: &mut Job) -> Option<f64> {
    view::estimate(job).price.value
}

fn status_of(job: &mut Job, path: &str, label: &str) -> (String, Status, String) {
    let el = view::elements(job).into_iter().find(|e| e.path == path).unwrap();
    let r = el.rows.into_iter().find(|r| r.label == label && !r.derived).unwrap();
    (r.text, r.status, r.basis)
}

#[test]
fn the_sheet_has_a_row_for_each_value_with_a_reference_of_its_own_and_none_for_a_clash() {
    let mut m = made();
    let sheet = fieldsheet::make(&mut m.job);
    let refs: Vec<&str> = sheet.rows.iter().map(|r| r.reference.as_str()).collect();
    for want in [format!("{} elevation", m.ground), format!("{} level height", m.ground), format!("{} elevation", m.upper), format!("{} line on the plan", m.duct), format!("{} type", m.duct), format!("{} height above its level", m.duct)] {
        assert!(refs.contains(&want.as_str()), "{want} is not a row: {refs:?}");
    }
    let mut once = refs.clone();
    once.sort();
    once.dedup();
    assert_eq!(once.len(), refs.len(), "each reference names one value only");
    // Nothing on it is Known: what is to be measured is what is not.
    assert!(sheet.rows.iter().all(|r| r.how_sure != Status::Known));
    // Sorted as the list is, by dollars.
    let dollars: Vec<f64> = sheet.rows.iter().filter_map(|r| r.dollars).collect();
    assert!(dollars.windows(2).all(|w| w[0] >= w[1]), "{dollars:?}");
    // The two clashes are questions and are not rows; the sheet says so.
    assert_eq!(sheet.questions_without_rows(), 2, "{:#?}", sheet.left);
    assert!(sheet.left.iter().all(|l| l.why.contains("measuring either one")), "{:#?}", sheet.left);
    let table = sheet.table();
    assert_eq!(table[0], HEADINGS.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    // A note's reference is empty: it is never read back as a measurement.
    assert!(table.iter().skip(1 + sheet.rows.len()).all(|r| r[REFERENCE].is_empty() && r[FOUND].is_empty()));
    // What the job had, and how sure, as the screen says it.
    let level = sheet.rows.iter().find(|r| r.reference == format!("{} level height", m.ground)).unwrap();
    assert_eq!((level.had.as_str(), level.how_sure), ("9 ft", Status::Assumed));
    assert!(level.to_do.starts_with("Measure its level height"), "{}", level.to_do);
    // A height of a run says what it is shot to.
    let height = sheet.rows.iter().find(|r| r.reference == format!("{} height above its level", m.duct)).unwrap();
    assert!(height.to_do.contains("bottom, center or top"), "{}", height.to_do);
}

#[test]
fn a_sheet_brought_straight_back_changes_nothing() {
    let mut m = made();
    let before = file::save(&mut m.job);
    let table = fieldsheet::make(&mut m.job).table();
    let n = fieldsheet::make(&mut m.job).rows.len();
    let brought = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "sheet.csv", false).unwrap();
    assert_eq!((brought.done.len(), brought.blank), (0, n));
    assert_eq!(file::save(&mut m.job), before);
}

/// The measurements `demos/field.sh` types, filled into the sheet.
fn fill_the_field_measurements(m: &mut Made, table: &mut [Vec<String>]) {
    fill(table, &format!("{} elevation", m.ground), "same");
    fill(table, &format!("{} level height", m.ground), "8'10\"1/2");
    fill(table, &format!("{} line on the plan", m.duct), "same");
    fill(table, &format!("{} type", m.duct), "same");
    fill(table, &format!("{} height above its level", m.duct), "bottom 7'10\"3/4");
}

/// The same five measurements typed one at a time, as `demos/field.sh` types them.
fn typed(m: &mut Made) {
    ed::measure(&mut m.job, &m.ground, "elevation", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.ground, "level height", Reading::Length("8'10\"1/2"), None, "JM with the laser").unwrap();
    ed::measure(&mut m.job, &m.duct, "line on the plan", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.duct, "type", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.duct, "height above its level", Reading::Length("7'10\"3/4"), Some(Justify::Bottom), "JM with the laser").unwrap();
}

#[test]
fn the_measurements_of_the_typed_demonstration_filled_in_come_to_what_typing_them_comes_to() {
    let mut by_sheet = made();
    let mut by_hand = made();
    assert_eq!(price(&mut by_sheet.job), Some(615.86));
    let mut table = fieldsheet::make(&mut by_sheet.job).table();
    let mut again = made();
    fill_the_field_measurements(&mut again, &mut table);
    let brought = fieldsheet::bring(&mut by_sheet.job, &table, &numbers(&table), "site visit.csv", false).unwrap();
    typed(&mut by_hand);
    assert_eq!(brought.done.len(), 5);
    // The price, the clashes and the list are what typing them leaves.
    assert_eq!(price(&mut by_sheet.job), Some(607.31));
    assert_eq!(price(&mut by_hand.job), price(&mut by_sheet.job));
    let (a, b) = (vif(&mut by_sheet.job), vif(&mut by_hand.job));
    assert_eq!((a.questions.len(), a.riding, a.riding_status), (b.questions.len(), b.riding, b.riding_status));
    let clashes = |job: &mut Job| building::clash::clashes(job, None).unwrap().found.iter().map(|c| (c.a_name.clone(), c.b_name.clone(), c.status, c.by.map(|x| (x * 12.0 * 100.0).round())) ).collect::<Vec<_>>();
    assert_eq!(clashes(&mut by_sheet.job), clashes(&mut by_hand.job));
    assert_eq!(clashes(&mut by_sheet.job).iter().filter(|c| c.2 == Status::Known).count(), 1, "one finding");
    // Each value is Known and its source names the sheet, its row, who and when.
    let (text, status, source) = status_of(&mut by_sheet.job, &by_sheet.ground, "level height");
    assert_eq!((text.as_str(), status), ("8' 10 1/2\"", Status::Known));
    assert!(source.contains("measured in the field by JM on 2026-11-26") && source.contains("site visit.csv") && source.contains("row "), "{source}");
    // What a typed measurement says is said for each row, with what followed, the price and the clashes.
    let said: Vec<String> = brought.done.iter().flat_map(|d| d.lines()).collect();
    let all = said.join("\n");
    assert!(all.contains("the price was $615.86 and is $607.31"), "{all}");
    assert!(all.contains("upper's floor is now at 8' 10 1/2\", Known; it was at 9' 0\""), "{all}");
    assert_eq!(all.matches("clash cleared").count(), 1, "{all}");
    assert_eq!(all.matches("clash confirmed, now a finding").count(), 1, "{all}");
    assert!(all.contains("in each other by 2 1/4 in"), "{all}");
    assert!(brought.closing().contains("5 values were measured"), "{}", brought.closing());
}

#[test]
fn rows_in_another_order_and_cut_down_come_back_the_same() {
    let mut a = made();
    let mut table = fieldsheet::make(&mut a.job).table();
    let mut helper = made();
    fill_the_field_measurements(&mut helper, &mut table);
    // The rows reversed, then every other unfilled row deleted.
    let head = table.remove(0);
    table.reverse();
    let mut kept: Vec<Vec<String>> = Vec::new();
    for (i, row) in table.into_iter().enumerate() {
        if !row[FOUND].is_empty() || i % 2 == 0 {
            kept.push(row);
        }
    }
    kept.insert(0, head);
    let mut b = made();
    let brought = fieldsheet::bring(&mut b.job, &kept, &numbers(&kept), "cut.csv", false).unwrap();
    assert_eq!(brought.done.len(), 5);
    assert_eq!(price(&mut b.job), Some(607.31));
    assert_eq!(status_of(&mut b.job, &b.duct, "height above its level").1, Status::Known);
    // The floor above is a Known floor plus a Known measurement however the rows lay: the rows go on in the job's order, not the sheet's.
    assert_eq!(status_of(&mut b.job, &b.upper, "elevation").1, Status::Known);
}

#[test]
fn one_figure_that_cannot_be_read_stops_everything_and_is_named_by_its_row() {
    let mut m = made();
    let before = file::save(&mut m.job);
    let mut table = fieldsheet::make(&mut m.job).table();
    fill(&mut table, &format!("{} elevation", m.ground), "same");
    fill(&mut table, &format!("{} level height", m.ground), "about nine feet");
    let row = 1 + table[1..].iter().position(|r| r[REFERENCE] == format!("{} level height", m.ground)).unwrap();
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap_err();
    assert!(why.iter().any(|w| w.starts_with(&format!("row {}", row + 1)) && w.contains("level height")), "{why:?}");
    assert!(why.last().unwrap().contains("nothing was brought in"));
    assert_eq!(file::save(&mut m.job), before, "the job is as it was");
    // Two bad rows are both named.
    fill(&mut table, &format!("{} line on the plan", m.duct), "10 ft");
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap_err();
    assert_eq!(why.iter().filter(|w| w.starts_with("row ")).count(), 2, "{why:?}");
    assert_eq!(file::save(&mut m.job), before);
}

#[test]
fn what_a_workbook_does_to_a_cell_is_refused_by_name_and_never_guessed() {
    let mut m = made();
    let before = file::save(&mut m.job);
    let mut table = fieldsheet::make(&mut m.job).table();
    // A bare number in place of text: no unit.
    fill(&mut table, &format!("{} level height", m.ground), "9");
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.xlsx", false).unwrap_err();
    assert!(why[0].contains("has no unit"), "{why:?}");
    // A date in place of a fraction.
    fill(&mut table, &format!("{} level height", m.ground), "a date or a time (day 46150)");
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.xlsx", false).unwrap_err();
    assert!(why[0].contains("turned what was written into a date"), "{why:?}");
    assert_eq!(file::save(&mut m.job), before);
    // The day a workbook kept for the date typed in On is written back as a day.
    fill(&mut table, &format!("{} level height", m.ground), "8'10\"1/2");
    table.iter_mut().find(|r| r[REFERENCE] == format!("{} level height", m.ground)).unwrap()[ON] = "a date or a time (day 46352)".to_string();
    fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.xlsx", false).unwrap();
    let (_, _, source) = status_of(&mut m.job, &m.ground, "level height");
    assert!(source.contains("on 2026-11-26"), "{source}");
}

#[test]
fn a_reference_the_job_does_not_have_and_a_row_with_no_reference_are_named() {
    let mut m = made();
    let before = file::save(&mut m.job);
    let mut table = fieldsheet::make(&mut m.job).table();
    let mut gone = table[1].clone();
    gone[REFERENCE] = "wall-9999 height".to_string();
    gone[FOUND] = "9 ft".to_string();
    table.push(gone);
    let mut none = table[1].clone();
    none[REFERENCE] = String::new();
    none[FOUND] = "9 ft".to_string();
    table.push(none);
    let mut twice = table[1].clone();
    twice[FOUND] = "same".to_string();
    table.push(twice.clone());
    table.push(twice);
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap_err();
    let all = why.join("\n");
    assert!(all.contains("the job has no wall-9999"), "{all}");
    assert!(all.contains("no reference"), "{all}");
    assert!(all.contains("measures the same value as row"), "{all}");
    assert_eq!(file::save(&mut m.job), before);
}

#[test]
fn a_value_changed_since_the_sheet_went_out_is_refused_with_both_and_taken_with_anyway() {
    let mut m = made();
    let mut table = fieldsheet::make(&mut m.job).table();
    // Somebody measures the storey's height after the sheet went out.
    ed::measure(&mut m.job, &m.ground, "level height", Reading::Length("8'11\""), None, "KT with the tape").unwrap();
    let after = file::save(&mut m.job);
    fill(&mut table, &format!("{} level height", m.ground), "8'10\"1/2");
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap_err();
    let all = why.join("\n");
    assert!(all.contains("the sheet had 9 ft, Assumed and the job has 8' 11\", Known (KT with the tape)"), "{all}");
    assert!(all.contains("measured since the sheet went out") && all.contains("without --anyway"), "{all}");
    assert_eq!(file::save(&mut m.job), after);
    // Taken with --anyway, and Known still.
    fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", true).unwrap();
    let (text, status, _) = status_of(&mut m.job, &m.ground, "level height");
    assert_eq!((text.as_str(), status), ("8' 10 1/2\"", Status::Known));
}

#[test]
fn a_reading_that_agrees_with_the_job_makes_the_value_known_at_that_figure_and_the_price_does_not_move() {
    let mut m = made();
    let mut table = fieldsheet::make(&mut m.job).table();
    fill(&mut table, &format!("{} level height", m.ground), "9 ft");
    let brought = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap();
    assert_eq!(brought.done.len(), 1);
    assert_eq!(brought.done[0].price, (Some(615.86), Some(615.86)));
    assert!(brought.done[0].lines().iter().any(|l| l.contains("the price is $615.86, as it was")), "{:?}", brought.done[0].lines());
    let (text, status, _) = status_of(&mut m.job, &m.ground, "level height");
    assert_eq!((text.as_str(), status), ("9 ft", Status::Known));
}

#[test]
fn a_second_sheet_that_measures_a_known_value_differently_is_said_with_both_and_left() {
    let mut m = made();
    let first = fieldsheet::make(&mut m.job).table();
    let mut second = first.clone();
    let mut one = first;
    fill(&mut one, &format!("{} level height", m.ground), "8'10\"1/2");
    fieldsheet::bring(&mut m.job, &one, &numbers(&one), "one.csv", false).unwrap();
    let after = file::save(&mut m.job);
    fill(&mut second, &format!("{} level height", m.ground), "8'11\"");
    let why = fieldsheet::bring(&mut m.job, &second, &numbers(&second), "two.csv", false).unwrap_err();
    let all = why.join("\n");
    assert!(all.contains("8' 10 1/2\", Known") && all.contains("8'11\""), "{all}");
    assert_eq!(file::save(&mut m.job), after);
    // A blank row of the second sheet does not set a Known value back to Assumed, and the reading that agrees is found the same again.
    let blank = fieldsheet::make(&mut m.job);
    assert!(blank.rows.iter().all(|r| r.reference != format!("{} level height", m.ground)), "a Known value is not on a new sheet");
    let mut agree = second.clone();
    fill(&mut agree, &format!("{} level height", m.ground), "8'10\"1/2");
    // Out of date (the sheet had Assumed), the figure is what the job has: the reading is the same again, and the source keeps who measured it first.
    let brought = fieldsheet::bring(&mut m.job, &agree, &numbers(&agree), "two.csv", false).unwrap();
    assert!(brought.done[0].said.contains("found the same again"), "{}", brought.done[0].said);
    let (_, status, source) = status_of(&mut m.job, &m.ground, "level height");
    assert_eq!(status, Status::Known);
    assert!(source.contains("row ") || source.contains("JM"), "{source}");
}

#[test]
fn a_value_typed_on_a_cost_line_is_a_row_and_is_measured_as_a_typed_one_is() {
    let mut m = made();
    let wall = view::elements(&mut m.job).into_iter().find(|e| e.kind == Kind::Wall).unwrap().path;
    m.job.add_line_measured(&wall, "Patch and match siding", "60 SF", Some(Class::Material), false, Status::Assumed, "walked it").unwrap();
    m.job.set_on_line(&wall, "unit_cost", Some("Patch and match siding"), Some("$3.00/SF"), Status::Known, "made-up list").unwrap();
    m.job.derive();
    let sheet = fieldsheet::make(&mut m.job);
    let row = sheet.rows.iter().find(|r| r.reference.contains("\"Patch and match siding\"")).unwrap_or_else(|| panic!("{:#?}", sheet.rows));
    assert_eq!(row.reference, format!("{wall} \"Patch and match siding\" measured"));
    assert_eq!((row.had.as_str(), row.how_sure), ("60 SF", Status::Assumed));
    let mut table = sheet.table();
    fill(&mut table, &row.reference.clone(), "72 SF");
    let before = price(&mut m.job).unwrap();
    let brought = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap();
    assert_eq!(brought.done.len(), 1);
    // 12 SF more at $3.00 is $36.00 more before overhead and profit; the price moved.
    assert!(price(&mut m.job).unwrap() > before);
    let ed_way = {
        let mut again = made();
        let wall = view::elements(&mut again.job).into_iter().find(|e| e.kind == Kind::Wall).unwrap().path;
        again.job.add_line_measured(&wall, "Patch and match siding", "60 SF", Some(Class::Material), false, Status::Assumed, "walked it").unwrap();
        again.job.set_on_line(&wall, "unit_cost", Some("Patch and match siding"), Some("$3.00/SF"), Status::Known, "made-up list").unwrap();
        again.job.derive();
        ed::measure_on_line(&mut again.job, &wall, "Patch and match siding", "measured", Reading::Length("72 SF"), "x").unwrap();
        price(&mut again.job)
    };
    assert_eq!(price(&mut m.job), ed_way);
}

#[test]
fn same_is_not_taken_for_a_value_that_changed_since_the_sheet_went_out() {
    let mut m = made();
    let mut table = fieldsheet::make(&mut m.job).table();
    ed::measure(&mut m.job, &m.ground, "level height", Reading::Length("8'11\""), None, "KT with the tape").unwrap();
    let after = file::save(&mut m.job);
    fill(&mut table, &format!("{} level height", m.ground), "same");
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap_err();
    assert!(why.join("\n").contains("the sheet had 9 ft, Assumed and the job has 8' 11\", Known"), "{why:?}");
    assert_eq!(file::save(&mut m.job), after);
}

#[test]
fn a_row_whose_stamp_says_known_and_differs_from_the_reading_needs_anyway() {
    // A sheet whose "The job has" was written over by hand to what the job has now: the value is Known, and a differing reading is a second measurement.
    let mut m = made();
    let mut table = fieldsheet::make(&mut m.job).table();
    ed::measure(&mut m.job, &m.ground, "level height", Reading::Length("8'11\""), None, "KT with the tape").unwrap();
    let after = file::save(&mut m.job);
    let row = table.iter_mut().find(|r| r[9] == format!("{} level height", m.ground)).unwrap();
    row[3] = "8' 11\"".to_string();
    row[4] = "Known".to_string();
    fill(&mut table, &format!("{} level height", m.ground), "8'10\"1/2");
    let why = fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", false).unwrap_err();
    assert!(why.join("\n").contains("A second measurement that differs is not put over it without --anyway"), "{why:?}");
    assert_eq!(file::save(&mut m.job), after);
    fieldsheet::bring(&mut m.job, &table, &numbers(&table), "s.csv", true).unwrap();
}
