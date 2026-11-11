//! Snapping: where a point picked near the pointer lands on the plan. It snaps to what
//! the plan has there (the ends and middles of walls and runs, where two of their lines
//! cross, the foot of the perpendicular from the last point picked), to a guide through
//! the last point picked at an angle (from east, and from the line before it), to a
//! wall's line, or to the nearest inch.
//!
//! PORTED CODE. This file is ported from BIMCraft, `crates/core/src/snap.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from.
//!
//! What was changed in the port, and nothing else:
//! * Lengths are feet. A point with nothing to snap to goes to the nearest inch, where
//!   BIMCraft's goes to the nearest 10 mm (`STEP`): each is the round step of its own
//!   measure, and 10 mm is not restated as 0.0328 ft. Where BIMCraft tells two points
//!   apart by 1 mm (`TOL`), this uses the program's one tolerance, 0.01 ft. A point that
//!   is meant to be exact is kept to four places of a foot, where BIMCraft keeps a micron.
//! * What is snapped to is given as a `Scene`, made from the job by `scene`, so the
//!   solver can be tried on a scene written out by hand. BIMCraft builds its scene and
//!   solves it in one call.
//! * The scene holds the walls entered on the storey being drawn on. BIMCraft's holds
//!   the walls its plan cuts, whatever level they were drawn on.
//! * **Added:** the ends and middles of ducts, pipes, conduit and cable, and their lines.
//!   BIMCraft's scene has walls, grids, columns, components and detail lines; a run here
//!   has to be able to start where another ends.
//! * Not taken: curves, arcs, centres of arcs, quadrants and tangents; annotation
//!   anchors; a scan's points; drawings brought in; sketches of floors and regions;
//!   alignment guides; the ruler.

use crate::building::{Tree, wall_section};
use crate::components::*;
use crate::job::Job;
use crate::skeleton::P2 as V;
use crate::{Placed, Segment, Wall};

/// How far off a guide, in degrees, a direction may be and still snap to it (less where
/// guides lie closer together than that). BIMCraft `snap.rs:17`.
const TOLERANCE: f64 = 4.0;
/// Points with nothing to snap to round to this: an inch, in feet.
pub const STEP_FT: f64 = 1.0 / 12.0;
/// The most lines near the pointer that are tried against each other. BIMCraft `snap.rs:21`.
const MAX_NEAR: usize = 64;
/// Two points nearer than this are one.
const TOL: f64 = crate::check::FIT_TOLERANCE_FT;

/// What a point snapped to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapKind {
    Endpoint,
    Midpoint,
    Intersection,
    /// Where a fixture or a piece of equipment stands.
    Centre,
    Perpendicular,
    /// The nearest point of a wall's line.
    Nearest,
    /// A point on an angle guide.
    Angle,
    /// Nothing to snap to: the nearest inch.
    Free,
}

impl SnapKind {
    /// What it is called where a person reads it.
    pub fn name(self) -> &'static str {
        match self {
            // What a person at a drawing board would call the place the pointer is on.
            SnapKind::Endpoint => "On an end",
            SnapKind::Midpoint => "On a middle",
            SnapKind::Intersection => "Where two cross",
            SnapKind::Centre => "On a centre",
            SnapKind::Perpendicular => "Square to it",
            SnapKind::Nearest => "On a line",
            SnapKind::Angle => "On the angle",
            SnapKind::Free => "",
        }
    }
}

/// The snaps in force.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snaps {
    pub endpoint: bool,
    pub midpoint: bool,
    pub intersection: bool,
    pub centre: bool,
    pub perpendicular: bool,
    pub nearest: bool,
    /// Degrees between the angles a direction snaps to, from east and from the line before
    /// it, from 1 to 180; anything else for none.
    pub angle: f64,
}

impl Snaps {
    /// No snap at all, and no angle steps.
    pub fn none() -> Snaps {
        Snaps { endpoint: false, midpoint: false, intersection: false, centre: false, perpendicular: false, nearest: false, angle: 0.0 }
    }
}

impl Default for Snaps {
    fn default() -> Snaps {
        Snaps { endpoint: true, midpoint: true, intersection: true, centre: true, perpendicular: true, nearest: true, angle: 15.0 }
    }
}

/// How far from the pointer a snap reaches, in feet: to a point, and to a line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reach {
    pub point: f64,
    pub line: f64,
}

/// A line through a point, at an angle, that a pick snapped to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub from: P2,
    /// Degrees from east, counter-clockwise, in [0, 360).
    pub angle: f64,
    /// With a line before the last point, the angle between that line and this one, degrees.
    pub corner: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub at: P2,
    pub kind: SnapKind,
    pub guide: Option<Guide>,
    /// The element it came from, by path: none for the points picked so far, a guide's
    /// angle or nothing.
    pub source: Option<String>,
}

