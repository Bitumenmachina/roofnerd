//! Walls, openings and what stands short, found in a scan's points (DECISIONS.md call 173).
//!
//! The scan is the one `scan.rs` reads and levels: feet, east, north and up, level by its
//! own floor, with the floor and the ceiling `Scan::floor_and_ceiling` works out. Nothing
//! here finds a second floor or a second ceiling. Plain geometry, no library.
//!
//! How it works, in the order it is done:
//!
//! 1. Between 3 in above the floor and 3 in below the ceiling (the floor's and the
//!    ceiling's own scatter set aside) are the points that can be a wall's. A 0.1 ft cell
//!    of plan that holds points at three or more heights, 6 in apart, is *upright*: it
//!    is a face seen edge on, not a floor or a table top.
//! 2. Lines are found in the upright cells by voting (a Hough transform: each cell votes
//!    for every line through it, the lines most voted for are real), the strongest first;
//!    each is refitted to the points near it and its cells taken out before the next.
//! 3. Along each line, the stretches where points run on (a gap of 1 ft 6 in or more
//!    parts one stretch from the next) that are at least 2 ft long are *faces*. A face's
//!    place is measured at the floor, from a fit of its offset against height, so that a
//!    scan that leans does not move it.
//! 4. A face whose points run from near the floor to near the ceiling is a wall's face;
//!    one that stops short is something standing in the room, counted and said.
//! 5. Faces within 1 degree of the scan's two main directions are made square to them.
//!    Two wall faces parallel, 2 in to 16 in apart, side by side, are one wall.
//! 6. Each wall's points are laid out as a grid of 3 in squares, along it and up it; a
//!    stretch at least 1 ft 6 in long with none is an opening, its edges measured to the
//!    point (what the scatter blurs is put back by counting, not by taking the nearest
//!    point). Where something stands in front of the stretch it is hidden, not open.
//!
//! What is found is a guess from points and is Assumed wherever it goes.

use crate::scan::Scan;
use std::collections::{HashMap, HashSet};

type P2 = [f64; 2];

// ---- the measures: each a choice where the call (173) gave none, named and said why ----

/// The cell of plan an upright face is looked for in: 0.1 ft (1 1/4 in), finer than a wall
/// is thick and coarser than the scatter of a phone's points (1.5 cm).
const CELL: f64 = 0.1;
/// A cell of fewer points than this is nothing seen: a stray, not a face.
const CELL_POINTS: u32 = 4;
/// Upright: points at this many heights, 6 in apart, in one cell (1 1/2 ft of face at least).
const UPRIGHT_HEIGHTS: u32 = 3;
/// How finely the direction of a line is told in the first vote, in degrees; the refit
/// after it is far finer.
const VOTE_DEGREES: f64 = 0.5;
/// How finely a line's place is told in the first vote, in feet; three of these are added
/// so that a line between two of them is not split.
const VOTE_FEET: f64 = 0.1;
/// A line with fewer upright cells than this under it is not looked at: 2 ft of a face
/// is at least this many cells.
const MIN_VOTES: u32 = 20;
/// The most lines looked for (a house has a few dozen).
const MOST_LINES: usize = 120;
/// How far from its line a point is still on a face, each side: 3 sigma of 1.5 cm of
/// scatter. Two faces nearer than this to each other cannot be told apart (the call's
/// least wall is 2 in = 0.167 ft; a face of 1.5 cm scatter keeps within 0.15).
const BAND: f64 = 0.15;
/// A face with fewer than this fraction (one in this many) of the usual points to the
/// square foot is a line through other things' points, not a face.
const SPARSE: f64 = 4.0;
/// Two lines cross, for the points of one to be taken off the other: at least this far
/// from parallel (the sine of 10 degrees).
const CROSSES: f64 = 0.17;
/// The points near a vote's line, before its line is fitted, are taken from this far:
/// the vote is told to 0.5 degree and 0.1 ft, which on a 30 ft wall is 0.13 ft at the far end.
const SEED_BAND: f64 = 0.2;
/// The width of the columns a face is looked at in, along it, and the height of the rows.
const SQUARE: f64 = 0.25;
/// A column needs this many points to be there at all (a stray or two are not a wall).
const COLUMN_POINTS: usize = 3;
/// A face is at least this long (call 173), and a gap this long or more in the points
/// along a line ends the face (the least opening, so that a doorway is inside a wall and
/// not between two).
const MIN_FACE: f64 = 2.0;
const GAP: f64 = 1.5;
/// The floor's and the ceiling's own scatter, set aside: no wall's point is taken from
/// the lowest or the highest 3 in.
const SET_ASIDE: f64 = 0.25;
/// A wall's face has points within 6 in of the ceiling over this share of its length, and
/// within 6 in of the floor over this share (what a door, a window's sill below nothing,
/// or something standing against it leaves out). On a grid of 3 in squares the one
/// square next to each end (the 3 in set aside lie beyond it) is "within 6 in".
const WALL_REACHES_CEILING: f64 = 0.8;
const WALL_REACHES_FLOOR: f64 = 0.5;
/// Square: within 1 degree of a main direction (call 173).
const SQUARE_DEGREES: f64 = 1.0;
/// Parallel, for two faces of one wall: within 1 degree; apart 2 in to 16 in; side by
/// side for at least this much of their length.
const PAIR_DEGREES: f64 = 1.0;
const PAIR_LEAST: f64 = 2.0 / 12.0;
const PAIR_MOST: f64 = 16.0 / 12.0;
const PAIR_ALONGSIDE: f64 = MIN_FACE;
/// An opening is at least this long (call 173), less what the edges are measured to (3 in
/// of 1/2 inch = this slack): a 1 ft 6 in doorway is not lost for being measured 1 ft 5 3/4 in.
const MIN_OPENING: f64 = 1.5;
const MIN_OPENING_SLACK: f64 = 0.04;
/// Something that stands within this of a stretch of wall, on the side the scan saw it
/// from, hides it (call 173).
const IN_FRONT: f64 = 3.0;
/// Something stands in front of a stretch only if it is this far or more from the wall's
/// own scatter.
const IN_FRONT_LEAST: f64 = 0.2;
/// A found face's end is carried to the line of another wall's face it comes up to, when
/// it is this near: the faces end at the faces they meet, not at the centre of the scatter.
const SNAP: f64 = 0.5;
/// A found wall's end is carried to another's line where it comes within this of it (call 173).
const CORNER: f64 = 1.0;
/// Two walls this nearly parallel (sine of the angle between) do not meet at a corner.
const NOT_PARALLEL: f64 = 0.5;
/// How far from its end the solid of an edge is looked at to count its density, and how
/// far the edge is expected to lie from the first empty square: see `edge`.
const EDGE_BACK: f64 = 0.4;
const EDGE_LOOK: f64 = 0.5;
/// The foot of a wall nearest its end is not counted for the density of an edge: the
/// points of a wall that crosses it were taken off there.
const NEAR_END: f64 = 0.3;
/// Parts of the scan joined into one thing standing short: ends within this (call 173's
/// corners, 1 ft).
const THING_JOIN: f64 = 1.0;
/// A floor cell is 1 ft square; a side of a wall is where the scan saw it from if this
/// many floor cells are on it within 6 ft.
const FLOOR_CELL: f64 = 1.0;
const SEEN_FLOOR_CELLS: usize = 5;
/// ... and if it shows at least a third as much floor as the other side does (the foot of
/// floor nearest a wall is not counted, for the wall's own base scatters into it).
const SEEN_SHARE: usize = 3;
const SEEN_REACH: f64 = 6.0;

/// A wall found.
#[derive(Clone, Debug)]
pub struct Wall {
    /// Its two ends on the plan, in feet. Where it has both faces, its centre line;
    /// where it was seen from one side, the seen face's line. West to east, or south to
    /// north where it runs more north-south.
    pub from: P2,
    pub to: P2,
    /// How thick, from the two faces; None where only one face was seen.
    pub thickness: Option<f64>,
    /// Where one face was seen: the unit direction, on the plan, from the face toward
    /// the room the scan saw it from. None with two faces.
    pub seen_from: Option<P2>,
    pub openings: Vec<Opening>,
    /// Stretches taken as wall that are not seen (something stood in front).
    pub hidden: Vec<Stretch>,
}

