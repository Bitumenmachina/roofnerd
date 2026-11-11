//! Drawing and editing in the plan view: the tool in hand, the points picked so far,
//! where the next pick would land, and the undo history.
//!
//! Nothing here decides what a wall or a run is. Each pick is one call to a verb of
//! `plumbline_world::editing`, which the command line calls too; where a pick lands is
//! `editing::snap`. Everything drawn here is entered Assumed, with the window as its
//! source, until somebody confirms it.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use plumbline_world::editing::{self as ed, snap, History};
use plumbline_world::value::Status;
use plumbline_world::{file, units, Kind};

use crate::model::{Doc, Entry, Snapshot, Tone, SOURCE};
use crate::scene::{Highlight, MainCamera, BAR_H, BROWSER_W, GAP, INSPECTOR_W, STATUS_H};
use crate::theme;

/// What the next picks make.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Wall,
    Door,
    Window,
    /// A duct, a pipe, a conduit or a cable.
    Run(Kind),
    /// A fixture or a piece of equipment.
    Thing(Kind),
    /// Move what is selected: from one pick to the next.
    Move,
    /// A fitting where runs meet: one pick on the joint.
    Fitting,
    /// A run joined to the fixture or the equipment its end is at: one pick on that end.
    Join,
    /// A drawing's scale: the two ends of a dimension on it, then the dimension typed.
    Scale,
    /// A drawing lined up: a place on it, then the place on the plan that is.
    LineUp,
}

impl Tool {
    /// The tools that draw, in the order the panel lists them.
    pub const DRAWING: [Tool; 9] = [Tool::Wall, Tool::Door, Tool::Window, Tool::Run(Kind::Duct), Tool::Run(Kind::Pipe), Tool::Run(Kind::Conduit), Tool::Run(Kind::Cable), Tool::Thing(Kind::Fixture), Tool::Thing(Kind::Equipment)];

    pub fn name(self) -> &'static str {
        match self {
            Tool::Wall => "Wall",
            Tool::Door => "Door",
            Tool::Window => "Window",
            Tool::Run(Kind::Duct) => "Duct",
            Tool::Run(Kind::Pipe) => "Pipe",
            Tool::Run(Kind::Conduit) => "Conduit",
            Tool::Run(_) => "Cable",
            Tool::Thing(Kind::Equipment) => "Equipment",
            Tool::Thing(_) => "Fixture",
            Tool::Move => "Move",
            Tool::Fitting => "Fitting",
            Tool::Join => "Join",
            Tool::Scale => "Scale",
            Tool::LineUp => "Line up",
        }
    }

    /// What has to be said before the tool can draw: nothing it draws is given a value
    /// nobody typed.
    pub fn asks(self) -> &'static str {
        match self {
            Tool::Wall => "Thickness, height",
            Tool::Door | Tool::Window => "Width, height, sill",
            Tool::Run(_) => "Size across, height above the floor, system",
            Tool::Thing(_) => "What it is, size (along x across x up, in feet), height above the floor",
            Tool::Move | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => "",
        }
    }

    /// The same, as it might be typed.
    pub fn like(self) -> &'static str {
        match self {
            Tool::Wall => "like 6 in, 9 ft",
            Tool::Door => "like 3 ft, 7 ft, 0 ft",
            Tool::Window => "like 4 ft, 3 ft, 3 ft",
            Tool::Run(Kind::Duct) => "like 6 in, 8 ft, Supply Air",
            Tool::Run(Kind::Pipe) => "like 3/4 in, 1 ft, Domestic Cold Water",
            Tool::Run(_) => "like 3/4 in, 8 ft, Power",
            Tool::Thing(_) => "like Lavatory, 2 x 1.5 x 3, 0 ft",
            Tool::Move | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => "",
        }
    }

    /// How many values its settings are. A fitting and a join take none: what they make is
    /// worked out from what is already drawn.
    pub fn takes(self) -> usize {
        match self {
            Tool::Wall => 2,
            Tool::Move | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => 0,
            _ => 3,
        }
    }
}

