//! The figures that price a house's outside trim (call 185): how far an eave or a rake overhangs the wall
//! under it, a wall's outside corners and the length of corner board they take, and the length of casing
//! round an opening. Fascia and soffit on an eave or a rake, corner boards on a wall, exterior casing on
//! a window or a door are priced by them.
//!
//! Ours. One arithmetic for each: it is worked here, from what is drawn, and the command line (`plumbline
//! trim`) and a cost line's quantity both read it from here. Each rests on the positions it was worked from
//! and is only as sure as the worst of them; where it cannot be worked it is Unknown, with the reason.
//!
//! Given only where a line names it (as a wall's `net_length` is): a thing no line asks the figure of does not
//! carry it, so a job saved before opens and saves the same and nothing in it is reported stale. `figures`
//! works all of them out for a reader that asks (the `trim` verb), whether a line names them or not.
//!
//! The rules, each the call's:
//!
//! * **Overhang** is how far the roof's edge lies outside the outside face of the wall under it, on the plan:
//!   the edge's distance from the wall's line, less the distance from that line to the wall's face on the edge's
//!   side. The wall under an edge is found as a roof is found over a wall (call 180, A2: `carries::edge_on`, by the
//!   plan alone: along the edge within the wall's thickness, or outside it by no more than 4 ft with no other wall
//!   between). Where several walls lie under an edge the one it shares most length with is taken, and of equal ones
//!   the highest. An edge that lies inside the wall's outside face has no overhang and says so.
//! * **Outside corners.** A corner is where two walls of one level end at one point (within the half of their
//!   thicknesses added, and 0.05 ft). It is an outside corner when it points out of the ring of walls it is in:
//!   the ring is followed round from wall to wall, and a corner is outside where the ring turns the way it goes
//!   round (a left turn going counter-clockwise). A straight join (within 1 degree) is no corner; a wall's end
//!   on the middle of another (a tee) or on nothing is no corner. A ring that is not closed, or a point where
//!   three walls end, leaves its corners not worked, and said. A ring lying wholly inside another ring (a closet)
//!   points out of that closet, not out of the house: it has none. A corner is counted on both its walls.
//! * **Casing.** A window's is its head, its two legs and the sill's apron: twice its width and its height. A
//!   door's is its head and its two legs: its width and twice its height.

use std::collections::{HashMap, HashSet};

pub mod lay;
pub use lay::{lay, pieces, Kind as PieceKind, Laid, Piece};

use bevy_ecs::prelude::*;

use crate::building::{self, plan_wall, wall_heights, Tree, Wall};
use crate::carries::{self, Body};
use crate::components::*;
use crate::job::Job;
use crate::value::{Derived, Status};

/// One figure of one thing.
#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub path: String,
    pub name: &'static str,
    pub figure: Derived,
}

/// The figure of an eave or a rake.
pub const OVERHANG: &str = "overhang";
/// The figures of a wall.
pub const OUTSIDE_CORNERS: &str = "outside_corners";
pub const CORNER_BOARD_LENGTH: &str = "corner_board_length";
/// The figure of a window or a door.
pub const CASING_LENGTH: &str = "casing_length";

const TOL: f64 = crate::check::FIT_TOLERANCE_FT;
/// The sine of 1 degree: walls that join within it join straight.
const STRAIGHT: f64 = 0.017_452_406_437_283_512;

/// The figures given only where a line names them.
pub(crate) const GIVEN_WHERE_NAMED: [&str; 5] = [OVERHANG, OUTSIDE_CORNERS, CORNER_BOARD_LENGTH, CASING_LENGTH, crate::gable::GABLE_AREA];

/// The paths of the things of these kinds that have a line naming one of these figures.
pub(crate) fn naming(world: &mut World, kinds: &[Kind], figures: &[&str]) -> HashSet<String> {
    let mut q = world.query::<(&Node, &Kind, &Cost)>();
    q.iter(world).filter(|(_, k, c)| kinds.contains(k) && figures.iter().any(|f| names(Some(c), f))).map(|(n, _, _)| n.path.clone()).collect()
}

/// Does a line of this thing name the figure?
pub(crate) fn names(cost: Option<&Cost>, figure: &str) -> bool {
    cost.is_some_and(|c| c.lines.iter().any(|l| !l.off && crate::formula::parse(&l.quantity).is_ok_and(|e| crate::formula::names(&e).iter().any(|n| n == figure))))
}

