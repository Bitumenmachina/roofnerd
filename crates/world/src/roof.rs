//! The lines a set of roof planes implies.
//!
//! In D1 an estimator enters the planes and then enters every eave, hip and ridge again as
//! a line. The planes already say where those lines are: a line is where two planes meet,
//! or where one plane stops. This module reads them off the planes:
//!
//! * an edge two planes share is a **ridge** if it is level and both planes fall away from
//!   it, a **hip** if it climbs and both fall away, a **valley** if both fall toward it;
//! * an edge only one plane has is an **eave** if it lies on that plane's eave line, a
//!   **rake** if it climbs, and a level high edge is a **ridge** on one plane (a shed's
//!   top). Whether that top edge is a ridge or a headwall depends on whether a wall
//!   stands there, which the planes cannot say.
//!
//! It is what the roof solver (D2) will hand its planes to. Until then it checks entered
//! linework against the planes, and it is the first check of planes against each other:
//! two planes that share an edge and disagree about its height are reported.
//!
//! Everything here is in feet. Nothing here changes the job.

use crate::check::{self, FIT_TOLERANCE_FT};
use crate::components::*;
use crate::geom;
use crate::job::Job;
use crate::value::Status;

/// A plane with everything needed to place it: outline, the outline edge its eave lies
/// along, pitch and eave height, all present and in feet.
#[derive(Clone, Debug)]
pub struct SolidPlane {
    pub path: String,
    pub points: Vec<P2>,
    pub eave: [P2; 2],
    pub pitch: f64,
    pub eave_height: f64,
    /// The worst status among its entered values.
    pub status: Status,
}

impl SolidPlane {
    pub fn height_at(&self, p: P2) -> f64 {
        let off = geom::point_to_line(p, self.eave[0], self.eave[1]).unwrap_or(0.0);
        self.eave_height + off * self.pitch / 12.0
    }

    /// The unit vector on the plan that points straight up the slope.
    fn uphill(&self) -> P2 {
        let [a, b] = self.eave;
        let len = geom::dist(a, b);
        let left = [-(b[1] - a[1]) / len, (b[0] - a[0]) / len];
        // The outline lies on one side of its eave edge; uphill points to that side.
        let side: f64 = self.points.iter().filter_map(|p| geom::signed_offset(*p, a, b)).sum();
        if side >= 0.0 {
            left
        } else {
            [-left[0], -left[1]]
        }
    }

    /// The unit vector square to edge `i`, pointing into the plane.
    fn inward(&self, i: usize) -> P2 {
        let n = self.points.len();
        let (p, q) = (self.points[i], self.points[(i + 1) % n]);
        let len = geom::dist(p, q);
        let left = [-(q[1] - p[1]) / len, (q[0] - p[0]) / len];
        // Traced anticlockwise, the inside is on the left of every edge.
        if signed_area(&self.points) >= 0.0 {
            left
        } else {
            [-left[0], -left[1]]
        }
    }

