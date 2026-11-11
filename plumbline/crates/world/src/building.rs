//! The rest of the building: storeys, walls with their doors and windows, floors, and what
//! runs through them. A duct, a pipe, a conduit and a cable are each a centre line in space
//! with a size. Equipment and fixtures are things set at a place. Any of them can belong to
//! a system (supply air, domestic hot water, power), and a system is an element of the job
//! like any other, so it can be read, counted and totalled.
//!
//! The names are the trade's and the open standard's, not ours: storey, wall, opening,
//! floor (IFC's slab), duct, pipe, conduit, cable, equipment, fixture, system, and the
//! system classifications a mechanical model already uses (Supply Air, Sanitary, Power).
//!
//! The rules are the same as for the roof. Every entered value is Known, Assumed or Unknown
//! and says where it came from. Every figure is worked out here, never in a window, takes
//! the worst status of what it was worked out from, and is refused with the reason when
//! its inputs do not fit together: a window wider than its wall gives no net area.
//!
//! Heights. A storey's floor is at its `elevation` above the job's datum. A wall, a floor
//! and an opening stand on their storey. A route's points and a placed thing's position
//! are given in space outright: east, north, and height above the datum. A riser passes
//! through storeys, and a point that had to be read against one storey or another could be
//! read wrongly.

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::check::MAX_COORDINATE_FT;
use crate::components::*;
use crate::geom;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Derived, Status, Val};
use crate::view::{self, Row, Sum};

pub mod model;
pub mod walls;

pub use model::*;

/// A point in space: east, north, height above the datum.
pub type P3 = [f64; 3];

/// A turn in a centre line sharper than this many degrees is a bend: a fitting.
pub const BEND_DEGREES: f64 = 1.0;
/// A cross-section beyond this (in feet) is a typing error.
pub const MAX_SECTION_FT: f64 = 20.0;

pub const K_LEVEL: &str = "pl::level";
pub const K_WALL: &str = "pl::wall";
pub const K_WALL_TYPE: &str = "pl::wall_type";
pub const K_OPENING: &str = "pl::opening";
pub const K_OPENING_TYPE: &str = "pl::opening_type";
pub const K_FLOOR: &str = "pl::floor";
pub const K_FLOOR_TYPE: &str = "pl::floor_type";
pub const K_ROUTE: &str = "pl::route";
pub const K_SIZE: &str = "pl::size";
pub const K_PLACED: &str = "pl::placed";
pub const K_SYSTEM: &str = "pl::system";
pub const K_IN_SYSTEM: &str = "pl::in_system";

/// `pl::route`: the centre line of a duct, pipe, conduit or cable, point to point in space.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub points: Val<Vec<P3>>,
}

impl Route {
    pub fn points_ft(&self) -> Option<Vec<P3>> {
        let d = units::coord_divisor(&self.points.unit).ok()?;
        self.points.value.as_ref().map(|pts| pts.iter().map(|p| [p[0] / d, p[1] / d, p[2] / d]).collect())
    }
}

/// `pl::size`: the cross-section of a route. Round by its diameter, or rectangular by its
/// width and depth. One or the other.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diameter: Option<Val<Len>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<Val<Len>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<Val<Len>>,
}

impl Size {
    pub fn round(diameter: Val<Len>) -> Size {
        Size { diameter: Some(diameter), width: None, depth: None }
    }
    pub fn rectangular(width: Val<Len>, depth: Val<Len>) -> Size {
        Size { diameter: None, width: Some(width), depth: Some(depth) }
    }
    /// The worst status among the dimensions it has.
    pub fn status(&self) -> Status {
        [&self.diameter, &self.width, &self.depth].into_iter().flatten().map(|v| v.status).max().unwrap_or(Status::Unknown)
    }
    /// The diameter in feet, for a round section with one.
    pub fn round_ft(&self) -> Option<f64> {
        self.diameter.as_ref().and_then(|d| d.feet())
    }
    /// Width and depth in feet, for a rectangular section with both.
    pub fn rect_ft(&self) -> Option<(f64, f64)> {
        self.width.as_ref().and_then(|w| w.feet()).zip(self.depth.as_ref().and_then(|d| d.feet()))
    }
    /// How far it is round the section, in feet.
    pub fn perimeter_ft(&self) -> Option<f64> {
        self.round_ft().map(|d| std::f64::consts::PI * d).or_else(|| self.rect_ft().map(|(w, d)| 2.0 * (w + d)))
    }
    /// The size as it is said: `3/4"`, `16" × 8"`, or that it is not known.
    pub fn said(&self) -> String {
        let one = |v: &Val<Len>| v.value.as_ref().map(|len| units::show_length(len, &v.unit));
        match (&self.diameter, &self.width, &self.depth) {
            (Some(d), _, _) => one(d).map(|s| format!("{s} round")).unwrap_or_else(|| "size unknown".into()),
            (None, Some(w), Some(d)) => match (one(w), one(d)) {
                (Some(w), Some(d)) => format!("{w} × {d}"),
                _ => "size unknown".into(),
            },
            _ => "size unknown".into(),
        }
    }
}

/// `pl::placed`: on Equipment or a Fixture. What it is, where the middle of its base is,
/// and how big it is: east-west, north-south, and tall.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placed {
    pub what: Val<String>,
    pub at: Val<P3>,
    pub size: Val<P3>,
}

impl Placed {
    pub fn at_ft(&self) -> Option<P3> {
        let d = units::coord_divisor(&self.at.unit).ok()?;
        self.at.value.map(|p| p.map(|x| x / d))
    }
    pub fn size_ft(&self) -> Option<P3> {
        let d = units::coord_divisor(&self.size.unit).ok()?;
        self.size.value.map(|p| p.map(|x| x / d))
    }
}

/// Whose work an element is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Discipline {
    Architecture,
    Mechanical,
    Electrical,
    Plumbing,
}

/// `pl::system`: on a System, whose it is and what it carries. The classification is one
/// of the names a mechanical model already uses: Supply Air, Return Air, Exhaust Air,
/// Refrigerant, Domestic Cold Water, Domestic Hot Water, Sanitary, Vent, Power, Lighting.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemInfo {
    pub discipline: Discipline,
    pub classification: String,
}

pub const CLASSIFICATIONS: [(&str, Discipline); 10] = [
    ("Supply Air", Discipline::Mechanical),
    ("Return Air", Discipline::Mechanical),
    ("Exhaust Air", Discipline::Mechanical),
    ("Refrigerant", Discipline::Mechanical),
    ("Domestic Cold Water", Discipline::Plumbing),
    ("Domestic Hot Water", Discipline::Plumbing),
    ("Sanitary", Discipline::Plumbing),
    ("Vent", Discipline::Plumbing),
    ("Power", Discipline::Electrical),
    ("Lighting", Discipline::Electrical),
];

/// `pl::in_system`: the system an element belongs to, by the system's path.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InSystem(pub Val<String>);

