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

mod dimensions;
mod finding;
mod parts;

pub use dimensions::*;
pub use finding::*;
use parts::*;

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
/// What a share of the wall found may fall short of its limit by, in feet, and still be at it: a sheet turned does not lose a drawing to the
/// last place of its arithmetic.
const SHARE_EPS: f64 = 1e-6;
const DRAWING_GAP: f64 = 8.0;
/// A gap in both lines of a wall wider than this is an opening only where something is drawn in it (call 191 item 2): a leaf, a swing, a
/// line across. A bare gap wider than this is two walls, with the gap said.
const BARE_GAP_MOST: f64 = 8.0;
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
    /// Where much of the sheet is in doubt (call 191 item 4): the wall found outside the largest drawing is at least half of it, and
    /// the drawings are not of like size to list as they were. (wall outside it, in feet; the largest drawing's, in feet.) Nothing is
    /// laid unasked then.
    pub unsettled: Option<(f64, f64)>,
    /// When a drawing was taken (call 191 item 1): how many walls of the other drawings were left out, and how much wall in feet.
    pub other_walls: (usize, f64),
    /// Walls the person's box cuts across (call 191 item 5): each from and to, in feet on the plan. Lines that cross the box's edge are
    /// left out whole, so each of these is not found, or found short.
    pub cut_by_box: Vec<[P2; 2]>,
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

