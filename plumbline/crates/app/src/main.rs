//! The 3D window and the sheet window.
//!
//! Both are views over one job world (`plumbline-world`). Nothing here works out a
//! quantity: the windows draw what the world holds, send the estimator's edits to it,
//! and draw again. The sheet is a second operating-system window on the same world, so
//! it can sit on another monitor and it follows every edit as it is made.
//!
//!   plumbline-app [job file] [roof path]
//!
//! A job file that is not there yet is a new job, written when it is first saved; with
//! no file named, the job is `job.json` in the folder the program was started in.
//!
//! The whole building is drawn: walls, floors, doors and windows, the roof, and the ducts,
//! pipes, conduit, cable, equipment and fixtures in it. Click anything to select it; click
//! the ground to let go. Click a row of the sheet to select it in the 3D view.
//!
//! The screen is laid out the way a building game's is (DECISIONS.md, call 55): the
//! price and Save, Undo, Redo along the top; levels, drawings and what is on show down
//! the left; the build bar along the bottom, with the fields of the tool in hand over
//! it; what is selected, as fields and buttons, down the right. Every button says the
//! key that does the same.
//!
//! Moving: drag with the right button to turn, with the middle button to pan; scroll to
//! zoom; W A S D move over the ground, Q and E go down and up.
//!   1 to 9, 0  take a build tool in hand            Esc      let go
//!   T          plan view or 3D                     V        go in on foot, or come out
//!   F          frame the building                  X        see through the building
//!   Ctrl+S, F2 save      Ctrl+Z, Z undo      Ctrl+Y, Y redo      Delete  delete
//!   F12        save a picture of both windows      PgUp/PgDn  one level at a time
//!
//! A value set here is entered as Assumed, with its source saying it was set in the
//! window: the estimator chose it, and nobody has yet said where it comes from.
//!
//! PLUMBLINE_GRAPHICS=plain draws without ambient occlusion, for a machine with no
//! graphics card. PLUMBLINE_MAP=<file> writes where each button is on screen, for a
//! script that drives the real pointer (`map`).
//!
//! * `model`: the open job, and the one reading of it every window draws.
//! * `theme`: the palette, the typeface, and what panels are made of.
//! * `scene`: the building in space.
//! * `panels`: the main window's panels, and what is written on the model.
//! * `plans`: the drawings laid under the storeys, read off their files.
//! * `sheet`: the sheet window.
//! * `map`: where each button is, written out for a script.

mod bring;
mod dress;
mod map;
mod model;
mod panels;
mod plans;
mod glass;
mod scene;
mod sheet;
mod skylight;
mod theme;
mod tools;

use std::path::PathBuf;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{PrimaryWindow, WindowPosition, WindowRef};

use model::{Doc, Entry, Show, Snapshot, Tab, Tone, SOURCE};
use plumbline_world::value::Status;
use plumbline_world::{file, solve, units, Footprint, Kind, Recipe, Rise, Solved, Val};
use scene::Orbit;

fn main() {
    let mut args = std::env::args().skip(1);
    // With no job named, the job is `job.json` in the folder the program was started in.
    let path = args.next().unwrap_or_else(|| "job.json".to_string());
    let roof_arg = args.next();
    // A job file that is not there yet is a new job: it is written when it is first saved.
    let fresh = !std::path::Path::new(&path).exists();
    let (mut job, report) = if fresh {
        let title = std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("job").replace(['-', '_'], " ");
        (plumbline_world::Job::new(&title, "plumbline", &plumbline_world::job::now()[..10]), file::LoadReport::default())
    } else {
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("plumbline-app: cannot read {path}: {e}");
                std::process::exit(1);
            }
        };
        match file::load(&text) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("plumbline-app: {path}: {e}");
                std::process::exit(1);
            }
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
    let (tone, message) = if fresh {
        (Tone::Done, format!("A new job. Next steps, on the left, says where to start. Save keeps it as {}.", std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("this job")))
    } else if report.stale.is_empty() {
        (Tone::Done, "Opened.".to_string())
    } else { (Tone::Warned, format!("Opened. Figures in the file did not match its geometry for {} and were worked out again.", report.stale.join(", "))) };
    let has_roof = roof.is_some();
    let has_lines = job.order.iter().any(|e| job.world.get::<plumbline_world::Cost>(*e).is_some_and(|c| !c.lines.is_empty()));
    // The job as it was opened: what the first step undone goes back to.
    let draw = tools::Draw { last_text: file::save(&mut job), ..default() };
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
        form: Vec::new(),
        form_at: 0,
        pressed: Vec::new(),
        go: None,
        steps_open: true,
        look_at: None,
        dropped: None,
        choosing: false,
        give: None,
        placing: None,
        lost: None,
        browse: None,
        fresh: false,
        entry_line: None,
        measuring: None,
        origin: [0.0; 3],
        framed: false,
        pointer: None,
        shots: 0,
        edited: fresh,
        plan: false,
        plan_level: None,
        walk: false,
        show: Show { architecture: true, roof: true, mechanical: true, electrical: true, plumbing: true, ghost: false, storey: None, certainty: false, clashes: false, ground: true, framing: false, finishes: true, off_levels: Default::default(), off_systems: Default::default(), away: [false; 3] },
        // The sheet opens on the estimate once anything is priced; before that, on what
        // has been taken off.
        tab: if has_lines { Tab::Estimate } else if has_roof { Tab::Roof } else { Tab::Building },
        draw,
    };
    let graphics = scene::Graphics { full: std::env::var("PLUMBLINE_GRAPHICS").map(|v| v != "plain").unwrap_or(true) };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "plumbline: model".into(), resolution: (1600u32, 960u32).into(), position: WindowPosition::At(IVec2::new(0, 0)), ..default() }),
            // Closing is ours to answer: work not saved is never dropped without a word.
            close_when_requested: false,
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
        .init_resource::<dress::Patterns>()
        .init_resource::<plans::Plans>()
        .init_resource::<panels::Scrolls>()
        .insert_resource(Orbit::new())
        .init_resource::<scene::Feel>()
        .init_resource::<scene::Carry>()
        .add_plugins(glass::GlassPlugin)
        .add_observer(scene::pressed)
        .add_observer(scene::dragging)
        .add_systems(Startup, (scene::setup, glass::setup, setup).chain())
        .add_systems(Last, closing)
        .add_systems(
            Update,
            ((tools::frame_begins, keys).chain(), plans::dropped, plans::read, tools::clicks, scene::carry, tools::record, scene::orbit_input, model::refresh, scene::rebuild, scene::tint, panels::rebuild, sheet::rebuild, (scene::place_camera, scene::ground_tone, glass::follow).chain(), tools::pointer, scene::draw_lines, tools::preview, panels::place_labels, panels::write_pointer, panels::scroll_panels, map::write).chain(),
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
        doc.say(Tone::Done, "On foot. W A S D to walk, drag with the right button to look round, V to come out.");
    } else if !walk && doc.walk {
        orbit.go_home();
        doc.say(Tone::Done, "Back outside.");
    }
    // A plan is of one level. Asked for with every level on show, on a job with a
    // roof or more than one level, it put the roof, solid, over everything: a finished
    // house on plan was a dark slab with no floor plan under it (J25B-3, the finding
    // that failed the look a third time). The plan puts one level on show: the lowest
    // with a door or a window in it, else the lowest. Leaving the plan puts every
    // level back, unless another was chosen meanwhile.
    if plan && !doc.plan && doc.show.storey.is_none() && (snap.storeys.len() > 1 || (!snap.storeys.is_empty() && !snap.planes.is_empty())) {
        let by_height = |a: &&plumbline_world::building::StoreyTotals, b: &&plumbline_world::building::StoreyTotals| a.elevation.unwrap_or(f64::MAX).total_cmp(&b.elevation.unwrap_or(f64::MAX));
        let level = snap.storeys.iter().filter(|s| s.doors + s.windows > 0).min_by(by_height).or_else(|| snap.storeys.iter().min_by(by_height));
        if let Some(level) = level {
            doc.show.storey = Some(level.path.clone());
            doc.plan_level = Some(level.path.clone());
        }
    } else if !plan && doc.plan {
        if doc.plan_level.is_some() && doc.show.storey == doc.plan_level {
            doc.show.storey = None;
        }
        doc.plan_level = None;
    }
    doc.plan = plan;
    doc.walk = walk;
    // A drawing under the plan lies over the floor on plan and under it in 3D.
    doc.redraw();
}