impl Kind {
    /// A centre line with a size: a duct, a pipe, a conduit or a cable.
    pub fn is_route(self) -> bool {
        matches!(self, Kind::Duct | Kind::Pipe | Kind::Conduit | Kind::Cable)
    }
    pub fn is_opening(self) -> bool {
        matches!(self, Kind::Door | Kind::Window)
    }
    pub fn is_placed(self) -> bool {
        matches!(self, Kind::Equipment | Kind::Fixture)
    }
    /// Whose work an element of this kind is when it belongs to no system.
    pub fn discipline(self) -> Discipline {
        match self {
            Kind::Duct => Discipline::Mechanical,
            Kind::Pipe => Discipline::Plumbing,
            Kind::Conduit | Kind::Cable => Discipline::Electrical,
            _ => Discipline::Architecture,
        }
    }
}

// ---------------------------------------------------------------- the entry checks

pub(crate) fn positive(what: &str, v: &Val<Len>, zero_ok: bool) -> Result<(), String> {
    v.check_length(what)?;
    if let Some(x) = v.feet() {
        if x < 0.0 || (!zero_ok && x == 0.0) {
            return Err(format!("{what}: must be {} zero", if zero_ok { "at least" } else { "more than" }));
        }
        if x > MAX_COORDINATE_FT {
            return Err(format!("{what}: is not a usable number (limit {MAX_COORDINATE_FT:e} ft)"));
        }
    }
    Ok(())
}

pub(crate) fn coords<T>(what: &str, v: &Val<T>) -> Result<f64, String> {
    v.check_basics(what)?;
    units::coord_divisor(&v.unit).map_err(|why| format!("{what}: {why}"))
}