pub(crate) fn path_number(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

// ------------------------------------------------------------------------------------------ walls as drawn

pub(crate) struct WallRec {
    pub(crate) path: String,
    pub(crate) storey: String,
    /// Its line, first end and second, in feet.
    pub(crate) line: [P2; 2],
    pub(crate) thick: f64,
    /// Its faces: how far to its left its left face is, and its right face (the same number, less the thickness).
    pub(crate) ext: f64,
    pub(crate) int: f64,
    pub(crate) top: f64,
    pub(crate) base: f64,
    /// As sure as its line and its section.
    pub(crate) status: Status,
    /// Its height, or why it has none; as sure as that.
    pub(crate) height: Result<f64, String>,
    pub(crate) height_status: Status,
}

/// Every wall that can be placed, and why each that cannot be is not.
pub(crate) fn walls_of(world: &mut World, tree: &Tree) -> (Vec<WallRec>, HashMap<String, String>) {
    let mut q = world.query::<(&Node, &Kind, &Wall)>();
    let mut all: Vec<(String, Wall)> = q.iter(world).filter(|(_, k, _)| **k == Kind::Wall).map(|(n, _, w)| (n.path.clone(), w.clone())).collect();
    all.sort_by_key(|(p, _)| path_number(p));
    let (mut out, mut not) = (Vec::new(), HashMap::new());
    for (path, w) in all {
        let Some(storey) = tree.storey_of(&path).map(str::to_string) else {
            not.insert(path, "it is under no level".to_string());
            continue;
        };
        let (placed, status, span, base_status) = match plan_wall(tree, &path, &w) {
            Ok(x) => x,
            Err(why) => {
                not.insert(path, why);
                continue;
            }
        };
        let heights = wall_heights(tree, &path, &w);
        out.push(WallRec {
            storey,
            line: [[placed.start.0, placed.start.1], [placed.end.0, placed.end.1]],
            thick: placed.ext - placed.int,
            ext: placed.ext,
            int: placed.int,
            top: placed.top,
            base: placed.base,
            status,
            height: heights.height.clone(),
            height_status: heights.status.max(span).max(base_status),
            path,
        });
    }
    (out, not)
}

pub(crate) fn body(w: &WallRec) -> Body {
    Body { path: w.path.clone(), storey: w.storey.clone(), line: w.line, thick: w.thick, base: w.base, top: w.top }
}

pub(crate) fn feet_said(x: f64) -> String {
    crate::view::figure_shown(x, 2)
}

// ------------------------------------------------------------------------------------------ overhang

/// Does this roof edge (of the roof whose footprint is `ring`) lie over this wall, by the plan alone (call 180, A2)? Along the edge
/// within the wall's thickness, or outside it by no more than 4 ft with no other wall between. The one rule trim's overhangs and a
/// gable's wall (call 192) both find the wall under an edge by.
pub(crate) fn lies_over(w: &WallRec, bodies: &[Body], edge: [P2; 2], ring: &[P2]) -> Option<carries::EdgeOn> {
    let b = body(w);
    let others: Vec<&Body> = bodies.iter().filter(|o| o.path != w.path).collect();
    let on = carries::edge_on(&b, edge, &others, ring)?;
    (on.across <= w.thick + TOL || (on.across <= carries::ROOF_REACH_FT + TOL && on.outward)).then_some(on)
}


/// The overhang of each eave and rake that has a figure asked (`wanted`: their paths; None for every one), by path.
pub(crate) fn overhangs(world: &mut World, wanted: Option<&HashSet<String>>) -> HashMap<String, Derived> {
    let mut edges: Vec<(String, Run)> = {
        let mut q = world.query::<(&Node, &Kind, &Run)>();
        q.iter(world).filter(|(n, k, _)| matches!(**k, Kind::Eave | Kind::Rake) && wanted.is_none_or(|w| w.contains(&n.path))).map(|(n, _, r)| (n.path.clone(), r.clone())).collect()
    };
    edges.sort_by_key(|(p, _)| path_number(p));
    if edges.is_empty() {
        return HashMap::new();
    }
    let tree = Tree::of(world);
    let (walls, _) = walls_of(world, &tree);
    let bodies: Vec<Body> = walls.iter().map(body).collect();
    let parents: HashMap<String, Option<String>> = {
        let mut q = world.query::<&Node>();
        q.iter(world).map(|n| (n.path.clone(), n.parent.clone())).collect()
    };
    let rings: HashMap<String, Option<Vec<P2>>> = {
        let mut q = world.query::<(&Node, &Footprint)>();
        q.iter(world).map(|(n, f)| (n.path.clone(), f.points_ft())).collect()
    };
    let formula = "how far the roof's edge lies outside the outside face of the wall under it, on the plan: its distance from the wall's line, less the distance from that line to the wall's face on its side";
    let mut out = HashMap::new();
    for (path, run) in edges {
        let mine = vec![format!("{path}#{K_RUN}")];
        let refused = |why: &str| Derived::refused("ft", formula, &mine, why);
        let Some(line) = run.ends_ft() else {
            out.insert(path.clone(), refused("its ends are not known"));
            continue;
        };
        let roof = run.planes.first().and_then(|p| parents.get(p)).cloned().flatten();
        let Some(Some(ring)) = roof.as_ref().and_then(|r| rings.get(r)).cloned() else {
            out.insert(path.clone(), refused("its roof has no footprint on the plan to find the wall under it by"));
            continue;
        };
        // The walls under it: by the plan alone, with the length each shares with it.
        let mut under: Vec<(f64, &WallRec, f64)> = Vec::new();
        for w in &walls {
            let Some(on) = lies_over(w, &bodies, line, &ring) else { continue };
            let (dx, dy) = (w.line[1][0] - w.line[0][0], w.line[1][1] - w.line[0][1]);
            let long = dx.hypot(dy);
            let s = |p: P2| ((p[0] - w.line[0][0]) * dx + (p[1] - w.line[0][1]) * dy) / long;
            let shared = (s(line[0]).max(s(line[1])).min(long) - s(line[0]).min(s(line[1])).max(0.0)).max(0.0);
            under.push((shared, w, on.across));
        }
        under.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.top.total_cmp(&a.1.top)).then(path_number(&a.1.path).cmp(&path_number(&b.1.path))));
        let Some((_, w, _)) = under.first() else {
            out.insert(path.clone(), refused("no wall is found under this edge: none lies along it within 4 ft with no other wall between"));
            continue;
        };
        let (dx, dy) = (w.line[1][0] - w.line[0][0], w.line[1][1] - w.line[0][1]);
        let long = dx.hypot(dy);
        let lateral = |p: P2| (dx * (p[1] - w.line[0][1]) - dy * (p[0] - w.line[0][0])) / long;
        let across = (lateral(line[0]) + lateral(line[1])) / 2.0;
        // The wall's face on the edge's side of its line.
        let face = if across >= 0.0 { w.ext } else { -w.int };
        let over = across.abs() - face;
        let inputs = vec![format!("{path}#{K_RUN}"), format!("{}#{}", w.path, building::K_WALL)];
        out.insert(
            path.clone(),
            if over < -TOL {
                let mut d = Derived::refused("ft", formula, &inputs, &format!("this edge lies {} ft inside the wall's outside face ({}): a roof edge is outside the wall it sits over", feet_said(-over), w.path));
                d.status = Status::Unknown;
                d
            } else {
                Derived::new(Some(over.max(0.0)), "ft", &[run.ends.status, w.status], formula, &inputs)
            },
        );
    }
    out
}

