//! Rooms' boundaries. A room's boundary is the smallest loop of its storey's
//! room-bounding lines around its point, less the islands inside it.
//!
//! PORTED CODE. This file is ported from BIMCraft, `crates/core/src/geom.rs` (the lines'
//! crossings, their split, the faces of the planar graph they make, `Region`,
//! `region_in`), `crates/core/src/rooms.rs` (a room's boundary, and the room placed
//! before it that has the space) and `crates/core/src/model.rs` (`RoomArea`, `Unbound`,
//! `region_holds`), at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10),
//! https://github.com/IPedrax/bimcraft, Copyright (c) 2026 IPedrax and the BIMCraft
//! contributors, used under the MIT license (BIMCraft is MIT OR Apache-2.0). The notice is
//! in NOTICE at the repository root. `PORTMAP.md` gives the line each item came from.
//!
//! What was changed in the port, and nothing else:
//! * Lengths are feet; BIMCraft's are millimetres. Its 1 mm tolerance is this program's
//!   fit tolerance, 0.01 ft, passed in by the caller.
//! * Each segment is cut by every other. BIMCraft finds the others through a grid and
//!   keeps cutting by every other as the tested twin of that (`faces_everywhere`); the
//!   twin is what is here.
//! * `bound` is given the storey's bounding walls and its rooms, where BIMCraft's `Rooms`
//!   holds its model and finds each level's once. A room that another has the space of is
//!   named by its place in the list given, not by an id.
//! * Not taken: room separation lines, room-bounding columns, a frozen room and the
//!   boundary it keeps.

use super::walls::{self, PlanWall};
use crate::skeleton::P2 as V;

/// Signed area of a ring (counter-clockwise positive).
pub fn area(ring: &[V]) -> f64 {
    ring_edges(ring).map(|(a, b)| a.cross(b)).sum::<f64>() / 2.0
}

pub fn perimeter(ring: &[V]) -> f64 {
    ring_edges(ring).map(|(a, b)| a.dist(b)).sum()
}

fn ring_edges(ring: &[V]) -> impl Iterator<Item = (V, V)> + '_ {
    ring.iter().copied().zip(ring.iter().copied().cycle().skip(1))
}

