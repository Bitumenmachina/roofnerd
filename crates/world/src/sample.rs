//! A made-up house for demonstrations and tests. Nothing here is from a real job.
//!
//! * A hip roof over a 40 ft by 30 ft footprint, 6:12 all round: four planes, four eaves,
//!   four hips and a ridge.
//! * A porch shed roof, 12 ft by 8 ft, whose pitch is not on the plan. It is Assumed.
//! * A low-slope area with a parapet whose height is not on the plan. It is Unknown.
//!
//! No prices are entered. Every unit cost is Unknown, because nobody has given one.

use crate::components::*;
use crate::job::Job;
use crate::value::Val;

const PLAN: &str = "sample plan A-201";

pub fn sample() -> Job {
    let mut job = Job::new("Sample house (not a real job)", "plumbline sample", "sample");
    let root = "job-0001".to_string();

    // ---- Main roof: hip roof, 40 x 30, ridge running east-west ----
    let main = job.add(&root, Kind::Roof, "main-roof");
    let copper = || Condition(Val::known("Standing seam copper, 16 oz".to_string(), "", "sample finish schedule"));
    let roofing_cost = || Cost {
        code: Val::assumed(
            CostCode { scheme: "MasterFormat".into(), code: "07 61 00".into(), title: "Sheet Metal Roofing".into() },
            "",
            "primary cost code scheme not ruled on yet (QUESTIONS.md Q1)",
        ),
        lines: vec![CostLine { description: "Copper standing seam, installed".into(), quantity: "surface_area".into(), unit_cost: Val::unknown("$/SF", "no price entered") }],
    };
    let unscheduled = || WorkCode { wbs: Val::unknown("", "schedule not built") };

    let plane = |job: &mut Job, parent: &str, name: &str, pts: Vec<P2>, eave_h: f64, pitch: Val<f64>| -> String {
        let p = job.add(parent, Kind::RoofPlane, name);
        job.put(&p, Outline { points: Val::known(pts, "ft", PLAN), eave_height: Val::known(eave_h, "ft", PLAN) });
        job.put(&p, Pitch(pitch));
        p
    };
    let six = || Val::known(6.0, "in/12", PLAN);

    // Each outline starts with its eave: point 0 to point 1 is the low edge.
    let south = plane(&mut job, &main, "south", vec![[0.0, 0.0], [40.0, 0.0], [25.0, 15.0], [15.0, 15.0]], 10.0, six());
    let east = plane(&mut job, &main, "east", vec![[40.0, 0.0], [40.0, 30.0], [25.0, 15.0]], 10.0, six());
    let north = plane(&mut job, &main, "north", vec![[40.0, 30.0], [0.0, 30.0], [15.0, 15.0], [25.0, 15.0]], 10.0, six());
    let west = plane(&mut job, &main, "west", vec![[0.0, 30.0], [0.0, 0.0], [15.0, 15.0]], 10.0, six());
    for p in [&south, &east, &north, &west] {
        job.put(p, copper());
        job.put(p, roofing_cost());
        job.put(p, unscheduled());
    }

    let run = |job: &mut Job, parent: &str, kind: Kind, name: &str, a: P2, b: P2, planes: &[&String]| -> String {
        let p = job.add(parent, kind, name);
        job.put(&p, Run { ends: Val::known([a, b], "ft", PLAN), planes: planes.iter().map(|s| s.to_string()).collect() });
        p
    };
    run(&mut job, &main, Kind::Eave, "eave-south", [0.0, 0.0], [40.0, 0.0], &[&south]);
    run(&mut job, &main, Kind::Eave, "eave-east", [40.0, 0.0], [40.0, 30.0], &[&east]);
    run(&mut job, &main, Kind::Eave, "eave-north", [40.0, 30.0], [0.0, 30.0], &[&north]);
    run(&mut job, &main, Kind::Eave, "eave-west", [0.0, 30.0], [0.0, 0.0], &[&west]);
    run(&mut job, &main, Kind::Ridge, "ridge", [15.0, 15.0], [25.0, 15.0], &[&south, &north]);
    run(&mut job, &main, Kind::Hip, "hip-southwest", [0.0, 0.0], [15.0, 15.0], &[&south, &west]);
    run(&mut job, &main, Kind::Hip, "hip-southeast", [40.0, 0.0], [25.0, 15.0], &[&south, &east]);
    run(&mut job, &main, Kind::Hip, "hip-northeast", [40.0, 30.0], [25.0, 15.0], &[&north, &east]);
    run(&mut job, &main, Kind::Hip, "hip-northwest", [0.0, 30.0], [15.0, 15.0], &[&north, &west]);

    // ---- Porch: one shed plane, pitch not dimensioned ----
    let porch = job.add(&root, Kind::Roof, "porch-roof");
    let shed = plane(
        &mut job,
        &porch,
        "shed",
        vec![[44.0, 0.0], [56.0, 0.0], [56.0, 8.0], [44.0, 8.0]],
        9.0,
        Val::assumed(3.0, "in/12", "not dimensioned on the sample plan; 3:12 assumed until measured"),
    );
    job.put(&shed, copper());
    job.put(&shed, roofing_cost());
    job.put(&shed, unscheduled());
    run(&mut job, &porch, Kind::Eave, "eave", [44.0, 0.0], [56.0, 0.0], &[&shed]);
    run(&mut job, &porch, Kind::Rake, "rake-west", [44.0, 0.0], [44.0, 8.0], &[&shed]);
    run(&mut job, &porch, Kind::Rake, "rake-east", [56.0, 0.0], [56.0, 8.0], &[&shed]);
    run(&mut job, &porch, Kind::Headwall, "headwall", [44.0, 8.0], [56.0, 8.0], &[&shed]);

    // ---- Low-slope area with a parapet of unknown height ----
    let flat = job.add(&root, Kind::Roof, "low-slope-roof");
    let area = job.add(&flat, Kind::LowSlopeArea, "deck");
    job.put(&area, Outline { points: Val::known(vec![[60.0, 0.0], [80.0, 0.0], [80.0, 20.0], [60.0, 20.0]], "ft", PLAN), eave_height: Val::known(11.0, "ft", PLAN) });
    job.put(&area, Pitch(Val::known(0.25, "in/12", PLAN)));
    job.put(&area, unscheduled());
    let parapet = job.add(&flat, Kind::Parapet, "parapet-south");
    job.put(&parapet, Run { ends: Val::known([[60.0, 0.0], [80.0, 0.0]], "ft", PLAN), planes: vec![] });
    job.put(&parapet, Height(Val::unknown("ft", "parapet height not shown on the sample plan")));

    job.derive();
    job
}
