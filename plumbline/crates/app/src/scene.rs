//! The building in space, drawn the way a game draws a world: a sky, a sun that casts
//! shadows, surfaces that take light, and a camera that eases instead of jumping.
//!
//! Everything drawn is an element of the job, placed by `plumbline-world`: walls with their
//! openings cut, floors, the roof's planes and lines, ducts, pipes, conduit and cable at
//! their own size along their own centre lines, equipment and fixtures. A click on any of
//! it selects that element. The ground and the sky are the only things here that are not
//! in the job; on a job with no walls a fascia and a wall are still drawn under the roof's
//! edge, for the eye only.

use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Exposure, ScalingMode};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{light_consts::lux, Atmosphere, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, DirectionalLightShadowMap, NotShadowCaster, SunDisk};
use bevy::mesh::Indices;
use bevy::pbr::{AtmosphereSettings, ScreenSpaceAmbientOcclusion};
use bevy::picking::pointer::PointerButton;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::PrimaryWindow;

use crate::model::{to_view, Doc, Snapshot, Tone};
use crate::theme;
use plumbline_world::building::Shape;
use plumbline_world::{Discipline, Kind};

pub const BAR_H: f32 = 48.0;
pub const STATUS_H: f32 = 32.0;
pub const GAP: f32 = 12.0;
pub const BROWSER_W: f32 = 252.0;
pub const INSPECTOR_W: f32 = 348.0;
/// How far the fascia hangs below the roof's edge, in feet. For the eye only.
const FASCIA_FT: f32 = 0.7;
/// The 3D view's angle of view, top to bottom, in radians: 40 degrees.
const LENS: f32 = 0.698_131_7;
/// The angle of view on foot: 68 degrees.
const WALK_LENS: f32 = 1.186_823_9;
/// Eye height on foot, in feet.
pub const EYE_FT: f32 = 5.5;
/// A run thinner than this is also drawn as a line, so it can be seen from far off.
const THIN_FT: f32 = 0.17;

#[derive(Component)]
pub struct SceneMesh;
#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct Ground;

/// Whether the full picture is drawn (sky, ambient occlusion, bloom) or the plain one. The
/// plain one is for a machine with no graphics card; ask for it with PLUMBLINE_GRAPHICS=plain.
#[derive(Resource, Clone, Copy)]
pub struct Graphics {
    pub full: bool,
}

/// Where the camera is going, and where it is now. Every frame it moves part of the way,
/// so a change of view is a movement, not a cut.
#[derive(Resource)]
pub struct Orbit {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub now: (Vec3, f32, f32, f32),
    /// Where F brings the view back to.
    pub home: (Vec3, f32),
}

pub const HOME_YAW: f32 = -0.66;
pub const HOME_PITCH: f32 = 0.27;

impl Orbit {
    pub fn new() -> Orbit {
        Orbit { target: Vec3::ZERO, yaw: HOME_YAW, pitch: HOME_PITCH, distance: 120.0, now: (Vec3::ZERO, HOME_YAW, HOME_PITCH, 120.0), home: (Vec3::ZERO, 120.0) }
    }
    pub fn go_home(&mut self) {
        self.target = self.home.0;
        self.distance = self.home.1;
        self.yaw = HOME_YAW;
        self.pitch = HOME_PITCH;
    }
}

/// One surface of one element, so it can be lit up when it is pointed at or selected.
pub struct Surface {
    pub path: String,
    pub system: Option<String>,
    pub handle: Handle<StandardMaterial>,
    pub colour: Color,
}

/// What is drawn as lines every frame, in view coordinates.
#[derive(Resource, Default)]
pub struct Drawing {
    /// The roof's lines: ends, colour, and whether it is drawn broken (no length).
    pub lines: Vec<(Vec3, Vec3, Color, bool)>,
    /// The centre lines of thin runs, and of runs with no size (drawn broken).
    pub runs: Vec<(Vec3, Vec3, Color, bool)>,
    /// Each element's outline, by path: drawn when it is selected.
    pub frames: Vec<(String, Vec<(Vec3, Vec3)>)>,
    pub surfaces: Vec<Surface>,
    /// The outlines of what is drawn as glass, by path, so the building keeps its shape.
    pub ghosts: Vec<String>,
    /// The fittings, by path. When a system is picked out its runs are drawn over in its
    /// colour; an elbow's twelve edges drawn over at that size are a blot, so they are not.
    pub fittings: Vec<String>,
    /// How far from the middle the ground's grid reaches, in feet.
    pub reach: f32,
}

