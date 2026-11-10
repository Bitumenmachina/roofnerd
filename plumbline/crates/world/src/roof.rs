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
    /// What a reader should know about how the planes meet, in sentences: planes that
    /// disagree about a shared edge, a step, an overlap, an edge three planes claim, a fold
    /// this program has no kind of line for.
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

/// The stretch of edge p-q that edge r-s lies along, as a range of distance along p-q.
///
/// A shared stretch ends at a corner: an end of one edge that sits on the other. So the
/// stretch runs between corners of r-s that are within `tol` of the line through p-q and
/// corners of p-q that are within `tol` of r-s. An edge that only crosses, or only touches
/// at a corner, has no two such corners apart and shares nothing. Two edges a hair out of
/// line with each other share the same stretch whichever of the two is asked.
fn alongside(p: P2, q: P2, r: P2, s: P2, tol: f64) -> Option<(f64, f64)> {
    let len = geom::dist(p, q);
    let along = |x: P2| (((x[0] - p[0]) * (q[0] - p[0]) + (x[1] - p[1]) * (q[1] - p[1])) / len).clamp(0.0, len);
    let mut at: Vec<f64> = Vec::new();
    for x in [r, s] {
        if geom::point_to_line(x, p, q).is_some_and(|d| d <= tol) {
            at.push(along(x));
        }
    }
    if geom::point_to_segment(p, r, s) <= tol {
        at.push(0.0);
    }
    if geom::point_to_segment(q, r, s) <= tol {
        at.push(len);
    }
    let lo = at.iter().copied().fold(f64::MAX, f64::min);
    let hi = at.iter().copied().fold(f64::MIN, f64::max);
    (at.len() >= 2 && hi - lo > tol).then_some((lo, hi))
}

fn same_segment(a: [P2; 2], b: [P2; 2], tol: f64) -> bool {
    (geom::dist(a[0], b[0]) <= tol && geom::dist(a[1], b[1]) <= tol) || (geom::dist(a[0], b[1]) <= tol && geom::dist(a[1], b[0]) <= tol)
}