/// What the window says on going to the plan or back to 3D (VISUAL.md A9). Where the
/// plan put one level on show by itself, it says which and how to see the others.
pub fn view_said(doc: &Doc, snap: &Snapshot, plan: bool, key: bool) -> String {
    let level = doc.plan_level.as_ref().filter(|_| plan).and_then(|l| snap.storeys.iter().find(|s| s.path == *l));
    match (plan, level) {
        (true, Some(level)) => format!("On plan: {}, north is up. A plan is of one level: Page Up and Page Down go to the others, and All levels shows the roof from above.{}", panels::pretty(&level.name), if key { " T again for 3D." } else { "" }),
        (true, None) => format!("On plan: north is up.{}", if key { " T again for 3D." } else { "" }),
        (false, _) => format!("In 3D.{}", if key { " T for the plan." } else { "" }),
    }
}

/// The selected plane's roof and the edge of the footprint it rises from.
fn selected_edge(doc: &mut Doc) -> Result<(String, usize), String> {
    let sel = doc.selected.clone().ok_or("no roof plane is selected. Click one, or press Tab.")?;
    let e = doc.job.find(&sel).ok_or("the selection is no longer in the job")?;
    let kind = doc.job.kind(&sel);
    if !kind.is_some_and(|k| k.is_area() || k.is_run()) {
        return Err(format!("{sel} is a {}; pitch and eave are changed on a roof plane", kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default()));
    }
    let solved = doc.job.world.get::<Solved>(e).ok_or_else(|| format!("{sel} was drawn by its own outline, not worked out from a roof's footprint: its pitch and its eave are not changed in this window yet"))?;
    let edge = solved.edge.ok_or_else(|| format!("{sel} is a line; select the plane whose pitch or eave is to change"))?;
    Ok((solved.from.clone(), edge))
}

fn change(doc: &mut Doc, what: &str, change: impl FnOnce(&mut Doc, &str, usize) -> Result<solve::Change, String>) {
    // What is priced before the change, so that what a plane takes with it when it
    // goes can be said to the cent (J23-3).
    let priced_before = plumbline_world::view::estimate(&mut doc.job);
    // What everything is called before the change: what goes with it is said by its
    // name ("plane 2", "hip 1-2"), not by the name it was kept under (J24A-16: the
    // message read "gone: plane-0028, hip-0037").
    let called: std::collections::HashMap<String, String> = {
        let mut q = doc.job.world.query::<&plumbline_world::Node>();
        q.iter(&doc.job.world).map(|n| (n.path.clone(), if n.name.is_empty() { n.path.clone() } else { n.name.clone() })).collect()
    };
    let name = |path: &String| called.get(path).cloned().unwrap_or_else(|| path.clone());
    // And how sure the roof's own values were: a change made here is Assumed, and one
    // that takes the place of a measurement has to say so (J24A-15: Steeper replaced a
    // pitch measured on site and the window said only "Pitch changed").
    let sure_before = selected_edge(doc).ok().and_then(|(roof_path, edge)| {
        let e = doc.job.find(&roof_path)?;
        let f = doc.job.world.get::<Footprint>(e)?;
        Some((f.edges.get(edge.wrapping_sub(1)).cloned(), f.points.status, f.points.source.clone()))
    });
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
            let lost: Vec<String> = report.removed.iter().filter(|r| r.1).map(|r| name(&r.0)).collect();
            let mut more = if report.removed.is_empty() { String::new() } else { format!("; gone: {}", report.removed.iter().map(|r| name(&r.0)).collect::<Vec<_>>().join(", ")) };
            // A measurement this change took the place of.
            let mut unmeasured = false;
            if let (Some((edge_was, points_was, points_from)), Some(f)) = (sure_before, doc.job.find(&roof_path).and_then(|e| doc.job.world.get::<Footprint>(e)).cloned()) {
                let rise_said = |v: &Val<Rise>| match v.value {
                    Some(Rise::Pitch(p)) => plumbline_world::view::pitch_said(p),
                    Some(Rise::Gable) => "a gable".to_string(),
                    None => "not known".to_string(),
                };
                if let (Some(was), Some(is)) = (edge_was, f.edges.get(edge.wrapping_sub(1))) {
                    if was.status == Status::Known && is.status != Status::Known {
                        more.push_str(&format!("\nEdge {edge} was {}, Known ({}). It is now {}, Assumed: what was measured is no longer what is drawn. Measure it again, or undo.", rise_said(&was), was.source, rise_said(is)));
                        unmeasured = true;
                    }
                }
                if points_was == Status::Known && f.points.status != Status::Known {
                    more.push_str(&format!("\nThe roof's footprint was Known ({points_from}). With an edge moved here it is Assumed: what was measured is no longer what is drawn. Measure it again, or undo."));
                    unmeasured = true;
                }
            }
            if !lost.is_empty() {
                more.push_str(&format!("\nLost with them: the assembly or cost lines on {}", lost.join(", ")));
                if let Some(went) = plumbline_world::view::went_with_said(&priced_before, &mut doc.job) {
                    more.push_str(&format!("\n{}{}", went[..1].to_uppercase(), &went[1..]));
                }
                // The planes that stay change size when one goes, and what is priced on
                // them changes with it: a hip made a gable drops a plane's dollars and
                // the two beside it take nearly all of them back. What went is said
                // above; here is what the job cost was and is, so the first is not
                // read as the whole of it.
                if let Some(said) = plumbline_world::view::job_cost_was_and_is_said(&priced_before, &mut doc.job) {
                    more.push_str(&format!("\n{said}"));
                }
            } else if let Some(said) = plumbline_world::view::job_cost_moved_said(&priced_before, &mut doc.job) {
                // Nothing went, and the dollars moved all the same: Steeper took the
                // price up $6.10 with not a word about it (J25A-13).
                more.push_str(&format!("\n{said}"));
            }
            for moved in &report.renumbered {
                more.push_str(&format!("\n{moved}"));
            }
            // The roof under the pointer has moved; what the pointer read is no longer so.
            doc.pointer = None;
            doc.hover = None;
            doc.edit(what);
            let tone = if lost.is_empty() && report.renumbered.is_empty() && !unmeasured { Tone::Done } else { Tone::Warned };
            // "9 lines" called a roof's eaves and hips what the estimate calls a cost line
            // (J22A-16, call 91).
            doc.say(tone, format!("{what} on edge {edge} of {roof_path}. The roof is now {}; its eaves, hips, valleys and ridges follow{more}", plumbline_world::view::so_many(report.planes, "plane")));
        }
        Err(why) => tools::cant(doc, "change the roof", why),
    }
    doc.redraw();
}