/// The few solids everything round or square is made of.
#[derive(Resource)]
pub struct Solids {
    cube: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    ball: Handle<Mesh>,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct RoofLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrokenLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct RunLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct GridLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct Highlight;

pub fn setup(mut commands: Commands, mut store: ResMut<GizmoConfigStore>, mut meshes: ResMut<Assets<Mesh>>, mut mediums: ResMut<Assets<ScatteringMedium>>, graphics: Res<Graphics>) {
    commands.insert_resource(Solids { cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)), cylinder: meshes.add(Cylinder::new(0.5, 1.0)), ball: meshes.add(Sphere::new(0.5).mesh().ico(3).expect("a sphere")) });
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
    let cascades = CascadeShadowConfigBuilder { first_cascade_far_bound: 180.0, maximum_distance: 1600.0, ..default() }.build();
    // The sun stands to the viewer's left, forty degrees up.
    let sun_at = Transform::from_xyz(-76.0, 64.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y);
    let camera_at = Transform::from_xyz(0.0, 40.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y);
    if graphics.full {
        // A real sky: the sun's light is scattered by an atmosphere, which also lights
        // whatever the sun does not reach. Ambient occlusion darkens where surfaces meet,
        // and the picture is steadied over several frames instead of being left jagged.
        let medium = mediums.add(ScatteringMedium::earth(256, 256));
        commands.spawn(Atmosphere::earth(medium));
        commands.spawn((
            Camera3d::default(),
            Camera { order: 0, ..default() },
            AtmosphereSettings::default(),
            AtmosphereEnvironmentMapLight::default(),
            Exposure { ev100: 13.9 },
            Tonemapping::AcesFitted,
            Bloom::NATURAL,
            Msaa::Off,
            TemporalAntiAliasing::default(),
            ScreenSpaceAmbientOcclusion::default(),
            IsDefaultUiCamera,
            MainCamera,
            camera_at,
        ));
        commands.spawn((DirectionalLight { illuminance: lux::RAW_SUNLIGHT, shadow_maps_enabled: true, ..default() }, SunDisk::EARTH, cascades, sun_at));
        // A little light from everywhere as well, so a room indoors and the ground in
        // shadow are not left to the blue of the sky alone.
        commands.insert_resource(GlobalAmbientLight { brightness: 12000.0, ..default() });
    } else {
        commands.spawn((
            Camera3d::default(),
            Camera { order: 0, ..default() },
            Msaa::Sample4,
            Tonemapping::None,
            DebandDither::Disabled,
            DistanceFog { color: theme::canvas(), falloff: FogFalloff::Linear { start: 4000.0, end: 9000.0 }, ..default() },
            IsDefaultUiCamera,
            MainCamera,
            camera_at,
        ));
        commands.spawn((DirectionalLight { illuminance: 1260.0, shadow_maps_enabled: true, ..default() }, cascades, sun_at));
        commands.insert_resource(GlobalAmbientLight { brightness: 1450.0, ..default() });
    }

    let (lines, _) = store.config_mut::<RoofLines>();
    lines.line.width = 3.0;
    lines.depth_bias = -0.004;
    // The centre lines of thin runs sit exactly where the run is: behind a wall they are hidden.
    let (runs, _) = store.config_mut::<RunLines>();
    runs.line.width = 2.0;
    runs.render_layers = RenderLayers::layer(0);
    let (lines, _) = store.config_mut::<RoofLines>();
    lines.render_layers = RenderLayers::layer(0);
    let (broken, _) = store.config_mut::<BrokenLines>();
    broken.line.width = 3.0;
    broken.line.style = GizmoLineStyle::Dashed { gap_scale: 2.5, line_scale: 3.0 };
    broken.depth_bias = -0.004;
    broken.render_layers = RenderLayers::layer(0);
    let (grid, _) = store.config_mut::<GridLines>();
    grid.line.width = 1.0;
    grid.render_layers = RenderLayers::layer(0);
    let (highlight, _) = store.config_mut::<Highlight>();
    highlight.line.width = 4.5;
    highlight.depth_bias = -0.2;
    highlight.render_layers = RenderLayers::layer(0);
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

/// Surfaces, each flat and each lit on the side its normal points to.
struct Faces {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl Faces {
    fn new() -> Faces {
        Faces { positions: Vec::new(), normals: Vec::new(), indices: Vec::new() }
    }
    fn add(&mut self, points: &[Vec3], tris: &[[u32; 3]], normal: Vec3) {
        let base = self.positions.len() as u32;
        self.positions.extend(points.iter().map(|p| [p.x, p.y, p.z]));
        self.normals.extend(points.iter().map(|_| [normal.x, normal.y, normal.z]));
        for t in tris {
            let (a, b, c) = (points[t[0] as usize], points[t[1] as usize], points[t[2] as usize]);
            let wound = if (b - a).cross(c - a).dot(normal) < 0.0 { [t[0], t[2], t[1]] } else { *t };
            self.indices.extend(wound.iter().map(|i| base + i));
        }
    }
    fn quad(&mut self, corners: [Vec3; 4], normal: Vec3) {
        self.add(&corners, &[[0, 1, 2], [0, 2, 3]], normal);
    }
    fn mesh(self) -> Mesh {
        let n = self.positions.len();
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 0.0]; n])
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (a.to_srgba(), b.to_srgba());
    Color::srgba(a.red + (b.red - a.red) * t, a.green + (b.green - a.green) * t, a.blue + (b.blue - a.blue) * t, a.alpha)
}