/// What the plan of a storey has to snap to. Each point and line holds the element it
/// comes from.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub points: Vec<(P2, SnapKind, Option<String>)>,
    /// Straight lines: walls' lines, and runs'.
    pub lines: Vec<(P2, P2, Option<String>)>,
    /// The walls: their line, their thickness and their path.
    pub walls: Vec<(P2, P2, f64, String)>,
}

impl Scene {
    /// A straight piece: its ends, its middle and its line (BIMCraft `snap.rs:226`).
    pub fn segment(&mut self, a: P2, b: P2, of: Option<String>) {
        let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        self.points.extend([(a, SnapKind::Endpoint, of.clone()), (b, SnapKind::Endpoint, of.clone()), (mid, SnapKind::Midpoint, of.clone())]);
        self.lines.push((a, b, of));
    }
}

/// What `storey`'s plan has to snap to: the walls and the runs entered on it, and where
/// its fixtures and equipment stand (BIMCraft `Scene::of`, `snap.rs:267`). `skip` leaves
/// one element out: the one being moved does not snap to where it was.
pub fn scene(job: &mut Job, storey: &str, skip: Option<&str>) -> Scene {
    let tree = Tree::of(&mut job.world);
    let mut s = Scene::default();
    let mut q = job.world.query::<(&Node, Option<&Wall>, Option<&Segment>, Option<&Placed>)>();
    let mut all: Vec<(&Node, Option<&Wall>, Option<&Segment>, Option<&Placed>)> = q.iter(&job.world).filter(|(n, ..)| n.parent.as_deref() == Some(storey) && skip != Some(n.path.as_str())).collect();
    all.sort_by(|a, b| a.0.path.cmp(&b.0.path));
    for (node, wall, segment, placed) in all {
        let of = Some(node.path.clone());
        if let Some([a, b]) = wall.and_then(|w| w.line_ft()) {
            s.segment(a, b, of.clone());
            // A wall with no thickness to be had is snapped to along its line and no wider.
            let thickness = wall.and_then(|w| wall_section(&tree, w).ok()).map(|t| t.0).unwrap_or(0.0);
            s.walls.push((a, b, thickness, node.path.clone()));
        }
        if let Some([a, b]) = segment.and_then(|r| r.line_ft()) {
            s.segment(a, b, of.clone());
        }
        if let Some(at) = placed.and_then(|p| p.at_ft()) {
            s.points.push((at, SnapKind::Centre, of));
        }
    }
    s
}

fn v(p: P2) -> V {
    V(p[0], p[1])
}

fn p2(q: V) -> P2 {
    [q.0, q.1]
}

/// `a` degrees turned into [0, 360).
fn norm(a: f64) -> f64 {
    let a = a.rem_euclid(360.0);
    if a >= 360.0 { 0.0 } else { a }
}

/// The direction of `d`, degrees from east, counter-clockwise, in [0, 360).
fn bearing(d: V) -> f64 {
    if d.0 == 0.0 && d.1 == 0.0 { 0.0 } else { norm(d.1.atan2(d.0).to_degrees()) }
}

/// To the nearest inch.
fn round(p: V) -> V {
    V((p.0 / STEP_FT).round() * STEP_FT, (p.1 / STEP_FT).round() * STEP_FT)
}

/// `p` to four places of a foot: what a point meant to be exact is, without float noise.
fn exact(p: V) -> V {
    V((p.0 * 1e4).round() / 1e4 + 0.0, (p.1 * 1e4).round() / 1e4 + 0.0)
}

/// How far `p` is from the segment `a` to `b` (BIMCraft `geom.rs:330`).
fn dist_to_segment(p: V, a: V, b: V) -> f64 {
    let d = b - a;
    let len2 = d.dot(d);
    let t = if len2 > 0.0 { ((p - a).dot(d) / len2).clamp(0.0, 1.0) } else { 0.0 };
    p.dist(a + d * t)
}

/// Where the line through `a` and `b` meets the line through `c` and `d`; `None` when they
/// are parallel (BIMCraft `geom.rs:336`).
fn line_intersection(a: V, b: V, c: V, d: V) -> Option<V> {
    let (r, s) = (b - a, d - c);
    let den = r.cross(s);
    if den.abs() <= 1e-12 * r.length() * s.length() {
        return None;
    }
    Some(a + r * ((c - a).cross(s) / den))
}

