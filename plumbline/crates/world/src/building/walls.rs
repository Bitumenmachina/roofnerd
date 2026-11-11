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
//! * Where walls meet, the walls are given as a list (`PlanWall`: a wall's line, its two
//!   faces, its base and top, whether it joins), and a wall is named by its place in the
//!   list. BIMCraft finds a wall's neighbours through an index of its model; here every
//!   other wall in the list is tried.
//! * The corner of two built-up walls where one butts the other's structure
//!   (`walls/compound.rs`) is how layers are drawn and is not taken: two walls meet in a
//!   mitre.
//! * Where BIMCraft uses its tolerance of 1 mm, the caller's is used, in feet. This
//!   program's is 0.01 ft, about 3 mm, the one tolerance it tells two points apart by.

use super::model::{Function, Location};
use crate::skeleton::P2 as V;

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
pub fn wall_faces(thickness: f64, layers: &[(Function, f64)], loc: Location) -> (f64, f64) {
    let ext = depth(thickness, layers, loc);
    (ext, ext - thickness)
}

/// How far to the left of its location line a wall's centre line lies.
pub fn centre(thickness: f64, layers: &[(Function, f64)], loc: Location) -> f64 {
    let (ext, int) = wall_faces(thickness, layers, loc);
    (ext + int) / 2.0
}

/// A wall as its neighbours on the plan see it: its location line from `start` to `end`,
/// how far to the left of that line its exterior and interior faces lie, the heights of
/// its base and its top above the datum, and whether it joins its neighbours.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanWall {
    pub start: V,
    pub end: V,
    pub ext: f64,
    pub int: f64,
    pub base: f64,
    pub top: f64,
    pub join: bool,
}

impl PlanWall {
    fn length(&self) -> f64 {
        self.start.dist(self.end)
    }
    /// The way it runs.
    fn tangent(&self) -> V {
        (self.end - self.start).unit()
    }
    /// The point `s` along it, `off` to its left.
    fn at(&self, s: f64, off: f64) -> V {
        let u = self.tangent();
        self.start + u * s + u.perp() * off
    }
    /// How far along it the point nearest `p` is (from 0 to its length), and how far to
    /// its left `p` lies.
    fn project(&self, p: V) -> (f64, f64) {
        let u = self.tangent();
        ((p - self.start).dot(u).clamp(0.0, self.length()), (p - self.start).dot(u.perp()))
    }
    /// How far `p` is from the line `off` to its left.
    fn distance(&self, p: V, off: f64) -> f64 {
        p.dist(self.at(self.project(p).0, off))
    }
}

/// Whether two walls overlap in height, so they join.
pub fn stacked(a: &PlanWall, b: &PlanWall) -> bool {
    a.base < b.top && b.base < a.top
}

/// A wall where it leaves a node: the way it goes, and where its faces' lines and its
/// centre line are there (each running that way): the face on the counter-clockwise side
/// of that way, the one on the clockwise side, and which wall's it is.
struct Arm {
    wall: usize,
    way: V,
    ccw: V,
    cw: V,
    centre: V,
    thick: f64,
}

/// Wall `w` (at place `i`) leaving the node at its start, or with `at_end`, at its end.
fn arm(i: usize, w: &PlanWall, at_end: bool) -> Arm {
    let s = if at_end { w.length() } else { 0.0 };
    let t = w.tangent();
    // Leaving from its end, it goes back: its interior face is on the counter-clockwise side.
    let (way, ccw, cw) = if at_end { (t * -1.0, w.int, w.ext) } else { (t, w.ext, w.int) };
    Arm { wall: i, way, ccw: w.at(s, ccw), cw: w.at(s, cw), centre: w.at(s, (w.ext + w.int) / 2.0), thick: w.ext - w.int }
}

/// The walls at wall `i`'s start (or `at_end`, its end), round the node counter-clockwise,
/// its own among them: those ending there, and a wall it ends on, both ways from where it
/// does. Whether it ends on one, too.
fn arms(walls: &[PlanWall], i: usize, at_end: bool, tol: f64) -> (Vec<Arm>, bool) {
    let w = &walls[i];
    let node = if at_end { w.end } else { w.start };
    let mut arms = vec![arm(i, w, at_end)];
    let mut through = false;
    // A wall that refuses to join meets nothing at this end: its own square end is its whole cap.
    if !w.join {
        return (arms, false);
    }
    // A wall that refuses to join is no candidate for anyone either: neither side is mitred.
    let joins = |o: &PlanWall| o.join && stacked(w, o);
    for (other, o) in walls.iter().enumerate() {
        if other == i {
            continue;
        }
        if o.start.dist(node) <= tol {
            if joins(o) {
                arms.push(arm(other, o, false));
            }
        } else if o.end.dist(node) <= tol {
            if joins(o) {
                arms.push(arm(other, o, true));
            }
        } else {
            // A wall it ends on: the node within its thickness, between its ends.
            let (s, across) = o.project(node);
            if s > tol && s < o.length() - tol && (o.int - tol..=o.ext + tol).contains(&across) && o.distance(node, across) <= tol && joins(o) {
                let (t, (e, n), c) = (o.tangent(), (o.at(s, o.ext), o.at(s, o.int)), o.at(s, (o.ext + o.int) / 2.0));
                arms.push(Arm { wall: other, way: t, ccw: e, cw: n, centre: c, thick: o.ext - o.int });
                arms.push(Arm { wall: other, way: t * -1.0, ccw: n, cw: e, centre: c, thick: o.ext - o.int });
                through = true;
            }
        }
    }
    arms.sort_by(|a, b| a.way.1.atan2(a.way.0).total_cmp(&b.way.1.atan2(b.way.0)));
    (arms, through)
}

