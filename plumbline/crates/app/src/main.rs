//! The 3D window and the sheet window.
//!
//! Both are views over one job world (`plumbline-world`). Nothing here works out a
//! quantity: the window draws what the world holds, sends the estimator's edits to it,
//! and draws again. The sheet is a second operating-system window on the same world, so
//! it can sit on another monitor and it follows every edit as it is made.
//!
//!   plumbline-app <job file> [roof path]
//!
//! Mouse: click a plane to select it; drag with the right button to orbit; scroll to zoom.
//! Keys, on the selected plane of a roof solved from a footprint:
//!   = / -      pitch up or down by 1 in 12        ] / [    move its eave out or in by 6 in
//!   P          type a pitch (6:12), Enter          M        type a distance to move its eave
//!   G          make its edge a gable               C        give it the next condition that fits
//!   Tab        select the next plane               arrows   orbit
//!   F2         save the job                        F12      save a picture of both windows
//!
//! A value set here is entered as Assumed, with its source saying it was set in the 3D
//! view: the estimator chose it, and nobody has yet said where it comes from.

use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{PrimaryWindow, WindowPosition, WindowRef};

use plumbline_world::value::Status;
use plumbline_world::{file, roof, solve, units, view, Condition, Footprint, Job, Kind, Recipe, Rise, Solved, Val};

const SOURCE: &str = "set in the 3D view; not yet confirmed against a drawing";

/// The open job and what the windows need to know about it.
#[derive(Resource)]
struct Doc {
    job: Job,
    path: PathBuf,
    /// The roof on show.
    roof: Option<String>,
    /// The selected element, by path.
    selected: Option<String>,
    /// What the last action did, or why it was refused.
    message: String,
    /// Goes up on every change, so views know to redraw.
    revision: u64,
    drawn: u64,
    /// A value being typed: which field, and the text so far.
    entry: Option<(Entry, String)>,
    /// The plan point the model's origin is drawn at, in feet. The view is drawn relative
    /// to it in 32-bit numbers; the model itself stays 64-bit.
    origin: [f64; 3],
    /// Where the pointer is on the roof, in plan feet and height.
    pointer: Option<[f64; 3]>,
    /// The lines to draw, in view coordinates.
    lines: Vec<(Vec3, Vec3, Kind, bool)>,
    shots: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum Entry {
    Pitch,
    Move,
}

#[derive(Resource)]
struct Orbit {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

/// A plane's mesh in the view. Redrawing removes every one of these and makes them again.
#[derive(Component)]
struct PlaneMesh;
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct SheetWindow;
#[derive(Component)]
struct SheetText;
#[derive(Component)]
struct HudText;

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
    let doc = Doc { job, path: PathBuf::from(path), roof, selected: None, message, revision: 1, drawn: 0, entry: None, origin: [0.0; 3], pointer: None, lines: Vec::new(), shots: 0 };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "plumbline: roof".into(), resolution: (1100u32, 760u32).into(), position: WindowPosition::At(IVec2::new(0, 0)), ..default() }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .insert_resource(ClearColor(Color::srgb(0.10, 0.11, 0.13)))
        .insert_resource(doc)
        .insert_resource(Orbit { target: Vec3::ZERO, yaw: -0.6, pitch: 0.62, distance: 80.0 })
        .add_systems(Startup, setup)
        .add_systems(Update, (keys, orbit_input, redraw, place_camera, draw_lines, write_hud).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3d::default(), MainCamera, Transform::from_xyz(0.0, 40.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y)));
    // One strong light from the south-west and a weak one from the north-east, so every
    // slope takes a different shade and the roof reads as a solid.
    commands.spawn((DirectionalLight { illuminance: 11000.0, ..default() }, Transform::from_xyz(-50.0, 45.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y)));
    commands.spawn((DirectionalLight { illuminance: 2200.0, ..default() }, Transform::from_xyz(60.0, 25.0, -40.0).looking_at(Vec3::ZERO, Vec3::Y)));
    commands.insert_resource(GlobalAmbientLight { brightness: 140.0, ..default() });

    // What the pointer and the selection say, over the 3D view.
    commands.spawn((Node { position_type: PositionType::Absolute, top: px(10.0), left: px(12.0), max_width: px(1070.0), ..default() },)).with_child((Text::new(""), TextFont::from_font_size(15.0), TextColor(Color::srgb(0.92, 0.93, 0.95)), HudText));

    // The sheet: a second window on the same world.
    let sheet = commands.spawn((Window { title: "plumbline: sheet".into(), resolution: (640u32, 760u32).into(), position: WindowPosition::At(IVec2::new(1110, 0)), ..default() }, SheetWindow)).id();
    // Its camera looks at a layer nothing 3D is on, so the roof's lines are not drawn over
    // the sheet. Text and panels are not on layers and still show.
    let sheet_camera = commands.spawn((Camera2d, RenderLayers::layer(1), RenderTarget::Window(WindowRef::Entity(sheet)))).id();
    commands
        .spawn((Node { position_type: PositionType::Absolute, top: px(10.0), left: px(12.0), max_width: px(616.0), ..default() }, UiTargetCamera(sheet_camera)))
        .with_child((Text::new(""), TextFont::from_font_size(14.0), TextColor(Color::srgb(0.92, 0.93, 0.95)), SheetText));
}

