//! The roof in space: its planes as surfaces, its lines drawn over them in the colour of
//! their kind, and enough of a building and a site under it that it reads as a roof.
//!
//! The walls, the fascia and the ground are there for the eye only. They are drawn from
//! the roof's own eaves and rakes down to the datum; they are not elements of the job and
//! nothing is measured off them.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ScalingMode, Viewport};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::mesh::Indices;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::PrimaryWindow;

use crate::model::{to_view, Doc, Snapshot};
use crate::theme;
use plumbline_world::value::Status;
use plumbline_world::Kind;

pub const BAR_H: f32 = 52.0;
pub const STATUS_H: f32 = 32.0;
pub const INSPECTOR_W: f32 = 348.0;
/// How far the fascia hangs below the roof's edge, in feet. For the eye only.
const FASCIA_FT: f32 = 0.7;
/// The 3D view's angle of view, top to bottom, in radians: 28 degrees.
const LENS: f32 = 0.488_692_2;

#[derive(Component)]
pub struct SceneMesh;
#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct GroundPlate;

#[derive(Resource)]
pub struct Orbit {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    /// Where F brings the view back to.
    pub home: (Vec3, f32),
}

/// The part of the main window the 3D view is drawn in, in logical pixels.
#[derive(Resource, Default)]
pub struct ViewRect(pub Rect);

