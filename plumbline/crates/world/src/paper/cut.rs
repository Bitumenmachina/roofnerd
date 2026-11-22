//! What of a plan's cut walls is inked: where walls join they read as one, the edge of what
//! they make together drawn and the line between them not.
//!
//! PORTED CODE. Ported from BIMCraft, `crates/core/src/render.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//!
//! What was changed, and nothing else:
//! * `cut_unions` (`render.rs:779`): the outlines of one fill are given as they are, one
//!   fill for one colour. BIMCraft's groups them by the colour the view's look gives each
//!   (`Look::cut_fill`); a plan set fills a page's cut walls in one colour, so the group is
//!   the page's, and `union` only turns every ring counter-clockwise, so that rings that
//!   overlap are one shape under the PDF's non-zero fill (BIMCraft fills "even-odd" in a
//!   renderer where overlapping walls would cancel, `render.rs:2788`).
//! * `cut_edges` (`render.rs:897`) is taken: for each outline, the runs of its edges that
//!   no other outline lies against on their outer side, cut where another outline's edge
//!   crosses or ends on them; the runs that end where the first begins joined; and
//!   `turn_corners` after. Lengths are feet, not millimetres: its 1 mm look-past (`PAST`) is
//!   a parameter, and its tolerances of a millionth of a millimetre are a ten-millionth of a
//!   foot. Its grid of boxes (`BoxGrid`, `render.rs:790`), which finds near outlines
//!   without asking each of every other, is not taken: a level's walls are asked every one,
//!   which is what BIMCraft's own test holds the grid to (`render.rs:917`). Its elements'
//!   ids are not carried: the runs come back in the order the outlines were given.
//! * `ink_cut` (`render.rs:976`): a run that comes back to where it starts is a closed
//!   ring, so none of its corners is a line's round end; the function returns the points
//!   and whether it closes, where BIMCraft draws them.
//! * `turn_corners` (`render.rs:986`) is taken as it is, without its cells of a micron
//!   (the same search by every end, which BIMCraft's own test holds its cells to).
//! * The small geometry it stands on (`geom::contains`, `param_on_segment`,
//!   `dist_to_segment`, `crossing`, `ring_edges`, `area`; `geom.rs:124`, `324`, `330` and
//!   so on) is written out here, point for point, as `building/rooms.rs` has it.

use crate::skeleton::P2 as V;

/// A hair, in feet, for two points to be one.
const SAME: f64 = 1e-7;

fn ring_edges(ring: &[V]) -> impl Iterator<Item = (V, V)> + '_ {
    ring.iter().copied().zip(ring.iter().copied().cycle().skip(1))
}

/// Signed area of a ring (counter-clockwise positive).
pub fn area(ring: &[V]) -> f64 {
    ring_edges(ring).map(|(a, b)| a.cross(b)).sum::<f64>() / 2.0
}

