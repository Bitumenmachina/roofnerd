//! Solving a roof from its footprint.
//!
//! A Roof with a footprint (an outline, an eave height and a rise for each edge) is
//! solved: the skeleton says where the slopes meet, which gives one plane to each sloping
//! edge; the planes give the hips, valleys, ridges, eaves and rakes (`roof::lines_of`).
//! The planes and lines are written into the job as elements under the roof.
//!
//! Solving again after a change keeps what it can, and it goes by where things are, not
//! by how they are numbered: a plane stays with the edge its eave lies along, so a plane's
//! condition and cost lines stay on the slope they were entered for even if a point is
//! added to the footprint and every edge after it takes a new number. What no longer
//! exists is removed and reported, with a warning if it carried cost lines. Elements
//! entered by hand are never touched: the solver owns only what carries `pl::solved`.
//!
//! What is solved is never trusted from the file. Every roof with a footprint is solved
//! again on load, and if its footprint cannot be solved (an edge with no pitch), the
//! planes and lines of an earlier solve lose their geometry: they are not left standing
//! as Known.

use std::collections::BTreeMap;

use crate::check::{self, FIT_TOLERANCE_FT};
use crate::components::*;
use crate::file;
use crate::geom;
use crate::job::Job;
use crate::roof;
use crate::skeleton;
use crate::value::{Status, Val};

/// What a solve did, for telling the estimator.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub planes: usize,
    pub lines: usize,
    pub added: Vec<String>,
    /// Removed elements, each with whether it carried cost lines or a condition.
    pub removed: Vec<(String, bool)>,
    /// Planes that kept their path but are now on an edge of another number.
    pub renumbered: Vec<String>,
    /// What the planes say about how they meet (see `roof::Found::problems`).
    pub notes: Vec<String>,
}

/// Solved coordinates are written to a millionth of a foot, so the file reads cleanly and
/// a re-solve that changes nothing writes the same bytes.
fn tidy(x: f64) -> f64 {
    (x * 1.0e6).round() / 1.0e6 + 0.0
}

fn word(kind: Kind) -> &'static str {
    match kind {
        Kind::Eave => "eave",
        Kind::Rake => "rake",
        Kind::Ridge => "ridge",
        Kind::Hip => "hip",
        Kind::Valley => "valley",
        Kind::PitchBreak => "break",
        _ => "line",
    }
}

/// Do two segments lie along one line and share a stretch of it?
fn overlap_along(a: [P2; 2], b: [P2; 2]) -> bool {
    let tol = FIT_TOLERANCE_FT;
    let on = |p: P2| geom::point_to_line(p, a[0], a[1]).is_some_and(|d| d <= tol);
    if !(on(b[0]) && on(b[1])) {
        return false;
    }
    let len = geom::dist(a[0], a[1]);
    let along = |p: P2| ((p[0] - a[0][0]) * (a[1][0] - a[0][0]) + (p[1] - a[0][1]) * (a[1][1] - a[0][1])) / len;
    let (lo, hi) = (along(b[0]).min(along(b[1])).max(0.0), along(b[0]).max(along(b[1])).min(len));
    hi - lo > tol
}

/// Do two segments run the same way (parallel, same direction)?
fn same_way(a: [P2; 2], b: [P2; 2]) -> bool {
    let (ax, ay, bx, by) = (a[1][0] - a[0][0], a[1][1] - a[0][1], b[1][0] - b[0][0], b[1][1] - b[0][1]);
    let (la, lb) = ((ax * ax + ay * ay).sqrt(), (bx * bx + by * by).sqrt());
    la > 0.0 && lb > 0.0 && ((ax * by - ay * bx) / (la * lb)).abs() < 1e-6 && ax * bx + ay * by > 0.0
}

/// The elements an earlier solve left under a roof: (path, name, edge if a plane).
fn solved_under(job: &mut Job, roof_path: &str) -> Vec<(String, String, Option<usize>)> {
    let children = job.children().get(roof_path).cloned().unwrap_or_default();
    let mut out = Vec::new();
    for (name, path) in children {
        let Some(c) = job.find(&path) else { continue };
        if let Some(s) = job.world.get::<Solved>(c) {
            if s.from == roof_path {
                out.push((path, name, s.edge));
            }
        }
    }
    out
}

