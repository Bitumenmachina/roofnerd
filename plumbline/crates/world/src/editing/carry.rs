//! A line of points carried out sideways by half a thickness (DECISIONS.md call 190): the
//! clicks of a wall traced by one face of a scan become the wall's centre.
//!
//! PORTED CODE. The join of two moved stretches is ported from BIMCraft,
//! `crates/core/src/draw.rs` (`offset_poly`, a polyline moved by an offset to its left, each
//! corner where the two moved stretches meet) with the bisector form of the same corner
//! from `crates/core/src/stairs.rs` (`offset`), at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license.
//! The notice is in NOTICE at the repository root. `PORTMAP.md` gives the line.
//!
//! What was changed in the port: lines of points only (BIMCraft's also moves arcs); either
//! side, by a flag, where BIMCraft's offset is signed; a corner sharper than 15 degrees is
//! cut square where BIMCraft's refuses it past a mitre reach of 50 offsets (about 1 degree).

/// The corner, in degrees between the two stretches, below which it is cut square and not
/// carried to a point (a mitre at 15 degrees reaches 7.7 offsets; at 1 degree, 115). The
/// lead's, Assumed: moves only by a written call.
pub const SHARPEST_CORNER_DEG: f64 = 15.0;

fn unit(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let n = dx.hypot(dy);
    [dx / n, dy / n]
}

