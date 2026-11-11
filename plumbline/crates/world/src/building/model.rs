//! The shapes of the building's elements: a level, a wall and its type, a door or window
//! and its type, a floor and its type, a straight run of duct, pipe, conduit or cable and
//! its type, a thing set at a place, and a fitting.
//!
//! PORTED CODE. The structs, their fields and the rules checked on them are ported from
//! BIMCraft, `crates/core/src/model.rs` (and the kinds of a fitting and the connectors
//! from `crates/core/src/family.rs`), at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from.
//!
//! What was changed in the port, and nothing else:
//! * BIMCraft's lengths are bare numbers of millimetres. Here each is a `Val`: a value in
//!   the unit it was entered in (feet, inches, feet and inches), Known, Assumed or Unknown,
//!   with its source. An Unknown carries no number.
//! * BIMCraft points at a level, a host wall and a type by Guid or by name in a table of
//!   types. Here every thing is an element with a path: an element's `level` is the storey
//!   it is under in the tree, an opening's `host` is the wall it is under, and a `type` is
//!   the path of a Type element that carries the type's component. So `level` and `host`
//!   are not fields, and a type has a path, flags and sources like anything else.
//! * A level keeps `height`, the storey's own, which BIMCraft's level does not have. Its
//!   `computation_height` is absent when it is the usual one, 4 ft here for BIMCraft's
//!   1200 mm.
//! * A room's `level` is the storey it is under and its `name` is its element's. Not
//!   taken: its `colour`, `params`, `base_offset`, `top_level`, `limit_offset`, `frozen`
//!   and `kept`.
//! * A wall's `start` and `end` are one value, `line`: a line read off a drawing has one
//!   source and one status.
//! * A wall has `top_level` or `height`, never both, where BIMCraft keeps a height it does
//!   not use. `base_offset`, `top_offset` and a floor's `offset` are absent when there is
//!   none, where BIMCraft writes a zero.
//! * A door's and a window's types are one table in BIMCraft each; here an opening type
//!   says which it is for (`category`).
//! * Not taken: a level's view range, crop, extent and graphics; a wall's `arc`,
//!   `attach_top`, `attach_base` and `params`; a layer's id, Guid and `variable`; an
//!   opening's `family`, `mark`, `flip_facing`, `flip_hand` and `params`; an opening
//!   type's `sill` and `operation`; a floor's openings, slope, arrow, `structural`,
//!   pattern, sketch, shape and `params`.
//! * BIMCraft's `Run` is `Segment` here, and its `RunType` is `SegmentType`: `Run` and
//!   `pl::run` were already a roof line's plan run in this program, and a segment is what
//!   IFC calls the same thing (BIMCraft `runs.rs:25`: IFCPIPESEGMENT, IFCDUCTSEGMENT,
//!   IFCCABLECARRIERSEGMENT, IFCCABLESEGMENT). A segment's `start` and `end` are one
//!   value, `line`. Its `system` is not a name on it but a component of its own that
//!   points at a system element. A segment type says which kind of run it is for
//!   (`category`, the key of BIMCraft's table of run types); it has a `depth` only when it
//!   is not round. Not taken: a run's `join` and `params`, a run type's `material` (a
//!   material here is a condition).
//! * BIMCraft's `Component` is `Placed` here. Families are not ported, so in place of a
//!   component's `family` and `type` it says `what` it is and its `size`. Not taken:
//!   `mirrored`, `params`.
//! * A fitting is a kind of element of its own here, carrying its kind (BIMCraft: a
//!   component of a family of the pipe-fitting category with a `kind`). The five kinds
//!   are BIMCraft's for a pipe fitting and are used for every kind of run.
//! * A connector is on the placed thing itself here, where BIMCraft's is on a family and
//!   worked out for a type. Its `at` and `size` are values, not formulas. Two kinds are
//!   added, `duct` and `conduit`: BIMCraft's network joins pipes only so far (its
//!   `docs/areas/structure-and-mep.md`: "Conduits and the other electrical runs are not in
//!   it yet"). Not taken: the `switch`, `data` and `earthing` kinds, a connector's id and
//!   Guid, its `direction`, and what sizes a circuit (`system`, `poles`, `voltage`,
//!   `load_va`, `power_factor`, `classification`).
//! * An opening whose head is above its wall's top is not allowed here; BIMCraft allows
//!   it (DECISIONS.md call 29).
//! * The checks are the `Val` rules of this project around BIMCraft's rules; each rule
//!   taken from BIMCraft names its line.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{coords, positive, usable};
use super::P3;
use crate::components::{Kind, P2};
use crate::units::{self, Len};
use crate::value::Val;

