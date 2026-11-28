use plumbline_world::file::{load, save};
use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;
use serde_json::Value;

pub fn entries(text: &str) -> Vec<(String, Value)> {
    let v: Value = serde_json::from_str(text).unwrap();
    v["data"].as_array().unwrap().iter().map(|e| (e["path"].as_str().unwrap().to_string(), e.clone())).collect()
}

pub fn touched(before: &str, after: &str) -> Vec<String> {
    let (a, b) = (entries(before), entries(after));
    assert_eq!(a.len(), b.len());
    a.iter().zip(&b).filter(|(x, y)| x.1 != y.1).map(|(x, _)| x.0.clone()).collect()
}

pub fn measure(job: &mut Job, path: &str, name: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0[name].clone()
}

pub fn edit(text: &str, f: impl Fn(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(text).unwrap();
    f(&mut v);
    serde_json::to_string(&v).unwrap()
}

pub fn entry<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
    v["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == path).unwrap()
}

pub fn attrs<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
    &mut entry(v, path)["attributes"]
}

pub fn show_rows(job: &mut Job, path: &str) -> Vec<(String, String, Status)> {
    plumbline_world::view::elements(job).into_iter().find(|e| e.path == path).unwrap().rows.into_iter().map(|r| (r.label, r.text, r.status)).collect()
}

pub fn feet_of(typed: &str) -> f64 {
    let (len, unit) = units::parse_length(typed).unwrap_or_else(|e| panic!("{typed:?} refused: {e}"));
    len.feet(unit).unwrap()
}

pub fn porch(v: &mut Value, points: Value, eave: Value) {
    attrs(v, "plane-0017")["pl::outline"]["points"]["value"] = points;
    attrs(v, "plane-0017")["pl::outline"]["eave"]["value"] = eave;
}

/// A one-plane job: a trapezoid whose low edge is y = 0 and whose right side is a rake.
pub fn trapezoid(eave: [P2; 2], points: Vec<P2>, lines: &[(Kind, &str, [P2; 2])]) -> Job {
    let mut job = Job::new("Trapezoid", "test", "test");
    let plane = job.add("job-0001", Kind::RoofPlane, "plane").unwrap();
    job.put(&plane, Outline { points: Val::known(points, "ft", "test"), eave: Val::known(eave, "ft", "test"), eave_height: Val::known(Len::One(0.0), "ft", "test") }).unwrap();
    job.put(&plane, Pitch(Val::known(6.0, "in/12", "test"))).unwrap();
    for (kind, name, ends) in lines {
        let p = job.add("job-0001", *kind, name).unwrap();
        job.put(&p, Run { ends: Val::known(*ends, "ft", "test"), planes: vec![plane.clone()] }).unwrap();
    }
    job.derive();
    let (job, _) = load(&save(&mut job)).unwrap();
    job
}

pub fn figures(job: &mut Job, path: &str, line: &str) -> LineFigures {
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[line].clone()
}

/// The sample parapet (20 LF) with the parapet condition applied and nothing entered yet.
pub fn parapet_job() -> Job {
    let mut job = sample();
    let added = job.apply("parapet-0024", "Parapet with copper coping", "sample finish schedule").unwrap();
    assert_eq!(added, ["Copper wall flashing", "Copper coping cap", "Continuous cleat", "Cleat fasteners"]);
    job.derive();
    job
}

/// A job of planes (outline, eave line; all 6:12 from 10 ft) and lines on them, built
/// unchecked and then loaded, so it has passed the entry checks.
pub fn roof_job(planes: &[(Vec<P2>, [P2; 2])], lines: &[(Kind, [P2; 2], &[usize])]) -> Result<Job, String> {
    let mut job = Job::new("Roof", "test", "test");
    let mut paths = Vec::new();
    for (i, (points, eave)) in planes.iter().enumerate() {
        let p = job.add("job-0001", Kind::RoofPlane, &format!("plane-{i}")).unwrap();
        job.put(&p, Outline { points: Val::known(points.clone(), "ft", "test"), eave: Val::known(*eave, "ft", "test"), eave_height: Val::known(Len::One(10.0), "ft", "test") }).unwrap();
        job.put(&p, Pitch(Val::known(6.0, "in/12", "test"))).unwrap();
        paths.push(p);
    }
    for (i, (kind, ends, on)) in lines.iter().enumerate() {
        let p = job.add("job-0001", *kind, &format!("line-{i}")).unwrap();
        job.put(&p, Run { ends: Val::known(*ends, "ft", "test"), planes: on.iter().map(|k| paths[*k].clone()).collect() }).unwrap();
    }
    job.derive();
    load(&save(&mut job)).map(|(job, _)| job)
}

