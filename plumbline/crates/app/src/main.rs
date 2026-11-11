//! The 3D window and the sheet window.
//!
//! Both are views over one job world (`plumbline-world`). Nothing here works out a
//! quantity: the windows draw what the world holds, send the estimator's edits to it,
//! and draw again. The sheet is a second operating-system window on the same world, so
//! it can sit on another monitor and it follows every edit as it is made.
//!
//!   plumbline-app <job file> [roof path]
//!
//! The whole building is drawn: walls, floors, doors and windows, the roof, and the ducts,
//! pipes, conduit, cable, equipment and fixtures in it. Click anything to select it; click
//! the ground to let go. Click a row of the sheet to select it in the 3D view.
//!
//! Moving: drag with the right button to turn, with the middle button to pan; scroll to
//! zoom; W A S D move over the ground, Q and E go down and up.
//!   T          plan view or 3D                     V        go in on foot, or come out
//!   F          frame the building                  Esc      let go of the selection
//! What is on show:
//!   1 architecture   2 roof   3 mechanical   4 electrical   5 plumbing
//!   X          see through the building            PgUp/PgDn  one storey at a time
//! On the selected plane of a roof solved from a footprint:
//!   = / -      pitch up or down by 1 in 12        ] / [    move its eave out or in by 6 in
//!   P          type a pitch (6:12), Enter          M        type a distance to move its eave
//!   G          make its edge a gable               C        give it the next condition that fits
//!   Tab        select the next plane
//!   F2         save the job                        F12      save a picture of both windows
//!
//! A value set here is entered as Assumed, with its source saying it was set in the 3D
//! view: the estimator chose it, and nobody has yet said where it comes from.
//!
//! PLUMBLINE_GRAPHICS=plain draws without the sky, ambient occlusion and bloom, for a
//! machine with no graphics card.
//!
//! * `model`: the open job, and the one reading of it every window draws.
//! * `theme`: the palette, the typeface, and what panels are made of.
//! * `scene`: the building in space.
//! * `panels`: the main window's panels, and what is written on the model.
//! * `sheet`: the sheet window.

mod model;
mod panels;
mod scene;
mod sheet;
mod theme;

use std::path::PathBuf;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{PrimaryWindow, WindowPosition, WindowRef};

use model::{Doc, Entry, Show, Snapshot, Tab, Tone, SOURCE};
use plumbline_world::value::Status;
use plumbline_world::{file, solve, units, Discipline, Footprint, Kind, Recipe, Rise, Solved, Val};
use scene::Orbit;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: plumbline-app <job file> [roof path]");
        std::process::exit(2);
    };
    let roof_arg = args.next();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("plumbline-app: cannot read {path}: {e}");
            std::process::exit(1);
        }
    };
    let (mut job, report) = match file::load(&text) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("plumbline-app: {path}: {e}");
            std::process::exit(1);
        }
    };
    // A roof named on the command line has to be a roof in the job.
    if let Some(named) = &roof_arg {
        if job.kind(named) != Some(Kind::Roof) {
            eprintln!("plumbline-app: {named} is not a roof in {path}");
            std::process::exit(1);
        }
    }
    // The roof on show: the one named, else the first solved from a footprint, else the first.
    let roof = roof_arg.or_else(|| solve::roofs(&mut job).into_iter().next()).or_else(|| job.order.iter().find_map(|e| (job.world.get::<Kind>(*e) == Some(&Kind::Roof)).then(|| job.world.get::<plumbline_world::Node>(*e).map(|n| n.path.clone())).flatten()));
    let (tone, message) = if report.stale.is_empty() { (Tone::Done, "opened".to_string()) } else { (Tone::Warned, format!("opened; figures in the file did not match its geometry for {} and were worked out again", report.stale.join(", "))) };
    let has_roof = roof.is_some();
    let doc = Doc {
        job,
        path: PathBuf::from(path),
        roof,
        selected: None,
        hover: None,
        message,
        tone,
        revision: 1,
        geometry: 1,
        entry: None,
        entry_line: None,
        origin: [0.0; 3],
        framed: false,
        pointer: None,
        shots: 0,
        edited: false,
        plan: false,
        walk: false,
        show: Show { architecture: true, roof: true, mechanical: true, electrical: true, plumbing: true, ghost: false, storey: None },
        tab: if has_roof { Tab::Roof } else { Tab::Building },
    };
    let graphics = scene::Graphics { full: std::env::var("PLUMBLINE_GRAPHICS").map(|v| v != "plain").unwrap_or(true) };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "plumbline: model".into(), resolution: (1600u32, 960u32).into(), position: WindowPosition::At(IVec2::new(0, 0)), ..default() }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .init_gizmo_group::<scene::RoofLines>()
        .init_gizmo_group::<scene::BrokenLines>()
        .init_gizmo_group::<scene::RunLines>()
        .init_gizmo_group::<scene::GridLines>()
        .init_gizmo_group::<scene::Highlight>()
        .insert_resource(ClearColor(theme::canvas()))
        .insert_resource(doc)
        .insert_resource(graphics)
        .init_resource::<Snapshot>()
        .init_resource::<scene::Drawing>()
        .init_resource::<panels::Scrolls>()
        .insert_resource(Orbit::new())
        .add_systems(Startup, (scene::setup, setup).chain())
        .add_systems(
            Update,
            (keys, scene::orbit_input, model::refresh, scene::rebuild, scene::tint, panels::rebuild, sheet::rebuild, scene::place_camera, scene::draw_lines, panels::place_labels, panels::write_pointer, panels::scroll_panels).chain(),
        )
        .run();
}