/// Where the segments `lines` cross each other, each with the element of the first of the
/// two (BIMCraft `snap.rs:588`).
fn crossings(lines: &[(V, V, Option<String>)]) -> Vec<(V, Option<String>)> {
    let on = |q: V, a: V, b: V| dist_to_segment(q, a, b) <= TOL;
    let mut out = Vec::new();
    for (i, (a, b, of)) in lines.iter().enumerate() {
        for (c, d, _) in lines.iter().skip(i + 1) {
            if let Some(q) = line_intersection(*a, *b, *c, *d) {
                if on(q, *a, *b) && on(q, *c, *d) {
                    out.push((q, of.clone()));
                }
            }
        }
    }
    out
}

/// Where the ray from `l` along the unit vector `u` meets the segment `a` to `b`
/// (BIMCraft `snap.rs:606`).
fn ray_hit(l: V, u: V, a: V, b: V) -> Option<V> {
    let e = b - a;
    let (len, den) = (e.length(), u.cross(e));
    if den.abs() <= 1e-12 * len {
        return None;
    }
    let (s, t) = ((a - l).cross(e) / den, (a - l).cross(u) / den);
    (s > 0.0 && t * len >= -TOL && t * len <= len + TOL).then(|| l + u * s)
}

/// The angles, in degrees, a direction from the last of `from` snaps to, every `inc`: from
/// east, and from the line before the last point (BIMCraft `snap.rs:618`).
fn rays(from: &[V], inc: f64) -> Vec<f64> {
    let n = (360.0 / inc - 1e-9).ceil().min(360.0) as usize;
    let mut out: Vec<f64> = (0..n).map(|k| k as f64 * inc).collect();
    if let [.., prev, last] = from {
        if prev.dist(*last) > TOL {
            let base = bearing(*last - *prev);
            out.extend((0..n).map(|k| norm(base + k as f64 * inc)));
        }
    }
    out
}

/// The angle at `last` between the line back to `prev` and the line on to `next`, in
/// degrees: a straight run on is 180, a right-angled turn 90; none while either line has
/// no length (BIMCraft `snap.rs:633`).
pub fn corner(prev: P2, last: P2, next: P2) -> Option<f64> {
    let (prev, last, next) = (v(prev), v(last), v(next));
    if prev.dist(last) <= TOL || last.dist(next) <= TOL {
        return None;
    }
    let turn = norm(bearing(next - last) - bearing(last - prev));
    Some(180.0 - if turn > 180.0 { 360.0 - turn } else { turn })
}

/// With a line before the last of `from`, the corner it makes with one going on in
/// direction `d`.
fn corner_on(from: &[V], d: V) -> Option<f64> {
    let [.., prev, last] = from else { return None };
    corner(p2(*prev), p2(*last), p2(*last + d.unit() * (2.0 * TOL)))
}

/// How points rank where two lie as near: an end before a crossing before a middle before
/// any other point (BIMCraft `snap.rs:650`).
fn rank(k: SnapKind) -> u8 {
    match k {
        SnapKind::Endpoint => 0,
        SnapKind::Intersection => 1,
        SnapKind::Midpoint => 2,
        _ => 4,
    }
}