pub fn length_of(job: &mut Job, name: &str) -> Derived {
    let path = job.children()["job-0001"][name].clone();
    measure(job, &path, "true_length")
}

pub fn footprint(points: Vec<P2>, rises: &[Rise], eave_ft: f64) -> Footprint {
    Footprint {
        points: Val::known(points, "ft", "test plan"),
        eave_height: Val::known(Len::One(eave_ft), "ft", "test plan"),
        edges: rises.iter().map(|r| Val::known(*r, if matches!(r, Rise::Gable) { "" } else { "in/12" }, "test plan")).collect(),
    }
}

pub fn child(job: &mut Job, parent: &str, name: &str) -> String {
    job.children()[parent][name].clone()
}

pub fn hip_roof() -> (Job, String) {
    let mut job = sample();
    let (roof_path, _) = solve::new_roof(&mut job, "house", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    (job, roof_path)
}

pub fn measure_of(job: &mut Job, path: &str, name: &str) -> plumbline_world::Derived {
    let e = job.find(path).unwrap();
    job.world.get::<plumbline_world::Measure>(e).unwrap().0.get(name).unwrap().clone()
}

pub fn path_named(job: &mut Job, name: &str) -> String {
    plumbline_world::view::elements(job).into_iter().find(|e| e.name == name).map(|e| e.path).unwrap_or_else(|| panic!("no element named {name}"))
}

/// A door or window type of a size, added to the job for a test.
pub fn opening_type(job: &mut Job, kind: Kind, width: Val<Len>, height: Val<Len>) -> Val<String> {
    let t = job.add("job-0001", Kind::Type, &format!("test type {}", job.order.len())).unwrap();
    job.put(&t, plumbline_world::OpeningType { category: kind, width, height }).unwrap();
    Val::known(t, "", "test")
}

/// The straight pieces a run of the sample house was entered as, in order: the element
/// of that name, or the elements named "<name>, piece 1 of n" and on.
pub fn pieces(job: &mut Job, name: &str) -> Vec<String> {
    let lead = format!("{name}, piece ");
    plumbline_world::view::elements(job).into_iter().filter(|e| e.name == name || e.name.starts_with(&lead)).map(|e| e.path).collect()
}

pub fn length_of_pieces(job: &mut Job, name: &str) -> (f64, Status) {
    let paths = pieces(job, name);
    assert!(!paths.is_empty(), "no run named {name}");
    let figures: Vec<plumbline_world::Derived> = paths.iter().map(|p| measure_of(job, p, "true_length")).collect();
    (plumbline_world::value::round4(figures.iter().map(|d| d.value.expect("a length")).sum()), figures.iter().map(|d| d.status).max().unwrap())
}

pub fn ft(x: f64) -> Val<Len> {
    Val::known(Len::One(x), "ft", "test")
}

pub fn named(path: &str) -> Val<String> {
    Val::known(path.to_string(), "", "test")
}

/// A job with one storey at the datum and a 2 in pipe type, a 1 in pipe type and a duct
/// type, for the network's tests: (job, storey, 2 in pipe, 1 in pipe, duct).
pub fn piping() -> (Job, String, String, String, String) {
    use plumbline_world::{Level, SegmentType};
    let mut job = Job::new("network test (not a real job)", "test", "test");
    let storey = job.add("job-0001", Kind::Storey, "ground").unwrap();
    job.put(&storey, Level { elevation: ft(0.0), height: ft(10.0), computation_height: None }).unwrap();
    let inch = |x: f64| Val::known(Len::One(x), "in", "test");
    let kind = |job: &mut Job, name: &str, t: SegmentType| {
        let p = job.add("job-0001", Kind::Type, name).unwrap();
        job.put(&p, t).unwrap();
        p
    };
    let two = kind(&mut job, "pipe 2 in", SegmentType::round(Kind::Pipe, inch(2.0)));
    let one = kind(&mut job, "pipe 1 in", SegmentType::round(Kind::Pipe, inch(1.0)));
    let duct = kind(&mut job, "duct 6 in", SegmentType::round(Kind::Duct, inch(6.0)));
    (job, storey, two, one, duct)
}

/// One straight piece from `a` to `b`, each [east, north, height above the storey].
pub fn piece(job: &mut Job, storey: &str, kind: Kind, name: &str, type_path: &str, a: [f64; 3], b: [f64; 3]) -> String {
    use plumbline_world::{Justify, Segment};
    let p = job.add(storey, kind, name).unwrap();
    job.put(&p, Segment { type_name: named(type_path), line: Val::known([[a[0], a[1]], [b[0], b[1]]], "ft", "test"), offset: ft(a[2]), end_offset: (a[2] != b[2]).then(|| ft(b[2])), justify: Justify::Centre }).unwrap();
    p
}

/// A placed thing with connectors, each (kind, domain, [x, y, z] in the thing's own axes).
pub fn thing(job: &mut Job, storey: &str, kind: Kind, name: &str, at: [f64; 3], angle: Option<f64>, connectors: &[(plumbline_world::ConnectorKind, Option<plumbline_world::Domain>, [f64; 3])]) -> String {
    use plumbline_world::{Connector, Connectors, Flow, Placed};
    let p = job.add(storey, kind, name).unwrap();
    job.put(&p, Placed { what: named(name), at: Val::known([at[0], at[1]], "ft", "test"), offset: ft(at[2]), angle: angle.map(|a| Val::known(a, "deg", "test")), size: Val::known([1.0, 1.0, 1.0], "ft", "test"), item: None }).unwrap();
    job.put(&p, Connectors(connectors.iter().map(|(kind, domain, at)| Connector { name: String::new(), kind: *kind, domain: *domain, flow: Flow::Both, at: Val::known(*at, "ft", "test"), size: None, facing: None, of_item: None }).collect())).unwrap();
    p
}

pub fn net(job: &mut Job) -> plumbline_world::building::network::Network {
    use plumbline_world::building::network::{network, TOLERANCE_FT};
    network(job, None, None, TOLERANCE_FT).unwrap()
}

pub fn kinds(n: &plumbline_world::building::network::Network) -> Vec<&'static str> {
    n.warnings.iter().map(|w| w.kind).collect()
}

