//! A plan set on paper: the plan of a level as it stands, the demolition plan and the
//! proposed plan, each a page of line work (call 168). The drawing is data first (the walls
//! with their weights, the dimensions as they are lettered, the rooms, the notes), and the
//! PDF is that data drawn.
//!
//! The drawing passes are BIMCraft's, ported where each function says (`paper/`); this file
//! is what BIMCraft has no function for and is ours: the pages (which of a level's things
//! each shows, and how), a dimension string made along a wall by itself on the side away
//! from the building, the rooms as they stand, a dash measured on paper, the notes, and the
//! fitting of the plan to the paper at a scale. The model's own figures are what is
//! lettered: a wall's length is its `plan_length` and a room's area its `floor_area`
//! (`building::planset`), and the drawing never measures its own lines to letter a figure.

use crate::building::planset::{level_named, level_plan, levels_from_the_lowest, LevelPlan, LevelRoom};
use crate::building::rooms;
use crate::building::walls::{self, PlanWall};
use crate::job::Job;
use crate::paper::letters::{self, Letter, MIN_CAP};
use crate::paper::strings::{self, Axis, Side};
use crate::paper::{self, cut, font, pen, size, Drawing, Prim, FAINT, INK, PAPER, POCHE};
use crate::skeleton::P2 as V;
use crate::value::Status;
use plumbline_roofing::drawing::feet_written;

/// How a thing is drawn on a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    /// New work, or everything on the page of a level with no mark: the cut's pen, its walls filled.
    Heavy,
    /// What stays, on a page that shows what is done to it: light, not filled.
    Light,
    /// What comes down: dashed.
    Dashed,
}

impl Weight {
    pub fn said(self) -> &'static str {
        match self {
            Weight::Heavy => "heavy",
            Weight::Light => "light",
            Weight::Dashed => "dashed",
        }
    }
}

/// What a page shows of the level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Show {
    /// A level with no mark on anything: all of it, heavy.
    Floor,
    /// What is marked existing or to demolish, as it stands, in full weight; new work is not on it.
    Existing,
    /// What stays, light; what comes down, dashed; new work is not on it.
    Demolition,
    /// What stays, light and unfilled; new work heavy and filled; what comes down is not on it.
    Proposed,
}

impl Show {
    pub fn name(self) -> &'static str {
        match self {
            Show::Floor => "FLOOR PLAN",
            Show::Existing => "EXISTING PLAN",
            Show::Demolition => "DEMOLITION PLAN",
            Show::Proposed => "PROPOSED PLAN",
        }
    }
    /// How a wall in `place` (0 to build, 1 existing, 2 to demolish) is drawn on this page,
    /// or not at all.
    fn weight(self, place: usize) -> Option<Weight> {
        match (self, place) {
            (Show::Floor, _) => Some(Weight::Heavy),
            (Show::Existing, 1 | 2) => Some(Weight::Heavy),
            (Show::Demolition, 1) => Some(Weight::Light),
            (Show::Demolition, 2) => Some(Weight::Dashed),
            (Show::Proposed, 1) => Some(Weight::Light),
            (Show::Proposed, 0) => Some(Weight::Heavy),
            _ => None,
        }
    }
    /// How a thing set in place (a cabinet, a fixture) in `place` is drawn on this page, or
    /// not at all: on the page of what is to be built, and on a level with no mark.
    fn thing_weight(self, place: usize) -> Option<Weight> {
        match (self, place) {
            (Show::Floor | Show::Proposed, 0) => Some(Weight::Heavy),
            (Show::Floor | Show::Proposed, 1) => Some(Weight::Light),
            _ => None,
        }
    }
    /// Whether the rooms on this page are the rooms as they stand.
    fn as_it_stands(self) -> bool {
        matches!(self, Show::Existing | Show::Demolition)
    }
}

/// What a line of the key is a sample of: a wall's line in a weight, or a room line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swatch {
    Line(Weight),
    RoomLine,
}

impl Swatch {
    pub fn said(self) -> &'static str {
        match self {
            Swatch::Line(w) => w.said(),
            Swatch::RoomLine => "long dashes, thin",
        }
    }
}

/// A wall as a page draws it.
#[derive(Clone, Debug)]
pub struct DrawnWall {
    pub path: String,
    pub name: String,
    pub weight: Weight,
    /// Whether its cut is filled.
    pub filled: bool,
}

/// A door or window as a page draws it.
#[derive(Clone, Debug)]
pub struct DrawnOpening {
    pub path: String,
    pub door: bool,
    pub wall: String,
    /// The gap in the wall, feet.
    pub gap: f64,
    pub weight: Weight,
}

/// A cabinet, fixture or other thing set in place, as a page draws its outline.
#[derive(Clone, Debug)]
pub struct DrawnThing {
    pub path: String,
    pub name: String,
    pub weight: Weight,
    /// Hung wholly above the height the walls are cut at: dashed.
    pub above_cut: bool,
}

/// The overall dimensions of a page's plan, as lettered outside the strings: across (east to
/// west) and up the page (north to south), the extent of the walls' lines as drawn.
#[derive(Clone, Debug)]
pub struct DrawnOverall {
    pub across: String,
    pub up: String,
    /// Whether they carry VIF: some length the strings rest on is not Known.
    pub vif: bool,
}

/// A dimension as it is lettered, on the wall it measures.
#[derive(Clone, Debug)]
pub struct DrawnDimension {
    pub wall: String,
    pub text: String,
    /// Whether it carries VIF, and whether its length is not known at all.
    pub vif: bool,
    pub unknown: bool,
}

/// A row of the room schedule: the mark on the plan, the room's name, and its area as the plan letters it.
#[derive(Clone, Debug)]
pub struct ScheduleRow {
    pub mark: String,
    pub name: String,
    pub area: String,
}

/// A room's label as it is lettered, and when it has no area, why in the model's words.
#[derive(Clone, Debug)]
pub struct RoomLabel {
    pub name: String,
    pub text: String,
    pub why: String,
}

/// A dimension string as it is lettered: where it is, what it measures, each part (with whether
/// it carries VIF) and the whole the parts add to (call 182).
#[derive(Clone, Debug)]
pub struct DrawnString {
    pub place: String,
    pub kind: String,
    pub parts: Vec<(String, bool)>,
    pub whole: String,
}

/// An angle lettered where two walls meet off square.
#[derive(Clone, Debug)]
pub struct DrawnAngle {
    pub corner: String,
    pub text: String,
}

/// One page.
#[derive(Clone, Debug)]
pub struct PlanPage {
    pub show: Show,
    pub walls: Vec<DrawnWall>,
    pub openings: Vec<DrawnOpening>,
    pub things: Vec<DrawnThing>,
    pub dimensions: Vec<DrawnDimension>,
    pub overall: DrawnOverall,
    /// The dimension strings as drawn: outside the plan three on a side at the most, then those
    /// along a wall off square and those inside the plan (call 182).
    pub strings: Vec<DrawnString>,
    /// Sides with no overall, each with why.
    pub no_overall: Vec<(String, String)>,
    pub angles: Vec<DrawnAngle>,
    /// How many room lines the page draws.
    pub room_lines: usize,
    /// The graphic scale: how long its bar is, as written.
    pub scale_bar: String,
    pub rooms: Vec<RoomLabel>,
    /// Rooms too small for their name and area at the printed scale: each a mark on the plan
    /// and a row of the room schedule at the side of the sheet (mark, then what it says).
    pub schedule: Vec<ScheduleRow>,
    /// The sheet's number, by the National CAD Standard's form: the discipline letter, a
    /// dash, the sheet type (1, plans) and the order in the set: A-101.
    pub sheet: String,
    /// Its place in the whole set of sheets (all levels, lowest first) and how many sheets there are.
    pub position: (usize, usize),
    /// The key: each kind of line this page draws, with how it is drawn.
    pub key: Vec<(String, Swatch)>,
    /// The boxes the page frames (title box, key, schedule), x0, y0, x1, y1 in points.
    pub frames: Vec<[f32; 4]>,
    /// The job's title as the title box letters it: one line, or two (the second cut at a word with an ellipsis).
    pub title_lines: Vec<String>,
    /// What the page says under its title, if anything: what comes down.
    pub under_title: Option<String>,
    pub notes: Vec<String>,
    pub drawing: Drawing,
}

/// A plan set: its pages and what it was written at.
#[derive(Clone, Debug)]
pub struct PlanSet {
    pub title: String,
    pub level: String,
    pub paper: size::Paper,
    pub scale: (u32, u32),
    pub date: String,
    pub pages: Vec<PlanPage>,
    /// Pages not written, each with why.
    pub not_written: Vec<(Show, String)>,
}

/// What the person asked of the page.
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub scale: Option<(u32, u32)>,
    pub paper: Option<size::Paper>,
    pub date: Option<String>,
}

// ---- Sizes on paper, in points ----

const MARGIN: f32 = 36.0;
const BORDER_AT: f32 = 18.0;
/// The dimension string's distance from the wall's outer face, and its ticks.
const DIM_OFF: f32 = 18.0;
/// How far apart the strings on one side of the plan lie.
const STRING_PITCH: f32 = 16.0;
/// A room line's dash and gap, points: long dashes, thin.
const ROOM_LINE_DASH: f32 = 14.0;
const ROOM_LINE_GAP: f32 = 4.0;
/// The width of the graphic scale in the band along the top.
const SCALE_W: f32 = 190.0;
/// A dimension's figures: over 3/32 in (`letters::MIN_CAP`).
const DIM_TEXT: f32 = 7.2;
/// The overall dimension's figures.
const OVERALL_TEXT: f32 = 9.0;
const ROOM_NAME: f32 = 9.0;
const NOTE_TEXT: f32 = 6.75;
const NOTE_PITCH: f32 = 9.5;
/// The same where a line carries a mark over a capital.
const ACCENT_PITCH: f32 = 11.0;
const TITLE_TEXT: f32 = 12.0;
const BOX_W: f32 = 252.0;
const BOX_H: f32 = 90.0;
/// The band along the top: the page's name at the left, the key and the north arrow at the right.
const TOP_BAND: f32 = 46.0;
const KEY_W: f32 = 150.0;
const ARROW_W: f32 = 52.0;
/// The sheet-number cell at the right of the title box.
const NUMBER_W: f32 = 90.0;
/// The room schedule beside the notes, when a room is too small to letter.
const SCHEDULE_W: f32 = 190.0;
/// How wide the area column of the room schedule is: "1234 SF VIF" at 3/32 in.
const SCHEDULE_AREA_W: f32 = 66.0;

fn lf(x: f64) -> String {
    feet_written(x)
}

fn p(v: V, k: f64) -> (f32, f32) {
    ((v.0 * k) as f32, (v.1 * k) as f32)
}

/// A dash measured on paper: about 8 points on and half that off, fitted so a run of `len`
/// points begins and ends on a dash (a closed ring, on a whole number of periods). None
/// when the run is too short to dash.
fn dash_for(len: f32, closed: bool) -> Option<Vec<f32>> {
    if len < 12.0 {
        return None;
    }
    if closed {
        let n = (len / 12.0).round().max(1.0);
        let period = len / n;
        return Some(vec![period * 2.0 / 3.0, period / 3.0]);
    }
    let n = ((2.0 * len / 8.0 + 1.0) / 3.0).round().max(1.0);
    let on = 2.0 * len / (3.0 * n - 1.0);
    Some(vec![on, on / 2.0])
}

fn run_len(pts: &[(f32, f32)], closed: bool) -> f32 {
    let mut total: f32 = pts.windows(2).map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1)).sum();
    if closed {
        if let (Some(a), Some(b)) = (pts.first(), pts.last()) {
            total += (a.0 - b.0).hypot(a.1 - b.1);
        }
    }
    total
}

