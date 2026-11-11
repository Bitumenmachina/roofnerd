//! Plan geometry: points, lines and outlines on the flat drawing, in feet.

use crate::components::P2;

pub fn dist(a: P2, b: P2) -> f64 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
}

/// Area of an outline on the plan. The same whichever way round it was traced.
pub fn plan_area(pts: &[P2]) -> f64 {
    let n = pts.len();
    let mut twice = 0.0;
    for i in 0..n {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        twice += p[0] * q[1] - q[0] * p[1];
    }
    twice.abs() / 2.0
}

/// How far a point is from a line segment.
pub fn point_to_segment(p: P2, a: P2, b: P2) -> f64 {
    let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
    let len2 = ex * ex + ey * ey;
    if len2 == 0.0 {
        return dist(p, a);
    }
    let t = (((p[0] - a[0]) * ex + (p[1] - a[1]) * ey) / len2).clamp(0.0, 1.0);
    dist(p, [a[0] + t * ex, a[1] + t * ey])
}

/// How far a point is from the infinite line through two points, measured square to it.
pub fn point_to_line(p: P2, a: P2, b: P2) -> Option<f64> {
    let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
    let len = (ex * ex + ey * ey).sqrt();
    if len == 0.0 {
        return None;
    }
    Some(((ex * (p[1] - a[1])) - (ey * (p[0] - a[0]))).abs() / len)
}

/// How far a point is from the nearest edge of an outline.
pub fn point_to_outline(p: P2, pts: &[P2]) -> f64 {
    let n = pts.len();
    (0..n).map(|i| point_to_segment(p, pts[i], pts[(i + 1) % n])).fold(f64::MAX, f64::min)
}

/// The edge of the outline that both points sit on, if there is one.
pub fn edge_holding(a: P2, b: P2, pts: &[P2], tolerance: f64) -> Option<usize> {
    let n = pts.len();
    (0..n).find(|&i| {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        point_to_segment(a, p, q) <= tolerance && point_to_segment(b, p, q) <= tolerance
    })
}

fn orient(a: P2, b: P2, c: P2) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn on_segment(a: P2, b: P2, p: P2) -> bool {
    p[0] >= a[0].min(b[0]) && p[0] <= a[0].max(b[0]) && p[1] >= a[1].min(b[1]) && p[1] <= a[1].max(b[1])
}

/// Do two segments touch or cross?
fn segments_meet(a: P2, b: P2, c: P2, d: P2) -> bool {
    let (o1, o2, o3, o4) = (orient(a, b, c), orient(a, b, d), orient(c, d, a), orient(c, d, b));
    if ((o1 > 0.0 && o2 < 0.0) || (o1 < 0.0 && o2 > 0.0)) && ((o3 > 0.0 && o4 < 0.0) || (o3 < 0.0 && o4 > 0.0)) {
        return true;
    }
    (o1 == 0.0 && on_segment(a, b, c)) || (o2 == 0.0 && on_segment(a, b, d)) || (o3 == 0.0 && on_segment(c, d, a)) || (o4 == 0.0 && on_segment(c, d, b))
}

