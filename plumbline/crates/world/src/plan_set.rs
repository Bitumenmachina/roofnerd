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

use crate::building::planset::{level_named, level_plan, LevelPlan, LevelRoom};
use crate::building::rooms;
use crate::building::walls::{self, PlanWall};
use crate::job::Job;
use crate::paper::letters::{self, Letter, MIN_CAP};
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

/// A room's label as it is lettered.
#[derive(Clone, Debug)]
pub struct RoomLabel {
    pub name: String,
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
    pub rooms: Vec<RoomLabel>,
    /// Rooms too small for their name and area at the printed scale: each a mark on the plan
    /// and a row of the room schedule at the side of the sheet (mark, then what it says).
    pub schedule: Vec<ScheduleRow>,
    /// The sheet's number, by the National CAD Standard's form: the discipline letter, a
    /// dash, the sheet type (1, plans) and the order in the set: A-101.
    pub sheet: String,
    /// The key: each kind of line this page draws, with how it is drawn.
    pub key: Vec<(String, Weight)>,
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
const DIM_OFF: f32 = 24.0;
/// A dimension's figures: over 3/32 in (`letters::MIN_CAP`).
const DIM_TEXT: f32 = 7.2;
/// The overall dimension's figures.
const OVERALL_TEXT: f32 = 9.0;
/// How far outside everything the plan has drawn on a side its overall dimension lies.
const OVERALL_GAP: f32 = 9.0;
const ROOM_NAME: f32 = 9.0;
const NOTE_TEXT: f32 = 6.75;
const NOTE_PITCH: f32 = 10.0;
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
    if plan.walls.is_empty() {
        let mut why = format!("{} has no wall on its plan to draw", plan.level);
        for (path, because) in &plan.walls_left_off {
            why.push_str(&format!("; {path}: {because}"));
        }
        return Err(why);
    }
    let tol = crate::check::FIT_TOLERANCE_FT;

