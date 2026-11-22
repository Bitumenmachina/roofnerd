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

/// A dimension as it is lettered, on the wall it measures.
#[derive(Clone, Debug)]
pub struct DrawnDimension {
    pub wall: String,
    pub text: String,
    /// Whether it carries VIF, and whether its length is not known at all.
    pub vif: bool,
    pub unknown: bool,
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
    pub dimensions: Vec<DrawnDimension>,
    pub rooms: Vec<RoomLabel>,
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
const DIM_TEXT: f32 = 7.2;
const ROOM_NAME: f32 = 9.0;
const NOTE_TEXT: f32 = 6.75;
const NOTE_PITCH: f32 = 10.0;
const TITLE_TEXT: f32 = 12.0;
const BOX_W: f32 = 252.0;
const BOX_H: f32 = 90.0;

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
    for show in &shows {
        pages.push(page_data(&plan, *show, tol));
    }

    // The scale and the paper: the plan with its dimensions and the notes fit the paper in
    // its margins.
    let date = opts.date.clone().unwrap_or_else(|| crate::job::now()[..10].to_string());
    let total = shows.len();
    let try_fit = |scale: (u32, u32), paper: size::Paper| -> Option<Vec<(Vec<String>, Vec<Prim>)>> { layout(&plan, &pages, scale, paper, &date, total) };
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
    for ((mut page, _), (notes, prims)) in pages.into_iter().zip(laid) {
        let (w, h) = paper_used.points();
        page.drawing = Drawing { width: w as f32, height: h as f32, prims };
        page.notes = notes;
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
}

fn page_data(plan: &LevelPlan, show: Show, tol: f64) -> (PlanPage, Layer) {
    let on: Vec<usize> = (0..plan.walls.len()).filter(|&i| show.weight(plan.walls[i].place).is_some()).collect();
    let page_walls: Vec<PlanWall> = on.iter().map(|&i| plan.walls[i].wall.clone()).collect();
    let mut drawn = Vec::new();
    let mut layer = Layer { walls: Vec::new(), openings: Vec::new(), outward: Vec::new(), rooms: Vec::new() };
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
        // An opening is drawn as light as the wall it is in unless it comes down itself.
        let weight = match (show, o.place) {
            (Show::Demolition, 2) => Weight::Dashed,
            _ => layer.walls[k].1,
        };
        layer.openings.push((k, o.span, o.door, weight));
        openings.push(DrawnOpening { path: o.path.clone(), door: o.door, wall: plan.walls[o.wall].path.clone(), gap: o.span[1] - o.span[0], weight });
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
    let page = PlanPage { show, walls: drawn, openings, dimensions, rooms: labels, under_title, notes: Vec::new(), drawing: Drawing::new(0.0, 0.0) };
    (page, layer)
}

/// The notes of a page, in words, before they are wrapped.
fn notes_of(plan: &LevelPlan, page: &PlanPage) -> Vec<String> {
    let mut notes = vec!["PLAN NORTH IS UP THE PAGE; TRUE NORTH IS NOT SET.".to_string(), "DOOR SWINGS ARE NOT SET.".to_string(), "DIMENSIONS ARE TO THE WALLS' LINES AS DRAWN.".to_string(), "OPENINGS ARE NOT DIMENSIONED.".to_string()];
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
    notes.push(format!("WALLS ARE CUT {} ABOVE THE FLOOR.", lf(plan.cut)));
    notes.push(if page.show.as_it_stands() { "ROOMS ARE AS THEY STAND TODAY.".to_string() } else { "ROOMS ARE AS THE WORK LEAVES THEM.".to_string() });
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
fn layout(plan: &LevelPlan, pages: &[(PlanPage, Layer)], scale: (u32, u32), paper: size::Paper, date: &str, total: usize) -> Option<Vec<(Vec<String>, Vec<Prim>)>> {
    let (w, h) = paper.points();
    let (w, h) = (w as f32, h as f32);
    let k = 72.0 * size::inches_to_the_foot(scale);
    let n = size::ratio(scale);
    // Every page's plan layer, in points from the level's origin, and the box that holds all of them.
    let mut layers: Vec<Vec<Prim>> = Vec::new();
    let mut bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (page, layer) in pages {
        let prims = plan_prims(page, layer, k as f64, n);
        for pr in &prims {
            let pts: Vec<(f32, f32)> = match pr {
                Prim::Fill { rings, .. } => rings.iter().flatten().copied().collect(),
                Prim::Line { pts, .. } => pts.clone(),
            };
            for (x, y) in pts {
                bounds = (bounds.0.min(x), bounds.1.min(y), bounds.2.max(x), bounds.3.max(y));
            }
        }
        layers.push(prims);
    }
    // The notes, wrapped to the room beside the title box; the strip along the bottom.
    let notes_w = w - 2.0 * MARGIN - BOX_W - 14.0;
    let mut all_notes: Vec<Vec<String>> = Vec::new();
    let mut strip = BOX_H;
    for (page, _) in pages {
        let lines: Vec<String> = notes_of(plan, page).iter().flat_map(|n| wrap(n, notes_w)).collect();
        strip = strip.max(lines.len() as f32 * NOTE_PITCH + 6.0);
        all_notes.push(lines);
    }
    let (x0, y0) = (MARGIN, MARGIN + strip + 12.0);
    let (x1, y1) = (w - MARGIN, h - MARGIN - 36.0);
    if bounds.0 > bounds.2 {
        return None;
    }
    if bounds.2 - bounds.0 > x1 - x0 || bounds.3 - bounds.1 > y1 - y0 {
        return None;
    }
    // Centred in what is left.
    let (dx, dy) = ((x0 + x1) / 2.0 - (bounds.0 + bounds.2) / 2.0, (y0 + y1) / 2.0 - (bounds.1 + bounds.3) / 2.0);
    let mut out = Vec::new();
    for (i, ((page, _), plan_layer)) in pages.iter().zip(layers).enumerate() {
        let mut d = Drawing::new(w, h);
        // The border, then the plan, then the page's name and its title box and notes.
        d.line(vec![(BORDER_AT, BORDER_AT), (w - BORDER_AT, BORDER_AT), (w - BORDER_AT, h - BORDER_AT), (BORDER_AT, h - BORDER_AT)], pen::points(pen::annotation_mm(pen::BORDER)), INK, None, true);
        for pr in plan_layer {
            d.prims.push(shifted(pr, dx, dy));
        }
        d.text_left(page.show.name(), (MARGIN, h - MARGIN - TITLE_TEXT), TITLE_TEXT, INK);
        if let Some(under) = &page.under_title {
            d.text_left(under, (MARGIN, h - MARGIN - TITLE_TEXT - 4.0 - DIM_TEXT - 6.0), DIM_TEXT, INK);
        }
        title_box(&mut d, plan, page, scale, date, i + 1, total, (w - MARGIN - BOX_W, MARGIN));
        for (j, line) in all_notes[i].iter().enumerate() {
            d.text_left(line, (MARGIN, MARGIN + strip - 8.0 - j as f32 * NOTE_PITCH), NOTE_TEXT, INK);
        }
        out.push((all_notes[i].clone(), d.prims));
    }
    Some(out)
}

fn shifted(pr: Prim, dx: f32, dy: f32) -> Prim {
    let mv = |pts: Vec<(f32, f32)>| -> Vec<(f32, f32)> { pts.into_iter().map(|(x, y)| (x + dx, y + dy)).collect() };
    match pr {
        Prim::Fill { rings, color } => Prim::Fill { rings: rings.into_iter().map(mv).collect(), color },
        Prim::Line { pts, width, color, dash, closed, round } => Prim::Line { pts: mv(pts), width, color, dash, closed, round },
    }
}

/// The title box: the job's title, the page, the level, the scale, the date and its place in
/// the set. Ported from BIMCraft's built-in title box (`sheet.rs:167`, `title_block`): a frame
/// in the border's pen, a rule under the heading, text in the sheet's pens; its rows are ours
/// (BIMCraft's are the project, the sheet's name, the scale and its number).
#[allow(clippy::too_many_arguments)]
fn title_box(d: &mut Drawing, plan: &LevelPlan, page: &PlanPage, scale: (u32, u32), date: &str, number: usize, total: usize, at: (f32, f32)) {
    let (x0, y0) = at;
    let (x1, y1) = (x0 + BOX_W, y0 + BOX_H);
    let wide = pen::points(pen::annotation_mm(pen::BORDER));
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    d.line(vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)], wide, INK, None, true);
    // Rows from the top: the job, the page, the level, the scale and date, its place.
    d.line(vec![(x0, y1 - 28.0), (x1, y1 - 28.0)], thin, INK, None, false);
    let fit = |text: &str, wanted: f32| -> f32 {
        let room = BOX_W - 10.0;
        let w = font::width(&text.to_uppercase(), wanted);
        if w <= room {
            wanted
        } else {
            (wanted * room / w).max(4.0)
        }
    };
    let title = if plan.title.is_empty() { "UNTITLED JOB".to_string() } else { plan.title.to_uppercase() };
    d.text_left(&title, (x0 + 5.0, y1 - 20.0), fit(&title, 10.0), INK);
    let name = page.show.name();
    d.text_left(name, (x0 + 5.0, y1 - 28.0 - 12.0), fit(name, 8.0), INK);
    let level = format!("LEVEL: {}", plan.level.to_uppercase());
    d.text_left(&level, (x0 + 5.0, y1 - 28.0 - 26.0), fit(&level, 7.2), INK);
    let sc = format!("SCALE: {}", size::scale_said(scale));
    d.text_left(&sc, (x0 + 5.0, y1 - 28.0 - 40.0), fit(&sc, 7.2), INK);
    let dt = format!("DATE: {date}   PAGE {number} OF {total}");
    d.text_left(&dt, (x0 + 5.0, y1 - 28.0 - 54.0), fit(&dt, 7.2), INK);
}

