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

/// A point in space: east, north, height above the datum.
pub type P3 = [f64; 3];

/// A turn in a centre line sharper than this many degrees is a bend: a fitting.
pub const BEND_DEGREES: f64 = 1.0;
/// A cross-section beyond this (in feet) is a typing error.
pub const MAX_SECTION_FT: f64 = 20.0;

pub const K_LEVEL: &str = "pl::level";
pub const K_WALL: &str = "pl::wall";
pub const K_OPENING: &str = "pl::opening";
pub const K_FLOOR: &str = "pl::floor";
pub const K_ROUTE: &str = "pl::route";
pub const K_SIZE: &str = "pl::size";
pub const K_PLACED: &str = "pl::placed";
pub const K_SYSTEM: &str = "pl::system";
pub const K_IN_SYSTEM: &str = "pl::in_system";

/// `pl::level`: on a Storey, how high its floor is above the datum and how far it is up to
/// the next floor.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    pub elevation: Val<Len>,
    pub height: Val<Len>,
}

/// `pl::wall`: a wall by its centre line on the plan, how thick it is and how tall. It
/// stands on its storey's floor.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wall {
    pub line: Val<[P2; 2]>,
    pub thickness: Val<Len>,
    pub height: Val<Len>,
}

impl Wall {
    pub fn line_ft(&self) -> Option<[P2; 2]> {
        let d = units::coord_divisor(&self.line.unit).ok()?;
        self.line.value.map(|l| l.map(|p| [p[0] / d, p[1] / d]))
    }
}

/// `pl::opening`: on a Door or a Window under a wall. `along` is how far from the wall
/// line's first end the opening starts; `sill` is how high its bottom is above the floor.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opening {
    pub along: Val<Len>,
    pub width: Val<Len>,
    pub height: Val<Len>,
    pub sill: Val<Len>,
}

/// `pl::floor`: a floor by its boundary on the plan and how thick it is. Its top is at its
/// storey's elevation.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    pub boundary: Val<Vec<P2>>,
    pub thickness: Val<Len>,
}

impl Floor {
    pub fn boundary_ft(&self) -> Option<Vec<P2>> {
        let d = units::coord_divisor(&self.boundary.unit).ok()?;
        self.boundary.value.as_ref().map(|pts| pts.iter().map(|p| [p[0] / d, p[1] / d]).collect())
    }
}

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