/// A plan point and a height, as a point in the view: x east, y up, z south.
fn to_view(origin: [f64; 3], p: [f64; 2], height: f64) -> Vec3 {
    Vec3::new((p[0] - origin[0]) as f32, (height - origin[2]) as f32, -((p[1] - origin[1]) as f32))
}

/// Cut a simple outline into triangles (ear clipping). Indices into `pts`.
fn triangles(pts: &[[f64; 2]]) -> Vec<[u32; 3]> {
    let n = pts.len();
    let area: f64 = (0..n).map(|i| pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1]).sum();
    let mut ring: Vec<usize> = if area >= 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
    let cross = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    let mut out = Vec::new();
    let mut guard = 0;
    while ring.len() > 3 && guard < 10_000 {
        guard += 1;
        let m = ring.len();
        let mut cut = None;
        for i in 0..m {
            let (a, b, c) = (pts[ring[(i + m - 1) % m]], pts[ring[i]], pts[ring[(i + 1) % m]]);
            if cross(a, b, c) <= 1e-12 {
                continue;
            }
            let inside = ring.iter().any(|&k| {
                let p = pts[k];
                p != a && p != b && p != c && cross(a, b, p) > 1e-12 && cross(b, c, p) > 1e-12 && cross(c, a, p) > 1e-12
            });
            if !inside {
                cut = Some(i);
                break;
            }
        }
        // A sliver with no clean ear left: take any corner rather than stop drawing.
        let i = cut.unwrap_or(0);
        let m = ring.len();
        out.push([ring[(i + m - 1) % m] as u32, ring[i] as u32, ring[(i + 1) % m] as u32]);
        ring.remove(i);
    }
    if ring.len() == 3 {
        out.push([ring[0] as u32, ring[1] as u32, ring[2] as u32]);
    }
    out
}

fn status_of(job: &mut Job, path: &str) -> Status {
    let Some(e) = job.find(path) else { return Status::Unknown };
    job.world.get::<plumbline_world::Measure>(e).and_then(|m| m.0.get("surface_area")).map(|d| d.status).unwrap_or(Status::Unknown)
}