fn setup(mut commands: Commands, mut fonts: ResMut<Assets<Font>>, camera: Query<Entity, With<scene::MainCamera>>) {
    commands.insert_resource(theme::load_fonts(&mut fonts));
    // The main window's panels are drawn by the 3D camera itself, over the finished picture.
    let ui = camera.single().expect("the 3D camera is there");
    // The sheet: a second window on the same world. Its camera looks at a layer nothing
    // 3D is on, so the model's lines are not drawn over the sheet.
    let window = commands.spawn((Window { title: "plumbline: sheet".into(), resolution: (880u32, 960u32).into(), position: WindowPosition::At(IVec2::new(1610, 0)), ..default() }, sheet::SheetWindow)).id();
    let sheet = commands.spawn((Camera2d, Msaa::Off, RenderLayers::layer(1), RenderTarget::Window(WindowRef::Entity(window)))).id();
    commands.insert_resource(panels::Cameras { ui, sheet });
}

/// Change between 3D, plan and on foot. Going in on foot puts the eye at standing height
/// on the storey on show, in the middle of the building; coming out puts the view back.
pub fn set_view(doc: &mut Doc, orbit: &mut Orbit, snap: &Snapshot, plan: bool, walk: bool) {
    if doc.plan == plan && doc.walk == walk {
        return;
    }
    if walk && !doc.walk {
        let floor = doc.show.storey.as_ref().and_then(|s| snap.storeys.iter().find(|x| x.path == *s)).or(snap.storeys.first()).and_then(|s| s.elevation).unwrap_or(0.0) as f32;
        orbit.target = Vec3::new(0.0, floor + scene::EYE_FT, 0.0);
        orbit.pitch = 0.04;
        orbit.yaw = 1.9;
        doc.say(Tone::Done, "on foot: W A S D to walk, drag with the right button to look round, V to come out");
    } else if !walk && doc.walk {
        orbit.go_home();
        doc.say(Tone::Done, "back outside");
    }
    doc.plan = plan;
    doc.walk = walk;
    doc.touch();
}

/// The selected plane's roof and the edge of the footprint it rises from.
fn selected_edge(doc: &mut Doc) -> Result<(String, usize), String> {
    let sel = doc.selected.clone().ok_or("select a roof plane first (click it, or press Tab)")?;
    let e = doc.job.find(&sel).ok_or("the selection is no longer in the job")?;
    let kind = doc.job.kind(&sel);
    if !kind.is_some_and(|k| k.is_area() || k.is_run()) {
        return Err(format!("{sel} is a {}; pitch and eave are changed on a roof plane", kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default()));
    }
    let solved = doc.job.world.get::<Solved>(e).ok_or_else(|| format!("{sel} was entered by hand, not solved from a footprint; edit it in the file"))?;
    let edge = solved.edge.ok_or_else(|| format!("{sel} is a line; select the plane whose pitch or eave is to change"))?;
    Ok((solved.from.clone(), edge))
}

