//! IFC models from other trades and designers, brought into a job.
//!
//! An IFC2x3 or IFC4 file is read with BIMCraft's STEP reader (`step.rs`, ported). What
//! is brought in is what this program has an element for and can read the shape of:
//!
//! * each building storey, with its floor height, and its height up to the next one;
//! * each wall whose body is a rectangle pushed straight up (how nearly every wall in a
//!   house is modelled), on its centre line, with its thickness and its height;
//! * each door and window that fills a hole in such a wall, by its width, its height,
//!   where its middle is along the wall and how high its sill is;
//! * each floor slab whose body is an outline pushed straight up or down;
//! * each space, as a room named by a point inside it;
//! * each duct, pipe, cable tray and cable whose body is a circle or a rectangle pushed
//!   along its run, on the system the file puts it on where that is one this program has.
//!
//! **Everything else is listed, by what it is and how many, with why**: a thing this
//! program has no element for (a beam, a stair, furniture), a wall that leans or whose
//! body is some other shape, a door in no wall. Nothing in the file goes missing without
//! being said. Whoever brings the model in says how sure it is and where it came from;
//! each value's source also names the thing in the file it was read off.
//!
//! The file's lengths are turned into feet (kept to a ten-thousandth of a foot), from
//! whatever unit the file says it is in. That is said in the report.
//!
//! Ours, on BIMCraft's reader. BIMCraft's own import (`import.rs`) builds BIMCraft's
//! model, measures every shape (curved walls, breps, meshes) and is some five thousand
//! lines; this reads the plain shapes straight, and says what it did not read.

pub mod step;

use std::collections::{BTreeMap, HashMap};

use plumbline_world::building::{self, Floor, FloorType, InSystem, Room, Segment, Wall};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::units::Len;
use plumbline_world::value::Val;
use plumbline_world::{Job, Kind};
use step::{Entity, File, V};

/// What bringing a model in did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Brought {
    /// The schema the file says it is in: IFC4, IFC2X3.
    pub schema: String,
    /// What the file's lengths are in, as said in it, and how many feet that is.
    pub unit: String,
    pub feet_per_unit: f64,
    /// How many of each kind of element were brought in, by what this program calls it.
    pub made: BTreeMap<String, usize>,
    /// What was not brought in: by what the file calls it, how many, and why. One kind of
    /// thing can be left out for more than one reason.
    pub left_out: Vec<(String, usize, String)>,
    /// Things done that whoever brought the model in should know.
    pub notes: Vec<String>,
}

type P3 = [f64; 3];

/// A place and a turn: three axes and where their meeting point is.
#[derive(Clone, Copy, Debug)]
struct Frame {
    x: P3,
    y: P3,
    z: P3,
    o: P3,
}

const WORLD: Frame = Frame { x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0], o: [0.0, 0.0, 0.0] };

fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: P3, b: P3) -> P3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: P3) -> Option<P3> {
    let l = dot(a, a).sqrt();
    (l > 1.0e-12).then(|| [a[0] / l, a[1] / l, a[2] / l])
}

impl Frame {
    /// A direction given in this frame, as it lies outside it.
    fn turn(&self, d: P3) -> P3 {
        [self.x[0] * d[0] + self.y[0] * d[1] + self.z[0] * d[2], self.x[1] * d[0] + self.y[1] * d[1] + self.z[1] * d[2], self.x[2] * d[0] + self.y[2] * d[1] + self.z[2] * d[2]]
    }
    /// A point given in this frame, where it is outside it.
    fn place(&self, p: P3) -> P3 {
        let t = self.turn(p);
        [t[0] + self.o[0], t[1] + self.o[1], t[2] + self.o[2]]
    }
    /// A frame given inside this one, as it lies outside it.
    fn then(&self, inner: &Frame) -> Frame {
        Frame { x: self.turn(inner.x), y: self.turn(inner.y), z: self.turn(inner.z), o: self.place(inner.o) }
    }
}

/// The outline a body is pushed from.
enum Profile {
    /// So long (its own x) by so wide (its own y), about a middle.
    Rect { x: f64, y: f64, at: [f64; 2], along: [f64; 2] },
    Circle { radius: f64, at: [f64; 2] },
    Outline(Vec<[f64; 2]>),
}

/// A body made by pushing an outline: where it is, which way it is pushed and how far.
struct Pushed {
    profile: Profile,
    frame: Frame,
    way: P3,
    depth: f64,
}