/// A job with one storey at the datum and a 6 in wall type: (job, storey, wall type).
pub fn walled() -> (Job, String, String) {
    use plumbline_world::{Level, WallType};
    let mut job = Job::new("room test (not a real job)", "test", "test");
    let storey = job.add("job-0001", Kind::Storey, "ground").unwrap();
    job.put(&storey, Level { elevation: ft(0.0), height: ft(10.0), computation_height: None }).unwrap();
    let t = job.add("job-0001", Kind::Type, "wall 6 in").unwrap();
    job.put(&t, WallType { thickness: Val::known(Len::One(6.0), "in", "test"), layers: Vec::new(), footing: false }).unwrap();
    (job, storey, t)
}

pub fn wall(job: &mut Job, storey: &str, type_path: &str, name: &str, a: [f64; 2], b: [f64; 2], tall: f64) -> String {
    let w = job.add(storey, Kind::Wall, name).unwrap();
    job.put(&w, plumbline_world::Wall::tall(named(type_path), Val::known([a, b], "ft", "test"), ft(tall))).unwrap();
    w
}

pub fn room(job: &mut Job, storey: &str, name: &str, at: [f64; 2]) -> String {
    let r = job.add(storey, Kind::Room, name).unwrap();
    job.put(&r, plumbline_world::Room { point: Val::known(at, "ft", "test"), number: None }).unwrap();
    r
}

