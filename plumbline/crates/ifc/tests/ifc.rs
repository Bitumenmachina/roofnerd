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
    // Call 50: what the model calls a kind of thing is said in words.
    assert_eq!(left, [("beam", 1), ("furniture", 1), ("stair", 1)]);
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
    assert_eq!(door.offset.source, "the designer's model (made up): made-up-house.ifc, \"Front door\", door 113 in the file");
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
    assert!(b.left_out.iter().any(|(c, n, why)| c == "wall" && *n == 4 && why.contains("not a plain shape")), "{:?}", b.left_out);
    assert!(b.left_out.iter().any(|(c, _, why)| c == "door" && why.contains("fills no hole in a wall that was brought in")), "{:?}", b.left_out);
    assert_eq!(b.made.get("walls"), None);
    assert_eq!(b.made.get("floors"), Some(&1));
    let _ = before;
}

// ---- The fifteenth judgment, part A. Each is the made-up model with one thing changed.

/// The text of the thing numbered `id` in a model's file, after its `#id=`.
fn thing(text: &str, id: &str) -> String {
    let start = format!("{id}=");
    text.lines().find(|l| l.starts_with(&start)).unwrap_or_else(|| panic!("no {id}"))[start.len()..].trim_end_matches(';').to_string()
}

/// What a thing is given, in order: the parts between its outer brackets, cut at the
/// commas that are not inside inner brackets or quotes.
fn given(thing: &str) -> Vec<String> {
    let inner = &thing[thing.find('(').unwrap() + 1..thing.rfind(')').unwrap()];
    let (mut parts, mut depth, mut quoted, mut part) = (Vec::new(), 0, false, String::new());
    for c in inner.chars() {
        match c {
            '\'' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                parts.push(std::mem::take(&mut part));
                continue;
            }
            _ => {}
        }
        part.push(c);
    }
    parts.push(part);
    parts
}

/// The number of the first thing of a class in the file, and the file with one of its
/// lines swapped for another.
fn first(text: &str, class: &str) -> String {
    let l = text.lines().find(|l| l.contains(&format!("={class}("))).unwrap_or_else(|| panic!("no {class}"));
    l[..l.find('=').unwrap()].to_string()
}

fn swapped(text: &str, id: &str, new: &str) -> String {
    let start = format!("{id}=");
    text.lines().map(|l| if l.starts_with(&start) { format!("{id}={new};") } else { l.to_string() }).collect::<Vec<_>>().join("\n") + "\n"
}

fn text() -> String {
    String::from_utf8(model()).unwrap()
}

const HOW: Entered = Entered { status: Status::Assumed, source: "the designer's model (made up)" };

fn fresh() -> Job {
    Job::new("brought in (not a real job)", "test", "test")
}

#[test]
fn j15_3_a_building_placed_at_a_height_does_not_lift_its_walls_off_their_floor() {
    // The site is placed 50 m up. Everything in the model is placed from it.
    let t = text();
    let site_at = given(&thing(&t, &given(&thing(&t, &first(&t, "IFCSITE")))[5]))[1].clone();
    let point = given(&thing(&t, &site_at))[0].clone();
    let lifted = swapped(&t, &point, "IFCCARTESIANPOINT((0.0,0.0,50.0))");
    let mut job = fresh();
    let b = import(&mut job, lifted.as_bytes(), "lifted.ifc", HOW).unwrap();
    // 50 m is 164.042 ft. The storeys still say their floors are at 0 and at 8 ft.
    assert!(b.notes.iter().any(|n| n.contains("2 of the model's storeys are placed 164.042 ft from where each says its floor is")), "{:?}", b.notes);
    job.derive();
    let mut q = job.world.query::<(&Node, &building::Level)>();
    let mut floors: Vec<f64> = q.iter(&job.world).map(|(_, l)| l.elevation.feet().unwrap()).collect();
    floors.sort_by(f64::total_cmp);
    assert_eq!(floors, [0.0, 8.0]);
    // No wall is lifted off its floor, and the figures are those of the model on the ground.
    let mut q = job.world.query::<(&Node, &building::Wall, &Measure)>();
    let walls: Vec<(Option<f64>, f64)> = q.iter(&job.world).map(|(_, w, m)| (w.base_offset.as_ref().and_then(|o| o.feet()), m.0["net_area"].value.unwrap())).collect();
    assert_eq!(walls.len(), 4);
    assert!(walls.iter().all(|(offset, _)| offset.is_none()), "{walls:?}");
    let mut nets: Vec<f64> = walls.iter().map(|w| w.1).collect();
    nets.sort_by(f64::total_cmp);
    assert_eq!(nets, [148.0, 160.0, 219.0, 240.0]);
    // The duct is 7 ft above the ground floor, as it was.
    let mut q = job.world.query::<(&Kind, &building::Segment)>();
    let duct = q.iter(&job.world).find(|(k, _)| **k == Kind::Duct).unwrap().1.offset.feet();
    assert_eq!(duct, Some(7.0));
}