pub(crate) fn usable(what: &str, xs: &[f64], divisor: f64) -> Result<(), String> {
    if xs.iter().any(|x| !x.is_finite() || (x / divisor).abs() > MAX_COORDINATE_FT) {
        return Err(format!("{what}: a coordinate is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    Ok(())
}

pub fn check_route(path: &str, kind: Kind, r: &Route) -> Result<(), String> {
    if !kind.is_route() {
        return Err(format!("{path}: a {kind:?} cannot carry a centre line; only a duct, a pipe, a conduit or a cable can"));
    }
    let d = coords(&format!("{path} centre line"), &r.points)?;
    if let Some(pts) = &r.points.value {
        if pts.len() < 2 {
            return Err(format!("{path} centre line: has {} point(s); a run goes from one point to another", pts.len()));
        }
        for (i, p) in pts.iter().enumerate() {
            usable(&format!("{path} centre line"), p, d)?;
            if i > 0 && *p == pts[i - 1] {
                return Err(format!("{path} centre line: points {i} and {} are the same point", i + 1));
            }
        }
    }
    Ok(())
}

pub fn check_size(path: &str, kind: Kind, s: &Size) -> Result<(), String> {
    if !kind.is_route() {
        return Err(format!("{path}: a {kind:?} cannot carry a cross-section; only a duct, a pipe, a conduit or a cable can"));
    }
    match (&s.diameter, &s.width, &s.depth) {
        (Some(_), None, None) | (None, Some(_), Some(_)) => {}
        _ => return Err(format!("{path} size: is round, by a diameter, or rectangular, by a width and a depth; it has to be one or the other")),
    }
    for (name, v) in [("diameter", &s.diameter), ("width", &s.width), ("depth", &s.depth)] {
        let Some(v) = v else { continue };
        positive(&format!("{path} {name}"), v, false)?;
        if v.feet().is_some_and(|x| x > MAX_SECTION_FT) {
            return Err(format!("{path} {name}: is over {MAX_SECTION_FT} ft across; that is not a duct or a pipe"));
        }
    }
    Ok(())
}

pub fn check_placed(path: &str, kind: Kind, p: &Placed) -> Result<(), String> {
    if !kind.is_placed() {
        return Err(format!("{path}: a {kind:?} cannot carry a placement; only equipment or a fixture can"));
    }
    p.what.check(&format!("{path} what it is"), "")?;
    if p.what.value.as_ref().is_some_and(|s| s.trim().is_empty()) {
        return Err(format!("{path} what it is: is blank; mark it Unknown if nobody has it"));
    }
    let d = coords(&format!("{path} position"), &p.at)?;
    if let Some(at) = &p.at.value {
        usable(&format!("{path} position"), at, d)?;
    }
    let d = coords(&format!("{path} size"), &p.size)?;
    if let Some(size) = &p.size.value {
        usable(&format!("{path} size"), size, d)?;
        if size.iter().any(|x| *x <= 0.0) {
            return Err(format!("{path} size: each of its three dimensions must be more than zero"));
        }
    }
    Ok(())
}

pub fn check_system(path: &str, kind: Kind, s: &SystemInfo) -> Result<(), String> {
    if kind != Kind::System {
        return Err(format!("{path}: a {kind:?} cannot carry a system's description; only a system can"));
    }
    match CLASSIFICATIONS.iter().find(|(name, _)| *name == s.classification) {
        None => Err(format!("{path} system: {:?} is not a classification this program knows; it takes {}", s.classification, CLASSIFICATIONS.map(|c| c.0).join(", "))),
        Some((_, d)) if *d != s.discipline => Err(format!("{path} system: {} is {d:?} work, but the system says {:?}", s.classification, s.discipline)),
        Some(_) => Ok(()),
    }
}

pub fn check_in_system(path: &str, kind: Kind, s: &InSystem) -> Result<(), String> {
    if !(kind.is_route() || kind.is_placed()) {
        return Err(format!("{path}: a {kind:?} cannot belong to a system; a duct, a pipe, a conduit, a cable, equipment or a fixture can"));
    }
    s.0.check(&format!("{path} system"), "")?;
    if s.0.value.as_ref().is_some_and(|p| p.trim().is_empty()) {
        return Err(format!("{path} system: is blank; mark it Unknown if nobody has it"));
    }
    Ok(())
}

/// What `check_whole`, `measures` and `placements` need to know of the job besides the
/// element in hand: who is under whom, each storey's level, and the types.
pub(crate) struct Tree {
    pub(crate) parents: HashMap<String, Option<String>>,
    pub(crate) kinds: HashMap<String, Kind>,
    pub(crate) names: HashMap<String, String>,
    pub(crate) levels: HashMap<String, Level>,
    pub(crate) wall_types: HashMap<String, WallType>,
    pub(crate) floor_types: HashMap<String, FloorType>,
    pub(crate) opening_types: HashMap<String, OpeningType>,
}

impl Tree {
    pub(crate) fn of(world: &mut World) -> Tree {
        let mut t = Tree { parents: HashMap::new(), kinds: HashMap::new(), names: HashMap::new(), levels: HashMap::new(), wall_types: HashMap::new(), floor_types: HashMap::new(), opening_types: HashMap::new() };
        let mut q = world.query::<(&Node, &Kind, Option<&Level>, Option<&WallType>, Option<&FloorType>, Option<&OpeningType>)>();
        for (node, kind, level, wall, floor, opening) in q.iter(world) {
            let p = node.path.clone();
            t.parents.insert(p.clone(), node.parent.clone());
            t.kinds.insert(p.clone(), *kind);
            t.names.insert(p.clone(), node.name.clone());
            if let (Kind::Storey, Some(l)) = (kind, level) {
                t.levels.insert(p.clone(), l.clone());
            }
            if *kind == Kind::Type {
                if let Some(c) = wall {
                    t.wall_types.insert(p.clone(), c.clone());
                }
                if let Some(c) = floor {
                    t.floor_types.insert(p.clone(), c.clone());
                }
                if let Some(c) = opening {
                    t.opening_types.insert(p.clone(), c.clone());
                }
            }
        }
        t
    }

    /// The storey an element stands on: the nearest one above it in the tree.
    pub(crate) fn storey_of(&self, path: &str) -> Option<&str> {
        let mut at = self.parents.get(path)?.as_deref();
        let mut steps = 0;
        while let Some(p) = at {
            if self.kinds.get(p) == Some(&Kind::Storey) {
                return self.kinds.get_key_value(p).map(|(k, _)| k.as_str());
            }
            at = self.parents.get(p)?.as_deref();
            steps += 1;
            if steps > self.parents.len() {
                return None;
            }
        }
        None
    }

    /// What an element is called, for a message: its name, or its path when it has none.
    pub(crate) fn called(&self, path: &str) -> String {
        self.names.get(path).filter(|n| !n.is_empty()).cloned().unwrap_or_else(|| path.to_string())
    }
}

/// The type a reference names, from a table of them, or why there is none.
pub(crate) fn typed<'a, T>(table: &'a HashMap<String, T>, r: &'a Val<String>, what: &str) -> Result<(&'a str, &'a T), String> {
    match &r.value {
        None => Err(format!("its type is not known ({})", r.source)),
        Some(p) => table.get(p).map(|t| (p.as_str(), t)).ok_or_else(|| format!("its type, {p}, is not {what} type in this job")),
    }
}

/// What can only be checked once every element is in: an opening is under a wall, a wall
/// and a floor are under a storey, a type is under the job and says what it is a type of,
/// a type named is a type of the right sort, a top level named is a storey, and a system
/// named is a system in the job.
pub fn check_whole(job: &mut Job) -> Result<(), String> {
    let tree = Tree::of(&mut job.world);
    let kinds = &tree.kinds;
    let mut q = job.world.query::<(&Node, &Kind, Option<&InSystem>, Option<&Wall>, Option<&Opening>, Option<&Floor>)>();
    for (node, kind, in_system, wall, opening, floor) in q.iter(&job.world) {
        let me = &node.path;
        let parent = node.parent.as_ref().and_then(|p| kinds.get(p)).copied();
        if kind.is_opening() && parent != Some(Kind::Wall) {
            return Err(format!("{me}: a {kind:?} is an opening in a wall and has to be under one"));
        }
        if matches!(kind, Kind::Wall | Kind::Floor) && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a {kind:?} stands on a storey and has to be under one"));
        }
        if *kind == Kind::Storey && parent != Some(Kind::Job) {
            return Err(format!("{me}: a storey is a floor of the building and has to be directly under the job, not under {}", node.parent.as_deref().unwrap_or("nothing")));
        }
        if *kind == Kind::Type {
            if parent != Some(Kind::Job) {
                return Err(format!("{me}: a type belongs to the whole job and has to be directly under it, not under {}", node.parent.as_deref().unwrap_or("nothing")));
            }
            let sorts = [tree.wall_types.contains_key(me), tree.floor_types.contains_key(me), tree.opening_types.contains_key(me)].into_iter().filter(|x| *x).count();
            if sorts != 1 {
                return Err(format!("{me}: a type is one sort of type (of a wall, a floor, a door or a window); this one carries {sorts}"));
            }
        }
        if let Some(named) = in_system.and_then(|s| s.0.value.as_ref()) {
            if kinds.get(named) != Some(&Kind::System) {
                return Err(format!("{me}: says it belongs to {named:?}, which is not a system in this job"));
            }
        }
        if let Some(w) = wall {
            if let Some(p) = &w.type_name.value {
                if !tree.wall_types.contains_key(p) {
                    return Err(format!("{me}: its type is {p:?}, which is not a wall type in this job"));
                }
            }
            if let Some(p) = w.top_level.as_ref().and_then(|t| t.value.as_ref()) {
                if kinds.get(p) != Some(&Kind::Storey) {
                    return Err(format!("{me}: its top level is {p:?}, which is not a storey in this job"));
                }
            }
        }
        if let Some(p) = opening.and_then(|o| o.type_name.value.as_ref()) {
            match tree.opening_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a door or window type in this job")),
                Some(t) if t.category != *kind => return Err(format!("{me}: is a {kind:?}, but its type, {p}, is a {:?}'s", t.category)),
                Some(_) => {}
            }
        }
        if let Some(p) = floor.and_then(|f| f.type_name.value.as_ref()) {
            if !tree.floor_types.contains_key(p) {
                return Err(format!("{me}: its type is {p:?}, which is not a floor type in this job"));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- what is worked out

const NOT_CONVERTED: &str = "is in a unit this program does not convert";

fn feet_text(name: &str, v: &Val<Len>) -> String {
    units::to_feet_text(name, &v.unit)
}

fn dist3(a: P3, b: P3) -> f64 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
}

/// How many corners a centre line has: turns sharper than `BEND_DEGREES`.
fn bends(pts: &[P3]) -> usize {
    let limit = BEND_DEGREES.to_radians().cos();
    (1..pts.len().saturating_sub(1))
        .filter(|&i| {
            let (u, v) = ([pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1], pts[i][2] - pts[i - 1][2]], [pts[i + 1][0] - pts[i][0], pts[i + 1][1] - pts[i][1], pts[i + 1][2] - pts[i][2]]);
            let (lu, lv) = (dist3([0.0; 3], u), dist3([0.0; 3], v));
            lu > 0.0 && lv > 0.0 && (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (lu * lv) < limit
        })
        .count()
}

/// One opening as its wall sees it: where it starts and ends along the wall, its sill and
/// its head, in feet, with the worst status among them. None when a value is missing.
struct Hole {
    path: String,
    span: Option<[f64; 4]>,
    area: Option<f64>,
    status: Status,
    /// What its area was worked out from.
    inputs: Vec<String>,
}

/// A door or window worked out against its type: the hole it makes, and its type's width
/// and height, or why it has none.
fn hole_of(tree: &Tree, me: &str, o: &Opening) -> (Hole, Result<(Val<Len>, Val<Len>), String>) {
    let mut inputs = vec![format!("{me}#{K_OPENING}")];
    match typed(&tree.opening_types, &o.type_name, "a door or window") {
        Err(why) => (Hole { path: me.into(), span: None, area: None, status: Status::Unknown, inputs }, Err(why)),
        Ok((type_path, t)) => {
            inputs.push(format!("{type_path}#{K_OPENING_TYPE}"));
            let (w, h) = (t.width.feet(), t.height.feet());
            let span = o.offset.feet().zip(w).zip(o.sill.feet().zip(h)).map(|((at, w), (s, h))| [at - w / 2.0, at + w / 2.0, s, s + h]);
            let status = [o.type_name.status, t.width.status, t.height.status, o.offset.status, o.sill.status].into_iter().max().unwrap_or(Status::Unknown);
            let area = w.zip(h).map(|(w, h)| w * h);
            (Hole { path: me.into(), span, area, status, inputs }, Ok((t.width.clone(), t.height.clone())))
        }
    }
}

/// A wall's base and its height, worked out from its storey, its top level and its
/// offsets, each with why it has none when it has none.
pub(crate) struct WallHeights {
    /// Its base above the datum.
    pub(crate) base: Result<f64, String>,
    pub(crate) height: Result<f64, String>,
    /// The worst status among what the height was worked out from.
    pub(crate) status: Status,
    pub(crate) formula: String,
    pub(crate) inputs: Vec<String>,
}

/// BIMCraft's `own_base` and `own_top` (`model.rs:3890`, `3896`): a wall's base is its
/// level plus its base offset; its top is its top level plus its top offset, or its base
/// plus its height.
pub(crate) fn wall_heights(tree: &Tree, me: &str, w: &Wall) -> WallHeights {
    let mut inputs = vec![format!("{me}#{K_WALL}")];
    let storey = tree.storey_of(me);
    let own = storey.and_then(|s| tree.levels.get(s));
    let own_elevation: Result<f64, String> = match (storey, own) {
        (None, _) => Err("it is under no storey".into()),
        (Some(_), None) => Err("its storey has no level entered".into()),
        (Some(_), Some(l)) => l.elevation.feet().ok_or_else(|| "its storey's elevation is not known".to_string()),
    };
    let (base_offset, base_status): (Result<f64, String>, Status) = match &w.base_offset {
        None => (Ok(0.0), Status::Known),
        Some(o) => (o.feet().ok_or_else(|| "its base offset is not known".to_string()), o.status),
    };
    let base = own_elevation.clone().and_then(|e| base_offset.clone().map(|o| e + o));
    let (height, status, formula) = match (&w.top_level, &w.height) {
        (None, Some(h)) => (h.feet().ok_or_else(|| "its height is not known".to_string()), h.status, units::to_feet_text("height", &h.unit)),
        (Some(t), _) => {
            let top = t.value.as_ref().ok_or_else(|| "its top level is not known".to_string()).and_then(|p| {
                inputs.push(format!("{p}#{K_LEVEL}"));
                let l = tree.levels.get(p).ok_or_else(|| format!("its top level, {p}, is not a storey with a level in this job"))?;
                let e = l.elevation.feet().ok_or_else(|| format!("the elevation of {}, its top level, is not known", tree.called(p)))?;
                Ok((e, l.elevation.status))
            });
            if let Some(s) = storey {
                inputs.push(format!("{s}#{K_LEVEL}"));
            }
            let (top_offset, top_offset_status) = match &w.top_offset {
                None => (Ok(0.0), Status::Known),
                Some(o) => (o.feet().ok_or_else(|| "its top offset is not known".to_string()), o.status),
            };
            let own_status = own.map(|l| l.elevation.status).unwrap_or(Status::Unknown);
            let status = [t.status, top.as_ref().map(|x| x.1).unwrap_or(Status::Unknown), own_status, base_status, top_offset_status].into_iter().max().unwrap_or(Status::Unknown);
            let height = top.and_then(|(e, _)| top_offset.map(|o| e + o)).and_then(|top| base.clone().map(|b| top - b));
            let mut f = "elevation of its top level".to_string();
            if w.top_offset.is_some() {
                f.push_str(" + top_offset");
            }
            f.push_str(" − elevation of its storey");
            if w.base_offset.is_some() {
                f.push_str(" − base_offset");
            }
            (height, status, f)
        }
        (None, None) => (Err("it has no top level and no height".into()), Status::Unknown, String::new()),
    };
    let height = height.and_then(|h| if h > 0.0 { Ok(h) } else { Err(format!("its top is {} ft above its base; a wall's top is above its base", view::figure_shown(h, 2))) });
    WallHeights { base, height, status, formula, inputs }
}

/// A wall's thickness and its layers (function and thickness, exterior first) in feet, or
/// why they are not to be had. The layers are empty when a layer's thickness is not known
/// and the wall's line does not need them.
pub(crate) fn wall_section(tree: &Tree, w: &Wall) -> Result<(f64, Vec<(Function, f64)>, Status), String> {
    let (_, t) = typed(&tree.wall_types, &w.type_name, "a wall")?;
    let thickness = t.thickness.feet().ok_or("its type's thickness is not known")?;
    let layers: Option<Vec<(Function, f64)>> = t.layers.iter().map(|l| l.thickness.feet().map(|x| (l.function, x))).collect();
    let needs_core = matches!(w.location, Location::CoreCentre | Location::ExteriorCore | Location::InteriorCore);
    let layers = match layers {
        Some(l) => l,
        None if needs_core => return Err("a layer of its type has no thickness, so its core line cannot be placed".into()),
        None => Vec::new(),
    };
    let status = t.layers.iter().map(|l| l.thickness.status).fold(w.type_name.status.max(t.thickness.status), Status::max);
    Ok((thickness, layers, status))
}

/// The figures of every element this module knows, by path. `derive` adds them to the
/// element's measure beside whatever else it carries.
pub fn measures(world: &mut World) -> HashMap<String, BTreeMap<String, Derived>> {
    let mut out: HashMap<String, BTreeMap<String, Derived>> = HashMap::new();
    let tree = Tree::of(world);

    // Openings first: each one's size and area, and each wall's list of them.
    let mut holes: HashMap<String, Vec<Hole>> = HashMap::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Opening>)>();
    for (node, kind, opening) in q.iter(world) {
        if !kind.is_opening() {
            continue;
        }
        let me = &node.path;
        let mut m = BTreeMap::new();
        let hole = match opening {
            None => {
                let input = [format!("{me}#{K_OPENING}")];
                for (name, unit) in kind.measures() {
                    m.insert(name.to_string(), Derived::refused(unit, "", &input, "no opening entered"));
                }
                Hole { path: me.clone(), span: None, area: None, status: Status::Unknown, inputs: input.to_vec() }
            }
            Some(o) => {
                let (hole, sizes) = hole_of(&tree, me, o);
                match sizes {
                    Err(why) => {
                        m.insert("width".into(), Derived::refused("ft", "its type's width", &hole.inputs, &why));
                        m.insert("height".into(), Derived::refused("ft", "its type's height", &hole.inputs, &why));
                        m.insert("opening_area".into(), Derived::refused("SF", "width × height", &hole.inputs, &why));
                    }
                    Ok((width, height)) => {
                        let sure = o.type_name.status;
                        m.insert("width".into(), Derived::new(width.feet(), "ft", &[sure, width.status], &units::to_feet_text("its type's width", &width.unit), &hole.inputs));
                        m.insert("height".into(), Derived::new(height.feet(), "ft", &[sure, height.status], &units::to_feet_text("its type's height", &height.unit), &hole.inputs));
                        m.insert("opening_area".into(), Derived::new(hole.area, "SF", &[sure, width.status, height.status], "width × height", &hole.inputs));
                    }
                }
                hole
            }
        };
        out.entry(me.clone()).or_default().extend(m);
        if let Some(parent) = &node.parent {
            holes.entry(parent.clone()).or_default().push(hole);
        }
    }

    let mut q = world.query::<(&Node, &Kind, Option<&Wall>, Option<&Floor>, Option<&Route>, Option<&Size>)>();
    for (node, kind, wall, floor, route, size) in q.iter(world) {
        let me = &node.path;
        let mut m = BTreeMap::new();
        match kind {
            Kind::Wall => {
                let input = [format!("{me}#{K_WALL}")];
                let f_len = format!("length of the wall's line on the plan{}", wall.filter(|w| w.line.unit == units::IN).map(|_| " ÷ 12 (entered in inches)").unwrap_or_default());
                let Some(w) = wall else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no wall line entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                let line = w.line_ft();
                let length = line.map(|[a, b]| geom::dist(a, b));
                let unit_fault = w.line.value.is_some() && line.is_none();
                let plan = if unit_fault { Derived::refused("LF", &f_len, &input, &format!("the wall line {NOT_CONVERTED}")) } else { Derived::new(length, "LF", &[w.line.status], &f_len, &input) };
                // Its height: entered, or from its storey up to its top level.
                let heights = wall_heights(&tree, me, w);
                let tall = match &heights.height {
                    Ok(h) => Derived::new(Some(*h), "ft", &[heights.status], &heights.formula, &heights.inputs),
                    Err(why) => Derived::refused("ft", &heights.formula, &heights.inputs, why),
                };
                let f_face = "plan_length × height";
                let face = match (&heights.height, unit_fault) {
                    (_, true) => Derived::refused("SF", f_face, &heights.inputs, &format!("the wall line {NOT_CONVERTED}")),
                    (Err(why), _) => Derived::refused("SF", f_face, &heights.inputs, why),
                    (Ok(h), _) => Derived::new(length.map(|l| l * h), "SF", &[w.line.status, heights.status], f_face, &heights.inputs),
                };
                // Its openings: every one has to have an area, lie within the wall, and
                // not lie over another, or the wall has no net area.
                let mine = holes.get(me).map(Vec::as_slice).unwrap_or(&[]);
                let mut inputs: Vec<String> = mine.iter().flat_map(|h| h.inputs.iter().cloned()).collect();
                inputs.dedup();
                let f_open = if mine.is_empty() { "the openings under this wall (it has none)".to_string() } else { format!("the opening_area of each of the {} opening(s) under this wall, added up", mine.len()) };
                let total = mine.iter().map(|h| h.area).sum::<Option<f64>>();
                let mut statuses: Vec<Status> = mine.iter().map(|h| if h.area.is_some() { h.status } else { Status::Unknown }).collect();
                statuses.push(Status::Known);
                let open = match mine.iter().find(|h| h.area.is_none()) {
                    Some(h) => Derived::refused("SF", &f_open, &inputs, &format!("{} has no area", h.path)),
                    None => Derived::new(total, "SF", &statuses, &f_open, &inputs),
                };
                for i in heights.inputs.iter().rev() {
                    if !inputs.contains(i) {
                        inputs.insert(0, i.clone());
                    }
                }
                let mut fault = None;
                if let (Some(l), Ok(h)) = (length, &heights.height) {
                    for (i, hole) in mine.iter().enumerate() {
                        let Some([from, to, sill, head]) = hole.span else { continue };
                        // It fits its wall from end to end (BIMCraft `model.rs:6678`).
                        if from < -crate::check::FIT_TOLERANCE_FT {
                            fault = Some(format!("{} starts {} ft before the start of its wall", hole.path, view::figure_shown(-from, 2)));
                        } else if to > l + crate::check::FIT_TOLERANCE_FT {
                            fault = Some(format!("{} ends {} ft along a wall {} ft long", hole.path, view::figure_shown(to, 2), view::figure_shown(l, 2)));
                        } else if head > h + crate::check::FIT_TOLERANCE_FT {
                            fault = Some(format!("{} reaches {} ft up a wall {} ft tall", hole.path, view::figure_shown(head, 2), view::figure_shown(*h, 2)));
                        } else if let Some(other) = mine[..i].iter().find(|o| o.span.is_some_and(|s| from < s[1] && s[0] < to && sill < s[3] && s[2] < head)) {
                            fault = Some(format!("{} and {} lie over each other", other.path, hole.path));
                        }
                        if fault.is_some() {
                            break;
                        }
                    }
                }
                let f_net = "face_area − opening_area";
                let net = match (fault, face.value, open.value) {
                    (Some(why), _, _) => Derived::refused("SF", f_net, &inputs, &why),
                    (None, Some(a), Some(b)) => {
                        let mut s = statuses.clone();
                        s.push(face.status);
                        Derived::new(Some(a - b), "SF", &s, f_net, &inputs)
                    }
                    (None, None, _) => Derived::refused("SF", f_net, &inputs, if face.note.is_empty() { "the wall has no face area" } else { &face.note }),
                    (None, _, None) => Derived::refused("SF", f_net, &inputs, "an opening under this wall has no area"),
                };
                m.insert("plan_length".into(), plan);
                m.insert("height".into(), tall);
                m.insert("face_area".into(), face);
                m.insert("opening_area".into(), open);
                m.insert("net_area".into(), net);
            }
            Kind::Floor => {
                let input = [format!("{me}#{K_FLOOR}")];
                let f = format!("area of the boundary on the plan{}", floor.filter(|f| f.boundary.unit == units::IN).map(|_| " ÷ 144 (entered in inches)").unwrap_or_default());
                m.insert(
                    "plan_area".into(),
                    match floor {
                        None => Derived::refused("SF", &f, &input, "no boundary entered"),
                        Some(fl) if fl.boundary.value.is_some() && fl.boundary_ft().is_none() => Derived::refused("SF", &f, &input, &format!("the boundary {NOT_CONVERTED}")),
                        Some(fl) => Derived::new(fl.boundary_ft().map(|p| geom::plan_area(&p)), "SF", &[fl.boundary.status], &f, &input),
                    },
                );
            }
            k if k.is_route() => {
                let input = [format!("{me}#{K_ROUTE}")];
                let f_len = format!("the lengths of the centre line's segments, in space, added up{}", route.filter(|r| r.points.unit == units::IN).map(|_| " ÷ 12 (entered in inches)").unwrap_or_default());
                let f_bends = format!("corners in the centre line (a turn of more than {BEND_DEGREES}°)");
                let pts = route.and_then(|r| r.points_ft());
                let length = pts.as_ref().map(|p| p.windows(2).map(|s| dist3(s[0], s[1])).sum::<f64>());
                let why = match route {
                    None => Some("no centre line entered".to_string()),
                    Some(r) if r.points.value.is_some() && pts.is_none() => Some(format!("the centre line {NOT_CONVERTED}")),
                    _ => None,
                };
                let status = route.map(|r| r.points.status).unwrap_or(Status::Unknown);
                let (len_d, bends_d) = match &why {
                    Some(why) => (Derived::refused("LF", &f_len, &input, why), Derived::refused("EA", &f_bends, &input, why)),
                    None => (Derived::new(length, "LF", &[status], &f_len, &input), Derived::new(pts.as_ref().map(|p| bends(p) as f64), "EA", &[status], &f_bends, &input)),
                };
                if *k == Kind::Duct {
                    let inputs = [input[0].clone(), format!("{me}#{K_SIZE}")];
                    let f = match size {
                        Some(s) if s.diameter.is_some() => format!("true_length × π × {}", s.diameter.as_ref().map(|d| feet_text("diameter", d)).unwrap_or_default()),
                        Some(s) => format!("true_length × 2 × ({} + {})", s.width.as_ref().map(|w| feet_text("width", w)).unwrap_or_default(), s.depth.as_ref().map(|d| feet_text("depth", d)).unwrap_or_default()),
                        None => "true_length × the distance round the duct".to_string(),
                    };
                    m.insert(
                        "surface_area".into(),
                        match (size, len_d.value) {
                            (None, _) => Derived::refused("SF", &f, &inputs, "no size entered"),
                            (Some(_), None) => Derived::refused("SF", &f, &inputs, "the duct has no length"),
                            (Some(s), Some(l)) => Derived::new(s.perimeter_ft().map(|p| l * p), "SF", &[len_d.status, s.status()], &f, &inputs),
                        },
                    );
                }
                m.insert("true_length".into(), len_d);
                m.insert("bends".into(), bends_d);
            }
            _ => {}
        }
        if !m.is_empty() {
            out.entry(me.clone()).or_default().extend(m);
        }
    }
    out
}

// ---------------------------------------------------------------- reading it back

/// The rows a flat reading of the job shows for this module's components on one element.
pub fn rows(w: &World, e: Entity, path: &str, names: &HashMap<String, String>) -> Vec<Row> {
    let mut rows = Vec::new();
    let pair = |[a, b]: &[P2; 2]| format!("({}, {}) to ({}, {})", view::num(a[0]), view::num(a[1]), view::num(b[0]), view::num(b[1]));
    let triple = |p: &P3| format!("({}, {}, {})", view::num(p[0]), view::num(p[1]), view::num(p[2]));
    let plain = |label: &str, text: String, basis: &str| Row { path: path.into(), label: label.into(), text, status: Status::Known, basis: basis.into(), note: String::new(), derived: false, number: None, unit: String::new() };
    let named = |p: &String| names.get(p).map(|n| format!("{n} ({p})")).unwrap_or_else(|| p.clone());
    let layers = |rows: &mut Vec<Row>, list: &[Layer]| {
        for (i, l) in list.iter().enumerate() {
            rows.push(view::entered(path, &format!("layer {}, {:?}", i + 1, l.function).to_lowercase(), &l.material, |m| m.clone()));
            rows.push(view::length(path, &format!("layer {} thickness", i + 1), &l.thickness));
        }
    };
    if let Some(c) = w.get::<Level>(e) {
        rows.push(view::length(path, "elevation", &c.elevation));
        rows.push(view::length(path, "storey height", &c.height));
    }
    if let Some(c) = w.get::<WallType>(e) {
        rows.push(view::length(path, "wall thickness", &c.thickness));
        layers(&mut rows, &c.layers);
    }
    if let Some(c) = w.get::<FloorType>(e) {
        rows.push(view::length(path, "floor thickness", &c.thickness));
        layers(&mut rows, &c.layers);
    }
    if let Some(c) = w.get::<OpeningType>(e) {
        rows.push(plain("a type of", format!("{:?}", c.category).to_lowercase(), "what the type is for"));
        rows.push(view::length(path, "width", &c.width));
        rows.push(view::length(path, "height", &c.height));
    }
    if let Some(c) = w.get::<Wall>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "wall line", &c.line, pair));
        if let Some(t) = &c.top_level {
            rows.push(view::entered(path, "top level", t, named));
        }
        if let Some(h) = &c.height {
            rows.push(view::length(path, "height", h));
        }
        if let Some(o) = &c.base_offset {
            rows.push(view::length(path, "base offset", o));
        }
        if let Some(o) = &c.top_offset {
            rows.push(view::length(path, "top offset", o));
        }
        if c.location != Location::Centre {
            rows.push(plain("its line is its", c.location.label().into(), "which line of the wall was drawn"));
        }
        if !c.room_bounding {
            rows.push(plain("bounds rooms", "no".into(), "a room runs past it"));
        }
        if !c.join {
            rows.push(plain("joins its neighbours", "no".into(), "its ends stay square"));
        }
    }
    if let Some(c) = w.get::<Opening>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::length(path, "centre, along the wall", &c.offset));
        rows.push(view::length(path, "sill", &c.sill));
    }
    if let Some(c) = w.get::<Floor>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "boundary", &c.boundary, |pts| format!("{} points", pts.len())));
        if let Some(o) = &c.offset {
            rows.push(view::length(path, "offset", o));
        }
    }
    if let Some(c) = w.get::<Placed>(e) {
        rows.push(view::entered(path, "what it is", &c.what, |s| s.clone()));
        rows.push(view::entered(path, "position", &c.at, triple));
        rows.push(view::entered(path, "size", &c.size, |s| format!("{} × {} × {}", view::num(s[0]), view::num(s[1]), view::num(s[2]))));
    }
    if let Some(c) = w.get::<Route>(e) {
        rows.push(view::entered(path, "centre line", &c.points, |pts| format!("{} points", pts.len())));
    }
    if let Some(c) = w.get::<Size>(e) {
        for (name, v) in [("diameter", &c.diameter), ("width", &c.width), ("depth", &c.depth)] {
            if let Some(v) = v {
                rows.push(view::length(path, name, v));
            }
        }
    }
    if let Some(c) = w.get::<SystemInfo>(e) {
        rows.push(Row { path: path.into(), label: "classification".into(), text: format!("{} ({:?})", c.classification, c.discipline), status: Status::Known, basis: "what the system is".into(), note: String::new(), derived: false, number: None, unit: String::new() });
    }
    if let Some(c) = w.get::<InSystem>(e) {
        rows.push(view::entered(path, "system", &c.0, |p| names.get(p).cloned().unwrap_or_else(|| p.clone())));
    }
    rows
}