/// `points` (a line of them, or with `closed` a ring) carried `by` to the left of the way
/// it runs (`left`) or to its right: each stretch moved square to itself, each corner where
/// two moved stretches meet, so that two stretches in one line stay in line. The ends of an
/// open line move square to their one stretch. A corner sharper than
/// `SHARPEST_CORNER_DEG` is not carried to a point miles away: it is cut square, that is
/// the end of the stretch before moved square to it, and then the start of the stretch
/// after moved square to it, two points for the one corner. Points that repeat are taken
/// once; a ring written with its first point again at the end is the same ring.
pub fn carried_out(points: &[[f64; 2]], by: f64, left: bool, closed: bool) -> Vec<[f64; 2]> {
    let mut p: Vec<[f64; 2]> = Vec::with_capacity(points.len());
    for q in points {
        if p.last().is_none_or(|l| (l[0] - q[0]).hypot(l[1] - q[1]) > 1e-9) {
            p.push(*q);
        }
    }
    if closed && p.len() > 1 && (p[0][0] - p[p.len() - 1][0]).hypot(p[0][1] - p[p.len() - 1][1]) <= 1e-9 {
        p.pop();
    }
    // Two points make one stretch, there and back is not a ring.
    let closed = closed && p.len() >= 3;
    if p.len() < 2 || by == 0.0 || !by.is_finite() {
        return p;
    }
    let s = if left { by } else { -by };
    let n = p.len();
    let stretches = if closed { n } else { n - 1 };
    let dir: Vec<[f64; 2]> = (0..stretches).map(|i| unit(p[i], p[(i + 1) % n])).collect();
    // The left of a stretch running along u is (-u.y, u.x).
    let side = |u: [f64; 2]| [-u[1] * s, u[0] * s];
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let before = if i > 0 { Some(dir[i - 1]) } else if closed { Some(dir[n - 1]) } else { None };
        let after = if i < stretches { Some(dir[i]) } else { None };
        match (before, after) {
            (None, Some(b)) | (Some(b), None) => {
                let o = side(b);
                out.push([p[i][0] + o[0], p[i][1] + o[1]]);
            }
            (Some(a), Some(b)) => {
                let (oa, ob) = (side(a), side(b));
                let dot = a[0] * b[0] + a[1] * b[1];
                // The angle between the two stretches at the corner: 180 degrees where
                // they run on in one line, 0 where the second turns back on the first.
                let between = (-dot).clamp(-1.0, 1.0).acos().to_degrees();
                if between < SHARPEST_CORNER_DEG {
                    out.push([p[i][0] + oa[0], p[i][1] + oa[1]]);
                    out.push([p[i][0] + ob[0], p[i][1] + ob[1]]);
                } else {
                    // Where the two moved stretches meet: half the sum of the two moves, over
                    // one plus the cosine of the turn (BIMCraft stairs.rs `offset`).
                    let k = 1.0 / (1.0 + dot);
                    out.push([p[i][0] + (oa[0] + ob[0]) * k, p[i][1] + (oa[1] + ob[1]) * k]);
                }
            }
            (None, None) => out.push(p[i]),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: &[[f64; 2]], b: &[[f64; 2]]) {
        assert_eq!(a.len(), b.len(), "{a:?} against {b:?}");
        for (p, q) in a.iter().zip(b) {
            assert!((p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9, "{a:?} against {b:?}");
        }
    }

    const ROOM: [[f64; 2]; 4] = [[0.0, 0.0], [30.0, 0.0], [30.0, 16.0], [0.0, 16.0]];

    #[test]
    fn the_inside_corners_of_a_room_carried_to_the_outside_are_the_walls_centre() {
        // The line runs counterclockwise: its left is the room, its right the outside.
        near(&carried_out(&ROOM, 0.25, false, true), &[[-0.25, -0.25], [30.25, -0.25], [30.25, 16.25], [-0.25, 16.25]]);
        near(&carried_out(&ROOM, 0.25, true, true), &[[0.25, 0.25], [29.75, 0.25], [29.75, 15.75], [0.25, 15.75]]);
        // The same ring written the other way round: left is now the outside.
        let cw: Vec<[f64; 2]> = ROOM.iter().rev().copied().collect();
        near(&carried_out(&cw, 0.25, true, true), &[[-0.25, 16.25], [30.25, 16.25], [30.25, -0.25], [-0.25, -0.25]]);
        // A ring written with its first point again at the end is the same ring.
        let again: Vec<[f64; 2]> = ROOM.iter().chain(ROOM.iter().take(1)).copied().collect();
        near(&carried_out(&again, 0.25, false, true), &carried_out(&ROOM, 0.25, false, true));
    }

    #[test]
    fn an_open_lines_ends_move_square_to_their_one_stretch() {
        let line = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
        near(&carried_out(&line, 1.0, true, false), &[[0.0, 1.0], [9.0, 1.0], [9.0, 10.0]]);
        near(&carried_out(&line, 1.0, false, false), &[[0.0, -1.0], [11.0, -1.0], [11.0, 10.0]]);
        // A turned line: the end of a stretch at 30 degrees moves square to it.
        let a = 30f64.to_radians();
        let two = carried_out(&[[0.0, 0.0], [10.0 * a.cos(), 10.0 * a.sin()]], 2.0, true, false);
        near(&two, &[[-2.0 * a.sin(), 2.0 * a.cos()], [10.0 * a.cos() - 2.0 * a.sin(), 10.0 * a.sin() + 2.0 * a.cos()]]);
    }

    #[test]
    fn two_stretches_in_one_line_stay_in_line() {
        near(&carried_out(&[[0.0, 0.0], [5.0, 0.0], [10.0, 0.0]], 0.5, true, false), &[[0.0, 0.5], [5.0, 0.5], [10.0, 0.5]]);
        near(&carried_out(&[[0.0, 0.0], [3.0, 4.0], [6.0, 8.0]], 5.0, false, false), &[[4.0, -3.0], [7.0, 1.0], [10.0, 5.0]]);
    }

    #[test]
    fn a_corner_under_fifteen_degrees_is_cut_square_not_carried_miles_away() {
        // The line goes 10 ft east and comes back at 5.7 degrees (1 ft up in 10).
        let sharp = [[0.0, 0.0], [10.0, 0.0], [0.0, 1.0]];
        let out = carried_out(&sharp, 0.5, true, false);
        assert_eq!(out.len(), 4, "the sharp corner is two points: {out:?}");
        let k = 101f64.sqrt();
        // End of the first stretch moved square to it, then the start of the second moved square to it.
        near(&out, &[[0.0, 0.5], [10.0, 0.5], [10.0 - 0.5 / k, -0.5 * 10.0 / k], [-0.5 / k, 1.0 - 0.5 * 10.0 / k]]);
        // No point lies further from the line than the move.
        for (q, from) in out.iter().zip([[0.0, 0.0], [10.0, 0.0], [10.0, 0.0], [0.0, 1.0]]) {
            assert!((q[0] - from[0]).hypot(q[1] - from[1]) <= 0.5 + 1e-9, "{q:?}");
        }
        // A reversal exactly (back on itself) is cut square too, and does not divide by zero.
        let back = carried_out(&[[0.0, 0.0], [10.0, 0.0], [0.0, 0.0 + 1e-12]], 0.5, true, false);
        assert!(back.iter().all(|q| q[0].is_finite() && q[1].is_finite()), "{back:?}");
        // At 20 degrees a corner is still carried to a point: its distance is the move over sin(10 degrees).
        let a = 20f64.to_radians();
        let mitred = carried_out(&[[0.0, 0.0], [10.0, 0.0], [10.0 - 10.0 * a.cos(), 10.0 * a.sin()]], 1.0, true, false);
        assert_eq!(mitred.len(), 3);
        let d = (mitred[1][0] - 10.0).hypot(mitred[1][1]);
        assert!((d - 1.0 / (a / 2.0).sin()).abs() < 1e-9, "{d}");
    }

    #[test]
    fn nothing_to_carry_comes_back_as_it_was() {
        assert_eq!(carried_out(&[], 1.0, true, false), Vec::<[f64; 2]>::new());
        assert_eq!(carried_out(&[[1.0, 2.0]], 1.0, true, true), vec![[1.0, 2.0]]);
        assert_eq!(carried_out(&ROOM, 0.0, true, true), ROOM.to_vec());
        // Repeated points are taken once; a ring of two is a line there and back.
        near(&carried_out(&[[0.0, 0.0], [0.0, 0.0], [4.0, 0.0]], 1.0, true, false), &[[0.0, 1.0], [4.0, 1.0]]);
        near(&carried_out(&[[0.0, 0.0], [4.0, 0.0]], 1.0, true, true), &[[0.0, 1.0], [4.0, 1.0]]);
    }
}
