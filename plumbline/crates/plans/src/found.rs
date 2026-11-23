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
/// A wall seen only above what stands before it (call 179, A2): a cell with points in the top band
/// (the 1 ft 3 in under the ceiling's own 3 in), at this many of its quarter-foot tiers, is upright
/// too: 9 in of wall showing above a cabinet 7 ft high, or more.
const TOP_BAND: f64 = 1.5;
const TOP_TIER: f64 = 0.25;
const TOP_TIERS: u32 = 3;
/// How finely the direction of a line is told in the first vote, in degrees; the refit
/// after it is far finer.
const VOTE_DEGREES: f64 = 0.5;
/// How finely a line's place is told in the first vote, in feet; three of these are added
/// so that a line between two of them is not split.
const VOTE_FEET: f64 = 0.1;
/// A face's points keep to its line within this many times as closely as the best face's keep to theirs, or it
/// is not a face; and a line is tried square to a face found if it is within this many degrees of its direction.
const STRAIGHT: f64 = 1.5;
const LEAN_MOST: f64 = 8.0;
/// The width, across a line, the points near it are counted in to tell where its face is.
const FINE_BIN: f64 = 0.025;
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
/// An upright stretch at least this long that is not a face (too short, or not straight) is counted and said (call 179, A5).
const SHORT_FACE: f64 = 1.0;
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
/// A face is looked at as a wall that something tall stands before, and not as a thing that stops short,
/// only where its points reach within 6 in of the ceiling over at least this share of its length (call 179,
/// A2), and the rest of it is hidden: a sofa's back reaches it over none.
const SOME_REACHES_CEILING: f64 = 0.3;
/// Square: within 1 degree of a main direction (call 173).
const SQUARE_DEGREES: f64 = 1.0;
/// Parallel, for two faces of one wall: within 1 degree; apart 2 in to 16 in; side by
/// side for at least this much of their length.
const PAIR_DEGREES: f64 = 1.0;
const PAIR_LEAST: f64 = 2.0 / 12.0;
const PAIR_MOST: f64 = 16.0 / 12.0;
/// ... and up to this far apart where no floor is seen between them (call 179, A6): of the floor a gap would
/// show, less than this share is seen, not counting 3 in each side of it.
const PAIR_MOST_BARE: f64 = 24.0 / 12.0;
const FLOOR_BETWEEN: f64 = 0.1;
const FLOOR_MARGIN: f64 = 0.25;
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
/// Openings within this of one another in width and in head are one type (call 179, A9): an inch.
const ONE_TYPE: f64 = 1.0 / 12.0;
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
/// A wall face stands behind a face, for the face to be "a wall or something tall standing at the
/// wall", when it lies this far behind it or nearer, on the side the scan did not see it from (call 179, A3).
const BEHIND: f64 = 3.0;
/// Something stands before a gap in a wall, for it to be hidden and not open, along at least this
/// share of the gap (call 179, A2).
const STANDS_ALONG: f64 = 0.9;


// ---- the ceiling over the plan (call 179, A4) ----

/// The side of a square of plan the ceiling is looked for in (3 ft apart or more on a thin scan, as the
/// squares of the walls are).
const CEIL_CELL: f64 = 1.0;
/// Nothing under this high above the floor is a ceiling.
const LEAST_CEILING: f64 = 5.0;
/// A surface is a ceiling over a cell when this share of the points a flat surface of that cell would hold lie
/// within 6 in of one height (a wall across the cell holds half that at most).
const CEILING_SHARE: f64 = 0.7;
/// The ceiling is one height where its middle 90 in 100 lie within 3 in of one another; two heights where they
/// lie in groups this far apart or more; otherwise it slopes.
const CEIL_FLAT: f64 = 0.25;
const CEIL_STEP: f64 = 0.4;
/// A group of heights is a ceiling of its own where it is this share of the whole.
const CEIL_GROUP: f64 = 0.05;
/// How far out from a wall, each side, the ceiling over it is looked at.
const CEIL_OUT: f64 = 1.5;

/// The ceiling over the plan, as its height above the floor.
struct Ceil {
    /// One height over the whole house: the usual, and what the scan's own floor and ceiling say.
    flat: Option<f64>,
    cells: HashMap<(i64, i64), f64>,
    side: f64,
    /// Where no cell is known near.
    middle: f64,
    lowest: f64,
    highest: f64,
}

impl Ceil {
    fn at(&self, p: P2) -> f64 {
        if let Some(h) = self.flat {
            return h;
        }
        let (cx, cy) = ((p[0] / self.side).floor() as i64, (p[1] / self.side).floor() as i64);
        for ring in 0..=3i64 {
            let mut best: Option<(i64, f64)> = None;
            for dx in -ring..=ring {
                for dy in -ring..=ring {
                    if dx.abs().max(dy.abs()) != ring {
                        continue;
                    }
                    if let Some(h) = self.cells.get(&(cx + dx, cy + dy)) {
                        if best.is_none_or(|b| dx * dx + dy * dy < b.0) {
                            best = Some((dx * dx + dy * dy, *h));
                        }
                    }
                }
            }
            if let Some((_, h)) = best {
                return h;
            }
        }
        self.middle
    }
    /// The ceiling over a wall at `p` whose normal is `n`: the lower of what is over either side of it.
    fn over(&self, p: P2, n: P2) -> f64 {
        if let Some(h) = self.flat {
            return h;
        }
        self.at([p[0] + n[0] * CEIL_OUT, p[1] + n[1] * CEIL_OUT]).min(self.at([p[0] - n[0] * CEIL_OUT, p[1] - n[1] * CEIL_OUT]))
    }
}

/// What the ceiling is, from the points over the floor: the ceiling to use (None where none shows), and a
/// sentence where it is not one height. `side` is the cell's side, `rho` how thick the scan is.
fn ceiling_of(scan: &Scan, floor: f64, scan_ceiling: Option<f64>, rho: Option<f64>, side: f64) -> (Option<Ceil>, Option<String>) {
    let key = |p: &[f64; 3]| ((p[0] / side).floor() as i64, (p[1] / side).floor() as i64);
    // (sorted by cell, then taken cell by cell: a map with a list for each cell costs more than this on a
    // scan of a few hundred thousand points)
    let mut high: Vec<((i64, i64), f32)> = Vec::new();
    let mut low: Vec<(i64, i64)> = Vec::new();
    for p in &scan.points {
        let z = p[2] - floor;
        if z >= LEAST_CEILING && z <= 30.0 {
            high.push((key(p), z as f32));
        } else if z.abs() <= SET_ASIDE {
            low.push(key(p));
        }
    }
    high.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    low.sort_unstable();
    low.dedup();
    let under: HashSet<(i64, i64)> = low.into_iter().collect();
    let least = (CEILING_SHARE * rho.unwrap_or(0.0) * side * side).max(6.0);
    let mut cells: HashMap<(i64, i64), f64> = HashMap::new();
    for run in high.chunk_by(|a, b| a.0 == b.0) {
        let mut bins = vec![0u32; 300];
        for (_, z) in run {
            bins[(((f64::from(*z) - LEAST_CEILING) / 0.1) as usize).min(299)] += 1;
        }
        // The highest half foot with as many points in it as a surface would hold.
        let window = (0..295).rev().find(|b| f64::from(bins[*b..*b + 5].iter().sum::<u32>()) >= least);
        if let Some(b) = window {
            let lo = LEAST_CEILING + b as f64 * 0.1;
            let inside: Vec<f64> = run.iter().map(|(_, z)| f64::from(*z)).filter(|z| *z >= lo && *z < lo + 0.5).collect();
            cells.insert(run[0].0, inside.iter().sum::<f64>() / inside.len() as f64);
        }
    }
    let covered = cells.keys().filter(|k| under.contains(*k)).count();
    if cells.len() < 5 || (covered as f64) < 0.3 * under.len() as f64 {
        return (scan_ceiling.map(|c| Ceil { flat: Some(c - floor), cells: HashMap::new(), side, middle: c - floor, lowest: c - floor, highest: c - floor }), None);
    }
    let mut hs: Vec<f64> = cells.values().copied().collect();
    hs.sort_by(f64::total_cmp);
    let (p5, p95) = (hs[hs.len() / 20], hs[hs.len() - 1 - hs.len() / 20]);
    let middle = hs[hs.len() / 2];
    if p95 - p5 <= CEIL_FLAT {
        let h = scan_ceiling.map_or(middle, |c| c - floor);
        return (Some(Ceil { flat: Some(h), cells: HashMap::new(), side, middle: h, lowest: h, highest: h }), None);
    }
    // Groups of heights a step apart or more.
    let mut groups: Vec<Vec<f64>> = vec![vec![hs[0]]];
    for h in &hs[1..] {
        if h - groups.last().and_then(|g| g.last()).copied().unwrap_or(*h) >= CEIL_STEP {
            groups.push(Vec::new());
        }
        if let Some(g) = groups.last_mut() {
            g.push(*h);
        }
    }
    groups.retain(|g| g.len() as f64 >= CEIL_GROUP * hs.len() as f64);
    let tape = |x: f64| plumbline_units(x);
    let note = if groups.len() >= 2 {
        let names: Vec<String> = groups.iter().map(|g| tape(g[g.len() / 2])).collect();
        format!("the ceiling is not one height: {}; each wall is judged against the ceiling over it (a wall between two rooms, against the lower), and a wall that runs under both is laid at the height of the ceiling over its middle (the step in its top is not drawn)", names.join(" and "))
    } else {
        format!("the ceiling slopes, from {} to {}; a wall under it is judged against the ceiling over it and laid at the height of the ceiling over its middle (its sloped top is not drawn)", tape(hs[0]), tape(hs[hs.len() - 1]))
    };
    (Some(Ceil { flat: None, cells, side, middle, lowest: hs[0], highest: hs[hs.len() - 1] }), Some(note))
}

