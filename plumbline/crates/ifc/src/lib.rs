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
/// The clashes written as an issue list another program opens (BCF 2.1; call 228), naming things by the ids `write` gives them.
pub mod bcf;
pub mod write;
pub mod gltf;
mod maker;
pub use maker::{maker_model, MadeItem, MakerModel};

mod bringing;
mod entry;
use bringing::*;
pub use entry::*;

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
    /// What was not asked for, by kind, when only a model's runs and what they serve were chosen (call 210 point 1): the walls,
    /// floors, roofs, footings, doors, windows and rooms of the file, counted and not brought in. Empty when the whole was asked.
    pub not_asked: Vec<(String, usize)>,
    /// What the things brought in do to the clash list of the job (call 210 point 4), worked by the model's own clash list before
    /// and after. None where the job was not asked (the whole model brought in by `import`).
    pub clashes: Option<ClashChange>,
}

/// What is chosen when a model is brought in (call 210 point 1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Bring {
    /// Everything in the file, as always, and still refused where it lies over what the job has.
    #[default]
    Whole,
    /// Only its runs and what they serve: ducts, pipes, conduit, cable, their fittings, every fixture and piece of equipment (a run reaches
    /// it or not: an air handler is wanted before its ducts are drawn), their systems and what is connected to what.
    RunsOnly,
}

/// How many clashes of each kind a job has, counted by the model's own clash list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClashCounts {
    pub same_space: usize,
    pub set_against: usize,
    pub holes: usize,
    pub laid_in: usize,
}

/// The clashes before and after a model was brought in, and the ones the things brought in make.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClashChange {
    pub before: ClashCounts,
    pub after: ClashCounts,
    /// The clashes of the job now that have a thing brought in as one of the two, by kind.
    pub made: ClashCounts,
    /// Each of those, said: what, with what, how, where.
    pub listed: Vec<String>,
}

impl Brought {
    /// What was not asked for, said: "not asked for: 1 floor and 4 walls". None where the whole was asked.
    pub fn not_asked_said(&self) -> Option<String> {
        (!self.not_asked.is_empty()).then(|| format!("not asked for: {}", not_asked_list(&self.not_asked)))
    }

    /// What the things brought in do to the job's clashes (call 210 point 4), a line each, in the words `clashes` uses: how many the
    /// things brought in make by kind, what the job had and has, and each of the new ones. Empty where the job was not asked.
    pub fn clashes_said(&self) -> Vec<String> {
        let Some(c) = &self.clashes else { return Vec::new() };
        let many = |n: usize, one: &str, more: &str| format!("{n} {}", if n == 1 { one } else { more });
        let mut out = Vec::new();
        let made = c.made;
        if made == ClashCounts::default() {
            out.push("the things brought in clash with nothing the job has".to_string());
        } else {
            let mut each = vec![many(made.holes, "hole to cut", "holes to cut"), format!("{} in the same space", made.same_space), format!("{} set against a wall", made.set_against)];
            if made.laid_in > 0 {
                each.push(format!("{} laid in a floor or a wall", made.laid_in));
            }
            out.push(format!("the things brought in make {} with what the job has", list_said(&each)));
        }
        let kinds = [
            ("holes to cut", c.before.holes, c.after.holes, made.holes),
            ("in the same space", c.before.same_space, c.after.same_space, made.same_space),
            ("set against a wall", c.before.set_against, c.after.set_against, made.set_against),
            ("laid in a floor or a wall", c.before.laid_in, c.after.laid_in, made.laid_in),
        ];
        for (what, before, after, mine) in kinds {
            if before != after {
                out.push(format!("the job had {before} {what} and now has {after}"));
            }
            if after.saturating_sub(before) != mine || before > after {
                out.push(format!("the job's own count of {what} moved by {} and the things brought in account for {mine}", after as i64 - before as i64));
            }
        }
        out.extend(c.listed.iter().map(|l| format!("  {l}")));
        out.push("they are in the clash list, with the job's own".to_string());
        out
    }

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
    // The file's own words for what this program calls a post and a room, and one word for a kind of thing in every reply: a fixture, a
    // piece of equipment (call 213 point 10).
    let equipment_said: Vec<String> = EQUIPMENT.iter().map(|c| said(c)).collect();
    let plain = |class: &str| match class {
        c if equipment_said.iter().any(|e| e == c) => "piece of equipment".to_string(),
        c if c.contains("as the model calls it") => "other thing".to_string(),
        "column" => "post".to_string(),
        "space" => "room".to_string(),
        "fixture or outlet" | "plumbing fixture" => "fixture".to_string(),
        c => c.to_string(),
    };
    let counted = |n: usize, kind: &str| -> String {
        if n == 1 {
            return format!("{n} {kind}");
        }
        let many = match kind {
            "piece of equipment" => "pieces of equipment".to_string(),
            k if k.ends_with('s') => k.to_string(),
            k => format!("{k}s"),
        };
        format!("{n} {many}")
    };
    let rest: Vec<String> = left_out.iter().filter(|(_, _, why)| !already(why) && !with_walls(why)).map(|(c, n, why)| format!("{} ({})", counted(*n, &plain(c)), why)).collect();
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
        said.push_str(&format!(" ({})", kinds.iter().map(|(k, n)| counted(*n, k)).collect::<Vec<_>>().join(", ")));
    }
    if openings > 0 {
        said.push_str(&format!("; {openings} door{} and window{} go with walls that were not brought in again", if openings == 1 { "" } else { "s" }, if openings == 1 { "" } else { "s" }));
    }
    if !rest.is_empty() {
        said.push_str(&format!("; and {}", rest.join("; ")));
    }
    Some(said)
}

/// What a model's fixtures, furnishings and equipment are called in the file (call 197 point 1; moved up to be seen by the choice of what
/// to bring in, call 210).
const FIXTURES: [&str; 7] = ["IFCSANITARYTERMINAL", "IFCFLOWTERMINAL", "IFCOUTLET", "IFCAIRTERMINAL", "IFCLIGHTFIXTURE", "IFCSTACKTERMINAL", "IFCWASTETERMINAL"];
const FURNISHINGS: [&str; 3] = ["IFCFURNITURE", "IFCFURNISHINGELEMENT", "IFCSYSTEMFURNITUREELEMENT"];
const EQUIPMENT: [&str; 28] = [
    "IFCDISTRIBUTIONFLOWELEMENT", "IFCDISTRIBUTIONELEMENT", "IFCDISTRIBUTIONCONTROLELEMENT", "IFCDISTRIBUTIONCHAMBERELEMENT", "IFCENERGYCONVERSIONDEVICE", "IFCFLOWMOVINGDEVICE", "IFCFLOWSTORAGEDEVICE", "IFCFLOWTREATMENTDEVICE",
    "IFCELECTRICDISTRIBUTIONBOARD", "IFCUNITARYEQUIPMENT", "IFCBOILER", "IFCPUMP", "IFCFAN", "IFCTANK", "IFCTRANSFORMER", "IFCCOMPRESSOR", "IFCCOIL", "IFCCONDENSER", "IFCEVAPORATOR", "IFCHEATEXCHANGER", "IFCFILTER",
    "IFCELECTRICGENERATOR", "IFCSWITCHINGDEVICE", "IFCPROTECTIVEDEVICE", "IFCAIRTOAIRHEATRECOVERY", "IFCCHILLER", "IFCHUMIDIFIER", "IFCELECTRICMOTOR",
];

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
