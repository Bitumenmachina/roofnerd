//! Plans under the tracing, in the window: a drawing's page read off its file, shown
//! flat under the storey it is a plan of, and its own lines handed to the snapping.
//!
//! A page is read on its own thread: a scanned sheet takes seconds, and the window keeps
//! drawing while it is read. What the file could not give is said when it arrives.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use plumbline_world::editing as ed;
use plumbline_world::plans::{self as world_plans, Drawing};
use plumbline_world::value::Status;
use plumbline_world::Node as Element;

use crate::model::{Doc, Tone};

/// How big one unit of a page with no scale is shown, in feet, so there is something to
/// pick the two ends of a dimension on. It says nothing about the building. A tenth of a
/// foot, so a pick, which is kept to a ten-thousandth of a foot, is a place on the page
/// to a thousandth of a unit: a line's end picked is that end.
pub const SHOWN_UNSCALED_FT: f64 = 0.1;

/// A drawing in the job, as the window needs it.
#[derive(Clone, Debug, PartialEq)]
pub struct Laid {
    pub path: String,
    pub name: String,
    pub storey: String,
    pub drawing: Drawing,
}

impl Laid {
    /// Feet to one unit of the page as it is shown, and where the page's lower-left
    /// corner is shown, on the plan. With no scale each unit is shown a fixed size.
    pub fn shown(&self) -> (f64, [f64; 2]) {
        match (self.drawing.feet_per_unit(), self.drawing.to_plan([0.0, 0.0])) {
            (Some(k), Some(corner)) => (k, corner),
            // With no scale the page's lower-left corner is the plan's (0, 0).
            _ => (SHOWN_UNSCALED_FT, [0.0, 0.0]),
        }
    }
    /// A place on the plan as a place on the page, as shown.
    /// To a millionth of a unit: the division leaves a last digit that is not on the
    /// page (126 comes back as 125.99999999999999).
    pub fn on_page(&self, plan: [f64; 2]) -> [f64; 2] {
        let (k, corner) = self.shown();
        let tidy = |x: f64| (x * 1.0e6).round() / 1.0e6 + 0.0;
        [tidy((plan[0] - corner[0]) / k), tidy((plan[1] - corner[1]) / k)]
    }
    pub fn on_plan(&self, page: [f64; 2]) -> [f64; 2] {
        let (k, corner) = self.shown();
        [corner[0] + page[0] * k, corner[1] + page[1] * k]
    }
    /// The drawing's scale as the model says it (`Drawing::scale_said`).
    pub fn scale_said(&self) -> Option<String> {
        self.drawing.scale_said()
    }
    /// Whether a place on the plan lies on the page as shown.
    pub fn holds(&self, plan: [f64; 2]) -> bool {
        let p = self.on_page(plan);
        p[0] >= 0.0 && p[1] >= 0.0 && p[0] <= self.drawing.size[0] && p[1] <= self.drawing.size[1]
    }
}

/// Every drawing in the job, in the job's order.
pub fn laid(job: &mut plumbline_world::Job) -> Vec<Laid> {
    let order = job.order.clone();
    order
        .iter()
        .filter_map(|e| {
            let node = job.world.get::<Element>(*e)?;
            let drawing = job.world.get::<Drawing>(*e)?;
            Some(Laid { path: node.path.clone(), name: node.name.clone(), storey: node.parent.clone().unwrap_or_default(), drawing: drawing.clone() })
        })
        .collect()
}

/// Where a drawing lies on the plan: its two opposite corners, in feet.
pub fn corners_of(job: &mut plumbline_world::Job, path: &str) -> Option<[[f64; 2]; 2]> {
    laid(job).into_iter().find(|l| l.path == path).map(|l| [l.on_plan([0.0, 0.0]), l.on_plan(l.drawing.size)])
}

/// A drawing's page as read.
pub struct Read {
    /// The file and page it was read from: a drawing pointed at another is read again.
    key: (String, usize),
    pub image: Handle<Image>,
    /// The straight lines on the page, in the page's units from its lower-left corner.
    pub lines: Vec<[[f64; 2]; 2]>,
    pub lost: Vec<String>,
    /// The file is not the one the drawing was made from.
    pub changed: bool,
    /// Where a scan's sheet shows a wall's face (call 190): the straight lines its points make,
    /// 2 ft long or more, in the page's units from its lower-left corner. None for a sheet that is
    /// not a scan's.
    pub faces: Vec<plumbline_plans::faces::Face>,
}

type Arrived = Result<(plumbline_plans::Page, bool, Vec<plumbline_plans::faces::Face>), String>;

/// The faces a scan's sheet shows, in the page's own units. The picture the page reader hands over
/// may be smaller than the page (a sheet over 4,096 dots a side is shrunk), so the dots to the foot the
/// faces are read at are the picture's own, and what is read is carried back to the page's units
/// (call 190). `feet_per_unit` is the drawing's scale.
pub fn scan_faces(page: &plumbline_plans::Page, feet_per_unit: f64) -> Vec<plumbline_plans::faces::Face> {
    let (pw, ph) = (f64::from(page.picture.width), f64::from(page.picture.height));
    if !(feet_per_unit > 0.0) || pw < 1.0 || ph < 1.0 || page.size[0] <= 0.0 || page.size[1] <= 0.0 {
        return Vec::new();
    }
    // Picture dots to the foot, along the page's width.
    let (kx, ky) = (page.size[0] / pw, page.size[1] / ph);
    let dots_per_foot = 1.0 / (feet_per_unit * kx);
    let mut faces = plumbline_plans::faces::faces_rgba(&page.picture.rgba, page.picture.width, page.picture.height, dots_per_foot);
    for f in &mut faces {
        for end in &mut f.line {
            *end = [end[0] * kx, end[1] * ky];
        }
        if let Some((_, gap)) = f.pair.as_mut() {
            // How far apart two faces lie, in page units, along the sheet's width.
            *gap *= kx;
        }
    }
    faces
}

#[derive(Resource, Default)]
pub struct Plans {
    pub read: HashMap<String, Read>,
    /// Why a drawing's page could not be read, by the drawing's path.
    pub faults: HashMap<String, ((String, usize), String)>,
    reading: HashMap<String, ((String, usize), Mutex<Receiver<Arrived>>)>,
}

impl Plans {
    pub fn is_reading(&self, drawing: &str) -> bool {
        self.reading.contains_key(drawing)
    }
}