fn rename(job: &mut Job, path: &str, name: &str) {
    if let Some(e) = job.find(path) {
        if let Some(mut node) = job.world.get_mut::<Node>(e) {
            node.name = name.to_string();
        }
    }
}

/// Solve the roof at `roof_path` from its footprint, replacing what an earlier solve made.
/// On an error the job may be part-changed; `edit`, `new_roof` and loading put that right.
pub fn solve(job: &mut Job, roof_path: &str) -> Result<Report, String> {
    let e = job.find(roof_path).ok_or_else(|| format!("no element {roof_path}"))?;
    let fp = job.world.get::<Footprint>(e).ok_or_else(|| format!("{roof_path} has no footprint to solve from"))?.clone();
    let kind = *job.world.get::<Kind>(e).ok_or_else(|| format!("{roof_path} has no kind"))?;
    check::footprint(roof_path, kind, &fp)?;
    let points = fp.points_ft().ok_or_else(|| format!("{roof_path}: the footprint's points are unknown; there is nothing to solve"))?;
    let mut speeds = Vec::new();
    for (i, edge) in fp.edges.iter().enumerate() {
        speeds.push(match edge.value {
            Some(Rise::Pitch(p)) => 12.0 / p,
            Some(Rise::Gable) => 0.0,
            None => return Err(format!("{roof_path}: edge {} has no pitch entered; a roof is solved when every edge has a pitch or is a gable", i + 1)),
        });
    }
    if fp.eave_height.feet().is_none() {
        return Err(format!("{roof_path}: the eave height is unknown; the roof cannot be placed"));
    }
    let faces = skeleton::roof_faces(std::slice::from_ref(&points), &speeds).map_err(|why| format!("{roof_path}: {why}"))?;
    for (i, loops) in faces.iter().enumerate() {
        if loops.len() > 1 {
            return Err(format!("{roof_path}: the slope from edge {} closes round a hole; roofs with courtyards are not solved yet", i + 1));
        }
    }

    // The solved geometry rests on every entered value of the footprint: one pitch
    // changed moves every plane's outline.
    let worst = fp.edges.iter().map(|e| e.status).chain([fp.points.status]).max().unwrap_or(Status::Unknown);
    let source = format!("solved from the footprint of {roof_path}");
    let n = points.len();
    let edge_line = |i: usize| [points[i], points[(i + 1) % n]];

    // What an earlier solve left: planes with the eave each was on, and lines by name.
    let mut old_planes: Vec<(String, usize, Option<[P2; 2]>)> = Vec::new();
    let mut old_lines: BTreeMap<String, String> = BTreeMap::new();
    for (path, name, edge) in solved_under(job, roof_path) {
        match edge {
            Some(edge) => {
                let eave = job.find(&path).and_then(|c| job.world.get::<Outline>(c)).and_then(|o| o.eave_ft());
                old_planes.push((path, edge, eave));
            }
            None => {
                old_lines.insert(name, path);
            }
        }
    }

    let mut report = Report::default();
    let carries = |job: &mut Job, path: &str| job.find(path).is_some_and(|c| job.world.get::<Cost>(c).is_some() || job.world.get::<Condition>(c).is_some());

    // Which old plane goes with which edge. By place first: the old plane whose eave
    // lies along this edge. Then by number, for an edge that was moved square to itself
    // and so lies along nothing it did before. What is left over is gone.
    let sloping: Vec<usize> = (0..n).filter(|i| faces[*i].first().is_some_and(|f| f.len() >= 3) && matches!(fp.edges[*i].value, Some(Rise::Pitch(_)))).collect();
    let mut keep: BTreeMap<usize, String> = BTreeMap::new();
    for &i in &sloping {
        if let Some(at) = old_planes.iter().position(|(_, _, eave)| eave.is_some_and(|ev| overlap_along(edge_line(i), ev) && same_way(edge_line(i), ev))) {
            keep.insert(i, old_planes.remove(at).0);
        }
    }
    for &i in &sloping {
        if keep.contains_key(&i) {
            continue;
        }
        if let Some(at) = old_planes.iter().position(|(_, edge, eave)| *edge == i + 1 && eave.is_none_or(|ev| same_way(edge_line(i), ev))) {
            keep.insert(i, old_planes.remove(at).0);
        }
    }
    for (path, _, _) in old_planes {
        let had = carries(job, &path);
        job.remove(&path)?;
        report.removed.push((path, had));
    }
    // Kept planes take their new names in two steps, so two that swap do not collide.
    for (i, path) in &keep {
        rename(job, path, &format!("\u{0}plane {}", i + 1));
    }

    // Planes: one to each sloping edge.
    let mut plane_of_edge: BTreeMap<usize, String> = BTreeMap::new();
    for &i in &sloping {
        let edge = i + 1;
        let outline: Vec<P2> = faces[i][0].iter().map(|(p, _)| [tidy(p[0]), tidy(p[1])]).collect();
        let name = format!("plane {edge}");
        let path = match keep.get(&i) {
            Some(path) => {
                let was = job.find(path).and_then(|c| job.world.get::<Solved>(c)).and_then(|s| s.edge);
                if was != Some(edge) {
                    report.renumbered.push(format!("{path} is on edge {edge} now (it was on edge {}); it kept its place, its assembly and its cost lines", was.unwrap_or(0)));
                }
                rename(job, path, &name);
                path.clone()
            }
            None => {
                let path = job.add(roof_path, Kind::RoofPlane, &name).map_err(|why| format!("{why}. The solver did not make that element; rename it or take it out so the solved plane can take the name"))?;
                report.added.push(path.clone());
                path
            }
        };
        let (a, b) = (points[i], points[(i + 1) % n]);
        let Some(Rise::Pitch(pitch)) = fp.edges[i].value else { continue };
        let new_outline = Outline {
            points: Val { value: Some(outline), unit: U_LENGTH.into(), status: worst, source: source.clone() },
            eave: Val { value: Some([[tidy(a[0]), tidy(a[1])], [tidy(b[0]), tidy(b[1])]]), unit: U_LENGTH.into(), status: fp.points.status, source: source.clone() },
            // As entered on the footprint: 126 in stays 126 in.
            eave_height: fp.eave_height.clone(),
        };
        let new_pitch = Pitch(Val { value: Some(pitch), unit: U_PITCH.into(), status: fp.edges[i].status, source: fp.edges[i].source.clone() });
        check::outline(&path, Kind::RoofPlane, &new_outline).map_err(|why| {
            if why.contains("cross or touch") || why.contains("doubles back") {
                format!("{roof_path}: the slope from edge {edge} comes back to touch itself, so its plane would be two pieces joined at a point; that is not solved yet. It comes of gables that pinch a slope: let one of them slope")
            } else {
                format!("{roof_path}: the roof these pitches make has a plane the checks refuse ({why})")
            }
        })?;
        job.put(&path, new_outline)?;
        job.put(&path, new_pitch)?;
        job.put(&path, Solved { from: roof_path.to_string(), edge: Some(edge) })?;
        plane_of_edge.insert(edge, path);
    }

    // Lines: read off this roof's planes.
    let (all, _) = roof::solid_planes(job);
    let mine: Vec<roof::SolidPlane> = all.into_iter().filter(|p| plane_of_edge.values().any(|path| *path == p.path)).collect();
    let found = roof::lines_of(&mine);
    report.notes = found.problems.clone();
    let edge_of = |path: &String| plane_of_edge.iter().find(|(_, p)| *p == path).map(|(e, _)| *e).unwrap_or(0);
    // A line is known by its kind and the edges of its planes. A line on one plane
    // (a rake, a shed's top) also says which footprint edge it stands over, so the two
    // rakes of one plane are told apart by where they are and not by which was found first.
    let mut named: Vec<(String, &roof::FoundLine)> = Vec::new();
    for line in &found.lines {
        let mut edges: Vec<usize> = line.planes.iter().map(edge_of).collect();
        edges.sort();
        let mut name = format!("{} {}", word(line.kind), edges.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("-"));
        if line.planes.len() == 1 && line.kind != Kind::Eave {
            let over = (0..n).find(|k| *k + 1 != edges[0] && line.ends.iter().all(|p| geom::point_to_segment(*p, points[*k], points[(*k + 1) % n]) <= FIT_TOLERANCE_FT));
            if let Some(k) = over {
                name = format!("{name} at {}", k + 1);
            }
        }
        named.push((name, line));
    }
    // Lines that still share a name are told apart by a letter, in the order of where
    // they lie (west to east, then south to north), which does not depend on the solve.
    named.sort_by(|x, y| {
        let mid = |l: &roof::FoundLine| [(l.ends[0][0] + l.ends[1][0]) / 2.0, (l.ends[0][1] + l.ends[1][1]) / 2.0];
        x.0.cmp(&y.0).then(mid(x.1)[0].total_cmp(&mid(y.1)[0])).then(mid(x.1)[1].total_cmp(&mid(y.1)[1]))
    });
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (base, line) in named {
        let count = seen.entry(base.clone()).or_insert(0);
        let name = if *count == 0 { base.clone() } else { format!("{base} {}", (b'a' + (*count as u8).min(25)) as char) };
        *count += 1;
        let path = match old_lines.remove(&name) {
            Some(path) if job.kind(&path) == Some(line.kind) => path,
            other => {
                if let Some(stale) = other {
                    let had = carries(job, &stale);
                    job.remove(&stale)?;
                    report.removed.push((stale, had));
                }
                let path = job.add(roof_path, line.kind, &name).map_err(|why| format!("{why}. The solver did not make that element; rename it or take it out so the solved line can take the name"))?;
                report.added.push(path.clone());
                path
            }
        };
        let mut planes: Vec<String> = line.planes.clone();
        planes.sort();
        let ends = [[tidy(line.ends[0][0]), tidy(line.ends[0][1])], [tidy(line.ends[1][0]), tidy(line.ends[1][1])]];
        let run = Run { ends: Val { value: Some(ends), unit: U_LENGTH.into(), status: line.status, source: source.clone() }, planes };
        check::run(&path, line.kind, &run)?;
        job.put(&path, run)?;
        job.put(&path, Solved { from: roof_path.to_string(), edge: None })?;
    }
    for (_, path) in old_lines {
        let had = carries(job, &path);
        job.remove(&path)?;
        report.removed.push((path, had));
    }

    // A plane entered by hand under a solved roof is no part of the solve.
    let by_hand: Vec<String> = job.children().get(roof_path).cloned().unwrap_or_default().into_values().filter(|p| job.kind(p).is_some_and(|k| k.is_area()) && !plane_of_edge.values().any(|q| q == p)).collect();
    for path in by_hand {
        report.notes.push(format!("{path} was entered by hand under a roof that is solved from its footprint; it is no part of the solve and may lie over the solved planes"));
    }
    report.planes = plane_of_edge.len();
    report.lines = found.lines.len();
    job.put(roof_path, SolveState { not_solved: String::new(), notes: report.notes.clone() })?;
    job.derive();
    Ok(report)
}