/// The wrapped lines of a note, capitals, to fit `width` points at `NOTE_TEXT`.
fn wrap(text: &str, width: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.to_uppercase().split_whitespace() {
        match lines.last_mut() {
            Some(l) if font::width(&format!("{l} {word}"), NOTE_TEXT) <= width => {
                l.push(' ');
                l.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The plan set of a level, or why there is none. The job is read, not changed.
pub fn plan_set(job: &mut Job, level: &str, opts: &Options) -> Result<PlanSet, String> {
    let storey = level_named(job, level)?;
    let plan = level_plan(job, &storey)?;
    // The sheets are numbered through the whole set: the levels from the lowest up (call 182).
    let (before, total) = {
        let (mut before, mut total) = (0usize, 0usize);
        for other in levels_from_the_lowest(job) {
            let n = if other == storey { shows_of(&plan).0.len() } else { level_plan(job, &other).map_or(0, |q| if q.walls.is_empty() { 0 } else { shows_of(&q).0.len() }) };
            if other == storey {
                before = total;
            }
            total += n;
        }
        (before, total)
    };
    if plan.walls.is_empty() {
        let mut why = format!("{} has no wall on its plan to draw", plan.level);
        for (path, because) in &plan.walls_left_off {
            why.push_str(&format!("; {path}: {because}"));
        }
        return Err(why);
    }
    let tol = crate::check::FIT_TOLERANCE_FT;

    // Which pages, and why not the others.
    let (shows, not_written) = shows_of(&plan);

    // What each page holds, as data.
    let mut pages: Vec<(PlanPage, Layer)> = Vec::new();
    for (i, show) in shows.iter().enumerate() {
        pages.push(page_data(&plan, *show, tol, before + i + 1));
    }

    // The scale and the paper: the plan with its dimensions and the notes fit the paper in
    // its margins.
    let date = match &opts.date {
        Some(d) => day(d)?,
        None => crate::job::now()[..10].to_string(),
    };
    let try_fit = |scale: (u32, u32), paper: size::Paper| -> Option<Vec<Laid>> { layout(&plan, &pages, scale, paper, &date, before, total) };
    let (scale, paper_used, laid) = match (opts.scale, opts.paper) {
        (s, Some(paper)) => {
            let scale = s.unwrap_or((1, 4));
            match try_fit(scale, paper) {
                Some(l) => (scale, paper, l),
                None => {
                    let better = size::SCALES.iter().rev().find(|s| try_fit(**s, paper).is_some());
                    return Err(match better {
                        Some(b) => format!("the plan of {} does not fit {} at {} with its dimensions and a 1/2 in margin; at {} it does", plan.level, paper.name, size::scale_said(scale), size::scale_said(*b)),
                        None => format!("the plan of {} does not fit {} at any scale this program draws at (the finest is {}); give a larger paper", plan.level, paper.name, size::scale_said((1, 8))),
                    });
                }
            }
        }
        (s, None) => {
            let scale = s.unwrap_or((1, 4));
            let found = size::PAPERS.iter().find_map(|paper| try_fit(scale, *paper).map(|l| (*paper, l)));
            match found {
                Some((paper, l)) => (scale, paper, l),
                None => {
                    let d = size::PAPERS[3];
                    let better = size::SCALES.iter().rev().find(|s| try_fit(**s, d).is_some());
                    return Err(match better {
                        Some(b) => format!("the plan of {} does not fit {} at {} with its dimensions and a 1/2 in margin; at {} it does", plan.level, d.name, size::scale_said(scale), size::scale_said(*b)),
                        None => format!("the plan of {} fits no paper this program writes ({} is the largest) at any scale it draws at", plan.level, d.name),
                    });
                }
            }
        }
    };
    let mut out = Vec::new();
    for (i, ((mut page, _), laid)) in pages.into_iter().zip(laid).enumerate() {
        page.position = (before + i + 1, total);
        page.drawing = laid.drawing;
        page.notes = laid.notes;
        page.schedule = laid.schedule;
        page.frames = laid.frames;
        page.title_lines = laid.title_lines;
        page.strings = laid.strings;
        page.angles = laid.angles;
        page.no_overall = laid.no_overall;
        page.scale_bar = laid.scale_bar;
        out.push(page);
    }
    Ok(PlanSet { title: plan.title.clone(), level: plan.level.clone(), paper: paper_used, scale, date, pages: out, not_written })
}

/// Which pages a level gets, and why not the others (call 168): one FLOOR PLAN when nothing is
/// marked; else the existing plan, and the demolition plan and the proposed plan when there is
/// something to show on them.
fn shows_of(plan: &LevelPlan) -> (Vec<Show>, Vec<(Show, String)>) {
    let any = |places: &[usize]| plan.walls.iter().any(|w| places.contains(&w.place)) || plan.openings.iter().any(|o| places.contains(&o.place));
    let mut shows: Vec<Show> = Vec::new();
    let mut not_written: Vec<(Show, String)> = Vec::new();
    if !any(&[1, 2]) {
        shows.push(Show::Floor);
    } else {
        shows.push(Show::Existing);
        if any(&[2]) {
            shows.push(Show::Demolition);
        } else {
            not_written.push((Show::Demolition, "nothing is marked to demolish".to_string()));
        }
        if any(&[0, 1]) {
            shows.push(Show::Proposed);
        } else {
            not_written.push((Show::Proposed, "nothing is left or new to show".to_string()));
        }
    }
    (shows, not_written)
}

/// A date as it is lettered on the title box: a day of the calendar, as 2026-11-22 or as it is
/// written in the States, 11/22/2026, or refused with why (call 182).
pub fn day(text: &str) -> Result<String, String> {
    let t = text.trim();
    let parts: Vec<&str> = if t.contains('-') { t.split('-').collect() } else { t.split('/').collect() };
    let nums: Vec<Option<u32>> = parts.iter().map(|p| if !p.is_empty() && p.len() <= 4 && p.bytes().all(|b| b.is_ascii_digit()) { p.parse().ok() } else { None }).collect();
    let refuse = || Err(format!("--date is the day to letter on the title box, as 2026-11-22 or 11/22/2026: {t:?} is not a day of the calendar"));
    let (y, m, d) = match (nums.as_slice(), t.contains('-')) {
        ([Some(y), Some(m), Some(d)], true) if parts[0].len() == 4 => (*y, *m, *d),
        ([Some(m), Some(d), Some(y)], false) if parts[2].len() == 4 => (*y, *m, *d),
        _ => return refuse(),
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let longest = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return refuse(),
    };
    if y < 1000 || d < 1 || d > longest {
        return refuse();
    }
    Ok(t.to_string())
}

/// What a page draws, in feet: the rings of its walls (joined among the walls on the page),
/// and the rest of what it letters.
struct Layer {
    /// For each wall on the page: its place in the level's list, its weight, its outline, and
    /// its centre line's two ends.
    walls: Vec<(usize, Weight, Vec<V>, PlanWall)>,
    /// Openings: the wall (a place in `walls` above), its span, whether a door, its weight, and
    /// whether its place is Known.
    openings: Vec<(usize, [f64; 2], bool, Weight, bool)>,
    /// Which side of each wall is outward: a unit vector.
    outward: Vec<V>,
    /// Whether each wall is on the outside of the plan (one side of it indoors).
    edge: Vec<bool>,
    /// Each wall's length as the model has it, and how sure that is; and its path.
    lengths: Vec<(Option<f64>, Status)>,
    paths: Vec<String>,
    /// A room: its name, its point, the area as lettered, and why there is none when there is none.
    rooms: Vec<(String, [f64; 2], String, String)>,
    /// Things set in place: the weight, the outline's corners, whether it hangs above the cut,
    /// its name, and how sure its place is.
    things: Vec<(Weight, Vec<V>, bool, String, Status)>,
    /// The room lines drawn on the page, in feet.
    room_lines: Vec<(V, V)>,
}

/// One page laid on the paper.
struct Laid {
    notes: Vec<String>,
    drawing: Drawing,
    schedule: Vec<ScheduleRow>,
    frames: Vec<[f32; 4]>,
    title_lines: Vec<String>,
    strings: Vec<DrawnString>,
    angles: Vec<DrawnAngle>,
    no_overall: Vec<(String, String)>,
    scale_bar: String,
}

fn page_data(plan: &LevelPlan, show: Show, tol: f64, number: usize) -> (PlanPage, Layer) {
    let on: Vec<usize> = (0..plan.walls.len()).filter(|&i| show.weight(plan.walls[i].place).is_some()).collect();
    let page_walls: Vec<PlanWall> = on.iter().map(|&i| plan.walls[i].wall.clone()).collect();
    let mut drawn = Vec::new();
    let mut places_drawn: Vec<usize> = Vec::new();
    let mut layer = Layer { walls: Vec::new(), openings: Vec::new(), outward: Vec::new(), edge: Vec::new(), lengths: Vec::new(), paths: Vec::new(), rooms: Vec::new(), things: Vec::new(), room_lines: Vec::new() };
    for (k, &i) in on.iter().enumerate() {
        let w = &plan.walls[i];
        let weight = show.weight(w.place).unwrap_or(Weight::Heavy);
        drawn.push(DrawnWall { path: w.path.clone(), name: w.name.clone(), weight, filled: weight == Weight::Heavy });
        layer.walls.push((i, weight, walls::outline(&page_walls, k, tol), w.wall.clone()));
        places_drawn.push(w.place);
        layer.lengths.push((w.length, w.length_status));
        layer.paths.push(w.path.clone());
    }
    // Which way is out, wall by wall: the side the walls of the page do not close round.
    let lines: Vec<(V, V)> = (0..page_walls.len()).map(|k| walls::centre_line(&page_walls, k, tol)).map(|[a, b]| (a, b)).collect();
    let loops: Vec<Vec<V>> = rooms::faces(&lines, tol).into_iter().filter(|f| rooms::area(f) < -tol * tol).collect();
    let indoors = |q: V| loops.iter().any(|r| rooms::contains(r, q));
    for w in &page_walls {
        let t = (w.end - w.start).unit();
        let left = t.perp();
        let mid = (w.start + w.end) * 0.5;
        let reach = (w.ext - w.int).abs() / 2.0 + 0.6;
        let (l_in, r_in) = (indoors(mid + left * reach), indoors(mid - left * reach));
        // The wall's own exterior side is its left; that is kept where neither side or both are indoors.
        layer.outward.push(if r_in && !l_in { left } else if l_in && !r_in { left * -1.0 } else { left });
        // A wall with a room on both sides is a partition; any other is on the outside of the plan.
        layer.edge.push(!(l_in && r_in));
    }
    let mut openings = Vec::new();
    for o in &plan.openings {
        let Some(k) = on.iter().position(|&i| i == o.wall) else { continue };
        let keep = match show {
            Show::Floor => true,
            Show::Existing | Show::Demolition => o.place == 1 || o.place == 2,
            Show::Proposed => o.place == 0 || o.place == 1,
        };
        if !keep {
            continue;
        }
        // An opening is drawn in the weight its own mark gives it on this sheet.
        let weight = show.weight(o.place).unwrap_or(Weight::Heavy);
        places_drawn.push(o.place);
        layer.openings.push((k, o.span, o.door, weight, o.status == Status::Known));
        openings.push(DrawnOpening { path: o.path.clone(), door: o.door, wall: plan.walls[o.wall].path.clone(), gap: o.span[1] - o.span[0], weight });
    }
    // Things set in place, drawn on the proposed plan (and a level with no mark) as outlines.
    let mut things = Vec::new();
    for t in &plan.things {
        let Some(weight) = show.thing_weight(t.place) else { continue };
        places_drawn.push(t.place);
        layer.things.push((weight, t.ring.to_vec(), t.above_cut, t.name.to_uppercase(), t.status));
        things.push(DrawnThing { path: t.path.clone(), name: t.name.clone(), weight, above_cut: t.above_cut });
    }
    // Dimensions: the model's own length for each wall.
    let mut dimensions = Vec::new();
    for &i in &on {
        let w = &plan.walls[i];
        let (text, vif, unknown) = match (w.length, w.length_status) {
            (Some(l), Status::Known) => (lf(l), false, false),
            (Some(l), _) => (format!("{} VIF", lf(l)), true, false),
            (None, _) => ("?".to_string(), false, true),
        };
        dimensions.push(DrawnDimension { wall: w.path.clone(), text, vif, unknown });
    }
    // Rooms.
    let rooms_here: &[LevelRoom] = if show.as_it_stands() { &plan.rooms_stand } else { &plan.rooms_left };
    let mut labels = Vec::new();
    for r in rooms_here {
        let area = match (r.area, r.status) {
            (Some(a), Status::Known) => format!("{} SF", a.round()),
            (Some(a), _) => format!("{} SF VIF", a.round()),
            (None, _) => "AREA NOT KNOWN".to_string(),
        };
        layer.rooms.push((r.name.to_uppercase(), r.point, area.clone(), r.why.clone()));
        labels.push(RoomLabel { name: r.name.clone(), text: format!("{} {area}", r.name.to_uppercase()), why: r.why.clone() });
    }
    // The room lines: those that part the rooms this page letters.
    for l in &plan.room_lines {
        if !show.as_it_stands() || !l.stands_for_demolished {
            layer.room_lines.push((l.a, l.b));
        }
    }
    // What comes down.
    let under_title = (show == Show::Demolition).then(|| {
        let gone: Vec<&crate::building::planset::LevelWall> = plan.walls.iter().filter(|w| w.place == 2).collect();
        let mut parts = Vec::new();
        if !gone.is_empty() {
            let any_unknown = gone.iter().any(|w| w.length.is_none());
            let sum: f64 = gone.iter().filter_map(|w| w.length).sum();
            let vif = gone.iter().any(|w| w.length.is_some() && w.length_status != Status::Known);
            let length = if any_unknown { "? LF".to_string() } else { format!("{} LF{}", crate::view::num(crate::value::round4(sum)), if vif { " VIF" } else { "" }) };
            parts.push(format!("{} COME{} DOWN, {length}", count(gone.len(), "WALL", "WALLS"), if gone.len() == 1 { "S" } else { "" }));
        }
        let doors = plan.openings.iter().filter(|o| o.place == 2 && o.door).count();
        let windows = plan.openings.iter().filter(|o| o.place == 2 && !o.door).count();
        if doors > 0 {
            parts.push(format!("{} COME{} DOWN", count(doors, "DOOR", "DOORS"), if doors == 1 { "S" } else { "" }));
        }
        if windows > 0 {
            parts.push(format!("{} COME{} DOWN", count(windows, "WINDOW", "WINDOWS"), if windows == 1 { "S" } else { "" }));
        }
        if parts.is_empty() {
            "NOTHING COMES DOWN".to_string()
        } else {
            parts.join("; ")
        }
    });
    // The overall: the extent of the walls' lines as drawn, east to west and north to south.
    let ends: Vec<V> = page_walls.iter().flat_map(|w| [w.start, w.end]).collect();
    let span = |f: fn(&V) -> f64| {
        let lo = ends.iter().map(f).fold(f64::INFINITY, f64::min);
        let hi = ends.iter().map(f).fold(f64::NEG_INFINITY, f64::max);
        if hi >= lo { hi - lo } else { 0.0 }
    };
    let overall = DrawnOverall { across: lf(span(|q| q.0)), up: lf(span(|q| q.1)), vif: dimensions.iter().any(|d| d.vif || d.unknown) };
    // The key: each kind of line this page draws, as this page draws it, and nothing else.
    let mut key: Vec<(String, Swatch)> = Vec::new();
    if show == Show::Floor {
        key.push(("NEW (NO MARK)".to_string(), Swatch::Line(Weight::Heavy)));
    } else {
        let drawn_in = |place: usize| places_drawn.contains(&place);
        for (place, name) in [(1usize, "EXISTING"), (2, "TO DEMOLISH"), (0, "NEW")] {
            if let Some(w) = show.weight(place) {
                if drawn_in(place) {
                    key.push((name.to_string(), Swatch::Line(w)));
                }
            }
        }
    }
    if !layer.room_lines.is_empty() {
        key.push(("ROOM LINE".to_string(), Swatch::RoomLine));
    }
    let room_lines = layer.room_lines.len();
    let page = PlanPage { show, walls: drawn, openings, things, dimensions, overall, strings: Vec::new(), no_overall: Vec::new(), angles: Vec::new(), room_lines, scale_bar: String::new(), rooms: labels, schedule: Vec::new(), sheet: format!("A-1{number:02}"), position: (number, number), key, frames: Vec::new(), title_lines: Vec::new(), under_title, notes: Vec::new(), drawing: Drawing::new(0.0, 0.0) };
    (page, layer)
}

/// The notes of a page, in words, before they are wrapped.
fn notes_of(plan: &LevelPlan, page: &PlanPage, art: &Art) -> Vec<String> {
    let crowded = faults_in(&art.d, &[]).len();
    let mut notes = vec!["PLAN NORTH IS UP THE PAGE; TRUE NORTH IS NOT SET.".to_string(), "DOOR SWINGS ARE NOT SET.".to_string(), "DIMENSIONS ARE TO THE WALLS' LINES AS DRAWN. OPENINGS ARE TO THEIR CENTERS.".to_string()];
    if !art.no_overall.is_empty() {
        let sides: Vec<String> = art.no_overall.iter().map(|(s, _)| s.to_uppercase()).collect();
        let off_square = art.angles.iter().len() > 0 || art.strings.iter().any(|s| s.kind == "along its wall");
        notes.push(format!("NO OVERALL ON THE {} SIDE{}: {} SO THE STRINGS DO NOT ADD TO IT.", sides.join(", "), if sides.len() == 1 { "" } else { "S" }, if off_square { "A WALL OFF SQUARE MAKES THE PLAN RUN PAST THE WALLS THERE," } else { "THE WALLS THERE DO NOT REACH ACROSS THE WHOLE PLAN," }));
    }
    if !art.angles.is_empty() || art.strings.iter().any(|s| s.kind == "along its wall") {
        notes.push("A WALL OFF SQUARE IS DIMENSIONED ALONG ITSELF; THE ANGLE BETWEEN WALLS IS LETTERED AT THE CORNER.".to_string());
    }
    if art.run_strings {
        notes.push("A STRING ALONG A WALL INSIDE THE PLAN GIVES, FROM THE WALL'S FIRST END, THE START OF A RUN OF CABINETS OR FIXTURES, ITS LENGTH, AND WHAT IS LEFT.".to_string());
    }
    if page.room_lines > 0 {
        notes.push("A ROOM LINE (LONG DASHES) PARTS ROOMS WHERE NO WALL STANDS; IT IS NO WALL.".to_string());
    }
    notes.push("THIS SHEET DOES NOT SHOW PIPES, DUCTS, WIRING OR THE FRAME.".to_string());
    if crowded > 0 {
        notes.push(format!("{} ON THIS SHEET RUN OVER EACH OTHER OR OVER A LINE AND CANNOT BE READ: THE WRITER OF THE SET HAS THEIR LIST.", count(crowded, "FIGURE OR NAME", "FIGURES OR NAMES")));
    }
    // What is set in place: drawn as an outline here, or on another sheet, or not at all.
    let all = plan.things.len();
    if !page.things.is_empty() {
        let hung = page.things.iter().filter(|t| t.above_cut).count();
        notes.push(format!("{} SET IN PLACE ARE DRAWN AS THEIR OUTLINES{}.", count(page.things.len(), "CABINET OR FIXTURE", "CABINETS AND FIXTURES"), if hung > 0 { format!("; {hung} DASHED HANG ABOVE THE CUT") } else { String::new() }));
    } else if all > 0 {
        notes.push(format!("{} SET IN PLACE ARE NOT SHOWN ON THIS SHEET.", count(all, "CABINET OR FIXTURE", "CABINETS AND FIXTURES")));
    }
    let gone = plan.things.iter().filter(|t| t.place == 2).count();
    if gone > 0 && !page.things.is_empty() {
        notes.push(format!("{} MARKED TO DEMOLISH IS NOT DRAWN.", count(gone, "THING", "THINGS")));
    }
    if !plan.things_left_off.is_empty() {
        notes.push(format!("{} NOT DRAWN: {}.", count(plan.things_left_off.len(), "THING SET IN PLACE IS", "THINGS SET IN PLACE ARE"), plan.things_left_off.iter().map(|(p, why)| format!("{p}, {why}")).collect::<Vec<_>>().join("; ")));
    }
    let vif = page.dimensions.iter().filter(|d| d.vif).count();
    let unknown = page.dimensions.iter().filter(|d| d.unknown).count();
    if vif == 0 {
        notes.push("NO LENGTH CARRIES VIF (VERIFY IN FIELD).".to_string());
    } else {
        notes.push(format!("{} TO VERIFY IN FIELD (VIF).", count(vif, "LENGTH IS", "LENGTHS ARE")));
    }
    if unknown > 0 {
        notes.push(format!("{} NO FIGURE IN THE JOB: THEIR STRINGS ARE FROM THE LINES AS DRAWN AND CARRY VIF.", count(unknown, "WALL HAS", "WALLS HAVE")));
    }
    let vif_areas = page.rooms.iter().filter(|r| r.text.ends_with(" VIF")).count();
    if vif_areas > 0 {
        notes.push(format!("{} TO VERIFY IN FIELD (VIF).", count(vif_areas, "AREA IS", "AREAS ARE")));
    }
    notes.push(format!("WALLS ARE CUT {} ABOVE THE FLOOR. {}", lf(plan.cut), if page.show.as_it_stands() { "ROOMS ARE AS THEY STAND TODAY." } else { "ROOMS ARE AS THE WORK LEAVES THEM." }));
    if page.show.as_it_stands() {
        let open: Vec<String> = plan.rooms_stand_missing.iter().filter(|(_, why)| why.is_empty()).map(|(n, _)| n.to_uppercase()).collect();
        if !open.is_empty() {
            notes.push(format!("NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS: {}.", open.join(", ")));
        }
        for (_, why) in plan.rooms_stand_missing.iter().filter(|(_, why)| !why.is_empty()) {
            notes.push(format!("NOT LETTERED: {}.", why.to_uppercase().trim_end_matches('.')));
        }
    }
    // A room with no area says why, in the model's own words.
    // (A reason that does not name the room is said once for the rooms it is the reason of.)
    let mut shared: Vec<(String, Vec<String>)> = Vec::new();
    for r in page.rooms.iter().filter(|r| !r.why.is_empty()) {
        if r.why.to_lowercase().contains(&r.name.to_lowercase()) {
            notes.push(r.why.to_uppercase());
        } else {
            match shared.iter_mut().find(|(w, _)| *w == r.why) {
                Some(g) => g.1.push(r.name.to_uppercase()),
                None => shared.push((r.why.clone(), vec![r.name.to_uppercase()])),
            }
        }
    }
    for (why, names) in shared {
        notes.push(format!("{} HAVE NO AREA: {}.", names.join(", "), why.to_uppercase().trim_end_matches('.')));
    }
    if !plan.walls_left_off.is_empty() {
        notes.push(format!("{} NOT DRAWN: {}.", count(plan.walls_left_off.len(), "WALL IS", "WALLS ARE"), plan.walls_left_off.iter().map(|(p, why)| format!("{p}, {why}")).collect::<Vec<_>>().join("; ")));
    }
    if !plan.openings_left_off.is_empty() {
        notes.push(format!("{} NOT DRAWN: {}.", count(plan.openings_left_off.len(), "DOOR OR WINDOW IS", "DOORS AND WINDOWS ARE"), plan.openings_left_off.iter().map(|(p, why)| format!("{p}, {why}")).collect::<Vec<_>>().join("; ")));
    }
    notes
}

/// Each page drawn at `scale` on `paper`, with its notes wrapped, or none if what is drawn
/// does not fit inside the margins.
fn layout(plan: &LevelPlan, pages: &[(PlanPage, Layer)], scale: (u32, u32), paper: size::Paper, date: &str, before: usize, total: usize) -> Option<Vec<Laid>> {
    let (w, h) = paper.points();
    let (w, h) = (w as f32, h as f32);
    let k = 72.0 * size::inches_to_the_foot(scale);
    let n = size::ratio(scale);
    // Every page's plan layer, in points from the level's origin, and the box that holds all of them.
    let mut arts: Vec<Art> = Vec::new();
    let mut bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (page, layer) in pages {
        let art = plan_prims(page, layer, k as f64, n);
        let (x0, y0, x1, y1) = extent(&art.d.prims);
        bounds = (bounds.0.min(x0), bounds.1.min(y0), bounds.2.max(x1), bounds.3.max(y1));
        arts.push(art);
    }
    // The notes, wrapped to the room beside the title box (and the room schedule, where a
    // page has one); the strip along the bottom.
    let mut all_notes: Vec<Vec<String>> = Vec::new();
    let mut strip = BOX_H;
    let mut pitch = NOTE_PITCH;
    for ((page, _), art) in pages.iter().zip(&arts) {
        let beside = if art.schedule.is_empty() { 0.0 } else { SCHEDULE_W + 10.0 };
        let notes_w = w - 2.0 * MARGIN - BOX_W - 14.0 - beside;
        let mut notes = notes_of(plan, page, art);
        if art.unnamed > 0 {
            notes.push(format!("{} DRAWN WITH NO NAME IN IT: TOO SMALL, OR HUNG OVER THE FLOOR.", count(art.unnamed, "THING IS", "THINGS ARE")));
        }
        let lines: Vec<String> = notes.iter().flat_map(|n| wrap(n, notes_w)).collect();
        // A mark over a capital (an accented letter) needs more room between lines.
        if lines.iter().any(|l| font::reach(l).0 > 0.0) || art.schedule.iter().any(|r| font::reach(&r.name).0 > 0.0) {
            pitch = pitch.max(ACCENT_PITCH);
        }
        all_notes.push(lines);
    }
    for ((_, _), (art, lines)) in pages.iter().zip(arts.iter().zip(&all_notes)) {
        strip = strip.max(lines.len() as f32 * pitch + 6.0);
        if !art.schedule.is_empty() {
            strip = strip.max((art.schedule.iter().map(|r| schedule_name_lines(r).len()).sum::<usize>() as f32 + 1.0) * pitch + 12.0);
        }
    }
    let (x0, y0) = (MARGIN, MARGIN + strip + 12.0);
    let (x1, y1) = (w - MARGIN, h - MARGIN - TOP_BAND);
    if bounds.0 > bounds.2 {
        return None;
    }
    if bounds.2 - bounds.0 > x1 - x0 || bounds.3 - bounds.1 > y1 - y0 {
        return None;
    }
    // Centred in what is left, with room under it for its graphic scale.
    let (dx, dy) = ((x0 + x1) / 2.0 - (bounds.0 + bounds.2) / 2.0, (y0 + y1) / 2.0 - (bounds.1 + bounds.3) / 2.0);
    let mut out = Vec::new();
    for (i, ((page, _), art)) in pages.iter().zip(arts).enumerate() {
        let mut d = Drawing::new(w, h);
        let mut frames: Vec<[f32; 4]> = Vec::new();
        // The border, then the plan, then the page's name, its key and north arrow, its title box and notes.
        d.line(vec![(BORDER_AT, BORDER_AT), (w - BORDER_AT, BORDER_AT), (w - BORDER_AT, h - BORDER_AT), (BORDER_AT, h - BORDER_AT)], pen::points(pen::annotation_mm(pen::BORDER)), INK, None, true);
        for pr in art.d.prims {
            d.prims.push(shifted(pr, dx, dy));
        }
        for l in art.d.letters {
            d.letters.push(Letter { centre: (l.centre.0 + dx, l.centre.1 + dy), ..l });
        }
        d.unlettered.extend(art.d.unlettered.iter().copied());
        // The graphic scale in the band along the top, beside the page's name.
        let scale_bar = graphic_scale(&mut d, scale, k as f32, (w - MARGIN - ARROW_W - 8.0 - KEY_W - 8.0 - SCALE_W, h - MARGIN - 6.0), SCALE_W);
        // The top band: the name and what comes down at the left, the key and the arrow at the right.
        let arrow_x = w - MARGIN - ARROW_W;
        let key_x = arrow_x - 8.0 - KEY_W;
        // The graphic scale takes the right of the band's left part; the name and what comes down keep to the rest.
        let left_w = key_x - MARGIN - 10.0 - SCALE_W - 10.0;
        let name = letters::cut_to(page.show.name(), TITLE_TEXT, left_w);
        d.text_left(&name, (MARGIN, h - MARGIN - TITLE_TEXT), TITLE_TEXT, INK);
        if let Some(under) = &page.under_title {
            for (j, line) in letters::lines_of(under, DIM_TEXT, left_w, 2).iter().enumerate() {
                d.text_left(line, (MARGIN, h - MARGIN - TITLE_TEXT - 4.0 - DIM_TEXT - 6.0 - j as f32 * 10.0), DIM_TEXT, INK);
            }
        }
        let key_h = 6.0 + 12.0 * page.key.len() as f32;
        key_box(&mut d, &mut frames, page, n, (key_x, h - MARGIN - 4.0 - key_h), (KEY_W, key_h));
        north_arrow(&mut d, (arrow_x, h - MARGIN - TOP_BAND + 4.0), TOP_BAND - 4.0);
        let title_lines = title_box(&mut d, &mut frames, plan, page, scale, date, before + i + 1, total, (w - MARGIN - BOX_W, MARGIN));
        for (j, line) in all_notes[i].iter().enumerate() {
            d.text_left(line, (MARGIN, MARGIN + strip - 8.0 - j as f32 * pitch), NOTE_TEXT, INK);
        }
        if !art.schedule.is_empty() {
            schedule_box(&mut d, &mut frames, &art.schedule, (w - MARGIN - BOX_W - 10.0 - SCHEDULE_W, MARGIN), (SCHEDULE_W, strip), pitch);
        }
        out.push(Laid { notes: all_notes[i].clone(), drawing: d, schedule: art.schedule, frames, title_lines, strings: art.strings, angles: art.angles, no_overall: art.no_overall, scale_bar });
    }
    Some(out)
}

/// How many lines a schedule row's name takes: one, or two when it does not go on one.
fn schedule_name_lines(row: &ScheduleRow) -> Vec<String> {
    letters::lines_of(&row.name, NOTE_TEXT, SCHEDULE_W - 10.0 - 22.0 - SCHEDULE_AREA_W, 2)
}

/// The room schedule: the rooms too small to letter on the plan, a row each (mark, name on one
/// line or two, area), under the heading.
fn schedule_box(d: &mut Drawing, frames: &mut Vec<[f32; 4]>, rows: &[ScheduleRow], (x0, y0): (f32, f32), (bw, bh): (f32, f32), pitch: f32) {
    let wide = pen::points(pen::annotation_mm(pen::BORDER));
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    d.line(vec![(x0, y0), (x0 + bw, y0), (x0 + bw, y0 + bh), (x0, y0 + bh)], wide, INK, None, true);
    frames.push([x0, y0, x0 + bw, y0 + bh]);
    d.text_left("ROOM SCHEDULE", (x0 + 5.0, y0 + bh - 11.0), NOTE_TEXT, INK);
    d.line(vec![(x0, y0 + bh - 15.0), (x0 + bw, y0 + bh - 15.0)], thin, INK, None, false);
    let mut y = y0 + bh - 15.0 - 10.0;
    for row in rows {
        d.text_left(&row.mark, (x0 + 5.0, y), NOTE_TEXT, INK);
        d.text_left(&letters::cut_to(&row.area, NOTE_TEXT, SCHEDULE_AREA_W - 4.0), (x0 + bw - SCHEDULE_AREA_W, y), NOTE_TEXT, INK);
        for line in schedule_name_lines(row) {
            d.text_left(&line, (x0 + 5.0 + 22.0, y), NOTE_TEXT, INK);
            y -= pitch;
        }
    }
}

/// The key: a sample of each kind of line the page draws, in the pen it is drawn in on the page.
fn key_box(d: &mut Drawing, frames: &mut Vec<[f32; 4]>, page: &PlanPage, n: f64, (x0, y0): (f32, f32), (bw, bh): (f32, f32)) {
    let wide = pen::points(pen::annotation_mm(pen::BORDER));
    d.line(vec![(x0, y0), (x0 + bw, y0), (x0 + bw, y0 + bh), (x0, y0 + bh)], wide, INK, None, true);
    frames.push([x0, y0, x0 + bw, y0 + bh]);
    for (j, (label, swatch)) in page.key.iter().enumerate() {
        let y = y0 + bh - 9.0 - j as f32 * 12.0;
        if *swatch == Swatch::RoomLine {
            // A bit of room line: thin, in long dashes.
            let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
            d.prims.push(Prim::Line { pts: vec![(x0 + 6.0, y + 0.5), (x0 + 34.0, y + 0.5)], width: thin, color: INK, dash: Some(vec![ROOM_LINE_DASH, ROOM_LINE_GAP]), closed: false, round: false });
            d.text_left(&letters::cut_to(label, NOTE_TEXT, bw - 44.0), (x0 + 40.0, y - 3.0), NOTE_TEXT, INK);
            continue;
        }
        let Swatch::Line(weight) = swatch else { continue };
        let (width, color, dashed) = ink_of(*weight, n);
        // A bit of wall in section: the outline in its pen, filled when the page fills it.
        let ring = vec![(x0 + 6.0, y - 2.0), (x0 + 34.0, y - 2.0), (x0 + 34.0, y + 3.0), (x0 + 6.0, y + 3.0)];
        if *weight == Weight::Heavy {
            d.fill(vec![ring.clone()], POCHE);
        }
        let dash = if dashed { dash_for(run_len(&ring, true), true) } else { None };
        d.line(ring, width, color, dash, true);
        d.text_left(&letters::cut_to(label, NOTE_TEXT, bw - 44.0), (x0 + 40.0, y - 3.0), NOTE_TEXT, INK);
    }
}

/// A north arrow, plan north up the page, at the right of the band: an arrow with its head
/// filled, N over it, and PLAN NORTH at its left (true north is not in the model).
fn north_arrow(d: &mut Drawing, (x0, y0): (f32, f32), bh: f32) {
    let thin = pen::points(pen::annotation_mm(pen::HEAVY));
    let cx = x0 + ARROW_W - 10.0;
    let top = y0 + bh - 12.0;
    d.line(vec![(cx, y0 + 2.0), (cx, top - 10.0)], thin, INK, None, false);
    d.fill(vec![vec![(cx, top), (cx - 4.5, top - 12.0), (cx + 4.5, top - 12.0)]], INK);
    d.text("N", (cx, top + 5.0), NOTE_TEXT + 1.0, 0.0, INK);
    d.text_left("PLAN", (x0, y0 + bh / 2.0 + 1.0), NOTE_TEXT, INK);
    d.text_left("NORTH", (x0, y0 + bh / 2.0 - 9.0), NOTE_TEXT, INK);
}

/// A graphic scale: a bar of four blocks, a whole number of feet long, with its feet lettered
/// under it and the scale said under that; it is true only on a sheet printed at full size, and
/// says so. `(x0, ytop)` is the bar's left end and top; `room` the width it may take. Returns the
/// length of its bar as written ("16'-0\"").
fn graphic_scale(d: &mut Drawing, scale: (u32, u32), k: f32, (x0, ytop): (f32, f32), room: f32) -> String {
    let want = room.min(216.0);
    let feet = [100.0f32, 80.0, 40.0, 20.0, 16.0, 8.0, 4.0].into_iter().find(|f| f * k <= want - 20.0).unwrap_or(4.0);
    let block = feet / 4.0 * k;
    let h = 4.0;
    let y = ytop - h;
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    for i in 0..4 {
        let ring = vec![(x0 + block * i as f32, y), (x0 + block * (i + 1) as f32, y), (x0 + block * (i + 1) as f32, y + h), (x0 + block * i as f32, y + h)];
        if i % 2 == 0 {
            d.fill(vec![ring.clone()], INK);
        }
        d.line(ring, thin, INK, None, true);
    }
    for (i, label) in [(0.0f32, "0".to_string()), (2.0, format!("{}", (feet / 2.0) as i64)), (4.0, format!("{} FT", feet as i64))] {
        let x = x0 + block * i;
        d.line(vec![(x, y - 1.0), (x, y + h + 1.0)], thin, INK, None, false);
        d.text(&label, (x, y - 7.0), NOTE_TEXT, 0.0, INK);
    }
    d.text_left(&format!("{}, PRINTED FULL SIZE", size::scale_said(scale)), (x0, y - 21.0), NOTE_TEXT, INK);
    strings::written(strings::sixteenths(f64::from(feet)))
}

fn shifted(pr: Prim, dx: f32, dy: f32) -> Prim {
    let mv = |pts: Vec<(f32, f32)>| -> Vec<(f32, f32)> { pts.into_iter().map(|(x, y)| (x + dx, y + dy)).collect() };
    match pr {
        Prim::Fill { rings, color } => Prim::Fill { rings: rings.into_iter().map(mv).collect(), color },
        Prim::Line { pts, width, color, dash, closed, round } => Prim::Line { pts: mv(pts), width, color, dash, closed, round },
    }
}

/// The box a set of prims lies in, points: x0, y0, x1, y1.
fn extent(prims: &[Prim]) -> (f32, f32, f32, f32) {
    let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for pr in prims {
        let pts: Vec<(f32, f32)> = match pr {
            Prim::Fill { rings, .. } => rings.iter().flatten().copied().collect(),
            Prim::Line { pts, .. } => pts.clone(),
        };
        for (x, y) in pts {
            b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
        }
    }
    b
}

/// The title box: the job's title, the page, the level, the scale, the date, and the sheet's
/// number and its place in the set. Ported from BIMCraft's built-in title box (`sheet.rs:167`,
/// `title_block`): a frame in the border's pen, a rule under the heading, text in the sheet's
/// pens; its rows are ours (BIMCraft's are the project, the sheet's name, the scale and its
/// number). Every row is lettered at its height or cut to its room (`letters::cut_to`), never
/// smaller than 3/32 in and never past the frame; a title too long for one line goes to two.
#[allow(clippy::too_many_arguments)]
fn title_box(d: &mut Drawing, frames: &mut Vec<[f32; 4]>, plan: &LevelPlan, page: &PlanPage, scale: (u32, u32), date: &str, number: usize, total: usize, at: (f32, f32)) -> Vec<String> {
    let (x0, y0) = at;
    let (x1, y1) = (x0 + BOX_W, y0 + BOX_H);
    let wide = pen::points(pen::annotation_mm(pen::BORDER));
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    d.line(vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)], wide, INK, None, true);
    frames.push([x0, y0, x1, y1]);
    // Rows from the top: the job, the page, the level, the scale, the date; and a cell for the
    // sheet's number at the right of the four lower rows.
    d.line(vec![(x0, y1 - 28.0), (x1, y1 - 28.0)], thin, INK, None, false);
    d.line(vec![(x1 - NUMBER_W, y0), (x1 - NUMBER_W, y1 - 28.0)], thin, INK, None, false);
    let room = BOX_W - NUMBER_W - 10.0;
    let title = if plan.title.is_empty() { "UNTITLED JOB".to_string() } else { plan.title.to_uppercase() };
    let title_lines = if font::width(&title, 10.0) <= BOX_W - 10.0 {
        d.text_left(&title, (x0 + 5.0, y1 - 20.0), 10.0, INK);
        vec![title.clone()]
    } else {
        let lines = letters::lines_of(&title, 8.0, BOX_W - 10.0, 2);
        for (j, line) in lines.iter().enumerate() {
            d.text_left(line, (x0 + 5.0, y1 - 12.5 - j as f32 * 12.5), 8.0, INK);
        }
        lines
    };
    let name = page.show.name();
    d.text_left(&letters::cut_to(name, 8.0, room), (x0 + 5.0, y1 - 28.0 - 12.0), 8.0, INK);
    let level = format!("LEVEL: {}", plan.level.to_uppercase());
    d.text_left(&letters::cut_to(&level, 7.2, room), (x0 + 5.0, y1 - 28.0 - 26.0), 7.2, INK);
    let sc = format!("SCALE: {}", size::scale_said(scale));
    d.text_left(&letters::cut_to(&sc, 7.2, room), (x0 + 5.0, y1 - 28.0 - 40.0), 7.2, INK);
    let dt = format!("DATE: {date}");
    d.text_left(&letters::cut_to(&dt, 7.2, room), (x0 + 5.0, y1 - 28.0 - 54.0), 7.2, INK);
    let mid = (y0 + y1 - 28.0) / 2.0;
    d.text(&page.sheet, (x1 - NUMBER_W / 2.0, mid + 8.0), 14.0, 0.0, INK);
    d.text(&format!("SHEET {number} OF {total}"), (x1 - NUMBER_W / 2.0, mid - 14.0), NOTE_TEXT, 0.0, INK);
    title_lines
}

/// How a weight is inked on a page at 1 : `n`: the pen's width, its colour, and whether dashed.
fn ink_of(weight: Weight, n: f64) -> (f32, paper::Rgb, bool) {
    match weight {
        Weight::Heavy => (pen::points(pen::model_mm(pen::CUT, n)), INK, false),
        Weight::Light => (pen::points(pen::model_mm(pen::PROJECTION, n)), FAINT, false),
        Weight::Dashed => (pen::points(pen::model_mm(pen::PROJECTION, n)), INK, true),
    }
}

/// What a page's plan is: its prims and lettering, and the rooms too small to letter.
struct Art {
    d: Drawing,
    schedule: Vec<ScheduleRow>,
    /// Things set in place drawn as an outline with no name in it.
    unnamed: usize,
    strings: Vec<DrawnString>,
    angles: Vec<DrawnAngle>,
    /// Sides where no overall is drawn, each with why.
    no_overall: Vec<(String, String)>,
    /// Whether a string was drawn along a wall inside the plan for a run of things set in place.
    run_strings: bool,
}

type Pt = (f32, f32);

/// Everything drawn so far that lettering must keep clear of: the lines (as segments), the cut
/// walls' fills, and the lettering already laid (and the circles round marks).
#[derive(Default)]
struct Ctx {
    segs: Vec<(Pt, Pt)>,
    fills: Vec<Vec<Pt>>,
    placed: Vec<Letter>,
    blocks: Vec<Letter>,
    unlettered: Vec<(char, Option<char>)>,
}

/// Whether `q` is inside the shape the rings make (an even number of edges to its right is out).
fn inside_rings(rings: &[Vec<Pt>], q: Pt) -> bool {
    let mut crossings = 0;
    for ring in rings {
        for i in 0..ring.len() {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            if (a.1 > q.1) != (b.1 > q.1) && q.0 < a.0 + (q.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

impl Ctx {
    fn line(&mut self, pts: &[Pt], closed: bool) {
        for w in pts.windows(2) {
            self.segs.push((w[0], w[1]));
        }
        if closed && pts.len() > 2 {
            self.segs.push((pts[pts.len() - 1], pts[0]));
        }
    }
    /// A piece of lettering is clear when it is over no other, no drawn line runs through it, and it is not on a cut wall.
    fn clear(&self, l: &Letter) -> bool {
        !self.placed.iter().chain(&self.blocks).any(|o| l.crosses(o)) && !self.segs.iter().any(|&(a, b)| l.touches_segment(a, b)) && !inside_rings(&self.fills, l.centre)
    }
    /// A line is clear when it runs through no lettering laid.
    fn line_clear(&self, a: Pt, b: Pt) -> bool {
        !self.placed.iter().chain(&self.blocks).any(|o| o.touches_segment(a, b))
    }
}

/// `text` lettered, centred at `c`, into the drawing and the lettering laid.
fn lettered(out: &mut Vec<Prim>, ctx: &mut Ctx, text: &str, c: Pt, h: f32, angle: f32) {
    let mut d = Drawing::new(0.0, 0.0);
    d.text(text, c, h, angle, INK);
    ctx.placed.extend(d.letters);
    ctx.unlettered.extend(d.unlettered);
    out.extend(d.prims);
}

/// A dimension string on the page: its stations on the line, the extension lines, which way
/// out of the house is, and each part's figure.
struct Dim {
    pts: Vec<Pt>,
    ext: Vec<(Pt, Pt)>,
    out: Pt,
    texts: Vec<(String, f32)>,
    /// A first or last figure may stand beyond the end of its string; a figure may be turned square to the string.
    ends_ok: bool,
    turned_ok: bool,
}

fn tick_of(s: Pt, t: Pt) -> (Pt, Pt) {
    let slash = (t.0 * 0.7071 - t.1 * 0.7071, t.1 * 0.7071 + t.0 * 0.7071);
    ((s.0 - slash.0 * 3.5, s.1 - slash.1 * 3.5), (s.0 + slash.0 * 3.5, s.1 + slash.1 * 3.5))
}

fn unit_of(a: Pt, b: Pt) -> Pt {
    let l = (b.0 - a.0).hypot(b.1 - a.1);
    if l < 1e-6 { (1.0, 0.0) } else { ((b.0 - a.0) / l, (b.1 - a.1) / l) }
}

/// The lines of a dimension string: the string, its extension lines and a tick at each station.
fn dim_lines(d: &Dim) -> (Vec<Prim>, Vec<(Pt, Pt)>) {
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    let heavy = pen::points(pen::annotation_mm(pen::HEAVY));
    let line = |pts: Vec<Pt>, width: f32| Prim::Line { pts, width, color: INK, dash: None, closed: false, round: false };
    let mut prims = Vec::new();
    let mut segs = Vec::new();
    let (a, b) = (d.pts[0], d.pts[d.pts.len() - 1]);
    prims.push(line(vec![a, b], thin));
    segs.push((a, b));
    for &(s, e) in &d.ext {
        prims.push(line(vec![s, e], thin));
        segs.push((s, e));
    }
    let t = unit_of(a, b);
    for &q in &d.pts {
        let (s, e) = tick_of(q, t);
        prims.push(line(vec![s, e], heavy));
        segs.push((s, e));
    }
    (prims, segs)
}

/// A figure read from below or from the right: the angle of a direction, turned so it is never upside down.
fn reading_angle(t: Pt) -> f32 {
    let mut angle = t.1.atan2(t.0);
    if angle > std::f32::consts::FRAC_PI_2 + 1e-4 {
        angle -= std::f32::consts::PI;
    } else if angle <= -std::f32::consts::FRAC_PI_2 + 1e-4 {
        angle += std::f32::consts::PI;
    }
    angle
}

/// Where each part's figure goes: over the middle of its part of the string on the side away
/// from the house, or on the other side, at the height asked or at 3/32 in; clear of what is drawn
/// and of the lines of the string itself. The bool is false when some figure has no clear place
/// (it is then put where it first would go, and the page's check says so).
fn dim_texts(d: &Dim, ctx: &Ctx, own: &[(Pt, Pt)]) -> (Vec<(String, Pt, f32, f32)>, bool) {
    let t = unit_of(d.pts[0], d.pts[d.pts.len() - 1]);
    let angle = reading_angle(t);
    let mut chosen: Vec<Letter> = Vec::new();
    let mut put: Vec<(String, Pt, f32, f32)> = Vec::new();
    let mut ok = true;
    for (i, (text, want)) in d.texts.iter().enumerate() {
        let (a, b) = (d.pts[i], d.pts[i + 1]);
        let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        let mut first: Option<(Pt, f32, f32)> = None;
        let mut found: Option<(Pt, f32, f32)> = None;
        let turned = reading_angle((-t.1, t.0));
        'search: for h in [*want, MIN_CAP] {
            let off = 3.0 + h / 2.0 + 1.0;
            let tw = font::width(text, h);
            let reach = tw / 2.0 + 7.0;
            // Over its part on either side of the string; the first or the last part, too short
            // for its figure, beyond the end of the string on its own line; or turned square to
            // the string, standing out from it.
            let mut spots: Vec<(Pt, f32)> = Vec::new();
            // Over the middle of its part, then slid along it a step at a time where the part is longer than the figure.
            let slack = ((b.0 - a.0).hypot(b.1 - a.1) - tw) / 2.0 - 3.0;
            let mut shifts = vec![0.0f32];
            let mut sh = 8.0f32;
            while sh <= slack {
                shifts.push(sh);
                shifts.push(-sh);
                sh += 8.0;
            }
            for shift in shifts {
                let m = (mid.0 + t.0 * shift, mid.1 + t.1 * shift);
                spots.push(((m.0 + d.out.0 * off, m.1 + d.out.1 * off), angle));
                spots.push(((m.0 - d.out.0 * off, m.1 - d.out.1 * off), angle));
            }
            if d.ends_ok {
                for extra in [0.0f32, 14.0, 28.0, 42.0] {
                    if i == 0 {
                        spots.push(((a.0 - t.0 * (reach + extra), a.1 - t.1 * (reach + extra)), angle));
                    }
                    if i + 1 == d.texts.len() {
                        spots.push(((b.0 + t.0 * (reach + extra), b.1 + t.1 * (reach + extra)), angle));
                    }
                }
            }
            if d.turned_ok {
                let o2 = 3.0 + tw / 2.0 + 1.0;
                spots.push(((mid.0 + d.out.0 * o2, mid.1 + d.out.1 * o2), turned));
                spots.push(((mid.0 - d.out.0 * o2, mid.1 - d.out.1 * o2), turned));
            }
            for (c, ang) in spots {
                let cand = Letter::centred(text, c, h, ang);
                first.get_or_insert((c, h, ang));
                if ctx.clear(&cand) && !chosen.iter().any(|o| cand.crosses(o)) && !own.iter().any(|&(p, q)| cand.touches_segment(p, q)) {
                    found = Some((c, h, ang));
                    break 'search;
                }
            }
        }
        let (c, h, angle) = match found {
            Some(f) => f,
            None => {
                ok = false;
                first.unwrap_or((mid, *want, angle))
            }
        };
        chosen.push(Letter::centred(text, c, h, angle));
        put.push((text.clone(), c, h, angle));
    }
    (put, ok)
}

/// Lay a dimension string on the page (its lines, then its figures), whether or not its figures
/// found a clear place.
fn commit_dim(out: &mut Vec<Prim>, ctx: &mut Ctx, d: &Dim, texts: Vec<(String, Pt, f32, f32)>, prims: Vec<Prim>, segs: Vec<(Pt, Pt)>) {
    out.extend(prims);
    ctx.segs.extend(segs);
    for (text, c, h, angle) in texts {
        lettered(out, ctx, &text, c, h, angle);
    }
    let _ = d;
}

/// The first of `cands` whose lines run through no lettering and whose figures all find a clear
/// place is laid; when none is, the first is laid all the same (and the page's check says what
/// is wrong). Returns which was laid and whether it was clear.
fn lay_inside(out: &mut Vec<Prim>, ctx: &mut Ctx, cands: &[Dim]) -> Option<(usize, bool)> {
    for (i, d) in cands.iter().enumerate() {
        let (prims, segs) = dim_lines(d);
        if segs.iter().any(|&(a, b)| !ctx.line_clear(a, b)) {
            continue;
        }
        let (texts, ok) = dim_texts(d, ctx, &segs);
        if ok {
            commit_dim(out, ctx, d, texts, prims, segs);
            return Some((i, true));
        }
    }
    let d = cands.first()?;
    let (prims, segs) = dim_lines(d);
    let (texts, _) = dim_texts(d, ctx, &segs);
    commit_dim(out, ctx, d, texts, prims, segs);
    Some((0, false))
}

fn with_vif(text: String, vif: bool) -> String {
    if vif { format!("{text} VIF") } else { text }
}

fn seg_cross(p: Pt, q: Pt, a: Pt, b: Pt) -> bool {
    let d = |u: Pt, v: Pt, w: Pt| (v.0 - u.0) * (w.1 - u.1) - (v.1 - u.1) * (w.0 - u.0);
    let (d1, d2, d3, d4) = (d(a, b, p), d(a, b, q), d(p, q, a), d(p, q, b));
    ((d1 > 0.0) != (d2 > 0.0)) && ((d3 > 0.0) != (d4 > 0.0))
}

/// How far, feet, from `q` along the unit direction `dir` to the nearest edge of any ring.
fn ray(rings: &[Vec<V>], q: V, dir: V) -> f64 {
    let mut best = f64::INFINITY;
    for ring in rings {
        for i in 0..ring.len() {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            let e = V(b.0 - a.0, b.1 - a.1);
            let den = dir.0 * e.1 - dir.1 * e.0;
            if den.abs() < 1e-12 {
                continue;
            }
            let wv = V(a.0 - q.0, a.1 - q.1);
            let t = (wv.0 * e.1 - wv.1 * e.0) / den;
            let u = (wv.0 * dir.1 - wv.1 * dir.0) / den;
            if t > 1e-9 && (0.0..=1.0).contains(&u) {
                best = best.min(t);
            }
        }
    }
    best
}

/// The heights a room's name and area can be lettered at, at its point, within the walls round
/// it and (inside) no lower than 3/32 in; none when even the least does not fit.
fn room_fit(rings: &[Vec<V>], point: V, k: f64, name: &str, area: &str) -> Option<(f32, f32)> {
    let reach = |dir: V| (ray(rings, point, dir) * k).min(1.0e4) as f32;
    let half_w = reach(V(1.0, 0.0)).min(reach(V(-1.0, 0.0))) - 2.0;
    let half_h = reach(V(0.0, 1.0)).min(reach(V(0.0, -1.0))) - 2.0;
    [(ROOM_NAME, DIM_TEXT), (DIM_TEXT, MIN_CAP), (MIN_CAP, MIN_CAP)].into_iter().find(|&(hn, ha)| font::width(name, hn).max(font::width(area, ha)) / 2.0 + 1.0 <= half_w && hn.max(ha) + 5.0 <= half_h)
}

/// The places a label may be tried at, nearest its point first: the point, then rings of eight
/// round it every 5 points out to `far`.
fn nearby(c: Pt, far: f32) -> Vec<Pt> {
    let mut v = vec![c];
    let mut r = 5.0;
    while r <= far {
        let steps = 8 + (r / 10.0) as usize * 2;
        for i in 0..steps {
            let a = i as f32 * std::f32::consts::TAU / steps as f32;
            v.push((c.0 + r * a.cos(), c.1 + r * a.sin()));
        }
        r += 5.0;
    }
    v
}

/// A page's plan in points from the level's origin: its walls (filled and inked), its
/// openings, what is set in place, its rooms, its room lines and its dimensions; and the
/// lettering of each, over no other lettering and over no line.
fn plan_prims(_page: &PlanPage, layer: &Layer, k: f64, n: f64) -> Art {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let mut out: Vec<Prim> = Vec::new();
    let mut ctx = Ctx::default();
    let fine = |mm: f64| pen::points(mm);
    let ink_of = |weight: Weight| ink_of(weight, n);
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    let heavy_rings: Vec<Vec<V>> = layer.walls.iter().filter(|w| w.1 == Weight::Heavy).map(|w| w.2.clone()).filter(|r| r.len() >= 3).collect();
    // The cut walls filled, as one shape.
    let fills: Vec<Vec<(f32, f32)>> = cut::union(&heavy_rings).iter().map(|r| r.iter().map(|q| p(*q, k)).collect()).collect();
    ctx.fills.extend(fills.iter().cloned());
    out.push(Prim::Fill { rings: fills, color: POCHE });
    out.retain(|pr| !matches!(pr, Prim::Fill { rings, .. } if rings.is_empty()));
    // Ink: each weight's walls joined among themselves, light first, heavy last.
    for weight in [Weight::Light, Weight::Dashed, Weight::Heavy] {
        let group: Vec<Vec<V>> = layer.walls.iter().filter(|w| w.1 == weight && w.2.len() >= 3).map(|w| w.2.clone()).collect();
        if group.is_empty() {
            continue;
        }
        let (width, color, dashed) = ink_of(weight);
        let reach = f64::from(width) / k;
        for runs in cut::cut_edges(&group, reach, 0.003) {
            for run in runs {
                let (pts, closed) = cut::ink_cut(&run);
                let pts: Vec<(f32, f32)> = pts.iter().map(|q| p(*q, k)).collect();
                ctx.line(&pts, closed);
                let dash = if dashed { dash_for(run_len(&pts, closed), closed) } else { None };
                out.push(Prim::Line { pts, width, color, dash, closed, round: false });
            }
        }
    }
    // Doors and windows: the gap cleared to the paper, jambs, and the leaf or the glass.
    for (wi, [s0, s1], door, weight, _) in &layer.openings {
        let w = &layer.walls[*wi].3;
        let t = (w.end - w.start).unit();
        let at = |s: f64, off: f64| w.start + t * s + t.perp() * off;
        let past = 0.5 / k;
        let gap = vec![at(*s0, w.int - past), at(*s1, w.int - past), at(*s1, w.ext + past), at(*s0, w.ext + past)];
        out.push(Prim::Fill { rings: vec![gap.iter().map(|q| p(*q, k)).collect()], color: PAPER });
        let (_, color, dashed) = ink_of(*weight);
        let (jamb, leaf, glass) = if *weight == Weight::Heavy { (pen::HEAVY, pen::MEDIUM, pen::PROJECTION) } else { (pen::PROJECTION, pen::PROJECTION, pen::PROJECTION) };
        let jamb_w = fine(pen::model_mm(jamb, n));
        for s in [*s0, *s1] {
            let pts = vec![p(at(s, w.int), k), p(at(s, w.ext), k)];
            ctx.line(&pts, false);
            out.push(Prim::Line { pts, width: jamb_w, color, dash: None, closed: false, round: false });
        }
        let mid = (w.ext + w.int) / 2.0;
        let lines: Vec<f64> = if *door { vec![mid] } else { vec![w.int, mid, w.ext] };
        for off in lines {
            let pts = vec![p(at(*s0, off), k), p(at(*s1, off), k)];
            ctx.line(&pts, false);
            let dash = if dashed { dash_for(run_len(&pts, false), false) } else { None };
            out.push(Prim::Line { pts, width: fine(pen::model_mm(if *door { leaf } else { glass }, n)), color, dash, closed: false, round: false });
        }
    }
    // Room lines: where no wall stands and a room ends, a thin line of long dashes.
    for (a, b) in &layer.room_lines {
        let pts = vec![p(*a, k), p(*b, k)];
        ctx.line(&pts, false);
        out.push(Prim::Line { pts, width: thin, color: INK, dash: Some(vec![ROOM_LINE_DASH, ROOM_LINE_GAP]), closed: false, round: false });
    }
    // What is set in place: the outline of each footprint, in the pen of a door's leaf; dashed
    // where it hangs above the cut. Its name is lettered inside it where it fits there and no
    // line runs through it; what hangs over the floor stands over what is named below it and
    // is not named, and the notes say how many things have no name.
    let mut rings_pts: Vec<Vec<(f32, f32)>> = Vec::new();
    for (weight, ring, above, ..) in &layer.things {
        let pts: Vec<(f32, f32)> = ring.iter().map(|q| p(*q, k)).collect();
        let color = if *weight == Weight::Light { FAINT } else { INK };
        let dash = if *above { dash_for(run_len(&pts, true), true) } else { None };
        ctx.line(&pts, true);
        out.push(Prim::Line { pts: pts.clone(), width: fine(pen::model_mm(pen::MEDIUM, n)), color, dash, closed: true, round: false });
        rings_pts.push(pts);
    }
    let mut unnamed = 0usize;
    for (i, (_, _, above, name, _)) in layer.things.iter().enumerate() {
        let pts = &rings_pts[i];
        let side = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).hypot(b.1 - a.1), (b.1 - a.1).atan2(b.0 - a.0));
        let ((len, mut angle), (wid, _)) = (side(pts[0], pts[1]), side(pts[1], pts[2]));
        let e1 = unit_of(pts[0], pts[1]);
        let e2 = unit_of(pts[0], pts[3]);
        if angle > std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle -= std::f32::consts::PI;
        } else if angle <= -std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle += std::f32::consts::PI;
        }
        let c = ((pts[0].0 + pts[2].0) / 2.0, (pts[0].1 + pts[2].1) / 2.0);
        let mut put = false;
        if !*above {
            'heights: for h in [DIM_TEXT, MIN_CAP] {
                let (w, hh) = (font::width(name, h) + 1.0, h + 1.0);
                if w + 4.0 > len || hh + 4.0 > wid {
                    continue;
                }
                // Along its outline's own axes, from the middle out, a step at a time.
                let (ru, rv) = ((len - w) / 2.0 - 2.0, (wid - hh) / 2.0 - 2.0);
                let mut at: Vec<(f32, f32)> = Vec::new();
                let mut u = -ru;
                while u <= ru + 0.01 {
                    let mut v = -rv;
                    while v <= rv + 0.01 {
                        at.push((u, v));
                        v += 3.0;
                    }
                    u += 3.0;
                }
                at.push((0.0, 0.0));
                at.sort_by(|a, b| (a.0 * a.0 + a.1 * a.1).partial_cmp(&(b.0 * b.0 + b.1 * b.1)).unwrap_or(std::cmp::Ordering::Equal));
                for (u, v) in at {
                    let cc = (c.0 + e1.0 * u + e2.0 * v, c.1 + e1.1 * u + e2.1 * v);
                    let l = Letter::centred(name, cc, h, angle);
                    if ctx.clear(&l) {
                        lettered(&mut out, &mut ctx, name, cc, h, angle);
                        put = true;
                        break 'heights;
                    }
                }
            }
        }
        if !put {
            unnamed += 1;
        }
    }
    // The rooms' rings, for the walls round each room, in feet.
    let rings: Vec<Vec<V>> = layer.walls.iter().map(|w| w.2.clone()).collect();
    let rings_pt: Vec<Vec<Pt>> = rings.iter().map(|r| r.iter().map(|q| p(*q, k)).collect()).collect();
    let mut made: Vec<DrawnString> = Vec::new();
    let mut angles: Vec<DrawnAngle> = Vec::new();
    let mut no_overall: Vec<(String, String)> = Vec::new();
    let mut run_strings = false;

    // ---- The strings outside the plan ----
    let sw: Vec<strings::W> = layer.walls.iter().enumerate().map(|(i, w)| strings::W { start: w.3.start, end: w.3.end, ext: w.3.ext, int: w.3.int, edge: layer.edge[i], out: layer.outward[i], sure: layer.lengths[i].0.is_some() && layer.lengths[i].1 == Status::Known }).collect();
    let sops: Vec<strings::Op> = layer.openings.iter().map(|(wi, [s0, s1], _, _, sure)| strings::Op { wall: *wi, s0: *s0, s1: *s1, sure: *sure }).collect();
    let sides = strings::sides(&sw, &sops, tol);
    for (side, why) in &sides.no_overall {
        no_overall.push((side.name().to_string(), why.clone()));
    }
    // How far out each side of the house reaches, points.
    let all_pts: Vec<Pt> = rings_pt.iter().flatten().copied().collect();
    let reach = |side: Side| -> f32 {
        let (lo, hi) = all_pts.iter().fold(((f32::MAX, f32::MAX), (f32::MIN, f32::MIN)), |a, q| ((a.0 .0.min(q.0), a.0 .1.min(q.1)), (a.1 .0.max(q.0), a.1 .1.max(q.1))));
        match side {
            Side::South => lo.1,
            Side::North => hi.1,
            Side::West => lo.0,
            Side::East => hi.0,
        }
    };
    let mut stack_len: Vec<(Side, usize)> = Vec::new();
    let mut dims: Vec<(Dim, DrawnString)> = Vec::new();
    for side in Side::ALL {
        let mine: Vec<&strings::Chain> = sides.chains.iter().filter(|c| c.side == side).collect();
        if mine.is_empty() {
            continue;
        }
        stack_len.push((side, mine.len()));
        let o = side.out();
        let o = (o.0 as f32, o.1 as f32);
        let horizontal = side.axis() == Axis::Across;
        let line_at = |i: usize| reach(side) + (o.0 + o.1) * (DIM_OFF + i as f32 * STRING_PITCH);
        let pt = |c: f64, cross: f32| if horizontal { ((c * k) as f32, cross) } else { (cross, (c * k) as f32) };
        // The outermost string a station is on carries its extension line.
        let outermost = |step: i64| mine.iter().enumerate().rev().find(|(_, c)| c.steps.contains(&step)).map(|(i, _)| i);
        for (i, chain) in mine.iter().enumerate() {
            let cross = line_at(i);
            let pts: Vec<Pt> = chain.stations.iter().map(|s| pt(s.c, cross)).collect();
            let mut ext = Vec::new();
            for (s, step) in chain.stations.iter().zip(&chain.steps) {
                if outermost(*step) == Some(i) {
                    let a = (s.anchor.0 as f32 * k as f32, s.anchor.1 as f32 * k as f32);
                    let from = (a.0 + o.0 * 3.0, a.1 + o.1 * 3.0);
                    let to = pt(s.c, cross + (o.0 + o.1) * 3.0);
                    ext.push((from, to));
                }
            }
            let want = if chain.kind == strings::Kind::Overall { OVERALL_TEXT } else { DIM_TEXT };
            let parts = chain.parts();
            let texts: Vec<(String, f32)> = parts.iter().zip(&chain.vif).map(|(t, v)| (with_vif(t.clone(), *v), want)).collect();
            let record = DrawnString { place: side.name().to_string(), kind: chain.kind.name().to_string(), parts: parts.iter().cloned().zip(chain.vif.iter().copied()).collect(), whole: chain.whole() };
            dims.push((Dim { pts, ext, out: o, texts, ends_ok: true, turned_ok: i + 1 == mine.len() }, record));
        }
    }
    // A wall off square is dimensioned along itself, outside the plan on its outer side.
    for (i, w) in sw.iter().enumerate() {
        if w.axis() != Axis::Angled || !w.edge {
            continue;
        }
        let (length, status) = layer.lengths[i];
        let sure = length.is_some() && status == Status::Known;
        let own = strings::own(i, length.unwrap_or_else(|| w.length()), sure, &sops);
        let t = (w.end - w.start).unit();
        let o = (w.out.0 as f32, w.out.1 as f32);
        let face = w.face();
        let at_pt = |s: f64| w.start + t * s + w.out * face;
        let pts: Vec<Pt> = own.at.iter().map(|&s| { let a = at_pt(s); ((a.0 * k) as f32 + o.0 * DIM_OFF, (a.1 * k) as f32 + o.1 * DIM_OFF) }).collect();
        let ext: Vec<(Pt, Pt)> = own.at.iter().zip(&pts).map(|(&s, q)| { let a = at_pt(s); (((a.0 * k) as f32 + o.0 * 3.0, (a.1 * k) as f32 + o.1 * 3.0), (q.0 + o.0 * 3.0, q.1 + o.1 * 3.0)) }).collect();
        let parts = own.parts();
        let texts: Vec<(String, f32)> = parts.iter().zip(&own.vif).map(|(t, v)| (with_vif(t.clone(), *v), DIM_TEXT)).collect();
        let record = DrawnString { place: layer.paths[i].clone(), kind: "along its wall".to_string(), parts: parts.iter().cloned().zip(own.vif.iter().copied()).collect(), whole: own.whole() };
        dims.push((Dim { pts, ext, out: o, texts, ends_ok: true, turned_ok: true }, record));
    }
    // Lines first, then the figures, so a figure keeps clear of every string's lines.
    let lines: Vec<(Vec<Prim>, Vec<(Pt, Pt)>)> = dims.iter().map(|(d, _)| dim_lines(d)).collect();
    for (prims, segs) in &lines {
        out.extend(prims.iter().cloned());
        ctx.segs.extend(segs.iter().copied());
    }
    for ((d, record), (_, segs)) in dims.iter().zip(&lines) {
        let (texts, _) = dim_texts(d, &ctx, &[]);
        let _ = segs;
        for (text, c, h, angle) in texts {
            lettered(&mut out, &mut ctx, &text, c, h, angle);
        }
        made.push(record.clone());
    }

    // ---- The rooms ----
    let mut schedule: Vec<ScheduleRow> = Vec::new();
    for (name, point, area, _) in &layer.rooms {
        let c0 = p(V(point[0], point[1]), k);
        let same_space = |c: Pt| !rings_pt.iter().any(|r| (0..r.len()).any(|i| seg_cross(c0, c, r[i], r[(i + 1) % r.len()])));
        let mut tried: Option<Drawing> = None;
        for c in nearby(c0, 160.0) {
            if !same_space(c) {
                continue;
            }
            let Some((hn, ha)) = room_fit(&rings, V(c.0 as f64 / k, c.1 as f64 / k), k, name, area) else { continue };
            let (ln, la) = (Letter::centred(name, (c.0, c.1 + hn / 2.0 + 2.5), hn, 0.0), Letter::centred(area, (c.0, c.1 - ha / 2.0 - 2.5), ha, 0.0));
            if ctx.clear(&ln) && ctx.clear(&la) && !ln.crosses(&la) {
                let mut d = Drawing::new(0.0, 0.0);
                d.text(name, (c.0, c.1 + hn / 2.0 + 2.5), hn, 0.0, INK);
                d.text(area, (c.0, c.1 - ha / 2.0 - 2.5), ha, 0.0, INK);
                tried = Some(d);
                break;
            }
        }
        match tried {
            Some(d) => {
                ctx.placed.extend(d.letters.clone());
                ctx.unlettered.extend(d.unlettered);
                out.extend(d.prims);
            }
            None => {
                // A numbered mark in a circle, and a row of the schedule at the side of the sheet.
                let mark = (schedule.len() + 1).to_string();
                let block = |c: Pt| Letter { text: String::new(), height: MIN_CAP, centre: c, half: (8.4, 8.4), angle: 0.0 };
                let spot = nearby(c0, 160.0).into_iter().filter(|&c| same_space(c)).find(|&c| {
                    let b = block(c);
                    !ctx.placed.iter().chain(&ctx.blocks).any(|o| b.crosses(o)) && !ctx.segs.iter().any(|&(a, z)| b.touches_segment(a, z)) && !inside_rings(&ctx.fills, c)
                });
                let c = spot.unwrap_or(c0);
                let mut d = Drawing::new(0.0, 0.0);
                d.text(&mark, c, MIN_CAP, 0.0, INK);
                let circle: Vec<(f32, f32)> = (0..24).map(|i| (c.0 + 8.0 * (i as f32 * std::f32::consts::TAU / 24.0).cos(), c.1 + 8.0 * (i as f32 * std::f32::consts::TAU / 24.0).sin())).collect();
                d.line(circle.clone(), pen::points(pen::annotation_mm(pen::PROJECTION)), INK, None, true);
                ctx.line(&circle, true);
                ctx.blocks.push(block(c));
                ctx.placed.extend(d.letters.clone());
                out.extend(d.prims);
                schedule.push(ScheduleRow { mark, name: name.clone(), area: area.clone() });
            }
        }
    }

    // ---- The strings inside the plan: along a wall with openings in it that is no outside wall,
    // from the nearest parallel wall to a partition that meets none, and along a run of things set in place. ----
    let nrm_of = |w: &strings::W, side: f64| {
        let t = (w.end - w.start).unit();
        let n = t.perp() * side;
        (t, n)
    };
    let face_on = |w: &strings::W, side: f64| if side > 0.0 { w.ext.max(w.int) } else { -w.ext.min(w.int) };
    for (i, w) in sw.iter().enumerate() {
        let has = sops.iter().any(|o| o.wall == i);
        if w.edge && w.axis() != Axis::Angled {
            continue; // an outside wall's openings are in its side's string
        }
        if w.edge && !has {
            continue;
        }
        if w.edge {
            continue; // an angled outside wall is dimensioned outside, above
        }
        // An interior wall: a string along it from its first end, through each opening's center, to its last
        // (an angled one with none: its own length).
        if !has && w.axis() != Axis::Angled {
            continue;
        }
        let (length, status) = layer.lengths[i];
        let sure = length.is_some() && status == Status::Known;
        let own = strings::own(i, length.unwrap_or_else(|| w.length()), sure, &sops);
        let parts = own.parts();
        let mut cands: Vec<Dim> = Vec::new();
        for d in [14.0f32, 24.0, 34.0, 46.0] {
            for side in [1.0f64, -1.0] {
                let (t, nv) = nrm_of(w, side);
                let face = face_on(w, side);
                let on = |s: f64| w.start + t * s + nv * face;
                let o = (nv.0 as f32, nv.1 as f32);
                let pts: Vec<Pt> = own.at.iter().map(|&s| { let a = on(s); ((a.0 * k) as f32 + o.0 * d, (a.1 * k) as f32 + o.1 * d) }).collect();
                let ext: Vec<(Pt, Pt)> = own.at.iter().zip(&pts).map(|(&s, q)| { let a = on(s); (((a.0 * k) as f32 + o.0 * 3.0, (a.1 * k) as f32 + o.1 * 3.0), (q.0 + o.0 * 3.0, q.1 + o.1 * 3.0)) }).collect();
                let texts = parts.iter().zip(&own.vif).map(|(t, v)| (with_vif(t.clone(), *v), DIM_TEXT)).collect();
                cands.push(Dim { pts, ext, out: o, texts, ends_ok: false, turned_ok: true });
            }
        }
        lay_inside(&mut out, &mut ctx, &cands);
        made.push(DrawnString { place: format!("{}, along it inside the plan", layer.paths[i]), kind: if has { "openings" } else { "wall" }.to_string(), parts: parts.iter().cloned().zip(own.vif.iter().copied()).collect(), whole: own.whole() });
    }
    for l in strings::locate(&sw, &sides.located, tol) {
        let (t_axis, vif) = (l.axis, l.vif);
        let text = with_vif(strings::written(l.steps[1]), vif);
        // Square to the two walls, at a few places along the part they share.
        let w = &sw[l.wall];
        let (lo, hi) = {
            let a = |q: &strings::W| { let (x, y) = (strings::along(q.start, t_axis), strings::along(q.end, t_axis)); (x.min(y), x.max(y)) };
            let (a1, a2) = (a(w), a(&sw[l.from]));
            (a1.0.max(a2.0), a1.1.min(a2.1))
        };
        let mut cands: Vec<Dim> = Vec::new();
        for f in [0.5, 0.35, 0.65, 0.2, 0.8, 0.1, 0.9] {
            let at = lo + (hi - lo) * f;
            let (p0, p1) = if t_axis == Axis::Across { ((at * k, l.from_cross * k), (at * k, l.to_cross * k)) } else { ((l.from_cross * k, at * k), (l.to_cross * k, at * k)) };
            let o = if t_axis == Axis::Across { (1.0f32, 0.0) } else { (0.0, 1.0) };
            cands.push(Dim { pts: vec![(p0.0 as f32, p0.1 as f32), (p1.0 as f32, p1.1 as f32)], ext: Vec::new(), out: o, texts: vec![(text.clone(), DIM_TEXT)], ends_ok: false, turned_ok: true });
        }
        lay_inside(&mut out, &mut ctx, &cands);
        made.push(DrawnString { place: format!("{}, from {}", layer.paths[l.wall], layer.paths[l.from]), kind: "located".to_string(), parts: vec![(strings::written(l.steps[1]), vif)], whole: strings::written(l.steps[1]) });
    }
    // Runs of things set in place against a wall, located by their start from the wall's first end.
    let thing_rings: Vec<([V; 4], bool)> = layer.things.iter().map(|t| ([t.1[0], t.1[1], t.1[2], t.1[3]], t.2)).collect();
    for run in strings::runs(&sw, &thing_rings) {
        let w = &sw[run.wall];
        let length = w.length();
        let stations: Vec<f64> = vec![0.0, run.s0.max(0.0), run.s1.min(length), length];
        let mut steps: Vec<i64> = Vec::new();
        let mut at: Vec<f64> = Vec::new();
        for s in stations {
            let n = strings::sixteenths(s);
            if steps.last() != Some(&n) {
                steps.push(n);
                at.push(s);
            }
        }
        if steps.len() < 3 {
            continue;
        }
        let unsure = !w.sure || run.things.iter().any(|&ti| layer.things[ti].4 != Status::Known);
        let parts: Vec<String> = steps.windows(2).map(|s| strings::written(s[1] - s[0])).collect();
        let (t, _) = nrm_of(w, run.side);
        let mut cands: Vec<Dim> = Vec::new();
        for d in [12.0f32, 22.0, 32.0] {
            let nv = t.perp() * run.side;
            let face = face_on(w, run.side);
            let on = |s: f64| w.start + t * s + nv * (face + run.depth);
            let o = (nv.0 as f32, nv.1 as f32);
            let pts: Vec<Pt> = at.iter().map(|&s| { let a = on(s); ((a.0 * k) as f32 + o.0 * d, (a.1 * k) as f32 + o.1 * d) }).collect();
            let ext: Vec<(Pt, Pt)> = at.iter().zip(&pts).map(|(&s, q)| { let a = on(s); (((a.0 * k) as f32 + o.0 * 3.0, (a.1 * k) as f32 + o.1 * 3.0), (q.0 + o.0 * 3.0, q.1 + o.1 * 3.0)) }).collect();
            let texts = parts.iter().map(|t| (with_vif(t.clone(), unsure), DIM_TEXT)).collect();
            cands.push(Dim { pts, ext, out: o, texts, ends_ok: false, turned_ok: true });
        }
        lay_inside(&mut out, &mut ctx, &cands);
        run_strings = true;
        made.push(DrawnString { place: format!("{}, along it inside the plan", layer.paths[run.wall]), kind: "run".to_string(), parts: parts.iter().cloned().map(|t| (t, unsure)).collect(), whole: strings::written(*steps.last().unwrap_or(&0)) });
    }

    // ---- Where walls meet off square: the angle, lettered at the corner ----
    for c in strings::corners(&sw, tol) {
        let text = strings::degrees_written(c.degrees);
        let at = p(c.at, k);
        let b = (c.bisector.0 as f32, c.bisector.1 as f32);
        let mut first: Option<(Pt, f32)> = None;
        let mut put = false;
        'angle: for h in [DIM_TEXT, MIN_CAP] {
            for far in [16.0f32, 24.0, 34.0, 46.0, 62.0] {
                // In the sector the angle is measured in, then in the one across the corner from it.
                for sign in [1.0f32, -1.0] {
                    let spot = (at.0 + b.0 * far * sign, at.1 + b.1 * far * sign);
                    first.get_or_insert((spot, h));
                    if ctx.clear(&Letter::centred(&text, spot, h, 0.0)) {
                        lettered(&mut out, &mut ctx, &text, spot, h, 0.0);
                        put = true;
                        break 'angle;
                    }
                }
            }
        }
        if !put {
            if let Some((spot, h)) = first {
                lettered(&mut out, &mut ctx, &text, spot, h, 0.0);
            }
        }
        angles.push(DrawnAngle { corner: format!("{} and {} at {},{}", layer.paths[c.a], layer.paths[c.b], crate::view::num(crate::value::round4(c.at.0)), crate::view::num(crate::value::round4(c.at.1))), text });
    }
    let _ = stack_len;
    let mut d = Drawing::new(0.0, 0.0);
    d.prims = out;
    d.letters = ctx.placed;
    d.unlettered = ctx.unlettered;
    Art { d, schedule, unnamed, strings: made, angles, no_overall, run_strings }
}

