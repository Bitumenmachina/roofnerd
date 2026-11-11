//! The open job, and one reading of it that every window draws from.
//!
//! `Doc` holds the job world and what the estimator is doing to it. `Snapshot` is read
//! from the world once after each change; the 3D scene, the panels and the sheet all
//! draw the same snapshot, so they cannot disagree. Nothing here works out a quantity.

use std::path::PathBuf;

use bevy::prelude::*;
use plumbline_world::value::Status;
use plumbline_world::{roof, view, Condition, Job, JobInfo, Kind, Measure, Pitch, Run, Solved};

/// The source written on a value set in the 3D view.
pub const SOURCE: &str = "set in the 3D view; not yet confirmed against a drawing";

#[derive(Clone, Copy, PartialEq)]
pub enum Entry {
    Pitch,
    Move,
}

#[derive(Resource)]
pub struct Doc {
    pub job: Job,
    pub path: PathBuf,
    /// The roof on show.
    pub roof: Option<String>,
    /// The selected element, by path.
    pub selected: Option<String>,
    /// The plane the pointer is over.
    pub hover: Option<String>,
    /// What the last action did, or why it was refused.
    pub message: String,
    /// Goes up on every change, so views know to draw again.
    pub revision: u64,
    /// A value being typed: which field, and the text so far.
    pub entry: Option<(Entry, String)>,
    /// The plan point the model's origin is drawn at, in feet. The view is drawn relative
    /// to it in 32-bit numbers; the model itself stays 64-bit.
    pub origin: [f64; 3],
    pub framed: bool,
    /// Where the pointer is on the roof, in plan feet and height.
    pub pointer: Option<[f64; 3]>,
    pub shots: u32,
    /// Changed since it was last saved.
    pub edited: bool,
    /// Looking straight down, as a roof plan is drawn.
    pub plan: bool,
}

impl Doc {
    pub fn touch(&mut self) {
        self.revision += 1;
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

#[derive(Resource, Default)]
pub struct Snapshot {
    pub revision: u64,
    pub title: String,
    pub file: String,
    pub roof_path: String,
    pub roof_name: String,
    /// Values in the whole job at each status: Known, Assumed, Unknown.
    pub tally: (usize, usize, usize),
    pub totals: Option<view::RoofTotals>,
    pub planes: Vec<PlaneInfo>,
    pub lines: Vec<LineInfo>,
    /// What the solve had to say, and the roof's own flagged values: what, and why.
    pub notes: Vec<(Status, String, String)>,
    /// The selected element with every value on it.
    pub selected: Option<view::ElementRows>,
    /// For a selected plane solved from a footprint: the roof and the edge it rises from.
    pub rises_from: Option<(String, usize)>,
    /// A selected plane's corners as plan east, plan north and height, in feet.
    pub corners: Vec<[f64; 3]>,
    /// The roof's extent on plan, low corner and high corner, and its highest point.
    pub extent: Option<([f64; 2], [f64; 2], f64)>,
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
    s.tally = view::tally(&elements);
    if doc.selected.as_ref().is_some_and(|sel| doc.job.find(sel).is_none()) {
        doc.selected = None;
    }
    if let Some(roof_path) = doc.roof.clone() {
        s.roof_name = elements.iter().find(|e| e.path == roof_path).map(|e| e.name.clone()).unwrap_or_default();
        let under: Vec<String> = doc.job.children().get(&roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
        let (solid, _) = roof::solid_planes(&mut doc.job);
        let w = &doc.job.world;
        let index: std::collections::HashMap<String, Entity> = doc.job.order.iter().filter_map(|e| w.get::<plumbline_world::Node>(*e).map(|n| (n.path.clone(), *e))).collect();
        let measure = |e: Entity, name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (d.value, d.status)).unwrap_or((None, Status::Unknown));
        for el in elements.iter().filter(|e| under.contains(&e.path)) {
            let Some(&e) = index.get(&el.path) else { continue };
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
                let (length, status) = measure(e, "true_length");
                let run = w.get::<Run>(e);
                let planes: Vec<String> = run.map(|r| r.planes.clone()).unwrap_or_default();
                // A line's height comes from a plane it lies on.
                let on = planes.iter().find_map(|p| solid.iter().find(|x| x.path == *p));
                let ends = run.and_then(|r| r.ends_ft()).zip(on).map(|([a, b], plane)| [[a[0], a[1], plane.height_at(a)], [b[0], b[1], plane.height_at(b)]]);
                s.lines.push(LineInfo { path: el.path.clone(), name: el.name.clone(), kind: el.kind, length, status, ends, planes });
            }
        }
        if let Some(el) = elements.iter().find(|e| e.path == roof_path) {
            for r in el.rows.iter().filter(|r| r.label == "solved roof") {
                s.notes.push((r.status, if r.text == "not solved" { "Not solved".to_string() } else { "Solved, with a note".to_string() }, r.note.clone()));
            }
            for r in el.rows.iter().filter(|r| r.status != Status::Known && r.label != "solved roof") {
                let mut what = r.label.clone();
                if let Some(first) = what.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                s.notes.push((r.status, format!("{what} of {}: {}", s.roof_name, crate::theme::trade(&r.text)), r.basis.clone()));
            }
        }
        let placed: Vec<&roof::SolidPlane> = s.planes.iter().filter_map(|p| p.solid.as_ref()).collect();
        if !placed.is_empty() {
            let mut lo = [f64::MAX; 2];
            let mut hi = [f64::MIN; 2];
            let mut top = f64::MIN;
            for plane in &placed {
                for p in &plane.points {
                    lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                    hi = [hi[0].max(p[0]), hi[1].max(p[1])];
                    top = top.max(plane.height_at(*p));
                }
            }
            s.extent = Some((lo, hi, top));
            // The view's origin: the middle of the roof on plan, at the datum. Set once,
            // so the picture does not jump when an edit moves the roof.
            if !doc.framed {
                doc.origin = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, 0.0];
            }
        }
        s.totals = Some(view::roof_totals(&mut doc.job, &roof_path));
        s.roof_path = roof_path;
    }
    if let Some(sel) = doc.selected.clone() {
        s.rises_from = doc.job.find(&sel).and_then(|e| doc.job.world.get::<Solved>(e)).and_then(|x| x.edge.map(|edge| (x.from.clone(), edge)));
        if let Some(solid) = s.planes.iter().find(|p| p.path == sel).and_then(|p| p.solid.as_ref()) {
            s.corners = solid.points.iter().map(|p| [p[0], p[1], solid.height_at(*p)]).collect();
        }
        s.selected = elements.into_iter().find(|e| e.path == sel);
    }
    *snap = s;
}

/// A plan point and a height, as a point in the view: x east, y up, z south.
pub fn to_view(origin: [f64; 3], p: [f64; 3]) -> Vec3 {
    Vec3::new((p[0] - origin[0]) as f32, (p[2] - origin[2]) as f32, -((p[1] - origin[1]) as f32))
}
