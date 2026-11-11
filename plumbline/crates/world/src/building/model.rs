//! The shapes of the building's elements: a level, a wall and its type, a door or window
//! and its type, a floor and its type.
//!
//! PORTED CODE. The structs, their fields and the rules checked on them are ported from
//! BIMCraft, `crates/core/src/model.rs`, at commit
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
//! * An opening whose head is above its wall's top is not allowed here; BIMCraft allows
//!   it (DECISIONS.md call 29).
//! * The checks are the `Val` rules of this project around BIMCraft's rules; each rule
//!   taken from BIMCraft names its line.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{coords, positive, usable};
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
}

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
    positive(&format!("{path} storey height"), &l.height, false)
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
