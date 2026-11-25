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
use crate::scene::{over_model_at, Highlight, MainCamera};
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
    /// A floor: one pick on the storey, and the floor is the outline its walls enclose.
    Floor,
    /// A room: a point inside it, then its name typed.
    Room,
    /// A room line: the two ends of a line that parts two rooms where no wall stands.
    RoomLine,
    /// A run of cabinet boxes along a wall (call 156): a click on the wall's face where the
    /// run starts, then the boxes' codes typed.
    Cabinets,
    /// A roof: the corners of its footprint in turn, and the first again to close it.
    Roof,
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
    /// The build bar, in the order it shows them: the building itself; what runs through
    /// it and stands in it (mechanical, electrical, plumbing); and moving what is there.
    pub const BUILDING: [Tool; 8] = [Tool::Wall, Tool::Door, Tool::Window, Tool::Floor, Tool::Room, Tool::RoomLine, Tool::Cabinets, Tool::Roof];
    pub const MEP: [Tool; 8] = [Tool::Run(Kind::Duct), Tool::Run(Kind::Pipe), Tool::Run(Kind::Conduit), Tool::Run(Kind::Cable), Tool::Fitting, Tool::Thing(Kind::Equipment), Tool::Thing(Kind::Fixture), Tool::Join];
    /// The tools a number key takes in hand, 1 to 9 and then 0.
    pub const KEYED: [Tool; 10] = [Tool::Wall, Tool::Door, Tool::Window, Tool::Floor, Tool::Room, Tool::Roof, Tool::Run(Kind::Duct), Tool::Run(Kind::Pipe), Tool::Run(Kind::Conduit), Tool::Run(Kind::Cable)];

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
            Tool::Floor => "Floor",
            Tool::Room => "Room",
            Tool::RoomLine => "Room line",
            Tool::Cabinets => "Cabinets",
            Tool::Roof => "Roof",
            Tool::Move => "Move",
            Tool::Fitting => "Fitting",
            Tool::Join => "Connect",
            Tool::Scale => "Set scale",
            Tool::LineUp => "Line up",
        }
    }

    /// The number key that takes it in hand, as it is written on its button.
    pub fn key(self) -> &'static str {
        match Tool::KEYED.iter().position(|t| *t == self) {
            Some(9) => "0",
            Some(i) => ["1", "2", "3", "4", "5", "6", "7", "8", "9"][i],
            None => "",
        }
    }

    /// What has to be said before the tool can draw, each with what might be typed:
    /// nothing it draws is given a value nobody typed. AFF is above finished floor, as
    /// a drawing writes it.
    pub fn fields(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Tool::Wall => &[("Thickness", "6 in"), ("Height", "9 ft")],
            Tool::Door => &[("Width", "3 ft"), ("Height", "7 ft"), ("Sill height", "0 ft")],
            Tool::Window => &[("Width", "4 ft"), ("Height", "3 ft"), ("Sill height", "3 ft")],
            Tool::Run(Kind::Duct) => &[("Size", "6 in"), ("Height AFF", "8 ft"), ("System", "Supply Air")],
            Tool::Run(Kind::Pipe) => &[("Size", "3/4 in"), ("Height AFF", "1 ft"), ("System", "Domestic Cold Water")],
            Tool::Run(_) => &[("Size", "3/4 in"), ("Height AFF", "8 ft"), ("System", "Power")],
            Tool::Thing(Kind::Equipment) => &[("What it is", "Boiler"), ("Size L x W x H, ft", "2 x 2 x 3"), ("Height AFF", "0 ft")],
            Tool::Thing(_) => &[("What it is", "Lavatory"), ("Size L x W x H, ft", "2 x 1.5 x 3"), ("Height AFF", "0 ft")],
            Tool::Floor => &[("Thickness", "4 in")],
            Tool::Roof => &[("Pitch", "6:12"), ("Eave height", "8 ft")],
            Tool::Room | Tool::RoomLine | Tool::Cabinets | Tool::Move | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => &[],
        }
    }

    /// How many values its settings are. A fitting and a join take none: what they make is
    /// worked out from what is already drawn.
    pub fn takes(self) -> usize {
        self.fields().len()
    }

    /// What to do with it in hand.
    pub fn how(self) -> &'static str {
        match self {
            Tool::Wall | Tool::Run(_) => "Click the start, then each corner. Type a length to go that far. Shift keeps it square. Esc ends.",
            Tool::Door | Tool::Window => "Click the place on a wall.",
            Tool::Thing(_) => "Click where it stands.",
            Tool::Floor => "Click anywhere on the level. The floor fills the outline its walls enclose.",
            Tool::Room => "Click inside the room, then type its name.",
            Tool::RoomLine => "Parts two rooms where no wall stands: click each end. Esc ends.",
            Tool::Cabinets => "Click a wall's face, on the room's side. The run starts where you click; type another start in Starts at. Then type the boxes, like B18 SB36 B24. Esc ends.",
            Tool::Roof => "Click each corner of the roof's outline, then the first again to close it.",
            Tool::Move => "Click a point on it, then where that point goes; or press on it and drag.",
            Tool::Fitting => "Click where runs meet.",
            Tool::Join => "Click where a run ends at a fixture or a piece of equipment.",
            Tool::Scale => "Click the two ends of a dimension on the drawing, then type the dimension.",
            Tool::LineUp => "Click a place on the drawing, then where that place is on the plan.",
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
    /// Each tool's settings as they were typed, by the tool's name: one value to a field.
    pub settings: BTreeMap<&'static str, Vec<String>>,
    pub history: History,
    /// The job's text after the last step recorded: what the next step is set against.
    pub last_text: String,
    /// Goes up with every edit the window makes, with what the edit was.
    pub edits: u64,
    pub seen: u64,
    /// What the window has said since the frame began: the sentence a step is
    /// remembered by, so that taking it back can say which line and what (J24A-12).
    pub said: Option<String>,
    pub label: String,
    /// The storey the points in `picked` were picked on. Picks on another start afresh.
    pub picked_on: Option<String>,
    /// Set by the pick that puts a tool down, so the same click's release does not also
    /// select what is under the pointer. A new press clears it.
    pub swallow: bool,
    /// When a room's point has been picked and its name is being typed: the source a
    /// thing picked there carries (the drawing it lies over, or the window).
    pub traced: Option<String>,
    /// The plan's snap scene, with the drawing and the storey it was made for and the
    /// element left out of it (what is being moved does not snap to itself).
    pub scene: Option<(u64, String, Option<String>, usize, snap::Scene)>,
    /// The wall and the room the Cabinets tool's click found, while the run's boxes are typed.
    pub cabinets: Option<(String, String)>,
    /// The switch "Countertop" beside the Cabinets fields is off (it is on to begin with).
    pub without_top: bool,
    /// The Wall tool's choice "Centerline" or "Face" as the person made it (call 190): Some(true) is
    /// Face, Some(false) Centerline; None, which is Face over a scan's sheet and Centerline anywhere else.
    pub face_choice: Option<bool>,
    /// The faces each scan's sheet shows under the storey, on the plan: what a wall's clicks land on and
    /// are carried by.
    pub sheets: Vec<crate::plans::SheetFaces>,
    /// The run of walls last laid by a scan's face: what "Other side" lays on the other side.
    pub faced: Option<Faced>,
    /// The length typed for the wall now being laid, in feet, while its pick is made (call 193, C).
    pub typed: Option<f64>,
    /// Where the Move tool's button went down, on screen: a drag from there is a move (call 193, E).
    pub pressed_at: Option<Vec2>,
}

/// A run of walls laid by the faces of a scan's sheet (call 190): the clicks, the walls they made, and
/// the lines those were laid on, so that the next click carries on from them and "Other side" can lay
/// them again.
#[derive(Clone, Debug, PartialEq)]
pub struct Faced {
    pub storey: String,
    /// The scan's drawing: its path and its name.
    pub drawing: String,
    pub name: String,
    pub clicks: Vec<[f64; 2]>,
    pub closed: bool,
    pub walls: Vec<String>,
    /// The line each wall was last laid on.
    pub lines: Vec<[[f64; 2]; 2]>,
    /// The thickness of the type the walls are of, in feet.
    pub thickness: f64,
    pub flipped: bool,
    /// Laid on the center line: the clicks are carried onto the faces and corners, and not out by half a wall.
    pub centre: bool,
    /// What the sheet showed for each stretch, and the side each body was laid on (true: the left of the
    /// way it runs).
    pub seen: Vec<crate::plans::Seen>,
    pub body_left: Vec<Option<bool>>,
    /// What each wall was drawn from before the faces laid it: its source when it stands on the center line.
    pub traced: String,
    /// The length each wall's card gave, so that a wall a later wall joined at its corner can be told of
    /// the length it has now (call 193, F).
    pub said: Vec<f64>,
    /// The lines the run's walls take in each way it can be laid (face with the body one side, face with
    /// the body the other, center line): the walls standing on any of them are the run, as laid or after
    /// an Undo (call 193, D).
    pub states: Vec<Vec<[[f64; 2]; 2]>>,
}

/// What the window enters is Assumed, and says where it came from: the window itself, or
/// the drawing it was traced from.
fn how(source: &str) -> ed::Entered<'_> {
    ed::Entered { status: Status::Assumed, source }
}

/// A tool's settings, checked as far as they can be without the job: each length is a
/// length, a size is three numbers. Where one will not do, which one and why.
fn checked(tool: Tool, values: &[String]) -> Result<Vec<String>, (usize, String)> {
    let mut parts: Vec<String> = values.iter().map(|s| s.trim().to_string()).collect();
    parts.resize(tool.takes(), String::new());
    if let Some(i) = parts.iter().position(String::is_empty) {
        return Err((i, format!("{} is needed, like {}", tool.fields()[i].0, tool.fields()[i].1)));
    }
    // A size is more than nothing; a height above the floor or a sill can be nothing.
    let length = |i: usize, zero_ok: bool| -> Result<(), (usize, String)> {
        let s = &parts[i];
        let feet = units::parse_length(s).and_then(|(len, unit)| len.feet(unit)).map_err(|why| (i, why))?;
        match feet {
            f if f < 0.0 => Err((i, format!("{s:?} is less than nothing"))),
            f if f == 0.0 && !zero_ok => Err((i, format!("{s:?} is nothing: a size is more than that"))),
            _ => Ok(()),
        }
    };
    match tool {
        Tool::Wall => {
            length(0, false)?;
            length(1, false)?;
            // A length typed where the thickness goes was taken, and a wall 20 ft
            // thick was drawn without a word (J24B-48). Nothing built in a house is
            // 4 ft thick; a wall that is can be made at the command line.
            let thick = units::parse_length(&parts[0]).and_then(|(len, unit)| len.feet(unit)).unwrap_or(0.0);
            if thick > 4.0 {
                return Err((0, format!("{:?} is not a wall's thickness: that is written like 6 in. A wall's length is typed after its first corner is clicked", parts[0])));
            }
        }
        Tool::Door | Tool::Window => {
            length(0, false)?;
            length(1, false)?;
            length(2, true)?;
        }
        Tool::Run(kind) => {
            length(0, false)?;
            length(1, true)?;
            // The system is one a run of this kind goes on.
            let goes_on = plumbline_world::building::run_goes_on(kind);
            match goes_on.iter().find(|c| c.eq_ignore_ascii_case(&parts[2])) {
                Some(class) => parts[2] = class.to_string(),
                None => return Err((2, format!("{:?} is not a system a {} goes on. There are: {}", parts[2], tool.name().to_lowercase(), goes_on.join(", ")))),
            }
        }
        Tool::Thing(_) => {
            size(&parts[1]).map_err(|why| (1, why))?;
            length(2, true)?;
        }
        Tool::Floor => length(0, false)?,
        Tool::Roof => {
            units::parse_pitch(&parts[0]).map_err(|why| (0, why))?;
            length(1, true)?;
        }
        Tool::Room | Tool::RoomLine | Tool::Cabinets | Tool::Move | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => {}
    }
    Ok(parts)
}

/// "2 x 1.5 x 3": along, across and up, in feet.
fn size(text: &str) -> Result<[f64; 3], String> {
    // "2 x 1.5 x 3", and "2 by 1.5 by 3" as it is said.
    let not_a_size = || format!("\"{}\" is not a size; write it along x across x up in feet, like 2 x 1.5 x 3", text.trim());
    let spelled = text.to_lowercase().replace(" by ", " x ");
    let numbers: Vec<f64> = spelled.split(['x', '\u{00D7}']).map(|s| units::parse_number(s.trim())).collect::<Result<_, _>>().map_err(|_| not_a_size())?;
    match numbers.as_slice() {
        [a, b, c] if *a > 0.0 && *b > 0.0 && *c > 0.0 => Ok([*a, *b, *c]),
        _ => Err(not_a_size()),
    }
}

/// Why something could not be done, as the screen says it: what, and why.
pub fn cant(doc: &mut Doc, what: &str, why: impl std::fmt::Display) {
    // The model names the two things it found one over the other as "the new wall: lies along wall 1 ..."; the
    // colon is not a builder's (call 193, F).
    let why = why.to_string().replace(": lies along ", " lies along ");
    doc.say(Tone::Refused, format!("Can't {what}: {why}"));
}

/// Each room's name and floor area, as the job works it out now (a room is said by its
/// name; the figure is the model's).
fn room_areas(job: &mut plumbline_world::Job) -> Vec<(String, String, Option<f64>, Option<f64>)> {
    let order = job.order.clone();
    order
        .iter()
        .filter_map(|e| {
            if job.world.get::<Kind>(*e) != Some(&Kind::Room) {
                return None;
            }
            let node = job.world.get::<plumbline_world::Node>(*e)?;
            let measures = job.world.get::<plumbline_world::components::Measure>(*e);
            let area = measures.and_then(|m| m.0.get("floor_area")).and_then(|d| d.value);
            let receptacles = measures.and_then(|m| m.0.get("receptacles")).and_then(|d| d.value);
            Some((node.path.clone(), node.name.clone(), area, receptacles))
        })
        .collect()
}