/// How thick a scan is, and what is worked from it (call 179, A1). Every count in this file that depends on
/// how many points there are was tuned on scans of about this many points to the square foot of floor (a
/// phone's scan of a room is 100 or more): on a scan as thick or thicker the measures are as they were; on a
/// thinner one the squares and the cells a face is looked for in are made larger by the square root of how
/// much thinner it is, so that each holds as many points as it did, and the least a face is voted for by is
/// made fewer. Under the least that has been tried and found to tell a wall (the thinnest scan of a house
/// this found every wall of, tried at several sizes and angles) nothing is found, and that is said.
const REFERENCE_DENSITY: f64 = 80.0;
const LEAST_DENSITY: f64 = 8.0;
/// A scan under this share of the reference thickness is said to be thin, and what that does to what it finds.
const THIN_NOTE_BELOW: f64 = 0.7;
/// A line is voted for by at least this many upright cells however thin the scan.
const MOST_THIN_VOTES: u32 = 8;

/// A number of points, in words.
fn figure(x: f64) -> String {
    format!("{} points", x.round() as i64)
}

/// Points to the square foot of the scan's floor: of the 1 ft squares that hold two or more points within 3 in
/// of the floor, the middle one's count. (The floor shows much as a wall does, at the same spacing; a square
/// at a wall's foot or under furniture holds fewer and does not set it.)
fn density(scan: &Scan, floor: f64) -> Option<f64> {
    let mut at: Vec<(i64, i64)> = scan.points.iter().filter(|p| (p[2] - floor).abs() <= SET_ASIDE).map(|p| (p[0].floor() as i64, p[1].floor() as i64)).collect();
    at.sort_unstable();
    let held: Vec<f64> = at.chunk_by(|a, b| a == b).map(|r| r.len()).filter(|n| *n >= 2).map(|n| n as f64).collect();
    (held.len() >= 10).then(|| median(held))
}

/// A house is not under this high from floor to ceiling, nor over the next, nor over the third
/// long (call 179, A7): a scan read the wrong way (meters as feet, or up as the other axis) is a
/// house that is not one, and nothing is laid on its word.
const LEAST_HIGH: f64 = 6.0;
const MOST_HIGH: f64 = 15.0;
const MOST_LONG: f64 = 150.0;

/// A wall found.
#[derive(Clone, Debug, Default)]
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
    /// Why this may not be a wall though it reaches the ceiling (call 179, A3): it is said, and
    /// not laid unless the person says it is a wall.
    pub doubt: Option<String>,
    /// How high the ceiling over it is, above the floor; None where it was not told.
    pub height: Option<f64>,
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
    /// Why this is not likely the house (a floor to ceiling under 6 ft or over 15 ft, or a
    /// house over 150 ft): said first, and nothing is laid without the person's word (call 179, A7).
    pub not_likely: Option<String>,
    /// The ends of the walls found that meet no other wall found (call 179, A2), as places on the
    /// plan, with which wall (its place in `walls`): where the walls found do not close. Walls in
    /// doubt are not counted, either way.
    pub open_ends: Vec<(usize, P2)>,
    /// How thick the scan is: points to the square foot of floor, where a floor shows (call 179, A1).
    pub density: Option<f64>,
    /// Why nothing is found: the scan is too thin to tell a wall from furniture.
    pub too_thin: Option<String>,
    /// Where the scan is thinner than the measures were tuned on: what that does to what is said.
    pub thin_note: Option<String>,
    /// Where the ceiling is not one height (two, or a slope): said, with what is done about it.
    pub ceiling_note: Option<String>,
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
    /// Its points reach the ceiling over most of its length (a wall, or what a wall's lower
    /// part is hidden from: whether the floor is reached too is settled once what stands in
    /// front of it is known).
    tall: bool,
    /// For each column of 3 in along it from `s0` at the time it was looked at: whether its points
    /// reach down to within 6 in of the floor.
    floor_cols: Vec<bool>,
    /// And whether they reach up to within 6 in of the ceiling.
    ceil_cols: Vec<bool>,
    /// Its points reach the ceiling over less than most of its length but over a good part of it: a wall
    /// that something tall stands before, or a thing that stops short (settled once what stands is known).
    partial: bool,
    /// It reaches the ceiling and not the floor and nothing stands before it: not a wall, not a thing.
    hung: bool,
    cols_from: f64,
    /// The ceiling's height over it, above the floor (the lower side's, where the two differ).
    ceil: f64,
    /// The side of the squares its columns were looked at in.
    sq: f64,
    /// How much of it the scan shows: the squares of 3 in along and up that hold points, in square feet.
    seen_area: f64,
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
    /// The side of a square, in feet: 3 in on a dense scan, larger on a thin one (`Measures`).
    sq: f64,
}

impl Grid {
    /// `tops(s)` is the ceiling's height over the way along `s`: squares from there up are not part of the wall
    /// (they are marked full, so that no hole is found above a lower ceiling).
    fn new(pts: &[[f64; 2]], s0: f64, s1: f64, high: f64, sq: f64, tops: &dyn Fn(f64) -> f64) -> Grid {
        let cols = (((s1 - s0) / sq).ceil() as usize).max(1);
        let rows = (((high - SET_ASIDE) / sq + 1e-9).floor() as usize).max(1);
        let mut n = vec![0u32; cols * rows];
        for p in pts {
            let (c, r) = (((p[0] - s0) / sq).floor(), ((p[1] - SET_ASIDE) / sq).floor());
            if c >= 0.0 && r >= 0.0 && (c as usize) < cols && (r as usize) < rows {
                n[r as usize * cols + c as usize] += 1;
            }
        }
        let held = median(n.iter().filter(|x| **x > 0).map(|x| f64::from(*x)).collect());
        let need = ((held / 4.0).ceil() as u32).max(2);
        for c in 0..cols {
            let top = tops(s0 + (c as f64 + 0.5) * sq) - SET_ASIDE;
            for r in 0..rows {
                if SET_ASIDE + r as f64 * sq >= top {
                    n[r * cols + c] = u32::MAX;
                }
            }
        }
        Grid { cols, rows, n, need, s0, sq }
    }
    fn full(&self, c: usize, r: usize) -> bool {
        self.n[r * self.cols + c] >= self.need
    }
    fn col_range(&self, c: usize) -> f64 {
        self.s0 + c as f64 * self.sq
    }
}

