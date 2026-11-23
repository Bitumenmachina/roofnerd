//! Walls, doors and windows found on a drawn plan: a PDF whose lines are its own, not a
//! scan of paper (DECISIONS.md call 181). Plain geometry over the straight lines a page
//! draws (`Page::lines`) and the sheet's scale; no dependency beyond this crate.
//!
//! What a wall is, on a drawing: two lines side by side, parallel, 3 in to 14 in apart at
//! the sheet's scale, each at least 2 ft long at scale, alongside one another for that
//! length. Its centre lies between them; its thickness is the distance between them
//! (Assumed: a drawn thickness is the drafter's). A line across both lines ends a wall
//! there. Three or more evenly spaced parallel lines are not walls (treads, siding,
//! boards). A pair that meets no other wall at either end is in doubt. A break in both
//! lines with the wall going on beyond it is an opening; lines across the break inside the
//! wall's thickness make it a window. Lettering, dimension lines, the sheet's border and
//! its title block, key and schedules are not walls, and what was left out is counted by
//! kind. Nothing here is Known, and what it cannot tell it says, with where.
//!
//! Every measure is a named constant below, with its reason. The plan's lines are in the
//! page's own units; the walls come back in feet from the page's lower-left corner, up and
//! to the right, so a drawing's own corner puts them on the plan.

use std::collections::{BTreeMap, HashMap};

pub type P2 = [f64; 2];

const INCH: f64 = 1.0 / 12.0;
/// The thinnest and thickest a wall is drawn: 3 in (a 2 x 2 furred wall) to 14 in (a 12 in block with finish).
pub const THIN_LEAST: f64 = 3.0 * INCH;
pub const THICK_MOST: f64 = 14.0 * INCH;
/// The shortest a wall's line is, at scale (call 181).
pub const MIN_FACE: f64 = 2.0;
/// Nor is a line a wall's face if it is shorter than this on the paper: 14.4 pt, twice the height of
/// the letters of a dimension. At small scales the strokes of an H stand 3 in to 14 in apart at scale.
pub const LETTER_PT: f64 = 14.4;
/// Lines whose directions differ by under this (0.11 degrees, in radians) and which lie on one line
/// are one face: a drawing's lines are exact, and a long line would otherwise leave its neighbours.
const ANGLE_FACE: f64 = 0.002;
/// Two faces are parallel if their directions differ by under this (0.3 degrees).
const ANGLE_PAIR: f64 = 0.0052;
/// A line lies on a face if both its ends are within 1/8 in of it.
const LINE_TOL: f64 = INCH / 8.0;
/// Pieces of one line that come within 1/4 in of one another are one stretch.
const MERGE_GAP: f64 = INCH / 4.0;
/// A break in a wall's line, or in both, no wider than the thickest wall is a joint (a wall crossing or
/// meeting it), not an opening.
const JOINT_MOST: f64 = THICK_MOST;
/// A line across a wall's two lines has its ends within 1/2 in of them.
const CAP_TOL: f64 = INCH / 2.0;
/// A line is across another if it is square to it within 3 degrees (radians).
const CROSS_ANGLE: f64 = 0.05;
/// Lines are evenly spaced if their spacings agree within 8 per cent or 1/4 in.
const EVEN_SHARE: f64 = 0.08;
const EVEN_ABS: f64 = INCH / 4.0;
/// Two lines are alongside one another if they overlap for 0.6 of the longer one's stretch: a short
/// line down a long wall (a window's glass) is inside the wall, not a wall of its own.
const OVERLAP_SHARE: f64 = 0.6;
/// A line inside a wall is shorter than both faces by this share.
const INSIDE_SHARE: f64 = 0.9;
/// A line is across a window if it covers 0.7 of its width.
const GLASS_SHARE: f64 = 0.7;
/// The widest opening taken: 16 ft (a garage door for two cars, a pair of patio doors). A wider break in
/// both lines is the end of one wall and the start of another, and is said.
const OPENING_MOST: f64 = 16.0;
/// A box is the sheet's border, not a title block or a key, if it covers 0.6 of the sheet.
const BORDER_SHARE: f64 = 0.6;
/// A dimension's tick or arrowhead is under 14 pt long on the paper and ends within 1.5 pt of its line.
const TICK_MAX_PT: f64 = 14.0;
const TERM_NEAR_PT: f64 = 1.5;
/// Corners of a box meet within 1 pt on the paper.
const TOUCH_PT: f64 = 1.0;
/// A run of short pieces is a curve if it has this many pieces, is this long at scale, and turns no more
/// than 35 degrees at a joint.
const CURVE_PIECES: usize = 5;
const CURVE_LONG: f64 = 4.0;
const CURVE_TURN: f64 = 35.0;
/// Two walls are of one thickness if they are drawn within 1/4 in of it: what a sheet's walls are, said by how many of them
/// are that thick (call 186).
const THICK_SAME: f64 = INCH / 4.0;
/// A pair is thicker than the other walls only if it is thicker than the thickest of them by at least this: 3 in, the least a wall
/// is drawn (call 186, narrowed). An 8 in garage wall among 6 1/2 in walls is an ordinary house; a 12 in shelf among 8 in walls is not.
const THICK_MORE: f64 = THIN_LEAST;
/// A group of walls that meet one another is apart from the plan if it is under this share of the plan's length: two plans of
/// like size on one sheet are both plans, a detail or a section beside one is far smaller (experiment, call 186 item 3).
const APART_SHARE: f64 = 0.5;
/// A wall's face is drawn twice when a second line stands within this of it (a finish, a sheathing line: the real sheets of call 188
/// had them 0.6 in and 1 in in). A stack of pairs whose every line is within this of the outermost two is one wall.
const DOUBLE_MAX: f64 = 1.5 * INCH;
/// Drawings are of like size if the smaller has at least this share of the larger's wall (call 188 item 6): the share under which a
/// group is still set apart without asking is `APART_SHARE`, and this is the same half.
pub const LIKE_SIZE: f64 = APART_SHARE;
/// Walls whose centre lines come within this of one another, 15 ft, are one drawing; the plan and a roof plan beside it stand further apart
/// (call 188 item 6, as sent back).
const DRAWING_GAP: f64 = 15.0;
/// A tag (a window's or a door's mark: a hexagon, a diamond) is a closed run of at least this many straight pieces, none longer than
/// `TAG_PIECE` and the whole no more than `TAG_SPAN` across (call 188 item 8).
const TAG_PIECES: usize = 5;
const TAG_PIECE: f64 = 1.5;
const TAG_SPAN: f64 = 3.0;
/// More lines than this are not all looked at: those under 4 in at scale are left out, and it says so.
const MOST_SEGS: usize = 150_000;
const SHORT_KEPT: f64 = 4.0 * INCH;

#[derive(Clone, Debug, PartialEq)]
pub struct Opening {
    /// A window (two or more lines across it inside the wall) or a door or open doorway.
    pub window: bool,
    /// Along the wall from its first end, in feet.
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wall {
    /// The centre line, in feet on the page, from the end `end_of` names first.
    pub from: P2,
    pub to: P2,
    pub thickness: f64,
    pub openings: Vec<Opening>,
    /// Why it is in doubt, where it is.
    pub doubt: Option<String>,
    /// How many lines the wall is drawn with where that is more than its two faces (a wall drawn in its layers: the
    /// outermost two are its faces and the others lie between them); 0 for a wall of two lines.
    pub lines: usize,
    /// How many pairs of lines were found over this one wall and joined into it (call 188 item 7): a face drawn twice makes
    /// two to four pairs of the one wall. 1 for a wall found once, from the one pair.
    pub joined: usize,
}

impl Wall {
    pub fn length(&self) -> f64 {
        ((self.to[0] - self.from[0]).powi(2) + (self.to[1] - self.from[1]).powi(2)).sqrt()
    }
}

/// A set of evenly spaced lines that were not taken as walls.
#[derive(Clone, Debug, PartialEq)]
pub struct Set {
    pub lines: usize,
    pub spacing: f64,
    pub at: P2,
}

/// A box on the sheet (a title block, a key, a schedule) whose lines were not looked at.
#[derive(Clone, Debug, PartialEq)]
pub struct Boxed {
    pub wide: f64,
    pub high: f64,
    pub at: P2,
    pub lines: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Counts {
    pub lines: usize,
    pub dimension: usize,
    pub border: usize,
    pub boxed: usize,
    pub set: usize,
    pub in_walls: usize,
    pub single: usize,
    pub small: usize,
    /// Stretches of two lines side by side shorter than a wall's least.
    pub short_stretches: usize,
    /// Breaks in both lines narrower than an opening (a wall crossing it).
    pub joints: usize,
    /// Lines across a wall too near another such line to end a wall.
    pub crosses_ignored: usize,
    pub left_out_short_lines: usize,
    /// Lines drawn dashed: a wall's lines are solid, so none is taken (call 186).
    pub dashed: usize,
    /// Lines across or inside a wall in a partition whose faces run on, where a person reads glass and it is no window (call 188).
    pub glass_inside: usize,
    /// Lines left out because they lie outside the box the person said the plan is in (call 188 item 6).
    pub outside: usize,
    /// Lines of tags (small closed shapes of straight pieces, a window's or a door's mark) left out (call 188).
    pub tags: usize,
}

/// Two or more dashed lines side by side: something overhead or hidden (a beam, a soffit, cabinets above), or, on
/// a renovation plan, a wall to be demolished. A wall's lines are solid (call 186), so it is never taken as a wall.
#[derive(Clone, Debug, PartialEq)]
pub struct Overhead {
    pub lines: usize,
    /// How far apart the outer two lines are, and how long the longest is.
    pub apart: f64,
    pub length: f64,
    pub at: P2,
}

/// What the page's reader knows of its lines beyond where they are (call 186): which are dashed, and the places
/// where a line was cut off at the edge of what the page shows (the page's own edge or a clip).
#[derive(Clone, Copy, Debug, Default)]
pub struct Extra<'a> {
    /// The places in `lines` of the dashed lines.
    pub dashed: &'a [usize],
    /// Places on the page, in its own units, where a line was cut off.
    pub cuts: &'a [P2],
}

#[derive(Clone, Debug, Default)]
pub struct Drawn {
    pub no_scale: bool,
    pub walls: Vec<Wall>,
    pub sets: Vec<Set>,
    pub boxes: Vec<Boxed>,
    /// The ends of the walls (not in doubt) that meet no other wall, with the wall's place in `walls`.
    pub open_ends: Vec<(usize, P2)>,
    /// The ends of the walls (not in doubt) that run to the edge of what the page shows, where a clip or the page's
    /// own edge cut the lines off: the wall may go on beyond it. Not open ends.
    pub cut_ends: Vec<(usize, P2)>,
    pub counts: Counts,
    /// Anything else it could not tell, each with where.
    pub notes: Vec<String>,
    /// Lines drawn in short straight pieces, end to end, turning a little at each (a curved wall, a bay,
    /// a door's swing): how many pieces, how long it is, where. They are not followed and no wall is found in them.
    pub curved: Vec<(usize, f64, P2)>,
    /// Dashed lines side by side, a wall's thickness apart: what they may be, where. Never walls.
    pub overhead: Vec<Overhead>,
    /// Outlines of single lines meeting end to end (a room or a house drawn as one line a wall): how many lines, where.
    /// A thickness cannot be read from a single line.
    pub single_outlines: Vec<(usize, P2)>,
    /// The drawings on the sheet where the walls found fall into more than one group of like size (call 188 item 6), the
    /// largest first. Empty where the sheet has one.
    pub drawings: Vec<Plan>,
    /// Which of the drawings was taken (counted from 1), when the person said so.
    pub chosen: Option<usize>,
    /// The box the person bounded the plan with, in feet on the plan (south-west, north-east corner), when they did.
    pub within: Option<[P2; 2]>,
}

/// One drawing of a sheet that holds more than one: a group of walls that meet one another.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// Its walls, by their place in `Drawn::walls`.
    pub walls: Vec<usize>,
    /// How much wall it has, in feet.
    pub length: f64,
    /// How much of it is wall in doubt, in feet.
    pub in_doubt: f64,
    /// The corners of the box it lies in, in feet on the plan: south-west, north-east.
    pub corners: [P2; 2],
}

/// What a person says of which part of a sheet is the plan (call 188 item 6).
#[derive(Clone, Copy, Debug, Default)]
pub struct Pick {
    /// The drawing to take, counted from 1 as the drawings are listed.
    pub plan: Option<usize>,
    /// A box on the sheet, two corners in feet on the plan: nothing outside it is looked at.
    pub within: Option<[P2; 2]>,
}

// ---- small geometry ----

fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn norm(a: P2) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: P2) -> P2 {
    let n = norm(a).max(1e-300);
    [a[0] / n, a[1] / n]
}
fn perp(u: P2) -> P2 {
    [-u[1], u[0]]
}
fn at(u: P2, n: P2, t: f64, c: f64) -> P2 {
    [u[0] * t + n[0] * c, u[1] * t + n[1] * c]
}

#[derive(Clone, Copy, Debug)]
struct Seg {
    a: P2,
    b: P2,
    len: f64,
}

type Iv = Vec<(f64, f64)>;

fn merged(mut v: Iv, gap: f64) -> Iv {
    v.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut out: Iv = Vec::new();
    for (a, b) in v {
        match out.last_mut() {
            Some(l) if a <= l.1 + gap => l.1 = l.1.max(b),
            _ => out.push((a, b)),
        }
    }
    out
}

fn intersect(a: &Iv, b: &Iv) -> Iv {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        let (lo, hi) = (a[i].0.max(b[j].0), a[i].1.min(b[j].1));
        if hi > lo {
            out.push((lo, hi));
        }
        if a[i].1 < b[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

fn covered(iv: &Iv, lo: f64, hi: f64) -> f64 {
    iv.iter().map(|(a, b)| (b.min(hi) - a.max(lo)).max(0.0)).sum()
}

// ---- faces: the lines that lie on one line, with where along it they are drawn ----

#[derive(Clone, Debug)]
struct Face {
    u: P2,
    n: P2,
    c: f64,
    iv: Iv,
    /// The same, bridged across joints; a bridged stretch is kept only if some piece of it is as long as
    /// a letter is tall on the paper (a row of lettering is no line, though its strokes line up).
    ivb: Iv,
    longest: f64,
    /// Each line's stretch as drawn, not merged: how many times over the face is drawn.
    raw: Iv,
}

impl Face {
    fn ends(&self, t0: f64, t1: f64) -> (P2, P2) {
        (at(self.u, self.n, t0, self.c), at(self.u, self.n, t1, self.c))
    }
    /// This face's stretches, as they lie along `other`'s direction.
    fn along(&self, other: &Face, iv: &Iv) -> Iv {
        let mut out: Iv = iv
            .iter()
            .map(|&(a, b)| {
                let (p, q) = self.ends(a, b);
                let (x, y) = (dot(other.u, p), dot(other.u, q));
                (x.min(y), x.max(y))
            })
            .collect();
        out.sort_by(|x, y| x.0.total_cmp(&y.0));
        out
    }
    /// How far this face lies from `other`'s line, along `other`'s normal, where `other` is at `t`.
    fn offset_from(&self, other: &Face, t: f64) -> f64 {
        let p = at(other.u, other.n, t, other.c);
        (self.c - dot(self.n, p)) * if dot(self.n, other.n) >= 0.0 { 1.0 } else { -1.0 }
    }
}

fn angle_of(u: P2) -> f64 {
    let mut t = u[1].atan2(u[0]);
    if t < 0.0 {
        t += std::f64::consts::PI;
    }
    if t >= std::f64::consts::PI {
        t -= std::f64::consts::PI;
    }
    t
}

/// The faces the segments `ids` make: lines on one line (both ends within 1/8 in of it, directions
/// within 0.11 degree) are one face. Each line is looked for among those near it by direction and
/// by where it lies, so a hatched sheet is no slower than a bare one. Returns the faces and, for
/// each segment, the face it is in.
fn faces_of(segs: &[Seg], ids: &[usize], letter: f64) -> (Vec<Face>, HashMap<usize, usize>) {
    let nb = (std::f64::consts::PI / ANGLE_FACE).ceil() as usize + 1;
    let frame = |b: usize| {
        let th = (b as f64 + 0.5) * ANGLE_FACE;
        let u = [th.cos(), th.sin()];
        (u, perp(u))
    };
    let mut buckets: BTreeMap<usize, Vec<(f64, usize)>> = BTreeMap::new();
    let mut dirs: HashMap<usize, (P2, usize)> = HashMap::new();
    for &i in ids {
        let s = &segs[i];
        let mut u = unit(sub(s.b, s.a));
        if u[1] < 0.0 || (u[1] == 0.0 && u[0] < 0.0) {
            u = [-u[0], -u[1]];
        }
        let b = ((angle_of(u) / ANGLE_FACE) as usize).min(nb - 1);
        let (_, n) = frame(b);
        let mid = [(s.a[0] + s.b[0]) / 2.0, (s.a[1] + s.b[1]) / 2.0];
        buckets.entry(b).or_default().push((dot(n, mid), i));
        dirs.insert(i, (u, b));
    }
    for v in buckets.values_mut() {
        v.sort_by(|x, y| x.0.total_cmp(&y.0));
    }
    let mut order: Vec<usize> = ids.to_vec();
    order.sort_by(|&x, &y| segs[y].len.total_cmp(&segs[x].len).then(x.cmp(&y)));
    let mut taken: HashMap<usize, usize> = HashMap::new();
    let mut faces: Vec<Face> = Vec::new();
    for &s in &order {
        if taken.contains_key(&s) {
            continue;
        }
        let (u, b) = dirs[&s];
        let n = perp(u);
        let c = dot(n, segs[s].a);
        let mid = [(segs[s].a[0] + segs[s].b[0]) / 2.0, (segs[s].a[1] + segs[s].b[1]) / 2.0];
        let id = faces.len();
        let mut spans: Iv = Vec::new();
        for nbk in [(b + nb - 1) % nb, b, (b + 1) % nb] {
            let Some(v) = buckets.get(&nbk) else { continue };
            let (_, nn) = frame(nbk);
            let target = dot(nn, mid);
            let lo = v.partition_point(|e| e.0 < target - 1.0);
            for &(_, i) in &v[lo..] {
                let (cc, _) = (0, 0);
                let _ = cc;
                // Entries are sorted by where they lie; stop past the window.
                let m = [(segs[i].a[0] + segs[i].b[0]) / 2.0, (segs[i].a[1] + segs[i].b[1]) / 2.0];
                if dot(nn, m) > target + 1.0 {
                    break;
                }
                if taken.contains_key(&i) {
                    continue;
                }
                let (ea, eb) = (dot(n, segs[i].a) - c, dot(n, segs[i].b) - c);
                if ea.abs() <= LINE_TOL && eb.abs() <= LINE_TOL {
                    taken.insert(i, id);
                    let (x, y) = (dot(u, segs[i].a), dot(u, segs[i].b));
                    spans.push((x.min(y), x.max(y)));
                }
            }
        }
        let iv = merged(spans.clone(), MERGE_GAP);
        let mut ivb: Iv = merged(iv.clone(), JOINT_MOST).into_iter().filter(|&(a, b)| iv.iter().any(|&(x, y)| x >= a - 1e-9 && y <= b + 1e-9 && y - x >= letter)).collect();
        // A pier (call 186): a piece of the line too short to count alone, on the same line as a stretch that does and no
        // further from it than the widest opening, is part of the wall that stretch is the face of, however short: it is
        // what lies between two openings, or between an opening and the wall's end.
        loop {
            let more: Vec<(f64, f64)> = iv.iter().copied().filter(|&(a, b)| !ivb.iter().any(|&(x, y)| x <= a + 1e-9 && y >= b - 1e-9) && ivb.iter().any(|&(x, y)| (a >= y && a - y <= OPENING_MOST) || (x >= b && x - b <= OPENING_MOST))).collect();
            if more.is_empty() {
                break;
            }
            ivb.extend(more);
            ivb.sort_by(|x, y| x.0.total_cmp(&y.0));
        }
        let longest = ivb.iter().map(|(a, b)| b - a).fold(0.0, f64::max);
        faces.push(Face { u, n, c, iv, ivb, longest, raw: spans });
    }
    (faces, taken)
}

// ---- dimension lines ----

/// The lines that are a dimension: a line with a tick or an arrowhead at each end, and the ticks,
/// arrowheads and extension lines of it. A tick crosses its line or two arrowhead lines lie either
/// side of it; one short oblique line on one side only is a wall's mitred end and is not a tick.
fn dimension_lines(segs: &[Seg], k: f64) -> Vec<bool> {
    dimension_scan(segs, k).0
}

/// A dimension line found: the line, and the two places it measures between (where its ticks cross it or its arrowheads point),
/// which are not always where the line itself ends.
#[derive(Clone, Copy, Debug)]
struct DimLine {
    between: [P2; 2],
}

fn dimension_scan(segs: &[Seg], k: f64) -> (Vec<bool>, Vec<DimLine>) {
    let mut dim = vec![false; segs.len()];
    let tick_max = TICK_MAX_PT * k;
    let near = TERM_NEAR_PT * k;
    let long_least = MIN_FACE.max(LETTER_PT * k);
    let cell = (tick_max * 2.0).max(1e-6);
    let key = |p: P2| ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, s) in segs.iter().enumerate() {
        if s.len <= tick_max {
            grid.entry(key(s.a)).or_default().push(i);
            let kb = key(s.b);
            if kb != key(s.a) {
                grid.entry(kb).or_default().push(i);
            }
        }
    }
    let dist_to = |p: P2, s: &Seg| -> f64 {
        let d = sub(s.b, s.a);
        let t = (dot(sub(p, s.a), d) / dot(d, d).max(1e-300)).clamp(0.0, 1.0);
        norm(sub(p, [s.a[0] + d[0] * t, s.a[1] + d[1] * t]))
    };
    let nearby = |p: P2| -> Vec<usize> {
        let (cx, cy) = key(p);
        let mut v = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(l) = grid.get(&(cx + dx, cy + dy)) {
                    v.extend(l.iter().copied());
                }
            }
        }
        v.sort_unstable();
        v.dedup();
        v
    };
    let mut ends_of: Vec<(usize, P2, Vec<usize>, [P2; 2])> = Vec::new();
    for (i, s) in segs.iter().enumerate() {
        if s.len < long_least {
            continue;
        }
        let u = unit(sub(s.b, s.a));
        let n = perp(u);
        let mut all_ticks: Vec<usize> = Vec::new();
        let mut both = true;
        let mut between: Vec<P2> = Vec::new();
        for e in [s.a, s.b] {
            let (mut crossing, mut plus, mut minus) = (false, false, false);
            let mut here: Vec<usize> = Vec::new();
            let (mut mids, mut apexes): (Vec<P2>, Vec<P2>) = (Vec::new(), Vec::new());
            for j in nearby(e) {
                let t = &segs[j];
                if j == i || dist_to(e, t) > near {
                    continue;
                }
                let tu = unit(sub(t.b, t.a));
                let sin = cross(u, tu).abs();
                if !(0.34..=0.94).contains(&sin) {
                    continue;
                }
                let (s0, s1) = (dot(n, sub(t.a, e)), dot(n, sub(t.b, e)));
                here.push(j);
                let apex = if norm(sub(t.a, e)) <= norm(sub(t.b, e)) { t.a } else { t.b };
                apexes.push(apex);
                if s0 * s1 < 0.0 && s0.abs().min(s1.abs()) >= 0.15 * t.len {
                    crossing = true;
                    mids.push([(t.a[0] + t.b[0]) / 2.0, (t.a[1] + t.b[1]) / 2.0]);
                } else if s0 + s1 > 0.0 {
                    plus = true;
                } else {
                    minus = true;
                }
            }
            if crossing || (plus && minus) {
                all_ticks.extend(here);
                let pts = if crossing { &mids } else { &apexes };
                let c = pts.iter().fold([0.0, 0.0], |m, p| [m[0] + p[0], m[1] + p[1]]);
                let c = [c[0] / pts.len() as f64, c[1] / pts.len() as f64];
                // Where it is along the line, on the line.
                let t = dot(u, sub(c, s.a));
                between.push([s.a[0] + u[0] * t, s.a[1] + u[1] * t]);
            } else {
                both = false;
                break;
            }
        }
        if both {
            ends_of.push((i, s.a, all_ticks, [between[0], between[1]]));
        }
    }
    let found: Vec<DimLine> = ends_of.iter().map(|e| DimLine { between: e.3 }).collect();
    for (i, _, ticks, _) in &ends_of {
        dim[*i] = true;
        for t in ticks {
            dim[*t] = true;
        }
    }
    // Extension lines: square to a dimension line and through its end.
    let dims: Vec<usize> = ends_of.iter().map(|e| e.0).collect();
    if !dims.is_empty() {
        let cell2 = (long_least * 4.0).max(1e-6);
        let _ = cell2;
        for (j, x) in segs.iter().enumerate() {
            if dim[j] || x.len <= tick_max {
                continue;
            }
            let xu = unit(sub(x.b, x.a));
            for &i in &dims {
                let s = &segs[i];
                let u = unit(sub(s.b, s.a));
                if dot(u, xu).abs() > CROSS_ANGLE {
                    continue;
                }
                if [s.a, s.b].iter().any(|e| dist_to(*e, x) <= near) {
                    dim[j] = true;
                    break;
                }
            }
        }
    }
    (dim, found)
}