/// Mark the selected thing as the work leaves it: existing, to demolish, new (call 149).
/// What `editing::mark` says is what the window says. A thing that carries cost lines is
/// refused until the estimator says what is to become of them: `marking` holds the
/// question, and the row's two buttons answer it. A step to undo, like any other.
pub fn mark_work(doc: &mut Doc, status: plumbline_world::WorkStatus, lines: ed::ItsLines) {
    let Some(sel) = doc.selected.clone() else {
        return cant(doc, "mark it", "nothing is selected");
    };
    let how = ed::Entered { status: Status::Assumed, source: SOURCE };
    let rooms_before = room_areas(&mut doc.job);
    // Everything that is selected is marked, in one step (call 169).
    let all = doc.all_selected();
    match ed::mark(&mut doc.job, &all, status, lines, how) {
        Ok(mut said) => {
            if all.len() > 1 {
                said.insert(0, format!("Marked {}: {}.", plumbline_world::view::so_many(all.len(), "thing"), status.said()));
            }
            doc.marking = None;
            doc.job.derive();
            // A room that the mark changes says so (call 161): what comes down opens the
            // room it parted, and what stays is a wall again.
            for (path, name, was, was_outlets) in rooms_before {
                let Some((_, _, now, now_outlets)) = room_areas(&mut doc.job).into_iter().find(|r| r.0 == path) else { continue };
                let num = plumbline_world::view::num;
                let area_moved = matches!((was, now), (Some(a), Some(b)) if (a - b).abs() > 1e-6);
                let outlets_moved = matches!((was_outlets, now_outlets), (Some(a), Some(b)) if (a - b).abs() > 1e-6);
                // What a mark changes in a room is said with its area: the receptacles
                // the walls need are worked from the same walls (call 169).
                match (area_moved, outlets_moved, was, now, was_outlets, now_outlets) {
                    (true, true, Some(was), Some(now), Some(wo), Some(no)) => said.push(format!("{name} is now {} SF (was {}), and needs {} receptacles (was {})", plumbline_world::view::figure(now, 2), plumbline_world::view::figure(was, 2), num(no), num(wo))),
                    (true, false, Some(was), Some(now), ..) => said.push(format!("{name} is now {} SF (was {})", plumbline_world::view::figure(now, 2), plumbline_world::view::figure(was, 2))),
                    (false, true, _, Some(now), Some(wo), Some(no)) => said.push(format!("{name} is still {} SF, and needs {} receptacles (was {})", plumbline_world::view::figure(now, 2), num(no), num(wo))),
                    _ => {}
                }
            }
            doc.edit(match status { plumbline_world::WorkStatus::Existing => "marked existing", plumbline_world::WorkStatus::Demolish => "marked to demolish", _ => "marked new work" });
            doc.say(Tone::Done, said.join("\n"));
            doc.redraw();
        }
        Err(why) => {
            // The model's sentence ends by telling a command line what to type; on the
            // window it is the two buttons that answer.
            match why.split_once(". Say which:") {
                Some((carries, _)) if matches!(lines, ed::ItsLines::Ask) => {
                    doc.marking = Some((sel, status));
                    doc.say(Tone::Ask, format!("{carries}. Say which: Take its lines off, or Keep its lines priced, as work done to it (paint, a patch, taking it down)."));
                    doc.touch();
                }
                _ => cant(doc, "mark it", why),
            }
        }
    }
}

/// Take a tool in hand. It goes to the plan and to a storey by itself: with the whole
/// building on show, the lowest storey is the one drawn on. With its settings not yet
/// said, it asks for them first.
pub fn start(doc: &mut Doc, tool: Tool) {
    begin(doc, tool, true)
}

/// `ask`: show the tool's sizes to be typed over before it is taken in hand (call 64).
/// Not asked again when they have just been typed.
fn begin(doc: &mut Doc, tool: Tool, ask: bool) {
    // On the bench (call 195) nothing is drawn: no tool is taken in hand, by a button or by a key.
    if doc.bench.is_some() {
        cant(doc, &format!("take the {} in hand", tool.name().to_lowercase()), "this is the bench, where nothing is drawn. Back to the house (Esc) first");
        doc.touch();
        return;
    }
    // A tool taken in hand again starts afresh: "Other side" is for the walls just laid, not for an older run.
    doc.draw.faced = None;
    doc.draw.picked.clear();
    doc.draw.tool = None;
    doc.letting_go(Some(Entry::Settings(tool)));
    if doc.entry.take().is_some() {
        doc.form.clear();
        doc.measuring = None;
    }
    if doc.show.storey.is_none() {
        let lowest = plumbline_world::building::storey_totals(&mut doc.job).into_iter().min_by(|a, b| a.elevation.unwrap_or(f64::MAX).total_cmp(&b.elevation.unwrap_or(f64::MAX))).map(|s| s.path);
        match lowest {
            Some(path) => doc.show.storey = Some(path),
            None => {
                // Nothing to draw on yet: the first thing a job needs is asked for.
                doc.open_form(Entry::Storey, vec![String::new(); 3], 0);
                doc.say(Tone::Done, "Add a level to draw on first: its name, the elevation of its floor, and floor-to-floor.");
                doc.redraw();
                return;
            }
        }
    }
    if !doc.plan || doc.walk {
        doc.go = Some((true, false));
    }
    let has_drawing = doc.show.storey.clone().is_some_and(|s| crate::plans::laid(&mut doc.job).iter().any(|l| l.storey == s));
    if tool == Tool::Move && doc.selected.is_none() {
        cant(doc, "move", "nothing is selected. Click what is to be moved first.");
    } else if matches!(tool, Tool::Scale | Tool::LineUp) && !has_drawing {
        cant(doc, if tool == Tool::Scale { "set a scale" } else { "line up" }, "this level has no drawing. Add one first.");
    } else if tool.takes() > 0 && (ask || !doc.draw.settings.contains_key(tool.name())) {
        // What it will place, to be read and typed over: the last sizes typed for it,
        // or, for a door, the height and sill a residential door has.
        // A wall's height is offered as the floor-to-floor of the level it is drawn
        // on: a figure the job holds, to accept or type over (call 86).
        let level_height = doc.show.storey.clone().and_then(|s| doc.job.find(&s)).and_then(|e| doc.job.world.get::<plumbline_world::Level>(e)).and_then(|l| l.height.value.as_ref().map(|v| units::show_length(v, &l.height.unit)));
        // And its thickness as that of the wall drawn last, where the job has walls:
        // a job opened again offered an empty field though every wall in it was 6 in.
        let wall_thickness = {
            let w = &doc.job.world;
            let last = doc.job.order.iter().rev().find_map(|e| w.get::<plumbline_world::Wall>(*e).and_then(|wall| wall.type_name.value.clone()));
            last.and_then(|t| doc.job.find(&t)).and_then(|e| doc.job.world.get::<plumbline_world::WallType>(e)).filter(|t| !t.footing).and_then(|t| t.thickness.value.as_ref().map(|v| units::show_length(v, &t.thickness.unit)))
        };
        // And a door or a window as the one placed last in the job: a job opened
        // again offered a door's height and sill and no width, though a 3 ft door
        // had been placed the day before (J24A-3).
        let last_opening: Option<Vec<String>> = match tool {
            Tool::Door | Tool::Window => {
                let category = if tool == Tool::Door { Kind::Door } else { Kind::Window };
                let placed: Vec<(String, String)> = doc.job.order.iter().rev().filter_map(|e| {
                    let o = doc.job.world.get::<plumbline_world::Opening>(*e)?;
                    Some((o.type_name.value.clone()?, units::feet_as_ft_in(o.sill.feet()?, 8)))
                }).collect();
                placed.into_iter().find_map(|(of, sill)| {
                    let t = doc.job.find(&of).and_then(|t| doc.job.world.get::<plumbline_world::OpeningType>(t))?;
                    if t.category != category {
                        return None;
                    }
                    // In feet and inches to the eighth, as a tape reads: a 6 ft 8 in
                    // door kept in decimal feet was offered as "6.67 ft" (J25B-28).
                    Some(vec![units::feet_as_ft_in(t.width.feet()?, 8), units::feet_as_ft_in(t.height.feet()?, 8), sill])
                })
            }
            _ => None,
        };
        let shown = doc.draw.settings.get(tool.name()).cloned().unwrap_or_else(|| match tool {
            Tool::Door | Tool::Window if last_opening.is_some() => last_opening.clone().unwrap_or_default(),
            Tool::Door => vec![String::new(), "6' 8\"".to_string(), "0 ft".to_string()],
            Tool::Wall => vec![wall_thickness.unwrap_or_default(), level_height.unwrap_or_default()],
            _ => vec![String::new(); tool.takes()],
        });
        let again = doc.draw.settings.contains_key(tool.name());
        doc.open_form(Entry::Settings(tool), shown, 0);
        doc.say(Tone::Done, if again { format!("{}: this is what it will place. Type over anything to change it. Enter, then click on the plan.", tool.name()) } else { format!("{}: type each value, Tab to the next. Enter, then click on the plan.", tool.name()) });
    } else {
        doc.draw.tool = Some(tool);
        doc.say(Tone::Done, format!("{} in hand.", tool.name()));
    }
    doc.redraw();
}

/// The settings typed for a tool: kept, shown, and the tool taken in hand. Where one of
/// them will not do, its field stays open to be put right.
pub fn settings_entered(doc: &mut Doc, tool: Tool, values: &[String]) {
    match checked(tool, values) {
        Ok(parts) => {
            doc.draw.settings.insert(tool.name(), parts);
            begin(doc, tool, false);
        }
        Err((at, why)) => {
            doc.reopen_form(Entry::Settings(tool), values.to_vec(), at);
            cant(doc, &format!("use that {}", tool.fields()[at].0.to_lowercase()), why);
        }
    }
    doc.touch();
}