/// What the window holds of the drawing in hand.
#[derive(Default)]
pub struct Draw {
    pub tool: Option<Tool>,
    /// The points picked so far for what is being drawn, in feet on the plan.
    pub picked: Vec<[f64; 2]>,
    /// Where a pick would land now, and what it would snap to.
    pub found: Option<snap::Found>,
    /// Feet to the pixel under the pointer, for drawing the snap mark at one size.
    pub scale: f64,
    /// Each tool's settings as they were typed, by the tool's name.
    pub settings: BTreeMap<&'static str, String>,
    pub history: History,
    /// The job's text after the last step recorded: what the next step is set against.
    pub last_text: String,
    /// Goes up with every edit the window makes, with what the edit was.
    pub edits: u64,
    pub seen: u64,
    pub label: String,
    /// The storey the points in `picked` were picked on. Picks on another start afresh.
    pub picked_on: Option<String>,
    /// Set by the pick that puts a tool down, so the same click's release does not also
    /// select what is under the pointer. A new press clears it.
    pub swallow: bool,
    /// The plan's snap scene, with the drawing and the storey it was made for and the
    /// element left out of it (what is being moved does not snap to itself).
    pub scene: Option<(u64, String, Option<String>, usize, snap::Scene)>,
}

/// What the window enters is Assumed, and says where it came from: the window itself, or
/// the drawing it was traced from.
fn how(source: &str) -> ed::Entered<'_> {
    ed::Entered { status: Status::Assumed, source }
}

/// A tool's settings, split and checked as far as they can be without the job: each
/// length is a length, a size is three numbers.
fn parts(tool: Tool, text: &str) -> Result<Vec<String>, String> {
    let mut parts: Vec<String> = text.split(',').map(|s| s.trim().to_string()).collect();
    if parts.len() != tool.takes() || parts.iter().any(String::is_empty) {
        return Err(format!("a {} takes: {} ({})", tool.name().to_lowercase(), tool.asks().to_lowercase(), tool.like()));
    }
    // A size is more than nothing; a height above the floor or a sill can be nothing.
    let length = |s: &str, zero_ok: bool| {
        let feet = units::parse_length(s).and_then(|(len, unit)| len.feet(unit))?;
        match feet {
            f if f < 0.0 => Err(format!("{s:?} is less than nothing")),
            f if f == 0.0 && !zero_ok => Err(format!("{s:?} is nothing: a size is more than that")),
            _ => Ok(()),
        }
    };
    match tool {
        Tool::Wall => parts.iter().try_for_each(|p| length(p, false))?,
        Tool::Door | Tool::Window => {
            length(&parts[0], false)?;
            length(&parts[1], false)?;
            length(&parts[2], true)?;
        }
        Tool::Run(kind) => {
            length(&parts[0], false)?;
            length(&parts[1], true)?;
            // The system is one a run of this kind goes on.
            let goes_on = plumbline_world::building::run_goes_on(kind);
            match goes_on.iter().find(|c| c.eq_ignore_ascii_case(&parts[2])) {
                Some(class) => parts[2] = class.to_string(),
                None => return Err(format!("{:?} is not a system a {} goes on; there are: {}", parts[2], tool.name().to_lowercase(), goes_on.join(", "))),
            }
        }
        Tool::Thing(_) => {
            size(&parts[1])?;
            length(&parts[2], true)?;
        }
        Tool::Move | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => {}
    }
    Ok(parts)
}

/// "2 x 1.5 x 3": along, across and up, in feet.
fn size(text: &str) -> Result<[f64; 3], String> {
    let numbers: Vec<f64> = text.split(['x', 'X', '\u{00D7}']).map(|s| units::parse_number(s.trim())).collect::<Result<_, _>>()?;
    match numbers.as_slice() {
        [a, b, c] if *a > 0.0 && *b > 0.0 && *c > 0.0 => Ok([*a, *b, *c]),
        _ => Err(format!("{text:?} is not a size; write it along x across x up in feet, like 2 x 1.5 x 3")),
    }
}

