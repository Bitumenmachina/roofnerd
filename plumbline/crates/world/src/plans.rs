//! Plans laid under the tracing: a Drawing is one page of a plan file (a PDF, or a
//! picture) under the storey it is a plan of, with the scale that turns a place on the
//! page into a place on the plan.
//!
//! Ours. BIMCraft reads no plan files (`grep -rn "underlay\|calibrat" crates/core/src`
//! finds nothing of the kind). The way a takeoff is given its scale is the trade's: two
//! points on the drawing and the distance between them.
//!
//! This crate reads no disk. A Drawing says where its file is, how long it was and its
//! fingerprint when the drawing was made from it; `plumbline-plans` reads the page, and
//! whoever shows it can tell that the file has changed. The picture is not in the job.
//!
//! What is traced over a drawing is Assumed, and says which drawing (Patrick's ruling of
//! 2026-11-11: traced values start as Assumed; a field measurement makes them Known).

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{Kind, Node, P2};
use crate::editing::{all_or_nothing, Entered};
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Status, Val};

pub const K_DRAWING: &str = "pl::drawing";

/// The scale of a drawing: two points on the page, in the page's own units from its
/// lower-left corner, and the distance between them on the building.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scale {
    pub from: P2,
    pub to: P2,
    pub distance: Val<Len>,
}

/// `pl::drawing`: on a Drawing, one page of a plan file laid under the tracing.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Drawing {
    /// Where the file is: its path from the folder the job is in.
    pub file: Val<String>,
    /// Which page of it, counted from 1.
    pub page: usize,
    /// The file's length in bytes and its fingerprint when the drawing was made from it.
    pub bytes: u64,
    pub checksum: String,
    /// The page's width and height in its own units (points for a PDF, dots for a picture).
    pub size: P2,
    /// None until it is calibrated. Nothing is traced over a drawing with no scale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Scale>,
    /// Where the page's lower-left corner sits on the plan, in feet.
    pub at: Val<P2>,
    /// What it was lined up by: a place on the page and the place on the plan that is.
    /// Kept so that a scale set again leaves that place where it was put.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lined_up: Option<LinedUp>,
    /// Scales it had before, each with what was traced over it at that scale and is
    /// still where that scale put it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub earlier: Vec<Earlier>,
}

/// A place on a drawing's page and the place on the plan, in feet, it was said to be.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinedUp {
    pub page: P2,
    pub plan: P2,
}

/// A scale and a place a drawing had and no longer has, and what was traced over it then.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Earlier {
    pub feet_per_unit: f64,
    /// Where the page's lower-left corner lay on the plan then, in feet. None in a job
    /// saved before this was kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<P2>,
    pub traced: Vec<String>,
}

/// Things traced over a drawing before its scale or its place changed, still in the
/// job and still where they were traced.
#[derive(Clone, Debug, PartialEq)]
pub struct Stale {
    /// Feet to a unit of the page when they were traced.
    pub feet_per_unit: f64,
    /// The scale has changed since.
    pub rescaled: bool,
    /// The sheet has been moved since (lined up again).
    pub moved: bool,
    pub traced: Vec<String>,
}

/// What setting a drawing's scale did.
#[derive(Clone, Debug, PartialEq)]
pub struct Calibrated {
    /// Feet on the building to one unit of the page.
    pub feet_per_unit: f64,
    /// What was traced over the drawing at the scale it had before, by path. None of it
    /// was moved: a length typed off a dimension is right whatever the sheet's scale was,
    /// and a point picked on the sheet is not, and the two cannot be told apart here.
    pub left_at_earlier_scale: Vec<String>,
    /// The drawing had been lined up, and the place it was lined up by is where it was.
    pub lined_up_again: bool,
}

