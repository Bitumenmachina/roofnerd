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
    /// The scale as a drawing's title block writes it, where the page is paper of a
    /// known size (a PDF's page, 72 units to the inch) and the scale is one of the
    /// architect's: `1/4" = 1'-0"`. A PDF at any other scale says it as an engineer's
    /// scale is written, an inch of paper to so many feet (call 80). A picture has no
    /// paper size, and says what its sheet covers on the ground. None with no scale.
    /// How sure it is is the drawing's status, which whoever shows this shows with it.
    /// Worked here and nowhere else (call 88).
    pub fn scale_said(&self) -> Option<String> {
        let k = self.feet_per_unit()?;
        if self.file.value.as_deref().is_some_and(|f| f.to_ascii_lowercase().ends_with(".pdf")) {
            // Inches of paper to a foot of building.
            let paper = 1.0 / (k * 72.0);
            const NAMED: [(f64, &str); 11] = [(0.0625, "1/16"), (0.09375, "3/32"), (0.125, "1/8"), (0.1875, "3/16"), (0.25, "1/4"), (0.375, "3/8"), (0.5, "1/2"), (0.75, "3/4"), (1.0, "1"), (1.5, "1 1/2"), (3.0, "3")];
            if let Some((_, name)) = NAMED.iter().find(|(inches, _)| (paper / inches - 1.0).abs() < 0.005) {
                return Some(format!("{name}\" = 1'-0\""));
            }
            return Some(format!("1\" = {}", units::feet_as_ft_in(1.0 / paper, 8)));
        }
        let [w, h] = self.size;
        Some(format!("sheet covers {} x {}", units::feet_as_ft_in(w * k, 1), units::feet_as_ft_in(h * k, 1)))
    }
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

/// A file's path from a folder, with `..` where it has to climb: how the job keeps the
/// place of a plan file (call 153: one copy, for the command line and the window).
pub fn path_from(folder: &std::path::Path, file: &std::path::Path) -> Result<String, String> {
    use std::path::{Component, PathBuf};
    let folder = if folder.as_os_str().is_empty() { std::path::Path::new(".") } else { folder };
    let folder = folder.canonicalize().map_err(|e| format!("cannot find the job's folder: {e}"))?;
    let file = file.canonicalize().map_err(|e| format!("cannot find {}: {e}", file.display()))?;
    let (a, b): (Vec<Component>, Vec<Component>) = (folder.components().collect(), file.components().collect());
    let shared = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut out = PathBuf::new();
    for _ in shared..a.len() {
        out.push("..");
    }
    for part in &b[shared..] {
        out.push(part);
    }
    out.to_str().map(str::to_string).ok_or_else(|| "the plan file's path is not text this program can keep".to_string())
}

