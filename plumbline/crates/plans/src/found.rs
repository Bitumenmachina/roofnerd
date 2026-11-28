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

mod ceiling;
mod laying;
mod pieces;
mod scan_levels;
mod search;

pub(crate) use ceiling::*;
pub use laying::*;
pub(crate) use pieces::*;
pub use scan_levels::*;
use search::*;

pub(crate) type P2 = [f64; 2];

// ---- the measures: each a choice where the call (173) gave none, named and said why ----

/// The cell of plan an upright face is looked for in: 0.1 ft (1 1/4 in), finer than a wall
/// is thick and coarser than the scatter of a phone's points (1.5 cm).
pub(crate) const CELL: f64 = 0.1;
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
/// The strongest lines the roughness of a scan's walls is measured on, and how far from each (feet) its points are taken (call 188, 1).
/// What is left out and is this long or longer is offered as "likely a wall" (call 188, 1), and how much of the ceiling's height a face that stops short stands for it to be.
pub(crate) const LIKELY_LEAST: f64 = 6.0;
/// Faces within this far (16 in, the thickest wall found as one, and not the 2 ft a bank of cabinets is deep) across, alongside one another for this share of the shorter's length, and seen from the same side, are one face scanned in passes (call 188, 2).
const PASS_APART: f64 = PAIR_MOST;
const PASS_ALONGSIDE: f64 = 0.5;
const PILOT_LINES: usize = 6;
const PILOT_WINDOW: f64 = 0.3;
/// How far from the floor its points are taken to tell how rough it is (feet).
const FLOOR_REACH: f64 = 0.25;
/// How far from its line a point is still on a face, each side: 3 sigma of 1.5 cm of
/// scatter. Two faces nearer than this to each other cannot be told apart (the call's
/// least wall is 2 in = 0.167 ft; a face of 1.5 cm scatter keeps within 0.15).
pub(crate) const BAND: f64 = 0.15;
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
pub(crate) const SQUARE: f64 = 0.25;
/// A column needs this many points to be there at all (a stray or two are not a wall).
const COLUMN_POINTS: usize = 3;
/// An upright stretch at least this long that is not a face (too short, or not straight) is counted and said (call 179, A5).
const SHORT_FACE: f64 = 1.0;
/// A face is at least this long (call 173), and a gap this long or more in the points
/// along a line ends the face (the least opening, so that a doorway is inside a wall and
/// not between two).
pub(crate) const MIN_FACE: f64 = 2.0;
pub(crate) const GAP: f64 = 1.5;
/// The floor's and the ceiling's own scatter, set aside: no wall's point is taken from
/// the lowest or the highest 3 in.
pub(crate) const SET_ASIDE: f64 = 0.25;
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
pub(crate) const SQUARE_DEGREES: f64 = 1.0;
/// Parallel, for two faces of one wall: within 1 degree; apart 2 in to 16 in; side by
/// side for at least this much of their length.
pub(crate) const PAIR_DEGREES: f64 = 1.0;
pub(crate) const PAIR_LEAST: f64 = 2.0 / 12.0;
pub(crate) const PAIR_MOST: f64 = 16.0 / 12.0;
/// ... and up to this far apart where no floor is seen between them (call 179, A6): of the floor a gap would
/// show, less than this share is seen, not counting 3 in each side of it.
pub(crate) const PAIR_MOST_BARE: f64 = 24.0 / 12.0;
pub(crate) const FLOOR_BETWEEN: f64 = 0.1;
const FLOOR_MARGIN: f64 = 0.25;
const PAIR_ALONGSIDE: f64 = MIN_FACE;
/// An opening is at least this long (call 173), less what the edges are measured to (3 in
/// of 1/2 inch = this slack): a 1 ft 6 in doorway is not lost for being measured 1 ft 5 3/4 in.
pub(crate) const MIN_OPENING: f64 = 1.5;
pub(crate) const MIN_OPENING_SLACK: f64 = 0.04;
/// Something that stands within this of a stretch of wall, on the side the scan saw it
/// from, hides it (call 173).
pub(crate) const IN_FRONT: f64 = 3.0;
/// Something stands in front of a stretch only if it is this far or more from the wall's
/// own scatter.
pub(crate) const IN_FRONT_LEAST: f64 = 0.2;
/// A found face's end is carried to the line of another wall's face it comes up to, when
/// it is this near: the faces end at the faces they meet, not at the centre of the scatter.
pub(crate) const SNAP: f64 = 0.5;
/// Openings within this of one another in width and in head are one type (call 179, A9): an inch.
pub(crate) const ONE_TYPE: f64 = 1.0 / 12.0;
/// A found wall's end is carried to another's line where it comes within this of it (call 173).
pub(crate) const CORNER: f64 = 1.0;
/// Two walls this nearly parallel (sine of the angle between) do not meet at a corner.
const NOT_PARALLEL: f64 = 0.5;
/// How far from its end the solid of an edge is looked at to count its density, and how
/// far the edge is expected to lie from the first empty square: see `edge`.
pub(crate) const EDGE_BACK: f64 = 0.4;
const EDGE_LOOK: f64 = 0.5;
/// The count puts an edge no further than the farthest point counted, unless the solid there is no thicker than where the density was counted: where
/// what is counted is more than this many times as thick (and this much over, in feet), the count says only that the solid changed (see `edge`).
const EDGE_THICKER: f64 = 1.5;
const EDGE_OVERSHOOT: f64 = 0.1;
/// The foot of a wall nearest its end is not counted for the density of an edge: the
/// points of a wall that crosses it were taken off there.
pub(crate) const NEAR_END: f64 = 0.3;
/// Parts of the scan joined into one thing standing short: ends within this (call 173's
/// corners, 1 ft).
const THING_JOIN: f64 = 1.0;
/// A floor cell is 1 ft square; a side of a wall is where the scan saw it from if this
/// many floor cells are on it within 6 ft.
pub(crate) const FLOOR_CELL: f64 = 1.0;
const SEEN_FLOOR_CELLS: usize = 5;
/// ... and if it shows at least a third as much floor as the other side does (the foot of
/// floor nearest a wall is not counted, for the wall's own base scatters into it).
const SEEN_SHARE: usize = 3;
const SEEN_REACH: f64 = 6.0;
/// A cell of the scan's floor is floor, and not the foot of a wall, where it holds this share of the points the middling cell holds
/// (a wall's foot, 3 in high, holds a quarter of what a foot of floor does or less).
const FLOOR_LIKE: f64 = 0.3;
/// A wall face stands behind a face, for the face to be "a wall or something tall standing at the
/// wall", when it lies this far behind it or nearer, on the side the scan did not see it from (call 179, A3).
pub(crate) const BEHIND: f64 = 3.0;
/// How far from a crossing wall's line a column of a face is inside that wall's body: half the thickest wall (a foot) is generous; a quarter of a
/// foot would miss the band the finder takes off at a corner (BAND + 0.05, 0.2 ft); this is a little over that (call 183, 4).
/// A stretch left out lies at a wall's open end when it comes within this of it (two corners).
pub const LEFT_NEAR: f64 = 2.0 * CORNER;
const CORNER_BODY: f64 = 0.35;
/// A curved face (call 183, 5): the pieces of upright points are looked at in cells of 0.3 ft; a piece of at least 5 ft that lies 0.2 ft or more off its
/// straight line, whose circle fit is at least three times as close (and CURVE_FIT is that share), of a radius from 2 ft 6 in to 80 ft over an arc of 30 to 270 degrees.
const CURVE_CELL: f64 = 0.3;
/// Points left over before a piece is looked at, and how long a wall may be to be taken of a curve (a chord of 4 ft; a bay's walls are as long, but leave nothing over).
const CURVE_LEFTOVER: usize = 60;
const CURVE_CHORD: f64 = 8.0;
const CURVE_LEAST: f64 = 5.0;
const CURVE_OFF: f64 = 0.2;
const CURVE_FIT: f64 = 0.34;
/// A face lies on a curve where its ends and its middle are this near its circle (the circle is fitted through both faces of a wall 4 1/2 in thick and through
/// the chords, which lie a quarter foot in at their middles; a foot's width in all is generous) and within the arc it runs over, and 10 degrees more.
const CURVE_ON: f64 = 0.8;
const CURVE_ARC_SLACK: f64 = 0.17;
const CURVE_RADIUS: (f64, f64) = (2.5, 80.0);
const CURVE_ARC: (f64, f64) = (30.0, 270.0);
/// A hole in a face (call 183, 3): a stretch of its height this high or more (2 ft) with points above it and below it, over this share of its length, with
/// this share of the points a surface would show standing behind the hole (a surface behind shows all of them; the stray and the scatter, none).
const HOLE_HIGH: f64 = 2.0;
const HOLE_SHARE: f64 = 0.6;
const HOLE_BACKED: f64 = 0.3;
/// The tiers of height a column is looked at in.
const HOLE_TIER: f64 = 0.25;
/// A wall with an opening in it has a piece that reaches the floor at each end at least this long (a jamb: the 6 in a door's casing leaves).
const END_PIECE: f64 = 0.5;
/// Something stands before a gap in a wall, for it to be hidden and not open, along at least this
/// share of the gap (call 179, A2).
const STANDS_ALONG: f64 = 0.9;


