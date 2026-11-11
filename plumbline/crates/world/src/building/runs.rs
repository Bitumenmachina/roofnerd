//! The geometry of one straight piece of duct, pipe, conduit or cable: the ends of its
//! centre line in space, and its section.
//!
//! PORTED CODE. This file is ported from BIMCraft, `crates/core/src/runs.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from.
//!
//! What was changed in the port, and nothing else:
//! * Lengths are feet, passed in as plain numbers already converted; BIMCraft's are
//!   millimetres. Its 1 mm tolerance for "one over the other in plan" is this program's
//!   fit tolerance, 0.01 ft.
//! * `Geom::of` is given the storey's elevation, the run's ends and its section, where
//!   BIMCraft's takes its model, the run and its type.
//! * Only what a length, a height and a picture need is here. BIMCraft's footprint,
//!   prism, tube, and the mitred outline of runs that join are drawing and are not taken.

use super::model::Justify;
use super::P3;
use crate::check::FIT_TOLERANCE_FT;
use crate::components::P2;

/// A run worked out: its centre line from `a` to `b` (feet: east, north, up from the
/// datum), and its section, `width` across by `depth` deep, square to that line; round,
/// `width` is the diameter and `depth` is as much.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geom {
    pub a: P3,
    pub b: P3,
    pub width: f64,
    pub depth: f64,
    pub round: bool,
}

impl Geom {
    /// A run from `start` to `end` on plan, standing on a storey whose floor is `base`
    /// above the datum, its `justify` line `offset` above that at the start and
    /// `end_offset` at the end. Its heights are those of its `justify` line, so its
    /// centre line is half the section's depth (square to the line) under it for a top,
    /// over it for a bottom.
    #[allow(clippy::too_many_arguments)]
    pub fn of(base: f64, start: P2, end: P2, offset: f64, end_offset: f64, justify: Justify, width: f64, depth: f64, round: bool) -> Geom {
        let (z0, z1) = (base + offset, base + end_offset);
        let depth = if round { width } else { depth };
        let g = Geom { a: [start[0], start[1], z0], b: [end[0], end[1], z1], width, depth, round };
        // Upright, it has no top or bottom to be the line.
        let shift = match justify {
            Justify::Top => -g.half(),
            Justify::Centre => 0.0,
            Justify::Bottom => g.half(),
        };
        Geom { a: [g.a[0], g.a[1], z0 + shift], b: [g.b[0], g.b[1], z1 + shift], ..g }
    }

    /// Along the centre line: its true length.
    pub fn length(&self) -> f64 {
        self.plan_length().hypot(self.b[2] - self.a[2])
    }

    /// Its length in plan.
    pub fn plan_length(&self) -> f64 {
        (self.b[0] - self.a[0]).hypot(self.b[1] - self.a[1])
    }

    /// Whether it stands upright: its ends one over the other in plan.
    pub fn vertical(&self) -> bool {
        self.plan_length() <= FIT_TOLERANCE_FT
    }

    /// Half the section's depth, taken straight up: more than half of it, square to the
    /// line, where the line slopes.
    pub fn half(&self) -> f64 {
        let plan = self.plan_length();
        if plan > FIT_TOLERANCE_FT {
            self.depth / 2.0 * self.length() / plan
        } else {
            0.0
        }
    }

    /// The lowest and the highest it reaches.
    pub fn z_span(&self) -> (f64, f64) {
        let h = self.half();
        (self.a[2].min(self.b[2]) - h, self.a[2].max(self.b[2]) + h)
    }
}