fn yes() -> bool {
    true
}

fn is_yes(b: &bool) -> bool {
    *b
}

fn is_centre(l: &Location) -> bool {
    *l == Location::Centre
}

/// `pl::level`: on a Storey, how high its floor is above the datum and how far it is up to
/// the next floor.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    pub elevation: Val<Len>,
    pub height: Val<Len>,
    /// How far above it its rooms find the walls that bound them. None is
    /// `COMPUTATION_HEIGHT_FT`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computation_height: Option<Val<Len>>,
}

/// How far above a storey's floor its rooms find the walls that bound them, unless the
/// storey says otherwise: 4 ft. (BIMCraft's is 1200 mm, `CUT_HEIGHT`; DECISIONS.md call 29.)
pub const COMPUTATION_HEIGHT_FT: f64 = 4.0;

/// The line of a wall its ends lie on: its centre, its core's centre, or a face, of its
/// finish or of its core, on its exterior or its interior side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Location {
    #[default]
    Centre,
    CoreCentre,
    ExteriorFinish,
    InteriorFinish,
    ExteriorCore,
    InteriorCore,
}

impl Location {
    /// As a drawing's notes would say it.
    pub fn label(self) -> &'static str {
        match self {
            Location::Centre => "wall centre line",
            Location::CoreCentre => "core centre line",
            Location::ExteriorFinish => "finish face, exterior",
            Location::InteriorFinish => "finish face, interior",
            Location::ExteriorCore => "core face, exterior",
            Location::InteriorCore => "core face, interior",
        }
    }
}

/// What a layer of a wall or a floor does: carries it (structure, its core), backs another
/// (substrate), insulates (thermal), seals it (a membrane, which may be as thin as
/// nothing), or finishes a face.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Function {
    #[default]
    Structure,
    Substrate,
    Thermal,
    Membrane,
    Finish,
}

/// A layer of a type: of a wall, from its exterior in; of a floor, from its top down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    #[serde(default)]
    pub function: Function,
    pub material: Val<String>,
    pub thickness: Val<Len>,
}

/// `pl::wall_type`: on a Type. How thick its walls are and, when it is built up, its layers
/// from the exterior side in, which make up that thickness. Its core runs from its first
/// structure layer to its last (all of it, without one).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WallType {
    pub thickness: Val<Len>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<Layer>,
}

/// `pl::floor_type`: on a Type. How thick its floors are and, when built up, its layers
/// from the top down.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloorType {
    pub thickness: Val<Len>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<Layer>,
}

/// `pl::opening_type`: on a Type. A door's or a window's size. `category` says which.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningType {
    pub category: Kind,
    pub width: Val<Len>,
    pub height: Val<Len>,
}