/// What is drawn as lines every frame, in view coordinates.
#[derive(Resource, Default)]
pub struct Drawing {
    pub lines: Vec<(Vec3, Vec3, Color)>,
    /// Each plane's outline, by path: for the selection and for the pointer.
    pub outlines: Vec<(String, Vec<Vec3>)>,
    /// Each plane's surface material and the colour it is drawn in, by path: the plane
    /// under the pointer is drawn a little lighter.
    pub surfaces: Vec<(String, Handle<StandardMaterial>, Color)>,
    /// How far from the roof's middle the ground's grid reaches, in feet.
    pub reach: f32,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct RoofLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct GridLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct Highlight;

pub fn setup(mut commands: Commands, mut store: ResMut<GizmoConfigStore>) {
    commands.spawn((Camera3d::default(), Camera { order: 0, ..default() }, Msaa::Sample4, bevy::core_pipeline::tonemapping::Tonemapping::None, bevy::core_pipeline::tonemapping::DebandDither::Disabled, DistanceFog { color: theme::canvas(), falloff: FogFalloff::Linear { start: 4000.0, end: 9000.0 }, ..default() }, MainCamera, Transform::from_xyz(0.0, 40.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y)));
    // The sun, from the viewer's left and a little behind, forty degrees up: the slopes
    // that face it are bright, the ones that do not are a shade down, and the building
    // drops a soft shadow on the ground to its right. Lit the way a model is photographed.
    commands.spawn((
        DirectionalLight { illuminance: 1260.0, shadow_maps_enabled: true, ..default() },
        CascadeShadowConfigBuilder { first_cascade_far_bound: 160.0, maximum_distance: 1400.0, ..default() }.build(),
        Transform::from_xyz(-76.0, 64.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // Plenty of light from everywhere, so nothing in shade goes dark.
    commands.insert_resource(bevy::light::DirectionalLightShadowMap { size: 4096 });
    commands.insert_resource(GlobalAmbientLight { brightness: 1450.0, ..default() });

    let (lines, _) = store.config_mut::<RoofLines>();
    lines.line.width = 3.2;
    lines.depth_bias = -0.02;
    lines.render_layers = RenderLayers::layer(0);
    let (grid, _) = store.config_mut::<GridLines>();
    grid.line.width = 1.0;
    grid.render_layers = RenderLayers::layer(0);
    let (highlight, _) = store.config_mut::<Highlight>();
    highlight.line.width = 5.5;
    highlight.depth_bias = -0.05;
    highlight.render_layers = RenderLayers::layer(0);
}

/// Keep the 3D view inside the frame the panels leave it.
pub fn fit_viewport(window: Query<&Window, With<PrimaryWindow>>, mut camera: Query<&mut Camera, With<MainCamera>>, mut rect: ResMut<ViewRect>) {
    let (Ok(window), Ok(mut camera)) = (window.single(), camera.single_mut()) else { return };
    let (w, h) = (window.width(), window.height());
    let logical = Rect::new(0.0, BAR_H, (w - INSPECTOR_W).max(64.0), (h - STATUS_H).max(BAR_H + 64.0));
    if rect.0 != logical {
        rect.0 = logical;
    }
    let scale = window.scale_factor();
    let position = UVec2::new((logical.min.x * scale) as u32, (logical.min.y * scale) as u32);
    let size = UVec2::new((logical.width() * scale) as u32, (logical.height() * scale) as u32);
    if size.x == 0 || size.y == 0 || position.x + size.x > window.physical_width() || position.y + size.y > window.physical_height() {
        return;
    }
    let same = camera.viewport.as_ref().is_some_and(|v| v.physical_position == position && v.physical_size == size);
    if !same {
        camera.viewport = Some(Viewport { physical_position: position, physical_size: size, ..default() });
    }
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

/// A flat piece of surface. Its triangles are wound to face the way `normal` points, so
/// the side the light falls on is the side that is lit.
fn flat_mesh(positions: &[Vec3], mut indices: Vec<u32>, normal: Vec3) -> Mesh {
    for t in indices.chunks_mut(3) {
        let (a, b, c) = (positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]);
        if (b - a).cross(c - a).dot(normal) < 0.0 {
            t.swap(1, 2);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[normal.x, normal.y, normal.z]; positions.len()])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 0.0]; positions.len()])
        .with_inserted_indices(Indices::U32(indices))
}

fn surface(colour: Color, roughness: f32) -> StandardMaterial {
    StandardMaterial { base_color: colour, perceptual_roughness: roughness, reflectance: 0.18, double_sided: true, cull_mode: None, ..default() }
}

/// The colour a plane is drawn in: what it is covered with, when anything says.
fn plane_colour(condition: Option<&str>, status: Status, has_area: bool) -> Color {
    if !has_area {
        return theme::hex(0xE2A9A2);
    }
    let _ = status;
    match condition {
        None => theme::roof_bare(),
        Some(c) if c.to_lowercase().contains("copper") => theme::roof_copper(),
        Some(_) => theme::roof_other(),
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (a.to_srgba(), b.to_srgba());
    Color::srgb(a.red + (b.red - a.red) * t, a.green + (b.green - a.green) * t, a.blue + (b.blue - a.blue) * t)
}

/// Draw the roof again from the snapshot whenever it has changed.
#[allow(clippy::too_many_arguments)]
pub fn rebuild(
    mut commands: Commands,
    mut doc: ResMut<Doc>,
    snap: Res<Snapshot>,
    mut drawn: Local<u64>,
    mut orbit: ResMut<Orbit>,
    mut drawing: ResMut<Drawing>,
    old: Query<Entity, With<SceneMesh>>,
    plate: Query<Entity, With<GroundPlate>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *drawn == snap.revision {
        return;
    }
    *drawn = snap.revision;
    for e in &old {
        commands.entity(e).despawn();
    }
    let origin = doc.origin;

    // Frame the roof once, and lay the ground under it.
    if !doc.framed {
        if let Some((lo, hi, top)) = snap.extent {
            doc.framed = true;
            let (w, d) = ((hi[0] - lo[0]) as f32, (hi[1] - lo[1]) as f32);
            let size = (w * w + d * d).sqrt();
            orbit.distance = size * 2.6;
            orbit.target = Vec3::new(0.0, top as f32 * 0.42, 0.0);
            orbit.home = (orbit.target, orbit.distance);
            // The ground: one sheet out to the horizon, in a colour that comes out as the
            // canvas once the sun is on it, with a grid that fades with distance from the
            // roof. All that marks the ground is the grid and the shadow.
            drawing.reach = (size * 2.6 / 10.0).ceil() * 10.0;
            for e in &plate {
                commands.entity(e).despawn();
            }
            let ground = StandardMaterial { base_color: theme::ground(), perceptual_roughness: 1.0, reflectance: 0.0, ..default() };
            commands
                .spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(12000.0, 12000.0))), MeshMaterial3d(materials.add(ground)), Transform::from_xyz(0.0, -0.03, 0.0), GroundPlate))
                .observe(|click: On<PointerClick>, mut doc: ResMut<Doc>| {
                    // A click on the ground lets go of the selection.
                    if click.button == PointerButton::Primary && doc.selected.is_some() && doc.entry.is_none() {
                        doc.selected = None;
                        doc.message = "nothing selected".into();
                        doc.touch();
                    }
                });
        }
    }

    drawing.lines.clear();
    drawing.outlines.clear();
    drawing.surfaces.clear();
    let wall = materials.add(surface(theme::wall(), 0.95));
    let fascia = materials.add(surface(theme::fascia(), 0.8));

    for info in &snap.planes {
        let Some(plane) = &info.solid else { continue };
        let positions: Vec<Vec3> = plane.points.iter().map(|p| to_view(origin, [p[0], p[1], plane.height_at(*p)])).collect();
        let tris = triangles(&plane.points);
        let Some(first) = tris.first() else { continue };
        let (a, b, c) = (positions[first[0] as usize], positions[first[1] as usize], positions[first[2] as usize]);
        let mut normal = (b - a).cross(c - a).normalize_or_zero();
        if normal.y < 0.0 {
            normal = -normal;
        }
        let selected = doc.selected.as_deref() == Some(info.path.as_str());
        let base = plane_colour(info.condition.as_deref(), info.worst(), info.area.is_some());
        let colour = if selected { mix(base, Color::WHITE, 0.14) } else { base };
        let path = info.path.clone();
        let (over, out) = (path.clone(), path.clone());
        let material = materials.add(surface(colour, 0.82));
        drawing.surfaces.push((info.path.clone(), material.clone(), colour));
        commands
            .spawn((Mesh3d(meshes.add(flat_mesh(&positions, tris.into_iter().flatten().collect(), normal))), MeshMaterial3d(material), SceneMesh))
            .observe(move |click: On<PointerClick>, mut doc: ResMut<Doc>| {
                if click.button != PointerButton::Primary || doc.entry.is_some() {
                    return;
                }
                doc.selected = Some(path.clone());
                doc.message = format!("selected {path}");
                doc.touch();
            })
            .observe(|moved: On<PointerMove>, mut doc: ResMut<Doc>| {
                if let Some(p) = moved.hit.position {
                    let o = doc.origin;
                    doc.pointer = Some([o[0] + f64::from(p.x), o[1] - f64::from(p.z), o[2] + f64::from(p.y)]);
                }
            })
            .observe(move |_: On<PointerOver>, mut doc: ResMut<Doc>| doc.hover = Some(over.clone()))
            .observe(move |_: On<PointerOut>, mut doc: ResMut<Doc>| {
                if doc.hover.as_deref() == Some(out.as_str()) {
                    doc.hover = None;
                    doc.pointer = None;
                }
            });
        drawing.outlines.push((info.path.clone(), positions.iter().map(|p| *p + normal * 0.06).collect()));
    }

    for line in &snap.lines {
        let Some([a, b]) = line.ends else { continue };
        let (a, b) = (to_view(origin, a), to_view(origin, b));
        let lift = Vec3::new(0.0, 0.05, 0.0);
        let colour = if line.length.is_none() { theme::red() } else { theme::line_colour(line.kind) };
        drawing.lines.push((a + lift, b + lift, colour));
        // Under each outer edge of the roof: a fascia, and a wall down to the datum.
        if matches!(line.kind, Kind::Eave | Kind::Rake) && a.y > FASCIA_FT + 0.5 && b.y > FASCIA_FT + 0.5 {
            let along = (b - a).with_y(0.0);
            let mut normal = along.cross(Vec3::Y).normalize_or_zero();
            // Outward is away from the plane whose edge this is.
            let inside = line.planes.iter().find_map(|p| snap.planes.iter().find(|x| x.path == *p)).and_then(|p| p.solid.as_ref()).map(|solid| {
                let n = solid.points.len().max(1) as f64;
                let c = [solid.points.iter().map(|q| q[0]).sum::<f64>() / n, solid.points.iter().map(|q| q[1]).sum::<f64>() / n];
                to_view(origin, [c[0], c[1], 0.0])
            });
            if inside.is_some_and(|c| normal.dot(((a + b) / 2.0 - c).with_y(0.0)) < 0.0) {
                normal = -normal;
            }
            let (fa, fb) = (a - Vec3::Y * FASCIA_FT, b - Vec3::Y * FASCIA_FT);
            let band = flat_mesh(&[a, b, fb, fa], vec![0, 1, 2, 0, 2, 3], normal);
            commands.spawn((Mesh3d(meshes.add(band)), MeshMaterial3d(fascia.clone()), SceneMesh));
            let face = flat_mesh(&[fa, fb, fb.with_y(0.0), fa.with_y(0.0)], vec![0, 1, 2, 0, 2, 3], normal);
            commands.spawn((Mesh3d(meshes.add(face)), MeshMaterial3d(wall.clone()), SceneMesh));
        }
    }
}