// ------------------------------------------------------------------------------------------ soffit at a corner

/// A corner where two roof edges (eaves or rakes) of one roof meet and their soffits overlap (call 191 item 12): the soffit taken off as
/// the total length of the edges times the overhang counts the square at the corner in both runs. That is how soffit is taken off
/// (BYU-Idaho's construction estimating text, books.byui.edu/construction_estimat/exterior_finishes_es; Angi's and Hunker's soffit
/// guides), with waste added on top: the formula stays, and the figure says how much of it is corner counted twice.
#[derive(Clone, Debug, PartialEq)]
pub struct Corner {
    pub a: String,
    pub b: String,
    /// Where the two edges meet, in feet.
    pub at: P2,
    /// The area both runs count, in SF; at an inside corner, the area neither counts.
    pub area: f64,
    /// An inside corner (the roof's edge turns in): the two runs leave the square at it uncovered, and count it in neither.
    pub inside: bool,
}

/// The intersection of two convex polygons (Sutherland-Hodgman), as a polygon.
fn clip(subject: Vec<P2>, clipper: &[P2]) -> Vec<P2> {
    let mut out = subject;
    for k in 0..clipper.len() {
        let (a, b) = (clipper[k], clipper[(k + 1) % clipper.len()]);
        let side = |p: P2| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        let input = std::mem::take(&mut out);
        for i in 0..input.len() {
            let (p, q) = (input[i], input[(i + 1) % input.len()]);
            let (sp, sq) = (side(p), side(q));
            if sp >= 0.0 {
                out.push(p);
            }
            if (sp >= 0.0) != (sq >= 0.0) {
                let t = sp / (sp - sq);
                out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
            }
        }
        if out.is_empty() {
            break;
        }
    }
    out
}