/// The twelve edges of a box, given its eight corners (bottom four, then top four).
fn box_edges(c: [Vec3; 8]) -> Vec<(Vec3, Vec3)> {
    let mut out = Vec::new();
    for i in 0..4 {
        out.push((c[i], c[(i + 1) % 4]));
        out.push((c[i + 4], c[(i + 1) % 4 + 4]));
        out.push((c[i], c[i + 4]));
    }
    out
}

/// Spawns one piece of one element and remembers it, so a click on it selects the element
/// and the element can be lit up.
struct Builder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    materials: &'a mut Assets<StandardMaterial>,
    drawing: &'a mut Drawing,
    ghost: bool,
}

impl Builder<'_, '_, '_> {
    /// `ghosted` says whether this piece turns to glass when the building is ghosted.
    #[allow(clippy::too_many_arguments)]
    fn piece(&mut self, path: &str, system: Option<&String>, mesh: Handle<Mesh>, at: Transform, colour: Color, roughness: f32, metallic: f32, ghosted: bool) {
        let see_through = ghosted && self.ghost;
        let colour = if see_through { colour.with_alpha(colour.alpha().min(0.06)) } else { colour };
        let blend = colour.alpha() < 0.999;
        let handle = self.materials.add(StandardMaterial {
            base_color: colour,
            perceptual_roughness: roughness,
            metallic,
            reflectance: 0.3,
            double_sided: true,
            cull_mode: None,
            alpha_mode: if blend { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        });
        self.drawing.surfaces.push(Surface { path: path.to_string(), system: system.cloned(), handle: handle.clone(), colour });
        if see_through && !self.drawing.ghosts.iter().any(|g| g == path) {
            self.drawing.ghosts.push(path.to_string());
        }
        let mut e = self.commands.spawn((Mesh3d(mesh), MeshMaterial3d(handle), at, SceneMesh));
        if blend {
            // What is drawn as glass does not darken what is inside it.
            e.insert(NotShadowCaster);
        }
        if see_through {
            // And a click goes through it, to what runs inside.
            e.insert(Pickable::IGNORE);
            return;
        }
        let (click, over, out) = (path.to_string(), path.to_string(), path.to_string());
        e.observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
            // With a drawing tool in hand a click is a pick, not a selection.
            if press.button == PointerButton::Primary && doc.entry.is_none() && doc.draw.tool.is_none() {
                doc.select(&click);
            }
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
    }
}