pub fn place_camera(orbit: Res<Orbit>, doc: Res<Doc>, mut camera: Query<(&mut Transform, &mut Projection, &mut DistanceFog), With<MainCamera>>) {
    let Ok((mut t, mut projection, mut fog)) = camera.single_mut() else { return };
    // The ground fades into the canvas far off; on plan nothing is far off.
    let (start, end) = if doc.plan { (50_000.0, 90_000.0) } else { (orbit.distance * 2.2, orbit.distance * 7.0) };
    if !matches!(fog.falloff, FogFalloff::Linear { start: s, end: e } if (s - start).abs() < 0.5 && (e - end).abs() < 0.5) {
        fog.falloff = FogFalloff::Linear { start, end };
    }
    if doc.plan {
        // Straight down with north up, drawn without perspective: a roof plan.
        *t = Transform::from_translation(orbit.target.with_y(orbit.home.0.y * 2.4 + 60.0)).looking_at(orbit.target.with_y(0.0), Vec3::NEG_Z);
        let height = orbit.distance * 0.455;
        let same = matches!(&*projection, Projection::Orthographic(o) if matches!(o.scaling_mode, ScalingMode::FixedVertical { viewport_height } if (viewport_height - height).abs() < 1e-3));
        if !same {
            *projection = Projection::from(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: height }, far: 2000.0, ..OrthographicProjection::default_3d() });
        }
    } else {
        let (sy, cy) = orbit.yaw.sin_cos();
        let (sp, cp) = orbit.pitch.sin_cos();
        let eye = orbit.target + Vec3::new(sy * cp, sp, cy * cp) * orbit.distance;
        *t = Transform::from_translation(eye).looking_at(orbit.target, Vec3::Y);
        // A long lens: the roof keeps its proportions instead of bulging toward the eye.
        if !matches!(&*projection, Projection::Perspective(p) if (p.fov - LENS).abs() < 1e-4) {
            *projection = Projection::from(PerspectiveProjection { fov: LENS, far: 20_000.0, ..default() });
        }
    }
}

