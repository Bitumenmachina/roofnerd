//! The 3D window and the sheet window.
//!
//! Both are views over one job world (`plumbline-world`). Nothing here works out a
//! quantity: the windows draw what the world holds, send the estimator's edits to it,
//! and draw again. The sheet is a second operating-system window on the same world, so
//! it can sit on another monitor and it follows every edit as it is made.
//!
//!   plumbline-app <job file> [roof path]
//!
//! Mouse: click a plane to select it, click the ground to let go; drag with the right
//! button to orbit, with the middle button to pan; scroll to zoom. Click a row of the
//! sheet to select it in the 3D view.
//! Keys, on the selected plane of a roof solved from a footprint:
//!   = / -      pitch up or down by 1 in 12        ] / [    move its eave out or in by 6 in
//!   P          type a pitch (6:12), Enter          M        type a distance to move its eave
//!   G          make its edge a gable               C        give it the next condition that fits
//!   Tab        select the next plane               Esc      let go of the selection
//!   T          plan view or 3D view                F        frame the roof
//!   F2         save the job                        F12      save a picture of both windows
//!
//! A value set here is entered as Assumed, with its source saying it was set in the 3D
//! view: the estimator chose it, and nobody has yet said where it comes from.
//!
//! * `model`: the open job, and the one reading of it every window draws.
//! * `theme`: the palette, the typeface, and what panels are made of.
//! * `scene`: the roof in space.
//! * `panels`: the main window around the 3D view, and what is written over the roof.
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

use model::{Doc, Entry, Snapshot, SOURCE};
use plumbline_world::value::Status;
use plumbline_world::{file, solve, units, Footprint, Kind, Recipe, Rise, Solved, Val};

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
    // The roof on show: the one named, else the first solved from a footprint, else the first.
    let roof = roof_arg.or_else(|| solve::roofs(&mut job).into_iter().next()).or_else(|| job.order.iter().find_map(|e| (job.world.get::<Kind>(*e) == Some(&Kind::Roof)).then(|| job.world.get::<plumbline_world::Node>(*e).map(|n| n.path.clone())).flatten()));
    let message = if report.stale.is_empty() { "opened".to_string() } else { format!("opened; figures in the file did not match its geometry for {} and were worked out again", report.stale.join(", ")) };
    let doc = Doc { job, path: PathBuf::from(path), roof, selected: None, hover: None, message, revision: 1, entry: None, origin: [0.0; 3], framed: false, pointer: None, shots: 0, edited: false, plan: false };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "plumbline: roof".into(), resolution: (1440u32, 900u32).into(), position: WindowPosition::At(IVec2::new(0, 0)), ..default() }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .init_gizmo_group::<scene::RoofLines>()
        .init_gizmo_group::<scene::GridLines>()
        .init_gizmo_group::<scene::Highlight>()
        .insert_resource(ClearColor(theme::canvas()))
        .insert_resource(doc)
        .init_resource::<Snapshot>()
        .init_resource::<scene::ViewRect>()
        .init_resource::<scene::Drawing>()
        .init_resource::<panels::Scrolls>()
        .insert_resource(scene::Orbit { target: Vec3::ZERO, yaw: HOME_YAW, pitch: HOME_PITCH, distance: 120.0, home: (Vec3::ZERO, 120.0) })
        .add_systems(Startup, (setup, scene::setup))
        .add_systems(
            Update,
            (keys, scene::orbit_input, scene::fit_viewport, model::refresh, scene::rebuild, panels::rebuild, sheet::rebuild, scene::tint_hover, scene::place_camera, scene::draw_lines, panels::place_labels, panels::write_pointer, panels::scroll_panels).chain(),
        )
        .run();
}

const HOME_YAW: f32 = -0.62;
const HOME_PITCH: f32 = 0.52;

fn setup(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    commands.insert_resource(theme::load_fonts(&mut fonts));
    let ui = panels::ui_camera(&mut commands);
    // The sheet: a second window on the same world. Its camera looks at a layer nothing
    // 3D is on, so the roof's lines are not drawn over the sheet.
    let window = commands.spawn((Window { title: "plumbline: sheet".into(), resolution: (860u32, 900u32).into(), position: WindowPosition::At(IVec2::new(1450, 0)), ..default() }, sheet::SheetWindow)).id();
    let sheet = commands.spawn((Camera2d, RenderLayers::layer(1), RenderTarget::Window(WindowRef::Entity(window)))).id();
    commands.insert_resource(panels::Cameras { ui, sheet });
}

/// The selected plane's roof and the edge of the footprint it rises from.
fn selected_edge(doc: &mut Doc) -> Result<(String, usize), String> {
    let sel = doc.selected.clone().ok_or("select a plane first (click it, or press Tab)")?;
    let e = doc.job.find(&sel).ok_or("the selection is no longer in the job")?;
    let solved = doc.job.world.get::<Solved>(e).ok_or_else(|| format!("{sel} was entered by hand, not solved from a footprint; edit it in the file"))?;
    let edge = solved.edge.ok_or_else(|| format!("{sel} is a line; select the plane whose pitch or eave is to change"))?;
    Ok((solved.from.clone(), edge))
}