// ---- the ceiling over the plan (call 179, A4) ----

/// The side of a square of plan the ceiling is looked for in (3 ft apart or more on a thin scan, as the
/// squares of the walls are).
pub(crate) const CEIL_CELL: f64 = 1.0;
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
/// A plane through the cells' heights is a slope where it leaves less than this share of the scatter a flat leaves (and rises by more than the flat's own scatter, 3 in).
const SLOPE_BETTER: f64 = 0.5;
/// A cell of ceiling is a surface where its points fill at least this share of the nine parts of the cell (call 183, 6).
const CEIL_FILL: f64 = 0.8;
/// How far out from a wall, each side, the ceiling over it is looked at.
pub(crate) const CEIL_OUT: f64 = 1.5;

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
    /// The ceiling was not read over it (no cell of ceiling points near, where the ceiling is not one height): its height is the house's middle one (call 183, 6).
    pub ceiling_not_read: bool,
    /// How deep it bows over what length, in feet, where the bow is an inch or more (call 188, 1); None where it is straight.
    pub bow: Option<(f64, f64)>,
    /// How many faces were joined into it: a wall scanned in passes (call 188, 2).
    pub joined: usize,
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

/// A stretch of upright points that was not taken as a wall, with where it is (call 183, 4).
#[derive(Clone, Debug)]
pub struct Left {
    pub from: P2,
    pub to: P2,
    /// Why, in a builder's words.
    pub why: String,
}