/// Orbit with the right button, pan with the middle one, zoom with the wheel.
#[allow(clippy::too_many_arguments)]
pub fn orbit_input(mut orbit: ResMut<Orbit>, buttons: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, scroll: Res<AccumulatedMouseScroll>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, doc: Res<Doc>, window: Query<&Window, With<PrimaryWindow>>, rect: Res<ViewRect>) {
    let inside = window.single().ok().and_then(|w| w.cursor_position()).is_some_and(|p| rect.0.contains(p));
    if buttons.pressed(MouseButton::Right) && !doc.plan {
        orbit.yaw -= motion.delta.x * 0.008;
        orbit.pitch = (orbit.pitch + motion.delta.y * 0.008).clamp(0.08, 1.5);
    }
    if buttons.pressed(MouseButton::Middle) || (buttons.pressed(MouseButton::Right) && doc.plan) {
        // Drag the ground under the pointer: a pixel moves the target by what a pixel covers.
        let per_px = orbit.distance * if doc.plan { 0.455 } else { 0.5 } / rect.0.height().max(1.0);
        let (sy, cy) = if doc.plan { (0.0, 1.0) } else { orbit.yaw.sin_cos() };
        let right = Vec3::new(cy, 0.0, -sy);
        let forward = Vec3::new(sy, 0.0, cy);
        let shift = right * (-motion.delta.x * per_px) + forward * (-motion.delta.y * per_px);
        orbit.target += shift;
    }
    if inside && scroll.delta.y != 0.0 {
        let steps = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y / 40.0 } else { scroll.delta.y };
        orbit.distance = (orbit.distance * (1.0 - steps * 0.1)).clamp(6.0, 4000.0);
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
            orbit.pitch = (orbit.pitch - step).max(0.08);
        }
    }
}

