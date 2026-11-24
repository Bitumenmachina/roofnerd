//! Call 199: an assembly changed for another writes the quantities of the one it becomes. A wall 20 ft long and
//! 10 ft high with no opening and two outside corners, the starter assemblies. The figures are worked by hand in
//! STATUS.md ("An assembly changed for another: a demonstration").
//!
//! 1. Plain lap-siding wall: Lap siding 20 x 10 = 200.00 SF, Corner boards 2 x 10 = 20.00 LF; the siding at a
//!    made-up $4.00/SF is $800.00.
//! 2. Changed for the wall over a stone wainscot: Lap siding 200 - 20 x 3 = 140.00 SF, still $4.00: $560.00;
//!    Hang the siding 140.00 SF; Corner boards 2 x (10 - 3) = 14.00 LF.
//! 3. Changed back: 200.00 SF, $800.00, 20.00 LF.
//! 4. The siding's quantity typed as net_area * 0.9 (180.00 SF) stays through the change.

use plumbline_world::editing as ed;
use plumbline_world::tables;
use plumbline_world::value::Status;
use plumbline_world::view::estimate;
use plumbline_world::*;

const PLAIN: &str = "Exterior wall, 2x6, lap siding";
const WAINSCOT: &str = "Exterior wall, 2x6, lap siding over a stone wainscot";

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up plan" }
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

