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
use crate::view::{self, Row, Sum, WorkSplit};

pub mod clash;
mod footings;
pub mod forms;
pub mod framing;
pub mod frames;
pub mod members;
pub mod model;
pub mod network;
pub mod planset;
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
pub const K_ROOM_LINE: &str = "pl::room_line";
pub const K_PLACED: &str = "pl::placed";
pub const K_COLUMN: &str = "pl::column";
pub const K_JOISTS_SPAN: &str = "pl::joists_span";
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
    /// A beam, a post or a pad footing (call 164): what carries the building. Framing bears
    /// on framing, and none of it is a run of anything that flows.
    pub fn is_framing(self) -> bool {
        matches!(self, Kind::Beam | Kind::Post | Kind::PadFooting)
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
    // What is to be demolished is gone when the work is done: a new wall may stand
    // where it stood (call 148).
    let gone = crate::components::demolished(&mut job.world);
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
    let mut q = job.world.query::<(&Node, &Kind, Option<&InSystem>, Option<&Wall>, Option<&Opening>, Option<&Floor>, Option<&Segment>, Option<&Stood>)>();
    for (node, kind, in_system, wall, opening, floor, segment, column) in q.iter(&job.world) {
        let me = &node.path;
        let parent = node.parent.as_ref().and_then(|p| kinds.get(p)).copied();
        if kind.is_opening() && parent != Some(Kind::Wall) {
            return Err(format!("{me}: {kind:#} is an opening in a wall and has to be under one"));
        }
        if *kind == Kind::Room && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a room is a space on a level and has to be directly under one"));
        }
        if *kind == Kind::RoomLine && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a room line parts the rooms of a level and has to be directly under one"));
        }
        if matches!(kind, Kind::Wall | Kind::Floor | Kind::Footing) && parent != Some(Kind::Storey) {
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
                // A footing lies below its base by its depth; a wall stands up from it. So a
                // wall on its footing, on the same line, is two things one over the other.
                let stands = heights.base.ok().zip(heights.height.ok()).map(|(b, h)| if *kind == Kind::Footing { (b - h, b) } else { (b, b + h) });
                standing.push((me.clone(), storey.clone(), centre, thick, stands));
            }
            if let Some(p) = &w.type_name.value {
                if !tree.wall_types.contains_key(p) {
                    return Err(format!("{me}: its type is {p:?}, which is not a {} type in this job", if *kind == Kind::Footing { "footing" } else { "wall" }));
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
        if let Some(p) = column.and_then(|c| c.type_name.value.as_ref()) {
            match tree.segment_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a {} type in this job", kind.said())),
                Some(t) if t.category != *kind => return Err(format!("{me}: is {kind:#}, but its type, {p}, is {:#}'s", t.category)),
                Some(_) => {}
            }
        }
        // A run's heights and a placed thing's are above the storey it is under.
        if (kind.is_segment() || kind.is_placed() || kind.is_framing()) && tree.storey_of(me).is_none() {
            return Err(format!("{me}: {kind:#} is set at a height above a level and has to be under one"));
        }
    }
    // One wall drawn over another on a storey is the same wall counted twice. Two that
    // stand one over the other (a wall on a knee wall) are two walls.
    standing.sort_by(|x, y| x.0.cmp(&y.0));
    for (i, (a, storey, line, thick, stands)) in standing.iter().enumerate() {
        for (b, other, its, as_thick, theirs) in &standing[i + 1..] {
            let tol = crate::check::FIT_TOLERANCE_FT;
            if gone.contains(a) != gone.contains(b) {
                continue;
            }
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
                // Said in the words of what the two are (a footing is held as a wall is).
                let what = if b.starts_with("footing") { "footing" } else { "wall" };
                return Err(format!("{b}: lies along {a} for {} ft: one {what} drawn over another would be counted twice", view::figure_shown(shared, 2)));
            }
        }
    }
    Ok(())
}

/// What may not be drawn, and is said of a job that already has it (call 179, B5 and B6): an
/// opening past the end of its wall or over another opening in it; two floors of one level in
/// each other; two beams, two posts or two pad footings in one place. Each is a thing counted
/// twice, or hung where there is no wall, as two walls on one line are. Each comes as a key
/// (the thing and which fault) and the sentence. A job saved before that holds one still opens
/// and still says it (`check` prints these); what an edit may not do is bring one in
/// (`editing::all_or_nothing` refuses a fault the job did not have).
pub fn drawn_faults(job: &mut Job) -> Vec<(String, String)> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let gone = crate::components::demolished(&mut job.world);
    let tree = Tree::of(&mut job.world);
    let w = &job.world;
    let mut out: Vec<(String, String)> = Vec::new();
    let mut nodes: Vec<(&Node, Entity)> = job.order.iter().filter_map(|&e| w.get::<Node>(e).map(|n| (n, e))).collect();
    nodes.sort_by(|x, y| x.0.path.cmp(&y.0.path));
    // Openings: within the wall, clear of one another.
    for (n, e) in &nodes {
        if w.get::<Kind>(*e) != Some(&Kind::Wall) {
            continue;
        }
        let Some(line) = w.get::<Wall>(*e).and_then(|c| c.line_ft()) else { continue };
        let length = geom::dist(line[0], line[1]);
        let wall_high = w.get::<Wall>(*e).and_then(|c| plan_wall(&tree, &n.path, c).ok()).map(|(placed, _, _, _)| placed.top - placed.base);
        let mut spans: Vec<(&str, [f64; 4])> = Vec::new();
        for (c, ce) in &nodes {
            if c.parent.as_deref() != Some(n.path.as_str()) || !w.get::<Kind>(*ce).is_some_and(|k| k.is_opening()) {
                continue;
            }
            // (An opening to demolish is no hole in the wall that is built: the wall's figures leave it out,
            // and a new one may stand where it stood.)
            if gone.contains(&c.path) {
                continue;
            }
            let Some(o) = w.get::<Opening>(*ce) else { continue };
            let Some(span) = hole_of(&tree, &c.path, o).0.span else { continue };
            let [from, to, _, head] = span;
            // A door or a window does not go above the top of its wall (call 180, C5).
            if let Some(high) = wall_high {
                if head > high + tol {
                    out.push((format!("{} over the top", c.path), format!("{} goes up {} from the floor and its wall is only {} high: an opening lies within the height of its wall", c.path, ft_in_said(head), ft_in_said(high))));
                }
            }
            if from < -tol {
                out.push((format!("{} off the start", c.path), format!("{} starts {} ft before the start of its wall: an opening lies within its wall", c.path, view::figure_shown(-from, 2))));
            } else if to > length + tol {
                out.push((format!("{} off the end", c.path), format!("{} ends {} ft along a wall {} ft long: an opening lies within its wall", c.path, view::figure_shown(to, 2), view::figure_shown(length, 2))));
            }
            for (other, s) in &spans {
                if from < s[1] && s[0] < to && span[2] < s[3] && s[2] < span[3] {
                    out.push((format!("{} over {other}", c.path), format!("{} and {other} lie over each other: an opening is clear of every other opening in its wall", c.path)));
                }
            }
            spans.push((c.path.as_str(), span));
        }
    }
    // Floors of one level: none in another.
    let floors: Vec<(&str, Option<&str>, Vec<P2>)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Floor)).filter_map(|(n, e)| Some((n.path.as_str(), n.parent.as_deref(), w.get::<Floor>(*e)?.boundary_ft()?))).collect();
    for (i, (a, level, ring)) in floors.iter().enumerate() {
        for (b, other, its) in &floors[i + 1..] {
            if level == other && gone.contains(*a) == gone.contains(*b) && rings_overlap(ring, its) {
                out.push((format!("{b} over {a}"), format!("{b} lies in {a} on the same level: two floors in one place would be counted twice")));
            }
        }
    }
    // Beams, posts and pad footings: not two in one place.
    // (Call 180, C4: for the length they share a line, within the width of the beam, and not only where one lies exactly on the other.)
    let beams: Vec<(&str, Option<&str>, [P2; 2], (f64, f64), f64)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Beam)).filter_map(|(n, e)| {
        let s = w.get::<Segment>(*e)?;
        let wide = tree.segment_types.get(s.type_name.value.as_deref()?).and_then(|t| t.width.feet()).unwrap_or(0.0);
        Some((n.path.as_str(), n.parent.as_deref(), s.line_ft()?, s.offsets_ft()?, wide))
    }).collect();
    for (i, (a, level, line, top, wide_a)) in beams.iter().enumerate() {
        for (b, other, its, their_top, wide_b) in &beams[i + 1..] {
            if level != other || gone.contains(*a) != gone.contains(*b) {
                continue;
            }
            let (la, lb) = (geom::dist(line[0], line[1]), geom::dist(its[0], its[1]));
            if la <= tol || lb <= tol {
                continue;
            }
            let (u, v) = ([(line[1][0] - line[0][0]) / la, (line[1][1] - line[0][1]) / la], [(its[1][0] - its[0][0]) / lb, (its[1][1] - its[0][1]) / lb]);
            // On one line: running within a degree of the same way, their lines within a beam's width of one another.
            if (u[0] * v[1] - u[1] * v[0]).abs() > 0.017_452_406_437_283_512 {
                continue;
            }
            let (along, across) = (|p: P2| (p[0] - line[0][0]) * u[0] + (p[1] - line[0][1]) * u[1], |p: P2| ((p[1] - line[0][1]) * u[0] - (p[0] - line[0][0]) * u[1]).abs());
            if across(its[0]).max(across(its[1])) > wide_a.max(*wide_b).max(tol) + tol {
                continue;
            }
            let (lo, hi) = (along(its[0]).min(along(its[1])).max(0.0), along(its[0]).max(along(its[1])).min(la));
            if hi - lo <= tol {
                continue;
            }
            // And at one height over what they share: the top of each at the two ends of the stretch.
            let height = |t: (f64, f64), len: f64, at: f64| t.0 + (t.1 - t.0) * (at / len);
            let flip = along(its[0]) > along(its[1]);
            let (theirs_at_lo, theirs_at_hi) = {
                let (s0, s1) = (along(its[0]), along(its[1]));
                let (first, second) = if flip { (their_top.1, their_top.0) } else { (their_top.0, their_top.1) };
                let (s_lo, s_hi) = (s0.min(s1), s0.max(s1));
                let on = |at: f64| first + (second - first) * ((at - s_lo) / (s_hi - s_lo).max(1.0e-12));
                (on(lo), on(hi))
            };
            if (height(*top, la, lo) - theirs_at_lo).abs() > tol || (height(*top, la, hi) - theirs_at_hi).abs() > tol {
                continue;
            }
            out.push((format!("{b} over {a}"), format!("{b} shares a line with {a} for {}: two beams in one place would be counted twice", ft_in_said(hi - lo))));
        }
    }
    for kind in [Kind::Post, Kind::PadFooting] {
        let columns: Vec<(&str, Option<&str>, P2, (f64, f64), [f64; 2])> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&kind)).filter_map(|(n, e)| {
            let c = w.get::<Stood>(*e)?;
            let (base, height) = (c.base_offset.feet()?, c.height.feet()?);
            // Its footprint: its type's width by its depth, or across for a round one or one with no depth.
            let t = tree.segment_types.get(c.type_name.value.as_deref()?);
            let across = t.and_then(|t| t.width.feet()).unwrap_or(0.0);
            let deep = t.and_then(|t| t.depth.as_ref().and_then(|d| d.feet())).unwrap_or(across);
            Some((n.path.as_str(), n.parent.as_deref(), c.at_ft()?, (base, base + height), [across, deep]))
        }).collect();
        for (i, (a, level, at, span, size)) in columns.iter().enumerate() {
            for (b, other, its, theirs, their_size) in &columns[i + 1..] {
                if level != other || gone.contains(*a) != gone.contains(*b) || !(span.0 < theirs.1 - tol && theirs.0 < span.1 - tol) {
                    continue;
                }
                // (Call 180, C4: pad footings whose plan areas overlap, not only the same point.)
                let overlap = if kind == Kind::PadFooting { (at[0] - its[0]).abs() < (size[0] + their_size[0]) / 2.0 - tol && (at[1] - its[1]).abs() < (size[1] + their_size[1]) / 2.0 - tol } else { false };
                if geom::dist(*at, *its) <= tol || overlap {
                    out.push((format!("{b} over {a}"), format!("{b} lies where {a} is: two {} in one place would be counted twice", if kind == Kind::Post { "posts" } else { "pad footings" })));
                }
            }
        }
    }
    out
}

/// Feet as a builder says them aloud: "4 ft", "11 ft 6 in".
fn ft_in_said(feet: f64) -> String {
    if (feet - feet.round()).abs() < 1.0e-6 {
        format!("{} ft", feet.round())
    } else {
        units::feet_as_ft_in(feet, 16).replacen("' ", " ft ", 1).replacen('"', " in", 1).trim_start_matches("0 ft ").to_string()
    }
}

