//! Walls, openings and what stands in front, found in a scan's points by the published method (DECISIONS.md call 189).
//!
//! **Where this comes from.** Cloud2BIM (V. Nezerka, "Cloud2BIM: an open-source automatic pipeline for efficient conversion of large-scale point clouds into
//! IFC format", arXiv 2503.11498; code https://github.com/VaclavNezerka/Cloud2BIM, MIT licence, "Copyright (c) 2025 Vaclav Nezerka": see NOTICE and LICENSES.md).
//! It finds walls where furniture does not reach: it keeps the points in the top tenth of a storey's height (`z_section_boundaries` [0.9, 1.0] in the
//! paper's table), lays them on the plan as a picture of where points are (`threshold` 0.01, a closing of `square(5)`), takes the outlines of what is
//! there (OpenCV `findContours`, which is Suzuki and Abe's border following of 1985), makes each outline straight (Douglas and Peucker, 1973;
//! `epsilon` 0.02 m), merges the pieces that lie on one line (`angle_tolerance` 3 degrees), pairs opposite parallel faces within a greatest
//! thickness into a wall (`group_segments`, `calculate_wall_axis`), takes the longest wall's direction as the axis and carries the ends to meet within
//! half the greatest thickness (`adjust_intersections`), and reads each wall's own face for openings where points are missing
//! (`identify_openings`). What is ported is that order of work and those measures, written here in Rust from the paper and the code read open (the code is
//! Python on OpenCV, NumPy and Open3D; none of it is linked and no line of it is copied). What is written here and is not theirs is named where it is:
//! the border following goes along the pixels' edges (the same borders as Suzuki and Abe find, the same two kinds, outer and hole); the density mask is
//! relative to what a wall of this scan would show, where theirs is "any point" (a tripod scanner's mm-fine, flat walls have no ghosts; a phone's have);
//! the thin outline of a wall seen from one side is told from the two faces of a wall seen from both (theirs invents a thickness for a lone face);
//! and everything call 173, 179, 183 and 188 settled about what is said and never done (Assumed, in doubt, laid only on the person's word, "not seen" is
//! not "open", what is left out said with where) is kept, on the same `Found` the first finder gives.
//!
//! The scan is the one `scan.rs` reads and levels: feet, east, north and up, level by its own floor, with the floor and the ceiling it works out.

use crate::found::{self, Found, Left, Opening, Stretch, Thing, Wall};
use crate::scan::Scan;
use std::collections::{HashMap, HashSet};

type P2 = [f64; 2];

// ---- measures, each with where it comes from ----

/// The band: the top tenth of the height (the paper's table: z_section_boundaries [0.9, 1.0]), down from the ceiling's own scatter set aside (`SET_ASIDE`).
const BAND_SHARE: f64 = 0.10;
/// The top tenth holds a tenth of a wall's points: a scan as thin as the made-up ones the first finder was tuned on (30 to the square foot) shows a wall's top at under 3
/// points to a pixel, and the picture breaks. The band is then taken deeper, to what gives a pixel this many points (the paper's scans have millions), and never
/// deeper than the top quarter (6 ft to 8 ft in an 8 ft room: furniture does not reach it).
const BAND_POINTS: f64 = 8.0;
const BAND_MOST: f64 = 0.25;
/// A pixel is "dense" where it holds at least this share of what a wall's face would show in it at the floor's own density (the paper's is any point,
/// threshold 0.01: for a scanner with no ghosts; this is call 188's own measure of what a face shows: a quarter of the floor's), and at least 2 points.
const MASK_SHARE: f64 = 0.25;
const MASK_LEAST: u32 = 2;
/// The paper closes the mask with a 5 pixel square at 1 cm a pixel, 5 cm: here one pixel either way (the pixel is no finer than 2 floor scatters).
const PIXEL_ROUGH: f64 = 2.0;
const PIXEL_MOST: f64 = 0.2;
/// Douglas and Peucker's tolerance: the paper's 0.02 m, and no less than a pixel (the outline is made of pixels' edges).
const EPS_PAPER: f64 = 0.02 / 0.3048;
/// A bowed wall is one wall: the tolerance of the straightening is no less than this share of the stretch's own length (call 188: a wall bows up to 1 in 20 to 1 in 27
/// on the real scans' measures; the paper's scans, from a tripod, are flat).
const BOW_RATIO: f64 = 1.0 / 20.0;
/// Pieces within this many degrees of one line are one line (the paper's angle_tolerance).
const ANGLE_TOL: f64 = 3.0;
/// A piece of an outline shorter than this is only a corner or a stub (the code's min_wall_length is 0.10 m; a stub of a corner and a bump are shorter than this on every
/// scan seen), and is not looked at; a wall is MIN_FACE (call 173) long at least.
const MIN_SEG: f64 = 0.5;
/// The least the points of a face are.
const FACE_POINTS: usize = 12;
/// Two faces are two (a wall of two faces), and not one rough face, only where the points across the gap are fewer in its middle third than this share of the outer thirds'.
const TROUGH: f64 = 0.5;
/// The scatter that sets two faces apart: they are at least this many scatters apart (a rough face's outline is as wide as it is rough).
const SEPARATE: f64 = 2.5;
/// Faces seen from one side only that lie within this far of one another and run side by side are one wall scanned in passes (call 188, 2: 16 in).
const PASS_APART: f64 = found::PAIR_MOST;
const PASS_DEGREES: f64 = 2.0;
/// The floor in front of a face tells which side the scan saw it from: of the 1 ft squares of floor within this far across, at least this many (call 188, same measures).
const SEEN_REACH: f64 = 6.0;
const SEEN_SHARE: f64 = 0.3;
/// A wall's foot is seen where its points reach down to within this far of the floor over at least this share of its length (call 173: half), counting what is hidden by something standing before it.
const FOOT_LOW: f64 = 1.0;
const FOOT_SHARE: f64 = 0.5;
/// The ceiling over a place, told again from the points over that place themselves (cells of 1 ft): the height where the most of them pile up in bins of
/// 0.05 ft, among those at three quarters of the house's height or more, where at least LOCAL_POINTS pile there. The first reading of the ceiling (`Ceil`) takes
/// differences under 3 in as one height, and a ceiling lower than it by a few inches over a part of the room puts its own points into the band that a wall's top is
/// looked for in: laid on the plan they are a patch, with edges that are not walls. Where the cell does not say, the first reading stands.
const LOCAL_CELL: f64 = 1.0;
const LOCAL_BIN: f64 = 0.05;
const LOCAL_POINTS: u32 = 8;
const LOCAL_FROM: f64 = 0.75;

/// A thing that stands is a 0.5 ft cell with at least this many points between 0.8 ft and 0.8 ft under the ceiling that no wall's face accounts for.
const STAND_CELL: f64 = 0.5;
const STAND_POINTS: u32 = 4;
/// A lot of standing cells is a thing where at least this many touch (a stray point or two in the air is not one).
const STAND_LOT: usize = 3;
const STAND_LOW: f64 = 0.8;
/// How far a wall leans, per foot of height, is told from the middles of its faces in slices of 1 ft, from at least 4 (call 188: 4 ft: LEAN_NEEDS), and no more than found.
const LEAN_SLICES: usize = 4;
const LEAN_WINDOW: f64 = 0.6;
const LEAN_AGREE: f64 = 0.1;
/// Pieces on one line are merged where their ends are within the greatest wall thickness of the code's config (max_wall_thickness 0.75 m): a wall's top is not
/// seen over a stretch where the scan missed it, and the wall is laid across the gap, which is said where it is (not seen, or a gap from floor to ceiling).
const MERGE_GAP: f64 = 0.75 / 0.3048;
/// A wall is a wall of 6 ft or more to be offered as likely (call 188, 1).
const MIN_FACE: f64 = found::MIN_FACE;

// ---- the density picture and its outlines ----

/// The picture of where points are: a pixel is occupied or not.
pub(crate) struct Mask {
    w: usize,
    h: usize,
    bits: Vec<bool>,
}

impl Mask {
    pub(crate) fn new(w: usize, h: usize) -> Mask {
        Mask { w, h, bits: vec![false; w * h] }
    }
    pub(crate) fn set(&mut self, c: usize, r: usize) {
        if c < self.w && r < self.h {
            self.bits[r * self.w + c] = true;
        }
    }
    pub(crate) fn get(&self, c: i64, r: i64) -> bool {
        c >= 0 && r >= 0 && (c as usize) < self.w && (r as usize) < self.h && self.bits[r as usize * self.w + c as usize]
    }
    /// Closing: one pixel out and then one pixel in (the paper's `closing` with a square kernel), so that a hole of a pixel or two in a wall's top is shut.
    pub(crate) fn closed(&self) -> Mask {
        let grown = |m: &Mask, want: bool| {
            let mut o = Mask::new(m.w, m.h);
            for r in 0..m.h as i64 {
                for c in 0..m.w as i64 {
                    let any = (-1..=1).any(|dr| (-1..=1).any(|dc| m.get(c + dc, r + dr) == want));
                    // out: occupied if any neighbour is; in: occupied only if no neighbour is empty (outside the picture counts as empty)
                    if (want && any) || (!want && !any) {
                        o.set(c as usize, r as usize);
                    }
                }
            }
            o
        };
        let out = grown(self, true);
        grown(&out, false)
    }