fn area_of(poly: &[P2]) -> f64 {
    (0..poly.len()).map(|k| poly[k][0] * poly[(k + 1) % poly.len()][1] - poly[(k + 1) % poly.len()][0] * poly[k][1]).sum::<f64>().abs() / 2.0
}

/// Every corner where the soffits of two edges of one roof lie over one another, worked from the overhangs and the plan alone.
pub fn soffit_corners(world: &mut World) -> Vec<Corner> {
    let overhang = overhangs(world, None);
    let mut edges: Vec<(String, [P2; 2], Option<String>, f64)> = Vec::new();
    {
        let parents: HashMap<String, Option<String>> = {
            let mut q = world.query::<&Node>();
            q.iter(world).map(|n| (n.path.clone(), n.parent.clone())).collect()
        };
        let rings: HashMap<String, Option<Vec<P2>>> = {
            let mut q = world.query::<(&Node, &Footprint)>();
            q.iter(world).map(|(n, f)| (n.path.clone(), f.points_ft())).collect()
        };
        let mut q = world.query::<(&Node, &Kind, &Run)>();
        let mut all: Vec<(String, Run)> = q.iter(world).filter(|(_, k, _)| matches!(**k, Kind::Eave | Kind::Rake)).map(|(n, _, r)| (n.path.clone(), r.clone())).collect();
        all.sort_by_key(|(p, _)| path_number(p));
        for (path, run) in all {
            let (Some(line), Some(d)) = (run.ends_ft(), overhang.get(&path).and_then(|o| o.worked())) else { continue };
            if d <= TOL {
                continue;
            }
            let roof = run.planes.first().and_then(|p| parents.get(p)).cloned().flatten();
            let ring = roof.as_ref().and_then(|r| rings.get(r)).cloned().flatten();
            // Which side of the edge is the roof: the side a short step in from its middle lies inside the footprint.
            let Some(ring) = ring else { continue };
            let (dx, dy) = (line[1][0] - line[0][0], line[1][1] - line[0][1]);
            let long = dx.hypot(dy);
            if long < TOL {
                continue;
            }
            let n = [-dy / long, dx / long];
            let mid = [(line[0][0] + line[1][0]) / 2.0, (line[0][1] + line[1][1]) / 2.0];
            let step = 0.02_f64.min(long / 4.0);
            let inward = if crate::vif::inside(&ring, [mid[0] + n[0] * step, mid[1] + n[1] * step]) { n } else { [-n[0], -n[1]] };
            edges.push((path, line, roof, d));
            let last = edges.len() - 1;
            // Keep the inward side with the edge by turning the line's ends so that (end 1 - end 0) rotated left is inward.
            if inward != n {
                edges[last].1 = [line[1], line[0]];
            }
        }
    }
    let mut out = Vec::new();
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            if edges[i].2 != edges[j].2 || edges[i].2.is_none() {
                continue;
            }
            for ea in 0..2 {
                for eb in 0..2 {
                    let (p, q) = (edges[i].1[ea], edges[j].1[eb]);
                    if (p[0] - q[0]).hypot(p[1] - q[1]) > 0.05 {
                        continue;
                    }
                    // The soffit of an edge from the corner: along it to its far end, and in from it by its overhang.
                    let rect = |k: usize, e: usize| -> Vec<P2> {
                        let (line, d) = (edges[k].1, edges[k].3);
                        let (near, far) = (line[e], line[1 - e]);
                        let (dx, dy) = (far[0] - near[0], far[1] - near[1]);
                        let long = dx.hypot(dy);
                        // Left of the edge's own direction (end 0 to end 1) is in; going from end 1, in is to the right.
                        let left = [-dy / long, dx / long];
                        let n = if e == 0 { left } else { [-left[0], -left[1]] };
                        vec![near, far, [far[0] + n[0] * d, far[1] + n[1] * d], [near[0] + n[0] * d, near[1] + n[1] * d]]
                    };
                    let (ra, rb) = (rect(i, ea), rect(j, eb));
                    // Both rectangles are wound the same way for the clip: make each counter-clockwise.
                    let ccw = |mut r: Vec<P2>| {
                        let signed: f64 = (0..4).map(|k| r[k][0] * r[(k + 1) % 4][1] - r[(k + 1) % 4][0] * r[k][1]).sum();
                        if signed < 0.0 {
                            r.reverse();
                        }
                        r
                    };
                    let (ra, rb) = (ccw(ra), ccw(rb));
                    let both = clip(ra, &rb);
                    let area = if both.len() >= 3 { area_of(&both) } else { 0.0 };
                    if area > 1e-6 {
                        out.push(Corner { a: edges[i].0.clone(), b: edges[j].0.clone(), at: p, area, inside: false });
                    } else {
                        // Edge `j` leaving the corner on the outer side of edge `i`'s soffit: the roof turns in, and the strips each side
                        // of the two edges leave the parallelogram between them (d x d at a right angle) in neither (call 191 item 17).
                        let unit = |k: usize, e: usize| {
                            let (near, far) = (edges[k].1[e], edges[k].1[1 - e]);
                            let (dx, dy) = (far[0] - near[0], far[1] - near[1]);
                            let long = dx.hypot(dy);
                            [dx / long, dy / long]
                        };
                        let (ui, uj) = (unit(i, ea), unit(j, eb));
                        let left = [-ui[1], ui[0]];
                        let inward = if ea == 0 { left } else { [-left[0], -left[1]] };
                        let sin = (ui[0] * uj[1] - ui[1] * uj[0]).abs();
                        if sin > 1e-3 && uj[0] * inward[0] + uj[1] * inward[1] < -1e-3 {
                            out.push(Corner { a: edges[i].0.clone(), b: edges[j].0.clone(), at: p, area: edges[i].3 * edges[j].3 / sin, inside: true });
                        }
                    }
                }
            }
        }
    }
    out
}