/// The roof's footprint cannot be solved: take the geometry off what an earlier solve
/// made, so it does not stand as if it were still true, and say why on the roof.
pub fn unsolve(job: &mut Job, roof_path: &str, why: &str) {
    let source = format!("{roof_path} cannot be solved: {why}");
    let fp = job.find(roof_path).and_then(|e| job.world.get::<Footprint>(e).cloned());
    for (path, _, edge) in solved_under(job, roof_path) {
        let Some(c) = job.find(&path) else { continue };
        if let Some(mut o) = job.world.get_mut::<Outline>(c) {
            o.points = Val { value: None, unit: U_LENGTH.into(), status: Status::Unknown, source: source.clone() };
            o.eave = Val { value: None, unit: U_LENGTH.into(), status: Status::Unknown, source: source.clone() };
            if let Some(fp) = &fp {
                o.eave_height = fp.eave_height.clone();
            }
        }
        // The plane's pitch is its edge's, whatever that now is.
        if let (Some(edge), Some(fp)) = (edge, &fp) {
            let rise = fp.edges.get(edge - 1);
            let pitch = match rise.and_then(|r| r.value) {
                Some(Rise::Pitch(p)) => rise.map(|r| Val { value: Some(p), unit: U_PITCH.into(), status: r.status, source: r.source.clone() }),
                _ => None,
            };
            let pitch = pitch.unwrap_or_else(|| Val { value: None, unit: U_PITCH.into(), status: Status::Unknown, source: rise.map(|r| r.source.clone()).filter(|s| !s.trim().is_empty()).unwrap_or_else(|| source.clone()) });
            if let Some(mut p) = job.world.get_mut::<Pitch>(c) {
                *p = Pitch(pitch);
            }
        }
        if let Some(mut r) = job.world.get_mut::<Run>(c) {
            r.ends = Val { value: None, unit: U_LENGTH.into(), status: Status::Unknown, source: source.clone() };
        }
    }
    let _ = job.put(roof_path, SolveState { not_solved: why.to_string(), notes: Vec::new() });
    job.derive();
}