    /// The borders of the occupied parts, each a closed loop of corners (in pixel units, the north up), the occupied side kept on the left: an outer border
    /// goes counterclockwise, the border of a hole clockwise. Parts that touch at a corner are one part (eight neighbours join). Where Suzuki and Abe follow
    /// the pixels along their centres this follows their edges, and finds the same borders.
    pub(crate) fn borders(&self) -> Vec<Vec<P2>> {
        type V = (i64, i64);
        let mut out: HashMap<V, Vec<V>> = HashMap::new();
        let mut order: Vec<(V, V)> = Vec::new();
        for r in 0..self.h as i64 {
            for c in 0..self.w as i64 {
                if !self.get(c, r) {
                    continue;
                }
                let mut add = |a: V, b: V| {
                    out.entry(a).or_default().push(b);
                    order.push((a, b));
                };
                if !self.get(c, r - 1) {
                    add((c, r), (c + 1, r));
                }
                if !self.get(c + 1, r) {
                    add((c + 1, r), (c + 1, r + 1));
                }
                if !self.get(c, r + 1) {
                    add((c + 1, r + 1), (c, r + 1));
                }
                if !self.get(c - 1, r) {
                    add((c, r + 1), (c, r));
                }
            }
        }
        // The next edge from an edge: at a corner where two parts touch, the right turn (so that the two are one part).
        let next = |e: (V, V)| -> (V, V) {
            let cands = &out[&e.1];
            if cands.len() == 1 {
                return (e.1, cands[0]);
            }
            let h = (e.1 .0 - e.0 .0, e.1 .1 - e.0 .1);
            let pick = cands.iter().copied().find(|c| {
                let h2 = (c.0 - e.1 .0, c.1 - e.1 .1);
                h.0 * h2.1 - h.1 * h2.0 < 0
            });
            (e.1, pick.unwrap_or(cands[0]))
        };
        let mut seen: HashSet<(V, V)> = HashSet::new();
        let mut loops: Vec<Vec<P2>> = Vec::new();
        for start in order {
            if seen.contains(&start) {
                continue;
            }
            let mut verts: Vec<V> = vec![start.0];
            let mut cur = start;
            loop {
                seen.insert(cur);
                let n = next(cur);
                if n == start || seen.contains(&n) {
                    break;
                }
                verts.push(n.0);
                cur = n;
            }
            // Only the corners: a vertex between two steps the same way is dropped.
            let m = verts.len();
            let corners: Vec<P2> = (0..m)
                .filter(|i| {
                    let (p, q, s) = (verts[(i + m - 1) % m], verts[*i], verts[(i + 1) % m]);
                    (q.0 - p.0, q.1 - p.1) != (s.0 - q.0, s.1 - q.1)
                })
                .map(|i| [verts[i].0 as f64, verts[i].1 as f64])
                .collect();
            if corners.len() >= 3 {
                loops.push(corners);
            }
        }
        loops
    }
}

/// The area a closed loop encloses, positive where it goes counterclockwise.
#[cfg(test)]
pub(crate) fn area(l: &[P2]) -> f64 {
    let n = l.len();
    (0..n).map(|i| l[i][0] * l[(i + 1) % n][1] - l[(i + 1) % n][0] * l[i][1]).sum::<f64>() / 2.0
}

fn dist_seg(p: P2, a: P2, b: P2) -> f64 {
    found::distance_to(p, a, b)
}

/// Douglas and Peucker (1973) on an open run of points `i` to `j`: the point farthest from the chord is kept where it is farther than the tolerance, which is
/// `eps` or `ratio` of the chord's length, whichever is the greater (a gentle bow is one line).
fn dp(p: &[P2], i: usize, j: usize, eps: f64, ratio: f64, keep: &mut [bool]) {
    if j <= i + 1 {
        return;
    }
    let (a, b) = (p[i], p[j]);
    let tol = eps.max(ratio * (b[0] - a[0]).hypot(b[1] - a[1]));
    let (mut far, mut at) = (0.0, i);
    for k in i + 1..j {
        let d = dist_seg(p[k], a, b);
        if d > far {
            far = d;
            at = k;
        }
    }
    if far > tol {
        keep[at] = true;
        dp(p, i, at, eps, ratio, keep);
        dp(p, at, j, eps, ratio, keep);
    }
}

/// An open run made straight: its ends and the points that must stay.
pub(crate) fn straighten_open(p: &[P2], eps: f64, ratio: f64) -> Vec<P2> {
    if p.len() < 3 {
        return p.to_vec();
    }
    let mut keep = vec![false; p.len()];
    keep[0] = true;
    keep[p.len() - 1] = true;
    dp(p, 0, p.len() - 1, eps, ratio, &mut keep);
    p.iter().zip(keep).filter(|(_, k)| *k).map(|(q, _)| *q).collect()
}

/// A closed loop made straight: cut at its first point and the one farthest from it, each half made straight, the halves put together.
pub(crate) fn straighten(p: &[P2], eps: f64, ratio: f64) -> Vec<P2> {
    if p.len() < 4 {
        return p.to_vec();
    }
    let m = (1..p.len()).max_by(|a, b| dist_seg(p[*a], p[0], p[0]).total_cmp(&dist_seg(p[*b], p[0], p[0]))).unwrap_or(1);
    let first = straighten_open(&p[..=m], eps, ratio);
    let mut back: Vec<P2> = p[m..].to_vec();
    back.push(p[0]);
    let second = straighten_open(&back, eps, ratio);
    let mut out = first;
    out.extend_from_slice(&second[1..second.len() - 1]);
    out
}

// ---- lines of the plan ----