/// Draw the building again from the snapshot whenever what is drawn has changed.
#[allow(clippy::too_many_arguments)]
pub fn rebuild(
    mut commands: Commands,
    mut doc: ResMut<Doc>,
    snap: Res<Snapshot>,
    mut drawn: Local<u64>,
    mut orbit: ResMut<Orbit>,
    mut drawing: ResMut<Drawing>,
    solids: Res<Solids>,
    graphics: Res<Graphics>,
    old: Query<Entity, With<SceneMesh>>,
    ground: Query<Entity, With<Ground>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *drawn == doc.geometry || snap.revision != doc.revision {
        return;
    }
    *drawn = doc.geometry;
    for e in &old {
        commands.entity(e).despawn();
    }
    let origin = doc.origin;
    let show = doc.show.clone();

    // Frame the building once, and lay the ground under it.
    if !doc.framed {
        // A job with nothing in it yet is framed as a plot to draw on: sixty feet by
        // forty about the origin, so there is ground and a grid before the first wall.
        if let Some((lo, hi, top)) = snap.extent.or(Some(([-30.0, -20.0], [30.0, 20.0], 10.0))) {
            doc.framed = true;
            let (w, d) = ((hi[0] - lo[0]) as f32, (hi[1] - lo[1]) as f32);
            let size = (w * w + d * d + (top as f32).powi(2)).sqrt();
            orbit.distance = size * 1.6;
            orbit.target = Vec3::new(0.0, top as f32 * 0.42, 0.0);
            orbit.home = (orbit.target, orbit.distance);
            orbit.now = (orbit.target, orbit.yaw, orbit.pitch, orbit.distance * 1.25);
            drawing.reach = (size * 2.4 / 10.0).ceil() * 10.0;
            for e in &ground {
                commands.entity(e).despawn();
            }
            let colour = if graphics.full { theme::grass() } else { theme::ground() };
            let material = StandardMaterial { base_color: colour, perceptual_roughness: 1.0, reflectance: 0.0, ..default() };
            commands
                .spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(24000.0, 24000.0))), MeshMaterial3d(materials.add(material)), Transform::from_xyz(0.0, -0.04, 0.0), Ground))
                .observe(|press: On<PointerClick>, mut doc: ResMut<Doc>| {
                    // A click on the ground lets go of the selection.
                    if press.button == PointerButton::Primary && doc.selected.is_some() && doc.entry.is_none() && doc.draw.tool.is_none() {
                        doc.selected = None;
                        doc.say(Tone::Done, "nothing selected");
                        doc.touch();
                    }
                });
        }
    }

    drawing.lines.clear();
    drawing.runs.clear();
    drawing.ghosts.clear();
    drawing.fittings.clear();
    drawing.frames.clear();
    drawing.surfaces.clear();
    let drawing = &mut *drawing;
    let mut build = Builder { commands: &mut commands, materials: &mut materials, drawing, ghost: show.ghost };
    let has_walls = snap.drawn.iter().any(|p| p.kind == Kind::Wall);

    // ---- The rest of the building ----
    for p in &snap.drawn {
        if !show.discipline(p.discipline) || show.storey.as_ref().is_some_and(|s| p.storey.as_ref() != Some(s)) {
            continue;
        }
        let colour = theme::system_colour(snap.classification(p.system.as_ref()));
        match &p.shape {
            Shape::Wall { a, b, base, height, thickness, holes } => {
                let (va, vb) = (to_view(origin, [a[0], a[1], *base]), to_view(origin, [b[0], b[1], *base]));
                let length = va.distance(vb);
                let along = (vb - va) / length.max(1e-6);
                let turn = Quat::from_rotation_y((b[1] - a[1]).atan2(b[0] - a[0]) as f32);
                let (h, t) = (*height as f32, *thickness as f32);
                let paint = if t >= 0.45 { theme::wall_outside() } else { theme::wall_inside() };
                // A solid piece of wall from `x0` to `x1` along it and `y0` to `y1` up it.
                let slab = |build: &mut Builder, x0: f32, x1: f32, y0: f32, y1: f32| {
                    if x1 - x0 < 0.005 || y1 - y0 < 0.005 {
                        return;
                    }
                    let centre = va + along * ((x0 + x1) / 2.0) + Vec3::Y * ((y0 + y1) / 2.0);
                    build.piece(&p.path, None, solids.cube.clone(), Transform { translation: centre, rotation: turn, scale: Vec3::new(x1 - x0, y1 - y0, t) }, paint, 0.92, 0.0, true);
                };
                let mut cuts: Vec<&(String, Kind, [f64; 4])> = holes.iter().collect();
                cuts.sort_by(|x, y| x.2[0].total_cmp(&y.2[0]));
                let mut at = 0.0f32;
                for (hole_path, kind, [from, to, sill, head]) in cuts {
                    let (from, to, sill, head) = (*from as f32, *to as f32, *sill as f32, *head as f32);
                    slab(&mut build, at, from, 0.0, h);
                    slab(&mut build, from, to, 0.0, sill);
                    slab(&mut build, from, to, head, h);
                    // What fills the opening: glass in a window, a leaf in a door.
                    let centre = va + along * ((from + to) / 2.0) + Vec3::Y * ((sill + head) / 2.0);
                    let (fill, thick, rough) = if *kind == Kind::Window { (theme::glass(), 0.08, 0.08) } else { (theme::door_leaf(), 0.14, 0.6) };
                    build.piece(hole_path, None, solids.cube.clone(), Transform { translation: centre, rotation: turn, scale: Vec3::new(to - from, head - sill, thick) }, fill, rough, 0.0, true);
                    let half = turn * Vec3::Z * (t / 2.0);
                    let (c0, c1) = (va + along * from + Vec3::Y * sill, va + along * to + Vec3::Y * sill);
                    let up = Vec3::Y * (head - sill);
                    build.drawing.frames.push((hole_path.clone(), box_edges([c0 - half, c1 - half, c1 + half, c0 + half, c0 - half + up, c1 - half + up, c1 + half + up, c0 + half + up])));
                    at = at.max(to);
                }
                slab(&mut build, at, length, 0.0, h);
                let half = turn * Vec3::Z * (t / 2.0);
                let up = Vec3::Y * h;
                build.drawing.frames.push((p.path.clone(), box_edges([va - half, vb - half, vb + half, va + half, va - half + up, vb - half + up, vb + half + up, va + half + up])));
            }
            Shape::Floor { boundary, top, thickness } => {
                let tris = triangles(boundary);
                let high: Vec<Vec3> = boundary.iter().map(|q| to_view(origin, [q[0], q[1], *top])).collect();
                let low: Vec<Vec3> = high.iter().map(|q| *q - Vec3::Y * *thickness as f32).collect();
                let mut faces = Faces::new();
                faces.add(&high, &tris, Vec3::Y);
                faces.add(&low, &tris, Vec3::NEG_Y);
                let middle = high.iter().copied().sum::<Vec3>() / high.len().max(1) as f32;
                let mut frame = Vec::new();
                for i in 0..high.len() {
                    let j = (i + 1) % high.len();
                    let mut normal = (high[j] - high[i]).cross(Vec3::Y).normalize_or_zero();
                    if normal.dot(((high[i] + high[j]) / 2.0 - middle).with_y(0.0)) < 0.0 {
                        normal = -normal;
                    }
                    faces.quad([high[i], high[j], low[j], low[i]], normal);
                    frame.push((high[i], high[j]));
                }
                build.piece(&p.path, None, meshes.add(faces.mesh()), Transform::IDENTITY, theme::floor_colour(), 0.75, 0.0, true);
                build.drawing.frames.push((p.path.clone(), frame));
            }
            Shape::Run { a, b, round, rect } => {
                // One straight piece. Where it meets the next, the fitting there closes the corner.
                let pts: Vec<Vec3> = [a, b].iter().map(|q| to_view(origin, **q)).collect();
                let frame: Vec<(Vec3, Vec3)> = pts.windows(2).map(|s| (s[0], s[1])).collect();
                let metal = if matches!(p.kind, Kind::Duct | Kind::Conduit) { 0.55 } else { 0.15 };
                match (round, rect) {
                    (Some(d), _) => {
                        let d = *d as f32;
                        for s in pts.windows(2) {
                            let turn = Quat::from_rotation_arc(Vec3::Y, (s[1] - s[0]).normalize_or(Vec3::Y));
                            build.piece(&p.path, p.system.as_ref(), solids.cylinder.clone(), Transform { translation: (s[0] + s[1]) / 2.0, rotation: turn, scale: Vec3::new(d, s[0].distance(s[1]), d) }, colour, 0.45, metal, false);
                        }
                        if d < THIN_FT {
                            build.drawing.runs.extend(frame.iter().map(|(a, b)| (*a, *b, colour, false)));
                        }
                    }
                    (None, Some((w, d))) => {
                        let (w, d) = (*w as f32, *d as f32);
                        for s in pts.windows(2) {
                            let dir = (s[1] - s[0]).normalize_or(Vec3::X);
                            // Lying flat, the depth is up and down; standing, the width runs east-west.
                            let side = if dir.y.abs() > 0.99 { Vec3::X } else { dir.cross(Vec3::Y).normalize() };
                            // The three have to be a right-handed set (side × upward = dir)
                            // or they are a mirror and not a turn: a riser drawn with a
                            // mirror for its turn is not drawn where it stands.
                            let upward = dir.cross(side).normalize();
                            let turn = Quat::from_mat3(&Mat3::from_cols(side, upward, dir));
                            let (from, to) = (s[0], s[1]);
                            build.piece(&p.path, p.system.as_ref(), solids.cube.clone(), Transform { translation: (from + to) / 2.0, rotation: turn, scale: Vec3::new(w, d, from.distance(to)) }, colour, 0.5, metal, false);
                        }
                    }
                    // No size: there is nothing to draw it as but its centre line, broken.
                    (None, None) => build.drawing.runs.extend(frame.iter().map(|(a, b)| (*a, *b, theme::red(), true))),
                }
                build.drawing.frames.push((p.path.clone(), frame));
            }
            Shape::Box { at, size, angle } => {
                let centre = to_view(origin, [at[0], at[1], at[2] + size[2] / 2.0]);
                let scale = Vec3::new(size[0] as f32, size[2] as f32, size[1] as f32);
                // Turned counter-clockwise from east, seen from above: about the upright axis.
                let turn = Quat::from_rotation_y((*angle as f32).to_radians());
                if p.kind == Kind::Fitting {
                    build.drawing.fittings.push(p.path.clone());
                    // A fitting is drawn in its system's colour, like the runs it joins: a
                    // ball where its section is round, a block where it is not.
                    let metal = if p.discipline == Discipline::Plumbing { 0.15 } else { 0.55 };
                    let solid = if size[0] == size[2] { solids.ball.clone() } else { solids.cube.clone() };
                    build.piece(&p.path, p.system.as_ref(), solid, Transform { translation: centre, rotation: turn, scale }, colour, 0.45, metal, false);
                } else {
                    let (paint, rough, metal) = match (p.kind, p.discipline) {
                        (Kind::Equipment, _) => (mix(theme::equipment(), colour, 0.18), 0.4, 0.6),
                        (_, Discipline::Plumbing) => (theme::porcelain(), 0.2, 0.0),
                        _ => (mix(theme::porcelain(), colour, 0.35), 0.5, 0.1),
                    };
                    build.piece(&p.path, p.system.as_ref(), solids.cube.clone(), Transform { translation: centre, rotation: turn, scale }, paint, rough, metal, false);
                }
                let h = scale / 2.0;
                let corner = |x: f32, y: f32, z: f32| centre + turn * Vec3::new(x * h.x, y * h.y, z * h.z);
                build.drawing.frames.push((
                    p.path.clone(),
                    box_edges([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)]),
                ));
            }
        }
    }

    // ---- The roof ----
    if show.roof && show.storey.is_none() {
        for info in &snap.planes {
            let Some(plane) = &info.solid else { continue };
            let positions: Vec<Vec3> = plane.points.iter().map(|p| to_view(origin, [p[0], p[1], plane.height_at(*p)])).collect();
            let tris = triangles(&plane.points);
            if tris.is_empty() {
                continue;
            }
            let normal = facing(&positions);
            // What it is covered with, when anything says. A plane with no area is a fault.
            let (paint, metal) = match (&info.condition, info.area.is_some()) {
                (_, false) => (theme::hex(0xE2A9A2), 0.0),
                (None, _) => (theme::roof_bare(), 0.0),
                (Some(c), _) if c.to_lowercase().contains("copper") => (theme::roof_copper(), 0.75),
                (Some(_), _) => (theme::roof_other(), 0.2),
            };
            let mut faces = Faces::new();
            faces.add(&positions, &tris, normal);
            build.piece(&info.path, None, meshes.add(faces.mesh()), Transform::IDENTITY, paint, if metal > 0.5 { 0.38 } else { 0.8 }, metal, true);
            let lifted: Vec<Vec3> = positions.iter().map(|p| *p + normal * 0.06).collect();
            build.drawing.frames.push((info.path.clone(), (0..lifted.len()).map(|i| (lifted[i], lifted[(i + 1) % lifted.len()])).collect()));
        }
        for line in &snap.lines {
            let Some([a, b]) = line.ends else { continue };
            let (a, b) = (to_view(origin, a), to_view(origin, b));
            let lift = Vec3::new(0.0, 0.05, 0.0);
            // No length: drawn broken and red, which no kind of line is.
            let (colour, broken) = if line.length.is_none() { (theme::red(), true) } else { (theme::line_colour(line.kind), false) };
            if !show.ghost {
                build.drawing.lines.push((a + lift, b + lift, colour, broken));
            }
            build.drawing.frames.push((line.path.clone(), vec![(a + lift, b + lift)]));
            // Under each outer edge of the roof, a fascia; and where the job has no walls
            // of its own, a wall down to the datum. Both are for the eye only.
            if matches!(line.kind, Kind::Eave | Kind::Rake) && a.y > FASCIA_FT + 0.5 && b.y > FASCIA_FT + 0.5 {
                let mut normal = (b - a).with_y(0.0).cross(Vec3::Y).normalize_or_zero();
                let inside = line.planes.iter().find_map(|p| snap.planes.iter().find(|x| x.path == *p)).and_then(|p| p.solid.as_ref()).map(|solid| {
                    let n = solid.points.len().max(1) as f64;
                    to_view(origin, [solid.points.iter().map(|q| q[0]).sum::<f64>() / n, solid.points.iter().map(|q| q[1]).sum::<f64>() / n, 0.0])
                });
                if inside.is_some_and(|c| normal.dot(((a + b) / 2.0 - c).with_y(0.0)) < 0.0) {
                    normal = -normal;
                }
                let (fa, fb) = (a - Vec3::Y * FASCIA_FT, b - Vec3::Y * FASCIA_FT);
                let mut trim = Faces::new();
                trim.quad([a, b, fb, fa], normal);
                let alpha = if show.ghost { 0.06 } else { 1.0 };
                let blend = if show.ghost { AlphaMode::Blend } else { AlphaMode::Opaque };
                let fascia = build.materials.add(StandardMaterial { base_color: theme::fascia().with_alpha(alpha), perceptual_roughness: 0.8, double_sided: true, cull_mode: None, alpha_mode: blend, ..default() });
                build.commands.spawn((Mesh3d(meshes.add(trim.mesh())), MeshMaterial3d(fascia), SceneMesh, NotShadowCaster));
                if !has_walls {
                    let mut face = Faces::new();
                    face.quad([fa, fb, fb.with_y(0.0), fa.with_y(0.0)], normal);
                    let wall = build.materials.add(StandardMaterial { base_color: theme::wall().with_alpha(alpha), perceptual_roughness: 0.95, double_sided: true, cull_mode: None, alpha_mode: blend, ..default() });
                    build.commands.spawn((Mesh3d(meshes.add(face.mesh())), MeshMaterial3d(wall), SceneMesh));
                }
            }
        }
    }
}