/// A window on its way shut: it goes the frame after it is marked, as the engine's own
/// closing does.
#[derive(Component)]
struct Shutting;

/// A window asked to close. The two windows are one program and one job: closing
/// either closes both. With work not saved, the asking is answered with a warning and
/// both stay; asked again with nothing changed since, they close. A warning is for the
/// work as it stood: an edit made after it is warned of afresh. (Found beside J19-46:
/// closing with work not saved lost it without a word. R21-1: one warning then served
/// for everything done after it, and "close again to leave" closed one window, left
/// the other, and the estimate could not be opened again.)
fn closing(mut commands: Commands, mut asked: MessageReader<bevy::window::WindowCloseRequested>, shutting: Query<Entity, With<Shutting>>, windows: Query<Entity, With<Window>>, mut doc: ResMut<Doc>, mut warned: Local<Option<u64>>) {
    for window in &shutting {
        commands.entity(window).despawn();
    }
    if !doc.edited {
        *warned = None;
    }
    if asked.read().count() == 0 {
        return;
    }
    if doc.edited && *warned != Some(doc.draw.edits) {
        *warned = Some(doc.draw.edits);
        doc.say(Tone::Refused, "Not closed: the job has changes that are not saved. Save (Ctrl+S), or close again to leave without saving them.");
        doc.touch();
    } else {
        for window in &windows {
            commands.entity(window).try_insert(Shutting);
        }
    }
}