/// Draw the roof again from the world whenever the world has changed.
fn redraw(mut commands: Commands, mut doc: ResMut<Doc>, mut orbit: ResMut<Orbit>, old: Query<Entity, With<PlaneMesh>>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut sheet: Query<&mut Text, With<SheetText>>) {
    if doc.drawn == doc.revision {
        return;
    }
    let first = doc.drawn == 0;
    doc.drawn = doc.revision;
    for e in &old {
        commands.entity(e).despawn();
    }
    let doc = &mut *doc;
    let Some(roof_path) = doc.roof.clone() else { return };
    let under: Vec<String> = doc.job.children().get(&roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let (planes, _) = roof::solid_planes(&mut doc.job);
    let planes: Vec<roof::SolidPlane> = planes.into_iter().filter(|p| under.contains(&p.path)).collect();

    // The view's origin: the middle of the roof on plan, at its eaves. Set once, so the
    // picture does not jump when an edit moves the roof.
    if first && !planes.is_empty() {
        let all: Vec<[f64; 2]> = planes.iter().flat_map(|p| p.points.iter().copied()).collect();
        let (lo, hi) = all.iter().fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])]));
        let low = planes.iter().map(|p| p.eave_height).fold(f64::MAX, f64::min);
        doc.origin = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, low];
        let size = ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2)).sqrt() as f32;
        orbit.distance = size * 1.25;
        orbit.target = Vec3::new(0.0, size * 0.08, 0.0);
    }

    for plane in &planes {
        let positions: Vec<Vec3> = plane.points.iter().map(|p| to_view(doc.origin, *p, plane.height_at(*p))).collect();
        let tris = triangles(&plane.points);
        let Some(first_tri) = tris.first() else { continue };
        let (a, b, c) = (positions[first_tri[0] as usize], positions[first_tri[1] as usize], positions[first_tri[2] as usize]);
        let mut normal = (b - a).cross(c - a).normalize_or_zero();
        if normal.y < 0.0 {
            normal = -normal;
        }
        let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[normal.x, normal.y, normal.z]; positions.len()])
            .with_inserted_indices(Indices::U32(tris.into_iter().flatten().collect()));
        // Colour says what the estimator needs at a glance: selected, flagged, or settled.
        let selected = doc.selected.as_deref() == Some(plane.path.as_str());
        let has_condition = doc.job.find(&plane.path).is_some_and(|e| doc.job.world.get::<Condition>(e).is_some_and(|c| c.0.value.is_some()));
        let colour = match (selected, status_of(&mut doc.job, &plane.path), has_condition) {
            (true, _, _) => Color::srgb(0.98, 0.78, 0.25),
            (_, Status::Unknown, _) => Color::srgb(0.75, 0.30, 0.30),
            (_, Status::Assumed, _) => Color::srgb(0.80, 0.72, 0.42),
            (_, _, true) => Color::srgb(0.72, 0.45, 0.24),
            _ => Color::srgb(0.62, 0.66, 0.72),
        };
        let material = materials.add(StandardMaterial { base_color: colour, perceptual_roughness: 0.85, double_sided: true, cull_mode: None, ..default() });
        let path = plane.path.clone();
        commands
            .spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material), PlaneMesh))
            .observe(move |_click: On<PointerClick>, mut doc: ResMut<Doc>| {
                doc.selected = Some(path.clone());
                doc.message = format!("selected {path}");
                doc.revision += 1;
            })
            .observe(|moved: On<PointerMove>, mut doc: ResMut<Doc>| {
                if let Some(p) = moved.hit.position {
                    let o = doc.origin;
                    doc.pointer = Some([o[0] + f64::from(p.x), o[1] - f64::from(p.z), o[2] + f64::from(p.y)]);
                }
            });
    }

    // The lines the world holds for this roof, to be drawn over the planes.
    doc.lines.clear();
    for path in &under {
        let Some(e) = doc.job.find(path) else { continue };
        let w = &doc.job.world;
        let (Some(kind), Some(run)) = (w.get::<Kind>(e).copied(), w.get::<plumbline_world::Run>(e)) else { continue };
        let Some([a, b]) = run.ends_ft() else { continue };
        // A line's height comes from a plane it lies on.
        let Some(plane) = run.planes.iter().find_map(|p| planes.iter().find(|s| s.path == *p)) else { continue };
        let flagged = w.get::<plumbline_world::Measure>(e).and_then(|m| m.0.get("true_length")).is_none_or(|d| d.value.is_none());
        let lift = Vec3::new(0.0, 0.04, 0.0);
        doc.lines.push((to_view(doc.origin, a, plane.height_at(a)) + lift, to_view(doc.origin, b, plane.height_at(b)) + lift, kind, flagged));
    }

    if let Ok(mut text) = sheet.single_mut() {
        text.0 = sheet_text(doc, &roof_path);
    }
}