/// Solve every roof that has a footprint, as loading does. A roof that cannot be solved
/// has its earlier planes and lines left without geometry.
pub fn solve_all(job: &mut Job) {
    for roof_path in roofs(job) {
        if let Err(why) = solve(job, &roof_path) {
            // The reason reads better without the roof's own path in front of it.
            let why = why.strip_prefix(&format!("{roof_path}: ")).unwrap_or(&why).to_string();
            unsolve(job, &roof_path, &why);
        }
    }
}

/// Put the job back exactly as it was saved in `before`.
fn restore(job: &mut Job, before: &str) -> Result<(), String> {
    let (old, _) = file::load(before).map_err(|why| format!("the job could not be put back as it was ({why})"))?;
    *job = old;
    Ok(())
}

/// Make a new roof under the job from a footprint and solve it. If it cannot be solved
/// the job is left exactly as it was.
pub fn new_roof(job: &mut Job, name: &str, footprint: Footprint) -> Result<(String, Report), String> {
    let before = file::save(job);
    let made = (|| {
        let path = job.add("job-0001", Kind::Roof, name)?;
        check::footprint(&path, Kind::Roof, &footprint)?;
        job.put(&path, footprint)?;
        let report = solve(job, &path)?;
        Ok((path, report))
    })();
    if made.is_err() {
        restore(job, &before)?;
    }
    made
}