/// Save the job to its file.
fn save(doc: &mut Doc) {
    match std::fs::write(&doc.path, file::save(&mut doc.job)) {
        Ok(()) => {
            doc.edited = false;
            // By its name: where it lies on the computer is not a builder's word (J19-47).
            // "Saved." and no more: the file's own name is not a builder's word and the
            // job's name is at the top of the window (VISUAL.md A8; J22A-1).
            doc.say(Tone::Done, "Saved.");
        }
        Err(e) => tools::cant(doc, "save", format!("{}: {e}", doc.path.display())),
    }
    doc.touch();
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

/// What is known of the keys coming in fast, as a laser in keyboard mode sends a reading:
/// a whole run of characters in less time than a hand types two.
#[derive(Default)]
struct Burst {
    /// When a character key last came in, in seconds.
    last: f64,
    /// Characters are coming in as a burst until then.
    until: f64,
    /// A character key that came in alone and has not acted yet: it waits to see
    /// whether it is the first of a burst.
    waiting: Vec<(KeyCode, f64)>,
    /// An entry has just been ended by the key a laser sends; what comes straight
    /// after it is a second reading.
    after_entry: bool,
    /// The burst going on has been said to be unused.
    said: bool,
    /// More keys came after the key that ended an entry, in the same frame: a second
    /// reading, not used, and not yet said.
    trailing: bool,
}

/// Two character keys closer together than this are a burst, not a hand: seconds.
const BURST_GAP: f64 = 0.12;
/// A burst is over this long after its last character: seconds.
const BURST_TAIL: f64 = 0.35;

#[allow(clippy::too_many_arguments)]
fn keys(mut seen_alone: Local<Option<String>>, mut under_ground: Local<f32>, mut commands: Commands, mut doc: ResMut<Doc>, mut orbit: ResMut<Orbit>, snap: Res<Snapshot>, mut typed: MessageReader<KeyboardInput>, mut struck: MessageReader<KeyboardInput>, time: Res<Time<Real>>, mut burst: Local<Burst>, keys: Res<ButtonInput<KeyCode>>, primary: Query<Entity, With<PrimaryWindow>>, sheet: Query<Entity, With<sheet::SheetWindow>>) {
    // A level put on show alone is looked down into: in 3D the view tilts to where
    // its rooms can be seen over its walls, and eases there like every other move.
    // (It took a level picked and then an arrow key, which nothing on screen says.)
    // The ground switched off: the view eases down by a little more than what lies
    // under it is deep, so that the footings come up from behind the tool bar; and
    // back when the ground is switched on again.
    let wanted = if doc.show.ground {
        0.0
    } else {
        let lowest = snap.drawn.iter().filter_map(|p| match &p.shape { plumbline_world::building::Shape::Wall { base, .. } => Some(*base as f32), _ => None }).fold(0.0_f32, f32::min);
        -lowest * 1.2
    };
    if (wanted - *under_ground).abs() > 1.0e-3 {
        orbit.target.y -= wanted - *under_ground;
        *under_ground = wanted;
    }
    if doc.show.storey != *seen_alone {
        if doc.show.storey.is_some() && !doc.plan && !doc.walk && orbit.pitch < 0.85 {
            orbit.pitch = 0.85;
        }
        *seen_alone = doc.show.storey.clone();
    }
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
    // Every key that came in since the last frame, to tell a hand from a laser: a
    // reading is a run of characters closer together than a hand types them.
    let now = time.elapsed_secs_f64();
    let mut alone: Vec<KeyCode> = Vec::new();
    let mut at_once: Vec<KeyCode> = Vec::new();
    let mut chord: Vec<KeyCode> = Vec::new();
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    for event in struck.read() {
        if !event.state.is_pressed() {
            continue;
        }
        if ctrl && doc.entry.is_none() {
            chord.push(event.key_code);
        } else if matches!(event.logical_key, Key::Character(_) | Key::Space) {
            alone.push(event.key_code);
        } else {
            at_once.push(event.key_code);
        }
    }
    if !alone.is_empty() {
        if alone.len() > 1 || now - burst.last < BURST_GAP || !burst.waiting.is_empty() {
            burst.until = now + BURST_TAIL;
        }
        burst.last = now;
    }
    let in_burst = now < burst.until;
    if !in_burst {
        burst.said = false;
        burst.after_entry = false;
    }
    // A change of view a tool or a button asked for.
    if let Some((plan, walk)) = doc.go.take() {
        set_view(doc, &mut orbit, &snap, plan, walk);
    }
    // A thing the view is to go to: its level is put on show if another is, and the
    // view is brought round it and closer.
    if let Some(path) = doc.look_at.take() {
        use plumbline_world::building::Shape;
        if snap.storeys.iter().any(|s| s.path == path) {
            doc.show.storey = Some(path);
            doc.redraw();
        } else if let Some(placed) = snap.drawn.iter().find(|p| p.path == path) {
            let at = match &placed.shape {
                Shape::Wall { a, b, base, height, .. } => [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, base + height / 2.0],
                Shape::Floor { boundary, top, .. } => {
                    let n = boundary.len().max(1) as f64;
                    [boundary.iter().map(|q| q[0]).sum::<f64>() / n, boundary.iter().map(|q| q[1]).sum::<f64>() / n, *top]
                }
                Shape::Run { a, b, .. } => [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0],
                Shape::Box { at, size, .. } => [at[0], at[1], at[2] + size[2] / 2.0],
            };
            if doc.show.storey.is_some() && doc.show.storey != placed.storey {
                doc.show.storey = placed.storey.clone();
            }
            if !doc.walk {
                orbit.target = model::to_view(doc.origin, at);
                orbit.distance = orbit.distance.min(orbit.home.1 * 0.55);
            }
            doc.redraw();
        }
    }
    // A value being typed takes the keyboard until Enter or Escape.
    if let Some((field, mut text)) = doc.entry.take() {
        // Saving is never put off by a field that has the keyboard (J19-46: with a
        // size being typed F2 did nothing and said nothing, the program was closed,
        // and three windows were lost). The job is saved as it stands, and the line
        // says the value being typed is not in it yet.
        if at_once.contains(&KeyCode::F2) || doc.pressed.contains(&KeyCode::F2) || (ctrl && alone.contains(&KeyCode::KeyS)) {
            save(doc);
            if doc.tone == Tone::Done {
                doc.message.push_str(" What is being typed is not in it yet: Enter sets it.");
            }
        }
        doc.pressed.clear();
        burst.waiting.clear();
        let before = (text.clone(), doc.form_at);
        let mut done = None;
        // Fields that go together: Tab or a comma goes on to the next; Enter goes to
        // the first still empty, and with none empty it is done.
        let together = !doc.form.is_empty();
        // Ctrl as the keys themselves tell it, in the order they came: on a slow frame
        // Ctrl, a letter and both let go arrive together, and by the frame's end Ctrl
        // is up (Ctrl+A typed an "a" into a thickness: J24A-1).
        let either = [KeyCode::ControlLeft, KeyCode::ControlRight];
        let mut ctrl = either.iter().any(|k| (keys.pressed(*k) && !keys.just_pressed(*k)) || (keys.just_released(*k) && !keys.just_pressed(*k)));
        let was_fresh = doc.fresh;
        for event in typed.read() {
            if event.logical_key == Key::Control {
                ctrl = event.state.is_pressed();
                continue;
            }
            if !event.state.is_pressed() {
                continue;
            }
            // The key that ends an entry ends it: what a laser sends after it, in the
            // same breath, is a second reading and no part of this one.
            if done.is_some() {
                burst.until = now + BURST_TAIL;
                burst.trailing = true;
                continue;
            }
            // Ctrl with a letter is a command (Ctrl+S saves), never a letter of the value.
            // Ctrl+A is the one a field answers: all of it is taken, to be typed over.
            if ctrl && matches!(event.logical_key, Key::Character(_)) {
                if matches!(&event.logical_key, Key::Character(c) if c.eq_ignore_ascii_case("a")) {
                    doc.fresh = !text.is_empty();
                }
                continue;
            }
            let comma = matches!(&event.logical_key, Key::Character(c) if c.as_str() == ",");
            match &event.logical_key {
                Key::Enter | Key::Tab if together => {
                    let at = doc.form_at;
                    doc.form[at] = text.trim().to_string();
                    let empty = doc.form.iter().position(String::is_empty);
                    let next = if event.logical_key == Key::Tab && at + 1 < doc.form.len() { Some(at + 1) } else { empty };
                    match next {
                        Some(i) => {
                            doc.form_at = i;
                            text = doc.form[i].clone();
                            doc.fresh = !text.is_empty();
                            // Enter that cannot finish says why (J18-2).
                            if event.logical_key == Key::Enter && text.is_empty() {
                                if let Entry::Settings(tool) = field {
                                    doc.say(Tone::Refused, format!("{}: {} is still to be typed. Then Enter, and click on the plan.", tool.name(), tool.fields()[i].0));
                                }
                                if field == Entry::Line {
                                    let name = ["What it is", "What it is priced by", "Its cost type"].get(i).copied().unwrap_or("A field");
                                    doc.say(Tone::Refused, format!("Cost line: {name} is still to be typed. Enter adds the line once all three are in."));
                                }
                                // A level's form said nothing (J22B-34).
                                if field == Entry::Storey {
                                    let name = ["Its name", "The elevation of its floor", "Floor-to-floor"].get(i).copied().unwrap_or("A field");
                                    doc.say(Tone::Refused, format!("Add level: {name} is still to be typed. Enter adds the level once all three are in."));
                                }
                            }
                        }
                        None => done = Some(true),
                    }
                }
                _ if together && comma => {
                    let at = doc.form_at;
                    if at + 1 < doc.form.len() {
                        doc.form[at] = text.trim().to_string();
                        doc.form_at = at + 1;
                        text = doc.form[at + 1].clone();
                        doc.fresh = !text.is_empty();
                    } else {
                        // In the last field there is no next: a comma is a comma (it
                        // was dropped: R21-9).
                        text.push(',');
                    }
                }
                // A laser set to send Tab after its reading: Tab ends an entry as Enter does.
                Key::Enter | Key::Tab => done = Some(true),
                Key::Escape => done = Some(false),
                // A filled field typed over: the first key replaces what was there.
                Key::Backspace if doc.fresh => {
                    text.clear();
                    doc.fresh = false;
                }
                // In an empty field it does nothing. (It went back into the field
                // before and began rubbing that out, while the empty one was still
                // drawn as it had been: four of them, to clear a field, put the next
                // typing in a level's name: J24B-46.)
                Key::Backspace => {
                    text.pop();
                }
                // The space after a comma is not the start of the next field.
                Key::Space if together && (text.is_empty() || doc.fresh) => {}
                Key::Space => text.push(' '),
                Key::Character(c) => {
                    if doc.fresh {
                        text.clear();
                        doc.fresh = false;
                    }
                    text.push_str(c)
                }
                _ => {}
            }
        }
        if done.is_some() {
            burst.after_entry = true;
        }
        let values = if done.is_some() { std::mem::take(&mut doc.form) } else { Vec::new() };
        match done {
            None => {
                if (text.as_str(), doc.form_at) != (before.0.as_str(), before.1) || doc.fresh != was_fresh {
                    doc.touch();
                }
                doc.entry = Some((field, text));
            }
            Some(false) => {
                if field == Entry::Distance || field == Entry::Room {
                    // The points picked for a scale, or for a room, go with what was not typed.
                    doc.draw.picked.clear();
                }
                doc.measuring = None;
                // A length let go leaves the line's start where it was: said, so that
                // "Cancelled" is not read as the wall cancelled (call 86).
                if field == Entry::Length && !doc.draw.picked.is_empty() {
                    doc.say(Tone::Done, "Length let go. The start is kept: click the next corner or type a length, or Esc again to let go of the start.");
                } else {
                    doc.say(Tone::Done, "Cancelled.");
                }
                doc.redraw();
            }
            Some(true) => match field {
                // A price typed here is entered as Assumed, like everything set in this
                // window: there is nowhere in it yet to say where a figure came from.
                Entry::Allowance | Entry::UnitCost | Entry::StretchOut | Entry::Rate => {
                    let what = match field {
                        Entry::Allowance => "allowance",
                        Entry::UnitCost => "unit_cost",
                        Entry::Rate => "order",
                        _ => "stretch_out",
                    };
                    let line = if field != Entry::Allowance { doc.entry_line.take() } else { None };
                    match doc.selected.clone() {
                        None => tools::cant(doc, "set a price", "nothing is selected"),
                        Some(sel) => match doc.job.set_on_line(&sel, what, line.as_deref(), Some(text.trim()), Status::Assumed, SOURCE) {
                            Ok(()) => {
                                doc.job.derive();
                                doc.edit(match field {
                                    Entry::Allowance => "Allowance",
                                    Entry::UnitCost => "Unit cost",
                                    Entry::Rate => "Production rate",
                                    _ => "Stretch-out",
                                });
                                let carried = line.clone();
                                let of = line.map(|l| format!(" of {l:?}")).unwrap_or_default();
                                // A price typed as a bare figure is said back as money: "24"
                                // is answered "$24.00", as the line then shows it.
                                let typed = text.trim();
                                let said = match typed.parse::<f64>() {
                                    Ok(x) if !matches!(field, Entry::StretchOut | Entry::Rate) => plumbline_world::view::money(x),
                                    _ => typed.to_string(),
                                };
                                // A roof is priced once: the same line on the roof's other
                                // planes, where it has no price (or no rate) yet, takes what
                                // was typed here, and it is said. One that has its own is
                                // left as it is. (Four planes alike took twenty entries:
                                // J24A-8.)
                                let mut also: Vec<String> = Vec::new();
                                if let (Some(l), true) = (carried.as_deref(), matches!(field, Entry::UnitCost | Entry::Rate)) {
                                    let est = plumbline_world::view::estimate(&mut doc.job);
                                    for sister in doc.job.sister_planes(&sel) {
                                        let open = est.lines.iter().find(|x| x.path == sister && x.description == l).is_some_and(|x| match field {
                                            Entry::UnitCost => x.material.is_none() && x.unit_cost.as_ref().is_none_or(|v| v.value.is_none()),
                                            _ => x.rate.as_ref().is_some_and(|r| r.value.is_none()),
                                        });
                                        if open && doc.job.set_on_line(&sister, what, Some(l), Some(text.trim()), Status::Assumed, SOURCE).is_ok() {
                                            also.push(doc.job.find(&sister).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or(sister));
                                        }
                                    }
                                    if !also.is_empty() {
                                        doc.job.derive();
                                    }
                                }
                                let also = match also.len() {
                                    0 => String::new(),
                                    1 => format!(" The same line on {} had none and takes it too.", plumbline_world::view::listed(&also)),
                    _ => format!(" The same line on {} had none: they take it too.", plumbline_world::view::listed(&also)),
                                };
                                // An allowance on a thing with no cost line is in the budget and
                                // in no line of the price: said the moment it is typed, with
                                // what to do, and not only as a figure on the top bar (J24A-9;
                                // whether it belongs in the price is QUESTIONS.md Q26).
                                let bare = field == Entry::Allowance && doc.job.find(&sel).is_some_and(|e| doc.job.world.get::<plumbline_world::Cost>(e).is_none_or(|c| c.lines.is_empty()));
                                let also = if bare { format!("{also} It is the budget's figure and is not in the price: give this cost lines (Assembly, or Add cost line) to carry it in.") } else { also };
                                doc.say(if bare { Tone::Warned } else { Tone::Done }, format!("{sel}: {}{of} set to {said}.{also}", if field == Entry::Rate { "production rate".to_string() } else { what.replace('_', " ") }));
                            }
                            Err(why) => {
                                tools::cant(doc, match field { Entry::StretchOut => "set that stretch-out", Entry::Rate => "set that production rate", _ => "set that price" }, why);
                                // The field stays open on the same line with what was
                                // typed, to be put right: typing replaces it. (It
                                // closed, and the "0" typed next to correct "-12" took
                                // the Cable tool in hand: J25A-16.)
                                doc.open(field, text.trim());
                                doc.entry_line = line;
                                doc.fresh = true;
                            }
                        },
                    }
                    doc.redraw();
                }
                Entry::Settings(tool) => tools::settings_entered(doc, tool, &values),
                Entry::Length => tools::typed_length(doc, &text),
                Entry::Storey => tools::new_storey(doc, &values),
                Entry::Line => tools::new_line(doc, &values),
                Entry::Drawing => plans::add(doc, &text),
                Entry::Distance => tools::scale_entered(doc, &text),
                Entry::Measure => tools::measured(doc, &text),
                Entry::Room => tools::room_named(doc, &text),
                Entry::Division => {
                    let line = doc.entry_line.take();
                    match (doc.selected.clone(), plumbline_world::Division::parse(&text)) {
                        (None, _) => tools::cant(doc, "set the division", "nothing is selected"),
                        (_, Err(why)) => tools::cant(doc, "set the division", why),
                        (Some(sel), Ok(division)) => match doc.job.set_division(&sel, line.as_deref(), division) {
                            Ok(()) => {
                                doc.job.derive();
                                doc.edit("Division");
                                doc.say(Tone::Done, format!("{sel}: {} is in {}.", line.map(|l| format!("{l:?}")).unwrap_or_else(|| "its line".to_string()), division.said()));
                            }
                            Err(why) => tools::cant(doc, "set the division", why),
                        },
                    }
                    doc.redraw();
                }
                Entry::BringIn(what) => bring::bring(doc, what, &text),
                // What it is made of, and the job's overhead and profit: Assumed, like the rest.
                Entry::Material | Entry::Overhead | Entry::Profit => {
                    let root = doc.job.world.get::<plumbline_world::Node>(doc.job.root()).map(|n| n.path.clone()).unwrap_or_default();
                    let (target, what) = match field {
                        Entry::Material => (doc.selected.clone(), "material"),
                        Entry::Overhead => (Some(root), "overhead"),
                        _ => (Some(root), "profit"),
                    };
                    match target {
                        None => tools::cant(doc, "set the material", "nothing is selected"),
                        Some(path) => match doc.job.set(&path, what, Some(text.trim()), Status::Assumed, SOURCE) {
                            Ok(()) => {
                                doc.job.derive();
                                doc.edit(what);
                                // A material the job does not have is said at once: the
                                // lines made of it now have no figure.
                                let known = field != Entry::Material || {
                                    let mut q = doc.job.world.query::<(&plumbline_world::Node, &plumbline_world::Kind)>();
                                    q.iter(&doc.job.world).any(|(n, k)| *k == plumbline_world::Kind::Material && (n.name == text.trim() || n.path == text.trim()))
                                };
                                if known {
                                    // Overhead and profit are said back as the recap
                                    // shows them: "8" is answered "8.00 %".
                                    let typed = text.trim();
                                    let said = match typed.parse::<f64>() {
                                        Ok(x) if matches!(what, "overhead" | "profit") => format!("{} %", plumbline_world::view::figure(x, 2)),
                                        _ => typed.to_string(),
                                    };
                                    // More than the job costs again is taken, and said
                                    // as what it is: 250 typed for 25 tripled a price
                                    // without a word (J22A-19).
                                    match plumbline_world::view::markup_large_said(typed).filter(|_| matches!(what, "overhead" | "profit")) {
                                        Some(large) => doc.say(Tone::Warned, format!("{path}: {what} set to {said}. {large}")),
                                        None => doc.say(Tone::Done, format!("{path}: {what} set to {said}.")),
                                    }
                                } else {
                                    doc.say(Tone::Warned, format!("{path}: material set to {:?}, which is not a material of this job. The lines made of it have no figure.", text.trim()));
                                }
                            }
                            Err(why) => tools::cant(doc, &format!("set the {what}"), why),
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
    // With a tool in hand: Esc lets go of what is being drawn, then of the tool; a digit
    // typed while a wall or a run has its start is the beginning of a length.
    if doc.draw.tool.is_some() {
        if keys.just_pressed(KeyCode::Escape) {
            if doc.draw.picked.is_empty() {
                doc.draw.tool = None;
                doc.say(Tone::Done, "Tool put down.");
            } else {
                doc.draw.picked.clear();
                doc.say(Tone::Done, "Let go. Click a new start, or Esc again to put the tool down.");
            }
            doc.redraw();
            return;
        }
        // (A move is a line too: from the point picked, so far toward the pointer.)
        let drawing_a_line = matches!(doc.draw.tool, Some(tools::Tool::Wall | tools::Tool::Run(_) | tools::Tool::Move)) && doc.draw.picked.len() == 1;
        for event in typed.read() {
            if let (true, true, Key::Character(c)) = (drawing_a_line, event.state.is_pressed(), &event.logical_key) {
                if c.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
                    doc.open(Entry::Length, c.to_string());
                    return;
                }
            }
        }
    }
    typed.clear();

    // A reading typed with nothing picked to take it does nothing to the job: its
    // characters are not keys, and the Tab or Enter after it is not either. It is said
    // once. (Escape, and the keys that save, still act.)
    // A second reading that came in the very frame the first ended: said now, however
    // long that frame took.
    if burst.trailing {
        burst.trailing = false;
        burst.said = true;
        let first = doc.message.clone();
        doc.say(Tone::Refused, format!("{first}\nA second reading came straight after that one and was not used. One reading to a value: click the next value, then shoot."));
        doc.touch();
    }
    if in_burst {
        burst.waiting.clear();
        if !burst.said {
            burst.said = true;
            if burst.after_entry {
                let first = doc.message.clone();
                doc.say(Tone::Refused, format!("{first}\nA second reading came straight after that one and was not used. One reading to a value: click the next value, then shoot."));
            } else {
                doc.say(Tone::Refused, "A reading came in with no value picked to take it, and was not used. Click the value first (in the panel on the right), then shoot or type.");
            }
            doc.touch();
        }
        at_once.retain(|k| matches!(k, KeyCode::Escape | KeyCode::F2));
    }
    // A character key that came in alone acts once it is plain that nothing follows it.
    burst.waiting.extend(alone.iter().filter(|_| !in_burst).map(|k| (*k, now)));
    let mut acting = ButtonInput::<KeyCode>::default();
    for key in at_once {
        acting.press(key);
    }
    // A button on screen is the key it stands for.
    for key in doc.pressed.drain(..) {
        acting.press(key);
    }
    // With Ctrl held: save, undo, redo, as in any program.
    if chord.contains(&KeyCode::KeyS) {
        save(doc);
    }
    if chord.contains(&KeyCode::KeyZ) {
        tools::step(doc, false);
    }
    if chord.contains(&KeyCode::KeyY) {
        tools::step(doc, true);
    }
    burst.waiting.retain(|(key, since)| {
        if now - since >= BURST_GAP {
            acting.press(*key);
            false
        } else {
            true
        }
    });
    let keys = &acting;

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
                Ok(_) => doc.open(field, ""),
                Err(why) => tools::cant(doc, if field == Entry::Pitch { "set a pitch" } else { "move an eave" }, why),
            }
            doc.touch();
        }
    }

    // Z takes the last step back and Y does it again; Delete removes what is selected.
    if keys.just_pressed(KeyCode::KeyZ) {
        tools::step(doc, false);
    }
    if keys.just_pressed(KeyCode::KeyY) {
        tools::step(doc, true);
    }
    if keys.just_pressed(KeyCode::Delete) {
        tools::delete(doc);
    }

    // A price on what is selected: B for a budget rate (its allowance), U for the unit
    // cost of its first cost line that has no price yet. Each says why not before a field
    // opens that could only be refused.
    for (key, field) in [(KeyCode::KeyB, Entry::Allowance), (KeyCode::KeyU, Entry::UnitCost)] {
        if keys.just_pressed(key) {
            let ready: Result<Option<String>, String> = match doc.selected.clone() {
                None => Err("nothing is selected".into()),
                Some(sel) => {
                    let kind = doc.job.kind(&sel);
                    let lines: Vec<(String, bool)> = doc.job.find(&sel).and_then(|e| doc.job.world.get::<plumbline_world::Cost>(e)).map(|c| c.lines.iter().map(|l| (l.description.clone(), l.unit_cost.value.is_some())).collect()).unwrap_or_default();
                    match (field, kind.and_then(|k| k.priced_by())) {
                        (Entry::Allowance, None) => Err(format!("a {} has no area or length an allowance could be a rate on", kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default())),
                        (Entry::Allowance, Some(_)) => Ok(None),
                        _ if lines.is_empty() => Err(format!("{sel} has no cost line to price. Give it an assembly first.")),
                        _ => match lines.iter().find(|l| !l.1) {
                            Some((line, _)) => Ok(Some(line.clone())),
                            None => Err(format!("every cost line of {sel} has a price already. This prices the next line that has none; a price that is there is changed on the estimate sheet.")),
                        },
                    }
                }
            };
            match ready {
                Ok(line) => {
                    doc.open(field, "");
                    doc.entry_line = line;
                }
                Err(why) => tools::cant(doc, if field == Entry::Allowance { "set an allowance" } else { "set a unit cost" }, why),
            }
            doc.touch();
        }
    }

    // N names what the selected element is made of; O and R type the job's overhead and
    // profit; L switches the selected element's Labor line off or on.
    for (key, field) in [(KeyCode::KeyN, Entry::Material), (KeyCode::KeyO, Entry::Overhead), (KeyCode::KeyR, Entry::Profit)] {
        if keys.just_pressed(key) {
            if field == Entry::Material && doc.selected.is_none() {
                tools::cant(doc, "set a material", "nothing is selected");
            } else {
                doc.open(field, "");
            }
            doc.touch();
        }
    }
    // A quarter turn of a thing set down, counter-clockwise seen from above: Q, or Turn
    // on its panel (J25A-3: a panelboard on a wall that runs north could not face the
    // room). On foot Q is the way down.
    if keys.just_pressed(KeyCode::KeyQ) && !doc.walk {
        match doc.selected.clone() {
            None => tools::cant(doc, "turn it", "nothing is selected"),
            Some(sel) => {
                let name = snap.selected.as_ref().map(|s| s.name.clone()).unwrap_or_else(|| sel.clone());
                match plumbline_world::editing::turn(&mut doc.job, &sel, 90.0, plumbline_world::editing::Entered { status: Status::Assumed, source: "turned by hand in the model" }) {
                    Ok(now) => {
                        doc.job.derive();
                        doc.edit("Turn");
                        let faces = match now.rem_euclid(360.0).round() as i32 {
                            0 => "its width runs east and west, as it was set down",
                            90 | 270 => "its width runs north and south",
                            _ => "its width runs east and west, turned about",
                        };
                        match scene::in_what_now(doc, &sel) {
                            Some(what) => doc.say(Tone::Refused, format!("Turned {name} a quarter: {faces}. It is in the same space as {what}.")),
                            None => doc.say(Tone::Done, format!("Turned {name} a quarter: {faces}. It is clear of everything.")),
                        }
                        doc.redraw();
                    }
                    Err(why) => tools::cant(doc, "turn it", why),
                }
            }
        }
        doc.touch();
    }
    if keys.just_pressed(KeyCode::KeyL) {
        match doc.selected.clone() {
            None => tools::cant(doc, "switch labor", "nothing is selected"),
            Some(sel) => {
                let labor: Vec<(String, bool)> = doc.job.find(&sel).and_then(|e| doc.job.world.get::<plumbline_world::Cost>(e)).map(|c| c.lines.iter().filter(|l| l.class == Some(plumbline_world::Class::Labor)).map(|l| (l.description.clone(), l.off)).collect()).unwrap_or_default();
                match labor.as_slice() {
                    [] => tools::cant(doc, "switch labor", format!("{sel} has no labor line")),
                    [(line, off)] => match doc.job.set_off(&sel, Some(line), !off) {
                        Ok(()) => {
                            doc.job.derive();
                            doc.edit("Labor on or off");
                            doc.say(Tone::Done, format!("{sel}: {line:?} switched {}.", if *off { "on" } else { "off" }));
                        }
                        Err(why) => tools::cant(doc, "switch labor", why),
                    },
                    many => tools::cant(doc, "switch labor", format!("{sel} has {} labor lines. One of several is switched on the estimate sheet.", many.len())),
                }
            }
        }
        doc.redraw();
    }

    // The number keys take a build tool in hand.
    for (key, tool) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9, KeyCode::Digit0].into_iter().zip(tools::Tool::KEYED) {
        if keys.just_pressed(key) {
            // With a tool in hand a number is never another tool: a length typed with
            // no start clicked took a second tool in hand by its first digit, and the
            // rest of the length went into that tool's sizes (call 86, J21-13).
            match doc.draw.tool {
                Some(held) => {
                    let first = if matches!(held, tools::Tool::Wall | tools::Tool::Run(_) | tools::Tool::Move) { "click where it starts first, then a number is a length" } else { "click where it goes" };
                    doc.say(Tone::Refused, format!("{} is in hand: {first}. A number typed now is not another tool. To change tools, click the tool, or press Esc first.", held.name()));
                    doc.touch();
                }
                None => tools::start(doc, tool),
            }
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
        let (plan, was_on_foot) = (!doc.plan, doc.walk);
        set_view(doc, &mut orbit, &snap, plan, false);
        // The view says where it went (VISUAL.md A9), unless coming off foot already did.
        if !was_on_foot {
            let said = view_said(doc, &snap, plan, true);
            doc.say(Tone::Done, said);
        }
    }
    if keys.just_pressed(KeyCode::KeyV) {
        let walk = !doc.walk;
        set_view(doc, &mut orbit, &snap, false, walk);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        // Frame says what it did, and on foot says why it did nothing (J22B-5, J22B-7).
        if doc.walk {
            doc.say(Tone::Refused, "Frame brings the whole job into view from outside. On foot there is no outside: press V to come out first.");
        } else {
            orbit.go_home();
            doc.say(Tone::Done, "The whole job, in view.");
        }
        doc.touch();
    }
    if keys.just_pressed(KeyCode::Escape) && doc.selected.is_some() {
        doc.selected = None;
        doc.say(Tone::Done, "Nothing selected.");
        doc.touch();
    }
    if keys.just_pressed(KeyCode::Tab) {
        // The next plane of the roof on show, in job order. With no roof, nothing.
        let planes: Vec<String> = snap.planes.iter().map(|p| p.path.clone()).collect();
        if !planes.is_empty() {
            let at = doc.selected.as_ref().and_then(|s| planes.iter().position(|p| p == s)).map(|i| (i + 1) % planes.len()).unwrap_or(0);
            doc.select(&planes[at]);
        }
    }
    if keys.just_pressed(KeyCode::KeyC) {
        // Open the list of assemblies that go on what is selected, to pick one from
        // (it used to put on the first that fitted, with no choice).
        match doc.selected.clone() {
            None => tools::cant(doc, "give an assembly", "nothing is selected. Click a wall, a floor, a room, a roof plane, a door or a window first"),
            Some(sel) => {
                let kind = doc.job.kind(&sel);
                let fits = {
                    let mut q = doc.job.world.query::<&Recipe>();
                    q.iter(&doc.job.world).any(|r| kind.is_some_and(|k| r.applies_to.contains(&k)))
                };
                let a = kind.map(|k| theme::kind_name(k).to_lowercase()).unwrap_or_default();
                if doc.choosing {
                    doc.choosing = false;
                    doc.say(Tone::Done, "The list of assemblies is put away.");
                } else if fits {
                    doc.choosing = true;
                    doc.say(Tone::Done, format!("Pick an assembly for this {a} from the list on the right."));
                } else {
                    tools::cant(doc, "give an assembly", format!("this job has no assembly that goes on a {a}. Under Bring in, on the left, Starter assemblies brings in a set for a whole house"));
                }
            }
        }
        doc.redraw();
    }
    // An assembly picked from the list: put on what is selected, in place of the one it
    // has if it has one.
    if let Some(name) = doc.give.take() {
        doc.choosing = false;
        if let Some(sel) = doc.selected.clone() {
            let called = doc.job.find(&sel).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| sel.clone());
            let priced_before = plumbline_world::view::estimate(&mut doc.job);
            match doc.job.change_assembly(&sel, &name, Status::Assumed, SOURCE) {
                Ok((added, taken)) => {
                    doc.job.derive();
                    // What the lines that came off took out of the job cost (J23-5).
                    let money = plumbline_world::view::came_off_said(&priced_before, &mut doc.job).filter(|s| s.contains('$')).and_then(|s| s.split_once(": ").map(|(_, rest)| format!(" {}{}", rest[..1].to_uppercase(), &rest[1..]))).unwrap_or_default();
                    doc.edit("Assembly");
                    // Its lines are on the estimate: the sheet turns to it.
                    doc.tab = model::Tab::Estimate;
                    let lines = |n: usize| plumbline_world::view::so_many(n, "cost line");
                    let off = if taken.is_empty() { String::new() } else { format!(" {} of the assembly it had came off: {}.", lines(taken.len()), taken.join(", ")) };
                    // A roof is one thing to an estimator: an assembly given to one of
                    // its planes goes on each of its other planes that has nothing
                    // priced on it yet, and it is said. A plane with lines of its own is
                    // left, and named. (One plane of four was priced and nothing said
                    // the other three were bare: J24A-6.)
                    let (mut too, mut left): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
                    for sister in doc.job.sister_planes(&sel) {
                        let its = doc.job.find(&sister).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| sister.clone());
                        let bare = doc.job.find(&sister).is_some_and(|e| doc.job.world.get::<plumbline_world::Cost>(e).is_none_or(|c| c.lines.is_empty()));
                        if bare && doc.job.change_assembly(&sister, &name, Status::Assumed, SOURCE).is_ok() {
                            too.push(its);
                        } else {
                            left.push(its);
                        }
                    }
                    if !too.is_empty() {
                        doc.job.derive();
                        doc.edit("Assembly");
                    }
                    let roof = match (too.is_empty(), left.is_empty()) {
                        (true, true) => String::new(),
                        (false, true) => format!(" So {} {}: the same lines on each.", if too.len() == 1 { "is" } else { "are" }, plumbline_world::view::listed(&too)),
                        (false, false) => format!(" So {} {}: the same lines on each. {} of the same roof had lines of its own and is as it was.", if too.len() == 1 { "is" } else { "are" }, plumbline_world::view::listed(&too), plumbline_world::view::listed(&left)),
                        (true, false) => format!(" {} of the same roof had lines of its own and is as it was.", plumbline_world::view::listed(&left)),
                    };
                    doc.say(if money.is_empty() { Tone::Done } else { Tone::Warned }, format!("{called} is now {name:?}: {} added.{off}{money}{roof}", lines(added.len())));
                }
                Err(why) => tools::cant(doc, "give that assembly", why),
            }
        }
        doc.redraw();
    }
    if keys.just_pressed(KeyCode::F2) {
        save(doc);
    }
}
