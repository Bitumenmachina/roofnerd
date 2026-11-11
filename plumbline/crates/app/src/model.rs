//! The open job, and one reading of it that every window draws from.
//!
//! `Doc` holds the job world and what the estimator is doing to it. `Snapshot` is read
//! from the world once after each change; the 3D scene, the panels and the sheet all
//! draw the same snapshot, so they cannot disagree. Nothing here works out a quantity.

use std::path::PathBuf;

use bevy::prelude::*;
use plumbline_world::building::{self, Placement, StoreyTotals, SystemTakeoff};
use plumbline_world::value::Status;
use plumbline_world::{roof, view, Condition, Discipline, Job, JobInfo, Kind, Measure, Pitch, Run, Solved};

/// The source written on a value set in the 3D view.
pub const SOURCE: &str = "set in the 3D view; not yet confirmed against a drawing";

#[derive(Clone, Copy, PartialEq)]
pub enum Entry {
    Pitch,
    Move,
    /// A design-phase rate on the selected element, in dollars per SF or LF.
    Allowance,
    /// The price of the selected element's cost line.
    UnitCost,
    /// What the selected element is made of: a material's name.
    Material,
    /// The job's overhead, a percentage.
    Overhead,
    /// The job's profit, a percentage.
    Profit,
    /// What a drawing tool draws with (a wall's thickness and height), typed once.
    Settings(crate::tools::Tool),
    /// A length typed while a wall or a run is being drawn.
    Length,
    /// A new storey: its name, its floor's height, and how far up to the next.
    Storey,
    /// A plan to lay under the storey on show: its file, and its page.
    Drawing,
    /// The distance between the two points picked on a drawing: its scale.
    Distance,
    /// What was measured on site for the value picked in the panel on the right.
    Measure,
}

/// How the last action went. Said by what happened, not read back out of the words.
#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Done,
    /// Done, with something the estimator has to know about.
    Warned,
    Refused,
}

/// Which table the sheet window is on.
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Roof,
    Building,
    Of(Discipline),
    /// The job priced from its allowances and from its cost lines, and the two compared.
    Money,
    /// The estimate sheet: every cost line in its column, the recap, and the suppliers.
    Estimate,
    /// What is in the same space as what, and what passes through a wall or a floor.
    Clashes,
}

/// What of the building is on show.
#[derive(Clone, PartialEq)]
pub struct Show {
    pub architecture: bool,
    pub roof: bool,
    pub mechanical: bool,
    pub electrical: bool,
    pub plumbing: bool,
    /// The building drawn as glass, so what runs inside it can be seen.
    pub ghost: bool,
    /// One storey alone, by path, with everything above it lifted off.
    pub storey: Option<String>,
}

impl Show {
    pub fn discipline(&self, d: Discipline) -> bool {
        match d {
            Discipline::Architecture => self.architecture,
            Discipline::Mechanical => self.mechanical,
            Discipline::Electrical => self.electrical,
            Discipline::Plumbing => self.plumbing,
        }
    }
    pub fn toggle(&mut self, d: Discipline) {
        match d {
            Discipline::Architecture => self.architecture = !self.architecture,
            Discipline::Mechanical => self.mechanical = !self.mechanical,
            Discipline::Electrical => self.electrical = !self.electrical,
            Discipline::Plumbing => self.plumbing = !self.plumbing,
        }
    }
}

#[derive(Resource)]
pub struct Doc {
    pub job: Job,
    pub path: PathBuf,
    /// The roof on show.
    pub roof: Option<String>,
    /// The selected element, by path.
    pub selected: Option<String>,
    /// The element the pointer is over.
    pub hover: Option<String>,
    /// What the last action did, or why it was refused.
    pub message: String,
    pub tone: Tone,
    /// Goes up on every change, so the panels know to draw again.
    pub revision: u64,
    /// Goes up when what is drawn in space changes: an edit, or what is on show.
    pub geometry: u64,
    /// A value being typed: which field, and the text so far.
    pub entry: Option<(Entry, String)>,
    /// When a unit cost is being typed: the cost line it is for, by its description.
    pub entry_line: Option<String>,
    /// When a measurement is being typed: the element, the value's name as the panel
    /// shows it, and whether it is one length.
    pub measuring: Option<(String, String, bool)>,
    /// The plan point the model's origin is drawn at, in feet. The view is drawn relative
    /// to it in 32-bit numbers; the model itself stays 64-bit.
    pub origin: [f64; 3],
    pub framed: bool,
    /// Where the pointer is on the model, in plan feet and height.
    pub pointer: Option<[f64; 3]>,
    pub shots: u32,
    /// Changed since it was last saved.
    pub edited: bool,
    /// Looking straight down, as a plan is drawn.
    pub plan: bool,
    /// On foot inside the model, at eye height.
    pub walk: bool,
    pub show: Show,
    pub tab: Tab,
    /// The drawing in hand, and the undo history.
    pub draw: crate::tools::Draw,
}