/// A new item for the job's library, typed in the window: what it is called, what kind
/// of thing it is, and its width, depth and height. Its sizes are Assumed, as typed
/// here and not yet checked against its maker's sheet; its maker and model are not
/// chosen; its trade is said on its panel.
pub fn new_item(doc: &mut Doc, values: &[String]) {
    use plumbline_world::units;
    use plumbline_world::value::Val;
    const TYPED: &str = ed::ITEM_SIZE_TYPED;
    let parts: Vec<&str> = values.iter().map(|s| s.trim()).collect();
    let made = (|| -> Result<(String, String), String> {
        let [name, kind, width, depth, height] = parts.as_slice() else { return Err("an item takes a name, a kind, and its width, depth and height".to_string()) };
        let category = match kind.to_lowercase().as_str() {
            "equipment" => Kind::Equipment,
            "fixture" => Kind::Fixture,
            "furnishing" | "furniture" => Kind::Furnishing,
            other => return Err(format!("{other:?} is not a kind of item: an item is equipment, a fixture or a furnishing")),
        };
        let size = |what: &str, typed: &str| -> Result<Val<units::Len>, String> {
            let (len, unit) = units::parse_length(typed).map_err(|why| format!("its {what}: {why}"))?;
            Ok(Val { value: Some(len), unit: unit.to_string(), status: Status::Assumed, source: TYPED.to_string() })
        };
        let not_chosen = || Val::<String>::unknown("", "not chosen");
        let item = plumbline_world::Item { category, maker: not_chosen(), model: not_chosen(), width: size("width", width)?, depth: size("depth", depth)?, height: size("height", height)?, weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
        let path = ed::item(&mut doc.job, name, item)?;
        Ok((path, name.to_string()))
    })();
    match made {
        Ok((path, name)) => {
            doc.job.derive();
            doc.edit("made an item");
            doc.selected = Some(path);
            doc.say(Tone::Done, format!("Added the item {name:?}: its sizes are Assumed, as typed, until they are checked against its maker's sheet. Its trade is not said yet: pick it under Trade, on the right. It is under Items, on the left: click it there, then click where one stands."));
        }
        Err(why) => {
            let at = (0..values.len()).rev().find(|&i| !values[i].trim().is_empty() && why.contains(&format!("{:?}", values[i].trim()))).or_else(|| values.iter().position(|v| v.trim().is_empty())).unwrap_or(0);
            doc.reopen_form(Entry::NewItem, values.to_vec(), at);
            cant(doc, "add the item", why);
        }
    }
    doc.redraw();
}

/// The item that is open, called something else.
pub fn rename_item(doc: &mut Doc, typed: &str) {
    let Some(path) = doc.selected.clone() else { return cant(doc, "rename it", "no item is open") };
    match ed::rename_item(&mut doc.job, &path, typed) {
        Ok(placed) => {
            doc.job.derive();
            doc.edit("renamed an item");
            let also = match placed {
                0 => String::new(),
                1 => " The one set down is called that too.".to_string(),
                n => format!(" The {n} set down are called that too."),
            };
            doc.say(Tone::Done, format!("The item is now called {:?}.{also}", typed.trim()));
        }
        Err(why) => {
            cant(doc, "rename it", why);
            doc.open(Entry::RenameItem, typed.trim());
            doc.fresh = true;
        }
    }
    doc.redraw();
}

/// A new storey typed as "name, elevation, height". It becomes the storey drawn on.
pub fn new_storey(doc: &mut Doc, values: &[String]) {
    let parts: Vec<&str> = values.iter().map(|s| s.trim()).collect();
    let made = match parts.as_slice() {
        [name, elevation, height] if !name.is_empty() => ed::storey(&mut doc.job, name, elevation, height, how(SOURCE)),
        _ => Err("a level takes a name, the elevation of its floor, and floor-to-floor, like Ground, 0 ft, 9 ft".to_string()),
    };
    match made {
        Ok(path) => {
            doc.show.storey = Some(path.clone());
            doc.edit("added a level");
            doc.say(Tone::Done, format!("Added level {:?}. It is the level drawn on.", parts[0]));
        }
        Err(why) => {
            // The form stays open, to be put right, with the cursor in the field
            // the refusal is about: it went back to the name, and what was typed to
            // put a floor-to-floor right was typed into the name (seen 2026-11-16).
            let at = (0..values.len()).rev().find(|&i| !values[i].trim().is_empty() && why.contains(&format!("{:?}", values[i].trim()))).or_else(|| values.iter().position(|v| v.trim().is_empty())).unwrap_or(0);
            doc.reopen_form(Entry::Storey, values.to_vec(), at);
            cant(doc, "add the level", why);
        }
    }
    doc.redraw();
}

/// A cost line added to what is selected, from the three fields of its form: what it
/// is, what it is priced by (one of the thing's own takeoff figures, or a formula on
/// them), and its cost type. It has no price yet, and says where one is typed. (Judge
/// 22A was stopped here: the window could switch a labor line off and had no way to
/// put one on.)
pub fn new_line(doc: &mut Doc, values: &[String]) {
    let parts: Vec<&str> = values.iter().map(|s| s.trim()).collect();
    let made = match (doc.selected.clone(), parts.as_slice()) {
        (None, _) => Err("nothing is selected. Click what the line is on first".to_string()),
        (Some(on), [what, by, kind]) if !what.is_empty() && !by.is_empty() && !kind.is_empty() => {
            // A measure typed outright ("60 SF": call 159) is the line's quantity, entered on
            // screen: Assumed, and says so, as every typed value does.
            let typed_measure = plumbline_world::units::looks_like_measure(by);
            // A takeoff figure is typed as it is read on the panel: "net area".
            let by = if by.chars().all(|c| c.is_ascii_alphabetic() || c == ' ') { by.to_lowercase().replace(' ', "_") } else { by.to_string() };
            // In-house work is by the hour: how fast it goes follows the cost type,
            // "labor at 40 SF/hr".
            let (kind, rate) = match kind.split_once(" at ").or_else(|| kind.split_once(',')) {
                Some((k, r)) => (k.trim(), Some(r.trim()).filter(|r| !r.is_empty())),
                None => (*kind, None),
            };
            plumbline_world::Class::parse(kind).and_then(|class| {
                let in_hours = matches!(class, plumbline_world::Class::Labor | plumbline_world::Class::Fabrication | plumbline_world::Class::Supervision);
                match (in_hours, rate) {
                    (true, Some(rate)) if typed_measure => ed::all_or_nothing(&mut doc.job, |job| {
                        if !(rate.contains("/hr") || rate.contains("hr/")) {
                            return Err(format!("{rate:?} is not a production rate; write how fast the work goes, like 40 SF/hr or 0.025 hr/SF"));
                        }
                        job.add_line_measured(&on, what, &by, None, false, Status::Assumed, crate::model::PRICED)?;
                        job.set_on_line(&on, "order", Some(what), Some(rate), Status::Assumed, crate::model::PRICED)?;
                        job.set_class(&on, Some(what), class)
                    }),
                    (false, None) if typed_measure => doc.job.add_line_measured(&on, what, &by, Some(class), false, Status::Assumed, crate::model::PRICED),
                    (true, Some(rate)) => doc.job.add_line_in_hours(&on, what, &by, class, rate, Status::Assumed, crate::model::PRICED),
                    (true, None) => Err(format!("{} is in-house work, priced by the hour: say how fast it goes after the cost type, like {} at 40 SF/hr (or 0.025 hr/SF)", class.title(), class.title().to_lowercase())),
                    (false, Some(_)) => Err(format!("{} is not priced by the hour: leave the rate off, or make it labor", class.title())),
                    (false, None) => doc.job.add_line(&on, what, &by, Some(class), false),
                }
                .map(|()| (on, class))
            })
        }
        _ => Err("a cost line takes what it is, what it is priced by, and its cost type, like Siding labor, net area, labor at 40 SF/hr".to_string()),
    };
    match made {
        Ok((on, class)) => {
            doc.job.derive();
            doc.edit("added a cost line");
            let typed = if plumbline_world::units::looks_like_measure(parts[1]) { format!(" Its quantity, {}, is Assumed: entered on screen, until it is measured.", parts[1]) } else { String::new() };
            doc.say(Tone::Done, format!("Added to {on}: {:?}, {}, with no price yet.{typed} Click its price on the sheet's Estimate tab and type it.", parts[0], class.title().to_lowercase()));
        }
        Err(why) => {
            // The form stays open, to be put right.
            doc.reopen_form(Entry::Line, values.to_vec(), values.iter().position(|v| v.trim().is_empty()).unwrap_or(1));
            cant(doc, "add the cost line", why);
        }
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
    // What is drawn on a sheet is traced from it. A thing moved, a fitting put on a
    // joint and a run joined to what it ends at are done in the window, whatever lies
    // under them. Nothing but the scale and the line-up is done on a storey while a
    // drawing under it has no scale.
    let setting_up = matches!(tool, Tool::Scale | Tool::LineUp);
    let traced = match crate::plans::source_for(doc, &storey, at) {
        Ok(_) if setting_up || matches!(tool, Tool::Move | Tool::Fitting | Tool::Join) => SOURCE.to_string(),
        Ok(traced) => traced,
        Err(_) if setting_up => SOURCE.to_string(),
        Err(why) => {
            cant(doc, "draw here", why);
            doc.touch();
            return;
        }
    };
    let traced = traced.as_str();
    let said = doc.draw.settings.get(tool.name()).cloned().unwrap_or_default();
    let mut s: Vec<&str> = said.iter().map(|v| v.trim()).collect();
    s.resize(tool.takes().max(3), "");
    // What the pick made, or why not; and the points still in hand afterwards.
    // Done, with something that has to be seen: said the way a refusal is.
    let mut warned = false;
    let made: Option<Result<String, String>> = match tool {
        Tool::Wall | Tool::Run(_) => {
            // Over a scan's sheet a wall is laid by the scan's face (call 190).
            if tool == Tool::Wall {
                match wall_by_face(doc, &storey, at, &s, traced) {
                    ByFace::NotByFace => {}
                    ByFace::From(said) => {
                        doc.say(Tone::Done, said);
                        doc.touch();
                        return;
                    }
                    ByFace::Laid(result) => {
                        match result {
                            Ok(what) => {
                                doc.edit(tool.name());
                                doc.say(Tone::Done, what);
                            }
                            Err(why) => cant(doc, &doing(tool), why),
                        }
                        doc.redraw();
                        return;
                    }
                }
            }
            doc.draw.picked.push(at);
            let [.., a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, format!("{}: from here. Click the next point, or type a length.", tool.name()));
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
            // How long it is, is the model's figure for what was just made, not the
            // window's own distance between the two picks (R21-3).
            if result.is_ok() {
                doc.job.derive();
            }
            // "Face" chosen where no scan lies under the point: said, and laid on its center line.
            let no_scan = tool == Tool::Wall && doc.draw.face_choice == Some(true);
            Some(result.map(|made| {
                let long = made.last().and_then(|path| doc.job.find(path)).and_then(|e| doc.job.world.get::<plumbline_world::Measure>(e)).and_then(|m| m.0.get("plan_length").and_then(|d| d.value));
                let mut said = match long {
                    Some(feet) => format!("Added {}, {} long.", made.join(", "), units::feet_as_ft_in(feet, 16)),
                    None => format!("Added {}.", made.join(", ")),
                };
                if no_scan {
                    said.push_str(" Face is chosen, and no scan lies under this point: the wall is laid on its center line.");
                }
                said
            }))
        }
        Tool::Door | Tool::Window => {
            let kind = if tool == Tool::Door { Kind::Door } else { Kind::Window };
            Some(match source.filter(|p| doc.job.kind(p) == Some(Kind::Wall)) {
                None => Err(format!("a {} goes in a wall. Click on one.", tool.name().to_lowercase())),
                Some(wall) => ed::all_or_nothing(&mut doc.job, |job| {
                    let t = ed::opening_type_for(job, kind, s[0], s[1], how(SOURCE))?;
                    ed::opening(job, &wall, kind, &t, ed::OnWall::Point(at), s[2], how(traced))
                })
                .map(|made| format!("Added {made} in {wall}.")),
            })
        }
        Tool::Thing(kind) => Some(size(s[1]).and_then(|size| ed::place(&mut doc.job, &storey, kind, s[0], at, s[2], Some(size), None, how(traced))).map(|made| format!("Added {made}."))),
        Tool::Fitting => Some(ed::fitting(&mut doc.job, &storey, at, None, None, how(traced)).map(|f| {
            let cut: Vec<&str> = f.cut.iter().map(|c| c.0.as_str()).collect();
            format!("Added {}, a {}{}.", f.path, f.kind.title().to_lowercase(), if cut.is_empty() { String::new() } else { format!("; {} cut there", cut.join(", ")) })
        })),
        Tool::Join => Some(ed::connect_at(&mut doc.job, &storey, at, how(traced)).map(|(thing, run, which)| format!("Connected {thing} to {run}, at its connection {which}."))),
        // A floor is the outline the storey's walls enclose: where on the storey the
        // click lands does not matter.
        Tool::Floor => Some(
            ed::all_or_nothing(&mut doc.job, |job| {
                let t = ed::floor_type_for(job, s[0], how(SOURCE))?;
                ed::floor(job, &storey, &t, ed::FloorBy::Walls, how(traced))
            })
            .map(|made| {
                let mut said = format!("Added {}, the floor inside this level's walls.", made.path);
                for note in &made.notes {
                    warned = true;
                    said.push_str(&format!("\n{note}"));
                }
                said
            }),
        ),
        // A room is a point inside it; its name is typed next.
        Tool::Room => {
            doc.draw.picked = vec![at];
            doc.draw.traced = Some(traced.to_string());
            doc.open(Entry::Room, "");
            doc.say(Tone::Done, "Room: type its name, like Kitchen.");
            return;
        }
        // A room line: one end, then the other; the tool stays in hand for the next.
        Tool::RoomLine => {
            doc.draw.picked.push(at);
            let [a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, "Room line: from here. Click the other end.");
                doc.touch();
                return;
            };
            let pts = [*a, *b];
            doc.draw.picked.clear();
            Some(ed::room_lines(&mut doc.job, &storey, &pts, how(traced)).map(|made| {
                // A line with an end that meets no wall and no other room line parts
                // nothing yet, and says so (call 169; the model works the ends out).
                let loose = made.iter().any(|m| !plumbline_world::building::room_line_unreached(&mut doc.job, m).is_empty());
                if loose {
                    format!("Added {}. It does not run from wall to wall, so it parts nothing yet.", made.join(", "))
                } else {
                    format!("Added {}. It parts two rooms where no wall stands.", made.join(", "))
                }
            }))
        }
        // A run of cabinet boxes: the click finds the wall's face and the room on that side
        // and where along the wall the run starts; the boxes' codes are typed next.
        Tool::Cabinets => {
            doc.draw.traced = Some(traced.to_string());
            match run_start(doc, &storey, at) {
                Err(why) => cant(doc, "set the run", why),
                Ok((wall, room, from)) => {
                    doc.draw.cabinets = Some((wall.clone(), room.clone()));
                    doc.open_form(Entry::Cabinets, vec![units::feet_as_ft_in(from, 16), String::new()], 1);
                    doc.say(Tone::Done, format!("Cabinets: the run starts where you click, {} along {wall}, in {room}; type another start in Starts at. Type the boxes, like B18 SB36 B24 (an appliance by its name in square brackets, like [Dishwasher]), then Enter.", units::feet_as_ft_in(from, 16)));
                }
            }
            doc.touch();
            return;
        }
        // A roof's footprint: each corner in turn; a pick on the first corner again, with
        // three or more picked, closes it, and the roof is worked out from it.
        Tool::Roof => {
            let closes = doc.draw.picked.len() >= 3 && doc.draw.picked.first().is_some_and(|first| (first[0] - at[0]).hypot(first[1] - at[1]) < 0.05);
            if !closes {
                if doc.draw.picked.last().is_some_and(|last| (last[0] - at[0]).hypot(last[1] - at[1]) < 0.05) {
                    cant(doc, "add that corner", "it is the one just clicked. Click the next, or the first again to close the outline.");
                } else {
                    doc.draw.picked.push(at);
                    let n = doc.draw.picked.len();
                    doc.say(Tone::Done, format!("Roof: {n} corner{}. Click the next{}.", if n == 1 { "" } else { "s" }, if n >= 3 { ", or the first again to close it" } else { "" }));
                }
                doc.touch();
                return;
            }
            // The corners are let go only when the roof is made: an outline that is
            // refused (a pitch that will not do, an outline that crosses itself) keeps
            // every corner clicked and the tool in hand, to set right and close again
            // (R21-4: the refusal threw them all away).
            let corners = doc.draw.picked.clone();
            let made = roof_over(doc, &corners, s[0], s[1], traced).map_err(|why| format!("{why}. The corners clicked are kept: set it right and click the first corner again, or press Esc to let them go"));
            if made.is_ok() {
                doc.draw.picked.clear();
                doc.draw.tool = None;
                doc.draw.swallow = true;
            }
            Some(made)
        }
        Tool::Scale => {
            doc.draw.picked.push(at);
            if doc.draw.picked.len() < 2 {
                doc.say(Tone::Done, "Scale: from here. Click the other end of the dimension.");
            } else {
                // Both ends are picked: the dimension between them is typed next.
                doc.open(Entry::Distance, "");
                doc.say(Tone::Done, "Scale: type the dimension between the two points.");
            }
            doc.touch();
            return;
        }
        Tool::LineUp => {
            doc.draw.picked.push(at);
            let [a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, "Line up: from here. Click where that place is on the plan.");
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
                    plumbline_world::plans::line_up(&mut doc.job, &l.path, on_page, b, ed::Entered { status: Status::Assumed, source: "lined up on screen" }).map(|left| {
                        let mut said = format!("{} is lined up: that place on it is ({}, {}) on the plan", l.name, units::feet_as_ft_in(b[0], 16), units::feet_as_ft_in(b[1], 16));
                        // What was traced over it before is where it was: that has to be seen.
                        if !left.is_empty() {
                            warned = true;
                            said.push_str(&format!(". NOT MOVED: {} traced over it where it lay before; check {} against the sheet", if left.len() == 1 { "1 thing".to_string() } else { format!("{} things", left.len()) }, left.join(", ")));
                        }
                        said
                    })
                }
            })
        }
        Tool::Move => {
            doc.draw.picked.push(at);
            let [a, b] = doc.draw.picked.as_slice() else {
                doc.say(Tone::Done, "Move: from here. Click where it goes.");
                doc.touch();
                return;
            };
            let (east, north) = (b[0] - a[0], b[1] - a[1]);
            doc.draw.picked.clear();
            doc.draw.tool = None;
            doc.draw.swallow = true;
            Some(match doc.selected.clone() {
                None => Err("nothing is selected".to_string()),
                // A wall takes with it what lies on its line (calls 170, 179 B and 180): the
                // walls that meet it, its floor's edge, its footing, the wall above and the
                // roof's edge. What went and what was left is the model's own account of it
                // (call 171, item 1), said in full. One step: one Undo takes all of it back.
                Some(sel) => ed::move_wall(&mut doc.job, &sel, east, north, how(traced), false).map(|moved| {
                    let mut said = format!("Moved {} {}.", moved.names.get(&sel).cloned().unwrap_or_else(|| sel.clone()), ed::way_said(east, north));
                    for line in moved.said() {
                        said.push_str("\n");
                        said.push_str(&line);
                    }
                    said
                }),
            })
        }
    };
    match made {
        Some(Ok(what)) => {
            doc.edit(tool.name());
            doc.say(if warned { Tone::Refused } else { Tone::Done }, what);
        }
        // A corner refused in a run says where the run still stands, so that the next click is not a surprise (call 197, point 12).
        Some(Err(why)) if matches!(tool, Tool::Wall | Tool::Run(_)) && doc.draw.picked.len() == 1 => {
            let start = doc.draw.picked[0];
            let stands = format!("{} still starts at ({}, {}): click the next corner, or press Esc to let it go.", if tool == Tool::Wall { "The run of walls" } else { "The run" }, units::feet_as_ft_in(start[0], 16), units::feet_as_ft_in(start[1], 16));
            let why = why.to_string();
            cant(doc, &doing(tool), format!("{why}{} {stands}", if why.ends_with('.') { "" } else { "." }));
        }
        Some(Err(why)) => cant(doc, &doing(tool), why),
        None => {}
    }
    doc.redraw();
}