// ---- boxes: the sheet's border, title block, key, schedules ----

#[derive(Clone, Debug)]
struct Rect {
    u: P2,
    n: P2,
    /// Low and high along `u`, low and high along `n`.
    t: (f64, f64),
    s: (f64, f64),
    sides: Vec<usize>,
}

fn rects_of(faces: &[Face], k: f64) -> Vec<Rect> {
    // Pieces long enough to be the side of a box.
    struct Piece {
        face: usize,
        a: P2,
        b: P2,
    }
    let least = LETTER_PT * k;
    let touch = TOUCH_PT * k;
    let mut pieces: Vec<Piece> = Vec::new();
    for (fi, f) in faces.iter().enumerate() {
        for &(t0, t1) in &f.iv {
            if t1 - t0 >= least {
                let (a, b) = f.ends(t0, t1);
                pieces.push(Piece { face: fi, a, b });
            }
        }
    }
    let cell = (touch * 4.0).max(1e-6);
    let key = |p: P2| ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<(usize, bool)>> = HashMap::new();
    for (i, p) in pieces.iter().enumerate() {
        grid.entry(key(p.a)).or_default().push((i, false));
        grid.entry(key(p.b)).or_default().push((i, true));
    }
    let touching = |q: P2| -> Vec<(usize, bool)> {
        let (cx, cy) = key(q);
        let mut v = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(l) = grid.get(&(cx + dx, cy + dy)) {
                    for &(i, end) in l {
                        let e = if end { pieces[i].b } else { pieces[i].a };
                        if norm(sub(e, q)) <= touch {
                            v.push((i, end));
                        }
                    }
                }
            }
        }
        v
    };
    let mut seen: std::collections::BTreeSet<[usize; 4]> = Default::default();
    let mut out = Vec::new();
    for (ai, a) in pieces.iter().enumerate() {
        let ua = unit(sub(a.b, a.a));
        let la = norm(sub(a.b, a.a));
        let at_a = touching(a.a);
        let at_b = touching(a.b);
        for &(bi, bend) in at_a.iter().take(6) {
            if bi == ai {
                continue;
            }
            let b = &pieces[bi];
            let fb = if bend { b.a } else { b.b };
            let ub = unit(sub(fb, a.a));
            if dot(ua, ub).abs() > 0.0175 {
                continue;
            }
            for &(ci, cend) in at_b.iter().take(6) {
                if ci == ai || ci == bi {
                    continue;
                }
                let c = &pieces[ci];
                let fc = if cend { c.a } else { c.b };
                let uc = unit(sub(fc, a.b));
                if dot(ub, uc) < 0.999 || (norm(sub(fb, a.a)) - norm(sub(fc, a.b))).abs() > 2.0 * touch {
                    continue;
                }
                for (di, _) in touching(fb) {
                    if [ai, bi, ci].contains(&di) {
                        continue;
                    }
                    let d = &pieces[di];
                    let far = if norm(sub(d.a, fb)) <= touch { d.b } else { d.a };
                    if norm(sub(far, fc)) > touch {
                        continue;
                    }
                    let mut key4 = [ai, bi, ci, di];
                    key4.sort_unstable();
                    if !seen.insert(key4) {
                        continue;
                    }
                    let lb = norm(sub(fb, a.a));
                    let n = perp(ua);
                    let sgn = if dot(n, ub) >= 0.0 { 1.0 } else { -1.0 };
                    let (t0, t1) = (dot(ua, a.a), dot(ua, a.b));
                    let (s0, s1) = (dot(n, a.a), dot(n, a.a) + sgn * lb);
                    out.push(Rect { u: ua, n, t: (t0.min(t1), t0.max(t1)), s: (s0.min(s1), s0.max(s1)), sides: vec![pieces[ai].face, pieces[bi].face, pieces[ci].face, pieces[di].face] });
                    let _ = la;
                }
            }
        }
    }
    out
}

// ---- two faces side by side ----

/// Two faces side by side: the stretches where both are drawn, long enough, and making up enough
/// of the longer one's, as they lie along `a`'s direction; and how far `b` lies from `a` (signed
/// along `a`'s normal). None if they are not parallel, not alongside, or not parallel enough to
/// have one thickness along the stretch.
fn alongside(faces: &[Face], a: usize, b: usize, min_len: f64) -> Option<(Iv, f64)> {
    let (fa, fb) = (&faces[a], &faces[b]);
    if cross(fa.u, fb.u).abs() > ANGLE_PAIR {
        return None;
    }
    let ivb_b = fb.along(fa, &fb.ivb);
    let mut keep: Iv = Vec::new();
    for (lo, hi) in intersect(&fa.ivb, &ivb_b) {
        let piece = |iv: &Iv| iv.iter().find(|(x, y)| *x <= lo + 1e-9 && *y >= hi - 1e-9).map(|(x, y)| y - x).unwrap_or(0.0);
        if hi - lo >= OVERLAP_SHARE * piece(&fa.ivb).max(piece(&ivb_b)) {
            keep.push((lo, hi));
        }
    }
    // The least length is of the wall's whole line, not of a piece between two openings (call 186): pieces no
    // further apart than the widest opening are one wall, and it is as long as its first piece's start to its last
    // piece's end. A piece alone is a wall only if it is long enough alone.
    let mut kept: Iv = Vec::new();
    let mut run: Iv = Vec::new();
    for &(lo, hi) in &keep {
        if run.last().is_some_and(|l| lo - l.1 > OPENING_MOST) {
            if run[run.len() - 1].1 - run[0].0 >= min_len {
                kept.append(&mut run);
            }
            run.clear();
        }
        run.push((lo, hi));
    }
    if !run.is_empty() && run[run.len() - 1].1 - run[0].0 >= min_len {
        kept.append(&mut run);
    }
    let keep = kept;
    if keep.is_empty() {
        return None;
    }
    let mid = (keep[0].0 + keep[0].1) / 2.0;
    let d = fb.offset_from(fa, mid);
    let (e0, e1) = (fb.offset_from(fa, keep[0].0), fb.offset_from(fa, keep[keep.len() - 1].1));
    if (e0 - e1).abs() > INCH {
        return None;
    }
    Some((keep, d))
}

/// For each long face, the faces alongside it within a wall's thickness: which, how far (signed along
/// its own normal) and the stretches, along its own direction.
fn neighbours(faces: &[Face], long: &[bool], min_len: f64) -> Vec<Vec<(usize, f64, Iv)>> {
    let nf = faces.len();
    let mut near: Vec<Vec<(usize, f64, Iv)>> = vec![Vec::new(); nf];
    let mut by_angle: Vec<usize> = (0..nf).filter(|&i| long[i]).collect();
    by_angle.sort_by(|&x, &y| angle_of(faces[x].u).total_cmp(&angle_of(faces[y].u)));
    let m = by_angle.len();
    for ix in 0..m {
        let a = by_angle[ix];
        let ta = angle_of(faces[a].u);
        for off in 1..m {
            let b = by_angle[(ix + off) % m];
            let diff = (angle_of(faces[b].u) - ta).rem_euclid(std::f64::consts::PI);
            if diff > ANGLE_PAIR {
                break;
            }
            if let Some((iv, d)) = alongside(faces, a, b, min_len) {
                if d.abs() >= THIN_LEAST - 1e-6 && d.abs() <= THICK_MOST + 1e-6 {
                    let back = faces[a].along(&faces[b], &iv);
                    near[a].push((b, d, iv));
                    near[b].push((a, -d, back));
                }
            }
        }
    }
    // A sheet whose long faces are all level (or all upright) has the loop above come round to the faces behind it, and each pair
    // is found from both its faces: a pair is listed once (call 188).
    for list in near.iter_mut() {
        let mut seen = std::collections::HashSet::new();
        list.retain(|(b, _, _)| seen.insert(*b));
    }
    near
}

/// The spans a page's dimension lines measure, line by line: along each line that is drawn (in pieces or whole, with gaps where a
/// label is), the places a tick or an arrowhead stands (a short oblique stroke touching the line, or one on each side of it), and each
/// run of two ticks next to one another is a span (a chain of dimensions on one line is several). Units are the page's own.
fn dimension_spans(segs: &[Seg]) -> Vec<DimLine> {
    let ids: Vec<usize> = (0..segs.len()).collect();
    let (faces, _) = faces_of(segs, &ids, LETTER_PT);
    let ticks: Vec<usize> = (0..segs.len()).filter(|&i| segs[i].len <= TICK_MAX_PT).collect();
    let mut out = Vec::new();
    for f in &faces {
        let (Some(first), Some(last)) = (f.iv.first(), f.iv.last()) else { continue };
        if f.longest < LETTER_PT && last.1 - first.0 < 2.0 * LETTER_PT {
            continue;
        }
        // Each tick's place along the line and which sides of it its strokes lie on.
        let mut marks: Vec<(f64, bool, bool)> = Vec::new();
        for &i in &ticks {
            let s = &segs[i];
            let tu = unit(sub(s.b, s.a));
            let sin = cross(f.u, tu).abs();
            if !(0.34..=0.94).contains(&sin) {
                continue;
            }
            let (da, db) = (dot(f.n, s.a) - f.c, dot(f.n, s.b) - f.c);
            // It touches the line: one end on it, or it crosses it.
            let touches = da.abs().min(db.abs()) <= TERM_NEAR_PT || da * db < 0.0;
            if !touches {
                continue;
            }
            let t = if da * db < 0.0 {
                let w = da.abs() / (da.abs() + db.abs());
                dot(f.u, s.a) + (dot(f.u, s.b) - dot(f.u, s.a)) * w
            } else if da.abs() <= db.abs() {
                dot(f.u, s.a)
            } else {
                dot(f.u, s.b)
            };
            if t < first.0 - TERM_NEAR_PT * 3.0 || t > last.1 + TERM_NEAR_PT * 3.0 {
                continue;
            }
            let (plus, minus) = if da * db < 0.0 { (true, true) } else { (da + db > 0.0, da + db < 0.0) };
            marks.push((t, plus, minus));
        }
        marks.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Marks within 1.5 pt of one another are one tick; it is a tick if it has a stroke on both sides or one across.
        let mut places: Vec<f64> = Vec::new();
        let mut i = 0;
        while i < marks.len() {
            let mut j = i;
            let (mut plus, mut minus, mut sum) = (false, false, 0.0);
            while j < marks.len() && marks[j].0 - marks[i].0 <= TERM_NEAR_PT * 2.0 {
                plus |= marks[j].1;
                minus |= marks[j].2;
                sum += marks[j].0;
                j += 1;
            }
            if plus && minus {
                places.push(sum / (j - i) as f64);
            }
            i = j;
        }
        for w in places.windows(2) {
            if w[1] - w[0] >= LETTER_PT {
                out.push(DimLine { between: [at(f.u, f.n, w[0], f.c), at(f.u, f.n, w[1], f.c)] });
            }
        }
    }
    out
}

// ---- tags ----

