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
pub mod write;

use std::collections::{BTreeMap, HashMap, HashSet};

use plumbline_world::building::{self, Floor, FloorType, InSystem, Room, Segment, Wall};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::units::{self, Len};
use plumbline_world::{solve, FittingKind, Footprint, Placed, Rise};
use plumbline_world::value::{Status, Val};
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
    /// The rooms of the job whose figures are not what they were before the model came in, worked by the model (call 206
    /// point 3): a wall brought in stands in a room and cuts it, and the rooms were not asked.
    pub rooms_moved: Vec<ed::RoomMoved>,
}

impl Brought {
    /// What was not brought in, a line for each: "4 floor slabs: why". The command line and the window both say it as it stands,
    /// so the one wording is the model's (call 209 point 3: the window built "4 x floor slab" for itself).
    pub fn left_out_said(&self) -> Vec<String> {
        self.left_out
            .iter()
            .map(|(class, n, why)| {
                // "4 floor slabs", and each reason as it reads for that many.
                let plural = *n > 1 && !class.ends_with('s') && !class.contains('"') && !class.contains(" or ");
                format!("{n} {class}{}: {why}", if plural { "s" } else { "" })
            })
            .collect()
    }
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

/// Why a roof of a model is left out: this program reads the roofs it wrote, by the outline it put in the file (call 206 point 8).
const NO_ROOF_OUTLINE: &str = "a roof with no outline of this program's in the file is not brought in: draw it from its footprint and its pitch";

/// What a kind of thing is called with its article: "a duct", "an outlet", "a piece of equipment" (call 206 point 9).
fn a_kind(kind: Kind) -> String {
    let stem = kind.stem();
    if stem == "equipment" {
        return "a piece of equipment".to_string();
    }
    format!("{} {stem}", if stem.chars().next().is_some_and(|c| "aeiou".contains(c)) { "an" } else { "a" })
}

/// A model that is in the job already, said once and by counts (call 206 point 9): no word of the file's own and no clause for
/// each fitting. It is that where most of what it has was left out as already there; what else was left out is said after, by
/// kind, as it always was.
fn already_in_the_job(left_out: &[(String, usize, String)]) -> Option<String> {
    let already = |why: &str| why.contains("already") || why.contains("counted twice") || why.contains("one over the other");
    let with_walls = |why: &str| why.contains("fills no hole in a wall that was brought in");
    let total: usize = left_out.iter().map(|(_, n, _)| n).sum();
    let twice: usize = left_out.iter().filter(|(_, _, why)| already(why)).map(|(_, n, _)| n).sum();
    let openings: usize = left_out.iter().filter(|(_, _, why)| with_walls(why)).map(|(_, n, _)| n).sum();
    if twice == 0 || twice * 2 < total {
        return None;
    }
    // The file's own words for what this program calls a post and a room.
    let plain = |class: &str| match class {
        c if c.contains("as the model calls it") => "other things".to_string(),
        "column" => "post".to_string(),
        "space" => "room".to_string(),
        c => c.to_string(),
    };
    let rest: Vec<String> = left_out.iter().filter(|(_, _, why)| !already(why) && !with_walls(why)).map(|(c, n, why)| format!("{n} {} ({})", plain(c), why)).collect();
    let mut said = format!(
        "this model is already in the job: nothing was brought in. {twice} of its {total} thing{} {} in the job already, in the same place, so a second would be counted twice",
        if total == 1 { "" } else { "s" },
        if twice == 1 { "is" } else { "are" }
    );
    // By kind: the fittings together, and no word of the file's own.
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for (class, n, _) in left_out.iter().filter(|(_, _, why)| already(why)) {
        let name = if class.ends_with(" fitting") { "fitting".to_string() } else { plain(class) };
        *kinds.entry(name).or_insert(0) += n;
    }
    let mut kinds: Vec<(String, usize)> = kinds.into_iter().collect();
    kinds.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    if !kinds.is_empty() {
        said.push_str(&format!(" ({})", kinds.iter().map(|(k, n)| format!("{n} {k}")).collect::<Vec<_>>().join(", ")));
    }
    if openings > 0 {
        said.push_str(&format!("; {openings} door{} and window{} go with walls that were not brought in again", if openings == 1 { "" } else { "s" }, if openings == 1 { "" } else { "s" }));
    }
    if !rest.is_empty() {
        said.push_str(&format!("; and {}", rest.join("; ")));
    }
    Some(said)
}

/// Two floors are at one height when they are within an eighth of an inch (call 206 point 3).
const SAME_HEIGHT_FT: f64 = 0.125 / 12.0;

/// How nearly straight up a wall's body has to be pushed, and how nearly level a floor
/// has to lie, to be read as upright or level: within about an eighth of an inch in
/// eight feet (the cosine of that lean).
const UPRIGHT: f64 = 0.999_999;

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
    /// The line a wall is drawn on, where the file gives one (the "Axis" of its shape): its two
    /// ends on plan, as the file places them. A body may run past this line at a corner.
    fn axis(&self, product: &'a Entity) -> Option<[[f64; 2]; 2]> {
        let (_, shape) = self.get(self.arg(product, 6))?;
        let frame = self.placed(self.arg(product, 5));
        for rep in Reader::list(self.arg(shape, 2)) {
            let Some((_, rep)) = self.get(rep) else { continue };
            if Reader::text(self.arg(rep, 1)) != Some("Axis") {
                continue;
            }
            for item in Reader::list(self.arg(rep, 3)) {
                let Some((_, line)) = self.get(item) else { continue };
                if line.name != "IFCPOLYLINE" {
                    continue;
                }
                let points: Vec<P3> = Reader::list(self.arg(line, 0)).iter().filter_map(|p| self.triple(p)).collect();
                if let [p, q] = points.as_slice() {
                    let (p, q) = (frame.place([p[0], p[1], 0.0]), frame.place([q[0], q[1], 0.0]));
                    return Some([[p[0], p[1]], [q[0], q[1]]]);
                }
            }
        }
        None
    }
    /// A run drawn with no body, only the line it is drawn on (the "Axis" of its shape, two points): its two
    /// ends as the file places them, in the file's unit. None where it has a body, or no such line.
    fn line_only(&self, product: &'a Entity) -> Option<(P3, P3)> {
        let (_, shape) = self.get(self.arg(product, 6))?;
        let frame = self.placed(self.arg(product, 5));
        let reps: Vec<&Entity> = Reader::list(self.arg(shape, 2)).iter().filter_map(|rep| self.get(rep).map(|(_, e)| e)).collect();
        if reps.len() != 1 || Reader::text(self.arg(reps[0], 1)) != Some("Axis") {
            return None;
        }
        let items = Reader::list(self.arg(reps[0], 3));
        let [item] = items else { return None };
        let (_, line) = self.get(item)?;
        if line.name != "IFCPOLYLINE" {
            return None;
        }
        let points: Vec<P3> = Reader::list(self.arg(line, 0)).iter().filter_map(|p| self.triple(p)).collect();
        match points.as_slice() {
            [p, q] if (0..3).any(|i| (p[i] - q[i]).abs() > 1.0e-9) => Some((frame.place(*p), frame.place(*q))),
            _ => None,
        }
    }
    fn name(&self, e: &'a Entity) -> String {
        Reader::text(self.arg(e, 2)).unwrap_or("").trim().to_string()
    }
    /// How many pieces a product's body is made of: the items of the shapes it calls its
    /// body, or of every shape it has where it calls none that.
    fn pieces(&self, product: &'a Entity) -> usize {
        let Some((_, shape)) = self.get(self.arg(product, 6)) else { return 0 };
        let reps: Vec<&Entity> = Reader::list(self.arg(shape, 2)).iter().filter_map(|rep| self.get(rep).map(|(_, e)| e)).collect();
        let body: Vec<&&Entity> = reps.iter().filter(|rep| Reader::text(self.arg(rep, 1)).is_some_and(|id| id.eq_ignore_ascii_case("body"))).collect();
        let count = |rep: &Entity| Reader::list(self.arg(rep, 3)).len();
        if body.is_empty() {
            reps.iter().map(|rep| Reader::list(self.arg(rep, 3)).iter().filter(|i| self.get(i).is_some_and(|(_, e)| e.name == "IFCEXTRUDEDAREASOLID")).count()).sum()
        } else {
            body.iter().map(|rep| count(rep)).sum()
        }
    }
    /// Whether a product's body is an outline with a hole in it, pushed.
    fn holed(&self, product: &'a Entity) -> bool {
        let Some((_, shape)) = self.get(self.arg(product, 6)) else { return false };
        Reader::list(self.arg(shape, 2)).iter().filter_map(|rep| self.get(rep)).flat_map(|(_, rep)| Reader::list(self.arg(rep, 3)).iter()).filter_map(|i| self.get(i)).any(|(_, solid)| self.get(self.arg(solid, 0)).is_some_and(|(_, area)| area.name == "IFCARBITRARYPROFILEDEFWITHVOIDS"))
    }
}

/// A wall's, or a strip footing's, body read as an upright box on its line (call 197: one reading
/// for both): the line it is drawn on, how long and how thick, where its base is and how tall it
/// is pushed, all in the file's own unit. Why not, where it is not.
struct Upright {
    a: [f64; 2],
    b: [f64; 2],
    length: f64,
    way: P3,
    thickness: f64,
    base: f64,
    depth: f64,
}