/// What a Wall pick came to over a scan.
enum ByFace {
    /// Not over a scan with Face on: the wall is laid as it always was.
    NotByFace,
    /// The first point of a wall: what to say.
    From(String),
    /// A wall laid by the scan's face, or why not.
    Laid(Result<String, String>),
}

/// A click this near the first of a run's clicks (in feet) closes the run on it.
const CLOSES_FT: f64 = 0.3;

fn near(a: [f64; 2], b: [f64; 2], within: f64) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) <= within
}

/// The line a wall is on now, in feet.
fn wall_line(job: &mut plumbline_world::Job, path: &str) -> Option<[[f64; 2]; 2]> {
    let e = job.find(path)?;
    job.world.get::<plumbline_world::Wall>(e)?.line.value
}

fn inches(feet: f64) -> String {
    units::show_length(&units::Len::One((feet * 12.0 * 16.0).round() / 16.0), "in")
}

/// Whether these walls are there on these lines.
fn walls_stand_on(job: &mut plumbline_world::Job, walls: &[String], lines: &[[[f64; 2]; 2]]) -> bool {
    walls.len() == lines.len() && walls.iter().zip(lines).all(|(w, l)| wall_line(job, w).is_some_and(|now| near(now[0], l[0], 1e-6) && near(now[1], l[1], 1e-6)))
}

/// Whether the run is still as it was laid: its walls are there on the lines they were laid on.
fn run_stands(job: &mut plumbline_world::Job, run: &Faced) -> bool {
    walls_stand_on(job, &run.walls, &run.lines)
}

/// A line as the walls keep it: to the ten thousandth of a foot.
fn tidy_line(l: &[[f64; 2]; 2]) -> [[f64; 2]; 2] {
    let t = |x: f64| (x * 1.0e4).round() / 1.0e4 + 0.0;
    [[t(l[0][0]), t(l[0][1])], [t(l[1][0]), t(l[1][1])]]
}

/// The lines a run's walls take, kept as they are kept.
fn laid_lines(laying: &crate::plans::ByFace) -> Vec<[[f64; 2]; 2]> {
    laying.lines.iter().map(tidy_line).collect()
}

/// The three ways a run can lie: laid by the face with the bodies one side, with them the other, and on the
/// center line. The lines of each, in that order.
fn run_states(run: &Faced, faces: &[crate::plans::SheetFace]) -> Vec<Vec<[[f64; 2]; 2]>> {
    [(false, false), (false, true), (true, false)]
        .into_iter()
        .map(|(centre, flipped)| laid_lines(&crate::plans::by_face(&run.clicks, run.closed, faces, if centre { 0.0 } else { run.thickness }, flipped)))
        .collect()
}

/// The faces of the sheet a run was laid by.
fn faces_of(doc: &Doc, run: &Faced) -> Vec<crate::plans::SheetFace> {
    doc.draw.sheets.iter().find(|f| f.path == run.drawing).map(|f| f.faces.clone()).unwrap_or_default()
}

/// Whether the run's walls stand in any of the ways it can be laid (the lines of each kept in `states`):
/// what an Undo of a turn leaves, and what the panel looks at to offer "Other side".
pub fn run_stands_any(snap: &Snapshot, run: &Faced) -> bool {
    let line_of = |path: &str| {
        snap.drawn.iter().find(|p| p.path == path).and_then(|p| match &p.shape {
            plumbline_world::building::Shape::Wall { a, b, .. } => Some([*a, *b]),
            _ => None,
        })
    };
    run.states.iter().any(|lines| run.walls.len() == lines.len() && run.walls.iter().zip(lines).all(|(w, l)| line_of(w).is_some_and(|now| near(now[0], l[0], 1e-3) && near(now[1], l[1], 1e-3))))
}

/// A run whose walls stand in another of the ways it can be laid than the way it remembers (an Undo of a
/// turn, or of a change of Centerline and Face, took them back) takes that way up again: it can be turned
/// again (call 193, D).
fn resync(doc: &mut Doc) {
    let Some(mut run) = doc.draw.faced.take() else { return };
    if !run_stands(&mut doc.job, &run) {
        let faces = faces_of(doc, &run);
        for (centre, flipped) in [(false, false), (false, true), (true, false)] {
            if (centre, flipped) == (run.centre, run.flipped) {
                continue;
            }
            let laying = crate::plans::by_face(&run.clicks, run.closed, &faces, if centre { 0.0 } else { run.thickness }, flipped);
            let lines = laid_lines(&laying);
            if walls_stand_on(&mut doc.job, &run.walls, &lines) {
                run.centre = centre;
                run.flipped = flipped;
                run.lines = lines;
                run.seen = laying.seen.clone();
                run.body_left = laying.body_left.clone();
                // The buttons say how the walls lie.
                doc.draw.face_choice = Some(!centre);
                break;
            }
        }
    }
    doc.draw.faced = Some(run);
}

/// How many of a run's walls lay differently, and the most any of them moved: its line sideways, or, where
/// none moved sideways, an end along it, in feet (call 193, B).
pub fn moved_by(before: &[[[f64; 2]; 2]], after: &[[[f64; 2]; 2]]) -> (usize, f64) {
    let (mut count, mut sideways, mut along) = (0, 0.0_f64, 0.0_f64);
    for (b, a) in before.iter().zip(after) {
        let ends = (b[0][0] - a[0][0]).hypot(b[0][1] - a[0][1]).max((b[1][0] - a[1][0]).hypot(b[1][1] - a[1][1]));
        if ends <= 1.0e-4 {
            continue;
        }
        count += 1;
        // The new middle's distance from the old line.
        let (d, mid) = ([b[1][0] - b[0][0], b[1][1] - b[0][1]], [(a[0][0] + a[1][0]) / 2.0 - b[0][0], (a[0][1] + a[1][1]) / 2.0 - b[0][1]]);
        let length = d[0].hypot(d[1]).max(1.0e-9);
        sideways = sideways.max(((d[0] * mid[1] - d[1] * mid[0]) / length).abs());
        along = along.max(ends);
    }
    (count, if sideways > 1.0e-4 { sideways } else { along })
}

/// The length each wall of the run has now, in feet.
fn run_lengths(job: &mut plumbline_world::Job, run: &Faced) -> Vec<f64> {
    run.walls.iter().map(|w| job.find(w).and_then(|e| job.world.get::<plumbline_world::Measure>(e)).and_then(|m| m.0.get("plan_length").and_then(|d| d.value)).unwrap_or(0.0)).collect()
}

/// Each wall of a run on the line the faces gave it, saying by its source which were laid by a face.
fn relay(job: &mut plumbline_world::Job, run: &Faced, laying: &crate::plans::ByFace) -> Result<(), String> {
    let by_face = !run.centre;
    use crate::plans::Seen;
    if laying.lines.len() != run.walls.len() {
        return Err("the walls and the faces do not agree on how many there are".into());
    }
    let tidy = |x: f64| (x * 1.0e4).round() / 1.0e4 + 0.0;
    for ((path, line), seen) in run.walls.iter().zip(&laying.lines).zip(&laying.seen) {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let mut w = job.world.get::<plumbline_world::Wall>(e).cloned().ok_or_else(|| format!("{path} is not a wall"))?;
        let line = [[tidy(line[0][0]), tidy(line[0][1])], [tidy(line[1][0]), tidy(line[1][1])]];
        let source = match seen {
            Seen::Nothing => w.line.source.clone(),
            _ if !by_face => run.traced.clone(),
            _ => format!("traced by its face over the scan {}", run.name),
        };
        w.line = plumbline_world::Val { value: Some(line), unit: w.line.unit.clone(), status: w.line.status.max(Status::Assumed), source };
        plumbline_world::building::check_wall(path, Kind::Wall, &w)?;
        job.put(path, w)?;
    }
    Ok(())
}

/// The side a run's walls lie on, as a builder says it: for a ring, outside or inside the line clicked.
fn side_said(run: &Faced) -> String {
    if run.closed {
        // Shoelace: counter-clockwise is positive, and its left is the inside.
        let c = &run.clicks;
        let area: f64 = (0..c.len()).map(|i| c[i][0] * c[(i + 1) % c.len()][1] - c[(i + 1) % c.len()][0] * c[i][1]).sum();
        if let Some(left) = run.body_left.iter().find_map(|l| *l) {
            return if left != (area > 0.0) { "outside the line clicked".to_string() } else { "inside the line clicked".to_string() };
        }
    }
    if run.flipped { "on the room's side of the line clicked".to_string() } else { "on the side away from the room".to_string() }
}

/// What the faces did to a run's last stretch, in words.
fn face_said(run: &Faced, seen: &crate::plans::Seen) -> String {
    use crate::plans::Seen;
    match seen {
        Seen::Face { room: Some(_) } => format!("Laid by the scan's face: the wall's body is {}, so its center is {} off the line clicked (half the type's {}). If that is the wrong side, press Other side.", side_said(run), inches(run.thickness / 2.0), inches(run.thickness)),
        Seen::Face { room: None } => "The scan cannot tell which side the room is on: the wall's body is laid to the right of the way you clicked. If that is the wrong side, press Other side.".to_string(),
        Seen::Both { gap } => format!("The scan shows both faces of this wall: {} between the faces, and the type is {}. The center line is laid between the faces.", inches(*gap), inches(run.thickness)),
        Seen::Nothing => "The scan shows no face here: the wall is laid on the line clicked.".to_string(),
    }
}

/// A laid wall of the plan the point is on, that has a face of the sheet within its thickness and the usual reach
/// of the point: what a click on a wall already standing on a face is by.
fn wall_on_a_face(doc: &Doc, faces: &[crate::plans::SheetFace], at: [f64; 2]) -> Option<String> {
    let (_, _, _, _, scene) = doc.draw.scene.as_ref()?;
    scene.walls.iter().find_map(|(a, b, thick, path)| {
        let (u, len2) = ([b[0] - a[0], b[1] - a[1]], (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2));
        if len2 < 1e-9 {
            return None;
        }
        let t = (((at[0] - a[0]) * u[0] + (at[1] - a[1]) * u[1]) / len2).clamp(0.0, 1.0);
        let d = (at[0] - (a[0] + u[0] * t)).hypot(at[1] - (a[1] + u[1] * t));
        (d <= thick / 2.0 + 0.05 && faces.iter().any(|f| f.distance(at) <= thick + crate::plans::ON_FACE_FT)).then(|| path.clone())
    })
}

