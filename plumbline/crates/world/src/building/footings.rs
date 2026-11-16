//! The concrete in strip footings where they meet: ours (BIMCraft has no footing; the
//! search is in PORTMAP.md under `footings`). Nothing here is public outside the model.
//!
//! A footing's concrete is its length times its width times its depth, less what of it
//! lies in a footing drawn before it, which is counted there. What lies in another is
//! worked exactly, from the two rectangles on plan: a stem that stops on a run, one that
//! crosses it, one that stops part way across it, one deeper than the run it stops on
//! (it keeps the concrete under the run). The one case nothing is taken off is the
//! plain corner, where two footings end at one point: there the square they share is
//! the square left open outside it, and length times width times depth is already
//! exact. (The twenty-first judgment, J21-1, J21-3, J21-4: the rule before this knew
//! the T and nothing else.)

use crate::components::P2;

pub(super) struct Strip {
    pub path: String,
    pub level: Option<String>,
    pub line: [P2; 2],
    pub width: f64,
    /// None: not known, and nothing is taken off it or for it.
    pub depth: Option<f64>,
    /// Its top above its level's floor, in feet.
    pub top: f64,
    /// The number in its path: the order the footings were drawn in.
    pub order: u32,
}

type Shape = Vec<P2>;

/// A footing's rectangle on plan, its corners counter-clockwise.
fn rectangle(line: [P2; 2], width: f64) -> Shape {
    let [a, b] = line;
    let (ux, uy) = (b[0] - a[0], b[1] - a[1]);
    let long = (ux * ux + uy * uy).sqrt().max(1.0e-12);
    let (nx, ny) = (-uy / long * width / 2.0, ux / long * width / 2.0);
    vec![[a[0] - nx, a[1] - ny], [b[0] - nx, b[1] - ny], [b[0] + nx, b[1] + ny], [a[0] + nx, a[1] + ny]]
}

fn area(shape: &Shape) -> f64 {
    let n = shape.len();
    (0..n).map(|i| shape[i][0] * shape[(i + 1) % n][1] - shape[(i + 1) % n][0] * shape[i][1]).sum::<f64>() / 2.0
}

/// A convex shape cut by the line through `a` and `b`: what lies to its left, and what
/// lies to its right.
fn cut(shape: &Shape, a: P2, b: P2) -> (Shape, Shape) {
    let side = |q: P2| (b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0]);
    let (mut left, mut right) = (Shape::new(), Shape::new());
    let n = shape.len();
    for i in 0..n {
        let (p, q) = (shape[i], shape[(i + 1) % n]);
        let (sp, sq) = (side(p), side(q));
        if sp >= 0.0 {
            left.push(p);
        }
        if sp <= 0.0 {
            right.push(p);
        }
        if (sp > 0.0 && sq < 0.0) || (sp < 0.0 && sq > 0.0) {
            let t = sp / (sp - sq);
            let x = [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
            left.push(x);
            right.push(x);
        }
    }
    (left, right)
}

/// A convex shape parted by another: what of it lies inside the other, and the pieces
/// of it that lie outside.
fn parted(shape: &Shape, by: &Shape) -> (Option<Shape>, Vec<Shape>) {
    let mut inside = shape.clone();
    let mut outside = Vec::new();
    for i in 0..by.len() {
        let (left, right) = cut(&inside, by[i], by[(i + 1) % by.len()]);
        if right.len() >= 3 && area(&right).abs() > 1.0e-9 {
            outside.push(right);
        }
        inside = left;
        if inside.len() < 3 || area(&inside).abs() <= 1.0e-9 {
            return (None, outside);
        }
    }
    (Some(inside), outside)
}

const SAME_POINT: f64 = 0.01;

fn near(a: P2, b: P2) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) < SAME_POINT
}

/// Whether two footings make a plain corner: an end of each at one point, the two not
/// in one line, and no third footing ending there or running through there.
fn corner(a: &Strip, b: &Strip, all: &[Strip]) -> bool {
    let turn = {
        let (ux, uy) = (a.line[1][0] - a.line[0][0], a.line[1][1] - a.line[0][1]);
        let (vx, vy) = (b.line[1][0] - b.line[0][0], b.line[1][1] - b.line[0][1]);
        ((ux * vy - uy * vx) / ((ux.hypot(uy) * vx.hypot(vy)).max(1.0e-12))).abs()
    };
    if turn < 0.2 {
        return false;
    }
    a.line.iter().any(|ea| {
        b.line.iter().any(|eb| near(*ea, *eb))
            && !all.iter().filter(|c| c.path != a.path && c.path != b.path && c.level == a.level).any(|c| {
                // A third footing there: ending at the point, or with the point on its plan.
                c.line.iter().any(|ec| near(*ec, *ea)) || parted(&vec![[ea[0] - 1.0e-4, ea[1] - 1.0e-4], [ea[0] + 1.0e-4, ea[1] - 1.0e-4], [ea[0] + 1.0e-4, ea[1] + 1.0e-4], [ea[0] - 1.0e-4, ea[1] + 1.0e-4]], &rectangle(c.line, c.width)).0.is_some()
            })
    })
}

