//! Solving a roof from its footprint.
//!
//! A Roof with a footprint (an outline, an eave height and a rise for each edge) is
//! solved: the skeleton says where the slopes meet, which gives one plane to each sloping
//! edge; the planes give the hips, valleys, ridges, eaves and rakes (`roof::lines_of`).
//! The planes and lines are written into the job as elements under the roof.
//!
//! Solving again after a change keeps what it can: the plane on edge 3 is still the
//! plane on edge 3, with its path, its condition and its cost lines, and a hip between
//! the same two planes is still that hip. What no longer exists is removed and reported,
//! with a warning if it carried cost lines. Elements entered by hand are never touched:
//! the solver owns only what carries `pl::solved`.

use std::collections::BTreeMap;

use crate::check;
use crate::components::*;
use crate::job::Job;
use crate::roof;
use crate::skeleton;
use crate::units::Len;
use crate::value::{Status, Val};

/// What a solve did, for telling the estimator.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub planes: usize,
    pub lines: usize,
    pub added: Vec<String>,
    /// Removed elements, each with whether it carried cost lines or a condition.
    pub removed: Vec<(String, bool)>,
    /// What the planes say about how they meet (see `roof::Found::problems`).
    pub notes: Vec<String>,
}

/// Solved coordinates are written to a millionth of a foot, so the file reads cleanly and
/// a re-solve that changes nothing writes the same bytes.
fn tidy(x: f64) -> f64 {
    (x * 1.0e6).round() / 1.0e6 + 0.0
}

fn name_of_line(kind: Kind, edges: &[usize]) -> String {
    let word = match kind {
        Kind::Eave => "eave",
        Kind::Rake => "rake",
        Kind::Ridge => "ridge",
        Kind::Hip => "hip",
        Kind::Valley => "valley",
        _ => "line",
    };
    format!("{word} {}", edges.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("-"))
}

