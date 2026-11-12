//! The rest of the building: storeys, walls with their doors and windows, floors, rooms,
//! and what runs through them. A duct, a pipe, a conduit and a cable are each straight
//! pieces with a section, joined by fittings. Equipment and fixtures are things set at a
//! place. Any of them can belong to a system (supply air, domestic hot water, power), and
//! a system is an element of the job like any other, so it can be read, counted and
//! totalled.
//!
//! The shapes of these things are BIMCraft's, ported: they are in `building/model.rs`,
//! with their geometry in `building/walls.rs`, `building/runs.rs`, `building/rooms.rs`
//! and `building/network.rs`. Each of those files says what it is ported from and what
//! was changed, and `PORTMAP.md` accounts for every public item, here and there.
//!
//! This file is what this program adds on top, and is ours: every figure is worked out
//! here, never in a window, takes the worst status of what it was worked out from, and is
//! refused with the reason when its inputs do not fit together (a window wider than its
//! wall gives no net area); a system is an element; a takeoff is by system; what cannot
//! be drawn is listed with why.
//!
//! PORTED CODE is in it too, of two sorts, both from BIMCraft, `crates/core/src/model.rs`,
//! `runs.rs` and `view3d.rs`, at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71,
//! https://github.com/IPedrax/bimcraft, Copyright (c) 2026 IPedrax and the BIMCraft
//! contributors, used under the MIT license; the notice is in NOTICE at the repository
//! root.
//! * Five public items, marked ported in `PORTMAP.md`: the point type `P3`, a run's
//!   system (`InSystem`), and which kinds are runs, openings and placed things
//!   (`is_segment`, `is_opening`, `is_placed`). Changed: a run's system is a component
//!   pointing at a system element, where BIMCraft's is a name on the run; the three lists
//!   of kinds are of this program's kinds.
//! * Four rules of BIMCraft's that the figures here are worked out by, each cited at the
//!   place it is used: a wall's base is its level plus its base offset and its top is its
//!   top level plus its top offset or its base plus its height (`own_base`, `own_top`,
//!   `model.rs:3890`, `3896`), in `wall_heights`; an opening fits its wall from end to end
//!   (`model.rs:6678`), in the net area; a wall bounds a storey's rooms when it spans the
//!   storey's computation height (`bounding`, `model.rs:3932`; `spans`, `model.rs:3960`),
//!   in `room_measures`; a run's true length is through its rise (`runs.rs:107`), in
//!   `measures`. Changed in each: the inputs are flagged values in feet, the result takes
//!   the worst status among them, and where BIMCraft refuses the model the figure is
//!   refused with the reason and the job still loads.
//!
//! The names are the trade's and the open standard's, not ours: storey, wall, opening,
//! floor (IFC's slab), room, duct, pipe, conduit, cable, fitting, equipment, fixture,
//! system. The ten system classifications are the trade's words; which value of IFC's each
//! will be exported as is in DECISIONS.md call 29.
//!
//! Heights. A storey's floor is at its `elevation` above the job's datum. A wall, a
//! floor, a room, a run and a placed thing each stand on the storey they are under in the
//! job, and a height entered on one is above that storey's floor. An opening's sill is
//! above the base of the wall it is in. A riser is a run whose two ends are one point on
//! the plan at two heights.

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

pub mod clash;
pub mod model;
pub mod network;
pub mod rooms;
pub mod runs;
pub mod walls;

pub use model::*;

/// A point in space: east, north, height above the datum.
pub type P3 = [f64; 3];

pub const K_LEVEL: &str = "pl::level";
pub const K_WALL: &str = "pl::wall";
pub const K_WALL_TYPE: &str = "pl::wall_type";
pub const K_OPENING: &str = "pl::opening";
pub const K_OPENING_TYPE: &str = "pl::opening_type";
pub const K_ITEM: &str = "pl::item";
pub const K_FLOOR: &str = "pl::floor";
pub const K_FLOOR_TYPE: &str = "pl::floor_type";
pub const K_SEGMENT: &str = "pl::segment";
pub const K_SEGMENT_TYPE: &str = "pl::segment_type";
pub const K_FITTING: &str = "pl::fitting";
pub const K_CONNECTORS: &str = "pl::connectors";
pub const K_ROOM: &str = "pl::room";
pub const K_PLACED: &str = "pl::placed";
pub const K_SYSTEM: &str = "pl::system";
pub const K_IN_SYSTEM: &str = "pl::in_system";

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

/// The classifications of system a kind of run goes on: a duct carries air, a pipe a
/// liquid or a gas, a conduit and a cable power. Empty for what is not a run: a fixture,
/// a piece of equipment and a fitting go on any system.
pub fn run_goes_on(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Duct => &["Supply Air", "Return Air", "Exhaust Air"],
        Kind::Pipe => &["Refrigerant", "Domestic Cold Water", "Domestic Hot Water", "Sanitary", "Vent"],
        Kind::Conduit | Kind::Cable => &["Power", "Lighting"],
        _ => &[],
    }
}

/// How far one wall's body lies in another's, in feet along it: none when the two bodies
/// do not overlap, or overlap only as walls that meet or cross do. Each wall is its centre
/// line and its thickness. A thickness that is not known is taken as three inches for
/// this test and no other: no wall in a house is thinner, and two lines an inch apart for
/// twenty feet are one wall drawn twice whatever its type says.
///
/// Two walls back to back, sharing a face, do not overlap. Two that run the same way
/// (within about three degrees) and overlap at all are one drawn over another. Two at an
/// angle are a corner, a T or a crossing, however sharp: they are in each other only
/// where they meet, and that is how walls are built.
fn bodies_overlap(a: [P2; 2], thick_a: f64, b: [P2; 2], thick_b: f64) -> Option<f64> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    const THINNEST_WALL_FT: f64 = 0.25;
    let known = |t: f64| if t > 0.0 { t } else { THINNEST_WALL_FT };
    let (thick_a, thick_b) = (known(thick_a), known(thick_b));
    // How near the two centre lines have to be for the bodies to be in each other.
    let reach = ((thick_a + thick_b) / 2.0 - tol).max(tol);
    // How much of `b`'s centre line is within `reach` of `a`'s, beside `a`'s length.
    let inside = |a: [P2; 2], b: [P2; 2]| -> f64 {
        let (dx, dy) = (a[1][0] - a[0][0], a[1][1] - a[0][1]);
        let len = dx.hypot(dy);
        if len <= tol {
            return 0.0;
        }
        let (ux, uy) = (dx / len, dy / len);
        let off = |p: P2| ux * (p[1] - a[0][1]) - uy * (p[0] - a[0][0]);
        let along = |p: P2| ux * (p[0] - a[0][0]) + uy * (p[1] - a[0][1]);
        // Along `b` from 0 to 1, each of those is a straight line: keep where it is
        // between its two limits.
        let (mut from, mut to) = (0.0_f64, 1.0_f64);
        for (f0, f1, low, high) in [(off(b[0]), off(b[1]), -reach, reach), (along(b[0]), along(b[1]), 0.0, len)] {
            let d = f1 - f0;
            if d.abs() < 1.0e-12 {
                if f0 < low || f0 > high {
                    return 0.0;
                }
                continue;
            }
            let (s0, s1) = ((low - f0) / d, (high - f0) / d);
            from = from.max(s0.min(s1));
            to = to.min(s0.max(s1));
        }
        (to - from).max(0.0) * (b[1][0] - b[0][0]).hypot(b[1][1] - b[0][1])
    };
    let shared = inside(a, b).max(inside(b, a));
    let (la, lb) = ((a[1][0] - a[0][0]).hypot(a[1][1] - a[0][1]), (b[1][0] - b[0][0]).hypot(b[1][1] - b[0][1]));
    if la <= tol || lb <= tol {
        return None;
    }
    let sine = ((a[1][0] - a[0][0]) * (b[1][1] - b[0][1]) - (a[1][1] - a[0][1]) * (b[1][0] - b[0][0])).abs() / (la * lb);
    (sine < 0.05 && shared > tol).then_some(shared)
}

/// `pl::in_system`: the system an element belongs to, by the system's path.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InSystem(pub Val<String>);