impl Drawing {
    /// Feet on the building to one unit of the page, when it has a scale that works out.
    pub fn feet_per_unit(&self) -> Option<f64> {
        let s = self.scale.as_ref()?;
        let on_page = (s.to[0] - s.from[0]).hypot(s.to[1] - s.from[1]);
        let feet = s.distance.feet()?;
        (on_page > 0.0 && feet > 0.0).then(|| feet / on_page)
    }
    fn corner(&self) -> Option<P2> {
        let d = units::coord_divisor(&self.at.unit).ok()?;
        self.at.value.map(|p| [p[0] / d, p[1] / d])
    }
    /// A place on the page as a place on the plan, in feet.
    pub fn to_plan(&self, p: P2) -> Option<P2> {
        let (k, o) = (self.feet_per_unit()?, self.corner()?);
        Some([o[0] + p[0] * k, o[1] + p[1] * k])
    }
    /// A place on the plan, in feet, as a place on the page.
    pub fn to_page(&self, q: P2) -> Option<P2> {
        let (k, o) = (self.feet_per_unit()?, self.corner()?);
        Some([(q[0] - o[0]) / k, (q[1] - o[1]) / k])
    }
    /// Whether the drawing is now at the scale and in the place it had then: what was
    /// traced then is where the sheet shows it again.
    fn is_as_it_was(&self, then: &Earlier) -> bool {
        let same_scale = self.feet_per_unit().is_some_and(|k| (then.feet_per_unit / k - 1.0).abs() <= 1.0e-9);
        let same_place = then.at.zip(self.corner()).is_some_and(|(a, b)| (a[0] - b[0]).abs() <= 1.0e-6 && (a[1] - b[1]).abs() <= 1.0e-6);
        same_scale && same_place
    }
    /// How sure a place read off this drawing is: no surer than its scale and where it lies.
    pub fn status(&self) -> Status {
        match &self.scale {
            None => Status::Unknown,
            Some(s) => s.distance.status.max(self.at.status),
        }
    }
}

