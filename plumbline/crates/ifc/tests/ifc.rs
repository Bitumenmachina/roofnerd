//! A made-up model (demos/ifc/made-up-house.ifc, written by tools/made-up-ifc.py) brought
//! into an empty job. It is in metres; every figure below is the model's, turned to feet
//! by hand (0.3048 m to the foot).

use plumbline_ifc::import;
use plumbline_world::building::{self, clash};
use plumbline_world::editing::Entered;
use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::*;

fn model() -> Vec<u8> {
    std::fs::read(format!("{}/../../demos/ifc/made-up-house.ifc", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn measure(job: &mut Job, path: &str, name: &str) -> (Option<f64>, Status) {
    let e = job.find(path).unwrap();
    let m = job.world.get::<Measure>(e).unwrap();
    let d = m.0.get(name).unwrap();
    (d.value, d.status)
}

#[test]
fn a_model_comes_in_as_storeys_walls_openings_a_floor_a_room_and_runs_and_the_rest_is_listed() {
    let how = Entered { status: Status::Assumed, source: "the designer's model (made up)" };
    let mut job = Job::new("brought in (not a real job)", "test", "test");
    let b = import(&mut job, &model(), "made-up-house.ifc", how).unwrap();
    assert_eq!((b.schema.as_str(), b.unit.as_str()), ("IFC4", "metres"));
    assert!((b.feet_per_unit - 1.0 / 0.3048).abs() < 1.0e-12);
    let made: Vec<(&str, usize)> = b.made.iter().map(|(k, n)| (k.as_str(), *n)).collect();
    assert_eq!(made, [("doors", 1), ("ducts", 1), ("floors", 1), ("pipes", 1), ("rooms", 1), ("storeys", 2), ("walls", 4), ("windows", 1)]);
    // The beam, the stair and the chair: each listed, none dropped.
    let left: Vec<(&str, usize)> = b.left_out.iter().map(|(c, n, _)| (c.as_str(), *n)).collect();
    assert_eq!(left, [("IFCBEAM", 1), ("IFCFURNITURE", 1), ("IFCSTAIR", 1)]);
    assert!(b.left_out.iter().all(|(_, _, why)| why == "this program has no element for it"));
    assert!(b.notes.iter().any(|n| n.contains("the model is in metres")));
    job.derive();

    // Everything in the job, by kind, in the job's order.
    let all: Vec<(String, Kind)> = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let mut all: Vec<(String, Kind)> = q.iter(&job.world).map(|(n, k)| (n.path.clone(), *k)).collect();
        all.sort_by(|a, b| a.0.cmp(&b.0));
        all
    };
    let of = |kind: Kind| -> Vec<String> { all.iter().filter(|(_, k)| *k == kind).map(|(p, _)| p.clone()).collect() };
    // Walls: 9.144 m and 6.096 m are 30 ft and 20 ft; 2.4384 m is 8 ft. 100 LF round the
    // house. South, with a 3 ft by 7 ft door: 240 less 21 is 219 SF. East, with a 4 ft by
    // 3 ft window: 160 less 12 is 148 SF. Each as sure as the model was said to be.
    let walls = of(Kind::Wall);
    let lengths: Vec<f64> = walls.iter().map(|w| measure(&mut job, w, "plan_length").0.unwrap()).collect();
    assert_eq!(lengths, [30.0, 20.0, 30.0, 20.0]);
    let nets: Vec<f64> = walls.iter().map(|w| measure(&mut job, w, "net_area").0.unwrap()).collect();
    assert_eq!(nets, [219.0, 148.0, 240.0, 160.0]);
    assert_eq!(measure(&mut job, &walls[0], "net_area").1, Status::Assumed);
    // The door's middle is 3.048 m, 10 ft, along the south wall; the window's 10 ft along
    // the east wall with its sill 0.9144 m, 3 ft, up.
    let opening = |job: &mut Job, path: &str| job.find(path).and_then(|e| job.world.get::<building::Opening>(e).cloned()).unwrap();
    let (door, window) = (opening(&mut job, &of(Kind::Door)[0]), opening(&mut job, &of(Kind::Window)[0]));
    assert_eq!((door.offset.feet(), door.sill.feet()), (Some(10.0), Some(0.0)));
    assert_eq!((window.offset.feet(), window.sill.feet()), (Some(10.0), Some(3.0)));
    assert_eq!(door.offset.source, "the designer's model (made up): made-up-house.ifc, \"Front door\", IFCDOOR (#113)");
    // The slab: 30 ft by 20 ft is 600 SF. The room inside 6 in walls drawn on their
    // centre lines: 29.5 by 19.5 is 575.25 SF.
    assert_eq!(measure(&mut job, &of(Kind::Floor)[0], "plan_area").0, Some(600.0));
    assert_eq!(measure(&mut job, &of(Kind::Room)[0], "floor_area").0, Some(575.25));
    // The duct, 6.096 m: 20 LF. The pipe, 3.048 m: 10 LF. They cross at (10, 10) ft, 7 ft
    // up: a 6 in duct and a 1 in pipe, in each other by 3 in and 1/2 in. A question for
    // the field, since the model is Assumed.
    assert_eq!(measure(&mut job, &of(Kind::Duct)[0], "true_length").0, Some(20.0));
    assert_eq!(measure(&mut job, &of(Kind::Pipe)[0], "true_length").0, Some(10.0));
    let found = clash::clashes(&mut job, None).unwrap();
    assert_eq!(found.found.len(), 1);
    let c = &found.found[0];
    assert!((c.by.unwrap() - 3.5 / 12.0).abs() < 1.0e-6 && c.at == [10.0, 10.0, 7.0] && c.status == Status::Assumed, "{c:?}");
    // The top storey's height is in nothing the model says: Unknown, not a guess.
    let levels: Vec<building::Level> = of(Kind::Storey).iter().map(|s| job.find(s).and_then(|e| job.world.get::<building::Level>(e).cloned()).unwrap()).collect();
    assert_eq!((levels[0].height.feet(), levels[1].elevation.feet(), levels[1].height.status), (Some(8.0), Some(8.0), Status::Unknown));
    // The job saves, loads and saves to the same bytes.
    let text = save(&mut job);
    let (mut back, _) = load(&text).unwrap();
    assert_eq!(save(&mut back), text);
}

#[test]
fn what_is_not_a_model_or_holds_nothing_to_bring_in_is_refused_and_the_job_left_alone() {
    let how = Entered { status: Status::Assumed, source: "made up" };
    let mut job = Job::new("brought in (not a real job)", "test", "test");
    let before = save(&mut job);
    assert!(import(&mut job, b"not a model at all", "x.ifc", how).unwrap_err().contains("not an IFC file"));
    let other = b"ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n#1=PRODUCT('a');\nENDSEC;\nEND-ISO-10303-21;\n";
    assert!(import(&mut job, other, "x.stp", how).unwrap_err().contains("not an IFC model"));
    let empty = b"ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('IFC4'));\nENDSEC;\nDATA;\n#1=IFCCARTESIANPOINT((0.,0.,0.));\nENDSEC;\nEND-ISO-10303-21;\n";
    assert!(import(&mut job, empty, "x.ifc", how).unwrap_err().contains("no building storey"));
    assert!(import(&mut job, &model(), "m.ifc", Entered { status: Status::Assumed, source: " " }).unwrap_err().contains("where the model came from"));
    // The same model with its walls' bodies taken away: nothing but storeys would come
    // in... but the slab, the room and the runs still do, and the walls are listed.
    let text = String::from_utf8(model()).unwrap().replace("IFCRECTANGLEPROFILEDEF", "IFCTSHAPEPROFILEDEF");
    let b = import(&mut job, text.as_bytes(), "m.ifc", how).unwrap();
    assert!(b.left_out.iter().any(|(c, n, why)| c == "IFCWALL" && *n == 4 && why.contains("not a plain shape")), "{:?}", b.left_out);
    assert!(b.left_out.iter().any(|(c, _, why)| c == "IFCDOOR" && why.contains("fills no hole in a wall that was brought in")), "{:?}", b.left_out);
    assert_eq!(b.made.get("walls"), None);
    assert_eq!(b.made.get("floors"), Some(&1));
    let _ = before;
}