/// What a window needs to draw an element: its shape in feet, in space.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A wall from `a` to `b` on plan, standing from `base` to `base + height`, with the
    /// openings cut in it: each as [from, to] along the wall and [sill, head] above `base`.
    Wall { a: P2, b: P2, base: f64, height: f64, thickness: f64, holes: Vec<(String, Kind, [f64; 4])> },
    Floor { boundary: Vec<P2>, top: f64, thickness: f64 },
    /// A centre line and its section: round by a diameter, rectangular by width and depth,
    /// or neither when its size is not known.
    Route { points: Vec<P3>, round: Option<f64>, rect: Option<(f64, f64)> },
    /// A box by the middle of its base and its size.
    Box { at: P3, size: P3 },
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    /// The storey it is under, by path.
    pub storey: Option<String>,
    /// The system it belongs to, by path.
    pub system: Option<String>,
    pub discipline: Discipline,
    pub shape: Shape,
    /// The worst status among the values its shape is drawn from.
    pub status: Status,
}

/// Every element of this module's kinds that has a shape, placed in space, and every one
/// that has none with the reason. A window draws the first list and has to say the second:
/// an element that cannot be drawn does not just go missing from the picture.
pub fn placements(job: &mut Job) -> (Vec<Placement>, Vec<(String, String)>) {
    let order = job.order.clone();
    let tree = Tree::of(&mut job.world);
    let w = &job.world;
    let node = |e: Entity| w.get::<Node>(e);
    let index: HashMap<String, Entity> = order.iter().filter_map(|e| node(*e).map(|n| (n.path.clone(), *e))).collect();
    let mut drawn = Vec::new();
    let mut not_drawn = Vec::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (node(e), w.get::<Kind>(e).copied()) else { continue };
        if !(matches!(kind, Kind::Wall | Kind::Floor) || kind.is_route() || kind.is_placed()) {
            continue;
        }
        let storey = tree.storey_of(&n.path).map(str::to_string);
        let system = w.get::<InSystem>(e).and_then(|s| s.0.value.clone());
        let discipline = system.as_ref().and_then(|p| index.get(p)).and_then(|s| w.get::<SystemInfo>(*s)).map(|s| s.discipline).unwrap_or(kind.discipline());
        // Where its storey's floor is: None when it is under no storey, Some(None) when
        // the storey's elevation is not known.
        let base = storey.as_ref().map(|s| tree.levels.get(s).and_then(|l| l.elevation.feet()));
        // Openings under this wall that cannot be cut in it, each with why.
        let mut uncut: Vec<(String, String)> = Vec::new();
        let shape: Result<(Shape, Status), String> = match kind {
            Kind::Wall => (|| {
                let c = w.get::<Wall>(e).ok_or("it has no wall line")?;
                let [a, b] = c.line_ft().ok_or("its line is not known")?;
                let heights = wall_heights(&tree, &n.path, c);
                let base = heights.base.clone()?;
                let height = heights.height.clone()?;
                let (thickness, layers, section_status) = wall_section(&tree, c)?;
                let mut status = c.line.status.max(heights.status).max(section_status);
                let length = geom::dist(a, b);
                // The shape is drawn about the wall's centre line, which lies to the left
                // of the line that was drawn by as much as the wall's `location` says.
                let off = walls::centre(thickness, &layers, c.location);
                let left = [-(b[1] - a[1]) / length * off, (b[0] - a[0]) / length * off];
                let (a, b) = ([a[0] + left[0], a[1] + left[1]], [b[0] + left[0], b[1] + left[1]]);
                let mut holes = Vec::new();
                for &child in &order {
                    let Some(cn) = node(child) else { continue };
                    if cn.parent.as_deref() != Some(n.path.as_str()) {
                        continue;
                    }
                    let Some(k) = w.get::<Kind>(child).copied().filter(|k| k.is_opening()) else { continue };
                    let Some(o) = w.get::<Opening>(child) else {
                        uncut.push((cn.path.clone(), "it has no opening entered, so it is not cut in its wall".into()));
                        continue;
                    };
                    // An opening is cut only where it is known and lies in the wall. One
                    // that is not cut is said, with why: it does not just go missing.
                    let (hole, _) = hole_of(&tree, &cn.path, o);
                    match hole.span {
                        None => uncut.push((cn.path.clone(), "where it is or how big it is is not known, so it is not cut in its wall".into())),
                        Some([from, to, _, _]) if from < -crate::check::FIT_TOLERANCE_FT || to > length + crate::check::FIT_TOLERANCE_FT => uncut.push((cn.path.clone(), format!("it runs past the end of {}, so it is not cut in it", n.path))),
                        Some([_, _, _, head]) if head > height + crate::check::FIT_TOLERANCE_FT => uncut.push((cn.path.clone(), format!("it stands taller than {}, so it is not cut in it", n.path))),
                        Some(span) => {
                            holes.push((cn.path.clone(), k, span));
                            status = status.max(hole.status);
                        }
                    }
                }
                Ok((Shape::Wall { a, b, base, height, thickness, holes }, status))
            })(),
            Kind::Floor => (|| {
                let c = w.get::<Floor>(e).ok_or("it has no boundary")?;
                let boundary = c.boundary_ft().ok_or("its boundary is not known")?;
                let level = base.ok_or("it is under no storey")?.ok_or("its storey's elevation is not known")?;
                let (offset, offset_status) = match &c.offset {
                    None => (0.0, Status::Known),
                    Some(o) => (o.feet().ok_or("its offset is not known")?, o.status),
                };
                let (_, t) = typed(&tree.floor_types, &c.type_name, "a floor")?;
                let thickness = t.thickness.feet().ok_or("its type's thickness is not known")?;
                Ok((Shape::Floor { boundary, top: level + offset, thickness }, c.boundary.status.max(c.type_name.status).max(t.thickness.status).max(offset_status)))
            })(),
            k if k.is_route() => (|| {
                let c = w.get::<Route>(e).ok_or("it has no centre line")?;
                let points = c.points_ft().ok_or("its centre line is not known")?;
                let size = w.get::<Size>(e);
                let status = c.points.status.max(size.map(|s| s.status()).unwrap_or(Status::Unknown));
                Ok((Shape::Route { points, round: size.and_then(|s| s.round_ft()), rect: size.and_then(|s| s.rect_ft()) }, status))
            })(),
            _ => (|| {
                let c = w.get::<Placed>(e).ok_or("it has no position")?;
                let at = c.at_ft().ok_or("its position is not known")?;
                let size = c.size_ft().ok_or("its size is not known")?;
                Ok((Shape::Box { at, size }, c.at.status.max(c.size.status)))
            })(),
        };
        match shape {
            Ok((shape, status)) => drawn.push(Placement { path: n.path.clone(), name: n.name.clone(), kind, storey, system, discipline, shape, status }),
            Err(why) => not_drawn.push((n.path.clone(), why)),
        }
        not_drawn.extend(uncut);
    }
    (drawn, not_drawn)
}