/// `pl::wall`: a wall by its location line on the plan, from its first end to its second.
/// Which line of the wall that is, is its `location`; its exterior side is to the left of
/// first to second, so walls drawn clockwise face out. It stands on the storey it is under,
/// `base_offset` above that storey's floor. Its top is at `top_level` (another storey)
/// plus `top_offset`, or, with no `top_level`, `height` above its base.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wall {
    /// The path of its type: a Type element carrying `pl::wall_type`.
    #[serde(rename = "type")]
    pub type_name: Val<String>,
    pub line: Val<[P2; 2]>,
    /// The storey its top is held to, by path. None for a wall `height` tall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_level: Option<Val<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<Val<Len>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_offset: Option<Val<Len>>,
    /// With a `top_level`, how far above it the top is (below it, negative).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_offset: Option<Val<Len>>,
    #[serde(default, skip_serializing_if = "is_centre")]
    pub location: Location,
    /// Whether it bounds rooms: on unless turned off, as for a low wall a room runs past.
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub room_bounding: bool,
    /// Whether it joins its neighbours at its ends: on unless turned off, so its corners
    /// mitre; an unjoined wall's ends stay square.
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub join: bool,
}

impl Wall {
    /// A wall of a type along a line, `height` tall, standing on its storey's floor, its
    /// line its centre line.
    pub fn tall(type_path: Val<String>, line: Val<[P2; 2]>, height: Val<Len>) -> Wall {
        Wall { type_name: type_path, line, top_level: None, height: Some(height), base_offset: None, top_offset: None, location: Location::Centre, room_bounding: true, join: true }
    }
    /// The same, with its top held to another storey.
    pub fn up_to(type_path: Val<String>, line: Val<[P2; 2]>, top_level: Val<String>) -> Wall {
        Wall { type_name: type_path, line, top_level: Some(top_level), height: None, base_offset: None, top_offset: None, location: Location::Centre, room_bounding: true, join: true }
    }
    pub fn line_ft(&self) -> Option<[P2; 2]> {
        let d = units::coord_divisor(&self.line.unit).ok()?;
        self.line.value.map(|l| l.map(|p| [p[0] / d, p[1] / d]))
    }
}

/// `pl::opening`: on a Door or a Window under its host wall. `offset` is how far its
/// centre is along the wall's line from the line's first end; `sill` is how high its
/// bottom is above the wall's base (0 for a door). Its width and height are its type's.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opening {
    /// The path of its type: a Type element carrying `pl::opening_type`.
    #[serde(rename = "type")]
    pub type_name: Val<String>,
    pub offset: Val<Len>,
    pub sill: Val<Len>,
}

/// `pl::floor`: a slab under its top, along `boundary`. Its top is at its storey's floor
/// level, `offset` above it. It is its type's thickness.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    /// The path of its type: a Type element carrying `pl::floor_type`.
    #[serde(rename = "type")]
    pub type_name: Val<String>,
    pub boundary: Val<Vec<P2>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<Val<Len>>,
}

impl Floor {
    pub fn boundary_ft(&self) -> Option<Vec<P2>> {
        let d = units::coord_divisor(&self.boundary.unit).ok()?;
        self.boundary.value.as_ref().map(|pts| pts.iter().map(|p| [p[0] / d, p[1] / d]).collect())
    }
}

/// `pl::room`: on a Room under its storey. A point inside the room: its boundary is found
/// from the walls around the point, so it follows them when they move. `number` is what
/// the plan calls it.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub point: Val<P2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<Val<String>>,
}

impl Room {
    pub fn point_ft(&self) -> Option<P2> {
        let d = units::coord_divisor(&self.point.unit).ok()?;
        self.point.value.map(|p| [p[0] / d, p[1] / d])
    }
}

/// A cross-section beyond this (in feet) is a typing error.
pub const MAX_SECTION_FT: f64 = 20.0;

/// Which line of a run's section its heights are: the top of it, its middle, or its bottom.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Justify {
    Top,
    #[default]
    Centre,
    Bottom,
}

fn is_centred(j: &Justify) -> bool {
    *j == Justify::Centre
}

/// `pl::segment_type`: on a Type. A run's section: `width` across by `depth` deep, or
/// round, `width` across. `category` says which kind of run it is a type of.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SegmentType {
    pub category: Kind,
    pub width: Val<Len>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<Val<Len>>,
    #[serde(default)]
    pub round: bool,
}