/// Take a tool in hand. With no storey to draw on it says so; with its settings not yet
/// said, it asks for them first.
pub fn start(doc: &mut Doc, tool: Tool) {
    doc.draw.picked.clear();
    if doc.show.storey.is_none() {
        doc.say(Tone::Refused, "refused: choose the storey to draw on first (the list above)");
    } else if !doc.plan {
        doc.say(Tone::Refused, "refused: drawing is done on the plan (T)");
    } else if tool == Tool::Move && doc.selected.is_none() {
        doc.say(Tone::Refused, "refused: select what is to be moved first");
    } else if matches!(tool, Tool::Scale | Tool::LineUp) && !doc.show.storey.clone().is_some_and(|s| crate::plans::laid(&mut doc.job).iter().any(|l| l.storey == s)) {
        doc.say(Tone::Refused, "refused: no drawing is laid under this storey");
    } else if tool.takes() > 0 && !doc.draw.settings.contains_key(tool.name()) {
        doc.entry_line = None;
        doc.entry = Some((Entry::Settings(tool), String::new()));
    } else {
        doc.draw.tool = Some(tool);
        doc.say(Tone::Done, match tool {
            Tool::Wall | Tool::Run(_) => format!("{}: pick its start, then each corner; type a length to go that far toward the pointer; Esc ends it", tool.name()),
            Tool::Door | Tool::Window => format!("{}: pick the place on a wall", tool.name()),
            Tool::Thing(_) => format!("{}: pick where it stands", tool.name()),
            Tool::Move => "Move: pick a point on it, then where that point goes".to_string(),
            Tool::Fitting => "Fitting: pick where runs meet".to_string(),
            Tool::Join => "Join: pick where a run ends at a fixture or a piece of equipment".to_string(),
            Tool::Scale => "Scale: pick the two ends of a dimension on the drawing, then type the dimension".to_string(),
            Tool::LineUp => "Line up: pick a place on the drawing, then the place on the plan it is".to_string(),
        });
    }
    doc.touch();
}