impl<'a> Reader<'a> {
    fn upright(&self, e: &'a Entity) -> Result<Upright, &'static str> {
        let r = self;
        if r.pieces(e) > 1 {
            return Err("its body is made of more than one piece: only a body of one piece is read");
        }
        let Some(body) = r.pushed(e) else {
            return Err("its body is not a plain shape pushed straight, as far as this can read it: it may be curved, cut to a roof, a mesh, or name a part that is not in the file");
        };
        let Profile::Rect { x, y, at, along } = body.profile else {
            return Err("its body is not a rectangle on plan");
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let up = frame.turn(body.way);
        // Upright to within an eighth of an inch in eight feet.
        if up[2].abs() < UPRIGHT {
            return Err("it does not stand upright");
        }
        // Its length is the longer side of the rectangle, its thickness the shorter.
        // Where the file gives the line the wall is drawn on, the side the line runs along is its length: a
        // wall shorter than it is thick (a pilaster, a 4 in return) has its longer side across the line (call 180, C3).
        let x_long = match r.axis(e) {
            Some([p, q]) => {
                let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                let long = dx.hypot(dy);
                let wx = frame.turn([along[0], along[1], 0.0]);
                let wy = frame.turn([-along[1], along[0], 0.0]);
                if long > 1.0e-9 && ((dx * wx[0] + dy * wx[1]) / long).abs() > 0.9999 {
                    true
                } else if long > 1.0e-9 && ((dx * wy[0] + dy * wy[1]) / long).abs() > 0.9999 {
                    false
                } else {
                    x >= y
                }
            }
            None => x >= y,
        };
        let (length, thickness, way) = if x_long { (x, y, [along[0], along[1], 0.0]) } else { (y, x, [-along[1], along[0], 0.0]) };
        let middle = frame.place([at[0], at[1], 0.0]);
        let way = frame.turn(way);
        let (mut a, mut b) = ([middle[0] - way[0] * length / 2.0, middle[1] - way[1] * length / 2.0], [middle[0] + way[0] * length / 2.0, middle[1] + way[1] * length / 2.0]);
        let mut length = length;
        let mut way = way;
        // A wall this program wrote carries the line it was drawn on: at a corner the body runs
        // past that line, or stops short of it, so the line, not the body, is the wall (call 179,
        // C1). Taken where it lies along the body (the same line, and its middle inside the body);
        // any other axis is left alone.
        if let Some([p, q]) = r.axis(e) {
            let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
            let long = dx.hypot(dy);
            let (mx, my) = ((p[0] + q[0]) / 2.0 - middle[0], (p[1] + q[1]) / 2.0 - middle[1]);
            let (u, v) = (mx * way[0] + my * way[1], mx * way[1] - my * way[0]);
            if long > 1.0e-9 && ((dx * way[0] + dy * way[1]) / long).abs() > 0.9999 && u.abs() <= length / 2.0 && v.abs() <= thickness / 2.0 + 1.0e-6 {
                let sign = if dx * way[0] + dy * way[1] >= 0.0 { 1.0 } else { -1.0 };
                a = p;
                b = q;
                length = long;
                way = [sign * way[0], sign * way[1], 0.0];
            }
        }
        let base = if up[2] > 0.0 { middle[2] } else { middle[2] - body.depth };
        Ok(Upright { a, b, length, way, thickness, base, depth: body.depth })
    }
}

/// A beam's body read as a level straight box (call 204, point 5): the middle of its first end, where it runs to on the plan,
/// its section as wide as it is across and as deep as it is up, and how high its top is, all in the file's unit.
struct BeamBody {
    a: [f64; 2],
    b: [f64; 2],
    width: f64,
    depth: f64,
    /// The height of the beam's middle (a level beam is at one height all along), and of its lowest face.
    middle_z: f64,
    bottom: f64,
}

/// A body read as a box: where its middle is (in the file's unit), its bottom, its three
/// sizes (along, across, up) and how far it is turned on plan, in degrees.
struct Boxed {
    middle: [f64; 2],
    bottom: f64,
    size: P3,
    angle: f64,
    round: bool,
}

impl<'a> Reader<'a> {
    /// A thing's body as the box it fills (call 197, point 1): a rectangle pushed straight is the
    /// box it is, a round body the square box round it, an outline the box round its corners along
    /// the body's own x. A body that leans, or is not a pushed shape, is not read.
    fn boxed(&self, e: &'a Entity) -> Option<Boxed> {
        let body = self.pushed(e)?;
        let frame = self.placed(self.arg(e, 5)).then(&body.frame);
        let up = frame.turn(body.way);
        if up[2].abs() < UPRIGHT {
            return None;
        }
        let low = |mid: f64| if up[2] > 0.0 { mid } else { mid - body.depth };
        match &body.profile {
            Profile::Rect { x, y, at, along } => {
                let mid = frame.place([at[0], at[1], 0.0]);
                let way = frame.turn([along[0], along[1], 0.0]);
                Some(Boxed { middle: [mid[0], mid[1]], bottom: low(mid[2]), size: [*x, *y, body.depth], angle: way[1].atan2(way[0]).to_degrees(), round: false })
            }
            Profile::Circle { radius, at } => {
                let mid = frame.place([at[0], at[1], 0.0]);
                Some(Boxed { middle: [mid[0], mid[1]], bottom: low(mid[2]), size: [radius * 2.0, radius * 2.0, body.depth], angle: 0.0, round: true })
            }
            Profile::Outline(points) => {
                let (lo, hi) = points.iter().fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])]));
                let mid = frame.place([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, 0.0]);
                let way = frame.turn([1.0, 0.0, 0.0]);
                Some(Boxed { middle: [mid[0], mid[1]], bottom: low(mid[2]), size: [hi[0] - lo[0], hi[1] - lo[1], body.depth], angle: way[1].atan2(way[0]).to_degrees(), round: false })
            }
        }
    }
    /// A beam's body: a rectangle pushed straight along a level line, its sides level and plumb. Why not, where it is not (call 204,
    /// point 5): a beam here is one rectangular section, level, square to its length and not turned about it.
    fn beam(&self, e: &'a Entity) -> Result<BeamBody, &'static str> {
        if self.pieces(e) > 1 {
            return Err("its body is made of more than one piece: only a body of one piece is read");
        }
        let Some(body) = self.pushed(e) else {
            return Err("its body is not a plain shape pushed straight, as far as this can read it: it may be curved, cut to a roof, a mesh, or name a part that is not in the file");
        };
        let (x, y, at, along) = match &body.profile {
            Profile::Rect { x, y, at, along } => (*x, *y, *at, *along),
            Profile::Circle { .. } => return Err("its section is round, not a rectangle: only a rectangular section is read"),
            Profile::Outline(_) => return Err("its section is not a rectangle: only a rectangular section is read"),
        };
        let frame = self.placed(self.arg(e, 5)).then(&body.frame);
        let d = frame.turn(body.way);
        let (ax, ay) = (frame.turn([along[0], along[1], 0.0]), frame.turn([-along[1], along[0], 0.0]));
        let dot = |p: P3, q: P3| p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
        if dot(ax, d).abs() > 1.0e-4 || dot(ay, d).abs() > 1.0e-4 {
            return Err("its section is not square to its length");
        }
        if d[2].abs() > 0.0015 {
            return Err("it slopes: a beam here is level");
        }
        // One side of the section plumb and the other level: the plumb one is its depth.
        let (depth, width) = if ax[2].abs() > UPRIGHT {
            (x, y)
        } else if ay[2].abs() > UPRIGHT {
            (y, x)
        } else {
            return Err("its section is turned about its length: a beam here has one side level and one plumb");
        };
        let start = frame.place([at[0], at[1], 0.0]);
        let end = [start[0] + d[0] * body.depth, start[1] + d[1] * body.depth, start[2] + d[2] * body.depth];
        Ok(BeamBody { a: [start[0], start[1]], b: [end[0], end[1]], width, depth, middle_z: start[2], bottom: start[2] - depth / 2.0 })
    }
    /// Where a thing stood, as the least and the greatest of each coordinate its body reaches (the
    /// file's unit), or its place alone where its body is not one this reads.
    fn reach(&self, e: &'a Entity) -> Option<([f64; 3], [f64; 3])> {
        let placed = self.placed(self.arg(e, 5));
        let Some(body) = self.pushed(e) else {
            return self.get(self.arg(e, 5)).map(|_| (placed.o, placed.o));
        };
        let frame = placed.then(&body.frame);
        let points: Vec<[f64; 2]> = match &body.profile {
            Profile::Rect { x, y, at, along } => {
                let across = [-along[1], along[0]];
                [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)].iter().map(|(u, v)| [at[0] + along[0] * x * u + across[0] * y * v, at[1] + along[1] * x * u + across[1] * y * v]).collect()
            }
            Profile::Circle { radius, at } => [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().map(|(u, v)| [at[0] + radius * u, at[1] + radius * v]).collect(),
            Profile::Outline(points) => points.clone(),
        };
        let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
        for p in points {
            for along in [0.0, body.depth] {
                let at = frame.place([p[0] + body.way[0] * along, p[1] + body.way[1] * along, body.way[2] * along]);
                for i in 0..3 {
                    lo[i] = lo[i].min(at[i]);
                    hi[i] = hi[i].max(at[i]);
                }
            }
        }
        Some((lo, hi))
    }
    /// The area of the one face a roof slab is written as, in the file's unit squared.
    fn face_area(&self, slab: &'a Entity) -> Option<f64> {
        let (_, shape) = self.get(self.arg(slab, 6))?;
        for rep in Reader::list(self.arg(shape, 2)) {
            let Some((_, rep)) = self.get(rep) else { continue };
            for item in Reader::list(self.arg(rep, 3)) {
                let Some((_, set)) = self.get(item) else { continue };
                if set.name != "IFCPOLYGONALFACESET" {
                    continue;
                }
                let (_, points) = self.get(self.arg(set, 0))?;
                let coords: Vec<P3> = Reader::list(self.arg(points, 0))
                    .iter()
                    .map(|c| {
                        let n: Vec<f64> = Reader::list(c).iter().filter_map(Reader::number).collect();
                        (n.len() == 3).then(|| [n[0], n[1], n[2]])
                    })
                    .collect::<Option<Vec<_>>>()?;
                let mut area = 0.0;
                for face in Reader::list(self.arg(set, 2)) {
                    let (_, face) = self.get(face)?;
                    let ring: Vec<P3> = Reader::list(self.arg(face, 0)).iter().filter_map(Reader::number).map(|i| coords.get((i as usize).wrapping_sub(1)).copied()).collect::<Option<Vec<_>>>()?;
                    let mut sum = [0.0; 3];
                    for i in 0..ring.len() {
                        let c = cross(ring[i], ring[(i + 1) % ring.len()]);
                        sum = [sum[0] + c[0], sum[1] + c[1], sum[2] + c[2]];
                    }
                    area += 0.5 * dot(sum, sum).sqrt();
                }
                return Some(area);
            }
        }
        None
    }
}

/// A thing brought in is called what the file calls it where nothing else in its place goes by
/// that, else that with its number in the file (call 197, point 5).
fn keep_name(job: &mut Job, path: &str, called: &str, id: u64) {
    let called = called.trim();
    let Some(e) = job.find(path) else { return };
    let (Some(parent), now) = (job.world.get::<plumbline_world::Node>(e).and_then(|n| n.parent.clone()), job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone()).unwrap_or_default()) else { return };
    if called.is_empty() || now == called {
        return;
    }
    let name = if job.children().get(&parent).is_some_and(|c| c.contains_key(called)) { format!("{called} {id}") } else { called.to_string() };
    if let Some(mut node) = job.world.get_mut::<plumbline_world::Node>(e) {
        node.name = name;
    }
}

/// What a thing is as the model this program wrote says it, less the mark of how sure it is:
/// `Ceiling register, 4 x 10 (Known; sample plan)` is `Ceiling register, 4 x 10`.
fn said_less_its_mark(text: &str) -> Option<String> {
    let at = text.rfind(" (")?;
    let inside = &text[at + 2..];
    ["Known;", "Assumed;", "Unknown;"].iter().any(|s| inside.starts_with(s)).then(|| text[..at].trim().to_string()).filter(|t| !t.is_empty())
}

/// What the model calls a kind of thing, as words: `IFCBEAM` is a beam. The model writes
/// every such name as one run of capitals, so the words of the commoner ones are given
/// here; any other is its name less the "IFC", as the model has it.
fn said(class: &str) -> String {
    const WORDS: [(&str, &str); 45] = [
        ("IFCBEAM", "beam"), ("IFCCOLUMN", "column"), ("IFCSTAIR", "stair"), ("IFCSTAIRFLIGHT", "stair flight"), ("IFCROOF", "roof"), ("IFCCOVERING", "covering"), ("IFCFURNITURE", "furniture"),
        ("IFCFURNISHINGELEMENT", "furnishing"), ("IFCBUILDINGELEMENTPROXY", "thing of no stated kind"), ("IFCMEMBER", "framing member"), ("IFCPLATE", "plate"), ("IFCRAILING", "railing"), ("IFCRAMP", "ramp"),
        ("IFCRAMPFLIGHT", "ramp flight"), ("IFCCURTAINWALL", "curtain wall"), ("IFCFOOTING", "footing"), ("IFCPILE", "pile"), ("IFCFLOWTERMINAL", "fixture or outlet"), ("IFCFLOWFITTING", "fitting"),
        ("IFCDUCTFITTING", "duct fitting"), ("IFCPIPEFITTING", "pipe fitting"), ("IFCCABLECARRIERFITTING", "tray fitting"), ("IFCCABLEFITTING", "cable fitting"), ("IFCFLOWSEGMENT", "run"),
        ("IFCDUCTSEGMENT", "duct"), ("IFCPIPESEGMENT", "pipe"), ("IFCCABLECARRIERSEGMENT", "cable tray"), ("IFCCABLESEGMENT", "cable"), ("IFCFLOWCONTROLLER", "valve or damper"),
        ("IFCVALVE", "valve"), ("IFCDAMPER", "damper"), ("IFCAIRTERMINAL", "grille or diffuser"), ("IFCSANITARYTERMINAL", "plumbing fixture"), ("IFCLIGHTFIXTURE", "light fixture"),
        ("IFCOUTLET", "outlet"), ("IFCWALL", "wall"), ("IFCSLAB", "floor slab"), ("IFCROOFSLAB", "roof slab"), ("IFCDOOR", "door"), ("IFCWINDOW", "window"), ("IFCSPACE", "space"), ("IFCOPENINGELEMENT", "opening"),
        ("IFCDISTRIBUTIONPORT", "connection point"), ("IFCGRID", "grid"), ("IFCANNOTATION", "note or dimension"),
    ];
    let base = class.trim_end_matches("STANDARDCASE").trim_end_matches("ELEMENTEDCASE");
    match WORDS.iter().find(|(c, _)| *c == base) {
        Some((_, words)) => words.to_string(),
        None => format!("\"{}\" (as the model calls it)", class.strip_prefix("IFC").unwrap_or(class).to_lowercase()),
    }
}

/// A point inside an outline: the middle of its corners where that is inside it, else
/// the middle of the widest stretch of the outline on a line across it at that height.
/// (The middle of an L-shaped room's corners can lie outside the room.)
fn point_inside(points: &[[f64; 2]]) -> [f64; 2] {
    let n = points.len() as f64;
    let mid = [points.iter().map(|p| p[0]).sum::<f64>() / n, points.iter().map(|p| p[1]).sum::<f64>() / n];
    // Where the outline crosses the line across it at `y`, in order.
    let crossings = |y: f64| -> Vec<f64> {
        let mut xs: Vec<f64> = (0..points.len())
            .filter_map(|i| {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                ((a[1] > y) != (b[1] > y)).then(|| a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]))
            })
            .collect();
        xs.sort_by(f64::total_cmp);
        xs
    };
    let xs = crossings(mid[1]);
    let inside: Vec<(f64, f64)> = xs.chunks_exact(2).map(|c| (c[0], c[1])).collect();
    if inside.iter().any(|(a, b)| *a < mid[0] && mid[0] < *b) {
        return mid;
    }
    match inside.iter().max_by(|p, q| (p.1 - p.0).total_cmp(&(q.1 - q.0))) {
        Some((a, b)) => [(a + b) / 2.0, mid[1]],
        None => mid,
    }
}

/// How many things in the file are written under a number another thing already has.
/// The reader keeps the later of each; this is how many earlier ones it lost.
fn numbers_used_twice(bytes: &[u8], kept: usize) -> usize {
    // Each thing is written `#12= NAME(...);`. Text in quotes can hold anything, so it
    // is stepped over.
    let (mut written, mut i, mut fresh) = (0usize, 0usize, true);
    while i < bytes.len() {
        match bytes[i] {
            b'\'' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
            }
            b';' => fresh = true,
            b'#' if fresh => {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                let digits = j > i + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if digits && bytes.get(j) == Some(&b'=') {
                    written += 1;
                }
                fresh = false;
            }
            c if !c.is_ascii_whitespace() => fresh = false,
            _ => {}
        }
        i += 1;
    }
    written.saturating_sub(kept)
}

