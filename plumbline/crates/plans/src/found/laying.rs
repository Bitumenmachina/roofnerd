use super::*;

/// Which end of a wall is its first, in words (west or south).
pub(crate) fn end_word(w: &Wall) -> &'static str {
    let (dx, dy) = (w.to[0] - w.from[0], w.to[1] - w.from[1]);
    if dx.abs() >= dy.abs() { "west" } else { "south" }
}

/// The distance from a place to a wall's line, as a stretch from end to end.
pub(crate) fn distance_to(p: P2, a: P2, b: P2) -> f64 {
    let (e, l) = (sub(b, a), (b[0] - a[0]).hypot(b[1] - a[1]));
    let t = if l < 1e-9 { 0.0 } else { (dot(sub(p, a), e) / (l * l)).clamp(0.0, 1.0) };
    (p[0] - (a[0] + e[0] * t)).hypot(p[1] - (a[1] + e[1] * t))
}

/// Where the walls found do not close (call 179, A2): each end of a wall that comes within a foot
/// (CORNER) of no other wall's line. What is missing there is said, not guessed.
pub(crate) fn open_ends(walls: &[Wall]) -> Vec<(usize, P2)> {
    let mut out = Vec::new();
    for (i, w) in walls.iter().enumerate().filter(|(_, w)| w.doubt.is_none()) {
        for p in [w.from, w.to] {
            let meets = walls.iter().enumerate().any(|(j, v)| j != i && v.doubt.is_none() && distance_to(p, v.from, v.to) <= CORNER + v.thickness.unwrap_or(0.0) / 2.0);
            if !meets {
                out.push((i, p));
            }
        }
    }
    out
}

/// What an opening's measuring needs to know of the wall it is in.
pub(super) struct WallBuild {
    /// (along, up) of the points of its faces.
    pub(super) pts: Vec<[f64; 2]>,
    pub(super) theta: f64,
    pub(super) rho: f64,
    pub(super) s0: f64,
    /// Its first end is at its far end along the way the points were measured.
    pub(super) flip: bool,
}

/// A wall on the plan from its line's direction, place and ends, put first end first.
pub(super) fn wall_of(theta: f64, rho: f64, s0: f64, s1: f64, thickness: Option<f64>, seen_from: Option<P2>) -> (Wall, bool) {
    let (d, n) = (dir(theta), normal(theta));
    let at = |s: f64| [s * d[0] + rho * n[0], s * d[1] + rho * n[1]];
    let (mut from, mut to) = (at(s0), at(s1));
    // West to east where it runs more east-west, south to north where more north-south.
    let flip = if d[0].abs() >= d[1].abs() { d[0] < 0.0 } else { d[1] < 0.0 };
    if flip {
        std::mem::swap(&mut from, &mut to);
    }
    (Wall { from, to, thickness, seen_from, openings: Vec::new(), hidden: Vec::new(), ..Wall::default() }, flip)
}

/// The height of a face that stops short: the top of its points, where their number
/// per 0.1 ft of height is still half of what is usual. The nearest point to the top
/// would be a stray's or the scatter's, and what stands on it (a top that spills into
/// the face's strip) is not counted: the first two such tenths in a row end it.
pub(super) fn top_of(pts: &[[f64; 3]]) -> f64 {
    let bins = pts.iter().map(|p| (p[2] / 0.1) as usize).max().unwrap_or(0) + 1;
    let mut n = vec![0usize; bins];
    for p in pts {
        n[(p[2] / 0.1) as usize] += 1;
    }
    let usual = median(n.iter().filter(|x| **x > 0).map(|x| *x as f64).collect());
    let first = n.iter().position(|x| *x > 0).unwrap_or(0);
    let mut i = first;
    while i < bins {
        if (n[i] as f64) < usual / 2.0 && (i + 1 >= bins || (n[i + 1] as f64) < usual / 2.0) {
            break;
        }
        i += 1;
    }
    i as f64 * 0.1
}

// ---- laying the found walls: ends carried to meet, one-sided walls given a thickness ----

/// A wall as it is laid: the centre line, and its openings measured along that line.
#[derive(Clone, Debug)]
pub struct Laid {
    pub from: P2,
    pub to: P2,
    /// Its thickness: measured (two faces), or the one named for walls seen from one side.
    pub thickness: f64,
    pub measured: bool,
    pub openings: Vec<Opening>,
    /// Which found wall (its place in `Found::walls`) it is.
    pub of: usize,
    /// How high, above the floor: the ceiling over it where the scan told it.
    pub height: Option<f64>,
}

/// What is laid and what is not, of a find. `outside` is the thickness walls seen from
/// one side are given (the type the person named for them); where it is None they are
/// not laid (their places in `Found::walls` come back second). A wall seen from one side
/// stands with its seen face on the scan's face and its thickness on the far side of it;
/// it is drawn with the room on its right so that its outside is to its left, as the
/// model draws its outsides. Each laid wall's end that comes within 1 ft of another's
/// line is carried to it, so that rooms close (call 173).
pub fn lay(found: &Found, outside: Option<f64>) -> (Vec<Laid>, Vec<usize>) {
    lay_with(found, outside, &[])
}