impl SegmentType {
    pub fn round(category: Kind, diameter: Val<Len>) -> SegmentType {
        SegmentType { category, width: diameter, depth: None, round: true }
    }
    pub fn rectangular(category: Kind, width: Val<Len>, depth: Val<Len>) -> SegmentType {
        SegmentType { category, width, depth: Some(depth), round: false }
    }
    /// The worst status among the dimensions it has.
    pub fn status(&self) -> crate::value::Status {
        self.depth.as_ref().map(|d| d.status).unwrap_or(crate::value::Status::Known).max(self.width.status)
    }
    /// The diameter in feet, for a round section with one.
    pub fn round_ft(&self) -> Option<f64> {
        self.width.feet().filter(|_| self.round)
    }
    /// Width and depth in feet, for a rectangular section with both.
    pub fn rect_ft(&self) -> Option<(f64, f64)> {
        self.width.feet().zip(self.depth.as_ref().and_then(|d| d.feet())).filter(|_| !self.round)
    }
    /// How far it is round the section, in feet.
    pub fn perimeter_ft(&self) -> Option<f64> {
        self.round_ft().map(|d| std::f64::consts::PI * d).or_else(|| self.rect_ft().map(|(w, d)| 2.0 * (w + d)))
    }
    /// The size as it was entered: `3/4" round`, `16" × 8"`, or that it is not known.
    pub fn said(&self) -> String {
        let one = |v: &Val<Len>| v.value.as_ref().map(|len| units::show_length(len, &v.unit));
        match (self.round, one(&self.width), self.depth.as_ref().map(one)) {
            (true, Some(d), _) => format!("{d} round"),
            (false, Some(w), Some(Some(d))) => format!("{w} × {d}"),
            _ => "size unknown".into(),
        }
    }
}

/// `pl::segment`: one straight piece of duct, pipe, conduit or cable, from the first end
/// of `line` to the second in plan. `offset` is how high its `justify` line is above the
/// storey it is under at the first end, and `end_offset` at the second (level when there
/// is none): a sloped piece has both, and a riser has the same point twice in plan and
/// two heights. Its section, its type's, is square to its axis.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    /// The path of its type: a Type element carrying `pl::segment_type`.
    #[serde(rename = "type")]
    pub type_name: Val<String>,
    pub line: Val<[P2; 2]>,
    pub offset: Val<Len>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_offset: Option<Val<Len>>,
    #[serde(default, skip_serializing_if = "is_centred")]
    pub justify: Justify,
}

impl Segment {
    pub fn line_ft(&self) -> Option<[P2; 2]> {
        let d = units::coord_divisor(&self.line.unit).ok()?;
        self.line.value.map(|l| l.map(|p| [p[0] / d, p[1] / d]))
    }
    /// Its two heights above its storey, in feet: at its first end and at its second.
    pub fn offsets_ft(&self) -> Option<(f64, f64)> {
        let first = self.offset.feet()?;
        match &self.end_offset {
            None => Some((first, first)),
            Some(e) => e.feet().map(|second| (first, second)),
        }
    }
}

/// `pl::placed`: on Equipment, a Fixture or a Fitting. What it is; where the middle of
/// its base is on the plan and how far that is above the storey it is under; how far it is
/// turned, in degrees counter-clockwise from east; and how big it is: along its own x
/// (east, before it is turned), along its own y, and tall.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placed {
    pub what: Val<String>,
    pub at: Val<P2>,
    pub offset: Val<Len>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<Val<f64>>,
    pub size: Val<P3>,
}