// ------------------------------------------------------------------------------------------ corners

/// A wall's two ends, and the walls that end near each.
struct Ends<'a> {
    walls: &'a [&'a WallRec],
    /// For each wall and each end: the (wall, end) pairs that end at the same point.
    meets: Vec<[Vec<(usize, usize)>; 2]>,
}

fn point_of(w: &WallRec, end: usize) -> P2 {
    w.line[end]
}

fn direction_out(w: &WallRec, end: usize) -> P2 {
    // Unit vector along the wall pointing away from `end`.
    let (a, b) = (w.line[end], w.line[1 - end]);
    let len = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-12);
    [(b[0] - a[0]) / len, (b[1] - a[1]) / len]
}

impl<'a> Ends<'a> {
    fn of(walls: &'a [&'a WallRec]) -> Ends<'a> {
        let mut meets = vec![[Vec::new(), Vec::new()]; walls.len()];
        for (i, a) in walls.iter().enumerate() {
            for (j, b) in walls.iter().enumerate() {
                if i == j {
                    continue;
                }
                let near = (a.thick + b.thick) / 2.0 + 0.05;
                for ea in 0..2 {
                    for eb in 0..2 {
                        let (p, q) = (point_of(a, ea), point_of(b, eb));
                        if (p[0] - q[0]).hypot(p[1] - q[1]) <= near {
                            meets[i][ea].push((j, eb));
                        }
                    }
                }
            }
        }
        Ends { walls, meets }
    }

    /// Walk the ring from wall `start` out of its end `out`: the walls in order, each with the end it is entered
    /// at, back to `start`; or why it does not close.
    fn ring(&self, start: usize, out: usize) -> Result<Vec<(usize, usize)>, String> {
        let mut ring = vec![(start, 1 - out)];
        let (mut at, mut leave) = (start, out);
        loop {
            let here = &self.meets[at][leave];
            match here.as_slice() {
                [] => return Err(format!("its ring of walls is not closed: {} ends free", self.walls[at].path)),
                [(next, entered)] => {
                    if *next == start {
                        return if *entered == 1 - out { Ok(ring) } else { Err("its ring of walls is not closed: it comes back to the wall it began at by its far end".to_string()) };
                    }
                    if ring.iter().any(|(w, _)| w == next) {
                        return Err(format!("its ring of walls is not closed: {} is met twice going round", self.walls[*next].path));
                    }
                    ring.push((*next, *entered));
                    at = *next;
                    leave = 1 - *entered;
                }
                many => {
                    return Err(format!("{} walls end at one point where {} ends, so which of its corners point out is not worked", many.len() + 1, self.walls[at].path));
                }
            }
        }
    }
}