/// Every roof in the job that has a footprint, in job order.
pub fn roofs(job: &mut Job) -> Vec<String> {
    job.order.iter().filter_map(|e| job.world.get::<Footprint>(*e).and(job.world.get::<Node>(*e)).map(|n| n.path.clone())).collect()
}

/// One change to a roof's footprint. Each is followed by a solve.
#[derive(Clone, Debug)]
pub enum Change {
    /// How edge `edge` (counted from 1) rises: a pitch, or a gable.
    Rise { edge: usize, rise: Val<Rise> },
    /// Move edge `edge` square to itself by `by_ft` feet: outward when positive. Its two
    /// corners slide along the edges either side of it, which get longer or shorter.
    MoveEdge { edge: usize, by_ft: f64, status: Status, source: String },
    /// The height of the eaves.
    EaveHeight(Val<crate::units::Len>),
}

/// Change a roof's footprint and solve it again. If the changed footprint is refused, or
/// cannot be solved, the job is left exactly as it was.
pub fn edit(job: &mut Job, roof_path: &str, change: Change) -> Result<Report, String> {
    let e = job.find(roof_path).ok_or_else(|| format!("no element {roof_path}"))?;
    let kind = *job.world.get::<Kind>(e).ok_or_else(|| format!("{roof_path} has no kind"))?;
    let old = job.world.get::<Footprint>(e).ok_or_else(|| format!("{roof_path} has no footprint; only a roof solved from a footprint can be changed this way"))?.clone();
    let mut new = old.clone();
    let n = new.edges.len();
    let edge_index = |edge: usize| if edge >= 1 && edge <= n { Ok(edge - 1) } else { Err(format!("{roof_path} has edges 1 to {n}; there is no edge {edge}")) };
    match change {
        Change::Rise { edge, rise } => {
            let i = edge_index(edge)?;
            new.edges[i] = rise;
        }
        Change::EaveHeight(h) => new.eave_height = h,
        Change::MoveEdge { edge, by_ft, status, source } => {
            let i = edge_index(edge)?;
            let d = crate::units::coord_divisor(&new.points.unit).map_err(|why| format!("{roof_path} footprint points: {why}"))?;
            let pts = new.points.value.clone().ok_or_else(|| format!("{roof_path}: the footprint's points are unknown"))?;
            let by = by_ft * d;
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            let (before, after) = (pts[(i + n - 1) % n], pts[(i + 2) % n]);
            let len = geom::dist(a, b);
            // Outward is to the right of the edge when the outline runs anticlockwise.
            let turn = |p: &[P2]| (0..p.len()).map(|k| p[k][0] * p[(k + 1) % p.len()][1] - p[(k + 1) % p.len()][0] * p[k][1]).sum::<f64>();
            let sign = if turn(&pts) > 0.0 { 1.0 } else { -1.0 };
            let out = [sign * (b[1] - a[1]) / len, -sign * (b[0] - a[0]) / len];
            let (a2, b2) = ([a[0] + out[0] * by, a[1] + out[1] * by], [b[0] + out[0] * by, b[1] + out[1] * by]);
            // Each corner goes to where the moved edge's line meets its neighbour's line.
            let meet = |p: P2, q: P2, r: P2, s: P2| -> Option<P2> {
                let (dx1, dy1, dx2, dy2) = (q[0] - p[0], q[1] - p[1], s[0] - r[0], s[1] - r[1]);
                let den = dx1 * dy2 - dy1 * dx2;
                if den.abs() < 1e-12 {
                    return None;
                }
                let t = ((r[0] - p[0]) * dy2 - (r[1] - p[1]) * dx2) / den;
                Some([tidy(p[0] + t * dx1), tidy(p[1] + t * dy1)])
            };
            let parallel = || format!("{roof_path}: edge {edge} runs in line with the edge beside it, so moving it square to itself leaves its corner nowhere to go");
            let mut moved = pts.clone();
            moved[i] = meet(a2, b2, before, a).ok_or_else(parallel)?;
            moved[(i + 1) % n] = meet(a2, b2, b, after).ok_or_else(parallel)?;
            // The footprint has to stay the same shape the same way round: the moved edge
            // and the two beside it still pointing the way they did. Moved past the far
            // wall, the roof would be turned inside out and still solve.
            let keeps = |was: [P2; 2], now: [P2; 2]| (was[1][0] - was[0][0]) * (now[1][0] - now[0][0]) + (was[1][1] - was[0][1]) * (now[1][1] - now[0][1]) > 0.0;
            let (ma, mb) = (moved[i], moved[(i + 1) % n]);
            if !(keeps([a, b], [ma, mb]) && keeps([before, a], [before, ma]) && keeps([b, after], [mb, after]) && turn(&moved) * turn(&pts) > 0.0) {
                return Err(format!("{roof_path}: moving edge {edge} by that much takes it past the other side of the footprint, or closes up the edge beside it; the roof would be turned inside out"));
            }
            let source = format!("{}; edge {edge} moved ({source})", new.points.source);
            new.points = Val { value: Some(moved), unit: new.points.unit.clone(), status: new.points.status.max(status), source };
        }
    }
    check::footprint(roof_path, kind, &new)?;
    let before = file::save(job);
    job.put(roof_path, new)?;
    match solve(job, roof_path) {
        Ok(report) => Ok(report),
        Err(why) => {
            restore(job, &before)?;
            Err(why)
        }
    }
}
