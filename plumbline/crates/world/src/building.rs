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

use std::collections::{BTreeMap, HashMap, HashSet};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::check::MAX_COORDINATE_FT;
use crate::components::*;
use crate::geom;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Derived, Status, Val};
use crate::view::{self, Row, Sum, WorkSplit};

pub mod cabinet;
pub mod clash;
pub mod connections;
mod footings;
pub mod forms;
pub mod shape;
pub mod framing;
pub mod frames;
pub mod members;
pub mod model;
pub mod network;
pub mod planset;
pub mod rooms;
pub mod runs;
pub mod walls;

pub use connections::{Facing, ItemConnection, ItemConnections};
pub use model::*;

mod faults;
mod figures;
mod room_figures;
mod takeoffs;
mod to_draw;
mod wall_faces;

pub use faults::*;
pub use figures::*;
use room_figures::*;
pub use takeoffs::*;
pub use to_draw::*;
pub use wall_faces::*;

/// A point in space: east, north, height above the datum.
pub type P3 = [f64; 3];

pub const K_LEVEL: &str = "pl::level";
pub const K_WALL: &str = "pl::wall";
pub const K_WALL_TYPE: &str = "pl::wall_type";
pub const K_OPENING: &str = "pl::opening";
pub const K_OPENING_TYPE: &str = "pl::opening_type";
pub const K_ITEM: &str = "pl::item";
pub const K_ITEM_CONNECTIONS: &str = "pl::item_connections";
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