impl Kind {
    /// A straight piece with a section: a duct, a pipe, a conduit or a cable.
    pub fn is_segment(self) -> bool {
        matches!(self, Kind::Duct | Kind::Pipe | Kind::Conduit | Kind::Cable)
    }
    pub fn is_opening(self) -> bool {
        matches!(self, Kind::Door | Kind::Window)
    }
    /// A thing set at a place: equipment, a fixture, or a fitting.
    pub fn is_placed(self) -> bool {
        matches!(self, Kind::Equipment | Kind::Fixture | Kind::Furnishing | Kind::Fitting)
    }
    /// Whose work an element of this kind is when it belongs to no system.
    pub fn discipline(self) -> Discipline {
        match self {
            Kind::Duct => Discipline::Mechanical,
            Kind::Pipe | Kind::Fitting => Discipline::Plumbing,
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
            return Err(format!("{what}: is not a length a job can have: it is not a figure, or it is more than 1,000,000 ft"));
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
        return Err(format!("{what}: a coordinate is not a place a job can have: it is not a figure, or it is more than 1,000,000 ft either way"));
    }
    Ok(())
}

pub fn check_system(path: &str, kind: Kind, s: &SystemInfo) -> Result<(), String> {
    if kind != Kind::System {
        return Err(format!("{path}: {kind:#} cannot carry a system's description; only a system can"));
    }
    match CLASSIFICATIONS.iter().find(|(name, _)| *name == s.classification) {
        None => Err(format!("{path} system: {:?} is not a classification this program knows; it takes {}", s.classification, CLASSIFICATIONS.map(|c| c.0).join(", "))),
        Some((_, d)) if *d != s.discipline => Err(format!("{path} system: {} is {d:?} work, but the system says {:?}", s.classification, s.discipline)),
        Some(_) => Ok(()),
    }
}

pub fn check_in_system(path: &str, kind: Kind, s: &InSystem) -> Result<(), String> {
    if !(kind.is_segment() || kind.is_placed()) {
        return Err(format!("{path}: {kind:#} cannot belong to a system; a duct, a pipe, a conduit, a cable, equipment, a fixture or a fitting can"));
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
    pub(crate) segment_types: HashMap<String, SegmentType>,
    /// The items: things to place, by path.
    pub(crate) items: std::collections::HashSet<String>,
}

impl Tree {
    pub(crate) fn of(world: &mut World) -> Tree {
        let mut t = Tree { parents: HashMap::new(), kinds: HashMap::new(), names: HashMap::new(), levels: HashMap::new(), wall_types: HashMap::new(), floor_types: HashMap::new(), opening_types: HashMap::new(), segment_types: HashMap::new(), items: std::collections::HashSet::new() };
        let mut q = world.query::<(&Node, &Kind, Option<&Level>, Option<&WallType>, Option<&FloorType>, Option<&OpeningType>, Option<&SegmentType>, Has<Item>)>();
        for (node, kind, level, wall, floor, opening, segment, item) in q.iter(world) {
            let p = node.path.clone();
            t.parents.insert(p.clone(), node.parent.clone());
            t.kinds.insert(p.clone(), *kind);
            t.names.insert(p.clone(), node.name.clone());
            if let (Kind::Storey, Some(l)) = (kind, level) {
                t.levels.insert(p.clone(), l.clone());
            }
            if *kind == Kind::Type {
                if item {
                    t.items.insert(p.clone());
                }
                if let Some(c) = wall {
                    t.wall_types.insert(p.clone(), c.clone());
                }
                if let Some(c) = floor {
                    t.floor_types.insert(p.clone(), c.clone());
                }
                if let Some(c) = opening {
                    t.opening_types.insert(p.clone(), c.clone());
                }
                if let Some(c) = segment {
                    t.segment_types.insert(p.clone(), c.clone());
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
/// and a floor are under a storey, a run and a placed thing are under a storey too, a type is under the job and says what it is a type of,
/// a type named is a type of the right sort, a top level named is a storey, and a system
/// named is a system in the job.
pub fn check_whole(job: &mut Job) -> Result<(), String> {
    let tree = Tree::of(&mut job.world);
    let kinds = &tree.kinds;
    let classed: HashMap<String, String> = {
        let mut q = job.world.query::<(&Node, &SystemInfo)>();
        q.iter(&job.world).map(|(n, s)| (n.path.clone(), s.classification.clone())).collect()
    };
    // Each wall's storey, its line, and how high it stands where that is entered outright.
    // Each wall with a line: its storey, its centre line, its thickness (none when its
    // type's is not known) and the heights it stands between, where both are known.
    let mut standing: Vec<(String, String, [P2; 2], f64, Option<(f64, f64)>)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, Option<&InSystem>, Option<&Wall>, Option<&Opening>, Option<&Floor>, Option<&Segment>)>();
    for (node, kind, in_system, wall, opening, floor, segment) in q.iter(&job.world) {
        let me = &node.path;
        let parent = node.parent.as_ref().and_then(|p| kinds.get(p)).copied();
        if kind.is_opening() && parent != Some(Kind::Wall) {
            return Err(format!("{me}: {kind:#} is an opening in a wall and has to be under one"));
        }
        if *kind == Kind::Room && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a room is a space on a level and has to be directly under one"));
        }
        if matches!(kind, Kind::Wall | Kind::Floor) && parent != Some(Kind::Storey) {
            return Err(format!("{me}: {kind:#} stands on a level and has to be under one"));
        }
        if *kind == Kind::Drawing && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a drawing is a plan of a level and has to be directly under one"));
        }
        if *kind == Kind::Prior && parent != Some(Kind::Job) {
            return Err(format!("{me}: a prior estimate is set against the whole job and has to be directly under it"));
        }
        if *kind == Kind::Storey && parent != Some(Kind::Job) {
            return Err(format!("{me}: a level is a floor of the building and has to be directly under the job, not under {}", node.parent.as_deref().unwrap_or("nothing")));
        }
        if *kind == Kind::Type {
            if parent != Some(Kind::Job) {
                return Err(format!("{me}: a type belongs to the whole job and has to be directly under it, not under {}", node.parent.as_deref().unwrap_or("nothing")));
            }
            let sorts = [tree.wall_types.contains_key(me), tree.floor_types.contains_key(me), tree.opening_types.contains_key(me), tree.segment_types.contains_key(me), tree.items.contains(me)].into_iter().filter(|x| *x).count();
            if sorts != 1 {
                return Err(format!("{me}: a type is one sort of type (of a wall, a floor, a door or a window, or a run) or an item to place; this one carries {sorts}"));
            }
        }
        if let Some(named) = in_system.and_then(|s| s.0.value.as_ref()) {
            if kinds.get(named) != Some(&Kind::System) {
                return Err(format!("{me}: says it belongs to {named:?}, which is not a system in this job"));
            }
            // A duct does not carry water. A fixture, equipment or a fitting can be on any system.
            let goes_on = run_goes_on(*kind);
            if let Some(class) = classed.get(named).filter(|c| !goes_on.is_empty() && !goes_on.contains(&c.as_str())) {
                return Err(format!("{me}: a {} is on {named}, which is a {class} system; a {} goes on: {}", kind.stem(), kind.stem(), goes_on.join(", ")));
            }
        }
        if let Some(w) = wall {
            if let (Some(line), Some(storey)) = (w.line_ft(), node.parent.as_ref()) {
                // Its body, not the line it was drawn on: the centre line lies to the
                // left of the drawn line by what its `location` says.
                let (thick, centre) = match wall_section(&tree, w) {
                    Ok((thickness, layers, _)) => {
                        let off = walls::centre(thickness, &layers, w.location);
                        let length = geom::dist(line[0], line[1]).max(1.0e-12);
                        let left = [-(line[1][1] - line[0][1]) / length * off, (line[1][0] - line[0][0]) / length * off];
                        (thickness, [[line[0][0] + left[0], line[0][1] + left[1]], [line[1][0] + left[0], line[1][1] + left[1]]])
                    }
                    Err(_) => (0.0, line),
                };
                // Where it stands, however its top is said: by a height or by a level.
                let heights = wall_heights(&tree, me, w);
                let stands = heights.base.ok().zip(heights.height.ok()).map(|(b, h)| (b, b + h));
                standing.push((me.clone(), storey.clone(), centre, thick, stands));
            }
            if let Some(p) = &w.type_name.value {
                if !tree.wall_types.contains_key(p) {
                    return Err(format!("{me}: its type is {p:?}, which is not a wall type in this job"));
                }
            }
            if let Some(p) = w.top_level.as_ref().and_then(|t| t.value.as_ref()) {
                if kinds.get(p) != Some(&Kind::Storey) {
                    return Err(format!("{me}: its top level is {p:?}, which is not a level in this job"));
                }
            }
        }
        if let Some(p) = opening.and_then(|o| o.type_name.value.as_ref()) {
            match tree.opening_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a door or window type in this job")),
                Some(t) if t.category != *kind => return Err(format!("{me}: is {kind:#}, but its type, {p}, is {:#}'s", t.category)),
                Some(_) => {}
            }
        }
        if let Some(p) = floor.and_then(|f| f.type_name.value.as_ref()) {
            if !tree.floor_types.contains_key(p) {
                return Err(format!("{me}: its type is {p:?}, which is not a floor type in this job"));
            }
        }
        if let Some(p) = segment.and_then(|r| r.type_name.value.as_ref()) {
            match tree.segment_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a run type in this job")),
                Some(t) if t.category != *kind => return Err(format!("{me}: is {kind:#}, but its type, {p}, is {:#}'s", t.category)),
                Some(_) => {}
            }
        }
        // A run's heights and a placed thing's are above the storey it is under.
        if (kind.is_segment() || kind.is_placed()) && tree.storey_of(me).is_none() {
            return Err(format!("{me}: {kind:#} is set at a height above a level and has to be under one"));
        }
    }
    // One wall drawn over another on a storey is the same wall counted twice. Two that
    // stand one over the other (a wall on a knee wall) are two walls.
    standing.sort_by(|x, y| x.0.cmp(&y.0));
    for (i, (a, storey, line, thick, stands)) in standing.iter().enumerate() {
        for (b, other, its, as_thick, theirs) in &standing[i + 1..] {
            let tol = crate::check::FIT_TOLERANCE_FT;
            // Where each stands, above the datum: walls on two storeys are compared too
            // (a wall held up to the roof and another on the floor above, on one line).
            // Where that is not known, walls of one storey are taken to stand together
            // and walls of two storeys apart.
            match (stands, theirs) {
                (Some((lo, hi)), Some((lo2, hi2))) => {
                    if *hi <= lo2 + tol || *hi2 <= lo + tol {
                        continue;
                    }
                }
                _ if storey != other => continue,
                _ => {}
            }
            if let Some(shared) = bodies_overlap(*line, *thick, *its, *as_thick) {
                return Err(format!("{b}: lies along {a} for {} ft: one wall drawn over another would be counted twice", view::figure_shown(shared, 2)));
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

/// One opening as its wall sees it: where it starts and ends along the wall, its sill and
/// its head, in feet, with the worst status among them. None when a value is missing.
struct Hole {
    path: String,
    span: Option<[f64; 4]>,
    area: Option<f64>,
    /// The worst status among everything about it: its type, its size, where it is.
    status: Status,
    /// The worst status among what its area alone rests on: its type and its size.
    area_status: Status,
    /// What its area was worked out from.
    inputs: Vec<String>,
}

/// A door or window worked out against its type: the hole it makes, and its type's width
/// and height, or why it has none.
fn hole_of(tree: &Tree, me: &str, o: &Opening) -> (Hole, Result<(Val<Len>, Val<Len>), String>) {
    let mut inputs = vec![format!("{me}#{K_OPENING}")];
    match typed(&tree.opening_types, &o.type_name, "a door or window") {
        Err(why) => (Hole { path: me.into(), span: None, area: None, status: Status::Unknown, area_status: Status::Unknown, inputs }, Err(why)),
        Ok((type_path, t)) => {
            inputs.push(format!("{type_path}#{K_OPENING_TYPE}"));
            let (w, h) = (t.width.feet(), t.height.feet());
            let span = o.offset.feet().zip(w).zip(o.sill.feet().zip(h)).map(|((at, w), (s, h))| [at - w / 2.0, at + w / 2.0, s, s + h]);
            let status = [o.type_name.status, t.width.status, t.height.status, o.offset.status, o.sill.status].into_iter().max().unwrap_or(Status::Unknown);
            let area = w.zip(h).map(|(w, h)| w * h);
            let area_status = o.type_name.status.max(t.width.status).max(t.height.status);
            (Hole { path: me.into(), span, area, status, area_status, inputs }, Ok((t.width.clone(), t.height.clone())))
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
    /// The worst status among everything its base and its top rest on: its storey's
    /// elevation and its base offset as well as what its height does. It is what "does
    /// this wall stand at such a height?" is no surer than.
    pub(crate) span: Status,
    /// What its base alone rests on: its storey's elevation and its base offset. A wall
    /// whose base is Known and whose top is not may reach higher or lower, and starts
    /// where it starts.
    pub(crate) base_status: Status,
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
        (None, _) => Err("it is under no level".into()),
        (Some(_), None) => Err("its level has no level entered".into()),
        (Some(_), Some(l)) => l.elevation.feet().ok_or_else(|| "its level's elevation is not known".to_string()),
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
                let l = tree.levels.get(p).ok_or_else(|| format!("its top level, {p}, is not a level with a level in this job"))?;
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
            f.push_str(" − elevation of its level");
            if w.base_offset.is_some() {
                f.push_str(" − base_offset");
            }
            (height, status, f)
        }
        (None, None) => (Err("it has no top level and no height".into()), Status::Unknown, String::new()),
    };
    let height = height.and_then(|h| if h > 0.0 { Ok(h) } else { Err(format!("its top is {} ft above its base; a wall's top is above its base", view::figure_shown(h, 2))) });
    let own_status = own.map(|l| l.elevation.status).unwrap_or(Status::Unknown);
    let span = status.max(own_status).max(base_status);
    WallHeights { base, height, status, span, base_status: own_status.max(base_status), formula, inputs }
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
    // In the order of their paths, whatever order the world holds them in: a wall's
    // list of openings, and so what its figures cite, is the same every time.
    let mut q = world.query::<(&Node, &Kind, Option<&Opening>)>();
    let mut openings: Vec<(&Node, &Kind, Option<&Opening>)> = q.iter(world).collect();
    openings.sort_by_key(|(n, ..)| path_number(&n.path));
    for (node, kind, opening) in openings {
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
                Hole { path: me.clone(), span: None, area: None, status: Status::Unknown, area_status: Status::Unknown, inputs: input.to_vec() }
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

    let mut q = world.query::<(&Node, &Kind, Option<&Wall>, Option<&Floor>, Option<&Segment>)>();
    for (node, kind, wall, floor, segment) in q.iter(world) {
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
                let f_open = if mine.is_empty() { "the openings in this wall (it has none)".to_string() } else { if mine.len() == 1 { "the opening_area of the one opening in this wall".to_string() } else { format!("the opening_area of each of the {} openings in this wall, added up", mine.len()) } };
                let total = mine.iter().map(|h| h.area).sum::<Option<f64>>();
                // What the openings come to is as sure as their sizes. Where each one is
                // matters to the net, not to this.
                let mut statuses: Vec<Status> = mine.iter().map(|h| if h.area.is_some() { h.area_status } else { Status::Unknown }).collect();
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
                // Every opening that does not fit is named, not the first alone.
                let mut faults: Vec<String> = Vec::new();
                // An opening that has a size and no place cannot be checked for fit.
                for h in mine.iter().filter(|h| h.area.is_some() && h.span.is_none()) {
                    faults.push(format!("where {} is in the wall is not known, so whether it fits cannot be checked", h.path));
                }
                if let (Some(l), Ok(h)) = (length, &heights.height) {
                    for (i, hole) in mine.iter().enumerate() {
                        let Some([from, to, sill, head]) = hole.span else { continue };
                        // It fits its wall from end to end (BIMCraft `model.rs:6678`).
                        if from < -crate::check::FIT_TOLERANCE_FT {
                            faults.push(format!("{} starts {} ft before the start of its wall", hole.path, view::figure_shown(-from, 2)));
                        } else if to > l + crate::check::FIT_TOLERANCE_FT {
                            faults.push(format!("{} ends {} ft along a wall {} ft long", hole.path, view::figure_shown(to, 2), view::figure_shown(l, 2)));
                        } else if head > h + crate::check::FIT_TOLERANCE_FT {
                            faults.push(format!("{} reaches {} ft up a wall {} ft tall", hole.path, view::figure_shown(head, 2), view::figure_shown(*h, 2)));
                        } else if let Some(other) = mine[..i].iter().find(|o| o.span.is_some_and(|s| from < s[1] && s[0] < to && sill < s[3] && s[2] < head)) {
                            faults.push(format!("{} and {} lie over each other", other.path, hole.path));
                        }
                    }
                }
                // An opening whose place is not known still has its size: the openings' area
                // stands, and it is the net that cannot be had. One that is known not to fit
                // (past an end, over the top, over another) is another matter: the sum of
                // the openings is then not what is cut out of this wall, and is not given.
                let misfit: Vec<&String> = faults.iter().filter(|f| !f.starts_with("where ")).collect();
                let open = match (misfit.is_empty(), open) {
                    (false, o) if o.value.is_some() => Derived::refused("SF", &f_open, &o.inputs, &misfit.iter().map(|f| f.as_str()).collect::<Vec<_>>().join("; ")),
                    (_, o) => o,
                };
                let fault = (!faults.is_empty()).then(|| faults.join("; "));
                let f_net = "face_area − opening_area";
                let net = match (fault, face.value, open.value) {
                    (Some(why), _, _) => Derived::refused("SF", f_net, &inputs, &why),
                    (None, Some(a), Some(b)) => {
                        // The net rests on where each opening is as well as on its size.
                        let mut s: Vec<Status> = mine.iter().map(|h| h.status).collect();
                        s.extend([face.status, open.status]);
                        Derived::new(Some(a - b), "SF", &s, f_net, &inputs)
                    }
                    (None, None, _) => Derived::refused("SF", f_net, &inputs, if face.note.is_empty() { "the wall has no face area" } else { &face.note }),
                    (None, _, None) => Derived::refused("SF", f_net, &inputs, "an opening in this wall has no area"),
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
            k if k.is_segment() => {
                let input = [format!("{me}#{K_SEGMENT}")];
                let Some(r) = segment else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no line entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                // Its true length: along its line on the plan when it is level, and
                // through its rise when its two ends are at different heights
                // (BIMCraft `runs.rs:107`).
                let line = r.line_ft();
                let plan = line.map(|[a, b]| geom::dist(a, b));
                let inches = if r.line.unit == units::IN { " ÷ 12 (entered in inches)" } else { "" };
                let (length, statuses, f_len) = match &r.end_offset {
                    None => (plan, vec![r.line.status], format!("length of its line on the plan{inches}")),
                    Some(e) => (
                        plan.zip(r.offsets_ft()).map(|(p, (z0, z1))| p.hypot(z1 - z0)),
                        vec![r.line.status, r.offset.status, e.status],
                        format!("√(length on the plan{inches}² + ({} − {})²)", feet_text("end_offset", e), feet_text("offset", &r.offset)),
                    ),
                };
                let len_d = if r.line.value.is_some() && line.is_none() { Derived::refused("LF", &f_len, &input, &format!("its line {NOT_CONVERTED}")) } else { Derived::new(length, "LF", &statuses, &f_len, &input) };
                if *k == Kind::Duct {
                    let skin = match typed(&tree.segment_types, &r.type_name, "a run") {
                        Err(why) => Derived::refused("SF", "true_length × the distance round the duct", &input, &why),
                        Ok((type_path, t)) => {
                            let inputs = [input[0].clone(), format!("{type_path}#{K_SEGMENT_TYPE}")];
                            let f = match (t.round, &t.depth) {
                                (true, _) => format!("true_length × π × {}", feet_text("its type's diameter", &t.width)),
                                (false, Some(d)) => format!("true_length × 2 × ({} + {})", feet_text("its type's width", &t.width), feet_text("its type's depth", d)),
                                (false, None) => "true_length × the distance round the duct".to_string(),
                            };
                            match len_d.value {
                                None => Derived::refused("SF", &f, &inputs, "the duct has no length"),
                                Some(l) => Derived::new(t.perimeter_ft().map(|p| l * p), "SF", &[len_d.status, r.type_name.status, t.status()], &f, &inputs),
                            }
                        }
                    };
                    m.insert("surface_area".into(), skin);
                }
                m.insert("true_length".into(), len_d);
            }
            _ => {}
        }
        if !m.is_empty() {
            out.entry(me.clone()).or_default().extend(m);
        }
    }
    room_measures(world, &tree, &mut out, &mut HashMap::new());
    out
}

/// Each room's outline at the faces of the walls round its point, in feet on the plan,
/// counter-clockwise: the same outline its floor area is the area of. A room no walls
/// close round has none and is not in the map.
pub fn room_outlines(job: &mut Job) -> HashMap<String, Vec<P2>> {
    let tree = Tree::of(&mut job.world);
    let mut outlines = HashMap::new();
    room_measures(&mut job.world, &tree, &mut HashMap::new(), &mut outlines);
    outlines
}

/// The number at the end of a path: paths are numbered in the order elements were added,
/// and a number is never given twice.
fn path_number(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

/// A wall as the walls round it see it on the plan; the worst status among what its line
/// and its thickness were worked out from; and the worst among what its base and top rest
/// on. Or why it cannot be placed.
fn plan_wall(tree: &Tree, me: &str, w: &Wall) -> Result<(walls::PlanWall, Status, Status, Status), String> {
    use crate::skeleton::P2 as V;
    let [a, b] = w.line_ft().ok_or("its line is not known")?;
    let heights = wall_heights(tree, me, w);
    let base = heights.base.clone()?;
    let height = heights.height.clone()?;
    let (thickness, layers, section_status) = wall_section(tree, w)?;
    let (ext, int) = walls::wall_faces(thickness, &layers, w.location);
    Ok((walls::PlanWall { start: V(a[0], a[1]), end: V(b[0], b[1]), ext, int, base, top: base + height, join: w.join }, w.line.status.max(section_status), heights.span, heights.base_status))
}

/// The outline a floor on a storey has when none is given, and what else the walls
/// enclose there.
#[derive(Clone, Debug, PartialEq)]
pub struct FloorOutline {
    /// The outer outline of the largest piece the storey's walls make, along their
    /// centre lines, counter-clockwise from its south-west corner, in feet.
    pub boundary: Vec<P2>,
    /// Loops of walls standing free inside it, joined to none of the outer walls and
    /// with no room in them: courtyards, which a floor would be cut round.
    pub courtyards: usize,
    /// Other pieces the storey's walls enclose, apart from this one.
    pub others: usize,
    /// The worst status among what the walls' lines and thicknesses rest on.
    pub status: Status,
}

/// The outline `floor` gives a floor on `storey` by default (BIMCraft `floor_outline`,
/// `commands.rs:2467`): along the centre lines of the walls that bound its rooms, the
/// outer outline of the largest piece they make. None when no walls enclose an area.
/// A wall of the storey that cannot be placed is why not.
pub fn floor_outline(job: &mut Job, storey: &str) -> Result<Option<FloorOutline>, String> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let mut q = job.world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Wall && n.parent.as_deref() == Some(storey)).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    let (mut placed, mut bounding, mut status) = (Vec::new(), Vec::new(), Status::Known);
    for (n, w) in all {
        let w = w.ok_or_else(|| format!("{} has no wall line entered, so what the walls of this level enclose is not known", tree.called(&n.path)))?;
        let (wall, shape, _, _) = plan_wall(&tree, &n.path, w).map_err(|why| format!("{} cannot be placed ({why}), so what the walls of this level enclose is not known", tree.called(&n.path)))?;
        if w.room_bounding {
            bounding.push(placed.len());
            status = status.max(shape);
        }
        placed.push(wall);
    }
    let lines: Vec<(V, V)> = bounding.iter().map(|&i| walls::centre_line(&placed, i, tol)).map(|[a, b]| (a, b)).collect();
    // Each piece's outline, clockwise, the largest first.
    let mut pieces: Vec<Vec<V>> = rooms::faces(&lines, tol).into_iter().filter(|f| rooms::area(f) < -tol * tol).collect();
    pieces.sort_by(|a, b| rooms::area(a).total_cmp(&rooms::area(b)));
    let Some((outer, inner)) = pieces.split_first() else { return Ok(None) };
    // The loops standing free inside the outline: the other pieces, inside it.
    let loops: Vec<&Vec<V>> = inner.iter().filter(|f| f.iter().all(|q| rooms::contains(outer, *q))).collect();
    let nested = |a: usize| (0..loops.len()).any(|b| b != a && loops[a].iter().all(|q| rooms::contains(loops[b], *q)));
    let around = |p: V| (0..loops.len()).filter(|&a| rooms::contains(loops[a], p)).max_by(|&a, &b| rooms::area(loops[a]).total_cmp(&rooms::area(loops[b])));
    let mut rq = job.world.query::<(&Node, &Room)>();
    let indoors: Vec<usize> = rq.iter(&job.world).filter(|(n, _)| n.parent.as_deref() == Some(storey)).filter_map(|(_, r)| r.point_ft()).filter_map(|p| around(V(p[0], p[1]))).collect();
    let courtyards = (0..loops.len()).filter(|&a| !nested(a) && !indoors.contains(&a)).count();
    // Counter-clockwise, its corners only, kept to a ten-thousandth of a foot, from its
    // south-west corner.
    let ccw: Vec<V> = outer.iter().rev().copied().collect();
    let mut ring: Vec<P2> = rooms::corners(&ccw).into_iter().map(|p| [(p.0 * 1.0e4).round() / 1.0e4 + 0.0, (p.1 * 1.0e4).round() / 1.0e4 + 0.0]).collect();
    if let Some(first) = ring.iter().enumerate().min_by(|(_, a), (_, b)| (a[1], a[0]).partial_cmp(&(b[1], b[0])).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i) {
        ring.rotate_left(first);
    }
    Ok(Some(FloorOutline { boundary: ring, courtyards, others: inner.len() - loops.len(), status }))
}

/// What a level's walls enclose, for telling an outside face from an inside one: the
/// outline of every separate piece the walls make (a house, and a garage that stands
/// apart from it), and the courtyards inside them (a loop of walls standing free inside
/// a piece, with no room in it). Each ring is in feet. A place is indoors when it is
/// inside a piece and in none of its courtyards. Walls that cannot be placed are why not.
pub struct Enclosed {
    pub pieces: Vec<Vec<P2>>,
    pub courtyards: Vec<Vec<P2>>,
}

impl Enclosed {
    pub fn indoors(&self, p: P2) -> bool {
        let within = |ring: &Vec<P2>| {
            let mut odd = false;
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
                    odd = !odd;
                }
            }
            odd
        };
        // In more pieces than courtyards: a shed standing in a courtyard is indoors again.
        self.pieces.iter().filter(|r| within(r)).count() > self.courtyards.iter().filter(|r| within(r)).count()
    }
}

/// See [`Enclosed`]. The pieces and loops are found as `floor_outline` finds them, and
/// a loop is a courtyard by the same test: it stands free inside a piece, inside no
/// other loop, and no room of the level is in it.
pub fn enclosed(job: &mut Job, storey: &str) -> Result<Enclosed, String> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let mut q = job.world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Wall && n.parent.as_deref() == Some(storey)).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    let (mut placed, mut bounding) = (Vec::new(), Vec::new());
    for (n, w) in all {
        let w = w.ok_or_else(|| format!("{} has no wall line entered", tree.called(&n.path)))?;
        let (wall, _, _, _) = plan_wall(&tree, &n.path, w).map_err(|why| format!("{} cannot be placed ({why})", tree.called(&n.path)))?;
        if w.room_bounding {
            bounding.push(placed.len());
        }
        placed.push(wall);
    }
    let lines: Vec<(V, V)> = bounding.iter().map(|&i| walls::centre_line(&placed, i, tol)).map(|[a, b]| (a, b)).collect();
    let outlines: Vec<Vec<V>> = rooms::faces(&lines, tol).into_iter().filter(|f| rooms::area(f) < -tol * tol).collect();
    // How many other outlines each lies wholly inside: none, it is a piece; one, it is a
    // loop standing free inside a piece.
    let inside_of = |a: usize| (0..outlines.len()).filter(|&b| b != a && outlines[a].iter().all(|q| rooms::contains(&outlines[b], *q))).count();
    let depth: Vec<usize> = (0..outlines.len()).map(inside_of).collect();
    let mut rq = job.world.query::<(&Node, &Room)>();
    let rooms_at: Vec<V> = rq.iter(&job.world).filter(|(n, _)| n.parent.as_deref() == Some(storey)).filter_map(|(_, r)| r.point_ft()).map(|p| V(p[0], p[1])).collect();
    let ring = |f: &Vec<V>| -> Vec<P2> { f.iter().map(|p| [p.0, p.1]).collect() };
    let mut out = Enclosed { pieces: Vec::new(), courtyards: Vec::new() };
    for (i, f) in outlines.iter().enumerate() {
        // A loop standing free inside a piece with no room in it is a courtyard, as the
        // floor's outline takes it; what stands free inside a courtyard is a piece again.
        let roomless = !rooms_at.iter().any(|p| rooms::contains(f, *p));
        match depth[i] {
            0 => out.pieces.push(ring(f)),
            1 if roomless => out.courtyards.push(ring(f)),
            2 if outlines.iter().enumerate().any(|(j, o)| j != i && depth[j] == 1 && f.iter().all(|q| rooms::contains(o, *q)) && !rooms_at.iter().any(|p| rooms::contains(o, *p))) => out.pieces.push(ring(f)),
            _ => {}
        }
    }
    Ok(out)
}

/// A wall of the job as the rooms see it.
struct RoomWall {
    path: String,
    wall: walls::PlanWall,
    /// What its line and thickness rest on.
    shape: Status,
    /// What its base and top rest on.
    span: Status,
    /// What its base alone rests on.
    base: Status,
    /// Whether it bounds rooms (`room_bounding`).
    bounds: bool,
}

/// What one room comes to against one set of walls: the floor inside the faces and the
/// distance round, and the area to centre lines, each with the walls it is measured to;
/// or why not.
#[derive(Clone, PartialEq)]
struct RoomFigures {
    finish: Result<(f64, f64, Vec<usize>), rooms::Unbound>,
    centre: Result<(f64, Vec<usize>), rooms::Unbound>,
    in_wall: bool,
    /// The corners of the room at its walls' faces, when it has any: where it is.
    corners: Vec<crate::skeleton::P2>,
    /// The room's outline at the faces of its walls, counter-clockwise, without what
    /// stands free inside it; empty when no walls close round its point.
    outline: Vec<crate::skeleton::P2>,
}

/// Every room of one storey measured against `walls`, of which those at `bounding` bound
/// the storey's rooms. Each of `walls` can join the others where they meet (BIMCraft
/// `rooms.rs:62` and `walls.rs:238`: a bounding wall's outline is cut against every wall
/// it meets, bounding or not).
fn rooms_among(walls: &[walls::PlanWall], bounding: &[usize], points: &[crate::skeleton::P2], tol: f64) -> Vec<RoomFigures> {
    use crate::skeleton::P2 as V;
    use rooms::{Plan, RoomArea};
    let (centre, finish) = (Plan::of_walls(walls, bounding, RoomArea::Centre, tol), Plan::of_walls(walls, bounding, RoomArea::Finish, tol));
    let centre_lines: Vec<Vec<(V, V)>> = bounding.iter().map(|&i| walls::centre_line(walls, i, tol)).map(|[a, b]| vec![(a, b)]).collect();
    let outlines: Vec<Vec<V>> = bounding.iter().map(|&i| walls::outline(walls, i, tol)).collect();
    let outline_edges: Vec<Vec<(V, V)>> = outlines.iter().map(|ring| ring.iter().copied().zip(ring.iter().copied().cycle().skip(1)).collect()).collect();
    // The walls a region is measured to: those with a line along one of its edges.
    let touching = |g: &rooms::Region, lines: &[Vec<(V, V)>]| -> Vec<usize> {
        let mids: Vec<V> = std::iter::once(&g.outer).chain(g.holes.iter()).flat_map(|ring| ring.iter().zip(ring.iter().cycle().skip(1)).map(|(a, b)| (*a + *b) * 0.5)).collect();
        (0..bounding.len())
            .filter(|&k| {
                lines[k].iter().any(|(a, b)| {
                    mids.iter().any(|m| {
                        let d = *b - *a;
                        let t = if d.dot(d) > 0.0 { ((*m - *a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
                        m.dist(*a + d * t) <= tol
                    })
                })
            })
            .map(|k| bounding[k])
            .collect()
    };
    (0..points.len())
        .map(|i| RoomFigures {
            corners: finish.bound(points, i, tol).map(|g| std::iter::once(&g.outer).chain(g.holes.iter()).flatten().copied().collect()).unwrap_or_default(),
            outline: finish.bound(points, i, tol).map(|g| g.outer).unwrap_or_default(),
            finish: finish.bound(points, i, tol).map(|g| (g.area(), g.perimeter(), touching(&g, &outline_edges))),
            centre: centre.bound(points, i, tol).map(|g| (g.area(), touching(&g, &centre_lines))),
            in_wall: !points[i].0.is_nan() && outlines.iter().any(|ring| rooms::contains(ring, points[i])),
        })
        .collect()
}

/// Each room's figures: the floor inside the faces of the walls round its point, the area
/// to those walls' centre lines, and how far it is round the room at their faces.
///
/// A room is found among the walls that stand at its storey's computation height
/// (BIMCraft `Model::bounding`, `model.rs:3932`, and `Model::spans`, `model.rs:3960`) and
/// bound rooms. A wall under the room's own storey that cannot be placed leaves every
/// room on that storey with no figure: what bounds them is not known. A figure is no
/// surer than the walls it is measured to. And where a wall's base or top rests on
/// something Assumed, each room is measured again with that wall taken away and with it
/// standing full height: a room that comes out differently either way is no surer than
/// that wall's heights, and says so.
fn room_measures(world: &mut World, tree: &Tree, out: &mut HashMap<String, BTreeMap<String, Derived>>, outlines: &mut HashMap<String, Vec<P2>>) {
    use crate::skeleton::P2 as V;
    use rooms::Unbound;
    let tol = crate::check::FIT_TOLERANCE_FT;
    // Every wall of the job: placed, or why not. A wall with nothing entered on it is a
    // wall that cannot be placed, not a wall that is not there.
    let mut placed: Vec<RoomWall> = Vec::new();
    let mut unplaced: Vec<(String, String)> = Vec::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(world).filter(|(_, k, _)| **k == Kind::Wall).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    for (n, w) in all {
        match w.map(|w| (w, plan_wall(tree, &n.path, w))) {
            None => unplaced.push((n.path.clone(), "it has no wall line entered".into())),
            Some((w, Ok((wall, shape, span, base)))) => placed.push(RoomWall { path: n.path.clone(), wall, shape, span, base, bounds: w.room_bounding }),
            Some((_, Err(why))) => unplaced.push((n.path.clone(), why)),
        }
    }
    // The rooms, storey by storey, in the order they were added: the first to be placed
    // in a space has it.
    let mut q = world.query::<(&Node, &Kind, Option<&Room>)>();
    let mut by_storey: BTreeMap<String, Vec<(String, Option<Room>)>> = BTreeMap::new();
    for (n, kind, room) in q.iter(world) {
        if *kind == Kind::Room {
            by_storey.entry(n.parent.clone().unwrap_or_default()).or_default().push((n.path.clone(), room.cloned()));
        }
    }
    const F_FLOOR: &str = "the floor inside the faces of the walls round its point";
    const F_GROSS: &str = "the area to the center lines of the walls round its point";
    const F_ROUND: &str = "the distance round the room at the faces of its walls";
    for (storey, mut list) in by_storey {
        list.sort_by_key(|(path, _)| path_number(path));
        let refuse_all = |out: &mut HashMap<String, BTreeMap<String, Derived>>, path: &str, inputs: &[String], why: &str| {
            let m = out.entry(path.to_string()).or_default();
            m.insert("floor_area".into(), Derived::refused("SF", F_FLOOR, inputs, why));
            m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, inputs, why));
            m.insert("perimeter".into(), Derived::refused("LF", F_ROUND, inputs, why));
        };
        // Where the storey's rooms find their walls.
        let level = tree.levels.get(&storey);
        let cut: Result<(f64, Status), String> = match level {
            None => Err("it is under no level with a level".into()),
            Some(l) => match (l.elevation.feet(), &l.computation_height) {
                (None, _) => Err("its level's elevation is not known".into()),
                (Some(e), None) => Ok((e + COMPUTATION_HEIGHT_FT, l.elevation.status)),
                (Some(e), Some(c)) => c.feet().map(|up| (e + up, l.elevation.status.max(c.status))).ok_or_else(|| "the height its level's rooms find their walls at is not known".to_string()),
            },
        };
        let stuck = unplaced.iter().find(|(path, _)| tree.storey_of(path) == Some(storey.as_str()));
        let (cut, cut_status) = match (cut, stuck) {
            (Err(why), _) => {
                for (path, _) in &list {
                    refuse_all(out, path, &[format!("{path}#{K_ROOM}")], &why);
                }
                continue;
            }
            (Ok(_), Some((wall, why))) => {
                for (path, _) in &list {
                    refuse_all(out, path, &[format!("{path}#{K_ROOM}"), format!("{wall}#{K_WALL}")], &format!("{} cannot be placed ({why}), so what bounds the rooms of this level is not known", tree.called(wall)));
                }
                continue;
            }
            (Ok(c), None) => c,
        };
        // A room with no point entered, or a point nobody has, still takes its place in
        // the order: it is given a point no region can hold.
        let nowhere = V(f64::NAN, f64::NAN);
        let points: Vec<V> = list.iter().map(|(_, r)| r.as_ref().and_then(|r| r.point_ft()).map(|p| V(p[0], p[1])).unwrap_or(nowhere)).collect();
        // The walls as they stand, and which of them bound this storey's rooms: those that
        // bound rooms and span the height the rooms are found at.
        let plan_walls: Vec<walls::PlanWall> = placed.iter().map(|w| w.wall.clone()).collect();
        let spans = |w: &walls::PlanWall| w.base <= cut && cut < w.top;
        let bounding: Vec<usize> = (0..placed.len()).filter(|&i| placed[i].bounds && spans(&placed[i].wall)).collect();
        let as_entered = rooms_among(&plan_walls, &bounding, &points, tol);
        for (i, (path, _)) in list.iter().enumerate() {
            if as_entered[i].outline.len() >= 3 {
                outlines.insert(path.clone(), as_entered[i].outline.iter().map(|p| [p.0, p.1]).collect());
            }
        }
        // Each wall whose base or top rests on something less than Known, tried both other
        // ways: not there at all, and standing through every height. A room that comes out
        // differently is no surer than that wall's heights.
        let mut rests_on: Vec<Vec<usize>> = vec![Vec::new(); list.len()];
        for k in (0..placed.len()).filter(|&k| placed[k].span != Status::Known) {
            let without: Vec<walls::PlanWall> = plan_walls.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, w)| w.clone()).collect();
            let shifted: Vec<usize> = bounding.iter().filter(|&&i| i != k).map(|&i| if i > k { i - 1 } else { i }).collect();
            let gone = rooms_among(&without, &shifted, &points, tol);
            // Standing as tall as it could: its top anywhere above, and its base anywhere
            // below only if its base is itself unsure. A wall that starts at a Known
            // height does not reach below it whatever its top turns out to be.
            let mut tall = plan_walls.clone();
            if placed[k].base != Status::Known {
                tall[k].base = f64::NEG_INFINITY;
            }
            tall[k].top = f64::INFINITY;
            let mut with: Vec<usize> = bounding.clone();
            if placed[k].bounds && spans(&tall[k]) && !with.contains(&k) {
                with.push(k);
                with.sort_unstable();
            }
            let standing = rooms_among(&tall, &with, &points, tol);
            // Compared by what a room comes to, not by which walls are listed (taking a
            // wall away renumbers the rest).
            let same = |a: &RoomFigures, b: &RoomFigures| {
                let near = |x: f64, y: f64| (x - y).abs() <= 1.0e-9;
                let finish = match (&a.finish, &b.finish) {
                    (Ok(x), Ok(y)) => near(x.0, y.0) && near(x.1, y.1),
                    (Err(x), Err(y)) => std::mem::discriminant(x) == std::mem::discriminant(y),
                    _ => false,
                };
                let centre = match (&a.centre, &b.centre) {
                    (Ok(x), Ok(y)) => near(x.0, y.0),
                    (Err(x), Err(y)) => std::mem::discriminant(x) == std::mem::discriminant(y),
                    _ => false,
                };
                finish && centre && a.in_wall == b.in_wall
            };
            for i in 0..list.len() {
                if !same(&as_entered[i], &gone[i]) || !same(&as_entered[i], &standing[i]) {
                    rests_on[i].push(k);
                }
            }
        }
        for (i, (path, room)) in list.iter().enumerate() {
            let input = format!("{path}#{K_ROOM}");
            let Some(room) = room else {
                refuse_all(out, path, &[input], "no point entered");
                continue;
            };
            if points[i].0.is_nan() {
                refuse_all(out, path, &[input], if room.point.value.is_some() { "its point is in a unit this program does not convert" } else { "its point is not known" });
                continue;
            }
            let found = &as_entered[i];
            let why = |u: Unbound| match u {
                // A point in the body of a wall is in no room, at its faces or at its
                // centre line: which side of the wall was meant is not something to guess.
                _ if found.in_wall => "its point is inside a wall".to_string(),
                Unbound::NotEnclosed => "no loop of walls closes round its point".to_string(),
                Unbound::Shared(before) => format!("{} was placed first and has the space this room's point is in", tree.called(&list[before].0)),
            };
            let figure = |value: f64, mine: &[usize], unit: &str, formula: &str| -> Derived {
                let mut inputs = vec![input.clone()];
                inputs.extend(mine.iter().map(|&k| format!("{}#{K_WALL}", placed[k].path)));
                let mut statuses: Vec<Status> = mine.iter().map(|&k| placed[k].shape.max(placed[k].span)).collect();
                statuses.extend([room.point.status, cut_status]);
                let mut notes: Vec<String> = Vec::new();
                for &k in &rests_on[i] {
                    statuses.push(placed[k].span);
                    if !inputs.contains(&format!("{}#{K_WALL}", placed[k].path)) {
                        inputs.push(format!("{}#{K_WALL}", placed[k].path));
                    }
                    notes.push(format!("whether {} ({}) bounds this room rests on its heights, which are {}", tree.called(&placed[k].path), placed[k].path, format!("{:?}", placed[k].span).to_lowercase()));
                }
                // A wall's end is cut to the walls it meets there, bounding or not. Where
                // that end is at this room (a corner of the room lies within the walls'
                // thickness of it), the room is no surer than the line and thickness of
                // each of those. An end at the far side of the building is not this room's.
                let thick = |w: &walls::PlanWall| (w.ext - w.int).abs();
                for (k, at_end) in mine.iter().flat_map(|&k| [(k, false), (k, true)]) {
                    let met = walls::meets(&plan_walls, k, at_end, tol);
                    let node = if at_end { plan_walls[k].end } else { plan_walls[k].start };
                    let reach = met.iter().map(|&n| thick(&plan_walls[n])).fold(thick(&plan_walls[k]), f64::max) + tol;
                    if !found.corners.iter().any(|c| c.dist(node) <= reach) {
                        continue;
                    }
                    for n in met {
                        if placed[n].shape != Status::Known && !mine.contains(&n) {
                            statuses.push(placed[n].shape);
                            let cited = format!("{}#{K_WALL}", placed[n].path);
                            if !inputs.contains(&cited) {
                                inputs.push(cited);
                                notes.push(format!("{} ({}) is cut to {} ({}), whose line or thickness is {}", tree.called(&placed[k].path), placed[k].path, tree.called(&placed[n].path), placed[n].path, format!("{:?}", placed[n].shape).to_lowercase()));
                            }
                        }
                    }
                }
                let mut d = Derived::new(Some(value), unit, &statuses, formula, &inputs);
                // A wall elsewhere that could not be placed is assumed not to reach this
                // storey, and the figure says so.
                if let (Some((wall, why)), true) = (unplaced.first(), d.value.is_some()) {
                    d.status = d.status.max(Status::Assumed);
                    notes.push(format!("assumes {} does not bound this room: it cannot be placed ({why})", tree.called(wall)));
                }
                if d.value.is_some() {
                    d.note = notes.join("; ");
                }
                d
            };
            let m = out.entry(path.clone()).or_default();
            match (&found.finish, found.in_wall) {
                (Ok((area, round, mine)), false) => {
                    m.insert("floor_area".into(), figure(*area, mine, "SF", F_FLOOR));
                    m.insert("perimeter".into(), figure(*round, mine, "LF", F_ROUND));
                }
                (other, _) => {
                    let reason = why(other.as_ref().err().copied().unwrap_or(Unbound::NotEnclosed));
                    m.insert("floor_area".into(), Derived::refused("SF", F_FLOOR, &[input.clone()], &reason));
                    m.insert("perimeter".into(), Derived::refused("LF", F_ROUND, &[input.clone()], &reason));
                }
            }
            match (&found.centre, found.in_wall) {
                (Ok((area, mine)), false) => {
                    m.insert("gross_area".into(), figure(*area, mine, "SF", F_GROSS));
                }
                (other, _) => {
                    let reason = why(other.as_ref().err().copied().unwrap_or(Unbound::NotEnclosed));
                    m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, &[input.clone()], &reason));
                }
            }
        }
    }
}