struct Reader<'a> {
    file: &'a File,
}

impl<'a> Reader<'a> {
    fn get(&self, v: &V) -> Option<(u64, &'a Entity)> {
        match v {
            V::Ref(id) => self.file.entities.get(id).map(|e| (*id, e)),
            _ => None,
        }
    }
    fn arg(&self, e: &'a Entity, i: usize) -> &'a V {
        e.args.get(i).unwrap_or(&V::Null)
    }
    fn number(v: &V) -> Option<f64> {
        match v {
            V::Real(x) => Some(*x),
            V::Int(i) => Some(*i as f64),
            V::Typed(_, inner) => Reader::number(inner),
            _ => None,
        }
    }
    fn text(v: &V) -> Option<&str> {
        match v {
            V::Str(s) => Some(s.as_str()),
            V::Typed(_, inner) => Reader::text(inner),
            _ => None,
        }
    }
    fn list(v: &V) -> &[V] {
        match v {
            V::List(l) => l,
            _ => &[],
        }
    }
    fn triple(&self, v: &V) -> Option<P3> {
        let (_, e) = self.get(v)?;
        let c: Vec<f64> = Reader::list(self.arg(e, 0)).iter().filter_map(Reader::number).collect();
        match c.as_slice() {
            [x, y] => Some([*x, *y, 0.0]),
            [x, y, z, ..] => Some([*x, *y, *z]),
            _ => None,
        }
    }
    /// `IfcAxis2Placement3D`: a point, the way its z goes and the way its x goes.
    fn axes(&self, v: &V) -> Frame {
        let Some((_, e)) = self.get(v) else { return WORLD };
        let o = self.triple(self.arg(e, 0)).unwrap_or([0.0; 3]);
        if e.name == "IFCAXIS2PLACEMENT2D" {
            let x = self.triple(self.arg(e, 1)).and_then(unit).unwrap_or([1.0, 0.0, 0.0]);
            return Frame { x, y: [-x[1], x[0], 0.0], z: [0.0, 0.0, 1.0], o };
        }
        let z = self.triple(self.arg(e, 1)).and_then(unit).unwrap_or([0.0, 0.0, 1.0]);
        let toward = self.triple(self.arg(e, 2)).unwrap_or(if z[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] });
        // The x given need not be square to z: what is square to it is used.
        let along = dot(toward, z);
        let x = unit([toward[0] - z[0] * along, toward[1] - z[1] * along, toward[2] - z[2] * along]).unwrap_or([1.0, 0.0, 0.0]);
        Frame { x, y: cross(z, x), z, o }
    }
    /// `IfcLocalPlacement`, through every placement it is placed in.
    fn placed(&self, v: &V) -> Frame {
        let mut chain: Vec<Frame> = Vec::new();
        let mut at = self.get(v);
        // No model nests placements this deep; a file that loops is not followed for ever.
        while let Some((_, e)) = at.filter(|_| chain.len() < 64) {
            if e.name != "IFCLOCALPLACEMENT" {
                break;
            }
            chain.push(self.axes(self.arg(e, 1)));
            at = self.get(self.arg(e, 0));
        }
        chain.iter().rev().fold(WORLD, |outer, inner| outer.then(inner))
    }
    /// The first pushed body among a product's shapes, in the product's own frame.
    fn pushed(&self, product: &'a Entity) -> Option<Pushed> {
        let (_, shape) = self.get(self.arg(product, 6))?;
        for rep in Reader::list(self.arg(shape, 2)) {
            let Some((_, rep)) = self.get(rep) else { continue };
            for item in Reader::list(self.arg(rep, 3)) {
                let Some((_, solid)) = self.get(item) else { continue };
                if solid.name != "IFCEXTRUDEDAREASOLID" {
                    continue;
                }
                let (_, area) = self.get(self.arg(solid, 0))?;
                let profile = match area.name.as_str() {
                    "IFCRECTANGLEPROFILEDEF" => {
                        let f = self.axes(self.arg(area, 2));
                        Profile::Rect { x: Reader::number(self.arg(area, 3))?, y: Reader::number(self.arg(area, 4))?, at: [f.o[0], f.o[1]], along: [f.x[0], f.x[1]] }
                    }
                    "IFCCIRCLEPROFILEDEF" | "IFCCIRCLEHOLLOWPROFILEDEF" => {
                        let f = self.axes(self.arg(area, 2));
                        Profile::Circle { radius: Reader::number(self.arg(area, 3))?, at: [f.o[0], f.o[1]] }
                    }
                    "IFCARBITRARYCLOSEDPROFILEDEF" => {
                        let (_, curve) = self.get(self.arg(area, 2))?;
                        if curve.name != "IFCPOLYLINE" {
                            return None;
                        }
                        let mut points: Vec<[f64; 2]> = Reader::list(self.arg(curve, 0)).iter().filter_map(|p| self.triple(p)).map(|p| [p[0], p[1]]).collect();
                        if points.len() > 1 && points.first() == points.last() {
                            points.pop();
                        }
                        if points.len() < 3 {
                            return None;
                        }
                        Profile::Outline(points)
                    }
                    _ => return None,
                };
                let frame = self.axes(self.arg(solid, 1));
                let way = self.triple(self.arg(solid, 2)).and_then(unit)?;
                let depth = Reader::number(self.arg(solid, 3))?;
                return Some(Pushed { profile, frame, way, depth });
            }
        }
        None
    }
    fn name(&self, e: &'a Entity) -> String {
        Reader::text(self.arg(e, 2)).unwrap_or("").trim().to_string()
    }
}