/// A wall's pick over a scan's sheet, set by the scan's face (call 190): the click lands on the face the
/// sheet shows, and the wall is laid with its center half its thickness to the side away from the room, or
/// between the two faces where the sheet shows both. A click with no face near stays where the hand put it
/// and says so. What is laid is a center line and a type, as any wall.
fn wall_by_face(doc: &mut Doc, storey: &str, at: [f64; 2], s: &[&str], traced: &str) -> ByFace {
    // Over a scan's sheet the clicks land on its faces; with Face (to begin with) the wall is laid by the
    // face, with Centerline its center is on the line the click and the face give.
    let Some(scan) = crate::plans::scan_under(doc, storey, at) else { return ByFace::NotByFace };
    let centre = doc.draw.face_choice == Some(false);
    let faces: Vec<crate::plans::SheetFace> = doc.draw.sheets.iter().find(|f| f.path == scan.path).map(|f| f.faces.clone()).unwrap_or_default();
    let thick = units::parse_length(s[0]).and_then(|(len, unit)| len.feet(unit)).unwrap_or(0.0);
    doc.draw.picked.push(at);
    let picked = doc.draw.picked.clone();
    let [.., a, b] = picked.as_slice() else {
        if centre {
            return ByFace::From("Wall: from here. Click the next point, or type a length.".to_string());
        }
        let said = if crate::plans::on_a_face(&faces, at) {
            "Wall: from here, on the scan's face. Click the next point, or type a length.".to_string()
        } else if let Some(wall) = wall_on_a_face(doc, &faces, at) {
            // A wall laid on a face stands off it by half its thickness: that is the face the point is by.
            format!("Wall: from here, on {wall}, which stands on the scan's face. Click the next point, or type a length.")
        } else {
            "Wall: from here. The scan shows no face here: this point stays where you clicked. Click the next point, or type a length.".to_string()
        };
        return ByFace::From(said);
    };
    let (a, b) = (*a, *b);
    resync(doc);
    let typed = doc.draw.typed.take();
    let mut run = match doc.draw.faced.take() {
        Some(run) if run.storey == storey && run.drawing == scan.path && run.centre == centre && !run.closed && run.clicks.last().is_some_and(|l| near(*l, a, 1e-6)) && (run.thickness - thick).abs() < 1e-9 && run_stands(&mut doc.job, &run) => run,
        _ => Faced { storey: storey.to_string(), drawing: scan.path.clone(), name: scan.name.clone(), clicks: vec![a], closed: false, walls: Vec::new(), lines: Vec::new(), thickness: thick, flipped: false, centre, seen: Vec::new(), body_left: Vec::new(), traced: traced.to_string(), said: Vec::new(), states: Vec::new() },
    };
    let closes = run.clicks.len() >= 3 && near(b, run.clicks[0], CLOSES_FT);
    if !closes {
        run.clicks.push(b);
    }
    let laying = crate::plans::by_face(&run.clicks, closes, &faces, if centre { 0.0 } else { thick }, run.flipped);
    let result = ed::all_or_nothing(&mut doc.job, |job| {
        let t = ed::wall_type_for(job, s[0], how(SOURCE))?;
        let made = ed::walls(job, storey, &t, &[a, b], false, ed::Top::Height(s[1]), how(traced))?;
        run.walls.extend(made);
        relay(job, &run, &laying)?;
        Ok(())
    });
    match result {
        Ok(()) => {
            run.closed = closes;
            run.lines = laid_lines(&laying);
            run.seen = laying.seen.clone();
            run.body_left = laying.body_left.clone();
            run.states = run_states(&run, &faces);
            doc.draw.picked = vec![b];
            doc.job.derive();
            let last = run.walls.last().cloned().unwrap_or_default();
            let lengths = run_lengths(&mut doc.job, &run);
            let long = lengths.last().copied();
            let mut said = match long {
                Some(feet) => format!("Added {last}, {} long.", units::feet_as_ft_in(feet, 16)),
                None => format!("Added {last}."),
            };
            if let (Some(seen), false) = (run.seen.last(), centre) {
                said.push(' ');
                said.push_str(&face_said(&run, seen));
            }
            // A length typed along the face is that; the wall runs on to its corner (call 193, C).
            if let (Some(typed), Some(feet)) = (typed, long) {
                if (typed - feet).abs() > 1.0 / 32.0 {
                    said.push_str(&format!(" {} was typed along the scan's face; the wall is {} to its corner.", units::feet_as_ft_in(typed, 16), units::feet_as_ft_in(feet, 16)));
                }
            }
            // Where the stretch has a face for only part of its length (call 193, F).
            if matches!(run.seen.last(), Some(crate::plans::Seen::Face { .. } | crate::plans::Seen::Both { .. })) {
                let bare = crate::plans::bare_length(&faces, a, b);
                if bare >= 0.25 {
                    said.push_str(&format!(" The scan shows no face along {} of it: a doorway, or what the scanner did not see.", units::feet_as_ft_in(bare, 16)));
                }
            }
            // A wall whose card gave its length before this wall joined its corner is told what it has now.
            run.said.resize(lengths.len().saturating_sub(1), 0.0);
            let mut grew = Vec::new();
            for (i, (now, was)) in lengths.iter().zip(run.said.iter_mut()).enumerate() {
                if (*now - *was).abs() > 1.0 / 32.0 && *was > 0.0 {
                    grew.push(format!("{} is now {} long, where it meets this wall", run.walls[i], units::feet_as_ft_in(*now, 16)));
                }
                *was = *now;
            }
            run.said = lengths.clone();
            if !grew.is_empty() {
                let mut sentence = grew.join("; ");
                if let Some(first) = sentence.get(0..1).map(str::to_uppercase) {
                    sentence.replace_range(0..1, &first);
                }
                said.push_str(&format!(" {sentence}."));
            }
            doc.draw.faced = Some(run);
            ByFace::Laid(Ok(said))
        }
        Err(why) => {
            doc.draw.picked = vec![a];
            ByFace::Laid(Err(why))
        }
    }
}

/// "Other side": the run of walls last laid by a scan's face, laid again with their bodies on the other
/// side of the line clicked (call 190). It is for the run last laid, with the tool in hand or put down
/// (call 193, D); after an Undo of a turn it turns again.
pub fn other_side(doc: &mut Doc) {
    resync(doc);
    let Some(mut run) = doc.draw.faced.clone() else {
        cant(doc, "lay the walls on the other side", "no wall has been laid by a scan's face");
        doc.touch();
        return;
    };
    let faces = faces_of(doc, &run);
    if !run_stands(&mut doc.job, &run) {
        cant(doc, "lay the walls on the other side", "those walls have been changed since they were laid. Undo back to where they were laid and press Other side again, or trace them again");
        doc.touch();
        return;
    }
    if run.centre {
        cant(doc, "lay the walls on the other side", "they are laid on the center line, which has no other side. Press Face to lay them by the face first");
        doc.touch();
        return;
    }
    run.flipped = !run.flipped;
    let ring: &[[f64; 2]] = &run.clicks;
    let laying = crate::plans::by_face(ring, run.closed, &faces, run.thickness, run.flipped);
    let result = ed::all_or_nothing(&mut doc.job, |job| relay(job, &run, &laying));
    match result {
        Ok(()) => {
            run.lines = laid_lines(&laying);
            run.seen = laying.seen.clone();
            run.body_left = laying.body_left.clone();
            let single = run.seen.iter().filter(|s| matches!(s, crate::plans::Seen::Face { .. })).count();
            let said = format!("Laid on the other side: {single} wall{} of this run now {} {}, so {} center{} {} {} off the line clicked (half the type's {}).", if single == 1 { "" } else { "s" }, if single == 1 { "has its body" } else { "have their bodies" }, side_said(&run), if single == 1 { "its" } else { "their" }, if single == 1 { "" } else { "s" }, if single == 1 { "is" } else { "are" }, inches(run.thickness / 2.0), inches(run.thickness));
            doc.job.derive();
            run.said = run_lengths(&mut doc.job, &run);
            doc.draw.faced = Some(run);
            doc.edit("laid the walls on the other side");
            doc.say(Tone::Done, said);
        }
        Err(why) => cant(doc, "lay the walls on the other side", why),
    }
    doc.redraw();
}

/// "Centerline" or "Face", pressed (call 190). With a run of walls laid and standing, the whole run is laid
/// again the one way, as "Other side" lays it again, so that its corners still meet; and what moved is said
/// (call 193, B).
pub fn set_by(doc: &mut Doc, face: bool) {
    doc.draw.face_choice = Some(face);
    resync(doc);
    let Some(mut run) = doc.draw.faced.clone() else {
        doc.touch();
        return;
    };
    // Already laid that way, or changed since it was laid: the next wall begins a run of its own.
    if run.centre == !face || !run_stands(&mut doc.job, &run) {
        doc.touch();
        return;
    }
    let faces = faces_of(doc, &run);
    run.centre = !face;
    let laying = crate::plans::by_face(&run.clicks, run.closed, &faces, if run.centre { 0.0 } else { run.thickness }, run.flipped);
    let result = ed::all_or_nothing(&mut doc.job, |job| relay(job, &run, &laying));
    match result {
        Ok(()) => {
            let after = laid_lines(&laying);
            let (count, most) = moved_by(&run.lines, &after);
            run.lines = after;
            run.seen = laying.seen.clone();
            run.body_left = laying.body_left.clone();
            doc.job.derive();
            run.said = run_lengths(&mut doc.job, &run);
            doc.draw.faced = Some(run);
            let way = if face { "by the scan's face" } else { "on the center line" };
            let said = if count == 0 {
                format!("Laid again {way}: no wall of this run moved.")
            } else {
                format!("Laid again {way}: {count} wall{} of this run moved{} {}, and the corners still meet.", if count == 1 { "" } else { "s" }, if count == 1 { "" } else { " up to" }, inches(most))
            };
            doc.edit(if face { "laid the walls by the face" } else { "laid the walls on the center line" });
            doc.say(Tone::Done, said);
        }
        Err(why) => cant(doc, "lay the walls again", why),
    }
    doc.redraw();
}

/// Where a click with the Cabinets tool fell (call 156): the wall whose face it is on, the
/// room on that side of it, and how far along the wall's line from its first end the run
/// starts, to the inch and snapped to the room's corner when within 6 in of it. Or why
/// nothing was found. What a face is and where it is clear is the model's
/// (`building::wall_face`); the click only says which.
fn run_start(doc: &mut Doc, storey: &str, at: [f64; 2]) -> Result<(String, String, f64), String> {
    // The wall nearest the click, if the click is on it or within 6 in of its face.
    let wall = {
        let Some((_, _, _, _, scene)) = &doc.draw.scene else { return Err("the plan is not ready: click again".to_string()) };
        let mut best: Option<(f64, String)> = None;
        for (a, b, thick, path) in &scene.walls {
            let (u, len2) = ([b[0] - a[0], b[1] - a[1]], (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2));
            if len2 < 1e-9 {
                continue;
            }
            let t = (((at[0] - a[0]) * u[0] + (at[1] - a[1]) * u[1]) / len2).clamp(0.0, 1.0);
            let d = (at[0] - (a[0] + u[0] * t)).hypot(at[1] - (a[1] + u[1] * t));
            if d <= thick / 2.0 + 0.5 && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                best = Some((d, path.clone()));
            }
        }
        best.map(|(_, path)| path).ok_or_else(|| "click on a wall's face, on the room's side of it".to_string())?
    };
    let rooms: Vec<String> = {
        let w = &doc.job.world;
        doc.job.order.iter().filter(|e| w.get::<Kind>(**e) == Some(&Kind::Room)).filter_map(|e| w.get::<plumbline_world::Node>(*e)).map(|n| n.path.clone()).collect()
    };
    // The rooms that have a face on this wall; the one the click is on the side of.
    let mut faces = Vec::new();
    let mut why = None;
    for room in rooms {
        match plumbline_world::building::wall_face(&mut doc.job, storey, &wall, &room) {
            Ok(face) => faces.push((room, face)),
            Err(e) => why = Some(e),
        }
    }
    if faces.is_empty() {
        return Err(match why {
            Some(why) if doc.job.find(&wall).is_some() => format!("{wall} has no room on either side that it closes: {why}"),
            _ => format!("{wall} has no room beside it: name the room first with the Room tool"),
        });
    }
    let side = |f: &plumbline_world::building::WallFace| (at[0] - f.start[0]) * f.into[0] + (at[1] - f.start[1]) * f.into[1];
    let (room, face) = faces.into_iter().filter(|(_, f)| side(f) > 0.0).max_by(|a, b| side(&a.1).total_cmp(&side(&b.1))).ok_or_else(|| format!("that is not on the face of {wall} any room has: click on the room's side of it"))?;
    let along = (at[0] - face.start[0]) * face.along[0] + (at[1] - face.start[1]) * face.along[1];
    let mut from = ((along * 12.0).round() / 12.0).max(0.0);
    if let Some(&(lo, _)) = face.clear.iter().min_by(|a, b| (from - a.0).abs().total_cmp(&(from - b.0).abs())) {
        if (from - lo).abs() <= 0.5 {
            from = lo;
        }
    }
    Ok((wall, room, from))
}

