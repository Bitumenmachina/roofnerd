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
    /// The scale as a drawing's title block writes it, where the page is paper of a
    /// known size (a PDF's page, 72 units to the inch) and the scale is one of the
    /// architect's; otherwise how big the sheet is on the ground. None with no scale.
    /// How sure it is is the drawing's status, which whoever shows this shows with it.
    pub fn scale_said(&self) -> Option<String> {
        let k = self.drawing.feet_per_unit()?;
        if self.drawing.file.value.as_deref().is_some_and(|f| f.to_ascii_lowercase().ends_with(".pdf")) {
            // Inches of paper to a foot of building.
            let paper = 1.0 / (k * 72.0);
            const NAMED: [(f64, &str); 11] = [(0.0625, "1/16"), (0.09375, "3/32"), (0.125, "1/8"), (0.1875, "3/16"), (0.25, "1/4"), (0.375, "3/8"), (0.5, "1/2"), (0.75, "3/4"), (1.0, "1"), (1.5, "1 1/2"), (3.0, "3")];
            if let Some((_, name)) = NAMED.iter().find(|(inches, _)| (paper / inches - 1.0).abs() < 0.005) {
                return Some(format!("{name}\" = 1'-0\""));
            }
        }
        let [w, h] = self.drawing.size;
        // To the inch: it is a sheet's size on the ground, not a figure anything is built to.
        Some(format!("sheet {} x {}", plumbline_world::units::feet_as_ft_in(w * k, 1), plumbline_world::units::feet_as_ft_in(h * k, 1)))
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
}

type Arrived = Result<(plumbline_plans::Page, bool), String>;

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
fn folder_of(job: &Path) -> PathBuf {
    job.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf()
}