#[test]
fn j15_11_12_22_what_is_not_read_is_listed_and_counted() {
    let t = text();
    // The south wall's body given twice over: a body of two pieces.
    let wall = first(&t, "IFCWALL");
    let shape = given(&thing(&t, &wall))[6].clone();
    let rep_list = given(&thing(&t, &shape))[2].clone();
    let rep = rep_list.trim_matches(|c| c == '(' || c == ')').to_string();
    let parts = given(&thing(&t, &rep));
    let solid = parts[3].trim_matches(|c| c == '(' || c == ')').to_string();
    let two = swapped(&t, &rep, &format!("IFCSHAPEREPRESENTATION({},{},{},({solid},{solid}))", parts[0], parts[1], parts[2]));
    // The beam given no place.
    let beam = first(&two, "IFCBEAM");
    let mut b_parts = given(&thing(&two, &beam));
    b_parts[5] = "$".to_string();
    let two = swapped(&two, &beam, &format!("IFCBEAM({})", b_parts.join(",")));
    // And the chair's number used a second time, for another chair.
    let chair = first(&two, "IFCFURNITURE");
    let again = two.replace("ENDSEC;\nEND-ISO", &format!("{chair}={};\nENDSEC;\nEND-ISO", thing(&two, &chair).replace("'Chair'", "'Another chair'")));
    assert_ne!(again, two);
    let mut job = fresh();
    let b = import(&mut job, again.as_bytes(), "odd.ifc", HOW).unwrap();
    assert_eq!(b.made.get("walls"), Some(&3));
    let left: Vec<(&str, usize, &str)> = b.left_out.iter().map(|(c, n, why)| (c.as_str(), *n, why.as_str())).collect();
    assert!(left.contains(&("wall", 1, "its body is made of more than one piece: only a body of one piece is read")), "{left:?}");
    assert!(left.contains(&("beam", 1, "the model gives it no place, and this program has no element for it")), "{left:?}");
    // The door that was in that wall is listed too: nothing is dropped unsaid.
    assert!(left.iter().any(|(c, n, why)| *c == "door" && *n == 1 && why.contains("fills no hole in a wall that was brought in")), "{left:?}");
    assert!(b.notes.iter().any(|n| n.starts_with("1 number in the file is used by more than one thing")), "{:?}", b.notes);
}