/// Solve the roof at `roof_path` from its footprint, replacing what an earlier solve made.
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
    let eave_height = fp.eave_height.feet().ok_or_else(|| format!("{roof_path}: the eave height is unknown; the roof cannot be placed"))?;
    let faces = skeleton::roof_faces(&[points.clone()], &speeds).map_err(|why| format!("{roof_path}: {why}"))?;

    // The solved geometry rests on every entered value of the footprint: one pitch
    // changed moves every plane's outline.
    let worst = fp.edges.iter().map(|e| e.status).chain([fp.points.status]).max().unwrap_or(Status::Unknown);
    let source = format!("solved from the footprint of {roof_path}");
    let n = points.len();

    // What an earlier solve left under this roof.
    let mut old_planes: BTreeMap<usize, String> = BTreeMap::new();
    let mut old_lines: BTreeMap<String, String> = BTreeMap::new();
    let children = job.children().get(roof_path).cloned().unwrap_or_default();
    for (name, path) in &children {
        let Some(c) = job.find(path) else { continue };
        let Some(s) = job.world.get::<Solved>(c) else { continue };
        if s.from != roof_path {
            continue;
        }
        match s.edge {
            Some(edge) => {
                old_planes.insert(edge, path.clone());
            }
            None => {
                old_lines.insert(name.clone(), path.clone());
            }
        }
    }

    let mut report = Report::default();
    let carries = |job: &mut Job, path: &str| job.find(path).is_some_and(|c| job.world.get::<Cost>(c).is_some() || job.world.get::<Condition>(c).is_some());

    // Planes: one to each sloping edge, kept by edge number.
    let mut plane_of_edge: BTreeMap<usize, String> = BTreeMap::new();
    for (i, loops) in faces.iter().enumerate() {
        let edge = i + 1;
        if loops.is_empty() {
            continue;
        }
        if loops.len() > 1 {
            return Err(format!("{roof_path}: the slope from edge {edge} closes round a hole; roofs with courtyards are not solved yet"));
        }
        let outline: Vec<P2> = loops[0].iter().map(|(p, _)| [tidy(p[0]), tidy(p[1])]).collect();
        if outline.len() < 3 {
            continue;
        }
        let path = match old_planes.remove(&edge) {
            Some(path) => path,
            None => {
                let path = job.add(roof_path, Kind::RoofPlane, &format!("plane {edge}"))?;
                report.added.push(path.clone());
                path
            }
        };
        let (a, b) = (points[i], points[(i + 1) % n]);
        let pitch = match fp.edges[i].value {
            Some(Rise::Pitch(p)) => p,
            _ => continue,
        };
        let new_outline = Outline {
            points: Val { value: Some(outline), unit: U_LENGTH.into(), status: worst, source: source.clone() },
            eave: Val { value: Some([[tidy(a[0]), tidy(a[1])], [tidy(b[0]), tidy(b[1])]]), unit: U_LENGTH.into(), status: fp.points.status, source: source.clone() },
            eave_height: Val { value: Some(Len::One(eave_height)), unit: U_LENGTH.into(), status: fp.eave_height.status, source: fp.eave_height.source.clone() },
        };
        let new_pitch = Pitch(Val { value: Some(pitch), unit: U_PITCH.into(), status: fp.edges[i].status, source: fp.edges[i].source.clone() });
        check::outline(&path, Kind::RoofPlane, &new_outline).map_err(|why| format!("the solver made a plane the checks refuse ({why}); this is a fault in the program, not in the footprint"))?;
        job.put(&path, new_outline)?;
        job.put(&path, new_pitch)?;
        job.put(&path, Solved { from: roof_path.to_string(), edge: Some(edge) })?;
        plane_of_edge.insert(edge, path);
    }
    for (_, path) in old_planes {
        let had = carries(job, &path);
        job.remove(&path)?;
        report.removed.push((path, had));
    }

    // Lines: read off this roof's planes.
    let (all, _) = roof::solid_planes(job);
    let mine: Vec<roof::SolidPlane> = all.into_iter().filter(|p| plane_of_edge.values().any(|path| *path == p.path)).collect();
    let found = roof::lines_of(&mine);
    report.notes = found.problems.clone();
    let edge_of = |path: &String| plane_of_edge.iter().find(|(_, p)| *p == path).map(|(e, _)| *e).unwrap_or(0);
    let mut names_used: BTreeMap<String, usize> = BTreeMap::new();
    for line in &found.lines {
        let mut edges: Vec<usize> = line.planes.iter().map(edge_of).collect();
        edges.sort();
        let base = name_of_line(line.kind, &edges);
        // Two lines of one kind between the same planes are told apart by a letter.
        let count = names_used.entry(base.clone()).or_insert(0);
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
                let path = job.add(roof_path, line.kind, &name)?;
                report.added.push(path.clone());
                path
            }
        };
        let mut planes: Vec<String> = line.planes.clone();
        planes.sort();
        let run = Run { ends: Val { value: Some(line.ends), unit: U_LENGTH.into(), status: line.status, source: source.clone() }, planes };
        check::run(&path, line.kind, &run)?;
        job.put(&path, run)?;
        job.put(&path, Solved { from: roof_path.to_string(), edge: None })?;
    }
    for (_, path) in old_lines {
        let had = carries(job, &path);
        job.remove(&path)?;
        report.removed.push((path, had));
    }
    report.planes = plane_of_edge.len();
    report.lines = found.lines.len();
    job.derive();
    Ok(report)
}

/// Make a new roof under the job from a footprint and solve it.
pub fn new_roof(job: &mut Job, name: &str, footprint: Footprint) -> Result<(String, Report), String> {
    let root = "job-0001".to_string();
    let parent = job.find(&root).map(|_| root).ok_or("the job has no job element")?;
    let path = job.add(&parent, Kind::Roof, name)?;
    if let Err(why) = check::footprint(&path, Kind::Roof, &footprint) {
        job.remove(&path)?;
        return Err(why);
    }
    job.put(&path, footprint)?;
    match solve(job, &path) {
        Ok(report) => Ok((path, report)),
        Err(why) => {
            // Leave nothing half-made behind.
            let children: Vec<String> = job.children().get(&path).map(|c| c.values().cloned().collect()).unwrap_or_default();
            for child in children {
                job.remove(&child)?;
            }
            job.remove(&path)?;
            Err(why)
        }
    }
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
    EaveHeight(Val<Len>),
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
            let len = crate::geom::dist(a, b);
            // Outward is to the right of the edge when the outline runs anticlockwise.
            let anticlockwise = (0..n).map(|k| pts[k][0] * pts[(k + 1) % n][1] - pts[(k + 1) % n][0] * pts[k][1]).sum::<f64>() > 0.0;
            let sign = if anticlockwise { 1.0 } else { -1.0 };
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
            new.points = Val { value: Some(moved), unit: new.points.unit.clone(), status: new.points.status.max(status), source };
        }
    }
    check::footprint(roof_path, kind, &new)?;
    job.put(roof_path, new)?;
    match solve(job, roof_path) {
        Ok(report) => Ok(report),
        Err(why) => {
            // Put the footprint back and solve it as it was, so nothing is left half-changed.
            job.put(roof_path, old)?;
            let _ = solve(job, roof_path);
            Err(why)
        }
    }
}