// ---------------------------------------------------------------- reading it back

/// The rows a flat reading of the job shows for this module's components on one element.
pub fn rows(w: &World, e: Entity, path: &str, names: &HashMap<String, String>) -> Vec<Row> {
    let mut rows = Vec::new();
    let pair = |[a, b]: &[P2; 2]| format!("({}, {}) to ({}, {})", view::num(a[0]), view::num(a[1]), view::num(b[0]), view::num(b[1]));
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
        rows.push(view::length(path, "level height", &c.height));
        if let Some(h) = &c.computation_height {
            rows.push(view::length(path, "rooms find their walls at", h));
        }
    }
    if let Some(c) = w.get::<Room>(e) {
        rows.push(view::entered(path, "a point inside it", &c.point, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
        if let Some(n) = &c.number {
            rows.push(view::entered(path, "number", n, |s| s.clone()));
        }
    }
    if let Some(c) = w.get::<WallType>(e) {
        rows.push(view::length(path, "wall thickness", &c.thickness));
        layers(&mut rows, &c.layers);
    }
    if let Some(c) = w.get::<FloorType>(e) {
        rows.push(view::length(path, "floor thickness", &c.thickness));
        layers(&mut rows, &c.layers);
    }
    if let Some(c) = w.get::<Item>(e) {
        rows.push(plain("an item, placed as", c.category.a(), "what kind of thing it is"));
        if !c.sheet.is_empty() {
            rows.push(plain("made from", c.sheet.clone(), "the maker's sheet it was read from"));
        }
        rows.push(view::entered(path, "maker", &c.maker, |s| s.clone()));
        rows.push(view::entered(path, "model", &c.model, |s| s.clone()));
        rows.push(view::length(path, "width", &c.width));
        rows.push(view::length(path, "depth", &c.depth));
        rows.push(view::length(path, "height", &c.height));
        if let Some(weight) = &c.weight {
            rows.push(view::entered(path, "weight", weight, |x| view::num(*x)));
        }
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
        rows.push(view::length(path, "center, along the wall", &c.offset));
        rows.push(view::length(path, "sill", &c.sill));
    }
    if let Some(c) = w.get::<Floor>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "boundary", &c.boundary, |pts| format!("{} points", pts.len())));
        if let Some(o) = &c.offset {
            rows.push(view::length(path, "offset", o));
        }
    }
    if let Some(c) = w.get::<SegmentType>(e) {
        rows.push(plain("a type of", format!("{:?}", c.category).to_lowercase(), "what the type is for"));
        rows.push(view::length(path, if c.round { "diameter" } else { "width" }, &c.width));
        if let Some(d) = &c.depth {
            rows.push(view::length(path, "depth", d));
        }
    }
    if let Some(c) = w.get::<Segment>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "line on the plan", &c.line, pair));
        rows.push(view::length(path, if c.end_offset.is_some() { "height above its level, first end" } else { "height above its level" }, &c.offset));
        if let Some(o) = &c.end_offset {
            rows.push(view::length(path, "height above its level, second end", o));
        }
        if c.justify != Justify::Centre {
            rows.push(plain("its heights are those of its", format!("{:?}", c.justify).to_lowercase(), "which line of its section the heights were given to"));
        }
    }
    if let Some(c) = w.get::<Fitting>(e) {
        rows.push(plain("fitting", c.kind.title().to_lowercase(), "what joins the runs here"));
    }
    if let Some(c) = w.get::<Placed>(e) {
        rows.push(view::entered(path, "what it is", &c.what, |s| s.clone()));
        rows.push(view::entered(path, "position", &c.at, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
        rows.push(view::length(path, "height above its level", &c.offset));
        if let Some(a) = &c.angle {
            // `entered` writes the unit after the figure: "180 deg".
            rows.push(view::entered(path, "turned", a, |x| view::num(*x)));
        }
        // Its size as a tape reads it, wide by deep by high: the same way an item's is said.
        let mut size = view::entered(path, "size", &c.size, |s| format!("{} × {} × {}", view::num(s[0]), view::num(s[1]), view::num(s[2])));
        if let Some([wide, deep, high]) = c.size_ft() {
            size.text = format!("{} × {} × {}", units::feet_as_ft_in(wide, 16), units::feet_as_ft_in(deep, 16), units::feet_as_ft_in(high, 16));
        }
        rows.push(size);
    }
    if let Some(c) = w.get::<Connectors>(e) {
        for (i, k) in c.0.iter().enumerate() {
            let what = [Some(format!("{:?}", k.kind).to_lowercase()), k.domain.map(|d| format!("{d:?}").to_lowercase()), (k.flow != Flow::Both).then(|| format!("{:?}", k.flow).to_lowercase())].into_iter().flatten().collect::<Vec<_>>().join(", ");
            let label = if k.name.is_empty() { format!("connector {} ({what})", i + 1) } else { format!("connector {}: {} ({what})", i + 1, k.name) };
            rows.push(view::entered(path, &label, &k.at, |p| format!("({}, {}, {})", view::num(p[0]), view::num(p[1]), view::num(p[2]))));
            if let Some(size) = &k.size {
                rows.push(view::length(path, &format!("connector {} size", i + 1), size));
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
    /// One straight piece from `a` to `b` along its centre line, and its section: round by
    /// a diameter, rectangular by width and depth, or neither when its size is not known.
    Run { a: P3, b: P3, round: Option<f64>, rect: Option<(f64, f64)> },
    /// A box by the middle of its base and its size, turned `angle` degrees
    /// counter-clockwise from east about that point.
    Box { at: P3, size: P3, angle: f64 },
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
        if !(matches!(kind, Kind::Wall | Kind::Floor) || kind.is_segment() || kind.is_placed()) {
            continue;
        }
        let storey = tree.storey_of(&n.path).map(str::to_string);
        let system = w.get::<InSystem>(e).and_then(|s| s.0.value.clone());
        let discipline = system.as_ref().and_then(|p| index.get(p)).and_then(|s| w.get::<SystemInfo>(*s)).map(|s| s.discipline).unwrap_or(kind.discipline());
        // Where its storey's floor is: None when it is under no storey, Some(None) when
        // the storey's elevation is not known.
        let base = storey.as_ref().map(|s| tree.levels.get(s).and_then(|l| l.elevation.feet()));
        // A shape stands no surer than the height of the floor it is measured from.
        let floor_status = storey.as_ref().and_then(|s| tree.levels.get(s)).map(|l| l.elevation.status).unwrap_or(Status::Known);
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
                // Where it stands as well as how tall it is: its storey's floor and its
                // base offset are part of its shape (`span`, not the height's own status).
                let mut status = c.line.status.max(heights.span).max(section_status);
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
                // A wall with an opening that could not be cut is drawn solid where the
                // opening is: its shape is not known to be right there.
                if !uncut.is_empty() {
                    status = status.max(Status::Assumed);
                }
                Ok((Shape::Wall { a, b, base, height, thickness, holes }, status))
            })(),
            Kind::Floor => (|| {
                let c = w.get::<Floor>(e).ok_or("it has no boundary")?;
                let boundary = c.boundary_ft().ok_or("its boundary is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (offset, offset_status) = match &c.offset {
                    None => (0.0, Status::Known),
                    Some(o) => (o.feet().ok_or("its offset is not known")?, o.status),
                };
                let (_, t) = typed(&tree.floor_types, &c.type_name, "a floor")?;
                let thickness = t.thickness.feet().ok_or("its type's thickness is not known")?;
                Ok((Shape::Floor { boundary, top: level + offset, thickness }, c.boundary.status.max(c.type_name.status).max(t.thickness.status).max(offset_status).max(floor_status)))
            })(),
            k if k.is_segment() => (|| {
                let c = w.get::<Segment>(e).ok_or("it has no line")?;
                let [p, q] = c.line_ft().ok_or("its line is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (z0, z1) = c.offsets_ft().ok_or("its height above its level is not known")?;
                let t = typed(&tree.segment_types, &c.type_name, "a run").ok().map(|x| x.1);
                let (round, rect) = (t.and_then(|t| t.round_ft()), t.and_then(|t| t.rect_ft()));
                let section = round.map(|d| (d, d)).or(rect);
                if c.justify != Justify::Centre && section.is_none() {
                    return Err("its size is not known and its heights are those of its top or its bottom, so its center line cannot be placed".into());
                }
                let (width, depth) = section.unwrap_or((0.0, 0.0));
                let g = runs::Geom::of(level, p, q, z0, z1, c.justify, width, depth, round.is_some());
                let sized = if section.is_some() { t.map(|t| t.status().max(c.type_name.status)).unwrap_or(Status::Unknown) } else { Status::Unknown };
                let status = c.line.status.max(c.offset.status).max(c.end_offset.as_ref().map(|e| e.status).unwrap_or(Status::Known)).max(sized).max(floor_status);
                Ok((Shape::Run { a: g.a, b: g.b, round, rect }, status))
            })(),
            _ => (|| {
                let c = w.get::<Placed>(e).ok_or("it has no position")?;
                let at = c.at_ft().ok_or("its position is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let offset = c.offset.feet().ok_or("its height above its level is not known")?;
                let size = c.size_ft().ok_or("its size is not known")?;
                let angle = c.angle_deg().ok_or("how far it is turned is not known")?;
                let status = c.at.status.max(c.offset.status).max(c.size.status).max(c.angle.as_ref().map(|a| a.status).unwrap_or(Status::Known)).max(floor_status);
                Ok((Shape::Box { at: [at[0], at[1], level + offset], size, angle }, status))
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
    /// The fittings on the system, counted by kind and by what each is said to be, with
    /// the worst status among those. A fitting with no kind entered is counted under None.
    pub fittings: Vec<(Option<FittingKind>, String, usize, Status)>,
    /// How its runs hang together: what is joined, what is open, what has no fitting.
    pub joined: Continuity,
}

/// One end of a run, or one connector, that meets nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenEnd {
    /// The run or the placed thing, by path.
    pub element: String,
    /// Said for a person: "hot riser: its second end", "lavatory 1: its Waste connector".
    pub said: String,
    pub at: P3,
}

/// How a system's runs hang together, read off the job's network (`network.rs`). Nothing
/// here stops a job from loading or a length from being counted: a job being entered has
/// open ends. It is what an estimator checks before trusting a count of fittings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Continuity {
    /// How many separate networks its runs fall into. One, when every run on the system
    /// is joined to every other.
    pub networks: usize,
    pub open: Vec<OpenEnd>,
    /// Where a run or a connector meets a run along its length with no fitting entered:
    /// each is a tee, or a mistake.
    pub no_fitting: Vec<P3>,
    /// What else looks wrong, said in a sentence each.
    pub warnings: Vec<String>,
    /// What could not be placed, and so was not checked, each with why.
    pub left_out: Vec<(String, String)>,
}

/// A size in feet, said in inches: `6 in`, `3/4 in`.
fn in_inches(feet: f64) -> String {
    units::show_length(&Len::One((feet * 12.0 * 1.0e6).round() / 1.0e6), units::IN)
}

/// What each system comes to: its runs by kind and size with their length, its fittings
/// counted by kind, and its equipment and fixtures counted by what they are.
///
/// A run said to be on a system with less than certainty (Assumed) carries that onto its
/// line: the line's length is no surer than that the run is on this system. Two groups
/// come after the systems, when there is anything in them, so nothing is left off for
/// want of a system: what is on no system, and what is on a system that is not known.
/// Not knowing the system is not the same as having none.
///
/// A run's size is its type's. Runs are grouped by their size as a size, not as it was
/// typed: 6 in, 0.5 ft and 0' 6" are one line. Lines are in order of size, smallest
/// first, with no size last: a run whose type or whose type's size nobody has is not
/// dropped and not guessed.
pub fn takeoff(job: &mut Job) -> Vec<SystemTakeoff> {
    let order = job.order.clone();
    let net = network::network(job, None, None, network::TOLERANCE_FT);
    let tree = Tree::of(&mut job.world);
    let w = &job.world;
    let blank = |name: &str, flag: Status| SystemTakeoff { path: String::new(), name: name.into(), discipline: Discipline::Architecture, classification: String::new(), flag, runs: view::sum_of("LF", &[]), lines: Vec::new(), placed: Vec::new(), fittings: Vec::new(), joined: Continuity::default() };
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
    let mut runs: BTreeMap<(usize, u8, u8, i64, i64), (Kind, String, Status, Figures)> = BTreeMap::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e).copied()) else { continue };
        if !(kind.is_segment() || kind.is_placed()) {
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
            if kind == Kind::Fitting {
                let fitting = w.get::<Fitting>(e).map(|f| f.kind);
                let status = what.1.max(sure).max(if fitting.is_some() { Status::Known } else { Status::Unknown });
                let list = &mut systems[at].fittings;
                match list.iter_mut().find(|x| x.0 == fitting && x.1 == what.0) {
                    Some(entry) => {
                        entry.2 += 1;
                        entry.3 = entry.3.max(status);
                    }
                    None => list.push((fitting, what.0, 1, status)),
                }
                continue;
            }
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
        // Its size is its type's. No type, a type nobody knows, or a type with no size:
        // each is a run of no known size.
        let sized = w.get::<Segment>(e).and_then(|r| typed(&tree.segment_types, &r.type_name, "a run").ok().map(|(_, t)| (t, r.type_name.status)));
        let micro = |feet: f64| (feet * 12.0 * 1.0e6).round() as i64;
        let (shape, a, b, said) = match sized.map(|(t, _)| (t.round_ft(), t.rect_ft())) {
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
        let entry = runs.entry((at, rank, shape, a, b)).or_insert((kind, said, Status::Known, Vec::new()));
        entry.2 = entry.2.max(if shape == 2 { Status::Unknown } else { sized.map(|(t, sure)| t.status().max(sure)).unwrap_or(Status::Unknown) });
        // A figure that is there is no surer than that the run is on this system.
        let (l, ls) = figure("true_length");
        entry.3.push((n.path.clone(), l, if l.is_some() { ls.max(sure) } else { ls }));
    }
    let mut all: Vec<Figures> = vec![Vec::new(); systems.len()];
    for ((at, ..), (_, _, _, lengths)) in &runs {
        all[*at].extend(lengths.iter().cloned());
    }
    for (i, figures) in all.iter().enumerate() {
        systems[i].runs = view::sum_of("LF", figures);
    }
    for ((at, ..), (kind, size, size_status, lengths)) in runs {
        systems[at].lines.push(TakeoffLine { kind, size, size_status, length: view::sum_of("LF", &lengths) });
    }
    for t in &mut systems {
        t.fittings.sort_by(|x, y| (x.0.is_none(), x.0, &x.1).cmp(&(y.0.is_none(), y.0, &y.1)));
    }
    // How each hangs together. An element is checked with the group its system puts it in.
    let group_of = |path: &str| -> usize {
        let e = order.iter().find(|e| w.get::<Node>(**e).is_some_and(|n| n.path == path));
        match e.and_then(|e| w.get::<InSystem>(*e)) {
            None => named,
            Some(s) => match s.0.value.as_ref().and_then(|p| systems[..named].iter().position(|x| x.path == *p)) {
                Some(i) if s.0.status != Status::Unknown => i,
                _ => named + 1,
            },
        }
    };
    match net {
        Err(why) => {
            for t in &mut systems {
                t.joined.warnings.push(format!("what is joined to what was not worked out: {why}"));
            }
        }
        Ok(net) => {
            let groups_of = |elements: &[String]| -> std::collections::BTreeSet<usize> { elements.iter().map(|p| group_of(p)).collect() };
            let mut found: Vec<Continuity> = vec![Continuity::default(); systems.len()];
            for g in &net.groups {
                // A network counts for each system that has a run in it.
                let runs: Vec<String> = g.elements.iter().filter(|p| tree.kinds.get(*p).is_some_and(|k| k.is_segment())).cloned().collect();
                for i in groups_of(&runs) {
                    found[i].networks += 1;
                }
            }
            for &n in &net.open {
                let node = &net.nodes[n];
                let Some(joint) = node.joints.first() else { continue };
                let name = tree.called(joint.element());
                let said = match joint {
                    network::Joint::RunEnd { end, .. } => format!("{name}: its {} end", if *end { "second" } else { "first" }),
                    network::Joint::RunSide { .. } => format!("{name}: along its length"),
                    network::Joint::Connector { element, index, .. } => {
                        let e = order.iter().find(|e| w.get::<Node>(**e).is_some_and(|n| n.path == *element));
                        let label = e.and_then(|e| w.get::<Connectors>(*e)).and_then(|c| c.0.get(*index)).map(|k| k.name.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| format!("connector {}", index + 1));
                        format!("{name}: its {label} connector")
                    }
                };
                // An open end of a run is its system's. A connector that meets nothing has
                // no run to say whose it is: a pipe connector that says what it carries
                // is listed on the systems that carry that (a water closet's open waste
                // outlet is the drain's to deal with, though the fixture is on the cold
                // water), and any other on the system the thing itself is on.
                let mut where_to: Vec<usize> = Vec::new();
                if let network::Joint::Connector { domain: Some(domain), .. } = joint {
                    let carried = domain.system_classification();
                    where_to = (0..named).filter(|&i| carried == Some(systems[i].classification.as_str())).collect();
                }
                if where_to.is_empty() {
                    where_to.push(group_of(joint.element()));
                }
                for i in where_to {
                    found[i].open.push(OpenEnd { element: joint.element().to_string(), said: said.clone(), at: node.at });
                }
            }
            for warning in &net.warnings {
                // A warning is said on the systems of the runs it is about. (A water closet
                // is on the cold water system and its waste connector is in the drain's
                // network: a fault in the drain is not the cold water's.)
                let runs: Vec<String> = warning.elements.iter().filter(|p| tree.kinds.get(*p).is_some_and(|k| k.is_segment())).cloned().collect();
                for i in groups_of(if runs.is_empty() { &warning.elements } else { &runs }) {
                    match (warning.kind, warning.at) {
                        ("joined_mid_run", Some(at)) => found[i].no_fitting.push(at),
                        _ => found[i].warnings.push(warning.message.clone()),
                    }
                }
            }
            for (path, why) in &net.left_out {
                found[group_of(path)].left_out.push((path.clone(), why.clone()));
            }
            for (t, joined) in systems.iter_mut().zip(found) {
                t.joined = joined;
            }
        }
    }
    systems.into_iter().enumerate().filter(|(i, t)| *i < named || !t.lines.is_empty() || !t.placed.is_empty() || !t.fittings.is_empty()).map(|(_, t)| t).collect()
}

/// The storey each element stands on, by path: the nearest storey above it in the tree.
/// An element under no storey is not in the map.
pub fn storeys_of(job: &mut Job) -> HashMap<String, String> {
    let tree = Tree::of(&mut job.world);
    tree.parents.keys().filter_map(|p| tree.storey_of(p).map(|s| (p.clone(), s.to_string()))).collect()
}

/// What a storey's walls, floors and rooms come to.
#[derive(Clone, Debug)]
pub struct StoreyTotals {
    pub path: String,
    pub name: String,
    pub elevation: Option<f64>,
    pub walls: Sum,
    pub wall_face: Sum,
    pub wall_net: Sum,
    pub floors: Sum,
    /// Its rooms: the floor inside their walls, added up.
    pub rooms: Sum,
    pub doors: usize,
    pub windows: usize,
    /// Its walls again, by the type of wall, in the order each type first appears: a
    /// 6 in outside wall and a 4 in partition are not one number to an estimator.
    pub walls_by_type: Vec<WallsOfType>,
}

/// The walls of one type on one level, added up.
#[derive(Clone, Debug, PartialEq)]
pub struct WallsOfType {
    /// The type, by path and by name; a wall whose type is not known is under "no type".
    pub path: String,
    pub name: String,
    pub length: Sum,
    pub face: Sum,
    pub net: Sum,
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
        let mut figures: [Vec<(String, Option<f64>, Status)>; 5] = Default::default();
        let (mut doors, mut windows) = (0, 0);
        // Each wall type met, in order: its path, and its walls' three figures.
        #[allow(clippy::type_complexity)]
        let mut of_type: Vec<(String, [Vec<(String, Option<f64>, Status)>; 3])> = Vec::new();
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
                    let its_type = w.get::<Wall>(e).and_then(|wall| wall.type_name.value.clone()).unwrap_or_default();
                    let at = match of_type.iter().position(|(t, _)| *t == its_type) {
                        Some(at) => at,
                        None => {
                            of_type.push((its_type, Default::default()));
                            of_type.len() - 1
                        }
                    };
                    for (i, name) in ["plan_length", "face_area", "net_area"].into_iter().enumerate() {
                        of_type[at].1[i].push(figure(name));
                    }
                }
                Kind::Floor => figures[3].push(figure("plan_area")),
                Kind::Room => figures[4].push(figure("floor_area")),
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
            rooms: view::sum_of("SF", &figures[4]),
            doors,
            windows,
            walls_by_type: of_type
                .into_iter()
                .map(|(path, f)| {
                    let name = order.iter().filter_map(|e| w.get::<Node>(*e)).find(|n| n.path == path).map(|n| n.name.clone()).unwrap_or_else(|| "no type".to_string());
                    WallsOfType { path, name, length: view::sum_of("LF", &f[0]), face: view::sum_of("SF", &f[1]), net: view::sum_of("SF", &f[2]) }
                })
                .collect(),
        });
    }
    out
}