/// A scan's sheet laid under a level (DECISIONS.md call 128, call 153): the picture is
/// written beside the job, the drawing is made from it as any is, given its scale (its
/// whole width is so many feet) and put in the scan's own place (its lower-left corner is
/// where the scan's own feet put it). Everything about it is Assumed. Returns the
/// drawing's path.
#[allow(clippy::too_many_arguments)]
pub fn lay_scan_sheet(job: &mut Job, level: &str, folder: &std::path::Path, called: &str, taken: &str, cut_at: f64, png: &[u8], checksum: &str, size: P2, cut_size: [u32; 2], dots: f64, corner: P2) -> Result<String, String> {
    let ft_in = |x: f64| units::feet_as_ft_in(x, 16);
    let sheet = folder.join(format!("{called} cut at {}.png", ft_in(cut_at).replace('\'', " ft").replace('"', " in").replace("  ", " ").trim()));
    std::fs::write(&sheet, png).map_err(|e| format!("the sheet could not be written beside the job ({}): {e}", folder.display()))?;
    let kept = path_from(folder, &sheet)?;
    let source = format!("scan {called}.ply, cut {} above its floor ({taken})", ft_in(cut_at));
    let how = Entered { status: Status::Assumed, source: &source };
    let made = add(job, level, called, Read { file: &kept, page: 1, bytes: png.len() as u64, checksum, size }, how)?;
    let wide = f64::from(cut_size[0]);
    calibrate(job, &made, [0.0, 0.0], [wide, 0.0], &format!("{} ft", crate::view::num(wide / dots)), how)?;
    line_up(job, &made, [0.0, 0.0], corner, how)?;
    Ok(made)
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

/// A wall found in a scan, as it is handed to be laid (DECISIONS.md call 173): its centre
/// line on the plan in feet, how thick it is, whether the scan measured that, and its
/// doors and windows, each along the line from its first end.
#[derive(Clone, Debug)]
pub struct FoundWall {
    pub from: P2,
    pub to: P2,
    /// Feet. Measured from the scan's two faces of it, or the thickness of the type the
    /// person named for walls the scan saw from one side.
    pub thickness: f64,
    pub measured: bool,
    pub openings: Vec<FoundOpening>,
}

/// An opening found: along the wall from its first end, and its heights above the floor, in feet.
#[derive(Clone, Copy, Debug)]
pub struct FoundOpening {
    pub window: bool,
    pub from: f64,
    pub to: f64,
    pub sill: f64,
    pub head: f64,
}

/// What laying found walls made.
#[derive(Clone, Debug, Default)]
pub struct FoundLaid {
    pub walls: Vec<String>,
    pub doors: Vec<String>,
    pub windows: Vec<String>,
    /// Wall, door and window types the program made for them (those the job had are used).
    pub types: Vec<(String, String)>,
}

/// How thick a wall type makes a wall, in feet.
pub fn wall_type_feet(job: &mut Job, type_path: &str) -> Result<f64, String> {
    job.find(type_path).and_then(|e| job.world.get::<crate::WallType>(e)).and_then(|t| t.thickness.feet()).ok_or_else(|| format!("{type_path} is not a wall type with a thickness"))
}

/// Walls found in a scan laid on a level, with their doors and windows, all Assumed, their
/// source "found in scan <name>" (call 173). Laid with the model's own wall, door and
/// window: nothing here makes a wall another way. A wall seen from one side is laid with
/// `outside`'s type; the thickness it was handed is that type's. All of it or none: a
/// refusal leaves the job as it was. Laying a second time from the same scan onto the
/// same level is refused.
pub fn lay_found(job: &mut Job, level: &str, scan_name: &str, height_ft: f64, outside: Option<&str>, found: &[FoundWall]) -> Result<FoundLaid, String> {
    let source = format!("found in scan {scan_name}");
    let how = Entered { status: Status::Assumed, source: &source };
    all_or_nothing(job, |job| {
        // The scan's walls already on this level: laying twice would make two of each.
        let mut q = job.world.query::<(&crate::Node, &crate::Wall)>();
        let already = q.iter(&job.world).filter(|(n, w)| n.parent.as_deref() == Some(level) && w.line.source == source).count();
        if already > 0 {
            return Err(format!("{level} already has {already} wall{} found in scan {scan_name}: laying it again would make two of each. Take those away first if it is to be laid again", if already == 1 { "" } else { "s" }));
        }
        let had: std::collections::BTreeSet<String> = {
            let mut t = job.world.query::<(&crate::Node, &crate::components::Kind)>();
            t.iter(&job.world).filter(|(_, k)| **k == Kind::Type).map(|(n, _)| n.path.clone()).collect()
        };
        let inches = |ft: f64| format!("{} in", crate::view::num((ft * 12.0 * 4.0).round() / 4.0));
        let mut out = FoundLaid::default();
        let top = format!("{} in", crate::view::num((height_ft * 12.0).round()));
        for w in found {
            let kind = if w.measured {
                crate::editing::wall_type_for(job, &inches(w.thickness), how)?
            } else {
                outside.ok_or("a wall seen from one side needs the wall type it is to be laid as")?.to_string()
            };
            let line = [[w.from[0], w.from[1]], [w.to[0], w.to[1]]].map(|p| p.map(|x| (x * 10_000.0).round() / 10_000.0 + 0.0));
            let made = crate::editing::walls(job, level, &kind, &line, false, crate::editing::Top::Height(&top), how)?;
            let wall = made.first().ok_or("a wall was not made")?.clone();
            for o in &w.openings {
                let across = format!("{} in", (((o.to - o.from) * 12.0).round()).max(1.0));
                let (kind_of, type_path, sill) = if o.window {
                    let t = crate::editing::opening_type_for(job, Kind::Window, &across, &format!("{} in", ((o.head - o.sill) * 12.0).round().max(1.0)), how)?;
                    (Kind::Window, t, format!("{} in", (o.sill * 12.0).round()))
                } else {
                    let t = crate::editing::opening_type_for(job, Kind::Door, &across, &format!("{} in", (o.head * 12.0).round().max(1.0)), how)?;
                    (Kind::Door, t, "0 in".to_string())
                };
                let at = format!("{:.4} ft", (o.from + o.to) / 2.0);
                let path = crate::editing::opening(job, &wall, kind_of, &type_path, crate::editing::OnWall::Offset(&at), &sill, how)?;
                if o.window { out.windows.push(path) } else { out.doors.push(path) }
            }
            out.walls.push(wall);
        }
        let mut t = job.world.query::<(&crate::Node, &crate::components::Kind)>();
        out.types = t.iter(&job.world).filter(|(n, k)| **k == Kind::Type && !had.contains(&n.path)).map(|(n, _)| (n.path.clone(), n.name.clone())).collect();
        out.types.sort();
        Ok(out)
    })
}

#[cfg(test)]
mod found_tests {
    use super::*;
    use crate::editing;

    fn job_with_a_level() -> (Job, String) {
        let mut job = Job::new("found walls (made up)", "plumbline", "2026-11-22");
        let how = Entered { status: Status::Assumed, source: "test" };
        let level = editing::storey(&mut job, "ground", "0 ft", "8 ft", how).unwrap();
        (job, level)
    }

    fn walls() -> Vec<FoundWall> {
        // A partition 4 1/2 in thick with a doorway, and an outside wall seen from one side with a window.
        vec![
            FoundWall { from: [18.0, -0.25], to: [18.0, 20.25], thickness: 0.375, measured: true, openings: vec![FoundOpening { window: false, from: 9.25, to: 12.25, sill: 0.0, head: 6.67 }] },
            FoundWall { from: [30.5, -0.25], to: [-0.25, -0.25], thickness: 0.5, measured: false, openings: vec![FoundOpening { window: true, from: 18.0, to: 23.0, sill: 3.0, head: 7.0 }] },
        ]
    }

    #[test]
    fn found_walls_are_laid_assumed_from_the_scan_with_the_models_own_wall_door_and_window() {
        let (mut job, level) = job_with_a_level();
        let how = Entered { status: Status::Assumed, source: "test" };
        let outside = editing::wall_type(&mut job, "Existing outside wall", "6 in", how).unwrap();
        let made = lay_found(&mut job, &level, "house", 8.0, Some(&outside), &walls()).unwrap();
        assert_eq!((made.walls.len(), made.doors.len(), made.windows.len()), (2, 1, 1));
        // Each wall's line is Assumed and says where it came from; none is Known.
        let mut q = job.world.query::<(&crate::Node, &crate::Wall)>();
        let all: Vec<(String, Status, String)> = q.iter(&job.world).map(|(n, w)| (n.path.clone(), w.line.status, w.line.source.clone())).collect();
        assert_eq!(all.len(), 2);
        assert!(all.iter().all(|(_, s, src)| *s == Status::Assumed && src == "found in scan house"), "{all:?}");
        // The partition's type is one the program made, 4 1/2 in.
        assert!(made.types.iter().any(|(_, name)| name == "wall 4.5 in"), "{:?}", made.types);
        crate::building::check_whole(&mut job).unwrap();
    }

    #[test]
    fn laying_the_same_scan_twice_on_a_level_is_refused_and_changes_nothing() {
        let (mut job, level) = job_with_a_level();
        let how = Entered { status: Status::Assumed, source: "test" };
        let outside = editing::wall_type(&mut job, "Existing outside wall", "6 in", how).unwrap();
        lay_found(&mut job, &level, "house", 8.0, Some(&outside), &walls()).unwrap();
        let before = crate::file::save(&mut job);
        let why = lay_found(&mut job, &level, "house", 8.0, Some(&outside), &walls()).unwrap_err();
        assert!(why.contains("already has 2 walls found in scan house"), "{why}");
        assert_eq!(crate::file::save(&mut job), before);
    }

    #[test]
    fn a_wall_seen_from_one_side_with_no_type_named_is_refused() {
        let (mut job, level) = job_with_a_level();
        let before = crate::file::save(&mut job);
        let why = lay_found(&mut job, &level, "house", 8.0, None, &walls()).unwrap_err();
        assert!(why.contains("needs the wall type"), "{why}");
        assert_eq!(crate::file::save(&mut job), before);
    }
}
