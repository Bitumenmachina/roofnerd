//! The building in space, as a model on a table under studio light: one light that casts
//! shadows, one that fills them, matte surfaces, a neutral ground with a grid and no
//! horizon, and a camera that eases instead of jumping. No sky, no sun, no grass.
//!
//! Everything drawn is an element of the job, placed by `plumbline-world`: walls with their
//! openings cut, floors, the roof's planes and lines, ducts, pipes, conduit and cable at
//! their own size along their own centre lines, equipment and fixtures. A click on any of
//! it selects that element. The ground is the only thing here that is not
//! in the job; on a job with no walls a fascia and a wall are still drawn under the roof's
//! edge, for the eye only.

use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, NotShadowCaster};
use bevy::mesh::Indices;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::PrimaryWindow;

use crate::dress;
use crate::model::{to_view, Doc, Snapshot, Tone};
use crate::theme;
use plumbline_world::finish::Outside;
use plumbline_world::building::Shape;
use plumbline_world::{Discipline, Kind};

/// The main window's layout: a bar along the top, a panel down each side, the build bar
/// over a line of status along the bottom. The model shows in what is left.
pub const TOP_H: f32 = 44.0;
pub const STATUS_H: f32 = 26.0;
pub const LEFT_W: f32 = 252.0;
pub const RIGHT_W: f32 = 344.0;
pub const BUILD_H: f32 = 104.0;
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
/// The light that casts the shadows. On plan it casts none.
#[derive(Component)]
pub struct KeyLight;

/// Whether the full picture is drawn (ambient occlusion, edges steadied over several
/// frames) or the plain one. The plain one is for a machine with no graphics card; ask
/// for it with PLUMBLINE_GRAPHICS=plain. The light and the ground are the same in both.
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

/// On plan the view is this much of the camera's distance tall, in feet.
const PLAN_HEIGHT: f32 = 0.62;

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
    /// Each door and window as a plan draws it: what it is, and the four corners of its
    /// span through the wall, just over the wall's top.
    pub openings: Vec<(String, Kind, [Vec3; 4])>,
    /// Each wall as a plan draws it: the four corners of its top.
    pub walls: Vec<[Vec3; 4]>,
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

pub fn setup(mut commands: Commands, mut store: ResMut<GizmoConfigStore>, mut meshes: ResMut<Assets<Mesh>>, graphics: Res<Graphics>) {
    commands.insert_resource(Solids { cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)), cylinder: meshes.add(Cylinder::new(0.5, 1.0)), ball: meshes.add(Sphere::new(0.5).mesh().ico(3).expect("a sphere")) });
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
    let cascades = CascadeShadowConfigBuilder { first_cascade_far_bound: 180.0, maximum_distance: 1600.0, ..default() }.build();
    // Studio light: the key stands to the viewer's left, forty degrees up, and casts the
    // shadows; a weaker fill from the right front opens them; the rest comes from all round.
    let key_at = Transform::from_xyz(-76.0, 64.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y);
    let fill_at = Transform::from_xyz(60.0, 38.0, 70.0).looking_at(Vec3::ZERO, Vec3::Y);
    let camera_at = Transform::from_xyz(0.0, 40.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y);
    let fog = DistanceFog { color: theme::canvas(), falloff: FogFalloff::Linear { start: 4000.0, end: 9000.0 }, ..default() };
    if graphics.full {
        // Ambient occlusion darkens where surfaces meet, and the picture is steadied over
        // several frames instead of being left jagged.
        commands.spawn((Camera3d::default(), Camera { order: 0, ..default() }, Msaa::Off, Tonemapping::None, DebandDither::Disabled, TemporalAntiAliasing::default(), ScreenSpaceAmbientOcclusion::default(), fog, IsDefaultUiCamera, MainCamera, camera_at));
    } else {
        commands.spawn((Camera3d::default(), Camera { order: 0, ..default() }, Msaa::Sample4, Tonemapping::None, DebandDither::Disabled, fog, IsDefaultUiCamera, MainCamera, camera_at));
    }
    commands.spawn((DirectionalLight { illuminance: 1080.0, shadow_maps_enabled: true, ..default() }, cascades, key_at, KeyLight));
    commands.spawn((DirectionalLight { illuminance: 260.0, shadow_maps_enabled: false, ..default() }, fill_at));
    commands.insert_resource(GlobalAmbientLight { brightness: 1350.0, ..default() });

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