fn change(doc: &mut Doc, what: &str, change: impl FnOnce(&mut Doc, &str, usize) -> Result<solve::Change, String>) {
    let result = selected_edge(doc).and_then(|(roof_path, edge)| {
        let c = change(doc, &roof_path, edge)?;
        solve::edit(&mut doc.job, &roof_path, c).map(|report| (roof_path, edge, report))
    });
    doc.message = match result {
        Ok((roof_path, edge, report)) => {
            // The plane on that edge keeps its path through a solve; if the edge is now a
            // gable its plane is gone and so is the selection.
            if doc.selected.as_ref().is_some_and(|s| doc.job.find(s).is_none()) {
                doc.selected = None;
            }
            let lost: Vec<String> = report.removed.iter().filter(|r| r.1).map(|r| r.0.clone()).collect();
            let mut gone = if report.removed.is_empty() { String::new() } else { format!("; gone: {}", report.removed.iter().map(|r| r.0.clone()).collect::<Vec<_>>().join(", ")) };
            if !lost.is_empty() {
                gone.push_str(&format!("\nLost with them: the condition or cost lines on {}", lost.join(", ")));
            }
            for moved in &report.renumbered {
                gone.push_str(&format!("\n{moved}"));
            }
            // The roof under the pointer has moved; what the pointer read is no longer so.
            doc.pointer = None;
            doc.hover = None;
            doc.edited = true;
            format!("{what} on edge {edge} of {roof_path}; solved again: {} planes, {} lines{gone}", report.planes, report.lines)
        }
        Err(why) => format!("refused: {why}"),
    };
    doc.touch();
}

fn current_pitch(doc: &mut Doc, roof_path: &str, edge: usize) -> Result<f64, String> {
    let e = doc.job.find(roof_path).ok_or("the roof is gone")?;
    match doc.job.world.get::<Footprint>(e).and_then(|f| f.edges.get(edge - 1)).and_then(|v| v.value) {
        Some(Rise::Pitch(p)) => Ok(p),
        Some(Rise::Gable) => Err("that edge is a gable".into()),
        None => Err("that edge has no pitch".into()),
    }
}

fn pitch_val(p: f64) -> Val<Rise> {
    Val { value: Some(Rise::Pitch(p)), unit: units::PITCH.into(), status: Status::Assumed, source: SOURCE.into() }
}

#[allow(clippy::too_many_arguments)]
fn keys(mut commands: Commands, mut doc: ResMut<Doc>, mut orbit: ResMut<scene::Orbit>, mut typed: MessageReader<KeyboardInput>, keys: Res<ButtonInput<KeyCode>>, primary: Query<Entity, With<PrimaryWindow>>, sheet: Query<Entity, With<sheet::SheetWindow>>) {
    let doc = &mut *doc;
    // A picture of each window, numbered, into the folder named by PLUMBLINE_SHOTS. It
    // works while a value is being typed, so the typing can be shown.
    if keys.just_pressed(KeyCode::F12) {
        let dir = std::env::var("PLUMBLINE_SHOTS").unwrap_or_else(|_| ".".into());
        doc.shots += 1;
        let n = doc.shots;
        if let Ok(window) = primary.single() {
            commands.spawn(Screenshot::window(window)).observe(save_to_disk(format!("{dir}/{n:02}-roof.png")));
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
                doc.message = "entry cancelled".into();
                doc.touch();
            }
            Some(true) => match field {
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
                Err(why) => doc.message = format!("refused: {why}"),
            }
            doc.touch();
        }
    }
    if keys.just_pressed(KeyCode::KeyT) {
        doc.plan = !doc.plan;
        doc.touch();
    }
    if keys.just_pressed(KeyCode::KeyF) {
        orbit.target = orbit.home.0;
        orbit.distance = orbit.home.1;
        orbit.yaw = HOME_YAW;
        orbit.pitch = HOME_PITCH;
    }
    if keys.just_pressed(KeyCode::Escape) && doc.selected.is_some() {
        doc.selected = None;
        doc.message = "nothing selected".into();
        doc.touch();
    }
    if keys.just_pressed(KeyCode::Tab) {
        // The next plane of the roof on show, in job order.
        if let Some(roof_path) = doc.roof.clone() {
            let under: Vec<String> = doc.job.children().get(&roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
            let in_order: Vec<String> = doc.job.order.iter().filter_map(|e| doc.job.world.get::<plumbline_world::Node>(*e).map(|n| n.path.clone())).filter(|p| under.contains(p)).collect();
            let planes: Vec<String> = in_order.into_iter().filter(|p| doc.job.kind(p).is_some_and(|k| k.is_area())).collect();
            if !planes.is_empty() {
                let at = doc.selected.as_ref().and_then(|s| planes.iter().position(|p| p == s)).map(|i| (i + 1) % planes.len()).unwrap_or(0);
                doc.selected = Some(planes[at].clone());
                doc.message = format!("selected {}", planes[at]);
                doc.touch();
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyC) {
        // Give the selected element the next condition whose recipe fits its kind.
        doc.message = match doc.selected.clone() {
            None => "refused: select a plane first".into(),
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
                    None => format!("refused: the job has no condition that goes on a {}", kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default()),
                    Some(name) => match doc.job.apply_as(&sel, name, Status::Assumed, SOURCE) {
                        Ok(lines) => {
                            doc.job.derive();
                            doc.edited = true;
                            format!("{sel} is now {name:?}: {} cost lines", lines.len())
                        }
                        Err(why) => format!("refused: {why}"),
                    },
                }
            }
        };
        doc.touch();
    }
    if keys.just_pressed(KeyCode::F2) {
        doc.message = match std::fs::write(&doc.path, file::save(&mut doc.job)) {
            Ok(()) => {
                doc.edited = false;
                format!("saved {}", doc.path.display())
            }
            Err(e) => format!("could not save: {e}"),
        };
        doc.touch();
    }
}
