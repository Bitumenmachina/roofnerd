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
use crate::value::{Status, Val};

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
            "which cost code scheme this job is to follow is not settled",
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
                "which cost code scheme this job is to follow is not settled",
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
    let sides = || [("sides".to_string(), Val::known(Len::One(2.0), "", "the assembly: one each side of the coping"))].into_iter().collect::<std::collections::BTreeMap<_, _>>();
    job.put(
        &recipe,
        Recipe {
            applies_to: vec![Kind::Parapet],
            cost: Cost {
                code: Val::assumed(
                    CostCode { scheme: "MasterFormat".into(), code: "07 62 00".into(), title: "Sheet Metal Flashing and Trim".into() },
                    "",
                    "which cost code scheme this job is to follow is not settled",
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
                    "which cost code scheme this job is to follow is not settled",
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
                code: Val::assumed(CostCode { scheme: "MasterFormat".into(), code: "07 61 00".into(), title: "Sheet Metal Roofing".into() }, "", "which cost code scheme this job is to follow is not settled"),
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

/// A run's section as a sample plan gives it: a size, or a line drawn with none.
enum Section {
    Is(crate::building::SegmentType),
    NotOnPlan(&'static str),
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

    // ---- Types: what walls, floors, doors and windows of one sort have in common ----
    let named = |p: &str, src: &str| Val::known(p.to_string(), "", src);
    let wall_type = |job: &mut Job, name: &str, thick: f64| -> String {
        let t = job.add(&root, Kind::Type, name).expect(OK);
        job.put(&t, WallType { thickness: inch(thick, A), layers: Vec::new() }).expect(OK);
        t
    };
    let outside = wall_type(&mut job, "Outside wall, 6 in", 6.0);
    let partition = wall_type(&mut job, "Partition, 4 1/2 in", 4.5);
    let wet_wall = wall_type(&mut job, "Wet wall, 6 in", 6.0);
    let floor_type = |job: &mut Job, name: &str, thick: f64| -> String {
        let t = job.add(&root, Kind::Type, name).expect(OK);
        job.put(&t, FloorType { thickness: inch(thick, A), layers: Vec::new() }).expect(OK);
        t
    };
    let slab = floor_type(&mut job, "Slab on grade, 4 in", 4.0);
    let framed = floor_type(&mut job, "Framed floor, 12 in", 12.0);
    // A door or window type is made the first time a size is met on the plan.
    let mut opening_types: Vec<(Kind, f64, f64, String)> = Vec::new();
    let mut opening_type = |job: &mut Job, kind: Kind, width: f64, height: f64| -> String {
        if let Some(t) = opening_types.iter().find(|t| t.0 == kind && t.1 == width && t.2 == height) {
            return t.3.clone();
        }
        let name = format!("{} {} ft × {} ft", if kind == Kind::Door { "Door" } else { "Window" }, crate::view::num(width), crate::view::num(height));
        let t = job.add(&root, Kind::Type, &name).expect(OK);
        job.put(&t, OpeningType { category: kind, width: ft(width, A), height: ft(height, A) }).expect(OK);
        opening_types.push((kind, width, height, t.clone()));
        t
    };

    // ---- Storeys, floors, walls, openings ----
    let mut storeys = Vec::new();
    for (name, elevation, height) in [("first floor", 0.0, 10.0), ("second floor", 10.0, 9.0)] {
        let s = job.add(&root, Kind::Storey, name).expect(OK);
        job.put(&s, Level { elevation: ft(elevation, A), height: ft(height, A), computation_height: None }).expect(OK);
        storeys.push(s);
    }
    for (i, (wall_height, floor)) in [(9.0, &slab), (8.0, &framed)].into_iter().enumerate() {
        let s = storeys[i].clone();
        let f = job.add(&s, Kind::Floor, "floor").expect(OK);
        job.put(&f, Floor { type_name: named(floor, A), boundary: Val::known(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 28.0], [0.0, 28.0]], "ft", A), offset: None }).expect(OK);
        // The first floor's outside walls run up to the second floor; the second floor's
        // are 9 ft tall. Partitions stop under the ceiling.
        let outside_wall = |job: &mut Job, name: &str, a: P2, b: P2| -> String {
            let w = job.add(&s, Kind::Wall, name).expect(OK);
            let line = Val::known([a, b], "ft", A);
            let wall = if i == 0 { Wall::up_to(named(&outside, A), line, named(&storeys[1], A)) } else { Wall::tall(named(&outside, A), line, ft(9.0, A)) };
            job.put(&w, wall).expect(OK);
            w
        };
        let inside_wall = |job: &mut Job, name: &str, a: P2, b: P2, of: &str| -> String {
            let w = job.add(&s, Kind::Wall, name).expect(OK);
            job.put(&w, Wall::tall(named(of, A), Val::known([a, b], "ft", A), ft(wall_height, A))).expect(OK);
            w
        };
        // An opening is given the way a framing plan dimensions it: to its centre.
        let mut open = |job: &mut Job, wall: &str, kind: Kind, name: &str, starts: f64, width: f64, height: f64, sill: f64| {
            let t = opening_type(job, kind, width, height);
            let o = job.add(wall, kind, name).expect(OK);
            job.put(&o, Opening { type_name: named(&t, A), offset: ft(starts + width / 2.0, A), sill: ft(sill, A) }).expect(OK);
        };
        let south = outside_wall(&mut job, "south wall", [0.0, 0.0], [40.0, 0.0]);
        let east = outside_wall(&mut job, "east wall", [40.0, 0.0], [40.0, 28.0]);
        let north = outside_wall(&mut job, "north wall", [40.0, 28.0], [0.0, 28.0]);
        let west = outside_wall(&mut job, "west wall", [0.0, 28.0], [0.0, 0.0]);
        let hall = inside_wall(&mut job, "hall partition", [0.0, 16.0], [40.0, 16.0], &partition);
        let p14 = inside_wall(&mut job, "partition at 14", [14.0, 16.0], [14.0, 28.0], &partition);
        let p24 = inside_wall(&mut job, "partition at 24", [24.0, 16.0], [24.0, 28.0], &partition);
        let wet = inside_wall(&mut job, "wet wall", [14.0, 22.0], [24.0, 22.0], &wet_wall);
        let _ = wet;
        if i == 0 {
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
        // The rooms: a point inside each. What each comes to is found from its walls.
        let rooms: [(&str, &str, P2); 5] = if i == 0 {
            [("living room", "101", [20.0, 8.0]), ("kitchen", "102", [7.0, 22.0]), ("bath", "103", [19.0, 19.0]), ("utility", "104", [19.0, 25.0]), ("office", "105", [32.0, 22.0])]
        } else {
            [("south bedrooms", "201", [20.0, 8.0]), ("bedroom 3", "202", [7.0, 22.0]), ("bath", "203", [19.0, 19.0]), ("hall closet", "204", [19.0, 25.0]), ("bedroom 4", "205", [32.0, 22.0])]
        };
        for (name, number, point) in rooms {
            let r = job.add(&s, Kind::Room, name).expect(OK);
            job.put(&r, Room { point: Val::known(point, "ft", A), number: Some(Val::known(number.to_string(), "", A)) }).expect(OK);
        }
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

    // A run is drawn on the sample's plans as a line that turns corners. In the job it is
    // one straight piece from each corner to the next, each of the run's type, with an
    // elbow at each corner: pipe and duct are measured through their fittings, centre to
    // centre, so the pieces come to the length of the line. A cable turns a corner with
    // no fitting. A type is made the first time a size is met.
    let base_of = |storey: &str| if storey == second { 10.0 } else { 0.0 };
    let tidy = |x: f64| (x * 1.0e6).round() / 1.0e6;
    let mut run_types: Vec<(String, String)> = Vec::new();
    let mut run = |job: &mut Job, storey: &str, kind: Kind, name: &str, sys: &str, src: &str, size: Section, pts: &[P3]| {
        let base = base_of(storey);
        // The type, or that nobody has it.
        let (type_ref, section) = match size {
            Section::NotOnPlan(why) => (Val::unknown("", why), None),
            Section::Is(t) => {
                let t = SegmentType { category: kind, ..t };
                let label = format!("{kind:?} {}{}", t.said(), if t.status() == Status::Assumed { ", assumed" } else { "" });
                let path = match run_types.iter().find(|x| x.0 == label) {
                    Some(x) => x.1.clone(),
                    None => {
                        let path = job.add(&root, Kind::Type, &label).expect(OK);
                        job.put(&path, t.clone()).expect(OK);
                        run_types.push((label, path.clone()));
                        path
                    }
                };
                (Val::known(path, "", src), Some(t))
            }
        };
        let pieces = pts.len() - 1;
        for (i, pair) in pts.windows(2).enumerate() {
            let (a, b) = (pair[0], pair[1]);
            let piece = if pieces == 1 { name.to_string() } else { format!("{name}, piece {} of {pieces}", i + 1) };
            let r = job.add(storey, kind, &piece).expect(OK);
            let end_offset = (a[2] != b[2]).then(|| ft(tidy(b[2] - base), src));
            job.put(&r, Segment { type_name: type_ref.clone(), line: Val::known([[a[0], a[1]], [b[0], b[1]]], "ft", src), offset: ft(tidy(a[2] - base), src), end_offset, justify: Justify::Centre }).expect(OK);
            job.put(&r, InSystem(Val::known(sys.to_string(), "", src))).expect(OK);
            if i + 1 < pieces && kind != Kind::Cable {
                // The elbow at the corner this piece ends at. It is the run's size; which
                // elbow is bought is not specified.
                let f = job.add(storey, Kind::Fitting, &format!("{name}, elbow {}", i + 1)).expect(OK);
                // (deep: how tall the elbow is drawn, in inches, so its middle is on the joint.)
                let (what, size, deep) = match &section {
                    None => (Val::unknown("", "its run is drawn on the plan with no size"), Val::unknown("in", "its run is drawn on the plan with no size"), 0.0),
                    Some(t) => {
                        let across = t.width.value.as_ref().and_then(|v| v.feet(&t.width.unit).ok()).unwrap_or(0.0) * 12.0;
                        let deep = t.depth.as_ref().and_then(|d| d.feet()).map(|d| d * 12.0).unwrap_or(across);
                        // A pipe's or a conduit's size is its diameter and is said without "round".
                        let what = format!("{} elbow", if kind == Kind::Duct { t.said() } else { t.said().replace(" round", "") });
                        let what = if t.status() == Status::Assumed { Val::assumed(what, "", RULE) } else { Val::known(what, "", src) };
                        (what, Val::assumed([tidy(across), tidy(across), tidy(deep)], "in", "drawn at its run's size; the fitting is not specified"), deep)
                    }
                };
                // An elbow joins the piece that ends here to the one that starts here: a
                // connector for each, both on the joint, of the run's size.
                let end = Connector { name: String::new(), kind: ConnectorKind::for_run(kind).expect(OK), domain: None, flow: Flow::Both, at: Val::known([0.0, 0.0, tidy(deep / 2.0)], "in", "on the joint"), size: section.as_ref().map(|t| t.width.clone()) };
                job.put(&f, Connectors(vec![end.clone(), end])).expect(OK);
                job.put(&f, Fitting { kind: FittingKind::Bend }).expect(OK);
                job.put(&f, Placed { what, at: Val::known([b[0], b[1]], "ft", src), offset: ft(tidy(b[2] - base - deep / 24.0), src), angle: None, size, item: None }).expect(OK);
                job.put(&f, InSystem(Val::known(sys.to_string(), "", src))).expect(OK);
            }
        }
    };
    let mut set: Vec<(String, String, P3)> = Vec::new();
    let mut thing = |job: &mut Job, storey: &str, kind: Kind, name: &str, sys: &str, src: &str, what: Val<String>, at: P3, size: Val<P3>| {
        let t = job.add(storey, kind, name).expect(OK);
        set.push((name.to_string(), t.clone(), at));
        job.put(&t, Placed { what, at: Val::known([at[0], at[1]], "ft", src), offset: ft(tidy(at[2] - base_of(storey)), src), angle: None, size, item: None }).expect(OK);
        job.put(&t, InSystem(Val::known(sys.to_string(), "", src))).expect(OK);
    };
    let said = |s: &str, src: &str| Val::known(s.to_string(), "", src);
    let rect = |w: f64, d: f64| Section::Is(SegmentType::rectangular(Kind::Duct, Val::assumed(Len::One(w), "in", RULE), Val::assumed(Len::One(d), "in", RULE)));
    let round_assumed = |d: f64| Section::Is(SegmentType::round(Kind::Duct, Val::assumed(Len::One(d), "in", RULE)));
    let round = |d: f64, src: &str| Section::Is(SegmentType::round(Kind::Pipe, Val::known(Len::One(d), "in", src)));
    let boxed = |x: f64, y: f64, z: f64, src: &str| Val::known([x, y, z], "ft", src);
    let typical = |x: f64, y: f64, z: f64| Val::assumed([x, y, z], "ft", "a typical size; the product is not specified");

    // ---- Mechanical ----
    thing(&mut job, &first, Kind::Equipment, "air handler", &supply, M, Val::assumed("Air handler, vertical".into(), "", "the unit is not specified"), [19.0, 25.0, 0.0], typical(2.0, 2.5, 5.0));
    thing(&mut job, &first, Kind::Equipment, "heat pump", &refrigerant, M, Val::assumed("Heat pump, outdoor unit".into(), "", "the unit is not specified"), [43.0, 25.0, 0.0], typical(3.0, 3.0, 3.0));
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
    run(&mut job, &first, Kind::Pipe, "hot to the kitchen sink", &hot, P, Section::NotOnPlan("drawn on the plan with no size"), &[[16.3, 26.5, 6.0], [16.3, 27.3, 6.0], [11.4, 27.3, 6.0], [11.4, 27.3, 2.3]]);
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
    let cable = || Section::Is(SegmentType::round(Kind::Cable, Val::assumed(Len::One(0.4), "in", "the jacket of a 12-2 cable, roughly; the cable is not specified")));
    run(&mut job, &first, Kind::Cable, "kitchen circuit", &power, E, cable(), &[[39.55, 20.0, 6.1], [39.55, 20.0, 9.6], [8.0, 20.0, 9.6], [8.0, 27.6, 9.6], [8.0, 27.6, 3.6]]);
    thing(&mut job, &first, Kind::Fixture, "receptacle, kitchen", &power, E, said("Duplex receptacle", E), [8.0, 27.65, 3.4], boxed(0.25, 0.1, 0.4, E));
    run(&mut job, &first, Kind::Cable, "living circuit", &power, E, cable(), &[[39.55, 19.5, 6.1], [39.55, 19.5, 9.7], [30.0, 19.5, 9.7], [30.0, 0.35, 9.7], [30.0, 0.35, 1.4]]);
    thing(&mut job, &first, Kind::Fixture, "receptacle, living", &power, E, said("Duplex receptacle", E), [30.0, 0.35, 1.2], boxed(0.25, 0.1, 0.4, E));
    run(&mut job, &first, Kind::Cable, "first-floor lights", &lighting, E, cable(), &[[39.55, 20.5, 6.1], [39.55, 20.5, 9.5], [20.0, 20.5, 9.5], [20.0, 8.0, 9.5], [20.0, 8.0, 9.0]]);
    thing(&mut job, &first, Kind::Fixture, "light, living", &lighting, E, said("Ceiling light", E), [20.0, 8.0, 8.8], boxed(1.0, 1.0, 0.2, E));
    run(&mut job, &second, Kind::Cable, "second-floor lights", &lighting, E, cable(), &[[39.55, 21.0, 6.1], [39.55, 21.0, 19.3], [20.0, 21.0, 19.3], [20.0, 7.0, 19.3], [20.0, 7.0, 18.0]]);
    thing(&mut job, &second, Kind::Fixture, "light, hall", &lighting, E, said("Ceiling light", E), [20.0, 7.0, 17.8], boxed(1.0, 1.0, 0.2, E));


    // ---- Connectors: where each run meets the thing it serves ----
    // Each is given where the plan shows the run arriving, and kept on the thing as a
    // point in the thing's own axes.
    let mut plugs: Vec<(String, &str, ConnectorKind, Option<Domain>, Flow, P3)> = vec![
        ("air handler".into(), "Supply, first floor", ConnectorKind::Duct, None, Flow::Out, [19.0, 25.0, 5.0]),
        ("air handler".into(), "Supply, attic", ConnectorKind::Duct, None, Flow::Out, [18.4, 26.2, 5.0]),
        ("air handler".into(), "Return", ConnectorKind::Duct, None, Flow::In, [20.0, 25.8, 0.9]),
        ("air handler".into(), "Refrigerant", ConnectorKind::Pipe, None, Flow::Both, [20.0, 25.0, 5.0]),
        ("heat pump".into(), "Refrigerant", ConnectorKind::Pipe, None, Flow::Both, [41.5, 25.0, 1.0]),
        ("return grille, first floor".into(), "Return", ConnectorKind::Duct, None, Flow::Out, [21.2, 23.0, 0.9]),
        ("return grille, second floor".into(), "Return", ConnectorKind::Duct, None, Flow::Out, [21.2, 27.0, 17.4]),
        ("water heater".into(), "Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, [15.7, 26.5, 5.0]),
        ("water heater".into(), "Hot water", ConnectorKind::Pipe, Some(Domain::Hot), Flow::Out, [16.3, 26.5, 5.0]),
        ("kitchen sink".into(), "Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, [10.6, 27.5, 2.3]),
        ("kitchen sink".into(), "Hot water", ConnectorKind::Pipe, Some(Domain::Hot), Flow::In, [11.4, 27.3, 2.3]),
        ("kitchen sink".into(), "Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, [11.0, 27.2, 1.5]),
        ("meter".into(), "Service", ConnectorKind::Conduit, None, Flow::Both, [40.45, 20.0, 4.0]),
        ("panel".into(), "Kitchen circuit", ConnectorKind::Power, None, Flow::Out, [39.55, 20.0, 6.1]),
        ("panel".into(), "Living circuit", ConnectorKind::Power, None, Flow::Out, [39.55, 19.5, 6.1]),
        ("panel".into(), "First-floor lights", ConnectorKind::Power, None, Flow::Out, [39.55, 20.5, 6.1]),
        ("panel".into(), "Second-floor lights", ConnectorKind::Power, None, Flow::Out, [39.55, 21.0, 6.1]),
        ("receptacle, kitchen".into(), "Power", ConnectorKind::Power, None, Flow::In, [8.0, 27.6, 3.6]),
        ("receptacle, living".into(), "Power", ConnectorKind::Power, None, Flow::In, [30.0, 0.35, 1.4]),
        ("light, living".into(), "Power", ConnectorKind::Power, None, Flow::In, [20.0, 8.0, 9.0]),
        ("light, hall".into(), "Power", ConnectorKind::Power, None, Flow::In, [20.0, 7.0, 18.0]),
    ];
    for (name, to) in [("living west", [6.0, 6.0]), ("living middle", [22.0, 6.0]), ("living east", [34.0, 6.0]), ("kitchen", [8.0, 24.0]), ("office", [32.0, 24.0])] {
        plugs.push((format!("register, {name}"), "Supply", ConnectorKind::Duct, None, Flow::In, [to[0], to[1], 8.4]));
    }
    for (name, x, y) in [("bedroom 1", 6.0, 6.0), ("bedroom 2", 34.0, 6.0), ("bedroom 3", 8.0, 23.0), ("bedroom 4", 32.0, 23.0)] {
        plugs.push((format!("register, {name}"), "Supply", ConnectorKind::Duct, None, Flow::In, [x, y, 18.15]));
    }
    for (z, n) in [(0.0, 1), (10.0, 2)] {
        plugs.push((format!("lavatory {n}"), "Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, [16.0, 21.8, z + 2.0]));
        plugs.push((format!("lavatory {n}"), "Hot water", ConnectorKind::Pipe, Some(Domain::Hot), Flow::In, [16.3, 21.8, z + 2.0]));
        plugs.push((format!("lavatory {n}"), "Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, [16.0, 21.6, z + 1.5]));
        plugs.push((format!("water closet {n}"), "Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, [18.2, 21.8, z + 0.7]));
        plugs.push((format!("water closet {n}"), "Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, [18.5, 21.0, z]));
        plugs.push((format!("tub {n}"), "Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, [22.6, 21.7, z + 2.5]));
        plugs.push((format!("tub {n}"), "Hot water", ConnectorKind::Pipe, Some(Domain::Hot), Flow::In, [22.3, 21.7, z + 2.5]));
        plugs.push((format!("tub {n}"), "Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, [22.6, 20.5, z]));
    }
    for (path, at) in set.iter().map(|(_, path, at)| (path.clone(), *at)).collect::<Vec<_>>() {
        let name = set.iter().find(|x| x.1 == path).map(|x| x.0.clone()).expect(OK);
        let mine: Vec<Connector> = plugs
            .iter()
            .filter(|p| p.0 == name)
            .map(|(_, label, kind, domain, flow, to)| Connector { name: label.to_string(), kind: *kind, domain: *domain, flow: *flow, at: Val::known([tidy(to[0] - at[0]), tidy(to[1] - at[1]), tidy(to[2] - at[2])], "ft", "where the plan shows the run arriving"), size: None })
            .collect();
        if !mine.is_empty() {
            job.put(&path, Connectors(mine)).expect(OK);
        }
    }
    assert!(plugs.iter().all(|p| set.iter().any(|x| x.0 == p.0)), "every connector in the sample is on a thing that is in it");

    job.derive();
    job
}

/// The sample roof with a made-up estimate on it: two materials with prices from three
/// suppliers, planes made of one of them with panels, underlayment, shop work and site
/// work on each, a plane done by a subcontractor, a foreman on the job, and one plane
/// left as the sample has it (a line with no price and no class). Every price, rate and
/// supplier here is made up. Overhead and profit are not entered.
pub fn priced() -> Job {
    use std::collections::BTreeMap;
    const MADE_UP: &str = "made-up price";
    const RATE: &str = "made-up rate";
    let mut job = sample();
    let named = |job: &mut Job, kind: Kind, name: &str| -> String {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).find(|(n, k)| **k == kind && n.name == name).map(|(n, _)| n.path.clone()).expect(OK)
    };
    let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).expect(OK);

    // Copper is sold by the pound, and its weight is its name: 16 oz to the SF is 1 lb.
    let material = |job: &mut Job, name: &str, lb_per_sf: f64, price: f64, quotes: &[(&str, f64)]| {
        let m = job.add(&root, Kind::Material, name).expect(OK);
        let supply = Supply {
            order: Some(Order { unit: "lb".into(), factor: Val::known(lb_per_sf, "lb/SF", "its weight is its name"), round_up: false }),
            price: Val::known(price, "$/lb", MADE_UP),
            quotes: quotes.iter().map(|(who, p)| (who.to_string(), Val::known(*p, "$/lb", "made-up quote"))).collect::<BTreeMap<_, _>>(),
        };
        job.put(&m, supply).expect(OK);
    };
    material(&mut job, "Copper, 16 oz", 1.0, 9.00, &[("Harbor Metals", 8.80), ("Ridge Supply", 9.25), ("Tinners Yard", 9.10)]);
    material(&mut job, "Copper, 20 oz", 1.25, 9.00, &[("Harbor Metals", 8.90), ("Ridge Supply", 9.30)]);

    let code = |job: &mut Job, path: &str| {
        let e = job.find(path).expect(OK);
        job.world.get::<Cost>(e).map(|c| c.code.clone()).expect(OK)
    };
    let hours = |per: &str, rate: f64| Some(Order { unit: HOURS.into(), factor: Val::known(rate, per, RATE), round_up: false });
    let made_of = |name: &str| MadeOf(Val::known(name.to_string(), "", "sample finish schedule"));

    for (plane, waste) in [("south", true), ("east", true)] {
        let p = named(&mut job, Kind::RoofPlane, plane);
        let lines = vec![
            CostLine { waste: waste.then(|| Val::known(10.0, "%", "made-up waste")), ..CostLine::of_material("Standing seam panels", "surface_area") },
            CostLine { class: Some(Class::Material), ..CostLine::of("Underlayment", "surface_area", Val::known(0.90, "$/SF", MADE_UP)) },
            CostLine { class: Some(Class::Fabrication), order: hours("SF/hr", 100.0), ..CostLine::of("Fabricate panels", "surface_area", Val::known(60.0, "$/hr", RATE)) },
            CostLine { class: Some(Class::Labor), order: hours("SF/hr", 25.0), ..CostLine::of("Install panels", "surface_area", Val::known(72.0, "$/hr", RATE)) },
        ];
        let cost = Cost { code: code(&mut job, &p), lines };
        job.put(&p, cost).expect(OK);
        job.put(&p, made_of("Copper, 16 oz")).expect(OK);
    }
    // The west plane is somebody else's work: a quantity and a price.
    let west = named(&mut job, Kind::RoofPlane, "west");
    let cost = Cost { code: code(&mut job, &west), lines: vec![CostLine { class: Some(Class::Subcontract), ..CostLine::of("Standing seam roofing, by subcontractor", "surface_area", Val::known(41.0, "$/SF", MADE_UP)) }] };
    job.put(&west, cost).expect(OK);
    // The porch is heavier copper. Its pitch is assumed, so everything on it is.
    let shed = named(&mut job, Kind::RoofPlane, "shed");
    let cost = Cost { code: code(&mut job, &shed), lines: vec![CostLine::of_material("Standing seam panels", "surface_area")] };
    job.put(&shed, cost).expect(OK);
    job.put(&shed, made_of("Copper, 20 oz")).expect(OK);
    let headwall = named(&mut job, Kind::Headwall, "headwall");
    let lines = vec![
        CostLine { stretch_out: Some(Val::known(Len::One(24.0), "in", "sample detail 4/A-501")), ..CostLine::of_material("Headwall flashing", "true_length * stretch_out") },
        CostLine { class: Some(Class::Labor), order: hours("hr/LF", 0.25), ..CostLine::of("Install headwall flashing", "true_length", Val::known(72.0, "$/hr", RATE)) },
    ];
    let cost = Cost { code: code(&mut job, &headwall), lines };
    job.put(&headwall, cost).expect(OK);
    job.put(&headwall, made_of("Copper, 20 oz")).expect(OK);
    // On the job itself: a foreman, five days of eight hours.
    let lines = vec![CostLine { class: Some(Class::Supervision), order: hours("hr/EA", 8.0), ..CostLine::of("Foreman, days on site", "5", Val::known(95.0, "$/hr", RATE)) }];
    job.put(&root, Cost { code: Val::unknown("", "not coded"), lines }).expect(OK);

    // A roofing job read by division: the roof is Division 7, and running the job is
    // Division 1. (It had none: every line of it stood in no division.)
    for e in job.order.clone() {
        let on_job = e == job.root();
        if let Some(mut cost) = job.world.get_mut::<Cost>(e) {
            for l in cost.lines.iter_mut() {
                l.division = Some(if on_job { Division::GeneralRequirements } else { Division::ThermalAndMoistureProtection });
            }
        }
    }

    let stale = job.derive();
    debug_assert!(!stale.is_empty());
    job
}

/// The sample whole house with a made-up price on everything in it, in all sixteen
/// divisions: what a whole-house estimate looks like when it is read division by
/// division. Every price is made up and says so. The lines are put on by the same verbs
/// a person uses; nothing here is priced that the house does not hold.
pub fn house_priced() -> Job {
    const MADE_UP: &str = "made-up price";
    let mut job = house();
    let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).expect(OK);
    let put = |job: &mut Job, path: &str, line: &str, quantity: &str, class: Class, division: u8, price: &str| {
        job.add_line(path, line, quantity, Some(class), false).expect(OK);
        job.set_on_line(path, "unit_cost", Some(line), Some(price), Status::Known, MADE_UP).expect(OK);
        job.set_division(path, Some(line), Division::of_number(division).expect(OK)).expect(OK);
    };
    use Class::{General, Material, Subcontract};

    // Everything in the house, by what it is and what it is called.
    let things: Vec<(String, String, Kind, Option<String>)> = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let all: Vec<(String, String, Kind, Option<String>)> = q.iter(&job.world).map(|(n, k)| (n.path.clone(), n.name.clone(), *k, n.parent.clone())).collect();
        job.order.clone().iter().filter_map(|e| job.world.get::<Node>(*e).map(|n| n.path.clone())).filter_map(|p| all.iter().find(|t| t.0 == p).cloned()).collect()
    };
    let lowest = crate::building::storey_totals(&mut job).into_iter().min_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0))).map(|s| s.path);
    let mechanical = ["register", "return grille", "water closet", "tub", "lavatory", "kitchen sink", "water heater", "heat pump", "air handler"];

    for (path, name, kind, parent) in &things {
        let on_ground = parent.is_some() && *parent == lowest;
        match kind {
            // An outside wall is framed, wrapped, insulated, sided, boarded and painted;
            // on the ground level it has a stone wainscot. A partition is framed and
            // boarded and painted both sides.
            Kind::Wall if name.ends_with(" wall") && name != "wet wall" => {
                put(&mut job, path, "Wall framing, 2x6 at 16 in", "face_area", Material, 6, "$4.20/SF");
                put(&mut job, path, "Sheathing and weather barrier", "net_area", Material, 7, "$2.10/SF");
                put(&mut job, path, "Batt insulation", "net_area", Material, 7, "$1.60/SF");
                put(&mut job, path, "Cedar lap siding, installed", "net_area", Material, 7, "$9.50/SF");
                put(&mut job, path, "Gypsum board, hung and finished", "net_area", Material, 9, "$2.40/SF");
                put(&mut job, path, "Paint, two coats", "net_area", Material, 9, "$1.10/SF");
                if on_ground {
                    put(&mut job, path, "Stone veneer wainscot, 3 ft", "plan_length", Subcontract, 4, "$95/LF");
                }
            }
            Kind::Wall => {
                put(&mut job, path, "Partition framing, 2x4 at 16 in", "face_area", Material, 6, "$3.10/SF");
                put(&mut job, path, "Gypsum board, both sides", "net_area", Material, 9, "$4.80/SF");
                put(&mut job, path, "Paint, both sides", "net_area", Material, 9, "$2.20/SF");
            }
            Kind::Door => {
                // The front door is the one door in an outside wall; the rest are inside.
                if name == "front door" {
                    put(&mut job, path, "Entry door, mahogany, six panel, prehung", "1", Material, 8, "$2,400");
                } else {
                    put(&mut job, path, "Interior door, six panel, prehung", "1", Material, 8, "$640");
                }
                put(&mut job, path, "Door hardware", "1", Material, 8, "$185");
                put(&mut job, path, "Casing and trim", "1", Material, 6, "$120");
            }
            Kind::Window => {
                put(&mut job, path, "Window unit, clad wood, double-hung, divided lites, bronze", "1", Material, 8, "$1,150");
                put(&mut job, path, "Flashing and sealant", "1", Material, 7, "$65");
                put(&mut job, path, "Window treatment", "1", Material, 12, "$380");
            }
            Kind::Floor if on_ground => {
                put(&mut job, path, "Excavation and gravel base", "plan_area", Subcontract, 2, "$1.85/SF");
                put(&mut job, path, "Concrete slab on grade, 4 in", "plan_area", Subcontract, 3, "$6.50/SF");
            }
            Kind::Floor => put(&mut job, path, "Floor framing and subfloor", "plan_area", Material, 6, "$8.75/SF"),
            Kind::Room => {
                if name == "bath" {
                    put(&mut job, path, "Tile flooring", "floor_area", Material, 9, "$18/SF");
                    put(&mut job, path, "Toilet and bath accessories", "1", Material, 10, "$450");
                } else {
                    put(&mut job, path, "Hardwood flooring", "floor_area", Material, 9, "$12/SF");
                }
                put(&mut job, path, "Base and trim", "perimeter", Material, 6, "$7.50/LF");
                put(&mut job, path, "Ceiling, gypsum board and paint", "floor_area", Material, 9, "$3.40/SF");
                match name.as_str() {
                    "kitchen" => {
                        put(&mut job, path, "Cabinets and countertops", "1", Material, 6, "$18,500");
                        put(&mut job, path, "Kitchen appliances", "1", Material, 11, "$8,500");
                    }
                    "utility" => put(&mut job, path, "Laundry appliances", "1", Material, 11, "$2,400"),
                    "hall closet" => put(&mut job, path, "Closet shelving", "1", Material, 10, "$650"),
                    _ => {}
                }
            }
            Kind::RoofPlane => {
                put(&mut job, path, "Roof sheathing", "surface_area", Material, 6, "$2.30/SF");
                put(&mut job, path, "Standing seam roofing", "surface_area", Material, 7, "$14/SF");
            }
            Kind::Eave => put(&mut job, path, "Gutter", "true_length", Material, 7, "$16.50/LF"),
            Kind::Ridge => {
                put(&mut job, path, "Steel ridge beam", "true_length", Material, 5, "$42/LF");
                put(&mut job, path, "Ridge cap and vent", "true_length", Material, 7, "$22/LF");
            }
            Kind::Hip => put(&mut job, path, "Hip cap", "true_length", Material, 7, "$22/LF"),
            Kind::Fitting => {
                let division = if job.kind(path).is_some() && name.to_lowercase().contains("conduit") { 16 } else { 15 };
                put(&mut job, path, "Fitting, installed", "1", Subcontract, division, "$38");
            }
            Kind::Duct => put(&mut job, path, "Ductwork, installed", "true_length", Subcontract, 15, "$18/LF"),
            Kind::Pipe => put(&mut job, path, "Piping, installed", "true_length", Subcontract, 15, "$9.50/LF"),
            Kind::Conduit => put(&mut job, path, "Conduit, installed", "true_length", Subcontract, 16, "$7.25/LF"),
            Kind::Cable => put(&mut job, path, "Cable, pulled", "true_length", Subcontract, 16, "$2.25/LF"),
            Kind::Fixture | Kind::Equipment => {
                let division = if mechanical.iter().any(|m| name.starts_with(m)) { 15 } else { 16 };
                let price = match name.as_str() {
                    n if n.starts_with("register") || n.starts_with("return grille") => "$85",
                    n if n.starts_with("receptacle") => "$145",
                    n if n.starts_with("light") => "$185",
                    "water heater" => "$2,100",
                    "heat pump" => "$7,800",
                    "air handler" => "$4,300",
                    "panel" => "$2,900",
                    "meter" => "$650",
                    _ => "$640",
                };
                put(&mut job, path, "Set and connected", "1", Subcontract, division, price);
            }
            _ => {}
        }
    }
    // On the job itself: what it takes to run it, the site, the foundation, and the
    // two things a house this size might have that no element here draws yet.
    put(&mut job, &root, "Building permit", "1", General, 1, "$4,200");
    put(&mut job, &root, "Temporary power and toilets", "1", General, 1, "$1,800");
    put(&mut job, &root, "Final cleaning", "1", General, 1, "$1,250");
    put(&mut job, &root, "Site clearing and grading", "1", Subcontract, 2, "$6,500");
    put(&mut job, &root, "Driveway and walks", "1", Subcontract, 2, "$9,800");
    put(&mut job, &root, "Footings and foundation walls", "1", Subcontract, 3, "$24,600");
    put(&mut job, &root, "Sauna, prefabricated", "1", Subcontract, 13, "$9,200");
    put(&mut job, &root, "Residential elevator, two stops", "1", Subcontract, 14, "$38,000");
    // The superintendent is in-house and in hours: sixty days of eight.
    let e = job.find(&root).expect(OK);
    let mut cost = job.world.get::<Cost>(e).cloned().expect(OK);
    cost.lines.insert(
        0,
        CostLine {
            class: Some(Class::Supervision),
            division: Some(Division::GeneralRequirements),
            order: Some(Order { unit: HOURS.into(), factor: Val::known(8.0, "hr/EA", "made-up rate"), round_up: false }),
            ..CostLine::of("Superintendent, days on site", "60", Val::known(85.0, "$/hr", "made-up rate"))
        },
    );
    job.put(&root, cost).expect(OK);
    job.set(&root, "overhead", Some("10"), Status::Known, MADE_UP).expect(OK);
    job.set(&root, "profit", Some("8"), Status::Known, MADE_UP).expect(OK);
    job.derive();
    job
}