/// What the file's lengths are in: the name as the file has it, and feet to one of it.
fn length_unit(r: &Reader) -> (String, f64) {
    const FT_PER_M: f64 = 1.0 / 0.3048;
    let si = |e: &Entity| -> Option<(String, f64)> {
        if !matches!(r.arg(e, 1), V::Enum(k) if k == "LENGTHUNIT") || !matches!(r.arg(e, 3), V::Enum(n) if n == "METRE") {
            return None;
        }
        Some(match r.arg(e, 2) {
            V::Enum(p) if p == "MILLI" => ("millimeters".to_string(), 0.001 * FT_PER_M),
            V::Enum(p) if p == "CENTI" => ("centimeters".to_string(), 0.01 * FT_PER_M),
            V::Enum(p) if p == "DECI" => ("decimeters".to_string(), 0.1 * FT_PER_M),
            _ => ("meters".to_string(), FT_PER_M),
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
                            // "the model is in feet", not "in foot".
                            let name = match name.as_str() {
                                "foot" => "feet".to_string(),
                                "inch" => "inches".to_string(),
                                _ => name,
                            };
                            return (name, factor * base);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    ("meters (the file does not say; an IFC file with no length unit is in meters)".to_string(), FT_PER_M)
}

/// A name for a thing brought in: what the model calls it where nothing else in that
/// place goes by that, else that with its number in the file; a thing the model does not
/// name is its kind and its number.
fn free_name(job: &mut Job, parent: &str, called: &str, kind: &str, id: u64) -> String {
    if called.is_empty() {
        format!("{kind} {id}")
    } else if job.children().get(parent).is_some_and(|c| c.contains_key(called)) {
        format!("{called} {id}")
    } else {
        called.to_string()
    }
}

fn dist3(a: P3, b: P3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
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
    import_whole(job, bytes, name, how).map_err(with_a_way_on)
}

/// What stands in the way of a model being brought in, and what a person can do about it today (call 209 point 4). A model comes
/// in whole: bringing in only what the job lacks is a job of its own and is not built. The two refusals that follow from the
/// model and the job both having the same thing say so, and what can be done.
fn with_a_way_on(why: String) -> String {
    if why.starts_with("this model is already in the job") {
        return format!("{why}. To bring it in again, bring it into a job of its own, or take what it brought in out of this job first. A model comes in whole: bringing in only what the job lacks is not there yet");
    }
    if why.contains("would be counted twice") {
        let said = if why.contains("nothing was brought in") { why } else { format!("{why}. Nothing was brought in") };
        return format!("{said}. To bring this model in, take out of the job what is already there on that ground (here, the floor), or bring the model into a job of its own. A model comes in whole: bringing in only what the job lacks is not there yet");
    }
    why
}

fn import_whole(job: &mut Job, bytes: &[u8], name: &str, how: Entered) -> Result<Brought, String> {
    let file = step::parse(bytes).map_err(|why| format!("it is not an IFC file this can read ({why})"))?;
    if !file.schema.starts_with("IFC") {
        return Err(format!("it is a file written the way a model is, but of another kind (\"{}\"): not an IFC model", file.schema));
    }
    if how.source.trim().is_empty() {
        return Err("say where the model came from".into());
    }
    let r = Reader { file: &file };
    let (unit_name, k) = length_unit(&r);
    let twice = numbers_used_twice(bytes, file.entities.len());
    // A model this program wrote says, value by value, how sure each is (in property
    // sets called Plumbline_Values). Which mark belonged to which value is not read
    // back, so where any is Assumed or Unknown nothing from the model comes in as
    // Known, whatever is said of it as a whole, and the report says so (J17B-16).
    let text = String::from_utf8_lossy(bytes);
    let (assumed, unknown) = if text.contains("'Plumbline_Values'") { (text.matches("(Assumed;").count(), text.matches("(Unknown;").count()) } else { (0, 0) };
    let said_known = how.status == Status::Known;
    // Everything read from a model is Assumed (call 180, C3): a model is what its maker drew, not what
    // was measured here, so a model is never Known on the strength of being said to be.
    let how = if said_known { Entered { status: Status::Assumed, source: how.source } } else { how };
    // What the model holds that is not a thing with a shape, and is not read at all.
    let count = |names: &[&str]| file.entities.values().filter(|e| names.contains(&e.name.as_str())).count();
    let (property_sets, materials) = (count(&["IFCPROPERTYSET", "IFCELEMENTQUANTITY"]), count(&["IFCMATERIAL", "IFCMATERIALLAYERSET", "IFCMATERIALCONSTITUENTSET", "IFCMATERIALPROFILESET"]));
    ed::all_or_nothing(job, |job| {
        // The rooms as they were, for the rooms the model changes to be named (call 206 point 3); a job with no room has none to change.
        let rooms_before = if job.world.query::<&plumbline_world::Room>().iter(&job.world).next().is_some() { ed::room_figures(job) } else { Vec::new() };
        let mut out = bring(job, &r, name, how, unit_name, k)?;
        if !rooms_before.is_empty() {
            let after = ed::room_figures(job);
            out.rooms_moved = ed::rooms_moved(&rooms_before, &after);
            if !out.rooms_moved.is_empty() {
                out.notes.push(format!(
                    "{} of the job {} other figures now, because walls brought in from the model stand in {}: {}",
                    if out.rooms_moved.len() == 1 { "1 room".to_string() } else { format!("{} rooms", out.rooms_moved.len()) },
                    if out.rooms_moved.len() == 1 { "has" } else { "have" },
                    if out.rooms_moved.len() == 1 { "it" } else { "them" },
                    out.rooms_moved.iter().map(|m| m.said()).collect::<Vec<_>>().join("; ")
                ));
            }
        }
        if said_known && assumed + unknown == 0 {
            out.notes.push("everything from this model was brought in Assumed, not Known as was said of it: a model is what its maker drew, not what was measured here. Measure it and mark it Known".to_string());
        }
        if assumed + unknown > 0 {
            out.notes.push(format!(
                "this model was written by BuildOS and marks {assumed} of its values Assumed and {unknown} Unknown: which value each mark was on is not read back, so {}",
                if said_known { "everything from it was brought in Assumed, not Known as was said of it" } else { "nothing from it is surer than was said of it as a whole" }
            ));
        }
        let property_sets = property_sets.saturating_sub(storey_heights(&r).len()).saturating_sub(out.made.get("roofs").copied().unwrap_or(0));
        if property_sets + materials > 0 {
            // What the model keeps on its things besides their shapes, said as a builder would (call 209 point 3): notes, and
            // materials only when there are some.
            let notes = if property_sets == 0 { String::new() } else { format!("the notes the model carries on its things ({property_sets} of them)") };
            let mats = if materials == 0 { String::new() } else { format!("{materials} material{} it names", if materials == 1 { "" } else { "s" }) };
            let both = [notes, mats].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" and ");
            out.notes.push(format!(
                "not read: {both}. A thing brought in has its shape, its place and its name, and, where the model gives them, its work mark (new, existing, to demolish or temporary) and its bearing mark, each Assumed; what the model says it is made of, and anything else it says about each thing, is not brought in"
            ));
        }
        if twice > 0 {
            out.notes.push(format!("{twice} number{} in the file {} used by more than one thing: the later of each was read and the earlier was not, so something may be missing or changed", if twice == 1 { "" } else { "s" }, if twice == 1 { "is" } else { "are" }));
        }
        Ok(out)
    })
}

/// The height each building level is given in the base quantities IFC4 gives a storey
/// (`GrossHeight`, in the model's unit), by the level's number in the file (call 180, C3).
fn storey_heights(r: &Reader) -> HashMap<u64, f64> {
    let mut out = HashMap::new();
    for rel in r.file.entities.values().filter(|e| e.name == "IFCRELDEFINESBYPROPERTIES") {
        let Some((_, set)) = r.get(r.arg(rel, 5)) else { continue };
        if set.name != "IFCELEMENTQUANTITY" || Reader::text(r.arg(set, 2)) != Some("Qto_BuildingStoreyBaseQuantities") {
            continue;
        }
        let height = Reader::list(r.arg(set, 5)).iter().filter_map(|q| r.get(q)).find(|(_, q)| q.name == "IFCQUANTITYLENGTH" && Reader::text(r.arg(q, 0)) == Some("GrossHeight")).and_then(|(_, q)| Reader::number(r.arg(q, 3)));
        let Some(height) = height else { continue };
        for thing in Reader::list(r.arg(rel, 4)) {
            if let V::Ref(id) = thing {
                out.insert(*id, height);
            }
        }
    }
    out
}

fn bring(job: &mut Job, r: &Reader, name: &str, how: Entered, unit_name: String, k: f64) -> Result<Brought, String> {
    let mut out = Brought { schema: r.file.schema.clone(), unit: unit_name, feet_per_unit: k, ..Brought::default() };
    let mut left: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut leave = |class: &str, why: &str| *left.entry((said(class), why.to_string())).or_insert(0) += 1;
    let mut made = |what: &str| *out.made.entry(what.to_string()).or_insert(0) += 1;
    // Every value says the thing in the file it was read off.
    let from = |e: &Entity, id: u64| -> String {
        let called = r.name(e);
        // The model's own word for the thing, less the "IFC" every one of them starts
        // with, and its number in the file, so it can be found there again.
        // (In the trade's word for it, and "#113", as the file itself writes its
        // numbers: "wall 44 in the file" read as forty-four inches, J17B-19.)
        let its = said(&e.name);
        format!("{}: {name}, {}{its} #{id} in the file", how.source, if called.is_empty() { String::new() } else { format!("\"{called}\", ") })
    };
    let val = |source: &str, len: f64| Val { value: Some(Len::One(tidy(len))), unit: "ft".to_string(), status: how.status, source: source.to_string() };
    let path_val = |source: &str, path: &str| Val { value: Some(path.to_string()), unit: String::new(), status: how.status, source: source.to_string() };
    let mut ids: Vec<u64> = r.file.entities.keys().copied().collect();
    ids.sort_unstable();
    let of = |class: &[&str]| -> Vec<(u64, &Entity)> { ids.iter().filter_map(|id| r.file.entities.get(id).map(|e| (*id, e))).filter(|(_, e)| class.contains(&e.name.as_str())).collect() };

    // ---- What the model says of each thing in its property sets: the work mark and whether a
    // wall carries (`Status` and `LoadBearing` in the common property set IFC4 gives its kind),
    // and what this program wrote its type's name as (call 175). Each comes in as the mark of
    // the thing, Assumed, and says the file it came from.
    #[derive(Default)]
    struct OfIt {
        status: Option<String>,
        carries: Option<bool>,
        type_name: Option<String>,
        what: Option<String>,
    }
    let mut said_of: HashMap<u64, OfIt> = HashMap::new();
    for (_, rel) in of(&["IFCRELDEFINESBYPROPERTIES"]) {
        let Some((_, set)) = r.get(r.arg(rel, 5)) else { continue };
        if set.name != "IFCPROPERTYSET" {
            continue;
        }
        let set_name = Reader::text(r.arg(set, 2)).unwrap_or("");
        let mut it = OfIt::default();
        for prop in Reader::list(r.arg(set, 4)) {
            let Some((_, p)) = r.get(prop) else { continue };
            if p.name != "IFCPROPERTYSINGLEVALUE" {
                continue;
            }
            let (called, value) = (Reader::text(r.arg(p, 0)).unwrap_or(""), r.arg(p, 2));
            match called {
                "Status" if set_name.starts_with("Pset_") && set_name.ends_with("Common") => it.status = Reader::text(value).map(str::to_string),
                "LoadBearing" if set_name.starts_with("Pset_") && set_name.ends_with("Common") => {
                    if let V::Typed(_, inner) = value {
                        it.carries = match inner.as_ref() {
                            V::Enum(x) if x == "T" => Some(true),
                            V::Enum(x) if x == "F" => Some(false),
                            _ => None,
                        };
                    }
                }
                "what it is" if set_name == "Plumbline_Values" => it.what = Reader::text(value).and_then(said_less_its_mark),
                "type" if set_name == "Plumbline_Values" => it.type_name = Reader::text(value).and_then(|t| t.find(" (type-").map(|at| t[..at].to_string())).filter(|n| !n.is_empty()),
                _ => {}
            }
        }
        for obj in Reader::list(r.arg(rel, 4)) {
            if let V::Ref(id) = obj {
                let at = said_of.entry(*id).or_default();
                at.status = at.status.take().or(it.status.clone());
                at.carries = at.carries.or(it.carries);
                at.type_name = at.type_name.take().or(it.type_name.clone());
                at.what = at.what.take().or(it.what.clone());
            }
        }
    }
    // The mark a thing comes in with, from what the model says of it; the file's own word
    // for the work, never Known on its say-so.
    let marks = |job: &mut Job, path: &str, id: u64, source: &str, kind: Kind| -> Result<(), String> {
        use plumbline_world::components::{Bearing, Hidden, Work, WorkStatus};
        let Some(it) = said_of.get(&id) else { return Ok(()) };
        if let Some(word) = &it.status {
            let mark = match word.as_str() {
                "NEW" => Some(WorkStatus::New),
                "EXISTING" => Some(WorkStatus::Existing),
                "DEMOLISH" => Some(WorkStatus::Demolish),
                "TEMPORARY" => Some(WorkStatus::Temporary),
                _ => None,
            };
            if let Some(mark) = mark {
                job.put(path, Work { status: Val { value: Some(mark), unit: String::new(), status: Status::Assumed, source: source.to_string() } })?;
            }
        }
        if let (Kind::Wall, Some(carries)) = (kind, it.carries) {
            let bearing = if carries { Bearing::Bearing } else { Bearing::NotBearing };
            job.put(path, Hidden { bearing: Some(Val { value: Some(bearing), unit: String::new(), status: Status::Assumed, source: source.to_string() }), in_the_wall: None })?;
        }
        Ok(())
    };

    // ---- Storeys, lowest first. Each is as tall as it is to the next one up.
    // A storey says how high its floor is (its elevation), and it is also placed. The
    // two differ when the building itself is placed at a height: the floor's height is
    // what the storey says, and everything on the storey is measured from where the
    // storey is placed, so a building 50 m up does not lift its walls 50 m off their floor.
    let mut storeys: Vec<(u64, String, f64, f64)> = of(&["IFCBUILDINGSTOREY"])
        .into_iter()
        .map(|(id, e)| {
            let placed = r.get(r.arg(e, 5)).map(|_| r.placed(r.arg(e, 5)).o[2] * k);
            let stated = Reader::number(r.arg(e, 9)).map(|z| z * k);
            (id, r.name(e), stated.or(placed).unwrap_or(0.0), placed.or(stated).unwrap_or(0.0))
        })
        .collect();
    storeys.sort_by(|a, b| a.2.total_cmp(&b.2));
    if storeys.is_empty() {
        return Err("the model has no building level: there is nothing to stand its walls and floors on".into());
    }
    let lifted: Vec<f64> = storeys.iter().map(|s| s.3 - s.2).filter(|d| d.abs() > 0.001).collect();
    if !lifted.is_empty() {
        out.notes.push(format!(
            "{} of the model's levels {} placed {} from where {} its floor is (the building, or the site, is placed at a height): each floor's height is what the level says, and what stands on a level is measured from where the level is placed",
            lifted.len(),
            if lifted.len() == 1 { "is" } else { "are" },
            feet(lifted[0]),
            if lifted.len() == 1 { "it says" } else { "each says" }
        ));
    }
    // The storeys the job already has, by name: another trade's model of the same
    // building stands on the same floors.
    let mut have: Vec<(String, String, Option<f64>)> = {
        let mut q = job.world.query::<(&plumbline_world::Node, &building::Level)>();
        q.iter(&job.world).map(|(n, l)| (n.name.clone(), n.path.clone(), l.elevation.feet())).collect()
    };
    let heights = storey_heights(r);
    let mut storey_path: HashMap<u64, (String, f64)> = HashMap::new();
    let mut shared: Vec<String> = Vec::new();
    // The levels the job had before this model: a level of the model at the height of one of them is that level, whatever
    // the two are called (call 206 point 3).
    let had: Vec<(String, String, Option<f64>)> = have.clone();
    let mut by_height: Vec<String> = Vec::new();
    for (i, (id, called, elevation, placed)) in storeys.iter().enumerate() {
        let source = from(&r.file.entities[id], *id);
        let entered = Entered { status: how.status, source: &source };
        let mut called = if called.is_empty() { format!("level at {}", feet(*elevation)) } else { called.clone() };
        // A level the job had at this height is that level, under the job's name, within an eighth of an inch.
        let same_height = |(_, _, at): &&(String, String, Option<f64>)| at.is_some_and(|a| (a - elevation).abs() <= SAME_HEIGHT_FT);
        if let Some((jname, path, _)) = had.iter().filter(same_height).find(|(n, _, _)| *n == called).or_else(|| had.iter().find(same_height)) {
            if *jname == called {
                shared.push(called.clone());
            } else {
                let theirs = heights.get(id).map(|h| h * k);
                let ours = job.find(path).and_then(|e| job.world.get::<building::Level>(e)).and_then(|l| l.height.feet());
                let differ = match (theirs, ours) {
                    (Some(t), Some(o)) if (t - o).abs() > SAME_HEIGHT_FT => format!("; the model gives it as {} high and the job has it {} high, and the job's stays", feet(t), feet(o)),
                    _ => String::new(),
                };
                by_height.push(format!("the model's level \"{called}\" has its floor at {}, the height of the job's \"{jname}\": it is that level, and what the model has on it was put on the job's own{differ}", feet(*elevation)));
            }
            storey_path.insert(*id, (path.clone(), *placed));
            continue;
        }
        // A storey of this name already in the job, at this height: it is that storey.
        // At another height the two cannot both be right, and the model is refused.
        if let Some((_, path, at)) = have.iter().find(|(n, _, _)| *n == called).filter(|_| !storey_path.values().any(|(p, _)| have.iter().any(|(n, hp, _)| *n == called && hp == p))) {
            match at {
                Some(at) if (at - elevation).abs() < 0.01 => {
                    shared.push(called.clone());
                    storey_path.insert(*id, (path.clone(), *placed));
                    continue;
                }
                _ => {
                    return Err(format!(
                        "the job already has a level called \"{called}\" with its floor at {}, and the model's level of that name has its floor at {}: they cannot both be right. Give one of them another name, or set the job's level to the model's height, and bring the model in again",
                        at.map(feet).unwrap_or_else(|| "a height nobody has entered".to_string()),
                        feet(*elevation)
                    ));
                }
            }
        }
        // Two storeys of one name in the model (two buildings, each with a first floor):
        // the later says which it is by its height.
        if have.iter().any(|(n, _, _)| *n == called) {
            let renamed = format!("{called}, at {}", feet(*elevation));
            out.notes.push(format!("the model has more than one level called \"{called}\": the one with its floor at {} was brought in as \"{renamed}\"", feet(*elevation)));
            called = renamed;
        }
        // The next level whose floor is higher than this one's: two levels at one height
        // (a mezzanine named apart, a model with a level drawn twice) both take their
        // height from the level above them, where there is one (R21-30: the lower of
        // the two was told "the model has no level above this one", which was untrue).
        let above = storeys[i + 1..].iter().find(|next| next.2 - elevation > 0.01);
        let path = match (heights.get(id), above) {
            // The height the model gives the level, where it gives one (a model this program wrote does).
            (Some(h), _) => ed::storey(job, &called, &feet(*elevation), &format!("{} ft", (h * k * 1.0e6).round() / 1.0e6 + 0.0), entered)?,
            (None, Some(next)) => {
                out.notes.push(format!("the model does not give the height of \"{called}\": it was taken as the distance to the floor of the level above, {}, which is the height and the floor's depth", feet(next.2 - elevation)));
                ed::storey(job, &called, &feet(*elevation), &feet(next.2 - elevation), entered)?
            }
            (None, None) => {
                // The top storey: how tall it is is in nothing the file says.
                let path = ed::storey(job, &called, &feet(*elevation), "1 ft", entered)?;
                let e = job.find(&path).ok_or("the new level is not there")?;
                let mut level = job.world.get::<building::Level>(e).cloned().ok_or("the new level has no level")?;
                level.height = Val::unknown("ft", "the model has no level above this one to say how tall it is");
                job.put(&path, level)?;
                path
            }
        };
        if !had.is_empty() {
            out.notes.push(format!("the model's level \"{called}\" has its floor at {}, which is not the height of any level the job has: it was added as a new level", feet(*elevation)));
        }
        have.push((called, path.clone(), Some(*elevation)));
        storey_path.insert(*id, (path, *placed));
        made("levels");
    }
    out.notes.extend(by_height);
    if !shared.is_empty() {
        out.notes.push(format!("the job already has {}: what the model has on {} was put on the job's own, and no second level was made", shared.iter().map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(", "), if shared.len() == 1 { "it" } else { "them" }));
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
        let below = storeys.iter().rev().find(|s| s.3 <= z + 0.01).unwrap_or(&storeys[0]);
        storey_path[&below.0].clone()
    };

    // ---- Walls.
    let mut wall_of: HashMap<u64, (String, P3, P3, f64)> = HashMap::new();
    let mut wall_marks: Vec<(String, u64, String)> = Vec::new();
    let mut told_apart: Vec<String> = Vec::new();
    for (id, e) in of(&["IFCWALL", "IFCWALLSTANDARDCASE", "IFCWALLELEMENTEDCASE"]) {
        let body = match r.upright(e) {
            Ok(body) => body,
            Err(why) => {
                leave(&e.name, why);
                continue;
            }
        };
        let (a, b, length, way, thickness, base) = (body.a, body.b, body.length, body.way, body.thickness, body.base);
        let (storey, floor) = storey_of(id, base * k);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let made_wall = (|| -> Result<String, String> {
            let size = format!("{} in", tidy(thickness * k * 12.0));
            let wall_type = match said_of.get(&id).and_then(|it| it.type_name.as_deref()) {
                Some(called) => ed::wall_type_named(job, called, &size, entered)?,
                None => ed::wall_type_for(job, &size, entered)?,
            };
            if let Some(called) = said_of.get(&id).and_then(|it| it.type_name.as_deref()) {
                let got = job.find(&wall_type).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone());
                if let Some(got) = got.filter(|g| g != called) {
                    let said = format!("\"{called}\" as \"{got}\"");
                    if !told_apart.contains(&said) {
                        told_apart.push(said);
                    }
                }
            }
            let made = ed::walls(job, &storey, &wall_type, &[[tidy(a[0] * k), tidy(a[1] * k)], [tidy(b[0] * k), tidy(b[1] * k)]], false, ed::Top::Height(&feet(body.depth * k)), entered)?;
            let path = made.into_iter().next().ok_or("no wall was made")?;
            // (A wall's mark is put on after its doors and windows are in: a door is not set in a wall that is
            // coming down, and the model has the door in that wall: call 179, C6.)
            wall_marks.push((path.clone(), id, source.clone()));
            // The model's own name for the wall, where it has one nothing else on the
            // storey goes by.
            let called = r.name(e);
            if !called.is_empty() && !job.children().get(&storey).is_some_and(|c| c.contains_key(&called)) {
                let w = job.find(&path).ok_or("the new wall is not there")?;
                if let Some(mut node) = job.world.get_mut::<plumbline_world::Node>(w) {
                    node.name = called;
                }
            }
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
            Err(why) => leave(&e.name, &why),
        }
    }
    if !told_apart.is_empty() {
        out.notes.push(format!("the model names a wall type the job already has at another thickness: {} came in with a number after the name, so that no wall was left out for its type's name", told_apart.join(", ")));
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
        let type_for = match said_of.get(&id).and_then(|it| it.type_name.as_deref()) {
            Some(called) => ed::opening_type_named(job, kind, called, &size(width), &size(height), entered),
            None => ed::opening_type_for(job, kind, &size(width), &size(height), entered),
        };
        let done = type_for.and_then(|t| ed::opening(job, wall, kind, &t, ed::OnWall::Offset(&feet(along * k)), &feet(sill * k), entered)).and_then(|path| marks(job, &path, id, &source, kind).map(|()| path));
        match done {
            Ok(path) => {
                keep_name(job, &path, &r.name(e), id);
                made(if kind == Kind::Door { "doors" } else { "windows" })
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    for (path, id, source) in &wall_marks {
        if let Err(why) = marks(job, path, *id, source, Kind::Wall) {
            out.notes.push(format!("a mark on {path} was not put on it: {why}"));
        }
    }
    for (id, e) in of(&["IFCOPENINGELEMENT", "IFCOPENINGSTANDARDCASE"]) {
        if !fills.values().any(|hole| *hole == id) {
            leave(&e.name, "a hole with no door or window in it: this program cuts a wall only for one of those, and a floor for nothing yet");
        }
    }
    // ---- The program's own roof (call 197, point 4). A roof this program wrote carries a list of
    // what is needed to make it again, `Plumbline_Roof`: its outline round the eaves, what each edge
    // does (a pitch, or a gable) and how high the eaves are. It is made by the change the Roof tool
    // makes (`solve::new_roof`), from that, Assumed and sourced to the file. A roof with no such list
    // is a roof from another program, and is left out as it was.
    let mut stood_on: Vec<(String, [f64; 3], [f64; 3])> = Vec::new();
    let mut done: HashSet<u64> = HashSet::new();
    // Where each run brought in lies, in feet and as the file places it: its ends.
    let mut run_ends: Vec<(String, P3, P3)> = Vec::new();
    // The same, by the run's number in the file, for what the file's ports say is joined to it.
    let mut run_by_id: HashMap<u64, (String, P3, P3)> = HashMap::new();
    // ---- Ports (call 200): what the file says is connected. A port is on what it is nested in
    // (`IfcRelNests`, in order; IFC2x3 and older files tie a port to its element by
    // `IfcRelConnectsPortToElement`), and two ports are tied by `IfcRelConnectsPorts`. A connection point is never a
    // thing of its own here.
    let port_ids: HashSet<u64> = of(&["IFCDISTRIBUTIONPORT", "IFCPORT"]).into_iter().map(|(id, _)| id).collect();
    let mut owner_of: HashMap<u64, u64> = HashMap::new();
    let mut ports_of: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, rel) in of(&["IFCRELNESTS"]) {
        let V::Ref(whole) = r.arg(rel, 4) else { continue };
        for part in Reader::list(r.arg(rel, 5)) {
            if let V::Ref(p) = part {
                if port_ids.contains(p) {
                    owner_of.insert(*p, *whole);
                    ports_of.entry(*whole).or_default().push(*p);
                }
            }
        }
    }
    for (_, rel) in of(&["IFCRELCONNECTSPORTTOELEMENT"]) {
        if let (V::Ref(p), V::Ref(whole)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            if port_ids.contains(p) && !owner_of.contains_key(p) {
                owner_of.insert(*p, *whole);
                ports_of.entry(*whole).or_default().push(*p);
            }
        }
    }
    let mut tied_to: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, rel) in of(&["IFCRELCONNECTSPORTS"]) {
        if let (V::Ref(a), V::Ref(b)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            tied_to.entry(*a).or_default().push(*b);
            tied_to.entry(*b).or_default().push(*a);
        }
    }
    let file_has_ports = !port_ids.is_empty();
    for id in &port_ids {
        done.insert(*id);
    }
    let mut runs_unsized = 0usize;
    let mut roof_slabs: HashSet<u64> = HashSet::new();
    {
        let mut sets: HashMap<u64, &Entity> = HashMap::new();
        for (_, rel) in of(&["IFCRELDEFINESBYPROPERTIES"]) {
            let Some((_, set)) = r.get(r.arg(rel, 5)) else { continue };
            if set.name == "IFCPROPERTYSET" && Reader::text(r.arg(set, 2)) == Some("Plumbline_Roof") {
                for obj in Reader::list(r.arg(rel, 4)) {
                    if let V::Ref(id) = obj {
                        sets.insert(*id, set);
                    }
                }
            }
        }
        let mut parts_of: HashMap<u64, Vec<u64>> = HashMap::new();
        for (_, rel) in of(&["IFCRELAGGREGATES"]) {
            if let V::Ref(whole) = r.arg(rel, 4) {
                for part in Reader::list(r.arg(rel, 5)) {
                    if let V::Ref(id) = part {
                        parts_of.entry(*whole).or_default().push(*id);
                    }
                }
            }
        }
        for (id, e) in of(&["IFCROOF"]) {
            let Some(set) = sets.get(&id) else { continue };
            let prop = |name: &str| -> Option<&Entity> { Reader::list(r.arg(set, 4)).iter().filter_map(|p| r.get(p)).map(|(_, p)| p).find(|p| Reader::text(r.arg(p, 0)) == Some(name)) };
            let parsed = (|| -> Result<(Vec<[f64; 2]>, Vec<Option<f64>>, f64), String> {
                let outline = prop("Outline").ok_or("it has no outline")?;
                let numbers: Vec<f64> = Reader::list(r.arg(outline, 2)).iter().filter_map(Reader::number).collect();
                if numbers.len() < 6 || numbers.len() % 2 != 0 {
                    return Err("its outline is not three or more corners".into());
                }
                let corners: Vec<[f64; 2]> = numbers.chunks(2).map(|c| [tidy(c[0] * k), tidy(c[1] * k)]).collect();
                let rises = prop("EdgeRise").ok_or("it does not say what each edge does")?;
                let edges = Reader::list(r.arg(rises, 2))
                    .iter()
                    .map(|v| {
                        let said = Reader::text(v).ok_or("an edge's rise is not written as a pitch or as a gable")?.trim().to_string();
                        if said.eq_ignore_ascii_case("gable") {
                            return Ok(None);
                        }
                        said.strip_suffix(":12").and_then(|n| n.trim().parse::<f64>().ok()).filter(|n| n.is_finite()).map(Some).ok_or(format!("\"{said}\" is not a pitch"))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                if edges.len() != corners.len() {
                    return Err(format!("it has {} corners and {} edges", corners.len(), edges.len()));
                }
                let height = prop("EaveHeight").and_then(|p| Reader::number(r.arg(p, 2))).ok_or("it does not say how high its eaves are")?;
                Ok((corners, edges, tidy(height * k)))
            })();
            let source = from(e, id);
            let made_roof = parsed.and_then(|(mut corners, mut edges, height)| {
                // Drawn anticlockwise, as the tool turns what is picked the other way about.
                let twice: f64 = (0..corners.len()).map(|i| corners[i][0] * corners[(i + 1) % corners.len()][1] - corners[(i + 1) % corners.len()][0] * corners[i][1]).sum();
                if twice < 0.0 {
                    let n = corners.len();
                    corners.reverse();
                    edges = (0..n).map(|j| edges[(2 * n - 2 - j) % n]).collect();
                }
                let sure = |x| Val { value: Some(x), unit: units::PITCH.to_string(), status: how.status, source: source.clone() };
                let footprint = Footprint {
                    edges: edges.iter().map(|x| match x { Some(p) => sure(Rise::Pitch(*p)), None => Val { value: Some(Rise::Gable), unit: String::new(), status: how.status, source: source.clone() } }).collect(),
                    points: Val { value: Some(corners), unit: units::FT.to_string(), status: how.status, source: source.clone() },
                    eave_height: Val { value: Some(Len::One(height)), unit: units::FT.to_string(), status: how.status, source: source.clone() },
                };
                // A roof with this very outline and these edges already in the job would be counted twice.
                let there = {
                    let mut q = job.world.query::<&Footprint>();
                    q.iter(&job.world).any(|f| f.points_ft().is_some_and(|p| p.len() == footprint.points.value.as_ref().map_or(0, |c| c.len()) && p.iter().zip(footprint.points.value.iter().flatten()).all(|(a, b)| (a[0] - b[0]).abs() < 0.001 && (a[1] - b[1]).abs() < 0.001)) && f.edges.iter().map(|e| e.value).eq(footprint.edges.iter().map(|e| e.value)))
                };
                if there {
                    return Err("a roof with this very outline and these edges is already in the job: a second would be counted twice".to_string());
                }
                let taken: Vec<String> = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &Kind)>();
                    q.iter(&job.world).filter(|(_, kind)| **kind == Kind::Roof).map(|(n, _)| n.name.clone()).collect()
                };
                let called = r.name(e);
                let called = if called.is_empty() || taken.contains(&called) { (1..).map(|n| if n == 1 { "roof".to_string() } else { format!("roof {n}") }).find(|n| !taken.contains(n)).unwrap_or_default() } else { called };
                solve::new_roof(job, &called, footprint)
            });
            let slabs = parts_of.get(&id).cloned().unwrap_or_default();
            match made_roof {
                Ok((path, report)) => {
                    done.insert(id);
                    roof_slabs.extend(slabs.iter().copied());
                    if let Err(why) = marks(job, &path, id, &source, Kind::Roof) {
                        out.notes.push(format!("a mark on {path} was not put on it: {why}"));
                    }
                    made("roofs");
                    for _ in 0..report.planes {
                        made("roof planes");
                    }
                    // The planes made are set beside the roof slabs in the file.
                    job.derive();
                    let mine: Vec<f64> = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &plumbline_world::Measure)>();
                        q.iter(&job.world).filter(|(n, kind, _)| **kind == Kind::RoofPlane && n.parent.as_deref() == Some(path.as_str())).filter_map(|(_, _, m)| m.0.get("surface_area").and_then(|d| d.value)).collect()
                    };
                    let theirs: Vec<f64> = slabs.iter().filter_map(|s| r.file.entities.get(s)).filter_map(|s| r.face_area(s)).map(|a| a * k * k).collect();
                    let (mut a, mut b) = (mine.clone(), theirs.clone());
                    a.sort_by(f64::total_cmp);
                    b.sort_by(f64::total_cmp);
                    let agree = a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 0.01);
                    if !agree {
                        let total = |v: &[f64]| (v.iter().sum::<f64>() * 100.0).round() / 100.0;
                        out.notes.push(format!(
                            "the roof's planes made here are {} and come to {} SF, and the roof slabs in the file are {} and come to {} SF: they differ, so the roof here is not the roof in the file (made from the roof's outline, its pitches and its eave height, which the file gives)",
                            a.len(),
                            total(&a),
                            b.len(),
                            total(&b)
                        ));
                    }
                }
                Err(why) => {
                    done.insert(id);
                    // Its slabs are part of it: the roof says why it was left out, and they are not left out again for a reason
                    // that is not so (they have the outline).
                    roof_slabs.extend(slabs.iter().copied());
                    if let Some(reach) = r.reach(e) {
                        stood_on.push((said(&e.name), [reach.0[0] * k, reach.0[1] * k, reach.0[2] * k], [reach.1[0] * k, reach.1[1] * k, reach.1[2] * k]));
                    }
                    leave(&e.name, &why);
                }
            }
        }
    }
    // ---- Floor slabs.
    let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
    let mut floor_types: HashMap<i64, String> = HashMap::new();
    let slab_ids: Vec<u64> = of(&["IFCSLAB", "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE"]).into_iter().map(|(id, _)| id).collect();
    let cut_floors: Vec<u64> = hole_in.values().copied().filter(|host| slab_ids.contains(host)).collect();
    let mut floors_cut = 0usize;
    for (id, e) in of(&["IFCSLAB", "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE"]) {
        // A slab the model says is a roof is a roof, not a floor (J17B-18).
        if matches!(r.arg(e, 8), V::Enum(k) if k == "ROOF") {
            if roof_slabs.contains(&id) {
                continue;
            }
            leave("IFCROOFSLAB", NO_ROOF_OUTLINE);
            continue;
        }
        if r.pieces(e) > 1 {
            leave(&e.name, "its body is made of more than one piece: only a body of one piece is read");
            continue;
        }
        if r.holed(e) {
            leave(&e.name, "its outline has a hole in it: this program has no floor with a hole yet, and would count the hole as floor");
            continue;
        }
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its body could not be read as an outline pushed straight: it may slope, be a mesh, or name a part that is not in the file");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let way = frame.turn(body.way);
        if way[2].abs() < UPRIGHT || frame.z[2].abs() < UPRIGHT {
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
                    // A floor type of this name and thickness the job already has (from
                    // a model brought in before) is this one.
                    let named = format!("floor {thickness} in");
                    let has = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &FloorType)>();
                        q.iter(&job.world).find(|(n, t)| n.name == named && t.thickness.feet().is_some_and(|f| (f * 12.0 - thickness).abs() < 1.0e-6)).map(|(n, _)| n.path.clone())
                    };
                    let path = match has {
                        Some(path) => path,
                        None => {
                            let path = job.add(&root, Kind::Type, &named)?;
                            let t = FloorType { thickness: Val { value: Some(Len::One(thickness)), unit: "in".into(), status: how.status, source: source.clone() }, layers: Vec::new() };
                            building::check_floor_type(&path, Kind::Type, &t)?;
                            job.put(&path, t)?;
                            path
                        }
                    };
                    floor_types.insert(key, path.clone());
                    path
                }
            };
            // A floor with this very outline already on the storey (the model brought in
            // a second time, or two trades' models that both carry the slab) would be
            // counted twice: it is not made again.
            let there = {
                let mut q = job.world.query::<(&plumbline_world::Node, &Floor)>();
                q.iter(&job.world).any(|(n, f)| n.parent.as_deref() == Some(storey.as_str()) && f.boundary_ft().is_some_and(|b| b.len() == boundary.len() && b.iter().zip(&boundary).all(|(p, q)| (p[0] - q[0]).abs() < 0.001 && (p[1] - q[1]).abs() < 0.001)))
            };
            if there {
                return Err("a floor with this very outline is already on this level: a second would be counted twice".into());
            }
            let called = free_name(job, &storey, &r.name(e), "floor", id);
            let path = job.add(&storey, Kind::Floor, &called)?;
            let offset = top * k - floor;
            // A floor the model cuts an opening in (a stair, a shaft): the opening is
            // not cut here, so its outline is more than the floor. That is said in the
            // outline's source, and the outline is no surer than Assumed.
            let cut = cut_floors.contains(&id);
            let outline = if cut { Val { value: Some(boundary.clone()), unit: "ft".into(), status: how.status.max(plumbline_world::Status::Assumed), source: format!("{source}; the model cuts an opening in this floor that is not cut here, so this outline is more than the floor") } } else { Val { value: Some(boundary.clone()), unit: "ft".into(), status: how.status, source: source.clone() } };
            let f = Floor { type_name: path_val(&source, &floor_type), boundary: outline, offset: (offset.abs() > 0.001).then(|| val(&source, offset)) };
            building::check_floor(&path, Kind::Floor, &f)?;
            job.put(&path, f)
        })();
        match done {
            Ok(()) => {
                made("floors");
                if cut_floors.contains(&id) {
                    floors_cut += 1;
                }
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    if floors_cut > 0 {
        out.notes.push(format!("{floors_cut} floor{} an opening cut in {} in the model (a stair, a shaft): the opening is not cut here, so {} area is more than the floor. Each is flagged Assumed and says so", if floors_cut == 1 { " has" } else { "s have" }, if floors_cut == 1 { "it" } else { "them" }, if floors_cut == 1 { "its" } else { "each one's" }));
    }
    // ---- Spaces, as rooms: a point inside each.
    for (id, e) in of(&["IFCSPACE"]) {
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its shape is not an outline pushed straight, so no point inside it could be found");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let middle = match &body.profile {
            Profile::Outline(points) => point_inside(points),
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
            Err(why) => leave(&e.name, &why),
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
        // A kind IFC has no word for is written by its name (the object type and the long name): a
        // name this program has for a system is that system.
        let by_name = || -> Option<&'static str> {
            [r.arg(group, 4), r.arg(group, 5)].into_iter().filter_map(Reader::text).find_map(|t| building::CLASSIFICATIONS.iter().find(|(c, _)| c.eq_ignore_ascii_case(t.trim())).map(|(c, _)| *c))
        };
        let class = match r.arg(group, 6) {
            V::Enum(t) => match t.as_str() {
                "USERDEFINED" => by_name(),
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
        // The job's own system of that kind where it has one; else a new one called what the file
        // calls it, with its capitals (call 197, point 5).
        let have: Option<String> = {
            let mut q = job.world.query::<(&plumbline_world::Node, &building::SystemInfo)>();
            let mut all: Vec<String> = q.iter(&job.world).filter(|(_, s)| s.classification.eq_ignore_ascii_case(class)).map(|(n, _)| n.path.clone()).collect();
            all.sort();
            all.into_iter().next()
        };
        let path = match have {
            Some(path) => path,
            None => {
                let called = r.name(group);
                let taken = job.children().get(&root).is_some_and(|c| c.contains_key(called.as_str()));
                if called.is_empty() || taken { ed::system_of_class(job, class)? } else { ed::system(job, &called, class)? }
            }
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
        if r.pieces(e) > 1 {
            leave(&e.name, "its body is made of more than one piece: only a body of one piece is read");
            continue;
        }
        // A run with no size is drawn as its line alone: it comes in as the run it is, its size Unknown (call 200).
        let line_only = if r.pushed(e).is_none() && r.pieces(e) == 0 { r.line_only(e) } else { None };
        let (start, end, across): (P3, P3, Option<(f64, Option<f64>)>) = if let Some((p, q)) = line_only {
            (p, q, None)
        } else {
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
            (start, end, Some(across))
        };
        let (storey, floor) = storey_of(id, start[2].min(end[2]) * k);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let inches = |x: f64| format!("{} in", tidy(x * k * 12.0));
        let done = (|| -> Result<String, String> {
            let run_type: Option<String> = match across {
                None => None,
                Some((width, None)) => Some(ed::segment_type_for(job, kind, &inches(width), entered)?),
                Some((width, Some(depth))) => {
                    let called = format!("{} by {}", inches(width), inches(depth));
                    let known = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &building::SegmentType)>();
                        q.iter(&job.world).find(|(n, t)| n.name == called && t.category == kind).map(|(n, _)| n.path.clone())
                    };
                    Some(match known {
                        Some(path) => path,
                        None => ed::segment_type(job, &called, kind, &inches(width), Some(&inches(depth)), entered)?,
                    })
                }
            };
            let (z0, z1) = (start[2] * k - floor, end[2] * k - floor);
            let line = [[tidy(start[0] * k), tidy(start[1] * k)], [tidy(end[0] * k), tidy(end[1] * k)]];
            // A run of this kind in this very place already on the storey would be
            // counted twice: it is not made again.
            let there = {
                let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Segment)>();
                q.iter(&job.world).any(|(n, other, seg)| {
                    n.parent.as_deref() == Some(storey.as_str()) && *other == kind && seg.line_ft().is_some_and(|l| (0..2).all(|i| (0..2).all(|j| (l[i][j] - line[i][j]).abs() < 0.001))) && seg.offsets_ft().is_some_and(|(a, b)| (a - z0).abs() < 0.001 && (b - z1).abs() < 0.001)
                })
            };
            if there {
                return Err(format!("{} in this very place is already on this level: a second would be counted twice", a_kind(kind)));
            }
            let called = free_name(job, &storey, &r.name(e), kind.stem(), id);
            let path = job.add(&storey, kind, &called)?;
            let segment = Segment {
                type_name: match &run_type {
                    Some(t) => path_val(&source, t),
                    None => Val::unknown("", "the model gives this run no size: it is drawn as its line alone"),
                },
                line: Val { value: Some(line), unit: "ft".into(), status: how.status, source: source.clone() },
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
            Ok(path)
        })();
        match done {
            Ok(path) => {
                if across.is_none() {
                    runs_unsized += 1;
                }
                run_by_id.insert(id, (path.clone(), [start[0] * k, start[1] * k, start[2] * k], [end[0] * k, end[1] * k, end[2] * k]));
                run_ends.push((path, [start[0] * k, start[1] * k, start[2] * k], [end[0] * k, end[1] * k, end[2] * k]));
                made(match kind {
                    Kind::Duct => "ducts",
                    Kind::Pipe => "pipes",
                    Kind::Conduit => "conduits",
                    _ => "cables",
                })
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    // The file's units to feet, for what stood where a left-out thing did (call 197, point 6).
    let reach_of = |e: &Entity| -> Option<([f64; 3], [f64; 3])> { r.reach(e).map(|(lo, hi)| ([lo[0] * k, lo[1] * k, lo[2] * k], [hi[0] * k, hi[1] * k, hi[2] * k])) };
    let has_place = |e: &Entity| r.get(r.arg(e, 5)).is_some_and(|(_, p)| p.name == "IFCLOCALPLACEMENT");
    // ---- Footings (call 197, point 3): a strip footing as a footing along its line, a pad footing as a
    // pad, each by the change that makes one. A footing of another sort, or whose body is not a straight
    // box, is left out with the reason.
    for (id, e) in of(&["IFCFOOTING"]) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let sort = match r.arg(e, 8) {
            V::Enum(t) => t.as_str(),
            _ => "",
        };
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let outcome: Result<String, String> = (|| {
            let kept = said_of.get(&id).and_then(|it| it.type_name.clone());
            match sort {
                "STRIP_FOOTING" => {
                    let body = r.upright(e).map_err(|why| why.to_string())?;
                    let (storey, floor) = storey_of(id, body.base * k);
                    if ((body.base + body.depth) * k - floor).abs() > 0.001 {
                        return Err("its top is not at its level's floor: a footing here has its top at the floor of its level".to_string());
                    }
                    let width = format!("{} in", tidy(body.thickness * k * 12.0));
                    let wanted = {
                        let (len, unit) = units::parse_length(&width)?;
                        len.feet(unit)?
                    };
                    let existing: Option<String> = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::WallType)>();
                        let mut all: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.footing && t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
                        all.sort();
                        all.into_iter().next()
                    };
                    let type_path = match existing {
                        Some(path) => path,
                        None => {
                            let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
                            let wanted_name = kept.clone().unwrap_or_else(|| format!("footing {width}"));
                            let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                            ed::footing_type(job, &name, &width, entered)?
                        }
                    };
                    let line = [[tidy(body.a[0] * k), tidy(body.a[1] * k)], [tidy(body.b[0] * k), tidy(body.b[1] * k)]];
                    // A footing along this very line already on the level (the model brought in a second time)
                    // would be counted twice: it is not made again.
                    let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() < 0.001 && (p[1] - q[1]).abs() < 0.001;
                    let there = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Wall)>();
                        q.iter(&job.world).any(|(n, kind, w)| *kind == Kind::Footing && n.parent.as_deref() == Some(storey.as_str()) && w.line_ft().is_some_and(|l| (near(l[0], line[0]) && near(l[1], line[1])) || (near(l[0], line[1]) && near(l[1], line[0]))))
                    };
                    if there {
                        return Err("it is already on this level: a footing along this very line would be counted twice".to_string());
                    }
                    let made = ed::footings(job, &storey, &type_path, &line, false, &feet(body.depth * k), entered)?;
                    made.into_iter().next().ok_or_else(|| "no footing was made".to_string())
                }
                "PAD_FOOTING" => {
                    let b = r.boxed(e).ok_or("its body is not a straight box")?;
                    if b.angle.abs() > 1.0e-6 {
                        return Err("it is turned on the plan: a pad footing here stands square to it".to_string());
                    }
                    let (storey, floor) = storey_of(id, b.bottom * k);
                    let (across, deep) = (format!("{} in", tidy(b.size[0] * k * 12.0)), format!("{} in", tidy(b.size[1] * k * 12.0)));
                    let ft = |typed: &str| -> Result<f64, String> {
                        let (len, unit) = units::parse_length(typed)?;
                        len.feet(unit)
                    };
                    let wanted = (ft(&across)?, ft(&deep)?);
                    let existing: Option<String> = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::SegmentType)>();
                        let mut all: Vec<String> = q
                            .iter(&job.world)
                            .filter(|(_, t)| t.category == Kind::PadFooting && t.round == b.round && t.width.feet().is_some_and(|x| (x - wanted.0).abs() < 1e-9) && (b.round || t.depth.as_ref().and_then(|d| d.feet()).is_some_and(|x| (x - wanted.1).abs() < 1e-9)))
                            .map(|(n, _)| n.path.clone())
                            .collect();
                        all.sort();
                        all.into_iter().next()
                    };
                    let type_path = match existing {
                        Some(path) => path,
                        None => {
                            let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
                            let wanted_name = kept.clone().unwrap_or_else(|| if b.round { format!("pad footing {across} round") } else { format!("pad footing {across} by {deep}") });
                            let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                            ed::framing_type(job, &name, Kind::PadFooting, &across, if b.round { None } else { Some(deep.as_str()) }, None, entered)?
                        }
                    };
                    ed::pad(job, &storey, &type_path, [tidy(b.middle[0] * k), tidy(b.middle[1] * k)], &format!("{} in", tidy(b.size[2] * k * 12.0)), &feet((b.bottom + b.size[2]) * k - floor), entered)
                }
                "" | "NOTDEFINED" => Err("the model does not say whether it is a strip footing or a pad footing".to_string()),
                other => Err(format!("its sort is a {}, which is neither a strip footing nor a pad footing: only those two are brought in", other.to_lowercase().replace('_', " "))),
            }
        })();
        match outcome {
            Ok(path) => {
                keep_name(job, &path, &r.name(e), id);
                if let Err(why) = marks(job, &path, id, &source, Kind::Footing) {
                    out.notes.push(format!("a mark on {path} was not put on it: {why}"));
                }
                made(if sort == "PAD_FOOTING" { "pad footings" } else { "footings" });
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- Beams and columns (call 204, point 5). A beam is a rectangle pushed level along its line, a column one pushed upright:
    // each comes in by the change that makes one (a beam along its line with its top, a post at its place, standing so tall), its
    // section as a type of its kind, sized from the body, Assumed and sourced to the file. A body that is not a straight box is
    // left out with the reason that is true of it.
    for (id, e) in of(&["IFCBEAM", "IFCCOLUMN"]) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let is_beam = e.name == "IFCBEAM";
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let outcome: Result<String, String> = (|| {
            let kept = said_of.get(&id).and_then(|it| it.type_name.clone());
            let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
            let ft = |typed: &str| -> Result<f64, String> {
                let (len, unit) = units::parse_length(typed)?;
                len.feet(unit)
            };
            if is_beam {
                let b = r.beam(e).map_err(|why| why.to_string())?;
                let (storey, floor) = storey_of(id, b.bottom * k);
                let (across, deep) = (format!("{} in", tidy(b.width * k * 12.0)), format!("{} in", tidy(b.depth * k * 12.0)));
                let wanted = (ft(&across)?, ft(&deep)?);
                let existing: Option<String> = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::SegmentType)>();
                    let mut all: Vec<String> = q
                        .iter(&job.world)
                        .filter(|(_, t)| t.category == Kind::Beam && !t.round && t.width.feet().is_some_and(|x| (x - wanted.0).abs() < 1e-9) && t.depth.as_ref().and_then(|d| d.feet()).is_some_and(|x| (x - wanted.1).abs() < 1e-9))
                        .map(|(n, _)| n.path.clone())
                        .collect();
                    all.sort();
                    all.into_iter().next()
                };
                let type_path = match existing {
                    Some(path) => path,
                    None => {
                        let wanted_name = kept.clone().unwrap_or_else(|| format!("beam {across} by {deep}"));
                        let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                        ed::framing_type(job, &name, Kind::Beam, &across, Some(deep.as_str()), None, entered)?
                    }
                };
                let line = [[tidy(b.a[0] * k), tidy(b.a[1] * k)], [tidy(b.b[0] * k), tidy(b.b[1] * k)]];
                // A beam along this very line already on the level (the model brought in a second time) would be counted twice.
                let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() < 0.001 && (p[1] - q[1]).abs() < 0.001;
                let there = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Segment)>();
                    q.iter(&job.world).any(|(n, kind, s)| *kind == Kind::Beam && n.parent.as_deref() == Some(storey.as_str()) && s.line_ft().is_some_and(|l| (near(l[0], line[0]) && near(l[1], line[1])) || (near(l[0], line[1]) && near(l[1], line[0]))))
                };
                if there {
                    return Err("it is already on this level: a beam along this very line would be counted twice".to_string());
                }
                // Held to its top, as a beam here is: its middle and half its depth, above the floor.
                let top = (b.middle_z + b.depth / 2.0) * k - floor;
                let made = ed::beams(job, &storey, &type_path, &line, &feet(top), entered)?;
                made.into_iter().next().ok_or_else(|| "no beam was made".to_string())
            } else {
                let b = r.boxed(e).ok_or("its body is not a straight box, standing upright, as far as this can read it")?;
                if b.angle.abs() > 1.0e-6 {
                    return Err("it is turned on the plan: a post here stands square to it".to_string());
                }
                let (storey, floor) = storey_of(id, b.bottom * k);
                let (across, deep) = (format!("{} in", tidy(b.size[0] * k * 12.0)), format!("{} in", tidy(b.size[1] * k * 12.0)));
                let wanted = (ft(&across)?, ft(&deep)?);
                let existing: Option<String> = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::SegmentType)>();
                    let mut all: Vec<String> = q
                        .iter(&job.world)
                        .filter(|(_, t)| t.category == Kind::Post && t.round == b.round && t.width.feet().is_some_and(|x| (x - wanted.0).abs() < 1e-9) && (b.round || t.depth.as_ref().and_then(|d| d.feet()).is_some_and(|x| (x - wanted.1).abs() < 1e-9)))
                        .map(|(n, _)| n.path.clone())
                        .collect();
                    all.sort();
                    all.into_iter().next()
                };
                let type_path = match existing {
                    Some(path) => path,
                    None => {
                        let wanted_name = kept.clone().unwrap_or_else(|| if b.round { format!("post {across} round") } else { format!("post {across} by {deep}") });
                        let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                        ed::framing_type(job, &name, Kind::Post, &across, if b.round { None } else { Some(deep.as_str()) }, None, entered)?
                    }
                };
                let at = [tidy(b.middle[0] * k), tidy(b.middle[1] * k)];
                let there = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &plumbline_world::Stood)>();
                    q.iter(&job.world).any(|(n, kind, c)| *kind == Kind::Post && n.parent.as_deref() == Some(storey.as_str()) && c.at_ft().is_some_and(|p| (p[0] - at[0]).abs() < 0.001 && (p[1] - at[1]).abs() < 0.001) && c.base_offset.feet().is_some_and(|x| (x - ((b.bottom * k) - floor)).abs() < 0.001))
                };
                if there {
                    return Err("it is already on this level: a post at this very place would be counted twice".to_string());
                }
                ed::post(job, &storey, &type_path, at, Some(&feet(b.size[2] * k)), Some(&feet(b.bottom * k - floor)), entered)
            }
        })();
        match outcome {
            Ok(path) => {
                keep_name(job, &path, &r.name(e), id);
                if let Err(why) = marks(job, &path, id, &source, if is_beam { Kind::Beam } else { Kind::Post }) {
                    out.notes.push(format!("a mark on {path} was not put on it: {why}"));
                }
                made(if is_beam { "beams" } else { "posts" });
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- Things set down: fixtures, equipment and furnishings (call 197, point 1). Each is a box the
    // size of its body in the file, set down by the change that sets one down, Assumed, and called what
    // the file calls it. A body this does not read leaves its size unknown, and says so.
    const FIXTURES: [&str; 7] = ["IFCSANITARYTERMINAL", "IFCFLOWTERMINAL", "IFCOUTLET", "IFCAIRTERMINAL", "IFCLIGHTFIXTURE", "IFCSTACKTERMINAL", "IFCWASTETERMINAL"];
    const FURNISHINGS: [&str; 3] = ["IFCFURNITURE", "IFCFURNISHINGELEMENT", "IFCSYSTEMFURNITUREELEMENT"];
    const EQUIPMENT: [&str; 28] = [
        "IFCDISTRIBUTIONFLOWELEMENT", "IFCDISTRIBUTIONELEMENT", "IFCDISTRIBUTIONCONTROLELEMENT", "IFCDISTRIBUTIONCHAMBERELEMENT", "IFCENERGYCONVERSIONDEVICE", "IFCFLOWMOVINGDEVICE", "IFCFLOWSTORAGEDEVICE", "IFCFLOWTREATMENTDEVICE",
        "IFCELECTRICDISTRIBUTIONBOARD", "IFCUNITARYEQUIPMENT", "IFCBOILER", "IFCPUMP", "IFCFAN", "IFCTANK", "IFCTRANSFORMER", "IFCCOMPRESSOR", "IFCCOIL", "IFCCONDENSER", "IFCEVAPORATOR", "IFCHEATEXCHANGER", "IFCFILTER",
        "IFCELECTRICGENERATOR", "IFCSWITCHINGDEVICE", "IFCPROTECTIVEDEVICE", "IFCAIRTOAIRHEATRECOVERY", "IFCCHILLER", "IFCHUMIDIFIER", "IFCELECTRICMOTOR",
    ];
    let mut set_down: Vec<(String, Kind, P3, [f64; 3])> = Vec::new();
    let mut thing_by_id: BTreeMap<u64, String> = BTreeMap::new();
    let every_class: Vec<&str> = FIXTURES.iter().chain(&FURNISHINGS).chain(&EQUIPMENT).copied().collect();
    for (id, e) in of(&every_class) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let kind = if FIXTURES.contains(&e.name.as_str()) {
            Kind::Fixture
        } else if FURNISHINGS.contains(&e.name.as_str()) {
            Kind::Furnishing
        } else {
            Kind::Equipment
        };
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let body = r.boxed(e);
        let origin = r.placed(r.arg(e, 5)).o;
        let (middle, bottom) = match &body {
            Some(b) => (b.middle, b.bottom),
            None => ([origin[0], origin[1]], origin[2]),
        };
        let (storey, floor) = storey_of(id, bottom * k);
        let called = r.name(e);
        let what = said_of.get(&id).and_then(|it| it.what.clone()).unwrap_or_else(|| if called.is_empty() { kind.stem().to_string() } else { called.clone() });
        let at = [tidy(middle[0] * k), tidy(middle[1] * k)];
        let offset = feet(bottom * k - floor);
        let outcome = (|| -> Result<String, String> {
            // A thing of this kind in this very place already on the level (the model brought in a second
            // time) would be counted twice: it is not set down again.
            let there = {
                let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Placed)>();
                q.iter(&job.world).any(|(n, other, p)| {
                    n.parent.as_deref() == Some(storey.as_str())
                        && *other == kind
                        && p.at_ft().is_some_and(|c| (c[0] - at[0]).abs() < 0.001 && (c[1] - at[1]).abs() < 0.001)
                        && p.offset.feet().is_some_and(|z| (z - (bottom * k - floor)).abs() < 0.001)
                        && (called.is_empty() || n.name == called || n.name.starts_with(&format!("{called} ")))
                })
            };
            if there {
                return Err(format!("{} in this very place is already on this level: a second would be counted twice", a_kind(kind)));
            }
            let size = body.as_ref().map(|b| [tidy(b.size[0] * k), tidy(b.size[1] * k), tidy(b.size[2] * k)]);
            let path = ed::place(job, &storey, kind, &what, at, &offset, size, system_of.get(&id).map(String::as_str), entered)?;
            if size.is_none() {
                let at_it = job.find(&path).ok_or("the new thing is not there")?;
                let mut placed = job.world.get::<Placed>(at_it).cloned().ok_or("the new thing has no place")?;
                placed.size = Val::unknown("ft", "the model's body for it is not a shape this reads, so how big it is is not known");
                job.put(&path, placed)?;
            }
            if let Some(b) = &body {
                if b.angle.abs() > 1.0e-6 {
                    let now = (b.angle * 1.0e6).round() / 1.0e6 + 0.0;
                    ed::turn(job, &path, now, entered)?;
                }
            }
            keep_name(job, &path, &called, id);
            Ok(path)
        })();
        match outcome {
            Ok(path) => {
                thing_by_id.insert(id, path.clone());
                set_down.push((path, kind, [at[0], at[1], bottom * k], body.as_ref().map(|b| [b.size[0] * k, b.size[1] * k, b.size[2] * k]).unwrap_or([0.0; 3])));
                made(match kind {
                    Kind::Fixture => "fixtures",
                    Kind::Furnishing => "furnishings",
                    _ => "pieces of equipment",
                });
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- Fittings, on the runs they join (point 2). A fitting in the file names no run: it has no
    // ports and no relation, only its place. It is put where runs' ends meet at that place, by the
    // change that puts a fitting there, which works out its kind from the runs.
    for (id, e) in of(&["IFCDUCTFITTING", "IFCPIPEFITTING", "IFCCABLECARRIERFITTING", "IFCCABLEFITTING", "IFCFLOWFITTING"]) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        // A fitting with no body at all (call 204, point 6: one the runs it joins give no size) is where its place is: the joint of the runs
        // there, and it comes in with its size Unknown, as the runs there are.
        let (middle, bottom, centre) = match r.boxed(e) {
            Some(b) => ([b.middle[0], b.middle[1]], b.bottom, b.bottom + b.size[2] / 2.0),
            None if r.pushed(e).is_none() && r.pieces(e) == 0 => {
                let o = r.placed(r.arg(e, 5)).o;
                ([o[0], o[1]], o[2], o[2])
            }
            None => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, "its body is not a box this reads, so where the runs it joins meet cannot be told");
                continue;
            }
        };
        let (storey, floor) = storey_of(id, bottom * k);
        let middle_z = centre * k - floor;
        let outcome = ed::fitting(job, &storey, [tidy(middle[0] * k), tidy(middle[1] * k)], Some(&feet(middle_z)), None, entered);
        match outcome {
            Ok(fitted) => {
                keep_name(job, &fitted.path, &r.name(e), id);
                // What the file calls it, against what the runs that meet there make it (call 200).
                let (said_kind, ok) = match r.arg(e, 8) {
                    V::Enum(w) => match w.as_str() {
                        "BEND" => (Some("bend"), matches!(fitted.kind, FittingKind::Bend)),
                        "JUNCTION" => (Some("junction"), matches!(fitted.kind, FittingKind::Tee | FittingKind::Cross)),
                        "TEE" => (Some("tee"), matches!(fitted.kind, FittingKind::Tee)),
                        "CROSS" => (Some("cross"), matches!(fitted.kind, FittingKind::Cross)),
                        "TRANSITION" | "REDUCER" => (Some("transition"), matches!(fitted.kind, FittingKind::Transition)),
                        "CONNECTOR" => (Some("coupling"), matches!(fitted.kind, FittingKind::Coupling)),
                        _ => (None, true),
                    },
                    _ => (None, true),
                };
                if let (Some(word), false) = (said_kind, ok) {
                    let made_kind = match fitted.kind {
                        FittingKind::Bend => "bend",
                        FittingKind::Tee => "tee",
                        FittingKind::Cross => "cross",
                        FittingKind::Transition => "transition",
                        FittingKind::Coupling => "coupling",
                    };
                    out.notes.push(format!(
                        "the fitting at ({}, {}, {}) is not what the file calls it: the file calls it a {word}, the runs there make a {made_kind}, and it is brought in as a {made_kind}",
                        tidy(middle[0] * k),
                        tidy(middle[1] * k),
                        tidy(middle_z)
                    ));
                }
                made("fittings");
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- What is connected to a fixture or a piece of equipment (call 200). A file with ports says it: each
    // port on a thing is its connection there, put where the file puts it by the change that makes a
    // connection; where the file ties it to the end of a run that stands at it, that run is joined to it, by
    // the change the Connect tool makes, and no other end is joined for being near. A file with no ports
    // says nothing, and reads as it always did: a run's end inside a thing's box is joined to it.
    if file_has_ports {
        let (mut connections, mut joined, mut no_sort) = (0usize, 0usize, 0usize);
        let mut apart: Vec<String> = Vec::new();
        let mut refused: Vec<String> = Vec::new();
        // The connections made that are not tied to the end of a run in the file, each by its thing and where (call 206 point 10).
        let mut loose: Vec<String> = Vec::new();
        for (id, thing) in &thing_by_id {
            let Some(ports) = ports_of.get(id) else { continue };
            let kind = job.kind(thing);
            if !matches!(kind, Some(Kind::Fixture | Kind::Equipment)) {
                continue;
            }
            for pid in ports {
                let Some(port) = r.file.entities.get(pid) else { continue };
                let place = r.placed(r.arg(port, 5)).o;
                let at = [place[0] * k, place[1] * k, place[2] * k];
                let sort = match r.arg(port, 8) {
                    V::Enum(w) => match w.as_str() {
                        "DUCT" => Some(plumbline_world::ConnectorKind::Duct),
                        "PIPE" => Some(plumbline_world::ConnectorKind::Pipe),
                        "CABLECARRIER" => Some(plumbline_world::ConnectorKind::Conduit),
                        "CABLE" => Some(plumbline_world::ConnectorKind::Power),
                        _ => None,
                    },
                    _ => None,
                };
                let domain = match r.arg(port, 9) {
                    V::Enum(w) => match w.as_str() {
                        "DOMESTICCOLDWATER" | "WATERSUPPLY" => Some(plumbline_world::Domain::Cold),
                        "DOMESTICHOTWATER" => Some(plumbline_world::Domain::Hot),
                        "SEWAGE" | "DRAINAGE" | "WASTEWATER" => Some(plumbline_world::Domain::Waste),
                        "VENT" => Some(plumbline_world::Domain::Vent),
                        "STORMWATER" | "RAINWATER" => Some(plumbline_world::Domain::Storm),
                        "GAS" => Some(plumbline_world::Domain::Gas),
                        _ => None,
                    },
                    _ => None,
                };
                let flow = match r.arg(port, 7) {
                    V::Enum(w) if w == "SOURCE" => plumbline_world::Flow::Out,
                    V::Enum(w) if w == "SINK" => plumbline_world::Flow::In,
                    _ => plumbline_world::Flow::Both,
                };
                // The end of a run that the file ties it to, where that run is one brought in.
                let mut tie: Option<(String, usize)> = None;
                for peer in tied_to.get(pid).into_iter().flatten() {
                    let Some(whole) = owner_of.get(peer) else { continue };
                    let Some((run, a, b)) = run_by_id.get(whole) else { continue };
                    let (da, db) = (dist3(*a, at), dist3(*b, at));
                    let (end, d) = if da <= db { (0, da) } else { (1, db) };
                    if d <= 0.05 {
                        tie = Some((run.clone(), end));
                    } else {
                        {
                        let mut called = |path: &str| job.find(path).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
                        apart.push(format!("a connection of {} is tied to the end of {}, which is {} ft from it", called(thing), called(run), tidy(d)));
                    }
                    }
                    break;
                }
                let sort_or_run = sort.or_else(|| tie.as_ref().and_then(|(run, _)| job.kind(run)).and_then(plumbline_world::ConnectorKind::for_run));
                let Some(sort) = sort_or_run else {
                    no_sort += 1;
                    continue;
                };
                let source = from(port, *pid);
                let entered = Entered { status: how.status, source: &source };
                let said = ed::PortSaid { name: r.name(port), kind: sort, domain, flow };
                match ed::connect_port(job, thing, tie.as_ref().map(|(run, end)| (run.as_str(), *end)), at, &said, entered) {
                    Ok(_) => {
                        connections += 1;
                        if tie.is_some() {
                            joined += 1;
                        } else {
                            let name = job.find(thing).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| thing.clone());
                            let port_name = r.name(port);
                            loose.push(format!("{name}{} at ({}, {}, {})", if port_name.is_empty() { String::new() } else { format!(", {port_name}") }, tidy(at[0]), tidy(at[1]), tidy(at[2])));
                        }
                    }
                    Err(why) if why.contains("already has a connection") => {}
                    Err(why) => refused.push(format!("{thing}: {why}")),
                }
            }
        }
        if connections > 0 {
            out.notes.push(format!(
                "{connections} connection{} on fixtures and equipment {} brought in from the file's connection points, each Assumed; {joined} {} tied to the end of a run and {} joined to it, and no other end was joined for being near",
                if connections == 1 { "" } else { "s" },
                if connections == 1 { "was" } else { "were" },
                if joined == 1 { "was" } else { "were" },
                if joined == 1 { "is" } else { "are" }
            ));
        }
        if !loose.is_empty() {
            let shown: Vec<String> = loose.iter().take(5).cloned().collect();
            out.notes.push(format!(
                "{} of the {connections} connection{} {} not tied to the end of a run in the file, and {} left loose: {}{}",
                loose.len(),
                if connections == 1 { "" } else { "s" },
                if loose.len() == 1 { "is" } else { "are" },
                if loose.len() == 1 { "is" } else { "are" },
                shown.join("; "),
                if loose.len() > 5 { format!("; and {} more", loose.len() - 5) } else { String::new() }
            ));
        }
        if !apart.is_empty() {
            out.notes.push(format!("{} connection point{} tied in the file to the end of a run that is not at it, and not joined: {}", apart.len(), if apart.len() == 1 { " is" } else { "s are" }, apart.join("; ")));
        }
        if no_sort > 0 {
            out.notes.push(format!("{no_sort} connection point{} on a fixture or a piece of equipment {} no sort (duct, pipe, cable) and no run to take it from: not brought in", if no_sort == 1 { "" } else { "s" }, if no_sort == 1 { "says" } else { "say" }));
        }
        if !refused.is_empty() {
            out.notes.push(format!("{} connection{} this program would not take: {}", refused.len(), if refused.len() == 1 { "" } else { "s" }, refused.join("; ")));
        }
    } else {
        let runs = run_ends.clone();
        let mut not_joined: Vec<String> = Vec::new();
        let mut joined = 0usize;
        let joining = format!("{}: the run ends at it, in the model brought in", how.source);
        for (thing, kind, at, size) in set_down.clone() {
            if !matches!(kind, Kind::Fixture | Kind::Equipment) || size == [0.0; 3] {
                continue;
            }
            let reach = size[0].hypot(size[1]) / 2.0 + 0.1;
            let near = |p: P3| (p[0] - at[0]).abs() <= reach && (p[1] - at[1]).abs() <= reach && p[2] >= at[2] - 0.1 && p[2] <= at[2] + size[2] + 0.1;
            for (run, a, b) in &runs {
                if !(near(*a) || near(*b)) {
                    continue;
                }
                match ed::connect(job, &thing, run, Entered { status: how.status, source: &joining }) {
                    Ok(_) => joined += 1,
                    Err(why) if why.contains("already has a connection") || why.starts_with("neither end of") => {}
                    Err(why) => not_joined.push(format!("{run} at {thing}: {why}")),
                }
            }
        }
        if joined > 0 {
            out.notes.push(format!("{joined} run{} ending at a fixture or a piece of equipment {} joined to it, where it stood", if joined == 1 { "" } else { "s" }, if joined == 1 { "was" } else { "were" }));
        }
        if !not_joined.is_empty() {
            out.notes.push(format!("{} run{} end{} at a fixture or a piece of equipment and could not be joined to it: {}", not_joined.len(), if not_joined.len() == 1 { "" } else { "s" }, if not_joined.len() == 1 { "s" } else { "" }, not_joined.join("; ")));
        }
    }
    if runs_unsized > 0 {
        out.notes.push(format!("{runs_unsized} run{} {} no size: {} drawn as a line alone and comes in with its size Unknown", if runs_unsized == 1 { "" } else { "s" }, if runs_unsized == 1 { "has" } else { "have" }, if runs_unsized == 1 { "it is" } else { "each is" }));
    }
    // Run ends that stand at a thing but are joined to nothing, said (call 200): nothing is joined for being
    // near, so where an end stands within a foot of a fixture or a piece of equipment and nothing joins it,
    // that is said, with how far.
    {
        job.derive();
        let net = building::network::network(job, None, None, building::network::TOLERANCE_FT)?;
        let mut near_but_open: Vec<String> = Vec::new();
        for node in net.open.iter().filter_map(|i| net.nodes.get(*i)) {
            let Some(building::network::Joint::RunEnd { run, .. }) = node.joints.first() else { continue };
            for (thing, kind, at, size) in &set_down {
                if !matches!(kind, Kind::Fixture | Kind::Equipment) || *size == [0.0; 3] {
                    continue;
                }
                let angle = job.find(thing).and_then(|e| job.world.get::<Placed>(e)).and_then(|p| p.angle_deg()).unwrap_or(0.0);
                let (sin, cos) = angle.to_radians().sin_cos();
                let (dx, dy, dz) = (node.at[0] - at[0], node.at[1] - at[1], node.at[2] - at[2]);
                let local = [dx * cos + dy * sin, -dx * sin + dy * cos, dz];
                let out_by = [(local[0].abs() - size[0] / 2.0).max(0.0), (local[1].abs() - size[1] / 2.0).max(0.0), (-local[2]).max(local[2] - size[2]).max(0.0)];
                let d = (out_by[0] * out_by[0] + out_by[1] * out_by[1] + out_by[2] * out_by[2]).sqrt();
                if d <= 1.0 {
                    let mut name = |path: &str| job.find(path).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
                    near_but_open.push(format!("{} {:.2} ft from {}", name(run), d, name(thing)));
                }
            }
        }
        if !near_but_open.is_empty() {
            near_but_open.sort();
            out.notes.push(format!(
                "{} run end{} stand{} within a foot of a fixture or a piece of equipment that the file gives no connection there, and read{} open: {}",
                near_but_open.len(),
                if near_but_open.len() == 1 { "" } else { "s" },
                if near_but_open.len() == 1 { "s" } else { "" },
                if near_but_open.len() == 1 { "s" } else { "" },
                near_but_open.join("; ")
            ));
        }
    }
    // ---- Everything else that is a thing with a place and a shape: listed.
    const READ: [&str; 22] = [
        "IFCBUILDINGSTOREY", "IFCWALL", "IFCWALLSTANDARDCASE", "IFCWALLELEMENTEDCASE", "IFCDOOR", "IFCDOORSTANDARDCASE", "IFCWINDOW", "IFCWINDOWSTANDARDCASE", "IFCOPENINGELEMENT", "IFCOPENINGSTANDARDCASE", "IFCSLAB",
        "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE", "IFCSPACE", "IFCDUCTSEGMENT", "IFCPIPESEGMENT", "IFCCABLECARRIERSEGMENT", "IFCCABLESEGMENT", "IFCFLOWSEGMENT", "IFCSITE", "IFCBUILDING", "IFCPROJECT",
    ];
    // A thing of the building with no place: the model names it (it has the model's
    // own 22-letter mark, and is not a tie between things, a type or a list of values)
    // and says nowhere it is.
    const NOT_THINGS: [&str; 14] = ["IFCPROPERTYSET", "IFCELEMENTQUANTITY", "IFCGROUP", "IFCSYSTEM", "IFCDISTRIBUTIONSYSTEM", "IFCDISTRIBUTIONCIRCUIT", "IFCZONE", "IFCACTOR", "IFCTASK", "IFCWORKPLAN", "IFCWORKSCHEDULE", "IFCCOSTITEM", "IFCCOSTSCHEDULE", "IFCPROJECTLIBRARY"];
    for id in &ids {
        let e = &r.file.entities[id];
        let is_placed = r.get(r.arg(e, 5)).is_some_and(|(_, p)| p.name == "IFCLOCALPLACEMENT" || p.name == "IFCGRIDPLACEMENT");
        if READ.contains(&e.name.as_str()) || done.contains(id) {
            continue;
        }
        if is_placed {
            if let Some(reach) = reach_of(e) {
                stood_on.push((said(&e.name), reach.0, reach.1));
            }
            leave(&e.name, if e.name == "IFCROOF" { NO_ROOF_OUTLINE } else { "nothing here to bring it in as" });
        } else if Reader::text(r.arg(e, 0)).is_some_and(|mark| mark.len() == 22) && e.args.len() >= 7 && !e.name.starts_with("IFCREL") && !e.name.ends_with("TYPE") && !e.name.ends_with("STYLE") && !e.name.ends_with("TEMPLATE") && !NOT_THINGS.contains(&e.name.as_str()) {
            leave(&e.name, "the model gives it no place, and there is nothing here to bring it in as");
        }
    }
    if guessed > 0 {
        out.notes.push(format!("{guessed} thing{} the model puts on no level: each was put on the highest level whose floor is not above it", if guessed == 1 { " is one" } else { "s are ones" }));
    }
    if systems_not_known > 0 {
        out.notes.push(format!("{systems_not_known} system{} in the model {} of a sort this program has no name for: what is on {} was brought in on no system", if systems_not_known == 1 { "" } else { "s" }, if systems_not_known == 1 { "is" } else { "are" }, if systems_not_known == 1 { "it" } else { "them" }));
    }
    // What follows from what was left out (call 197, point 6): the runs' ends that stood on a thing not
    // brought in read open here, where they read joined in the model.
    if !stood_on.is_empty() {
        job.derive();
        let net = building::network::network(job, None, None, building::network::TOLERANCE_FT)?;
        let mut by_what: BTreeMap<String, usize> = BTreeMap::new();
        for at in net.open.iter().filter_map(|i| net.nodes.get(*i)).map(|n| n.at) {
            let reach = 0.1;
            if let Some((what, _, _)) = stood_on.iter().find(|(_, lo, hi)| (0..3).all(|i| at[i] >= lo[i] - reach && at[i] <= hi[i] + reach)) {
                *by_what.entry(what.clone()).or_insert(0) += 1;
            }
        }
        let total: usize = by_what.values().sum();
        if total > 0 {
            out.notes.push(format!(
                "{total} run end{} read open here: {} stood on {} that {} not brought in ({})",
                if total == 1 { "" } else { "s" },
                if total == 1 { "it" } else { "each" },
                if by_what.len() == 1 && total == 1 { "a thing" } else { "things" },
                if by_what.len() == 1 && total == 1 { "was" } else { "were" },
                by_what.iter().map(|(what, n)| format!("{n} on {what}")).collect::<Vec<_>>().join(", ")
            ));
        }
    }
    out.notes.push(format!("the model is in {}: every length was turned into feet and kept to a ten-thousandth of a foot", out.unit));
    out.left_out = left.into_iter().map(|((class, why), n)| (class, n, why)).collect();
    if out.made.iter().all(|(what, _)| what == "levels") {
        if let Some(said) = already_in_the_job(&out.left_out) {
            return Err(said);
        }
        return Err(format!("nothing in the model but its levels could be brought in: {}", out.left_out.iter().map(|(c, n, why)| format!("{n} {c} ({why})")).collect::<Vec<_>>().join("; ")));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::point_inside;

    /// Whether a point is inside an outline: a line east from it crosses the outline an
    /// odd number of times.
    fn inside(points: &[[f64; 2]], p: [f64; 2]) -> bool {
        (0..points.len()).filter(|i| {
            let (a, b) = (points[*i], points[(*i + 1) % points.len()]);
            (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0])
        }).count() % 2 == 1
    }

    #[test]
    fn j15_16_a_rooms_point_is_inside_the_room() {
        // A square: the middle of its corners.
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        assert_eq!(point_inside(&square), [2.0, 2.0]);
        // An L, 10 by 10 with its upper right 8 by 8 cut out, drawn with a corner for
        // every foot of its long arms, as a model traced from a drawing has them: the
        // middle of its corners falls in the part cut out.
        let mut l: Vec<[f64; 2]> = (0..=10).map(|x| [x as f64, 0.0]).collect();
        l.extend([[10.0, 2.0], [2.0, 2.0]]);
        l.extend((2..=10).map(|y| [2.0, y as f64]));
        l.extend((0..=10).rev().map(|y| [0.0, y as f64]).skip(0).take(10));
        let n = l.len() as f64;
        let mid = [l.iter().map(|p| p[0]).sum::<f64>() / n, l.iter().map(|p| p[1]).sum::<f64>() / n];
        assert!(!inside(&l, mid), "the test's L has to have its middle outside it: {mid:?}");
        let p = point_inside(&l);
        assert!(inside(&l, p), "{p:?} is not inside the L");
        // A U: either leg will do, and the point is in one of them.
        let u = [[0.0, 0.0], [9.0, 0.0], [9.0, 9.0], [6.0, 9.0], [6.0, 2.0], [3.0, 2.0], [3.0, 9.0], [0.0, 9.0]];
        assert!(inside(&u, point_inside(&u)));
    }
}