/// What the file's lengths are in: the name as the file has it, and feet to one of it.
fn length_unit(r: &Reader) -> (String, f64) {
    const FT_PER_M: f64 = 1.0 / 0.3048;
    let si = |e: &Entity| -> Option<(String, f64)> {
        if !matches!(r.arg(e, 1), V::Enum(k) if k == "LENGTHUNIT") || !matches!(r.arg(e, 3), V::Enum(n) if n == "METRE") {
            return None;
        }
        Some(match r.arg(e, 2) {
            V::Enum(p) if p == "MILLI" => ("millimetres".to_string(), 0.001 * FT_PER_M),
            V::Enum(p) if p == "CENTI" => ("centimetres".to_string(), 0.01 * FT_PER_M),
            V::Enum(p) if p == "DECI" => ("decimetres".to_string(), 0.1 * FT_PER_M),
            _ => ("metres".to_string(), FT_PER_M),
        })
    };
    for e in r.file.entities.values().filter(|e| e.name == "IFCUNITASSIGNMENT") {
        for u in Reader::list(r.arg(e, 0)) {
            let Some((_, u)) = r.get(u) else { continue };
            match u.name.as_str() {
                "IFCSIUNIT" => {
                    if let Some(found) = si(u) {
                        return found;
                    }
                }
                // A foot or an inch, said as so many of an SI unit.
                "IFCCONVERSIONBASEDUNIT" if matches!(r.arg(u, 1), V::Enum(k) if k == "LENGTHUNIT") => {
                    let name = Reader::text(r.arg(u, 2)).unwrap_or("").to_lowercase();
                    if let Some((_, measure)) = r.get(r.arg(u, 3)) {
                        let (factor, base) = (Reader::number(r.arg(measure, 0)), r.get(r.arg(measure, 1)).and_then(|(_, b)| si(b)));
                        if let (Some(factor), Some((_, base))) = (factor, base) {
                            return (name, factor * base);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    ("metres (the file does not say; an IFC file with no length unit is in metres)".to_string(), FT_PER_M)
}

fn tidy(x: f64) -> f64 {
    (x * 1.0e4).round() / 1.0e4 + 0.0
}

fn feet(x: f64) -> String {
    format!("{} ft", tidy(x))
}

/// Bring a model in. `name` is what the file is called, for the sources of what is made.
/// All of it or none: if the job would not check afterwards, it is as it was.
pub fn import(job: &mut Job, bytes: &[u8], name: &str, how: Entered) -> Result<Brought, String> {
    let file = step::parse(bytes).map_err(|why| format!("it is not an IFC file this can read ({why})"))?;
    if !file.schema.starts_with("IFC") {
        return Err(format!("it is a STEP file of the schema {:?}, not an IFC model", file.schema));
    }
    if how.source.trim().is_empty() {
        return Err("say where the model came from".into());
    }
    let r = Reader { file: &file };
    let (unit_name, k) = length_unit(&r);
    ed::all_or_nothing(job, |job| bring(job, &r, name, how, unit_name, k))
}

fn bring(job: &mut Job, r: &Reader, name: &str, how: Entered, unit_name: String, k: f64) -> Result<Brought, String> {
    let mut out = Brought { schema: r.file.schema.clone(), unit: unit_name, feet_per_unit: k, ..Brought::default() };
    let mut left: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut leave = |class: &str, why: &str| *left.entry((class.to_string(), why.to_string())).or_insert(0) += 1;
    let mut made = |what: &str| *out.made.entry(what.to_string()).or_insert(0) += 1;
    // Every value says the thing in the file it was read off.
    let from = |e: &Entity, id: u64| -> String {
        let called = r.name(e);
        format!("{}: {name}, {}{} (#{id})", how.source, if called.is_empty() { String::new() } else { format!("\"{called}\", ") }, e.name)
    };
    let val = |source: &str, len: f64| Val { value: Some(Len::One(tidy(len))), unit: "ft".to_string(), status: how.status, source: source.to_string() };
    let path_val = |source: &str, path: &str| Val { value: Some(path.to_string()), unit: String::new(), status: how.status, source: source.to_string() };
    let mut ids: Vec<u64> = r.file.entities.keys().copied().collect();
    ids.sort_unstable();
    let of = |class: &[&str]| -> Vec<(u64, &Entity)> { ids.iter().filter_map(|id| r.file.entities.get(id).map(|e| (*id, e))).filter(|(_, e)| class.contains(&e.name.as_str())).collect() };

    // ---- Storeys, lowest first. Each is as tall as it is to the next one up.
    let mut storeys: Vec<(u64, String, f64)> = of(&["IFCBUILDINGSTOREY"]).into_iter().map(|(id, e)| (id, r.name(e), Reader::number(r.arg(e, 9)).unwrap_or_else(|| r.placed(r.arg(e, 5)).o[2]) * k)).collect();
    storeys.sort_by(|a, b| a.2.total_cmp(&b.2));
    if storeys.is_empty() {
        return Err("the model has no building storey: there is nothing to stand its walls and floors on".into());
    }
    let mut storey_path: HashMap<u64, (String, f64)> = HashMap::new();
    for (i, (id, called, elevation)) in storeys.iter().enumerate() {
        let source = from(&r.file.entities[id], *id);
        let entered = Entered { status: how.status, source: &source };
        let called = if called.is_empty() { format!("storey at {}", feet(*elevation)) } else { called.clone() };
        let path = match storeys.get(i + 1) {
            Some(next) if next.2 - elevation > 0.01 => ed::storey(job, &called, &feet(*elevation), &feet(next.2 - elevation), entered)?,
            _ => {
                // The top storey: how tall it is is in nothing the file says.
                let path = ed::storey(job, &called, &feet(*elevation), "1 ft", entered)?;
                let e = job.find(&path).ok_or("the new storey is not there")?;
                let mut level = job.world.get::<building::Level>(e).cloned().ok_or("the new storey has no level")?;
                level.height = Val::unknown("ft", "the model has no storey above this one to say how tall it is");
                job.put(&path, level)?;
                path
            }
        };
        storey_path.insert(*id, (path, *elevation));
        made("storeys");
    }
    // ---- What stands on which storey.
    let mut on: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELCONTAINEDINSPATIALSTRUCTURE"]) {
        if let V::Ref(storey) = r.arg(rel, 5) {
            for thing in Reader::list(r.arg(rel, 4)) {
                if let V::Ref(id) = thing {
                    on.insert(*id, *storey);
                }
            }
        }
    }
    for (_, rel) in of(&["IFCRELAGGREGATES"]) {
        if let V::Ref(whole) = r.arg(rel, 4) {
            for part in Reader::list(r.arg(rel, 5)) {
                if let V::Ref(id) = part {
                    on.entry(*id).or_insert(*whole);
                }
            }
        }
    }
    // The storey a thing is on: the one the file puts it on, or the highest whose floor
    // is not above where the thing is.
    let mut guessed = 0usize;
    let mut storey_of = |id: u64, z: f64| -> (String, f64) {
        if let Some(found) = on.get(&id).and_then(|s| storey_path.get(s)) {
            return found.clone();
        }
        guessed += 1;
        let below = storeys.iter().rev().find(|s| s.2 <= z + 0.01).unwrap_or(&storeys[0]);
        storey_path[&below.0].clone()
    };

    // ---- Walls.
    let mut wall_of: HashMap<u64, (String, P3, P3, f64)> = HashMap::new();
    for (id, e) in of(&["IFCWALL", "IFCWALLSTANDARDCASE", "IFCWALLELEMENTEDCASE"]) {
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its body is not a plain shape pushed straight (it may be curved, cut to a roof, or a mesh)");
            continue;
        };
        let Profile::Rect { x, y, at, along } = body.profile else {
            leave(&e.name, "its body is not a rectangle on plan");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let up = frame.turn(body.way);
        if up[2].abs() < 0.999 {
            leave(&e.name, "it does not stand upright");
            continue;
        }
        // Its length is the longer side of the rectangle, its thickness the shorter.
        let (length, thickness, way) = if x >= y { (x, y, [along[0], along[1], 0.0]) } else { (y, x, [-along[1], along[0], 0.0]) };
        let middle = frame.place([at[0], at[1], 0.0]);
        let way = frame.turn(way);
        let (a, b) = ([middle[0] - way[0] * length / 2.0, middle[1] - way[1] * length / 2.0], [middle[0] + way[0] * length / 2.0, middle[1] + way[1] * length / 2.0]);
        let base = if up[2] > 0.0 { middle[2] } else { middle[2] - body.depth };
        let (storey, floor) = storey_of(id, base * k);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let made_wall = (|| -> Result<String, String> {
            let wall_type = ed::wall_type_for(job, &format!("{} in", tidy(thickness * k * 12.0)), entered)?;
            let made = ed::walls(job, &storey, &wall_type, &[[tidy(a[0] * k), tidy(a[1] * k)], [tidy(b[0] * k), tidy(b[1] * k)]], false, ed::Top::Height(&feet(body.depth * k)), entered)?;
            let path = made.into_iter().next().ok_or("no wall was made")?;
            if (base * k - floor).abs() > 0.001 {
                let w = job.find(&path).ok_or("the new wall is not there")?;
                let mut wall = job.world.get::<Wall>(w).cloned().ok_or("the new wall has no line")?;
                wall.base_offset = Some(val(&source, base * k - floor));
                job.put(&path, wall)?;
            }
            Ok(path)
        })();
        match made_wall {
            Ok(path) => {
                wall_of.insert(id, (path, [a[0], a[1], base], [way[0], way[1], 0.0], length));
                made("walls");
            }
            Err(why) => leave(&e.name, &format!("this program would not take it: {why}")),
        }
    }
    // ---- Doors and windows: each fills a hole, and each hole is in a wall.
    let mut hole_in: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELVOIDSELEMENT"]) {
        if let (V::Ref(wall), V::Ref(hole)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            hole_in.insert(*hole, *wall);
        }
    }
    let mut fills: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELFILLSELEMENT"]) {
        if let (V::Ref(hole), V::Ref(filling)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            fills.insert(*filling, *hole);
        }
    }
    for (id, e) in of(&["IFCDOOR", "IFCDOORSTANDARDCASE", "IFCWINDOW", "IFCWINDOWSTANDARDCASE"]) {
        let kind = if e.name.starts_with("IFCDOOR") { Kind::Door } else { Kind::Window };
        let Some((wall, start, way, _)) = fills.get(&id).and_then(|hole| hole_in.get(hole)).and_then(|w| wall_of.get(w)) else {
            leave(&e.name, "it fills no hole in a wall that was brought in");
            continue;
        };
        let (Some(height), Some(width)) = (Reader::number(r.arg(e, 8)), Reader::number(r.arg(e, 9))) else {
            leave(&e.name, "the model does not give its width and its height");
            continue;
        };
        // It is placed by a lower corner, its own x along its width.
        let frame = r.placed(r.arg(e, 5));
        let along = dot([frame.o[0] - start[0], frame.o[1] - start[1], 0.0], *way) + dot(frame.x, *way) * width / 2.0;
        let sill = frame.o[2] - start[2];
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let size = |x: f64| format!("{} in", tidy(x * k * 12.0));
        let done = ed::opening_type_for(job, kind, &size(width), &size(height), entered).and_then(|t| ed::opening(job, wall, kind, &t, ed::OnWall::Offset(&feet(along * k)), &feet(sill * k), entered));
        match done {
            Ok(_) => made(if kind == Kind::Door { "doors" } else { "windows" }),
            Err(why) => leave(&e.name, &format!("this program would not take it: {why}")),
        }
    }
    for (id, e) in of(&["IFCOPENINGELEMENT", "IFCOPENINGSTANDARDCASE"]) {
        if !fills.values().any(|hole| *hole == id) {
            leave(&e.name, "a hole with no door or window in it: this program cuts a wall only for one of those");
        }
    }
    // ---- Floor slabs.
    let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
    let mut floor_types: HashMap<i64, String> = HashMap::new();
    for (id, e) in of(&["IFCSLAB", "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE"]) {
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its body is not an outline pushed straight (it may slope, or be a mesh)");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let way = frame.turn(body.way);
        if way[2].abs() < 0.999 || frame.z[2].abs() < 0.999 {
            leave(&e.name, "it is not level");
            continue;
        }
        let corners: Vec<[f64; 2]> = match &body.profile {
            Profile::Outline(points) => points.clone(),
            Profile::Rect { x, y, at, along } => {
                let across = [-along[1], along[0]];
                [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)].iter().map(|(u, v)| [at[0] + along[0] * x * u + across[0] * y * v, at[1] + along[1] * x * u + across[1] * y * v]).collect()
            }
            Profile::Circle { .. } => {
                leave(&e.name, "it is round");
                continue;
            }
        };
        let boundary: Vec<[f64; 2]> = corners.iter().map(|p| frame.place([p[0], p[1], 0.0])).map(|p| [tidy(p[0] * k), tidy(p[1] * k)]).collect();
        let top = if way[2] > 0.0 { frame.o[2] + body.depth } else { frame.o[2] };
        let (storey, floor) = storey_of(id, top * k);
        let source = from(e, id);
        let thickness = tidy(body.depth * k * 12.0);
        let done = (|| -> Result<(), String> {
            let key = (thickness * 1.0e4).round() as i64;
            let floor_type = match floor_types.get(&key) {
                Some(path) => path.clone(),
                None => {
                    let path = job.add(&root, Kind::Type, &format!("floor {thickness} in"))?;
                    let t = FloorType { thickness: Val { value: Some(Len::One(thickness)), unit: "in".into(), status: how.status, source: source.clone() }, layers: Vec::new() };
                    building::check_floor_type(&path, Kind::Type, &t)?;
                    job.put(&path, t)?;
                    floor_types.insert(key, path.clone());
                    path
                }
            };
            let called = r.name(e);
            let path = job.add(&storey, Kind::Floor, &if called.is_empty() { format!("floor {id}") } else { format!("{called} {id}") })?;
            let offset = top * k - floor;
            let f = Floor { type_name: path_val(&source, &floor_type), boundary: Val { value: Some(boundary.clone()), unit: "ft".into(), status: how.status, source: source.clone() }, offset: (offset.abs() > 0.001).then(|| val(&source, offset)) };
            building::check_floor(&path, Kind::Floor, &f)?;
            job.put(&path, f)
        })();
        match done {
            Ok(()) => made("floors"),
            Err(why) => leave(&e.name, &format!("this program would not take it: {why}")),
        }
    }
    // ---- Spaces, as rooms: a point inside each.
    for (id, e) in of(&["IFCSPACE"]) {
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its shape is not an outline pushed straight, so no point inside it could be found");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let middle = match &body.profile {
            Profile::Outline(points) => {
                let n = points.len() as f64;
                [points.iter().map(|p| p[0]).sum::<f64>() / n, points.iter().map(|p| p[1]).sum::<f64>() / n]
            }
            Profile::Rect { at, .. } | Profile::Circle { at, .. } => *at,
        };
        let at = frame.place([middle[0], middle[1], 0.0]);
        let (storey, _) = storey_of(id, at[2] * k);
        let source = from(e, id);
        let (number, long) = (r.name(e), Reader::text(r.arg(e, 7)).unwrap_or("").trim().to_string());
        let called = match (long.is_empty(), number.is_empty()) {
            (false, false) => format!("{long} {number}"),
            (false, true) => long,
            (true, false) => format!("room {number}"),
            (true, true) => format!("room {id}"),
        };
        let done = (|| -> Result<(), String> {
            let path = job.add(&storey, Kind::Room, &called)?;
            let room = Room { point: Val { value: Some([tidy(at[0] * k), tidy(at[1] * k)]), unit: "ft".into(), status: how.status, source: source.clone() }, number: None };
            building::check_room(&path, Kind::Room, &room)?;
            job.put(&path, room)
        })();
        match done {
            Ok(()) => made("rooms"),
            Err(why) => leave(&e.name, &format!("this program would not take it: {why}")),
        }
    }
    // ---- Systems the file names, where they are ones this program has.
    let mut system_of: HashMap<u64, String> = HashMap::new();
    let mut systems_not_known = 0usize;
    for (_, rel) in of(&["IFCRELASSIGNSTOGROUP"]) {
        let Some((_, group)) = r.get(r.arg(rel, 6)) else { continue };
        if !matches!(group.name.as_str(), "IFCDISTRIBUTIONSYSTEM" | "IFCSYSTEM" | "IFCDISTRIBUTIONCIRCUIT") {
            continue;
        }
        let class = match r.arg(group, 6) {
            V::Enum(t) => match t.as_str() {
                "AIRCONDITIONING" | "VENTILATION" | "HEATING" => Some("Supply Air"),
                "EXHAUST" => Some("Exhaust Air"),
                "DOMESTICCOLDWATER" => Some("Domestic Cold Water"),
                "DOMESTICHOTWATER" => Some("Domestic Hot Water"),
                "SEWAGE" | "WASTEWATER" | "DRAINAGE" => Some("Sanitary"),
                "VENT" => Some("Vent"),
                "REFRIGERATION" => Some("Refrigerant"),
                "ELECTRICAL" => Some("Power"),
                "LIGHTING" => Some("Lighting"),
                _ => None,
            },
            _ => None,
        };
        let Some(class) = class else {
            systems_not_known += 1;
            continue;
        };
        let path = match ed::system_of_class(job, class) {
            Ok(path) => path,
            Err(_) => ed::system(job, &r.name(group), class)?,
        };
        for thing in Reader::list(r.arg(rel, 4)) {
            if let V::Ref(id) = thing {
                system_of.insert(*id, path.clone());
            }
        }
    }
    // ---- Ducts, pipes, cable trays and cables.
    let kind_of_type: HashMap<u64, Kind> = {
        let mut found = HashMap::new();
        for (_, rel) in of(&["IFCRELDEFINESBYTYPE"]) {
            let kind = r.get(r.arg(rel, 5)).and_then(|(_, t)| match t.name.as_str() {
                "IFCDUCTSEGMENTTYPE" => Some(Kind::Duct),
                "IFCPIPESEGMENTTYPE" => Some(Kind::Pipe),
                "IFCCABLECARRIERSEGMENTTYPE" => Some(Kind::Conduit),
                "IFCCABLESEGMENTTYPE" => Some(Kind::Cable),
                _ => None,
            });
            if let Some(kind) = kind {
                for thing in Reader::list(r.arg(rel, 4)) {
                    if let V::Ref(id) = thing {
                        found.insert(*id, kind);
                    }
                }
            }
        }
        found
    };
    for (id, e) in of(&["IFCDUCTSEGMENT", "IFCPIPESEGMENT", "IFCCABLECARRIERSEGMENT", "IFCCABLESEGMENT", "IFCFLOWSEGMENT"]) {
        let kind = match e.name.as_str() {
            "IFCDUCTSEGMENT" => Some(Kind::Duct),
            "IFCPIPESEGMENT" => Some(Kind::Pipe),
            "IFCCABLECARRIERSEGMENT" => Some(Kind::Conduit),
            "IFCCABLESEGMENT" => Some(Kind::Cable),
            _ => kind_of_type.get(&id).copied(),
        };
        let Some(kind) = kind else {
            leave(&e.name, "the model does not say whether it is a duct, a pipe, a tray or a cable");
            continue;
        };
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its body is not a circle or a rectangle pushed along its run");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let way = frame.turn(body.way);
        let (start, across): (P3, (f64, Option<f64>)) = match &body.profile {
            Profile::Circle { radius, at } => (frame.place([at[0], at[1], 0.0]), (radius * 2.0, None)),
            Profile::Rect { x, y, at, along } => {
                // Its width is whichever side lies level; its depth the other.
                let level = frame.turn([along[0], along[1], 0.0])[2].abs() < 0.5;
                (frame.place([at[0], at[1], 0.0]), if level { (*x, Some(*y)) } else { (*y, Some(*x)) })
            }
            Profile::Outline(_) => {
                leave(&e.name, "its body is not a circle or a rectangle pushed along its run");
                continue;
            }
        };
        let end = [start[0] + way[0] * body.depth, start[1] + way[1] * body.depth, start[2] + way[2] * body.depth];
        let (storey, floor) = storey_of(id, start[2].min(end[2]) * k);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let inches = |x: f64| format!("{} in", tidy(x * k * 12.0));
        let done = (|| -> Result<(), String> {
            let run_type = match across.1 {
                None => ed::segment_type_for(job, kind, &inches(across.0), entered)?,
                Some(depth) => {
                    let called = format!("{} by {}", inches(across.0), inches(depth));
                    let known = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &building::SegmentType)>();
                        q.iter(&job.world).find(|(n, t)| n.name == called && t.category == kind).map(|(n, _)| n.path.clone())
                    };
                    match known {
                        Some(path) => path,
                        None => ed::segment_type(job, &called, kind, &inches(across.0), Some(&inches(depth)), entered)?,
                    }
                }
            };
            let called = r.name(e);
            let path = job.add(&storey, kind, &if called.is_empty() { format!("{} {id}", kind.stem()) } else { format!("{called} {id}") })?;
            let (z0, z1) = (start[2] * k - floor, end[2] * k - floor);
            let segment = Segment {
                type_name: path_val(&source, &run_type),
                line: Val { value: Some([[tidy(start[0] * k), tidy(start[1] * k)], [tidy(end[0] * k), tidy(end[1] * k)]]), unit: "ft".into(), status: how.status, source: source.clone() },
                offset: val(&source, z0),
                end_offset: ((z1 - z0).abs() > 0.001).then(|| val(&source, z1)),
                justify: Default::default(),
            };
            building::check_segment(&path, kind, &segment)?;
            job.put(&path, segment)?;
            if let Some(system) = system_of.get(&id) {
                let on = InSystem(path_val(&source, system));
                building::check_in_system(&path, kind, &on)?;
                job.put(&path, on)?;
            }
            Ok(())
        })();
        match done {
            Ok(()) => made(match kind {
                Kind::Duct => "ducts",
                Kind::Pipe => "pipes",
                Kind::Conduit => "conduits",
                _ => "cables",
            }),
            Err(why) => leave(&e.name, &format!("this program would not take it: {why}")),
        }
    }
    // ---- Everything else that is a thing with a place and a shape: listed.
    const READ: [&str; 22] = [
        "IFCBUILDINGSTOREY", "IFCWALL", "IFCWALLSTANDARDCASE", "IFCWALLELEMENTEDCASE", "IFCDOOR", "IFCDOORSTANDARDCASE", "IFCWINDOW", "IFCWINDOWSTANDARDCASE", "IFCOPENINGELEMENT", "IFCOPENINGSTANDARDCASE", "IFCSLAB",
        "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE", "IFCSPACE", "IFCDUCTSEGMENT", "IFCPIPESEGMENT", "IFCCABLECARRIERSEGMENT", "IFCCABLESEGMENT", "IFCFLOWSEGMENT", "IFCSITE", "IFCBUILDING", "IFCPROJECT",
    ];
    for id in &ids {
        let e = &r.file.entities[id];
        let is_placed = r.get(r.arg(e, 5)).is_some_and(|(_, p)| p.name == "IFCLOCALPLACEMENT" || p.name == "IFCGRIDPLACEMENT");
        if is_placed && !READ.contains(&e.name.as_str()) {
            leave(&e.name, "this program has no element for it");
        }
    }
    if guessed > 0 {
        out.notes.push(format!("{guessed} thing{} the model puts on no storey: each was put on the highest storey whose floor is not above it", if guessed == 1 { " is one" } else { "s are ones" }));
    }
    if systems_not_known > 0 {
        out.notes.push(format!("{systems_not_known} system{} in the model {} of a sort this program has no name for: what is on {} was brought in on no system", if systems_not_known == 1 { "" } else { "s" }, if systems_not_known == 1 { "is" } else { "are" }, if systems_not_known == 1 { "it" } else { "them" }));
    }
    out.notes.push(format!("the model is in {}: every length was turned into feet and kept to a ten-thousandth of a foot", out.unit));
    out.left_out = left.into_iter().map(|((class, why), n)| (class, n, why)).collect();
    if out.made.iter().all(|(what, _)| what == "storeys") {
        return Err(format!("nothing in the model but its storeys could be brought in: {}", out.left_out.iter().map(|(c, n, why)| format!("{n} {c} ({why})")).collect::<Vec<_>>().join("; ")));
    }
    Ok(out)
}