impl PlanPage {
    /// What is wrong with the page's lettering, each said: a piece under 3/32 in, a piece
    /// lying over another, a piece outside the sheet's frame, a piece across the edge of a
    /// box (the title box, the key, the schedule), and (call 182) a piece with any line of the
    /// sheet drawn through it (a wall, a string, an outline, a room line) or lying on a cut
    /// wall. Measured on the lettering as it is written on the page, after it is laid.
    pub fn lettering_faults(&self) -> Vec<String> {
        faults_in(&self.drawing, &self.frames)
    }
}

/// What is wrong with the lettering of a drawing (see `PlanPage::lettering_faults`); `frames` are
/// the boxes lettering must not cross the edge of.
fn faults_in(drawing: &Drawing, frames: &[[f32; 4]]) -> Vec<String> {
    {
        let mut out = Vec::new();
        let frame = if drawing.width > 0.0 { [BORDER_AT, BORDER_AT, drawing.width - BORDER_AT, drawing.height - BORDER_AT] } else { [f32::MIN, f32::MIN, f32::MAX, f32::MAX] };
        let ls = &drawing.letters;
        let mut flagged = vec![false; ls.len()];
        for (i, l) in ls.iter().enumerate() {
            if l.height < MIN_CAP - 1e-3 {
                out.push(format!("the lettering {:?} is {:.2} pt high, under 3/32 in", l.text, l.height));
            }
            if !l.within(frame) {
                out.push(format!("the lettering {:?} runs outside the sheet's frame", l.text));
            }
            for fr in frames {
                let inside = l.corners().iter().filter(|&&(x, y)| x >= fr[0] && x <= fr[2] && y >= fr[1] && y <= fr[3]).count();
                if inside != 0 && inside != 4 {
                    out.push(format!("the lettering {:?} runs across the edge of a box", l.text));
                    flagged[i] = true;
                }
            }
            for o in &ls[i + 1..] {
                if l.crosses(o) {
                    out.push(format!("the lettering {:?} lies over {:?}", l.text, o.text));
                    flagged[i] = true;
                }
            }
        }
        // Lines through lettering: every drawn line but the strokes of the lettering itself.
        for pr in &drawing.prims {
            match pr {
                Prim::Line { pts, closed, round: false, .. } => {
                    let mut segs: Vec<((f32, f32), (f32, f32))> = pts.windows(2).map(|w| (w[0], w[1])).collect();
                    if *closed && pts.len() > 2 {
                        segs.push((pts[pts.len() - 1], pts[0]));
                    }
                    for (i, l) in ls.iter().enumerate() {
                        if !flagged[i] && segs.iter().any(|&(a, b)| l.touches_segment(a, b)) {
                            out.push(format!("the lettering {:?} has a line drawn through it", l.text));
                            flagged[i] = true;
                        }
                    }
                }
                Prim::Fill { rings, color } if *color == POCHE => {
                    let rings_pt: Vec<Vec<(f32, f32)>> = rings.clone();
                    for (i, l) in ls.iter().enumerate() {
                        if !flagged[i] && inside_rings(&rings_pt, l.centre) {
                            out.push(format!("the lettering {:?} lies on a cut wall", l.text));
                            flagged[i] = true;
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }
}

impl PlanSet {
    /// Everything the program knows is not right on the pages as written, each in plain
    /// words with its page: said whenever the set is written, whether or not the list was
    /// asked for (call 182 item 4).
    pub fn faults(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (i, page) in self.pages.iter().enumerate() {
            for f in page.lettering_faults() {
                out.push(format!("page {} of {}, {}: {f}", i + 1, self.pages.len(), page.show.name()));
            }
        }
        out
    }

    /// The set as a PDF: a page of line work for each, the same bytes for the same set; its own
    /// title is the job's title (call 182).
    pub fn pdf(&self) -> Vec<u8> {
        let drawings: Vec<Drawing> = self.pages.iter().map(|p| p.drawing.clone()).collect();
        paper::pdf::pdf_titled(&drawings, &self.title, &format!("{}, {}", self.level, self.date))
    }

    /// The letters written that are not lettered as written: each with what it is lettered as
    /// (its plain letter, or the question mark), and how many of it a page carries at the most.
    pub fn unlettered(&self) -> Vec<(char, Option<char>, usize)> {
        let mut out: Vec<(char, Option<char>, usize)> = Vec::new();
        for page in &self.pages {
            let mut here: Vec<(char, Option<char>, usize)> = Vec::new();
            for &(c, plain) in &page.drawing.unlettered {
                match here.iter_mut().find(|h| h.0 == c) {
                    Some(h) => h.2 += 1,
                    None => here.push((c, plain, 1)),
                }
            }
            for h in here {
                match out.iter_mut().find(|o| o.0 == h.0) {
                    Some(o) => o.2 = o.2.max(h.2),
                    None => out.push(h),
                }
            }
        }
        out
    }

    /// What of the lettering could not be lettered as written, said: how many letters, and which.
    pub fn unlettered_said(&self) -> String {
        let u = self.unlettered();
        if u.is_empty() {
            return "unlettered: 0 letters".to_string();
        }
        let n: usize = u.iter().map(|x| x.2).sum();
        let which: Vec<String> = u.iter().map(|(c, p, _)| format!("{c} as {}", p.map_or("?".to_string(), |p| p.to_string()))).collect();
        format!("unlettered: {n} letters could not be lettered as written ({})", which.join(", "))
    }

    /// What is on each page, in words and figures, so a figure on paper can be checked without
    /// reading the picture.
    pub fn list(&self) -> String {
        use std::fmt::Write as _;
        let mut s = String::new();
        let _ = writeln!(s, "PLAN SET: {}, {}", self.title, self.level);
        let _ = writeln!(s, "{}, {} paper ({} by {} in), {}, {}", count(self.pages.len(), "page", "pages"), self.paper.name, crate::view::num(self.paper.width_in), crate::view::num(self.paper.height_in), size::scale_said(self.scale), self.date);
        let _ = writeln!(s, "{}", self.unlettered_said());
        for (show, why) in &self.not_written {
            let _ = writeln!(s, "not written: {}: {why}", show.name());
        }
        for (i, page) in self.pages.iter().enumerate() {
            let _ = writeln!(s, "\npage {} of {}: {}", i + 1, self.pages.len(), page.show.name());
            let _ = writeln!(s, "  sheet: {}, sheet {} of {}", page.sheet, page.position.0, page.position.1);
            let _ = writeln!(s, "  north arrow: plan north, up the page (true north is not set)");
            let _ = writeln!(s, "  key: {}", page.key.iter().map(|(name, w)| format!("{name}, {}", w.said())).collect::<Vec<_>>().join("; "));
            let _ = writeln!(s, "  graphic scale: {} bar, true at full size", if page.scale_bar.is_empty() { "none".to_string() } else { page.scale_bar.clone() });
            let _ = writeln!(s, "  overall: across {}, up the page {}", page.overall.across, page.overall.up);
            if let Some(u) = &page.under_title {
                let _ = writeln!(s, "  under the title: {u}");
            }
            let _ = writeln!(s, "  walls: {} ({} heavy, {} light, {} dashed)", page.walls.len(), page.walls.iter().filter(|w| w.weight == Weight::Heavy).count(), page.walls.iter().filter(|w| w.weight == Weight::Light).count(), page.walls.iter().filter(|w| w.weight == Weight::Dashed).count());
            for w in &page.walls {
                let _ = writeln!(s, "    {} {:?}: {}{}", w.path, w.name, w.weight.said(), if w.filled { ", filled" } else { "" });
            }
            let _ = writeln!(s, "  openings: {}", page.openings.len());
            for o in &page.openings {
                let _ = writeln!(s, "    {} in {}: {}, gap {}, {}{}", o.path, o.wall, if o.door { "door" } else { "window" }, lf(o.gap), o.weight.said(), if o.door { ", no swing" } else { "" });
            }
            let _ = writeln!(s, "  things set in place: {}", page.things.len());
            for t in &page.things {
                let _ = writeln!(s, "    {} {:?}: {}{}", t.path, t.name, t.weight.said(), if t.above_cut { ", above the cut, dashed" } else { "" });
            }
            let _ = writeln!(s, "  dimensions: {}", page.dimensions.len());
            for d in &page.dimensions {
                let _ = writeln!(s, "    {}  {}", d.wall, d.text);
            }
            let _ = writeln!(s, "  strings: {}", page.strings.len());
            for st in &page.strings {
                let parts: Vec<String> = st.parts.iter().map(|(t, v)| if *v { format!("{t} VIF") } else { t.clone() }).collect();
                let _ = writeln!(s, "    {}, {}: {} (adds to {})", st.place, st.kind, parts.join(" "), st.whole);
            }
            for (side, why) in &page.no_overall {
                let _ = writeln!(s, "    no overall on the {side} side: {why}");
            }
            let _ = writeln!(s, "  angles: {}", page.angles.len());
            for a in &page.angles {
                let _ = writeln!(s, "    {}: {}", a.corner, a.text);
            }
            let _ = writeln!(s, "  room lines: {}", page.room_lines);
            let _ = writeln!(s, "  rooms: {}", page.rooms.len());
            for r in &page.rooms {
                let _ = writeln!(s, "    {}", r.text);
            }
            let _ = writeln!(s, "  room schedule: {}", page.schedule.len());
            for r in &page.schedule {
                let _ = writeln!(s, "    {}  {} {}", r.mark, r.name, r.area);
            }
            let faults = page.lettering_faults();
            let smallest = page.drawing.letters.iter().map(|l| l.height).fold(f32::MAX, f32::min);
            let _ = writeln!(s, "  title box: {}", page.title_lines.join(" / "));
            let _ = writeln!(s, "  lettering: {} pieces, smallest {:.2} pt (3/32 in is 6.75 pt)", page.drawing.letters.len(), if page.drawing.letters.is_empty() { 0.0 } else { smallest });
            let _ = writeln!(s, "  lettering faults: {}", faults.len());
            for f in &faults {
                let _ = writeln!(s, "    {f}");
            }
            let _ = writeln!(s, "  notes:");
            for n in &page.notes {
                let _ = writeln!(s, "    {n}");
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare(d: Drawing) -> PlanPage {
        PlanPage { show: Show::Floor, walls: Vec::new(), openings: Vec::new(), things: Vec::new(), dimensions: Vec::new(), overall: DrawnOverall { across: String::new(), up: String::new(), vif: false }, strings: Vec::new(), no_overall: Vec::new(), angles: Vec::new(), room_lines: 0, scale_bar: String::new(), rooms: Vec::new(), schedule: Vec::new(), sheet: "A-101".into(), position: (1, 1), key: Vec::new(), frames: Vec::new(), title_lines: Vec::new(), under_title: None, notes: Vec::new(), drawing: d }
    }

    #[test]
    fn a_date_is_a_day_of_the_calendar_or_it_is_refused() {
        for good in ["2026-11-22", "2028-02-29", "11/22/2026", " 2026-12-31 "] {
            assert!(day(good).is_ok(), "{good}");
        }
        for bad in ["yesterday", "2026-13-45", "2026-02-30", "2027-02-29", "2100-02-29", "11/31/2026", "", "26-11-22", "2026-11", "0-0-0000"] {
            assert!(day(bad).is_err_and(|e| e.contains("is not a day of the calendar")), "{bad}");
        }
    }

    #[test]
    fn a_line_drawn_through_lettering_is_a_fault_and_a_line_beside_it_is_not() {
        // A sheet of 200 by 100 points: the lettering "12'-0\"" with a line through it; the same lettering with the line clear.
        let mut through = Drawing::new(200.0, 100.0);
        through.text("12'-0\"", (100.0, 50.0), 7.2, 0.0, INK);
        through.line(vec![(60.0, 50.0), (140.0, 50.0)], 0.5, INK, None, false);
        let faults = bare(through).lettering_faults();
        assert_eq!(faults.len(), 1, "{faults:?}");
        assert!(faults[0].contains("has a line drawn through it"), "{faults:?}");
        let mut beside = Drawing::new(200.0, 100.0);
        beside.text("12'-0\"", (100.0, 50.0), 7.2, 0.0, INK);
        beside.line(vec![(60.0, 70.0), (140.0, 70.0)], 0.5, INK, None, false);
        assert!(bare(beside).lettering_faults().is_empty());
        // Lettering on a cut wall is a fault too.
        let mut on_wall = Drawing::new(200.0, 100.0);
        on_wall.fill(vec![vec![(60.0, 40.0), (140.0, 40.0), (140.0, 60.0), (60.0, 60.0)]], POCHE);
        on_wall.text("BED", (100.0, 50.0), 9.0, 0.0, INK);
        assert!(bare(on_wall).lettering_faults().iter().any(|f| f.contains("lies on a cut wall")));
    }

    #[test]
    fn what_is_wrong_is_said_with_its_page_whether_or_not_the_list_is_asked_for() {
        let mut d = Drawing::new(200.0, 100.0);
        d.text("ISLAND", (100.0, 50.0), 7.2, 0.0, INK);
        d.text("1", (100.0, 52.0), 6.75, 0.0, INK);
        let set = PlanSet { title: "T".into(), level: "L".into(), paper: size::PAPERS[0], scale: (1, 4), date: "2026-11-22".into(), pages: vec![bare(d)], not_written: Vec::new() };
        let said = set.faults();
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].starts_with("page 1 of 1, FLOOR PLAN: the lettering "), "{said:?}");
    }

    #[test]
    fn letters_no_glyph_is_made_for_are_counted_by_page_and_the_file_carries_the_title() {
        let mut d = Drawing::new(200.0, 100.0);
        d.text("Łódź & Ω", (100.0, 50.0), 7.2, 0.0, INK);
        d.text_left("Ł", (10.0, 10.0), 7.2, INK);
        let set = PlanSet { title: "Henderson & Sons Résidence".into(), level: "Première étage".into(), paper: size::PAPERS[0], scale: (1, 4), date: "2026-11-22".into(), pages: vec![bare(d)], not_written: Vec::new() };
        assert_eq!(set.unlettered(), vec![('Ł', Some('L'), 2), ('Ź', Some('Z'), 1), ('Ω', None, 1)]);
        assert!(set.unlettered_said().starts_with("unlettered: 4 letters could not be lettered as written (Ł as L, Ź as Z, Ω as ?)"), "{}", set.unlettered_said());
        let bytes = set.pdf();
        let text: String = bytes.iter().map(|&b| b as char).collect();
        // "Henderson & Sons Résidence" in UTF-16 hex: the ampersand is 0026, the é is 00E9.
        assert!(text.contains("/Title <FEFF"), "{text}");
        assert!(text.contains("00200026002000530"), "the ampersand is in the file's title");
        assert!(text.contains("00E9"), "the é is in the file's title");
        assert!(text.contains("/Info "));
        // The same set, the same bytes.
        assert_eq!(bytes, set.pdf());
    }
}