/// What is drawn out of reach of what should carry it (call 179, B1): a room that reaches
/// past the floor of its level, and a wall that stands off a footing that has no wall on it.
/// Cautions, never refusals: the drawing may be what is meant, and `move --alone` leaves
/// exactly this. Each comes as a key and the sentence. A level with no floor drawn says
/// nothing of its rooms, and a footing with no wall near it says nothing.
pub fn reach_cautions(job: &mut Job) -> Vec<(String, String)> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let gone = crate::components::demolished(&mut job.world);
    let tree = Tree::of(&mut job.world);
    let outlines = room_outlines(job);
    let bodies = crate::carries::bodies(job);
    let w = &job.world;
    let mut out: Vec<(String, String)> = Vec::new();
    let mut nodes: Vec<(&Node, Entity)> = job.order.iter().filter_map(|&e| w.get::<Node>(e).map(|n| (n, e))).collect();
    nodes.sort_by(|x, y| x.0.path.cmp(&y.0.path));
    // Rooms and floors of one level.
    let floors: Vec<(&Node, Vec<P2>)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Floor)).filter_map(|(n, e)| Some((*n, w.get::<Floor>(*e)?.boundary_ft()?))).filter(|(n, _)| !gone.contains(&n.path)).collect();
    let inside = |p: P2, ring: &[P2]| crate::vif::inside(ring, p) || geom::point_to_outline(p, ring) <= tol;
    for (n, _) in nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Room)) {
        let Some(poly) = outlines.get(&n.path) else { continue };
        let there: Vec<&(&Node, Vec<P2>)> = floors.iter().filter(|(f, _)| f.parent == n.parent).collect();
        if there.is_empty() || gone.contains(&n.path) {
            continue;
        }
        // How far each corner of the room is outside every floor of its level.
        let reach = |ring: Option<&Vec<P2>>| -> f64 {
            poly.iter()
                .map(|p| {
                    let rings: Vec<&Vec<P2>> = match ring {
                        Some(r) => vec![r],
                        None => there.iter().map(|(_, r)| r).collect(),
                    };
                    if rings.iter().any(|r| inside(*p, r)) { 0.0 } else { rings.iter().map(|r| geom::point_to_outline(*p, r)).fold(f64::MAX, f64::min) }
                })
                .fold(0.0, f64::max)
        };
        let past = reach(None);
        if past > tol {
            let nearest = there.iter().min_by(|a, b| reach(Some(&a.1)).total_cmp(&reach(Some(&b.1)))).map(|(f, _)| f.name.clone()).unwrap_or_default();
            out.push((format!("{} past its floor", n.path), format!("{} reaches {} past {nearest}: the floor is drawn smaller than the room (a wall was moved, or the floor was drawn short)", if n.name.is_empty() { n.path.clone() } else { n.name.clone() }, ft_in_said(past))));
        }
    }
    // A floor or a roof that no longer reaches the walls under it, or reaches past them (call 180, B1): an edge
    // along a wall with no other wall between, the wall outside it by more than its thickness, or the edge
    // beyond a wall inside it by more than a floor's 2 ft or an eave's 4 ft. A floor is looked at against the
    // walls of its own level, a roof against every wall (a roof is over a wall by the plan alone).
    {
        let roofs: Vec<(&Node, Vec<P2>)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Roof)).filter_map(|(n, e)| Some((*n, w.get::<Footprint>(*e)?.points_ft()?))).filter(|(n, _)| !gone.contains(&n.path)).collect();
        let several = bodies.iter().map(|b| b.storey.as_str()).collect::<std::collections::BTreeSet<_>>().len() > 1;
        let called = |path: &str, storey: &str| match (tree.names.get(path), tree.names.get(storey)) {
            (Some(n), Some(l)) if several && !l.is_empty() => format!("{n} on {l}"),
            (Some(n), _) => n.clone(),
            _ => path.to_string(),
        };
        let things: Vec<(&Node, &Vec<P2>, bool)> = floors.iter().map(|(n, r)| (*n, r, false)).chain(roofs.iter().map(|(n, r)| (*n, r, true))).collect();
        for (thing, ring, is_roof) in things {
            let reach = if is_roof { crate::carries::ROOF_REACH_FT } else { crate::carries::FLOOR_REACH_FT };
            for b in bodies.iter().filter(|b| !gone.contains(&b.path) && (is_roof || Some(b.storey.as_str()) == thing.parent.as_deref())) {
                let found = crate::carries::edges_all(b, ring, &bodies.iter().filter(|o| !gone.contains(&o.path)).cloned().collect::<Vec<_>>(), is_roof);
                let Some((_, e)) = found.iter().filter(|(_, e)| (!e.wall_inside && e.across > b.thick + tol) || (e.wall_inside && e.outward && e.across > reach + tol)).min_by(|x, y| x.1.across.total_cmp(&y.1.across)) else { continue };
                let name = if thing.name.is_empty() { thing.path.clone() } else { thing.name.clone() };
                let wall = called(&b.path, &b.storey);
                out.push((format!("{} off {}", b.path, thing.path), if e.wall_inside {
                    format!("{name} reaches {} past {wall}: more than {} ft, the most a {} is drawn out from a wall that goes with it", ft_in_said(e.across), reach, if is_roof { "roof's eave" } else { "floor's edge" })
                } else {
                    format!("{wall} stands {} outside {name}: {name} no longer reaches it", ft_in_said(e.across))
                }));
            }
        }
    }
    // Footings with no wall on them, and a wall that stands off one.
    let several = bodies.iter().map(|b| b.storey.as_str()).collect::<std::collections::BTreeSet<_>>().len() > 1;
    let called = |path: &str, storey: &str| match (tree.names.get(path), tree.names.get(storey)) {
        (Some(n), Some(l)) if several && !l.is_empty() => format!("{n} on {l}"),
        (Some(n), _) => n.clone(),
        _ => path.to_string(),
    };
    for (n, e) in nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Footing)) {
        let Some(line) = w.get::<Wall>(*e).and_then(|c| c.line_ft()) else { continue };
        let Some(level) = n.parent.as_deref() else { continue };
        if gone.contains(&n.path) {
            continue;
        }
        let flen = geom::dist(line[0], line[1]);
        if flen <= tol {
            continue;
        }
        let on_it = bodies.iter().any(|b| b.storey == level && !gone.contains(&b.path) && crate::carries::over(b.line, b.thick, line).is_some());
        if on_it {
            continue;
        }
        let (ux, uy) = ((line[1][0] - line[0][0]) / flen, (line[1][1] - line[0][1]) / flen);
        // The nearest wall of its level that runs along it, within 10 ft, and overlaps it.
        let mut near: Option<(f64, &crate::carries::Body)> = None;
        for b in bodies.iter().filter(|b| b.storey == level && !gone.contains(&b.path)) {
            let (bx, by) = (b.line[1][0] - b.line[0][0], b.line[1][1] - b.line[0][1]);
            let blen = bx.hypot(by);
            if blen <= tol || (ux * by - uy * bx).abs() / blen > 0.017_452_406_437_283_512 {
                continue;
            }
            let along = |p: P2| ux * (p[0] - line[0][0]) + uy * (p[1] - line[0][1]);
            let (lo, hi) = (along(b.line[0]).min(along(b.line[1])).max(0.0), along(b.line[0]).max(along(b.line[1])).min(flen));
            if hi - lo <= tol {
                continue;
            }
            let lateral = (ux * (b.line[0][1] - line[0][1]) - uy * (b.line[0][0] - line[0][0])).abs();
            if lateral <= 10.0 && near.map(|(l, _)| lateral < l).unwrap_or(true) {
                near = Some((lateral, b));
            }
        }
        if let Some((lateral, b)) = near {
            out.push((format!("{} off {}", b.path, n.path), format!("{} stands {} off {}, which has no wall on it: the footing was left where it was", called(&b.path, &b.storey), ft_in_said(lateral), n.name)));
        }
    }
    out
}