impl Wall {
    pub fn length(&self) -> f64 {
        (self.to[0] - self.from[0]).hypot(self.to[1] - self.from[1])
    }
}

/// An opening in a wall: the stretch along it from its `from` end, and its heights above the floor.
#[derive(Clone, Copy, Debug)]
pub struct Opening {
    /// A window (clear from a sill) or a doorway (clear from the floor).
    pub window: bool,
    pub from: f64,
    pub to: f64,
    /// Zero for a doorway.
    pub sill: f64,
    pub head: f64,
}

/// A stretch of a wall, along it from its first end.
#[derive(Clone, Copy, Debug)]
pub struct Stretch {
    pub from: f64,
    pub to: f64,
}

/// Something that stands in the room and is not a wall.
#[derive(Clone, Copy, Debug)]
pub struct Thing {
    /// How tall, above the floor.
    pub height: f64,
    pub at: P2,
}

/// What was found.
#[derive(Clone, Debug, Default)]
pub struct Found {
    pub floor: Option<f64>,
    pub ceiling: Option<f64>,
    pub walls: Vec<Wall>,
    pub things: Vec<Thing>,
    /// How many wall faces were made square to the scan's main directions, of how many.
    pub squared: (usize, usize),
    /// The scan's main direction, in degrees from east, 0 to 90, where there is one.
    pub main: Option<f64>,
    /// Why nothing, or what, was left out, in a sentence each.
    pub left_out: Vec<String>,
    /// No ceiling shows: nothing is found, for a wall cannot be told from a tall cabinet.
    pub no_ceiling: bool,
}

impl Found {
    pub fn tallest(&self) -> Option<f64> {
        self.things.iter().map(|t| t.height).fold(None, |a, h| Some(a.map_or(h, |a: f64| a.max(h))))
    }
}

// ---- the plan geometry of a line ----

fn dir(theta: f64) -> P2 {
    [theta.cos(), theta.sin()]
}
fn normal(theta: f64) -> P2 {
    [-theta.sin(), theta.cos()]
}
fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// A face: a stretch of a line, with the points on it as (along, offset from the line, up).
struct Face {
    theta: f64,
    /// Offset of its line from the origin along its normal, measured at the floor.
    rho: f64,
    s0: f64,
    s1: f64,
    /// (along, across, height above the floor), along and across as the line was when its
    /// points were taken.
    pts: Vec<[f64; 3]>,
    wall: bool,
    squared: bool,
    /// Which side, +1 along the normal, -1 against, the scan saw it from; 0 both or unknown.
    seen: [bool; 2],
    height: f64,
}

impl Face {
    fn at(&self, s: f64) -> P2 {
        let (d, n) = (dir(self.theta), normal(self.theta));
        [s * d[0] + self.rho * n[0], s * d[1] + self.rho * n[1]]
    }
    fn mid(&self) -> f64 {
        (self.s0 + self.s1) / 2.0
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// The edge of a solid, measured to the point. A 1.5 cm scatter blurs an edge; the
/// nearest point on the hollow side lies a little inside it. What does not blur is the
/// count: from `x0`, a place well inside the solid, the points between there and the
/// hollow number so many, and the solid's density (points to the foot) is counted from
/// a stretch beyond `x0` that is all solid. The edge is where that many points, at that
/// density, begin. `up` says the solid lies toward larger values. `v` are the places of
/// the points in the strip, `room` how far the solid runs beyond `x0`, and `hollow_end`
/// how far into the hollow the counting goes (1 ft at most). Where the solid is too short
/// to count the density in, `None`.
fn edge(v: &[f64], x0: f64, up: bool, room: f64, hollow_end: f64) -> Option<f64> {
    let look = EDGE_LOOK.min(room);
    if look < 0.2 {
        return None;
    }
    let (dense, between) = if up {
        let back = (x0 - 1.0).max(hollow_end);
        (v.iter().filter(|x| **x >= x0 && **x < x0 + look).count(), v.iter().filter(|x| **x >= back && **x < x0).count())
    } else {
        let back = (x0 + 1.0).min(hollow_end);
        (v.iter().filter(|x| **x <= x0 && **x > x0 - look).count(), v.iter().filter(|x| **x <= back && **x > x0).count())
    };
    if dense < 5 {
        return None;
    }
    let density = dense as f64 / look;
    let covered = between as f64 / density;
    Some(if up { x0 - covered } else { x0 + covered })
}

/// A grid of 3 in squares along a wall and up it, with the count of points in each.
struct Grid {
    cols: usize,
    rows: usize,
    n: Vec<u32>,
    /// The count a square needs to be there: a quarter of what the middling square that
    /// has any holds, at least 2.
    need: u32,
    s0: f64,
}

impl Grid {
    fn new(pts: &[[f64; 2]], s0: f64, s1: f64, high: f64) -> Grid {
        let cols = (((s1 - s0) / SQUARE).ceil() as usize).max(1);
        let rows = (((high - SET_ASIDE) / SQUARE + 1e-9).floor() as usize).max(1);
        let mut n = vec![0u32; cols * rows];
        for p in pts {
            let (c, r) = (((p[0] - s0) / SQUARE).floor(), ((p[1] - SET_ASIDE) / SQUARE).floor());
            if c >= 0.0 && r >= 0.0 && (c as usize) < cols && (r as usize) < rows {
                n[r as usize * cols + c as usize] += 1;
            }
        }
        let held = median(n.iter().filter(|x| **x > 0).map(|x| f64::from(*x)).collect());
        let need = ((held / 4.0).ceil() as u32).max(2);
        Grid { cols, rows, n, need, s0 }
    }
    fn full(&self, c: usize, r: usize) -> bool {
        self.n[r * self.cols + c] >= self.need
    }
    fn col_range(&self, c: usize) -> f64 {
        self.s0 + c as f64 * SQUARE
    }
}

/// A face is a wall's, or it stops short: its points run from near the floor to near the
/// ceiling over most of its length, or they do not (and then how tall it is).
fn classify(f: &mut Face, height: f64, high: f64) {
    let g = Grid::new(&f.pts.iter().map(|p| [p[0], p[2]]).collect::<Vec<_>>(), f.s0, f.s1, high);
    let cols = g.cols as f64;
    let floor_cols = (0..g.cols).filter(|c| g.full(*c, 0)).count() as f64;
    // Within 6 in of the ceiling: the squares whose tops are above it. (The last square is
    // not taken as the one: how many squares fit under the ceiling depends on the inch
    // the floor and the ceiling were read to, and a scan turned and leaned read a hair over.)
    let near_ceiling = (0..g.rows).filter(|r| SET_ASIDE + (*r as f64 + 1.0) * SQUARE > height - 0.5).collect::<Vec<_>>();
    let ceiling_cols = (0..g.cols).filter(|c| near_ceiling.iter().any(|r| g.full(*c, *r))).count() as f64;
    f.wall = ceiling_cols / cols >= WALL_REACHES_CEILING && floor_cols / cols >= WALL_REACHES_FLOOR;
    f.height = if f.wall { 0.0 } else { top_of(&f.pts) };
}

/// Whether a point is on a face already found that this line crosses: at a corner the
/// crossing face's points lie within a band of the line too and would pull it, and its
/// ends, away from where the thing itself is; and a line that only joins the faces of two
/// things (through a sofa's front and back) has nothing of its own.
fn crosses_face(p: P2, theta: f64, faces: &[Face]) -> bool {
    faces.iter().any(|g| {
        (theta - g.theta).sin().abs() >= CROSSES && (dot(p, normal(g.theta)) - g.rho).abs() <= BAND + 0.05 && dot(p, dir(g.theta)) >= g.s0 - 0.2 && dot(p, dir(g.theta)) <= g.s1 + 0.2
    })
}

/// The strongest line the cells vote for: (direction index k, offset along that line's normal, votes): the line runs at k steps from east.
fn strongest(votes: &[P2], cos: &[f64], sin: &[f64]) -> Option<(usize, f64, u32)> {
    let reach = votes.iter().map(|p| p[0].abs() + p[1].abs()).fold(0.0, f64::max);
    let nb = (2.0 * reach / VOTE_FEET).ceil() as usize + 4;
    let mut acc = vec![0u32; cos.len() * nb];
    for p in votes {
        for k in 0..cos.len() {
            let rho = p[0] * cos[k] + p[1] * sin[k];
            acc[k * nb + ((rho + reach) / VOTE_FEET) as usize + 1] += 1;
        }
    }
    let mut best: Option<(usize, usize, u32)> = None;
    for k in 0..cos.len() {
        for b in 1..nb - 1 {
            let score = acc[k * nb + b - 1] + acc[k * nb + b] + acc[k * nb + b + 1];
            if best.is_none_or(|x| score > x.2) {
                best = Some((k, b, score));
            }
        }
    }
    best.filter(|x| x.2 >= MIN_VOTES).map(|(k, b, score)| (k, (b as f64 - 1.0 + 0.5) * VOTE_FEET - reach, score))
}

/// The stretches along a line that points run through, as (from, to) along it: columns of
/// 3 in with at least COLUMN_POINTS points each are there, and a gap of GAP or more parts one
/// stretch from the next. `s` is sorted.
fn runs_of(s: &[f64]) -> Vec<(f64, f64)> {
    let Some((first, last)) = s.first().zip(s.last()) else { return Vec::new() };
    let cols = (((last - first) / SQUARE).floor() as usize) + 1;
    let mut count = vec![0usize; cols];
    for x in s {
        count[(((x - first) / SQUARE).floor() as usize).min(cols - 1)] += 1;
    }
    let gap_cols = (GAP / SQUARE).round() as usize;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let (mut begun, mut at): (Option<usize>, usize) = (None, 0);
    for (c, n) in count.iter().enumerate() {
        if *n >= COLUMN_POINTS {
            match begun {
                Some(_) if c - at <= gap_cols => {}
                Some(b) => {
                    runs.push((b, at));
                    begun = Some(c);
                }
                None => begun = Some(c),
            }
            at = c;
        }
    }
    if let Some(b) = begun {
        runs.push((b, at));
    }
    runs.into_iter().map(|(a, b)| (first + a as f64 * SQUARE, first + (b + 1) as f64 * SQUARE)).collect()
}

/// The direction (radians, 0 to pi) of the points' own long way, and their mean.
fn long_way(pts: &[P2]) -> Option<(f64, P2)> {
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p[0]).sum::<f64>() / n, pts.iter().map(|p| p[1]).sum::<f64>() / n);
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for p in pts {
        let (x, y) = (p[0] - mx, p[1] - my);
        sxx += x * x;
        sxy += x * y;
        syy += y * y;
    }
    let mut t = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    if t < 0.0 {
        t += std::f64::consts::PI;
    }
    Some((t, [mx, my]))
}