/// What of a footing lies in footings drawn before it and is counted there: so many
/// cubic feet, and the footings it lies in.
pub(super) fn counted_elsewhere(me: &Strip, all: &[Strip]) -> (f64, Vec<String>) {
    let Some(deep) = me.depth else { return (0.0, Vec::new()) };
    let whole = rectangle(me.line, me.width);
    // Each piece of its plan, with how deep the concrete of earlier footings already is there.
    let mut pieces: Vec<(Shape, f64)> = vec![(whole.clone(), 0.0)];
    let mut within = Vec::new();
    let mut before: Vec<&Strip> = all.iter().filter(|o| o.path != me.path && o.level == me.level && o.order < me.order && (o.top - me.top).abs() < 1.0e-6 && o.depth.is_some()).collect();
    before.sort_by_key(|o| o.order);
    for other in before {
        if corner(me, other, all) {
            continue;
        }
        let its = rectangle(other.line, other.width);
        let theirs = other.depth.unwrap_or(0.0);
        let mut next = Vec::with_capacity(pieces.len() + 4);
        let mut met = false;
        for (piece, filled) in pieces {
            let (inside, outside) = parted(&piece, &its);
            if let Some(inside) = inside {
                met = true;
                next.push((inside, filled.max(theirs)));
            }
            next.extend(outside.into_iter().map(|o| (o, filled)));
        }
        pieces = next;
        if met {
            within.push(other.path.clone());
        }
    }
    let own: f64 = pieces.iter().map(|(piece, filled)| area(piece).abs() * (deep - filled).max(0.0)).sum();
    let taken = (area(&whole).abs() * deep - own).max(0.0);
    if taken < 1.0e-9 {
        (0.0, Vec::new())
    } else {
        (taken, within)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(n: u32, line: [P2; 2], depth: f64) -> Strip {
        Strip { path: format!("footing-{n:04}"), level: Some("level-0002".into()), line, width: 2.0, depth: Some(depth), top: 0.0, order: n }
    }
    /// The cubic feet of all of them together: each one's own, less what is counted elsewhere.
    fn concrete(all: &[Strip]) -> f64 {
        let cf: f64 = all.iter().map(|s| (s.line[1][0] - s.line[0][0]).hypot(s.line[1][1] - s.line[0][1]) * s.width * s.depth.unwrap() - counted_elsewhere(s, all).0).sum();
        (cf * 1.0e6).round() / 1.0e6
    }

    /// Every figure worked by hand: footings 24 in wide, 12 in deep unless said.
    #[test]
    fn concrete_where_footings_meet_is_counted_once() {
        let run = || strip(1, [[0.0, 0.0], [40.0, 0.0]], 1.0);
        // A T: a 20 ft stem on the run's middle. 80 + 40 - 2 = 118.
        assert_eq!(concrete(&[run(), strip(2, [[20.0, 0.0], [20.0, 20.0]], 1.0)]), 118.0);
        // A cross: 80 + 40 less the 2 by 2 square both hold = 116.
        assert_eq!(concrete(&[run(), strip(2, [[20.0, -10.0], [20.0, 10.0]], 1.0)]), 116.0);
        // Three at a point: two runs end to end and a stem where they meet. 40 + 40 + 38.
        assert_eq!(concrete(&[strip(1, [[0.0, 0.0], [20.0, 0.0]], 1.0), strip(2, [[20.0, 0.0], [40.0, 0.0]], 1.0), strip(3, [[20.0, 0.0], [20.0, 20.0]], 1.0)]), 118.0);
        // A stem that stops part way across the run, past its centre line: 1.5 ft of it
        // lies in the run. 80 + 37 = 117.
        assert_eq!(concrete(&[run(), strip(2, [[25.0, -0.5], [25.0, 19.5]], 1.0)]), 117.0);
        // A stem 18 in deep on a 12 in run keeps the concrete under the run: 80 + 57 + 1.
        assert_eq!(concrete(&[run(), strip(2, [[20.0, 0.0], [20.0, 20.0]], 1.5)]), 138.0);
        // A run deeper than the stem: 120 + 38.
        assert_eq!(concrete(&[strip(1, [[0.0, 0.0], [40.0, 0.0]], 1.5), strip(2, [[20.0, 0.0], [20.0, 20.0]], 1.0)]), 158.0);
        // A ring, 40 by 28 on its centre lines: nothing off at a corner. 136 x 2 = 272.
        let ring = [strip(1, [[0.0, 0.0], [40.0, 0.0]], 1.0), strip(2, [[40.0, 0.0], [40.0, 28.0]], 1.0), strip(3, [[40.0, 28.0], [0.0, 28.0]], 1.0), strip(4, [[0.0, 28.0], [0.0, 0.0]], 1.0)];
        assert_eq!(concrete(&ring), 272.0);
        // Whichever was drawn first, the total is the same: the stem before the run.
        assert_eq!(concrete(&[strip(1, [[20.0, 0.0], [20.0, 20.0]], 1.0), strip(2, [[0.0, 0.0], [40.0, 0.0]], 1.0)]), 118.0);
        // Two that do not touch take nothing from each other.
        assert_eq!(concrete(&[run(), strip(2, [[0.0, 10.0], [40.0, 10.0]], 1.0)]), 160.0);
    }
}