fn mark(status: Status) -> &'static str {
    match status {
        Status::Known => " ",
        Status::Assumed => "~",
        Status::Unknown => "?",
    }
}

fn amount(v: Option<f64>, unit: &str) -> String {
    match v {
        Some(x) => {
            let s = format!("{x:.4}");
            format!("{} {unit}", s.trim_end_matches('0').trim_end_matches('.'))
        }
        None => "unknown".into(),
    }
}

/// The sheet: every plane and line of the roof on show, with its flag, and what they come to.
fn sheet_text(doc: &mut Doc, roof_path: &str) -> String {
    let mut out = String::new();
    let name = doc.job.find(roof_path).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone())).unwrap_or_default();
    out.push_str(&format!("{}   {roof_path}  {name}\n", doc.path.file_name().and_then(|f| f.to_str()).unwrap_or("")));
    out.push_str("~ Assumed   ? Unknown   (no mark) Known\n\n");
    let under: Vec<String> = doc.job.children().get(roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let elements = view::elements(&mut doc.job);
    let row = |el: &view::ElementRows, label: &str| el.rows.iter().find(|r| r.label == label).map(|r| (r.text.clone(), r.status));
    out.push_str("PLANES                      pitch          surface\n");
    for el in elements.iter().filter(|e| under.contains(&e.path) && e.kind.is_area()) {
        let (pitch, ps) = row(el, "pitch").unwrap_or(("unknown".into(), Status::Unknown));
        let (area, s) = row(el, "surface_area").unwrap_or(("unknown".into(), Status::Unknown));
        let condition = row(el, "condition").map(|c| c.0).unwrap_or_default();
        let here = if doc.selected.as_deref() == Some(el.path.as_str()) { ">" } else { " " };
        out.push_str(&format!("{here}{:<11} {:<10} {}{:<12} {}{:<15} {condition}\n", el.path, el.name, mark(ps), pitch, mark(s), area));
    }
    out.push_str("\nLINES                                      length\n");
    for el in elements.iter().filter(|e| under.contains(&e.path) && e.kind.is_run()) {
        let (len, s) = row(el, "true_length").unwrap_or(("unknown".into(), Status::Unknown));
        out.push_str(&format!(" {:<13} {:<16} {:<9} {}{}\n", el.path, el.name, format!("{:?}", el.kind), mark(s), len));
    }
    // What the solve had to say, and the roof's own flagged values, before any total.
    if let Some(el) = elements.iter().find(|e| e.path == roof_path) {
        for r in el.rows.iter().filter(|r| r.label == "solved roof") {
            out.push_str(&format!("\n{}{}: {}\n", mark(r.status), if r.text == "not solved" { "NOT SOLVED" } else { "note" }, r.note));
        }
        for r in el.rows.iter().filter(|r| r.status != Status::Known && r.label != "solved roof") {
            out.push_str(&format!("{}{} {}: {}\n", mark(r.status), roof_path, r.label, r.text));
        }
    }
    let totals = view::roof_totals(&mut doc.job, roof_path);
    out.push_str("\nTOTALS\n");
    let line = |label: &str, sum: &view::Sum| {
        let left_out = if sum.missing.is_empty() { String::new() } else { format!("   no total: {} of {} have no figure ({})", sum.missing.len(), sum.members, sum.missing.join(", ")) };
        format!(" {label:<22} {}{}{left_out}\n", mark(sum.status), amount(sum.value, sum.unit))
    };
    out.push_str(&line(&format!("surface, {} planes", totals.surface.members), &totals.surface));
    if let Some(sf) = totals.surface.value {
        out.push_str(&format!(" {:<22} {}{}\n", "", mark(totals.surface.status), amount(Some(sf / 100.0), "SQ")));
    }
    for (kind, sum) in &totals.lines {
        out.push_str(&line(&format!("{kind:?}, {}", sum.members), sum));
    }
    // The selected element's cost lines, worked out.
    if let Some(sel) = doc.selected.clone() {
        if let Some(el) = elements.iter().find(|e| e.path == sel) {
            let lines: Vec<&view::Row> = el.rows.iter().filter(|r| r.label.starts_with("quantity:") || r.label.starts_with("with waste:") || r.label.starts_with("to order:")).collect();
            if !lines.is_empty() {
                out.push_str(&format!("\nCOST LINES ON {sel}\n"));
                for r in lines {
                    out.push_str(&format!(" {}{:<44} {}\n", mark(r.status), r.label, r.text));
                }
            }
        }
    }
    out.push_str(&format!("\n{}\n", doc.message));
    out
}