/// Cut a simple outline into triangles, for drawing: the core's cut, taken as it comes
/// even where an outline was too thin to cut cleanly.
fn triangles(pts: &[[f64; 2]]) -> Vec<[u32; 3]> {
    plumbline_world::geom::triangles(pts).0.into_iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect()
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
    /// With the building shown by how sure it is: the colour everything of the thing
    /// being built is drawn in, whatever it is made of.
    wash: Option<Color>,
    /// Set while the wall over and under an opening is built: on plan those pieces are
    /// the opening, as a plan draws it, and a click on them selects it rather than the wall.
    on_plan: Option<String>,
    /// The pattern the next piece is dressed in. Taken by that piece.
    pattern: Option<dress::Pattern>,
}

impl Builder<'_, '_, '_> {
    /// `ghosted` says whether this piece turns to glass when the building is ghosted.
    #[allow(clippy::too_many_arguments)]
    fn piece(&mut self, path: &str, system: Option<&String>, mesh: Handle<Mesh>, at: Transform, colour: Color, roughness: f32, metallic: f32, ghosted: bool) {
        let see_through = ghosted && self.ghost;
        // Shown by how sure it is, a thing is one flat color whatever it is dressed in.
        let pattern = self.pattern.take().filter(|_| self.wash.is_none());
        let colour = self.wash.map(|w| w.with_alpha(colour.alpha())).unwrap_or(colour);
        let (roughness, metallic) = if self.wash.is_some() { (0.95, 0.0) } else { (roughness, metallic) };
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
            base_color_texture: pattern.as_ref().map(|p| p.colour.clone()),
            normal_map_texture: pattern.as_ref().map(|p| p.relief.clone()),
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
        let on_plan = self.on_plan.clone();
        e.observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
            // With a drawing tool in hand a click is a pick, not a selection.
            if press.button == PointerButton::Primary && doc.entry.is_none() && doc.draw.tool.is_none() && !doc.draw.swallow {
                let path = on_plan.as_ref().filter(|_| doc.plan).unwrap_or(&click).clone();
                doc.select(&path);
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
    plans: Res<crate::plans::Plans>,
    old: Query<Entity, With<SceneMesh>>,
    ground: Query<Entity, With<Ground>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut patterns: ResMut<dress::Patterns>,
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
    // On the plan a floor is paper, as a drawing leaves it: what is drawn on it reads.
    let on_plan_view = doc.plan;

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
            let material = StandardMaterial { base_color: theme::ground(), perceptual_roughness: 1.0, reflectance: 0.0, ..default() };
            commands
                .spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(24000.0, 24000.0))), MeshMaterial3d(materials.add(material)), Transform::from_xyz(0.0, -0.04, 0.0), Ground))
                .observe(|press: On<PointerClick>, mut doc: ResMut<Doc>| {
                    // A click on the ground lets go of the selection.
                    if press.button == PointerButton::Primary && doc.selected.is_some() && doc.entry.is_none() && doc.draw.tool.is_none() && !doc.draw.swallow {
                        doc.selected = None;
                        doc.say(Tone::Done, "Nothing selected.");
                        doc.touch();
                    }
                });
        }
    }

    // ---- The drawings laid under the storeys: each sheet flat at its storey's floor ----
    // On the plan the sheet is over the floor, to be traced; in 3D it is under it. It is
    // shown as it is, unlit, and takes no click: what is picked is the place on the floor.
    for l in &snap.drawings {
        if show.storey.as_ref().is_some_and(|s| *s != l.storey) {
            continue;
        }
        let Some(read) = plans.read.get(&l.path) else { continue };
        let floor = snap.storeys.iter().find(|s| s.path == l.storey).and_then(|s| s.elevation).unwrap_or(0.0) + if doc.plan { 0.02 } else { -0.02 };
        let [w, h] = l.drawing.size;
        let corner = |x: f64, y: f64| {
            let p = l.on_plan([x, y]);
            let v = to_view(origin, [p[0], p[1], floor]);
            [v.x, v.y, v.z]
        };
        let sheet = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![corner(0.0, 0.0), corner(w, 0.0), corner(w, h), corner(0.0, h)])
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 4])
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]])
            .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
        let paper = StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(read.image.clone()), unlit: true, double_sided: true, cull_mode: None, ..default() };
        commands.spawn((Mesh3d(meshes.add(sheet)), MeshMaterial3d(materials.add(paper)), Transform::IDENTITY, SceneMesh, NotShadowCaster, Pickable::IGNORE));
    }

    drawing.lines.clear();
    drawing.runs.clear();
    drawing.ghosts.clear();
    drawing.fittings.clear();
    drawing.openings.clear();
    drawing.walls.clear();
    drawing.frames.clear();
    drawing.surfaces.clear();
    let drawing = &mut *drawing;
    let mut build = Builder { commands: &mut commands, materials: &mut materials, drawing, ghost: show.ghost, wash: None, on_plan: None, pattern: None };
    let has_walls = snap.drawn.iter().any(|p| p.kind == Kind::Wall);

    // ---- The rest of the building ----
    for p in &snap.drawn {
        if !show.discipline(p.discipline) || show.storey.as_ref().is_some_and(|s| p.storey.as_ref() != Some(s)) {
            continue;
        }
        let colour = theme::system_colour(snap.classification(p.system.as_ref()));
        build.wash = show.certainty.then(|| theme::certainty(p.status));
        match &p.shape {
            Shape::Wall { a, b, base, height, thickness, holes } => {
                let (va, vb) = (to_view(origin, [a[0], a[1], *base]), to_view(origin, [b[0], b[1], *base]));
                let length = va.distance(vb);
                let along = (vb - va) / length.max(1e-6);
                let turn = Quat::from_rotation_y((b[1] - a[1]).atan2(b[0] - a[0]) as f32);
                let (h, t) = (*height as f32, *thickness as f32);
                let half_t = t / 2.0;
                // What it is dressed in, read from what is priced on it (call 58): which
                // face is outside, the cladding on that face, a wainscot under it, and
                // how far past its ends it runs to close its corners.
                let wd = snap.dress.walls.get(&p.path);
                let outside = wd.map(|d| d.outside).unwrap_or(Outside::NotKnown);
                let has_out = matches!(outside, Outside::Left | Outside::Right);
                let out = wd.and_then(|d| d.out).map(|o| Vec3::new(o[0] as f32, 0.0, -(o[1] as f32))).unwrap_or(turn * Vec3::Z);
                let past = wd.map(|d| [d.past[0] as f32, d.past[1] as f32]).unwrap_or([0.0; 2]);
                let clad = wd.and_then(|d| d.cladding.as_ref()).filter(|_| outside != Outside::Neither);
                let low = wd.and_then(|d| d.wainscot.as_ref()).filter(|_| outside != Outside::Neither);
                // A wall with both faces outdoors, or with no outline to say which face is
                // out, is dressed on both.
                let both = wd.is_some_and(|d| d.both()) && (clad.is_some() || low.is_some());
                let upper = clad.map(|c| patterns.get(&mut images, dress::Look::of_cladding(c.what), c.tone.map(|t| t.rgb)));
                let lower = low.map(|(c, to, _)| (patterns.get(&mut images, dress::Look::of_cladding(c.what), c.tone.map(|t| t.rgb)), *to as f32));
                let masonry = clad.is_some_and(|c| c.what.masonry());
                let part = dress::Part { o: va, x: along, y: Vec3::Y, z: out };
                // The wall round its doors and windows: the same pieces the clash test
                // uses, so what is drawn solid is what is tested as solid. On plan a piece
                // over or under an opening is the opening, and a click on it selects it.
                let spans: Vec<[f64; 4]> = holes.iter().map(|hole| hole.2).collect();
                for [x0, x1, y0, y1] in plumbline_world::geom::face_less_holes(f64::from(length), *height, &spans) {
                    build.on_plan = holes.iter().find(|hole| hole.2[0] <= x0 + 1e-6 && hole.2[1] >= x1 - 1e-6).map(|hole| hole.0.clone());
                    let (x0, x1, y0, y1) = (x0 as f32, x1 as f32, y0 as f32, y1 as f32);
                    if x1 - x0 < 0.005 || y1 - y0 < 0.005 {
                        continue;
                    }
                    let (x0, x1) = (if x0 <= 1e-4 { -past[0] } else { x0 }, if x1 >= length - 1e-4 { length + past[1] } else { x1 });
                    // Four skins: the plain inside and edges, a bare outside face, the
                    // cladding, and the wainscot under it.
                    let (mut plain, mut bare, mut over, mut under) = (dress::Skin::default(), dress::Skin::default(), dress::Skin::default(), dress::Skin::default());
                    let mut face = |z: f32, normal: Vec3, dressed: bool| {
                        let quad = |ya: f32, yb: f32| [part.at(x0, ya, z), part.at(x1, ya, z), part.at(x1, yb, z), part.at(x0, yb, z)];
                        if !dressed {
                            plain.quad(quad(y0, y1), normal, None);
                            return;
                        }
                        let split = lower.as_ref().map(|l| l.1.clamp(y0, y1)).unwrap_or(y0);
                        if let Some((pattern, _)) = lower.as_ref().filter(|_| split > y0 + 1e-4) {
                            under.quad(quad(y0, split), normal, Some(dress::Lay { across: along, up: Vec3::Y, tile: pattern.tile }));
                        }
                        if y1 > split + 1e-4 {
                            match &upper {
                                Some(pattern) => over.quad(quad(split, y1), normal, Some(dress::Lay { across: along, up: Vec3::Y, tile: pattern.tile })),
                                None => bare.quad(quad(split, y1), normal, None),
                            }
                        }
                    };
                    face(half_t, out, has_out || both);
                    face(-half_t, -out, both);
                    let p4 = |x: f32, y: f32, z: f32| part.at(x, y, z);
                    plain.quad([p4(x0, y1, -half_t), p4(x1, y1, -half_t), p4(x1, y1, half_t), p4(x0, y1, half_t)], Vec3::Y, None);
                    plain.quad([p4(x0, y0, -half_t), p4(x1, y0, -half_t), p4(x1, y0, half_t), p4(x0, y0, half_t)], Vec3::NEG_Y, None);
                    // An end that runs past its corner is closed by the wall it corners
                    // with: its own end would lie in that wall's face and fight it.
                    if !(x0 < -1e-4 && past[0] > 1e-4) {
                        plain.quad([p4(x0, y0, -half_t), p4(x0, y0, half_t), p4(x0, y1, half_t), p4(x0, y1, -half_t)], -along, None);
                    }
                    if !(x1 > length + 1e-4 && past[1] > 1e-4) {
                        plain.quad([p4(x1, y0, -half_t), p4(x1, y0, half_t), p4(x1, y1, half_t), p4(x1, y1, -half_t)], along, None);
                    }
                    build.piece(&p.path, None, meshes.add(plain.mesh()), Transform::IDENTITY, theme::wall_inside(), 0.92, 0.0, true);
                    if !bare.is_empty() {
                        build.piece(&p.path, None, meshes.add(bare.mesh()), Transform::IDENTITY, theme::wall_outside(), 0.92, 0.0, true);
                    }
                    if let (false, Some(pattern)) = (over.is_empty(), &upper) {
                        build.pattern = Some(pattern.clone());
                        build.piece(&p.path, None, meshes.add(over.mesh()), Transform::IDENTITY, theme::dressed(), pattern.rough, pattern.metal, true);
                    }
                    if let (false, Some((pattern, _))) = (under.is_empty(), &lower) {
                        build.pattern = Some(pattern.clone());
                        build.piece(&p.path, None, meshes.add(under.mesh()), Transform::IDENTITY, theme::dressed(), pattern.rough, pattern.metal, true);
                    }
                }
                build.on_plan = None;
                // Siding stops against a corner board at each corner; masonry turns the corner itself.
                if has_out && clad.is_some() && !masonry {
                    let mut boards = dress::Skin::default();
                    let from = lower.as_ref().map(|l| l.1.min(h)).unwrap_or(0.0);
                    let (wide, proud) = (4.5 / 12.0, 1.0 / 12.0);
                    if past[0] > 1e-4 {
                        boards.block(&part, [-past[0] - proud, -past[0] + wide], [from, h], [half_t, half_t + proud]);
                    }
                    if past[1] > 1e-4 {
                        boards.block(&part, [length + past[1] - wide, length + past[1] + proud], [from, h], [half_t, half_t + proud]);
                    }
                    if !boards.is_empty() {
                        build.piece(&p.path, None, meshes.add(boards.mesh()), Transform::IDENTITY, theme::trim(), 0.8, 0.0, true);
                    }
                }
                for (hole_path, kind, [from, to, sill, head]) in holes {
                    let (from, to, sill, head) = (*from as f32, *to as f32, *sill as f32, *head as f32);
                    // What fills the opening: a window or a door, built of its parts.
                    let od = snap.dress.openings.get(hole_path);
                    let stood = dress::Part { o: va + along * from + Vec3::Y * sill, x: along, y: Vec3::Y, z: out };
                    let sides = [has_out || both, both];
                    let unit = if *kind == Kind::Window { dress::window(&stood, to - from, head - sill, t, od, sides, masonry) } else { dress::door(&stood, to - from, head - sill, t, od, sides, masonry) };
                    let named = od.and_then(|d| d.tone).map(|t| Color::srgb_u8(t.rgb[0], t.rgb[1], t.rgb[2]));
                    // A window is the color named for it, or trim white; an entry door
                    // nobody has named a color for is wood, and an inside door is painted.
                    let made = named.unwrap_or(if *kind == Kind::Door && has_out { theme::door_leaf() } else { theme::trim() });
                    if !unit.unit.is_empty() {
                        build.piece(hole_path, None, meshes.add(unit.unit.mesh()), Transform::IDENTITY, made, 0.6, 0.0, true);
                    }
                    if !unit.trim.is_empty() {
                        build.piece(hole_path, None, meshes.add(unit.trim.mesh()), Transform::IDENTITY, theme::trim(), 0.8, 0.0, true);
                    }
                    if !unit.glass.is_empty() {
                        build.piece(hole_path, None, meshes.add(unit.glass.mesh()), Transform::IDENTITY, theme::glass(), 0.08, 0.0, true);
                    }
                    for knob in &unit.knobs {
                        build.piece(hole_path, None, solids.ball.clone(), Transform { translation: *knob, scale: Vec3::splat(2.25 / 12.0), ..default() }, theme::hardware(), 0.4, 0.7, true);
                    }
                    let half = turn * Vec3::Z * (t / 2.0);
                    let (c0, c1) = (va + along * from + Vec3::Y * sill, va + along * to + Vec3::Y * sill);
                    let up = Vec3::Y * (head - sill);
                    build.drawing.frames.push((hole_path.clone(), box_edges([c0 - half, c1 - half, c1 + half, c0 + half, c0 - half + up, c1 - half + up, c1 + half + up, c0 + half + up])));
                    let over = Vec3::Y * (h + 0.06);
                    let (j0, j1) = (va + along * from + over, va + along * to + over);
                    build.drawing.openings.push((hole_path.clone(), *kind, [j0 - half, j1 - half, j1 + half, j0 + half]));
                }
                let half = turn * Vec3::Z * (t / 2.0);
                let up = Vec3::Y * h;
                build.drawing.frames.push((p.path.clone(), box_edges([va - half, vb - half, vb + half, va + half, va - half + up, vb - half + up, vb + half + up, va + half + up])));
                // On plan a wall is its two faces and its ends, in ink: a wall traced over
                // a drawing has to read against the sheet.
                let over = up + Vec3::Y * 0.05;
                build.drawing.walls.push([va - half + over, vb - half + over, vb + half + over, va + half + over]);
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
                build.piece(&p.path, None, meshes.add(faces.mesh()), Transform::IDENTITY, if on_plan_view { theme::plan_paper() } else { theme::floor_colour() }, 0.75, 0.0, true);
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
    build.wash = None;
    // In 3D the roof is drawn with every level on show, and with the top level alone on
    // show: it sits on that one. (It was drawn only with every level showing, and a job
    // with one level has no such choice: its roof was never drawn.) On the plan of one
    // level it is its lines, broken, further down.
    let top_level = snap.storeys.iter().filter(|s| s.elevation.is_some()).max_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0))).map(|s| s.path.clone());
    let roof_in_3d = show.storey.is_none() || (!on_plan_view && show.storey == top_level);
    if show.roof && roof_in_3d {
        for info in &snap.planes {
            build.wash = show.certainty.then(|| theme::certainty(info.worst()));
            let Some(plane) = &info.solid else { continue };
            let positions: Vec<Vec3> = plane.points.iter().map(|p| to_view(origin, [p[0], p[1], plane.height_at(*p)])).collect();
            let tris = triangles(&plane.points);
            if tris.is_empty() {
                continue;
            }
            let normal = facing(&positions);
            // What it is covered with, when anything priced on it says (call 58). A
            // plane with no area is a fault; one nothing names a roofing for is bare.
            let said = snap.dress.roofs.get(&info.path).filter(|_| info.area.is_some());
            match said {
                Some(r) => {
                    let pattern = patterns.get(&mut images, dress::Look::of_roofing(r.what), r.tone.map(|t| t.rgb));
                    // The pattern runs level across the plane, its top up the slope.
                    let across = Vec3::Y.cross(normal).normalize_or(Vec3::X);
                    let mut up = normal.cross(across);
                    if up.y < 0.0 {
                        up = -up;
                    }
                    let mut skin = dress::Skin::default();
                    skin.add(&positions, &tris, normal, Some(dress::Lay { across, up, tile: pattern.tile }));
                    build.pattern = Some(pattern.clone());
                    build.piece(&info.path, None, meshes.add(skin.mesh()), Transform::IDENTITY, theme::dressed(), pattern.rough, pattern.metal, true);
                }
                None => {
                    let paint = if info.area.is_some() { theme::roof_bare() } else { theme::hex(0xE2A9A2) };
                    let mut faces = Faces::new();
                    faces.add(&positions, &tris, normal);
                    build.piece(&info.path, None, meshes.add(faces.mesh()), Transform::IDENTITY, paint, 0.8, 0.0, true);
                }
            }
            build.wash = None;
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
                // A gutter where one is priced on the eave: hung on the fascia, 5 in out.
                if let Some(g) = snap.dress.gutters.get(&line.path).filter(|_| line.kind == Kind::Eave) {
                    let run = (b - a).normalize_or(Vec3::X);
                    let hung = dress::Part { o: a, x: run, y: Vec3::Y, z: normal };
                    let mut gutter = dress::Skin::default();
                    gutter.block(&hung, [0.0, a.distance(b)], [-4.5 / 12.0, -0.6 / 12.0], [0.02, 5.0 / 12.0]);
                    let paint = g.tone.map(|t| Color::srgb_u8(t.rgb[0], t.rgb[1], t.rgb[2])).unwrap_or(theme::trim());
                    build.piece(&line.path, None, meshes.add(gutter.mesh()), Transform::IDENTITY, paint, 0.6, if g.tone.is_some_and(|t| t.word == "copper") { 0.7 } else { 0.0 }, true);
                }
                if !has_walls {
                    let mut face = Faces::new();
                    face.quad([fa, fb, fb.with_y(0.0), fa.with_y(0.0)], normal);
                    let wall = build.materials.add(StandardMaterial { base_color: theme::wall().with_alpha(alpha), perceptual_roughness: 0.95, double_sided: true, cull_mode: None, alpha_mode: blend, ..default() });
                    build.commands.spawn((Mesh3d(meshes.add(face.mesh())), MeshMaterial3d(wall), SceneMesh));
                }
            }
        }
    }
    // On the plan of one level the roof over it is drawn as a plan draws what is
    // overhead: its lines, broken. (With every level on show the roof itself is drawn.)
    if show.roof && show.storey.is_some() && on_plan_view && !show.ghost {
        for line in &snap.lines {
            let Some([a, b]) = line.ends else { continue };
            let (a, b) = (to_view(origin, a), to_view(origin, b));
            build.drawing.lines.push((a, b, theme::ink_soft(), true));
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
pub fn place_camera(mut orbit: ResMut<Orbit>, doc: Res<Doc>, time: Res<Time>, mut camera: Query<(&mut Transform, &mut Projection, Option<&mut DistanceFog>), With<MainCamera>>, mut sun: Query<&mut DirectionalLight, With<KeyLight>>) {
    // A plan is a drawing, not a view: it has no shadows to hide what is on the floor.
    for mut light in &mut sun {
        if light.shadow_maps_enabled == doc.plan {
            light.shadow_maps_enabled = !doc.plan;
        }
    }
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
        let height = distance * PLAN_HEIGHT;
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
    window.cursor_position().is_some_and(|p| over_model_at(window, p))
}

/// Is this place in the window over the model, not over a panel?
pub fn over_model_at(window: &Window, p: Vec2) -> bool {
    let (w, h) = (window.width(), window.height());
    p.y > TOP_H && p.y < h - STATUS_H - BUILD_H && p.x > LEFT_W && p.x < w - RIGHT_W
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
        // (On plan that is exact: the view is PLAN_HEIGHT of the distance tall.)
        let per_px = orbit.distance * if doc.plan { PLAN_HEIGHT } else { 0.56 } / window.height().max(1.0);
        let shift = right * (-motion.delta.x * per_px) + toward * (-motion.delta.y * per_px);
        orbit.target += shift;
    }
    if inside && scroll.delta.y != 0.0 && !doc.walk {
        let steps = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y / 40.0 } else { scroll.delta.y };
        let was = orbit.distance;
        orbit.distance = (orbit.distance * (1.0 - steps * 0.12)).clamp(4.0, 6000.0);
        // On plan the wheel zooms about the pointer: the place under it stays under it,
        // which is how a dimension's end on a drawing is closed in on.
        if let (true, Some(cursor)) = (doc.plan, window.cursor_position()) {
            let per_px = was * PLAN_HEIGHT / window.height().max(1.0);
            let off = cursor - Vec2::new(window.width(), window.height()) / 2.0;
            let under = orbit.target + right * (off.x * per_px) + toward * (off.y * per_px);
            orbit.target = under + (orbit.target - under) * (orbit.distance / was);
        }
    }
    // While a value is typed the keys are its letters; with Ctrl held they are Save and Undo.
    if doc.entry.is_some() || keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
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
pub fn draw_lines(doc: Res<Doc>, snap: Res<Snapshot>, drawing: Res<Drawing>, mut lines: Gizmos<RoofLines>, mut runs: Gizmos<RunLines>, mut broken: Gizmos<BrokenLines>, mut grid: Gizmos<GridLines>, mut highlight: Gizmos<Highlight>) {
    // The grid: a line every ten feet, a stronger one every fifty, fading out with
    // distance from the building so the ground has no edge.
    let reach = drawing.reach;
    if reach > 0.0 && !doc.walk {
        let (minor, major) = (theme::grid_minor(), theme::grid_major());
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
    // On plan a door or a window is its span through the wall: looking straight down,
    // the wall over it hides it. A window has its glass down the middle. Which way a door
    // swings is not in the model, so no swing is drawn.
    if doc.plan {
        for c in &drawing.walls {
            for i in 0..4 {
                lines.line(c[i], c[(i + 1) % 4], theme::ink());
            }
        }
        for (path, kind, c) in &drawing.openings {
            // The selected one is drawn over in the accent, as anything selected is.
            if doc.selected.as_ref() == Some(path) {
                for i in 0..4 {
                    highlight.line(c[i], c[(i + 1) % 4], theme::accent());
                }
                continue;
            }
            let colour = if *kind == Kind::Window { theme::plan_window() } else { theme::plan_door() };
            for i in 0..4 {
                lines.line(c[i], c[(i + 1) % 4], colour);
            }
            if *kind == Kind::Window {
                lines.line((c[0] + c[3]) / 2.0, (c[1] + c[2]) / 2.0, colour);
            }
        }
    }
    for (a, b, colour, dashed) in &drawing.runs {
        if *dashed {
            broken.line(*a, *b, *colour);
        } else {
            runs.line(*a, *b, *colour);
        }
    }
    // With the sheet on its Clashes tab, each place two things are in the same space is
    // marked on the model: three short strokes, seen through whatever is in front.
    if matches!(doc.tab, crate::model::Tab::Clashes | crate::model::Tab::Verify) || doc.show.clashes {
        if let Some(Ok(found)) = &snap.clashes {
            for c in found.found.iter().filter(|c| !c.through) {
                let at = to_view(doc.origin, c.at);
                for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                    highlight.line(at - axis * 0.75, at + axis * 0.75, theme::clash());
                }
            }
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