/// One line of a system's takeoff: the runs of one kind and one size.
#[derive(Clone, Debug)]
pub struct TakeoffLine {
    pub kind: Kind,
    /// The size, said in inches whatever unit each run's was entered in, so that one
    /// size is one line.
    pub size: String,
    /// The worst status among the sizes of the runs on this line.
    pub size_status: Status,
    pub length: Sum,
    pub bends: Sum,
}

#[derive(Clone, Debug)]
pub struct SystemTakeoff {
    /// The system's path. Empty for the two groups that are not systems: what is on no
    /// system, and what is on a system nobody has named.
    pub path: String,
    pub name: String,
    pub discipline: Discipline,
    pub classification: String,
    /// How sure it is that what is listed here belongs here. Known for a system and for
    /// what is on none; Unknown for what is on a system that is not known.
    pub flag: Status,
    /// All of its runs, whatever their size: how many and how far.
    pub runs: Sum,
    pub lines: Vec<TakeoffLine>,
    /// Equipment and fixtures on the system: what each is, how many, and the worst status
    /// among what they are said to be and whether they are on this system.
    pub placed: Vec<(Kind, String, usize, Status)>,
}

/// A size in feet, said in inches: `6 in`, `3/4 in`.
fn in_inches(feet: f64) -> String {
    units::show_length(&Len::One((feet * 12.0 * 1.0e6).round() / 1.0e6), units::IN)
}