/// A picture's smaller copies, each half the one before down to one pixel, one after the
/// other, and how many pictures that makes with the first. Each pixel of a copy is the
/// mean of the four (or two) it is made of.
fn smaller_copies(rgba: &[u8], width: u32, height: u32) -> (Vec<u8>, u32) {
    let (mut w, mut h) = (width as usize, height as usize);
    let mut last: Vec<u8> = Vec::new();
    let mut out = Vec::new();
    let mut levels = 1;
    while w > 1 || h > 1 {
        let from: &[u8] = if levels == 1 { rgba } else { &last };
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            let (y0, y1) = ((2 * y).min(h - 1), (2 * y + 1).min(h - 1));
            for x in 0..nw {
                let (x0, x1) = ((2 * x).min(w - 1), (2 * x + 1).min(w - 1));
                for c in 0..4 {
                    let sum = u32::from(from[(y0 * w + x0) * 4 + c]) + u32::from(from[(y0 * w + x1) * 4 + c]) + u32::from(from[(y1 * w + x0) * 4 + c]) + u32::from(from[(y1 * w + x1) * 4 + c]);
                    next[(y * nw + x) * 4 + c] = ((sum + 2) / 4) as u8;
                }
            }
        }
        out.extend_from_slice(&next);
        last = next;
        (w, h) = (nw, nh);
        levels += 1;
    }
    (out, levels)
}

/// The folder a job's plan files are found from.
pub fn folder_of(job: &Path) -> PathBuf {
    job.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf()
}

/// Start reading the page of each drawing that has not been read, and take in each page
/// that has arrived.
pub fn read(mut doc: ResMut<Doc>, mut plans: ResMut<Plans>, mut images: ResMut<Assets<Image>>, mut seen: Local<u64>) {
    // Pages that have arrived.
    let mut arrived: Vec<(String, (String, usize), Arrived)> = Vec::new();
    for (path, (key, waiting)) in &plans.reading {
        // Not yet is not the same as never: a reader that stopped without sending its
        // page (a file that broke it) is a fault to be said, or the drawing would read
        // "reading its page" for the rest of the day (R21-20).
        use std::sync::mpsc::TryRecvError;
        let stopped = || Err("the page could not be read: reading it stopped part way. The file may be damaged; print it to a new PDF, or save the picture again, and add it again".to_string());
        match waiting.lock().map(|r| r.try_recv()) {
            Ok(Ok(result)) => arrived.push((path.clone(), key.clone(), result)),
            Ok(Err(TryRecvError::Empty)) => {}
            Ok(Err(TryRecvError::Disconnected)) | Err(_) => arrived.push((path.clone(), key.clone(), stopped())),
        }
    }
    for (path, key, result) in arrived {
        plans.reading.remove(&path);
        match result {
            Ok((page, changed, faces)) => {
                // The sheet with its smaller copies after it, so a whole page seen small
                // keeps its thin lines.
                let (width, height) = (page.picture.width, page.picture.height);
                // A scan of a big house is a few points to a square foot of a sheet that is
                // hundreds of feet across: each point is thickened with the sheet's size, so
                // that what was brought in is seen and not an empty sheet (call 178).
                let mut rgba = page.picture.rgba;
                let size_ft = laid(&mut doc.job).iter().find(|l| l.path == path && l.drawing.file.source.starts_with("scan ")).map(|l| l.drawing.size[0].max(l.drawing.size[1]));
                if let Some(ft) = size_ft {
                    thicken(&mut rgba, width as usize, height as usize, ((ft / 12.0).floor() as usize).min(16));
                }
                let (smaller, levels) = smaller_copies(&rgba, width, height);
                let mut image = Image::new(Extent3d { width, height, depth_or_array_layers: 1 }, TextureDimension::D2, rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
                if let Some(data) = image.data.as_mut() {
                    data.extend_from_slice(&smaller);
                    image.texture_descriptor.mip_level_count = levels;
                }
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear, mipmap_filter: ImageFilterMode::Linear, ..default() });
                // "The drawing" first: a drawing is named for its file, and a sentence that
                // began with the name gave it a capital it does not have ("Usda-7213").
                let mut said = format!("The drawing {path}: its page is read ({})", if page.lines.is_empty() { "it is a scan: there are no drawn lines on it to snap to".to_string() } else { format!("{} drawn lines to snap to", page.lines.len()) });
                if !page.lost.is_empty() {
                    said.push_str(&format!("; {} not read from it: select the drawing to see what", if page.lost.len() == 1 { "1 thing".to_string() } else { format!("{} things", page.lost.len()) }));
                }
                if changed {
                    said.push_str("; ITS FILE HAS CHANGED since the drawing was made from it");
                }
                // A scan's sheet read says nothing of its own: what the scan came to was
                // said when it was brought in, and is not to be covered by this (call 161).
                let from_scan = laid(&mut doc.job).iter().any(|l| l.path == path && l.drawing.file.source.starts_with("scan "));
                if changed || !from_scan {
                    doc.say(if changed { Tone::Refused } else { Tone::Done }, said);
                }
                // What the scan's sheet shows to trace over, said once, on the card that brought it in
                // if that is still up (call 190): it is the card a person is reading just then.
                if from_scan {
                    let name = laid(&mut doc.job).iter().find(|l| l.path == path).map(|l| l.name.clone()).unwrap_or_default();
                    let head = format!("Brought in the scan {name:?}");
                    if doc.message.starts_with(&head) {
                        let line = match faces.len() {
                            0 => "No straight line 2 ft long or more shows on its sheet: trace it by eye.".to_string(),
                            1 => "Its sheet shows 1 straight line 2 ft long or more, for the Wall tool to land on.".to_string(),
                            n => format!("Its sheet shows {n} straight lines 2 ft long or more, for the Wall tool to land on."),
                        };
                        if !doc.message.contains(&line) {
                            doc.message.push('\n');
                            doc.message.push_str(&line);
                        }
                    }
                }
                plans.read.insert(path, Read { key, image: images.add(image), lines: page.lines, lost: page.lost, changed, faces });
            }
            Err(why) => {
                doc.say(Tone::Refused, format!("{path}: its page cannot be shown: {why}"));
                plans.faults.insert(path, (key, why));
            }
        }
        doc.redraw();
    }
    // Drawings not yet read, looked for whenever the job has changed.
    if *seen == doc.revision {
        return;
    }
    *seen = doc.revision;
    let folder = folder_of(&doc.path);
    let all = laid(&mut doc.job);
    plans.read.retain(|path, r| all.iter().any(|l| l.path == *path && r.key == (l.drawing.file.value.clone().unwrap_or_default(), l.drawing.page)));
    plans.faults.retain(|path, (key, _)| all.iter().any(|l| l.path == *path && *key == (l.drawing.file.value.clone().unwrap_or_default(), l.drawing.page)));
    for l in all {
        let key = (l.drawing.file.value.clone().unwrap_or_default(), l.drawing.page);
        if plans.read.contains_key(&l.path) || plans.faults.contains_key(&l.path) || plans.reading.contains_key(&l.path) {
            continue;
        }
        let (file, page, bytes, checksum, size) = (folder.join(&key.0), key.1, l.drawing.bytes, l.drawing.checksum.clone(), l.drawing.size);
        // A scan's sheet has its faces read on this thread too (call 190).
        let scan_scale = if l.drawing.file.source.starts_with("scan ") { l.drawing.feet_per_unit() } else { None };
        let (send, receive) = channel::<Arrived>();
        std::thread::spawn(move || {
            let result = std::fs::read(&file).map_err(|e| format!("{} {}. Put it back beside the job, or take this drawing out and lay the file again from where it is now", file.file_name().and_then(|n| n.to_str()).unwrap_or("the file"), why_not_opened(&e))).and_then(|data| {
                let changed = data.len() as u64 != bytes || plumbline_plans::checksum(&data) != checksum;
                plumbline_plans::read(&data, page).and_then(|p| {
                    // A drawing made at one size and a page of another do not fit: the
                    // sheet would lie wrong under what was traced over it.
                    if (p.size[0] - size[0]).abs() > 0.01 || (p.size[1] - size[1]).abs() > 0.01 {
                        return Err(format!("its page is {} by {} in the file, and the drawing was made at {} by {}", p.size[0], p.size[1], size[0], size[1]));
                    }
                    let faces = scan_scale.map(|k| scan_faces(&p, k)).unwrap_or_default();
                    Ok((p, changed, faces))
                })
            });
            let _ = send.send(result);
        });
        plans.reading.insert(l.path, (key, Mutex::new(receive)));
    }
}

