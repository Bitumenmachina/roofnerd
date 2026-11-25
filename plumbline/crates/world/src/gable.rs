//! A wall under a rake, or under the high edge of a shed roof, rises to the roof (call 192, the model's half).
//!
//! Under an eave a wall stops at its top and the soffit closes it. Under a rake (a gable end; the sloping sides of a shed
//! roof) and under the high edge of a shed roof the wall is carried up to the roof's surface over the wall's line, and the
//! triangle or trapezoid between the wall's top and the roof is the gable: bought as siding, sheathing and weather barrier,
//! and never counted in the wall's `net_area`. Two figures of the wall, worked out here and never typed:
//!
//! * `gable_area` (SF): the area between the wall's top and the roof's surface over its line, over the stretch of the wall's
//!   line a rake or a high edge lies over (end to end of the wall where the roof's edge runs the whole of it);
//! * the wall's top as a line of points, `(feet along the wall's line from its first end, feet above the job's datum)`, which
//!   is what the window draws the wall up to.
//!
//! Ours. BIMCraft attaches a wall's top to a roof as a setting typed on the wall (`attach_top`, `walls.rs` `Bound`, `prisms`);
//! nothing there finds a gable. Here the roof over a wall is found as trim finds it (`trim::lies_over`, by the plan alone) and
//! the roof's height is `roof::SolidPlane::height_at`; the one rule taken from BIMCraft is its `Bound::over`: where two planes
//! cover a point, the lower is the roof's surface there. Nothing is saved: a job read again works it out again.
//!
//! What is an eave and what is not is told by the kind of the roof's edge over the wall, the model's own: a one-plane Eave is
//! one; a one-plane Rake is not; a one-plane Ridge (a level high edge only one plane has: a shed's top) or Headwall is not.
//! A hip roof has eaves all round and no wall of it changes. A line two planes share (a ridge, hip, valley) is never over a
//! wall as an edge of the roof. The roof's thickness is not taken: the roof is its plane.
//!
//! Each figure is as sure as the worst of what it rests on (the wall's line and height, the roof's lines and planes) and says
//! what that is. "None" is worked, not missing: a wall under an eave, or with no roof over it, has no gable, says so, and is
//! Known (nothing it rests on can add one); a gable that cannot be worked (a plane of the roof that cannot be placed) is
//! Unknown and says why, and a line that buys it has no figure.
//!
//! Given only where a line of the wall names it, as the trim's figures are (`trim::names`): a thing no line asks the figure of
//! does not carry it, so a job saved before opens and saves the same and nothing in it is reported stale. `figures` works all
//! of them out for a reader that asks (the `gable` verb), whether a line names them or not.

use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;

use crate::building::{self, Tree};
use crate::carries::Body;
use crate::components::*;
use crate::geom;
use crate::job::Job;
use crate::roof::{self, SolidPlane};
use crate::trim::{self, body, lies_over, path_number, walls_of};
use crate::value::{Derived, Status};

/// The figure of a wall.
pub const GABLE_AREA: &str = "gable_area";

const TOL: f64 = crate::check::FIT_TOLERANCE_FT;

const FORMULA: &str = "the area between the wall's top and the roof's surface over its line, over the stretch of the wall a rake or the high edge of a shed roof lies over: at each point the roof's eave height plus its rise at its pitch over the distance from its eave line, less the wall's top";

/// A wall's gable: its area, and its top.
#[derive(Clone, Debug, PartialEq)]
pub struct Gable {
    pub path: String,
    /// The area, or why there is none (value 0 and a note: "none: ..."), or why it is not known.
    pub area: Derived,
    /// The wall's top over the stretch that rises, `(feet along the wall's line from its first end, feet above the datum)`;
    /// empty where the wall has no gable.
    pub top: Vec<(f64, f64)>,
    /// Why there is none, where there is none ("an eave is over it"): kept off the figure's note, which a line that prices it
    /// would carry into its quantity, so a house with no gable is priced and noted as it was.
    pub why: String,
}