/// How each wall's ends are, one level at a time: for wall `i`, end `e`: is it an outside corner, and what the answer
/// rests on, or why it is not worked.
fn end_of(ends: &Ends, all: &[&WallRec], i: usize, e: usize) -> Result<(bool, Vec<usize>), String> {
    let w = all[i];
    match ends.meets[i][e].as_slice() {
        // Free, or on the middle of another wall: no corner.
        [] => Ok((false, Vec::new())),
        [(j, ej)] => {
            let (u, v) = (direction_out(w, e), direction_out(all[*j], *ej));
            // A straight join is no corner.
            if (u[0] * v[1] - u[1] * v[0]).abs() <= STRAIGHT {
                return Ok((false, Vec::new()));
            }
            // Which side is the house: the ring this corner is in, followed round from this wall away from this end.
            let ring = ends.ring(i, 1 - e)?;
            // Points of the ring, in order, and which way it goes round.
            let points: Vec<P2> = ring.iter().map(|(w, entered)| point_of(all[*w], *entered)).collect();
            let area: f64 = (0..points.len()).map(|k| points[k][0] * points[(k + 1) % points.len()][1] - points[(k + 1) % points.len()][0] * points[k][1]).sum::<f64>() / 2.0;
            if area.abs() <= TOL {
                return Err("its ring of walls encloses nothing".to_string());
            }
            // The ring was followed from this wall away from this end, so it comes back to this end last: it arrives along the
            // wall it meets here (travelling toward this end, against `v`) and goes on along this wall (along `u`). It turns
            // the way it goes round when the turn from one to the other has the sign of the ring's area.
            let turn = (-v[0]) * u[1] - (-v[1]) * u[0];
            let convex = turn * area > 0.0;
            // Inside another ring (a closet): it points out of that, not out of the house.
            let inside_another = (0..all.len()).any(|k| {
                if ring.iter().any(|(w, _)| *w == k) || all[k].storey != w.storey {
                    return false;
                }
                [0usize, 1].iter().any(|&out| ends.ring(k, out).ok().is_some_and(|other| {
                    let outer: Vec<P2> = other.iter().map(|(w, entered)| point_of(all[*w], *entered)).collect();
                    !ring.iter().any(|(w, _)| other.iter().any(|(x, _)| x == w)) && points.iter().all(|p| crate::vif::inside(&outer, *p))
                }))
            });
            Ok((convex && !inside_another, ring.iter().map(|(w, _)| *w).collect()))
        }
        many => Err(format!("{} walls end at one point where {} ends, so which of its corners point out is not worked", many.len() + 1, w.path)),
    }
}