/// Whether two outlines have any area in common: their edges cross, one has a corner strictly
/// inside the other, or a piece of one (the middle of a triangle of it) is strictly inside the
/// other. Outlines that only touch along an edge, or at a corner, do not.
fn rings_overlap(a: &[P2], b: &[P2]) -> bool {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let strictly_in = |p: P2, ring: &[P2]| {
        if geom::point_to_outline(p, ring) <= tol {
            return false;
        }
        let mut inside = false;
        for i in 0..ring.len() {
            let (u, v) = (ring[i], ring[(i + 1) % ring.len()]);
            if (u[1] > p[1]) != (v[1] > p[1]) && p[0] < u[0] + (p[1] - u[1]) / (v[1] - u[1]) * (v[0] - u[0]) {
                inside = !inside;
            }
        }
        inside
    };
    let side = |p: P2, q: P2, r: P2| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    for i in 0..a.len() {
        let (p, q) = (a[i], a[(i + 1) % a.len()]);
        for j in 0..b.len() {
            let (r, t) = (b[j], b[(j + 1) % b.len()]);
            // Crossing, each edge clear of the other's ends by more than the tolerance.
            let (d1, d2, d3, d4) = (side(p, q, r), side(p, q, t), side(r, t, p), side(r, t, q));
            let (lp, lr) = (geom::dist(p, q).max(1.0e-9), geom::dist(r, t).max(1.0e-9));
            if d1 * d2 < 0.0 && d3 * d4 < 0.0 && d1.abs() / lp > tol && d2.abs() / lp > tol && d3.abs() / lr > tol && d4.abs() / lr > tol {
                return true;
            }
        }
    }
    let middles = |ring: &[P2]| -> Vec<P2> { geom::triangles(ring).0.iter().map(|t| [(ring[t[0]][0] + ring[t[1]][0] + ring[t[2]][0]) / 3.0, (ring[t[0]][1] + ring[t[1]][1] + ring[t[2]][1]) / 3.0]).collect() };
    a.iter().any(|p| strictly_in(*p, b)) || b.iter().any(|p| strictly_in(*p, a)) || middles(a).into_iter().any(|p| strictly_in(p, b)) || middles(b).into_iter().any(|p| strictly_in(p, a))
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

/// How wide each door and window is, in feet, by path: its type's width. One whose type
/// or width is not known is left out (call 170: a wall moved asks whether its openings
/// still lie within it).
pub(crate) fn opening_widths(job: &mut Job) -> HashMap<String, f64> {
    let tree = Tree::of(&mut job.world);
    let mut out = HashMap::new();
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    for (n, k, o) in q.iter(&job.world) {
        if k.is_opening() {
            if let Some(w) = typed(&tree.opening_types, &o.type_name, "a door or window").ok().and_then(|(_, t)| t.width.feet()) {
                out.insert(n.path.clone(), w);
            }
        }
    }
    out
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
    // A door or window to demolish in a wall that stays is filled in: it is no hole in
    // its wall when the work is done (call 150).
    let gone = crate::components::demolished(world);
    let tree = Tree::of(world);

    // Openings first: each one's size and area, and each wall's list of them.
    let mut holes: HashMap<String, Vec<Hole>> = HashMap::new();
    // In the order of their paths, whatever order the world holds them in: a wall's
    // list of openings, and so what its figures cite, is the same every time.
    let mut q = world.query::<(&Node, &Kind, Option<&Opening>, Option<&Cost>)>();
    let mut openings: Vec<(&Node, &Kind, Option<&Opening>, Option<&Cost>)> = q.iter(world).collect();
    openings.sort_by_key(|(n, ..)| path_number(&n.path));
    for (node, kind, opening, cost) in openings {
        if !kind.is_opening() {
            continue;
        }
        let me = &node.path;
        let mut m = BTreeMap::new();
        let hole = match opening {
            None => {
                let input = [format!("{me}#{K_OPENING}")];
                for (name, unit) in kind.measures().iter().filter(|(n, _)| !crate::trim::GIVEN_WHERE_NAMED.contains(n)) {
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
        // Its casing, where a line of it names it (call 185).
        if crate::trim::names(cost, crate::trim::CASING_LENGTH) {
            let casing = crate::trim::casing(*kind, m.get("width"), m.get("height"), &hole.inputs);
            m.insert(crate::trim::CASING_LENGTH.into(), casing);
        }
        out.entry(me.clone()).or_default().extend(m);
        if let Some(parent) = node.parent.as_ref().filter(|_| !gone.contains(me)) {
            holes.entry(parent.clone()).or_default().push(hole);
        }
    }

    // Every strip footing's level, line and width: where one stops on the run of
    // another (a T), the part of it inside the other is that other's concrete.
    let strips: Vec<footings::Strip> = {
        let mut fq = world.query::<(&Node, &Kind, &Wall)>();
        fq.iter(world)
            .filter(|(_, k, _)| **k == Kind::Footing)
            .filter_map(|(n, _, w)| {
                let line = w.line_ft()?;
                let width = typed(&tree.wall_types, &w.type_name, "a footing").ok().and_then(|(_, t)| t.thickness.feet())?;
                let order = n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u32::MAX);
                Some(footings::Strip { path: n.path.clone(), level: n.parent.clone(), line, width, depth: w.height.as_ref().and_then(|h| h.feet()), top: w.base_offset.as_ref().and_then(|o| o.feet()).unwrap_or(0.0), order })
            })
            .collect()
    };
    // Every thing set in place, for a countertop to see what stands under it (call 162).
    let set_in_place: Vec<(String, String, Option<String>, Placed)> = {
        let mut sq = world.query::<(&Node, &Kind, &Placed)>();
        sq.iter(world).filter(|(_, k, _)| matches!(**k, Kind::Furnishing | Kind::Equipment | Kind::Fixture)).map(|(n, _, p)| (n.path.clone(), p.what.value.clone().unwrap_or_else(|| n.name.clone()), tree.storey_of(&n.path).map(str::to_string), p.clone())).collect()
    };
    // Each wall's outside corners, where a line of it names them: worked from every wall of its level (call 185).
    let corner_figures = {
        let wanted = crate::trim::naming(world, &[Kind::Wall], &[crate::trim::OUTSIDE_CORNERS, crate::trim::CORNER_BOARD_LENGTH]);
        if wanted.is_empty() { HashMap::new() } else { crate::trim::corners(world, Some(&wanted)) }
    };
    let mut q = world.query::<(&Node, &Kind, Option<&Wall>, Option<&Floor>, Option<&Segment>, Option<&Cost>, Option<&MadeOf>, Option<&Condition>, Option<&Placed>, Option<&Stood>, Option<&JoistsSpan>)>();
    for (node, kind, wall, floor, segment, cost, made_of, condition, placed, column, joists_span) in q.iter(world) {
        let me = &node.path;
        let mut m = BTreeMap::new();
        match kind {
            Kind::Wall => {
                let input = [format!("{me}#{K_WALL}")];
                let f_len = format!("length of the wall's line on the plan{}", wall.filter(|w| w.line.unit == units::IN).map(|_| " ÷ 12 (entered in inches)").unwrap_or_default());
                let Some(w) = wall else {
                    for (name, unit) in kind.measures().iter().filter(|(n, _)| !crate::trim::GIVEN_WHERE_NAMED.contains(n)) {
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
                // Its length less what comes down to its floor (call 175): a door, and a window
                // whose sill is at the floor. Given only where a line of this wall's names it,
                // as the frame's figures are: a wall that prices nothing on it has no new figure.
                let names_net_length = cost.is_some_and(|c| c.lines.iter().any(|l| !l.off && crate::formula::parse(&l.quantity).is_ok_and(|e| crate::formula::names(&e).iter().any(|n| n == "net_length"))));
                let net_length = if !names_net_length {
                    None
                } else {
                    let f_nl = "plan_length − the widths of the doors and of the windows whose sill is at the floor";
                    Some(match (&fault, length) {
                        _ if unit_fault => Derived::refused("LF", f_nl, &inputs, &format!("the wall line {NOT_CONVERTED}")),
                        (Some(why), _) => Derived::refused("LF", f_nl, &inputs, why),
                        (None, None) => Derived::refused("LF", f_nl, &inputs, "the wall has no length"),
                        (None, Some(l)) => match mine.iter().find(|h| h.span.is_none()) {
                            Some(h) => Derived::refused("LF", f_nl, &inputs, &format!("{} has no size, so how much of the wall it takes is not known", h.path)),
                            None => {
                                let cut: f64 = mine.iter().filter_map(|h| h.span).filter(|s| s[2] <= crate::check::FIT_TOLERANCE_FT).map(|s| s[1] - s[0]).sum();
                                let mut s: Vec<Status> = mine.iter().map(|h| h.status).collect();
                                s.push(w.line.status);
                                Derived::new(Some(l - cut), "LF", &s, f_nl, &inputs)
                            }
                        },
                    })
                };
                // The net area of the wall below a height a line of it names (call 179, C2): its face up
                // to that height, less the parts of its openings that lie below it. A stone wainscot's
                // 3 ft is such a height, and the siding is the wall's net area less this. The height is
                // a length entered on the line (`net_area_below_wainscot` takes the line's `wainscot`);
                // it is as sure as that entry. Given only where a line names it.
                let mut below: Vec<(String, Derived)> = Vec::new();
                for l in cost.iter().flat_map(|c| c.lines.iter()).filter(|l| !l.off) {
                    let Ok(e) = crate::formula::parse(&l.quantity) else { continue };
                    for name in crate::formula::names(&e).into_iter().filter(|n| n.starts_with(crate::formula::BELOW)) {
                        if below.iter().any(|(n, _)| *n == name) {
                            // A second line with the same name: the two have to give one height.
                            let first = below.iter().position(|(n, _)| *n == name).unwrap_or(0);
                            let height_of = |x: &crate::components::CostLine| x.inputs.get(&name[crate::formula::BELOW.len()..]).map(|v| v.feet());
                            let one = cost.iter().flat_map(|c| c.lines.iter()).filter(|x| !x.off && crate::formula::parse(&x.quantity).is_ok_and(|e| crate::formula::names(&e).contains(&name))).map(height_of).collect::<Vec<_>>();
                            if one.windows(2).any(|w| w[0] != w[1]) {
                                let f_below = "the face up to that height less the parts of its openings below it";
                                below[first].1 = Derived::refused("SF", f_below, &inputs, &format!("two lines of this wall give {:?} two heights", &name[crate::formula::BELOW.len()..]));
                            }
                            continue;
                        }
                        let input_name = &name[crate::formula::BELOW.len()..];
                        let f_below = "the face up to that height less the parts of its openings below it";
                        let figure = match l.inputs.get(input_name) {
                            None => Derived::refused("SF", f_below, &inputs, &format!("{:?} is not a height entered on the line", crate::view::words(input_name))),
                            Some(hv) => match (&fault, length, hv.feet()) {
                                _ if unit_fault => Derived::refused("SF", f_below, &inputs, &format!("the wall line {NOT_CONVERTED}")),
                                (Some(why), _, _) => Derived::refused("SF", f_below, &inputs, why),
                                (None, None, _) => Derived::refused("SF", f_below, &inputs, "the wall has no length"),
                                (None, _, None) => Derived::refused("SF", f_below, &inputs, &format!("the height {:?} is not known", crate::view::words(input_name))),
                                (None, Some(l_ft), Some(h_ft)) => match mine.iter().find(|h| h.span.is_none()) {
                                    Some(h) => Derived::refused("SF", f_below, &inputs, &format!("{} has no size, so how much of the wall it takes is not known", h.path)),
                                    None => {
                                        let cut: f64 = mine.iter().filter_map(|h| h.span).map(|[from, to, sill, head]| (to - from) * (head.clamp(0.0, h_ft) - sill.clamp(0.0, h_ft))).sum();
                                        let mut st: Vec<Status> = mine.iter().map(|h| h.status).collect();
                                        st.extend([w.line.status, hv.status]);
                                        Derived::new(Some(l_ft * h_ft - cut), "SF", &st, f_below, &inputs)
                                    }
                                },
                            },
                        };
                        below.push((name, figure));
                    }
                }
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
                if let Some(nl) = net_length {
                    m.insert("net_length".into(), nl);
                }
                for (name, figure) in below {
                    m.insert(name, figure);
                }
                // How thick it is: its type's thickness, as sure as the type and the
                // wall's choice of it. A foundation wall's concrete is net_area ×
                // thickness (call 130), worked here and not typed beside the wall.
                let f = "the thickness its type gives";
                let mut t_inputs = input.to_vec();
                let thick = match typed(&tree.wall_types, &w.type_name, "a wall") {
                    Ok((type_path, t)) => {
                        t_inputs.push(format!("{type_path}#{K_WALL_TYPE}"));
                        match t.thickness.feet() {
                            Some(x) => Derived::new(Some(x), "ft", &[w.type_name.status, t.thickness.status], f, &t_inputs),
                            None => Derived::refused("ft", f, &t_inputs, "its type's thickness is not known"),
                        }
                    }
                    Err(why) => Derived::refused("ft", f, &t_inputs, &why),
                };
                m.insert("thickness".into(), thick);
                // Its outside corners and the corner board they take, where a line of it names them (call 185).
                if let Some([n, l]) = corner_figures.get(me) {
                    m.insert(crate::trim::OUTSIDE_CORNERS.into(), n.clone());
                    m.insert(crate::trim::CORNER_BOARD_LENGTH.into(), l.clone());
                }
                // Its frame, counted, where what is priced on it names the lumber and the
                // spacing it is framed with (call 137): the frame the window draws, laid out
                // by the same rule (`framing::frame`), from the same line (`finish::framing_of`).
                // Not given at all where nothing names a framing: a wall framed by nobody is
                // not a wall with no studs.
                if let Some((_, spacing)) = crate::finish::framing_of(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str)) {
                    let mut f_inputs = inputs.clone();
                    f_inputs.push(format!("{me}#{K_COST}"));
                    let names = [("studs", "EA", "every stud, king, jack and cripple of its frame, one stud each; a corner's and an intersection's extra studs not counted"), ("plate_length", "LF", "its bottom plate and two top plates"), ("header_length", "LF", "a header over each opening, jack to jack"), ("sill_length", "LF", "a rough sill under each window")];
                    // An opening that does not fit its wall (past an end, over the top, over
                    // another) is not framed as if it were cut to fit: the frame is refused,
                    // as the wall's net area is, with the same reason (judged 2026-11-18). Nor
                    // is one whose head is in the top plates: it has no room for a header.
                    let misfits: Vec<&String> = faults.iter().filter(|f| !f.starts_with("where ")).collect();
                    let plates = 2.0 * 1.5 / 12.0;
                    let why = match (length, &heights.height) {
                        _ if unit_fault => Some(format!("the wall line {NOT_CONVERTED}")),
                        (None, _) => Some("the wall has no length".to_string()),
                        (_, Err(why)) => Some(why.clone()),
                        _ if !misfits.is_empty() => Some(misfits.iter().map(|f| f.as_str()).collect::<Vec<_>>().join("; ")),
                        (_, Ok(h)) => mine
                            .iter()
                            .find(|o| o.span.is_none())
                            .map(|o| format!("where {} is in the wall is not known", o.path))
                            .or_else(|| mine.iter().find(|o| o.span.is_some_and(|sp| sp[3] > h - plates + 1e-9)).map(|o| format!("the head of {} is in the top plates: there is no room for its header", o.path))),
                    };
                    match (why, length, &heights.height) {
                        (None, Some(l), Ok(h)) => {
                            let spans: Vec<[f64; 4]> = mine.iter().filter_map(|h| h.span).collect();
                            let tally = framing::tally(&framing::frame(l, *h, &spans, spacing));
                            let of = |pieces: &[framing::Piece], count: bool| -> f64 { tally.iter().filter(|(p, _, _)| pieces.contains(p)).map(|(_, n, len)| if count { *n as f64 } else { *len }).sum() };
                            use framing::Piece::*;
                            let figures = [of(&[Stud, King, Jack, Cripple], true), of(&[BottomPlate, TopPlate], false), of(&[Header], false), of(&[Sill], false)];
                            let mut statuses: Vec<Status> = mine.iter().map(|h| h.status).collect();
                            statuses.extend([w.line.status, heights.status]);
                            for ((name, unit, f), v) in names.iter().zip(figures) {
                                // The count rests on a rule as well as on the wall: studs on
                                // the spacing the line names, the corners left to the
                                // allowance. Assumed, and its formula says what is not in it.
                                let mut st = statuses.clone();
                                if *name == "studs" {
                                    st.push(Status::Assumed);
                                }
                                m.insert(name.to_string(), Derived::new(Some(v), unit, &st, f, &f_inputs));
                            }
                        }
                        (why, _, _) => {
                            let why = why.unwrap_or_else(|| "the frame cannot be laid out".to_string());
                            for (name, unit, f) in names {
                                m.insert(name.to_string(), Derived::refused(unit, f, &f_inputs, &why));
                            }
                        }
                    }
                }
            }
            // A strip footing: its length on plan, its width from its type, how deep it
            // is, and the concrete in it. Length times width times depth is the volume of
            // a ring of footings exactly: where two meet at a corner point they overlap
            // in a square and leave the same square open outside it.
            Kind::Footing => {
                let input = [format!("{me}#{K_WALL}")];
                let Some(w) = wall else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no footing line entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                let line = w.line_ft();
                let length = line.map(|[a, b]| geom::dist(a, b));
                let f_len = "length of the footing's line on the plan";
                let plan = if w.line.value.is_some() && line.is_none() { Derived::refused("LF", f_len, &input, &format!("the footing's line {NOT_CONVERTED}")) } else { Derived::new(length, "LF", &[w.line.status], f_len, &input) };
                let deep = match w.height.as_ref().map(|h| (h.feet(), h.status)) {
                    Some((Some(d), status)) => Derived::new(Some(d), "ft", &[status], "its depth, as entered", &input),
                    Some((None, _)) => Derived::refused("ft", "its depth, as entered", &input, "its depth is not known"),
                    None => Derived::refused("ft", "its depth, as entered", &input, "no depth entered"),
                };
                let mut inputs = input.to_vec();
                let wide = match typed(&tree.wall_types, &w.type_name, "a footing") {
                    Ok((type_path, t)) => {
                        inputs.push(format!("{type_path}#{K_WALL_TYPE}"));
                        match t.thickness.feet() {
                            Some(x) => Derived::new(Some(x), "ft", &[w.type_name.status, t.thickness.status], "the width its type gives", &inputs),
                            None => Derived::refused("ft", "the width its type gives", &inputs, "its type's width is not known"),
                        }
                    }
                    Err(why) => Derived::refused("ft", "the width its type gives", &inputs, &why),
                };
                // What of it lies in a footing drawn before it is that footing's concrete
                // and is taken off this one's, and the formula says so: worked exactly
                // from the two rectangles on plan (`footings`). A plain corner takes
                // nothing off: there the square the two share is the square left open.
                // (J20-1: a 20 ft stem on a 40 ft run, 24 by 12, is 118 CF by hand and
                // was 120. J21-1, J21-3, J21-4: a cross, three at a point, a stem part
                // way across and a stem deeper than its run were still counted twice.)
                let (taken, within) = match strips.iter().find(|s| s.path == *me) {
                    Some(mine) => footings::counted_elsewhere(mine, &strips),
                    None => (0.0, Vec::new()),
                };
                for other in &within {
                    inputs.push(format!("{other}#{K_WALL}"));
                }
                let f_vol = if within.is_empty() { "plan_length × width × depth ÷ 27".to_string() } else { format!("(plan_length × width × depth − {} CF that lies in {} and is counted there) ÷ 27", view::figure_shown(taken, 4), within.join(" and ")) };
                let f_vol = f_vol.as_str();
                // From the three as they were worked, not as they are written (call 133):
                // 20 in by 10 in is 20/12 by 10/12, not 1.6667 by 0.8333.
                let volume = match (plan.worked(), wide.worked(), deep.worked()) {
                    (Some(l), Some(x), Some(d)) => Derived::new(Some((l * x * d - taken).max(0.0) / 27.0), "CY", &[plan.status, wide.status, deep.status], f_vol, &inputs),
                    _ => {
                        let why = [&plan, &wide, &deep].iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default();
                        Derived::refused("CY", f_vol, &inputs, &why)
                    }
                };
                m.insert("plan_length".into(), plan);
                m.insert("width".into(), wide);
                m.insert("depth".into(), deep);
                m.insert("volume".into(), volume);
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
                // The distance round it on the plan: a slab's edge form, its thickened
                // edge, the sill seal on it (call 130).
                let f = format!("the distance round the boundary on the plan{}", floor.filter(|f| f.boundary.unit == units::IN).map(|_| " ÷ 12 (entered in inches)").unwrap_or_default());
                m.insert(
                    "perimeter".into(),
                    match floor {
                        None => Derived::refused("LF", &f, &input, "no boundary entered"),
                        Some(fl) if fl.boundary.value.is_some() && fl.boundary_ft().is_none() => Derived::refused("LF", &f, &input, &format!("the boundary {NOT_CONVERTED}")),
                        Some(fl) => Derived::new(fl.boundary_ft().map(|p| geom::perimeter(&p)), "LF", &[fl.boundary.status], &f, &input),
                    },
                );
                // How thick it is: its type's thickness, as sure as the type and the
                // floor's choice of it. A slab's concrete is plan_area × thickness (call 130).
                let f = "the thickness its type gives";
                let mut inputs = input.to_vec();
                let thick = match floor.map(|fl| (fl, typed(&tree.floor_types, &fl.type_name, "a floor"))) {
                    None => Derived::refused("ft", f, &inputs, "no boundary entered"),
                    Some((_, Err(why))) => Derived::refused("ft", f, &inputs, &why),
                    Some((fl, Ok((type_path, t)))) => {
                        inputs.push(format!("{type_path}#{K_FLOOR_TYPE}"));
                        match t.thickness.feet() {
                            Some(x) => Derived::new(Some(x), "ft", &[fl.type_name.status, t.thickness.status], f, &inputs),
                            None => Derived::refused("ft", f, &inputs, "its type's thickness is not known"),
                        }
                    }
                };
                m.insert("thickness".into(), thick);
                // Its joists, counted where what is priced on it names the lumber and the
                // spacing they are framed with (call 165), by the rule a wall's studs are
                // (`finish::members_named`); not given at all where nothing names joists.
                if let Some((_, spacing)) = crate::finish::members_named(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str), &["joist"]) {
                    let mut f_inputs = input.to_vec();
                    f_inputs.push(format!("{me}#{K_COST}"));
                    if joists_span.is_some() {
                        f_inputs.push(format!("{me}#{K_JOISTS_SPAN}"));
                    }
                    m.extend(joist_figures(floor, joists_span, spacing, &f_inputs));
                }
            }
            k if k.is_segment() || *k == Kind::Beam => {
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
                // A beam's figures (call 164): its length along it, its section from its
                // type, and what it weighs where its type says what a foot of it weighs.
                if *k == Kind::Beam {
                    let formula = "length × its type's weight per foot";
                    match typed(&tree.segment_types, &r.type_name, "a beam") {
                        Err(why) => {
                            for (name, unit, f) in [("width", "ft", "its type's width"), ("depth", "ft", "its type's depth"), ("weight", "LB", formula), ("section_area", "SF", "width × depth"), ("volume", "CY", "section_area × length ÷ 27")] {
                                m.insert(name.into(), Derived::refused(unit, f, &input, &why));
                            }
                        }
                        Ok((type_path, t)) => {
                            let inputs = [input[0].clone(), format!("{type_path}#{K_SEGMENT_TYPE}")];
                            let sure = r.type_name.status;
                            let size = |v: Option<&Val<Len>>, what: &str| match v {
                                Some(v) => match v.feet() {
                                    Some(x) => Derived::new(Some(x), "ft", &[sure, v.status], &units::to_feet_text(what, &v.unit), &inputs),
                                    None => Derived::refused("ft", what, &inputs, &format!("{what} is not known")),
                                },
                                None => Derived::refused("ft", what, &inputs, &format!("{what} is not known")),
                            };
                            let wide = size(Some(&t.width), if t.round { "its type's diameter" } else { "its type's width" });
                            let deep = if t.round { size(Some(&t.width), "its type's diameter") } else { size(t.depth.as_ref(), "its type's depth") };
                            // What the BIM file's base quantities are (BIMCraft `member_quantities`).
                            let (area, volume) = section_figures(&wide, &deep, t.round, &len_d, &inputs);
                            m.insert("width".into(), wide);
                            m.insert("depth".into(), deep);
                            m.insert("section_area".into(), area);
                            m.insert("volume".into(), volume);
                            let weight = match (&t.weight_per_foot, len_d.worked()) {
                                (None, _) => Derived::refused("LB", formula, &inputs, "its type gives no weight per foot: type what a foot of it weighs, from its shape's name or its maker's sheet"),
                                (Some(w), _) if w.value.is_none() => Derived::refused("LB", formula, &inputs, "its type's weight per foot is not known"),
                                (Some(_), None) => Derived::refused("LB", formula, &inputs, &len_d.note),
                                (Some(w), Some(l)) => Derived::new(w.value.map(|per| l * per), "LB", &[len_d.status, sure, w.status], formula, &inputs),
                            };
                            m.insert("weight".into(), weight);
                        }
                    }
                    m.insert("length".into(), len_d);
                } else {
                    m.insert("true_length".into(), len_d);
                }
            }
            // A post, and a pad footing, which is a post laid flat (call 164): where it
            // stands, how tall (how thick), and its section from its type.
            Kind::Post | Kind::PadFooting => {
                let input = [format!("{me}#{K_COLUMN}")];
                let post = *kind == Kind::Post;
                let Some(c) = column else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no position entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                let tall = match c.height.feet() {
                    Some(h) => Derived::new(Some(h), if post { "LF" } else { "ft" }, &[c.height.status], if post { "its height, as entered" } else { "its thickness, as entered" }, &input),
                    None => Derived::refused(if post { "LF" } else { "ft" }, if post { "its height, as entered" } else { "its thickness, as entered" }, &input, if post { "its height is not known" } else { "its thickness is not known" }),
                };
                let mut inputs = input.to_vec();
                // The section its type gives: across and deep (a round one is as deep as it is wide).
                let section: Result<(Derived, Derived), String> = match typed(&tree.segment_types, &c.type_name, if post { "a post" } else { "a pad footing" }) {
                    Err(why) => Err(why),
                    Ok((type_path, t)) => {
                        inputs.push(format!("{type_path}#{K_SEGMENT_TYPE}"));
                        let sure = c.type_name.status;
                        let one = |v: Option<&Val<Len>>, what: &str, inputs: &[String]| match v.and_then(|v| v.feet().map(|x| (x, v.status))) {
                            Some((x, status)) => Derived::new(Some(x), "ft", &[sure, status], &format!("the {what} its type gives"), inputs),
                            None => Derived::refused("ft", &format!("the {what} its type gives"), inputs, &format!("its type's {what} is not known")),
                        };
                        if t.round {
                            Ok((one(Some(&t.width), "diameter", &inputs), one(Some(&t.width), "diameter", &inputs)))
                        } else {
                            Ok((one(Some(&t.width), "width", &inputs), one(t.depth.as_ref(), "depth", &inputs)))
                        }
                    }
                };
                match (section, post) {
                    (Err(why), true) => {
                        m.insert("width".into(), Derived::refused("ft", "the width its type gives", &inputs, &why));
                        m.insert("depth".into(), Derived::refused("ft", "the depth its type gives", &inputs, &why));
                        m.insert("section_area".into(), Derived::refused("SF", "width × depth", &inputs, &why));
                        m.insert("volume".into(), Derived::refused("CY", "section_area × height ÷ 27", &inputs, &why));
                        m.insert("height".into(), tall);
                    }
                    (Ok((w, d)), true) => {
                        let round = typed(&tree.segment_types, &c.type_name, "a post").ok().is_some_and(|(_, t)| t.round);
                        let (area, volume) = section_figures(&w, &d, round, &tall, &inputs);
                        m.insert("width".into(), w);
                        m.insert("depth".into(), d);
                        m.insert("section_area".into(), area);
                        m.insert("volume".into(), volume);
                        m.insert("height".into(), tall);
                    }
                    (section, false) => {
                        // A pad: its sides are 2 × (width + length) × thickness of form, its
                        // concrete the three multiplied, in cubic yards. Each from the sizes as
                        // they were worked, so 36 in by 36 in by 18 in is 3 by 3 by 1.5.
                        let (w, l) = match section {
                            Ok(x) => x,
                            Err(why) => (Derived::refused("ft", "the width its type gives", &inputs, &why), Derived::refused("ft", "the depth its type gives", &inputs, &why)),
                        };
                        let figure = |name: &str, unit: &str, f: &str, list: &[&Derived], by: &dyn Fn(f64, f64, f64) -> f64| match (w.worked(), l.worked(), tall.worked()) {
                            (Some(a), Some(b), Some(t)) => Derived::new(Some(by(a, b, t)), unit, &list.iter().map(|d| d.status).collect::<Vec<_>>(), f, &inputs),
                            _ => {
                                let why = list.iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default();
                                let _ = name;
                                Derived::refused(unit, f, &inputs, &why)
                            }
                        };
                        let area = figure("plan_area", "SF", "width × length", &[&w, &l], &|a, b, _| a * b);
                        let form = figure("form_area", "SF", "2 × (width + length) × thickness", &[&w, &l, &tall], &|a, b, t| 2.0 * (a + b) * t);
                        let volume = figure("volume", "CY", "width × length × thickness ÷ 27", &[&w, &l, &tall], &|a, b, t| a * b * t / 27.0);
                        m.insert("width".into(), w);
                        m.insert("length".into(), l);
                        m.insert("thickness".into(), tall);
                        m.insert("plan_area".into(), area);
                        m.insert("form_area".into(), form);
                        m.insert("volume".into(), volume);
                    }
                }
            }
            // A thing set in place: the figures of its size, as sure as its size (call 155).
            Kind::Equipment | Kind::Fixture | Kind::Furnishing => {
                let input = [format!("{me}#{K_PLACED}")];
                let (f_w, f_d, f_h, f_a) = ("its width, along its own x before it is turned", "its depth, from its front to its back", "its height, from its underside", "width × depth");
                match placed {
                    None => {
                        for (name, unit) in kind.measures() {
                            m.insert(name.to_string(), Derived::refused(unit, "", &input, "no position and size entered"));
                        }
                    }
                    Some(p) => match p.size_ft() {
                        Some([w, d, h]) => {
                            let st = [p.size.status];
                            let width = Derived::new(Some(w), "ft", &st, f_w, &input);
                            let depth = Derived::new(Some(d), "ft", &st, f_d, &input);
                            m.insert("height".into(), Derived::new(Some(h), "ft", &st, f_h, &input));
                            // From the sizes as they were worked, so 24 in by 25 1/2 in is 2 by 2.125.
                            let mut area = Derived::new(Some(w * d), "SF", &[width.status, depth.status], f_a, &input);
                            // A countertop over a box that turns a corner does not have the leg
                            // that turns it: the area does not move, and says so (call 162).
                            if p.what.value.as_deref().is_some_and(|what| what.eq_ignore_ascii_case("Countertop")) {
                                area.note = counter_legs_said(me, p, [w, d, h], &set_in_place).join("; ");
                            }
                            m.insert("plan_area".into(), area);
                            m.insert("width".into(), width);
                            m.insert("depth".into(), depth);
                        }
                        None => {
                            let why = if p.size.value.is_some() { format!("its size {NOT_CONVERTED}") } else { format!("it has no size: {}", p.size.source) };
                            for (name, unit) in kind.measures() {
                                m.insert(name.to_string(), Derived::refused(unit, "", &input, &why));
                            }
                        }
                    },
                }
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

/// Which way a floor's joists run with what was said of it, for what a wall carries
/// (call 176): the same rule as `joist_run`, read from it, so that a count of joists, the
/// frame as drawn and what crosses a wall can never disagree. `said` is None where the
/// way was not said (the short way is then taken).
pub(crate) fn joists_run(ring: &[P2], span: Option<&JoistsSpan>) -> Option<(P2, Option<(bool, Status)>)> {
    joist_run(ring, span).map(|(run, said, status)| (run, said.map(|long| (long, status))))
}

/// What is not in a count of joists or rafters, said on every figure of it (call 165).
pub const NOT_COUNTED: &str = "blocking, hangers and doubled members are not in this count";

/// Which way a floor's joists run, from its boundary and what is said of it: the way (a
/// direction on the plan), whether anything was said, and how sure that is. Square to the
/// longest edge (the short way) unless the floor says the long way; not said, the short way,
/// Assumed. One rule, for what is counted and for what is drawn (call 177).
pub(crate) fn joist_run(ring: &[P2], span: Option<&JoistsSpan>) -> Option<(P2, Option<bool>, Status)> {
    let longest = (0..ring.len()).map(|i| (ring[i], ring[(i + 1) % ring.len()])).max_by(|a, b| geom::dist(a.0, a.1).total_cmp(&geom::dist(b.0, b.1)));
    let (p, q) = longest.filter(|(p, q)| geom::dist(*p, *q) > 1.0e-9)?;
    let along = [q[0] - p[0], q[1] - p[1]];
    let said = span.and_then(|s| s.0.value.as_deref().map(|w| (w == LONG_WAY, s.0.status)));
    let (long_way, way_status) = said.unwrap_or((false, Status::Assumed));
    let run = if long_way { along } else { [-along[1], along[0]] };
    Some((run, said.map(|(long, _)| long), way_status))
}

/// A floor's joists counted across its boundary (call 165): `joists`, `joist_length`,
/// `joist_longest` and `rim_length`. They run square to the longest edge (the short way) unless
/// the floor says the long way; not said, the short way is taken, Assumed, and the figures say so.
pub(crate) fn joist_figures(floor: Option<&Floor>, span: Option<&JoistsSpan>, spacing: f64, inputs: &[String]) -> BTreeMap<String, Derived> {
    const NAMES: [(&str, &str, &str); 4] = [
        ("joists", "EA", "one joist at each multiple of the spacing from the first side, and one inside each edge that runs with them, each cut to the boundary"),
        ("joist_length", "LF", "the lengths of its joists, added"),
        ("joist_longest", "ft", "the longest of its joists: the stock they are bought in is that length or the next up"),
        ("rim_length", "LF", "the boundary's edges measured square to the joists: the band their ends meet"),
    ];
    let refused = |why: &str| NAMES.iter().map(|(n, u, f)| (n.to_string(), Derived::refused(u, f, inputs, why))).collect::<BTreeMap<_, _>>();
    let Some(floor) = floor else { return refused("no boundary entered") };
    let Some(ring) = floor.boundary_ft() else {
        return refused(if floor.boundary.value.is_some() { "the boundary is in a unit this program does not convert" } else { "no boundary entered" });
    };
    // Which way they run: square to the longest edge, or along it.
    let Some((run, said, way_status)) = joist_run(&ring, span) else { return refused("the boundary has no length") };
    let mut note = Vec::new();
    if said.is_none() {
        note.push("joists usually span the short way: say which way these run".to_string());
    }
    let layout = members::members_across(&ring, run, spacing).and_then(|a| members::rim_length(&ring, run).map(|r| (a, r)));
    match layout {
        Err(why) => refused(&why),
        Ok((across, rim)) => {
            if across.left_out > 0 {
                note.push(format!("{} piece{} under 6 in left out", across.left_out, if across.left_out == 1 { "" } else { "s" }));
            }
            note.push(NOT_COUNTED.to_string());
            let status = [floor.boundary.status, way_status];
            let figures = [across.count() as f64, across.total(), across.longest(), rim];
            NAMES
                .iter()
                .zip(figures)
                .map(|((n, u, f), v)| {
                    let mut d = Derived::new(Some(v), u, &status, f, inputs);
                    d.note = note.join("; ");
                    (n.to_string(), d)
                })
                .collect()
        }
    }
}

/// The area of a beam's or a post's section, and its volume over its length, from the two
/// sizes as they were worked: a round section is a circle across its width, a rectangular
/// one its width by its depth (BIMCraft `member_quantities`, `ifc.rs:707`). `along` is its
/// length (a post's height).
fn section_figures(wide: &Derived, deep: &Derived, round: bool, along: &Derived, inputs: &[String]) -> (Derived, Derived) {
    let statuses = [wide.status, deep.status];
    let f_area = if round { "π × diameter² ÷ 4" } else { "width × depth" };
    let area = match (wide.worked(), deep.worked()) {
        (Some(w), Some(d)) => Derived::new(Some(if round { std::f64::consts::PI * w * w / 4.0 } else { w * d }), "SF", &statuses, f_area, inputs),
        _ => Derived::refused("SF", f_area, inputs, &[wide, deep].iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default()),
    };
    let f_vol = "section_area × its length ÷ 27";
    let volume = match (area.worked(), along.worked()) {
        (Some(a), Some(l)) => Derived::new(Some(a * l / 27.0), "CY", &[area.status, along.status], f_vol, inputs),
        _ => Derived::refused("CY", f_vol, inputs, &[&area, along].iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default()),
    };
    (area, volume)
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

/// A wall's face on one room's side: where it is, which way the room lies from it, and
/// how much of it is clear to stand things against (call 155; BIMCraft hosts a thing on a
/// wall by the wall's path and a distance along it, `commands.rs` `host`: here the room's
/// outline at the faces of its walls gives the face and its corners).
#[derive(Clone, Debug, PartialEq)]
pub struct WallFace {
    /// The wall, and the room, as people call them.
    pub wall_called: String,
    pub room_called: String,
    /// The wall's line, first end and second, in feet on the plan.
    pub start: P2,
    pub end: P2,
    /// The way the wall runs, a unit vector, and the way the room lies from the face.
    pub along: P2,
    pub into: P2,
    /// How far the face is from the wall's line.
    pub off: f64,
    /// The stretches of the face along the wall's line, in feet from its first end, with
    /// no corner or cross wall in them, in order.
    pub clear: Vec<(f64, f64)>,
    /// The doors in the wall: path, and where each starts and ends along it.
    pub doors: Vec<(String, f64, f64)>,
    /// The windows in the wall that is to stay: path, what it is called, where it starts and
    /// ends along the wall, and its sill and head in feet above the wall's base (call 157).
    pub windows: Vec<(String, String, f64, f64, f64, f64)>,
    /// As sure as the wall's line and thickness.
    pub status: Status,
}

/// The face of `wall` that `room` has, or why it has none. Both must be on `storey`.
pub fn wall_face(job: &mut Job, storey: &str, wall: &str, room: &str) -> Result<WallFace, String> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    if job.kind(wall) != Some(Kind::Wall) {
        return Err(format!("{wall} is not a wall"));
    }
    if job.kind(room) != Some(Kind::Room) {
        return Err(format!("{room} is not a room"));
    }
    let (wall_called, room_called) = (tree.called(wall), tree.called(room));
    if tree.storey_of(wall) != Some(storey) {
        return Err(format!("{wall_called} ({wall}) is not on {}", tree.called(storey)));
    }
    if tree.storey_of(room) != Some(storey) {
        return Err(format!("{room_called} ({room}) is not on {}", tree.called(storey)));
    }
    let e = job.find(wall).ok_or_else(|| format!("no element {wall}"))?;
    let w = job.world.get::<Wall>(e).cloned().ok_or_else(|| format!("{wall_called} has no wall line entered"))?;
    let (pw, shape, _, _) = plan_wall(&tree, wall, &w).map_err(|why| format!("{wall_called} ({wall}) cannot be placed: {why}"))?;
    let outline = room_outlines(job).remove(room).ok_or_else(|| format!("no walls close round {room_called}'s point, so it has no faces to stand things against"))?;
    let (a, b) = ([pw.start.0, pw.start.1], [pw.end.0, pw.end.1]);
    let length = geom::dist(a, b);
    if length <= tol {
        return Err(format!("{wall_called} has no length"));
    }
    let u = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
    let n = [-u[1], u[0]];
    let reach = pw.ext.abs() + pw.int.abs() + tol;
    // The room's sides that lie along this wall, on one side of its line.
    let mut found: Vec<(f64, f64, f64)> = Vec::new();
    for i in 0..outline.len() {
        let (p, q) = (outline[i], outline[(i + 1) % outline.len()]);
        let (dp, dq) = ((p[0] - a[0]) * n[0] + (p[1] - a[1]) * n[1], (q[0] - a[0]) * n[0] + (q[1] - a[1]) * n[1]);
        if (dp - dq).abs() > tol || dp.abs() > reach || dp.abs() <= tol {
            continue;
        }
        let (sp, sq) = ((p[0] - a[0]) * u[0] + (p[1] - a[1]) * u[1], (q[0] - a[0]) * u[0] + (q[1] - a[1]) * u[1]);
        let (lo, hi) = (sp.min(sq).max(0.0), sp.max(sq).min(length));
        if hi - lo > tol {
            found.push((dp, lo, hi));
        }
    }
    let Some(&(first, _, _)) = found.first() else {
        return Err(format!("{wall_called} ({wall}) is not one of the walls round {room_called}: it has no face on that room's side"));
    };
    // Its first side's side of the wall; any that lie on the other side are left alone.
    let mut sides: Vec<(f64, f64)> = found.iter().filter(|(d, _, _)| (d.signum() - first.signum()).abs() < 0.5).map(|&(_, lo, hi)| (lo, hi)).collect();
    sides.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut clear: Vec<(f64, f64)> = Vec::new();
    for (lo, hi) in sides {
        match clear.last_mut() {
            Some(last) if lo <= last.1 + tol => last.1 = last.1.max(hi),
            _ => clear.push((lo, hi)),
        }
    }
    let gone = crate::components::demolished(&mut job.world);
    let mut doors: Vec<(String, f64, f64)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    let mut mine: Vec<(String, f64, f64, u64)> = Vec::new();
    for (node, kind, o) in q.iter(&job.world) {
        if *kind == Kind::Door && node.parent.as_deref() == Some(wall) && !gone.contains(&node.path) {
            if let Some([from, to, _, _]) = hole_of(&tree, &node.path, o).0.span {
                mine.push((node.path.clone(), from, to, path_number(&node.path)));
            }
        }
    }
    mine.sort_by_key(|m| m.3);
    doors.extend(mine.into_iter().map(|(p, f, t, _)| (p, f, t)));
    let mut windows: Vec<(String, String, f64, f64, f64, f64, u64)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    for (node, kind, o) in q.iter(&job.world) {
        if *kind == Kind::Window && node.parent.as_deref() == Some(wall) && !gone.contains(&node.path) {
            if let Some([from, to, sill, head]) = hole_of(&tree, &node.path, o).0.span {
                windows.push((node.path.clone(), tree.called(&node.path), from, to, sill, head, path_number(&node.path)));
            }
        }
    }
    windows.sort_by_key(|m| m.6);
    let windows = windows.into_iter().map(|(p, c, f, t, s, h, _)| (p, c, f, t, s, h)).collect();
    Ok(WallFace { wall_called, room_called, start: a, end: b, along: u, into: [first.signum() * n[0], first.signum() * n[1]], off: first.abs(), clear, doors, windows, status: shape })
}

/// What is set against a wall's faces and would stand when the wall is gone: each thing
/// set in place (furnishing, equipment, fixture) on the wall's storey that is not itself to
/// demolish, whose plan is along the wall and touches the wall's thickness or a face of it.
/// Path and name, in the order the job holds them (call 157). Nothing is moved or removed.
pub fn things_against(job: &mut Job, wall: &str) -> Result<Vec<(String, String)>, String> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    if job.kind(wall) != Some(Kind::Wall) {
        return Err(format!("{wall} is not a wall"));
    }
    let e = job.find(wall).ok_or_else(|| format!("no element {wall}"))?;
    let w = job.world.get::<Wall>(e).cloned().ok_or_else(|| format!("{wall} has no wall line entered"))?;
    let Ok((pw, _, _, _)) = plan_wall(&tree, wall, &w) else { return Ok(Vec::new()) };
    let storey = tree.storey_of(wall).map(str::to_string);
    let (a, b) = ([pw.start.0, pw.start.1], [pw.end.0, pw.end.1]);
    let length = geom::dist(a, b);
    if length <= tol {
        return Ok(Vec::new());
    }
    let u = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
    let n = [-u[1], u[0]];
    let band = pw.ext.abs().max(pw.int.abs()) + tol;
    let gone = crate::components::demolished(&mut job.world);
    let (drawn, _) = placements(job);
    let mut found = Vec::new();
    for p in drawn {
        let Shape::Box { at, size, angle } = p.shape else { continue };
        if !matches!(p.kind, Kind::Furnishing | Kind::Equipment | Kind::Fixture) || p.storey != storey || gone.contains(&p.path) {
            continue;
        }
        let (sin, cos) = angle.to_radians().sin_cos();
        let (mut s_lo, mut s_hi, mut d_lo, mut d_hi) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let (lx, ly) = (sx * size[0] / 2.0, sy * size[1] / 2.0);
            let (x, y) = (at[0] + lx * cos - ly * sin - a[0], at[1] + lx * sin + ly * cos - a[1]);
            let (s, d) = (x * u[0] + y * u[1], x * n[0] + y * n[1]);
            (s_lo, s_hi, d_lo, d_hi) = (s_lo.min(s), s_hi.max(s), d_lo.min(d), d_hi.max(d));
        }
        if s_hi.min(length) - s_lo.max(0.0) > tol && d_hi >= -band && d_lo <= band {
            found.push((p.path.clone(), p.name.clone()));
        }
    }
    Ok(found)
}

/// The ends of a room line that meet no wall and no other room line of its storey, in feet
/// on the plan: an end meets a wall where it lies in the wall's outline or on its edge, and
/// another room line where it lies on it. Such a line parts nothing until it does (call 157).
pub fn room_line_unreached(job: &mut Job, line: &str) -> Vec<P2> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let Some(storey) = tree.storey_of(line).map(str::to_string) else { return Vec::new() };
    let gone = crate::components::demolished(&mut job.world);
    let mine = job.find(line).and_then(|e| job.world.get::<RoomLine>(e).and_then(|r| r.line_ft()));
    let Some([a, b]) = mine else { return Vec::new() };
    let mut others: Vec<(V, V)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &RoomLine)>();
    for (n, k, r) in q.iter(&job.world) {
        if *k == Kind::RoomLine && n.path != line && tree.storey_of(&n.path) == Some(storey.as_str()) {
            if let Some([c, d]) = r.line_ft() {
                others.push((V(c[0], c[1]), V(d[0], d[1])));
            }
        }
    }
    let mut pws: Vec<walls::PlanWall> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &Wall)>();
    for (n, k, w) in q.iter(&job.world) {
        if *k == Kind::Wall && !gone.contains(&n.path) && tree.storey_of(&n.path) == Some(storey.as_str()) {
            if let Ok((pw, _, _, _)) = plan_wall(&tree, &n.path, w) {
                pws.push(pw);
            }
        }
    }
    let rings: Vec<Vec<V>> = (0..pws.len()).map(|i| walls::outline(&pws, i, tol)).collect();
    let reaches = |p: V| {
        rings.iter().any(|ring| rooms::contains(ring, p) || ring.iter().zip(ring.iter().cycle().skip(1)).any(|(x, y)| on_line(p, *x, *y, tol))) || others.iter().any(|(x, y)| on_line(p, *x, *y, tol))
    };
    [a, b].into_iter().filter(|p| !reaches(V(p[0], p[1]))).collect()
}

/// The number at the end of a path: paths are numbered in the order elements were added,
/// and a number is never given twice.
fn path_number(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

/// A wall as the walls round it see it on the plan; the worst status among what its line
/// and its thickness were worked out from; and the worst among what its base and top rest
/// on. Or why it cannot be placed.
pub(crate) fn plan_wall(tree: &Tree, me: &str, w: &Wall) -> Result<(walls::PlanWall, Status, Status, Status), String> {
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
    /// What stands free inside it, at its walls' faces: a room within the room, a
    /// chase. Its sides are the room's walls too (judged 2026-11-19).
    islands: Vec<Vec<crate::skeleton::P2>>,
    /// The room lines (by their place in the list given) with a side of the room along
    /// them: the room is measured to them (call 152).
    along: Vec<usize>,
}

/// Every room of one storey measured against `walls`, of which those at `bounding` bound
/// the storey's rooms, and `room_lines`, which part them where no wall stands. Each of `walls` can join the others where they meet (BIMCraft
/// `rooms.rs:62` and `walls.rs:238`: a bounding wall's outline is cut against every wall
/// it meets, bounding or not).
fn rooms_among(walls: &[walls::PlanWall], bounding: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], points: &[crate::skeleton::P2], tol: f64) -> Vec<RoomFigures> {
    use crate::skeleton::P2 as V;
    use rooms::{Plan, RoomArea};
    let (centre, finish) = (Plan::of_walls(walls, bounding, room_lines, RoomArea::Centre, tol), Plan::of_walls(walls, bounding, room_lines, RoomArea::Finish, tol));
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
    // The room lines a region has a side along.
    let along = |g: &rooms::Region| -> Vec<usize> {
        let mids: Vec<V> = std::iter::once(&g.outer).chain(g.holes.iter()).flat_map(|ring| ring.iter().zip(ring.iter().cycle().skip(1)).map(|(a, b)| (*a + *b) * 0.5)).collect();
        (0..room_lines.len()).filter(|&k| mids.iter().any(|m| on_line(*m, room_lines[k].0, room_lines[k].1, tol))).collect()
    };
    (0..points.len())
        .map(|i| RoomFigures {
            along: finish.bound(points, i, tol).map(|g| along(&g)).unwrap_or_default(),
            corners: finish.bound(points, i, tol).map(|g| std::iter::once(&g.outer).chain(g.holes.iter()).flatten().copied().collect()).unwrap_or_default(),
            outline: finish.bound(points, i, tol).map(|g| g.outer).unwrap_or_default(),
            islands: finish.bound(points, i, tol).map(|g| g.holes).unwrap_or_default(),
            finish: finish.bound(points, i, tol).map(|g| (g.area(), g.perimeter(), touching(&g, &outline_edges))),
            centre: centre.bound(points, i, tol).map(|g| (g.area(), touching(&g, &centre_lines))),
            in_wall: !points[i].0.is_nan() && outlines.iter().any(|ring| rooms::contains(ring, points[i])),
        })
        .collect()
}

/// Whether `m` lies on the line from `a` to `b`, to within `tol`.
fn on_line(m: crate::skeleton::P2, a: crate::skeleton::P2, b: crate::skeleton::P2, tol: f64) -> bool {
    let d = b - a;
    let t = if d.dot(d) > 0.0 { ((m - a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
    m.dist(a + d * t) <= tol
}

/// A room's wall area (call 144): each side of its outline at its walls' faces, as long as
/// it is and as tall as the wall it lies against, less the part of each door and window
/// in that wall that lies along the side. A door between two rooms is taken off both:
/// both its faces are cut. A side along a room line is no wall and takes nothing. With the
/// statuses and inputs of the openings it took off.
fn room_wall_area(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, tol: f64) -> Result<(f64, Vec<Status>, Vec<String>), String> {
    use crate::skeleton::P2 as V;
    if outline.len() < 3 {
        return Err("no loop of walls closes round its point".into());
    }
    // An opening that does not fit its wall (past an end, over its top) leaves the wall
    // with no net area, and the room is no surer of what it takes off (judged
    // 2026-11-19: the room's figure stood Known beside the wall's Unknown).
    for &k in mine {
        let (w, length) = (&walls[k], walls[k].start.dist(walls[k].end));
        for hole in holes.get(&placed[k].path).map(Vec::as_slice).unwrap_or(&[]) {
            if let Some([from, to, _, head]) = hole.span {
                if from < -tol || to > length + tol || head > w.top - w.base + tol {
                    return Err(format!("{} does not fit its wall, so what it takes off this room's walls is not known", hole.path));
                }
            }
        }
    }
    let rings: Vec<(usize, Vec<(V, V)>)> = mine.iter().map(|&k| {
        let ring = walls::outline(walls, k, tol);
        let edges = ring.iter().copied().zip(ring.iter().copied().cycle().skip(1)).collect();
        (k, edges)
    }).collect();
    let on = |m: V, a: V, b: V| {
        let d = b - a;
        let t = if d.dot(d) > 0.0 { ((m - a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
        m.dist(a + d * t) <= tol
    };
    let (mut total, mut statuses, mut cited) = (0.0, Vec::new(), Vec::new());
    let sides: Vec<(V, V)> = std::iter::once(outline).chain(islands.iter().map(Vec::as_slice)).flat_map(|ring| (0..ring.len()).map(move |i| (ring[i], ring[(i + 1) % ring.len()]))).collect();
    for (p, q) in sides {
        let length = p.dist(q);
        if length <= tol {
            continue;
        }
        let mid = (p + q) * 0.5;
        let Some(&(k, _)) = rings.iter().find(|(_, edges)| edges.iter().any(|(a, b)| on(mid, *a, *b))) else {
            // A side along a room line has no wall: it takes no wall area (call 152).
            if room_lines.iter().any(|(a, b)| on(mid, *a, *b)) {
                continue;
            }
            return Err(format!("the side of the room from ({}, {}) to ({}, {}) lies along no wall's face", view::num(p.0), view::num(p.1), view::num(q.0), view::num(q.1)));
        };
        let w = &walls[k];
        let tall = w.top - w.base;
        total += length * tall;
        let (s1, s2) = (w.project(p).0, w.project(q).0);
        let (lo, hi) = (s1.min(s2), s1.max(s2));
        for hole in holes.get(&placed[k].path).map(Vec::as_slice).unwrap_or(&[]) {
            let Some([from, to, sill, head]) = hole.span else {
                return Err(format!("where {} is in its wall is not known, so what it takes off this room's walls is not", hole.path));
            };
            let along = to.min(hi) - from.max(lo);
            if along > tol {
                total -= along * (head.min(tall) - sill.max(0.0)).max(0.0);
                statuses.push(hole.status);
                cited.extend(hole.inputs.iter().cloned());
            }
        }
    }
    Ok((total, statuses, cited))
}

/// The runs of wall along a room's floor line (calls 147 and 157): each ring of it (round
/// the room, and round what stands inside it) cut at its doors into runs; a side along a
/// room line cuts it as a door does (call 152); a ring with no door is one run all the way
/// round. A door to demolish is filled in and is no doorway. The length of each run, with
/// the statuses and inputs of the doors that cut it. Receptacle outlets (`room_receptacles`)
/// and the room's `wall_run` are both worked from these: one walk of the floor line.
#[allow(clippy::too_many_arguments)]
fn room_runs(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, against: &[&Standing], tol: f64) -> Result<(Vec<f64>, Vec<Status>, Vec<String>, Vec<usize>), String> {
    use crate::skeleton::P2 as V;
    // Which of `against` cover some of the floor line (call 162).
    let mut touched: Vec<usize> = Vec::new();
    let rings: Vec<(usize, Vec<(V, V)>)> = mine.iter().map(|&k| {
        let ring = walls::outline(walls, k, tol);
        let edges = ring.iter().copied().zip(ring.iter().copied().cycle().skip(1)).collect();
        (k, edges)
    }).collect();
    let on = |m: V, a: V, b: V| {
        let d = b - a;
        let t = if d.dot(d) > 0.0 { ((m - a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
        m.dist(a + d * t) <= tol
    };
    let (mut runs, mut statuses, mut cited) = (Vec::new(), Vec::new(), Vec::new());
    for ring in std::iter::once(outline).chain(islands.iter().map(Vec::as_slice)) {
        // The doorways along this ring, as stretches of its length from its first corner.
        let mut round = 0.0;
        let mut cuts: Vec<(f64, f64)> = Vec::new();
        for i in 0..ring.len() {
            let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
            let length = p.dist(q);
            if length > tol {
                let mid = (p + q) * 0.5;
                if let Some(&(k, _)) = rings.iter().find(|(_, edges)| edges.iter().any(|(a, b)| on(mid, *a, *b))) {
                    let w = &walls[k];
                    let (s1, s2) = (w.project(p).0, w.project(q).0);
                    for hole in holes.get(&placed[k].path).map(Vec::as_slice).unwrap_or(&[]) {
                        if kinds.get(&hole.path) != Some(&Kind::Door) {
                            continue;
                        }
                        let Some([from, to, _, _]) = hole.span else {
                            return Err(format!("where {} is in its wall is not known, so where it cuts this room's floor line is not", hole.path));
                        };
                        let (a, b) = (from.max(s1.min(s2)), to.min(s1.max(s2)));
                        if b - a > tol {
                            let (x, y) = ((a - s1).abs(), (b - s1).abs());
                            cuts.push((round + x.min(y), round + x.max(y)));
                            statuses.push(hole.status);
                            cited.extend(hole.inputs.iter().cloned());
                        }
                    }
                    // What stands on the floor against this side covers it, by the length
                    // of the side its plan lies along (call 162): a cut, as a door is.
                    if !against.is_empty() && length > tol {
                        let u = (q - p) * (1.0 / length);
                        let mut n = V(-u.1, u.0);
                        let into_room = |at: V| rooms::contains(outline, at) && !islands.iter().any(|i| rooms::contains(i, at));
                        if !into_room(mid + n * 0.05) && into_room(mid - n * 0.05) {
                            n = V(-n.0, -n.1);
                        }
                        for (i, thing) in against.iter().enumerate() {
                            let (mut s_lo, mut s_hi, mut d_lo, mut d_hi) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
                            for c in thing.corners {
                                let (s, d) = ((c - p).dot(u), (c - p).dot(n));
                                (s_lo, s_hi, d_lo, d_hi) = (s_lo.min(s), s_hi.max(s), d_lo.min(d), d_hi.max(d));
                            }
                            let (lo, hi) = (s_lo.max(0.0), s_hi.min(length));
                            // Its plan lies against the face (within the fit tolerance) and
                            // reaches into the room.
                            if d_lo <= tol && d_hi > tol && hi - lo > tol {
                                cuts.push((round + lo, round + hi));
                                statuses.push(thing.status);
                                cited.push(format!("{}#{K_PLACED}", thing.path));
                                if !touched.contains(&i) {
                                    touched.push(i);
                                }
                            }
                        }
                    }
                } else if room_lines.iter().any(|(a, b)| on(mid, *a, *b)) {
                    // A side along a room line is no wall space, and cuts the floor line as a doorway does (call 152).
                    cuts.push((round, round + length));
                }
            }
            round += length;
        }
        if round <= tol {
            continue;
        }
        if cuts.is_empty() {
            runs.push(round);
            continue;
        }
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Doorways that touch or overlap are one.
        let mut merged: Vec<(f64, f64)> = Vec::new();
        for c in cuts {
            match merged.last_mut() {
                Some(last) if c.0 <= last.1 + tol => last.1 = last.1.max(c.1),
                _ => merged.push(c),
            }
        }
        // The runs between them, the last running on past the first corner to the first.
        for i in 0..merged.len() {
            let start = merged[i].1;
            let end = if i + 1 < merged.len() { merged[i + 1].0 } else { merged[0].0 + round };
            runs.push(end - start);
        }
    }
    Ok((runs, statuses, cited, touched))
}

/// What a countertop leaves out where a box under it is deeper than the top is (call 162): a
/// box that turns a corner (36 in on each wall) is taken as the top's depth, and the leg that
/// turns the corner, (the box's depth less the top's) by the top's depth, is not in the
/// area. Said, not guessed: the model holds a top as a rectangle. One sentence for each such
/// box under the top. The boxes are those on the same storey standing on the floor no taller
/// than a base box, square to the top, in its length and with their backs on its wall.
fn counter_legs_said(me: &str, top: &Placed, size: [f64; 3], things: &[(String, String, Option<String>, Placed)]) -> Vec<String> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let (Some(at), Some(angle)) = (top.at_ft(), top.angle_deg()) else { return Vec::new() };
    let (long, deep) = (size[0], size[1]);
    let (sin, cos) = angle.to_radians().sin_cos();
    let storey = things.iter().find(|t| t.0 == me).and_then(|t| t.2.clone());
    let mut out = Vec::new();
    for (path, name, on, b) in things {
        let (Some(bat), Some([bw, bd, bh]), Some(bangle), Some(up)) = (b.at_ft(), b.size_ft(), b.angle_deg(), b.offset.feet()) else { continue };
        if path == me || *on != storey || up > tol || bh > 34.5 / 12.0 + tol || b.what.value.as_deref().is_some_and(|w| w.eq_ignore_ascii_case("Countertop")) {
            continue;
        }
        let (dx, dy) = (bat[0] - at[0], bat[1] - at[1]);
        let (lx, ly) = (dx * cos + dy * sin, -dx * sin + dy * cos);
        let turned = (bangle - angle).to_radians();
        let (w, d) = if turned.sin().abs() < 1.0e-6 {
            (bw, bd)
        } else if turned.cos().abs() < 1.0e-6 {
            (bd, bw)
        } else {
            continue;
        };
        let in_length = lx - w / 2.0 >= -long / 2.0 - tol && lx + w / 2.0 <= long / 2.0 + tol;
        let back_on_its_wall = ((ly + d / 2.0) - deep / 2.0).abs() <= tol;
        if in_length && back_on_its_wall && d > deep + tol {
            let inches = |feet: f64| units::show_length(&units::Len::One((feet * 12.0 * 64.0).round() / 64.0), units::IN);
            out.push(format!("the leg that turns the corner over {name}, {} by {} ({:.2} SF), is not in this area: add it where the top is priced by area.", inches(d - deep), inches(deep), (d - deep) * deep));
        }
    }
    out
}

/// A thing set in place that stands on the floor, by its plan: its four corners on the
/// plan, in feet, and how sure they are (call 162).
struct Standing {
    path: String,
    corners: [crate::skeleton::P2; 4],
    status: Status,
}

/// The fewest receptacle outlets a room's walls need (call 147, NEC 210.52(A)): each run of
/// its floor line (`room_runs`) of 2 ft or more takes one outlet for each 12 ft or part, a
/// shorter run none. With the statuses and inputs of the doors that cut it.
#[allow(clippy::too_many_arguments)]
fn room_receptacles(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, tol: f64) -> Result<(f64, Vec<Status>, Vec<String>), String> {
    // How many outlets a run of this length needs: a run under 2 ft is no wall space.
    let need = |run: f64| if run < 2.0 - tol { 0.0 } else { (run / 12.0 - 1.0e-9).ceil().max(1.0) };
    let (runs, statuses, cited, _) = room_runs(outline, islands, mine, room_lines, walls, placed, holes, kinds, &[], tol)?;
    Ok((runs.into_iter().map(need).sum(), statuses, cited))
}

/// A room's run of wall (call 157): the length of its floor line along walls, each side
/// that lies along a wall less the doorways in it, nothing for a side along a room line,
/// islands standing in the room with it. The sum of `room_runs`.
#[allow(clippy::too_many_arguments)]
fn room_wall_run(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, tol: f64) -> Result<(f64, Vec<Status>, Vec<String>), String> {
    let (runs, statuses, cited, _) = room_runs(outline, islands, mine, room_lines, walls, placed, holes, kinds, &[], tol)?;
    Ok((runs.into_iter().sum(), statuses, cited))
}

/// A room's run of base (call 162): its run of wall less the length of its walls' faces that
/// things set in place cover at the floor. A thing covers a face where its plan lies against
/// it and it stands on the floor; what it covers is the length of the face its plan lies
/// along, two things over the same length counting it once. The figure, the statuses and
/// inputs of what it rests on, and how many things covered something.
#[allow(clippy::too_many_arguments)]
fn room_base_run(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, against: &[&Standing], tol: f64) -> Result<(f64, Vec<Status>, Vec<String>, usize), String> {
    let (runs, statuses, cited, touched) = room_runs(outline, islands, mine, room_lines, walls, placed, holes, kinds, against, tol)?;
    Ok((runs.into_iter().sum(), statuses, cited, touched.len()))
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
    // A wall to be demolished bounds no room: rooms are the rooms the work leaves.
    let gone = crate::components::demolished(world);
    let mut placed: Vec<RoomWall> = Vec::new();
    let mut unplaced: Vec<(String, String)> = Vec::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(world).filter(|(_, k, _)| **k == Kind::Wall).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    for (n, w) in all {
        match w.map(|w| (w, plan_wall(tree, &n.path, w))) {
            None => unplaced.push((n.path.clone(), "it has no wall line entered".into())),
            Some((w, Ok((mut wall, shape, span, base)))) => {
                // Nor do the walls it met end on it: they meet each other where it stood.
                let stands = !gone.contains(&n.path);
                wall.join = wall.join && stands;
                placed.push(RoomWall { path: n.path.clone(), wall, shape, span, base, bounds: w.room_bounding && stands });
            }
            Some((_, Err(why))) => unplaced.push((n.path.clone(), why)),
        }
    }
    // Every room line (call 152): its storey, where it lies, or why it cannot be placed.
    // Like a wall's, a room line with no figure leaves its storey's rooms with none.
    struct Parting {
        path: String,
        line: Option<[crate::skeleton::P2; 2]>,
        status: Status,
        why: String,
    }
    let mut partings: BTreeMap<String, Vec<Parting>> = BTreeMap::new();
    {
        let mut q = world.query::<(&Node, &Kind, Option<&RoomLine>)>();
        let mut all: Vec<(&Node, Option<&RoomLine>)> = q.iter(world).filter(|(_, k, _)| **k == Kind::RoomLine).map(|(n, _, r)| (n, r)).collect();
        all.sort_by_key(|(n, _)| path_number(&n.path));
        for (n, r) in all {
            let (line, status, why) = match r {
                None => (None, Status::Unknown, "it has no line entered".to_string()),
                Some(r) => match (r.line.value.is_some(), r.line_ft()) {
                    (_, Some([a, b])) => (Some([crate::skeleton::P2(a[0], a[1]), crate::skeleton::P2(b[0], b[1])]), r.line.status, String::new()),
                    (true, None) => (None, r.line.status, "its line is in a unit this program does not convert".to_string()),
                    (false, None) => (None, r.line.status, "its line is not known".to_string()),
                },
            };
            partings.entry(n.parent.clone().unwrap_or_default()).or_default().push(Parting { path: n.path.clone(), line, status, why });
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
    const F_OUTLETS: &str = "round the room at its walls' faces, its floor line cut at each door into runs of wall; each run of 2 ft or more, one receptacle outlet for every 12 ft or part (no point more than 6 ft from one: NEC 2023 210.52(A)(1) and (A)(2), as Mike Holt's Understanding the 2023 NEC gives them); a room with no door is one run all the way round";
    const OUTLETS_NOTE: &str = "for the rooms NEC 210.52(A) covers (kitchen, family room, dining room, living room, sunroom, parlor, library, den, bedroom, recreation room and the like): fireplaces, stationary appliances and fixed cabinets also break wall space and are not drawn; a bathroom, a hallway, a foyer and a kitchen's counters have rules of their own (210.52(D), (H), (I), (C)), not worked here";
    const F_RUN: &str = "the room's floor line along its walls: each side of the room at its walls' faces that lies along a wall, less the doorways in it (a door to demolish is filled in and is no doorway), nothing for a side along a room line; what stands inside the room is taken with it";
    const F_BASE: &str = "the room's wall run, less the length of its walls' faces that things set in place cover at the floor (what is hung on a wall covers nothing, nor does what is to demolish; two things over the same length count it once)";
    const F_WALLS: &str = "each side of the room at its walls' faces × the height of the wall it is against, less the part of each door and window in that wall along the side";
    // Each wall's doors and windows, for the room's wall area (call 144).
    let holes_of: HashMap<String, Vec<Hole>> = {
        let mut q = world.query::<(&Node, &Kind, &Opening)>();
        let mut all: Vec<(String, Hole)> = q.iter(world).filter(|(n, k, _)| k.is_opening() && !gone.contains(&n.path)).filter_map(|(n, _, o)| n.parent.clone().map(|p| (p, hole_of(tree, &n.path, o).0))).collect();
        all.sort_by_key(|(_, h)| path_number(&h.path));
        let mut map: HashMap<String, Vec<Hole>> = HashMap::new();
        for (p, h) in all {
            map.entry(p).or_default().push(h);
        }
        map
    };
    // What is set in place and stands on the floor, by its plan: it covers the base of the
    // wall it is against (call 162). What is hung (up from the floor), what is to demolish
    // and what has no plan covers nothing.
    let standing: Vec<(Option<String>, Standing)> = {
        let mut q = world.query::<(&Node, &Kind, &Placed)>();
        let mut all: Vec<(u64, Option<String>, Standing)> = q
            .iter(world)
            .filter(|(n, k, _)| matches!(**k, Kind::Furnishing | Kind::Equipment | Kind::Fixture) && !gone.contains(&n.path))
            .filter_map(|(n, _, p)| {
                if p.offset.feet()? > tol {
                    return None;
                }
                let (at, size) = (p.at_ft()?, p.size_ft()?);
                let (sin, cos) = p.angle_deg()?.to_radians().sin_cos();
                let corner = |sx: f64, sy: f64| {
                    let (lx, ly) = (sx * size[0] / 2.0, sy * size[1] / 2.0);
                    crate::skeleton::P2(at[0] + lx * cos - ly * sin, at[1] + lx * sin + ly * cos)
                };
                let status = [p.at.status, p.size.status, p.offset.status, p.angle.as_ref().map(|a| a.status).unwrap_or(Status::Known)].into_iter().max().unwrap_or(Status::Known);
                Some((path_number(&n.path), tree.storey_of(&n.path).map(str::to_string), Standing { path: n.path.clone(), corners: [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)], status }))
            })
            .collect();
        all.sort_by_key(|(n, _, _)| *n);
        all.into_iter().map(|(_, storey, s)| (storey, s)).collect()
    };
    for (storey, mut list) in by_storey {
        list.sort_by_key(|(path, _)| path_number(path));
        let refuse_all = |out: &mut HashMap<String, BTreeMap<String, Derived>>, path: &str, inputs: &[String], why: &str| {
            let m = out.entry(path.to_string()).or_default();
            m.insert("floor_area".into(), Derived::refused("SF", F_FLOOR, inputs, why));
            m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, inputs, why));
            m.insert("perimeter".into(), Derived::refused("LF", F_ROUND, inputs, why));
            m.insert("wall_area".into(), Derived::refused("SF", F_WALLS, inputs, why));
            m.insert("wall_run".into(), Derived::refused("LF", F_RUN, inputs, why));
            m.insert("base_run".into(), Derived::refused("LF", F_BASE, inputs, why));
            m.insert("receptacles".into(), Derived::refused("EA", F_OUTLETS, inputs, why));
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
        let parting = partings.get(&storey).map(Vec::as_slice).unwrap_or(&[]);
        if let Some(p) = parting.iter().find(|p| p.line.is_none()) {
            for (path, _) in &list {
                refuse_all(out, path, &[format!("{path}#{K_ROOM}"), format!("{}#{K_ROOM_LINE}", p.path)], &format!("{} cannot be placed ({}), so what bounds the rooms of this level is not known", tree.called(&p.path), p.why));
            }
            continue;
        }
        let room_lines: Vec<(V, V)> = parting.iter().filter_map(|p| p.line).map(|[a, b]| (a, b)).collect();
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
        let as_entered = rooms_among(&plan_walls, &bounding, &room_lines, &points, tol);
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
            let gone = rooms_among(&without, &shifted, &room_lines, &points, tol);
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
            let standing = rooms_among(&tall, &with, &room_lines, &points, tol);
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
                Unbound::Shared(before) => format!("{} has no area yet: its point is in the space {} has. Part the two with a wall or a room line, and each gets its own.", tree.called(path), tree.called(&list[before].0)),
            };
            let figure = |value: f64, mine: &[usize], unit: &str, formula: &str| -> Derived {
                let mut inputs = vec![input.clone()];
                inputs.extend(mine.iter().map(|&k| format!("{}#{K_WALL}", placed[k].path)));
                let mut statuses: Vec<Status> = mine.iter().map(|&k| placed[k].shape.max(placed[k].span)).collect();
                statuses.extend([room.point.status, cut_status]);
                // A room measured to a room line is no surer than the line (call 152).
                for &k in &found.along {
                    statuses.push(parting[k].status);
                    inputs.push(format!("{}#{K_ROOM_LINE}", parting[k].path));
                }
                let mut notes: Vec<String> = Vec::new();
                // Walls whose heights are equally sure are said in one sentence, not one
                // each: a room of four traced walls read as the same clause four times.
                let mut by_how_sure: Vec<(String, Vec<String>)> = Vec::new();
                for &k in &rests_on[i] {
                    statuses.push(placed[k].span);
                    if !inputs.contains(&format!("{}#{K_WALL}", placed[k].path)) {
                        inputs.push(format!("{}#{K_WALL}", placed[k].path));
                    }
                    let (how, wall) = (format!("{:?}", placed[k].span).to_lowercase(), format!("{} ({})", tree.called(&placed[k].path), placed[k].path));
                    match by_how_sure.iter_mut().find(|g| g.0 == how) {
                        Some(group) => group.1.push(wall),
                        None => by_how_sure.push((how, vec![wall])),
                    }
                }
                for (how, walls) in by_how_sure {
                    notes.push(match walls.as_slice() {
                        [one] => format!("its outline counts on {one}, whose height is {how}"),
                        [most @ .., last] => format!("its outline counts on {} and {last}, whose heights are {how}", most.join(", ")),
                        [] => continue,
                    });
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
                    let walls_d = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol) {
                        Ok((value, statuses, cited)) => {
                            let mut d = figure(value, mine, "SF", F_WALLS);
                            if d.value.is_some() {
                                d.status = statuses.into_iter().fold(d.status, Status::max);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                            }
                            d
                        }
                        Err(why) => Derived::refused("SF", F_WALLS, std::slice::from_ref(&input), &why),
                    };
                    // The run of wall: wall_area's status and inputs, and the doorways it is cut at (call 157).
                    let run_d = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol).and_then(|(_, area_statuses, area_cited)| room_wall_run(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, &tree.kinds, tol).map(|(v, s, c)| (v, area_statuses.into_iter().chain(s).collect::<Vec<_>>(), area_cited.into_iter().chain(c).collect::<Vec<_>>()))) {
                        Ok((value, statuses, cited)) => {
                            let mut d = figure(value, mine, "LF", F_RUN);
                            if d.value.is_some() {
                                d.status = statuses.into_iter().fold(d.status, Status::max);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                            }
                            d
                        }
                        Err(why) => Derived::refused("LF", F_RUN, std::slice::from_ref(&input), &why),
                    };
                    // The base: the run of wall less what stands against it on the floor (call 162).
                    let on_this_storey: Vec<&Standing> = standing.iter().filter(|(s, _)| s.as_deref() == Some(storey.as_str())).map(|(_, s)| s).collect();
                    let base_d = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol).and_then(|(_, area_statuses, area_cited)| room_base_run(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, &tree.kinds, &on_this_storey, tol).map(|(v, s, c, n)| (v, area_statuses.into_iter().chain(s).collect::<Vec<_>>(), area_cited.into_iter().chain(c).collect::<Vec<_>>(), n))) {
                        Ok((value, statuses, cited, things)) => {
                            let mut d = figure(value, mine, "LF", F_BASE);
                            if d.value.is_some() {
                                d.status = statuses.into_iter().fold(d.status, Status::max);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                                // What it took off the wall run is said (less what is set against the walls).
                                let wall_run = run_d.value;
                                if let (Some(run), true) = (wall_run, things > 0) {
                                    let took = format!("less {} ft behind {} set against the walls", crate::view::num(run - value), if things == 1 { "1 thing".to_string() } else { format!("{things} things") });
                                    d.note = if d.note.is_empty() { took } else { format!("{}; {took}", d.note) };
                                }
                            }
                            d
                        }
                        Err(why) => Derived::refused("LF", F_BASE, std::slice::from_ref(&input), &why),
                    };
                    m.insert("wall_area".into(), walls_d);
                    m.insert("wall_run".into(), run_d);
                    m.insert("base_run".into(), base_d);
                    let outlets = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol).and_then(|_| room_receptacles(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, &tree.kinds, tol)) {
                        Ok((count, statuses, cited)) => {
                            let mut d = figure(count, mine, "EA", F_OUTLETS);
                            if d.value.is_some() {
                                // What it leaves out is a fact of the room, not of the walls.
                                d.status = statuses.into_iter().fold(d.status, Status::max).max(Status::Assumed);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                                d.note = if d.note.is_empty() { OUTLETS_NOTE.to_string() } else { format!("{}; {OUTLETS_NOTE}", d.note) };
                            }
                            d
                        }
                        Err(why) => Derived::refused("EA", F_OUTLETS, std::slice::from_ref(&input), &why),
                    };
                    m.insert("receptacles".into(), outlets);
                }
                (other, _) => {
                    let reason = why(other.as_ref().err().copied().unwrap_or(Unbound::NotEnclosed));
                    m.insert("floor_area".into(), Derived::refused("SF", F_FLOOR, std::slice::from_ref(&input), &reason));
                    m.insert("perimeter".into(), Derived::refused("LF", F_ROUND, std::slice::from_ref(&input), &reason));
                    m.insert("wall_area".into(), Derived::refused("SF", F_WALLS, std::slice::from_ref(&input), &reason));
                    m.insert("wall_run".into(), Derived::refused("LF", F_RUN, std::slice::from_ref(&input), &reason));
                    m.insert("base_run".into(), Derived::refused("LF", F_BASE, std::slice::from_ref(&input), &reason));
                    m.insert("receptacles".into(), Derived::refused("EA", F_OUTLETS, std::slice::from_ref(&input), &reason));
                }
            }
            // The faces can close a room that the wall lines do not: two walls drawn
            // to stop a couple of inches short of each other, inside each other's
            // thickness. The area inside the faces is then right, and the area to the
            // wall centers runs out through the gap and takes its neighbors (J22A-3: a
            // bedroom of 144 SF inside its walls and 865 SF to their centers, shown
            // as a figure). The centers cannot lie farther out than the widest wall
            // all the way round, so a figure past that is not this room's and is
            // refused.
            let widest = plan_walls.iter().map(|w| (w.ext - w.int).abs()).fold(0.0, f64::max);
            let runs_out = match (&found.centre, &found.finish) {
                (Ok((gross, _)), Ok((inside, round, _))) => *gross > inside + round * widest + 4.0 * widest * widest + 1.0,
                _ => false,
            };
            match (&found.centre, found.in_wall) {
                (Ok(_), false) if runs_out => {
                    let reason = "the walls round this room close at their faces and not on their lines: two of them stop short of each other by less than a wall's thickness, and the area to their centers runs out through the gap. Draw them to meet";
                    m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, std::slice::from_ref(&input), reason));
                }
                (Ok((area, mine)), false) => {
                    m.insert("gross_area".into(), figure(*area, mine, "SF", F_GROSS));
                }
                (other, _) => {
                    let reason = why(other.as_ref().err().copied().unwrap_or(Unbound::NotEnclosed));
                    m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, std::slice::from_ref(&input), &reason));
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
    if let Some(c) = w.get::<RoomLine>(e) {
        rows.push(view::entered(path, "line", &c.line, pair));
    }
    if let Some(c) = w.get::<WallType>(e) {
        // A footing's type gives a width; a wall's, a thickness. It is a footing's where
        // it was made as one (call 87), or, from a file written before types said so,
        // where footings are drawn on it.
        let footings = c.footing || w.iter_entities().any(|x| x.get::<Kind>() == Some(&Kind::Footing) && x.get::<Wall>().is_some_and(|c| c.type_name.value.as_deref() == Some(path)));
        rows.push(view::length(path, if footings { "footing width" } else { "wall thickness" }, &c.thickness));
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
        // Which trade's library it is kept in, and what one of it is drawn as.
        rows.push(plain("trade", c.division.map(|d| format!("Division {} {}", d.number(), d.title())).unwrap_or_else(|| "not said".to_string()), "which trade's item it is"));
        let drawn = match c.form.as_deref() {
            Some(said) => said.to_string(),
            None => forms::form_of(w.get::<Node>(e).map(|n| n.name.as_str()).unwrap_or("")).map(|f| format!("{}, by its name", f.name())).unwrap_or_else(|| format!("{}, its name saying nothing else", forms::BOX)),
        };
        rows.push(plain("drawn as", drawn, "what one of it is drawn as, inside the box of its sizes"));
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
    // A footing is held as a wall is, and is listed in a footing's words: its line, how
    // deep it is, and where its top is.
    if let Some(c) = w.get::<Wall>(e).filter(|_| w.get::<Kind>(e) == Some(&Kind::Footing)) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "footing line", &c.line, pair));
        if let Some(h) = &c.height {
            rows.push(view::length(path, "depth", h));
        }
        if let Some(o) = &c.base_offset {
            rows.push(view::length(path, "top, above its level's floor", o));
        }
    } else if let Some(c) = w.get::<Wall>(e) {
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
    if let Some(c) = w.get::<JoistsSpan>(e) {
        rows.push(view::entered(path, "joists span", &c.0, |s| s.clone()));
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
        if let Some(per) = &c.weight_per_foot {
            rows.push(view::entered(path, "weight per foot", per, |x| view::num(*x)));
        }
    }
    if let Some(c) = w.get::<Stood>(e) {
        let pad = w.get::<Kind>(e) == Some(&Kind::PadFooting);
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "position", &c.at, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
        rows.push(view::length(path, "base, above its level's floor", &c.base_offset));
        rows.push(view::length(path, if pad { "thickness" } else { "height" }, &c.height));
        // A pad's top is where it was said to be; it is held as its base and its thickness.
        if let (true, Some(b), Some(h)) = (pad, c.base_offset.feet(), c.height.feet()) {
            let mut top = plain("top", crate::structure::at_height(b + h), "its base and its thickness");
            top.status = c.base_offset.status.max(c.height.status);
            rows.push(top);
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
            rows.push(plain("height is to its", format!("{:?}", c.justify).to_lowercase(), "the bottom, the center or the top of its section: where its heights are taken to"));
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
            size.text = units::size_said([wide, deep, high]);
        }
        rows.push(size);
    }
    if let Some(c) = w.get::<Connectors>(e) {
        for (i, k) in c.0.iter().enumerate() {
            let what = [Some(format!("{:?}", k.kind).to_lowercase()), k.domain.map(|d| format!("{d:?}").to_lowercase()), (k.flow != Flow::Both).then(|| format!("{:?}", k.flow).to_lowercase())].into_iter().flatten().collect::<Vec<_>>().join(", ");
            let label = if k.name.is_empty() { format!("connection {} ({what})", i + 1) } else { format!("connection {}: {} ({what})", i + 1, k.name) };
            rows.push(view::entered(path, &label, &k.at, |p| format!("({}, {}, {})", view::num(p[0]), view::num(p[1]), view::num(p[2]))));
            if let Some(size) = &k.size {
                rows.push(view::length(path, &format!("connection {} size", i + 1), size));
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
    /// What a placed thing is drawn as, inside its box: what its item says, else what
    /// its name says. None: the box, and always for anything that is not placed.
    pub form: Option<forms::Form>,
    /// What it is called by its size, in the words the trade uses (call 177): "3 in waste",
    /// "3/4 in cold water", "16 x 8 supply" for a run on a system; its type's own name for a
    /// run that is on none, a beam, a post, a pad footing, a footing or a wall. None for
    /// what has no type to name. Worked here, once; a window only writes it.
    pub tag: Option<String>,
    /// A vertical drain of 3 in or more: a stack (call 177). Its tag says so.
    pub stack: bool,
}

/// A length in feet as inches the way a pipe is sized: to the sixteenth as a fraction
/// where it is one ("3/4", "1 1/2"), else as a decimal ("0.4").
fn inches_said(feet: f64) -> String {
    let inches = feet * 12.0;
    let sixteenths = inches * 16.0;
    if (sixteenths - sixteenths.round()).abs() > 0.02 {
        let d = format!("{:.2}", inches);
        return d.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    let parts = sixteenths.round() as u64;
    let (whole, mut top, mut bottom) = (parts / 16, parts % 16, 16);
    while top > 0 && top % 2 == 0 {
        top /= 2;
        bottom /= 2;
    }
    match (whole, top) {
        (w, 0) => format!("{w}"),
        (0, t) => format!("{t}/{bottom}"),
        (w, t) => format!("{w} {t}/{bottom}"),
    }
}

/// What a classification of system is called in the trade, in a size: a drain carries
/// "waste", a supply duct "supply".
pub fn trade_word(classification: &str) -> Option<&'static str> {
    Some(match classification {
        "Supply Air" => "supply",
        "Return Air" => "return",
        "Exhaust Air" => "exhaust",
        "Refrigerant" => "refrigerant",
        "Domestic Cold Water" => "cold water",
        "Domestic Hot Water" => "hot water",
        "Sanitary" => "waste",
        "Vent" => "vent",
        "Power" => "power",
        "Lighting" => "lighting",
        _ => return None,
    })
}

/// A size and what it carries, said as a tag (call 177): the section by itself, then the
/// system's word. A run on no system, or on one with no trade word, is its type's name. One
/// whose size is not known says so.
pub fn run_tag(type_name: Option<&str>, round: Option<f64>, rect: Option<(f64, f64)>, classification: Option<&str>) -> Option<String> {
    let size = match (round, rect) {
        (Some(d), _) => Some(format!("{} in", inches_said(d))),
        (None, Some((w, d))) => Some(format!("{} x {}", inches_said(w), inches_said(d))),
        _ => None,
    };
    match (size, classification.and_then(trade_word)) {
        (Some(size), Some(word)) => Some(format!("{size} {word}")),
        // Its size is not known: said, and not left off or guessed.
        (None, Some(word)) => Some(format!("{word}, size not known")),
        (Some(size), None) => Some(type_name.map(str::to_string).unwrap_or(size)),
        (None, None) => Some(type_name.map(|t| format!("{t}, size not known")).unwrap_or_else(|| "size not known".to_string())),
    }
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
        if !(matches!(kind, Kind::Wall | Kind::Floor | Kind::Footing) || kind.is_segment() || kind.is_placed() || kind.is_framing()) {
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
            // A footing is placed as a wall is: its line, its type's width, and from its
            // base up as far as it is deep.
            Kind::Wall | Kind::Footing => (|| {
                let c = w.get::<Wall>(e).ok_or(if kind == Kind::Footing { "it has no line" } else { "it has no wall line" })?;
                let [a, b] = c.line_ft().ok_or("its line is not known")?;
                let heights = wall_heights(&tree, &n.path, c);
                let height = heights.height.clone()?;
                // A footing's top is at its level's floor (or as far over it as it says),
                // and it lies below that by its depth; a wall stands up from its base.
                let base = heights.base.clone()? - if kind == Kind::Footing { height } else { 0.0 };
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
            k if k.is_segment() || k == Kind::Beam => (|| {
                let c = w.get::<Segment>(e).ok_or("it has no line")?;
                let [p, q] = c.line_ft().ok_or("its line is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (z0, z1) = c.offsets_ft().ok_or("its height above its level is not known")?;
                let t = typed(&tree.segment_types, &c.type_name, "a run").ok().map(|x| x.1);
                let (round, rect) = (t.and_then(|t| t.round_ft()), t.and_then(|t| t.rect_ft()));
                let section = round.map(|d| (d, d)).or(rect);
                if c.justify != Justify::Centre && section.is_none() {
                    return Err("its size is not known and its height is to its top or its bottom, so its center line cannot be placed".into());
                }
                let (width, depth) = section.unwrap_or((0.0, 0.0));
                let g = runs::Geom::of(level, p, q, z0, z1, c.justify, width, depth, round.is_some());
                let sized = if section.is_some() { t.map(|t| t.status().max(c.type_name.status)).unwrap_or(Status::Unknown) } else { Status::Unknown };
                let status = c.line.status.max(c.offset.status).max(c.end_offset.as_ref().map(|e| e.status).unwrap_or(Status::Known)).max(sized).max(floor_status);
                Ok((Shape::Run { a: g.a, b: g.b, round, rect }, status))
            })(),
            // A post stands as a vertical run, its section from its type; a pad footing is a
            // box of its type's width and length and its own thickness, its top where it
            // was said to be (call 164).
            Kind::Post | Kind::PadFooting => (|| {
                let c = w.get::<Stood>(e).ok_or("it has no position")?;
                let at = c.at_ft().ok_or("its position is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (b, h) = (c.base_offset.feet().ok_or("its base's height above its level is not known")?, c.height.feet().ok_or("its height is not known")?);
                let t = typed(&tree.segment_types, &c.type_name, if kind == Kind::Post { "a post" } else { "a pad footing" }).ok().map(|x| x.1);
                let (round, rect) = (t.and_then(|t| t.round_ft()), t.and_then(|t| t.rect_ft()));
                let sized = if round.is_some() || rect.is_some() { t.map(|t| t.status().max(c.type_name.status)).unwrap_or(Status::Unknown) } else { Status::Unknown };
                let status = c.at.status.max(c.base_offset.status).max(c.height.status).max(sized).max(floor_status);
                if kind == Kind::Post {
                    Ok((Shape::Run { a: [at[0], at[1], level + b], b: [at[0], at[1], level + b + h], round, rect }, status))
                } else {
                    let (wide, long) = rect.ok_or("its type's width and depth are not known")?;
                    Ok((Shape::Box { at: [at[0], at[1], level + b], size: [wide, long, h], angle: 0.0 }, status))
                }
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
            Ok((shape, status)) => {
                // An item says what it is drawn as, or leaves it to its name.
                let form = match &shape {
                    Shape::Box { .. } if kind.is_framing() => None,
                    Shape::Box { .. } => {
                        let said = w.get::<Placed>(e).and_then(|c| c.item.as_ref()).and_then(|item| index.get(item)).and_then(|t| w.get::<Item>(*t)).and_then(|it| it.form.clone());
                        match said {
                            Some(said) => forms::drawn_as(&said).ok().flatten(),
                            None => forms::form_of(&n.name),
                        }
                    }
                    _ => None,
                };
                // What it is called by its size (call 177): worked here, from its type.
                let type_path = match kind {
                    Kind::Wall | Kind::Footing => w.get::<Wall>(e).and_then(|c| c.type_name.value.clone()),
                    Kind::Post | Kind::PadFooting => w.get::<Stood>(e).and_then(|c| c.type_name.value.clone()),
                    k if k.is_segment() || k == Kind::Beam => w.get::<Segment>(e).and_then(|c| c.type_name.value.clone()),
                    _ => None,
                };
                let type_name = type_path.as_ref().and_then(|t| tree.names.get(t)).map(String::as_str);
                let class = system.as_ref().and_then(|p| index.get(p)).and_then(|s| w.get::<SystemInfo>(*s)).map(|s| s.classification.as_str());
                let (tag, stack) = match &shape {
                    Shape::Run { a, b, round, rect } if kind.is_segment() => {
                        let upright = (a[0] - b[0]).hypot(a[1] - b[1]) < 1.0e-6 && (a[2] - b[2]).abs() > 1.0e-6;
                        let stack = upright && class == Some("Sanitary") && round.is_some_and(|d| d >= 0.25 - 1.0e-6);
                        let tag = run_tag(type_name, *round, *rect, class);
                        (tag.map(|t| if stack { format!("{t} stack") } else { t }), stack)
                    }
                    _ if type_name.is_some() => (type_name.map(str::to_string), false),
                    _ => (None, false),
                };
                drawn.push(Placement { path: n.path.clone(), name: n.name.clone(), kind, storey, system, discipline, shape, status, form, tag, stack })
            }
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
    /// The length given three ways where a run on this line is not new work (call 158).
    pub length_by_work: Option<WorkSplit>,
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
    /// All of its runs given three ways, where one is not new work (call 158).
    pub runs_by_work: Option<WorkSplit>,
    pub lines: Vec<TakeoffLine>,
    /// Equipment and fixtures on the system: what each is, how many, and the worst status
    /// among what they are said to be and whether they are on this system.
    pub placed: Vec<(Kind, String, usize, Status)>,
    /// How many of each `placed` entry are to build, existing to remain and to demolish,
    /// in step with `placed` (call 158).
    pub placed_by_work: Vec<[usize; 3]>,
    /// The fittings on the system, counted by kind and by what each is said to be, with
    /// the worst status among those. A fitting with no kind entered is counted under None.
    pub fittings: Vec<(Option<FittingKind>, String, usize, Status)>,
    /// The same, in step with `fittings` (call 158).
    pub fittings_by_work: Vec<[usize; 3]>,
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
    let blank = |name: &str, flag: Status| SystemTakeoff { path: String::new(), name: name.into(), discipline: Discipline::Architecture, classification: String::new(), flag, runs: view::sum_of("LF", &[]), runs_by_work: None, lines: Vec::new(), placed: Vec::new(), placed_by_work: Vec::new(), fittings: Vec::new(), fittings_by_work: Vec::new(), joined: Continuity::default() };
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
    // a run with no size last. Beside each figure, where it is: to build, existing or to
    // demolish (call 158).
    let mut runs: BTreeMap<(usize, u8, u8, i64, i64), (Kind, String, Status, Figures, Vec<usize>)> = BTreeMap::new();
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
        let place = view::work_place(w, e);
        if kind.is_placed() {
            let what = w.get::<Placed>(e).map(|p| (p.what.value.clone().unwrap_or_else(|| "not said".into()), p.what.status)).unwrap_or(("not said".into(), Status::Unknown));
            if kind == Kind::Fitting {
                let fitting = w.get::<Fitting>(e).map(|f| f.kind);
                let status = what.1.max(sure).max(if fitting.is_some() { Status::Known } else { Status::Unknown });
                let list = &mut systems[at].fittings;
                match list.iter().position(|x| x.0 == fitting && x.1 == what.0) {
                    Some(i) => {
                        list[i].2 += 1;
                        list[i].3 = list[i].3.max(status);
                        systems[at].fittings_by_work[i][place] += 1;
                    }
                    None => {
                        list.push((fitting, what.0, 1, status));
                        let mut n = [0; 3];
                        n[place] = 1;
                        systems[at].fittings_by_work.push(n);
                    }
                }
                continue;
            }
            let list = &mut systems[at].placed;
            match list.iter().position(|x| x.0 == kind && x.1 == what.0) {
                Some(i) => {
                    list[i].2 += 1;
                    list[i].3 = list[i].3.max(what.1).max(sure);
                    systems[at].placed_by_work[i][place] += 1;
                }
                None => {
                    list.push((kind, what.0, 1, what.1.max(sure)));
                    let mut n = [0; 3];
                    n[place] = 1;
                    systems[at].placed_by_work.push(n);
                }
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
        let entry = runs.entry((at, rank, shape, a, b)).or_insert((kind, said, Status::Known, Vec::new(), Vec::new()));
        entry.2 = entry.2.max(if shape == 2 { Status::Unknown } else { sized.map(|(t, sure)| t.status().max(sure)).unwrap_or(Status::Unknown) });
        // A figure that is there is no surer than that the run is on this system.
        let (l, ls) = figure("true_length");
        entry.3.push((n.path.clone(), l, if l.is_some() { ls.max(sure) } else { ls }));
        entry.4.push(place);
    }
    let mut all: Vec<Figures> = vec![Vec::new(); systems.len()];
    let mut all_places: Vec<Vec<usize>> = vec![Vec::new(); systems.len()];
    for ((at, ..), (_, _, _, lengths, places)) in &runs {
        all[*at].extend(lengths.iter().cloned());
        all_places[*at].extend(places.iter().copied());
    }
    for (i, figures) in all.iter().enumerate() {
        systems[i].runs = view::sum_of("LF", figures);
        systems[i].runs_by_work = view::split_by_work("LF", figures, &all_places[i]);
    }
    for ((at, ..), (kind, size, size_status, lengths, places)) in runs {
        systems[at].lines.push(TakeoffLine { kind, size, size_status, length: view::sum_of("LF", &lengths), length_by_work: view::split_by_work("LF", &lengths, &places) });
    }
    for t in &mut systems {
        // Sorted with the figures of where each is (call 158), so that the two stay in step.
        let mut both: Vec<_> = std::mem::take(&mut t.fittings).into_iter().zip(std::mem::take(&mut t.fittings_by_work)).collect();
        both.sort_by(|(x, _), (y, _)| (x.0.is_none(), x.0, &x.1).cmp(&(y.0.is_none(), y.0, &y.1)));
        (t.fittings, t.fittings_by_work) = both.into_iter().unzip();
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
                        let label = e.and_then(|e| w.get::<Connectors>(*e)).and_then(|c| c.0.get(*index)).map(|k| k.name.clone()).filter(|s| !s.is_empty());
                        // By its name where it has one ("its Waste connection"); else
                        // by its number ("its connection 2").
                        match label {
                            Some(label) => format!("{name}: its {label} connection"),
                            None => format!("{name}: its connection {}", index + 1),
                        }
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
    /// Its strip footings: their length, and the concrete in them. Never in the walls.
    pub footings: Sum,
    pub footing_volume: Sum,
    /// What of the above is not new work, given three ways (call 158): to build,
    /// existing to remain, to demolish. Empty where nothing on the level is marked.
    pub by_work: StoreyByWork,
}

/// A level's counts given three ways where something on that line is not new work (call
/// 158). Each is None where everything on the line is new, and the line prints as it did.
/// Each is the sum of the same figures the line itself sums.
#[derive(Clone, Debug, Default)]
pub struct StoreyByWork {
    pub walls: Option<WorkSplit>,
    pub wall_face: Option<WorkSplit>,
    pub wall_net: Option<WorkSplit>,
    pub floors: Option<WorkSplit>,
    pub footings: Option<WorkSplit>,
    pub footing_volume: Option<WorkSplit>,
    /// Doors, and windows: how many to build, existing to remain, to demolish.
    pub doors: Option<[usize; 3]>,
    pub windows: Option<[usize; 3]>,
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
        let mut figures: [Vec<(String, Option<f64>, Status)>; 7] = Default::default();
        let (mut doors, mut windows) = (0, 0);
        let mut places: [Vec<usize>; 7] = Default::default();
        let (mut door_places, mut window_places) = ([0usize; 3], [0usize; 3]);
        // Each wall type met, in order: its path, and its walls' three figures.
        #[allow(clippy::type_complexity)]
        let mut of_type: Vec<(String, [Vec<(String, Option<f64>, Status)>; 3])> = Vec::new();
        for &e in &order {
            let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
            if !under(&n.path, &sn.path) {
                continue;
            }
            let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (n.path.clone(), d.value, d.status)).unwrap_or((n.path.clone(), None, Status::Unknown));
            let place = view::work_place(w, e);
            match kind {
                Kind::Wall => {
                    for p in &mut places[..3] {
                        p.push(place);
                    }
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
                Kind::Footing => {
                    places[5].push(place);
                    places[6].push(place);
                    figures[5].push(figure("plan_length"));
                    figures[6].push(figure("volume"));
                }
                Kind::Floor => {
                    places[3].push(place);
                    figures[3].push(figure("plan_area"))
                }
                Kind::Room => figures[4].push(figure("floor_area")),
                Kind::Door => {
                    doors += 1;
                    door_places[place] += 1;
                }
                Kind::Window => {
                    windows += 1;
                    window_places[place] += 1;
                }
                _ => {}
            }
        }
        out.push(StoreyTotals {
            path: sn.path.clone(),
            name: sn.name.clone(),
            elevation: w.get::<Level>(s).and_then(|l| l.elevation.feet()),
            walls: view::sum_of_what_is_there("LF", &figures[0]),
            wall_face: view::sum_of_what_is_there("SF", &figures[1]),
            wall_net: view::sum_of_what_is_there("SF", &figures[2]),
            floors: view::sum_of_what_is_there("SF", &figures[3]),
            rooms: view::sum_of_what_is_there("SF", &figures[4]),
            doors,
            windows,
            footings: view::sum_of_what_is_there("LF", &figures[5]),
            footing_volume: view::sum_of_what_is_there("CY", &figures[6]),
            by_work: StoreyByWork {
                walls: view::split_by_work("LF", &figures[0], &places[0]),
                wall_face: view::split_by_work("SF", &figures[1], &places[1]),
                wall_net: view::split_by_work("SF", &figures[2], &places[2]),
                floors: view::split_by_work("SF", &figures[3], &places[3]),
                footings: view::split_by_work("LF", &figures[5], &places[5]),
                footing_volume: view::split_by_work("CY", &figures[6], &places[6]),
                doors: (door_places[1] + door_places[2] > 0).then_some(door_places),
                windows: (window_places[1] + window_places[2] > 0).then_some(window_places),
            },
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