/// Find the walls of a scan that has been levelled by its floor.
pub fn find(scan: &Scan) -> Found {
    let mut out = Found::default();
    let (floor, ceiling) = scan.floor_and_ceiling();
    out.floor = floor;
    out.ceiling = ceiling;
    let Some(floor) = floor else {
        out.left_out.push("no floor shows in the scan, so what stands on it cannot be told".into());
        return out;
    };
    let Some(ceiling) = ceiling else {
        out.no_ceiling = true;
        out.left_out.push("no ceiling shows in the scan: without one, a wall cannot be told from a tall cabinet standing in the room, so no wall is taken".into());
        return out;
    };
    let height = ceiling - floor;
    let high = height - SET_ASIDE;

    // The points that can be a wall's: [east, north, up above the floor].
    let band: Vec<[f64; 3]> = scan.points.iter().filter_map(|p| {
        let z = p[2] - floor;
        (z >= SET_ASIDE && z <= high).then_some([p[0], p[1], z])
    }).collect();
    let key = |x: f64, y: f64| ((x / CELL).floor() as i64, (y / CELL).floor() as i64);
    // Upright cells: points at several heights in one cell of plan.
    struct Cell {
        n: u32,
        heights: u64,
        sx: f64,
        sy: f64,
    }
    let mut cells: HashMap<(i64, i64), Cell> = HashMap::new();
    for p in &band {
        if p[2] < 0.5 || p[2] > height - 0.5 {
            continue;
        }
        let c = cells.entry(key(p[0], p[1])).or_insert(Cell { n: 0, heights: 0, sx: 0.0, sy: 0.0 });
        c.n += 1;
        c.heights |= 1u64 << (((p[2] - 0.5) / 0.5) as u32).min(63);
        c.sx += p[0];
        c.sy += p[1];
    }
    let upright: HashSet<(i64, i64)> = cells.iter().filter(|(_, c)| c.n >= CELL_POINTS && c.heights.count_ones() >= UPRIGHT_HEIGHTS).map(|(k, _)| *k).collect();
    let mut votes: Vec<P2> = cells.iter().filter(|(k, _)| upright.contains(*k)).map(|(_, c)| [c.sx / f64::from(c.n), c.sy / f64::from(c.n)]).collect();
    votes.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    let pool: Vec<P2> = band.iter().filter(|p| upright.contains(&key(p[0], p[1]))).map(|p| [p[0], p[1]]).collect();
    // Where the scan saw the floor: cells a foot square with floor points in them.
    let mut floor_cells: HashMap<(i64, i64), u32> = HashMap::new();
    for p in &scan.points {
        if (p[2] - floor).abs() <= SET_ASIDE {
            *floor_cells.entry(((p[0] / FLOOR_CELL).floor() as i64, (p[1] / FLOOR_CELL).floor() as i64)).or_default() += 1;
        }
    }
    let floor_cells: Vec<P2> = floor_cells.into_iter().filter(|(_, n)| *n >= 3).map(|((a, b), _)| [(a as f64 + 0.5) * FLOOR_CELL, (b as f64 + 0.5) * FLOOR_CELL]).collect();

    // ---- lines, strongest first; each one's stretches are faces ----
    let steps = (180.0 / VOTE_DEGREES) as usize;
    let (cos, sin): (Vec<f64>, Vec<f64>) = (0..steps).map(|k| { let n = normal((k as f64 * VOTE_DEGREES).to_radians()); (n[0], n[1]) }).unzip();
    let mut faces: Vec<Face> = Vec::new();
    for _ in 0..MOST_LINES {
        let Some((k, rho0, _)) = strongest(&votes, &cos, &sin) else { break };
        // The vote's line, as coarse as the vote: the points within SEED_BAND of it, all
        // heights, along it. Its stretches are looked at one at a time: two things on one
        // line with a gap between them (a sofa and a cabinet) are two, and each is fitted by
        // itself, so that a line through both does not lean across the room.
        let theta0 = (k as f64 * VOTE_DEGREES).to_radians();
        let (d0, n0) = (dir(theta0), normal(theta0));
        let mut on: Vec<f64> = band.iter().filter(|p| (dot([p[0], p[1]], n0) - rho0).abs() <= SEED_BAND && !crosses_face([p[0], p[1]], theta0, &faces)).map(|p| dot([p[0], p[1]], d0)).collect();
        on.sort_by(f64::total_cmp);
        let mut taken_out: Vec<(f64, f64)> = Vec::new();
        for (sa, sb) in runs_of(&on) {
            taken_out.push((sa, sb));
            if sb - sa < MIN_FACE {
                continue;
            }
            // Fitted to the upright points of the stretch, nearer each time.
            let (mut theta, mut rho) = (theta0, rho0);
            for _ in 0..3 {
                let (d, n) = (dir(theta), normal(theta));
                let near: Vec<P2> = pool.iter().copied().filter(|p| (dot(*p, n) - rho).abs() <= if theta == theta0 { SEED_BAND } else { BAND } && dot(*p, d) >= sa - 0.2 && dot(*p, d) <= sb + 0.2 && !crosses_face(*p, theta, &faces)).collect();
                let Some((t, m)) = long_way(&near) else { break };
                // `long_way` gives an angle within 0 to pi; keep the one nearest the vote's.
                theta = if (t - theta0).abs() > std::f64::consts::FRAC_PI_2 { if t > theta0 { t - std::f64::consts::PI } else { t + std::f64::consts::PI } } else { t };
                rho = dot(m, normal(theta));
            }
            let (d, n) = (dir(theta), normal(theta));
            let mut pts: Vec<[f64; 3]> = band.iter().filter(|p| (dot([p[0], p[1]], n) - rho).abs() <= BAND && !crosses_face([p[0], p[1]], theta, &faces)).map(|p| [dot([p[0], p[1]], d), dot([p[0], p[1]], n), p[2]]).filter(|p| p[0] >= sa - 0.2 && p[0] <= sb + 0.2).collect();
            if pts.len() < 20 {
                continue;
            }
            // Offset against height: the line's place at the floor (z = 0), unless the face
            // leans so far that it is not a plumb face (more than 2 degrees), when its middle.
            let mean_d = pts.iter().map(|p| p[1]).sum::<f64>() / pts.len() as f64;
            let mean_z = pts.iter().map(|p| p[2]).sum::<f64>() / pts.len() as f64;
            let (mut szz, mut szd) = (0.0, 0.0);
            for p in &pts {
                szz += (p[2] - mean_z) * (p[2] - mean_z);
                szd += (p[2] - mean_z) * (p[1] - mean_d);
            }
            let slope = if szz > 1e-9 { szd / szz } else { 0.0 };
            let at_floor = if slope.abs() <= 0.035 { mean_d - slope * mean_z } else { mean_d };
            pts.sort_by(|p, q| p[0].total_cmp(&q[0]));
            // Its ends, to the point, as an edge (what is beyond is empty or another wall).
            let s: Vec<f64> = pts.iter().map(|p| p[0]).collect();
            let (first_s, last_s) = (s[0], s[s.len() - 1]);
            let s0 = edge(&s, first_s + EDGE_BACK, true, last_s - first_s - EDGE_BACK, f64::MIN).unwrap_or(first_s);
            let s1 = edge(&s, last_s - EDGE_BACK, false, last_s - first_s - EDGE_BACK, f64::MAX).unwrap_or(last_s);
            if s1 - s0 < MIN_FACE {
                continue;
            }
            // A face stands: its points are at several heights (a table top seen edge on is a
            // line of points all at one).
            let mut tiers: HashMap<usize, usize> = HashMap::new();
            for p in &pts {
                *tiers.entry((p[2] / 0.5) as usize).or_default() += 1;
            }
            if tiers.values().filter(|n| **n >= 10).count() < UPRIGHT_HEIGHTS as usize {
                continue;
            }
            // The same face found twice (a second vote that fell on the first's edge): once.
            let twice = faces.iter().any(|f| {
                let apart = (f.theta - theta).abs();
                let across = (dot(f.at(f.mid()), n) - rho).abs();
                let along = f.s1.min(s1) - f.s0.max(s0);
                apart.to_degrees() < 2.0 && across < BAND && along > 0.5 * (s1 - s0).min(f.s1 - f.s0)
            });
            if twice {
                continue;
            }
            let mut face = Face { theta, rho: at_floor, s0, s1, pts, wall: false, squared: false, seen: [false, false], height: 0.0 };
            classify(&mut face, height, high);
            faces.push(face);
            // Its cells are out of the next vote.
            votes.retain(|p| !((dot(*p, n) - rho).abs() <= BAND && dot(*p, d) >= sa - 0.3 && dot(*p, d) <= sb + 0.3));
        }
        // Whatever the vote's own cells were, they are out of the next vote: the ones near
        // its line, where its stretches ran.
        let before = votes.len();
        votes.retain(|p| !((dot(*p, n0) - rho0).abs() <= SEED_BAND && taken_out.iter().any(|(a, b)| dot(*p, d0) >= a - 0.3 && dot(*p, d0) <= b + 0.3)));
        if votes.len() == before {
            votes.retain(|p| (dot(*p, n0) - rho0).abs() > SEED_BAND);
        }
    }

    // A line drawn through the points of other things (a sofa's front and back, a table's
    // corner) has a few points of its own: a face with under a quarter of the usual
    // number of points to the square foot of face is not one.
    let per_foot: Vec<f64> = faces.iter().map(|f| f.pts.len() as f64 / ((f.s1 - f.s0) * if f.wall { height } else { f.height.max(0.5) })).collect();
    let usual = median(per_foot.clone());
    let mut keep = per_foot.iter().map(|d| *d >= usual / SPARSE);
    faces.retain(|_| keep.next().unwrap_or(true));
    let standing: Vec<usize> = (0..faces.len()).filter(|i| !faces[*i].wall).collect();
    // ---- square to the scan's main directions ----
    let wall_ids: Vec<usize> = (0..faces.len()).filter(|i| faces[*i].wall).collect();
    out.squared = (0, wall_ids.len());
    if !wall_ids.is_empty() {
        // The main direction: the length-weighted mean of the faces' directions taken four
        // times round (so that a face and the one square to it count together).
        let (mut c, mut s) = (0.0, 0.0);
        for i in &wall_ids {
            let w = faces[*i].s1 - faces[*i].s0;
            c += w * (4.0 * faces[*i].theta).cos();
            s += w * (4.0 * faces[*i].theta).sin();
        }
        let main = (s.atan2(c) / 4.0).rem_euclid(std::f64::consts::FRAC_PI_2);
        out.main = Some(main.to_degrees());
        for i in &wall_ids {
            let f = &mut faces[*i];
            // Nearest of main, main + 90, main + 180, ...
            let quarter = std::f64::consts::FRAC_PI_2;
            let best = main + ((f.theta - main) / quarter).round() * quarter;
            let off = (f.theta - best).abs();
            if off.to_degrees() <= SQUARE_DEGREES {
                // Turned about its middle: the same line through the same middle.
                let mid = f.at(f.mid());
                let shift_from = dot(mid, dir(f.theta));
                f.theta = best;
                f.rho = dot(mid, normal(best));
                let shift = dot(mid, dir(best)) - shift_from;
                f.s0 += shift;
                f.s1 += shift;
                for p in &mut f.pts {
                    p[0] += shift;
                }
                f.squared = true;
                out.squared.0 += 1;
            }
        }
    }

    // ---- a wall face's end that comes up to another wall's face is carried to it ----
    // (the faces found run a little past, or a little short of, where they meet, by what
    // the crossing wall's own points did to them)
    let lines: Vec<(f64, f64, f64, f64)> = wall_ids.iter().map(|i| (faces[*i].theta, faces[*i].rho, faces[*i].s0, faces[*i].s1)).collect();
    for i in &wall_ids {
        let f = &mut faces[*i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        for end in 0..2 {
            let here = if end == 0 { f.s0 } else { f.s1 };
            let mut best: Option<(f64, f64)> = None;
            for (gt, gr, g0, g1) in &lines {
                let cross = (f.theta - gt).sin();
                if cross.abs() < NOT_PARALLEL {
                    continue;
                }
                // Where this line meets the other's: s along this one with s d + rho n on gr = p . n_g.
                let (gd, gn) = (dir(*gt), normal(*gt));
                let denom = dot(d, gn);
                if denom.abs() < 1e-9 {
                    continue;
                }
                let s = (gr - f.rho * dot(n, gn)) / denom;
                let q = [s * d[0] + f.rho * n[0], s * d[1] + f.rho * n[1]];
                let along_g = dot(q, gd);
                if (s - here).abs() <= SNAP && along_g >= g0 - SNAP && along_g <= g1 + SNAP && best.is_none_or(|b| (s - here).abs() < (b.0 - here).abs()) {
                    best = Some((s, 0.0));
                }
            }
            if let Some((s, _)) = best {
                if end == 0 { f.s0 = s } else { f.s1 = s }
            }
        }
    }

    // ---- which side the scan saw each wall face from ----
    for i in &wall_ids {
        let f = &mut faces[*i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        let mut sides = [0usize; 2];
        for c in &floor_cells {
            let (s, a) = (dot(*c, d), dot(*c, n) - f.rho);
            if s >= f.s0 && s <= f.s1 && a.abs() > FLOOR_CELL && a.abs() <= SEEN_REACH {
                sides[usize::from(a < 0.0)] += 1;
            }
        }
        f.seen = [sides[0] >= SEEN_FLOOR_CELLS && sides[0] * SEEN_SHARE >= sides[1], sides[1] >= SEEN_FLOOR_CELLS && sides[1] * SEEN_SHARE >= sides[0]];
        if !f.seen[0] && !f.seen[1] {
            f.seen = [sides[0] >= sides[1], sides[0] < sides[1]];
        }
    }

    // ---- what stands short: parts joined at their ends into things ----
    let mut group: Vec<usize> = (0..standing.len()).collect();
    fn root(g: &mut Vec<usize>, i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        g[i] = r;
        r
    }
    for a in 0..standing.len() {
        for b in a + 1..standing.len() {
            let (fa, fb) = (&faces[standing[a]], &faces[standing[b]]);
            let near = [fa.at(fa.s0), fa.at(fa.s1)].iter().any(|p| [fb.at(fb.s0), fb.at(fb.s1)].iter().any(|q| (p[0] - q[0]).hypot(p[1] - q[1]) <= THING_JOIN));
            if near {
                let (ra, rb) = (root(&mut group, a), root(&mut group, b));
                group[ra] = rb;
            }
        }
    }
    let mut things: HashMap<usize, Thing> = HashMap::new();
    for a in 0..standing.len() {
        let r = root(&mut group, a);
        let f = &faces[standing[a]];
        let m = f.at(f.mid());
        let t = things.entry(r).or_insert(Thing { height: 0.0, at: m });
        t.height = t.height.max(f.height);
    }
    let mut things: Vec<Thing> = things.into_values().collect();
    things.sort_by(|a, b| a.at[0].total_cmp(&b.at[0]).then(a.at[1].total_cmp(&b.at[1])));
    out.things = things;
    // Where something stands: the cells of the plan its faces' points are in, for what hides a wall.
    let mut stands: HashSet<(i64, i64)> = HashSet::new();
    let mut stand_at: Vec<P2> = Vec::new();
    for i in &standing {
        let f = &faces[*i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        for p in &f.pts {
            let q = [p[0] * d[0] + p[1] * n[0], p[0] * d[1] + p[1] * n[1]];
            if stands.insert(((q[0] / 0.25).floor() as i64, (q[1] / 0.25).floor() as i64)) {
                stand_at.push(q);
            }
        }
    }

    // ---- pair faces into walls ----
    let mut taken: HashSet<usize> = HashSet::new();
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    for (x, a) in wall_ids.iter().enumerate() {
        for b in &wall_ids[x + 1..] {
            let (fa, fb) = (&faces[*a], &faces[*b]);
            let mut dt = (fa.theta - fb.theta).abs();
            dt = dt.min(std::f64::consts::PI - dt);
            if dt.to_degrees() > PAIR_DEGREES {
                continue;
            }
            // Which side of one the other's middle is on, and how far.
            let across = dot(fb.at(fb.mid()), normal(fa.theta)) - fa.rho;
            if across.abs() < PAIR_LEAST || across.abs() > PAIR_MOST {
                continue;
            }
            // Side by side along the first's own measure of the way along.
            let shift = dot(fb.at(fb.mid()), dir(fa.theta)) - fb.mid();
            let overlap = fa.s1.min(fb.s1 + shift) - fa.s0.max(fb.s0 + shift);
            if overlap >= PAIR_ALONGSIDE {
                pairs.push((*a, *b, overlap));
            }
        }
    }
    pairs.sort_by(|p, q| q.2.total_cmp(&p.2));
    let mut walls: Vec<(Wall, WallBuild)> = Vec::new();
    for (a, b, _) in pairs {
        if taken.contains(&a) || taken.contains(&b) {
            continue;
        }
        taken.insert(a);
        taken.insert(b);
        let (fa, fb) = (&faces[a], &faces[b]);
        let theta = fa.theta;
        let (d, n) = (dir(theta), normal(theta));
        // Both in the first's frame.
        let rb = dot(fb.at(fb.mid()), n);
        let shift = dot(fb.at(fb.mid()), d) - dot(fb.at(fb.mid()), dir(fb.theta));
        let centre = (fa.rho + rb) / 2.0;
        let (s0, s1) = (fa.s0.min(fb.s0 + shift), fa.s1.max(fb.s1 + shift));
        let mut pts: Vec<[f64; 2]> = fa.pts.iter().map(|p| [p[0], p[2]]).collect();
        pts.extend(fb.pts.iter().map(|p| [p[0] + shift, p[2]]));
        // Seen from the side each face faces away from the other: both, so none of the one-sided.
        let (w, flip) = wall_of(theta, centre, s0, s1, Some((rb - fa.rho).abs()), None);
        walls.push((w, WallBuild { pts, theta, rho: centre, s0, flip }));
    }
    for i in &wall_ids {
        if taken.contains(i) {
            continue;
        }
        let f = &faces[*i];
        // The side it was seen from: the normal's, or against it.
        let toward = if f.seen[1] && !f.seen[0] { -1.0 } else { 1.0 };
        let n = normal(f.theta);
        let seen_from = Some([n[0] * toward, n[1] * toward]);
        let pts: Vec<[f64; 2]> = f.pts.iter().map(|p| [p[0], p[2]]).collect();
        let (w, flip) = wall_of(f.theta, f.rho, f.s0, f.s1, None, seen_from);
        walls.push((w, WallBuild { pts, theta: f.theta, rho: f.rho, s0: f.s0, flip }));
    }

    // ---- the openings, and what is not seen ----
    for (w, b) in &mut walls {
        let (d, n) = (dir(b.theta), normal(b.theta));
        let length = w.length();
        let g = Grid::new(&b.pts.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>(), b.s0, b.s0 + length, high);
        // Empty squares, joined; the wide ones are holes.
        let mut seen = vec![false; g.cols * g.rows];
        let mut holes: Vec<(usize, usize, usize, usize, usize)> = Vec::new();
        for r in 0..g.rows {
            for c in 0..g.cols {
                if seen[r * g.cols + c] || g.full(c, r) {
                    continue;
                }
                let (mut stack, mut cells) = (vec![(c, r)], Vec::new());
                seen[r * g.cols + c] = true;
                while let Some((cc, rr)) = stack.pop() {
                    cells.push((cc, rr));
                    let near = [(cc.wrapping_sub(1), rr), (cc + 1, rr), (cc, rr.wrapping_sub(1)), (cc, rr + 1)];
                    for (nc, nr) in near {
                        if nc < g.cols && nr < g.rows && !seen[nr * g.cols + nc] && !g.full(nc, nr) {
                            seen[nr * g.cols + nc] = true;
                            stack.push((nc, nr));
                        }
                    }
                }
                let (c0, c1) = (cells.iter().map(|x| x.0).min().unwrap_or(0), cells.iter().map(|x| x.0).max().unwrap_or(0));
                let (r0, r1) = (cells.iter().map(|x| x.1).min().unwrap_or(0), cells.iter().map(|x| x.1).max().unwrap_or(0));
                let box_cells = (c1 - c0 + 1) * (r1 - r0 + 1);
                // A hole is a clean block of empty squares, wide enough, not at the wall's ends.
                if (c1 - c0 + 1) as f64 * SQUARE >= MIN_OPENING - 0.25 && cells.len() as f64 >= 0.8 * box_cells as f64 && c0 > 0 && c1 + 1 < g.cols {
                    holes.push((c0, c1, r0, r1, cells.len()));
                }
            }
        }
        holes.sort();
        for (c0, c1, r0, r1, _) in holes {
            let (ls, rs) = (g.col_range(c0), g.col_range(c1 + 1));
            let (zb, zt) = (SET_ASIDE + r0 as f64 * SQUARE, SET_ASIDE + (r1 + 1) as f64 * SQUARE);
            if r1 + 1 >= g.rows {
                out.left_out.push(format!("a gap {:.0} in wide in a wall runs from the floor to the ceiling: it is not taken as an opening, and the wall is laid across it", (rs - ls) * 12.0));
                continue;
            }
            // Along: between the points of the strip inside the hole's own height.
            let strip_lo = if r0 <= 1 { SET_ASIDE } else { zb + 0.3 };
            let strip_hi = (zt - 0.3).max(strip_lo + 0.1);
            let in_strip: Vec<f64> = b.pts.iter().filter(|p| p[1] >= strip_lo && p[1] <= strip_hi).map(|p| p[0]).collect();
            let hole_mid = (ls + rs) / 2.0;
            let el = edge(&in_strip, ls - EDGE_BACK, false, ls - EDGE_BACK - b.s0 - NEAR_END, hole_mid).unwrap_or(ls);
            let er = edge(&in_strip, rs + EDGE_BACK, true, b.s0 + length - rs - EDGE_BACK - NEAR_END, hole_mid).unwrap_or(rs);
            if er - el < MIN_OPENING - MIN_OPENING_SLACK {
                continue;
            }
            // Up: between the points of the strip inside the hole's own width.
            let in_up: Vec<f64> = b.pts.iter().filter(|p| p[0] >= el + 0.3 && p[0] <= er - 0.3).map(|p| p[1]).collect();
            let head = edge(&in_up, zt + EDGE_BACK, true, high - zt - EDGE_BACK, zb).unwrap_or(zt);
            let window = r0 > 1;
            let sill = if window { edge(&in_up, zb - EDGE_BACK, false, zb - EDGE_BACK - SET_ASIDE, zt).unwrap_or(zb) } else { 0.0 };
            let (from, to) = (el - b.s0, er - b.s0);
            let opening = if b.flip { Opening { window, from: length - to, to: length - from, sill, head } } else { Opening { window, from, to, sill, head } };
            // Something standing in front, on the side the scan saw the wall from, hides it.
            let blocked = |side: f64| -> bool {
                stand_at.iter().any(|q| {
                    let (s, a) = (dot(*q, d), (dot(*q, n) - b.rho) * side);
                    // On the scanned side the thickness of a wall is its half either way; across both faces from the centre.
                    let half = w.thickness.map_or(0.0, |t| t / 2.0);
                    s >= el && s <= er && a > half + IN_FRONT_LEAST && a <= half + IN_FRONT
                })
            };
            let hidden = if w.thickness.is_some() { blocked(1.0) && blocked(-1.0) } else {
                let toward = w.seen_from.map_or(1.0, |s| if dot(s, n) >= 0.0 { 1.0 } else { -1.0 });
                blocked(toward)
            };
            if hidden {
                w.hidden.push(Stretch { from: opening.from, to: opening.to });
            } else {
                w.openings.push(opening);
            }
        }
    }

    // ---- first end first: west to east, or south to north ----
    out.walls = walls.into_iter().map(|(w, _)| w).collect();
    out.walls.sort_by(|a, b| a.from[0].total_cmp(&b.from[0]).then(a.from[1].total_cmp(&b.from[1])));
    out
}

/// What an opening's measuring needs to know of the wall it is in.
struct WallBuild {
    /// (along, up) of the points of its faces.
    pts: Vec<[f64; 2]>,
    theta: f64,
    rho: f64,
    s0: f64,
    /// Its first end is at its far end along the way the points were measured.
    flip: bool,
}

/// A wall on the plan from its line's direction, place and ends, put first end first.
fn wall_of(theta: f64, rho: f64, s0: f64, s1: f64, thickness: Option<f64>, seen_from: Option<P2>) -> (Wall, bool) {
    let (d, n) = (dir(theta), normal(theta));
    let at = |s: f64| [s * d[0] + rho * n[0], s * d[1] + rho * n[1]];
    let (mut from, mut to) = (at(s0), at(s1));
    // West to east where it runs more east-west, south to north where more north-south.
    let flip = if d[0].abs() >= d[1].abs() { d[0] < 0.0 } else { d[1] < 0.0 };
    if flip {
        std::mem::swap(&mut from, &mut to);
    }
    (Wall { from, to, thickness, seen_from, openings: Vec::new(), hidden: Vec::new() }, flip)
}

/// The height of a face that stops short: the top of its points, where their number
/// per 0.1 ft of height is still half of what is usual. The nearest point to the top
/// would be a stray's or the scatter's, and what stands on it (a top that spills into
/// the face's strip) is not counted: the first two such tenths in a row end it.
fn top_of(pts: &[[f64; 3]]) -> f64 {
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
}

/// What is laid and what is not, of a find. `outside` is the thickness walls seen from
/// one side are given (the type the person named for them); where it is None they are
/// not laid (their places in `Found::walls` come back second). A wall seen from one side
/// stands with its seen face on the scan's face and its thickness on the far side of it;
/// it is drawn with the room on its right so that its outside is to its left, as the
/// model draws its outsides. Each laid wall's end that comes within 1 ft of another's
/// line is carried to it, so that rooms close (call 173).
pub fn lay(found: &Found, outside: Option<f64>) -> (Vec<Laid>, Vec<usize>) {
    let mut laid: Vec<Laid> = Vec::new();
    let mut not_laid: Vec<usize> = Vec::new();
    for (i, w) in found.walls.iter().enumerate() {
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
        laid.push(Laid { from, to, thickness, measured, openings: Vec::new(), of: i });
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
    (laid, not_laid)
}

fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn unit(a: P2) -> P2 {
    let l = a[0].hypot(a[1]).max(1e-12);
    [a[0] / l, a[1] / l]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A little scatter, the same every time.
    struct Jitter(u64);
    impl Jitter {
        fn next(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((self.0 >> 33) as f64 / (1u64 << 31) as f64 - 0.5) * 0.06
        }
    }

    /// A made-up room's floor and ceiling, 8 ft apart, over `w` by `d` feet, a point every
    /// 1 1/4 in each way; with no ceiling when asked.
    fn room(w: f64, d: f64, ceiling: bool, j: &mut Jitter) -> Vec<[f64; 3]> {
        let mut pts = Vec::new();
        let (nx, ny) = ((w / 0.1) as usize, (d / 0.1) as usize);
        for i in 0..=nx {
            for k in 0..=ny {
                let (x, y) = (i as f64 * 0.1, k as f64 * 0.1);
                pts.push([x + j.next(), y + j.next(), j.next()]);
                if ceiling {
                    pts.push([x + j.next(), y + j.next(), 8.0 + j.next()]);
                }
            }
        }
        pts
    }

    /// Points on a vertical face from (x0, y0) to (x1, y1), from `z0` to `z1` up, leaving out
    /// the stretch `gap` (along it from its start, from, to, and its heights low, high).
    fn face(a: P2, b: P2, z0: f64, z1: f64, gap: Option<[f64; 4]>, j: &mut Jitter) -> Vec<[f64; 3]> {
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let mut pts = Vec::new();
        let (n, m) = ((len / 0.04) as usize, ((z1 - z0) / 0.1) as usize);
        for i in 0..=n {
            for k in 0..=m {
                let (t, z) = (i as f64 * 0.04, z0 + k as f64 * 0.1);
                if gap.is_some_and(|g| t >= g[0] && t <= g[1] && z >= g[2] && z <= g[3]) {
                    continue;
                }
                let f = t / len;
                pts.push([a[0] + (b[0] - a[0]) * f + j.next(), a[1] + (b[1] - a[1]) * f + j.next(), z + j.next()]);
            }
        }
        pts
    }

    fn found_in(points: Vec<[f64; 3]>) -> Found {
        find(&Scan { points, lost: Vec::new() })
    }

    #[test]
    fn two_faces_make_a_wall_and_give_its_thickness() {
        let mut j = Jitter(1);
        let mut pts = room(20.0, 12.0, true, &mut j);
        // A partition: two faces 4 1/2 in apart, 12 ft long, floor to ceiling.
        for x in [10.0 - 0.1875, 10.0 + 0.1875] {
            pts.extend(face([x, 0.0], [x, 12.0], 0.0, 8.0, None, &mut j));
        }
        let f = found_in(pts);
        assert_eq!(f.walls.len(), 1, "{:?}", f.walls);
        let w = &f.walls[0];
        assert!(w.seen_from.is_none(), "two faces are seen from both sides");
        assert!((w.thickness.unwrap() - 0.375).abs() < 1.0 / 24.0, "{:?}", w.thickness);
        assert!((w.from[0] - 10.0).abs() < 1.0 / 12.0 && (w.to[0] - 10.0).abs() < 1.0 / 12.0, "{w:?}");
        assert!((w.length() - 12.0).abs() < 1.0 / 6.0, "{}", w.length());
    }

    #[test]
    fn a_lone_face_is_a_wall_seen_from_one_side_and_has_no_thickness() {
        let mut j = Jitter(2);
        let mut pts = room(20.0, 12.0, true, &mut j);
        // An outside wall seen from the room (east of it) and nothing on its far side.
        pts.extend(face([0.0, 0.0], [0.0, 12.0], 0.0, 8.0, None, &mut j));
        let f = found_in(pts);
        assert_eq!(f.walls.len(), 1, "{:?}", f.walls);
        let w = &f.walls[0];
        assert!(w.thickness.is_none(), "the scan does not give a one-sided wall's thickness: {:?}", w.thickness);
        let toward = w.seen_from.unwrap();
        assert!(toward[0] > 0.99, "seen from the east side: {toward:?}");
        assert!(w.from[0].abs() < 1.0 / 12.0 && w.to[0].abs() < 1.0 / 12.0, "{w:?}");
    }

    #[test]
    fn a_face_that_stops_short_of_the_ceiling_is_not_a_wall_and_is_counted() {
        let mut j = Jitter(3);
        let mut pts = room(20.0, 12.0, true, &mut j);
        // A cabinet's face, 6 ft long and 7 ft tall, in the room: a foot short of the ceiling.
        pts.extend(face([5.0, 6.0], [11.0, 6.0], 0.0, 7.0, None, &mut j));
        let f = found_in(pts);
        assert!(f.walls.is_empty(), "{:?}", f.walls);
        assert_eq!(f.things.len(), 1);
        assert!((f.tallest().unwrap() - 7.0).abs() < 2.0 / 12.0, "{:?}", f.tallest());
    }

    #[test]
    fn a_face_shorter_than_2_ft_is_nothing() {
        let mut j = Jitter(4);
        let mut pts = room(20.0, 12.0, true, &mut j);
        pts.extend(face([5.0, 6.0], [6.6, 6.0], 0.0, 8.0, None, &mut j));
        let f = found_in(pts);
        assert!(f.walls.is_empty() && f.things.is_empty(), "{:?} {:?}", f.walls, f.things);
    }

    #[test]
    fn clear_from_the_floor_is_a_doorway() {
        let mut j = Jitter(5);
        let mut pts = room(20.0, 12.0, true, &mut j);
        // A wall seen from one side, 12 ft long, with a gap 3 ft wide from 4 ft along it, floor to 6 ft 8 in.
        pts.extend(face([0.0, 0.0], [0.0, 12.0], 0.0, 8.0, Some([4.0, 7.0, 0.0, 6.67]), &mut j));
        let f = found_in(pts);
        assert_eq!(f.walls.len(), 1);
        let o = f.walls[0].openings.first().expect("an opening");
        assert!(!o.window, "{o:?}");
        assert!((o.to - o.from - 3.0).abs() < 2.0 / 12.0 && (o.from - 4.0).abs() < 2.0 / 12.0, "{o:?}");
        assert!(o.sill == 0.0 && (o.head - 6.67).abs() < 2.0 / 12.0, "{o:?}");
    }

    #[test]
    fn clear_from_a_sill_is_a_window() {
        let mut j = Jitter(6);
        let mut pts = room(20.0, 12.0, true, &mut j);
        pts.extend(face([0.0, 0.0], [0.0, 12.0], 0.0, 8.0, Some([4.0, 9.0, 3.0, 7.0]), &mut j));
        let f = found_in(pts);
        let o = f.walls[0].openings.first().expect("an opening");
        assert!(o.window, "{o:?}");
        assert!((o.to - o.from - 5.0).abs() < 2.0 / 12.0, "{o:?}");
        assert!((o.sill - 3.0).abs() < 2.0 / 12.0 && (o.head - 7.0).abs() < 2.0 / 12.0, "{o:?}");
    }

    #[test]
    fn not_seen_is_not_open() {
        let mut j = Jitter(7);
        let mut pts = room(20.0, 14.0, true, &mut j);
        // A gap 5 ft wide to 7 ft up (the wall seen above it), but a cabinet 7 ft tall stands 2 ft in front of it.
        pts.extend(face([0.0, 0.0], [0.0, 14.0], 0.0, 8.0, Some([4.0, 9.0, 0.0, 7.0]), &mut j));
        pts.extend(face([2.0, 3.5], [2.0, 9.5], 0.0, 7.0, None, &mut j));
        let f = found_in(pts);
        assert_eq!(f.walls.len(), 1, "{:?}", f.walls);
        assert!(f.walls[0].openings.is_empty(), "what is behind a cabinet is not an opening: {:?}", f.walls[0].openings);
        assert_eq!(f.walls[0].hidden.len(), 1, "{:?}", f.walls[0]);
        assert!((f.walls[0].hidden[0].to - f.walls[0].hidden[0].from - 5.0).abs() < 4.0 / 12.0, "{:?}", f.walls[0].hidden);
    }

    #[test]
    fn no_ceiling_finds_nothing_and_says_so() {
        let mut j = Jitter(8);
        let mut pts = room(20.0, 12.0, false, &mut j);
        pts.extend(face([0.0, 0.0], [0.0, 12.0], 0.0, 8.0, None, &mut j));
        let f = found_in(pts);
        assert!(f.no_ceiling && f.walls.is_empty() && f.things.is_empty());
        assert!(f.left_out.iter().any(|l| l.contains("no ceiling")), "{:?}", f.left_out);
    }

    #[test]
    fn faces_within_a_degree_of_the_main_directions_are_squared_and_counted() {
        let mut j = Jitter(9);
        let mut pts = room(24.0, 16.0, true, &mut j);
        // Three walls: one east-west, one north-south, one east-west and 0.6 degree off; and a
        // fourth 3 degrees off, which stays as it is.
        pts.extend(face([0.0, 0.0], [20.0, 0.0], 0.0, 8.0, None, &mut j));
        pts.extend(face([0.0, 0.0], [0.0, 14.0], 0.0, 8.0, None, &mut j));
        let t = 0.6f64.to_radians();
        pts.extend(face([0.0, 14.0], [20.0 * t.cos(), 14.0 + 20.0 * t.sin()], 0.0, 8.0, None, &mut j));
        let t = 3.0f64.to_radians();
        pts.extend(face([21.0, 1.0], [21.0 - 8.0 * t.sin(), 1.0 + 8.0 * t.cos()], 0.0, 8.0, None, &mut j));
        let f = found_in(pts);
        assert_eq!(f.squared.1, f.walls.len(), "{:?}", f.squared);
        assert_eq!(f.squared, (3, 4), "{:?} {:?}", f.squared, f.walls);
    }

    #[test]
    fn a_found_walls_end_is_carried_to_the_line_it_comes_within_a_foot_of() {
        let wall = |from: P2, to: P2| Wall { from, to, thickness: Some(0.5), seen_from: None, openings: Vec::new(), hidden: Vec::new() };
        // A wall along the south, and another going north from near its east end, 0.4 ft short of it.
        let found = Found { walls: vec![wall([0.0, 0.0], [20.0, 0.0]), wall([20.0, 0.4], [20.0, 12.0])], ..Found::default() };
        let (laid, none) = lay(&found, None);
        assert!(none.is_empty() && laid.len() == 2);
        assert!(laid[1].from[1].abs() < 1e-9 && (laid[1].from[0] - 20.0).abs() < 1e-9, "{:?}", laid[1].from);
        // And one that is a yard away is not.
        let far = Found { walls: vec![wall([0.0, 0.0], [20.0, 0.0]), wall([20.0, 3.0], [20.0, 12.0])], ..Found::default() };
        assert!((lay(&far, None).0[1].from[1] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_wall_seen_from_one_side_is_laid_only_with_a_thickness_and_stands_outside_its_face() {
        let seen = Wall { from: [0.0, 0.0], to: [0.0, 12.0], thickness: None, seen_from: Some([1.0, 0.0]), openings: Vec::new(), hidden: Vec::new() };
        let found = Found { walls: vec![seen], ..Found::default() };
        let (laid, not) = lay(&found, None);
        assert!(laid.is_empty() && not == vec![0], "not laid without a type");
        let (laid, not) = lay(&found, Some(0.5));
        assert!(not.is_empty());
        // Its centre is 3 in outside the seen face (the room is east of it); drawn with the room on its right.
        assert!((laid[0].from[0] + 0.25).abs() < 1e-9 && (laid[0].to[0] + 0.25).abs() < 1e-9, "{laid:?}");
        assert!(laid[0].to[1] > laid[0].from[1], "the room, east, is on the right of a wall drawn north");
    }

    /// The two-room house of tools/made-up-scan.py --as-scanned, made here: 30 ft by 20 ft inside, 8 ft
    /// high; the outside walls one face each; a partition of two faces 4 1/2 in apart with a doorway;
    /// a window in the south wall; tall cabinets with the north wall behind them not seen; a table
    /// and a sofa; scatter of about 1.5 cm, strays, the house turned `turn` degrees about its
    /// southwest corner and the whole scan leaning half a degree, then levelled by its floor as
    /// the command line does it.
    fn house(turn: f64) -> Scan {
        let mut j = Jitter(77);
        let fuzz = |j: &mut Jitter| (j.next() + j.next() + j.next()) * 1.4;
        let mut pts: Vec<[f64; 3]> = Vec::new();
        let (nx, ny) = (250usize, 167usize);
        for i in 0..=nx {
            for k in 0..=ny {
                let (x, y) = (i as f64 * 0.12, k as f64 * 0.12);
                pts.push([x, y, 0.0]);
                pts.push([x, y, 8.0]);
            }
        }
        let wall = |a: P2, b: P2, top: f64, gap: Option<[f64; 4]>, j: &mut Jitter| {
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            let mut out = Vec::new();
            for i in 0..=((len / 0.06) as usize) {
                for k in 0..=((top / 0.12) as usize) {
                    let (t, z) = (i as f64 * 0.06, k as f64 * 0.12);
                    if gap.is_some_and(|g| t >= g[0] && t <= g[1] && z >= g[2] && z <= g[3]) {
                        continue;
                    }
                    out.push([a[0] + (b[0] - a[0]) * t / len, a[1] + (b[1] - a[1]) * t / len, z]);
                }
            }
            let _ = j;
            out
        };
        pts.extend(wall([0.0, 0.0], [30.0, 0.0], 8.0, Some([8.0, 13.0, 3.0, 7.0]), &mut j));
        pts.extend(wall([30.0, 0.0], [30.0, 20.0], 8.0, None, &mut j));
        pts.extend(wall([0.0, 20.0], [30.0, 20.0], 8.0, Some([20.0, 29.0, 0.0, 7.0]), &mut j));
        pts.extend(wall([0.0, 0.0], [0.0, 20.0], 8.0, None, &mut j));
        for x in [18.0 - 0.1875, 18.0 + 0.1875] {
            pts.extend(wall([x, 0.0], [x, 20.0], 8.0, Some([9.0, 12.0, 0.0, 6.67]), &mut j));
        }
        pts.extend(wall([4.0, 6.0], [10.0, 6.0], 2.5, None, &mut j));
        pts.extend(wall([4.0, 9.0], [10.0, 9.0], 2.5, None, &mut j));
        pts.extend(wall([4.0, 6.0], [4.0, 9.0], 2.5, None, &mut j));
        pts.extend(wall([10.0, 6.0], [10.0, 9.0], 2.5, None, &mut j));
        pts.extend(wall([1.0, 14.0], [8.0, 14.0], 3.0, None, &mut j));
        pts.extend(wall([1.0, 17.0], [8.0, 17.0], 3.0, None, &mut j));
        pts.extend(wall([1.0, 14.0], [1.0, 17.0], 3.0, None, &mut j));
        pts.extend(wall([8.0, 14.0], [8.0, 17.0], 3.0, None, &mut j));
        pts.extend(wall([20.0, 18.0], [29.0, 18.0], 7.0, None, &mut j));
        // Strays outside, one in fifty.
        let n = pts.len() / 50;
        for i in 0..n {
            let f = i as f64 / n as f64;
            pts.push([-6.0 + 42.0 * ((f * 7919.0) % 1.0), -6.0 + 32.0 * ((f * 104729.0) % 1.0), -1.0 + 11.0 * ((f * 1299709.0) % 1.0)]);
        }
        let (t, lean) = (turn.to_radians(), 0.5f64.to_radians());
        let points = pts
            .into_iter()
            .map(|p| {
                let (x, y, z) = (p[0] + fuzz(&mut j), p[1] + fuzz(&mut j), p[2] + fuzz(&mut j));
                let (x, y) = (x * t.cos() - y * t.sin(), x * t.sin() + y * t.cos());
                [x * lean.cos() - z * lean.sin(), y, x * lean.sin() + z * lean.cos()]
            })
            .collect();
        let mut scan = Scan { points, lost: Vec::new() };
        scan.level_by_floor();
        scan
    }

    /// A house lies at any angle on a phone's scan: the same house, found the same, turned
    /// 0, 5, 17, 33, 45, 60, 75 and 88 degrees (33 once read no wall at all: how many squares
    /// fit under a ceiling read to the inch was taken for where the ceiling is).
    #[test]
    fn the_same_house_is_found_the_same_at_any_angle() {
        for turn in [0.0, 5.0, 17.0, 33.0, 45.0, 60.0, 75.0, 88.0] {
            let f = find(&house(turn));
            let at = format!("turned {turn}");
            assert_eq!(f.walls.len(), 5, "{at}: {:?}", f.walls);
            assert_eq!(f.walls.iter().map(|w| w.openings.iter().filter(|o| !o.window).count()).sum::<usize>(), 1, "{at}");
            assert_eq!(f.walls.iter().map(|w| w.openings.iter().filter(|o| o.window).count()).sum::<usize>(), 1, "{at}");
            assert_eq!(f.things.len(), 3, "{at}: {:?}", f.things);
            let part = f.walls.iter().find(|w| w.thickness.is_some()).unwrap_or_else(|| panic!("{at}: no wall of two faces"));
            assert!((part.thickness.unwrap() - 0.375).abs() < 1.0 / 24.0, "{at}: {:?}", part.thickness);
            // The rooms' faces: back in the house's own frame the outside faces are at east 0 and 30, north 0 and 20.
            let t = turn.to_radians();
            let back = |p: P2| [p[0] * t.cos() + p[1] * t.sin(), -p[0] * t.sin() + p[1] * t.cos()];
            for w in f.walls.iter().filter(|w| w.thickness.is_none()) {
                let (a, b) = (back(w.from), back(w.to));
                let (along_x, along_y) = ((a[0] - b[0]).abs() > 10.0, (a[1] - b[1]).abs() > 10.0);
                assert!(along_x != along_y, "{at}: {a:?} {b:?}");
                let (face, want): (f64, &[f64]) = if along_x { (a[1], &[0.0, 20.0]) } else { (a[0], &[0.0, 30.0]) };
                assert!(want.iter().any(|x| (face - x).abs() < 1.0 / 12.0), "{at}: a face at {face} is not at {want:?}");
            }
            assert!((part.length() - 20.0).abs() < 1.0 / 6.0, "{at}: {}", part.length());
        }
    }

    /// The speed: a few hundred thousand points in seconds, with strays and scatter in them.
    #[test]
    fn a_scan_of_a_few_hundred_thousand_points_is_quick() {
        let mut j = Jitter(10);
        let mut pts = room(30.0, 20.0, true, &mut j);
        for (a, b) in [([0.0, 0.0], [30.0, 0.0]), ([30.0, 0.0], [30.0, 20.0]), ([0.0, 20.0], [30.0, 20.0]), ([0.0, 0.0], [0.0, 20.0])] {
            for _ in 0..8 {
                pts.extend(face(a, b, 0.0, 8.0, None, &mut j));
            }
        }
        for i in 0..4000 {
            pts.push([i as f64 * 0.013 % 40.0 - 5.0, i as f64 * 0.0071 % 30.0 - 5.0, i as f64 * 0.0031 % 11.0 - 1.0]);
        }
        assert!(pts.len() > 100_000, "{}", pts.len());
        let t = std::time::Instant::now();
        let f = found_in(pts);
        assert_eq!(f.walls.len(), 4, "{:?}", f.walls);
        assert!(t.elapsed().as_secs() < 20, "{:?}", t.elapsed());
    }
}