impl Placed {
    pub fn at_ft(&self) -> Option<P2> {
        let d = units::coord_divisor(&self.at.unit).ok()?;
        self.at.value.map(|p| p.map(|x| x / d))
    }
    pub fn size_ft(&self) -> Option<P3> {
        let d = units::coord_divisor(&self.size.unit).ok()?;
        self.size.value.map(|p| p.map(|x| x / d))
    }
    /// How far it is turned, in degrees: none entered is none turned.
    pub fn angle_deg(&self) -> Option<f64> {
        match &self.angle {
            None => Some(0.0),
            Some(a) => a.value,
        }
    }
}

/// What a fitting is: an elbow, a tee, a cross, a change of size, or a coupling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FittingKind {
    Bend,
    Tee,
    Cross,
    Transition,
    Coupling,
}

impl FittingKind {
    /// As a takeoff says it.
    pub fn label(self) -> &'static str {
        match self {
            FittingKind::Bend => "Bend",
            FittingKind::Tee => "Tee",
            FittingKind::Cross => "Cross",
            FittingKind::Transition => "Transition",
            FittingKind::Coupling => "Coupling",
        }
    }
}

/// `pl::fitting`: on a Fitting, what joins runs. Where it is is its `pl::placed`.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fitting {
    pub kind: FittingKind,
}

/// What a connector joins: `pipe` a pipe of its `domain`, `duct` a duct, `conduit` a
/// conduit, `power` a cable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorKind {
    Power,
    Pipe,
    Duct,
    Conduit,
}

impl ConnectorKind {
    /// The kind of run that joins a connector of this kind.
    pub fn joins(self) -> Kind {
        match self {
            ConnectorKind::Power => Kind::Cable,
            ConnectorKind::Pipe => Kind::Pipe,
            ConnectorKind::Duct => Kind::Duct,
            ConnectorKind::Conduit => Kind::Conduit,
        }
    }
    /// The kind of connector a run of `kind` joins.
    pub fn of(kind: Kind) -> Option<ConnectorKind> {
        [ConnectorKind::Power, ConnectorKind::Pipe, ConnectorKind::Duct, ConnectorKind::Conduit].into_iter().find(|k| k.joins() == kind)
    }
}

/// What a pipe connector carries: cold or hot water, waste (sewage), vent, storm water or gas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    Cold,
    Hot,
    Waste,
    Vent,
    Storm,
    Gas,
}

/// Which way what it carries goes through it: into the thing (`in`, as water into a
/// basin), out of it (`out`, as waste from a water closet) or either way (`both`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Flow {
    In,
    Out,
    #[default]
    Both,
}

fn is_both(f: &Flow) -> bool {
    *f == Flow::Both
}

/// Where a placed thing joins a system: where a pipe, a duct, a conduit or a cable meets
/// it. `at` is in the thing's own axes: x along its angle, y to its left, z up from its
/// base.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connector {
    /// What it is called on the thing (`Waste`, `Cold water`), for people; none is fine.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    pub kind: ConnectorKind,
    /// What a `pipe` connector carries; none on a fitting, which takes its pipe's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,
    #[serde(default, skip_serializing_if = "is_both")]
    pub flow: Flow,
    pub at: Val<P3>,
    /// Its size: a pipe connector's nominal diameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Val<Len>>,
}

impl Connector {
    pub fn at_ft(&self) -> Option<P3> {
        let d = units::coord_divisor(&self.at.unit).ok()?;
        self.at.value.map(|p| p.map(|x| x / d))
    }
}

/// `pl::connectors`: on Equipment, a Fixture or a Fitting, where runs meet it.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connectors(pub Vec<Connector>);

/// The most connectors one thing may have.
pub const MAX_CONNECTORS: usize = 64;

// ---------------------------------------------------------------- the entry checks

/// A reference to a type: a path, flagged and sourced, not blank.
pub(crate) fn type_ref(what: &str, v: &Val<String>) -> Result<(), String> {
    v.check(what, "")?;
    if v.value.as_ref().is_some_and(|p| p.trim().is_empty()) {
        return Err(format!("{what}: is blank; mark it Unknown if nobody has it"));
    }
    Ok(())
}