fn change(doc: &mut Doc, what: &str, change: impl FnOnce(&mut Doc, &str, usize) -> Result<solve::Change, String>) {
    let result = selected_edge(doc).and_then(|(roof_path, edge)| {
        let c = change(doc, &roof_path, edge)?;
        solve::edit(&mut doc.job, &roof_path, c).map(|report| (roof_path, edge, report))
    });
    match result {
        Ok((roof_path, edge, report)) => {
            // The plane on that edge keeps its path through a solve; if the edge is now a
            // gable its plane is gone and so is the selection.
            if doc.selected.as_ref().is_some_and(|s| doc.job.find(s).is_none()) {
                doc.selected = None;
            }
            let lost: Vec<String> = report.removed.iter().filter(|r| r.1).map(|r| r.0.clone()).collect();
            let mut more = if report.removed.is_empty() { String::new() } else { format!("; gone: {}", report.removed.iter().map(|r| r.0.clone()).collect::<Vec<_>>().join(", ")) };
            if !lost.is_empty() {
                more.push_str(&format!("\nLost with them: the condition or cost lines on {}", lost.join(", ")));
            }
            for moved in &report.renumbered {
                more.push_str(&format!("\n{moved}"));
            }
            // The roof under the pointer has moved; what the pointer read is no longer so.
            doc.pointer = None;
            doc.hover = None;
            doc.edited = true;
            let tone = if lost.is_empty() && report.renumbered.is_empty() { Tone::Done } else { Tone::Warned };
            doc.say(tone, format!("{what} on edge {edge} of {roof_path}; solved again: {} planes, {} lines{more}", report.planes, report.lines));
        }
        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
    }
    doc.redraw();
}

fn current_pitch(doc: &mut Doc, roof_path: &str, edge: usize) -> Result<f64, String> {
    let e = doc.job.find(roof_path).ok_or("the roof is gone")?;
    match doc.job.world.get::<Footprint>(e).and_then(|f| f.edges.get(edge.wrapping_sub(1))).and_then(|v| v.value) {
        Some(Rise::Pitch(p)) => Ok(p),
        Some(Rise::Gable) => Err("that edge is a gable".into()),
        None => Err("that edge has no pitch".into()),
    }
}

fn pitch_val(p: f64) -> Val<Rise> {
    Val { value: Some(Rise::Pitch(p)), unit: units::PITCH.into(), status: Status::Assumed, source: SOURCE.into() }
}