    // Which pages, and why not the others.
    let any = |places: &[usize]| plan.walls.iter().any(|w| places.contains(&w.place)) || plan.openings.iter().any(|o| places.contains(&o.place));
    let marked = any(&[1, 2]);
    let mut shows: Vec<Show> = Vec::new();
    let mut not_written: Vec<(Show, String)> = Vec::new();
    if !marked {
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

    // What each page holds, as data.
    let mut pages: Vec<(PlanPage, Layer)> = Vec::new();
    for (i, show) in shows.iter().enumerate() {
        pages.push(page_data(&plan, *show, tol, i + 1));
    }

    // The scale and the paper: the plan with its dimensions and the notes fit the paper in
    // its margins.
    let date = opts.date.clone().unwrap_or_else(|| crate::job::now()[..10].to_string());
    let total = shows.len();
    let try_fit = |scale: (u32, u32), paper: size::Paper| -> Option<Vec<Laid>> { layout(&plan, &pages, scale, paper, &date, total) };
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
    for ((mut page, _), laid) in pages.into_iter().zip(laid) {
        page.drawing = laid.drawing;
        page.notes = laid.notes;
        page.schedule = laid.schedule;
        page.frames = laid.frames;
        page.title_lines = laid.title_lines;
        out.push(page);
    }
    Ok(PlanSet { title: plan.title.clone(), level: plan.level.clone(), paper: paper_used, scale, date, pages: out, not_written })
}

/// What a page draws, in feet: the rings of its walls (joined among the walls on the page),
/// and the rest of what it letters.
struct Layer {
    /// For each wall on the page: its place in the level's list, its weight, its outline, and
    /// its centre line's two ends.
    walls: Vec<(usize, Weight, Vec<V>, PlanWall)>,
    /// Openings: the wall (a place in `walls` above), its span, and its weight.
    openings: Vec<(usize, [f64; 2], bool, Weight)>,
    /// Which side of each wall is outward: a unit vector.
    outward: Vec<V>,
    rooms: Vec<(String, [f64; 2], String)>,
    /// Things set in place: the weight, the outline's corners, and whether it hangs above the cut.
    things: Vec<(Weight, Vec<V>, bool, String)>,
}

/// One page laid on the paper.
struct Laid {
    notes: Vec<String>,
    drawing: Drawing,
    schedule: Vec<ScheduleRow>,
    frames: Vec<[f32; 4]>,
    title_lines: Vec<String>,
}

fn page_data(plan: &LevelPlan, show: Show, tol: f64, number: usize) -> (PlanPage, Layer) {
    let on: Vec<usize> = (0..plan.walls.len()).filter(|&i| show.weight(plan.walls[i].place).is_some()).collect();
    let page_walls: Vec<PlanWall> = on.iter().map(|&i| plan.walls[i].wall.clone()).collect();
    let mut drawn = Vec::new();
    let mut layer = Layer { walls: Vec::new(), openings: Vec::new(), outward: Vec::new(), rooms: Vec::new(), things: Vec::new() };
    for (k, &i) in on.iter().enumerate() {
        let w = &plan.walls[i];
        let weight = show.weight(w.place).unwrap_or(Weight::Heavy);
        drawn.push(DrawnWall { path: w.path.clone(), name: w.name.clone(), weight, filled: weight == Weight::Heavy });
        layer.walls.push((i, weight, walls::outline(&page_walls, k, tol), w.wall.clone()));
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
        layer.openings.push((k, o.span, o.door, weight));
        openings.push(DrawnOpening { path: o.path.clone(), door: o.door, wall: plan.walls[o.wall].path.clone(), gap: o.span[1] - o.span[0], weight });
    }
    // Things set in place, drawn on the proposed plan (and a level with no mark) as outlines.
    let mut things = Vec::new();
    for t in &plan.things {
        let Some(weight) = show.thing_weight(t.place) else { continue };
        layer.things.push((weight, t.ring.to_vec(), t.above_cut, t.name.to_uppercase()));
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
            (None, _) => "? SF".to_string(),
        };
        layer.rooms.push((r.name.to_uppercase(), r.point, area.clone()));
        labels.push(RoomLabel { name: r.name.clone(), text: format!("{} {area}", r.name.to_uppercase()) });
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
    // The key: each kind of line this page draws, as this page draws it.
    let mut key = Vec::new();
    if show == Show::Floor {
        key.push(("NEW (NO MARK)".to_string(), Weight::Heavy));
    } else {
        for (place, name) in [(1usize, "EXISTING"), (2, "TO DEMOLISH"), (0, "NEW")] {
            if let Some(w) = show.weight(place) {
                key.push((name.to_string(), w));
            }
        }
    }
    let page = PlanPage { show, walls: drawn, openings, things, dimensions, overall, rooms: labels, schedule: Vec::new(), sheet: format!("A-1{number:02}"), key, frames: Vec::new(), title_lines: Vec::new(), under_title, notes: Vec::new(), drawing: Drawing::new(0.0, 0.0) };
    (page, layer)
}

/// The notes of a page, in words, before they are wrapped.
fn notes_of(plan: &LevelPlan, page: &PlanPage) -> Vec<String> {
    let mut notes = vec!["PLAN NORTH IS UP THE PAGE; TRUE NORTH IS NOT SET.".to_string(), "DOOR SWINGS ARE NOT SET. OPENINGS ARE NOT DIMENSIONED.".to_string(), "DIMENSIONS ARE TO THE WALLS' LINES AS DRAWN.".to_string()];
    notes.push("THIS SHEET DOES NOT SHOW PIPES, DUCTS, WIRING OR THE FRAME.".to_string());
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
        notes.push(format!("{} NOT KNOWN, LETTERED ?.", count(unknown, "LENGTH IS", "LENGTHS ARE")));
    }
    let vif_areas = page.rooms.iter().filter(|r| r.text.ends_with(" VIF")).count();
    if vif_areas > 0 {
        notes.push(format!("{} TO VERIFY IN FIELD (VIF).", count(vif_areas, "AREA IS", "AREAS ARE")));
    }
    notes.push(format!("WALLS ARE CUT {} ABOVE THE FLOOR. {}", lf(plan.cut), if page.show.as_it_stands() { "ROOMS ARE AS THEY STAND TODAY." } else { "ROOMS ARE AS THE WORK LEAVES THEM." }));
    if page.show.as_it_stands() && !plan.rooms_stand_missing.is_empty() {
        notes.push(format!("NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS: {}.", plan.rooms_stand_missing.join(", ")));
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
fn layout(plan: &LevelPlan, pages: &[(PlanPage, Layer)], scale: (u32, u32), paper: size::Paper, date: &str, total: usize) -> Option<Vec<Laid>> {
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
    for ((page, _), art) in pages.iter().zip(&arts) {
        let beside = if art.schedule.is_empty() { 0.0 } else { SCHEDULE_W + 10.0 };
        let notes_w = w - 2.0 * MARGIN - BOX_W - 14.0 - beside;
        let mut notes = notes_of(plan, page);
        if art.unnamed > 0 {
            notes.push(format!("{} DRAWN WITH NO NAME IN IT: TOO SMALL, OR HUNG OVER THE FLOOR.", count(art.unnamed, "THING IS", "THINGS ARE")));
        }
        let lines: Vec<String> = notes.iter().flat_map(|n| wrap(n, notes_w)).collect();
        strip = strip.max(lines.len() as f32 * NOTE_PITCH + 6.0);
        if !art.schedule.is_empty() {
            strip = strip.max((art.schedule.iter().map(|r| schedule_name_lines(r).len()).sum::<usize>() as f32 + 1.0) * NOTE_PITCH + 12.0);
        }
        all_notes.push(lines);
    }
    let (x0, y0) = (MARGIN, MARGIN + strip + 12.0);
    let (x1, y1) = (w - MARGIN, h - MARGIN - TOP_BAND);
    if bounds.0 > bounds.2 {
        return None;
    }
    if bounds.2 - bounds.0 > x1 - x0 || bounds.3 - bounds.1 > y1 - y0 {
        return None;
    }
    // Centred in what is left.
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
        // The top band: the name and what comes down at the left, the key and the arrow at the right.
        let arrow_x = w - MARGIN - ARROW_W;
        let key_x = arrow_x - 8.0 - KEY_W;
        let left_w = key_x - MARGIN - 10.0;
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
        let title_lines = title_box(&mut d, &mut frames, plan, page, scale, date, i + 1, total, (w - MARGIN - BOX_W, MARGIN));
        for (j, line) in all_notes[i].iter().enumerate() {
            d.text_left(line, (MARGIN, MARGIN + strip - 8.0 - j as f32 * NOTE_PITCH), NOTE_TEXT, INK);
        }
        if !art.schedule.is_empty() {
            schedule_box(&mut d, &mut frames, &art.schedule, (w - MARGIN - BOX_W - 10.0 - SCHEDULE_W, MARGIN), (SCHEDULE_W, strip));
        }
        out.push(Laid { notes: all_notes[i].clone(), drawing: d, schedule: art.schedule, frames, title_lines });
    }
    Some(out)
}

/// How many lines a schedule row's name takes: one, or two when it does not go on one.
fn schedule_name_lines(row: &ScheduleRow) -> Vec<String> {
    letters::lines_of(&row.name, NOTE_TEXT, SCHEDULE_W - 10.0 - 22.0 - SCHEDULE_AREA_W, 2)
}

/// The room schedule: the rooms too small to letter on the plan, a row each (mark, name on one
/// line or two, area), under the heading.
fn schedule_box(d: &mut Drawing, frames: &mut Vec<[f32; 4]>, rows: &[ScheduleRow], (x0, y0): (f32, f32), (bw, bh): (f32, f32)) {
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
            y -= NOTE_PITCH;
        }
    }
}