/// The rows a flat reading of the job shows for a drawing: its file, the dimension its
/// scale was set from (or that it has none, which is a thing not known), and where it
/// lies. They are counted with every other value that is Known, Assumed or Unknown.
pub fn rows(w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity, path: &str) -> Vec<crate::view::Row> {
    use crate::view::{self, Row};
    let Some(d) = w.get::<Drawing>(e) else { return Vec::new() };
    let mut rows = vec![view::entered(path, "file", &d.file, |f| format!("{f}, page {}", d.page))];
    match &d.scale {
        Some(s) => rows.push(view::length(path, "scale dimension", &s.distance)),
        None => rows.push(Row { path: path.into(), label: "scale".into(), text: "none".into(), status: Status::Unknown, basis: "not set: nothing can be traced over it".into(), note: String::new(), derived: false, number: None, unit: String::new() }),
    }
    rows.push(view::entered(path, "corner on the plan", &d.at, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
    // What was traced before the sheet's scale or place changed and was not moved: said
    // wherever the drawing's values are, flagged, until those things are gone or checked.
    let listed: Vec<&String> = d.earlier.iter().filter(|then| !d.is_as_it_was(then)).flat_map(|then| &then.traced).collect();
    if !listed.is_empty() {
        let names: std::collections::HashMap<String, String> = w.iter_entities().filter_map(|x| x.get::<Node>().map(|n| (n.path.clone(), n.name.clone()))).collect();
        let still: Vec<String> = listed.iter().filter_map(|p| names.get(*p).cloned()).collect();
        if !still.is_empty() {
            rows.push(Row {
                path: path.into(),
                label: "traced before it was rescaled or moved".into(),
                text: format!("{}: {}", if still.len() == 1 { "1 thing".to_string() } else { format!("{} things", still.len()) }, still.join(", ")),
                status: Status::Assumed,
                basis: "these were not moved when the sheet's scale or place was changed: check each against the sheet".into(),
                note: String::new(),
                derived: false,
                number: None,
                unit: String::new(),
            });
        }
    }
    rows
}

pub fn check(path: &str, kind: Kind, d: &Drawing) -> Result<(), String> {
    if kind != Kind::Drawing {
        return Err(format!("{path}: a {kind:?} cannot carry a drawing's page and scale; only a Drawing can"));
    }
    d.file.check_basics(&format!("{path} file"))?;
    if d.file.value.as_ref().is_some_and(|f| f.trim().is_empty() || f.chars().any(char::is_control)) {
        return Err(format!("{path} file: is not a usable path"));
    }
    if d.page == 0 {
        return Err(format!("{path}: pages are counted from 1"));
    }
    if !(d.size[0].is_finite() && d.size[1].is_finite() && d.size[0] > 0.0 && d.size[1] > 0.0) {
        return Err(format!("{path}: its page has no usable size"));
    }
    if d.checksum.len() != 16 || !d.checksum.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{path}: its file's fingerprint is not sixteen hexadecimal digits"));
    }
    d.at.check_basics(&format!("{path} place on the plan"))?;
    units::coord_divisor(&d.at.unit).map_err(|why| format!("{path} place on the plan: {why}"))?;
    if d.at.value.is_some_and(|p| !p.iter().all(|x| x.is_finite() && x.abs() <= crate::check::MAX_COORDINATE_FT * 12.0)) {
        return Err(format!("{path} place on the plan: is not a usable point"));
    }
    if let Some(s) = &d.scale {
        let ok = |p: P2| p.iter().all(|x| x.is_finite());
        if !ok(s.from) || !ok(s.to) {
            return Err(format!("{path} scale: its two points are not usable numbers"));
        }
        // Less than one unit of the page apart (a point, a dot) is one point: no
        // dimension on a drawing is that short, and a scale from it would be noise.
        if (s.to[0] - s.from[0]).hypot(s.to[1] - s.from[1]) < 1.0 {
            return Err(format!("{path} scale: its two points are one point; a scale is the distance between two places on the drawing"));
        }
        let on_page = |p: P2| p[0] >= -1.0 && p[1] >= -1.0 && p[0] <= d.size[0] + 1.0 && p[1] <= d.size[1] + 1.0;
        if !on_page(s.from) || !on_page(s.to) {
            return Err(format!("{path} scale: its two points are not both on the page, which is {} by {}", crate::view::num(d.size[0]), crate::view::num(d.size[1])));
        }
        s.distance.check_length(&format!("{path} scale distance"))?;
        if s.distance.feet().is_some_and(|f| !(f > 0.0)) {
            return Err(format!("{path} scale distance: is more than nothing"));
        }
    }
    if let Some(l) = &d.lined_up {
        if !l.page.iter().chain(&l.plan).all(|x| x.is_finite() && x.abs() <= crate::check::MAX_COORDINATE_FT * 1.0e3) {
            return Err(format!("{path}: what it was lined up by is not two usable points"));
        }
    }
    for e in &d.earlier {
        if !(e.feet_per_unit.is_finite() && e.feet_per_unit > 0.0) {
            return Err(format!("{path}: an earlier scale of it is not a scale"));
        }
    }
    Ok(())
}

/// What `add` is told about the file, by whoever read it.
pub struct Read<'a> {
    pub file: &'a str,
    pub page: usize,
    pub bytes: u64,
    pub checksum: &'a str,
    pub size: P2,
}

/// A page of a plan file, laid under a storey. It has no scale yet. Its lower-left corner
/// is put at the plan's own corner (0, 0) until it is lined up with something.
pub fn add(job: &mut Job, storey: &str, name: &str, read: Read, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        match job.kind(storey) {
            Some(Kind::Storey) => {}
            Some(other) => return Err(format!("{storey} is a {other:?}, not a level: a drawing is laid under the level it is a plan of")),
            None => return Err(format!("no element {storey}")),
        }
        let drawing = Drawing {
            file: Val { value: Some(read.file.to_string()), unit: String::new(), status: how.status, source: how.source.to_string() },
            page: read.page,
            bytes: read.bytes,
            checksum: read.checksum.to_string(),
            size: read.size,
            scale: None,
            at: Val { value: Some([0.0, 0.0]), unit: units::FT.into(), status: Status::Assumed, source: "its lower-left corner put at the plan's corner; not yet lined up with anything".into() },
            lined_up: None,
            earlier: Vec::new(),
        };
        let path = job.add(storey, Kind::Drawing, name)?;
        check(&path, Kind::Drawing, &drawing)?;
        job.put(&path, drawing)?;
        Ok(path)
    })
}