impl Doc {
    /// The window changed the job: it is no longer what was saved, and what was done is
    /// a step that can be taken back.
    pub fn edit(&mut self, what: &str) {
        self.edited = true;
        self.draw.edits += 1;
        self.draw.label = what.to_string();
    }
    /// The panels have something new to show.
    pub fn touch(&mut self) {
        self.revision += 1;
    }
    /// What is drawn in space has changed too.
    pub fn redraw(&mut self) {
        self.revision += 1;
        self.geometry += 1;
    }
    pub fn say(&mut self, tone: Tone, message: impl Into<String>) {
        self.tone = tone;
        self.message = message.into();
    }
    pub fn select(&mut self, path: &str) {
        self.selected = Some(path.to_string());
        self.say(Tone::Done, format!("selected {path}"));
        self.touch();
    }
}

pub struct PlaneInfo {
    pub path: String,
    pub name: String,
    pub pitch: Option<f64>,
    pub pitch_status: Status,
    pub area: Option<f64>,
    pub area_status: Status,
    pub condition: Option<String>,
    /// The plane placed in space, when it has everything that takes.
    pub solid: Option<roof::SolidPlane>,
}

impl PlaneInfo {
    pub fn worst(&self) -> Status {
        self.pitch_status.max(self.area_status)
    }
    /// "plane 3" is facet 3; a plane with a name of its own keeps it.
    pub fn short(&self) -> String {
        self.name.strip_prefix("plane ").unwrap_or(&self.name).to_string()
    }
}

pub struct LineInfo {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    pub length: Option<f64>,
    pub status: Status,
    /// Its two ends as plan east, plan north, height, when a plane it lies on is placed.
    pub ends: Option<[[f64; 3]; 2]>,
    pub planes: Vec<String>,
}

/// A system as the windows list it.
pub struct SystemInfoRow {
    pub takeoff: SystemTakeoff,
    /// How many of its elements are drawn.
    pub drawn: usize,
}

#[derive(Resource, Default)]
pub struct Snapshot {
    pub revision: u64,
    pub title: String,
    pub file: String,
    pub roof_path: String,
    pub roof_name: String,
    /// Values in the whole job at each status: Known, Assumed, Unknown.
    pub tally: (usize, usize, usize),
    /// What the job comes to both ways (`view::pricing`).
    pub pricing: Option<view::Pricing>,
    /// The estimate sheet and its recap (`view::estimate`).
    pub estimate: Option<view::Estimate>,
    /// The drawings laid under the storeys.
    pub drawings: Vec<crate::plans::Laid>,
    /// What clashes (`building::clash::clashes`), or why that could not be worked out.
    pub clashes: Option<Result<plumbline_world::building::clash::Clashes, String>>,
    pub totals: Option<view::RoofTotals>,
    pub planes: Vec<PlaneInfo>,
    pub lines: Vec<LineInfo>,
    /// What is not yet Known, grouped by what it rests on: status, what, why.
    pub notes: Vec<(Status, String, String)>,
    /// The selected element with every value on it.
    pub selected: Option<view::ElementRows>,
    /// Which of the selected element's entered values a measurement can be put on: each
    /// by its name, and whether it is one length.
    pub takes: Vec<(String, bool)>,
    /// For a selected plane solved from a footprint: the roof and the edge it rises from.
    pub rises_from: Option<(String, usize)>,
    /// A selected plane's corners as plan east, plan north and height, in feet.
    pub corners: Vec<[f64; 3]>,
    /// Everything in the job on plan, low corner and high corner, and its highest point.
    pub extent: Option<([f64; 2], [f64; 2], f64)>,
    /// The rest of the building, placed in space.
    pub drawn: Vec<Placement>,
    /// What has no shape to draw, and why.
    pub not_drawn: Vec<(String, String)>,
    pub storeys: Vec<StoreyTotals>,
    pub systems: Vec<SystemInfoRow>,
    /// How many elements of each discipline are drawn.
    pub counts: Vec<(Discipline, usize)>,
    /// Every wall, floor and room as the sheet lists it: storey path, then the element's rows.
    pub fabric: Vec<(String, view::ElementRows)>,
}

impl Snapshot {
    pub fn system(&self, path: &str) -> Option<&SystemTakeoff> {
        self.systems.iter().map(|s| &s.takeoff).find(|s| s.path == path)
    }
    /// The colour an element of the building is drawn in, by the system it is on.
    pub fn classification(&self, system: Option<&String>) -> &str {
        system.and_then(|p| self.system(p)).map(|s| s.classification.as_str()).unwrap_or("")
    }
}