/// The way a flat outline in space faces, turned to the upper side.
pub fn facing(positions: &[Vec3]) -> Vec3 {
    let mut normal = Vec3::ZERO;
    for i in 0..positions.len() {
        let (u, v) = (positions[i], positions[(i + 1) % positions.len()]);
        normal += Vec3::new((u.y - v.y) * (u.z + v.z), (u.z - v.z) * (u.x + v.x), (u.x - v.x) * (u.y + v.y));
    }
    let normal = normal.normalize_or_zero();
    if normal.y < 0.0 {
        -normal
    } else {
        normal
    }
}

/// Light up what is pointed at and what is selected. A selected system keeps its colour
/// and glows, and everything on other systems goes grey, so the one system stands out.
pub fn tint(doc: Res<Doc>, drawing: Res<Drawing>, mut materials: ResMut<Assets<StandardMaterial>>, mut shown: Local<(Option<String>, Option<String>, u64)>) {
    if shown.0 == doc.hover && shown.1 == doc.selected && shown.2 == doc.geometry {
        return;
    }
    *shown = (doc.hover.clone(), doc.selected.clone(), doc.geometry);
    let a_system = doc.selected.as_ref().is_some_and(|s| drawing.surfaces.iter().any(|x| x.system.as_ref() == Some(s)));
    for surface in &drawing.surfaces {
        let chosen = doc.selected.as_deref() == Some(surface.path.as_str());
        let on_system = a_system && surface.system.as_ref() == doc.selected.as_ref();
        let pointed = doc.hover.as_deref() == Some(surface.path.as_str());
        // What is selected keeps the colour of what it is made of, a little lighter: the
        // outline drawn round it says it is selected, and copper stays copper.
        let (colour, glow) = if chosen {
            (mix(surface.colour, Color::WHITE, 0.16).with_alpha(surface.colour.alpha().max(0.55)), LinearRgba::from(theme::accent()) * 0.08)
        } else if on_system {
            (surface.colour, LinearRgba::from(surface.colour) * 0.3)
        } else if pointed {
            (mix(surface.colour, Color::WHITE, 0.22).with_alpha(surface.colour.alpha().max(0.3)), LinearRgba::BLACK)
        } else if a_system && surface.system.is_some() {
            (mix(surface.colour, theme::hex(0xB8BEC6), 0.85), LinearRgba::BLACK)
        } else {
            (surface.colour, LinearRgba::BLACK)
        };
        if let Some(mut material) = materials.get_mut(&surface.handle) {
            if material.base_color != colour || material.emissive != glow {
                material.base_color = colour;
                material.emissive = glow;
            }
        }
    }
}

