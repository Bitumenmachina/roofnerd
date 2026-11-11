//! Wall geometry. A wall's location line runs from its first end to its second; its faces
//! lie across it where its type and its location line put them, the exterior side to its
//! left.
//!
//! PORTED CODE. This file is ported from BIMCraft, `crates/core/src/walls.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from.
//!
//! What was changed in the port, and nothing else:
//! * Lengths are feet, passed in as plain numbers already converted; BIMCraft's are
//!   millimetres. Nothing here holds a tolerance.
//! * Straight walls only: BIMCraft's `Axis` also carries an arc.
//! * A wall is given as its thickness and its layers' thicknesses, since a type here is an
//!   element and not a row of a table in the model.

use super::model::{Function, Location};

/// How far from the exterior face of a wall of `thickness` with `layers` (each its
/// function and its thickness, from the exterior in) its location line `loc` lies. The
/// core runs from the first structure layer to the last; without one, it is all of it.
pub fn depth(thickness: f64, layers: &[(Function, f64)], loc: Location) -> f64 {
    let mut core: Option<(f64, f64)> = None;
    let mut at = 0.0;
    for (function, t) in layers {
        if *function == Function::Structure {
            core = Some((core.map_or(at, |c| c.0), at + t));
        }
        at += t;
    }
    let (c0, c1) = core.unwrap_or((0.0, thickness));
    match loc {
        Location::Centre => thickness / 2.0,
        Location::CoreCentre => (c0 + c1) / 2.0,
        Location::ExteriorFinish => 0.0,
        Location::InteriorFinish => thickness,
        Location::ExteriorCore => c0,
        Location::InteriorCore => c1,
    }
}

/// A wall's exterior and interior faces: how far to the left of its location line they lie.
pub fn faces(thickness: f64, layers: &[(Function, f64)], loc: Location) -> (f64, f64) {
    let ext = depth(thickness, layers, loc);
    (ext, ext - thickness)
}

/// How far to the left of its location line a wall's centre line lies.
pub fn centre(thickness: f64, layers: &[(Function, f64)], loc: Location) -> f64 {
    let (ext, int) = faces(thickness, layers, loc);
    (ext + int) / 2.0
}