/// A length that may be below zero (an offset), in a unit this program knows.
pub(crate) fn offset_ok(what: &str, v: &Val<Len>) -> Result<(), String> {
    v.check_length(what)?;
    if v.feet().is_some_and(|x| x.abs() > crate::check::MAX_COORDINATE_FT) {
        return Err(format!("{what}: is not a usable number (limit {:e} ft either way)", crate::check::MAX_COORDINATE_FT));
    }
    Ok(())
}

pub fn check_level(path: &str, kind: Kind, l: &Level) -> Result<(), String> {
    if kind != Kind::Storey {
        return Err(format!("{path}: a {kind:?} cannot carry a level; only a storey can"));
    }
    offset_ok(&format!("{path} elevation"), &l.elevation)?;
    if let Some(c) = &l.computation_height {
        positive(&format!("{path} computation height"), c, true)?;
    }
    positive(&format!("{path} storey height"), &l.height, false)
}

pub fn check_room(path: &str, kind: Kind, r: &Room) -> Result<(), String> {
    if kind != Kind::Room {
        return Err(format!("{path}: a {kind:?} cannot carry a room's point; only a room can"));
    }
    let d = coords(&format!("{path} room point"), &r.point)?;
    if let Some(p) = &r.point.value {
        usable(&format!("{path} room point"), p, d)?;
    }
    if let Some(n) = &r.number {
        n.check(&format!("{path} room number"), "")?;
    }
    Ok(())
}

/// The layers of a type each make sense, and add up to its thickness when every one of
/// them and the thickness are there to add (BIMCraft `model.rs:6245`).
fn check_layers(path: &str, thickness: &Val<Len>, layers: &[Layer]) -> Result<(), String> {
    // BIMCraft's limit on a type's layers (`MAX_LAYERS`, `model.rs:481`).
    const MAX_LAYERS: usize = 32;
    positive(&format!("{path} thickness"), thickness, false)?;
    if layers.len() > MAX_LAYERS {
        return Err(format!("{path}: has {} layers; a type takes up to {MAX_LAYERS}", layers.len()));
    }
    for (i, l) in layers.iter().enumerate() {
        let who = format!("{path} layer {}", i + 1);
        l.material.check(&format!("{who} material"), "")?;
        // A membrane may be as thin as nothing; no other layer may.
        positive(&format!("{who} thickness"), &l.thickness, l.function == Function::Membrane)?;
    }
    let parts: Option<Vec<f64>> = layers.iter().map(|l| l.thickness.feet()).collect();
    if let (false, Some(parts), Some(whole)) = (layers.is_empty(), parts, thickness.feet()) {
        let sum: f64 = parts.iter().sum();
        if (sum - whole).abs() > 1.0e-6 {
            return Err(format!("{path}: its layers add up to {} ft, not its thickness, {} ft", crate::view::num(sum), crate::view::num(whole)));
        }
    }
    Ok(())
}

pub fn check_wall_type(path: &str, kind: Kind, t: &WallType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: a {kind:?} cannot carry a wall type; only a type can"));
    }
    check_layers(&format!("{path} wall type"), &t.thickness, &t.layers)
}

pub fn check_floor_type(path: &str, kind: Kind, t: &FloorType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: a {kind:?} cannot carry a floor type; only a type can"));
    }
    check_layers(&format!("{path} floor type"), &t.thickness, &t.layers)
}

pub fn check_opening_type(path: &str, kind: Kind, t: &OpeningType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: a {kind:?} cannot carry an opening type; only a type can"));
    }
    if !t.category.is_opening() {
        return Err(format!("{path} opening type: is for a {:?}; an opening type is for a Door or a Window", t.category));
    }
    positive(&format!("{path} opening width"), &t.width, false)?;
    positive(&format!("{path} opening height"), &t.height, false)
}