fn drawing_of(job: &mut Job, path: &str) -> Result<Drawing, String> {
    let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
    job.world.get::<Drawing>(e).cloned().ok_or_else(|| format!("{path} is not a drawing"))
}

/// The source a thing traced over this drawing carries.
fn traced_text(job: &mut Job, drawing: &str, d: &Drawing) -> String {
    let name = job.find(drawing).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap_or_default();
    format!("traced from {name} ({}, page {})", d.file.value.clone().unwrap_or_default(), d.page)
}

/// Every element with a value traced over this drawing, by path, in the job's order.
pub fn traced_over(job: &mut Job, drawing: &str) -> Result<Vec<String>, String> {
    let d = drawing_of(job, drawing)?;
    let text = traced_text(job, drawing, &d);
    Ok(crate::view::elements(job).into_iter().filter(|e| e.rows.iter().any(|r| !r.derived && r.basis.starts_with(&text))).map(|e| e.path).collect())
}

/// Give a drawing its scale: two points on the page (in its own units, from its
/// lower-left corner) and the distance between them.
///
/// Set again on a drawing that had one, it moves nothing that was traced: those things
/// are listed in what comes back and kept on the drawing as traced at its earlier scale,
/// so the sheet and what was traced over it are never silently at two scales. A drawing
/// that was lined up stays lined up: the place it was lined up by is where it was.
pub fn calibrate(job: &mut Job, drawing: &str, from: P2, to: P2, distance: &str, how: Entered) -> Result<Calibrated, String> {
    all_or_nothing(job, |job| {
        let mut d = drawing_of(job, drawing)?;
        let (was, corner_was) = (d.feet_per_unit(), d.corner());
        let (len, unit) = units::parse_length(distance)?;
        d.scale = Some(Scale { from, to, distance: Val { value: Some(len), unit: unit.into(), status: how.status, source: how.source.to_string() } });
        check(drawing, Kind::Drawing, &d)?;
        let k = d.feet_per_unit().ok_or_else(|| format!("{drawing}: that scale does not work out to a distance"))?;
        let mut left = Vec::new();
        if let Some(was) = was.filter(|was| (was / k - 1.0).abs() > 1.0e-9) {
            // What is traced now and is not already down as traced at a scale before that.
            let already: Vec<&String> = d.earlier.iter().flat_map(|e| &e.traced).collect();
            left = traced_over(job, drawing)?.into_iter().filter(|p| !already.contains(&p)).collect();
            if !left.is_empty() {
                d.earlier.push(Earlier { feet_per_unit: was, at: corner_was, traced: left.clone() });
            }
        }
        let lined_up_again = match &d.lined_up {
            Some(l) => {
                let tidy = |x: f64| (x * 1.0e6).round() / 1.0e6 + 0.0;
                d.at.value = Some([tidy(l.plan[0] - l.page[0] * k), tidy(l.plan[1] - l.page[1] * k)]);
                was.is_some()
            }
            None => false,
        };
        // A scale and a place the sheet is back at: what was traced then is where the
        // sheet shows it again, and is no longer listed.
        let now = d.clone();
        d.earlier.retain(|then| !now.is_as_it_was(then) && then.traced.iter().any(|p| job.find(p).is_some()));
        check(drawing, Kind::Drawing, &d)?;
        job.put(drawing, d)?;
        Ok(Calibrated { feet_per_unit: k, left_at_earlier_scale: left, lined_up_again })
    })
}