fn place_camera(orbit: Res<Orbit>, mut camera: Query<&mut Transform, With<MainCamera>>) {
    let Ok(mut t) = camera.single_mut() else { return };
    let (sy, cy) = orbit.yaw.sin_cos();
    let (sp, cp) = orbit.pitch.sin_cos();
    let eye = orbit.target + Vec3::new(sy * cp, sp, cy * cp) * orbit.distance;
    *t = Transform::from_translation(eye).looking_at(orbit.target, Vec3::Y);
}

fn orbit_input(mut orbit: ResMut<Orbit>, buttons: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, scroll: Res<AccumulatedMouseScroll>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, doc: Res<Doc>) {
    if buttons.pressed(MouseButton::Right) {
        orbit.yaw -= motion.delta.x * 0.008;
        orbit.pitch = (orbit.pitch + motion.delta.y * 0.008).clamp(0.05, 1.5);
    }
    if scroll.delta.y != 0.0 {
        orbit.distance = (orbit.distance * (1.0 - scroll.delta.y * 0.1)).clamp(3.0, 2000.0);
    }
    if doc.entry.is_none() {
        let step = time.delta_secs() * 1.2;
        if keys.pressed(KeyCode::ArrowLeft) {
            orbit.yaw -= step;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            orbit.yaw += step;
        }
        if keys.pressed(KeyCode::ArrowUp) {
            orbit.pitch = (orbit.pitch + step).min(1.5);
        }
        if keys.pressed(KeyCode::ArrowDown) {
            orbit.pitch = (orbit.pitch - step).max(0.05);
        }
    }
}