/// Start reading the page of each drawing that has not been read, and take in each page
/// that has arrived.
pub fn read(mut doc: ResMut<Doc>, mut plans: ResMut<Plans>, mut images: ResMut<Assets<Image>>, mut seen: Local<u64>) {
    // Pages that have arrived.
    let mut arrived: Vec<(String, (String, usize), Arrived)> = Vec::new();
    for (path, (key, waiting)) in &plans.reading {
        if let Ok(result) = waiting.lock().map_err(|_| ()).and_then(|r| r.try_recv().map_err(|_| ())) {
            arrived.push((path.clone(), key.clone(), result));
        }
    }
    for (path, key, result) in arrived {
        plans.reading.remove(&path);
        match result {
            Ok((page, changed)) => {
                // The sheet with its smaller copies after it, so a whole page seen small
                // keeps its thin lines.
                let (width, height) = (page.picture.width, page.picture.height);
                let (smaller, levels) = smaller_copies(&page.picture.rgba, width, height);
                let mut image = Image::new(Extent3d { width, height, depth_or_array_layers: 1 }, TextureDimension::D2, page.picture.rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
                if let Some(data) = image.data.as_mut() {
                    data.extend_from_slice(&smaller);
                    image.texture_descriptor.mip_level_count = levels;
                }
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear, mipmap_filter: ImageFilterMode::Linear, ..default() });
                let mut said = format!("{path}: its page is read ({})", if page.lines.is_empty() { "nothing on it to snap to".to_string() } else { format!("{} lines to snap to", page.lines.len()) });
                if !page.lost.is_empty() {
                    said.push_str(&format!("; {} not read from it: select the drawing to see what", if page.lost.len() == 1 { "1 thing".to_string() } else { format!("{} things", page.lost.len()) }));
                }
                if changed {
                    said.push_str("; ITS FILE HAS CHANGED since the drawing was made from it");
                }
                doc.say(if changed { Tone::Refused } else { Tone::Done }, said);
                plans.read.insert(path, Read { key, image: images.add(image), lines: page.lines, lost: page.lost, changed });
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
        let (file, page, bytes, checksum) = (folder.join(&key.0), key.1, l.drawing.bytes, l.drawing.checksum.clone());
        let (send, receive) = channel::<Arrived>();
        std::thread::spawn(move || {
            let result = std::fs::read(&file).map_err(|e| format!("{} cannot be read ({e})", file.display())).and_then(|data| {
                let changed = data.len() as u64 != bytes || plumbline_plans::checksum(&data) != checksum;
                plumbline_plans::read(&data, page).map(|p| (p, changed))
            });
            let _ = send.send(result);
        });
        plans.reading.insert(l.path, (key, Mutex::new(receive)));
    }
}

/// A plan file named in the window ("plans/a-1.pdf" or "plans/a-1.pdf, 2"), or dropped
/// on it: laid under the storey on show as a drawing with no scale yet. The file is read
/// here once, for its page's size; its picture arrives a moment later.
pub fn add(doc: &mut Doc, text: &str) {
    let Some(storey) = doc.show.storey.clone() else {
        doc.say(Tone::Refused, "refused: choose the storey the drawing is a plan of first");
        doc.touch();
        return;
    };
    let (file, page) = match text.rsplit_once(',') {
        Some((file, page)) if page.trim().parse::<usize>().is_ok() => (file.trim(), page.trim().parse::<usize>().unwrap_or(1)),
        _ => (text.trim(), 1),
    };
    let made = (|| -> Result<String, String> {
        if file.is_empty() {
            return Err("name the plan's file: a PDF, a PNG or a JPEG".into());
        }
        let bytes = std::fs::read(file).map_err(|e| format!("cannot read {file}: {e}"))?;
        // Only what the drawing needs to be made: the page's size. Its picture is read
        // on a thread of its own, so the window keeps drawing.
        let (_, pages, size) = plumbline_plans::size_of(&bytes, page).map_err(|why| format!("{file}: {why}"))?;
        let kept = from_folder(&folder_of(&doc.path), Path::new(file))?;
        let name = Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or("drawing").to_string();
        let name = if pages > 1 { format!("{name}, page {page}") } else { name };
        // Which file it is is a fact, read off the disk here; where it lies and its
        // scale are not, and say so themselves.
        let how = ed::Entered { status: Status::Known, source: "the file laid under the plan in the window" };
        let path = world_plans::add(&mut doc.job, &storey, &name, world_plans::Read { file: &kept, page, bytes: bytes.len() as u64, checksum: &plumbline_plans::checksum(&bytes), size }, how)?;
        Ok(format!("{path}: {name} is laid under the plan; it has no scale yet: set it from one dimension"))
    })();
    match made {
        Ok(said) => {
            doc.edit("drawing");
            doc.framed = false;
            doc.say(Tone::Done, said);
        }
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.redraw();
}

/// A file's path from a folder, with `..` where it has to climb.
fn from_folder(folder: &Path, file: &Path) -> Result<String, String> {
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
                add(&mut doc, file);
            }
        }
    }
}

/// The drawing a pick on the plan is on: the last one laid under the storey whose page,
/// as shown, holds the point.
pub fn under(doc: &mut Doc, storey: &str, at: [f64; 2]) -> Option<Laid> {
    laid(&mut doc.job).into_iter().filter(|l| l.storey == storey && l.holds(at)).next_back()
}

/// How something drawn on a storey is entered. Over a drawing it is traced: Assumed, and
/// saying which drawing. A storey with a drawing that has no scale is not traced over.
pub fn source_for(doc: &mut Doc, storey: &str) -> Result<String, String> {
    let here: Vec<Laid> = laid(&mut doc.job).into_iter().filter(|l| l.storey == storey).collect();
    if let Some(l) = here.iter().find(|l| l.drawing.feet_per_unit().is_none()) {
        return Err(format!("{} has no scale: set it from one dimension before tracing over it", l.name));
    }
    match here.last() {
        Some(l) => world_plans::traced_from(&mut doc.job, &l.path),
        None => Ok(crate::model::SOURCE.to_string()),
    }
}