/// The other walls wall `i` meets at its start (or with `at_end`, its end), by their
/// place in the list: those that end where it ends and one it ends on, as `arms` finds
/// them. That end of it is cut to these.
pub(crate) fn meets(walls: &[PlanWall], i: usize, at_end: bool, tol: f64) -> Vec<usize> {
    let mut out: Vec<usize> = arms(walls, i, at_end, tol).0.into_iter().map(|a| a.wall).filter(|&w| w != i).collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Where the line through `a` and `b` meets the line through `c` and `d`; `None` when
/// they are parallel (BIMCraft `geom.rs:336`).
fn line_intersection(a: V, b: V, c: V, d: V) -> Option<V> {
    let (r, s) = (b - a, d - c);
    let den = r.cross(s);
    if den.abs() <= 1e-12 * r.length() * s.length() {
        return None;
    }
    Some(a + r * ((c - a).cross(s) / den))
}

/// The corner points that close wall `i`'s start, or with `at_end`, its end: from its face
/// on the clockwise side of the way it leaves the node round to the other. Walls ending
/// where it ends meet it at any angle, their faces running on until they cross (no
/// farther from the node than the thickest is thick, either way: past that, the end is
/// square); a wall it ends on stops it at its face. Where three or more walls meet, the
/// node itself fills between them.
pub fn cap(walls: &[PlanWall], i: usize, at_end: bool, tol: f64) -> Vec<V> {
    let w = &walls[i];
    let (arms, through) = arms(walls, i, at_end, tol);
    let n = arms.len();
    let Some(k) = arms.iter().position(|a| a.wall == i) else { return Vec::new() };
    let (prev, me, next) = (&arms[(k + n - 1) % n], &arms[k], &arms[(k + 1) % n]);
    if n < 2 {
        return vec![me.cw, me.ccw];
    }
    let node = if at_end { w.end } else { w.start };
    let limit = arms.iter().map(|a| a.thick).fold(0.0, f64::max);
    // Where a face of this wall crosses a neighbour's facing face, near enough to the node
    // either way; else its own end. A crossing far along the wall is no corner at all: two
    // walls meeting at a shallow angle cross metres in.
    let meet = |face: V, other: V, other_way: V| line_intersection(face, face + me.way, other, other + other_way).filter(|c| (-limit..=limit).contains(&(*c - node).dot(me.way))).unwrap_or(face);
    let (cw, ccw) = (meet(me.cw, prev.ccw, prev.way), meet(me.ccw, next.cw, next.way));
    if through {
        vec![cw, ccw]
    } else {
        vec![cw, node, ccw]
    }
}

/// Wall `i`'s centre line: each end it joins others at runs on (or back) to the nearest of
/// their centre lines, so rooms close at the centre lines whatever line walls stand on.
pub fn centre_line(walls: &[PlanWall], i: usize, tol: f64) -> [V; 2] {
    let w = &walls[i];
    let mid = (w.ext + w.int) / 2.0;
    let mut line = [w.at(0.0, mid), w.at(w.length(), mid)];
    for at_end in [false, true] {
        let (arms, _) = arms(walls, i, at_end, tol);
        let Some(me) = arms.iter().find(|a| a.wall == i) else { continue };
        let reach = 2.0 * arms.iter().map(|a| a.thick).fold(tol, f64::max);
        let crossing = arms.iter().filter(|a| a.wall != i).filter_map(|a| line_intersection(me.centre, me.centre + me.way, a.centre, a.centre + a.way)).min_by(|a, b| a.dist(me.centre).total_cmp(&b.dist(me.centre)));
        if let Some(c) = crossing.filter(|c| c.dist(me.centre) <= reach) {
            line[usize::from(at_end)] = c;
        }
    }
    line
}

/// Wall `i`'s outline in plan: its start cap, its exterior face, its end cap and its
/// interior face.
pub fn outline(walls: &[PlanWall], i: usize, tol: f64) -> Vec<V> {
    let mut ring = cap(walls, i, false, tol);
    ring.extend(cap(walls, i, true, tol));
    ring.dedup_by(|a, b| a.dist(*b) < 1e-9);
    ring
}
