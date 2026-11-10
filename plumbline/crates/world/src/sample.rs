//! A made-up house for demonstrations and tests. Nothing here is from a real job.
//!
//! * A hip roof over a 40 ft by 30 ft footprint, 6:12 all round: four planes, four eaves,
//!   four hips and a ridge.
//! * A porch shed roof, 12 ft by 8 ft, whose pitch is not on the plan. It is Assumed.
//!   Its headwall carries a sheet-metal flashing line with a 24 in stretch-out.
//! * A low-slope area with a parapet whose height is not on the plan. It is Unknown.
//!
//! * Three conditions as recipes: "Parapet with copper coping" (four lines), "Copper
//!   flashing" (one) and "Standing seam copper roof" (two). None is applied to anything
//!   in the sample.
//!
//! No prices are entered. Every unit cost is Unknown, because nobody has given one.

use crate::components::*;
use crate::job::Job;
use crate::units::Len;
use crate::value::Val;

const PLAN: &str = "sample plan A-201";
/// The sample is fixed data; a failure here is a bug in this file.
const OK: &str = "the sample job is well formed";

pub fn sample() -> Job {
    let mut job = Job::new("Sample house (not a real job)", "plumbline sample", "sample");
    let root = "job-0001".to_string();

    // ---- Main roof: hip roof, 40 x 30, ridge running east-west ----
    let main = job.add(&root, Kind::Roof, "main-roof").expect(OK);
    let copper = || Condition(Val::known("Standing seam copper, 16 oz".to_string(), "", "sample finish schedule"));
    let roofing_cost = || Cost {
        code: Val::assumed(
            CostCode { scheme: "MasterFormat".into(), code: "07 61 00".into(), title: "Sheet Metal Roofing".into() },
            "",
            "primary cost code scheme not ruled on yet (QUESTIONS.md Q1)",
        ),
        lines: vec![CostLine::of("Copper standing seam, installed", "surface_area", Val::unknown("$/SF", "no price entered"))],
    };
    let unscheduled = || WorkCode { wbs: Val::unknown("", "schedule not built") };

    let plane = |job: &mut Job, parent: &str, name: &str, pts: Vec<P2>, eave_h: f64, pitch: Val<f64>| -> String {
        let p = job.add(parent, Kind::RoofPlane, name).expect(OK);
        // Every outline here is traced starting along its eave.
        let eave = [pts[0], pts[1]];
        job.put(&p, Outline { points: Val::known(pts, "ft", PLAN), eave: Val::known(eave, "ft", PLAN), eave_height: Val::known(Len::One(eave_h), "ft", PLAN) }).expect(OK);
        job.put(&p, Pitch(pitch)).expect(OK);
        p
    };
    let six = || Val::known(6.0, "in/12", PLAN);

    // Each outline starts with its eave: point 0 to point 1 is the low edge.
    let south = plane(&mut job, &main, "south", vec![[0.0, 0.0], [40.0, 0.0], [25.0, 15.0], [15.0, 15.0]], 10.0, six());
    let east = plane(&mut job, &main, "east", vec![[40.0, 0.0], [40.0, 30.0], [25.0, 15.0]], 10.0, six());
    let north = plane(&mut job, &main, "north", vec![[40.0, 30.0], [0.0, 30.0], [15.0, 15.0], [25.0, 15.0]], 10.0, six());
    let west = plane(&mut job, &main, "west", vec![[0.0, 30.0], [0.0, 0.0], [15.0, 15.0]], 10.0, six());
    for p in [&south, &east, &north, &west] {
        job.put(p, copper()).expect(OK);
        job.put(p, roofing_cost()).expect(OK);
        job.put(p, unscheduled()).expect(OK);
    }

    let run = |job: &mut Job, parent: &str, kind: Kind, name: &str, a: P2, b: P2, planes: &[&String]| -> String {
        let p = job.add(parent, kind, name).expect(OK);
        job.put(&p, Run { ends: Val::known([a, b], "ft", PLAN), planes: planes.iter().map(|s| s.to_string()).collect() }).expect(OK);
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
    let porch = job.add(&root, Kind::Roof, "porch-roof").expect(OK);
    let shed = plane(
        &mut job,
        &porch,
        "shed",
        vec![[44.0, 0.0], [56.0, 0.0], [56.0, 8.0], [44.0, 8.0]],
        9.0,
        Val::assumed(3.0, "in/12", "not dimensioned on the sample plan; 3:12 assumed until measured"),
    );
    job.put(&shed, copper()).expect(OK);
    job.put(&shed, roofing_cost()).expect(OK);
    job.put(&shed, unscheduled()).expect(OK);
    run(&mut job, &porch, Kind::Eave, "eave", [44.0, 0.0], [56.0, 0.0], &[&shed]);
    run(&mut job, &porch, Kind::Rake, "rake-west", [44.0, 0.0], [44.0, 8.0], &[&shed]);
    run(&mut job, &porch, Kind::Rake, "rake-east", [56.0, 0.0], [56.0, 8.0], &[&shed]);
    let headwall = run(&mut job, &porch, Kind::Headwall, "headwall", [44.0, 8.0], [56.0, 8.0], &[&shed]);
    // The headwall flashing is sheet metal: priced by its area, which is the headwall's
    // length times the flashing's stretch-out. The detail gives the stretch-out in inches.
    job.put(
        &headwall,
        Cost {
            code: Val::assumed(
                CostCode { scheme: "MasterFormat".into(), code: "07 62 00".into(), title: "Sheet Metal Flashing and Trim".into() },
                "",
                "primary cost code scheme not ruled on yet (QUESTIONS.md Q1)",
            ),
            lines: vec![CostLine {
                stretch_out: Some(Val::known(Len::One(24.0), "in", "sample detail 4/A-501")),
                ..CostLine::of("Copper headwall flashing", "true_length * stretch_out", Val::unknown("$/SF", "no price entered"))
            }],
        },
    )
    .expect(OK);

    // ---- Low-slope area with a parapet of unknown height ----
    let flat = job.add(&root, Kind::Roof, "low-slope-roof").expect(OK);
    let area = job.add(&flat, Kind::LowSlopeArea, "deck").expect(OK);
    job.put(
        &area,
        Outline {
            points: Val::known(vec![[60.0, 0.0], [80.0, 0.0], [80.0, 20.0], [60.0, 20.0]], "ft", PLAN),
            eave: Val::known([[60.0, 0.0], [80.0, 0.0]], "ft", PLAN),
            eave_height: Val::known(Len::One(11.0), "ft", PLAN),
        },
    )
    .expect(OK);
    job.put(&area, Pitch(Val::known(0.25, "in/12", PLAN))).expect(OK);
    job.put(&area, unscheduled()).expect(OK);
    let parapet = job.add(&flat, Kind::Parapet, "parapet-south").expect(OK);
    job.put(&parapet, Run { ends: Val::known([[60.0, 0.0], [80.0, 0.0]], "ft", PLAN), planes: vec![] }).expect(OK);
    job.put(&parapet, Height(Val::unknown("ft", "parapet height not shown on the sample plan"))).expect(OK);

    // ---- A condition as a recipe: what a parapet with a copper coping is made of ----
    // The stretch-out, the spacing, the waste and the sheet size are the detail's and the
    // supplier's to give. The recipe leaves them blank; it does not guess them.
    let blank = "not entered; the detail gives it";
    let recipe = job.add(&root, Kind::Condition, "Parapet with copper coping").expect(OK);
    let sides = || [("sides".to_string(), Val::known(Len::One(2.0), "", "the condition: one each side of the coping"))].into_iter().collect::<std::collections::BTreeMap<_, _>>();
    job.put(
        &recipe,
        Recipe {
            applies_to: vec![Kind::Parapet],
            cost: Cost {
                code: Val::assumed(
                    CostCode { scheme: "MasterFormat".into(), code: "07 62 00".into(), title: "Sheet Metal Flashing and Trim".into() },
                    "",
                    "primary cost code scheme not ruled on yet (QUESTIONS.md Q1)",
                ),
                lines: vec![
                    CostLine::of("Copper wall flashing", "plan_length * height", Val::unknown("$/SF", "no price entered")),
                    CostLine {
                        stretch_out: Some(Val::unknown("in", blank)),
                        waste: Some(Val::unknown("%", "not entered")),
                        order: Some(Order { unit: "sheet".into(), factor: Val::unknown("SF/sheet", "not entered; the supplier's sheet size"), round_up: true }),
                        ..CostLine::of("Copper coping cap", "plan_length * stretch_out", Val::unknown("$/sheet", "no price entered"))
                    },
                    CostLine { inputs: sides(), ..CostLine::of("Continuous cleat", "plan_length * sides", Val::unknown("$/LF", "no price entered")) },
                    CostLine {
                        inputs: {
                            let mut m = sides();
                            m.insert("spacing".to_string(), Val::unknown("in", blank));
                            m
                        },
                        ..CostLine::of("Cleat fasteners", "plan_length * sides / spacing", Val::unknown("$/EA", "no price entered"))
                    },
                ],
            },
        },
    )
    .expect(OK);

    // ---- And a one-line recipe for any run of copper flashing ----
    let flashing = job.add(&root, Kind::Condition, "Copper flashing").expect(OK);
    job.put(
        &flashing,
        Recipe {
            applies_to: vec![Kind::Eave, Kind::Rake, Kind::Valley, Kind::Hip, Kind::Ridge, Kind::Headwall],
            cost: Cost {
                code: Val::assumed(
                    CostCode { scheme: "MasterFormat".into(), code: "07 62 00".into(), title: "Sheet Metal Flashing and Trim".into() },
                    "",
                    "primary cost code scheme not ruled on yet (QUESTIONS.md Q1)",
                ),
                lines: vec![CostLine { stretch_out: Some(Val::unknown("in", blank)), ..CostLine::of("Copper flashing", "true_length * stretch_out", Val::unknown("$/SF", "no price entered")) }],
            },
        },
    )
    .expect(OK);

    // ---- And a recipe for a roof surface, so a plane picked in the 3D view has one to take ----
    let seam = job.add(&root, Kind::Condition, "Standing seam copper roof").expect(OK);
    job.put(
        &seam,
        Recipe {
            applies_to: vec![Kind::RoofPlane],
            cost: Cost {
                code: Val::assumed(CostCode { scheme: "MasterFormat".into(), code: "07 61 00".into(), title: "Sheet Metal Roofing".into() }, "", "primary cost code scheme not ruled on yet (QUESTIONS.md Q1)"),
                lines: vec![
                    CostLine { waste: Some(Val::unknown("%", "not entered")), ..CostLine::of("Copper standing seam panels", "surface_area", Val::unknown("$/SF", "no price entered")) },
                    CostLine::of("Underlayment", "surface_area", Val::unknown("$/SF", "no price entered")),
                ],
            },
        },
    )
    .expect(OK);

    job.derive();
    job
}