/// The settings typed for a tool: kept, shown, and the tool taken in hand.
pub fn settings_entered(doc: &mut Doc, tool: Tool, text: &str) {
    match parts(tool, text) {
        Ok(parts) => {
            doc.draw.settings.insert(tool.name(), parts.join(", "));
            start(doc, tool);
        }
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.touch();
}

/// A new storey typed as "name, elevation, height". It becomes the storey drawn on.
pub fn new_storey(doc: &mut Doc, text: &str) {
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    let made = match parts.as_slice() {
        [name, elevation, height] if !name.is_empty() => ed::storey(&mut doc.job, name, elevation, height, how(SOURCE)),
        _ => Err("a storey takes: name, the height of its floor, how far up to the next (like Ground, 0 ft, 9 ft)".to_string()),
    };
    match made {
        Ok(path) => {
            doc.show.storey = Some(path.clone());
            doc.edit("new storey");
            doc.say(Tone::Done, format!("{path}: storey {:?}; it is the storey drawn on", parts[0]));
        }
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.redraw();
}

/// One pick with the tool in hand: at a point of the plan, snapped, and the element the
/// snap came from if it came from one.
pub fn pick(doc: &mut Doc, at: [f64; 2], source: Option<String>) {
    let (Some(tool), Some(storey)) = (doc.draw.tool, doc.show.storey.clone()) else { return };
    // A chain begun on one storey does not carry on onto another.
    if doc.draw.picked_on.as_deref() != Some(storey.as_str()) {
        doc.draw.picked.clear();
        doc.draw.picked_on = Some(storey.clone());
    }
    // Over a drawing a thing is traced, and says which drawing. Over one with no scale
    // nothing is traced: the scale comes first.
    let traced = if matches!(tool, Tool::Scale | Tool::LineUp) {
        String::new()
    } else {
        match crate::plans::source_for(doc, &storey) {
            Ok(traced) => traced,
            Err(why) => {
                doc.say(Tone::Refused, format!("refused: {why}"));
                doc.touch();
                return;
            }
        }
    };
    let traced = traced.as_str();
    let said = doc.draw.settings.get(tool.name()).cloned().unwrap_or_default();
    let s: Vec<&str> = said.split(',').map(str::trim).collect();
    // What the pick made, or why not; and the points still in hand afterwards.
    let made: Option<Result<String, String>> = match tool {
        Tool::Wall | Tool::Run(_) => {
            doc.draw.picked.push(at);
            let [.., a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, format!("{}: from here; pick the next point, or type a length", tool.name()));
                doc.touch();
                return;
            };
            let pts = [*a, *b];
            let result = ed::all_or_nothing(&mut doc.job, |job| match tool {
                Tool::Wall => {
                    let t = ed::wall_type_for(job, s[0], how(SOURCE))?;
                    ed::walls(job, &storey, &t, &pts, false, ed::Top::Height(s[1]), how(traced))
                }
                Tool::Run(kind) => {
                    let t = ed::segment_type_for(job, kind, s[0], how(SOURCE))?;
                    let system = ed::system_of_class(job, s[2])?;
                    ed::runs(job, &storey, kind, &t, &pts, s[1], Some(&system), how(traced))
                }
                _ => unreachable!("only a wall or a run is drawn point to point"),
            });
            // The next piece starts where this one ended; a refused one leaves its start.
            doc.draw.picked = vec![if result.is_ok() { pts[1] } else { pts[0] }];
            Some(result.map(|made| format!("{} {} long", made.join(", "), units::feet_as_ft_in(((pts[1][0] - pts[0][0]).powi(2) + (pts[1][1] - pts[0][1]).powi(2)).sqrt(), 16))))
        }
        Tool::Door | Tool::Window => {
            let kind = if tool == Tool::Door { Kind::Door } else { Kind::Window };
            Some(match source.filter(|p| doc.job.kind(p) == Some(Kind::Wall)) {
                None => Err(format!("a {} goes in a wall: point at one", tool.name().to_lowercase())),
                Some(wall) => ed::all_or_nothing(&mut doc.job, |job| {
                    let t = ed::opening_type_for(job, kind, s[0], s[1], how(SOURCE))?;
                    ed::opening(job, &wall, kind, &t, ed::OnWall::Point(at), s[2], how(traced))
                })
                .map(|made| format!("{made} in {wall}")),
            })
        }
        Tool::Thing(kind) => Some(size(s[1]).and_then(|size| ed::place(&mut doc.job, &storey, kind, s[0], at, s[2], Some(size), None, how(traced))).map(|made| format!("{made}: {}", s[0]))),
        Tool::Fitting => Some(ed::fitting(&mut doc.job, &storey, at, None, None, how(traced)).map(|f| {
            let cut: Vec<&str> = f.cut.iter().map(|c| c.0.as_str()).collect();
            format!("{}: a {}{}", f.path, f.kind.title().to_lowercase(), if cut.is_empty() { String::new() } else { format!("; {} cut there", cut.join(", ")) })
        })),
        Tool::Join => Some(ed::connect_at(&mut doc.job, &storey, at, how(traced)).map(|(thing, run, which)| format!("{thing} is joined to {run} by its connector {which}"))),
        Tool::Scale => {
            doc.draw.picked.push(at);
            if doc.draw.picked.len() < 2 {
                doc.say(Tone::Done, "Scale: from here; pick the other end of the dimension");
            } else {
                // Both ends are picked: the dimension between them is typed next.
                doc.entry_line = None;
                doc.entry = Some((Entry::Distance, String::new()));
                doc.say(Tone::Done, "Scale: type the dimension between the two points");
            }
            doc.touch();
            return;
        }
        Tool::LineUp => {
            doc.draw.picked.push(at);
            let [a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, "Line up: from here; pick the place on the plan it is");
                doc.touch();
                return;
            };
            let (a, b) = (*a, *b);
            doc.draw.picked.clear();
            doc.draw.tool = None;
            doc.draw.swallow = true;
            Some(match crate::plans::under(doc, &storey, a) {
                None => Err("the first point is not on a drawing".to_string()),
                Some(l) => {
                    let on_page = l.on_page(a);
                    plumbline_world::plans::line_up(&mut doc.job, &l.path, on_page, b, ed::Entered { status: Status::Assumed, source: "lined up in the window" }).map(|()| format!("{} is lined up: that place on it is ({}, {}) on the plan", l.name, units::feet_as_ft_in(b[0], 16), units::feet_as_ft_in(b[1], 16)))
                }
            })
        }
        Tool::Move => {
            doc.draw.picked.push(at);
            let [a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, "Move: from here; pick where it goes");
                doc.touch();
                return;
            };
            let (east, north) = (b[0] - a[0], b[1] - a[1]);
            doc.draw.picked.clear();
            doc.draw.tool = None;
            doc.draw.swallow = true;
            Some(match doc.selected.clone() {
                None => Err("nothing is selected to move".to_string()),
                Some(sel) => ed::move_by(&mut doc.job, &sel, east, north, how(traced)).map(|()| format!("{sel} moved {} east, {} north", units::feet_as_ft_in(east, 16), units::feet_as_ft_in(north, 16))),
            })
        }
    };
    match made {
        Some(Ok(what)) => {
            doc.edit(tool.name());
            doc.say(Tone::Done, what);
        }
        Some(Err(why)) => doc.say(Tone::Refused, format!("refused: {why}")),
        None => {}
    }
    doc.redraw();
}