/// The boxes typed for a run, set along the wall the click found (call 156): what
/// `editing::cabinets` says is what the window says, and a refusal sets nothing.
pub fn cabinets_entered(doc: &mut Doc, values: &[String]) {
    let (Some((wall, room)), Some(storey)) = (doc.draw.cabinets.clone(), doc.show.storey.clone()) else {
        return cant(doc, "set the run", "click a wall's face first");
    };
    let from = values.first().map(|v| v.trim()).unwrap_or("").to_string();
    // The boxes as the command line takes them: codes side by side, and an appliance or any
    // item of the job by its name in square brackets, like [Dishwasher] (call 161).
    let codes = match ed::box_list(values.get(1).map(String::as_str).unwrap_or("")) {
        Ok(codes) => codes,
        Err(why) => {
            doc.reopen_form(Entry::Cabinets, values.to_vec(), 1);
            cant(doc, "set the run", why);
            return doc.redraw();
        }
    };
    let traced = doc.draw.traced.clone().unwrap_or_else(|| SOURCE.to_string());
    let top = !doc.draw.without_top;
    match ed::cabinets(&mut doc.job, &storey, &wall, &room, &from, &codes, top, how(&traced)) {
        Ok(made) => {
            doc.job.derive();
            doc.edit("set a run of cabinets");
            let num = plumbline_world::view::num;
            let mut said = format!(
                "Set {} along {wall}, in {room}: a run of {} ft, from {} ft to {} ft along the wall; {} ft left to the corner.",
                if made.boxes.len() == 1 { "1 box".to_string() } else { format!("{} boxes", made.boxes.len()) },
                num(made.run),
                num(made.from),
                num(made.from + made.run),
                num(made.left)
            );
            if let Some(up) = made.hung {
                said.push_str(&format!(" The boxes hang {} up.", units::feet_as_ft_in(up, 16)));
            }
            if let Some((path, _)) = &made.countertop {
                let figure = |doc: &mut Doc, name: &str| doc.job.find(path).and_then(|e| doc.job.world.get::<plumbline_world::components::Measure>(e)).and_then(|m| m.0.get(name).and_then(|d| d.worked()));
                if let (Some(long), Some(area)) = (figure(doc, "width"), figure(doc, "plan_area")) {
                    said.push_str(&format!("\nCountertop: {} ft long by 25 1/2 in deep, 34 1/2 in up; {} SF, front edge {} LF.", num(long), plumbline_world::view::figure(area, 2), num(long)));
                }
            }
            doc.draw.cabinets = None;
            doc.say(Tone::Done, said);
        }
        Err(why) => {
            let at = if why.contains("starts so far") || why.contains("where the run starts") { 0 } else { 1 };
            doc.reopen_form(Entry::Cabinets, values.to_vec(), at);
            // The window says "the tool chest" where the model does (call 169).
            // The list of what there is is under the field: the sentence does not say it again.
            let why = match why.split_once(". It has: ") {
                Some((cant_find, _)) => format!("{}. The list above the field has the boxes and the appliances this job has.", cant_find),
                None => why,
            };
            cant(doc, "set the run", why);
        }
    }
    doc.redraw();
}

/// Set the layout in the job file `name` (kept in this job's folder) beside this job, on the
/// sheet's Layouts tab (call 156). The file is read and never written.
pub fn set_beside(doc: &mut Doc, name: &str) {
    if doc.beside.len() >= 2 {
        return cant(doc, "set it beside this job", "two other layouts are set beside it already, the most. Take one away first");
    }
    let path = crate::plans::folder_of(&doc.path).join(name);
    let read = std::fs::read_to_string(&path).map_err(|e| format!("{name} cannot be read: {e}")).and_then(|text| file::load(&text).map_err(|why| format!("{name}: {why}")));
    match read {
        Ok((job, _)) => {
            let title = job.world.get::<plumbline_world::JobInfo>(job.root()).map(|i| i.title.clone()).unwrap_or_default();
            doc.beside.push(crate::model::Beside { file: name.to_string(), job });
            doc.choosing_layout = false;
            doc.say(Tone::Done, format!("Set {title} beside this job: the table is on the sheet, under Layouts. Nothing is written to either file."));
            doc.touch();
        }
        Err(why) => cant(doc, "set it beside this job", why),
    }
}

/// The jobs kept in this job's folder, each its file's name and its title: only files that
/// are jobs, and not this one (call 161). Read when a list of them is opened, not at every
/// frame.
pub fn jobs_here(doc: &Doc) -> Vec<(String, String)> {
    let here = crate::plans::folder_of(&doc.path);
    let mine = doc.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let mut found: Vec<(String, String)> = std::fs::read_dir(&here)
        .map(|read| {
            read.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_string))
                .filter(|n| !n.starts_with('.') && n.to_lowercase().ends_with(".json") && *n != mine)
                .filter_map(|n| {
                    let text = std::fs::read_to_string(here.join(&n)).ok()?;
                    let (job, _) = file::load(&text).ok()?;
                    let title = job.world.get::<plumbline_world::JobInfo>(job.root()).map(|i| i.title.clone()).filter(|t| !t.trim().is_empty()).unwrap_or_else(|| crate::panels::plain_stem(&n));
                    Some((n, title))
                })
                .collect()
        })
        .unwrap_or_default();
    found.sort_by_key(|(n, t)| (t.to_lowercase(), n.to_lowercase()));
    found
}

/// "Set beside another layout" or "Open another layout" pressed: the list of jobs in this
/// job's folder is opened (or put away again).
pub fn choose_layout(doc: &mut Doc, to_open: bool) {
    let (was_set, was_open) = (doc.choosing_layout, doc.opening_layout);
    doc.choosing_layout = !to_open && !was_set;
    doc.opening_layout = to_open && !was_open;
    if doc.entry.as_ref().is_some_and(|(e, _)| *e == Entry::LayoutName) {
        doc.entry = None;
    }
    if doc.choosing_layout || doc.opening_layout {
        doc.jobs_here = jobs_here(doc);
    }
    doc.touch();
}

/// "Keep this as another layout" pressed: the title is asked for, this job's own title and
/// "B" offered (call 161).
pub fn ask_layout_name(doc: &mut Doc) {
    doc.choosing_layout = false;
    doc.opening_layout = false;
    let title = doc.job.world.get::<plumbline_world::JobInfo>(doc.job.root()).map(|i| i.title.clone()).unwrap_or_default();
    doc.open(Entry::LayoutName, format!("{} B", title.trim()));
    doc.fresh = true;
    doc.say(Tone::Done, "Keep this as another layout: type its title, then Enter. Esc lets go.");
}

/// Keep a copy of this job, as it is on screen now, beside it under `title`, and set it
/// beside this job (call 161). Its file is named from the title, in this job's folder;
/// a file of that name is never written over. This job's own file is not touched.
pub fn keep_layout(doc: &mut Doc, title: &str) {
    let title = title.trim();
    if title.is_empty() {
        doc.open(Entry::LayoutName, "");
        return cant(doc, "keep this as another layout", "it needs a title: type one, then Enter");
    }
    if doc.beside.len() >= 2 {
        return cant(doc, "keep this as another layout", "two other layouts are set beside this job already, the most. Take one away first");
    }
    let name = format!("{}.json", plumbline_world::job::slug(title));
    let folder = crate::plans::folder_of(&doc.path);
    let to = folder.join(&name);
    if to.exists() || doc.path.file_name().is_some_and(|n| n.to_string_lossy() == name) {
        doc.open(Entry::LayoutName, title);
        doc.fresh = true;
        return cant(doc, "keep this as another layout", format!("a job is kept here under that title already. Give this layout another title"));
    }
    let made = file::load(&file::save(&mut doc.job)).map(|(job, _)| job);
    match made {
        Ok(mut job) => {
            let root = job.root();
            if let Some(mut info) = job.world.get_mut::<plumbline_world::JobInfo>(root) {
                info.title = title.to_string();
            }
            let text = file::save(&mut job);
            match std::fs::write(&to, &text) {
                Ok(()) => match file::load(&text) {
                    Ok((job, _)) => {
                        doc.beside.push(crate::model::Beside { file: name.clone(), job });
                        doc.say(Tone::Done, format!("Kept a copy of this job as {title}, beside it, and set it beside this one: the table is on the sheet, under Layouts. Open another layout opens it to be changed; this job's own file is as it was."));
                    }
                    Err(why) => cant(doc, "keep this as another layout", why),
                },
                Err(e) => cant(doc, "keep this as another layout", format!("{} {}", crate::panels::plain_stem(&name), crate::plans::why_not_opened(&e))),
            }
        }
        Err(why) => cant(doc, "keep this as another layout", why),
    }
    doc.touch();
}

/// "Open another layout": the job in the file `name`, kept in this job's folder, is opened
/// in this window in place of this one. With changes not saved it opens nothing (call 161).
pub fn open_layout(doc: &mut Doc, name: &str) {
    if doc.edited {
        doc.say(Tone::Ask, "Save first (Ctrl+S).");
        doc.touch();
        return;
    }
    let path = crate::plans::folder_of(&doc.path).join(name);
    match crate::build_doc(path.to_string_lossy().to_string(), None) {
        Ok(mut next) => {
            let title = next.job.world.get::<plumbline_world::JobInfo>(next.job.root()).map(|i| i.title.clone()).unwrap_or_default();
            next.revision = doc.revision + 1;
            next.geometry = doc.geometry + 1;
            // The pictures taken go on being counted: the next is not the first again.
            next.shots = doc.shots;
            // The sheet stays on the Layouts tab, where it was asked for.
            next.tab = doc.tab;
            next.say(Tone::Done, format!("Opened {title}. The other layouts are as they were; set one beside this to compare."));
            *doc = next;
        }
        Err(why) => cant(doc, "open that layout", why),
    }
    doc.touch();
}

/// Take the `which`th layout set beside this job away; the job file it was read from is
/// as it was.
pub fn take_layout_away(doc: &mut Doc, which: usize) {
    if which < doc.beside.len() {
        let gone = doc.beside.remove(which);
        let title = gone.job.world.get::<plumbline_world::JobInfo>(gone.job.root()).map(|i| i.title.clone()).filter(|t| !t.is_empty()).unwrap_or_else(|| crate::panels::plain_stem(&gone.file));
        doc.say(Tone::Done, format!("Took {title} away from beside this job. Its file is as it was."));
        doc.touch();
    }
}

/// What a tool does, as a refusal says it: "Can't draw the wall".
fn doing(tool: Tool) -> String {
    match tool {
        Tool::Wall | Tool::Run(_) => format!("draw the {}", tool.name().to_lowercase()),
        Tool::Door | Tool::Window | Tool::Thing(_) | Tool::Floor | Tool::Room | Tool::Roof | Tool::Fitting => format!("add the {}", tool.name().to_lowercase()),
        Tool::RoomLine => "draw the room line".to_string(),
        Tool::Cabinets => "set the run".to_string(),
        Tool::Move => "move it".to_string(),
        Tool::Join => "connect it".to_string(),
        Tool::Scale => "set the scale".to_string(),
        Tool::LineUp => "line the drawing up".to_string(),
    }
}

/// A length typed while a wall or a run is being drawn: the next point is that far from
/// the last one, toward where the pointer is.
pub fn typed_length(doc: &mut Doc, text: &str) {
    let Some(last) = doc.draw.picked.last().copied() else { return };
    let toward = doc.draw.found.as_ref().map(|f| f.at).unwrap_or([last[0] + 1.0, last[1]]);
    match units::parse_length(text).and_then(|(len, unit)| len.feet(unit)) {
        Ok(feet) if feet > 0.0 => {
            doc.draw.typed = Some(feet);
            pick(doc, ed::toward(last, toward, feet), None);
            doc.draw.typed = None;
        }
        Ok(_) => cant(doc, "use that length", "a length is more than nothing"),
        Err(why) => cant(doc, "use that length", why),
    }
    doc.touch();
}

/// What was measured on site, typed or sent by a laser, for the value picked in the
/// panel: the reading as the tape or the laser has it, or "same" when the value is as
/// drawn. For a run's height, "bottom" or "top" before the reading says where the shot
/// was taken to. Who measured can follow a comma. The value becomes Known.
pub fn measured(doc: &mut Doc, text: &str) {
    let Some((path, what, _)) = doc.measuring.take() else { return };
    // Who measured comes after a comma and a space, or after a comma before anything
    // that is not a figure. A comma hard against a figure (8,5 ft) is part of the
    // reading, and is refused there for what it is.
    let (reading, who) = match text.split_once(", ") {
        Some((reading, who)) => (reading.trim(), who.trim()),
        None => match text.split_once(',') {
            Some((reading, who)) if !who.starts_with(|c: char| c.is_ascii_digit()) => (reading.trim(), who.trim()),
            _ => (text.trim(), ""),
        },
    };
    let line = |word: &&str| match word.to_ascii_lowercase().as_str() {
        "bottom" => Some(plumbline_world::Justify::Bottom),
        "top" => Some(plumbline_world::Justify::Top),
        "centre" | "center" => Some(plumbline_world::Justify::Centre),
        _ => None,
    };
    let mut words: Vec<&str> = reading.split_whitespace().collect();
    let mut to = None;
    if let Some(j) = words.first().and_then(line) {
        to = Some(j);
        words.remove(0);
    } else if let Some(j) = words.last().and_then(line) {
        to = Some(j);
        words.pop();
    }
    let reading = words.join(" ");
    if reading.is_empty() {
        cant(doc, "mark it", "nothing was typed. Type what the tape or the laser reads, or same if it is as drawn.");
        doc.touch();
        return;
    }
    // An item's size (call 197, points 8 and 9): typed, it is the item changed and is Assumed; typed the same again it
    // is checked. The model's own measurement does both of its steps; what the window adds is the words and the
    // certainty, and nothing is worked out here.
    if to.is_none() && item_size(doc, &path, &what, &reading, who) {
        doc.redraw();
        return;
    }
    let same = reading.eq_ignore_ascii_case("same");
    let by = if same { "checked on site" } else { "measured on site" };
    let source = if who.is_empty() { by.to_string() } else { format!("{by}: {who}") };
    match ed::measure(&mut doc.job, &path, &what, if same { ed::Reading::Same } else { ed::Reading::Length(&reading) }, to, &source) {
        Ok(done) => {
            doc.edit("measured it");
            // What else moved with it has to be seen: it is said in full over the model.
            let tone = if done.also.is_empty() { Tone::Done } else { Tone::Warned };
            let mut said = done.said;
            for also in &done.also {
                said.push('\n');
                said.push_str(also);
            }
            doc.say(tone, said);
        }
        Err(why) => cant(doc, "use that reading", why),
    }
    doc.redraw();
}