/// Ease the camera toward where it is going, and set the lens for the view it is in.
pub fn place_camera(mut orbit: ResMut<Orbit>, doc: Res<Doc>, time: Res<Time>, mut camera: Query<(&mut Transform, &mut Projection, Option<&mut DistanceFog>), With<MainCamera>>) {
    let Ok((mut t, mut projection, fog)) = camera.single_mut() else { return };
    // On plan the eye goes straight overhead before the perspective is taken out.
    let (yaw, pitch) = if doc.plan { (0.0, 1.5607) } else { (orbit.yaw, orbit.pitch) };
    let ease = 1.0 - (-time.delta_secs() * 9.0).exp();
    let (target, distance) = (orbit.target, orbit.distance);
    let now = &mut orbit.now;
    now.0 = now.0.lerp(target, ease);
    now.1 += (yaw - now.1) * ease;
    now.2 += (pitch - now.2) * ease;
    now.3 += (distance - now.3) * ease;
    let (at, yaw, pitch, distance) = *now;
    if let Some(mut fog) = fog {
        let (start, end) = if doc.plan { (50_000.0, 90_000.0) } else { (distance * 2.2, distance * 7.0) };
        if !matches!(fog.falloff, FogFalloff::Linear { start: s, end: e } if (s - start).abs() < 0.5 && (e - end).abs() < 0.5) {
            fog.falloff = FogFalloff::Linear { start, end };
        }
    }
    let overhead = doc.plan && (pitch - 1.5607).abs() < 0.02 && yaw.abs() < 0.02;
    if doc.walk {
        // On foot: the eye is at the target, looking where the yaw and pitch point.
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let looking = Vec3::new(-sy * cp, -sp, -cy * cp);
        *t = Transform::from_translation(at).looking_to(looking, Vec3::Y);
        if !matches!(&*projection, Projection::Perspective(p) if (p.fov - WALK_LENS).abs() < 1e-4) {
            *projection = Projection::from(PerspectiveProjection { fov: WALK_LENS, near: 0.2, far: 20_000.0, ..default() });
        }
    } else if overhead {
        // Straight down with north up, drawn without perspective: a plan.
        *t = Transform::from_translation(at.with_y(orbit.home.0.y * 2.4 + 80.0)).looking_at(at.with_y(0.0), Vec3::NEG_Z);
        let height = distance * 0.62;
        let same = matches!(&*projection, Projection::Orthographic(o) if matches!(o.scaling_mode, ScalingMode::FixedVertical { viewport_height } if (viewport_height - height).abs() < 1e-3));
        if !same {
            *projection = Projection::from(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: height }, far: 2000.0, ..OrthographicProjection::default_3d() });
        }
    } else {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let (lens, back) = (LENS, distance);
        let eye = at + Vec3::new(sy * cp, sp, cy * cp) * back;
        let up = if pitch > 1.5 { Vec3::NEG_Z } else { Vec3::Y };
        *t = Transform::from_translation(eye).looking_at(at, up);
        if !matches!(&*projection, Projection::Perspective(p) if (p.fov - lens).abs() < 1e-4) {
            *projection = Projection::from(PerspectiveProjection { fov: lens, near: 0.5, far: 40_000.0, ..default() });
        }
    }
}

