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
//!   is not round. Not taken: a run's `join` and `params`, a run type's `material` (what
//!   an element is made of is said on the element, `pl::material`, by the name of a
//!   Material element that carries a price; BIMCraft's material is colour and pattern).
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
//! * The most connectors on one thing is 64 where BIMCraft's is 8 a family, and the
//!   largest section is 20 ft where BIMCraft's largest connector is 2000 mm: a panelboard
//!   has a connector a circuit, and a duct is wider than a pipe.
//! * A type's layers have to add up to its thickness within a billionth of a foot, where
//!   BIMCraft's is a millionth of a millimetre.
//! * BIMCraft refuses a model in which a wall's top is not above its base
//!   (`model.rs:6592`). Here such a wall loads, has no height, face or net area, and says
//!   why: whether its top is above its base depends on two storeys' elevations, and a job
//!   in which one of them is being changed has to stay open.
//! * An opening whose head is above its wall's top is not allowed here; BIMCraft allows
//!   it (DECISIONS.md call 29).
//! * A beam is a `Segment` too (BIMCraft's beam is a run), its heights to its top; a post and
//!   a pad footing are `Stood`, BIMCraft's `Column` (a pad footing a column laid flat), both
//!   of a `SegmentType` that gives their section (call 164). A type may carry a weight per
//!   foot, a beam's only: BIMCraft has none for any member but rebar.
//! * The checks are the `Val` rules of this project around BIMCraft's rules; each rule
//!   taken from BIMCraft names its line where it is applied.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{coords, positive, usable};
use super::P3;
use crate::components::{Kind, P2};
use crate::units::{self, Len};
use crate::value::{Status, Val};

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
            Location::Centre => "wall center line",
            Location::CoreCentre => "core center line",
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
    /// Made as a strip footing's type: its thickness is the footing's width, and only
    /// footings are drawn on it (call 87). Left out of the file for a wall's type.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub footing: bool,
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

/// `pl::joists_span` (call 165): on a Floor, which way its joists run: `the short way` (square
/// to the floor's longest edge) or `the long way`, with how sure that is and where it came
/// from. Left out, nothing was said, and the joists are taken to span the short way, Assumed.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JoistsSpan(pub Val<String>);

/// The two ways joists run, as a builder says them.
pub const SHORT_WAY: &str = "the short way";
pub const LONG_WAY: &str = "the long way";

impl JoistsSpan {
    /// The words typed for it: the two ways, in any capitals and with or without "the".
    pub fn read(word: &str) -> Result<&'static str, String> {
        match word.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase().as_str() {
            "the short way" | "short way" | "short" => Ok(SHORT_WAY),
            "the long way" | "long way" | "long" => Ok(LONG_WAY),
            _ => Err(format!("{:?} is not a way for joists to run: they span \"the short way\" or \"the long way\"", word.trim())),
        }
    }
}

pub fn check_joists_span(path: &str, kind: Kind, j: &JoistsSpan) -> Result<(), String> {
    if kind != Kind::Floor {
        return Err(format!("{path}: {kind:#} is not a floor; the way joists span is said of a floor"));
    }
    j.0.check(&format!("{path} joists span"), "")?;
    if j.0.value.as_deref().is_some_and(|v| v != SHORT_WAY && v != LONG_WAY) {
        return Err(format!("{path} joists span: joists span \"the short way\" or \"the long way\""));
    }
    Ok(())
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

/// `pl::room_line`: on a Room line under its level (call 152). Its two ends on the plan,
/// in feet or in inches, with how sure and where from. It bounds the rooms of its level as
/// a wall of no thickness would; it has no type, no height and no thickness, and is built
/// of nothing.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomLine {
    pub line: Val<[P2; 2]>,
}