/// What each system comes to: its runs by kind and size with their length and bends, and
/// its equipment and fixtures counted by what they are.
///
/// A run said to be on a system with less than certainty (Assumed) carries that onto its
/// line: the line's length is no surer than that the run is on this system. Two groups
/// come after the systems, when there is anything in them, so nothing is left off for
/// want of a system: what is on no system, and what is on a system that is not known.
/// Not knowing the system is not the same as having none.
///
/// Runs are grouped by their size as a size, not as it was typed: 6 in, 0.5 ft and 0' 6"
/// are one line. Lines are in order of size, smallest first, with no size last.
pub fn takeoff(job: &mut Job) -> Vec<SystemTakeoff> {
    let order = job.order.clone();
    let w = &job.world;
    let blank = |name: &str, flag: Status| SystemTakeoff { path: String::new(), name: name.into(), discipline: Discipline::Architecture, classification: String::new(), flag, runs: view::sum_of("LF", &[]), lines: Vec::new(), placed: Vec::new() };
    let mut systems: Vec<SystemTakeoff> = Vec::new();
    for &e in &order {
        if let (Some(n), Some(info)) = (w.get::<Node>(e), w.get::<SystemInfo>(e)) {
            systems.push(SystemTakeoff { path: n.path.clone(), name: n.name.clone(), discipline: info.discipline, classification: info.classification.clone(), ..blank("", Status::Known) });
        }
    }
    let named = systems.len();
    systems.push(blank("On no system", Status::Known));
    systems.push(blank("System not known", Status::Unknown));
    type Figures = Vec<(String, Option<f64>, Status)>;
    // By system, then kind, then size: round before rectangular, small before large, and
    // a run with no size last.
    let mut runs: BTreeMap<(usize, u8, u8, i64, i64), (Kind, String, Status, Figures, Figures)> = BTreeMap::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e).copied()) else { continue };
        if !(kind.is_route() || kind.is_placed()) {
            continue;
        }
        // Which group it goes in, and how sure that is.
        let (at, sure) = match w.get::<InSystem>(e) {
            None => (named, Status::Known),
            Some(s) => match s.0.value.as_ref().and_then(|p| systems[..named].iter().position(|x| x.path == *p)) {
                Some(i) if s.0.status != Status::Unknown => (i, s.0.status),
                _ => (named + 1, Status::Known),
            },
        };
        if kind.is_placed() {
            let what = w.get::<Placed>(e).map(|p| (p.what.value.clone().unwrap_or_else(|| "not said".into()), p.what.status)).unwrap_or(("not said".into(), Status::Unknown));
            let list = &mut systems[at].placed;
            match list.iter_mut().find(|x| x.0 == kind && x.1 == what.0) {
                Some(entry) => {
                    entry.2 += 1;
                    entry.3 = entry.3.max(what.1).max(sure);
                }
                None => list.push((kind, what.0, 1, what.1.max(sure))),
            }
            continue;
        }
        let size = w.get::<Size>(e);
        let micro = |feet: f64| (feet * 12.0 * 1.0e6).round() as i64;
        let (shape, a, b, said) = match size.map(|s| (s.round_ft(), s.rect_ft())) {
            Some((Some(d), _)) => (0, micro(d), 0, format!("{} round", in_inches(d))),
            Some((None, Some((width, depth)))) => (1, micro(width), micro(depth), format!("{} × {}", in_inches(width), in_inches(depth))),
            _ => (2, 0, 0, "size unknown".to_string()),
        };
        let rank = match kind {
            Kind::Duct => 0,
            Kind::Pipe => 1,
            Kind::Conduit => 2,
            _ => 3,
        };
        let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (d.value, d.status)).unwrap_or((None, Status::Unknown));
        let entry = runs.entry((at, rank, shape, a, b)).or_insert((kind, said, Status::Known, Vec::new(), Vec::new()));
        entry.2 = entry.2.max(if shape == 2 { Status::Unknown } else { size.map(|s| s.status()).unwrap_or(Status::Unknown) });
        // A figure that is there is no surer than that the run is on this system.
        let (l, ls) = figure("true_length");
        let (bends, bs) = figure("bends");
        entry.3.push((n.path.clone(), l, if l.is_some() { ls.max(sure) } else { ls }));
        entry.4.push((n.path.clone(), bends, if bends.is_some() { bs.max(sure) } else { bs }));
    }
    let mut all: Vec<Figures> = vec![Vec::new(); systems.len()];
    for ((at, ..), (_, _, _, lengths, _)) in &runs {
        all[*at].extend(lengths.iter().cloned());
    }
    for (i, figures) in all.iter().enumerate() {
        systems[i].runs = view::sum_of("LF", figures);
    }
    for ((at, ..), (kind, size, size_status, lengths, bends)) in runs {
        systems[at].lines.push(TakeoffLine { kind, size, size_status, length: view::sum_of("LF", &lengths), bends: view::sum_of("EA", &bends) });
    }
    systems.into_iter().enumerate().filter(|(i, t)| *i < named || !t.lines.is_empty() || !t.placed.is_empty()).map(|(_, t)| t).collect()
}