/// Is the pointer over the model, not over a panel?
pub fn over_model(window: &Window) -> bool {
    window.cursor_position().is_some_and(|p| {
        let (w, h) = (window.width(), window.height());
        p.y > BAR_H + GAP * 2.0 && p.y < h - STATUS_H - GAP * 2.0 && p.x > BROWSER_W + GAP * 2.0 && p.x < w - INSPECTOR_W - GAP * 2.0
    })
}

/// Orbit with the right button, pan with the middle one or W A S D, zoom with the wheel.
/// On foot the same keys walk and the right button looks round.
#[allow(clippy::too_many_arguments)]
pub fn orbit_input(mut orbit: ResMut<Orbit>, buttons: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, scroll: Res<AccumulatedMouseScroll>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, doc: Res<Doc>, window: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window.single() else { return };
    let inside = over_model(window);
    let dt = time.delta_secs().min(0.1);
    if buttons.pressed(MouseButton::Right) && !doc.plan {
        let turn = if doc.walk { -1.0 } else { 1.0 };
        orbit.yaw -= motion.delta.x * 0.006 * turn;
        orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(if doc.walk { -1.3 } else { 0.06 }, if doc.walk { 1.3 } else { 1.52 });
    }
    let (sy, cy) = if doc.plan { (0.0, 1.0) } else { orbit.yaw.sin_cos() };
    let right = Vec3::new(cy, 0.0, -sy);
    let toward = Vec3::new(sy, 0.0, cy);
    if !doc.walk && (buttons.pressed(MouseButton::Middle) || (buttons.pressed(MouseButton::Right) && doc.plan)) {
        // Drag the ground under the pointer: a pixel moves the target by what a pixel covers.
        let per_px = orbit.distance * 0.56 / window.height().max(1.0);
        let shift = right * (-motion.delta.x * per_px) + toward * (-motion.delta.y * per_px);
        orbit.target += shift;
    }
    if inside && scroll.delta.y != 0.0 && !doc.walk {
        let steps = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y / 40.0 } else { scroll.delta.y };
        orbit.distance = (orbit.distance * (1.0 - steps * 0.12)).clamp(4.0, 6000.0);
    }
    if doc.entry.is_some() {
        return;
    }
    // W A S D move over the ground; Q and E go down and up. Shift goes faster.
    let pace = if doc.walk { 9.0 } else { orbit.distance * 0.55 } * if keys.pressed(KeyCode::ShiftLeft) { 2.5 } else { 1.0 } * dt;
    let (ahead, across) = (-toward, if doc.walk { -right } else { right });
    let mut step = Vec3::ZERO;
    for (key, way) in [(KeyCode::KeyW, ahead), (KeyCode::KeyS, -ahead), (KeyCode::KeyD, across), (KeyCode::KeyA, -across), (KeyCode::KeyE, Vec3::Y), (KeyCode::KeyQ, Vec3::NEG_Y)] {
        if keys.pressed(key) {
            step += way;
        }
    }
    orbit.target += step * pace;
    let spin = dt * 1.3;
    if keys.pressed(KeyCode::ArrowLeft) {
        orbit.yaw -= spin;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        orbit.yaw += spin;
    }
    if !doc.walk {
        if keys.pressed(KeyCode::ArrowUp) {
            orbit.pitch = (orbit.pitch + spin).min(1.52);
        }
        if keys.pressed(KeyCode::ArrowDown) {
            orbit.pitch = (orbit.pitch - spin).max(0.06);
        }
    }
}

