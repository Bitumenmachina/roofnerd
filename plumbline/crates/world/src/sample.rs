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

/// A made-up whole house: two storeys, 40 ft by 28 ft, under a hip roof solved from its
/// footprint, with its heating and cooling, its plumbing and its wiring. Nothing here is
/// from a real job, and nothing in it was engineered: sizes that a designer would work out
/// are entered as Assumed and say so.
///
/// * Storeys at 0 and 10 ft. Outside walls, four partitions a floor, doors and windows.
/// * Mechanical: an air handler and a heat pump; supply air in a first-floor soffit and in
///   the attic, with branches to registers; return air; a refrigerant line.
/// * Plumbing: a water heater, a kitchen sink and two stacked baths; cold and hot water,
///   a sanitary stack and its vent through the roof. One branch has no size on the plan.
/// * Electrical: a meter, a panel, the service conduit, and four cable runs to
///   receptacles and lights.
pub fn house() -> Job {
    use crate::building::*;
    const A: &str = "sample plan A-101";
    const M: &str = "sample plan M-101";
    const P: &str = "sample plan P-101";
    const E: &str = "sample plan E-101";
    const RULE: &str = "sized by rule of thumb; no load calculation has been done";
    let mut job = Job::new("Sample whole house (not a real job)", "plumbline sample", "sample");
    let root = "job-0001".to_string();
    let ft = |x: f64, src: &str| Val::known(Len::One(x), "ft", src);
    let inch = |x: f64, src: &str| Val::known(Len::One(x), "in", src);

    // ---- Storeys, floors, walls, openings ----
    let mut storeys = Vec::new();
    for (name, elevation, height, wall_height, floor_thickness) in [("first floor", 0.0, 10.0, 9.0, 4.0), ("second floor", 10.0, 9.0, 8.0, 12.0)] {
        let s = job.add(&root, Kind::Storey, name).expect(OK);
        job.put(&s, Level { elevation: ft(elevation, A), height: ft(height, A) }).expect(OK);
        let f = job.add(&s, Kind::Floor, "floor").expect(OK);
        job.put(&f, Floor { boundary: Val::known(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 28.0], [0.0, 28.0]], "ft", A), thickness: inch(floor_thickness, A) }).expect(OK);
        let wall = |job: &mut Job, name: &str, a: P2, b: P2, thick: f64, tall: f64| -> String {
            let w = job.add(&s, Kind::Wall, name).expect(OK);
            job.put(&w, Wall { line: Val::known([a, b], "ft", A), thickness: inch(thick, A), height: ft(tall, A) }).expect(OK);
            w
        };
        let open = |job: &mut Job, wall: &str, kind: Kind, name: &str, along: f64, width: f64, height: f64, sill: f64| {
            let o = job.add(wall, kind, name).expect(OK);
            job.put(&o, Opening { along: ft(along, A), width: ft(width, A), height: ft(height, A), sill: ft(sill, A) }).expect(OK);
        };
        let south = wall(&mut job, "south wall", [0.0, 0.0], [40.0, 0.0], 6.0, height);
        let east = wall(&mut job, "east wall", [40.0, 0.0], [40.0, 28.0], 6.0, height);
        let north = wall(&mut job, "north wall", [40.0, 28.0], [0.0, 28.0], 6.0, height);
        let west = wall(&mut job, "west wall", [0.0, 28.0], [0.0, 0.0], 6.0, height);
        let hall = wall(&mut job, "hall partition", [0.0, 16.0], [40.0, 16.0], 4.5, wall_height);
        let p14 = wall(&mut job, "partition at 14", [14.0, 16.0], [14.0, 28.0], 4.5, wall_height);
        let p24 = wall(&mut job, "partition at 24", [24.0, 16.0], [24.0, 28.0], 4.5, wall_height);
        let wet = wall(&mut job, "wet wall", [14.0, 22.0], [24.0, 22.0], 6.0, wall_height);
        let _ = wet;
        if elevation == 0.0 {
            open(&mut job, &south, Kind::Door, "front door", 18.5, 3.0, 6.67, 0.0);
        } else {
            open(&mut job, &south, Kind::Window, "window over the door", 17.5, 5.0, 4.5, 2.5);
        }
        open(&mut job, &south, Kind::Window, "west window", 5.0, 5.0, 5.0, 2.5);
        open(&mut job, &south, Kind::Window, "east window", 30.0, 5.0, 5.0, 2.5);
        open(&mut job, &east, Kind::Window, "front window", 5.0, 4.0, 5.0, 2.5);
        open(&mut job, &east, Kind::Window, "back window", 22.0, 3.0, 4.0, 3.5);
        open(&mut job, &north, Kind::Window, "east window", 5.0, 4.0, 4.0, 3.5);
        open(&mut job, &north, Kind::Window, "west window", 31.0, 4.0, 3.5, 4.0);
        open(&mut job, &west, Kind::Window, "back window", 4.0, 3.0, 4.0, 3.5);
        open(&mut job, &west, Kind::Window, "front window", 19.0, 4.0, 5.0, 2.5);
        open(&mut job, &hall, Kind::Door, "west door", 8.0, 2.67, 6.67, 0.0);
        open(&mut job, &hall, Kind::Door, "bath door", 15.0, 2.5, 6.67, 0.0);
        open(&mut job, &hall, Kind::Door, "east door", 28.0, 2.67, 6.67, 0.0);
        open(&mut job, &p14, Kind::Door, "door", 8.0, 2.5, 6.67, 0.0);
        let _ = p24;
        storeys.push(s);
    }
    let (first, second) = (storeys[0].clone(), storeys[1].clone());

    // ---- The roof: a hip roof with a one-foot overhang, eaves at 19 ft ----
    let footprint = Footprint {
        points: Val::known(vec![[-1.0, -1.0], [41.0, -1.0], [41.0, 29.0], [-1.0, 29.0]], "ft", A),
        eave_height: ft(19.0, A),
        edges: (0..4).map(|_| Val::known(Rise::Pitch(6.0), "in/12", A)).collect(),
    };
    crate::solve::new_roof(&mut job, "roof", footprint).expect(OK);

    // ---- Systems ----
    let system = |job: &mut Job, name: &str, discipline: Discipline, classification: &str| -> String {
        let s = job.add(&root, Kind::System, name).expect(OK);
        job.put(&s, SystemInfo { discipline, classification: classification.into() }).expect(OK);
        s
    };
    let supply = system(&mut job, "Supply air", Discipline::Mechanical, "Supply Air");
    let ret = system(&mut job, "Return air", Discipline::Mechanical, "Return Air");
    let refrigerant = system(&mut job, "Refrigerant", Discipline::Mechanical, "Refrigerant");
    let cold = system(&mut job, "Cold water", Discipline::Plumbing, "Domestic Cold Water");
    let hot = system(&mut job, "Hot water", Discipline::Plumbing, "Domestic Hot Water");
    let sanitary = system(&mut job, "Sanitary", Discipline::Plumbing, "Sanitary");
    let vent = system(&mut job, "Vent", Discipline::Plumbing, "Vent");
    let power = system(&mut job, "Power", Discipline::Electrical, "Power");
    let lighting = system(&mut job, "Lighting", Discipline::Electrical, "Lighting");

    let run = |job: &mut Job, storey: &str, kind: Kind, name: &str, sys: &str, src: &str, size: Size, pts: &[P3]| -> String {
        let r = job.add(storey, kind, name).expect(OK);
        job.put(&r, Route { points: Val::known(pts.to_vec(), "ft", src) }).expect(OK);
        job.put(&r, size).expect(OK);
        job.put(&r, InSystem(Val::known(sys.to_string(), "", src))).expect(OK);
        r
    };
    let thing = |job: &mut Job, storey: &str, kind: Kind, name: &str, sys: &str, src: &str, what: Val<String>, at: P3, size: Val<P3>| {
        let t = job.add(storey, kind, name).expect(OK);
        job.put(&t, Placed { what, at: Val::known(at, "ft", src), size }).expect(OK);
        job.put(&t, InSystem(Val::known(sys.to_string(), "", src))).expect(OK);
    };
    let said = |s: &str, src: &str| Val::known(s.to_string(), "", src);
    let rect = |w: f64, d: f64| Size::rectangular(Val::assumed(Len::One(w), "in", RULE), Val::assumed(Len::One(d), "in", RULE));
    let round_assumed = |d: f64| Size::round(Val::assumed(Len::One(d), "in", RULE));
    let round = |d: f64, src: &str| Size::round(Val::known(Len::One(d), "in", src));
    let boxed = |x: f64, y: f64, z: f64, src: &str| Val::known([x, y, z], "ft", src);
    let typical = |x: f64, y: f64, z: f64| Val::assumed([x, y, z], "ft", "a typical size; the product is not selected");

    // ---- Mechanical ----
    thing(&mut job, &first, Kind::Equipment, "air handler", &supply, M, Val::assumed("Air handler, vertical".into(), "", "the unit is not selected"), [19.0, 25.0, 0.0], typical(2.0, 2.5, 5.0));
    thing(&mut job, &first, Kind::Equipment, "heat pump", &refrigerant, M, Val::assumed("Heat pump, outdoor unit".into(), "", "the unit is not selected"), [43.0, 25.0, 0.0], typical(3.0, 3.0, 3.0));
    run(&mut job, &first, Kind::Duct, "first-floor trunk, west", &supply, M, rect(16.0, 8.0), &[[19.0, 25.0, 5.0], [19.0, 25.0, 8.4], [19.0, 17.0, 8.4], [2.0, 17.0, 8.4]]);
    run(&mut job, &first, Kind::Duct, "first-floor trunk, east", &supply, M, rect(14.0, 8.0), &[[19.0, 17.0, 8.4], [38.0, 17.0, 8.4]]);
    for (name, from, to) in [("living west", [6.0, 17.0], [6.0, 6.0]), ("living middle", [22.0, 17.0], [22.0, 6.0]), ("living east", [34.0, 17.0], [34.0, 6.0]), ("kitchen", [8.0, 17.0], [8.0, 24.0]), ("office", [32.0, 17.0], [32.0, 24.0])] {
        run(&mut job, &first, Kind::Duct, &format!("branch to {name}"), &supply, M, round_assumed(6.0), &[[from[0], from[1], 8.4], [to[0], to[1], 8.4]]);
        thing(&mut job, &first, Kind::Fixture, &format!("register, {name}"), &supply, M, said("Ceiling register, 4 × 10", M), [to[0], to[1], 8.85], boxed(0.83, 0.33, 0.12, M));
    }
    run(&mut job, &first, Kind::Duct, "riser to the attic", &supply, M, rect(12.0, 8.0), &[[18.4, 26.2, 5.0], [18.4, 26.2, 20.0], [18.4, 14.0, 20.0]]);
    run(&mut job, &second, Kind::Duct, "attic trunk", &supply, M, rect(12.0, 8.0), &[[4.0, 14.0, 20.0], [36.0, 14.0, 20.0]]);
    for (name, x, y) in [("bedroom 1", 6.0, 6.0), ("bedroom 2", 34.0, 6.0), ("bedroom 3", 8.0, 23.0), ("bedroom 4", 32.0, 23.0)] {
        run(&mut job, &second, Kind::Duct, &format!("branch to {name}"), &supply, M, round_assumed(6.0), &[[x, 14.0, 20.0], [x, y, 20.0], [x, y, 18.15]]);
        thing(&mut job, &second, Kind::Fixture, &format!("register, {name}"), &supply, M, said("Ceiling register, 4 × 10", M), [x, y, 17.9], boxed(0.83, 0.33, 0.12, M));
    }
    run(&mut job, &first, Kind::Duct, "return riser", &ret, M, rect(20.0, 10.0), &[[21.2, 27.0, 17.4], [21.2, 27.0, 0.9], [20.0, 25.8, 0.9]]);
    run(&mut job, &first, Kind::Duct, "first-floor return", &ret, M, rect(20.0, 10.0), &[[21.2, 23.0, 0.9], [21.2, 27.0, 0.9]]);
    thing(&mut job, &first, Kind::Fixture, "return grille, first floor", &ret, M, said("Return grille, 20 × 20", M), [21.2, 22.4, 0.1], boxed(1.67, 0.15, 1.67, M));
    thing(&mut job, &second, Kind::Fixture, "return grille, second floor", &ret, M, said("Return grille, 20 × 20", M), [21.2, 27.0, 17.8], boxed(1.67, 1.67, 0.15, M));
    run(&mut job, &first, Kind::Pipe, "refrigerant line set", &refrigerant, M, round(0.875, M), &[[41.5, 25.0, 1.0], [39.5, 25.0, 1.0], [39.5, 25.0, 9.5], [20.0, 25.0, 9.5], [20.0, 25.0, 5.0]]);

    // ---- Plumbing ----
    thing(&mut job, &first, Kind::Equipment, "water heater", &hot, P, Val::assumed("Water heater, 50 gal".into(), "", "the size is not on the plan"), [16.0, 26.5, 0.0], typical(2.0, 2.0, 5.0));
    for (storey, z, n) in [(&first, 0.0, 1), (&second, 10.0, 2)] {
        thing(&mut job, storey, Kind::Fixture, &format!("lavatory {n}"), &cold, P, said("Lavatory", P), [16.0, 21.2, z + 2.3], boxed(1.6, 1.3, 0.6, P));
        thing(&mut job, storey, Kind::Fixture, &format!("water closet {n}"), &cold, P, said("Water closet", P), [18.5, 20.8, z], boxed(1.3, 2.2, 2.4, P));
        thing(&mut job, storey, Kind::Fixture, &format!("tub {n}"), &cold, P, said("Bathtub, 5 ft", P), [22.6, 19.2, z], boxed(2.5, 5.0, 1.6, P));
    }
    thing(&mut job, &first, Kind::Fixture, "kitchen sink", &cold, P, said("Kitchen sink, double bowl", P), [11.0, 27.0, 2.3], boxed(2.75, 1.8, 0.7, P));
    // Cold water.
    run(&mut job, &first, Kind::Pipe, "water service", &cold, P, round(1.0, P), &[[-2.0, 25.0, -2.0], [0.6, 25.0, -2.0], [0.6, 25.0, 0.7], [0.6, 27.5, 0.7], [15.5, 27.5, 0.7]]);
    run(&mut job, &first, Kind::Pipe, "cold main to the baths", &cold, P, round(0.75, P), &[[15.5, 27.5, 0.7], [15.5, 22.4, 0.7], [22.6, 22.4, 0.7]]);
    run(&mut job, &first, Kind::Pipe, "cold to the water heater", &cold, P, round(0.75, P), &[[15.5, 27.5, 0.7], [15.5, 27.5, 5.6], [15.7, 26.5, 5.6], [15.7, 26.5, 5.0]]);
    run(&mut job, &first, Kind::Pipe, "cold to the kitchen sink", &cold, P, round(0.5, P), &[[10.6, 27.5, 0.7], [10.6, 27.5, 2.3]]);
    run(&mut job, &first, Kind::Pipe, "cold riser", &cold, P, round(0.75, P), &[[17.0, 22.4, 0.7], [17.0, 22.4, 10.7]]);
    run(&mut job, &second, Kind::Pipe, "cold header, second floor", &cold, P, round(0.75, P), &[[16.0, 22.4, 10.7], [22.6, 22.4, 10.7]]);
    // Hot water. The kitchen branch is drawn on the sample plan with no size on it.
    run(&mut job, &first, Kind::Pipe, "hot main to the baths", &hot, P, round(0.75, P), &[[16.3, 26.5, 5.0], [16.3, 26.5, 6.0], [16.3, 22.7, 6.0], [16.3, 22.7, 1.0], [22.3, 22.7, 1.0]]);
    run(&mut job, &first, Kind::Pipe, "hot to the kitchen sink", &hot, P, Size::round(Val::unknown("in", "drawn on the plan with no size")), &[[16.3, 26.5, 6.0], [16.3, 27.3, 6.0], [11.4, 27.3, 6.0], [11.4, 27.3, 2.3]]);
    run(&mut job, &first, Kind::Pipe, "hot riser", &hot, P, round(0.75, P), &[[17.4, 22.7, 1.0], [17.4, 22.7, 11.0]]);
    run(&mut job, &second, Kind::Pipe, "hot header, second floor", &hot, P, round(0.75, P), &[[16.3, 22.7, 11.0], [22.3, 22.7, 11.0]]);
    for (storey, z, n) in [(&first, 0.0, 1), (&second, 10.0, 2)] {
        run(&mut job, storey, Kind::Pipe, &format!("cold to lavatory {n}"), &cold, P, round(0.5, P), &[[16.0, 22.4, z + 0.7], [16.0, 22.4, z + 2.0], [16.0, 21.8, z + 2.0]]);
        run(&mut job, storey, Kind::Pipe, &format!("cold to water closet {n}"), &cold, P, round(0.5, P), &[[18.2, 22.4, z + 0.7], [18.2, 21.8, z + 0.7]]);
        run(&mut job, storey, Kind::Pipe, &format!("cold to tub {n}"), &cold, P, round(0.5, P), &[[22.6, 22.4, z + 0.7], [22.6, 22.4, z + 2.5], [22.6, 21.7, z + 2.5]]);
        run(&mut job, storey, Kind::Pipe, &format!("hot to lavatory {n}"), &hot, P, round(0.5, P), &[[16.3, 22.7, z + 1.0], [16.3, 22.7, z + 2.0], [16.3, 21.8, z + 2.0]]);
        run(&mut job, storey, Kind::Pipe, &format!("hot to tub {n}"), &hot, P, round(0.5, P), &[[22.3, 22.7, z + 1.0], [22.3, 22.7, z + 2.5], [22.3, 21.7, z + 2.5]]);
        // Waste from each fixture to the stack in the wet wall.
        let low = z - 0.8;
        run(&mut job, storey, Kind::Pipe, &format!("waste from water closet {n}"), &sanitary, P, round(3.0, P), &[[18.5, 21.0, z], [18.5, 21.0, low], [18.5, 22.0, low]]);
        run(&mut job, storey, Kind::Pipe, &format!("waste from lavatory {n}"), &sanitary, P, round(1.5, P), &[[16.0, 21.6, z + 1.5], [16.0, 22.0, z + 1.5], [18.5, 22.0, z + 1.5]]);
        run(&mut job, storey, Kind::Pipe, &format!("waste from tub {n}"), &sanitary, P, round(2.0, P), &[[22.6, 20.5, z], [22.6, 20.5, low], [22.6, 22.0, low], [18.5, 22.0, low]]);
    }
    run(&mut job, &first, Kind::Pipe, "soil stack", &sanitary, P, round(3.0, P), &[[18.5, 22.0, -1.6], [18.5, 22.0, 11.5]]);
    run(&mut job, &first, Kind::Pipe, "building drain", &sanitary, P, round(4.0, P), &[[18.5, 22.0, -1.6], [18.5, 30.5, -1.6]]);
    run(&mut job, &first, Kind::Pipe, "waste from the kitchen sink", &sanitary, P, round(2.0, P), &[[11.0, 27.2, 1.5], [11.0, 27.6, 1.5], [11.0, 27.6, -1.6], [18.5, 27.6, -1.6]]);
    run(&mut job, &second, Kind::Pipe, "vent through the roof", &vent, P, round(2.0, P), &[[18.5, 22.0, 11.5], [18.5, 22.0, 23.6]]);

    // ---- Electrical ----
    thing(&mut job, &first, Kind::Equipment, "meter", &power, E, said("Meter socket, 200 A", E), [40.45, 20.0, 4.0], boxed(0.4, 1.0, 1.3, E));
    thing(&mut job, &first, Kind::Equipment, "panel", &power, E, said("Panelboard, 200 A", E), [39.55, 20.0, 3.5], boxed(0.35, 1.2, 2.6, E));
    run(&mut job, &first, Kind::Conduit, "service entrance", &power, E, round(2.0, E), &[[40.45, 20.0, 4.0], [40.45, 20.0, -2.0], [44.0, 20.0, -2.0]]);
    let cable = || Size::round(Val::assumed(Len::One(0.4), "in", "the jacket of a 12-2 cable, roughly; the cable is not selected"));
    run(&mut job, &first, Kind::Cable, "kitchen circuit", &power, E, cable(), &[[39.55, 20.0, 6.1], [39.55, 20.0, 9.6], [8.0, 20.0, 9.6], [8.0, 27.6, 9.6], [8.0, 27.6, 3.6]]);
    thing(&mut job, &first, Kind::Fixture, "receptacle, kitchen", &power, E, said("Duplex receptacle", E), [8.0, 27.65, 3.4], boxed(0.25, 0.1, 0.4, E));
    run(&mut job, &first, Kind::Cable, "living circuit", &power, E, cable(), &[[39.55, 19.5, 6.1], [39.55, 19.5, 9.7], [30.0, 19.5, 9.7], [30.0, 0.35, 9.7], [30.0, 0.35, 1.4]]);
    thing(&mut job, &first, Kind::Fixture, "receptacle, living", &power, E, said("Duplex receptacle", E), [30.0, 0.35, 1.2], boxed(0.25, 0.1, 0.4, E));
    run(&mut job, &first, Kind::Cable, "first-floor lights", &lighting, E, cable(), &[[39.55, 20.5, 6.1], [39.55, 20.5, 9.5], [20.0, 20.5, 9.5], [20.0, 8.0, 9.5], [20.0, 8.0, 9.0]]);
    thing(&mut job, &first, Kind::Fixture, "light, living", &lighting, E, said("Ceiling light", E), [20.0, 8.0, 8.8], boxed(1.0, 1.0, 0.2, E));
    run(&mut job, &second, Kind::Cable, "second-floor lights", &lighting, E, cable(), &[[39.55, 21.0, 6.1], [39.55, 21.0, 19.3], [20.0, 21.0, 19.3], [20.0, 7.0, 19.3], [20.0, 7.0, 18.0]]);
    thing(&mut job, &second, Kind::Fixture, "light, hall", &lighting, E, said("Ceiling light", E), [20.0, 7.0, 17.8], boxed(1.0, 1.0, 0.2, E));

    job.derive();
    job
}