/// Where a point picked at `p` lands on the plan: what the plan has there, the foot of the
/// perpendicular from the last of `from` (the points this pick follows), a guide through
/// that point at an angle, a wall's line, else the nearest inch. `square` keeps the step
/// from the last point along east or north (BIMCraft `solve`, `snap.rs:736`).
pub fn snap(scene: &Scene, p: P2, reach: Reach, from: &[P2], square: bool, o: &Snaps) -> Found {
    let from: Vec<V> = from.iter().map(|q| v(*q)).collect();
    let last = from.last().copied();
    let mut p = v(p);
    // The coordinate a square step pinned to the last point's: north if it runs east.
    let mut pinned = None;
    if let (true, Some(f)) = (square, last) {
        if (p.0 - f.0).abs() > (p.1 - f.1).abs() {
            p.1 = f.1;
            pinned = Some((true, f.1));
        } else {
            p.0 = f.0;
            pinned = Some((false, f.0));
        }
    }
    let by_distance = |a: &V, b: &V| a.dist(p).total_cmp(&b.dist(p));
    // The lines that pass near the pointer: what it can cross, or stand square to.
    let mut near: Vec<(V, V, Option<String>)> = scene.lines.iter().map(|(a, b, of)| (v(*a), v(*b), of.clone())).filter(|(a, b, _)| dist_to_segment(p, *a, *b) <= reach.point).collect();
    near.sort_by(|x, y| dist_to_segment(p, x.0, x.1).total_cmp(&dist_to_segment(p, y.0, y.1)));
    near.truncate(MAX_NEAR);

    // What the plan has here: its ends, middles and centres, the points picked so far, crossings.
    let wanted = |k: SnapKind| match k {
        SnapKind::Endpoint => o.endpoint,
        SnapKind::Midpoint => o.midpoint,
        SnapKind::Centre => o.centre,
        _ => true,
    };
    let mut points: Vec<(V, SnapKind, Option<String>)> = scene.points.iter().filter(|(_, k, _)| wanted(*k)).map(|(q, k, of)| (v(*q), *k, of.clone())).collect();
    if o.endpoint {
        points.extend(from.iter().map(|q| (*q, SnapKind::Endpoint, None)));
    }
    if o.intersection {
        points.extend(crossings(&near).into_iter().map(|(q, of)| (q, SnapKind::Intersection, of)));
    }
    let hit = points.into_iter().filter(|(q, _, _)| q.dist(p) <= reach.point).min_by(|a, b| by_distance(&a.0, &b.0).then(rank(a.1).cmp(&rank(b.1))));
    if let Some((at, kind, source)) = hit {
        return Found { at: p2(exact(at)), kind, guide: None, source };
    }

    // The foot of the perpendicular from the last point.
    if let (true, Some(l)) = (o.perpendicular, last) {
        let foot = near
            .iter()
            .filter_map(|(a, b, of)| {
                let (len, u) = ((*b - *a).length(), (*b - *a).unit());
                let t = (l - *a).dot(u);
                (len > 0.0 && (0.0..=len).contains(&t)).then(|| (*a + u * t, of.clone()))
            })
            .filter(|(f, _)| f.dist(p) <= reach.point && f.dist(l) > STEP_FT)
            .min_by(|x, y| by_distance(&x.0, &y.0));
        if let Some((f, source)) = foot {
            let guide = Guide { from: p2(l), angle: bearing(f - l), corner: corner_on(&from, f - l) };
            return Found { at: p2(exact(f)), kind: SnapKind::Perpendicular, guide: Some(guide), source };
        }
    }

    // A guide through the last point at the angle the pointer is nearest.
    if let (true, Some(l)) = ((1.0..=180.0).contains(&o.angle), last) {
        if p.dist(l) > 2.0 * STEP_FT {
            let d = p - l;
            let tolerance = TOLERANCE.min(0.4 * o.angle);
            let best = rays(&from, o.angle)
                .into_iter()
                .map(|a| {
                    let u = V(a.to_radians().cos(), a.to_radians().sin());
                    (a, u, u.dot(d), u.cross(d).abs())
                })
                .filter(|(_, _, along, perp)| *along > 0.0 && *perp <= reach.line && (perp / along).atan().to_degrees() <= tolerance)
                .min_by(|x, y| x.3.total_cmp(&y.3));
            if let Some((angle, u, along, _)) = best {
                let guide = Some(Guide { from: p2(l), angle, corner: corner_on(&from, u) });
                // Where it crosses a wall or a run near the pointer.
                let cross = near.iter().filter_map(|(a, b, of)| Some((ray_hit(l, u, *a, *b)?, of.clone()))).filter(|(q, _)| q.dist(p) <= reach.point).min_by(|x, y| by_distance(&x.0, &y.0));
                // Else the point on the inch that lies on it, if one does, else as long as
                // the nearest inch.
                let grid = round(p);
                let on_grid = u.cross(grid - l).abs() <= 1e-6 && u.dot(grid - l) > 0.0;
                return match cross {
                    Some((q, source)) if o.intersection => Found { at: p2(exact(q)), kind: SnapKind::Intersection, guide, source },
                    _ if on_grid => Found { at: p2(exact(grid)), kind: SnapKind::Angle, guide, source: None },
                    _ => Found { at: p2(exact(l + u * ((along / STEP_FT).round().max(1.0) * STEP_FT))), kind: SnapKind::Angle, guide, source: None },
                };
            }
        }
    }

    // A wall's line, to the nearest inch along it from its start.
    if o.nearest {
        let on = scene
            .walls
            .iter()
            .filter(|(a, b, _, _)| v(*a).dist(v(*b)) > 0.0)
            .map(|(a, b, t, path)| (v(*a), v(*b), dist_to_segment(p, v(*a), v(*b)) - t / 2.0, path))
            .filter(|(_, _, d, _)| *d <= reach.line)
            .min_by(|x, y| x.2.total_cmp(&y.2));
        if let Some((a, b, _, path)) = on {
            let (len, u) = ((b - a).length(), (b - a).unit());
            let along = (p - a).dot(u).clamp(0.0, len);
            let at = a + u * (((along / STEP_FT).round() * STEP_FT).min(len));
            return Found { at: p2(exact(at)), kind: SnapKind::Nearest, guide: None, source: Some(path.clone()) };
        }
    }
    // Rounded, but not off the line a square step keeps to.
    let mut at = round(p);
    match pinned {
        Some((true, y)) => at.1 = y,
        Some((false, x)) => at.0 = x,
        None => {}
    }
    Found { at: p2(exact(at)), kind: SnapKind::Free, guide: None, source: None }
}