pub fn check_wall(path: &str, kind: Kind, w: &Wall) -> Result<(), String> {
    if kind != Kind::Wall {
        return Err(format!("{path}: a {kind:?} cannot carry a wall's line; only a wall can"));
    }
    type_ref(&format!("{path} wall type"), &w.type_name)?;
    let d = coords(&format!("{path} wall line"), &w.line)?;
    if let Some([a, b]) = w.line.value {
        usable(&format!("{path} wall line"), &[a[0], a[1], b[0], b[1]], d)?;
        if a == b {
            return Err(format!("{path} wall line: the two ends are the same point"));
        }
    }
    match (&w.top_level, &w.height) {
        (Some(_), Some(_)) => return Err(format!("{path}: has a top level and a height; its top is held to a storey or it is a height tall, one or the other")),
        (None, None) => return Err(format!("{path}: has no top level and no height; enter one, or a height marked Unknown")),
        (Some(t), None) => type_ref(&format!("{path} top level"), t)?,
        (None, Some(h)) => positive(&format!("{path} wall height"), h, false)?,
    }
    if let Some(o) = &w.base_offset {
        offset_ok(&format!("{path} base offset"), o)?;
    }
    if let Some(o) = &w.top_offset {
        if w.top_level.is_none() {
            return Err(format!("{path}: has a top offset but no top level; a wall a height tall has none"));
        }
        offset_ok(&format!("{path} top offset"), o)?;
    }
    Ok(())
}

pub fn check_opening(path: &str, kind: Kind, o: &Opening) -> Result<(), String> {
    if !kind.is_opening() {
        return Err(format!("{path}: a {kind:?} cannot carry an opening; only a door or a window can"));
    }
    type_ref(&format!("{path} opening type"), &o.type_name)?;
    positive(&format!("{path} offset along the wall"), &o.offset, true)?;
    // Its sill on or over the wall's base (BIMCraft `model.rs:6690`).
    positive(&format!("{path} sill height"), &o.sill, true)
}

pub fn check_floor(path: &str, kind: Kind, f: &Floor) -> Result<(), String> {
    if kind != Kind::Floor {
        return Err(format!("{path}: a {kind:?} cannot carry a floor's boundary; only a floor can"));
    }
    type_ref(&format!("{path} floor type"), &f.type_name)?;
    let d = coords(&format!("{path} floor boundary"), &f.boundary)?;
    if let Some(pts) = &f.boundary.value {
        for p in pts {
            usable(&format!("{path} floor boundary"), p, d)?;
        }
    }
    if let Some(pts) = f.boundary_ft() {
        if let Some(fault) = crate::geom::outline_fault(&pts) {
            return Err(format!("{path} floor boundary: {fault}"));
        }
    }
    if let Some(o) = &f.offset {
        offset_ok(&format!("{path} floor offset"), o)?;
    }
    Ok(())
}

pub fn check_segment_type(path: &str, kind: Kind, t: &SegmentType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: a {kind:?} cannot carry a run's type; only a type can"));
    }
    if !t.category.is_segment() {
        return Err(format!("{path} run type: is for a {:?}; a run type is for a Duct, a Pipe, a Conduit or a Cable", t.category));
    }
    match (t.round, &t.depth) {
        (true, Some(_)) => return Err(format!("{path} run type: is round and has a depth; a round section has its diameter, as its width, and nothing else")),
        (false, None) => return Err(format!("{path} run type: is not round and has no depth; a section is round, by a diameter, or rectangular, by a width and a depth")),
        _ => {}
    }
    for (name, v) in [(if t.round { "diameter" } else { "width" }, Some(&t.width)), ("depth", t.depth.as_ref())] {
        let Some(v) = v else { continue };
        positive(&format!("{path} {name}"), v, false)?;
        if v.feet().is_some_and(|x| x > MAX_SECTION_FT) {
            return Err(format!("{path} {name}: is over {MAX_SECTION_FT} ft across; that is not a duct or a pipe"));
        }
    }
    Ok(())
}