fn draw_lines(doc: Res<Doc>, mut gizmos: Gizmos) {
    for (a, b, kind, flagged) in &doc.lines {
        let colour = if *flagged {
            Color::srgb(1.0, 0.25, 0.25)
        } else {
            match kind {
                Kind::Ridge => Color::srgb(0.95, 0.95, 0.98),
                Kind::Hip => Color::srgb(1.0, 0.62, 0.20),
                Kind::Valley => Color::srgb(0.35, 0.65, 1.0),
                Kind::Eave => Color::srgb(0.45, 0.85, 0.50),
                Kind::Rake => Color::srgb(0.80, 0.60, 0.95),
                Kind::PitchBreak => Color::srgb(0.95, 0.90, 0.35),
                _ => Color::srgb(0.7, 0.7, 0.7),
            }
        };
        gizmos.line(*a, *b, colour);
    }
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
                gone.push_str(&format!("\nLOST WITH THEM: the condition or cost lines on {}", lost.join(", ")));
            }
            for moved in &report.renumbered {
                gone.push_str(&format!("\n{moved}"));
            }
            // The roof under the pointer has moved; what the pointer read is no longer so.
            doc.pointer = None;
            format!("{what} on edge {edge} of {roof_path}; solved again: {} planes, {} lines{gone}", report.planes, report.lines)
        }
        Err(why) => format!("refused: {why}"),
    };
    doc.revision += 1;
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
fn keys(mut commands: Commands, mut doc: ResMut<Doc>, mut typed: MessageReader<KeyboardInput>, keys: Res<ButtonInput<KeyCode>>, primary: Query<Entity, With<PrimaryWindow>>, sheet: Query<Entity, With<SheetWindow>>) {
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
            None => doc.entry = Some((field, text)),
            Some(false) => doc.message = "entry cancelled".into(),
            Some(true) => match field {
                Entry::Pitch => change(doc, &format!("pitch {text}"), |_, _, edge| Ok(solve::Change::Rise { edge, rise: pitch_val(units::parse_pitch(&text)?) })),
                Entry::Move => change(doc, &format!("eave moved {text}"), |_, _, edge| {
                    let (len, unit) = units::parse_length(&text)?;
                    Ok(solve::Change::MoveEdge { edge, by_ft: len.feet(unit)?, status: Status::Assumed, source: SOURCE.into() })
                }),
            },
        }
        doc.revision += 1;
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
    if keys.just_pressed(KeyCode::KeyP) {
        doc.entry = Some((Entry::Pitch, String::new()));
        doc.revision += 1;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        doc.entry = Some((Entry::Move, String::new()));
        doc.revision += 1;
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
                doc.revision += 1;
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
                    None => format!("refused: the job has no condition that goes on a {}", kind.map(|k| format!("{k:?}")).unwrap_or_default()),
                    Some(name) => match doc.job.apply_as(&sel, name, Status::Assumed, SOURCE) {
                        Ok(lines) => {
                            doc.job.derive();
                            format!("{sel} is now {name:?}: {} cost lines", lines.len())
                        }
                        Err(why) => format!("refused: {why}"),
                    },
                }
            }
        };
        doc.revision += 1;
    }
    if keys.just_pressed(KeyCode::F2) {
        doc.message = match std::fs::write(&doc.path, file::save(&mut doc.job)) {
            Ok(()) => format!("saved {}", doc.path.display()),
            Err(e) => format!("could not save: {e}"),
        };
        doc.revision += 1;
    }
}

/// What the pointer and the selection say, over the 3D view.
fn write_hud(mut doc: ResMut<Doc>, mut hud: Query<&mut Text, With<HudText>>) {
    let Ok(mut text) = hud.single_mut() else { return };
    let doc = &mut *doc;
    let mut out = String::new();
    if let Some(p) = doc.pointer {
        out.push_str(&format!("pointer   east {}   north {}   height {}\n", units::feet_as_ft_in(p[0], 16), units::feet_as_ft_in(p[1], 16), units::feet_as_ft_in(p[2], 16)));
    } else {
        out.push_str("pointer   (move over the roof)\n");
    }
    if let Some(sel) = doc.selected.clone() {
        if let Some(e) = doc.job.find(&sel) {
            let w = &doc.job.world;
            let name = w.get::<plumbline_world::Node>(e).map(|n| n.name.clone()).unwrap_or_default();
            out.push_str(&format!("selected  {sel}  {name}\n"));
            if let Some(o) = w.get::<plumbline_world::Outline>(e) {
                if let Some(pts) = o.points_ft() {
                    // The model's own corners, to the sixteenth: these are 64-bit, not the picture's.
                    for p in pts.iter().take(6) {
                        out.push_str(&format!("  corner  east {:<14} north {}\n", units::feet_as_ft_in(p[0], 16), units::feet_as_ft_in(p[1], 16)));
                    }
                }
            }
        }
    }
    if let Some((field, typed)) = &doc.entry {
        out.push_str(&format!("\n{}: {typed}_   (Enter to set, Esc to cancel)\n", if *field == Entry::Pitch { "pitch, like 8:12" } else { "move the eave by, like 1' 6\" or -6 in" }));
    }
    out.push_str(&format!("\n{}", doc.message));
    if text.0 != out {
        text.0 = out;
    }
}