/// What is wrong with an outline, if anything: an edge with no length, or edges that
/// cross or touch where they should not (a bowtie has no honest area).
pub fn outline_fault(pts: &[P2]) -> Option<String> {
    let n = pts.len();
    if n < 3 {
        return Some("an area needs at least three points".into());
    }
    for i in 0..n {
        if dist(pts[i], pts[(i + 1) % n]) == 0.0 {
            return Some(format!("points {} and {} are the same point", i + 1, (i + 1) % n + 1));
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            // Neighbouring edges share a corner; that is not a crossing.
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if segments_meet(pts[i], pts[(i + 1) % n], pts[j], pts[(j + 1) % n]) {
                return Some(format!("edge {} and edge {} cross or touch; the outline has to be a simple loop", i + 1, j + 1));
            }
        }
    }
    // Neighbouring edges that fold straight back on each other.
    for i in 0..n {
        let (a, b, c) = (pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
        if orient(a, b, c) == 0.0 && on_segment(a, b, c) && dist(b, c) > 0.0 && ((c[0] - b[0]) * (a[0] - b[0]) + (c[1] - b[1]) * (a[1] - b[1])) > 0.0 {
            return Some(format!("the outline doubles back on itself at point {}", (i + 1) % n + 1));
        }
    }
    if plan_area(pts) == 0.0 {
        return Some("the outline encloses no area".into());
    }
    None
}

/// Which side of the line through `a` and `b` a point is on, and how far: positive to the
/// left of the direction a to b, negative to the right.
pub fn signed_offset(p: P2, a: P2, b: P2) -> Option<f64> {
    let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
    let len = (ex * ex + ey * ey).sqrt();
    if len == 0.0 {
        return None;
    }
    Some(((ex * (p[1] - a[1])) - (ey * (p[0] - a[0]))) / len)
}

/// Does the whole segment run along the outline? Every stretch of it has to lie on an
/// outline edge: the edges within `tolerance` of the segment's line are laid along it and
/// have to cover it from end to end. A line that touches the outline at its ends and cuts
/// across the plane, or across a notch outside it, does not pass, however short the gap.
/// On failure: how far along the segment (0 to 1) the first uncovered stretch is, and how
/// far that point is from the outline.
pub fn runs_along_outline(a: P2, b: P2, pts: &[P2], tolerance: f64) -> Result<(), (f64, f64)> {
    let len = dist(a, b);
    if len == 0.0 {
        return Ok(());
    }
    let along = |x: P2| ((x[0] - a[0]) * (b[0] - a[0]) + (x[1] - a[1]) * (b[1] - a[1])) / len;
    let n = pts.len();
    let mut covered: Vec<(f64, f64)> = Vec::new();
    for i in 0..n {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        let (Some(d0), Some(d1)) = (signed_offset(p, a, b), signed_offset(q, a, b)) else { continue };
        // The part of this edge within `tolerance` of the segment's line, as a range of
        // s from 0 at p to 1 at q. The offset runs straight from d0 to d1 along the edge.
        let (s0, s1) = if d0.abs() <= tolerance && d1.abs() <= tolerance {
            (0.0, 1.0)
        } else if d0 == d1 {
            continue;
        } else {
            let (x, y) = ((-tolerance - d0) / (d1 - d0), (tolerance - d0) / (d1 - d0));
            (x.min(y).max(0.0), x.max(y).min(1.0))
        };
        if s1 < s0 {
            continue;
        }
        let at = |s: f64| [p[0] + s * (q[0] - p[0]), p[1] + s * (q[1] - p[1])];
        let (t0, t1) = (along(at(s0)), along(at(s1)));
        covered.push((t0.min(t1), t0.max(t1)));
    }
    covered.sort_by(|x, y| x.0.total_cmp(&y.0));
    // Walk the cover. What is not covered is added up: a hundred hairline gaps are as
    // much missing roof as one wide one, so it is the total that has to stay inside the
    // tolerance, not each gap.
    let fail = |from: f64, to: f64| {
        let t = (from + to) / 2.0;
        let p = [a[0] + t / len * (b[0] - a[0]), a[1] + t / len * (b[1] - a[1])];
        Err((t / len, point_to_outline(p, pts)))
    };
    let (mut reach, mut bare) = (0.0_f64, 0.0_f64);
    let mut widest: Option<(f64, f64)> = None;
    let mut gap = |from: f64, to: f64, bare: &mut f64| {
        if to > from {
            *bare += to - from;
            if widest.is_none_or(|(f, t)| to - from > t - f) {
                widest = Some((from, to));
            }
        }
    };
    for (lo, hi) in covered {
        if lo >= len {
            break;
        }
        gap(reach, lo.max(0.0), &mut bare);
        reach = reach.max(hi);
    }
    gap(reach, len, &mut bare);
    if bare > tolerance {
        let (from, to) = widest.unwrap_or((0.0, len));
        return fail(from, to);
    }
    Ok(())
}

/// Cut a simple outline into triangles by clipping ears. Each triangle is three places in
/// `pts`. The second value says whether every cut was clean: where an outline is so thin
/// that no ear is left to clip, a corner is taken anyway so a picture can still be drawn,
/// and whoever needs the triangles to be true (a solid to test for clashes) must not use
/// them.
pub fn triangles(pts: &[P2]) -> (Vec<[usize; 3]>, bool) {
    let n = pts.len();
    if n < 3 {
        return (Vec::new(), false);
    }
    let area: f64 = (0..n).map(|i| pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1]).sum();
    let mut ring: Vec<usize> = if area >= 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
    let cross = |a: P2, b: P2, c: P2| (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    let mut out = Vec::new();
    let mut clean = true;
    let mut guard = 0;
    while ring.len() > 3 && guard < 10_000 {
        guard += 1;
        let m = ring.len();
        let mut cut = None;
        for i in 0..m {
            let (a, b, c) = (pts[ring[(i + m - 1) % m]], pts[ring[i]], pts[ring[(i + 1) % m]]);
            if cross(a, b, c) <= 1e-12 {
                continue;
            }
            let inside = ring.iter().any(|&k| {
                let p = pts[k];
                p != a && p != b && p != c && cross(a, b, p) > 1e-12 && cross(b, c, p) > 1e-12 && cross(c, a, p) > 1e-12
            });
            if !inside {
                cut = Some(i);
                break;
            }
        }
        // A sliver with no clean ear left: take any corner rather than stop.
        clean &= cut.is_some();
        let i = cut.unwrap_or(0);
        let m = ring.len();
        out.push([ring[(i + m - 1) % m], ring[i], ring[(i + 1) % m]]);
        ring.remove(i);
    }
    if ring.len() == 3 {
        out.push([ring[0], ring[1], ring[2]]);
    } else {
        clean = false;
    }
    (out, clean)
}