impl RoomLine {
    pub fn line_ft(&self) -> Option<[P2; 2]> {
        let d = units::coord_divisor(&self.line.unit).ok()?;
        self.line.value.map(|l| l.map(|p| [p[0] / d, p[1] / d]))
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
    /// What a foot of it weighs, in pounds, where its type says (a steel shape's name says
    /// it: W8x18 is 18 lb/ft); a beam's only (call 164). The program holds no table of
    /// shapes: it is typed, with how sure it is and where it came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight_per_foot: Option<Val<f64>>,
}

impl SegmentType {
    pub fn round(category: Kind, diameter: Val<Len>) -> SegmentType {
        SegmentType { category, width: diameter, depth: None, round: true, weight_per_foot: None }
    }
    pub fn rectangular(category: Kind, width: Val<Len>, depth: Val<Len>) -> SegmentType {
        SegmentType { category, width, depth: Some(depth), round: false, weight_per_foot: None }
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
    /// The item it is one of, by path, when it was placed from one: its size is its
    /// item's, and follows it when the item is measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
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
    pub fn title(self) -> &'static str {
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
    pub fn for_run(kind: Kind) -> Option<ConnectorKind> {
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

impl Domain {
    /// The classification of the system that carries it, where the list of systems has
    /// one: storm water and gas have none there yet.
    pub fn system_classification(self) -> Option<&'static str> {
        match self {
            Domain::Cold => Some("Domestic Cold Water"),
            Domain::Hot => Some("Domestic Hot Water"),
            Domain::Waste => Some("Sanitary"),
            Domain::Vent => Some("Vent"),
            Domain::Storm | Domain::Gas => None,
        }
    }

    /// What a system of this classification carries, where a connector can say it.
    pub fn of_system(classification: &str) -> Option<Domain> {
        [Domain::Cold, Domain::Hot, Domain::Waste, Domain::Vent, Domain::Storm, Domain::Gas].into_iter().find(|d| d.system_classification() == Some(classification))
    }
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
    /// Which way it faces, in the thing's own axes (call 211): the way a pipe leaves it. None where nobody said, as it is on a
    /// connection a run made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facing: Option<super::connections::Facing>,
    /// The connection of the item it was made from, by the row's name, where the thing was set down from an item (call 211): what
    /// the thing follows when the item's rows or sizes change. None on a connection a run or a model brought in made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of_item: Option<String>,
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

/// The most connectors one thing may have. (BIMCraft's is 8 a family, `family.rs:698`.
/// Here it is 64: a connector is on the thing itself, and a panelboard is one thing with a
/// connector for each circuit.)
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
        return Err(format!("{path}: {kind:#} cannot carry a level; only a level can"));
    }
    offset_ok(&format!("{path} elevation"), &l.elevation)?;
    if let Some(c) = &l.computation_height {
        positive(&format!("{path} computation height"), c, true)?;
    }
    positive(&format!("{path} level height"), &l.height, false)
}

pub fn check_room(path: &str, kind: Kind, r: &Room) -> Result<(), String> {
    if kind != Kind::Room {
        return Err(format!("{path}: {kind:#} cannot carry a room's point; only a room can"));
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

/// A room line is on a level, has a line of two ends that are two points, farther apart
/// than a point is told from another (BIMCraft `room_separator`, `commands.rs:6938`: two
/// points make a line).
pub fn check_room_line(path: &str, kind: Kind, r: &RoomLine) -> Result<(), String> {
    if kind != Kind::RoomLine {
        return Err(format!("{path}: {kind:#} cannot carry a room line; only a room line can"));
    }
    let d = coords(&format!("{path} room line"), &r.line)?;
    if let Some([a, b]) = r.line.value {
        usable(&format!("{path} room line"), &[a[0], a[1], b[0], b[1]], d)?;
        if a == b {
            return Err(format!("{path} room line: the two ends are the same point"));
        }
        let length = crate::geom::dist([a[0] / d, a[1] / d], [b[0] / d, b[1] / d]);
        if length <= crate::check::FIT_TOLERANCE_FT {
            return Err(format!("{path} room line: is {} ft long; a room line is longer than the {} ft two points are told apart by", crate::view::num(length), crate::check::FIT_TOLERANCE_FT));
        }
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
        // BIMCraft's allowance is a millionth of a millimetre, on numbers typed in
        // millimetres. Here the layers may be typed in inches and the whole in feet, so
        // the allowance is what converting leaves behind and no more: a billionth of a foot.
        if (sum - whole).abs() > 1.0e-9 {
            return Err(format!("{path}: its layers add up to {} ft, not its thickness, {} ft", crate::view::num(sum), crate::view::num(whole)));
        }
    }
    Ok(())
}

pub fn check_wall_type(path: &str, kind: Kind, t: &WallType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: {kind:#} cannot carry a wall type; only a type can"));
    }
    check_layers(&format!("{path} wall type"), &t.thickness, &t.layers)
}

pub fn check_floor_type(path: &str, kind: Kind, t: &FloorType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: {kind:#} cannot carry a floor type; only a type can"));
    }
    check_layers(&format!("{path} floor type"), &t.thickness, &t.layers)
}

/// `pl::item`: on a Type. A thing that can be placed any number of times, as its maker's
/// sheet describes it: what kind of thing it is, who makes it, its model number, how
/// wide (along the wall), deep (out from it) and high it is, and what it weighs. Each
/// value says where it came from: read off a sheet it is Assumed and its source is the
/// sheet, the page and the line as written; not on the sheet, it is Unknown.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub category: Kind,
    pub maker: Val<String>,
    pub model: Val<String>,
    pub width: Val<Len>,
    pub depth: Val<Len>,
    pub height: Val<Len>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<Val<f64>>,
    /// The sheet it was made from, as it was named when it was brought in.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sheet: String,
    /// Which of the sixteen divisions' item it is: the trade whose library it is kept
    /// in. None: not said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<crate::components::Division>,
    /// What it is drawn as: a form by its name (`forms::Form::name`), or "box". None:
    /// what its name says, and a box if its name says nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<String>,
    /// "Priced as": the name of an assembly that goes on its kind (equipment, fixture, furnishing), which
    /// each of it set down takes (call 196). None: not priced, and what is set down from it is said to be
    /// set down and not priced. A price has one home, the price list; this names no price.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priced_as: Option<String>,
}

impl Item {
    /// Its size in feet (wide, deep, high), how sure that is and where it is from; or
    /// which of the three is not known.
    pub fn size_ft(&self) -> Result<(P3, Status, String), String> {
        let parts = [("width", &self.width), ("depth", &self.depth), ("height", &self.height)];
        let missing: Vec<&str> = parts.iter().filter(|(_, v)| v.feet().is_none()).map(|(n, _)| *n).collect();
        if !missing.is_empty() {
            return Err(format!("its {} {} not known", missing.join(" and "), if missing.len() == 1 { "is" } else { "are" }));
        }
        let feet = [self.width.feet().unwrap_or(0.0), self.depth.feet().unwrap_or(0.0), self.height.feet().unwrap_or(0.0)];
        let status = parts.iter().map(|(_, v)| v.status).max().unwrap_or(Status::Known);
        // One source when the three share it; else each with its own.
        let source = if parts.iter().all(|(_, v)| v.source == self.width.source) { self.width.source.clone() } else { parts.iter().map(|(n, v)| format!("{n}: {}", v.source)).collect::<Vec<_>>().join("; ") };
        Ok((feet, status, source))
    }
}

pub fn check_item(path: &str, kind: Kind, t: &Item) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: {kind:#} cannot carry an item; only a type can"));
    }
    if !matches!(t.category, Kind::Equipment | Kind::Fixture | Kind::Furnishing) {
        return Err(format!("{path} item: is {:#}; an item is a piece of equipment, a fixture or a furnishing", t.category));
    }
    let what = format!("{path} item");
    // What it is drawn as is a word of the built-in forms or a shape of the job's: whether
    // there is such a shape is for whoever holds the job to say (`shape::check_drawn_as`).
    if t.form.as_deref().is_some_and(|f| f.trim().is_empty() || f.trim() != f) {
        return Err(format!("{what}: drawn as: has to name a shape, or be left out"));
    }
    if t.priced_as.as_deref().is_some_and(|a| a.trim().is_empty() || a.trim() != a) {
        return Err(format!("{what}: priced as: has to name an assembly, or be left out"));
    }
    t.maker.check_basics(&format!("{what} maker"))?;
    t.model.check_basics(&format!("{what} model"))?;
    for (name, v) in [("width", &t.width), ("depth", &t.depth), ("height", &t.height)] {
        positive(&format!("{what} {name}"), v, false)?;
    }
    if let Some(w) = &t.weight {
        w.check(&format!("{what} weight"), "lb")?;
        if w.value.is_some_and(|x| !(x.is_finite() && x >= 0.0)) {
            return Err(format!("{what} weight: has to be nothing or more"));
        }
    }
    Ok(())
}

pub fn check_opening_type(path: &str, kind: Kind, t: &OpeningType) -> Result<(), String> {
    if kind != Kind::Type {
        return Err(format!("{path}: {kind:#} cannot carry an opening type; only a type can"));
    }
    if !t.category.is_opening() {
        return Err(format!("{path} opening type: is for a {:?}; an opening type is for a Door or a Window", t.category));
    }
    positive(&format!("{path} opening width"), &t.width, false)?;
    positive(&format!("{path} opening height"), &t.height, false)
}

pub fn check_wall(path: &str, kind: Kind, w: &Wall) -> Result<(), String> {
    // A footing is held as a wall is: a line, a type, and how deep it is.
    if !matches!(kind, Kind::Wall | Kind::Footing) {
        return Err(format!("{path}: {kind:#} cannot carry a wall's line; only a wall or a footing can"));
    }
    // Said in the words of what it is: a footing has a footing line and a depth.
    let (it, tall) = if kind == Kind::Footing { ("footing", "depth") } else { ("wall", "height") };
    type_ref(&format!("{path} {it} type"), &w.type_name)?;
    let d = coords(&format!("{path} {it} line"), &w.line)?;
    if let Some([a, b]) = w.line.value {
        usable(&format!("{path} {it} line"), &[a[0], a[1], b[0], b[1]], d)?;
        if a == b {
            return Err(format!("{path} {it} line: the two ends are the same point"));
        }
        // Too short to be a wall (BIMCraft `line`, `model.rs:7838`: no longer than its
        // tolerance).
        let length = crate::geom::dist([a[0] / d, a[1] / d], [b[0] / d, b[1] / d]);
        if length <= crate::check::FIT_TOLERANCE_FT {
            return Err(format!("{path} {it} line: is {} ft long; {kind:#} is longer than the {} ft two points are told apart by", crate::view::num(length), crate::check::FIT_TOLERANCE_FT));
        }
    }
    // Its top is held to a level or it is a height tall (BIMCraft `model.rs:6586`).
    match (&w.top_level, &w.height) {
        (Some(_), Some(_)) => return Err(format!("{path}: has a top level and a height; its top is held to a level or it is a height tall, one or the other")),
        (None, None) => return Err(format!("{path}: has no top level and no height; enter one, or a height marked Unknown")),
        (Some(t), None) => type_ref(&format!("{path} top level"), t)?,
        (None, Some(h)) => {
            positive(&format!("{path} {it} {tall}"), h, false)?;
            // A strip footing is not dug as deep as a house is tall: a depth past 20 ft
            // is a slip of the hand (1000 for 10.00, feet for inches), and it is said
            // (J21-6: one 1,000 ft deep was taken without a word, 740 CY of it).
            if kind == Kind::Footing && h.feet().is_some_and(|d| d > 20.0) {
                return Err(format!("{path} footing depth: is more than 20 ft, which is not a strip footing's depth; check the figure and its unit"));
            }
        }
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
        return Err(format!("{path}: {kind:#} cannot carry an opening; only a door or a window can"));
    }
    type_ref(&format!("{path} opening type"), &o.type_name)?;
    positive(&format!("{path} offset along the wall"), &o.offset, true)?;
    // Its sill on or over the wall's base (BIMCraft `model.rs:6690`).
    positive(&format!("{path} sill height"), &o.sill, true)
}

pub fn check_floor(path: &str, kind: Kind, f: &Floor) -> Result<(), String> {
    if kind != Kind::Floor {
        return Err(format!("{path}: {kind:#} cannot carry a floor's boundary; only a floor can"));
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
        return Err(format!("{path}: {kind:#} cannot carry a run's type; only a type can"));
    }
    if !(t.category.is_segment() || t.category.is_framing()) {
        return Err(format!("{path} run type: is for a {:?}; a run type is for a Duct, a Pipe, a Conduit, a Cable, a Beam, a Post or a PadFooting", t.category));
    }
    // A pad footing is square or rectangular (BIMCraft lays a column flat, and a round
    // one laid flat is a drum, not a pad); a weight per foot is a beam's (call 164).
    if t.category == Kind::PadFooting && t.round {
        return Err(format!("{path} pad footing type: is round; a pad footing is square or rectangular, by a width and a depth"));
    }
    if let Some(w) = &t.weight_per_foot {
        if t.category != Kind::Beam {
            return Err(format!("{path} run type: a weight per foot is a beam's; a {} has none", t.category.said()));
        }
        w.check(&format!("{path} weight per foot"), "lb/ft")?;
        if w.value.is_some_and(|x| !(x.is_finite() && x > 0.0)) {
            return Err(format!("{path} weight per foot: has to be more than nothing"));
        }
    }
    // A round section has one size (BIMCraft `model.rs:6265`: as deep as it is wide).
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
    if !(kind.is_segment() || kind == Kind::Beam) {
        return Err(format!("{path}: {kind:#} cannot carry a run's line; only a duct, a pipe, a conduit, a cable or a beam can"));
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
        // A beam given the very same point twice was meant stood on end: said as that, where
        // BIMCraft would say only that it is too short (its two ends are one point).
        if kind == Kind::Beam && crate::geom::dist(a, b) < 1.0e-9 && (z1 - z0).abs() < 1.0e-9 {
            return Err(format!("{path} line: its ends are one over the other; a beam runs along, and what stands is a post"));
        }
        let length = crate::geom::dist(a, b).hypot(z1 - z0);
        if length <= crate::check::FIT_TOLERANCE_FT {
            return Err(format!("{path} line: is {} ft long; a run goes from one point to another", crate::view::num(length)));
        }
        // A beam runs along (BIMCraft `runs.rs:62-76`): its ends are not one over the
        // other, and it is not sloped past 75 degrees; what stands is a post.
        if kind == Kind::Beam {
            let plan = crate::geom::dist(a, b);
            if plan <= crate::check::FIT_TOLERANCE_FT {
                return Err(format!("{path} line: its ends are one over the other; a beam runs along, and what stands is a post"));
            }
            let tilt = (z1 - z0).abs().atan2(plan).to_degrees();
            if tilt > MAX_BEAM_SLOPE_DEG {
                return Err(format!("{path} line: it slopes {tilt:.1} degrees, more than the {MAX_BEAM_SLOPE_DEG} degrees a beam may; what stands is a post"));
            }
        }
    }
    Ok(())
}

/// The steepest a beam may slope (BIMCraft `runs.rs` `MAX_SLOPE`): past it, what stands
/// is a post.
pub const MAX_BEAM_SLOPE_DEG: f64 = 75.0;

/// `pl::column`: on a Post or a Pad footing (call 164). BIMCraft's `Column` (`model.rs:1526`):
/// a point on plan, a type, how far its base is above the floor of the level it is under,
/// and how tall it is; a pad footing is one laid flat, its `height` its thickness and its
/// `base_offset` its top less that (`commands.rs:8674-8686`, so a top below the floor
/// gives a minus offset). Its type is a `SegmentType` of the category Post or PadFooting.
/// Not taken: `angle` (a post is not turned), `top_level` (a height is typed; one the
/// program fills in says so), `room_bounding`, `params`.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stood {
    /// The path of its type: a Type element carrying `pl::segment_type`.
    #[serde(rename = "type")]
    pub type_name: Val<String>,
    pub at: Val<P2>,
    /// How far its base is above the floor of its level (below it, negative).
    pub base_offset: Val<Len>,
    /// How tall it is (a pad footing: how thick).
    pub height: Val<Len>,
}

impl Stood {
    pub fn at_ft(&self) -> Option<P2> {
        let d = units::coord_divisor(&self.at.unit).ok()?;
        self.at.value.map(|p| p.map(|x| x / d))
    }
}

pub fn check_column(path: &str, kind: Kind, c: &Stood) -> Result<(), String> {
    if !matches!(kind, Kind::Post | Kind::PadFooting) {
        return Err(format!("{path}: {kind:#} cannot carry a column's place; only a post or a pad footing can"));
    }
    type_ref(&format!("{path} type"), &c.type_name)?;
    let d = coords(&format!("{path} position"), &c.at)?;
    if let Some(at) = &c.at.value {
        usable(&format!("{path} position"), at, d)?;
    }
    offset_ok(&format!("{path} base"), &c.base_offset)?;
    let what = if kind == Kind::PadFooting { "thickness" } else { "height" };
    // Its top must be above its base (BIMCraft: a column's top above its base; a footing's
    // thickness above zero, `commands.rs:8674`).
    positive(&format!("{path} {what}"), &c.height, false).map_err(|why| if kind == Kind::PadFooting { format!("{why}; a pad footing has a thickness, and nobody gave one") } else { format!("{why}; a post's top is above its base") })?;
    Ok(())
}

pub fn check_placed(path: &str, kind: Kind, p: &Placed) -> Result<(), String> {
    if !kind.is_placed() {
        return Err(format!("{path}: {kind:#} cannot carry a placement; only equipment, a fixture or a fitting can"));
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
        return Err(format!("{path}: {kind:#} cannot carry a fitting's kind; only a fitting can"));
    }
    Ok(())
}

pub fn check_connectors(path: &str, kind: Kind, c: &Connectors) -> Result<(), String> {
    if !kind.is_placed() {
        return Err(format!("{path}: {kind:#} cannot carry connections; only equipment, a fixture or a fitting can"));
    }
    if c.0.len() > MAX_CONNECTORS {
        return Err(format!("{path}: has {} connections; a thing takes up to {MAX_CONNECTORS}", c.0.len()));
    }
    for (i, k) in c.0.iter().enumerate() {
        let who = format!("{path} connection {}", i + 1);
        // Only a pipe connector carries a domain (BIMCraft `family.rs`, `Family::check`).
        if k.domain.is_some() && k.kind != ConnectorKind::Pipe {
            return Err(format!("{who}: is a {:?} connection and says what it carries; only a pipe connection does", k.kind).to_lowercase());
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