#[allow(clippy::too_many_arguments)]
fn keys(mut commands: Commands, mut doc: ResMut<Doc>, mut orbit: ResMut<Orbit>, snap: Res<Snapshot>, mut typed: MessageReader<KeyboardInput>, keys: Res<ButtonInput<KeyCode>>, primary: Query<Entity, With<PrimaryWindow>>, sheet: Query<Entity, With<sheet::SheetWindow>>) {
    let doc = &mut *doc;
    // A picture of each window, numbered, into the folder named by PLUMBLINE_SHOTS. It
    // works while a value is being typed, so the typing can be shown.
    if keys.just_pressed(KeyCode::F12) {
        let dir = std::env::var("PLUMBLINE_SHOTS").unwrap_or_else(|_| ".".into());
        doc.shots += 1;
        let n = doc.shots;
        if let Ok(window) = primary.single() {
            commands.spawn(Screenshot::window(window)).observe(save_to_disk(format!("{dir}/{n:02}-model.png")));
        }
        if let Ok(window) = sheet.single() {
            commands.spawn(Screenshot::window(window)).observe(save_to_disk(format!("{dir}/{n:02}-sheet.png")));
        }
    }
    // A value being typed takes the keyboard until Enter or Escape.
    if let Some((field, mut text)) = doc.entry.take() {
        let before = text.clone();
        let mut done = None;
        for event in typed.read() {
            if !event.state.is_pressed() {
                continue;
            }
            match &event.logical_key {
                Key::Enter => done = Some(true),
                Key::Escape => done = Some(false),
                Key::Backspace => {
                    text.pop();
                }
                Key::Space => text.push(' '),
                Key::Character(c) => text.push_str(c),
                _ => {}
            }
        }
        match done {
            None => {
                if text != before {
                    doc.touch();
                }
                doc.entry = Some((field, text));
            }
            Some(false) => {
                doc.say(Tone::Done, "entry cancelled");
                doc.touch();
            }
            Some(true) => match field {
                // A price typed here is entered as Assumed, like everything set in this
                // window: there is nowhere in it yet to say where a figure came from.
                Entry::Allowance | Entry::UnitCost => {
                    let what = if field == Entry::Allowance { "allowance" } else { "unit_cost" };
                    let line = if field == Entry::UnitCost { doc.entry_line.take() } else { None };
                    match doc.selected.clone() {
                        None => doc.say(Tone::Refused, "refused: select something first"),
                        Some(sel) => match doc.job.set_on_line(&sel, what, line.as_deref(), Some(text.trim()), Status::Assumed, SOURCE) {
                            Ok(()) => {
                                doc.job.derive();
                                doc.edited = true;
                                let of = line.map(|l| format!(" of {l:?}")).unwrap_or_default();
                                doc.say(Tone::Done, format!("{sel}: {}{of} set to {}", what.replace('_', " "), text.trim()));
                            }
                            Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
                        },
                    }
                    doc.redraw();
                }
                Entry::Pitch => change(doc, &format!("pitch {text}"), |_, _, edge| Ok(solve::Change::Rise { edge, rise: pitch_val(units::parse_pitch(&text)?) })),
                Entry::Move => change(doc, &format!("eave moved {text}"), |_, _, edge| {
                    let (len, unit) = units::parse_length(&text)?;
                    Ok(solve::Change::MoveEdge { edge, by_ft: len.feet(unit)?, status: Status::Assumed, source: SOURCE.into() })
                }),
            },
        }
        return;
    }
    typed.clear();

    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::Minus) {
        let step = if keys.just_pressed(KeyCode::Equal) { 1.0 } else { -1.0 };
        change(doc, "pitch changed", |doc, roof_path, edge| Ok(solve::Change::Rise { edge, rise: pitch_val(current_pitch(doc, roof_path, edge)? + step) }));
    }
    if keys.just_pressed(KeyCode::BracketRight) || keys.just_pressed(KeyCode::BracketLeft) {
        let by_ft = if keys.just_pressed(KeyCode::BracketRight) { 0.5 } else { -0.5 };
        change(doc, if by_ft > 0.0 { "eave moved out 6 in" } else { "eave moved in 6 in" }, |_, _, edge| Ok(solve::Change::MoveEdge { edge, by_ft, status: Status::Assumed, source: SOURCE.into() }));
    }
    if keys.just_pressed(KeyCode::KeyG) {
        change(doc, "made a gable", |_, _, edge| Ok(solve::Change::Rise { edge, rise: Val { value: Some(Rise::Gable), unit: String::new(), status: Status::Assumed, source: SOURCE.into() } }));
    }
    for (key, field) in [(KeyCode::KeyP, Entry::Pitch), (KeyCode::KeyM, Entry::Move)] {
        if keys.just_pressed(key) {
            // Say why not before a field opens that could only be refused.
            match selected_edge(doc) {
                Ok(_) => doc.entry = Some((field, String::new())),
                Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
            }
            doc.touch();
        }
    }

    // A price on what is selected: B for a budget rate (its allowance), U for the unit
    // cost of its first cost line that has no price yet. Each says why not before a field
    // opens that could only be refused.
    for (key, field) in [(KeyCode::KeyB, Entry::Allowance), (KeyCode::KeyU, Entry::UnitCost)] {
        if keys.just_pressed(key) {
            let ready: Result<Option<String>, String> = match doc.selected.clone() {
                None => Err("select something first".into()),
                Some(sel) => {
                    let kind = doc.job.kind(&sel);
                    let lines: Vec<(String, bool)> = doc.job.find(&sel).and_then(|e| doc.job.world.get::<plumbline_world::Cost>(e)).map(|c| c.lines.iter().map(|l| (l.description.clone(), l.unit_cost.value.is_some())).collect()).unwrap_or_default();
                    match (field, kind.and_then(|k| k.priced_by())) {
                        (Entry::Allowance, None) => Err(format!("a {} has no area or length an allowance could be a rate on", kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default())),
                        (Entry::Allowance, Some(_)) => Ok(None),
                        _ if lines.is_empty() => Err(format!("{sel} has no cost line to price; give it a condition first (C)")),
                        _ => match lines.iter().find(|l| !l.1) {
                            Some((line, _)) => Ok(Some(line.clone())),
                            None => Err(format!("every cost line of {sel} has a price; to change one, `plumbline set <job> {sel} unit_cost <price> --line \"<line>\"`")),
                        },
                    }
                }
            };
            match ready {
                Ok(line) => {
                    doc.entry_line = line;
                    doc.entry = Some((field, String::new()));
                }
                Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
            }
            doc.touch();
        }
    }

    // What is on show.
    for (key, discipline) in [(KeyCode::Digit1, Some(Discipline::Architecture)), (KeyCode::Digit2, None), (KeyCode::Digit3, Some(Discipline::Mechanical)), (KeyCode::Digit4, Some(Discipline::Electrical)), (KeyCode::Digit5, Some(Discipline::Plumbing))] {
        if keys.just_pressed(key) {
            match discipline {
                Some(d) => doc.show.toggle(d),
                None => doc.show.roof = !doc.show.roof,
            }
            doc.redraw();
        }
    }
    if keys.just_pressed(KeyCode::KeyX) {
        doc.show.ghost = !doc.show.ghost;
        doc.redraw();
    }
    if keys.just_pressed(KeyCode::PageUp) || keys.just_pressed(KeyCode::PageDown) {
        // Whole building, then each storey from the bottom up, and round again.
        let mut stops: Vec<Option<String>> = vec![None];
        stops.extend(snap.storeys.iter().map(|s| Some(s.path.clone())));
        let at = stops.iter().position(|s| *s == doc.show.storey).unwrap_or(0);
        let next = if keys.just_pressed(KeyCode::PageUp) { (at + 1) % stops.len() } else { (at + stops.len() - 1) % stops.len() };
        doc.show.storey = stops[next].clone();
        doc.redraw();
    }
    if keys.just_pressed(KeyCode::KeyT) {
        let plan = !doc.plan;
        set_view(doc, &mut orbit, &snap, plan, false);
    }
    if keys.just_pressed(KeyCode::KeyV) {
        let walk = !doc.walk;
        set_view(doc, &mut orbit, &snap, false, walk);
    }
    if keys.just_pressed(KeyCode::KeyF) && !doc.walk {
        orbit.go_home();
    }
    if keys.just_pressed(KeyCode::Escape) && doc.selected.is_some() {
        doc.selected = None;
        doc.say(Tone::Done, "nothing selected");
        doc.touch();
    }
    if keys.just_pressed(KeyCode::Tab) {
        // The next plane of the roof on show, in job order.
        let planes: Vec<String> = snap.planes.iter().map(|p| p.path.clone()).collect();
        if planes.is_empty() {
            doc.say(Tone::Refused, "refused: there is no roof plane to select");
        } else {
            let at = doc.selected.as_ref().and_then(|s| planes.iter().position(|p| p == s)).map(|i| (i + 1) % planes.len()).unwrap_or(0);
            doc.selected = Some(planes[at].clone());
            doc.say(Tone::Done, format!("selected {}", planes[at]));
        }
        doc.touch();
    }
    if keys.just_pressed(KeyCode::KeyC) {
        // Give the selected element the next condition whose recipe fits its kind.
        match doc.selected.clone() {
            None => doc.say(Tone::Refused, "refused: select something first"),
            Some(sel) => {
                let kind = doc.job.kind(&sel);
                let mut recipes: Vec<String> = Vec::new();
                let mut q = doc.job.world.query::<(&plumbline_world::Node, &Recipe)>();
                for (node, recipe) in q.iter(&doc.job.world) {
                    if kind.is_some_and(|k| recipe.applies_to.contains(&k)) {
                        recipes.push(node.name.clone());
                    }
                }
                match recipes.first() {
                    None => doc.say(Tone::Refused, format!("refused: the job has no condition that goes on a {}", kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default())),
                    Some(name) => match doc.job.apply_as(&sel, name, Status::Assumed, SOURCE) {
                        Ok(lines) => {
                            doc.job.derive();
                            doc.edited = true;
                            doc.say(Tone::Done, format!("{sel} is now {name:?}: {} cost lines", lines.len()));
                        }
                        Err(why) => doc.say(Tone::Refused, format!("refused: {why}")),
                    },
                }
            }
        }
        doc.redraw();
    }
    if keys.just_pressed(KeyCode::F2) {
        match std::fs::write(&doc.path, file::save(&mut doc.job)) {
            Ok(()) => {
                doc.edited = false;
                doc.say(Tone::Done, format!("saved {}", doc.path.display()));
            }
            Err(e) => doc.say(Tone::Refused, format!("could not save {}: {e}", doc.path.display())),
        }
        doc.touch();
    }
}