/// The same, with the walls the person has said are walls (`take`, places in `Found::walls`) though
/// the find doubted them (call 179, A3): a wall in doubt is laid only so.
pub fn lay_with(found: &Found, outside: Option<f64>, take: &[usize]) -> (Vec<Laid>, Vec<usize>) {
    let mut laid: Vec<Laid> = Vec::new();
    let mut not_laid: Vec<usize> = Vec::new();
    for (i, w) in found.walls.iter().enumerate() {
        if w.doubt.is_some() && !take.contains(&i) {
            not_laid.push(i);
            continue;
        }
        let (from, to, thickness, measured) = match (w.thickness, w.seen_from, outside) {
            (Some(t), _, _) => (w.from, w.to, t, true),
            (None, Some(inward), Some(t)) => {
                // The centre is half a thickness out from the seen face, away from the room.
                let shift = [-inward[0] * t / 2.0, -inward[1] * t / 2.0];
                let (a, b) = ([w.from[0] + shift[0], w.from[1] + shift[1]], [w.to[0] + shift[0], w.to[1] + shift[1]]);
                // The room on the right of the way it is drawn: the right-hand side of (dx, dy) is (dy, -dx).
                let right_of_a_to_b = (b[1] - a[1]) * inward[0] - (b[0] - a[0]) * inward[1];
                if right_of_a_to_b > 0.0 { (a, b, t, false) } else { (b, a, t, false) }
            }
            _ => {
                not_laid.push(i);
                continue;
            }
        };
        laid.push(Laid { from, to, thickness, measured, openings: Vec::new(), of: i, height: w.height });
    }
    // Ends carried to another's line.
    let lines: Vec<(P2, P2)> = laid.iter().map(|l| (l.from, l.to)).collect();
    for (k, l) in laid.iter_mut().enumerate() {
        for end in 0..2 {
            let (p, q) = if end == 0 { (l.from, l.to) } else { (l.to, l.from) };
            let d = unit(sub(p, q));
            let mut best: Option<(f64, P2)> = None;
            for (m, (a, b)) in lines.iter().enumerate() {
                if m == k {
                    continue;
                }
                let e = unit(sub(*b, *a));
                let cross = d[0] * e[1] - d[1] * e[0];
                if cross.abs() < NOT_PARALLEL {
                    continue;
                }
                // Where the way on from the end meets the other's line: p + t d = a + u e.
                let w = sub(*a, p);
                let t = (w[0] * e[1] - w[1] * e[0]) / cross;
                let u = (w[0] * d[1] - w[1] * d[0]) / cross;
                let length = (b[0] - a[0]).hypot(b[1] - a[1]);
                // The end is within CORNER of the other's line, and the meeting is on the
                // other wall (or within CORNER of its end).
                if (t * cross).abs() <= CORNER && u >= -CORNER && u <= length + CORNER && best.is_none_or(|x| t.abs() < x.0) {
                    best = Some((t.abs(), [p[0] + t * d[0], p[1] + t * d[1]]));
                }
            }
            if let Some((_, hit)) = best {
                if end == 0 {
                    l.from = hit;
                } else {
                    l.to = hit;
                }
            }
        }
    }
    // The openings, along the laid line from its first end: where each one's middle is on
    // the found line, projected onto it (a sideways shift of a wall seen from one side
    // does not change how far along it is).
    for l in laid.iter_mut() {
        let w = &found.walls[l.of];
        let d = unit(sub(w.to, w.from));
        let (e, length) = (unit(sub(l.to, l.from)), (l.to[0] - l.from[0]).hypot(l.to[1] - l.from[1]));
        for o in &w.openings {
            let mid_along = (o.from + o.to) / 2.0;
            let mid = [w.from[0] + d[0] * mid_along, w.from[1] + d[1] * mid_along];
            let centre = dot(sub(mid, l.from), e).clamp(0.0, length);
            let half = (o.to - o.from) / 2.0;
            l.openings.push(Opening { from: centre - half, to: centre + half, ..*o });
        }
    }
    one_type_of_openings(&mut laid);
    (laid, not_laid)
}

/// Openings that read within an inch of one another in width and in head (a window: in width and in height) are
/// one type, not one each for the inch of noise between them (call 179, A9): each takes the size of the first
/// of its kind it is within an inch of, about its own middle.
pub(super) fn one_type_of_openings(laid: &mut [Laid]) {
    let mut reps: Vec<(bool, f64, f64)> = Vec::new();
    for l in laid.iter_mut() {
        for o in l.openings.iter_mut() {
            let (width, tall) = (o.to - o.from, if o.window { o.head - o.sill } else { o.head });
            match reps.iter().find(|(w, rw, rt)| *w == o.window && (rw - width).abs() <= ONE_TYPE && (rt - tall).abs() <= ONE_TYPE) {
                Some((_, rw, rt)) => {
                    let mid = (o.from + o.to) / 2.0;
                    o.from = mid - rw / 2.0;
                    o.to = mid + rw / 2.0;
                    if o.window {
                        o.head = o.sill + rt;
                    } else {
                        o.head = *rt;
                    }
                }
                None => reps.push((o.window, width, tall)),
            }
        }
    }
}

pub(super) fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
pub(super) fn unit(a: P2) -> P2 {
    let l = a[0].hypot(a[1]).max(1e-12);
    [a[0] / l, a[1] / l]
}