/// The ground's grid, the roof's lines, thin runs, and the outline of what is selected.
pub fn draw_lines(doc: Res<Doc>, drawing: Res<Drawing>, graphics: Res<Graphics>, mut lines: Gizmos<RoofLines>, mut runs: Gizmos<RunLines>, mut broken: Gizmos<BrokenLines>, mut grid: Gizmos<GridLines>, mut highlight: Gizmos<Highlight>) {
    // The grid: a line every ten feet, a stronger one every fifty, fading out with
    // distance from the building so the ground has no edge.
    let reach = drawing.reach;
    if reach > 0.0 && !doc.walk {
        let (minor, major) = if graphics.full { (Color::srgba(0.16, 0.2, 0.14, 0.16), Color::srgba(0.16, 0.2, 0.14, 0.3)) } else { (theme::grid_minor(), theme::grid_major()) };
        let fade = |x: f32, z: f32| (1.0 - (x * x + z * z).sqrt() / reach).clamp(0.0, 1.0).powf(1.4);
        let steps = (reach / 10.0) as i32;
        for i in -steps..=steps {
            let at = i as f32 * 10.0;
            let base = if i % 5 == 0 { major } else { minor };
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
    for (a, b, colour, dashed) in &drawing.lines {
        if *dashed {
            broken.line(*a, *b, *colour);
        } else {
            lines.line(*a, *b, *colour);
        }
    }
    // What is drawn as glass keeps its edges, faintly.
    for (_, frame) in drawing.frames.iter().filter(|(p, _)| drawing.ghosts.contains(p)) {
        for (a, b) in frame {
            grid.line(*a, *b, Color::srgba(0.12, 0.16, 0.22, 0.34));
        }
    }
    for (a, b, colour, dashed) in &drawing.runs {
        if *dashed {
            broken.line(*a, *b, *colour);
        } else {
            runs.line(*a, *b, *colour);
        }
    }
    if let Some(selected) = &doc.selected {
        for (_, frame) in drawing.frames.iter().filter(|(p, _)| p == selected) {
            for (a, b) in frame {
                highlight.line(*a, *b, theme::accent());
            }
        }
        // A selected system: every run on it is drawn over in its own colour, so even a
        // half-inch pipe can be followed through the building.
        let mut done: Vec<&str> = Vec::new();
        for surface in drawing.surfaces.iter().filter(|s| s.system.as_ref() == Some(selected)) {
            if done.contains(&surface.path.as_str()) || drawing.fittings.contains(&surface.path) {
                continue;
            }
            done.push(&surface.path);
            for (_, frame) in drawing.frames.iter().filter(|(p, _)| *p == surface.path) {
                for (a, b) in frame {
                    highlight.line(*a, *b, surface.colour.with_alpha(1.0));
                }
            }
        }
    }
}