/// The closed shapes of short straight pieces a plan marks its windows and doors with (a hexagon, a diamond): at least `TAG_PIECES`
/// pieces, none longer than `TAG_PIECE`, end to end in a loop, the whole no more than `TAG_SPAN` across. Each comes back as the
/// places of the lines in it. A tag is not a wall, and its lines are not lines across a wall or inside one (call 188 item 8).
fn tags_of(segs: &[Seg], skip: &[bool]) -> Vec<Vec<usize>> {
    let short: Vec<usize> = (0..segs.len()).filter(|&i| !skip[i] && segs[i].len <= TAG_PIECE).collect();
    let cell = CAP_TOL * 2.0;
    let key = |p: P2| ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
    let mut at_key: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (n, &i) in short.iter().enumerate() {
        for e in [segs[i].a, segs[i].b] {
            let (kx, ky) = key(e);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    at_key.entry((kx + dx, ky + dy)).or_default().push(n);
                }
            }
        }
    }
    let mut group: Vec<usize> = (0..short.len()).collect();
    fn root_of(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        r
    }
    for (n, &i) in short.iter().enumerate() {
        for e in [segs[i].a, segs[i].b] {
            for &m in at_key.get(&key(e)).map(|v| v.as_slice()).unwrap_or(&[]) {
                let j = short[m];
                if m != n && [segs[j].a, segs[j].b].iter().any(|f| norm(sub(*f, e)) <= CAP_TOL) {
                    let (ra, rb) = (root_of(&mut group, n), root_of(&mut group, m));
                    group[ra] = rb;
                }
            }
        }
    }
    let mut comps: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for n in 0..short.len() {
        let r = root_of(&mut group, n);
        comps.entry(r).or_default().push(short[n]);
    }
    let mut out = Vec::new();
    for members in comps.into_values() {
        // The distinct pieces (a line drawn over and over is one piece), and each corner's count of piece ends: a loop has every
        // corner twice (an even number, for pieces drawn twice over).
        let mut pieces: Vec<(i64, i64, i64, i64)> = Vec::new();
        let mut ends: HashMap<(i64, i64), usize> = HashMap::new();
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for &i in &members {
            let (a, b) = (segs[i].a, segs[i].b);
            let (ka, kb) = (key(a), key(b));
            let id = if ka <= kb { (ka.0, ka.1, kb.0, kb.1) } else { (kb.0, kb.1, ka.0, ka.1) };
            if !pieces.contains(&id) {
                pieces.push(id);
                *ends.entry(ka).or_default() += 1;
                *ends.entry(kb).or_default() += 1;
            }
            for p in [a, b] {
                for d in 0..2 {
                    lo[d] = lo[d].min(p[d]);
                    hi[d] = hi[d].max(p[d]);
                }
            }
        }
        if pieces.len() >= TAG_PIECES && ends.values().all(|&n| n == 2) && (hi[0] - lo[0]).max(hi[1] - lo[1]) <= TAG_SPAN {
            out.push(members);
        }
    }
    out
}

// ---- dashed lines: what is overhead or hidden ----

/// Dashed lines side by side, a wall's thickness apart and long enough to be a wall's: each such group is one thing
/// overhead or hidden (a beam, a soffit, cabinets above) or, on a renovation plan, a wall to be demolished. Judged by
/// the rules for a wall's two lines, so that what would have been laid as a wall is said as what it may be.
fn overhead_of(dsegs: &[Seg], k: f64, origin: P2) -> Vec<Overhead> {
    if dsegs.is_empty() {
        return Vec::new();
    }
    let min_len = MIN_FACE.max(LETTER_PT * k);
    let ids: Vec<usize> = (0..dsegs.len()).collect();
    let (faces, _) = faces_of(dsegs, &ids, LETTER_PT * k);
    let long: Vec<bool> = faces.iter().map(|f| f.longest >= min_len).collect();
    let near = neighbours(&faces, &long, min_len);
    // Faces that lie side by side are one group.
    let mut group: Vec<usize> = (0..faces.len()).collect();
    fn root(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        let mut c = i;
        while g[c] != r {
            let n = g[c];
            g[c] = r;
            c = n;
        }
        r
    }
    for (a, list) in near.iter().enumerate() {
        for (b, _, _) in list {
            let (ra, rb) = (root(&mut group, a), root(&mut group, *b));
            group[ra] = rb;
        }
    }
    let mut by: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (f, l) in near.iter().enumerate() {
        if !l.is_empty() {
            let r = root(&mut group, f);
            by.entry(r).or_default().push(f);
        }
    }
    let mut out: Vec<Overhead> = Vec::new();
    for members in by.values() {
        let mut apart: f64 = 0.0;
        let mut centre = [0.0, 0.0];
        let mut n = 0.0;
        for &a in members {
            for (_, d, iv) in &near[a] {
                apart = apart.max(d.abs());
                let (t0, t1) = (iv[0].0, iv[iv.len() - 1].1);
                let c = at(faces[a].u, faces[a].n, (t0 + t1) / 2.0, faces[a].c + d / 2.0);
                centre = [centre[0] + c[0], centre[1] + c[1]];
                n += 1.0;
            }
        }
        let length = members.iter().map(|&f| faces[f].longest).fold(0.0, f64::max);
        out.push(Overhead { lines: members.len(), apart, length, at: shift([centre[0] / n, centre[1] / n], origin) });
    }
    out.sort_by(|a, b| a.at[1].total_cmp(&b.at[1]).then(a.at[0].total_cmp(&b.at[0])));
    out
}

// ---- the finding ----

#[derive(Clone, Debug)]
struct Cand {
    a: usize,
    cc: f64,
    thick: f64,
    t0: f64,
    t1: f64,
    /// From, to, whether it is a window, and whether its faces are cut clean through (not run on past it).
    openings: Vec<(f64, f64, bool, bool)>,
    doubt: Option<String>,
    /// Its doubt is that a line of it is already the face of another wall: it is no wall in the way of the others.
    loser: bool,
    lines: usize,
    /// How many pairs of lines were joined into it (call 188 item 7).
    joined: usize,
}

/// A length as a tape says it, to the quarter inch.
pub fn inch_said(feet: f64) -> String {
    let q = (feet.abs() * 12.0 * 4.0).round() as i64;
    let (ft, rem) = (q / 48, q % 48);
    let (whole, frac) = (rem / 4, rem % 4);
    let f = match frac {
        1 => " 1/4",
        2 => " 1/2",
        3 => " 3/4",
        _ => "",
    };
    format!("{}{}' {}{}\"", if feet < 0.0 && q > 0 { "-" } else { "" }, ft, whole, f)
}

fn shift(p: P2, by: P2) -> P2 {
    [p[0] + by[0], p[1] + by[1]]
}

fn place_said(p: P2) -> String {
    format!("{} east, {} north", inch_said(p[0]), inch_said(p[1]))
}

#[derive(Clone, Copy, PartialEq)]
enum Why {
    Dimension,
    Border,
    Boxed,
    Set,
    Tag,
}

/// The walls a drawn page holds, found from its lines. `feet_per_unit` is the sheet's scale (None:
/// it has none, and nothing is found), `page` its size in its own units, `origin` where the page's
/// lower-left corner lies on the plan, in feet: every place that comes back is a place on the plan.
pub fn find(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2) -> Drawn {
    find_on(lines, feet_per_unit, page, origin, &Extra::default())
}

/// `find`, told which of the lines are dashed and where the page cut lines off.
pub fn find_on(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra) -> Drawn {
    find_all(lines, feet_per_unit, page, origin, extra)
}

/// `find_on`, with the part of the sheet the person says is the plan: a box round it (`within`, corners in feet on the plan), a
/// drawing by its number (`plan`, counted from 1 as `Drawn::drawings` lists them, the largest first), or both: the number then
/// counts the drawings found inside the box. A number the sheet does not have is refused, with how many it has.
pub fn find_in(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra, pick: &Pick) -> Result<Drawn, String> {
    let Some(k) = feet_per_unit.filter(|k| k.is_finite() && *k > 0.0) else {
        return Ok(find_all(lines, feet_per_unit, page, origin, extra));
    };
    // Lines kept to a box, with the dashed marks and the cuts that go with them.
    let kept_in = |bx: [P2; 2]| -> (Vec<[[f64; 2]; 2]>, Vec<usize>, Vec<P2>, usize) {
        let (lo, hi) = ([bx[0][0].min(bx[1][0]), bx[0][1].min(bx[1][1])], [bx[0][0].max(bx[1][0]), bx[0][1].max(bx[1][1])]);
        let slack = INCH / 12.0;
        let inside = |p: P2| {
            let (x, y) = (p[0] * k + origin[0], p[1] * k + origin[1]);
            x >= lo[0] - slack && x <= hi[0] + slack && y >= lo[1] - slack && y <= hi[1] + slack
        };
        let dashed_at: std::collections::HashSet<usize> = extra.dashed.iter().copied().collect();
        let (mut ls, mut ds) = (Vec::new(), Vec::new());
        for (i, l) in lines.iter().enumerate() {
            if inside(l[0]) && inside(l[1]) {
                if dashed_at.contains(&i) {
                    ds.push(*l);
                } else {
                    ls.push(*l);
                }
            }
        }
        let outside = lines.len() - ls.len() - ds.len();
        // The solid lines first, the dashed ones after them, marked.
        let first_dashed = ls.len();
        let marks: Vec<usize> = (first_dashed..first_dashed + ds.len()).collect();
        ls.extend(ds);
        let cuts: Vec<P2> = extra.cuts.iter().copied().filter(|p| inside(*p)).collect();
        (ls, marks, cuts, outside)
    };
    let mut first = if let Some(bx) = pick.within {
        let (ls, marks, cuts, outside) = kept_in(bx);
        let mut d = find_all(&ls, feet_per_unit, page, origin, &Extra { dashed: &marks, cuts: &cuts });
        d.counts.outside = outside;
        d.within = Some(bx);
        d
    } else {
        find_all(lines, feet_per_unit, page, origin, extra)
    };
    let Some(n) = pick.plan else { return Ok(first) };
    let have = first.drawings.len().max(1);
    if n == 0 || n > have {
        return Err(format!("--plan {n}: this sheet has {have} {}, numbered 1 to {have}", if have == 1 { "drawing" } else { "drawings" }));
    }
    if first.drawings.is_empty() {
        first.chosen = Some(1);
        return Ok(first);
    }
    // The drawing's own box, a little wider than its walls' lines.
    let [c0, c1] = first.drawings[n - 1].corners;
    let margin = 1.0;
    let (ls, marks, cuts, outside) = kept_in([[c0[0] - margin, c0[1] - margin], [c1[0] + margin, c1[1] + margin]]);
    let mut d = find_all(&ls, feet_per_unit, page, origin, &Extra { dashed: &marks, cuts: &cuts });
    d.counts.outside = outside + first.counts.outside;
    d.within = first.within;
    d.drawings = first.drawings.clone();
    d.chosen = Some(n);
    Ok(d)
}

