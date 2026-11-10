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
            return Some(format!("points {i} and {} are the same point", (i + 1) % n));
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            // Neighbouring edges share a corner; that is not a crossing.
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if segments_meet(pts[i], pts[(i + 1) % n], pts[j], pts[(j + 1) % n]) {
                return Some(format!("edge {i} and edge {j} cross or touch; the outline has to be a simple loop"));
            }
        }
    }
    // Neighbouring edges that fold straight back on each other.
    for i in 0..n {
        let (a, b, c) = (pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
        if orient(a, b, c) == 0.0 && on_segment(a, b, c) && dist(b, c) > 0.0 && ((c[0] - b[0]) * (a[0] - b[0]) + (c[1] - b[1]) * (a[1] - b[1])) > 0.0 {
            return Some(format!("the outline doubles back on itself at point {}", (i + 1) % n));
        }
    }
    if plan_area(pts) == 0.0 {
        return Some("the outline encloses no area".into());
    }
    None
}