/// Four walls round a rectangle from the origin, drawn counter-clockwise from the
/// south-west corner: [south, east, north, west].
pub fn four_walls(job: &mut Job, storey: &str, type_path: &str, east: f64, north: f64) -> [String; 4] {
    [
        wall(job, storey, type_path, "south", [0.0, 0.0], [east, 0.0], 9.0),
        wall(job, storey, type_path, "east", [east, 0.0], [east, north], 9.0),
        wall(job, storey, type_path, "north", [east, north], [0.0, north], 9.0),
        wall(job, storey, type_path, "west", [0.0, north], [0.0, 0.0], 9.0),
    ]
}

pub fn figures_of(job: &mut Job, room: &str) -> [Option<f64>; 3] {
    ["floor_area", "gross_area", "perimeter"].map(|name| measure_of(job, room, name).value)
}

pub fn priced_of(job: &mut Job, path: &str, name: &str) -> plumbline_world::Derived {
    let e = job.find(path).unwrap();
    job.world.get::<plumbline_world::Priced>(e).unwrap_or_else(|| panic!("{path} has no price")).0.get(name).unwrap_or_else(|| panic!("{path} has no {name}")).clone()
}

pub fn extension_of(job: &mut Job, path: &str, line: &str) -> plumbline_world::Derived {
    let e = job.find(path).unwrap();
    job.world.get::<plumbline_world::LineQuantities>(e).unwrap().0.get(line).unwrap().extension.clone().expect("an extension")
}

/// The sample job priced both ways with made-up prices. Nothing here is from a real job.
/// South plane: 419.2627 SF. Shed: 98.9545 SF, Assumed (its pitch is). Headwall: 12 LF,
/// with a flashing line of 24 SF.
pub fn priced_sample() -> Job {
    let mut job = sample();
    let known = Status::Known;
    job.set("plane-0003", "allowance", Some("$28.00/SF"), known, "made-up budget rate").unwrap();
    job.set("plane-0017", "allowance", Some("28"), known, "made-up budget rate").unwrap();
    job.set("headwall-0021", "allowance", Some("40.00 $/LF"), known, "made-up budget rate").unwrap();
    job.set("plane-0003", "unit_cost", Some("31.50"), known, "made-up quote").unwrap();
    job.set("headwall-0021", "unit_cost", Some("$18.25/SF"), known, "made-up quote").unwrap();
    job.derive();
    job
}

/// A 20 ft by 10 ft box of 6 in walls 10 ft tall, and a room in its west half.
pub fn boxed() -> (Job, String, String, String) {
    let (mut job, s, t) = walled();
    for (name, a, b) in [("south", [0.0, 0.0], [20.0, 0.0]), ("east", [20.0, 0.0], [20.0, 10.0]), ("north", [20.0, 10.0], [0.0, 10.0]), ("west", [0.0, 10.0], [0.0, 0.0])] {
        wall(&mut job, &s, &t, name, a, b, 10.0);
    }
    let r = room(&mut job, &s, "room", [5.0, 5.0]);
    (job, s, t, r)
}

pub fn class_total(est: &view::Estimate, class: Class) -> &view::ClassTotal {
    est.classes.iter().find(|c| c.class == class).unwrap()
}

/// An empty job with a storey, a 6 in wall type, a door type (3 ft by 7 ft) and a window
/// type (4 ft by 3 ft), every value Known. Returns the job and those four paths.
pub fn drawn() -> (Job, String, String, String, String) {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("drawn house (not a real job)", "test", "test");
    let s = ed::storey(&mut job, "ground", "0 ft", "9 ft", how).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", how).unwrap();
    let dt = ed::opening_type(&mut job, "door 3 by 7", Kind::Door, "3 ft", "7 ft", how).unwrap();
    let nt = ed::opening_type(&mut job, "window 4 by 3", Kind::Window, "4 ft", "3 ft", how).unwrap();
    (job, s, wt, dt, nt)
}

/// The priced job with nothing Assumed in it: the shed's pitch Known, overhead and profit Known.
pub fn priced_known() -> Job {
    let mut job = sample::priced();
    job.set("plane-0017", "pitch", Some("3"), Status::Known, "made-up dimension").unwrap();
    job.set("job-0001", "overhead", Some("10"), Status::Known, "made-up markup").unwrap();
    job.set("job-0001", "profit", Some("8"), Status::Known, "made-up markup").unwrap();
    job.derive();
    job
}