/// What a storey's walls and floors come to.
#[derive(Clone, Debug)]
pub struct StoreyTotals {
    pub path: String,
    pub name: String,
    pub elevation: Option<f64>,
    pub walls: Sum,
    pub wall_face: Sum,
    pub wall_net: Sum,
    pub floors: Sum,
    pub doors: usize,
    pub windows: usize,
}

pub fn storey_totals(job: &mut Job) -> Vec<StoreyTotals> {
    let order = job.order.clone();
    let w = &job.world;
    let parents: HashMap<String, Option<String>> = order.iter().filter_map(|e| w.get::<Node>(*e)).map(|n| (n.path.clone(), n.parent.clone())).collect();
    let storeys: Vec<String> = order.iter().filter(|e| w.get::<Kind>(**e) == Some(&Kind::Storey)).filter_map(|e| w.get::<Node>(*e)).map(|n| n.path.clone()).collect();
    // An element counts on the nearest storey above it in the tree, and on no other.
    let under = |path: &str, storey: &str| {
        let mut at = parents.get(path).cloned().flatten();
        let mut steps = 0;
        while let Some(p) = at {
            if storeys.contains(&p) {
                return p == storey;
            }
            at = parents.get(&p).cloned().flatten();
            steps += 1;
            if steps > parents.len() {
                break;
            }
        }
        false
    };
    let mut out = Vec::new();
    for &s in &order {
        let (Some(sn), Some(Kind::Storey)) = (w.get::<Node>(s), w.get::<Kind>(s)) else { continue };
        let mut figures: [Vec<(String, Option<f64>, Status)>; 4] = Default::default();
        let (mut doors, mut windows) = (0, 0);
        for &e in &order {
            let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
            if !under(&n.path, &sn.path) {
                continue;
            }
            let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (n.path.clone(), d.value, d.status)).unwrap_or((n.path.clone(), None, Status::Unknown));
            match kind {
                Kind::Wall => {
                    figures[0].push(figure("plan_length"));
                    figures[1].push(figure("face_area"));
                    figures[2].push(figure("net_area"));
                }
                Kind::Floor => figures[3].push(figure("plan_area")),
                Kind::Door => doors += 1,
                Kind::Window => windows += 1,
                _ => {}
            }
        }
        out.push(StoreyTotals {
            path: sn.path.clone(),
            name: sn.name.clone(),
            elevation: w.get::<Level>(s).and_then(|l| l.elevation.feet()),
            walls: view::sum_of("LF", &figures[0]),
            wall_face: view::sum_of("SF", &figures[1]),
            wall_net: view::sum_of("SF", &figures[2]),
            floors: view::sum_of("SF", &figures[3]),
            doors,
            windows,
        });
    }
    out
}