    /// Does the plane fall away from edge `i` (the edge is its high side there), fall
    /// toward it, or neither (a flat plane, or an edge that runs straight up the slope)?
    fn falls(&self, i: usize) -> Fall {
        let (u, n) = (self.uphill(), self.inward(i));
        let dot = u[0] * n[0] + u[1] * n[1];
        if self.pitch == 0.0 || dot.abs() < 1e-9 {
            Fall::Neither
        } else if dot < 0.0 {
            Fall::Away
        } else {
            Fall::Toward
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fall {
    Away,
    Toward,
    Neither,
}

fn signed_area(pts: &[P2]) -> f64 {
    let n = pts.len();
    (0..n).map(|i| pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1]).sum::<f64>() / 2.0
}

/// A line the planes imply.
#[derive(Clone, Debug, PartialEq)]
pub struct FoundLine {
    pub kind: Kind,
    pub ends: [P2; 2],
    /// The planes it lies on, in job order.
    pub planes: Vec<String>,
    pub status: Status,
}

#[derive(Clone, Debug, Default)]
pub struct Found {
    pub lines: Vec<FoundLine>,
    /// What does not fit, in sentences: planes that disagree about a shared edge, an edge
    /// three planes claim, a fold this program has no kind of line for.
    pub problems: Vec<String>,
    /// Shared edges that gave no line because of a problem above.
    pub disputed: Vec<[P2; 2]>,
}

/// The planes in the job that can be placed, in job order, and the ones that cannot.
pub fn solid_planes(job: &mut Job) -> (Vec<SolidPlane>, Vec<String>) {
    let (mut planes, mut left_out) = (Vec::new(), Vec::new());
    for &e in &job.order {
        let w = &job.world;
        let (Some(node), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        if !kind.is_area() {
            continue;
        }
        let path = node.path.clone();
        let (Some(o), Some(p)) = (w.get::<Outline>(e), w.get::<Pitch>(e)) else {
            left_out.push(format!("{path}: no outline or no pitch entered"));
            continue;
        };
        let (Some(points), Some(eave_line), Some(eave_height), Some(pitch)) = (o.points_ft(), o.eave_ft(), o.eave_height.feet(), p.0.value) else {
            left_out.push(format!("{path}: its outline, eave line, eave height or pitch is unknown"));
            continue;
        };
        let Ok(eave) = check::eave_edge(&points, eave_line) else {
            left_out.push(format!("{path}: its eave line does not fit its outline"));
            continue;
        };
        let status = [o.points.status, o.eave.status, o.eave_height.status, p.0.status].into_iter().max().unwrap_or(Status::Unknown);
        planes.push(SolidPlane { path, points, eave, pitch, eave_height, status });
    }
    (planes, left_out)
}

/// A stretch of one plane's edge, by distance along it, and the other plane on it if any.
struct Piece {
    from: f64,
    to: f64,
    other: Option<usize>,
}

/// The lines these planes imply.
pub fn lines_of(planes: &[SolidPlane]) -> Found {
    let tol = FIT_TOLERANCE_FT;
    let mut found = Found::default();
    // A line's end that falls on a traced corner is that corner, to the last digit, not a
    // point worked out along an edge that lands a hair off it.
    let corners: Vec<P2> = planes.iter().flat_map(|p| p.points.iter().copied()).collect();
    let snap = |x: P2| corners.iter().copied().find(|c| geom::dist(*c, x) <= tol).unwrap_or(x);
    for (ai, a) in planes.iter().enumerate() {
        let n = a.points.len();
        for i in 0..n {
            let (p, q) = (a.points[i], a.points[(i + 1) % n]);
            let len = geom::dist(p, q);
            let along = |x: P2| ((x[0] - p[0]) * (q[0] - p[0]) + (x[1] - p[1]) * (q[1] - p[1])) / len;
            let at = |t: f64| [p[0] + (q[0] - p[0]) * t / len, p[1] + (q[1] - p[1]) * t / len];

            // Every stretch of this edge that another plane's edge lies along.
            let mut shared: Vec<(f64, f64, usize, usize)> = Vec::new();
            for (bi, b) in planes.iter().enumerate() {
                if bi == ai {
                    continue;
                }
                let m = b.points.len();
                for j in 0..m {
                    let (r, s) = (b.points[j], b.points[(j + 1) % m]);
                    let off = |x: P2| geom::point_to_line(x, p, q).unwrap_or(f64::MAX);
                    if off(r) > tol || off(s) > tol {
                        continue;
                    }
                    let (lo, hi) = (along(r).min(along(s)).max(0.0), along(r).max(along(s)).min(len));
                    if hi - lo > tol {
                        shared.push((lo, hi, bi, j));
                    }
                }
            }
            shared.sort_by(|x, y| x.0.total_cmp(&y.0));

            // Walk the edge: shared stretches, and the gaps between them that only this
            // plane has.
            let mut pieces: Vec<(Piece, Option<usize>)> = Vec::new();
            let mut cursor = 0.0;
            for (lo, hi, bi, j) in &shared {
                if *lo < cursor - tol {
                    found.problems.push(format!("more than two planes meet along one line near ({:.2}, {:.2}): {} and {} and another; a line has two sides", at(*lo)[0], at(*lo)[1], a.path, planes[*bi].path));
                    continue;
                }
                if *lo - cursor > tol {
                    pieces.push((Piece { from: cursor, to: *lo, other: None }, None));
                }
                pieces.push((Piece { from: *lo, to: *hi, other: Some(*bi) }, Some(*j)));
                cursor = *hi;
            }
            if len - cursor > tol {
                pieces.push((Piece { from: cursor, to: len, other: None }, None));
            }

            for (piece, other_edge) in pieces {
                let ends = [snap(at(piece.from)), snap(at(piece.to))];
                let (za, zb) = (a.height_at(ends[0]), a.height_at(ends[1]));
                let level = (za - zb).abs() <= tol;
                match (piece.other, other_edge) {
                    (Some(bi), Some(j)) => {
                        // Each shared stretch is seen from both planes; report it once.
                        if bi < ai {
                            continue;
                        }
                        let b = &planes[bi];
                        let (zb0, zb1) = (b.height_at(ends[0]), b.height_at(ends[1]));
                        if (za - zb0).abs() > tol || (zb - zb1).abs() > tol {
                            found.problems.push(format!(
                                "{} and {} share an edge from ({:.2}, {:.2}) to ({:.2}, {:.2}) but disagree about its height: {} puts its ends at {:.2} ft and {:.2} ft, {} at {:.2} ft and {:.2} ft",
                                a.path, b.path, ends[0][0], ends[0][1], ends[1][0], ends[1][1], a.path, za, zb, b.path, zb0, zb1
                            ));
                            found.disputed.push(ends);
                            continue;
                        }
                        let kind = match (a.falls(i), b.falls(j)) {
                            (Fall::Away, Fall::Away) if level => Kind::Ridge,
                            (Fall::Away, Fall::Away) => Kind::Hip,
                            (Fall::Toward, Fall::Toward) => Kind::Valley,
                            _ => {
                                found.problems.push(format!(
                                    "{} and {} meet from ({:.2}, {:.2}) to ({:.2}, {:.2}) where one falls toward the line and the other away, or one is flat: a change of pitch, which is not a ridge, a hip or a valley",
                                    a.path, b.path, ends[0][0], ends[0][1], ends[1][0], ends[1][1]
                                ));
                                found.disputed.push(ends);
                                continue;
                            }
                        };
                        found.lines.push(FoundLine { kind, ends, planes: vec![a.path.clone(), b.path.clone()], status: a.status.max(b.status) });
                    }
                    _ => {
                        let on_eave = geom::point_to_line(ends[0], a.eave[0], a.eave[1]).unwrap_or(f64::MAX) <= tol && geom::point_to_line(ends[1], a.eave[0], a.eave[1]).unwrap_or(f64::MAX) <= tol;
                        let kind = if on_eave {
                            Kind::Eave
                        } else if !level {
                            Kind::Rake
                        } else if a.falls(i) == Fall::Away {
                            Kind::Ridge
                        } else {
                            // Level, not the eave line, and not the high side: a stepped-back eave.
                            Kind::Eave
                        };
                        found.lines.push(FoundLine { kind, ends, planes: vec![a.path.clone()], status: a.status });
                    }
                }
            }
        }
    }
    merge_straight_runs(&mut found.lines, tol);
    found
}

/// Two found lines end to end in one straight line, of one kind on the same planes, are
/// one line: an eave traced with a point in the middle of it is still one eave.
fn merge_straight_runs(lines: &mut Vec<FoundLine>, tol: f64) {
    let mut i = 0;
    while i < lines.len() {
        let mut merged = false;
        for j in (i + 1)..lines.len() {
            let (a, b) = (&lines[i], &lines[j]);
            if a.kind != b.kind || a.planes != b.planes {
                continue;
            }
            for (ea, eb) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                if geom::dist(a.ends[ea], b.ends[eb]) > tol {
                    continue;
                }
                let (far_a, far_b) = (a.ends[1 - ea], b.ends[1 - eb]);
                // Straight through the joint, and running on, not doubling back.
                let straight = geom::point_to_line(a.ends[ea], far_a, far_b).is_some_and(|d| d <= tol);
                let onward = geom::dist(far_a, far_b) > geom::dist(far_a, a.ends[ea]).max(geom::dist(far_b, b.ends[eb]));
                if straight && onward {
                    let joined = FoundLine { kind: a.kind, ends: [far_a, far_b], planes: a.planes.clone(), status: a.status.max(b.status) };
                    lines[i] = joined;
                    lines.remove(j);
                    merged = true;
                    break;
                }
            }
            if merged {
                break;
            }
        }
        if !merged {
            i += 1;
        }
    }
}

/// An entered line set against the lines the planes imply.
#[derive(Clone, Debug, Default)]
pub struct Comparison {
    /// Entered lines that match an implied one: (path, kind).
    pub agree: Vec<(String, Kind)>,
    /// Entered lines the planes do not imply, each with why.
    pub entered_only: Vec<String>,
    /// Lines the planes imply that nobody has entered.
    pub implied_only: Vec<FoundLine>,
}

fn same_segment(a: [P2; 2], b: [P2; 2], tol: f64) -> bool {
    (geom::dist(a[0], b[0]) <= tol && geom::dist(a[1], b[1]) <= tol) || (geom::dist(a[0], b[1]) <= tol && geom::dist(a[1], b[0]) <= tol)
}

/// A level top edge on one plane is a ridge to the planes and may be a headwall to the
/// estimator, who knows there is a wall there. The two are the same line.
fn same_kind(entered: Kind, implied: &FoundLine) -> bool {
    entered == implied.kind || (entered == Kind::Headwall && implied.kind == Kind::Ridge && implied.planes.len() == 1)
}

/// Set the job's entered roof lines (everything but parapets, which lie on no plane)
/// against what its planes imply.
pub fn compare(job: &mut Job, found: &Found) -> Comparison {
    let tol = FIT_TOLERANCE_FT;
    let mut out = Comparison::default();
    let mut taken = vec![false; found.lines.len()];
    for &e in &job.order {
        let w = &job.world;
        let (Some(node), Some(kind), Some(run)) = (w.get::<Node>(e), w.get::<Kind>(e), w.get::<Run>(e)) else { continue };
        if *kind == Kind::Parapet {
            continue;
        }
        let Some(ends) = run.ends_ft() else {
            out.entered_only.push(format!("{} ({kind:?}): its ends are unknown", node.path));
            continue;
        };
        let mut planes = run.planes.clone();
        planes.sort();
        let here: Vec<usize> = (0..found.lines.len()).filter(|&i| !taken[i] && same_segment(ends, found.lines[i].ends, tol)).collect();
        let exact = here.iter().copied().find(|&i| {
            let mut theirs = found.lines[i].planes.clone();
            theirs.sort();
            same_kind(*kind, &found.lines[i]) && theirs == planes
        });
        match (exact, here.first()) {
            (Some(i), _) => {
                taken[i] = true;
                out.agree.push((node.path.clone(), *kind));
            }
            (None, Some(&i)) => {
                taken[i] = true;
                let f = &found.lines[i];
                out.entered_only.push(format!("{} is entered as a {kind:?} on {}; the planes make that line a {:?} on {}", node.path, if run.planes.is_empty() { "no plane".to_string() } else { run.planes.join(" and ") }, f.kind, f.planes.join(" and ")));
            }
            (None, None) if found.disputed.iter().any(|d| same_segment(ends, *d, tol)) => {
                out.entered_only.push(format!("{} ({kind:?}) runs along an edge its planes do not agree about; see the planes that do not fit each other", node.path))
            }
            (None, None) => out.entered_only.push(format!("{} ({kind:?}) from ({}, {}) to ({}, {}): no edge of any plane runs there", node.path, ends[0][0], ends[0][1], ends[1][0], ends[1][1])),
        }
    }
    out.implied_only = found.lines.iter().zip(&taken).filter(|(_, t)| !**t).map(|(l, _)| l.clone()).collect();
    out
}