/// A face reaches the ceiling or stops short of it: its points run to within 6 in of the ceiling
/// over most of its length, or they do not (and then how tall it is). Whether it also reaches the
/// floor is kept for the columns it is looked at in: where it does not, what stands in front of it
/// may be why (call 179, A2), which is settled once what stands is known.
fn classify(f: &mut Face, ceil: &Ceil, sq: f64) {
    // The ceiling over each column of it: where it is not one height, the lower of the two sides' (a wall
    // between a low room and a high one reaches the ceiling if it reaches the low one's: call 179, A4).
    let n_of = normal(f.theta);
    let cols_n = (((f.s1 - f.s0) / sq).ceil() as usize).max(1);
    let heights: Vec<f64> = (0..cols_n).map(|c| ceil.over(f.at(f.s0 + (c as f64 + 0.5) * sq), n_of)).collect();
    f.ceil = median(heights.clone());
    let high = heights.iter().copied().fold(0.0, f64::max) - SET_ASIDE;
    let tops = |s: f64| heights[(((s - f.s0) / sq).floor().max(0.0) as usize).min(cols_n - 1)];
    let g = Grid::new(&f.pts.iter().map(|p| [p[0], p[2]]).collect::<Vec<_>>(), f.s0, f.s1, high, sq, &tops);
    let cols = g.cols as f64;
    // Within 6 in of the ceiling: the strip from 6 in under it to the 3 in set aside under it, each column
    // of it with as many points as a square of the grid needs, scaled to the strip's height (it is a quarter
    // of a foot, and a square on a thin scan is more). Not the squares whose tops are above it: how many
    // squares fit under the ceiling depends on the inch the floor and the ceiling were read to, and on how
    // large a square is, and a scan turned and leaned read a hair over.
    // On a thin scan the strip is as much taller as the squares are larger (`Measures`): a strip of 3 in holds
    // two or three points to a column and chance alone empties a third of them.
    let strip_high = SET_ASIDE * sq / SQUARE;
    let mut strip = vec![0u32; g.cols];
    for p in &f.pts {
        let c = ((p[0] - f.s0) / sq).floor();
        if c >= 0.0 && (c as usize) < g.cols && p[2] >= heights[c as usize] - SET_ASIDE - strip_high {
            strip[c as usize] += 1;
        }
    }
    let need_top = ((f64::from(g.need) * strip_high / sq).ceil() as u32).max(2);
    f.ceil_cols = strip.iter().map(|n| *n >= need_top).collect();
    let ceiling_cols = f.ceil_cols.iter().filter(|x| **x).count() as f64;
    f.floor_cols = (0..g.cols).map(|c| g.n[c] >= g.need).collect();
    f.cols_from = f.s0;
    f.sq = sq;
    f.seen_area = (0..g.rows).map(|r| (0..g.cols).filter(|c| g.n[r * g.cols + *c] != u32::MAX && g.full(*c, r)).count()).sum::<usize>() as f64 * sq * sq;
    f.tall = ceiling_cols / cols >= WALL_REACHES_CEILING;
    f.partial = !f.tall && ceiling_cols / cols >= SOME_REACHES_CEILING;
    f.hung = false;
    f.wall = f.tall || f.partial;
    f.height = if f.tall { 0.0 } else { top_of(&f.pts) };
}

/// Where, along `f`, the line of `h` crosses `f`'s own line, and where along `h` that is.
fn cross_at(f: &Face, h: &Face) -> Option<(f64, f64)> {
    let (d, n) = (dir(f.theta), normal(f.theta));
    let (hd, hn) = (dir(h.theta), normal(h.theta));
    let denom = dot(d, hn);
    if (f.theta - h.theta).sin().abs() < NOT_PARALLEL || denom.abs() < 1e-9 {
        return None;
    }
    let s = (h.rho - f.rho * dot(n, hn)) / denom;
    let q = [s * d[0] + f.rho * n[0], s * d[1] + f.rho * n[1]];
    Some((s, dot(q, hd)))
}

/// Faces that reach the ceiling and are "a wall or something tall standing at the wall" (call
/// 179, A3): seen from one side only, both ends meeting no wall, with a wall face behind them
/// within 3 ft that goes on past an end of theirs. A cabinet bank within 6 in of the ceiling is
/// that: its back is not seen, the wall behind it shows either side. What it is that stands there
/// is the person's to say. (A wall of two faces that stops in the room has a face behind each
/// face too, but that face goes no further than the other: it is the wall's other side.)
fn tall_at_a_wall(faces: &[Face], wall_ids: &[usize]) -> HashMap<usize, String> {
    let mut candidates: HashMap<usize, f64> = HashMap::new();
    for &i in wall_ids {
        let f = &faces[i];
        if !f.tall || f.seen[0] == f.seen[1] {
            continue;
        }
        let side = if f.seen[0] { 1.0 } else { -1.0 };
        let n = normal(f.theta);
        let behind: Vec<(f64, f64, f64)> = wall_ids
            .iter()
            .filter(|&&j| j != i)
            .filter_map(|&j| {
                let g = &faces[j];
                let mut dt = (f.theta - g.theta).abs();
                dt = dt.min(std::f64::consts::PI - dt);
                let across = (dot(g.at(g.mid()), n) - f.rho) * -side;
                if dt.to_degrees() > PAIR_DEGREES || across <= 0.0 || across > BEHIND {
                    return None;
                }
                let shift = dot(g.at(g.mid()), dir(f.theta)) - g.mid();
                Some((g.s0 + shift, g.s1 + shift, across))
            })
            .collect();
        let near = |e: f64| behind.iter().any(|b| e >= b.0 - BEHIND && e <= b.1 + BEHIND);
        let goes_on = behind.iter().any(|b| b.0 < f.s0 - CORNER || b.1 > f.s1 + CORNER);
        if near(f.s0) && near(f.s1) && goes_on {
            candidates.insert(i, behind.iter().map(|b| b.2).fold(f64::MAX, f64::min));
        }
    }
    let mut out = HashMap::new();
    for (&i, &across) in &candidates {
        let f = &faces[i];
        let meets = |e: f64| {
            wall_ids.iter().filter(|j| **j != i && !candidates.contains_key(j)).any(|&j| {
                let h = &faces[j];
                cross_at(f, h).is_some_and(|(s, along)| (s - e).abs() <= CORNER && along >= h.s0 - CORNER && along <= h.s1 + CORNER)
            })
        };
        if !meets(f.s0) && !meets(f.s1) {
            out.insert(i, format!("a wall or something tall standing at the wall: it reaches the ceiling, but neither end meets a wall and a wall face stands {} behind it", plumbline_units(across)));
        }
    }
    out
}

/// The same, against every face but `skip`.
/// How far the points of a face lie from its line, root mean square.
fn rms_across(pts: &[[f64; 3]], rho: f64) -> f64 {
    (pts.iter().map(|p| (p[1] - rho) * (p[1] - rho)).sum::<f64>() / pts.len().max(1) as f64).sqrt()
}