/// The key: a sample of each kind of line the page draws, in the pen it is drawn in on the page.
fn key_box(d: &mut Drawing, frames: &mut Vec<[f32; 4]>, page: &PlanPage, n: f64, (x0, y0): (f32, f32), (bw, bh): (f32, f32)) {
    let wide = pen::points(pen::annotation_mm(pen::BORDER));
    d.line(vec![(x0, y0), (x0 + bw, y0), (x0 + bw, y0 + bh), (x0, y0 + bh)], wide, INK, None, true);
    frames.push([x0, y0, x0 + bw, y0 + bh]);
    for (j, (label, weight)) in page.key.iter().enumerate() {
        let y = y0 + bh - 9.0 - j as f32 * 12.0;
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
            d.text_left(line, (x0 + 5.0, y1 - 13.0 - j as f32 * 11.0), 8.0, INK);
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

/// A page's plan in points from the level's origin: its walls (filled and inked), its
/// openings, what is set in place, its rooms and its dimensions; and the lettering of each,
/// none over another.
fn plan_prims(page: &PlanPage, layer: &Layer, k: f64, n: f64) -> Art {
    let mut out: Vec<Prim> = Vec::new();
    let fine = |mm: f64| pen::points(mm);
    let ink_of = |weight: Weight| ink_of(weight, n);
    let heavy_rings: Vec<Vec<V>> = layer.walls.iter().filter(|w| w.1 == Weight::Heavy).map(|w| w.2.clone()).filter(|r| r.len() >= 3).collect();
    // The cut walls filled, as one shape.
    let fills: Vec<Vec<(f32, f32)>> = cut::union(&heavy_rings).iter().map(|r| r.iter().map(|q| p(*q, k)).collect()).collect();
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
                let dash = if dashed { dash_for(run_len(&pts, closed), closed) } else { None };
                out.push(Prim::Line { pts, width, color, dash, closed, round: false });
            }
        }
    }
    // Doors and windows: the gap cleared to the paper, jambs, and the leaf or the glass.
    for (wi, [s0, s1], door, weight) in &layer.openings {
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
            out.push(Prim::Line { pts: vec![p(at(s, w.int), k), p(at(s, w.ext), k)], width: jamb_w, color, dash: None, closed: false, round: false });
        }
        let mid = (w.ext + w.int) / 2.0;
        let lines: Vec<f64> = if *door { vec![mid] } else { vec![w.int, mid, w.ext] };
        for off in lines {
            let pts = vec![p(at(*s0, off), k), p(at(*s1, off), k)];
            let dash = if dashed { dash_for(run_len(&pts, false), false) } else { None };
            out.push(Prim::Line { pts, width: fine(pen::model_mm(if *door { leaf } else { glass }, n)), color, dash, closed: false, round: false });
        }
    }
    // What is set in place: the outline of each footprint, in the pen of a door's leaf; dashed
    // where it hangs above the cut. Its name is lettered inside it where it fits there and no
    // line of another thing's outline runs through it; what stands on the floor is named, on a patch of paper where the dashed outline of
    // what hangs over it runs under the name; what hangs over the floor is not named (it
    // stands over what is), and the notes say how many things have no name.
    let mut lettering = Drawing::new(0.0, 0.0);
    let mut edges: Vec<((f32, f32), (f32, f32), bool)> = Vec::new();
    let mut rings_pts: Vec<Vec<(f32, f32)>> = Vec::new();
    for (weight, ring, above, _) in &layer.things {
        let pts: Vec<(f32, f32)> = ring.iter().map(|q| p(*q, k)).collect();
        let color = if *weight == Weight::Light { FAINT } else { INK };
        let dash = if *above { dash_for(run_len(&pts, true), true) } else { None };
        out.push(Prim::Line { pts: pts.clone(), width: fine(pen::model_mm(pen::MEDIUM, n)), color, dash, closed: true, round: false });
        edges.extend((0..pts.len()).map(|i| (pts[i], pts[(i + 1) % pts.len()], *above)));
        rings_pts.push(pts);
    }
    let mut unnamed = 0usize;
    let mut by_height: Vec<usize> = (0..layer.things.len()).collect();
    by_height.sort_by_key(|&i| layer.things[i].2);
    for i in by_height {
        let (_, _, _, name) = &layer.things[i];
        let pts = &rings_pts[i];
        // Along its first side, level when that side is near level, else turned to read from below.
        let side = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).hypot(b.1 - a.1), (b.1 - a.1).atan2(b.0 - a.0));
        let ((len, mut angle), (wid, _)) = (side(pts[0], pts[1]), side(pts[1], pts[2]));
        if angle > std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle -= std::f32::consts::PI;
        } else if angle <= -std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle += std::f32::consts::PI;
        }
        let c = ((pts[0].0 + pts[2].0) / 2.0, (pts[0].1 + pts[2].1) / 2.0);
        // What hangs over the floor stands over what is named below it, and is not named.
        let hgt = if layer.things[i].2 { None } else { [DIM_TEXT, MIN_CAP].into_iter().find(|h| font::width(name, *h) + 4.0 <= len && h + 4.0 <= wid) };
        let named = hgt.is_some_and(|h| {
            let l = Letter::centred(name, c, h, angle);
            !edges.iter().any(|&(a, b, hung)| !hung && l.touches_segment(a, b)) && !lettering.letters.iter().any(|o| l.crosses(o))
        });
        match (named, hgt) {
            (true, Some(h)) => {
                // The name is set on a patch of paper: a dashed outline of what hangs over this runs under it.
                let l = Letter::centred(name, c, h, angle);
                out.push(Prim::Fill { rings: vec![l.corners().to_vec()], color: PAPER });
                lettering.text(name, c, h, angle, INK);
            }
            _ => unnamed += 1,
        }
    }
    let mut placed: Vec<Letter> = lettering.letters.clone();
    out.extend(lettering.prims);
    // Rooms: the name over the area at the room's point; where they do not fit there at their
    // height (3/32 in at the least) or would lie over lettering, a mark and a row of the schedule.
    let rings: Vec<Vec<V>> = layer.walls.iter().map(|w| w.2.clone()).collect();
    let mut schedule: Vec<ScheduleRow> = Vec::new();
    for (name, point, area) in &layer.rooms {
        let c = p(V(point[0], point[1]), k);
        let mut tried = None;
        if let Some((hn, ha)) = room_fit(&rings, V(point[0], point[1]), k, name, area) {
            let mut d = Drawing::new(0.0, 0.0);
            d.text(name, (c.0, c.1 + hn / 2.0 + 2.5), hn, 0.0, INK);
            d.text(area, (c.0, c.1 - ha / 2.0 - 2.5), ha, 0.0, INK);
            if !d.letters.iter().any(|l| placed.iter().any(|o| l.crosses(o))) {
                tried = Some(d);
            }
        }
        match tried {
            Some(d) => {
                placed.extend(d.letters.clone());
                out.extend(d.prims);
            }
            None => {
                let mark = (schedule.len() + 1).to_string();
                let mut d = Drawing::new(0.0, 0.0);
                d.text(&mark, c, MIN_CAP, 0.0, INK);
                let circle: Vec<(f32, f32)> = (0..24).map(|i| (c.0 + 8.0 * (i as f32 * std::f32::consts::TAU / 24.0).cos(), c.1 + 8.0 * (i as f32 * std::f32::consts::TAU / 24.0).sin())).collect();
                d.line(circle, pen::points(pen::annotation_mm(pen::PROJECTION)), INK, None, true);
                placed.extend(d.letters.clone());
                out.extend(d.prims);
                schedule.push(ScheduleRow { mark, name: name.clone(), area: area.clone() });
            }
        }
    }
    // Dimensions: one string along each wall, on its outer side; its figure over the middle
    // of the string, or, where that is too short or taken, beyond its end or stood off.
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    let heavy = pen::points(pen::annotation_mm(pen::HEAVY));
    let tick_at = |out: &mut Vec<Prim>, s: (f32, f32), t: (f32, f32)| {
        let slash = (t.0 * 0.7071 + -t.1 * 0.7071, t.1 * 0.7071 + t.0 * 0.7071);
        out.push(Prim::Line { pts: vec![(s.0 - slash.0 * 3.5, s.1 - slash.1 * 3.5), (s.0 + slash.0 * 3.5, s.1 + slash.1 * 3.5)], width: heavy, color: INK, dash: None, closed: false, round: false });
    };
    let mut string_ends: Vec<(f32, f32)> = Vec::new();
    for (((_, _, _, w), dim), out_n) in layer.walls.iter().zip(&page.dimensions).zip(&layer.outward) {
        let t = (w.end - w.start).unit();
        let face = if out_n.dot(t.perp()) > 0.0 { w.ext.max(w.int) } else { -w.ext.min(w.int) };
        let nrm = (out_n.0 as f32, out_n.1 as f32);
        let base = |q: V| -> (f32, f32) {
            let c = p(q, k);
            (c.0 + nrm.0 * (face * k) as f32, c.1 + nrm.1 * (face * k) as f32)
        };
        let beyond = |b: (f32, f32), by: f32| (b.0 + nrm.0 * by, b.1 + nrm.1 * by);
        let (a, b) = (base(w.start), base(w.end));
        let (sa, sb) = (beyond(a, DIM_OFF), beyond(b, DIM_OFF));
        string_ends.extend([sa, sb]);
        out.push(Prim::Line { pts: vec![sa, sb], width: thin, color: INK, dash: None, closed: false, round: false });
        // Extension lines from just off the wall to a little past the string, and a tick at each end.
        let tick = (t.0 as f32, t.1 as f32);
        for (e, s) in [(a, sa), (b, sb)] {
            out.push(Prim::Line { pts: vec![beyond(e, 3.0), beyond(s, 3.0)], width: thin, color: INK, dash: None, closed: false, round: false });
            tick_at(&mut out, s, tick);
        }
        // Its length, turned to read from below or from the right.
        let mut angle = (t.1).atan2(t.0) as f32;
        if angle > std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle -= std::f32::consts::PI;
        } else if angle <= -std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle += std::f32::consts::PI;
        }
        let mid = ((sa.0 + sb.0) / 2.0, (sa.1 + sb.1) / 2.0);
        let len = (sb.0 - sa.0).hypot(sb.1 - sa.1);
        let mut chosen: Option<Letter> = None;
        let mut first: Option<Letter> = None;
        'search: for h in [DIM_TEXT, MIN_CAP] {
            let text_w = font::width(&dim.text, h);
            let off = 3.0 + h / 2.0 + 1.0;
            let mut spots: Vec<(f32, f32)> = Vec::new();
            if text_w + 6.0 <= len {
                spots.push(beyond(mid, off));
            }
            // Too short for it, or taken: past either end on the string's own line, then stood
            // off from the string a line of lettering at a time.
            let along = |s: (f32, f32), by: f32| (s.0 + tick.0 * by, s.1 + tick.1 * by);
            spots.push(along(sb, text_w / 2.0 + 6.0));
            spots.push(along(sa, -(text_w / 2.0 + 6.0)));
            for step in 1..=2 {
                spots.push(beyond(mid, off + step as f32 * (h + 2.0)));
            }
            for c in spots {
                let cand = Letter::centred(&dim.text, c, h, angle);
                first.get_or_insert_with(|| cand.clone());
                if !placed.iter().any(|o| cand.crosses(o)) {
                    chosen = Some(cand);
                    break 'search;
                }
            }
        }
        // None is free: it goes where it first would, and the page's lettering check says so.
        let cand = chosen.or(first).unwrap_or_else(|| Letter::centred(&dim.text, mid, DIM_TEXT, angle));
        let mut d = Drawing::new(0.0, 0.0);
        d.text(&dim.text, cand.centre, cand.height, angle, INK);
        placed.extend(d.letters.clone());
        out.extend(d.prims);
    }
    // The overall dimension on each side, outside everything drawn so far, with extension
    // lines from the nearest string's end: the extent of the walls' lines as drawn.
    let ends: Vec<(f32, f32)> = layer.walls.iter().flat_map(|w| [p(w.3.start, k), p(w.3.end, k)]).collect();
    if !ends.is_empty() {
        let (xlo, xhi) = ends.iter().fold((f32::MAX, f32::MIN), |a, e| (a.0.min(e.0), a.1.max(e.0)));
        let (ylo, yhi) = ends.iter().fold((f32::MAX, f32::MIN), |a, e| (a.0.min(e.1), a.1.max(e.1)));
        let mut drawn_box = extent(&out);
        for l in &placed {
            for (x, y) in l.corners() {
                drawn_box = (drawn_box.0.min(x), drawn_box.1.min(y), drawn_box.2.max(x), drawn_box.3.max(y));
            }
        }
        let vif = if page.overall.vif { " VIF" } else { "" };
        let (across, up) = (format!("{}{vif}", page.overall.across), format!("{}{vif}", page.overall.up));
        let near = |a: f32, b: f32| (a - b).abs() < 1.0;
        // Each side: where its string lies, the two ends it measures between, and which way is out.
        // (kind, line position, from, to, direction out, text)
        let sides: [(bool, f32, f32, f32, f32, &str); 4] = [
            (true, drawn_box.3 + OVERALL_GAP, xlo, xhi, 1.0, &across),
            (true, drawn_box.1 - OVERALL_GAP, xlo, xhi, -1.0, &across),
            (false, drawn_box.0 - OVERALL_GAP, ylo, yhi, -1.0, &up),
            (false, drawn_box.2 + OVERALL_GAP, ylo, yhi, 1.0, &up),
        ];
        for (horizontal, at, from, to, dir, text) in sides {
            let pt = |s: f32, along: f32| if horizontal { (along, s) } else { (s, along) };
            let (pa, pb) = (pt(at, from), pt(at, to));
            out.push(Prim::Line { pts: vec![pa, pb], width: thin, color: INK, dash: None, closed: false, round: false });
            // Extension lines run out from the string end that stands at each end of the extent,
            // on this side; where none does, from the plan's own corner.
            for along in [from, to] {
                let on_side = |e: &&(f32, f32)| {
                    let (a_ok, s_ok) = if horizontal { (near(e.0, along), (e.1 - if dir > 0.0 { yhi } else { ylo }) * dir > -0.5) } else { (near(e.1, along), (e.0 - if dir > 0.0 { xhi } else { xlo }) * dir > -0.5) };
                    a_ok && s_ok
                };
                let from_s = string_ends.iter().filter(on_side).map(|e| if horizontal { e.1 } else { e.0 }).fold(None::<f32>, |m, v| Some(m.map_or(v, |m| if dir > 0.0 { m.max(v) } else { m.min(v) })));
                let start = from_s.unwrap_or(if horizontal { if dir > 0.0 { yhi } else { ylo } } else if dir > 0.0 { xhi } else { xlo });
                // From a little past the string's end to a little past the overall string.
                let (s0, s1) = (start + dir * 3.0, at + dir * 3.0);
                out.push(Prim::Line { pts: vec![pt(s0, along), pt(s1, along)], width: thin, color: INK, dash: None, closed: false, round: false });
            }
            let tick = if horizontal { (1.0, 0.0) } else { (0.0, 1.0) };
            tick_at(&mut out, pa, tick);
            tick_at(&mut out, pb, tick);
            let mid = (from + to) / 2.0;
            let off = dir * (3.0 + OVERALL_TEXT / 2.0 + 1.0);
            let c = pt(at + off, mid);
            let mut d = Drawing::new(0.0, 0.0);
            d.text(text, c, OVERALL_TEXT, if horizontal { 0.0 } else { std::f32::consts::FRAC_PI_2 }, INK);
            placed.extend(d.letters.clone());
            out.extend(d.prims);
        }
    }
    let mut d = Drawing::new(0.0, 0.0);
    d.prims = out;
    d.letters = placed;
    Art { d, schedule, unnamed }
}