/// A page's plan in points from the level's origin: its walls (filled and inked), its
/// openings, its rooms and its dimensions.
fn plan_prims(page: &PlanPage, layer: &Layer, k: f64, n: f64) -> Vec<Prim> {
    let mut out: Vec<Prim> = Vec::new();
    let fine = |mm: f64| pen::points(mm);
    let ink_of = |weight: Weight| -> (f32, paper::Rgb, bool) {
        match weight {
            Weight::Heavy => (fine(pen::model_mm(pen::CUT, n)), INK, false),
            Weight::Light => (fine(pen::model_mm(pen::PROJECTION, n)), FAINT, false),
            Weight::Dashed => (fine(pen::model_mm(pen::PROJECTION, n)), INK, true),
        }
    };
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
    // Rooms: the name over the area, at the room's point.
    for (name, point, area) in &layer.rooms {
        let c = p(V(point[0], point[1]), k);
        let mut d = Drawing::new(0.0, 0.0);
        d.text(name, (c.0, c.1 + 6.0), ROOM_NAME, 0.0, INK);
        d.text(area, (c.0, c.1 - 6.0), DIM_TEXT, 0.0, INK);
        out.extend(d.prims);
    }
    // Dimensions: one string along each wall, on its outer side.
    let thin = pen::points(pen::annotation_mm(pen::PROJECTION));
    let heavy = pen::points(pen::annotation_mm(pen::HEAVY));
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
        out.push(Prim::Line { pts: vec![sa, sb], width: thin, color: INK, dash: None, closed: false, round: false });
        // Extension lines from just off the wall to a little past the string, and a tick at each end.
        for (e, s) in [(a, sa), (b, sb)] {
            out.push(Prim::Line { pts: vec![beyond(e, 3.0), beyond(s, 3.0)], width: thin, color: INK, dash: None, closed: false, round: false });
            let tick = (t.0 as f32, t.1 as f32);
            let slash = (tick.0 * 0.7071 + -tick.1 * 0.7071, tick.1 * 0.7071 + tick.0 * 0.7071);
            out.push(Prim::Line { pts: vec![(s.0 - slash.0 * 3.5, s.1 - slash.1 * 3.5), (s.0 + slash.0 * 3.5, s.1 + slash.1 * 3.5)], width: heavy, color: INK, dash: None, closed: false, round: false });
        }
        // Its length, turned to read from below or from the right, on the outer side of its string.
        let mut angle = (t.1).atan2(t.0) as f32;
        if angle > std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle -= std::f32::consts::PI;
        } else if angle <= -std::f32::consts::FRAC_PI_2 + 1e-4 {
            angle += std::f32::consts::PI;
        }
        let mid = ((sa.0 + sb.0) / 2.0, (sa.1 + sb.1) / 2.0);
        let c = beyond(mid, 3.0 + DIM_TEXT / 2.0 + 1.0);
        let mut d = Drawing::new(0.0, 0.0);
        d.text(&dim.text, c, DIM_TEXT, angle, INK);
        out.extend(d.prims);
    }
    out
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
            let _ = writeln!(s, "  dimensions: {}", page.dimensions.len());
            for d in &page.dimensions {
                let _ = writeln!(s, "    {}  {}", d.wall, d.text);
            }
            let _ = writeln!(s, "  rooms: {}", page.rooms.len());
            for r in &page.rooms {
                let _ = writeln!(s, "    {}", r.text);
            }
            let _ = writeln!(s, "  notes:");
            for n in &page.notes {
                let _ = writeln!(s, "    {n}");
            }
        }
        s
    }
}