/// The ground's grid, the roof's lines, and the outline of what is selected or pointed at.
pub fn draw_lines(doc: Res<Doc>, drawing: Res<Drawing>, mut lines: Gizmos<RoofLines>, mut grid: Gizmos<GridLines>, mut highlight: Gizmos<Highlight>) {
    // The grid: a line every ten feet, a stronger one every fifty, fading out with
    // distance from the roof so the ground has no edge.
    let reach = drawing.reach;
    if reach > 0.0 {
        let fade = |x: f32, z: f32| (1.0 - (x * x + z * z).sqrt() / reach).clamp(0.0, 1.0).powf(1.4);
        let steps = (reach / 10.0) as i32;
        for i in -steps..=steps {
            let at = i as f32 * 10.0;
            let base = if i % 5 == 0 { theme::grid_major() } else { theme::grid_minor() };
            for j in -steps..steps {
                let (from, to) = (j as f32 * 10.0, (j + 1) as f32 * 10.0);
                let (a0, a1) = (fade(at, from), fade(at, to));
                if a0 <= 0.0 && a1 <= 0.0 {
                    continue;
                }
                let (c0, c1) = (base.with_alpha(base.alpha() * a0), base.with_alpha(base.alpha() * a1));
                grid.line_gradient(Vec3::new(at, 0.0, from), Vec3::new(at, 0.0, to), c0, c1);
                grid.line_gradient(Vec3::new(from, 0.0, at), Vec3::new(to, 0.0, at), c0, c1);
            }
        }
    }
    for (a, b, colour) in &drawing.lines {
        lines.line(*a, *b, *colour);
    }
    let mut ring = |path: &str, colour: Color| {
        if let Some((_, points)) = drawing.outlines.iter().find(|(p, _)| p == path) {
            for i in 0..points.len() {
                highlight.line(points[i], points[(i + 1) % points.len()], colour);
            }
        }
    };
    if let Some(selected) = &doc.selected {
        ring(selected, theme::accent());
    }
}

/// The plane under the pointer is drawn a little lighter, so it is plain what a click
/// would pick. Its lines keep their colours.
pub fn tint_hover(doc: Res<Doc>, drawing: Res<Drawing>, mut materials: ResMut<Assets<StandardMaterial>>, mut shown: Local<(Option<String>, u64)>) {
    if shown.0 == doc.hover && shown.1 == doc.revision {
        return;
    }
    *shown = (doc.hover.clone(), doc.revision);
    for (path, handle, colour) in &drawing.surfaces {
        let wanted = if doc.hover.as_deref() == Some(path.as_str()) { mix(*colour, Color::WHITE, 0.16) } else { *colour };
        if let Some(mut material) = materials.get_mut(handle) {
            if material.base_color != wanted {
                material.base_color = wanted;
            }
        }
    }
}