/// Line a drawing up: the place on its page `on_page` is the place on the plan `on_plan`
/// (in feet). A second storey's plan is laid over the first by a corner they share.
///
/// Lined up again after things were traced over it, the sheet moves and they do not:
/// they are what comes back, and they are kept on the drawing as traced where it lay
/// before, as a changed scale keeps them, so the sheet and what was traced over it are
/// never silently in two places.
pub fn line_up(job: &mut Job, drawing: &str, on_page: P2, on_plan: P2, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        let mut d = drawing_of(job, drawing)?;
        let k = d.feet_per_unit().ok_or_else(|| format!("{drawing} has no scale yet: give it one before lining it up"))?;
        let corner_was = d.corner();
        let corner = [on_plan[0] - on_page[0] * k, on_plan[1] - on_page[1] * k];
        let tidy = |x: f64| (x * 1.0e6).round() / 1.0e6 + 0.0;
        let corner = [tidy(corner[0]), tidy(corner[1])];
        let mut left = Vec::new();
        if corner_was.is_some_and(|was| (was[0] - corner[0]).abs() > 1.0e-6 || (was[1] - corner[1]).abs() > 1.0e-6) {
            let already: Vec<&String> = d.earlier.iter().flat_map(|e| &e.traced).collect();
            left = traced_over(job, drawing)?.into_iter().filter(|p| !already.contains(&p)).collect();
            if !left.is_empty() {
                d.earlier.push(Earlier { feet_per_unit: k, at: corner_was, traced: left.clone() });
            }
        }
        d.at = Val { value: Some(corner), unit: units::FT.into(), status: how.status, source: how.source.to_string() };
        d.lined_up = Some(LinedUp { page: on_page, plan: on_plan });
        let now = d.clone();
        d.earlier.retain(|then| !now.is_as_it_was(then) && then.traced.iter().any(|p| job.find(p).is_some()));
        check(drawing, Kind::Drawing, &d)?;
        job.put(drawing, d)?;
        Ok(left)
    })
}

/// What was traced over a drawing before its scale or its place changed, is still in
/// the job, and is still where it was traced.
pub fn stale(job: &mut Job, drawing: &str) -> Result<Vec<Stale>, String> {
    let d = drawing_of(job, drawing)?;
    let k = d.feet_per_unit();
    Ok(d.earlier
        .iter()
        .filter(|then| !d.is_as_it_was(then))
        .map(|then| {
            let rescaled = !k.is_some_and(|k| (then.feet_per_unit / k - 1.0).abs() <= 1.0e-9);
            Stale { feet_per_unit: then.feet_per_unit, rescaled, moved: !rescaled || then.at.zip(d.corner()).is_some_and(|(a, b)| (a[0] - b[0]).abs() > 1.0e-6 || (a[1] - b[1]).abs() > 1.0e-6), traced: then.traced.iter().filter(|p| job.find(p).is_some()).cloned().collect() }
        })
        .filter(|s| !s.traced.is_empty())
        .collect())
}

/// What was traced over a drawing at a scale it no longer has and is still in the job:
/// each earlier scale (feet to a unit of the page) with those things' paths.
pub fn at_earlier_scales(job: &mut Job, drawing: &str) -> Result<Vec<(f64, Vec<String>)>, String> {
    Ok(stale(job, drawing)?.into_iter().filter(|s| s.rescaled).map(|s| (s.feet_per_unit, s.traced)).collect())
}

/// The storey a drawing is a plan of.
pub fn storey_of(job: &mut Job, drawing: &str) -> Result<String, String> {
    drawing_of(job, drawing)?;
    job.find(drawing).and_then(|e| job.world.get::<Node>(e).and_then(|n| n.parent.clone())).ok_or_else(|| format!("{drawing} is under no level"))
}

/// How a thing traced over a drawing is entered: Assumed, and saying which drawing. Refused
/// for a drawing with no scale: a tracing with no scale is a number nobody entered.
pub fn traced_from(job: &mut Job, drawing: &str) -> Result<String, String> {
    let d = drawing_of(job, drawing)?;
    if d.feet_per_unit().is_none() {
        return Err(format!("{drawing} has no scale: calibrate it from one dimension before tracing over it"));
    }
    Ok(traced_text(job, drawing, &d))
}