/// A length typed while a wall or a run is being drawn: the next point is that far from
/// the last one, toward where the pointer is.
pub fn typed_length(doc: &mut Doc, text: &str) {
    let Some(last) = doc.draw.picked.last().copied() else { return };
    let toward = doc.draw.found.as_ref().map(|f| f.at).unwrap_or([last[0] + 1.0, last[1]]);
    match units::parse_length(text).and_then(|(len, unit)| len.feet(unit)) {
        Ok(feet) if feet > 0.0 => pick(doc, ed::toward(last, toward, feet), None),
        Ok(_) => doc.say(Tone::Refused, "refused: a length is more than nothing"),
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.touch();
}

/// The dimension typed for a drawing's scale, after its two ends were picked. The
/// dimension is read off the drawing, so the scale is Assumed.
pub fn scale_entered(doc: &mut Doc, text: &str) {
    let picked = doc.draw.picked.clone();
    doc.draw.picked.clear();
    doc.draw.tool = None;
    let made = match (picked.as_slice(), doc.show.storey.clone()) {
        ([a, b], Some(storey)) => match crate::plans::under(doc, &storey, *a) {
            None => Err("the first point is not on a drawing".to_string()),
            Some(l) => {
                let (from, to) = (l.on_page(*a), l.on_page(*b));
                plumbline_world::plans::calibrate(&mut doc.job, &l.path, from, to, text, ed::Entered { status: Status::Assumed, source: "read off the drawing" }).map(|k| format!("{}: {} between the two points; the drawing is {} wide", l.name, text.trim(), units::feet_as_ft_in(l.drawing.size[0] * k, 16)))
            }
        },
        _ => Err("pick the two ends of a dimension first".to_string()),
    };
    match made {
        Ok(said) => {
            doc.edit("scale");
            doc.framed = false;
            doc.say(Tone::Done, said);
        }
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.redraw();
}

/// Take the last step back, or with `redo` do it again.
pub fn step(doc: &mut Doc, redo: bool) {
    let stepped = if redo { doc.draw.history.redo(&mut doc.job) } else { doc.draw.history.undo(&mut doc.job) };
    match stepped {
        Ok(label) => {
            doc.draw.last_text = file::save(&mut doc.job);
            doc.edited = true;
            doc.draw.picked.clear();
            // What the step took away is no longer there to be selected, shown or drawn on.
            if doc.selected.as_ref().is_some_and(|p| doc.job.find(p).is_none()) {
                doc.selected = None;
            }
            if doc.show.storey.as_ref().is_some_and(|p| doc.job.find(p).is_none()) {
                doc.show.storey = None;
                doc.draw.tool = None;
            }
            if doc.roof.as_ref().is_some_and(|p| doc.job.find(p).is_none()) {
                doc.roof = None;
            }
            doc.say(Tone::Done, format!("{}: {label}", if redo { "done again" } else { "undone" }));
        }
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.pointer = None;
    doc.hover = None;
    doc.redraw();
}

/// Take what is selected out of the job.
pub fn delete(doc: &mut Doc) {
    match doc.selected.clone() {
        None => doc.say(Tone::Refused, "refused: select what is to be removed first"),
        Some(sel) => match ed::remove(&mut doc.job, &sel) {
            Ok(gone) => {
                doc.selected = None;
                doc.hover = None;
                doc.edit("remove");
                doc.say(Tone::Done, format!("removed {}", gone.join(", ")));
            }
            Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
        },
    }
    doc.redraw();
}

/// Whether the pointer is over the model and not over a panel.
fn over_model(window: &Window, at: Vec2) -> bool {
    at.x > BROWSER_W + GAP * 2.0 && at.x < window.width() - INSPECTOR_W - GAP * 2.0 && at.y > BAR_H + GAP * 2.0 && at.y < window.height() - STATUS_H - GAP * 2.0
}

/// Where the pointer is on the storey's floor, snapped, while a tool is in hand.
pub fn pointer(mut doc: ResMut<Doc>, snap_shot: Res<Snapshot>, plans: Res<crate::plans::Plans>, keys: Res<ButtonInput<KeyCode>>, windows: Query<&Window, With<PrimaryWindow>>, camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>) {
    let doc = doc.bypass_change_detection();
    let (Some(tool), Some(storey), true) = (doc.draw.tool, doc.show.storey.clone(), doc.plan) else {
        doc.draw.found = None;
        return;
    };
    // While a length is being typed the direction it goes in stays where it was.
    if doc.entry.is_some() {
        return;
    }
    let (Ok(window), Ok((camera, at))) = (windows.single(), camera.single()) else { return };
    let Some(cursor) = window.cursor_position().filter(|c| over_model(window, *c)) else {
        doc.draw.found = None;
        return;
    };
    let elevation = snap_shot.storeys.iter().find(|s| s.path == storey).and_then(|s| s.elevation).unwrap_or(0.0);
    let floor = (elevation - doc.origin[2]) as f32;
    let on_floor = |cursor: Vec2| -> Option<[f64; 2]> {
        let ray = camera.viewport_to_world(at, cursor).ok()?;
        let down = ray.direction.y;
        if down.abs() < 1e-6 {
            return None;
        }
        let hit = ray.origin + *ray.direction * ((floor - ray.origin.y) / down);
        Some([doc.origin[0] + f64::from(hit.x), doc.origin[1] - f64::from(hit.z)])
    };
    let (Some(p), Some(beside)) = (on_floor(cursor), on_floor(cursor + Vec2::X)) else { return };
    let scale = ((beside[0] - p[0]).powi(2) + (beside[1] - p[1]).powi(2)).sqrt();
    let reach = snap::Reach { point: 10.0 * scale, line: 8.0 * scale };
    // The plan's scene, made again when what is drawn has changed. What is being moved
    // is left out of it: it does not snap to where it was.
    let skip = if tool == Tool::Move { doc.selected.clone() } else { None };
    let pages = plans.read.len();
    if !doc.draw.scene.as_ref().is_some_and(|(geometry, of, without, read, _)| *geometry == doc.geometry && *of == storey && *without == skip && *read == pages) {
        let mut scene = snap::scene(&mut doc.job, &storey, skip.as_deref());
        // A drawing's own lines, where its file has them: their ends, their middles and
        // their crossings are places to pick, as a wall's are.
        for l in snap_shot.drawings.iter().filter(|l| l.storey == storey) {
            if let Some(read) = plans.read.get(&l.path) {
                for line in &read.lines {
                    scene.segment(l.on_plan(line[0]), l.on_plan(line[1]), None);
                }
            }
        }
        doc.draw.scene = Some((doc.geometry, storey.clone(), skip.clone(), pages, scene));
    }
    let Some((_, _, _, _, scene)) = &doc.draw.scene else { return };
    // A chain begun on another storey is not carried on here.
    let picked: &[[f64; 2]] = if doc.draw.picked_on.as_deref() == Some(storey.as_str()) { &doc.draw.picked } else { &[] };
    // Shift holds the next point square to the last: due east, west, north or south of it.
    let square = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let found = match tool {
        // A door or a window goes on a wall's line and nowhere else.
        Tool::Door | Tool::Window => snap::snap(scene, p, reach, &[], false, &snap::Snaps { nearest: true, ..snap::Snaps::none() }),
        Tool::Wall | Tool::Run(_) | Tool::Move => snap::snap(scene, p, reach, picked, square, &snap::Snaps::default()),
        Tool::Thing(_) | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => snap::snap(scene, p, reach, &[], false, &snap::Snaps::default()),
    };
    doc.pointer = Some([found.at[0], found.at[1], elevation]);
    doc.draw.scale = scale;
    doc.draw.found = Some(found);
}

/// A click on the model with a tool in hand is a pick.
pub fn clicks(mut doc: ResMut<Doc>, mouse: Res<ButtonInput<MouseButton>>, windows: Query<&Window, With<PrimaryWindow>>) {
    if mouse.just_pressed(MouseButton::Left) && doc.draw.swallow {
        doc.bypass_change_detection().draw.swallow = false;
    }
    if !mouse.just_pressed(MouseButton::Left) || doc.draw.tool.is_none() || doc.entry.is_some() || !doc.plan {
        return;
    }
    let Ok(window) = windows.single() else { return };
    if !window.cursor_position().is_some_and(|c| over_model(window, c)) {
        return;
    }
    let Some(found) = doc.draw.found.clone() else { return };
    pick(&mut doc, found.at, found.source);
}

/// Each edit the window has made since the last look becomes a step that can be undone.
pub fn record(mut doc: ResMut<Doc>) {
    if doc.draw.edits == doc.draw.seen {
        return;
    }
    let doc = &mut *doc;
    doc.draw.seen = doc.draw.edits;
    let before = std::mem::take(&mut doc.draw.last_text);
    let label = doc.draw.label.clone();
    doc.draw.history.record(&label, before, &mut doc.job);
    doc.draw.last_text = file::save(&mut doc.job);
}

/// What is being drawn, drawn: the points picked, the line to where the next would land,
/// and a mark there.
pub fn preview(doc: Res<Doc>, snap_shot: Res<Snapshot>, mut lines: Gizmos<Highlight>) {
    let (Some(_), Some(storey)) = (doc.draw.tool, doc.show.storey.as_ref()) else { return };
    // Points picked on another storey are not this storey's to show.
    let picked: &[[f64; 2]] = if doc.draw.picked_on.as_ref() == Some(storey) { &doc.draw.picked } else { &[] };
    let elevation = snap_shot.storeys.iter().find(|s| s.path == *storey).and_then(|s| s.elevation).unwrap_or(0.0);
    let o = doc.origin;
    let place = |p: [f64; 2]| Vec3::new((p[0] - o[0]) as f32, (elevation - o[2]) as f32 + 0.3, -(p[1] - o[1]) as f32);
    let ink = theme::accent();
    for pair in picked.windows(2) {
        lines.line(place(pair[0]), place(pair[1]), ink);
    }
    let Some(found) = &doc.draw.found else { return };
    let at = place(found.at);
    if let Some(last) = picked.last() {
        lines.line(place(*last), at, ink);
    }
    // The mark: a cross where nothing was snapped to, a square where something was.
    let r = (doc.draw.scale * 7.0) as f32;
    if found.kind == snap::SnapKind::Free {
        lines.line(at - Vec3::X * r, at + Vec3::X * r, ink);
        lines.line(at - Vec3::Z * r, at + Vec3::Z * r, ink);
    } else {
        let corners = [Vec3::new(-r, 0.0, -r), Vec3::new(r, 0.0, -r), Vec3::new(r, 0.0, r), Vec3::new(-r, 0.0, r)];
        for i in 0..4 {
            lines.line(at + corners[i], at + corners[(i + 1) % 4], ink);
        }
    }
}