#[test]
fn j15_21_another_model_stands_on_the_jobs_own_storeys() {
    let mut job = fresh();
    import(&mut job, &model(), "made-up-house.ifc", HOW).unwrap();
    let storeys = |job: &mut Job| {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).filter(|(_, k)| **k == Kind::Storey).count()
    };
    assert_eq!(storeys(&mut job), 2);
    // The same model again: everything in it is in the job already. Nothing is made
    // twice; it is refused, each thing with why, and the job is as it was.
    let before = save(&mut job);
    let why = import(&mut job, &model(), "made-up-house.ifc", HOW).unwrap_err();
    assert!(why.starts_with("nothing in the model but its storeys could be brought in"), "{why}");
    for said in ["a duct in this very place is already on this storey", "a pipe in this very place is already on this storey", "a floor with this very outline is already on this storey", "one wall drawn over another would be counted twice"] {
        assert!(why.contains(said), "{said}: {why}");
    }
    assert_eq!(save(&mut job), before);
    // With the duct taken out of the job, the model brought in again, as another
    // trade's model of the same house would come: the duct stands on the job's own
    // ground storey, and no storey is made.
    let duct = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).find(|(_, k)| **k == Kind::Duct).map(|(n, _)| n.path.clone()).unwrap()
    };
    plumbline_world::editing::remove(&mut job, &duct).unwrap();
    let b = import(&mut job, &model(), "made-up-house.ifc", HOW).unwrap();
    assert_eq!(storeys(&mut job), 2);
    assert_eq!((b.made.get("storeys"), b.made.get("ducts"), b.made.len()), (None, Some(&1), 1));
    assert!(b.notes.iter().any(|n| n.contains("the job already has \"Ground\", \"Roof\"")), "{:?}", b.notes);
    for kind in [Kind::Wall, Kind::Floor, Kind::Duct, Kind::Pipe] {
        let mut q = job.world.query::<&Kind>();
        assert_eq!(q.iter(&job.world).filter(|k| **k == kind).count(), if kind == Kind::Wall { 4 } else { 1 }, "{kind:?}");
    }
    let mut q = job.world.query::<(&Node, &Kind)>();
    let (ground, on) = (q.iter(&job.world).find(|(n, k)| **k == Kind::Storey && n.name == "Ground").map(|(n, _)| n.path.clone()), q.iter(&job.world).find(|(_, k)| **k == Kind::Duct).and_then(|(n, _)| n.parent.clone()));
    assert_eq!(ground, on);
    // A job whose storey of that name is at another height: refused, with both heights.
    let mut other = fresh();
    plumbline_world::editing::storey(&mut other, "Ground", "1 ft", "9 ft", Entered { status: Status::Known, source: "made-up survey" }).unwrap();
    let before = save(&mut other);
    let why = import(&mut other, &model(), "made-up-house.ifc", HOW).unwrap_err();
    assert!(why.contains("already has a storey called \"Ground\" with its floor at 1 ft") && why.contains("has its floor at 0 ft"), "{why}");
    assert_eq!(save(&mut other), before);
    // Two storeys of one name in one model: the later is named with its height.
    let t = text();
    let roof = t.lines().find(|l| l.contains("=IFCBUILDINGSTOREY(") && l.contains("'Roof'")).unwrap().to_string();
    let twins = t.replace(&roof, &roof.replace("'Roof'", "'Ground'"));
    let mut job = fresh();
    let b = import(&mut job, twins.as_bytes(), "twins.ifc", HOW).unwrap();
    assert!(b.notes.iter().any(|n| n.contains("more than one storey called \"Ground\"") && n.contains("\"Ground, at 8 ft\"")), "{:?}", b.notes);
    assert_eq!(storeys(&mut job), 2);
}

#[test]
fn j15_25_a_wall_that_leans_is_not_upright_and_things_keep_the_models_names() {
    let t = text();
    let wall = first(&t, "IFCWALL");
    let shape = given(&thing(&t, &wall))[6].clone();
    let rep = given(&thing(&t, &shape))[2].trim_matches(|c| c == '(' || c == ')').to_string();
    let solid = given(&thing(&t, &rep))[3].trim_matches(|c| c == '(' || c == ')').to_string();
    let way = given(&thing(&t, &solid))[2].clone();
    // Two degrees off upright: 3 3/8 in out in 8 ft.
    let leaning = swapped(&t, &way, "IFCDIRECTION((0.0349,0.0,0.99939))");
    let mut job = fresh();
    let b = import(&mut job, leaning.as_bytes(), "leaning.ifc", HOW).unwrap();
    assert!(b.left_out.iter().any(|(c, n, why)| c == "wall" && *n == 1 && why == "it does not stand upright"), "{:?}", b.left_out);
    // The three that stand are called what the model calls them.
    let mut q = job.world.query::<(&Node, &Kind)>();
    let mut names: Vec<String> = q.iter(&job.world).filter(|(_, k)| matches!(**k, Kind::Wall | Kind::Floor | Kind::Duct | Kind::Pipe)).map(|(n, _)| n.name.clone()).collect();
    names.sort();
    assert_eq!(names, ["Cold water pipe", "East wall", "Floor slab", "North wall", "Supply duct", "West wall"]);
}