/// Why a file could not be opened, in words: the machine's own are not shown.
pub fn why_not_opened(e: &std::io::Error) -> &'static str {
    match e.kind() {
        std::io::ErrorKind::NotFound => "is not there",
        std::io::ErrorKind::PermissionDenied => "is there, and this program is not allowed to open it",
        _ => "could not be opened",
    }
}

/// A plan file named in the window ("plans/a-1.pdf" or "plans/a-1.pdf, 2"), or dropped
/// on it: laid under the storey on show as a drawing with no scale yet. The file is read
/// here once, for its page's size; its picture arrives a moment later.
pub fn add(doc: &mut Doc, text: &str) {
    // With the whole building on show, the drawing goes under its lowest level.
    if doc.show.storey.is_none() {
        doc.show.storey = plumbline_world::building::storey_totals(&mut doc.job).into_iter().min_by(|a, b| a.elevation.unwrap_or(f64::MAX).total_cmp(&b.elevation.unwrap_or(f64::MAX))).map(|s| s.path);
    }
    let Some(storey) = doc.show.storey.clone() else {
        crate::tools::cant(doc, "add the drawing", "this job has no level for it to be a plan of. Add a level first.");
        doc.touch();
        return;
    };
    if !doc.plan {
        doc.go = Some((true, false));
    }
    let (file, page) = match text.rsplit_once(',') {
        Some((file, page)) if page.trim().parse::<usize>().is_ok() => (file.trim(), page.trim().parse::<usize>().unwrap_or(1)),
        _ => (text.trim(), 1),
    };
    let made = (|| -> Result<String, String> {
        if file.is_empty() {
            return Err("name the plan's file: a PDF, a PNG or a JPEG".into());
        }
        // A name that is not a whole path is looked for from the job's own folder, where
        // a plan is kept beside its job; then from where the program was started.
        let beside = folder_of(&doc.path).join(file);
        let found = if Path::new(file).is_absolute() || !beside.exists() { Path::new(file).to_path_buf() } else { beside };
        let bytes = std::fs::read(&found).map_err(|e| format!("{file} {}. Name it as it lies from the job's folder, like plans/a-1.pdf", why_not_opened(&e)))?;
        let file = found.to_str().ok_or("the file's name has letters this cannot use")?;
        // Only what the drawing needs to be made: the page's size. Its picture is read
        // on a thread of its own, so the window keeps drawing.
        let (_, pages, size) = plumbline_plans::size_of(&bytes, page).map_err(|why| format!("{file}: {why}"))?;
        let kept = from_folder(&folder_of(&doc.path), Path::new(file))?;
        let name = Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or("drawing").to_string();
        let name = if pages > 1 { format!("{name}, page {page}") } else { name };
        // Which file it is is a fact, read off the disk here; where it lies and its
        // scale are not, and say so themselves.
        let how = ed::Entered { status: Status::Known, source: "the file laid under the plan on screen" };
        let path = world_plans::add(&mut doc.job, &storey, &name, world_plans::Read { file: &kept, page, bytes: bytes.len() as u64, checksum: &plumbline_plans::checksum(&bytes), size }, how)?;
        let twice = laid(&mut doc.job).iter().filter(|l| l.storey == storey && l.drawing.file.value.as_deref() == Some(kept.as_str()) && l.drawing.page == page).count() > 1;
        let _ = path;
        Ok(format!("Added drawing {name:?}. It has no scale yet: click Set scale, then the two ends of one dimension on it.{}", if twice { " This page of this file was already under this level: there are two of it now." } else { "" }))
    })();
    match made {
        Ok(said) => {
            doc.edit("added a drawing");
            doc.framed = false;
            doc.frame_on = laid(&mut doc.job).last().map(|l| l.path.clone()).and_then(|path| corners_of(&mut doc.job, &path));
            doc.say(Tone::Done, said);
        }
        Err(why) => crate::tools::cant(doc, "add the drawing", why),
    }
    doc.redraw();
}

/// A file's path from a folder, with `..` where it has to climb.
pub fn from_folder(folder: &Path, file: &Path) -> Result<String, String> {
    use std::path::Component;
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

/// A file dropped on the window is a plan to lay under the storey on show.
pub fn dropped(mut doc: ResMut<Doc>, mut drops: MessageReader<FileDragAndDrop>) {
    for drop in drops.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = drop {
            if let Some(file) = path_buf.to_str() {
                crate::bring::dropped(&mut doc, file);
            }
        }
    }
}