/// Read the world again whenever it has changed.
pub fn refresh(mut doc: ResMut<Doc>, mut snap: ResMut<Snapshot>) {
    if snap.revision == doc.revision {
        return;
    }
    let doc = &mut *doc;
    let mut s = Snapshot { revision: doc.revision, ..default() };
    s.title = doc.job.world.get::<JobInfo>(doc.job.root()).map(|j| j.title.clone()).unwrap_or_default();
    s.file = doc.path.file_name().and_then(|f| f.to_str()).unwrap_or("").to_string();
    let elements = view::elements(&mut doc.job);
    // What each thing is called, for everything written on screen (`theme::spoken`).
    crate::theme::name_these(elements.iter().map(|e| (e.path.clone(), e.name.clone())).collect());
    s.tally = view::tally(&elements);
    s.pricing = Some(view::pricing(&mut doc.job));
    s.estimate = Some(view::estimate(&mut doc.job));
    s.clashes = Some(plumbline_world::building::clash::clashes(&mut doc.job, None));
    if doc.selected.as_ref().is_some_and(|sel| doc.job.find(sel).is_none()) {
        doc.selected = None;
    }
    let mut lo = [f64::MAX; 2];
    let mut hi = [f64::MIN; 2];
    let mut top = f64::MIN;
    let mut reach = |p: [f64; 3]| {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
        top = top.max(p[2]);
    };

    // The roof on show.
    let mut solved: Vec<String> = Vec::new();
    if let Some(roof_path) = doc.roof.clone() {
        s.roof_name = elements.iter().find(|e| e.path == roof_path).map(|e| e.name.clone()).unwrap_or_default();
        let under: Vec<String> = doc.job.children().get(&roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
        let (solid, _) = roof::solid_planes(&mut doc.job);
        let w = &doc.job.world;
        let index: std::collections::HashMap<String, Entity> = doc.job.order.iter().filter_map(|e| w.get::<plumbline_world::Node>(*e).map(|n| (n.path.clone(), *e))).collect();
        let measure = |e: Entity, name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (d.value, d.status)).unwrap_or((None, Status::Unknown));
        for el in elements.iter().filter(|e| under.contains(&e.path)) {
            let Some(&e) = index.get(&el.path) else { continue };
            if w.get::<Solved>(e).is_some() {
                solved.push(el.path.clone());
            }
            if el.kind.is_area() {
                let (area, area_status) = measure(e, "surface_area");
                let pitch = w.get::<Pitch>(e).map(|p| (p.0.value, p.0.status)).unwrap_or((None, Status::Unknown));
                s.planes.push(PlaneInfo {
                    path: el.path.clone(),
                    name: el.name.clone(),
                    pitch: pitch.0,
                    pitch_status: pitch.1,
                    area,
                    area_status,
                    condition: w.get::<Condition>(e).and_then(|c| c.0.value.clone()),
                    solid: solid.iter().find(|p| p.path == el.path).cloned(),
                });
            } else if el.kind.is_run() {
                // A parapet stands on the plan; every other line is measured on the slope.
                let (length, status) = measure(e, if el.kind == Kind::Parapet { "plan_length" } else { "true_length" });
                let run = w.get::<Run>(e);
                let planes: Vec<String> = run.map(|r| r.planes.clone()).unwrap_or_default();
                // A line's height comes from a plane it lies on.
                let on = planes.iter().find_map(|p| solid.iter().find(|x| x.path == *p));
                let ends = run.and_then(|r| r.ends_ft()).zip(on).map(|([a, b], plane)| [[a[0], a[1], plane.height_at(a)], [b[0], b[1], plane.height_at(b)]]);
                s.lines.push(LineInfo { path: el.path.clone(), name: el.name.clone(), kind: el.kind, length, status, ends, planes });
            }
        }
        for plane in s.planes.iter().filter_map(|p| p.solid.as_ref()) {
            for p in &plane.points {
                reach([p[0], p[1], plane.height_at(*p)]);
            }
        }
        s.totals = Some(view::roof_totals(&mut doc.job, &roof_path));
        s.roof_path = roof_path;
    }

    // The rest of the building.
    let (drawn, not_drawn) = building::placements(&mut doc.job);
    for p in &drawn {
        match &p.shape {
            building::Shape::Wall { a, b, base, height, .. } => {
                reach([a[0], a[1], base + height]);
                reach([b[0], b[1], base + height]);
            }
            building::Shape::Floor { boundary, top, .. } => boundary.iter().for_each(|q| reach([q[0], q[1], *top])),
            // What runs out of the building (a service, a drain) does not set the frame.
            _ => {}
        }
    }
    for d in [Discipline::Architecture, Discipline::Mechanical, Discipline::Electrical, Discipline::Plumbing] {
        s.counts.push((d, drawn.iter().filter(|p| p.discipline == d).count()));
    }
    s.systems = building::takeoff(&mut doc.job).into_iter().map(|t| SystemInfoRow { drawn: drawn.iter().filter(|p| p.system.as_deref() == Some(t.path.as_str())).count(), takeoff: t }).collect();
    s.storeys = building::storey_totals(&mut doc.job);
    // By the storey each is under in the job, whether or not it can be drawn: a wall with
    // no height is still a row of its storey's table.
    let on = building::storeys_of(&mut doc.job);
    for el in elements.iter().filter(|e| matches!(e.kind, Kind::Wall | Kind::Floor | Kind::Room)) {
        s.fabric.push((on.get(&el.path).cloned().unwrap_or_default(), el.clone()));
    }
    s.drawn = drawn;
    s.not_drawn = not_drawn;
    // A drawing laid under a storey is part of what there is to look at.
    s.drawings = crate::plans::laid(&mut doc.job);
    for l in &s.drawings {
        let (a, b) = (l.on_plan([0.0, 0.0]), l.on_plan(l.drawing.size));
        reach([a[0], a[1], 0.0]);
        reach([b[0], b[1], 0.0]);
    }

    if lo[0] <= hi[0] {
        s.extent = Some((lo, hi, top));
        // The view's origin: the middle of the building on plan, at the datum. Set once,
        // so the picture does not jump when an edit moves something.
        if !doc.framed {
            doc.origin = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, 0.0];
        }
    }

    // What is not yet Known. Solved planes and lines rest on their roof's footprint, which
    // is listed itself; a condition's recipe is a blank form, not this job's unknowns.
    let mut groups: Vec<(Status, String, Vec<String>)> = Vec::new();
    for el in &elements {
        if el.kind == Kind::Condition || solved.contains(&el.path) {
            continue;
        }
        for r in el.rows.iter().filter(|r| r.status != Status::Known) {
            if r.label == "solved roof" {
                s.notes.push((r.status, if r.text == "not solved" { format!("{}: not solved", el.name) } else { format!("{}: solved, with a note", el.name) }, r.note.clone()));
                continue;
            }
            if r.derived {
                // A figure refused for a reason of its own (an opening that does not fit,
                // a line that does not fit its planes) is said with the reason. One that
                // is only flagged because an input is, is covered by that input's entry.
                // What an element comes to follows from its lines' extensions, and a line
                // with no price is already listed by its unit cost: saying each of those
                // again would bury the one thing there is to settle. An extension refused
                // for a reason of its own (less than a cent) is still said.
                let follows = r.label.starts_with("comes to,") || (r.label.starts_with("extension:") && (r.note.starts_with("the line has no price") || r.note.starts_with("the line has no quantity")));
                if !r.note.is_empty() && !follows {
                    s.notes.push((r.status, format!("{} of {} has no figure", crate::panels::pretty(&r.label), el.name), r.note.clone()));
                }
                continue;
            }
            let mut what = r.label.clone();
            if let Some(first) = what.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            let said = format!("{what} of {}: {}", el.name, crate::theme::trade(&r.text));
            match groups.iter_mut().find(|g| g.0 == r.status && g.1 == r.basis) {
                Some(g) => g.2.push(said),
                None => groups.push((r.status, r.basis.clone(), vec![said])),
            }
        }
    }
    for (path, why) in &s.not_drawn {
        // Said by its name, with its path after it: a person reads the name.
        let name = elements.iter().find(|e| e.path == *path).map(|e| e.name.as_str()).filter(|n| !n.is_empty());
        let called = name.map(|n| format!("{n} ({path})")).unwrap_or_else(|| path.clone());
        s.notes.push((Status::Unknown, format!("{called} is not drawn"), why.clone()));
    }
    for (status, why, said) in groups {
        let what = if said.len() == 1 { said[0].clone() } else { format!("{} values, the first: {}", said.len(), said[0]) };
        s.notes.push((status, what, why));
    }
    s.notes.sort_by(|a, b| b.0.cmp(&a.0));

    if let Some(sel) = doc.selected.clone() {
        s.rises_from = doc.job.find(&sel).and_then(|e| doc.job.world.get::<Solved>(e)).and_then(|x| x.edge.map(|edge| (x.from.clone(), edge)));
        if let Some(solid) = s.planes.iter().find(|p| p.path == sel).and_then(|p| p.solid.as_ref()) {
            s.corners = solid.points.iter().map(|p| [p[0], p[1], solid.height_at(*p)]).collect();
        }
        s.selected = elements.into_iter().find(|e| e.path == sel);
        s.takes = plumbline_world::editing::takes(&mut doc.job, &sel);
    }
    *snap = s;
}

/// A plan point and a height, as a point in the view: x east, y up, z south.
pub fn to_view(origin: [f64; 3], p: [f64; 3]) -> Vec3 {
    Vec3::new((p[0] - origin[0]) as f32, (p[2] - origin[2]) as f32, -((p[1] - origin[1]) as f32))
}
