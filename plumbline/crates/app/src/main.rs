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
        walk: false,
        show: Show { architecture: true, roof: true, mechanical: true, electrical: true, plumbing: true, ghost: false, storey: None, certainty: false, clashes: false, ground: true, framing: false, finishes: true, off_levels: Default::default(), off_systems: Default::default() },
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
            (keys, plans::dropped, plans::read, tools::clicks, scene::carry, tools::record, scene::orbit_input, model::refresh, scene::rebuild, scene::tint, panels::rebuild, sheet::rebuild, (scene::place_camera, scene::ground_tone, glass::follow).chain(), tools::pointer, scene::draw_lines, tools::preview, panels::place_labels, panels::write_pointer, panels::scroll_panels, map::write).chain(),
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
    doc.plan = plan;
    doc.walk = walk;
    // A drawing under the plan lies over the floor on plan and under it in 3D.
    doc.redraw();
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
                more.push_str(&format!("\nLost with them: the assembly or cost lines on {}", lost.join(", ")));
            }
            for moved in &report.renumbered {
                more.push_str(&format!("\n{moved}"));
            }
            // The roof under the pointer has moved; what the pointer read is no longer so.
            doc.pointer = None;
            doc.hover = None;
            doc.edit(what);
            let tone = if lost.is_empty() && report.renumbered.is_empty() { Tone::Done } else { Tone::Warned };
            doc.say(tone, format!("{what} on edge {edge} of {roof_path}. The roof is now {} planes, {} lines{more}", report.planes, report.lines));
        }
        Err(why) => tools::cant(doc, "change the roof", why),
    }
    doc.redraw();
}

/// Save the job to its file.
/// A window on its way shut: it goes the frame after it is marked, as the engine's own
/// closing does.
#[derive(Component)]
struct Shutting;

/// A window asked to close. With work not saved, the first asking is answered with a
/// warning and the window stays; a second asking, or a job that is saved, closes it.
/// (Found beside J19-46: closing with work not saved lost it without a word.)
fn closing(mut commands: Commands, mut asked: MessageReader<bevy::window::WindowCloseRequested>, shutting: Query<Entity, With<Shutting>>, mut doc: ResMut<Doc>, mut warned: Local<bool>) {
    for window in &shutting {
        commands.entity(window).despawn();
    }
    if !doc.edited {
        *warned = false;
    }
    for ask in asked.read() {
        if doc.edited && !*warned {
            *warned = true;
            doc.say(Tone::Refused, "Not closed: the job has changes that are not saved. Save (Ctrl+S), or close again to leave without saving them.");
            doc.touch();
        } else {
            commands.entity(ask.window).try_insert(Shutting);
        }
    }
}

fn save(doc: &mut Doc) {
    match std::fs::write(&doc.path, file::save(&mut doc.job)) {
        Ok(()) => {
            doc.edited = false;
            // By its name: where it lies on the computer is not a builder's word (J19-47).
            doc.say(Tone::Done, format!("Saved {}.", doc.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "the job".into())));
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
        for event in typed.read() {
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
            if ctrl && matches!(event.logical_key, Key::Character(_)) {
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
                Key::Backspace => {
                    if text.is_empty() && together && doc.form_at > 0 {
                        doc.form_at -= 1;
                        text = doc.form[doc.form_at].clone();
                    } else {
                        text.pop();
                    }
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
                if (text.as_str(), doc.form_at) != (before.0.as_str(), before.1) {
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
                doc.say(Tone::Done, "Cancelled.");
                doc.redraw();
            }
            Some(true) => match field {
                // A price typed here is entered as Assumed, like everything set in this
                // window: there is nowhere in it yet to say where a figure came from.
                Entry::Allowance | Entry::UnitCost | Entry::StretchOut => {
                    let what = match field {
                        Entry::Allowance => "allowance",
                        Entry::UnitCost => "unit_cost",
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
                                    _ => "Stretch-out",
                                });
                                let of = line.map(|l| format!(" of {l:?}")).unwrap_or_default();
                                // A price typed as a bare figure is said back as money: "24"
                                // is answered "$24.00", as the line then shows it.
                                let typed = text.trim();
                                let said = match typed.parse::<f64>() {
                                    Ok(x) if field != Entry::StretchOut => format!("${}", plumbline_world::view::figure(x, 2)),
                                    _ => typed.to_string(),
                                };
                                doc.say(Tone::Done, format!("{sel}: {}{of} set to {said}.", what.replace('_', " ")));
                            }
                            Err(why) => tools::cant(doc, if field == Entry::StretchOut { "set that stretch-out" } else { "set that price" }, why),
                        },
                    }
                    doc.redraw();
                }
                Entry::Settings(tool) => tools::settings_entered(doc, tool, &values),
                Entry::Length => tools::typed_length(doc, &text),
                Entry::Storey => tools::new_storey(doc, &values),
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
                                    doc.say(Tone::Done, format!("{path}: {what} set to {said}."));
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
            tools::start(doc, tool);
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
            match doc.job.change_assembly(&sel, &name, Status::Assumed, SOURCE) {
                Ok((added, taken)) => {
                    doc.job.derive();
                    doc.edit("Assembly");
                    // Its lines are on the estimate: the sheet turns to it.
                    doc.tab = model::Tab::Estimate;
                    let lines = |n: usize| if n == 1 { "1 cost line".to_string() } else { format!("{n} cost lines") };
                    let off = if taken.is_empty() { String::new() } else { format!(" {} of the assembly it had came off: {}.", lines(taken.len()), taken.join(", ")) };
                    doc.say(Tone::Done, format!("{called} is now {name:?}: {} added.{off}", lines(added.len())));
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