impl PlanPage {
    /// What is wrong with the page's lettering, each said: a piece under 3/32 in, a piece
    /// lying over another, a piece outside the sheet's frame, a piece across the edge of a
    /// box (the title box, the key, the schedule). Measured on the lettering as it is written
    /// on the page, after it is laid.
    pub fn lettering_faults(&self) -> Vec<String> {
        let mut out = Vec::new();
        let frame = [BORDER_AT, BORDER_AT, self.drawing.width - BORDER_AT, self.drawing.height - BORDER_AT];
        let ls = &self.drawing.letters;
        for (i, l) in ls.iter().enumerate() {
            if l.height < MIN_CAP - 1e-3 {
                out.push(format!("{:?} is {:.2} pt high, under 3/32 in", l.text, l.height));
            }
            if !l.within(frame) {
                out.push(format!("{:?} is outside the sheet's frame", l.text));
            }
            for fr in &self.frames {
                let inside = l.corners().iter().filter(|&&(x, y)| x >= fr[0] && x <= fr[2] && y >= fr[1] && y <= fr[3]).count();
                if inside != 0 && inside != 4 {
                    out.push(format!("{:?} is across the edge of a box", l.text));
                }
            }
            for o in &ls[i + 1..] {
                if l.crosses(o) {
                    out.push(format!("{:?} is over {:?}", l.text, o.text));
                }
            }
        }
        out
    }
}