/// Even-odd point in ring.
pub fn contains(ring: &[V], p: V) -> bool {
    // `(a.1 > p.1) != (b.1 > p.1)` rules out a horizontal edge, so the division is safe.
    ring_edges(ring).filter(|(a, b)| (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (b.0 - a.0) / (b.1 - a.1)).count() % 2 == 1
}

/// How far along `a`→`b` the point nearest `p` lies, from 0 at `a` to 1 at `b`.
fn param_on_segment(p: V, a: V, b: V) -> f64 {
    let d = b - a;
    let len2 = d.dot(d);
    if len2 > 0.0 {
        ((p - a).dot(d) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn dist_to_segment(p: V, a: V, b: V) -> f64 {
    p.dist(a + (b - a) * param_on_segment(p, a, b))
}

/// Where `a`→`b` properly crosses `c`→`d`, as a parameter along `a`→`b` strictly inside (0, 1).
fn crossing(a: V, b: V, c: V, d: V) -> Option<f64> {
    let (r, s) = (b - a, d - c);
    let den = r.cross(s);
    if den.abs() <= 1e-12 * r.length() * s.length() {
        return None; // parallel
    }
    let t = (c - a).cross(s) / den;
    let u = (c - a).cross(r) / den;
    (t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0).then_some(t)
}

/// The segments cut wherever another one crosses them or ends on them.
fn split(segs: &[(V, V)], tol: f64) -> Vec<(V, V)> {
    let mut out = Vec::new();
    for (i, &(a, b)) in segs.iter().enumerate() {
        let mut ts = vec![0.0, 1.0];
        for (j, &(c, d)) in segs.iter().enumerate() {
            if i == j {
                continue;
            }
            // An end of the other segment on this one: a T-junction or a collinear overlap.
            for q in [c, d] {
                if dist_to_segment(q, a, b) <= tol {
                    ts.push(param_on_segment(q, a, b));
                }
            }
            ts.extend(crossing(a, b, c, d));
        }
        ts.sort_by(f64::total_cmp);
        let pts: Vec<V> = ts.iter().map(|&t| a + (b - a) * t).collect();
        out.extend(pts.windows(2).filter_map(|w| match w {
            [p, q] if p.dist(*q) > tol => Some((*p, *q)),
            _ => None,
        }));
    }
    out
}

/// Every face of the planar graph the segments make, with ends closer than `tol` joined:
/// each bounded face counter-clockwise (positive area), and the outline of each connected
/// piece clockwise (negative area).
pub fn faces(segs: &[(V, V)], tol: f64) -> Vec<Vec<V>> {
    let split = split(segs, tol);
    // Ends closer than `tol` are one node: the first node within `tol` of an end, in the
    // order they were met.
    let mut nodes: Vec<V> = Vec::new();
    let mut node = |p: V| match nodes.iter().position(|q| q.dist(p) <= tol) {
        Some(i) => i,
        None => {
            nodes.push(p);
            nodes.len() - 1
        }
    };
    let mut edges = std::collections::BTreeSet::new();
    for (a, b) in split {
        let (i, j) = (node(a), node(b));
        if i != j {
            edges.insert((i.min(j), i.max(j)));
        }
    }
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for &(i, j) in &edges {
        around[i].push(j);
        around[j].push(i);
    }
    let angle = |from: usize, to: usize| {
        let d = nodes[to] - nodes[from];
        d.1.atan2(d.0)
    };
    for (i, list) in around.iter_mut().enumerate() {
        list.sort_by(|&a, &b| angle(i, a).total_cmp(&angle(i, b)));
    }
    // Walk each half-edge once, keeping the face on the left: at every node take the edge
    // just clockwise of the one we arrived along.
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for &(i, j) in &edges {
        for start in [(i, j), (j, i)] {
            let (mut u, mut v) = start;
            let mut face = Vec::new();
            while seen.insert((u, v)) {
                face.push(nodes[u]);
                let list = &around[v];
                let k = list.iter().position(|&w| w == u).unwrap_or(0);
                let w = list[(k + list.len() - 1) % list.len()];
                (u, v) = (v, w);
            }
            if !face.is_empty() {
                out.push(face);
            }
        }
    }
    out
}

/// A room-shaped region: an outer ring (counter-clockwise) and the islands standing inside it.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub outer: Vec<V>,
    pub holes: Vec<Vec<V>>,
}

impl Region {
    pub fn area(&self) -> f64 {
        area(&self.outer) - self.holes.iter().map(|h| area(h).abs()).sum::<f64>()
    }
    pub fn perimeter(&self) -> f64 {
        perimeter(&self.outer) + self.holes.iter().map(|h| perimeter(h)).sum::<f64>()
    }
}

/// The region around `p` among faces made by [`faces`]: the smallest loop of segments
/// enclosing it, minus any separate pieces standing inside it. `None` when nothing
/// encloses `p`.
pub fn region_in(all: &[Vec<V>], p: V, tol: f64) -> Option<Region> {
    let outer = all.iter().filter(|f| area(f) > tol * tol && contains(f, p)).min_by(|a, b| area(a).total_cmp(&area(b)))?;
    // Outlines of other pieces lying inside it; ours is the one around `p`.
    let islands: Vec<&Vec<V>> = all.iter().filter(|f| area(f) < 0.0 && !contains(f, p) && f.iter().all(|q| contains(outer, *q))).collect();
    let holes = islands.iter().filter(|h| !islands.iter().any(|o| !std::ptr::eq(*o, **h) && h.iter().all(|q| contains(o, *q)))).map(|h| h.iter().rev().copied().collect()).collect();
    Some(Region { outer: outer.clone(), holes })
}

/// Whether `p` lies in region `g`: inside its outer ring, in none of its islands.
pub fn region_holds(g: &Region, p: V) -> bool {
    contains(&g.outer, p) && !g.holes.iter().any(|h| contains(h, p))
}

/// Where a room's area and perimeter are measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomArea {
    /// At the walls' centre lines.
    Centre,
    /// At the walls' finish faces: the floor inside them.
    Finish,
}

/// Why a room has no boundary: no loop of walls closes round its point, or a room placed
/// before it (the one at this place in the list given) has the space it is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unbound {
    NotEnclosed,
    Shared(usize),
}

/// What a storey's rooms are found among, at one measure: the loops its bounding walls'
/// lines make and, at the finish faces, the walls' outlines, a point in one of which is in
/// a wall and not in a room.
pub struct Plan {
    faces: Vec<Vec<V>>,
    walls: Vec<Vec<V>>,
}

impl Plan {
    /// The plan of `bounding` (a storey's room-bounding walls, each able to join the
    /// others) measured at `at`.
    pub fn of(bounding: &[PlanWall], at: RoomArea, tol: f64) -> Plan {
        match at {
            RoomArea::Centre => {
                let lines: Vec<(V, V)> = (0..bounding.len()).map(|i| walls::centre_line(bounding, i, tol)).map(|[a, b]| (a, b)).collect();
                Plan { faces: faces(&lines, tol), walls: Vec::new() }
            }
            RoomArea::Finish => {
                let outlines: Vec<Vec<V>> = (0..bounding.len()).map(|i| walls::outline(bounding, i, tol)).collect();
                let lines: Vec<(V, V)> = outlines.iter().flat_map(|ring| ring_edges(ring)).collect();
                Plan { faces: faces(&lines, tol), walls: outlines }
            }
        }
    }

    /// The boundary of the room whose point is `points[me]`, or why it has none: the loop
    /// round its point, unless its point is in a wall or a room earlier in `points` has
    /// the space (two rooms cannot share a space, so the later is left unbounded).
    pub fn bound(&self, points: &[V], me: usize, tol: f64) -> Result<Region, Unbound> {
        let p = points[me];
        if self.walls.iter().any(|ring| contains(ring, p)) {
            return Err(Unbound::NotEnclosed);
        }
        let g = region_in(&self.faces, p, tol).ok_or(Unbound::NotEnclosed)?;
        match points[..me].iter().position(|q| region_holds(&g, *q)) {
            Some(before) => Err(Unbound::Shared(before)),
            None => Ok(g),
        }
    }
}
