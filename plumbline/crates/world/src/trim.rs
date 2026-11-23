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
pub(crate) const GIVEN_WHERE_NAMED: [&str; 4] = [OVERHANG, OUTSIDE_CORNERS, CORNER_BOARD_LENGTH, CASING_LENGTH];

/// The paths of the things of these kinds that have a line naming one of these figures.
pub(crate) fn naming(world: &mut World, kinds: &[Kind], figures: &[&str]) -> HashSet<String> {
    let mut q = world.query::<(&Node, &Kind, &Cost)>();
    q.iter(world).filter(|(_, k, c)| kinds.contains(k) && figures.iter().any(|f| names(Some(c), f))).map(|(n, _, _)| n.path.clone()).collect()
}

/// Does a line of this thing name the figure?
pub(crate) fn names(cost: Option<&Cost>, figure: &str) -> bool {
    cost.is_some_and(|c| c.lines.iter().any(|l| !l.off && crate::formula::parse(&l.quantity).is_ok_and(|e| crate::formula::names(&e).iter().any(|n| n == figure))))
}

fn path_number(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

// ------------------------------------------------------------------------------------------ walls as drawn

struct WallRec {
    path: String,
    storey: String,
    /// Its line, first end and second, in feet.
    line: [P2; 2],
    thick: f64,
    /// Its faces: how far to its left its left face is, and its right face (the same number, less the thickness).
    ext: f64,
    int: f64,
    top: f64,
    base: f64,
    /// As sure as its line and its section.
    status: Status,
    /// Its height, or why it has none; as sure as that.
    height: Result<f64, String>,
    height_status: Status,
}

/// Every wall that can be placed, and why each that cannot be is not.
fn walls_of(world: &mut World, tree: &Tree) -> (Vec<WallRec>, HashMap<String, String>) {
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

fn body(w: &WallRec) -> Body {
    Body { path: w.path.clone(), storey: w.storey.clone(), line: w.line, thick: w.thick, base: w.base, top: w.top }
}

fn feet_said(x: f64) -> String {
    crate::view::figure_shown(x, 2)
}

// ------------------------------------------------------------------------------------------ overhang

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
            let b = body(w);
            let others: Vec<&Body> = bodies.iter().filter(|o| o.path != w.path).collect();
            let Some(on) = carries::edge_on(&b, line, &others, &ring) else { continue };
            if !(on.across <= w.thick + TOL || (on.across <= carries::ROOF_REACH_FT + TOL && on.outward)) {
                continue;
            }
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