impl PlanSet {
    /// The set as a PDF: a page of line work for each, the same bytes for the same set.
    pub fn pdf(&self) -> Vec<u8> {
        let drawings: Vec<Drawing> = self.pages.iter().map(|p| p.drawing.clone()).collect();
        paper::pdf::pdf(&drawings)
    }

    /// What is on each page, in words and figures, so a figure on paper can be checked without
    /// reading the picture.
    pub fn list(&self) -> String {
        use std::fmt::Write as _;
        let mut s = String::new();
        let _ = writeln!(s, "PLAN SET: {}, {}", self.title, self.level);
        let _ = writeln!(s, "{}, {} paper ({} by {} in), {}, {}", count(self.pages.len(), "page", "pages"), self.paper.name, crate::view::num(self.paper.width_in), crate::view::num(self.paper.height_in), size::scale_said(self.scale), self.date);
        for (show, why) in &self.not_written {
            let _ = writeln!(s, "not written: {}: {why}", show.name());
        }
        for (i, page) in self.pages.iter().enumerate() {
            let _ = writeln!(s, "\npage {} of {}: {}", i + 1, self.pages.len(), page.show.name());
            let _ = writeln!(s, "  sheet: {}, sheet {} of {}", page.sheet, i + 1, self.pages.len());
            let _ = writeln!(s, "  north arrow: plan north, up the page (true north is not set)");
            let _ = writeln!(s, "  key: {}", page.key.iter().map(|(name, w)| format!("{name}, {}", w.said())).collect::<Vec<_>>().join("; "));
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