fn positive(what: &str, v: &Val<Len>, zero_ok: bool) -> Result<(), String> {
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

fn coords<T>(what: &str, v: &Val<T>) -> Result<f64, String> {
    v.check_basics(what)?;
    units::coord_divisor(&v.unit).map_err(|why| format!("{what}: {why}"))
}

fn usable(what: &str, xs: &[f64], divisor: f64) -> Result<(), String> {
    if xs.iter().any(|x| !x.is_finite() || (x / divisor).abs() > MAX_COORDINATE_FT) {
        return Err(format!("{what}: a coordinate is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    Ok(())
}

pub fn check_level(path: &str, kind: Kind, l: &Level) -> Result<(), String> {
    if kind != Kind::Storey {
        return Err(format!("{path}: a {kind:?} cannot carry a level; only a storey can"));
    }
    l.elevation.check_length(&format!("{path} elevation"))?;
    if l.elevation.feet().is_some_and(|x| x.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{path} elevation: is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    positive(&format!("{path} storey height"), &l.height, false)
}

pub fn check_wall(path: &str, kind: Kind, w: &Wall) -> Result<(), String> {
    if kind != Kind::Wall {
        return Err(format!("{path}: a {kind:?} cannot carry a wall's line; only a wall can"));
    }
    let d = coords(&format!("{path} wall line"), &w.line)?;
    if let Some([a, b]) = w.line.value {
        usable(&format!("{path} wall line"), &[a[0], a[1], b[0], b[1]], d)?;
        if a == b {
            return Err(format!("{path} wall line: the two ends are the same point"));
        }
    }
    positive(&format!("{path} wall thickness"), &w.thickness, false)?;
    positive(&format!("{path} wall height"), &w.height, false)
}

pub fn check_opening(path: &str, kind: Kind, o: &Opening) -> Result<(), String> {
    if !kind.is_opening() {
        return Err(format!("{path}: a {kind:?} cannot carry an opening; only a door or a window can"));
    }
    positive(&format!("{path} distance along the wall"), &o.along, true)?;
    positive(&format!("{path} opening width"), &o.width, false)?;
    positive(&format!("{path} opening height"), &o.height, false)?;
    positive(&format!("{path} sill height"), &o.sill, true)
}

pub fn check_floor(path: &str, kind: Kind, f: &Floor) -> Result<(), String> {
    if kind != Kind::Floor {
        return Err(format!("{path}: a {kind:?} cannot carry a floor's boundary; only a floor can"));
    }
    let d = coords(&format!("{path} floor boundary"), &f.boundary)?;
    if let Some(pts) = &f.boundary.value {
        for p in pts {
            usable(&format!("{path} floor boundary"), p, d)?;
        }
    }
    if let Some(pts) = f.boundary_ft() {
        if let Some(fault) = geom::outline_fault(&pts) {
            return Err(format!("{path} floor boundary: {fault}"));
        }
    }
    positive(&format!("{path} floor thickness"), &f.thickness, false)
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

/// What can only be checked once every element is in: an opening is under a wall, a wall
/// and a floor are under a storey, and a system named is a system in the job.
pub fn check_whole(job: &mut Job) -> Result<(), String> {
    let mut kinds: HashMap<String, Kind> = HashMap::new();
    let mut q = job.world.query::<(&Node, &Kind)>();
    for (node, kind) in q.iter(&job.world) {
        kinds.insert(node.path.clone(), *kind);
    }
    let mut q = job.world.query::<(&Node, &Kind, Option<&InSystem>)>();
    for (node, kind, in_system) in q.iter(&job.world) {
        let parent = node.parent.as_ref().and_then(|p| kinds.get(p)).copied();
        if kind.is_opening() && parent != Some(Kind::Wall) {
            return Err(format!("{}: a {kind:?} is an opening in a wall and has to be under one", node.path));
        }
        if matches!(kind, Kind::Wall | Kind::Floor) && parent != Some(Kind::Storey) {
            return Err(format!("{}: a {kind:?} stands on a storey and has to be under one", node.path));
        }
        if *kind == Kind::Storey && parent != Some(Kind::Job) {
            return Err(format!("{}: a storey is a floor of the building and has to be directly under the job, not under {}", node.path, node.parent.as_deref().unwrap_or("nothing")));
        }
        if let Some(named) = in_system.and_then(|s| s.0.value.as_ref()) {
            if kinds.get(named) != Some(&Kind::System) {
                return Err(format!("{}: says it belongs to {named:?}, which is not a system in this job", node.path));
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
}

/// The figures of every element this module knows, by path. `derive` adds them to the
/// element's measure beside whatever else it carries.
pub fn measures(world: &mut World) -> HashMap<String, BTreeMap<String, Derived>> {
    let mut out: HashMap<String, BTreeMap<String, Derived>> = HashMap::new();

    // Openings first: each one's area, and each wall's list of them.
    let mut holes: HashMap<String, Vec<Hole>> = HashMap::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Opening>)>();
    for (node, kind, opening) in q.iter(world) {
        if !kind.is_opening() {
            continue;
        }
        let me = &node.path;
        let input = [format!("{me}#{K_OPENING}")];
        let (d, hole) = match opening {
            None => (Derived::refused("SF", "width × height", &input, "no opening entered"), Hole { path: me.clone(), span: None, area: None, status: Status::Unknown }),
            Some(o) => {
                let f = format!("{} × {}", feet_text("width", &o.width), feet_text("height", &o.height));
                let area = o.width.feet().zip(o.height.feet()).map(|(w, h)| w * h);
                let d = Derived::new(area, "SF", &[o.width.status, o.height.status], &f, &input);
                let span = o.along.feet().zip(o.width.feet()).zip(o.sill.feet().zip(o.height.feet())).map(|((a, w), (s, h))| [a, a + w, s, s + h]);
                let status = [o.along.status, o.width.status, o.height.status, o.sill.status].into_iter().max().unwrap_or(Status::Unknown);
                (d.clone(), Hole { path: me.clone(), span, area: d.value, status })
            }
        };
        out.entry(me.clone()).or_default().insert("opening_area".into(), d);
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
                let f_face = format!("plan_length × {}", feet_text("height", &w.height));
                let face = if unit_fault { Derived::refused("SF", &f_face, &input, &format!("the wall line {NOT_CONVERTED}")) } else { Derived::new(length.zip(w.height.feet()).map(|(l, h)| l * h), "SF", &[w.line.status, w.height.status], &f_face, &input) };
                // Its openings: every one has to have an area, lie within the wall, and
                // not lie over another, or the wall has no net area.
                let mine = holes.get(me).map(Vec::as_slice).unwrap_or(&[]);
                let mut inputs: Vec<String> = mine.iter().map(|h| format!("{}#{K_OPENING}", h.path)).collect();
                let f_open = if mine.is_empty() { "the openings under this wall (it has none)".to_string() } else { format!("the opening_area of each of the {} opening(s) under this wall, added up", mine.len()) };
                let total = mine.iter().map(|h| h.area).sum::<Option<f64>>();
                let mut statuses: Vec<Status> = mine.iter().map(|h| if h.area.is_some() { h.status } else { Status::Unknown }).collect();
                statuses.push(Status::Known);
                let open = match mine.iter().find(|h| h.area.is_none()) {
                    Some(h) => Derived::refused("SF", &f_open, &inputs, &format!("{} has no area", h.path)),
                    None => Derived::new(total, "SF", &statuses, &f_open, &inputs),
                };
                inputs.insert(0, input[0].clone());
                let mut fault = None;
                if let (Some(l), Some(h)) = (length, w.height.feet()) {
                    for (i, hole) in mine.iter().enumerate() {
                        let Some([from, to, sill, head]) = hole.span else { continue };
                        if to > l + crate::check::FIT_TOLERANCE_FT {
                            fault = Some(format!("{} ends {} ft along a wall {} ft long", hole.path, view::figure_shown(to, 2), view::figure_shown(l, 2)));
                        } else if head > h + crate::check::FIT_TOLERANCE_FT {
                            fault = Some(format!("{} reaches {} ft up a wall {} ft tall", hole.path, view::figure_shown(head, 2), view::figure_shown(h, 2)));
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
                    (None, None, _) => Derived::refused("SF", f_net, &inputs, "the wall has no face area"),
                    (None, _, None) => Derived::refused("SF", f_net, &inputs, "an opening under this wall has no area"),
                };
                m.insert("plan_length".into(), plan);
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
    if let Some(c) = w.get::<Level>(e) {
        rows.push(view::length(path, "elevation", &c.elevation));
        rows.push(view::length(path, "storey height", &c.height));
    }
    if let Some(c) = w.get::<Wall>(e) {
        rows.push(view::entered(path, "wall line", &c.line, pair));
        rows.push(view::length(path, "thickness", &c.thickness));
        rows.push(view::length(path, "height", &c.height));
    }
    if let Some(c) = w.get::<Opening>(e) {
        rows.push(view::length(path, "along the wall", &c.along));
        rows.push(view::length(path, "width", &c.width));
        rows.push(view::length(path, "height", &c.height));
        rows.push(view::length(path, "sill", &c.sill));
    }
    if let Some(c) = w.get::<Floor>(e) {
        rows.push(view::entered(path, "boundary", &c.boundary, |pts| format!("{} points", pts.len())));
        rows.push(view::length(path, "thickness", &c.thickness));
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
    let w = &job.world;
    let node = |e: Entity| w.get::<Node>(e);
    let index: HashMap<String, Entity> = order.iter().filter_map(|e| node(*e).map(|n| (n.path.clone(), *e))).collect();
    // The storey an element stands on: the nearest one above it in the tree.
    let storey_of = |start: &Node| -> Option<(String, Option<f64>)> {
        let mut at = start.parent.clone();
        let mut steps = 0;
        while let Some(p) = at {
            let e = *index.get(&p)?;
            if w.get::<Kind>(e) == Some(&Kind::Storey) {
                return Some((p, w.get::<Level>(e).and_then(|l| l.elevation.feet())));
            }
            at = node(e)?.parent.clone();
            steps += 1;
            if steps > order.len() {
                return None;
            }
        }
        None
    };
    let mut drawn = Vec::new();
    let mut not_drawn = Vec::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (node(e), w.get::<Kind>(e).copied()) else { continue };
        if !(matches!(kind, Kind::Wall | Kind::Floor) || kind.is_route() || kind.is_placed()) {
            continue;
        }
        let storey = storey_of(n);
        let system = w.get::<InSystem>(e).and_then(|s| s.0.value.clone());
        let discipline = system.as_ref().and_then(|p| index.get(p)).and_then(|s| w.get::<SystemInfo>(*s)).map(|s| s.discipline).unwrap_or(kind.discipline());
        let base = storey.as_ref().map(|s| s.1);
        // Openings under this wall that cannot be cut in it, each with why.
        let mut uncut: Vec<(String, String)> = Vec::new();
        let shape: Result<(Shape, Status), String> = match kind {
            Kind::Wall => (|| {
                let c = w.get::<Wall>(e).ok_or("it has no wall line")?;
                let [a, b] = c.line_ft().ok_or("its line is not known")?;
                let base = base.ok_or("it is under no storey")?.ok_or("its storey's elevation is not known")?;
                let height = c.height.feet().ok_or("its height is not known")?;
                let thickness = c.thickness.feet().ok_or("its thickness is not known")?;
                let mut status = c.line.status.max(c.height.status).max(c.thickness.status);
                let length = geom::dist(a, b);
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
                    match o.along.feet().zip(o.width.feet()).zip(o.sill.feet().zip(o.height.feet())) {
                        None => uncut.push((cn.path.clone(), "where it is or how big it is is not known, so it is not cut in its wall".into())),
                        Some(((from, width), _)) if from + width > length + crate::check::FIT_TOLERANCE_FT => uncut.push((cn.path.clone(), format!("it runs past the end of {}, so it is not cut in it", n.path))),
                        Some((_, (sill, tall))) if sill + tall > height + crate::check::FIT_TOLERANCE_FT => uncut.push((cn.path.clone(), format!("it stands taller than {}, so it is not cut in it", n.path))),
                        Some(((from, width), (sill, tall))) => {
                            holes.push((cn.path.clone(), k, [from, from + width, sill, sill + tall]));
                            status = status.max(o.along.status).max(o.width.status).max(o.sill.status).max(o.height.status);
                        }
                    }
                }
                Ok((Shape::Wall { a, b, base, height, thickness, holes }, status))
            })(),
            Kind::Floor => (|| {
                let c = w.get::<Floor>(e).ok_or("it has no boundary")?;
                let boundary = c.boundary_ft().ok_or("its boundary is not known")?;
                let top = base.ok_or("it is under no storey")?.ok_or("its storey's elevation is not known")?;
                let thickness = c.thickness.feet().ok_or("its thickness is not known")?;
                Ok((Shape::Floor { boundary, top, thickness }, c.boundary.status.max(c.thickness.status)))
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
            Ok((shape, status)) => drawn.push(Placement { path: n.path.clone(), name: n.name.clone(), kind, storey: storey.map(|s| s.0), system, discipline, shape, status }),
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