/// Even-odd point in ring.
fn contains(ring: &[V], p: V) -> bool {
    ring_edges(ring).filter(|(a, b)| (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (b.0 - a.0) / (b.1 - a.1)).count() % 2 == 1
}

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

/// The rings of one fill, each counter-clockwise, to be filled as one shape.
pub fn union(rings: &[Vec<V>]) -> Vec<Vec<V>> {
    rings
        .iter()
        .filter(|r| r.len() >= 3)
        .map(|r| {
            let mut r = r.clone();
            if area(&r) < 0.0 {
                r.reverse();
            }
            r
        })
        .collect()
}

/// What of each outline is the edge of what the outlines cut together, as runs of points:
/// its edges, but for the stretches another outline lies against on their outer side (within
/// `past` of the edge); where two runs meet, each carried on `reach` round the corner
/// (`turn_corners`). One entry for each outline, in order.
pub fn cut_edges(outlines: &[Vec<V>], reach: f64, past: f64) -> Vec<Vec<Vec<V>>> {
    let boxes: Vec<(V, V)> = outlines
        .iter()
        .map(|o| o.iter().fold((V(f64::MAX, f64::MAX), V(f64::MIN, f64::MIN)), |(lo, hi), p| (V(lo.0.min(p.0), lo.1.min(p.1)), V(hi.0.max(p.0), hi.1.max(p.1)))))
        .collect();
    let near = |i: usize, j: usize| {
        let ((a, b), (c, d)) = (boxes[i], boxes[j]);
        i != j && a.0 <= d.0 + past && c.0 <= b.0 + past && a.1 <= d.1 + past && c.1 <= b.1 + past
    };
    let mut out = Vec::with_capacity(outlines.len());
    for (i, o) in outlines.iter().enumerate() {
        let others: Vec<&[V]> = (0..outlines.len()).filter(|&j| near(i, j)).map(|j| outlines[j].as_slice()).collect();
        if others.is_empty() {
            let mut ring = o.clone();
            ring.extend(o.first().copied());
            out.push(vec![ring]);
            continue;
        }
        // Outward is to the right of a counter-clockwise ring.
        let ccw = area(o) > 0.0;
        let mut runs: Vec<Vec<V>> = Vec::new();
        for (a, b) in ring_edges(o) {
            let len = a.dist(b);
            if len <= 0.0 {
                continue;
            }
            // Cut where another outline's edge crosses it, or starts or ends on it.
            let mut ts = vec![0.0, 1.0];
            for (c, e) in others.iter().flat_map(|r| ring_edges(r)) {
                ts.extend(crossing(a, b, c, e));
                for p in [c, e] {
                    if dist_to_segment(p, a, b) < SAME * len.max(1.0) {
                        ts.push(param_on_segment(p, a, b));
                    }
                }
            }
            ts.sort_by(f64::total_cmp);
            ts.dedup_by(|x, y| (*x - *y) * len < SAME);
            let out_of = ((b - a).perp() * if ccw { -1.0 } else { 1.0 }).unit() * past;
            for w in ts.windows(2) {
                let (p, q) = (a + (b - a) * w[0], a + (b - a) * w[1]);
                let probe = (p + q) * 0.5 + out_of;
                if others.iter().any(|r| contains(r, probe)) {
                    continue;
                }
                match runs.last_mut() {
                    Some(run) if run.last().is_some_and(|l| l.dist(p) < SAME) => run.push(q),
                    _ => runs.push(vec![p, q]),
                }
            }
        }
        // A run that ends where the first starts goes on round.
        if runs.len() > 1 {
            if let (Some(first), Some(last)) = (runs.first().and_then(|r| r.first().copied()), runs.last().and_then(|r| r.last().copied())) {
                if first.dist(last) < SAME {
                    let mut last = runs.pop().unwrap_or_default();
                    last.extend(runs[0].iter().skip(1));
                    runs[0] = last;
                }
            }
        }
        out.push(runs);
    }
    turn_corners(&mut out, reach);
    out
}

/// A run of `cut_edges` as it is inked: one that comes back to where it starts (an outline
/// nothing lies against) is a closed ring, so none of its corners is a line's round end.
/// The points, and whether they close.
pub fn ink_cut(run: &[V]) -> (Vec<V>, bool) {
    match run.split_last() {
        Some((last, ring)) if ring.len() > 2 && ring[0].dist(*last) < SAME => (ring.to_vec(), true),
        _ => (run.to_vec(), false),
    }
}

/// Each run that ends where another ends (two walls' outlines meeting at a corner) carried on
/// `reach` (or the other's first edge, if shorter) along the other: so the corner is a join
/// inside a line, mitred square, not two ends meeting with a notch between them. What is
/// carried on draws over the other's line, in its place.
fn turn_corners(runs: &mut [Vec<Vec<V>>], reach: f64) {
    // Every end: its point, and the way its run goes from it.
    let ends: Vec<(V, V)> = runs
        .iter()
        .flatten()
        .filter(|r| r.len() >= 2 && r.first().zip(r.last()).is_some_and(|(a, b)| a.dist(*b) > SAME))
        .flat_map(|r| {
            let n = r.len();
            [(r[0], r[1]), (r[n - 1], r[n - 2])]
        })
        .collect();
    let on = |at: V, from: V| {
        let first = (0..ends.len()).find(|&i| ends.get(i).is_some_and(|(p, q)| p.dist(at) < SAME && q.dist(from) > SAME));
        first.and_then(|i| ends.get(i)).map(|(p, q)| {
            let len = p.dist(*q);
            *p + (*q - *p) * (reach.min(len) / len)
        })
    };
    for r in runs.iter_mut().flatten() {
        let n = r.len();
        if n < 2 || r[0].dist(r[n - 1]) <= SAME {
            continue;
        }
        let (head, tail) = (on(r[0], r[1]), on(r[n - 1], r[n - 2]));
        if let Some(q) = tail {
            r.push(q);
        }
        if let Some(q) = head {
            r.insert(0, q);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<V> {
        vec![V(x0, y0), V(x1, y0), V(x1, y1), V(x0, y1)]
    }

    #[test]
    fn two_rectangles_end_to_end_are_inked_round_both_together_and_not_between() {
        // A wall 0 to 10 and another 10 to 20, each half a foot thick: the line where they
        // meet is not inked, the rest is.
        let (a, b) = (rect(0.0, 0.0, 10.0, 0.5), rect(10.0, 0.0, 20.0, 0.5));
        let runs = cut_edges(&[a, b], 0.01, 0.003);
        let inked: f64 = runs.iter().flatten().map(|r| r.windows(2).map(|w| w[0].dist(w[1])).sum::<f64>()).sum();
        // 2 x (10 + 0.5 + 10 + 0.5) less the shared end (0.5 twice) plus the corner carried on (0.01 each way, four corners)
        assert!((inked - (2.0 * 21.0 - 1.0 + 4.0 * 0.01)).abs() < 1e-9, "{inked}");
        // No run lies along x = 10 for the full thickness.
        for r in runs.iter().flatten() {
            for w in r.windows(2) {
                let along_the_joint = (w[0].0 - 10.0).abs() < 1e-9 && (w[1].0 - 10.0).abs() < 1e-9 && w[0].dist(w[1]) > 0.02;
                assert!(!along_the_joint, "{w:?}");
            }
        }
    }

    #[test]
    fn one_wall_alone_is_a_closed_ring() {
        let runs = cut_edges(&[rect(0.0, 0.0, 10.0, 0.5)], 0.01, 0.003);
        assert_eq!(runs.len(), 1);
        let (pts, closed) = ink_cut(&runs[0][0]);
        assert!(closed);
        assert_eq!(pts.len(), 4);
    }

    #[test]
    fn rings_are_turned_counter_clockwise_to_fill_as_one() {
        let cw: Vec<V> = rect(0.0, 0.0, 1.0, 1.0).into_iter().rev().collect();
        assert!(area(&union(&[cw])[0]) > 0.0);
    }
}