fn find_all(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra) -> Drawn {
    let mut out = Drawn::default();
    let Some(k) = feet_per_unit.filter(|k| k.is_finite() && *k > 0.0) else {
        out.no_scale = true;
        return out;
    };
    let min_len = MIN_FACE.max(LETTER_PT * k);
    let dashed_at: std::collections::HashSet<usize> = extra.dashed.iter().copied().collect();
    let mut segs: Vec<Seg> = Vec::new();
    let mut dsegs: Vec<Seg> = Vec::new();
    for (li, l) in lines.iter().enumerate() {
        let a = [l[0][0] * k, l[0][1] * k];
        let b = [l[1][0] * k, l[1][1] * k];
        let len = norm(sub(b, a));
        if a.iter().chain(b.iter()).all(|v| v.is_finite()) && len > 1e-9 {
            // A wall's lines are solid (call 186): a dashed line is looked at for what it may be and for nothing else.
            if dashed_at.contains(&li) {
                dsegs.push(Seg { a, b, len });
            } else {
                segs.push(Seg { a, b, len });
            }
        }
    }
    out.counts.lines = segs.len() + dsegs.len();
    out.counts.dashed = dsegs.len();
    out.overhead = overhead_of(&dsegs, k, origin);
    if segs.len() > MOST_SEGS {
        let before = segs.len();
        segs.retain(|s| s.len >= SHORT_KEPT);
        out.counts.left_out_short_lines = before - segs.len();
        out.notes.push(format!("the sheet draws {before} lines, more than can all be looked at: the {} under 4 in long at scale were left out", before - segs.len()));
    }
    let n_segs = segs.len();
    let mut why: Vec<Option<Why>> = vec![None; n_segs];

    // 1. Dimension lines, with their ticks and extension lines.
    for (i, d) in dimension_lines(&segs, k).iter().enumerate() {
        if *d {
            why[i] = Some(Why::Dimension);
        }
    }

    // 1b. Tags: a window's or a door's mark is no wall, no line across one and no glass (call 188 item 8).
    {
        let skip: Vec<bool> = why.iter().map(|w| w.is_some()).collect();
        for tag in tags_of(&segs, &skip) {
            for i in tag {
                why[i] = Some(Why::Tag);
            }
            out.counts.tags += 1;
        }
    }

    // 2. Boxes: the border, and title blocks, keys and schedules. A box of walls is not one: a ring of
    // walls is two boxes one thickness apart, and a box with lines beside its sides inside it is walls.
    let alive: Vec<usize> = (0..n_segs).filter(|&i| why[i].is_none()).collect();
    let (faces0, of0) = faces_of(&segs, &alive, LETTER_PT * k);
    let long0: Vec<bool> = faces0.iter().map(|f| f.longest >= min_len).collect();
    let near0 = neighbours(&faces0, &long0, min_len);
    let rects = rects_of(&faces0, k);
    let page_area = page[0] * k * page[1] * k;
    let centre_of = |r: &Rect| at(r.u, r.n, (r.t.0 + r.t.1) / 2.0, (r.s.0 + r.s.1) / 2.0);
    for r in &rects {
        let (w, h) = (r.t.1 - r.t.0, r.s.1 - r.s.0);
        let is_border = page_area > 0.0 && w * h >= BORDER_SHARE * page_area;
        if !is_border {
            if w.min(h) <= THICK_MOST {
                continue;
            }
            let c = centre_of(r);
            let ring = rects.iter().any(|q| {
                let along = dot(q.u, r.u);
                if along.abs() < 0.999 && along.abs() > 0.0175 {
                    return false;
                }
                let (qa, qb) = if along.abs() > 0.5 { (q.t.1 - q.t.0, q.s.1 - q.s.0) } else { (q.s.1 - q.s.0, q.t.1 - q.t.0) };
                let d = (w - qa) / 2.0;
                norm(sub(centre_of(q), c)) <= CAP_TOL && (THIN_LEAST - 1e-3..=THICK_MOST + 1e-3).contains(&d) && ((h - qb) / 2.0 - d).abs() <= INCH / 4.0
            });
            // Sides with a line beside them (a wall's other face): a room's inner faces have a wall round
            // them on every side and a ring of walls has one inside; a title block has them at two sides at most.
            let inner = r.sides.iter().filter(|&&s| near0[s].iter().any(|(b, _, _)| !r.sides.contains(b))).count();
            if ring || inner >= 3 {
                continue;
            }
        }
        let slack = TOUCH_PT * k;
        let mut count = 0;
        for &i in &alive {
            if why[i].is_some() {
                continue;
            }
            let s = &segs[i];
            let inside = |p: P2| dot(r.u, p) >= r.t.0 - slack && dot(r.u, p) <= r.t.1 + slack && dot(r.n, p) >= r.s.0 - slack && dot(r.n, p) <= r.s.1 + slack;
            if r.sides.contains(&of0[&i]) {
                why[i] = Some(if is_border { Why::Border } else { Why::Boxed });
                count += usize::from(!is_border);
            } else if !is_border && inside(s.a) && inside(s.b) {
                why[i] = Some(Why::Boxed);
                count += 1;
            }
        }
        if !is_border {
            let corner = at(r.u, r.n, r.t.0, r.s.0);
            out.boxes.push(Boxed { wide: w, high: h, at: corner, lines: count });
        }
    }

    // 3. The faces of what is left.
    let alive: Vec<usize> = (0..n_segs).filter(|&i| why[i].is_none()).collect();
    let (faces, of) = faces_of(&segs, &alive, LETTER_PT * k);
    let nf = faces.len();
    let long: Vec<bool> = faces.iter().map(|f| f.longest >= min_len).collect();
    let near = neighbours(&faces, &long, min_len);

    // Lines inside a wall: a shorter line between two that are a wall's faces (glass, a sash, hatching).
    let mut inside = vec![false; nf];
    for f in 0..nf {
        if !long[f] {
            continue;
        }
        'f: for (p, dp, _) in near[f].iter().filter(|e| e.1 < 0.0) {
            for (q, dq, _) in near[f].iter().filter(|e| e.1 > 0.0) {
                if dq - dp <= THICK_MOST + 1e-6 && faces[f].longest < INSIDE_SHARE * faces[*p].longest.min(faces[*q].longest) {
                    inside[f] = true;
                    break 'f;
                }
            }
        }
    }
    // Evenly spaced runs of three or more are not walls.
    let mut in_set = vec![false; nf];
    {
        let mut order: Vec<usize> = (0..nf).filter(|&i| long[i] && !inside[i]).collect();
        order.sort_by(|&x, &y| ((angle_of(faces[x].u) / ANGLE_PAIR) as i64).cmp(&((angle_of(faces[y].u) / ANGLE_PAIR) as i64)).then(faces[x].c.total_cmp(&faces[y].c)));
        for &start in &order {
            if in_set[start] {
                continue;
            }
            let mut run = vec![start];
            let mut spacing: Option<f64> = None;
            loop {
                let last = *run.last().unwrap();
                let mut next: Option<(usize, f64)> = None;
                for (b, d, _) in &near[last] {
                    if *d <= 0.0 || inside[*b] || in_set[*b] || run.contains(b) {
                        continue;
                    }
                    let ok = spacing.map_or(true, |s| (d - s).abs() <= (EVEN_SHARE * s).max(EVEN_ABS));
                    if ok && next.map_or(true, |(_, nd)| *d < nd) {
                        next = Some((*b, *d));
                    }
                }
                let Some((b, d)) = next else { break };
                spacing.get_or_insert(d);
                run.push(b);
            }
            if run.len() >= 3 {
                let mut centre = [0.0, 0.0];
                for &f in &run {
                    in_set[f] = true;
                    let (p, q) = faces[f].ends(faces[f].iv[0].0, faces[f].iv.last().unwrap().1);
                    centre = [centre[0] + (p[0] + q[0]) / 2.0, centre[1] + (p[1] + q[1]) / 2.0];
                }
                for &i in &alive {
                    if run.contains(&of[&i]) {
                        why[i] = Some(Why::Set);
                    }
                }
                let cnt = run.len() as f64;
                out.sets.push(Set { lines: run.len(), spacing: spacing.unwrap_or(0.0), at: [centre[0] / cnt, centre[1] / cnt] });
            }
        }
    }

    // 4. Pairs and the walls they make.
    let usable = |f: usize| long[f] && !inside[f] && !in_set[f];
    let mut pairs: Vec<(usize, usize, f64, Iv)> = Vec::new();
    for a in 0..nf {
        if usable(a) {
            for (b, d, iv) in &near[a] {
                if *b > a && usable(*b) {
                    pairs.push((a, *b, *d, iv.clone()));
                }
            }
        }
    }
    // A face in two pairs over the same stretch is not a clean pair.
    let np = pairs.len();
    let mut conflict: Vec<Option<String>> = vec![None; np];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); np];
    for i in 0..np {
        for j in (i + 1)..np {
            let Some(f) = [pairs[i].0, pairs[i].1].into_iter().find(|f| *f == pairs[j].0 || *f == pairs[j].1) else { continue };
            let on_f = |p: &(usize, usize, f64, Iv)| if p.0 == f { p.3.clone() } else { faces[p.0].along(&faces[f], &p.3) };
            let shared: f64 = intersect(&on_f(&pairs[i]), &on_f(&pairs[j])).iter().map(|(a, b)| b - a).sum();
            if shared >= 1.0 {
                let said = format!("another line lies beside it ({} and {} from the same line, not evenly spaced): it may be a wall and a line drawn near it", inch_said(pairs[i].2.abs()).replace("0' ", ""), inch_said(pairs[j].2.abs()).replace("0' ", ""));
                conflict[i].get_or_insert(said.clone());
                conflict[j].get_or_insert(said);
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }
    // Pairs that share lines are a stack. A line is the face of one wall only, and a wall drawn in its layers is one wall
    // (call 186). A stack's outermost two lines, if they are a wall's thickness apart and the sheet has other walls of
    // that thickness (or other stacks that outer), are one wall with the lines between them its layers. Else the pair of
    // the thickness most of the sheet's clean walls have claims the lines it uses, and a pair that would use one again
    // is no wall in the way: in doubt, said so. Else it cannot be told, and all of them are in doubt, as they were.
    let mut loser = vec![false; np];
    let mut dropped = vec![false; np];
    let mut layered = vec![0usize; np];
    let mut joined = vec![1usize; np];
    {
        let mut comp = vec![usize::MAX; np];
        let mut comps: Vec<Vec<usize>> = Vec::new();
        for i in 0..np {
            if comp[i] != usize::MAX {
                continue;
            }
            let c = comps.len();
            let mut stack = vec![i];
            comp[i] = c;
            let mut members = Vec::new();
            while let Some(x) = stack.pop() {
                members.push(x);
                for &y in &adj[x] {
                    if comp[y] == usize::MAX {
                        comp[y] = c;
                        stack.push(y);
                    }
                }
            }
            members.sort_unstable();
            comps.push(members);
        }
        // Each component's outermost pair, where it has one, and what it holds.
        struct Stack {
            outer: Option<usize>,
            faces: usize,
            /// Each face's place across the stack, in feet from the first face, and where the outermost two are.
            offs: Vec<f64>,
            span: (f64, f64),
        }
        let mut stacks: Vec<Stack> = Vec::new();
        for m in &comps {
            if m.len() == 1 {
                stacks.push(Stack { outer: Some(m[0]), faces: 2, offs: Vec::new(), span: (0.0, 0.0) });
                continue;
            }
            let mut fs: Vec<usize> = m.iter().flat_map(|&i| [pairs[i].0, pairs[i].1]).collect();
            fs.sort_unstable();
            fs.dedup();
            let r = pairs[m[0]].0;
            let t_mid = pairs[m[0]].3.first().map(|x| (x.0 + x.1) / 2.0).unwrap_or(0.0);
            let off = |f: usize| if f == r { 0.0 } else { faces[f].offset_from(&faces[r], t_mid) };
            let lo = *fs.iter().min_by(|&&x, &&y| off(x).total_cmp(&off(y))).unwrap();
            let hi = *fs.iter().max_by(|&&x, &&y| off(x).total_cmp(&off(y))).unwrap();
            let outer = m.iter().copied().find(|&i| (pairs[i].0 == lo && pairs[i].1 == hi) || (pairs[i].0 == hi && pairs[i].1 == lo));
            let offs: Vec<f64> = fs.iter().map(|&f| off(f)).collect();
            stacks.push(Stack { outer, faces: fs.len(), offs, span: (off(lo), off(hi)) });
        }
        let span = |c: usize| stacks[c].outer.map(|i| pairs[i].2.abs());
        for (c, m) in comps.iter().enumerate() {
            if m.len() == 1 {
                continue;
            }
            // 1. Layers: the outer pair's thickness is one other walls or stacks have.
            if let (Some(o), Some(sp)) = (stacks[c].outer, span(c)) {
                let others = (0..comps.len()).filter(|&c2| c2 != c && span(c2).is_some_and(|s2| (s2 - sp).abs() <= THICK_SAME)).count();
                if others >= 1 && stacks[c].faces >= 3 {
                    for &i in m {
                        dropped[i] = i != o;
                        conflict[i] = None;
                    }
                    layered[o] = stacks[c].faces;
                    joined[o] = m.len();
                    continue;
                }
                // 1b. A wall whose faces are each drawn twice (call 188 item 7): every line of the stack is within a finish's
                // depth of one of its two outermost lines. It is one wall, whether or not another wall has its span.
                if stacks[c].faces >= 3 && stacks[c].offs.iter().all(|&x| (x - stacks[c].span.0).abs() <= DOUBLE_MAX + 1e-9 || (x - stacks[c].span.1).abs() <= DOUBLE_MAX + 1e-9) {
                    for &i in m {
                        dropped[i] = i != o;
                        conflict[i] = None;
                    }
                    layered[o] = stacks[c].faces;
                    joined[o] = m.len();
                    continue;
                }
            }
            // 2. The pair of the thickness most clean walls have claims its lines.
            let strength = |i: usize| (0..comps.len()).filter(|&c2| comps[c2].len() == 1 && (pairs[comps[c2][0]].2.abs() - pairs[i].2.abs()).abs() <= THICK_SAME).count();
            let mut ranked: Vec<(usize, usize)> = m.iter().map(|&i| (strength(i), i)).filter(|(n, _)| *n >= 1).collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            let mut resolved = false;
            if let Some(&(top, w)) = ranked.first() {
                // A tie between two pairs that do not share a line is no tie: the claim is on the lines of one pair.
                let tie = ranked.iter().skip(1).any(|&(n, i)| n == top && adj[w].contains(&i));
                if !tie {
                    resolved = true;
                    // A pair of the stack that shares no line with it is the same wall over the same stretch, and joined (call 188).
                    for &i in m {
                        if i != w && !adj[w].contains(&i) {
                            dropped[i] = true;
                            conflict[i] = None;
                            joined[w] += 1;
                        }
                    }
                    let said = format!("another line lies beside it: one of its lines is already the face of a wall {} thick, so it is not a second wall (a counter's edge, a ledge, a line drawn near a wall)", inch_said(pairs[w].2.abs()).replace("0' ", ""));
                    conflict[w] = None;
                    // The line beside it is said once, as the nearest pair it would make; the others over the same line are
                    // the same thing.
                    let mut by_line: BTreeMap<usize, usize> = BTreeMap::new();
                    for &i in m {
                        if i != w && adj[w].contains(&i) {
                            let line = [pairs[i].0, pairs[i].1].into_iter().find(|f| *f != pairs[w].0 && *f != pairs[w].1).unwrap_or(pairs[i].1);
                            match by_line.get(&line) {
                                Some(&keep) if pairs[keep].2.abs() <= pairs[i].2.abs() => dropped[i] = true,
                                Some(&keep) => {
                                    dropped[keep] = true;
                                    loser[keep] = false;
                                    by_line.insert(line, i);
                                }
                                None => {
                                    by_line.insert(line, i);
                                }
                            }
                        }
                    }
                    for &i in by_line.values() {
                        conflict[i] = Some(said.clone());
                        loser[i] = true;
                    }
                }
            }
            // 3. It cannot be told which pair is the wall: the pairs over one stretch are one thing, from the outermost line to the
            // outermost, in doubt (call 188 item 7): found once, with how many pairs it was made of.
            if !resolved {
                let o = stacks[c].outer.unwrap_or_else(|| *m.iter().max_by(|&&x, &&y| pairs[x].2.abs().total_cmp(&pairs[y].2.abs())).unwrap());
                for &i in m {
                    dropped[i] = i != o;
                }
                conflict[o] = conflict[o].take().or_else(|| m.iter().find_map(|&i| conflict[i].clone()));
                joined[o] = m.len();
            }
        }
    }

    let mut cands: Vec<Cand> = Vec::new();
    for (pi, (a, b, d, ov)) in pairs.iter().enumerate() {
        if dropped[pi] {
            continue;
        }
        let (fa, fb) = (&faces[*a], &faces[*b]);
        let cc = fa.c + d / 2.0;
        let thick = d.abs();
        let b_in_a = fb.along(fa, &fb.iv);
        let both: Iv = intersect(&fa.iv, &b_in_a).into_iter().filter(|&(lo, hi)| ov.iter().any(|&(o0, o1)| lo < o1 && hi > o0)).collect();
        if both.is_empty() {
            continue;
        }
        let union = merged(fa.iv.iter().chain(b_in_a.iter()).copied().collect(), 0.0);
        // Lines across the pair: square to it, with an end on each of its lines. A line drawn in pieces
        // (a cap drawn as two halves) is one line.
        let mut crosses: Vec<f64> = Vec::new();
        for g in &faces {
            if dot(g.u, fa.u).abs() > CROSS_ANGLE {
                continue;
            }
            // The pieces of this line that lie between the pair's two lines, none reaching past either, joined:
            // a cap drawn in halves is one cap, and a room line or a grid line that runs on through the wall
            // (and a dashed one) is none, by itself or in company with the cap it crosses.
            // A line across may run past the faces by less than the wall's thickness and be a line across all the same
            // (call 186): a cap drawn a little long. Past that it runs on through the wall (a room line, a grid line).
            let (lo, hi) = (0.0_f64.min(*d) - thick - CAP_TOL, 0.0_f64.max(*d) + thick + CAP_TOL);
            let mut inside: Vec<(f64, f64)> = Vec::new();
            let mut at_t = 0.0;
            for &(x, y) in &g.raw {
                let (p, q) = g.ends(x, y);
                let (e0, e1) = (dot(fa.n, p) - fa.c, dot(fa.n, q) - fa.c);
                if e0 >= lo && e0 <= hi && e1 >= lo && e1 <= hi {
                    inside.push((e0.min(e1), e0.max(e1)));
                    at_t = (dot(fa.u, p) + dot(fa.u, q)) / 2.0;
                }
            }
            let across = merged(inside, CAP_TOL);
            if across.iter().any(|&(e0, e1)| e0 <= 0.0_f64.min(*d) + CAP_TOL && e1 >= 0.0_f64.max(*d) - CAP_TOL) {
                crosses.push(at_t);
            }
        }
        crosses.sort_by(|x, y| x.total_cmp(y));
        crosses.dedup_by(|x, y| (*x - *y).abs() < CAP_TOL);
        // Groups of stretches: joints join, openings are kept, a long break in one line splits.
        let mut groups: Vec<(Iv, Vec<(f64, f64)>)> = Vec::new();
        let mut cur: (Iv, Vec<(f64, f64)>) = (vec![both[0]], Vec::new());
        for w in both.windows(2) {
            let (g0, g1) = (w[0].1, w[1].0);
            // The widest stretch of the break where neither line is drawn.
            let mut none = (0.0, 0.0);
            let mut from = g0;
            for &(x, y) in union.iter().filter(|&&(x, y)| y > g0 && x < g1) {
                if x > from && x.min(g1) - from > none.1 - none.0 {
                    none = (from, x.min(g1));
                }
                from = from.max(y);
            }
            if from < g1 && g1 - from > none.1 - none.0 {
                none = (from, g1);
            }
            if none.1 - none.0 > OPENING_MOST {
                groups.push(std::mem::replace(&mut cur, (vec![w[1]], Vec::new())));
                out.notes.push(format!("both lines of a wall stop for {} at {}, wider than any opening: taken as one wall ending and another beginning", inch_said(none.1 - none.0), place_said(shift(at(fa.u, fa.n, none.0, cc), origin))));
            } else if none.1 - none.0 > JOINT_MOST {
                cur.1.push(none);
                cur.0.push(w[1]);
            } else if g1 - g0 <= JOINT_MOST {
                if g1 - g0 > MERGE_GAP {
                    out.counts.joints += 1;
                }
                cur.0.push(w[1]);
            } else {
                groups.push(std::mem::replace(&mut cur, (vec![w[1]], Vec::new())));
                out.notes.push(format!("one line of a wall goes on for {} past the other, at {}: the wall is taken to end there", inch_said(g1 - g0), place_said(shift(at(fa.u, fa.n, g0, cc), origin))));
            }
        }
        groups.push(cur);
        let (lo, hi) = (0.0_f64.min(*d), 0.0_f64.max(*d));
        // Lines across a stretch, inside the wall's thickness: lines between the two faces, and a face
        // drawn again across the stretch, as a line of its own from jamb to jamb.
        let across = |s0: f64, s1: f64| -> usize {
            let inner = faces
                .iter()
                .enumerate()
                .filter(|(gi, g)| {
                    if gi == a || gi == b || cross(g.u, fa.u).abs() > ANGLE_PAIR {
                        return false;
                    }
                    let off = g.offset_from(fa, (s0 + s1) / 2.0);
                    off > lo + INCH / 16.0 && off < hi - INCH / 16.0 && covered(&g.along(fa, &g.iv), s0, s1) >= GLASS_SHARE * (s1 - s0)
                })
                .count();
            // A face drawn again across the stretch: a line of its own from one jamb to the other.
            let redrawn = [fa, fb].iter().filter(|f| f.along(fa, &f.raw).iter().any(|&(x, y)| (x - s0).abs() <= CAP_TOL && (y - s1).abs() <= CAP_TOL)).count();
            inner + redrawn
        };
        for (pieces, mut opens) in groups {
            let (g_lo, g_hi) = (pieces[0].0, pieces.last().unwrap().1);
            let cont = |t: f64| pieces.iter().any(|&(x, y)| t >= x + CAP_TOL && t <= y - CAP_TOL);
            // A line across within a thickness of the end of a piece, where the next piece or the opening begins, is the
            // jamb of that opening with the faces run on past it, as drafters draw them (call 186): it ends no wall.
            let jamb = |t: f64| pieces.iter().any(|&(x, y)| (x > g_lo + 1e-9 && (t - x).abs() < thick) || (y < g_hi - 1e-9 && (t - y).abs() < thick));
            let mine: Vec<f64> = crosses.iter().copied().filter(|&t| t > g_lo && t < g_hi && cont(t) && !jamb(t)).collect();
            // An opening's sides are its jambs where the faces run on past them: the nearest line across within a thickness
            // of where the faces stop.
            for o in opens.iter_mut() {
                let near_to = |at: f64| crosses.iter().copied().filter(|&t| (t - at).abs() > CAP_TOL && (t - at).abs() < thick).min_by(|p, q| (p - at).abs().total_cmp(&(q - at).abs()));
                if let Some(t) = near_to(o.0).filter(|&t| t < o.0) {
                    o.0 = t;
                }
                if let Some(t) = near_to(o.1).filter(|&t| t > o.1) {
                    o.1 = t;
                }
            }
            // Lines across where both lines go on: a window's jambs, or the end of a wall. (What is in `opens` so far is cut clean
            // through both lines; what is added below has its faces running on.)
            let n_cut = opens.len();
            let mut ends: Vec<f64> = Vec::new();
            let mut i = 0;
            while i < mine.len() {
                if i + 1 < mine.len() {
                    let (t0, t1) = (mine[i], mine[i + 1]);
                    if pieces.iter().any(|&(x, y)| t0 >= x && t1 <= y) && t1 - t0 > JOINT_MOST && t1 - t0 <= OPENING_MOST && across(t0, t1) >= 1 {
                        opens.push((t0, t1));
                        i += 2;
                        continue;
                    }
                }
                ends.push(mine[i]);
                i += 1;
            }
            // A line across ends a wall only if it stands alone: both walls it makes are long enough to be
            // walls and no other line across is nearer than that (hatching across a wall is a row of them).
            let mut bounds = vec![g_lo];
            for (ix, &t) in ends.iter().enumerate() {
                let before = if ix == 0 { g_lo } else { ends[ix - 1] };
                let after = ends.get(ix + 1).copied().unwrap_or(g_hi);
                if t - before >= min_len && after - t >= min_len {
                    bounds.push(t);
                } else {
                    out.counts.crosses_ignored += 1;
                }
            }
            bounds.push(g_hi);
            for w in bounds.windows(2) {
                let (s0, s1) = (w[0], w[1]);
                // Its whole line, not a piece of it between openings (call 186).
                let inside_it: Vec<&(f64, f64)> = pieces.iter().filter(|&&(x, y)| y > s0 + 1e-9 && x < s1 - 1e-9).collect();
                let whole = inside_it.iter().map(|p| p.1.min(s1)).fold(0.0, f64::max) - inside_it.iter().map(|p| p.0.max(s0)).fold(f64::MAX, f64::min);
                if whole < min_len {
                    out.counts.short_stretches += 1;
                    continue;
                }
                let mut ops: Vec<(f64, f64, bool, bool)> = opens
                    .iter()
                    .enumerate()
                    .filter(|&(_, &(x, y))| x >= s0 - 1e-9 && y <= s1 + 1e-9)
                    .map(|(ix, &(x, y))| (x, y, across(x, y) >= 2, ix < n_cut))
                    .collect();
                ops.sort_by(|p, q| p.0.total_cmp(&q.0));
                cands.push(Cand { a: *a, cc, thick, t0: s0, t1: s1, openings: ops, doubt: conflict[pi].clone(), loser: loser[pi], lines: layered[pi], joined: joined[pi] });
            }
        }
    }

    // 5. Carry ends to meet; which walls meet nothing.
    let nw = cands.len();
    let line_of = |c: &Cand| -> (P2, P2, P2) {
        let fa = &faces[c.a];
        (at(fa.u, fa.n, c.t0, c.cc), at(fa.u, fa.n, c.t1, c.cc), fa.u)
    };
    let mut ends: Vec<[P2; 2]> = Vec::with_capacity(nw);
    let mut t_ends: Vec<[f64; 2]> = Vec::with_capacity(nw);
    let mut meets: Vec<[bool; 2]> = vec![[false; 2]; nw];
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for i in 0..nw {
        let (p, q, ui) = line_of(&cands[i]);
        let (mut e, mut te) = ([p, q], [cands[i].t0, cands[i].t1]);
        for end in 0..2 {
            let me = if end == 0 { p } else { q };
            let mut best: Option<(f64, P2, usize)> = None;
            let mut touch: Option<usize> = None;
            for j in 0..nw {
                if j == i || cands[j].loser {
                    continue;
                }
                let (pj, qj, uj) = line_of(&cands[j]);
                let reach = cands[i].thick.max(cands[j].thick);
                let sinv = cross(ui, uj);
                if sinv.abs() < 0.087 {
                    // Going on in line: its end is at this one.
                    if dot(perp(uj), sub(me, pj)).abs() <= reach / 2.0 + CAP_TOL && norm(sub(me, pj)).min(norm(sub(me, qj))) <= reach {
                        touch = Some(j);
                    }
                    continue;
                }
                // Where this wall's line meets that one's.
                let s = cross(sub(pj, me), uj) / sinv;
                let x = [me[0] + ui[0] * s, me[1] + ui[1] * s];
                let along_j = dot(uj, sub(x, pj));
                if s.abs() <= reach && along_j >= -cands[i].thick && along_j <= norm(sub(qj, pj)) + cands[i].thick && best.map_or(true, |(bd, _, _)| s.abs() < bd) {
                    best = Some((s.abs(), x, j));
                }
            }
            if let Some((_, x, j)) = best {
                e[end] = x;
                te[end] = dot(ui, x);
                meets[i][end] = true;
                edges.push((i, j));
            } else if let Some(j) = touch {
                meets[i][end] = true;
                edges.push((i, j));
            }
        }
        ends.push(e);
        t_ends.push(te);
    }
    for i in 0..nw {
        if cands[i].doubt.is_none() && !meets[i][0] && !meets[i][1] {
            let len = norm(sub(ends[i][1], ends[i][0]));
            cands[i].doubt = Some(format!("two lines {} apart and {} long that meet no other wall at either end (a counter's edge, a beam shown, a stair's side)", inch_said(cands[i].thick).replace("0' ", ""), inch_said(len)));
        }
    }
    // The plan is the largest group of walls that meet one another; a group far smaller than it, apart from it, is another
    // plan, a detail or a section (call 186, item 3).
    let mut group: Vec<usize> = (0..nw).collect();
    fn root_g(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        r
    }
    for &(a, b) in &edges {
        let (ra, rb) = (root_g(&mut group, a), root_g(&mut group, b));
        group[ra] = rb;
    }
    let mut total: BTreeMap<usize, f64> = BTreeMap::new();
    for i in 0..nw {
        if !cands[i].loser {
            *total.entry(root_g(&mut group, i)).or_default() += norm(sub(ends[i][1], ends[i][0]));
        }
    }
    {
        if let Some((&plan, &plan_len)) = total.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
            for i in 0..nw {
                let r = root_g(&mut group, i);
                if r != plan && cands[i].doubt.is_none() && total[&r] < APART_SHARE * plan_len {
                    cands[i].doubt = Some(format!("apart from the plan: it meets no wall of the largest group of walls on the sheet ({} of wall), and the group it is in is {} long: another plan, a detail or a section", inch_said(plan_len), inch_said(total[&r])));
                }
            }
        }
    }
    // A pair thicker than every other wall found, and alone at that thickness, is in doubt (call 186, narrowed): a closet
    // shelf, a bench, a ledge. A thin partition is not, nor two thick walls, nor a sheet of one wall.
    for i in 0..nw {
        if cands[i].doubt.is_some() || cands[i].loser {
            continue;
        }
        let others: Vec<f64> = (0..nw).filter(|&j| j != i && !cands[j].loser).map(|j| cands[j].thick).collect();
        if !others.is_empty() && others.iter().all(|&t| cands[i].thick >= t + THICK_MORE) {
            cands[i].doubt = Some(format!("two lines {} apart, thicker than every other wall found on the sheet ({} at most) and the only one that thick (a closet shelf, a bench, a ledge)", inch_said(cands[i].thick).replace("0' ", ""), inch_said(others.iter().cloned().fold(0.0, f64::max)).replace("0' ", "")));
        }
    }

    // A window drawn by lines across a stretch of a wall whose two faces run on past it, in a wall with a room on both sides, is no
    // window (call 188 item 8): a shower's glass, a pocket or sliding door's leaf drawn in a partition. A window is in a wall with a
    // face to the outside, or one cut clean through. The outside is told by looking straight out from the stretch on both sides: where
    // one side meets no wall of the plan, the wall has a face to the outside.
    let roots: Vec<usize> = (0..nw).map(|i| root_g(&mut group, i)).collect();
    for i in 0..nw {
        if cands[i].openings.iter().all(|o| !o.2 || o.3) {
            continue;
        }
        let fa = &faces[cands[i].a];
        let hit = |p: P2, dir: P2| -> bool {
            (0..nw).any(|j| {
                if j == i || cands[j].loser || cands[j].doubt.is_some() || roots[j] != roots[i] {
                    return false;
                }
                let (q0, q1) = (ends[j][0], ends[j][1]);
                let d = sub(q1, q0);
                let len = norm(d);
                if len < 1e-9 {
                    return false;
                }
                let uj = [d[0] / len, d[1] / len];
                let den = cross(dir, uj);
                if den.abs() < 0.087 {
                    return false;
                }
                let w = sub(q0, p);
                let (sd, td) = (cross(w, uj) / den, cross(w, dir) / den);
                sd > cands[i].thick / 2.0 && td >= -cands[j].thick && td <= len + cands[j].thick
            })
        };
        let mut kept = Vec::new();
        for o in cands[i].openings.clone() {
            if o.2 && !o.3 {
                let p = at(fa.u, fa.n, (o.0 + o.1) / 2.0, cands[i].cc);
                let outside = !hit(p, fa.n) || !hit(p, [-fa.n[0], -fa.n[1]]);
                if !outside {
                    out.counts.glass_inside += 1;
                    out.notes.push(format!("lines across a stretch {} wide of a wall at {}, and lines inside it, are not read as a window: both its faces run on past it and it has a room on each side (a shower's glass, or a sliding or pocket door's leaf drawn in the wall); a window is in a wall with a face to the outside, or one cut clean through", inch_said(o.1 - o.0), place_said(shift(p, origin))));
                    continue;
                }
            }
            kept.push(o);
        }
        cands[i].openings = kept;
    }

    // 6. The walls, each from its first end: the west end of one that runs east and west, else the south.
    let mid_of = |i: usize| [(ends[i][0][0] + ends[i][1][0]) / 2.0, (ends[i][0][1] + ends[i][1][1]) / 2.0];
    let mut order: Vec<usize> = (0..nw).collect();
    order.sort_by(|&x, &y| {
        let (mx, my) = (mid_of(x), mid_of(y));
        (mx[1] * 100.0).round().total_cmp(&(my[1] * 100.0).round()).then((mx[0] * 100.0).round().total_cmp(&(my[0] * 100.0).round()))
    });
    let mut new_index = vec![0usize; nw];
    for (rank, &i) in order.iter().enumerate() {
        new_index[i] = rank;
        let c = &cands[i];
        let (mut p, mut q) = (ends[i][0], ends[i][1]);
        let [t0, t1] = t_ends[i];
        let d = sub(q, p);
        let flip = if d[0].abs() >= d[1].abs() { d[0] < 0.0 } else { d[1] < 0.0 };
        // `t` grows from the unflipped first end; a flipped wall is measured from the other end.
        let mut ops: Vec<Opening> = c.openings.iter().map(|&(x, y, w, _)| if flip { Opening { window: w, from: t1 - y, to: t1 - x } } else { Opening { window: w, from: x - t0, to: y - t0 } }).collect();
        if flip {
            std::mem::swap(&mut p, &mut q);
        }
        ops.sort_by(|x, y| x.from.total_cmp(&y.from));
        out.walls.push(Wall { from: p, to: q, thickness: c.thick, openings: ops, doubt: c.doubt.clone(), lines: c.lines, joined: c.joined });
    }
    // The drawings on the sheet (call 188 item 6, as sent back): told from EVERY wall found, in doubt or not, so a plan whose walls are
    // all in doubt is still a drawing. Walls lying within `DRAWING_GAP` of one another (nearest points of their centre lines) are one
    // drawing: a plan's pieces need not meet to belong together, and a roof plan stands far from the floor plan. The size of a drawing
    // is all the wall found in it (a loser, a line already a wall's face, is not wall); two of like size are listed, largest first.
    {
        let mut cl: Vec<usize> = (0..nw).collect();
        fn root_c(g: &mut [usize], i: usize) -> usize {
            let mut r = i;
            while g[r] != r {
                r = g[r];
            }
            r
        }
        let seg_gap = |i: usize, j: usize| -> f64 {
            let (a, b, c, d) = (ends[i][0], ends[i][1], ends[j][0], ends[j][1]);
            let to_seg = |p: P2, q0: P2, q1: P2| {
                let v = sub(q1, q0);
                let t = (dot(sub(p, q0), v) / dot(v, v).max(1e-300)).clamp(0.0, 1.0);
                norm(sub(p, [q0[0] + v[0] * t, q0[1] + v[1] * t]))
            };
            let crossing = {
                let (d1, d2) = (cross(sub(b, a), sub(c, a)), cross(sub(b, a), sub(d, a)));
                let (d3, d4) = (cross(sub(d, c), sub(a, c)), cross(sub(d, c), sub(b, c)));
                d1 * d2 < 0.0 && d3 * d4 < 0.0
            };
            if crossing {
                0.0
            } else {
                to_seg(a, c, d).min(to_seg(b, c, d)).min(to_seg(c, a, b)).min(to_seg(d, a, b))
            }
        };
        for i in 0..nw {
            for j in (i + 1)..nw {
                if seg_gap(i, j) <= DRAWING_GAP {
                    let (ra, rb) = (root_c(&mut cl, i), root_c(&mut cl, j));
                    cl[ra] = rb;
                }
            }
        }
        let mut sizes: BTreeMap<usize, (f64, f64)> = BTreeMap::new();
        for i in 0..nw {
            let r = root_c(&mut cl, i);
            let e = sizes.entry(r).or_default();
            if !cands[i].loser {
                let l = norm(sub(ends[i][1], ends[i][0]));
                e.0 += l;
                if cands[i].doubt.is_some() {
                    e.1 += l;
                }
            }
        }
        let largest = sizes.values().map(|v| v.0).fold(0.0, f64::max);
        let mut like: Vec<(usize, f64, f64)> = sizes.iter().filter(|(_, v)| v.0 >= LIKE_SIZE * largest && v.0 > 0.0).map(|(&r, v)| (r, v.0, v.1)).collect();
        if like.len() >= 2 {
            for (root, length, in_doubt) in like.drain(..) {
                let mut members: Vec<usize> = (0..nw).filter(|&i| root_c(&mut cl, i) == root).collect();
                members.sort_by_key(|&i| new_index[i]);
                let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
                for &i in &members {
                    for e in ends[i] {
                        for d in 0..2 {
                            lo[d] = lo[d].min(e[d]);
                            hi[d] = hi[d].max(e[d]);
                        }
                    }
                }
                out.drawings.push(Plan { walls: members.iter().map(|&i| new_index[i]).collect(), length, in_doubt, corners: [lo, hi] });
            }
            // Ties in size are told apart by where they lie: the westmost first.
            out.drawings.sort_by(|a, b| b.length.partial_cmp(&a.length).unwrap_or(std::cmp::Ordering::Equal).then(a.corners[0][0].total_cmp(&b.corners[0][0])));
        }
    }
    // An end that meets nothing is open, unless the page cut the lines off there: where a clip or the page's own edge ends
    // what shows, the wall may go on beyond it (call 186).
    let cuts: Vec<P2> = extra.cuts.iter().map(|p| [p[0] * k, p[1] * k]).collect();
    // (A wall in doubt that runs to the edge of what shows is said to be cut off as well: what is said of the walls closing
    // rests on what they meet.)
    for i in 0..nw {
        for end in 0..2 {
            if !meets[i][end] {
                let at_cut = cuts.iter().any(|c| norm(sub(*c, ends[i][end])) <= cands[i].thick + CAP_TOL);
                if at_cut {
                    out.cut_ends.push((new_index[i], ends[i][end]));
                } else if cands[i].doubt.is_none() {
                    out.open_ends.push((new_index[i], ends[i][end]));
                }
            }
        }
    }
    out.open_ends.sort_by(|x, y| x.0.cmp(&y.0));
    out.cut_ends.sort_by(|x, y| x.0.cmp(&y.0));

    // 7. What each line was, counted by kind.
    for i in 0..n_segs {
        match why[i] {
            Some(Why::Dimension) => out.counts.dimension += 1,
            Some(Why::Border) => out.counts.border += 1,
            Some(Why::Boxed) => out.counts.boxed += 1,
            Some(Why::Set) => out.counts.set += 1,
            Some(Why::Tag) => {}
            None => {
                let s = &segs[i];
                let m = [(s.a[0] + s.b[0]) / 2.0, (s.a[1] + s.b[1]) / 2.0];
                let in_wall = out.walls.iter().any(|w| {
                    let u = unit(sub(w.to, w.from));
                    let (t, o) = (dot(u, sub(m, w.from)), dot(perp(u), sub(m, w.from)));
                    t >= -w.thickness && t <= w.length() + w.thickness && o.abs() <= w.thickness / 2.0 + CAP_TOL
                });
                if in_wall {
                    out.counts.in_walls += 1;
                } else if s.len < min_len {
                    out.counts.small += 1;
                } else {
                    out.counts.single += 1;
                }
            }
        }
    }
    // Long lines left over that meet end to end in an outline (a house or a room drawn with one line a wall): a thickness
    // cannot be read from one line, and it is said (call 186).
    {
        let single: Vec<usize> = (0..n_segs)
            .filter(|&i| why[i].is_none() && segs[i].len >= min_len && {
                let m = [(segs[i].a[0] + segs[i].b[0]) / 2.0, (segs[i].a[1] + segs[i].b[1]) / 2.0];
                !out.walls.iter().any(|w| {
                    let u = unit(sub(w.to, w.from));
                    let (t, o) = (dot(u, sub(m, w.from)), dot(perp(u), sub(m, w.from)));
                    t >= -w.thickness && t <= w.length() + w.thickness && o.abs() <= w.thickness / 2.0 + CAP_TOL
                })
            })
            .collect();
        let cell = CAP_TOL * 2.0;
        let key = |p: P2| ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
        let mut at_key: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (n, &i) in single.iter().enumerate() {
            for e in [segs[i].a, segs[i].b] {
                let (kx, ky) = key(e);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        at_key.entry((kx + dx, ky + dy)).or_default().push(n);
                    }
                }
            }
        }
        let mut group: Vec<usize> = (0..single.len()).collect();
        fn root_of(g: &mut [usize], i: usize) -> usize {
            let mut r = i;
            while g[r] != r {
                r = g[r];
            }
            r
        }
        for (n, &i) in single.iter().enumerate() {
            for e in [segs[i].a, segs[i].b] {
                for &m in at_key.get(&key(e)).map(|v| v.as_slice()).unwrap_or(&[]) {
                    let j = single[m];
                    if m != n && [segs[j].a, segs[j].b].iter().any(|f| norm(sub(*f, e)) <= CAP_TOL) {
                        let (ra, rb) = (root_of(&mut group, n), root_of(&mut group, m));
                        group[ra] = rb;
                    }
                }
            }
        }
        let mut comps: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for n in 0..single.len() {
            let r = root_of(&mut group, n);
            comps.entry(r).or_default().push(single[n]);
        }
        for members in comps.values().filter(|m| m.len() >= 3) {
            let c = members.iter().fold([0.0, 0.0], |m, &j| [m[0] + (segs[j].a[0] + segs[j].b[0]) / 2.0, m[1] + (segs[j].a[1] + segs[j].b[1]) / 2.0]);
            out.single_outlines.push((members.len(), shift([c[0] / members.len() as f64, c[1] / members.len() as f64], origin)));
        }
        // Said only of a sheet that found no wall: where walls were found, a north arrow or a symbol is an outline of lines
        // end to end too, and the walls are not drawn as single lines.
        if out.walls.iter().any(|w| w.doubt.is_none()) {
            out.single_outlines.clear();
        }
        out.single_outlines.sort_by(|a, b| a.1[1].total_cmp(&b.1[1]).then(a.1[0].total_cmp(&b.1[0])));
    }
    // Short pieces that run on end to end and turn a little at each joint: a curve drawn in straight pieces.
    {
        let small: Vec<usize> = (0..n_segs)
            .filter(|&i| why[i].is_none() && segs[i].len < min_len && {
                let m = [(segs[i].a[0] + segs[i].b[0]) / 2.0, (segs[i].a[1] + segs[i].b[1]) / 2.0];
                !out.walls.iter().any(|w| {
                    let u = unit(sub(w.to, w.from));
                    let (t, o) = (dot(u, sub(m, w.from)), dot(perp(u), sub(m, w.from)));
                    t >= -w.thickness && t <= w.length() + w.thickness && o.abs() <= w.thickness / 2.0 + CAP_TOL
                })
            })
            .collect();
        let cell = INCH / 2.0;
        let key = |p: P2| ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
        let mut at_key: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for &i in &small {
            at_key.entry(key(segs[i].a)).or_default().push(i);
            at_key.entry(key(segs[i].b)).or_default().push(i);
        }
        let mut seen: std::collections::HashSet<usize> = Default::default();
        for &i in &small {
            if seen.contains(&i) {
                continue;
            }
            // The run through this piece, followed out from it both ways while each joint has exactly two pieces.
            let mut run: Vec<usize> = vec![i];
            seen.insert(i);
            let mut smooth = true;
            for end in 0..2 {
                let (mut cur, mut p) = (i, if end == 0 { segs[i].a } else { segs[i].b });
                loop {
                    let here: Vec<usize> = at_key.get(&key(p)).map(|v| v.iter().copied().filter(|&j| j != cur).collect()).unwrap_or_default();
                    if here.len() != 1 || seen.contains(&here[0]) {
                        break;
                    }
                    let n = here[0];
                    let (da, db) = (unit(sub(segs[cur].b, segs[cur].a)), unit(sub(segs[n].b, segs[n].a)));
                    let turn = dot(da, db).abs().clamp(0.0, 1.0).acos().to_degrees();
                    if turn > CURVE_TURN {
                        smooth = false;
                        break;
                    }
                    seen.insert(n);
                    run.push(n);
                    let far = if norm(sub(segs[n].a, p)) <= norm(sub(segs[n].b, p)) { segs[n].b } else { segs[n].a };
                    p = far;
                    cur = n;
                }
            }
            let total: f64 = run.iter().map(|&j| segs[j].len).sum();
            if smooth && run.len() >= CURVE_PIECES && total >= CURVE_LONG {
                let c = run.iter().fold([0.0, 0.0], |m, &j| [m[0] + (segs[j].a[0] + segs[j].b[0]) / 2.0, m[1] + (segs[j].a[1] + segs[j].b[1]) / 2.0]);
                out.curved.push((run.len(), total, shift([c[0] / run.len() as f64, c[1] / run.len() as f64], origin)));
            }
        }
    }
    // Places on the plan.
    for w in out.walls.iter_mut() {
        w.from = shift(w.from, origin);
        w.to = shift(w.to, origin);
    }
    for e in out.open_ends.iter_mut().chain(out.cut_ends.iter_mut()) {
        e.1 = shift(e.1, origin);
    }
    for st in out.sets.iter_mut() {
        st.at = shift(st.at, origin);
    }
    for p in out.drawings.iter_mut() {
        p.corners = [shift(p.corners[0], origin), shift(p.corners[1], origin)];
    }
    for b in out.boxes.iter_mut() {
        b.at = shift(b.at, origin);
    }
    out
}