/// The drawing a pick on the plan is on: the last one laid under the storey whose page,
/// as shown, holds the point.
pub fn under(doc: &mut Doc, storey: &str, at: [f64; 2]) -> Option<Laid> {
    laid(&mut doc.job).into_iter().rfind(|l| l.storey == storey && l.holds(at))
}

/// How something drawn at a place on a storey is entered. On a drawing it is traced:
/// Assumed, and saying which drawing, the one the place is on (the last laid, where two
/// lie over one another). Off every sheet it is drawn in the window. While any drawing
/// under the storey has no scale, nothing is drawn on the storey at all.
pub fn source_for(doc: &mut Doc, storey: &str, at: [f64; 2]) -> Result<String, String> {
    let here: Vec<Laid> = laid(&mut doc.job).into_iter().filter(|l| l.storey == storey).collect();
    if let Some(l) = here.iter().find(|l| l.drawing.feet_per_unit().is_none()) {
        return Err(format!("{} has no scale: set it from one dimension before tracing over it", l.name));
    }
    match here.iter().rev().find(|l| l.holds(at)) {
        Some(l) => world_plans::traced_from(&mut doc.job, &l.path),
        None => Ok(crate::model::SOURCE.to_string()),
    }
}

/// Each dark point of a picture made `r` dots thicker each way.
fn thicken(rgba: &mut [u8], width: usize, height: usize, r: usize) {
    if r == 0 {
        return;
    }
    // One stamp for each dot: at its upper left corner.
    let is_dark = |rgba: &[u8], x: usize, y: usize| rgba[(y * width + x) * 4] < 128;
    let dark: Vec<(usize, usize)> = (0..height).flat_map(|y| (0..width).map(move |x| (x, y))).filter(|(x, y)| is_dark(rgba, *x, *y) && (*x == 0 || !is_dark(rgba, x - 1, *y)) && (*y == 0 || !is_dark(rgba, *x, y - 1))).collect();
    for (x, y) in dark {
        for yy in y.saturating_sub(r)..=(y + r).min(height - 1) {
            for xx in x.saturating_sub(r)..=(x + r).min(width - 1) {
                let at = (yy * width + xx) * 4;
                rgba[at..at + 3].copy_from_slice(&[40, 40, 40]);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// A wall traced over a scan, set by its face (DECISIONS.md call 190)
//
// A scan shows the face of a wall, not its centre. The clicks of a wall are carried to the faces the
// scan's sheet shows (`SheetFace`, from `Read::faces`), and the wall is laid with its centre half its
// thickness to the side away from the room, or, where the sheet shows both faces of it, between them.
// What a line of clicks becomes when carried out is `plumbline_world::editing::carry::carried_out`; what
// is here is which face each stretch of the clicks is on, and where two faces meet.
// ---------------------------------------------------------------------------------------------

type P2 = [f64; 2];

/// A click is on a face when it lies within this many feet of the face's line. Assumed (the lead's call
/// 190 reads a face to 1/16 in on a made-up scan and a real phone scan is out by a few inches).
pub const ON_FACE_FT: f64 = 0.25;
/// Two faces that cross within this many feet of the click that joins them are one corner, and the click
/// is carried to where they cross. Assumed.
pub const CORNER_REACH_FT: f64 = 1.5;
/// Two stretches whose faces run nearer parallel than this (the sine of the angle between them) are not a
/// corner. About 10 degrees. Assumed.
pub const NOT_PARALLEL: f64 = 0.17;

/// A face of a scan's sheet, laid on the plan in feet.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetFace {
    /// From its west end to its east, or from its south end to its north.
    pub line: [P2; 2],
    /// Some(true): the room is to the left of `line[0]` to `line[1]`; Some(false): to its right; None: the
    /// sheet cannot tell.
    pub room: Option<bool>,
    /// The other face of the same wall: its place in the list of faces and how far apart the two are, in feet.
    pub pair: Option<(usize, f64)>,
}

/// The faces of one scan's sheet, on the plan.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetFaces {
    /// The drawing's path and its name.
    pub path: String,
    pub name: String,
    pub faces: Vec<SheetFace>,
}

/// A drawing's faces laid on the plan, if it is a scan's.
pub fn sheet_faces(l: &Laid, read: &Read) -> Option<SheetFaces> {
    if !l.drawing.file.source.starts_with("scan ") {
        return None;
    }
    let (k, _) = l.shown();
    let faces = read.faces.iter().map(|f| SheetFace { line: [l.on_plan(f.line[0]), l.on_plan(f.line[1])], room: f.room, pair: f.pair.map(|(j, gap)| (j, gap * k)) }).collect();
    Some(SheetFaces { path: l.path.clone(), name: l.name.clone(), faces })
}

/// Whether a drawing is a scan's sheet.
pub fn is_scan(l: &Laid) -> bool {
    l.drawing.file.source.starts_with("scan ")
}

/// The scan under a place on a storey: the last one laid whose page holds it.
pub fn scan_under(doc: &mut Doc, storey: &str, at: P2) -> Option<Laid> {
    under(doc, storey, at).filter(is_scan)
}

fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn len(a: P2) -> f64 {
    a[0].hypot(a[1])
}
fn unit(a: P2, b: P2) -> P2 {
    let d = sub(b, a);
    let n = len(d);
    if n == 0.0 {
        [1.0, 0.0]
    } else {
        [d[0] / n, d[1] / n]
    }
}

impl SheetFace {
    fn dir(&self) -> P2 {
        unit(self.line[0], self.line[1])
    }
    /// How far a point lies to the left of the face's line (minus: to its right).
    fn left_of(&self, p: P2) -> f64 {
        cross(self.dir(), sub(p, self.line[0]))
    }
    /// A point carried square onto the face's line.
    fn onto(&self, p: P2) -> P2 {
        let d = self.dir();
        let t = dot(sub(p, self.line[0]), d);
        [self.line[0][0] + d[0] * t, self.line[0][1] + d[1] * t]
    }
    /// How far along the face's line, from its first end, a point lies.
    fn along(&self, p: P2) -> f64 {
        dot(sub(p, self.line[0]), self.dir())
    }
    fn length(&self) -> f64 {
        len(sub(self.line[1], self.line[0]))
    }
    /// How far a point is from the face itself (its ends count), in feet.
    pub fn distance(&self, p: P2) -> f64 {
        let t = self.along(p).clamp(0.0, self.length());
        let d = self.dir();
        len(sub(p, [self.line[0][0] + d[0] * t, self.line[0][1] + d[1] * t]))
    }
}

/// Whether a click lies on a face of the sheet.
pub fn on_a_face(faces: &[SheetFace], p: P2) -> bool {
    faces.iter().any(|f| f.distance(p) <= ON_FACE_FT)
}

/// The face the stretch from `p` to `q` runs along: both its ends lie within `ON_FACE_FT` of the face's
/// line and one of them over the face itself. Of those, the nearest, then the longer.
fn face_of_stretch(faces: &[SheetFace], p: P2, q: P2) -> Option<usize> {
    let mut best: Option<(usize, f64, f64)> = None;
    for (i, f) in faces.iter().enumerate() {
        // Where the sheet shows both faces of a wall, a click on either is on the wall.
        let other = f.pair.and_then(|(j, _)| faces.get(j));
        let off = |x: P2| other.map_or(f.left_of(x).abs(), |o| f.left_of(x).abs().min(o.left_of(x).abs()));
        let (dp, dq) = (off(p), off(q));
        if dp > ON_FACE_FT || dq > ON_FACE_FT {
            continue;
        }
        let over = |t: f64| t >= -ON_FACE_FT && t <= f.length() + ON_FACE_FT;
        if !(over(f.along(p)) || over(f.along(q))) {
            continue;
        }
        let (near, long) = (dp + dq, f.length());
        // Nearer by more than a half inch; then the longer.
        let better = match best {
            None => true,
            Some((_, n, l)) => near < n - 0.04 || (near <= n + 0.04 && long > l),
        };
        if better {
            best = Some((i, near, long));
        }
    }
    best.map(|b| b.0)
}

/// How much of the stretch from `p` to `q` has no face of the sheet beside it, in feet: its length less what the
/// faces it runs along cover of it. A face is one the stretch runs along as `face_of_stretch` takes it (both ends within
/// `ON_FACE_FT` of its line, or of its pair's); what it covers is where it lies along the stretch. A doorway in a
/// partition is such a gap, and so is a stretch the scanner did not see (call 193, F).
pub fn bare_length(faces: &[SheetFace], p: P2, q: P2) -> f64 {
    let total = len(sub(q, p));
    if total < 1e-9 {
        return 0.0;
    }
    let mut spans: Vec<[f64; 2]> = Vec::new();
    for f in faces {
        let other = f.pair.and_then(|(j, _)| faces.get(j));
        let off = |x: P2| other.map_or(f.left_of(x).abs(), |o| f.left_of(x).abs().min(o.left_of(x).abs()));
        if off(p) > ON_FACE_FT || off(q) > ON_FACE_FT {
            continue;
        }
        // The face's two ends along the stretch, as fractions of it.
        let (ap, aq) = (f.along(p), f.along(q));
        if (aq - ap).abs() < 1e-9 {
            continue;
        }
        let (u0, u1) = ((0.0 - ap) / (aq - ap), (f.length() - ap) / (aq - ap));
        let (lo, hi) = (u0.min(u1).max(0.0), u0.max(u1).min(1.0));
        if hi > lo {
            spans.push([lo, hi]);
        }
    }
    spans.sort_by(|x, y| x[0].total_cmp(&y[0]));
    let (mut covered, mut reach) = (0.0, 0.0_f64);
    for [lo, hi] in spans {
        let lo = lo.max(reach);
        if hi > lo {
            covered += hi - lo;
            reach = hi;
        }
    }
    (total * (1.0 - covered)).max(0.0)
}

/// Where two faces' lines cross, if they are not parallel.
fn crossing(a: &SheetFace, b: &SheetFace) -> Option<P2> {
    let (da, db) = (a.dir(), b.dir());
    let s = cross(da, db);
    if s.abs() < NOT_PARALLEL {
        return None;
    }
    let t = cross(sub(b.line[0], a.line[0]), db) / s;
    Some([a.line[0][0] + da[0] * t, a.line[0][1] + da[1] * t])
}

/// How far, in feet, a line's free end may be carried to meet another face (a partition's end that the scan
/// stops short of the wall it meets). Assumed.
pub const TEE_REACH_FT: f64 = 0.6;

/// Where the end of a stretch on face `f`, at `p`, meets another face that crosses it: the crossing of
/// `f`'s line with the nearest face that is not parallel to it (nor its own pair), if that crossing lies on
/// that face and within `TEE_REACH_FT` of `p`.
fn tee(faces: &[SheetFace], f: usize, p: P2) -> Option<P2> {
    let own = &faces[f];
    let mut best: Option<(f64, P2)> = None;
    for (g, other) in faces.iter().enumerate() {
        if g == f || own.pair.is_some_and(|(j, _)| j == g) {
            continue;
        }
        let Some(x) = crossing(own, other) else { continue };
        let reach = len(sub(x, p));
        // On the other face itself, to a few inches past its ends.
        let on_it = other.along(x) >= -ON_FACE_FT && other.along(x) <= other.length() + ON_FACE_FT;
        if on_it && reach <= TEE_REACH_FT && best.is_none_or(|b| reach < b.0) {
            best = Some((reach, x));
        }
    }
    best.map(|b| b.1)
}

/// Where two lines (each by its two points) meet, if they are not parallel.
fn meet(a: [P2; 2], b: [P2; 2]) -> Option<P2> {
    let (da, db) = (unit(a[0], a[1]), unit(b[0], b[1]));
    let s = cross(da, db);
    if s.abs() < NOT_PARALLEL {
        return None;
    }
    let t = cross(sub(b[0], a[0]), db) / s;
    Some([a[0][0] + da[0] * t, a[0][1] + da[1] * t])
}

/// What the sheet showed for one stretch of the clicks.
#[derive(Clone, Debug, PartialEq)]
pub enum Seen {
    /// The stretch is on one face. `room`: which side the room is on, as seen running along the
    /// stretch (Some(true): to its left); None where the sheet cannot tell.
    Face { room: Option<bool> },
    /// The stretch is on a face and the sheet shows the wall's other face too, this far apart in feet.
    Both { gap: f64 },
    /// No face is near: the stretch stays where the hand put it.
    Nothing,
}

/// A line of clicks laid by the faces under it.
#[derive(Clone, Debug, PartialEq)]
pub struct ByFace {
    /// The centre line of the wall of each stretch of the clicks (a ring's last runs back to its first).
    pub lines: Vec<[P2; 2]>,
    pub seen: Vec<Seen>,
    /// The side the bodies lie on, for each stretch that has one: true is the left of the way it runs.
    pub body_left: Vec<Option<bool>>,
}

/// `clicks` (a line of them, or with `closed` a ring) laid by the faces: each stretch carried onto the
/// face it runs along, each corner where two faces cross, and then out by half the `thickness` to the
/// side away from the room (to the other side with `flip`), or, where both faces of a wall are shown,
/// to between them. A stretch with no face under it is as clicked. One line of stretches carried the
/// same way is carried out together, so that each corner is where the two walls' faces meet.
pub fn by_face(clicks: &[P2], closed: bool, faces: &[SheetFace], thickness: f64, flip: bool) -> ByFace {
    let n = clicks.len();
    if n < 2 {
        return ByFace { lines: Vec::new(), seen: Vec::new(), body_left: Vec::new() };
    }
    let closed = closed && n >= 3;
    let stretches = if closed { n } else { n.saturating_sub(1) };
    let at = |i: usize| clicks[i % n];
    let on: Vec<Option<usize>> = (0..stretches).map(|i| face_of_stretch(faces, at(i), at(i + 1))).collect();
    // Where each stretch starts and ends: carried onto its face's line, and at a corner where the face
    // before and the face after cross near the click.
    let mut start: Vec<P2> = (0..stretches).map(|i| on[i].map_or(at(i), |f| faces[f].onto(at(i)))).collect();
    let mut end: Vec<P2> = (0..stretches).map(|i| on[i].map_or(at(i + 1), |f| faces[f].onto(at(i + 1)))).collect();
    let mut joined = vec![false; stretches];
    for j in 0..n {
        // The corner at click j: between stretch j-1 and stretch j.
        let (before, after) = if j == 0 { (if closed { Some(stretches - 1) } else { None }, 0) } else if j == n - 1 && !closed { (Some(j - 1), usize::MAX) } else { (Some(j - 1), j) };
        let (Some(b), true) = (before, after != usize::MAX) else { continue };
        let (Some(fb), Some(fa)) = (on[b], on[after]) else { continue };
        if fb == fa {
            // One face all the way: the stretches meet on its line.
            let m = faces[fb].onto(at(j));
            end[b] = m;
            start[after] = m;
            joined[b] = true;
            continue;
        }
        if let Some(x) = crossing(&faces[fb], &faces[fa]) {
            if len(sub(x, at(j))) <= CORNER_REACH_FT {
                end[b] = x;
                start[after] = x;
                joined[b] = true;
                continue;
            }
        }
        // Faces that do not cross near the click: the two ends are as near as they are; where they are
        // within a few inches of one another (a face cut by a doorway), they are one point.
        if len(sub(end[b], start[after])) <= 0.05 {
            let m = [(end[b][0] + start[after][0]) / 2.0, (end[b][1] + start[after][1]) / 2.0];
            end[b] = m;
            start[after] = m;
            joined[b] = true;
        }
    }
    // The free ends of a line of clicks that run to a wall the scan stops short of are carried to it.
    if !closed {
        if let Some(f) = on[0] {
            if let Some(x) = tee(faces, f, start[0]) {
                start[0] = x;
            }
        }
        if let Some(f) = on[stretches - 1] {
            if let Some(x) = tee(faces, f, end[stretches - 1]) {
                end[stretches - 1] = x;
            }
        }
    }
    // The way each stretch is carried: by how much, and to which side.
    let mut seen = Vec::with_capacity(stretches);
    let mut carry: Vec<(f64, bool)> = Vec::with_capacity(stretches);
    let mut body_left = Vec::with_capacity(stretches);
    for i in 0..stretches {
        let Some(f) = on[i] else {
            seen.push(Seen::Nothing);
            carry.push((0.0, true));
            body_left.push(None);
            continue;
        };
        let face = &faces[f];
        let run = unit(start[i], end[i]);
        // Whether the face's own way of running is this stretch's.
        let same = dot(face.dir(), run) >= 0.0;
        if let Some((g, gap)) = face.pair.filter(|(g, _)| *g < faces.len()) {
            // The centre goes between the two faces: half the distance toward the other.
            let middle = [(faces[g].line[0][0] + faces[g].line[1][0]) / 2.0, (faces[g].line[0][1] + faces[g].line[1][1]) / 2.0];
            let toward_left = face.left_of(middle) > 0.0;
            let left = if same { toward_left } else { !toward_left };
            seen.push(Seen::Both { gap });
            carry.push((gap / 2.0, left));
            body_left.push(None);
        } else {
            let room_left = face.room.map(|r| if same { r } else { !r });
            // The body is on the side away from the room; where the sheet cannot tell, to the right of the
            // way the stretch runs (outward of a ring clicked counter-clockwise).
            let left = !room_left.unwrap_or(true) != flip;
            seen.push(Seen::Face { room: room_left });
            carry.push((thickness / 2.0, left));
            body_left.push(Some(left));
        }
    }
    // One line of stretches that meet end to end and are carried the same way is carried out together.
    let together = stretches >= 2 && (0..if closed { stretches } else { stretches - 1 }).all(|i| joined[i]) && carry.iter().all(|c| (c.0 - carry[0].0).abs() < 1e-9 && c.1 == carry[0].1) && carry[0].0 > 0.0;
    if together {
        let mut ring: Vec<P2> = (0..stretches).map(|i| start[i]).collect();
        if !closed {
            ring.push(end[stretches - 1]);
        }
        let out = plumbline_world::editing::carry::carried_out(&ring, carry[0].0, carry[0].1, closed);
        let wanted = if closed { stretches } else { stretches + 1 };
        if out.len() == wanted {
            let lines = (0..stretches).map(|i| [out[i], out[(i + 1) % out.len()]]).collect();
            return ByFace { lines, seen, body_left };
        }
    }
    let mut lines: Vec<[P2; 2]> = (0..stretches)
        .map(|i| {
            let (by, left) = carry[i];
            let two = plumbline_world::editing::carry::carried_out(&[start[i], end[i]], by, left, false);
            if two.len() == 2 { [two[0], two[1]] } else { [start[i], end[i]] }
        })
        .collect();
    // Stretches carried by different amounts, or to different sides, or beside one the sheet shows no face of:
    // each corner is still where the two walls' own lines meet.
    let reach = 4.0 * carry.iter().fold(0.0_f64, |m, c| m.max(c.0)) + 0.3;
    let corners = if closed { stretches } else { stretches - 1 };
    for b in 0..corners {
        let a = (b + 1) % stretches;
        match (on[b].is_some(), on[a].is_some()) {
            (true, true) if joined[b] => {
                if let Some(x) = meet(lines[b], lines[a]) {
                    if len(sub(x, lines[b][1])) <= reach && len(sub(x, lines[a][0])) <= reach {
                        lines[b][1] = x;
                        lines[a][0] = x;
                    }
                }
            }
            // A stretch with no face beside one that has: it runs to where that wall's line ends.
            (true, false) if len(sub(lines[b][1], lines[a][0])) <= reach => lines[a][0] = lines[b][1],
            (false, true) if len(sub(lines[b][1], lines[a][0])) <= reach => lines[b][1] = lines[a][0],
            _ => {}
        }
    }
    ByFace { lines, seen, body_left }
}

#[cfg(test)]
mod by_face_tests {
    use super::*;

    fn face(a: P2, b: P2, room: Option<bool>) -> SheetFace {
        SheetFace { line: [a, b], room, pair: None }
    }

    /// The house of demos/tracew.sh as a scan's sheet shows it: 30 by 16 ft inside, the four outside
    /// walls one face each (seen from indoors), a partition with both faces, 4 1/2 in apart, 18 ft in.
    fn house() -> Vec<SheetFace> {
        let mut f = vec![
            face([0.0, 0.0], [30.0, 0.0], Some(true)),
            face([30.0, 0.0], [30.0, 16.0], Some(true)),
            face([0.0, 16.0], [30.0, 16.0], Some(false)),
            face([0.0, 0.0], [0.0, 16.0], Some(false)),
            // The partition's faces stop 2 in short of the walls they meet: the scan sees nothing in the corner.
            face([17.8125, 0.2], [17.8125, 15.8], Some(false)),
            face([18.1875, 0.2], [18.1875, 15.8], Some(true)),
        ];
        f[4].pair = Some((5, 0.375));
        f[5].pair = Some((4, 0.375));
        f
    }

    fn close(a: P2, b: P2) -> bool {
        (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
    }

    #[test]
    fn the_outside_walls_are_carried_out_by_half_the_type() {
        // Four clicks a few inches off the faces' ends, then back to the first.
        let clicks = [[0.0, 0.1], [29.9, 0.0], [30.0, 15.9], [0.1, 16.0]];
        let laid = by_face(&clicks, true, &house(), 0.5, false);
        assert_eq!(laid.lines.len(), 4);
        let corners = [[-0.25, -0.25], [30.25, -0.25], [30.25, 16.25], [-0.25, 16.25]];
        for (i, line) in laid.lines.iter().enumerate() {
            assert!(close(line[0], corners[i]) && close(line[1], corners[(i + 1) % 4]), "wall {i}: {line:?}");
        }
        assert!(laid.seen.iter().all(|s| matches!(s, Seen::Face { room: Some(true) })), "{:?}", laid.seen);
        // 30.5 + 16.5 + 30.5 + 16.5
        let length: f64 = laid.lines.iter().map(|l| (l[1][0] - l[0][0]).hypot(l[1][1] - l[0][1])).sum();
        assert!((length - 94.0).abs() < 1e-9, "{length}");
    }

    #[test]
    fn the_other_side_puts_the_bodies_inside() {
        let clicks = [[0.0, 0.0], [30.0, 0.0], [30.0, 16.0], [0.0, 16.0]];
        let laid = by_face(&clicks, true, &house(), 0.5, true);
        let corners = [[0.25, 0.25], [29.75, 0.25], [29.75, 15.75], [0.25, 15.75]];
        for (i, line) in laid.lines.iter().enumerate() {
            assert!(close(line[0], corners[i]) && close(line[1], corners[(i + 1) % 4]), "wall {i}: {line:?}");
        }
    }

    #[test]
    fn the_same_ring_clicked_clockwise_is_carried_out_too() {
        // The body is away from the room whichever way the hand goes round.
        let clicks = [[0.0, 0.0], [0.0, 16.0], [30.0, 16.0], [30.0, 0.0]];
        let laid = by_face(&clicks, true, &house(), 0.5, false);
        let corners = [[-0.25, -0.25], [-0.25, 16.25], [30.25, 16.25], [30.25, -0.25]];
        for (i, line) in laid.lines.iter().enumerate() {
            assert!(close(line[0], corners[i]) && close(line[1], corners[(i + 1) % 4]), "wall {i}: {line:?}");
        }
    }

    #[test]
    fn a_partition_goes_between_its_two_faces_from_either_face() {
        for x in [17.8125, 18.1875] {
            for flip in [false, true] {
                // Clicked south to north, and north to south.
                let up = by_face(&[[x, 0.2], [x, 15.8]], false, &house(), 0.375, flip);
                assert!(close(up.lines[0][0], [18.0, 0.0]) && close(up.lines[0][1], [18.0, 16.0]), "{:?}", up.lines);
                assert_eq!(up.seen, vec![Seen::Both { gap: 0.375 }]);
                let down = by_face(&[[x, 15.8], [x, 0.2]], false, &house(), 0.375, flip);
                assert!(close(down.lines[0][0], [18.0, 16.0]) && close(down.lines[0][1], [18.0, 0.0]), "{:?}", down.lines);
            }
        }
    }

    #[test]
    fn a_partition_clicked_on_one_face_at_one_end_and_the_other_at_the_other() {
        let laid = by_face(&[[17.8125, 0.2], [18.1875, 15.8]], false, &house(), 0.375, false);
        assert!(close(laid.lines[0][0], [18.0, 0.0]) && close(laid.lines[0][1], [18.0, 16.0]), "{:?}", laid.lines);
    }

    #[test]
    fn a_ring_with_one_side_the_sheet_shows_no_face_of_still_meets_at_its_corners() {
        // The west wall's face is not seen (a wardrobe stands in front of it).
        let mut faces = house();
        faces.remove(3);
        let clicks = [[0.0, 0.0], [30.0, 0.0], [30.0, 16.0], [0.0, 16.0]];
        let laid = by_face(&clicks, true, &faces, 0.5, false);
        assert_eq!(laid.seen[3], Seen::Nothing, "{:?}", laid.seen);
        for i in 0..4 {
            assert!(close(laid.lines[i][1], laid.lines[(i + 1) % 4][0]), "wall {i} meets wall {}: {:?}", (i + 1) % 4, laid.lines);
        }
        // The three seen sides are carried out 3 in and their corners are where their lines meet.
        assert!(close(laid.lines[0][1], [30.25, -0.25]) && close(laid.lines[1][1], [30.25, 16.25]), "{:?}", laid.lines);
        // The unseen side stays on the line clicked, from end to end of its neighbours.
        assert!((laid.lines[3][0][0] - 0.0).abs() < 1e-6 && (laid.lines[3][1][0] - 0.0).abs() < 1e-6, "{:?}", laid.lines);
    }

    #[test]
    fn a_click_with_no_face_near_stays_where_it_was_put() {
        let clicks = [[9.0, 3.0], [12.0, 3.0]];
        let laid = by_face(&clicks, false, &house(), 0.5, false);
        assert_eq!(laid.seen, vec![Seen::Nothing]);
        assert!(close(laid.lines[0][0], [9.0, 3.0]) && close(laid.lines[0][1], [12.0, 3.0]));
        assert!(!on_a_face(&house(), [9.0, 3.0]));
        assert!(on_a_face(&house(), [9.0, 0.05]));
    }

    #[test]
    fn a_sheet_that_cannot_tell_the_room_is_laid_to_the_right_and_says_so() {
        let faces = vec![face([0.0, 0.0], [30.0, 0.0], None)];
        let laid = by_face(&[[0.0, 0.0], [30.0, 0.0]], false, &faces, 0.5, false);
        assert_eq!(laid.seen, vec![Seen::Face { room: None }]);
        assert!(close(laid.lines[0][0], [0.0, -0.25]), "{:?}", laid.lines);
    }

    #[test]
    fn one_face_cut_by_a_doorway_is_still_one_wall() {
        // A south wall seen in two pieces with a 3 ft doorway between, traced across it in two clicks.
        let faces = vec![face([0.0, 0.0], [10.0, 0.0], Some(true)), face([13.0, 0.0], [30.0, 0.0], Some(true))];
        let laid = by_face(&[[0.0, 0.0], [30.0, 0.0]], false, &faces, 0.5, false);
        assert!(close(laid.lines[0][0], [0.0, -0.25]) && close(laid.lines[0][1], [30.0, -0.25]), "{:?}", laid.lines);
    }

    /// A sheet the page reader shrank: the dots to the foot are the picture's own, and what is read comes back
    /// in the page's units (call 190).
    #[test]
    fn a_sheet_that_was_shrunk_gives_its_faces_in_the_pages_units() {
        // The page is 8192 by 2048 dots at 48 to the foot; the picture the reader made of it is half that.
        let (pw, ph) = (4096u32, 1024u32);
        let mut rgba = vec![255u8; (pw * ph * 4) as usize];
        // A band of points 8 rows deep, rows 500 to 507 from the top, from dot 100 to dot 2100.
        for y in 500..508 {
            for x in 100..2100 {
                let at = ((y * pw + x) * 4) as usize;
                rgba[at..at + 4].copy_from_slice(&[100, 100, 100, 255]);
            }
        }
        let page = plumbline_plans::Page { sort: plumbline_plans::Sort::Png, pages: 1, size: [8192.0, 2048.0], picture: plumbline_plans::Picture { width: pw, height: ph, rgba }, lines: Vec::new(), dashed: Vec::new(), cuts: Vec::new(), pictures: 0, lost: Vec::new() };
        let faces = scan_faces(&page, 1.0 / 48.0);
        assert_eq!(faces.len(), 1, "{faces:?}");
        let [a, b] = faces[0].line;
        // The band's middle is 3.5 dots below row 500, so row 503.5 from the top: 1024 - 503.5 = 520.5 from
        // the bottom in picture dots, twice that in page units (the half dot's own correction is a dot).
        assert!((a[1] - 2.0 * 520.5).abs() < 6.0 && (b[1] - 2.0 * 520.5).abs() < 6.0, "{faces:?}");
        assert!((a[0] - 2.0 * 100.0).abs() < 12.0 && (b[0] - 2.0 * 2100.0).abs() < 12.0, "{faces:?}");
        // Read as if its dots were the page's, it would be half as far across.
        assert!(b[0] - a[0] > 3900.0, "{faces:?}");
    }

    #[test]
    fn a_doorway_in_a_face_is_a_stretch_with_no_face() {
        // The west face of a partition 16 ft long with a 3 ft doorway 6 ft along it.
        let faces = vec![face([10.0, 0.0], [10.0, 6.0], Some(true)), face([10.0, 9.0], [10.0, 16.0], Some(true))];
        assert!((bare_length(&faces, [10.0, 0.0], [10.0, 16.0]) - 3.0).abs() < 1e-9);
        // Walked the other way, the same.
        assert!((bare_length(&faces, [10.1, 16.0], [9.9, 0.0]) - 3.0).abs() < 1e-2);
        // Nothing missing along a whole face, and a stretch with no face near it at all is all bare.
        assert!(bare_length(&house(), [0.0, 0.0], [30.0, 0.0]) < 1e-9);
        assert!((bare_length(&house(), [5.0, 5.0], [9.0, 5.0]) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn a_stretch_clicked_across_a_partition_from_one_face_to_the_other_is_covered_by_both() {
        // The partition of the made-up house, clicked from near its west face at the south to near its east face at the
        // north, the stretch lying slantwise across it: no gap, and with a 3 ft doorway cut in both faces, 3 ft of gap.
        assert!(bare_length(&house(), [17.8125, 0.2], [18.1875, 15.8]) < 1e-9);
        let mut f = house();
        f[4] = face([17.8125, 0.2], [17.8125, 9.0], Some(false));
        let mut g = face([17.8125, 12.0], [17.8125, 15.8], Some(false));
        g.pair = Some((5, 0.375));
        f.push(g);
        f[4].pair = Some((5, 0.375));
        f[5] = face([18.1875, 0.2], [18.1875, 9.0], Some(true));
        f[5].pair = Some((4, 0.375));
        let mut h = face([18.1875, 12.0], [18.1875, 15.8], Some(true));
        h.pair = Some((4, 0.375));
        f.push(h);
        let bare = bare_length(&f, [17.8125, 0.2], [18.1875, 15.8]);
        assert!((bare - 3.0).abs() < 0.05, "{bare}");
    }
}