/// A size typed on an item's panel. True when it was an item's size and has been dealt with (said, done or
/// refused); false when it is anything else, or not a length, and goes the way a measurement goes.
/// What it is and what follows it is the model's (`ed::item_size`): differing from the size it has, the item changed and
/// Assumed, in the words New item uses; the same size again (or "same"), the check, Known, "checked on site".
fn item_size(doc: &mut Doc, path: &str, what: &str, reading: &str, who: &str) -> bool {
    use plumbline_world::Item;
    // The guard is the window's: only an item's width, depth or height, and only a length or the word "same". What a size
    // typed there is, and what follows it, is the model's (`editing::item_size`, call 200).
    let Some(e) = doc.job.find(path) else { return false };
    if doc.job.world.get::<Item>(e).is_none() || !matches!(what, "width" | "depth" | "height") {
        return false;
    }
    if !reading.eq_ignore_ascii_case("same") && units::parse_length(reading).is_err() {
        return false;
    }
    match ed::item_size(&mut doc.job, path, what, reading, who) {
        Ok(done) => {
            doc.edit(if done.how == ed::HowSized::Checked { "checked an item's size" } else { "typed an item's size" });
            doc.say(if done.name_carries_old { Tone::Warned } else { Tone::Done }, done.said());
        }
        Err(why) => cant(doc, "use that size", why),
    }
    true
}

/// A roof worked out from the footprint picked: its corners (turned counter-clockwise
/// if they were picked the other way), one pitch on every side, and the height of its
/// eaves. It is called "roof", and the next "roof 2".
fn roof_over(doc: &mut Doc, corners: &[[f64; 2]], pitch: &str, eaves: &str, source: &str) -> Result<String, String> {
    use plumbline_world::{solve, Footprint, Rise, Val};
    let rise = units::parse_pitch(pitch)?;
    let (len, unit) = units::parse_length(eaves)?;
    let mut points = corners.to_vec();
    let twice_area: f64 = (0..points.len()).map(|i| points[i][0] * points[(i + 1) % points.len()][1] - points[(i + 1) % points.len()][0] * points[i][1]).sum();
    if twice_area < 0.0 {
        points.reverse();
    }
    let entered = |x| Val { value: Some(x), unit: units::PITCH.into(), status: Status::Assumed, source: source.to_string() };
    let footprint = Footprint {
        edges: points.iter().map(|_| entered(Rise::Pitch(rise))).collect(),
        points: Val { value: Some(points), unit: units::FT.into(), status: Status::Assumed, source: source.to_string() },
        eave_height: Val { value: Some(len), unit: unit.into(), status: Status::Assumed, source: source.to_string() },
    };
    let taken: Vec<String> = {
        let mut q = doc.job.world.query::<(&plumbline_world::Node, &Kind)>();
        q.iter(&doc.job.world).filter(|(_, k)| **k == Kind::Roof).map(|(n, _)| n.name.clone()).collect()
    };
    let name = if taken.iter().any(|n| n == "roof") { (2..).map(|n| format!("roof {n}")).find(|n| !taken.contains(n)).unwrap_or_else(|| "roof".to_string()) } else { "roof".to_string() };
    let (path, report) = solve::new_roof(&mut doc.job, &name, footprint)?;
    doc.roof = Some(path.clone());
    Ok(format!("Added {path}: {} plane{}, {pitch} on every side, eaves at {eaves}.", report.planes, if report.planes == 1 { "" } else { "s" }))
}

/// The name typed for a room, after a point inside it was picked.
pub fn room_named(doc: &mut Doc, text: &str) {
    let (at, storey) = (doc.draw.picked.last().copied(), doc.show.storey.clone());
    let traced = doc.draw.traced.take().unwrap_or_else(|| SOURCE.to_string());
    doc.draw.picked.clear();
    let made = match (at, storey) {
        (Some(at), Some(storey)) if !text.trim().is_empty() => ed::room(&mut doc.job, &storey, text.trim(), at, how(&traced)),
        (Some(_), Some(_)) => Err("a room has a name. Type it, like Kitchen.".to_string()),
        _ => Err("click inside the room first".to_string()),
    };
    match made {
        Ok(path) => {
            doc.job.derive();
            doc.edit("drew a room");
            // What it came to is said with it: a room that took the whole floor
            // because its walls do not close is seen here, at once, and not three
            // rooms later (J22A-2). The figure is the model's.
            let area = doc.job.find(&path).and_then(|e| doc.job.world.get::<plumbline_world::Measure>(e)).and_then(|m| m.0.get("floor_area").cloned());
            let said = match area.as_ref().map(|d| (d.value, d.note.as_str())) {
                Some((Some(x), _)) => format!("{} SF inside its walls. If that is more than the room, a wall round it does not close: draw it to meet. Click inside the next room, or Esc.", plumbline_world::view::figure(x, 2)),
                // The model's own sentence names the room and what to do (call 169).
                Some((None, why)) if why.contains(" has no area yet: ") => why.to_string(),
                Some((None, why)) if !why.is_empty() => format!("It has no area yet: {why}."),
                _ => "Click inside the next one, or Esc.".to_string(),
            };
            doc.say(Tone::Done, format!("Added room {:?}: {said}", text.trim()));
        }
        Err(why) => cant(doc, "add the room", why),
    }
    doc.redraw();
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
                plumbline_world::plans::calibrate(&mut doc.job, &l.path, from, to, text, ed::Entered { status: Status::Assumed, source: "read off the drawing" }).map(|done| {
                    let mut said = format!("{}: {} between the two points; the drawing is {} wide", l.name, text.trim(), units::feet_as_ft_in(l.drawing.size[0] * done.feet_per_unit, 16));
                    if !done.left_at_earlier_scale.is_empty() {
                        said.push_str(&format!(". NOT MOVED: {} traced at the scale it had before; check {} against the sheet", if done.left_at_earlier_scale.len() == 1 { "1 thing".to_string() } else { format!("{} things", done.left_at_earlier_scale.len()) }, if done.left_at_earlier_scale.len() == 1 { "it" } else { "them" }));
                    }
                    (said, !done.left_at_earlier_scale.is_empty())
                })
            }
        },
        _ => Err("click the two ends of a dimension first".to_string()),
    };
    match made {
        Ok((said, warned)) => {
            doc.edit("set the scale");
            doc.framed = false;
            // What was left at the old scale is said the way a refusal is: it has to be seen.
            doc.say(if warned { Tone::Refused } else { Tone::Done }, said);
        }
        Err(why) => cant(doc, "set the scale", why),
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
            doc.let_go_of_what_is_gone();
            // And a roof the step brought back is taken up again: it was in the job
            // and priced, and not drawn until the job was opened again (J27A-22).
            if doc.roof.is_none() {
                doc.roof = doc.job.order.iter().find_map(|e| (doc.job.world.get::<Kind>(*e) == Some(&Kind::Roof)).then(|| doc.job.world.get::<plumbline_world::Node>(*e).map(|n| n.path.clone())).flatten());
            }
            doc.carried = plumbline_world::view::carried_now(&mut doc.job);
            doc.say(Tone::Done, format!("{}: {label}.", if redo { "Redone" } else { "Undone" }));
        }
        Err(why) => cant(doc, if redo { "redo" } else { "undo" }, why),
    }
    doc.pointer = None;
    doc.hover = None;
    doc.redraw();
}

/// What is said of a wall the work opens (call 160), in the window: whether it is bearing,
/// and what runs inside it. Said on screen it is Assumed, as every typed value is, until it
/// is looked at in the field.
pub fn said_of_wall(doc: &mut Doc, what: &str, word: &str) {
    let Some(sel) = doc.selected.clone() else { return cant(doc, "say it", "nothing is selected") };
    let name = doc.job.find(&sel).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| sel.clone());
    if word.is_empty() {
        return cant(doc, "say what is in the wall", "type it in words, like a 2 in drain and two cables, or nothing");
    }
    match doc.job.set(&sel, what, Some(word), Status::Assumed, crate::model::PRICED) {
        Ok(()) => {
            doc.job.derive();
            doc.edit(if what == "bearing" { "Bearing" } else { "In the wall" });
            let said = if what == "bearing" { format!("{name} is {word}") } else { format!("In {name}: {word}") };
            doc.say(Tone::Done, format!("{said}. Assumed: entered on screen, until it is looked at in the field."));
        }
        Err(why) => cant(doc, "say it", why),
    }
    doc.redraw();
}

/// One cost line taken off a thing (call 166), with whatever was entered on it. Says what
/// the line came to before it went, as the estimate had it. One step to undo.
pub fn take_line_off(doc: &mut Doc, path: &str, name: &str, description: &str) {
    doc.job.derive();
    let before = plumbline_world::view::estimate(&mut doc.job);
    let was = before.lines.iter().find(|l| l.path == path && l.description == description).map(|l| (l.quantity.value, l.quantity.unit.clone(), l.amount.value));
    match doc.job.remove_line(path, description) {
        Ok(_) => {
            doc.job.derive();
            doc.edit("took a line off");
            let num = plumbline_world::view::num;
            let stood = match was {
                Some((Some(q), unit, Some(amount))) => format!("{} {unit}, {}", num(q), plumbline_world::view::money(amount)),
                Some((Some(q), unit, None)) => format!("{} {unit}, with no price on it", num(q)),
                _ => "with no figure on it".to_string(),
            };
            doc.say(Tone::Done, format!("Took {description:?} off {name}: {stood}. Undo puts it back."));
            doc.redraw();
        }
        Err(why) => cant(doc, "take the line off", why),
    }
}

/// Every wall (door, window, floor, footing) on the level this one is on, selected, this
/// one in hand (call 169). One click for "mark all the walls existing".
pub fn select_all_on_level(doc: &mut Doc, path: &str, kind: Kind) {
    let on = plumbline_world::building::storeys_of(&mut doc.job);
    let Some(level) = on.get(path).cloned() else { return cant(doc, "select them", "it is on no level") };
    let mut same: Vec<String> = on.iter().filter(|(p, l)| **l == level && p.as_str() != path && doc.job.kind(p) == Some(kind)).map(|(p, _)| p.clone()).collect();
    same.sort();
    let name = plumbline_world::building::storey_totals(&mut doc.job).into_iter().find(|s| s.path == level).map(|s| crate::panels::pretty(&s.name)).unwrap_or(level);
    let n = same.len() + 1;
    doc.choosing = false;
    let unmarked = doc.left_unmarked().map(|u| format!("\n{u}")).unwrap_or_default();
    doc.together = same;
    doc.selected = Some(path.to_string());
    let what = theme::kind_name(kind).to_lowercase();
    doc.say(Tone::Done, format!("Selected all {n} {what}{} on {name}. WORK and Delete act on all of them; Esc lets go.{unmarked}", if n == 1 { "" } else { "s" }));
    doc.touch();
}

/// Take what is selected out of the job.
pub fn delete(doc: &mut Doc) {
    // Several selected (call 169): all go, in one step that one Undo takes back.
    if !doc.together.is_empty() {
        let before = plumbline_world::view::estimate(&mut doc.job);
        let mut gone = Vec::new();
        for path in doc.all_selected() {
            if doc.job.find(&path).is_none() {
                continue;
            }
            match ed::remove(&mut doc.job, &path) {
                Ok(g) => gone.extend(g),
                Err(why) => {
                    if gone.is_empty() {
                        return cant(doc, "delete them", why);
                    }
                    break;
                }
            }
        }
        let went = plumbline_world::view::went_with_said(&before, &mut doc.job);
        doc.selected = None;
        doc.together.clear();
        doc.hover = None;
        doc.let_go_of_what_is_gone();
        doc.edit("deleted it");
        let said = format!("Deleted {}: {}.", plumbline_world::view::so_many(gone.len(), "thing"), gone.join(", "));
        match went {
            Some(went) => doc.say(Tone::Warned, format!("{said} {}{}", went[..1].to_uppercase(), &went[1..])),
            None => doc.say(Tone::Done, said),
        }
        doc.redraw();
        return;
    }
    match doc.selected.clone() {
        None => cant(doc, "delete", "nothing is selected. Click what is to be deleted first."),
        Some(sel) => {
            // A drawing taken away leaves what was traced over it: how much is said.
            let traced = plumbline_world::plans::traced_over(&mut doc.job, &sel).ok().map(|t| (doc.job.find(&sel).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone())).unwrap_or_default(), t.len()));
            // What is priced on it, and on what goes with it, is said as it goes.
            let before = plumbline_world::view::estimate(&mut doc.job);
            match ed::remove(&mut doc.job, &sel) {
            Ok(gone) => {
                let went = plumbline_world::view::went_with_said(&before, &mut doc.job);
                doc.selected = None;
                doc.hover = None;
                // A level deleted while it was on show alone, a roof, a thing in hand.
                doc.let_go_of_what_is_gone();
                doc.edit("deleted it");
                let said = match &traced {
                    Some((name, n)) if *n > 0 => format!("Deleted {name}. {} traced over it {} where {} and still {} so.", if *n == 1 { "1 thing".to_string() } else { format!("{n} things") }, if *n == 1 { "stays" } else { "stay" }, if *n == 1 { "it was" } else { "they were" }, if *n == 1 { "says" } else { "say" }),
                    // A roof takes its planes and their eaves, hips and ridges: named
                    // one by one they filled the card.
                    _ if gone.len() > 4 => format!("Deleted {}, and {} with it.", gone.last().cloned().unwrap_or_default(), plumbline_world::view::so_many(gone.len() - 1, "thing")),
                    _ => format!("Deleted {}.", gone.join(", ")),
                };
                match went {
                    Some(went) => doc.say(Tone::Warned, format!("{said} {}{}", went[..1].to_uppercase(), &went[1..])),
                    None => doc.say(Tone::Done, said),
                }
            }
            Err(why) => cant(doc, "delete it", why),
            }
        }
    }
    doc.redraw();
}