fn crosses_other(p: P2, theta: f64, faces: &[Face], skip: usize) -> bool {
    faces.iter().enumerate().any(|(j, g)| {
        j != skip && (theta - g.theta).sin().abs() >= CROSSES && (dot(p, normal(g.theta)) - g.rho).abs() <= BAND + 0.05 && dot(p, dir(g.theta)) >= g.s0 - SNAP && dot(p, dir(g.theta)) <= g.s1 + SNAP
    })
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
fn strongest(votes: &[P2], cos: &[f64], sin: &[f64], least: u32) -> Option<(usize, f64, u32)> {
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
    best.filter(|x| x.2 >= least).map(|(k, b, score)| (k, (b as f64 - 1.0 + 0.5) * VOTE_FEET - reach, score))
}

/// The stretches along a line that points run through, as (from, to) along it: columns of
/// 3 in with at least COLUMN_POINTS points each are there, and a gap of GAP or more parts one
/// stretch from the next. `s` is sorted.
fn runs_of(s: &[f64], sq: f64) -> Vec<(f64, f64)> {
    let Some((first, last)) = s.first().zip(s.last()) else { return Vec::new() };
    let cols = (((last - first) / sq).floor() as usize) + 1;
    let mut count = vec![0usize; cols];
    for x in s {
        count[(((x - first) / sq).floor() as usize).min(cols - 1)] += 1;
    }
    let gap_cols = (GAP / sq).round() as usize;
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
    runs.into_iter().map(|(a, b)| (first + a as f64 * sq, first + (b + 1) as f64 * sq)).collect()
}

/// A vote's line told again from the points near it (see where it is used): its direction from all of them, and
/// its place from the fullest 3 in across it of them.
fn seed_line(pool: &[P2], theta: f64, rho: f64) -> (f64, f64) {
    let n = normal(theta);
    let near: Vec<P2> = pool.iter().copied().filter(|p| (dot(*p, n) - rho).abs() <= SEED_BAND + 0.05).collect();
    if near.len() < 20 {
        return (theta, rho);
    }
    // Its own direction, if that is within a degree and a half of the vote's (more is another wall's points
    // crossing it).
    let theta = match long_way(&near) {
        Some((t, _)) => {
            let t = if (t - theta).abs() > std::f64::consts::FRAC_PI_2 { if t > theta { t - std::f64::consts::PI } else { t + std::f64::consts::PI } } else { t };
            if (t - theta).abs().to_degrees() <= 1.5 { t } else { theta }
        }
        None => theta,
    };
    let n = normal(theta);
    let offs: Vec<f64> = near.iter().map(|p| dot(*p, n)).collect();
    let bins = (2.0 * (SEED_BAND + 0.05) / FINE_BIN).ceil() as usize + 1;
    let mut count = vec![0u32; bins];
    for o in &offs {
        let b = ((o - (rho - SEED_BAND - 0.05)) / FINE_BIN).floor();
        if b >= 0.0 && (b as usize) < bins {
            count[b as usize] += 1;
        }
    }
    // The fullest three bins together (a face's scatter is a few bins wide).
    let best = (0..bins).max_by_key(|b| (b.saturating_sub(1)..=(*b + 1).min(bins - 1)).map(|i| count[i]).sum::<u32>()).unwrap_or(0);
    let centre = rho - SEED_BAND - 0.05 + (best as f64 + 0.5) * FINE_BIN;
    let mine: Vec<f64> = offs.into_iter().filter(|o| (o - centre).abs() <= 0.1).collect();
    if mine.is_empty() {
        return (theta, rho);
    }
    (theta, mine.iter().sum::<f64>() / mine.len() as f64)
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

/// How long a scan is on the plan, east to west and north to south: the middle 95 in 100 of its
/// points, so that the strays outside the house (one in fifty) do not make it long.
fn span(scan: &Scan, axis: usize) -> f64 {
    let mut v: Vec<f64> = scan.points.iter().map(|p| p[axis]).collect();
    if v.len() < 40 {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() - 1 - v.len() / 40] - v[v.len() / 40]
}

fn plan_size(scan: &Scan) -> (f64, f64) {
    (span(scan, 0), span(scan, 1))
}

/// Whether this is likely a house at all, from the scan's own size (call 179, A7): the sentence
/// when it is not.
fn unlikely(scan: &Scan, heights: Option<(f64, f64)>) -> Option<String> {
    let ft_in = |x: f64| plumbline_units(x);
    let check = "this is not likely the house: check the measure and which way is up";
    if let Some((lowest, highest)) = heights {
        if lowest < LEAST_HIGH {
            return Some(format!("{check} (floor to ceiling reads {}, under {})", ft_in(lowest), ft_in(LEAST_HIGH)));
        }
        if highest > MOST_HIGH {
            return Some(format!("{check} (floor to ceiling reads {}, over {})", ft_in(highest), ft_in(MOST_HIGH)));
        }
    } else {
        // No ceiling shows: where the whole scan is lower than a room, that is why.
        let high = span(scan, 2);
        if high > 0.0 && high < LEAST_HIGH {
            return Some(format!("{check} (the whole scan is only {} high, and {} by {} across; a room is at least {} high)", ft_in(high), ft_in(plan_size(scan).0), ft_in(plan_size(scan).1), ft_in(LEAST_HIGH)));
        }
    }
    let (east, north) = plan_size(scan);
    if east.max(north) > MOST_LONG {
        return Some(format!("{check} (the scan is {} across, and a house is not over {})", ft_in(east.max(north)), ft_in(MOST_LONG)));
    }
    None
}

fn plumbline_units_count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// A length as a tape reads it, feet and inches to the inch.
fn plumbline_units(feet: f64) -> String {
    let inches = (feet * 12.0).round() as i64;
    format!("{}' {}\"", inches.div_euclid(12), inches.rem_euclid(12))
}

/// Find the walls of a scan that has been levelled by its floor.
pub fn find(scan: &Scan) -> Found {
    let mut out = Found::default();
    let (floor, scan_ceiling) = scan.floor_and_ceiling();
    out.floor = floor;
    out.ceiling = scan_ceiling;
    let Some(floor) = floor else {
        out.not_likely = unlikely(scan, None);
        out.left_out.push("no floor shows in the scan, so what stands on it cannot be told".into());
        return out;
    };
    // How thick the scan is, and the measures that depend on it (call 179, A1).
    let rho = density(scan, floor);
    out.density = rho;
    if let Some(r) = rho.filter(|r| *r < LEAST_DENSITY) {
        out.not_likely = unlikely(scan, scan_ceiling.map(|c| (c - floor, c - floor)));
        out.too_thin = Some(format!("the scan is too thin to tell a wall from furniture: about {} to the square foot, and it takes {} or more", figure(r), figure(LEAST_DENSITY)));
        out.left_out.push(out.too_thin.clone().unwrap_or_default());
        return out;
    }
    let k = rho.map_or(1.0, |r| (REFERENCE_DENSITY / r).sqrt().max(1.0));
    let (sq, cell) = (SQUARE * k, CELL * k);
    if rho.is_some_and(|r| r < THIN_NOTE_BELOW * REFERENCE_DENSITY) {
        out.thin_note = Some(format!("this scan is thinner than the measures here were made for ({} to the square foot, against {} and more): a wall is told to a few inches, an opening only to about {} in, and one may be missed or split: check each", figure(rho.unwrap_or(0.0)), figure(REFERENCE_DENSITY), (sq * 12.0).round()));
    }
    // The ceiling over the plan: one height, two, or a slope (call 179, A4).
    let (ceil, ceil_note) = ceiling_of(scan, floor, scan_ceiling, rho, CEIL_CELL * k);
    out.ceiling_note = ceil_note;
    out.not_likely = unlikely(scan, ceil.as_ref().map(|c| (c.lowest, c.highest)));
    let Some(ceil) = ceil else {
        out.no_ceiling = true;
        out.left_out.push("no ceiling shows in the scan: without one, a wall cannot be told from a tall cabinet standing in the room, so no wall is taken".into());
        return out;
    };
    if scan_ceiling.is_none() || ceil.flat.is_none() {
        out.ceiling = Some(floor + ceil.middle);
    }
    let high = ceil.highest - SET_ASIDE;
    let min_votes = ((f64::from(MIN_VOTES) / k).round() as u32).max(MOST_THIN_VOTES);

    // The points that can be a wall's: [east, north, up above the floor].
    let band: Vec<[f64; 3]> = scan.points.iter().filter_map(|p| {
        let z = p[2] - floor;
        (z >= SET_ASIDE && z <= ceil.at([p[0], p[1]]) - SET_ASIDE).then_some([p[0], p[1], z])
    }).collect();
    let key = |x: f64, y: f64| ((x / cell).floor() as i64, (y / cell).floor() as i64);
    // Upright cells: points at several heights in one cell of plan.
    struct Cell {
        n: u32,
        heights: u64,
        /// The points in the top band (1 ft 6 in under the ceiling, less the 3 in set aside), and the
        /// quarter-foot tiers of height they are at: what shows of a wall above what stands before it.
        top: u32,
        top_tiers: u64,
        all: u32,
        sx: f64,
        sy: f64,
    }
    let mut cells: HashMap<(i64, i64), Cell> = HashMap::new();
    for p in &band {
        let here = ceil.at([p[0], p[1]]);
        let mid = p[2] >= 0.5 && p[2] <= here - 0.5;
        let top = p[2] >= here - TOP_BAND;
        if !mid && !top {
            continue;
        }
        let c = cells.entry(key(p[0], p[1])).or_insert(Cell { n: 0, heights: 0, top: 0, top_tiers: 0, all: 0, sx: 0.0, sy: 0.0 });
        if mid {
            c.n += 1;
            c.heights |= 1u64 << (((p[2] - 0.5) / 0.5) as u32).min(63);
        }
        if top {
            c.top += 1;
            c.top_tiers |= 1u64 << (((p[2] - (here - TOP_BAND)) / TOP_TIER) as u32).min(63);
        }
        c.all += 1;
        c.sx += p[0];
        c.sy += p[1];
    }
    let upright: HashSet<(i64, i64)> = cells.iter().filter(|(_, c)| (c.n >= CELL_POINTS && c.heights.count_ones() >= UPRIGHT_HEIGHTS) || (c.top >= CELL_POINTS && c.top_tiers.count_ones() >= TOP_TIERS)).map(|(k, _)| *k).collect();
    let mut votes: Vec<P2> = cells.iter().filter(|(k, _)| upright.contains(*k)).map(|(_, c)| [c.sx / f64::from(c.all), c.sy / f64::from(c.all)]).collect();
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
    // How closely the points of a face keep to its line, as the best faces found so far keep (call 179, A5).
    let mut ref_rms: Option<f64> = None;
    let mut not_straight = 0usize;
    let mut short = 0usize;
    for _ in 0..MOST_LINES {
        let Some((k, rho0, _)) = strongest(&votes, &cos, &sin, min_votes) else { break };
        // The vote's line, as coarse as the vote: the points within SEED_BAND of it, all
        // heights, along it. Its stretches are looked at one at a time: two things on one
        // line with a gap between them (a sofa and a cabinet) are two, and each is fitted by
        // itself, so that a line through both does not lean across the room.
        // The vote told the line to half a degree and its place to 0.3 ft (three bins are added so that a line
        // between two is not split): two faces nearer than that to one another, a wall 3 1/2 in thick, are one
        // peak, and a line fitted to the points of both leans across them and then across the next (call 179,
        // A5). The line is told again from the points themselves, its direction from all that lie near it and its
        // place from the fullest quarter-inch of them: one face, the fuller; the other is the next line's.
        let (theta0, rho0) = seed_line(&pool, (k as f64 * VOTE_DEGREES).to_radians(), rho0);
        let (d0, n0) = (dir(theta0), normal(theta0));
        let mut on: Vec<f64> = band.iter().filter(|p| (dot([p[0], p[1]], n0) - rho0).abs() <= SEED_BAND && !crosses_face([p[0], p[1]], theta0, &faces)).map(|p| dot([p[0], p[1]], d0)).collect();
        on.sort_by(f64::total_cmp);
        let mut taken_out: Vec<(f64, f64)> = Vec::new();
        for (sa, sb) in runs_of(&on, sq) {
            taken_out.push((sa, sb));
            if sb - sa < MIN_FACE {
                if sb - sa >= SHORT_FACE {
                    short += 1;
                }
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
            let points_of = |theta: f64, rho: f64, faces: &[Face]| -> Vec<[f64; 3]> {
                let (d, n) = (dir(theta), normal(theta));
                band.iter().filter(|p| (dot([p[0], p[1]], n) - rho).abs() <= BAND && !crosses_face([p[0], p[1]], theta, faces)).map(|p| [dot([p[0], p[1]], d), dot([p[0], p[1]], n), p[2]]).filter(|p| p[0] >= sa - 0.2 && p[0] <= sb + 0.2).collect()
            };
            let mut pts: Vec<[f64; 3]> = points_of(theta, rho, &faces);
            // A face is fitted along its whole length or it is not found (call 179, A5): its points keep to
            // its line as closely as the other faces' keep to theirs. A line that leans across two walls near
            // one another (a closet's, 3 1/2 in thick and a few feet long) has points all across its band. It is
            // tried again square to a face already found, the way the walls of a house lie; if its points do not
            // keep to that either, it is not a face, and that is said.
            let (mut theta, mut rho) = (theta, rho);
            if let Some(r) = ref_rms.filter(|_| pts.len() >= 20) {
                if rms_across(&pts, rho) > STRAIGHT * r {
                    let square: Option<f64> = faces.iter().flat_map(|f| [f.theta, f.theta + std::f64::consts::FRAC_PI_2, f.theta - std::f64::consts::FRAC_PI_2]).filter(|t| (t - theta).abs().to_degrees() <= LEAN_MOST).min_by(|a, b| (a - theta).abs().total_cmp(&(b - theta).abs()));
                    let again = square.and_then(|t| {
                        let n = normal(t);
                        let d = dir(t);
                        let offs: Vec<f64> = pool.iter().filter(|p| dot(**p, d) >= sa - 0.2 && dot(**p, d) <= sb + 0.2 && (dot(**p, n) - rho).abs() <= 0.4).map(|p| dot(*p, n)).collect();
                        let best = offs.iter().copied().max_by_key(|o| offs.iter().filter(|q| (**q - *o).abs() <= 0.075).count())?;
                        let near: Vec<f64> = offs.into_iter().filter(|o| (o - best).abs() <= 0.075).collect();
                        let r2 = near.iter().sum::<f64>() / near.len() as f64;
                        let p2 = points_of(t, r2, &faces);
                        (p2.len() >= 20 && rms_across(&p2, r2) <= STRAIGHT * r).then_some((t, r2, p2))
                    });
                    match again {
                        Some((t, r2, p2)) => {
                            theta = t;
                            rho = r2;
                            pts = p2;
                        }
                        None => {
                            not_straight += 1;
                            continue;
                        }
                    }
                }
            }
            let (d, n) = (dir(theta), normal(theta));
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
                if s1 - s0 >= SHORT_FACE {
                    short += 1;
                }
                continue;
            }
            // A face stands: its points are at several heights (a table top seen edge on is a
            // line of points all at one).
            let mut tiers: HashMap<usize, usize> = HashMap::new();
            for p in &pts {
                *tiers.entry((p[2] / 0.5) as usize).or_default() += 1;
            }
            let mut top_tiers: HashMap<usize, usize> = HashMap::new();
            let here = ceil.over([(sa + sb) / 2.0 * d[0] + rho * n[0], (sa + sb) / 2.0 * d[1] + rho * n[1]], n);
            for p in pts.iter().filter(|p| p[2] >= here - TOP_BAND) {
                *top_tiers.entry(((p[2] - (here - TOP_BAND)) / TOP_TIER) as usize).or_default() += 1;
            }
            if tiers.values().filter(|n| **n >= 10).count() < UPRIGHT_HEIGHTS as usize && top_tiers.values().filter(|n| **n >= 10).count() < TOP_TIERS as usize {
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
            if pts.len() >= 500 {
                let r = rms_across(&pts, rho);
                ref_rms = Some(ref_rms.map_or(r, |q: f64| q.min(r)));
            }
            let mut face = Face { theta, rho: at_floor, s0, s1, pts, wall: false, squared: false, seen: [false, false], height: 0.0, tall: false, floor_cols: Vec::new(), ceil_cols: Vec::new(), partial: false, hung: false, cols_from: 0.0, ceil: 0.0, sq, seen_area: 0.0 };
            classify(&mut face, &ceil, sq);
            faces.push(face);
            // Its cells are out of the next vote.
            // (as far as the seed band: cells at a corner are drawn a little off their line by the wall that crosses it)
            votes.retain(|p| !((dot(*p, n) - rho).abs() <= SEED_BAND && dot(*p, d) >= sa - 0.3 && dot(*p, d) <= sb + 0.3));
        }
        // Whatever the vote's own cells were, they are out of the next vote: the ones near
        // its line, where its stretches ran.
        let before = votes.len();
        votes.retain(|p| !((dot(*p, n0) - rho0).abs() <= SEED_BAND && taken_out.iter().any(|(a, b)| dot(*p, d0) >= a - 0.3 && dot(*p, d0) <= b + 0.3)));
        if votes.len() == before {
            votes.retain(|p| (dot(*p, n0) - rho0).abs() > SEED_BAND);
        }
    }

    if short + not_straight > 0 {
        let mut why: Vec<String> = Vec::new();
        if short > 0 {
            why.push(format!("{short} under 2 ft long (a wall is at least 2 ft long)"));
        }
        if not_straight > 0 {
            why.push(format!("{not_straight} not straight along the whole length (furniture, or two walls close together)"));
        }
        out.left_out.push(format!("{} of upright points not taken as walls: {}", plumbline_units_count(short + not_straight, "stretch", "stretches"), why.join(", ")));
    }
    // A line drawn through the points of other things (a sofa's front and back, a table's
    // corner) has a few points of its own: a face with under a quarter of the usual
    // number of points to the square foot of face is not one.
    // (a face that reaches the ceiling and the floor over half its length is measured over its whole face,
    // one that does not over its length and its height)
    let floor_reach = |f: &Face| f.floor_cols.iter().filter(|x| **x).count() as f64 / f.floor_cols.len().max(1) as f64;
    let per_foot: Vec<f64> = faces.iter().map(|f| f.pts.len() as f64 / ((f.s1 - f.s0) * if f.tall && floor_reach(f) >= WALL_REACHES_FLOOR { f.ceil } else { f.height.max(0.5) })).collect();
    let usual = median(per_foot.clone());
    // A face that reaches the ceiling is also kept if what the scan shows of it is dense: a wall seen only
    // above what stands before it has few points to its whole face and as many to what it shows as any (call
    // 179, A2). A face that stops short is held to the whole of what it spans: a line through other things'
    // points shows little and spans much.
    let keep: Vec<bool> = faces.iter().zip(&per_foot).map(|(f, d)| *d >= usual / SPARSE || (f.tall && f.pts.len() as f64 / f.seen_area.max(sq * sq) >= usual / SPARSE)).collect();
    let mut keep = keep.into_iter();
    faces.retain(|_| keep.next().unwrap_or(true));
    // Each face's ends again, now that every face is known: where another face crosses it, that face's
    // points near the corner are not this face's (call 179, A2: a wall seen only above what stands before
    // it has few points of its own, and a corner's worth of the crossing wall's would carry its end far past
    // the corner). Its points are taken off, and its ends and its reach to ceiling and floor looked at again.
    for i in 0..faces.len() {
        let f = &faces[i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        let kept: Vec<[f64; 3]> = f.pts.iter().copied().filter(|p| !crosses_other([p[0] * d[0] + p[1] * n[0], p[0] * d[1] + p[1] * n[1]], f.theta, &faces, i)).collect();
        if kept.len() < 20 || kept.len() == f.pts.len() {
            continue;
        }
        let mut s: Vec<f64> = kept.iter().map(|p| p[0]).collect();
        s.sort_by(f64::total_cmp);
        // From where points run on, not from a stray one a little way off the end (a column of 3 in needs three).
        let runs = runs_of(&s, sq);
        let (Some(first), Some(last)) = (runs.first(), runs.last()) else { continue };
        let (first_s, last_s) = (s.iter().copied().find(|x| *x >= first.0).unwrap_or(s[0]), s.iter().rev().copied().find(|x| *x <= last.1).unwrap_or(s[s.len() - 1]));
        let s0 = edge(&s, first_s + EDGE_BACK, true, last_s - first_s - EDGE_BACK, f64::MIN).unwrap_or(first_s);
        let s1 = edge(&s, last_s - EDGE_BACK, false, last_s - first_s - EDGE_BACK, f64::MAX).unwrap_or(last_s);
        if s1 - s0 < MIN_FACE {
            continue;
        }
        let f = &mut faces[i];
        f.s0 = s0;
        f.s1 = s1;
        classify(f, &ceil, sq);
    }
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

    // ---- a face that reaches the ceiling and is not shown to be a wall (call 179, A3) ----
    let doubts = tall_at_a_wall(&faces, &wall_ids);

    // ---- what a wall is hidden by: the ceiling or the floor not reached where something stands before it ----
    // (call 179, A2) Where something stands: the cells of the plan the points of what stops short, and of
    // what is in doubt, are in. A wall that something tall stands before is not "stood short", nor is one that
    // something low stands before: where it is not seen at the floor or the ceiling and something stands within
    // 3 ft before it, on the side the scan saw it from, that is why, and the column counts as reached.
    let stand_of = |ids: &[usize], faces: &[Face]| -> Vec<P2> {
        let mut stands: HashSet<(i64, i64)> = HashSet::new();
        let mut stand_at: Vec<P2> = Vec::new();
        for i in ids {
            let f = &faces[*i];
            let (d, n) = (dir(f.theta), normal(f.theta));
            for p in &f.pts {
                let q = [p[0] * d[0] + p[1] * n[0], p[0] * d[1] + p[1] * n[1]];
                if stands.insert(((q[0] / 0.25).floor() as i64, (q[1] / 0.25).floor() as i64)) {
                    stand_at.push(q);
                }
            }
        }
        stand_at
    };
    let stoppers: Vec<usize> = (0..faces.len()).filter(|i| !faces[*i].wall || doubts.contains_key(i)).collect();
    let stand_at = stand_of(&stoppers, &faces);
    let mut wall_ids = wall_ids;
    let mut demoted: Vec<(usize, bool)> = Vec::new();
    for i in wall_ids.clone() {
        if doubts.contains_key(&i) {
            continue;
        }
        let f = &faces[i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        // Which sides something may stand on to hide it: the side the scan saw it from, or either.
        let sides: Vec<f64> = if f.seen[0] != f.seen[1] { vec![if f.seen[0] { 1.0 } else { -1.0 }] } else { vec![1.0, -1.0] };
        let hidden_column = |c: usize| {
            let (a, b) = (f.cols_from + c as f64 * f.sq, f.cols_from + (c + 1) as f64 * f.sq);
            stand_at.iter().any(|q| {
                let (s, across) = (dot(*q, d), dot(*q, n) - f.rho);
                s >= a && s <= b && sides.iter().any(|side| across * side > IN_FRONT_LEAST && across * side <= IN_FRONT)
            })
        };
        let cols = f.floor_cols.len().max(1) as f64;
        let hidden: Vec<bool> = (0..f.floor_cols.len()).map(hidden_column).collect();
        let up = f.ceil_cols.iter().zip(&hidden).filter(|(there, h)| **there || **h).count() as f64;
        let down = f.floor_cols.iter().zip(&hidden).filter(|(there, h)| **there || **h).count() as f64;
        if up / cols < WALL_REACHES_CEILING {
            demoted.push((i, false));
        } else if down / cols < WALL_REACHES_FLOOR {
            out.left_out.push(format!("a face {} long at {} east, {} north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung from the ceiling, or a wall seen only above)", plumbline_units(f.s1 - f.s0), plumbline_units(f.at(f.mid())[0]), plumbline_units(f.at(f.mid())[1])));
            demoted.push((i, true));
        }
    }
    for (i, hung) in demoted {
        faces[i].wall = false;
        faces[i].hung = hung;
        wall_ids.retain(|j| *j != i);
    }
    let standing: Vec<usize> = (0..faces.len()).filter(|i| !faces[*i].wall && !faces[*i].hung).collect();

    // ---- one wall seen in two parts, with something standing along the gap between them ----
    // (call 179, A2) A wall's face that stops where a cabinet bank begins and starts again where it
    // ends is one wall, not two with the room open between: not seen is not open. The gap is said
    // where the openings are, as not seen.
    {
        let mut again = true;
        while again {
            again = false;
            'pairs: for &a in &wall_ids {
                for &b in &wall_ids {
                    // `a` is the longer of the two: its line is the one the other is to lie on (a stub of
                    // 2 ft is told to a degree or two at best, and 10 ft of wall to a tenth).
                    if a == b || doubts.contains_key(&a) || doubts.contains_key(&b) || faces[a].s1 - faces[a].s0 < faces[b].s1 - faces[b].s0 {
                        continue;
                    }
                    let (fa, fb) = (&faces[a], &faces[b]);
                    let (d, n) = (dir(fa.theta), normal(fa.theta));
                    let ends = [fb.at(fb.s0), fb.at(fb.s1)];
                    if ends.iter().any(|e| (dot(*e, n) - fa.rho).abs() > BAND + 0.05) {
                        continue;
                    }
                    let (o0, o1) = (dot(ends[0], d).min(dot(ends[1], d)), dot(ends[0], d).max(dot(ends[1], d)));
                    let gap = if o0 >= fa.s1 - 0.5 { (fa.s1, o0) } else if o1 <= fa.s0 + 0.5 { (o1, fa.s0) } else { continue };
                    // Something stands before the whole gap, on the side the scan saw the wall from.
                    let sides: Vec<f64> = if fa.seen[0] != fa.seen[1] { vec![if fa.seen[0] { 1.0 } else { -1.0 }] } else { vec![1.0, -1.0] };
                    let steps = (((gap.1 - gap.0) / sq).ceil() as usize).max(1);
                    let covered = (0..steps).filter(|k| {
                        let (lo, hi) = (gap.0 + *k as f64 * sq, gap.0 + (*k + 1) as f64 * sq);
                        stand_at.iter().any(|q| {
                            let (sq, ac) = (dot(*q, d), dot(*q, n) - fa.rho);
                            sq >= lo && sq <= hi && sides.iter().any(|side| ac * side > IN_FRONT_LEAST && ac * side <= IN_FRONT)
                        })
                    }).count();
                    if covered as f64 >= STANDS_ALONG * steps as f64 {
                        let (db, nb) = (dir(fb.theta), normal(fb.theta));
                        let moved: Vec<[f64; 3]> = fb.pts.iter().map(|p| { let q = [p[0] * db[0] + p[1] * nb[0], p[0] * db[1] + p[1] * nb[1]]; [dot(q, d), dot(q, n), p[2]] }).collect();
                        let seen = [fa.seen[0] || fb.seen[0], fa.seen[1] || fb.seen[1]];
                        let (s0, s1) = (fa.s0.min(o0), fa.s1.max(o1));
                        let fa = &mut faces[a];
                        fa.pts.extend(moved);
                        fa.s0 = s0;
                        fa.s1 = s1;
                        fa.seen = seen;
                        wall_ids.retain(|j| *j != b);
                        again = true;
                        break 'pairs;
                    }
                }
            }
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
    // ---- pair faces into walls ----
    let mut taken: HashSet<usize> = HashSet::new();
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    for (x, a) in wall_ids.iter().enumerate() {
        for b in &wall_ids[x + 1..] {
            if doubts.contains_key(a) || doubts.contains_key(b) {
                continue;
            }
            let (fa, fb) = (&faces[*a], &faces[*b]);
            let mut dt = (fa.theta - fb.theta).abs();
            dt = dt.min(std::f64::consts::PI - dt);
            if dt.to_degrees() > PAIR_DEGREES {
                continue;
            }
            // Which side of one the other's middle is on, and how far.
            let across = dot(fb.at(fb.mid()), normal(fa.theta)) - fa.rho;
            if across.abs() < PAIR_LEAST || across.abs() > PAIR_MOST_BARE {
                continue;
            }
            // Side by side along the first's own measure of the way along.
            let shift = dot(fb.at(fb.mid()), dir(fa.theta)) - fb.mid();
            let overlap = fa.s1.min(fb.s1 + shift) - fa.s0.max(fb.s0 + shift);
            if overlap < PAIR_ALONGSIDE {
                continue;
            }
            // Past 16 in, two faces are one wall (a thick wall of masonry, a chase) only where no floor is seen
            // between them, and where they end together: a cabinet bank 2 ft deep before a wall, or two walls
            // with a closet between, are not (call 179, A6).
            if across.abs() > PAIR_MOST {
                let (lo, hi) = (fa.s0.max(fb.s0 + shift), fa.s1.min(fb.s1 + shift));
                let (na, da) = (normal(fa.theta), dir(fa.theta));
                let (ra, rb) = (fa.rho.min(fa.rho + across), fa.rho.max(fa.rho + across));
                let width = (rb - ra - 2.0 * FLOOR_MARGIN).max(0.0);
                let seen = scan.points.iter().filter(|p| {
                    let q = [p[0], p[1]];
                    (p[2] - floor).abs() <= SET_ASIDE && dot(q, na) > ra + FLOOR_MARGIN && dot(q, na) < rb - FLOOR_MARGIN && dot(q, da) > lo + FLOOR_MARGIN && dot(q, da) < hi - FLOOR_MARGIN
                }).count() as f64;
                let would = rho.unwrap_or(REFERENCE_DENSITY) * width * (hi - lo - 2.0 * FLOOR_MARGIN).max(0.0);
                let ends_agree = (fa.s0 - (fb.s0 + shift)).abs() <= CORNER && (fa.s1 - (fb.s1 + shift)).abs() <= CORNER;
                if seen > FLOOR_BETWEEN * would || !ends_agree {
                    continue;
                }
            }
            pairs.push((*a, *b, overlap));
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
        // Where the two faces end differently by no more than a wall is thick, they meet a wall across them: the
        // wall ends at its near face, the shorter of the two, and not at its far one (call 179, A9). Where they
        // end far apart one of them is hidden, and the wall is as long as both show.
        let s0 = { let (a0, b0) = (fa.s0, fb.s0 + shift); if (a0 - b0).abs() <= PAIR_MOST { a0.max(b0) } else { a0.min(b0) } };
        let s1 = { let (a1, b1) = (fa.s1, fb.s1 + shift); if (a1 - b1).abs() <= PAIR_MOST { a1.min(b1) } else { a1.max(b1) } };
        let mut pts: Vec<[f64; 2]> = fa.pts.iter().map(|p| [p[0], p[2]]).collect();
        pts.extend(fb.pts.iter().map(|p| [p[0] + shift, p[2]]));
        // Seen from the side each face faces away from the other: both, so none of the one-sided.
        let (mut w, flip) = wall_of(theta, centre, s0, s1, Some((rb - fa.rho).abs()), None);
        w.height = Some(fa.ceil.min(fb.ceil));
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
        let (mut w, flip) = wall_of(f.theta, f.rho, f.s0, f.s1, None, seen_from);
        w.doubt = doubts.get(i).cloned();
        w.height = Some(f.ceil);
        walls.push((w, WallBuild { pts, theta: f.theta, rho: f.rho, s0: f.s0, flip }));
    }

    // ---- the openings, and what is not seen ----
    for (w, b) in &mut walls {
        let (d, n) = (dir(b.theta), normal(b.theta));
        let length = w.length();
        let tops = |s: f64| ceil.over([s * d[0] + b.rho * n[0], s * d[1] + b.rho * n[1]], n);
        let g = Grid::new(&b.pts.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>(), b.s0, b.s0 + length, high, sq, &tops);
        // Empty squares, joined; the wide ones are holes. A square empty by the scatter's chance (a
        // lone one, or a thin string of them through the solid) is not part of a hole: only squares
        // with at least three empty neighbours are joined, so that noise does not carry a hole out
        // to the wall's end (call 179, A2).
        let empty = |c: usize, r: usize| !g.full(c, r);
        let core = |c: usize, r: usize| {
            empty(c, r) && [(c.wrapping_sub(1), r), (c + 1, r), (c, r.wrapping_sub(1)), (c, r + 1)].iter().filter(|(nc, nr)| *nc < g.cols && *nr < g.rows && empty(*nc, *nr)).count() >= 3
        };
        let mut seen = vec![false; g.cols * g.rows];
        let mut holes: Vec<(usize, usize, usize, usize, usize, bool)> = Vec::new();
        for r in 0..g.rows {
            for c in 0..g.cols {
                if seen[r * g.cols + c] || !core(c, r) {
                    continue;
                }
                let (mut stack, mut cells) = (vec![(c, r)], Vec::new());
                seen[r * g.cols + c] = true;
                while let Some((cc, rr)) = stack.pop() {
                    cells.push((cc, rr));
                    let near = [(cc.wrapping_sub(1), rr), (cc + 1, rr), (cc, rr.wrapping_sub(1)), (cc, rr + 1)];
                    for (nc, nr) in near {
                        if nc < g.cols && nr < g.rows && !seen[nr * g.cols + nc] && core(nc, nr) {
                            seen[nr * g.cols + nc] = true;
                            stack.push((nc, nr));
                        }
                    }
                }
                let (c0, c1) = (cells.iter().map(|x| x.0).min().unwrap_or(0), cells.iter().map(|x| x.0).max().unwrap_or(0));
                let (r0, r1) = (cells.iter().map(|x| x.1).min().unwrap_or(0), cells.iter().map(|x| x.1).max().unwrap_or(0));
                let box_cells = (c1 - c0 + 1) * (r1 - r0 + 1);
                // A hole is a clean block of empty squares, wide enough, not at the wall's ends.
                // One that runs to an end of the wall is no opening (a wall does not end in a doorway)
                // but may be what stands before the wall's end hiding it (call 179, A2): kept to be said so.
                if (c1 - c0 + 1) as f64 * sq >= MIN_OPENING - sq && cells.len() as f64 >= 0.8 * box_cells as f64 {
                    holes.push((c0, c1, r0, r1, cells.len(), c0 == 0 || c1 + 1 == g.cols));
                }
            }
        }
        holes.sort();
        for (c0, c1, r0, r1, _, at_end) in holes {
            let (ls, rs) = (g.col_range(c0), g.col_range(c1 + 1));
            let (zb, zt) = (SET_ASIDE + r0 as f64 * sq, SET_ASIDE + (r1 + 1) as f64 * sq);
            if at_end {
                let (el, er) = (ls.max(b.s0), rs.min(b.s0 + length));
                let blocked = |side: f64| -> bool {
                    stand_at.iter().any(|q| {
                        let (s, a) = (dot(*q, d), (dot(*q, n) - b.rho) * side);
                        let half = w.thickness.map_or(0.0, |t| t / 2.0);
                        s >= el && s <= er && a > half + IN_FRONT_LEAST && a <= half + IN_FRONT
                    })
                };
                let hidden = if w.thickness.is_some() { blocked(1.0) && blocked(-1.0) } else { blocked(w.seen_from.map_or(1.0, |sf| if dot(sf, n) >= 0.0 { 1.0 } else { -1.0 })) };
                if hidden {
                    let (from, to) = (el - b.s0, er - b.s0);
                    w.hidden.push(if b.flip { Stretch { from: length - to, to: length - from } } else { Stretch { from, to } });
                }
                continue;
            }
            if r1 + 1 >= g.rows {
                // Nothing at all from floor to ceiling over this stretch: where something stands before it
                // (a bank of cabinets to the ceiling, with the wall behind it unseen) it is not seen, and is
                // taken as wall and said so (call 179, A2); where nothing does, it is said as it was.
                let (el, er) = (ls.max(b.s0), rs.min(b.s0 + length));
                let blocked = |side: f64| -> bool {
                    stand_at.iter().any(|q| {
                        let (sq, a) = (dot(*q, d), (dot(*q, n) - b.rho) * side);
                        let half = w.thickness.map_or(0.0, |t| t / 2.0);
                        sq >= el && sq <= er && a > half + IN_FRONT_LEAST && a <= half + IN_FRONT
                    })
                };
                let hidden = if w.thickness.is_some() { blocked(1.0) && blocked(-1.0) } else { blocked(w.seen_from.map_or(1.0, |sf| if dot(sf, n) >= 0.0 { 1.0 } else { -1.0 })) };
                if hidden {
                    let (from, to) = (el - b.s0, er - b.s0);
                    w.hidden.push(if b.flip { Stretch { from: length - to, to: length - from } } else { Stretch { from, to } });
                } else {
                    out.left_out.push(format!("a gap {} wide in a wall, {} from its {} end, runs from the floor to the ceiling: it is not taken as an opening, and the wall is laid across it", plumbline_units(rs - ls), plumbline_units(if b.flip { length - (rs - b.s0) } else { ls - b.s0 }), end_word(w)));
                }
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
            let head = edge(&in_up, zt + EDGE_BACK, true, tops((el + er) / 2.0) - SET_ASIDE - zt - EDGE_BACK, zb).unwrap_or(zt);
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
    out.open_ends = open_ends(&out.walls);
    out
}

/// Which end of a wall is its first, in words (west or south).
fn end_word(w: &Wall) -> &'static str {
    let (dx, dy) = (w.to[0] - w.from[0], w.to[1] - w.from[1]);
    if dx.abs() >= dy.abs() { "west" } else { "south" }
}

/// The distance from a place to a wall's line, as a stretch from end to end.
fn distance_to(p: P2, a: P2, b: P2) -> f64 {
    let (e, l) = (sub(b, a), (b[0] - a[0]).hypot(b[1] - a[1]));
    let t = if l < 1e-9 { 0.0 } else { (dot(sub(p, a), e) / (l * l)).clamp(0.0, 1.0) };
    (p[0] - (a[0] + e[0] * t)).hypot(p[1] - (a[1] + e[1] * t))
}

/// Where the walls found do not close (call 179, A2): each end of a wall that comes within a foot
/// (CORNER) of no other wall's line. What is missing there is said, not guessed.
fn open_ends(walls: &[Wall]) -> Vec<(usize, P2)> {
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
    (Wall { from, to, thickness, seen_from, openings: Vec::new(), hidden: Vec::new(), ..Wall::default() }, flip)
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
fn one_type_of_openings(laid: &mut [Laid]) {
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
        let wall = |from: P2, to: P2| Wall { from, to, thickness: Some(0.5), seen_from: None, openings: Vec::new(), hidden: Vec::new(), ..Wall::default() };
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
        let seen = Wall { from: [0.0, 0.0], to: [0.0, 12.0], thickness: None, seen_from: Some([1.0, 0.0]), openings: Vec::new(), hidden: Vec::new(), ..Wall::default() };
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