/// The lines these planes imply.
pub fn lines_of(planes: &[SolidPlane]) -> Found {
    let tol = FIT_TOLERANCE_FT;
    let mut found = Found::default();
    // A line's end that falls on a traced corner is that corner, to the last digit, not a
    // point worked out along an edge that lands a hair off it.
    let corners: Vec<P2> = planes.iter().flat_map(|p| p.points.iter().copied()).collect();
    let snap = |x: P2| corners.iter().copied().find(|c| geom::dist(*c, x) <= tol).unwrap_or(x);
    // Each shared stretch is seen from both of its planes. What is said about it is kept
    // with the pair and the stretch, so it is said once, whichever plane comes first.
    let mut said: Vec<(usize, usize, [P2; 2])> = Vec::new();
    let mut flat_noted: Vec<usize> = Vec::new();

    for (ai, a) in planes.iter().enumerate() {
        let n = a.points.len();
        for i in 0..n {
            let (p, q) = (a.points[i], a.points[(i + 1) % n]);
            let len = geom::dist(p, q);
            let at = |t: f64| [p[0] + (q[0] - p[0]) * t / len, p[1] + (q[1] - p[1]) * t / len];

            // Every stretch of this edge that another plane's edge lies along.
            let mut shared: Vec<(f64, f64, usize, usize)> = Vec::new();
            for (bi, b) in planes.iter().enumerate() {
                if bi == ai {
                    continue;
                }
                let m = b.points.len();
                for j in 0..m {
                    if let Some((lo, hi)) = alongside(p, q, b.points[j], b.points[(j + 1) % m], tol) {
                        shared.push((lo, hi, bi, j));
                    }
                }
            }
            shared.sort_by(|x, y| x.0.total_cmp(&y.0));

            // Walk the edge: shared stretches, and the gaps between them that only this
            // plane has.
            let mut pieces: Vec<(f64, f64, Option<(usize, usize)>)> = Vec::new();
            let mut cursor = 0.0;
            for (lo, hi, bi, j) in &shared {
                if *lo < cursor - tol {
                    let near = snap(at(*lo));
                    found.problems.push(format!("more than two planes meet along one line near ({:.2}, {:.2}), {} and {} among them; a line has two sides", near[0], near[1], a.path, planes[*bi].path));
                    continue;
                }
                if *lo - cursor > tol {
                    pieces.push((cursor, *lo, None));
                }
                pieces.push((*lo, *hi, Some((*bi, *j))));
                cursor = *hi;
            }
            if len - cursor > tol {
                pieces.push((cursor, len, None));
            }

            for (from, to, other) in pieces {
                let ends = [snap(at(from)), snap(at(to))];
                let (za, zb) = (a.height_at(ends[0]), a.height_at(ends[1]));
                let level = (za - zb).abs() <= tol;
                // This plane's own edge here, when no other plane meets it in space.
                let mut own_edge = |found: &mut Found| {
                    if a.pitch == 0.0 {
                        if !flat_noted.contains(&ai) {
                            flat_noted.push(ai);
                            found.problems.push(format!("{} is flat (no pitch): its edges are not eaves, rakes or ridges, so none are listed for it", a.path));
                        }
                        return;
                    }
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
                };
                let Some((bi, j)) = other else {
                    own_edge(&mut found);
                    continue;
                };
                let b = &planes[bi];
                let (lo_i, hi_i) = (ai.min(bi), ai.max(bi));
                let first_time = !said.iter().any(|(x, y, seg)| *x == lo_i && *y == hi_i && same_segment(*seg, ends, 3.0 * tol));
                let mut say = |found: &mut Found, text: String| {
                    if first_time {
                        said.push((lo_i, hi_i, ends));
                        found.problems.push(text);
                        found.disputed.push(ends);
                    }
                };
                let span = format!("from ({:.2}, {:.2}) to ({:.2}, {:.2})", ends[0][0], ends[0][1], ends[1][0], ends[1][1]);
                // Two planes meet along an edge from opposite sides of it. On the same
                // side, one lies over the other.
                let (na, nb) = (a.inward(i), b.inward(j));
                if na[0] * nb[0] + na[1] * nb[1] > 0.0 {
                    say(&mut found, format!("{} and {} both lie on the same side of an edge they share {span}: one overlaps the other, and no line is read there", a.path, b.path));
                    continue;
                }
                let (zb0, zb1) = (b.height_at(ends[0]), b.height_at(ends[1]));
                if (za - zb0).abs() > tol || (zb - zb1).abs() > tol {
                    let b_level = (zb0 - zb1).abs() <= tol;
                    if level && b_level {
                        // Level on both, at two heights: on plan one line, in the air two.
                        // A step with a wall between them, or pitches that do not fit; the
                        // planes cannot say which. Each plane keeps its own edge.
                        say(&mut found, format!("{} and {} meet on plan {span} at different heights ({:.2} ft and {:.2} ft): a step with a wall between them, or pitches that do not fit. Each plane's own edge is listed", a.path, b.path, za, zb0));
                        own_edge(&mut found);
                    } else {
                        // Not level, and not at one height: a wall between them where a
                        // gable stands beside a lower roof, or planes that do not fit.
                        // Either way each plane has an edge of its own there.
                        say(&mut found, format!("{} and {} share an edge {span} but disagree about its height: {} puts its ends at {:.2} ft and {:.2} ft, {} at {:.2} ft and {:.2} ft. A wall stands between them there, or they do not fit. Each plane's own edge is listed", a.path, b.path, a.path, za, zb, b.path, zb0, zb1));
                        own_edge(&mut found);
                    }
                    continue;
                }
                let kind = match (a.falls(i), b.falls(j)) {
                    (Fall::Away, Fall::Away) if level => Kind::Ridge,
                    (Fall::Away, Fall::Away) => Kind::Hip,
                    (Fall::Toward, Fall::Toward) => Kind::Valley,
                    // One falls to the line and the other on from it, at different pitches:
                    // a change of pitch.
                    (Fall::Away, Fall::Toward) | (Fall::Toward, Fall::Away) if a.pitch != b.pitch => Kind::PitchBreak,
                    (fa, fb) => {
                        let (ua, ub) = (a.uphill(), b.uphill());
                        let one_plane = a.pitch == b.pitch && a.pitch != 0.0 && ua[0] * ub[0] + ua[1] * ub[1] > 1.0 - 1e-9;
                        say(
                            &mut found,
                            if one_plane {
                                format!("{} and {} are one plane traced in two pieces, joined {span}; there is no line there", a.path, b.path)
                            } else if a.pitch == 0.0 && b.pitch == 0.0 {
                                format!("{} and {} are both flat and meet {span}; there is no line there", a.path, b.path)
                            } else if fa == Fall::Neither || fb == Fall::Neither {
                                format!("{} and {} meet {span} where one of them is flat or runs straight up the slope beside the line; that is not a ridge, a hip or a valley", a.path, b.path)
                            } else {
                                format!("{} and {} meet {span} at one pitch, one falling toward the line and the other away, but not in the same direction; that is not a ridge, a hip, a valley or a change of pitch", a.path, b.path)
                            },
                        );
                        continue;
                    }
                };
                if first_time {
                    said.push((lo_i, hi_i, ends));
                    found.lines.push(FoundLine { kind, ends, planes: vec![planes[lo_i].path.clone(), planes[hi_i].path.clone()], status: a.status.max(b.status) });
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
    struct Entered {
        path: String,
        kind: Kind,
        ends: Option<[P2; 2]>,
        planes: Vec<String>,
    }
    let mut entered: Vec<Entered> = Vec::new();
    for &e in &job.order {
        let w = &job.world;
        let (Some(node), Some(kind), Some(run)) = (w.get::<Node>(e), w.get::<Kind>(e), w.get::<Run>(e)) else { continue };
        if *kind != Kind::Parapet {
            entered.push(Entered { path: node.path.clone(), kind: *kind, ends: run.ends_ft(), planes: run.planes.clone() });
        }
    }
    let sorted = |v: &[String]| {
        let mut v = v.to_vec();
        v.sort();
        v
    };
    // First, every entered line that is exactly an implied one takes it. A right line is
    // never crowded out by a wrong one entered before it on the same edge.
    let mut taken_by: Vec<Option<usize>> = vec![None; found.lines.len()];
    let mut matched = vec![false; entered.len()];
    for (ei, en) in entered.iter().enumerate() {
        let Some(ends) = en.ends else { continue };
        let hit = (0..found.lines.len()).find(|&i| taken_by[i].is_none() && same_segment(ends, found.lines[i].ends, tol) && same_kind(en.kind, &found.lines[i]) && sorted(&found.lines[i].planes) == sorted(&en.planes));
        if let Some(i) = hit {
            taken_by[i] = Some(ei);
            matched[ei] = true;
            out.agree.push((en.path.clone(), en.kind));
        }
    }
    // Then say what is wrong with each of the rest.
    let placed: Vec<String> = solid_planes(job).0.into_iter().map(|p| p.path).collect();
    for (ei, en) in entered.iter().enumerate() {
        if matched[ei] {
            continue;
        }
        if let Some(out_of_it) = en.planes.iter().find(|p| !placed.contains(p)) {
            out.entered_only.push(format!("{} ({:?}) names {out_of_it}, which could not be placed (see the planes left out), so nothing is said about this line", en.path, en.kind));
            continue;
        }
        let Some(ends) = en.ends else {
            out.entered_only.push(format!("{} ({:?}): its ends are unknown", en.path, en.kind));
            continue;
        };
        let on = if en.planes.is_empty() { "no plane".to_string() } else { en.planes.join(" and ") };
        let here = (0..found.lines.len()).find(|&i| same_segment(ends, found.lines[i].ends, tol));
        let within = (0..found.lines.len()).find(|&i| {
            let f = found.lines[i].ends;
            geom::point_to_segment(ends[0], f[0], f[1]) <= tol && geom::point_to_segment(ends[1], f[0], f[1]) <= tol
        });
        // An edge the planes dispute is said first: what the planes make of the line is
        // not to be relied on there.
        if found.disputed.iter().any(|d| same_segment(ends, *d, tol)) {
            out.entered_only.push(format!("{} ({:?}) runs along an edge its planes do not agree about; see what is said about those planes", en.path, en.kind));
            continue;
        }
        out.entered_only.push(match (here, within) {
            (Some(i), _) => {
                let f = &found.lines[i];
                let exact = same_kind(en.kind, f) && sorted(&f.planes) == sorted(&en.planes);
                match taken_by[i] {
                    Some(other) if exact => format!("{} is the same line as {}, entered twice", en.path, entered[other].path),
                    _ => format!("{} is entered as a {:?} on {on}; the planes make that line a {:?} on {}", en.path, en.kind, f.kind, f.planes.join(" and ")),
                }
            }
            (None, Some(i)) => {
                let f = &found.lines[i];
                format!("{} ({:?}) covers only part of the {:?} the planes make from ({}, {}) to ({}, {})", en.path, en.kind, f.kind, f.ends[0][0], f.ends[0][1], f.ends[1][0], f.ends[1][1])
            }
            (None, None) => format!("{} ({:?}) from ({}, {}) to ({}, {}): no edge of any plane runs there", en.path, en.kind, ends[0][0], ends[0][1], ends[1][0], ends[1][1]),
        });
    }
    out.implied_only = found.lines.iter().zip(&taken_by).filter(|(_, t)| t.is_none()).map(|(l, _)| l.clone()).collect();
    out
}