pub fn check_segment(path: &str, kind: Kind, r: &Segment) -> Result<(), String> {
    if !kind.is_segment() {
        return Err(format!("{path}: a {kind:?} cannot carry a run's line; only a duct, a pipe, a conduit or a cable can"));
    }
    type_ref(&format!("{path} run type"), &r.type_name)?;
    let d = coords(&format!("{path} line"), &r.line)?;
    if let Some([a, b]) = r.line.value {
        usable(&format!("{path} line"), &[a[0], a[1], b[0], b[1]], d)?;
    }
    offset_ok(&format!("{path} offset"), &r.offset)?;
    if let Some(e) = &r.end_offset {
        offset_ok(&format!("{path} end offset"), e)?;
    }
    // Too short to be a run (BIMCraft `runs.rs:63`): its ends are the same point in space.
    if let (Some([a, b]), Some((z0, z1))) = (r.line_ft(), r.offsets_ft()) {
        let length = crate::geom::dist(a, b).hypot(z1 - z0);
        if length <= crate::check::FIT_TOLERANCE_FT {
            return Err(format!("{path} line: is {} ft long; a run goes from one point to another", crate::view::num(length)));
        }
    }
    Ok(())
}

pub fn check_placed(path: &str, kind: Kind, p: &Placed) -> Result<(), String> {
    if !kind.is_placed() {
        return Err(format!("{path}: a {kind:?} cannot carry a placement; only equipment, a fixture or a fitting can"));
    }
    p.what.check(&format!("{path} what it is"), "")?;
    if p.what.value.as_ref().is_some_and(|s| s.trim().is_empty()) {
        return Err(format!("{path} what it is: is blank; mark it Unknown if nobody has it"));
    }
    let d = coords(&format!("{path} position"), &p.at)?;
    if let Some(at) = &p.at.value {
        usable(&format!("{path} position"), at, d)?;
    }
    offset_ok(&format!("{path} offset"), &p.offset)?;
    if let Some(a) = &p.angle {
        a.check(&format!("{path} angle"), "deg")?;
        if a.value.is_some_and(|x| !x.is_finite() || x.abs() > 360.0) {
            return Err(format!("{path} angle: is not an angle between −360 and 360 degrees"));
        }
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

pub fn check_fitting(path: &str, kind: Kind, _f: &Fitting) -> Result<(), String> {
    if kind != Kind::Fitting {
        return Err(format!("{path}: a {kind:?} cannot carry a fitting's kind; only a fitting can"));
    }
    Ok(())
}

pub fn check_connectors(path: &str, kind: Kind, c: &Connectors) -> Result<(), String> {
    if !kind.is_placed() {
        return Err(format!("{path}: a {kind:?} cannot carry connectors; only equipment, a fixture or a fitting can"));
    }
    if c.0.len() > MAX_CONNECTORS {
        return Err(format!("{path}: has {} connectors; a thing takes up to {MAX_CONNECTORS}", c.0.len()));
    }
    for (i, k) in c.0.iter().enumerate() {
        let who = format!("{path} connector {}", i + 1);
        // Only a pipe connector carries a domain (BIMCraft `family.rs`, `Family::check`).
        if k.domain.is_some() && k.kind != ConnectorKind::Pipe {
            return Err(format!("{who}: is a {:?} connector and says what it carries; only a pipe connector does", k.kind).to_lowercase());
        }
        let d = coords(&format!("{who} position"), &k.at)?;
        if let Some(at) = &k.at.value {
            usable(&format!("{who} position"), at, d)?;
        }
        if let Some(size) = &k.size {
            positive(&format!("{who} size"), size, false)?;
            if size.feet().is_some_and(|x| x > MAX_SECTION_FT) {
                return Err(format!("{who} size: is over {MAX_SECTION_FT} ft across"));
            }
        }
    }
    Ok(())
}