/// A roof line over a wall.
struct Over {
    kind: Kind,
    path: String,
    /// The stretch of the wall's line it lies over, as feet along it.
    span: (f64, f64),
    status: Status,
    /// The plane whose edge it is.
    plane: String,
}

/// A roof line as the model holds it.
struct Line {
    kind: Kind,
    path: String,
    ends: [P2; 2],
    status: Status,
    planes: Vec<String>,
}

fn none(path: &str, why: &str, statuses: &[Status], inputs: Vec<String>) -> Gable {
    let area = Derived::new(Some(0.0), "SF", statuses, &format!("none: {why}"), &inputs);
    Gable { path: path.to_string(), area, top: Vec::new(), why: why.to_string() }
}

fn refused(path: &str, why: &str, inputs: &[String]) -> Gable {
    Gable { path: path.to_string(), area: Derived::refused("SF", FORMULA, inputs, why), top: Vec::new(), why: String::new() }
}

fn cross(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// The roof's surface over a point: of the planes that cover it, the lower.
fn roof_over(planes: &[SolidPlane], p: P2) -> Option<&SolidPlane> {
    planes.iter().filter(|pl| crate::vif::inside(&pl.points, p) || geom::point_to_outline(p, &pl.points) <= TOL).min_by(|a, b| a.height_at(p).total_cmp(&b.height_at(p)))
}

/// Every gable of the walls asked for (`wanted`: their paths; None for every wall), by path.
pub(crate) fn gables(world: &mut World, wanted: Option<&HashSet<String>>) -> HashMap<String, Gable> {
    let asked = |p: &str| wanted.is_none_or(|w| w.contains(p));
    let tree = Tree::of(world);
    let (walls, not_placed) = walls_of(world, &tree);
    let mut out: HashMap<String, Gable> = HashMap::new();
    for (path, why) in &not_placed {
        if asked(path) {
            out.insert(path.clone(), refused(path, &format!("the wall cannot be placed ({why})"), &[format!("{path}#{}", building::K_WALL)]));
        }
    }
    if walls.iter().all(|w| !asked(&w.path)) {
        return out;
    }
    let bodies: Vec<Body> = walls.iter().map(body).collect();

    // The roof's lines that can lie over a wall: one plane's own edges. A line two planes share is inside the roof.
    let mut lines: Vec<Line> = {
        let mut q = world.query::<(&Node, &Kind, &Run)>();
        q.iter(world)
            .filter(|(_, k, r)| matches!(**k, Kind::Eave | Kind::Rake | Kind::Ridge | Kind::Headwall) && r.planes.len() == 1)
            .filter_map(|(n, k, r)| r.ends_ft().map(|ends| Line { kind: *k, path: n.path.clone(), ends, status: r.ends.status, planes: r.planes.clone() }))
            .collect()
    };
    lines.sort_by_key(|l| path_number(&l.path));
    let unplaced_lines = {
        let mut q = world.query::<(&Kind, &Run)>();
        q.iter(world).filter(|(k, r)| matches!(**k, Kind::Eave | Kind::Rake | Kind::Ridge | Kind::Headwall) && r.ends_ft().is_none()).count()
    };
    let parents: HashMap<String, Option<String>> = {
        let mut q = world.query::<&Node>();
        q.iter(world).map(|n| (n.path.clone(), n.parent.clone())).collect()
    };
    let rings: HashMap<String, Option<Vec<P2>>> = {
        let mut q = world.query::<(&Node, &Footprint)>();
        q.iter(world).map(|(n, f)| (n.path.clone(), f.points_ft())).collect()
    };
    let outlines: HashMap<String, Option<Vec<P2>>> = {
        let mut q = world.query::<(&Node, &Outline)>();
        q.iter(world).map(|(n, o)| (n.path.clone(), o.points_ft())).collect()
    };
    let (planes, left_out) = {
        let mut q = world.query::<(Entity, &Node)>();
        let mut order: Vec<(u64, Entity)> = q.iter(world).map(|(e, n)| (path_number(&n.path), e)).collect();
        order.sort();
        let order: Vec<Entity> = order.into_iter().map(|(_, e)| e).collect();
        roof::solid_planes_in(world, &order)
    };
    let planes_exist = !planes.is_empty() || !left_out.is_empty();

    // Which roof lines lie over which wall, and over what stretch of it.
    let mut over: Vec<Vec<Over>> = walls.iter().map(|_| Vec::new()).collect();
    for l in &lines {
        let plane = &l.planes[0];
        let ring = match parents.get(plane).cloned().flatten().and_then(|r| rings.get(&r).cloned().flatten()) {
            Some(r) => r,
            None => match outlines.get(plane).cloned().flatten() {
                Some(o) => o,
                None => continue,
            },
        };
        let mut found: Vec<(usize, (f64, f64))> = Vec::new();
        for (i, w) in walls.iter().enumerate() {
            if lies_over(w, &bodies, l.ends, &ring).is_none() {
                continue;
            }
            let (dx, dy) = (w.line[1][0] - w.line[0][0], w.line[1][1] - w.line[0][1]);
            let long = dx.hypot(dy);
            let s = |p: P2| ((p[0] - w.line[0][0]) * dx + (p[1] - w.line[0][1]) * dy) / long;
            let (lo, hi) = (s(l.ends[0]).min(s(l.ends[1])).max(0.0), s(l.ends[0]).max(s(l.ends[1])).min(long));
            if hi - lo > TOL {
                found.push((i, (lo, hi)));
            }
        }
        // A wall of a level below, under a wall of the same line above, has the roof over the one above: only the highest.
        let kept: Vec<(usize, (f64, f64))> = found
            .iter()
            .filter(|(i, a)| !found.iter().any(|(j, b)| j != i && walls[*j].storey != walls[*i].storey && walls[*j].top > walls[*i].top + TOL && a.1.min(b.1) - a.0.max(b.0) > TOL))
            .copied()
            .collect();
        for (i, span) in kept {
            over[i].push(Over { kind: l.kind, path: l.path.clone(), span, status: l.status, plane: l.planes[0].clone() });
        }
    }

    for (i, w) in walls.iter().enumerate() {
        if !asked(&w.path) {
            continue;
        }
        let mine = format!("{}#{}", w.path, building::K_WALL);
        let rising: Vec<&Over> = over[i].iter().filter(|o| o.kind != Kind::Eave).collect();
        if rising.is_empty() {
            let eaves: Vec<&Over> = over[i].iter().filter(|o| o.kind == Kind::Eave).collect();
            let mut inputs = vec![mine];
            inputs.extend(eaves.iter().map(|o| format!("{}#{K_RUN}", o.path)));
            // "None" under an eave rests on where the roof's edge lies and what kind it is: the wall's line and top, the eave's line, the
            // plane. Where there is no roof it rests on nothing in the job, and is Known.
            let mut statuses = vec![w.status, w.height_status];
            for o in &eaves {
                statuses.push(o.status);
                statuses.push(planes.iter().find(|p| p.path == o.plane).map(|p| p.status).unwrap_or(Status::Assumed));
                inputs.push(format!("{}#{K_OUTLINE}", o.plane));
            }
            let mut g = if !eaves.is_empty() {
                none(&w.path, "an eave is over it", &statuses, inputs)
            } else if !planes_exist {
                none(&w.path, "no roof is drawn over it", &[Status::Known], inputs)
            } else {
                none(&w.path, "no edge of a roof lies over it", &[Status::Known], inputs)
            };
            if unplaced_lines > 0 && eaves.is_empty() {
                g.why.push_str(&format!("; {unplaced_lines} roof line{} whose ends are not known could not be looked for over it", if unplaced_lines == 1 { "" } else { "s" }));
            }
            out.insert(w.path.clone(), g);
            continue;
        }
        let wall_top = match &w.height {
            Ok(_) => w.top,
            Err(why) => {
                out.insert(w.path.clone(), refused(&w.path, why, &[mine]));
                continue;
            }
        };
        // The stretch the rake or the high edge lies over: one, or the figure is not worked.
        let mut spans: Vec<(f64, f64)> = rising.iter().map(|o| o.span).collect();
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut merged: Vec<(f64, f64)> = Vec::new();
        for s in spans {
            match merged.last_mut() {
                Some(last) if s.0 <= last.1 + TOL => last.1 = last.1.max(s.1),
                _ => merged.push(s),
            }
        }
        let mut inputs = vec![mine.clone()];
        inputs.extend(rising.iter().map(|o| format!("{}#{K_RUN}", o.path)));
        if merged.len() > 1 {
            out.insert(w.path.clone(), refused(&w.path, "a rake or a high edge lies over separate stretches of this wall, and a gable over each is not worked", &inputs));
            continue;
        }
        let (s0, s1) = merged[0];
        let long = (w.line[1][0] - w.line[0][0]).hypot(w.line[1][1] - w.line[0][1]);
        let u = [(w.line[1][0] - w.line[0][0]) / long, (w.line[1][1] - w.line[0][1]) / long];
        let at = |s: f64| -> P2 { [w.line[0][0] + u[0] * s, w.line[0][1] + u[1] * s] };
        // Where along the stretch the roof's planes begin and end over the wall's line.
        let mut cuts: Vec<f64> = vec![s0, s1];
        for pl in &planes {
            let n = pl.points.len();
            for k in 0..n {
                let (q0, q1) = (pl.points[k], pl.points[(k + 1) % n]);
                let e = [q1[0] - q0[0], q1[1] - q0[1]];
                let denom = cross(u, e);
                if denom.abs() < 1e-9 {
                    // Along the line: where the edge's ends lie on it.
                    for q in [q0, q1] {
                        if geom::point_to_line(q, w.line[0], w.line[1]).is_some_and(|d| d <= TOL) {
                            let s = (q[0] - w.line[0][0]) * u[0] + (q[1] - w.line[0][1]) * u[1];
                            if s > s0 && s < s1 {
                                cuts.push(s);
                            }
                        }
                    }
                    continue;
                }
                let d = [q0[0] - w.line[0][0], q0[1] - w.line[0][1]];
                let (s, t) = (cross(d, e) / denom, cross(d, u) / denom);
                if s > s0 && s < s1 && (-TOL..=1.0 + TOL).contains(&t) {
                    cuts.push(s);
                }
            }
        }
        cuts.sort_by(f64::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        // The roof's height at each cut, from the plane over the stretch either side of it.
        let mut raw: Vec<(f64, f64)> = Vec::new();
        let mut used: Vec<(String, Status)> = Vec::new();
        let mut fault: Option<String> = None;
        for pair in cuts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let Some(pl) = roof_over(&planes, at((a + b) / 2.0)) else {
                fault = Some(format!("no plane of the roof that can be placed lies over the wall's line from {} ft to {} ft along it{}", trim::feet_said(a), trim::feet_said(b), if left_out.is_empty() { String::new() } else { format!(" ({})", left_out.join("; ")) }));
                break;
            };
            if !used.iter().any(|(p, _)| *p == pl.path) {
                used.push((pl.path.clone(), pl.status));
            }
            for (s, z) in [(a, pl.height_at(at(a))), (b, pl.height_at(at(b)))] {
                match raw.last() {
                    Some(&(ls, lz)) if (ls - s).abs() < 1e-6 && (lz - z).abs() < 1e-6 => {}
                    _ => raw.push((s, z)),
                }
            }
        }
        if let Some(why) = fault {
            out.insert(w.path.clone(), refused(&w.path, &why, &inputs));
            continue;
        }
        // The wall's top where the roof is above it, and the wall's own top where it is not: cut where they cross.
        let mut top: Vec<(f64, f64)> = Vec::new();
        for (k, &(s, z)) in raw.iter().enumerate() {
            if k > 0 {
                let (ps, pz) = raw[k - 1];
                if (pz - wall_top) * (z - wall_top) < 0.0 && s > ps {
                    let x = ps + (s - ps) * (wall_top - pz) / (z - pz);
                    top.push((x, wall_top));
                }
            }
            top.push((s, z.max(wall_top)));
        }
        // No point that lies on the line between its neighbours.
        let mut k = 1;
        while k + 1 < top.len() {
            let (a, b, c) = (top[k - 1], top[k], top[k + 1]);
            let on_line = ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs() < 1e-9 && (b.0 - a.0) * (c.0 - b.0) >= 0.0;
            if on_line {
                top.remove(k);
            } else {
                k += 1;
            }
        }
        let area: f64 = top.windows(2).map(|p| (p[1].0 - p[0].0) * ((p[0].1 + p[1].1) / 2.0 - wall_top)).sum();
        let mut statuses = vec![w.status, w.height_status];
        statuses.extend(rising.iter().map(|o| o.status));
        statuses.extend(used.iter().map(|(_, s)| *s));
        for (p, _) in &used {
            inputs.push(format!("{p}#{K_OUTLINE}"));
            inputs.push(format!("{p}#{K_PITCH}"));
        }
        let mut area = Derived::new(Some(area), "SF", &statuses, FORMULA, &inputs);
        if s0 > TOL || s1 < long - TOL {
            area.note = format!("a rake or high edge lies over {} ft of this wall's {} ft, from {} ft along it: the gable is worked over that part", trim::feet_said(s1 - s0), trim::feet_said(long), trim::feet_said(s0));
        }
        out.insert(w.path.clone(), Gable { path: w.path.clone(), area, top, why: String::new() });
    }
    out
}

/// The wall's top at `s` feet along its line from its first end: read off the top line (`Gable::top`) between the two points
/// either side of it. None where the wall has no top line (no gable) or `s` is outside the stretch the line covers, where the wall
/// stands at its own top. A wall's top at a place along it is BIMCraft's idea (`top_at`, `walls.rs`), which reads the roof a wall is
/// attached to; here it is read off the model's own line, which nothing else works out again (call 204).
pub(crate) fn top_over(top: &[(f64, f64)], s: f64) -> Option<f64> {
    let (first, last) = (top.first()?, top.last()?);
    if s < first.0 - TOL || s > last.0 + TOL {
        return None;
    }
    for pair in top.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if s <= b.0 + TOL {
            return Some(if (b.0 - a.0).abs() < 1e-9 { a.1.max(b.1) } else { a.1 + (b.1 - a.1) * ((s - a.0) / (b.0 - a.0)).clamp(0.0, 1.0) });
        }
    }
    Some(last.1)
}

/// Every wall's gable in the job, in the order of their paths, worked out whether a line names it or not.
pub fn figures(job: &mut Job) -> Vec<Gable> {
    let mut all: Vec<Gable> = gables(&mut job.world, None).into_values().collect();
    all.sort_by_key(|g| path_number(&g.path));
    all
}

/// The walls that rest on a gable's figure: one a line of it names (`gable_area`), or one that is framed (its frame says the
/// gable's is not counted).
pub(crate) fn asked_of(world: &mut World) -> (HashSet<String>, HashSet<String>) {
    let named = trim::naming(world, &[Kind::Wall], &[GABLE_AREA]);
    let mut framed: HashSet<String> = HashSet::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Cost>, Option<&MadeOf>, Option<&Condition>)>();
    for (n, k, cost, made_of, condition) in q.iter(world) {
        if *k == Kind::Wall && crate::finish::framing_of(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str)).is_some() {
            framed.insert(n.path.clone());
        }
    }
    (named, framed)
}

/// What a framed wall's figures say where it has a gable.
pub const FRAME_NOT_COUNTED: &str = "this wall has a gable above its top: the frame of the gable (its studs cut to the rake) is not counted in this figure";