/// Every wall's outside corners and corner-board length, by path (`wanted`: their paths; None for every wall).
pub(crate) fn corners(world: &mut World, wanted: Option<&HashSet<String>>) -> HashMap<String, [Derived; 2]> {
    let tree = Tree::of(world);
    let (walls, not_placed) = walls_of(world, &tree);
    let mut out = HashMap::new();
    let f_count = "the ends of this wall that are outside corners of the ring of walls it is in, a corner that points out of the house: counted on both its walls";
    let f_board = "outside_corners × height: a board up each outside corner, one on each wall's face";
    for (path, why) in &not_placed {
        if wanted.is_none_or(|x| x.contains(path)) {
            let mine = vec![format!("{path}#{}", building::K_WALL)];
            let why = format!("the wall cannot be placed ({why})");
            out.insert(path.clone(), [Derived::refused("EA", f_count, &mine, &why), Derived::refused("LF", f_board, &mine, &why)]);
        }
    }
    for storey in walls.iter().map(|w| w.storey.clone()).collect::<std::collections::BTreeSet<_>>() {
        let level: Vec<&WallRec> = walls.iter().filter(|w| w.storey == storey).collect();
        let ends = Ends::of(&level);
        for (i, w) in level.iter().enumerate() {
            if wanted.is_some_and(|x| !x.contains(&w.path)) {
                continue;
            }
            let mine = vec![format!("{}#{}", w.path, building::K_WALL)];
            let mut count = 0.0;
            let mut statuses = vec![w.status];
            let mut inputs = mine.clone();
            let mut fault: Option<String> = None;
            for e in 0..2 {
                match end_of(&ends, &level, i, e) {
                    Ok((outside, ring)) => {
                        if outside {
                            count += 1.0;
                        }
                        for r in ring {
                            statuses.push(level[r].status);
                            let input = format!("{}#{}", level[r].path, building::K_WALL);
                            if !inputs.contains(&input) {
                                inputs.push(input);
                            }
                        }
                    }
                    Err(why) => {
                        fault.get_or_insert(why);
                    }
                }
            }
            let n = match &fault {
                Some(why) => Derived::refused("EA", f_count, &mine, why),
                None => Derived::new(Some(count), "EA", &statuses, f_count, &inputs),
            };
            let l = match (&n.value, &w.height) {
                (None, _) => Derived::refused("LF", f_board, &mine, &n.note),
                (_, Err(why)) => Derived::refused("LF", f_board, &mine, why),
                (Some(_), Ok(h)) => Derived::new(Some(n.worked().unwrap_or(0.0) * h), "LF", &[n.status, w.height_status], f_board, &inputs),
            };
            out.insert(w.path.clone(), [n, l]);
        }
    }
    out
}

// ------------------------------------------------------------------------------------------ casing

/// An opening's casing length from its width and height (as figured by `building::measures`).
pub(crate) fn casing(kind: Kind, width: Option<&Derived>, height: Option<&Derived>, inputs: &[String]) -> Derived {
    let f = if kind == Kind::Window { "2 × (width + height): the head, the two legs and the sill's apron" } else { "width + 2 × height: the head and the two legs" };
    let (Some(w), Some(h)) = (width, height) else { return Derived::refused("LF", f, inputs, "it has no size") };
    match (w.worked(), h.worked()) {
        (Some(w_ft), Some(h_ft)) => Derived::new(Some(if kind == Kind::Window { 2.0 * (w_ft + h_ft) } else { w_ft + 2.0 * h_ft }), "LF", &[w.status, h.status], f, inputs),
        _ => Derived::refused("LF", f, inputs, if !w.note.is_empty() { &w.note } else if !h.note.is_empty() { &h.note } else { "its width or its height is not known" }),
    }
}

// ------------------------------------------------------------------------------------------ for a reader

/// Every figure of every eave, rake, wall, window and door in the job, in the order of their paths, worked out
/// whether a line names it or not.
pub fn figures(job: &mut Job) -> Vec<Figure> {
    let mut out: Vec<Figure> = Vec::new();
    let overhangs = overhangs(&mut job.world, None);
    let corners = corners(&mut job.world, None);
    let mut q = job.world.query::<(&Node, &Kind, Option<&Measure>)>();
    let mut things: Vec<(String, Kind, Option<Measure>)> = q.iter(&job.world).map(|(n, k, m)| (n.path.clone(), *k, m.cloned())).collect();
    things.sort_by_key(|(p, ..)| path_number(p));
    for (path, kind, measure) in things {
        match kind {
            Kind::Eave | Kind::Rake => {
                if let Some(d) = overhangs.get(&path) {
                    out.push(Figure { path, name: OVERHANG, figure: d.clone() });
                }
            }
            Kind::Wall => {
                if let Some([n, l]) = corners.get(&path) {
                    out.push(Figure { path: path.clone(), name: OUTSIDE_CORNERS, figure: n.clone() });
                    out.push(Figure { path, name: CORNER_BOARD_LENGTH, figure: l.clone() });
                }
            }
            Kind::Window | Kind::Door => {
                let m = measure.unwrap_or_default();
                let inputs = vec![format!("{path}#{}", building::K_OPENING)];
                out.push(Figure { path, name: CASING_LENGTH, figure: casing(kind, m.0.get("width"), m.0.get("height"), &inputs) });
            }
            _ => {}
        }
    }
    out
}