/// A curved face found (call 183, 5): said, never laid.
#[derive(Clone, Debug)]
pub struct Curve {
    pub from: P2,
    pub to: P2,
    pub centre: P2,
    pub radius: f64,
    /// Along the arc, in feet.
    pub length: f64,
    /// The middle of the arc as an angle about its centre, and how far either way it runs (radians).
    pub mid: f64,
    pub span: (f64, f64),
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
    /// The floors of the levels the scan holds, above the lowest (call 183, 8); empty where it holds one. Which of them was found, and what is said of it.
    pub levels: Vec<f64>,
    pub level_found: Option<usize>,
    pub level_note: Option<String>,
    /// The scan is thicker in one part than another: what is said of it, with where (call 183, 7).
    pub zone_note: Option<String>,
    /// Each stretch of upright points left out, with where it is, how long and why (call 183, 4); `left_out` says them in words.
    pub left: Vec<Left>,
    /// Curved faces: said, and not laid (curved walls are not found).
    pub curves: Vec<Curve>,
    /// The faces counted as standing short (furniture, cabinets), with where each is: said only where one lies at a wall's open end, for it
    /// may be a wall that was not told (call 183, 4).
    pub stood: Vec<Left>,
    /// What stood beyond a wall with no way to it (call 183, 2), taken out of what stood short: each said.
    pub beyond_things: Vec<String>,
    /// Which floor is the house, where that is a choice (call 183, 2).
    pub beyond_note: Option<String>,
    /// How rough the scan's walls are, in feet (call 188, 1), where they are rougher than the made-up scans the finder was built on (and the band a face's points lie in is wider by as much): the robust spread of a wall's points about its own plane; None where it was not wider.
    pub scatter: Option<f64>,
    /// The scan was thinned before it was read, from this many points to this many (call 188, 4).
    pub thinned: Option<(usize, usize)>,
}

impl Found {
    /// The stretches left out that lie within `reach` of a place on the plan (call 183, 4: where leaving one out leaves a wall's end open).
    pub fn left_near(&self, p: P2, reach: f64) -> Vec<&Left> {
        self.left.iter().chain(self.stood.iter()).filter(|l| distance_to(p, l.from, l.to) <= reach).collect()
    }

    pub fn tallest(&self) -> Option<f64> {
        self.things.iter().map(|t| t.height).fold(None, |a, h| Some(a.map_or(h, |a: f64| a.max(h))))
    }
}

// ---- the plan geometry of a line ----

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