/// Whether the pointer is over the model and not over a panel.
fn over_model(window: &Window, at: Vec2) -> bool {
    over_model_at(window, at)
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
        let mut sheets = Vec::new();
        for l in snap_shot.drawings.iter().filter(|l| l.storey == storey) {
            if let Some(read) = plans.read.get(&l.path) {
                for line in &read.lines {
                    scene.segment(l.on_plan(line[0]), l.on_plan(line[1]), None);
                }
                // A scan's sheet shows the faces of its walls: places to pick, and what a wall
                // traced over it is set by (call 190).
                if let Some(faces) = crate::plans::sheet_faces(l, read) {
                    for f in &faces.faces {
                        scene.segment(f.line[0], f.line[1], None);
                    }
                    sheets.push(faces);
                }
            }
        }
        doc.draw.sheets = sheets;
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
        Tool::Wall | Tool::RoomLine | Tool::Run(_) | Tool::Move => snap::snap(scene, p, reach, picked, square, &snap::Snaps::default()),
        // A roof's outline is drawn corner to corner, as a wall is.
        Tool::Roof => snap::snap(scene, p, reach, picked, square, &snap::Snaps::default()),
        // A floor and a room are a click anywhere inside: nothing to snap to.
        Tool::Floor | Tool::Room | Tool::Cabinets => snap::snap(scene, p, reach, &[], false, &snap::Snaps::none()),
        Tool::Thing(_) | Tool::Fitting | Tool::Join | Tool::Scale | Tool::LineUp => snap::snap(scene, p, reach, &[], false, &snap::Snaps::default()),
    };
    doc.pointer = Some([found.at[0], found.at[1], elevation]);
    doc.draw.scale = scale;
    doc.draw.found = Some(found);
}

/// A click on the model with a tool in hand is a pick.
pub fn clicks(mut doc: ResMut<Doc>, mouse: Res<ButtonInput<MouseButton>>, windows: Query<&Window, With<PrimaryWindow>>) {
    // A press on what is to be moved, a drag and a release is a move (call 193, E): the press was the point
    // taken, and the release, further along, is where it goes. A release close to the press is a click, and
    // the next click says where it goes, as it always did.
    if mouse.just_released(MouseButton::Left) {
        if let Some(from) = doc.draw.pressed_at.take() {
            let to = windows.single().ok().and_then(|w| w.cursor_position());
            if doc.draw.tool == Some(Tool::Move) && doc.draw.picked.len() == 1 && doc.entry.is_none() && to.is_some_and(|c| c.distance(from) > 8.0) {
                match doc.draw.found.clone() {
                    Some(found) => pick(&mut doc, found.at, found.source),
                    // Let go over a panel: no place on the plan to take.
                    None => {
                        doc.say(Tone::Ask, "Move: let go over a panel, where there is no place on the plan. Click where it goes, or press on it and drag again.");
                        doc.touch();
                    }
                }
            }
        }
    }
    if mouse.just_pressed(MouseButton::Left) && doc.draw.swallow {
        doc.bypass_change_detection().draw.swallow = false;
    }
    // A click on the model while a tool's sizes are still open places nothing, and
    // says so (J18-2): it is never a click that does nothing.
    if mouse.just_pressed(MouseButton::Left) {
        if let Some((Entry::Settings(tool), _)) = doc.entry.clone() {
            if windows.single().ok().and_then(|w| w.cursor_position().map(|c| over_model(w, c))).unwrap_or(false) {
                // Every field still empty, the one being typed in too if nothing is in
                // it yet (call 86: it named the others and left that one out).
                let typed_so_far = doc.entry.as_ref().map(|(_, t)| t.trim().is_empty()).unwrap_or(true);
                let empty = doc.form.iter().enumerate().filter(|(i, v)| if *i == doc.form_at { typed_so_far } else { v.is_empty() }).map(|(i, _)| tool.fields()[i].0).collect::<Vec<_>>();
                doc.say(Tone::Ask, format!("Nothing was placed: the {}'s sizes are still open. {}Press Enter first, then click.", tool.name().to_lowercase(), if empty.is_empty() { String::new() } else { format!("Still to be typed: {}. ", empty.join(", ")) }));
                doc.touch();
            }
        }
    }
    // A click on the plan with a length typed and not yet entered places nothing either:
    // that is said (call 86, R21-8).
    if mouse.just_pressed(MouseButton::Left) && doc.draw.tool.is_some() && matches!(doc.entry, Some((Entry::Length, _))) && windows.single().ok().and_then(|w| w.cursor_position().map(|c| over_model(w, c))).unwrap_or(false) {
        doc.say(Tone::Ask, "Nothing was placed: the length typed is not set yet. Press Enter to go that far toward the pointer, or Esc to let the length go.");
        doc.touch();
    }
    // The same while a run's boxes are being typed (call 156).
    if mouse.just_pressed(MouseButton::Left) && doc.draw.tool == Some(Tool::Cabinets) && matches!(doc.entry, Some((Entry::Cabinets, _))) && windows.single().ok().and_then(|w| w.cursor_position().map(|c| over_model(w, c))).unwrap_or(false) {
        doc.say(Tone::Ask, "Nothing was set: the run's boxes are not entered yet. Press Enter to set the run, or Esc to let it go, then click the next wall.");
        doc.touch();
    }
    if !mouse.just_pressed(MouseButton::Left) || doc.draw.tool.is_none() || doc.entry.is_some() || !doc.plan {
        return;
    }
    let Ok(window) = windows.single() else { return };
    if !window.cursor_position().is_some_and(|c| over_model(window, c)) {
        return;
    }
    let Some(found) = doc.draw.found.clone() else { return };
    let from_here = doc.draw.tool == Some(Tool::Move) && doc.draw.picked.is_empty();
    pick(&mut doc, found.at, found.source);
    doc.draw.pressed_at = if from_here && doc.draw.picked.len() == 1 { window.cursor_position() } else { None };
}

/// The frame begins: nothing has been said in it yet. (What a later part of the last
/// frame said is not what the next step is remembered by.)
pub fn frame_begins(mut doc: ResMut<Doc>) {
    if doc.draw.said.is_some() {
        doc.bypass_change_detection().draw.said = None;
    }
}

/// Each edit the window has made since the last look becomes a step that can be undone.
pub fn record(mut doc: ResMut<Doc>) {
    if doc.draw.edits == doc.draw.seen {
        return;
    }
    let doc = &mut *doc;
    doc.draw.seen = doc.draw.edits;
    let before = std::mem::take(&mut doc.draw.last_text);
    // The step is remembered by what the window said as it was made ("Plane 1: unit
    // cost of "Asphalt shingles" set to $105.00"), its first line; where nothing was
    // said, by the short name of what was done.
    let label = match doc.draw.said.take().as_deref().and_then(|m| m.lines().next()).map(str::trim) {
        Some(line) if !line.is_empty() => line.trim_end_matches('.').to_string(),
        _ => doc.draw.label.clone(),
    };
    doc.draw.history.record(&label, before, &mut doc.job);
    doc.draw.last_text = file::save(&mut doc.job);
}

/// What is being drawn, drawn: the points picked, the line to where the next would land,
/// and a mark there.
pub fn preview(doc: Res<Doc>, snap_shot: Res<Snapshot>, mut lines: Gizmos<Highlight>, mut fine: Gizmos<crate::scene::GridLines>) {
    let (Some(_), Some(storey)) = (doc.draw.tool, doc.show.storey.as_ref()) else { return };
    // Points picked on another storey are not this storey's to show.
    let picked: &[[f64; 2]] = if doc.draw.picked_on.as_ref() == Some(storey) { &doc.draw.picked } else { &[] };
    let elevation = snap_shot.storeys.iter().find(|s| s.path == *storey).and_then(|s| s.elevation).unwrap_or(0.0);
    let o = doc.origin;
    let place = |p: [f64; 2]| Vec3::new((p[0] - o[0]) as f32, (elevation - o[2]) as f32 + 0.3, -(p[1] - o[1]) as f32);
    let ink = theme::accent();
    // With the Wall tool up over a scan, the faces a click will land on are drawn as fine lines (call 193, G).
    if doc.draw.tool == Some(Tool::Wall) {
        for sheet in &doc.draw.sheets {
            for f in &sheet.faces {
                fine.line(place(f.line[0]), place(f.line[1]), ink.with_alpha(0.8));
            }
        }
    }
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

#[cfg(test)]
mod run_tests {
    use super::moved_by;

    #[test]
    fn a_run_laid_by_the_face_and_then_on_the_center_line_moved_3_in() {
        // The made-up house's south and east walls by the face (3 in outside), then on the faces' own lines.
        let before = [[[-0.25, -0.25], [30.25, -0.25]], [[30.25, -0.25], [30.25, 15.75]]];
        let after = [[[0.0, 0.0], [30.0, 0.0]], [[30.0, 0.0], [30.0, 15.75]]];
        let (count, far) = moved_by(&before, &after);
        assert_eq!(count, 2);
        assert!((far - 0.25).abs() < 1e-9, "{far}");
    }

    #[test]
    fn walls_that_stay_are_not_counted() {
        let same = [[[0.0, 0.0], [10.0, 0.0]]];
        assert_eq!(moved_by(&same, &same), (0, 0.0));
        // A wall that only ran on along itself: moved by how far its end went.
        let (count, far) = moved_by(&same, &[[[0.0, 0.0], [10.5, 0.0]]]);
        assert_eq!(count, 1);
        assert!((far - 0.5).abs() < 1e-9);
    }
}

/// Call 197 (points 10 to 13 and 16): the words the window gives, and where each is worked out.
#[cfg(test)]
mod w197_tests {
    use super::*;
    use plumbline_world::value::Status;

    fn doc_with_a_wall() -> (Doc, String) {
        let mut doc = crate::build_doc("/nonexistent/w197-test.json".to_string(), None).expect("a new job");
        let how = ed::Entered { status: Status::Assumed, source: "made-up plan" };
        let level = ed::storey(&mut doc.job, "Ground", "0 ft", "9 ft", how).expect("a level");
        let t = ed::wall_type_for(&mut doc.job, "6 in", how).expect("a wall type");
        ed::walls(&mut doc.job, &level, &t, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), how).expect("a wall");
        doc.job.derive();
        doc.show.storey = Some(level.clone());
        (doc, level)
    }

    #[test]
    fn a_corner_refused_says_where_the_run_still_starts() {
        let (mut doc, _) = doc_with_a_wall();
        doc.draw.tool = Some(Tool::Wall);
        doc.draw.settings.insert("Wall", vec!["6 in".to_string(), "9 ft".to_string()]);
        // A new wall laid along the one that stands: the model refuses it.
        pick(&mut doc, [4.0, 0.0], None);
        pick(&mut doc, [12.0, 0.0], None);
        assert!(doc.message.starts_with("Can't draw the wall:"), "{}", doc.message);
        assert!(doc.message.contains("The run of walls still starts at (4' 0\", 0' 0\"): click the next corner, or press Esc to let it go."), "{}", doc.message);
    }

    #[test]
    fn a_mark_asked_for_and_left_says_nothing_was_marked() {
        let (mut doc, _) = doc_with_a_wall();
        let wall = doc.job.order.iter().filter_map(|e| doc.job.world.get::<plumbline_world::Node>(*e)).find(|n| n.path.starts_with("wall-")).map(|n| n.path.clone()).expect("the wall");
        doc.marking = Some((wall.clone(), plumbline_world::WorkStatus::Demolish));
        doc.select("level-0002");
        assert!(doc.marking.is_none());
        assert!(doc.message.contains("was not marked to demolish: its question was left unanswered, and nothing was changed."), "{}", doc.message);
        // Nothing pending: nothing said of it.
        doc.select(&wall);
        assert!(!doc.message.contains("not marked"), "{}", doc.message);
    }

    #[test]
    fn what_an_assembly_changed_for_another_did_to_a_shared_line_is_said_in_the_command_lines_words() {
        let rewritten = vec![plumbline_world::Rewritten { line: "Lap siding".into(), was: "200.00 SF".into(), now: "140.00 SF".into(), was_written: "net_area".into(), now_written: "net_area - net_area_below_wainscot".into() }];
        let left = vec![plumbline_world::Left { line: "Corner boards".into(), has: "18.00 LF".into(), has_written: "2 * height".into(), would: "14.00 LF".into(), would_written: "2 * (height - wainscot)".into(), why: plumbline_world::LeftAsIs::Entered }];
        let said = crate::shared_lines_said(&rewritten, &left);
        assert!(said.contains("1 cost line it already had now reads as the new assembly's quantity, and kept its price, its waste and how it is ordered:\n  Lap siding: 200.00 SF to 140.00 SF (net_area to net_area - net_area_below_wainscot)"), "{said}");
        assert!(said.contains("1 cost line it already had was left as it is:\n  Corner boards: left as entered, 18.00 LF (2 * height); the new assembly writes 2 * (height - wainscot), which comes to 14.00 LF"), "{said}");
        assert_eq!(crate::shared_lines_said(&[], &[]), "");
    }
}