/// The wall, priced at a made-up $4.00/SF on its siding, with the plain assembly on it.
fn wall() -> (Job, String) {
    let mut job = Job::new("A wall (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "10 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    // A ring 20 ft by 16 ft: its first wall is 20 ft long and 10 ft high, with no opening and two outside corners.
    let wall = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [20.0, 0.0], [20.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("10 ft"), known()).unwrap().remove(0);
    tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    job.apply(&wall, PLAIN, "made-up").unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Lap siding"), Some("$4.00/SF"), Status::Known, "made-up price").unwrap();
    job.derive();
    (job, wall)
}

fn quantity(job: &mut Job, wall: &str, line: &str) -> (Option<f64>, String) {
    let l = estimate(job).lines.into_iter().find(|l| l.path == wall && l.description == line).unwrap_or_else(|| panic!("no line {line}"));
    (l.quantity.value.map(|v| (v * 100.0).round() / 100.0), l.quantity.unit.to_string())
}

fn amount(job: &mut Job, wall: &str, line: &str) -> Option<f64> {
    estimate(job).lines.into_iter().find(|l| l.path == wall && l.description == line).and_then(|l| l.amount.value).map(|v| (v * 100.0).round() / 100.0)
}

#[test]
fn a_plain_wall_changed_for_the_wainscot_wall_writes_the_new_quantities_and_keeps_the_price() {
    let (mut job, wall) = wall();
    assert_eq!(quantity(&mut job, &wall, "Lap siding"), (Some(200.0), "SF".into()));
    assert_eq!(quantity(&mut job, &wall, "Corner boards"), (Some(20.0), "LF".into()));
    assert_eq!(amount(&mut job, &wall, "Lap siding"), Some(800.0));
    // 2. The wall over a stone wainscot.
    let changed = job.change_assembly_said(&wall, WAINSCOT, Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(quantity(&mut job, &wall, "Lap siding"), (Some(140.0), "SF".into()));
    assert_eq!(quantity(&mut job, &wall, "Hang the siding"), (Some(140.0), "SF".into()));
    assert_eq!(quantity(&mut job, &wall, "Corner boards"), (Some(14.0), "LF".into()));
    assert_eq!(amount(&mut job, &wall, "Lap siding"), Some(560.0), "the price stays: $4.00 on 140 SF");
    // The reply names each line whose quantity changed, with the old and the new.
    let named: Vec<(&str, &str, &str)> = changed.rewritten.iter().map(|r| (r.line.as_str(), r.was.as_str(), r.now.as_str())).collect();
    assert!(named.contains(&("Lap siding", "200.00 SF", "140.00 SF")), "{named:?}");
    assert!(named.contains(&("Hang the siding", "200.00 SF", "140.00 SF")), "{named:?}");
    assert!(named.contains(&("Corner boards", "20.00 LF", "14.00 LF")), "{named:?}");
    assert_eq!(named.len(), 3, "{named:?}");
    assert!(changed.added.iter().any(|a| a.starts_with("Stone veneer wainscot")), "{:?}", changed.added);
    assert!(changed.left.is_empty(), "{:?}", changed.left);
    // What the formula and the entry now are.
    let siding = changed.rewritten.iter().find(|r| r.line == "Lap siding").unwrap();
    assert_eq!(siding.now_written, "net_area - net_area_below_wainscot + gable_area, wainscot = 3 ft");
    // 3. Changed back.
    let back = job.change_assembly_said(&wall, PLAIN, Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(quantity(&mut job, &wall, "Lap siding"), (Some(200.0), "SF".into()));
    assert_eq!(quantity(&mut job, &wall, "Corner boards"), (Some(20.0), "LF".into()));
    assert_eq!(amount(&mut job, &wall, "Lap siding"), Some(800.0));
    assert!(back.taken.iter().any(|t| t.starts_with("Stone veneer wainscot")), "{:?}", back.taken);
    assert_eq!(back.rewritten.len(), 3);
    // The entry the new assembly's formula took is not left on a line that no longer uses it.
    let e = job.find(&wall).unwrap();
    let line = job.world.get::<Cost>(e).unwrap().lines.iter().find(|l| l.description == "Lap siding").cloned().unwrap();
    assert!(line.inputs.is_empty(), "{:?}", line.inputs);
}

#[test]
fn a_quantity_a_person_changed_stays_and_the_reply_says_what_the_new_assembly_would_have_written() {
    let (mut job, wall) = wall();
    job.set_on_line(&wall, "quantity", Some("Lap siding"), Some("net_area * 0.9"), Status::Known, "made-up estimator").unwrap();
    job.derive();
    assert_eq!(quantity(&mut job, &wall, "Lap siding"), (Some(180.0), "SF".into()));
    let changed = job.change_assembly_said(&wall, WAINSCOT, Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(quantity(&mut job, &wall, "Lap siding"), (Some(180.0), "SF".into()), "left as entered");
    assert_eq!(quantity(&mut job, &wall, "Corner boards"), (Some(14.0), "LF".into()), "an untouched line follows");
    let left = changed.left.iter().find(|l| l.line == "Lap siding").unwrap_or_else(|| panic!("{:?}", changed.left));
    assert_eq!((left.has.as_str(), left.would.as_str()), ("180.00 SF", "140.00 SF"));
    assert_eq!(left.has_written, "net_area * 0.9");
    assert_eq!(left.would_written, "net_area - net_area_below_wainscot + gable_area, wainscot = 3 ft");
    assert_eq!(left.why, plumbline_world::LeftAsIs::Entered);
    assert!(changed.rewritten.iter().all(|r| r.line != "Lap siding"));
    assert_eq!(amount(&mut job, &wall, "Lap siding"), Some(720.0), "its price stays: $4.00 on 180 SF");
}

#[test]
fn where_the_old_assembly_is_no_longer_in_the_job_a_line_stays_and_the_reply_says_so() {
    let (mut job, wall) = wall();
    // The job as it was saved by a program that kept no assembly under that name: the wall says it has one the job does not.
    let gone = Condition(Val { value: Some("An assembly no longer in this job".into()), unit: String::new(), status: Status::Known, source: "made-up".into() });
    job.put(&wall, gone).unwrap();
    job.derive();
    let changed = job.change_assembly_said(&wall, WAINSCOT, Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(quantity(&mut job, &wall, "Lap siding"), (Some(200.0), "SF".into()), "its lines cannot be compared: it stays");
    let left = changed.left.iter().find(|l| l.line == "Lap siding").unwrap_or_else(|| panic!("{:?}", changed.left));
    assert_eq!(left.why, plumbline_world::LeftAsIs::OldAssemblyGone);
    assert_eq!(left.would, "140.00 SF");
    assert!(changed.rewritten.is_empty(), "{:?}", changed.rewritten);
}

#[test]
fn what_a_person_entered_on_a_shared_line_but_its_quantity_stays_as_it_was() {
    let (mut job, wall) = wall();
    job.set_on_line(&wall, "waste", Some("Lap siding"), Some("10"), Status::Known, "made-up").unwrap();
    job.set_on_line(&wall, "source_note_unused", Some("Lap siding"), None, Status::Known, "made-up").ok();
    job.set_off(&wall, Some("Hang the siding"), true).unwrap();
    job.derive();
    let at = job.find(&wall).unwrap();
    let before = job.world.get::<Cost>(at).unwrap().lines.iter().find(|l| l.description == "Lap siding").cloned().unwrap();
    job.change_assembly_said(&wall, WAINSCOT, Status::Known, "made-up").unwrap();
    let after = job.world.get::<Cost>(at).unwrap().lines.iter().find(|l| l.description == "Lap siding").cloned().unwrap();
    assert_eq!((&before.unit_cost, &before.waste, &before.order, before.off, before.class, before.division), (&after.unit_cost, &after.waste, &after.order, after.off, after.class, after.division));
    assert_ne!(before.quantity, after.quantity);
    let off = |job: &mut Job| job.world.get::<Cost>(at).unwrap().lines.iter().find(|l| l.description == "Hang the siding").map(|l| l.off);
    assert_eq!(off(&mut job), Some(true), "a labor line switched off stays off");
}