/// The storey of `drawn`, a 6 in round duct type, a supply air system, and a duct of two
/// pieces at 8 ft above the floor: (2, 2) to (10, 2) to (10, 14). Every value Known.
pub fn ducted() -> (Job, String, String, String, Vec<String>) {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    let air = ed::system(&mut job, "Supply air", "supply air").unwrap();
    let pieces = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[2.0, 2.0], [10.0, 2.0], [10.0, 14.0]], "8 ft", Some(&air), how).unwrap();
    (job, s, duct, air, pieces)
}

/// How a system hangs together: its networks, its open ends, and the places where a run
/// meets another along its length with no fitting.
pub fn joined_of(job: &mut Job, system: &str) -> (usize, usize, usize) {
    let t = plumbline_world::building::takeoff(job);
    let j = &t.iter().find(|x| x.name == system).unwrap().joined;
    (j.networks, j.open.len(), j.no_fitting.len())
}

/// A system's fittings as the takeoff counts them, in the order of what each is called.
pub fn fittings_of(job: &mut Job, system: &str) -> Vec<(String, String, usize, Status)> {
    let t = plumbline_world::building::takeoff(job);
    let mut out: Vec<(String, String, usize, Status)> = t.iter().find(|x| x.name == system).unwrap().fittings.iter().map(|(kind, what, count, status)| (kind.map(|k| k.title().to_string()).unwrap_or_default(), what.clone(), *count, *status)).collect();
    out.sort();
    out
}

/// The clashes in a job as a test reads them: each as its two names, whether it is hard,
/// whether it is a run through a wall or a floor, how much by (to four places), and how
/// sure it is.
pub fn clashes_of(job: &mut Job, clearance: Option<f64>) -> Vec<(String, String, bool, bool, Option<f64>, Status)> {
    use plumbline_world::building::clash::{clashes, ClashKind};
    clashes(job, clearance).unwrap().found.iter().map(|c| (c.a_name.clone(), c.b_name.clone(), c.how == ClashKind::Hard, c.through, c.by.map(|by| (by * 1.0e4).round() / 1.0e4), c.status)).collect()
}

/// Whether a figure is the one worked by hand, to the fourth place.
pub fn about(got: Option<f64>, want: f64) -> bool {
    got.is_some_and(|g| (g - want).abs() <= 2.0e-4)
}

/// One layout of "Two layouts side by side" (call 154): the house of the room-line test, the
/// addition `x` ft out, its walls priced at $12/SF (Division 6) and the Kitchen's floor at
/// $8/SF (Division 9).
pub fn addition_layout(title: &str, x: f64) -> Job {
    use plumbline_world::editing as ed;
    let known = || ed::Entered { status: Status::Known, source: "made-up survey" };
    let assumed = || ed::Entered { status: Status::Assumed, source: "made-up addition plan" };
    let mut job = Job::new(title, "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let outer = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    ed::mark(&mut job, &[outer[0].clone(), outer[2].clone(), outer[3].clone()], WorkStatus::Existing, ed::ItsLines::Keep, known()).unwrap();
    ed::mark(&mut job, &[outer[1].clone()], WorkStatus::Demolish, ed::ItsLines::Ask, known()).unwrap();
    let new = ed::walls(&mut job, &level, &w6, &[[24.0, 0.0], [x, 0.0], [x, 16.0], [24.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    ed::room(&mut job, &level, "Living", [6.0, 8.0], known()).unwrap();
    let kitchen = ed::room(&mut job, &level, "Kitchen", [x - 6.0, 8.0], known()).unwrap();
    ed::room_lines(&mut job, &level, &[[24.0, 0.0], [24.0, 16.0]], assumed()).unwrap();
    for (path, what, quantity, division, rate) in new.iter().map(|w| (w.clone(), "Wall, framed and finished", "face_area", Division::WoodAndPlastics, "$12.00/SF")).chain([(kitchen, "Flooring", "floor_area", Division::Finishes, "$8.00/SF")]) {
        job.add_line(&path, what, quantity, Some(Class::Material), false).unwrap();
        job.set_division(&path, Some(what), division).unwrap();
        job.set(&path, "unit_cost", Some(rate), Status::Known, "made-up price").unwrap();
    }
    job.derive();
    job
}