fn dot2(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn sub2(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn unit2(a: P2) -> P2 {
    let l = a[0].hypot(a[1]);
    if l < 1e-12 {
        [1.0, 0.0]
    } else {
        [a[0] / l, a[1] / l]
    }
}

/// A face as a line of the plan: its direction `d` (the occupied side, where it comes from an outline, is on its left), where it lies across (`rho`, along the
/// normal to the left), the stretch of it along `d`, how many points it was fitted to and how far they lie off it (a median deviation).
#[derive(Clone, Debug)]
struct Fc {
    d: P2,
    rho: f64,
    s0: f64,
    s1: f64,
    n: usize,
    spread: f64,
}

impl Fc {
    fn nrm(&self) -> P2 {
        [-self.d[1], self.d[0]]
    }
    fn at(&self, s: f64) -> P2 {
        let n = self.nrm();
        [s * self.d[0] + self.rho * n[0], s * self.d[1] + self.rho * n[1]]
    }
    fn len(&self) -> f64 {
        self.s1 - self.s0
    }
    fn mid(&self) -> P2 {
        self.at((self.s0 + self.s1) / 2.0)
    }
    fn across(&self, p: P2) -> f64 {
        dot2(p, self.nrm()) - self.rho
    }
    fn along(&self, p: P2) -> f64 {
        dot2(p, self.d)
    }
    fn from_ends(a: P2, b: P2) -> Fc {
        let d = unit2(sub2(b, a));
        let n = [-d[1], d[0]];
        Fc { d, rho: dot2(a, n), s0: dot2(a, d), s1: dot2(b, d), n: 0, spread: 0.0 }
    }
    /// The same line the other way round.
    fn reversed(&self) -> Fc {
        Fc { d: [-self.d[0], -self.d[1]], rho: -self.rho, s0: -self.s1, s1: -self.s0, ..self.clone() }
    }
    /// The same line, running the way `like` runs (within a quarter turn).
    fn like(&self, like: P2) -> Fc {
        if dot2(self.d, like) < 0.0 {
            self.reversed()
        } else {
            self.clone()
        }
    }
    /// The angle between two faces' lines, in degrees, 0 to 90 (a line is the same either way round).
    fn degrees(&self, o: &Fc) -> f64 {
        dot2(self.d, o.d).abs().clamp(0.0, 1.0).acos().to_degrees()
    }
}

/// The points of the band, in cells of a foot, for finding the ones near a line.
struct Cells {
    by: HashMap<(i64, i64), Vec<u32>>,
}

impl Cells {
    fn new(pts: &[[f64; 3]]) -> Cells {
        let mut by: HashMap<(i64, i64), Vec<u32>> = HashMap::new();
        for (i, p) in pts.iter().enumerate() {
            by.entry((p[0].floor() as i64, p[1].floor() as i64)).or_default().push(i as u32);
        }
        Cells { by }
    }
    /// Each point within `r` of the box of the stretch `a` to `b`.
    fn near(&self, a: P2, b: P2, r: f64, mut f: impl FnMut(usize)) {
        let (x0, x1) = ((a[0].min(b[0]) - r).floor() as i64, (a[0].max(b[0]) + r).floor() as i64);
        let (y0, y1) = ((a[1].min(b[1]) - r).floor() as i64, (a[1].max(b[1]) + r).floor() as i64);
        if (x1 - x0 + 1) * (y1 - y0 + 1) > 4_000_000 {
            return;
        }
        for cx in x0..=x1 {
            for cy in y0..=y1 {
                if let Some(v) = self.by.get(&(cx, cy)) {
                    for i in v {
                        f(*i as usize);
                    }
                }
            }
        }
    }
}

/// The points of `pts` within `tol` across `f` and along it from `grow` before its start to `grow` after its end, as plan places.
fn select(f: &Fc, pts: &[[f64; 3]], cells: &Cells, tol: f64, grow: f64) -> Vec<P2> {
    let mut sel: Vec<P2> = Vec::new();
    cells.near(f.at(f.s0 - grow), f.at(f.s1 + grow), tol + 0.1, |i| {
        let q = [pts[i][0], pts[i][1]];
        let s = f.along(q);
        if s >= f.s0 - grow && s <= f.s1 + grow && f.across(q).abs() <= tol {
            sel.push(q);
        }
    });
    sel
}

/// A face again from the points near it: the line that fits them best (total least squares: the direction of the points' spread, so that a line at any angle is
/// as well told as one along an axis), twice, the window following the line; None where too few points are near. Its ends are where the outline had them.
fn refit(f: &Fc, pts: &[[f64; 3]], cells: &Cells, tol: f64, grow: f64) -> Option<Fc> {
    let mut cur = f.clone();
    for _ in 0..2 {
        let sel = select(&cur, pts, cells, tol, grow);
        if sel.len() < FACE_POINTS {
            return None;
        }
        let n = sel.len() as f64;
        let (mx, my) = (sel.iter().map(|p| p[0]).sum::<f64>() / n, sel.iter().map(|p| p[1]).sum::<f64>() / n);
        let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
        for p in &sel {
            let (dx, dy) = (p[0] - mx, p[1] - my);
            sxx += dx * dx;
            sxy += dx * dy;
            syy += dy * dy;
        }
        let ang = 0.5 * (2.0 * sxy).atan2(sxx - syy);
        let mut d = [ang.cos(), ang.sin()];
        if dot2(d, cur.d) < 0.0 {
            d = [-d[0], -d[1]];
        }
        let nn = [-d[1], d[0]];
        let rho = dot2([mx, my], nn);
        let (pa, pb) = (cur.at(cur.s0), cur.at(cur.s1));
        let (a, b) = (dot2(pa, d), dot2(pb, d));
        let mut off: Vec<f64> = sel.iter().map(|p| (dot2(*p, nn) - rho).abs()).collect();
        off.sort_by(f64::total_cmp);
        cur = Fc { d, rho, s0: a.min(b), s1: a.max(b), n: sel.len(), spread: 1.4826 * off[off.len() / 2] };
    }
    Some(cur)
}

/// The middle of a list.
fn median(v: Vec<f64>) -> f64 {
    found::median(v)
}

/// The floor's 1 ft squares that hold floor points: where the scan saw the floor.
struct Floor {
    cells: Vec<P2>,
}

impl Floor {
    /// How many squares lie beyond `f` on one side (`side` +1 along its normal or -1), `from` to `from + SEEN_REACH` across, over the stretch `s0` to `s1`.
    fn beyond(&self, f: &Fc, s0: f64, s1: f64, side: f64, from: f64) -> usize {
        self.cells
            .iter()
            .filter(|c| {
                let (s, a) = (f.along(**c), f.across(**c) * side);
                s >= s0 && s <= s1 && a >= from && a <= from + SEEN_REACH
            })
            .count()
    }
}

/// How far a wall leans: the places across its line of the points in slices of a foot of height (each slice's fullest 0.05 ft, then the median of the points
/// near it), and the slope of those against height by the median of the slopes between pairs (Theil and Sen), where at least LEAN_SLICES slices agree within
/// LEAN_AGREE; (0, 0) where they do not (nothing is said of a lean that is not told). Returns the slope and where the wall is across at the floor. `pts` are (along, across, up).
fn lean_of(pts: &[[f64; 3]]) -> (f64, f64) {
    let mut slices: Vec<(f64, f64)> = Vec::new();
    let top = pts.iter().map(|p| p[2]).fold(0.0, f64::max);
    let mut z = 0.0;
    while z + 1.0 <= top + 0.5 {
        let here: Vec<f64> = pts.iter().filter(|p| p[2] >= z && p[2] < z + 1.0).map(|p| p[1]).collect();
        if here.len() >= 20 {
            let mut bins: HashMap<i64, usize> = HashMap::new();
            for a in &here {
                *bins.entry((a / 0.05).floor() as i64).or_default() += 1;
            }
            if let Some((b, _)) = bins.iter().max_by_key(|(k, v)| (**v, -**k)) {
                let c = (*b as f64 + 0.5) * 0.05;
                let near: Vec<f64> = here.iter().copied().filter(|a| (a - c).abs() <= 0.08).collect();
                slices.push((z + 0.5, median(near)));
            }
        }
        z += 1.0;
    }
    if slices.len() < LEAN_SLICES {
        return (0.0, 0.0);
    }
    let mut slopes: Vec<f64> = Vec::new();
    for i in 0..slices.len() {
        for j in i + 1..slices.len() {
            slopes.push((slices[j].1 - slices[i].1) / (slices[j].0 - slices[i].0));
        }
    }
    let b = median(slopes);
    let mid = median(slices.iter().map(|s| s.1 - b * s.0).collect());
    let agree = slices.iter().filter(|s| (s.1 - (mid + b * s.0)).abs() <= LEAN_AGREE).count();
    if agree >= LEAN_SLICES && b.abs() <= crate::found_188::TILT_MOST {
        (b, mid)
    } else {
        (0.0, 0.0)
    }
}

/// What a wall candidate is before it is a `Wall`: its line (a wall of two faces: the centre line) and, for two faces, the thickness.
#[derive(Clone, Debug)]
struct Cand {
    line: Fc,
    thickness: Option<f64>,
    /// How many outlines' faces were joined into it (passes of one wall).
    joined: usize,
}

/// How long two faces run side by side.
fn alongside(a: &Fc, b: &Fc) -> f64 {
    let (b0, b1) = (a.along(b.at(b.s0)), a.along(b.at(b.s1)));
    a.s1.min(b0.max(b1)) - a.s0.max(b0.min(b1))
}

// ---- the finder ----

/// The walls of one level of a scan, by the points near its ceiling (call 189). The same `Found` as the first finder's, and what is said of it is said the same
/// way: everything is a guess from points, Assumed; a wall seen from one side has no thickness made up; what is in doubt is laid only on the person's word;
/// "not seen" is not "open"; what is left out is said with where.
pub(crate) fn find_core(scan: &Scan) -> Found {
    let mut out = Found::default();
    let (floor, scan_ceiling) = scan.floor_and_ceiling();
    out.floor = floor;
    out.ceiling = scan_ceiling;
    let Some(floor) = floor else {
        out.not_likely = found::unlikely(scan, None);
        out.left_out.push("no floor shows in the scan, so what stands on it cannot be told".into());
        return out;
    };
    // How thick the scan is, and what depends on it (call 179, A1).
    let rho = found::density(scan, floor);
    out.density = rho;
    if let Some(r) = rho.filter(|r| *r < found::LEAST_DENSITY) {
        out.not_likely = found::unlikely(scan, scan_ceiling.map(|c| (c - floor, c - floor)));
        out.too_thin = Some(format!("the scan is too thin to tell a wall from furniture: about {} to the square foot, and it takes {} or more", found::figure(r), found::figure(found::LEAST_DENSITY)));
        out.left_out.push(out.too_thin.clone().unwrap_or_default());
        return out;
    }
    let k = rho.map_or(1.0, |r| (found::REFERENCE_DENSITY / r).sqrt().max(1.0));
    let sq = found::SQUARE * k;
    if rho.is_some_and(|r| r < found::THIN_NOTE_BELOW * found::REFERENCE_DENSITY) {
        out.thin_note = Some(format!("this scan is thinner than the measures here were made for ({} to the square foot, against {} and more): a wall is told to a few inches, an opening only to about {} in, and one may be missed or split: check each", found::figure(rho.unwrap_or(0.0)), found::figure(found::REFERENCE_DENSITY), (sq * 12.0).round()));
    }
    // The ceiling over the plan: one height, two, or a slope (call 179, A4).
    let (ceil, ceil_note) = found::ceiling_of(scan, floor, scan_ceiling, rho, found::CEIL_CELL * k);
    out.ceiling_note = ceil_note;
    out.not_likely = found::unlikely(scan, ceil.as_ref().map(|c| (c.lowest, c.highest)));
    let Some(ceil) = ceil else {
        out.no_ceiling = true;
        out.left_out.push("no ceiling shows in the scan: without one, a wall cannot be told from a tall cabinet standing in the room, so no wall is taken".into());
        return out;
    };
    if scan_ceiling.is_none() || ceil.flat.is_none() {
        out.ceiling = Some(floor + ceil.middle);
    }
    let high = ceil.highest - found::SET_ASIDE;
    let big = scan.points.len() >= crate::found_188::PROGRESS_FROM;

    // ---- 1. the points that are near the ceiling, where furniture does not reach (per place where the ceiling is not one height) ----
    let mut inner: Vec<[f64; 3]> = Vec::new();
    let mut band: Vec<[f64; 3]> = Vec::new();
    let rough = crate::found_188::floor_scatter(&scan.points, floor, 0.25);
    let px0 = (found::CELL * k).max(PIXEL_ROUGH * rough.unwrap_or(0.0)).min(PIXEL_MOST.max(found::CELL * k));
    let share = (BAND_POINTS / (rho.unwrap_or(found::REFERENCE_DENSITY) * px0 * ceil.middle)).clamp(BAND_SHARE, BAND_MOST);
    if share > BAND_SHARE + 1e-9 {
        let note = format!("the points near the ceiling are taken from the top {} in 100 of the height (not the top tenth), for the scan is thin", (share * 100.0).round());
        out.thin_note = Some(match out.thin_note.take() {
            Some(t) => format!("{t}; {note}"),
            None => note,
        });
    }
    // the ceiling over each foot square, told from its own points
    let local: HashMap<(i64, i64), f64> = {
        let mut piles: HashMap<(i64, i64, i64), u32> = HashMap::new();
        for p in &scan.points {
            let z = p[2] - floor;
            if z >= LOCAL_FROM * ceil.lowest && z <= ceil.highest + 1.0 {
                *piles.entry(((p[0] / LOCAL_CELL).floor() as i64, (p[1] / LOCAL_CELL).floor() as i64, (z / LOCAL_BIN).floor() as i64)).or_default() += 1;
            }
        }
        let mut best: HashMap<(i64, i64), (u32, i64)> = HashMap::new();
        for ((a, b, z), n) in piles {
            let e = best.entry((a, b)).or_insert((0, 0));
            if n > e.0 || (n == e.0 && z > e.1) {
                *e = (n, z);
            }
        }
        best.into_iter().filter(|(_, (n, _))| *n >= LOCAL_POINTS).map(|(k, (_, z))| (k, (z as f64 + 0.5) * LOCAL_BIN)).collect()
    };
    let band_top = |x: f64, y: f64, z_ceil: f64| -> f64 {
        local.get(&((x / LOCAL_CELL).floor() as i64, (y / LOCAL_CELL).floor() as i64)).copied().filter(|l| (l - z_ceil).abs() <= 1.0).unwrap_or(z_ceil)
    };
    for p in &scan.points {
        let z = p[2] - floor;
        if z < found::SET_ASIDE {
            continue;
        }
        let h = ceil.at([p[0], p[1]]);
        if z > h - found::SET_ASIDE {
            continue;
        }
        inner.push([p[0], p[1], z]);
        let hl = band_top(p[0], p[1], h);
        if z >= hl * (1.0 - share) && z <= hl - found::SET_ASIDE {
            band.push([p[0], p[1], z]);
        }
    }
    if band.len() < FACE_POINTS * 4 {
        out.left_out.push("too few points near the ceiling to find a wall by: no wall is taken".into());
        return out;
    }
    let band_high = ceil.middle * share - found::SET_ASIDE;

    // ---- 2. laid on the plan as how dense they are ----
    let mut px = px0;
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in &band {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    while ((hi[0] - lo[0]) / px + 8.0) * ((hi[1] - lo[1]) / px + 8.0) > 20_000_000.0 {
        px *= 1.5;
    }
    let (w, h) = (((hi[0] - lo[0]) / px).ceil() as usize + 8, ((hi[1] - lo[1]) / px).ceil() as usize + 8);
    let origin = [lo[0] - 4.0 * px, lo[1] - 4.0 * px];
    let mut counts = vec![0u32; w * h];
    for p in &band {
        let (c, r) = (((p[0] - origin[0]) / px) as usize, ((p[1] - origin[1]) / px) as usize);
        counts[r.min(h - 1) * w + c.min(w - 1)] += 1;
    }
    let expect = band_high.max(0.1) * rho.unwrap_or(found::REFERENCE_DENSITY) * px;
    let least = ((MASK_SHARE * expect).ceil() as u32).max(MASK_LEAST);
    let mut mask = Mask::new(w, h);
    for r in 0..h {
        for c in 0..w {
            if counts[r * w + c] >= least {
                mask.set(c, r);
            }
        }
    }
    drop(counts);
    let mask = mask.closed();
    if big {
        crate::found_188::progress("the points near the ceiling are laid on the plan; taking their outlines");
    }

    // ---- 3. the outlines of the dense parts, made straight ----
    let eps = EPS_PAPER.max(px);
    let to_feet = |v: P2| [origin[0] + v[0] * px, origin[1] + v[1] * px];
    let mut segs: Vec<Fc> = Vec::new();
    for l in mask.borders() {
        let feet: Vec<P2> = l.iter().map(|v| to_feet(*v)).collect();
        let poly = straighten(&feet, eps, BOW_RATIO);
        let m = poly.len();
        for i in 0..m {
            let (a, b) = (poly[i], poly[(i + 1) % m]);
            if (b[0] - a[0]).hypot(b[1] - a[1]) >= MIN_SEG {
                segs.push(Fc::from_ends(a, b));
            }
        }
    }
    let cells = Cells::new(&band);
    // ---- the points' own roughness, and the band their faces lie in (call 188, 1) ----
    let tol1 = found::BAND * crate::found_188::band_scale(rough);
    let mut fs: Vec<Fc> = segs.iter().filter_map(|s| refit(s, &band, &cells, tol1, px)).collect();
    let wall_sigma = {
        let sp: Vec<f64> = fs.iter().filter(|f| f.len() >= 3.0).map(|f| f.spread).collect();
        (sp.len() >= 2).then(|| median(sp))
    };
    let scatter = match (rough, wall_sigma) {
        (Some(a), Some(b)) => a.max(b),
        (a, b) => a.or(b).unwrap_or(0.0),
    };
    let tol = found::BAND * crate::found_188::band_scale(Some(scatter));
    if tol > tol1 + 1e-9 {
        fs = segs.iter().filter_map(|s| refit(s, &band, &cells, tol, px)).collect();
        out.scatter = Some(scatter);
    }
    let sep = found::PAIR_LEAST.max(SEPARATE * scatter);

    // ---- 4. pieces on one line are one line (the paper's angle tolerance 3 degrees; ends within a gap of 1 ft 6 in) ----
    loop {
        let mut joined: Option<(usize, usize)> = None;
        'find: for i in 0..fs.len() {
            for j in i + 1..fs.len() {
                if fs[i].degrees(&fs[j]) <= ANGLE_TOL && dot2(fs[i].d, fs[j].d) > 0.0 && fs[i].across(fs[j].mid()).abs() <= tol / 2.0 && fs[j].across(fs[i].mid()).abs() <= tol / 2.0 {
                    let (b0, b1) = (fs[i].along(fs[j].at(fs[j].s0)), fs[i].along(fs[j].at(fs[j].s1)));
                    let gap = fs[i].s0.max(b0.min(b1)) - fs[i].s1.min(b0.max(b1));
                    if gap <= MERGE_GAP {
                        joined = Some((i, j));
                        break 'find;
                    }
                }
            }
        }
        let Some((i, j)) = joined else { break };
        let (a, b) = (fs[i].clone(), fs[j].clone());
        let (b0, b1) = (a.along(b.at(b.s0)), a.along(b.at(b.s1)));
        let rho_b = dot2(b.mid(), a.nrm());
        let wa = a.len();
        let wb = b.len();
        let c = Fc { d: a.d, rho: (a.rho * wa + rho_b * wb) / (wa + wb), s0: a.s0.min(b0.min(b1)), s1: a.s1.max(b0.max(b1)), n: a.n + b.n, spread: a.spread.max(b.spread) };
        let merged = refit(&c, &band, &cells, tol, px).unwrap_or(c);
        fs.remove(j);
        fs[i] = merged;
    }

    // ---- 5. where the floor was seen (a foot square), and which side of a face the scan saw it from ----
    let mut floor_counts: HashMap<(i64, i64), u32> = HashMap::new();
    for p in &scan.points {
        if (p[2] - floor).abs() <= found::SET_ASIDE {
            *floor_counts.entry(((p[0] / found::FLOOR_CELL).floor() as i64, (p[1] / found::FLOOR_CELL).floor() as i64)).or_default() += 1;
        }
    }
    let floor_dense: Vec<P2> = {
        let m = median(floor_counts.values().map(|n| f64::from(*n)).collect());
        floor_counts.iter().filter(|(_, n)| f64::from(**n) >= (0.3 * m).max(3.0)).map(|((a, b), _)| [(*a as f64 + 0.5) * found::FLOOR_CELL, (*b as f64 + 0.5) * found::FLOOR_CELL]).collect()
    };
    let floor_cells = Floor { cells: floor_counts.into_iter().filter(|(_, n)| *n >= 3).map(|((a, b), _)| [(*&a as f64 + 0.5) * found::FLOOR_CELL, (*&b as f64 + 0.5) * found::FLOOR_CELL]).collect() };
    let centroid = {
        let n = floor_cells.cells.len().max(1) as f64;
        [floor_cells.cells.iter().map(|c| c[0]).sum::<f64>() / n, floor_cells.cells.iter().map(|c| c[1]).sum::<f64>() / n]
    };
    // The side a face was seen from: +1 along its normal or -1: where more floor lies in front of it (call 188), else toward the middle of the floor seen.
    let seen_side = |f: &Fc, s0: f64, s1: f64, from: f64| -> f64 {
        let (p, m) = (floor_cells.beyond(f, s0, s1, 1.0, from), floor_cells.beyond(f, s0, s1, -1.0, from));
        if p != m {
            if p > m { 1.0 } else { -1.0 }
        } else if f.across(centroid) >= 0.0 {
            1.0
        } else {
            -1.0
        }
    };

    // ---- 6. opposite faces paired into walls; one face the two sides of a thin outline; a lone face a wall seen from one side ----
    let floor_pts: Vec<P2> = scan.points.iter().filter(|p| (p[2] - floor).abs() <= found::SET_ASIDE).map(|p| [p[0], p[1]]).collect();
    let bimodal = |a: &Fc, t: f64, lo: f64, hi: f64| -> bool {
        let (mut na, mut nm, mut nb) = (0usize, 0usize, 0usize);
        for q in select(&Fc { s0: lo, s1: hi, ..a.clone() }, &band, &cells, t + 0.15, 0.0) {
            let c = a.across(q);
            if c < -0.1 || c > t + 0.1 {
                continue;
            }
            if c < t / 3.0 {
                na += 1;
            } else if c < 2.0 * t / 3.0 {
                nm += 1;
            } else {
                nb += 1;
            }
        }
        na.min(nb) >= 10 && (nm as f64) <= TROUGH * na.min(nb) as f64
    };
    let collapse = |a: &Fc, b: &Fc| -> Fc {
        let rho_b = dot2(b.mid(), a.nrm());
        let (b0, b1) = (a.along(b.at(b.s0)), a.along(b.at(b.s1)));
        let c = Fc { d: a.d, rho: (a.rho + rho_b) / 2.0, s0: a.s0.min(b0.min(b1)), s1: a.s1.max(b0.max(b1)), n: a.n + b.n, spread: a.spread.max(b.spread) };
        refit(&c, &band, &cells, (rho_b - a.rho).abs() / 2.0 + tol, px).unwrap_or(c)
    };
    // Pass 1: the two sides of one thin outline (and passes a centimeter or two apart) are one face: parallel within the paper's 3 degrees, nearer than the
    // scatter sets two faces apart, side by side.
    let mut next_id = 0usize;
    let mut faces: Vec<(Fc, usize, usize)> = fs
        .into_iter()
        .map(|f| {
            next_id += 1;
            (f, 0usize, next_id)
        })
        .collect();
    let side_by_side = |a: &Fc, b: &Fc| -> Option<(f64, f64)> {
        if a.degrees(b) > ANGLE_TOL {
            return None;
        }
        let ov = alongside(a, &b.like(a.d));
        if ov < MIN_FACE.min(0.5 * a.len().min(b.len())) {
            return None;
        }
        Some(((a.across(b.mid()).abs() + b.across(a.mid()).abs()) / 2.0, ov))
    };
    loop {
        let mut best: Option<(usize, usize, f64)> = None;
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                if let Some((t, ov)) = side_by_side(&faces[i].0, &faces[j].0) {
                    if t < sep && best.is_none_or(|x| ov > x.2) {
                        best = Some((i, j, ov));
                    }
                }
            }
        }
        let Some((i, j, _)) = best else { break };
        let b = faces[j].0.like(faces[i].0.d);
        let c = collapse(&faces[i].0, &b);
        let joined = faces[i].1 + faces[j].1;
        faces[i].0 = c;
        faces[i].1 = joined;
        faces.remove(j);
    }
    // Pass 2: faces farther apart than that, nearer than 2 ft, side by side: two faces of one wall where the points across them are two layers with little
    // between and the scan saw floor on both outer sides; one rough face where they are not two layers; one wall in passes where the floor was seen on one
    // side only. (The paper pairs any parallel faces within its greatest thickness; a lone face's thickness is made up there, and is not here.)
    let mut rejected: HashSet<(usize, usize)> = HashSet::new();
    let mut cands: Vec<Cand> = Vec::new();
    loop {
        let mut best: Option<(usize, usize, f64, f64)> = None;
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                if rejected.contains(&(faces[i].2, faces[j].2)) {
                    continue;
                }
                if let Some((t, ov)) = side_by_side(&faces[i].0, &faces[j].0) {
                    if t <= found::PAIR_MOST_BARE && best.is_none_or(|x| ov > x.3) {
                        best = Some((i, j, t, ov));
                    }
                }
            }
        }
        let Some((i, j, t, _)) = best else { break };
        let bj = faces[j].0.like(faces[i].0.d);
        let (a, b) = if faces[i].0.across(bj.mid()) >= 0.0 { (faces[i].0.clone(), bj) } else { (bj, faces[i].0.clone()) };
        let t = a.across(b.mid()).abs().max(t);
        let (b0, b1) = (a.along(b.at(b.s0)), a.along(b.at(b.s1)));
        let (lo_s, hi_s) = (a.s0.max(b0.min(b1)), a.s1.min(b0.max(b1)));
        let two_faces = bimodal(&a, t, lo_s, hi_s);
        let behind = floor_cells.beyond(&a, lo_s, hi_s, -1.0, 0.3);
        let front = floor_cells.beyond(&a, lo_s, hi_s, 1.0, t + 0.3);
        // floor on a side: at least this many of the 1 ft squares there are floor (a third of the squares of the strip, and three), not strays' few
        let need = ((SEEN_SHARE * (hi_s - lo_s) * SEEN_REACH).ceil() as usize).max(3);
        if !two_faces || behind < need || front < need {
            let c = collapse(&a, &b);
            let joined = faces[i].1 + faces[j].1 + usize::from(two_faces);
            faces[i].0 = c;
            faces[i].1 = joined;
            faces.remove(j);
            continue;
        }
        if t > found::PAIR_MOST {
            // Over 16 in: a wall only where no floor lies between its faces (call 179, A6).
            let between = floor_pts
                .iter()
                .filter(|q| {
                    let (s, c) = (a.along(**q), a.across(**q));
                    s >= lo_s && s <= hi_s && c > 0.1 && c < t - 0.1
                })
                .count() as f64;
            let expected = (t - 0.2) * (hi_s - lo_s) * rho.unwrap_or(found::REFERENCE_DENSITY);
            if between > found::FLOOR_BETWEEN.max(0.25) * expected {
                rejected.insert((faces[i].2, faces[j].2));
                continue;
            }
        }
        let rho_b = dot2(b.mid(), a.nrm());
        let c = Fc { d: a.d, rho: (a.rho + rho_b) / 2.0, s0: (a.s0 + b0.min(b1)) / 2.0, s1: (a.s1 + b0.max(b1)) / 2.0, n: a.n + b.n, spread: a.spread.max(b.spread) };
        cands.push(Cand { line: c, thickness: Some(t), joined: 0 });
        faces.remove(j);
        faces.remove(i);
    }
    for (f, joined, _) in faces {
        cands.push(Cand { line: f, thickness: None, joined });
    }
    // ---- one wall seen from one side twice, in passes or in pieces the pairing left, is one wall (call 188, 2) ----
    loop {
        let mut again: Option<(usize, usize)> = None;
        'p: for i in 0..cands.len() {
            for j in i + 1..cands.len() {
                let (a, b) = (&cands[i], &cands[j]);
                if a.thickness.is_some() || b.thickness.is_some() || a.line.degrees(&b.line) > PASS_DEGREES {
                    continue;
                }
                let t = a.line.across(b.line.mid()).abs();
                let ov = alongside(&a.line, &b.line.like(a.line.d));
                let (la, lb) = (a.line.len(), b.line.len());
                if t <= PASS_APART && ov >= 0.5 * la.min(lb) && seen_side(&a.line, a.line.s0, a.line.s1, 0.5) == seen_side(&b.line, b.line.s0, b.line.s1, 0.5) {
                    again = Some((i, j));
                    break 'p;
                }
            }
        }
        let Some((i, j)) = again else { break };
        let (a, b) = (cands[i].clone(), cands[j].clone());
        let bl = b.line.like(a.line.d);
        let c = collapse(&a.line, &bl);
        cands[i] = Cand { line: c, thickness: None, joined: a.joined + b.joined + 1 };
        cands.remove(j);
    }
    // What is shorter than a wall is not one, and is said where it is longer than a foot and no wall is near.
    let mut left: Vec<Left> = Vec::new();
    let (keep, short): (Vec<Cand>, Vec<Cand>) = cands.into_iter().partition(|c| c.line.len() >= MIN_FACE);
    let mut cands = keep;
    for s in short {
        if s.line.len() >= 1.0 {
            left.push(Left { from: s.line.at(s.line.s0), to: s.line.at(s.line.s1), why: format!("under {} ft long (a wall is at least {} ft long)", MIN_FACE, MIN_FACE) });
        }
    }
    if cands.is_empty() {
        out.left_out.push("no outline near the ceiling is a wall: nothing straight of 2 ft or more is there".into());
        for l in &left {
            out.left_out.push(format!("a stretch of upright points {} long, from {} east, {} north to {} east, {} north, not taken as walls: {}", found::plumbline_units((l.to[0] - l.from[0]).hypot(l.to[1] - l.from[1])), found::plumbline_units(l.from[0]), found::plumbline_units(l.from[1]), found::plumbline_units(l.to[0]), found::plumbline_units(l.to[1]), l.why));
        }
        out.left = left;
        return out;
    }
    if big {
        crate::found_188::progress(&format!("{} walls found by their tops; reading each wall's face", cands.len()));
    }

    // ---- 7. the longest wall sets the axis; walls within a degree of it or its square are made square to it (the paper; call 173) ----
    {
        let longest = cands.iter().max_by(|a, b| a.line.len().total_cmp(&b.line.len())).map(|c| c.line.d);
        if let Some(d) = longest {
            let main = d[1].atan2(d[0]).to_degrees().rem_euclid(90.0);
            out.main = Some(main);
            let mut squared = 0;
            for c in cands.iter_mut() {
                let ang = c.line.d[1].atan2(c.line.d[0]).to_degrees();
                let off = (ang - main).rem_euclid(90.0);
                let off = if off > 45.0 { off - 90.0 } else { off };
                if off.abs() <= found::SQUARE_DEGREES {
                    squared += 1;
                    if off.abs() > 1e-9 {
                        let mid = c.line.mid();
                        let (h2, nd) = (c.line.len() / 2.0, (ang - off).to_radians());
                        let nd = [nd.cos(), nd.sin()];
                        let nn = [-nd[1], nd[0]];
                        let rho = dot2(mid, nn);
                        let s = dot2(mid, nd);
                        c.line = Fc { d: nd, rho, s0: s - h2, s1: s + h2, ..c.line.clone() };
                    }
                }
            }
            out.squared = (squared, cands.len());
        }
    }

    // ---- 8. each wall's lean (told from its points the whole height), and its place at the floor ----
    struct Built {
        line: Fc,
        thickness: Option<f64>,
        joined: usize,
        /// (along, up) of the points of its face, the whole height.
        wp: Vec<[f64; 2]>,
        side: f64,
        /// Where its line lay across at the band near the ceiling, before it was moved to the floor.
        top_rho: f64,
    }
    let cells_all = Cells::new(&inner);
    let mut built: Vec<Built> = Vec::new();
    for c in &cands {
        let half = c.thickness.map_or(0.0, |t| t / 2.0);
        let near: Vec<[f64; 3]> = {
            let mut v = Vec::new();
            let l = &c.line;
            cells_all.near(l.at(l.s0 - 0.3), l.at(l.s1 + 0.3), half + LEAN_WINDOW + 0.2, |i| {
                let q = [inner[i][0], inner[i][1]];
                let (s, a) = (l.along(q), l.across(q));
                if s >= l.s0 - 0.3 && s <= l.s1 + 0.3 && a.abs() <= half + LEAN_WINDOW {
                    v.push([s, a, inner[i][2]]);
                }
            });
            v
        };
        let (lean, shift) = if c.thickness.is_none() && c.line.len() >= found::MIN_FACE { lean_of(&near) } else { (0.0, 0.0) };
        let mut line = c.line.clone();
        line.rho += shift;
        let wp: Vec<[f64; 2]> = near.iter().filter(|p| (p[1] - (shift + lean * p[2])).abs() <= half + tol).map(|p| [p[0], p[2]]).collect();
        let side = if c.thickness.is_some() { 1.0 } else { seen_side(&line, line.s0, line.s1, 0.5) };
        built.push(Built { line, thickness: c.thickness, joined: c.joined, wp, side, top_rho: c.line.rho });
    }

    // ---- 9. walls carried to meet: an end within a foot (half the greatest thickness, the paper's snap) of where another wall's line crosses its own ----
    {
        let lines: Vec<(Fc, Option<f64>)> = built.iter().map(|b| (b.line.clone(), b.thickness)).collect();
        for (i, b) in built.iter_mut().enumerate() {
            for end in 0..2 {
                let at = if end == 0 { b.line.s0 } else { b.line.s1 };
                let mut best: Option<(f64, f64)> = None;
                for (j, (o, ot)) in lines.iter().enumerate() {
                    if i == j || b.line.degrees(o) < 20.0 {
                        continue;
                    }
                    // where the two lines cross
                    let det = b.line.d[0] * o.d[1] - b.line.d[1] * o.d[0];
                    if det.abs() < 1e-9 {
                        continue;
                    }
                    let q = cross_point(&b.line, o);
                    let along_o = o.along(q);
                    if along_o < o.s0 - found::CORNER || along_o > o.s1 + found::CORNER {
                        continue;
                    }
                    let s_cross = b.line.along(q);
                    // a lone face ends at the near face of a wall of two faces it meets, not at its centre
                    let pull = match ot {
                        Some(t) if b.thickness.is_none() => (t / 2.0) / b.line.degrees(o).to_radians().sin().max(0.3),
                        _ => 0.0,
                    };
                    let s_end = if end == 0 { s_cross + pull } else { s_cross - pull };
                    if (s_cross - at).abs() <= found::CORNER && best.is_none_or(|(d, _)| (s_cross - at).abs() < d) {
                        best = Some(((s_cross - at).abs(), s_end));
                    }
                }
                if let Some((_, s_end)) = best {
                    if end == 0 {
                        b.line.s0 = s_end;
                    } else {
                        b.line.s1 = s_end;
                    }
                }
            }
        }
    }

    // ---- 10. what stands in the room: cells of 0.5 ft with points between 0.8 ft up and 0.8 ft under the ceiling that no wall's face accounts for ----
    let mut stand: HashMap<(i64, i64), (u32, f64)> = HashMap::new();
    for p in &inner {
        let h = ceil.at([p[0], p[1]]);
        if p[2] < STAND_LOW || p[2] > h - STAND_LOW {
            continue;
        }
        let q = [p[0], p[1]];
        let on_a_wall = built.iter().any(|b| {
            let l = &b.line;
            let s = l.along(q);
            s >= l.s0 - 0.5 && s <= l.s1 + 0.5 && l.across(q).abs() <= b.thickness.map_or(0.0, |t| t / 2.0) + tol + 0.3 + 0.04 * p[2]
        });
        if on_a_wall {
            continue;
        }
        let e = stand.entry(((q[0] / STAND_CELL).floor() as i64, (q[1] / STAND_CELL).floor() as i64)).or_insert((0, 0.0));
        e.0 += 1;
        e.1 = e.1.max(p[2]);
    }
    let stand_cells: Vec<((i64, i64), f64)> = stand.iter().filter(|(_, (n, _))| *n >= STAND_POINTS).map(|(k, (_, z))| (*k, *z)).collect();
    let mut stand_at: Vec<P2> = Vec::new();
    {
        // Things: cells that touch (eight neighbours), each lot a thing, as tall as its tallest point.
        let set: HashMap<(i64, i64), f64> = stand_cells.iter().cloned().collect();
        let mut seen: HashSet<(i64, i64)> = HashSet::new();
        let mut keys: Vec<(i64, i64)> = set.keys().copied().collect();
        keys.sort();
        for k0 in keys {
            if seen.contains(&k0) {
                continue;
            }
            let (mut stack, mut lot) = (vec![k0], Vec::new());
            seen.insert(k0);
            while let Some(k) = stack.pop() {
                lot.push(k);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let n = (k.0 + dx, k.1 + dy);
                        if set.contains_key(&n) && seen.insert(n) {
                            stack.push(n);
                        }
                    }
                }
            }
            if lot.len() >= STAND_LOT {
                stand_at.extend(lot.iter().map(|k| [(k.0 as f64 + 0.5) * STAND_CELL, (k.1 as f64 + 0.5) * STAND_CELL]));
                let top = lot.iter().map(|k| set[k]).fold(0.0, f64::max);
                let n = lot.len() as f64;
                out.things.push(Thing { height: top, at: [lot.iter().map(|k| (k.0 as f64 + 0.5) * STAND_CELL).sum::<f64>() / n, lot.iter().map(|k| (k.1 as f64 + 0.5) * STAND_CELL).sum::<f64>() / n] });
            }
        }
    }

    // ---- 11. each wall's own face read: where its foot is seen, its bow, where points are missing from floor to ceiling, and what stands in front ----
    let mut walls: Vec<(Wall, WallKeep)> = Vec::new();
    struct WallKeep {
        line: Fc,
    }
    let mut demoted: Vec<String> = Vec::new();
    for b in built.iter_mut() {
        let line = b.line.clone();
        let half = b.thickness.map_or(0.0, |t| t / 2.0);
        let n = line.nrm();
        let length = line.len();
        let (pa, pb) = (line.at(line.s0), line.at(line.s1));
        let flip = if (pb[0] - pa[0]).abs() >= (pb[1] - pa[1]).abs() { pb[0] < pa[0] } else { pb[1] < pa[1] };
        let (from, to) = if flip { (pb, pa) } else { (pa, pb) };
        let seen_from = b.thickness.is_none().then(|| [n[0] * b.side, n[1] * b.side]);
        let mut w = Wall { from, to, thickness: b.thickness, seen_from, joined: b.joined, ..Wall::default() };
        w.height = Some(ceil.over(line.at((line.s0 + line.s1) / 2.0), n));
        // the bow (a lone face of 6 ft or more): the points' offset across the line fitted to a parabola over its length, c + k u^2 with u from -1 to 1;
        // the depth, middle against ends, is |k| (called a bow where an inch or more, and no more than half a foot)
        if b.thickness.is_none() && length >= crate::found_188::BOW_LEAST_LEN {
            let top = Fc { rho: b.top_rho, ..line.clone() };
            let uy: Vec<(f64, f64)> = select(&top, &band, &cells, tol, 0.0).into_iter().map(|q| (2.0 * (top.along(q) - top.s0) / length - 1.0, top.across(q))).collect();
            if uy.len() >= 30 {
                let m = uy.len() as f64;
                let (mx, my) = (uy.iter().map(|p| p.0 * p.0).sum::<f64>() / m, uy.iter().map(|p| p.1).sum::<f64>() / m);
                let (sxy, sxx) = uy.iter().fold((0.0, 0.0), |(a, b), p| (a + (p.0 * p.0 - mx) * (p.1 - my), b + (p.0 * p.0 - mx).powi(2)));
                let depth = if sxx > 1e-9 { (sxy / sxx).abs() } else { 0.0 };
                if depth >= crate::found_188::BOW_SAY && depth <= crate::found_188::BOW_MOST {
                    w.bow = Some((depth, length));
                }
            }
        }
        // the foot: columns of a quarter foot along the wall with points down to a foot of the floor, or hidden by something standing before them
        let cols = ((length / 0.25).ceil() as usize).max(1);
        let mut low = vec![0usize; cols];
        for p in &b.wp {
            if p[1] <= FOOT_LOW {
                let c = (((p[0] - line.s0) / 0.25).floor() as i64).clamp(0, cols as i64 - 1) as usize;
                low[c] += 1;
            }
        }
        let blocked_at = |s: f64| -> bool {
            stand_at.iter().any(|q| {
                let (sq_, a) = (line.along(*q), line.across(*q));
                let side_ok = if b.thickness.is_some() { true } else { a * b.side > half + found::IN_FRONT_LEAST && a * b.side <= half + found::IN_FRONT };
                let side_ok = side_ok && (b.thickness.is_none() || (a.abs() > half + found::IN_FRONT_LEAST && a.abs() <= half + found::IN_FRONT));
                (sq_ - s).abs() <= 0.25 && side_ok
            })
        };
        let reach = (0..cols).filter(|c| low[*c] >= 2 || blocked_at(line.s0 + (*c as f64 + 0.5) * 0.25)).count();
        let down = (0..cols).filter(|c| low[*c] >= 2).count();
        if (reach as f64) / (cols as f64) < FOOT_SHARE {
            if length >= found::LIKELY_LEAST {
                w.doubt = Some(format!("likely a wall: a face {} long that reaches the ceiling, its foot not seen over {} of its length (something low stood before it, or it hangs from the ceiling)", found::plumbline_units(length), found::plumbline_units((1.0 - down as f64 / cols as f64) * length)));
            } else {
                let m = line.mid();
                demoted.push(format!("a face {} long at {} east, {} north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung from the ceiling, or a wall seen only above)", found::plumbline_units(length), found::plumbline_units(m[0]), found::plumbline_units(m[1])));
                continue;
            }
        }
        // the openings: a grid of squares along the wall and up it; clean blocks of empty squares at least 1 ft 6 in wide are holes; one at a wall's end, or
        // from floor to ceiling, is what something stands before (not seen) or is left as it was; the rest are doorways and windows (call 173, 179)
        let tops = |s: f64| ceil.over(line.at(s), n);
        let g = found::Grid::new(&b.wp, line.s0, line.s1, high, sq, &tops);
        let empty = |c: usize, r: usize| !g.full(c, r);
        let core = |c: usize, r: usize| {
            empty(c, r) && [(c.wrapping_sub(1), r), (c + 1, r), (c, r.wrapping_sub(1)), (c, r + 1)].iter().filter(|(nc, nr)| *nc < g.cols && *nr < g.rows && empty(*nc, *nr)).count() >= 3
        };
        let mut seen = vec![false; g.cols * g.rows];
        let mut holes: Vec<(usize, usize, usize, usize, bool)> = Vec::new();
        for r in 0..g.rows {
            for c in 0..g.cols {
                if seen[r * g.cols + c] || !core(c, r) {
                    continue;
                }
                let (mut stack, mut cs) = (vec![(c, r)], Vec::new());
                seen[r * g.cols + c] = true;
                while let Some((cc, rr)) = stack.pop() {
                    cs.push((cc, rr));
                    for (nc, nr) in [(cc.wrapping_sub(1), rr), (cc + 1, rr), (cc, rr.wrapping_sub(1)), (cc, rr + 1)] {
                        if nc < g.cols && nr < g.rows && !seen[nr * g.cols + nc] && core(nc, nr) {
                            seen[nr * g.cols + nc] = true;
                            stack.push((nc, nr));
                        }
                    }
                }
                let (c0, c1) = (cs.iter().map(|x| x.0).min().unwrap_or(0), cs.iter().map(|x| x.0).max().unwrap_or(0));
                let (r0, r1) = (cs.iter().map(|x| x.1).min().unwrap_or(0), cs.iter().map(|x| x.1).max().unwrap_or(0));
                let box_cells = (c1 - c0 + 1) * (r1 - r0 + 1);
                if (c1 - c0 + 1) as f64 * sq >= found::MIN_OPENING - sq && cs.len() as f64 >= 0.8 * box_cells as f64 {
                    holes.push((c0, c1, r0, r1, c0 == 0 || c1 + 1 == g.cols));
                }
            }
        }
        holes.sort();
        let blocked = |el: f64, er: f64, sides: &[f64]| -> bool {
            sides.iter().all(|side| {
                stand_at.iter().any(|q| {
                    let (s, a) = (line.along(*q), line.across(*q) * side);
                    s >= el && s <= er && a > half + found::IN_FRONT_LEAST && a <= half + found::IN_FRONT
                })
            })
        };
        let sides: Vec<f64> = if b.thickness.is_some() { vec![1.0, -1.0] } else { vec![b.side] };
        let say = |from: f64, to: f64| if flip { Stretch { from: length - to, to: length - from } } else { Stretch { from, to } };
        for (c0, c1, r0, r1, at_end) in holes {
            let (ls, rs) = (g.col_range(c0), g.col_range(c1 + 1));
            let (zb, zt) = (found::SET_ASIDE + r0 as f64 * sq, found::SET_ASIDE + (r1 + 1) as f64 * sq);
            if at_end || r1 + 1 >= g.rows {
                let (el, er) = (ls.max(line.s0), rs.min(line.s1));
                if blocked(el, er, &sides) {
                    w.hidden.push(say(el - line.s0, er - line.s0));
                } else if !at_end {
                    out.left_out.push(format!("a gap {} wide in a wall, {} from its {} end, runs from the floor to the ceiling: it is not taken as an opening, and the wall is laid across it", found::plumbline_units(rs - ls), found::plumbline_units(if flip { length - (rs - line.s0) } else { ls - line.s0 }), found::end_word(&w)));
                }
                continue;
            }
            let strip_lo = if r0 <= 1 { found::SET_ASIDE } else { zb + 0.3 };
            let strip_hi = (zt - 0.3).max(strip_lo + 0.1);
            let in_strip: Vec<f64> = b.wp.iter().filter(|p| p[1] >= strip_lo && p[1] <= strip_hi).map(|p| p[0]).collect();
            let hole_mid = (ls + rs) / 2.0;
            let el = found::edge(&in_strip, ls - found::EDGE_BACK, false, ls - found::EDGE_BACK - line.s0 - found::NEAR_END, hole_mid).unwrap_or(ls);
            let er = found::edge(&in_strip, rs + found::EDGE_BACK, true, line.s1 - rs - found::EDGE_BACK - found::NEAR_END, hole_mid).unwrap_or(rs);
            if er - el < found::MIN_OPENING - found::MIN_OPENING_SLACK {
                continue;
            }
            let in_up: Vec<f64> = b.wp.iter().filter(|p| p[0] >= el + 0.3 && p[0] <= er - 0.3).map(|p| p[1]).collect();
            let head = found::edge(&in_up, zt + found::EDGE_BACK, true, tops((el + er) / 2.0) - found::SET_ASIDE - zt - found::EDGE_BACK, zb).unwrap_or(zt);
            let window = r0 > 1;
            let sill = if window { found::edge(&in_up, zb - found::EDGE_BACK, false, zb - found::EDGE_BACK - found::SET_ASIDE, zt).unwrap_or(zb) } else { 0.0 };
            let (f0, f1) = (el - line.s0, er - line.s0);
            let opening = if flip { Opening { window, from: length - f1, to: length - f0, sill, head } } else { Opening { window, from: f0, to: f1, sill, head } };
            if blocked(el, er, &sides) {
                w.hidden.push(Stretch { from: opening.from, to: opening.to });
            } else {
                w.openings.push(opening);
            }
        }
        walls.push((w, WallKeep { line }));
    }
    for d in demoted {
        out.left_out.push(d);
    }
    // ---- 12. a face that reaches the ceiling and meets no wall at either end is in doubt (call 179, A3 and call 183, 3) ----
    {
        let lines: Vec<Fc> = walls.iter().map(|(_, k)| k.line.clone()).collect();
        let thick: Vec<bool> = walls.iter().map(|(w, _)| w.thickness.is_some()).collect();
        for i in 0..walls.len() {
            if walls[i].0.doubt.is_some() || thick[i] {
                continue;
            }
            let l = &lines[i];
            let meets = |e: f64| {
                (0..lines.len()).filter(|j| *j != i && walls[*j].0.doubt.is_none()).any(|j| {
                    let o = &lines[j];
                    if l.degrees(o) < 20.0 {
                        return false;
                    }
                    let q = cross_point(l, o);
                    (l.along(q) - e).abs() <= found::CORNER && o.along(q) >= o.s0 - found::CORNER && o.along(q) <= o.s1 + found::CORNER
                })
            };
            if meets(l.s0) || meets(l.s1) {
                continue;
            }
            let side = walls[i].0.seen_from.map_or(1.0, |sf| if dot2(sf, l.nrm()) >= 0.0 { 1.0 } else { -1.0 });
            let behind: Option<f64> = (0..lines.len())
                .filter(|j| *j != i)
                .filter_map(|j| {
                    let o = &lines[j];
                    let across = -side * l.across(o.mid());
                    let along = alongside(l, &o.like(l.d));
                    (l.degrees(o) <= found::PAIR_DEGREES + 1.0 && across > 0.0 && across <= found::BEHIND && along > 0.0).then_some(across)
                })
                .fold(None, |a: Option<f64>, x| Some(a.map_or(x, |a| a.min(x))));
            walls[i].0.doubt = Some(match behind {
                Some(across) => format!("a wall or something tall standing at the wall: it reaches the ceiling, but neither end meets a wall and a wall face stands {} behind it", found::plumbline_units(across)),
                None => "a face that reaches the ceiling and meets no wall at either end: a wall seen from one side that stops in the room, or the front of something tall standing in it".to_string(),
            });
        }
    }

    // ---- what lies beyond a wall with no way to it (call 183, 2): in doubt, said ----
    let walls_only: Vec<Wall> = walls.iter().map(|(w, _)| w.clone()).collect();
    let (pieces, gone, note) = crate::found_183::beyond(walls_only, &mut out.things, &floor_dense);
    out.beyond_note = note;
    let mut pieces = pieces;
    for (w, _) in pieces.iter_mut() {
        let d = unit2(sub2(w.to, w.from));
        let mid = [(w.from[0] + w.to[0]) / 2.0, (w.from[1] + w.to[1]) / 2.0];
        let nn = [-d[1], d[0]];
        w.ceiling_not_read = !(ceil.reads([mid[0] + nn[0] * found::CEIL_OUT, mid[1] + nn[1] * found::CEIL_OUT]) || ceil.reads([mid[0] - nn[0] * found::CEIL_OUT, mid[1] - nn[1] * found::CEIL_OUT]));
    }
    pieces.sort_by(|a, b| a.0.from[0].total_cmp(&b.0.from[0]).then(a.0.from[1].total_cmp(&b.0.from[1])));
    let place_of = |orig: usize| pieces.iter().position(|(w, o)| *o == orig && !w.doubt.as_deref().is_some_and(|d| d.starts_with("in doubt: beyond"))).or_else(|| pieces.iter().position(|(_, o)| *o == orig)).unwrap_or(0);
    out.beyond_things = gone.into_iter().map(|(_, s)| crate::found_183::resolve(&s, &place_of)).collect();
    let numbered: Vec<(Wall, usize)> = pieces.iter().map(|(w, o)| (Wall { doubt: w.doubt.as_ref().map(|d| crate::found_183::resolve(d, &place_of)), ..w.clone() }, *o)).collect();
    out.walls = numbered.into_iter().map(|(w, _)| w).collect();
    crate::found_183::one_face_one_wall(&mut out.walls);
    out.open_ends = found::open_ends(&out.walls);
    // Every stretch left out is said, with where it is and how long (call 183, 4), unless a wall found lies along it.
    left.retain(|l| !out.walls.iter().any(|w| found::distance_to(l.from, w.from, w.to) <= 1.0 && found::distance_to(l.to, w.from, w.to) <= 1.0));
    left.sort_by(|a, b| a.from[0].total_cmp(&b.from[0]).then(a.from[1].total_cmp(&b.from[1])));
    for l in &left {
        out.left_out.push(format!("a stretch of upright points {} long, from {} east, {} north to {} east, {} north, not taken as walls: {}", found::plumbline_units((l.to[0] - l.from[0]).hypot(l.to[1] - l.from[1])), found::plumbline_units(l.from[0]), found::plumbline_units(l.from[1]), found::plumbline_units(l.to[0]), found::plumbline_units(l.to[1]), l.why));
    }
    out.left = left;
    out
}

/// Where two lines cross.
fn cross_point(a: &Fc, o: &Fc) -> P2 {
    let (n1, n2) = (a.nrm(), o.nrm());
    let det = n1[0] * n2[1] - n1[1] * n2[0];
    if det.abs() < 1e-12 {
        return a.mid();
    }
    [(a.rho * n2[1] - o.rho * n1[1]) / det, (n1[0] * o.rho - n2[0] * a.rho) / det]
}