// ---- the sheet's own dimension strings (call 188 item 9) ----

/// One dimension string the page holds as words, with the dimension line it stands by.
#[derive(Clone, Debug, PartialEq)]
pub struct DimString {
    /// As written ("30'-0\"").
    pub text: String,
    /// What it says, in feet.
    pub feet: f64,
    /// Where the string is, in page units.
    pub at: P2,
    /// The two ends its dimension line measures between (its ticks), in page units, and how far apart they are.
    pub from: P2,
    pub to: P2,
    pub paper: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dimensions {
    pub strings: Vec<DimString>,
    /// The scale most of them give, in feet to a page unit, and which of them (by place in `strings`) give it.
    pub scale: Option<f64>,
    pub agree: Vec<usize>,
    /// Those that do not, each with what it measures at that scale, in feet.
    pub off: Vec<(usize, f64)>,
    /// Strings that read as a dimension and have no dimension line beside them.
    pub loose: Vec<(String, P2)>,
}

/// What a string of a plan says as a length in feet, where it is written as an architect's dimension: feet, a tick, and inches
/// that may carry a fraction ("30'-0\"", "5'-8 1/2\"", "16' 5\"", "10'"). Anything else is None: a room's name, a sheet's number,
/// inches alone (which has no feet to say it by).
pub fn feet_written(text: &str) -> Option<f64> {
    let norm: String = text
        .chars()
        .map(|c| match c {
            '\u{2019}' | '\u{2018}' | '\u{2032}' | '\u{00B4}' | '`' => '\'',
            '\u{201D}' | '\u{201C}' | '\u{2033}' | '\u{00A8}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' | '\u{2010}' => '-',
            c => c,
        })
        .collect();
    let t = norm.trim();
    let (ft, rest) = t.split_once('\'')?;
    let ft = ft.trim();
    if ft.is_empty() || !ft.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let feet: f64 = ft.parse().ok()?;
    let rest = rest.trim().trim_start_matches('-').trim();
    let rest = rest.strip_suffix('"').unwrap_or(rest).trim();
    if rest.is_empty() {
        return Some(feet);
    }
    let mut inches = 0.0;
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    for (n, part) in parts.iter().enumerate() {
        if let Some((a, b)) = part.split_once('/') {
            let (a, b): (f64, f64) = (a.parse().ok()?, b.parse().ok()?);
            if b == 0.0 || n == 0 && parts.len() == 2 {
                return None;
            }
            inches += a / b;
        } else {
            if n == 1 || !part.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            inches += part.parse::<f64>().ok()?;
        }
    }
    if inches >= 12.0 {
        return None;
    }
    Some(feet + inches / 12.0)
}

/// The dimension strings of a page (its words, each where it stands) paired with the dimension lines they stand by, and whether the
/// scales they give agree (call 188 item 9). `lines` are the page's straight lines in its own units, `dashed` the places of the
/// dashed ones (not read). A string is paired with the dimension line parallel to it, within four letters' height of it and over its
/// span; the line is measured between its ticks or arrowheads, not between its ends (a line may run past its ticks). The scale most
/// of them give, within an inch or 1 per cent of what each says, is the scale taken; two that disagree with nothing to say which is
/// right give none, and a string that is off it is said with the length it measures at the scale the others give. What reads as a
/// dimension and has no dimension line beside it is `loose`, and never used.
pub fn dimension_strings(lines: &[[[f64; 2]; 2]], dashed: &[usize], texts: &[crate::Text]) -> Dimensions {
    let mut out = Dimensions::default();
    let dashed: std::collections::HashSet<usize> = dashed.iter().copied().collect();
    let segs: Vec<Seg> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !dashed.contains(i))
        .filter_map(|(_, l)| {
            let len = norm(sub(l[1], l[0]));
            (len > 1e-9 && l.iter().flatten().all(|v| v.is_finite())).then_some(Seg { a: l[0], b: l[1], len })
        })
        .collect();
    let dims = dimension_spans(&segs);
    let mut taken = vec![false; dims.len()];
    for t in texts {
        let Some(feet) = feet_written(&t.text) else { continue };
        let u = unit(t.dir);
        let n = perp(u);
        let len = t.text.chars().count() as f64 * 0.5 * t.tall;
        let centre = [t.at[0] + u[0] * len / 2.0 + n[0] * 0.35 * t.tall, t.at[1] + u[1] * len / 2.0 + n[1] * 0.35 * t.tall];
        let mut best: Option<(usize, f64)> = None;
        for (j, d) in dims.iter().enumerate() {
            if taken[j] {
                continue;
            }
            let (a, b) = (d.between[0], d.between[1]);
            let (du, span) = (unit(sub(b, a)), norm(sub(b, a)));
            if cross(u, du).abs() > 0.1 || span < 1e-9 {
                continue;
            }
            let along = dot(du, sub(centre, a));
            let across = dot(perp(du), sub(centre, a)).abs();
            if along < -t.tall || along > span + t.tall || across > 4.0 * t.tall + 4.0 {
                continue;
            }
            if best.map_or(true, |(_, bd)| across < bd) {
                best = Some((j, across));
            }
        }
        match best {
            Some((j, _)) => {
                taken[j] = true;
                let d = dims[j];
                out.strings.push(DimString { text: t.text.trim().to_string(), feet, at: centre, from: d.between[0], to: d.between[1], paper: norm(sub(d.between[1], d.between[0])) });
            }
            None => out.loose.push((t.text.trim().to_string(), centre)),
        }
    }
    let n = out.strings.len();
    let tol = |feet: f64| INCH.max(0.01 * feet);
    let scale_of = |d: &DimString| d.feet / d.paper;
    if n == 1 {
        out.scale = Some(scale_of(&out.strings[0]));
        out.agree = vec![0];
    } else if n >= 2 {
        let agreeing = |i: usize| -> Vec<usize> {
            let k = scale_of(&out.strings[i]);
            (0..n).filter(|&j| (out.strings[j].paper * k - out.strings[j].feet).abs() <= tol(out.strings[j].feet)).collect()
        };
        let best = (0..n).map(|i| (agreeing(i), i)).max_by(|a, b| a.0.len().cmp(&b.0.len()).then(out.strings[a.1].feet.total_cmp(&out.strings[b.1].feet)));
        if let Some((set, _)) = best.filter(|(set, _)| set.len() >= 2) {
            let k = set.iter().map(|&j| scale_of(&out.strings[j])).sum::<f64>() / set.len() as f64;
            out.scale = Some(k);
            out.off = (0..n).filter(|j| !set.contains(j)).map(|j| (j, out.strings[j].paper * k)).collect();
            out.agree = set;
        }
    }
    out
}
