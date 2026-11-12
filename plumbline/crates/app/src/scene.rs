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
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
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

/// The sky's dome.
#[derive(Component)]
pub struct Sky;

/// The sky's light, kept so that the small picture behind the frosted panels is lit as
/// the window is.
#[derive(Resource, Clone)]
pub struct SkyLight(pub EnvironmentMapLight);

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

/// How bright the sky's light is and how bright the sun, in the engine's measures (the
/// sky in candela to the square metre, the sun in lux), for the camera's standing
/// exposure. Set by eye on the made-up whole house: the sun about three times what the
/// sky gives a wall, so that a wall in the sun and a wall out of it are plainly two
/// walls and neither is white.
pub const SKY_LIGHT: f32 = 1150.0;
pub const SUN_LIGHT: f32 = 3600.0;
/// The sun is a little warm, as sunlight is beside the sky's fill.
pub const SUN_COLOUR: Color = Color::srgb(1.0, 0.955, 0.89);

/// How the range of light is folded to the screen in 3D. On plan nothing is folded: a
/// plan is a drawing, and its paper and its inks are the colours they are given.
pub const FOLD: Tonemapping = Tonemapping::TonyMcMapface;

pub fn setup(mut commands: Commands, mut store: ResMut<GizmoConfigStore>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>, graphics: Res<Graphics>) {
    commands.insert_resource(Solids { cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)), cylinder: meshes.add(Cylinder::new(0.5, 1.0)), ball: meshes.add(Sphere::new(0.5).mesh().ico(3).expect("a sphere")) });
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
    let cascades = CascadeShadowConfigBuilder { first_cascade_far_bound: 180.0, maximum_distance: 1600.0, ..default() }.build();
    // The light of a place: one sun, to the viewer's left and forty degrees up, which
    // casts the shadows; and the sky, which lights every surface by how much of the sky
    // and how much of the ground it faces, and which glass and metal reflect
    // (`skylight`). There is no second lamp and no flat light from everywhere.
    let key_at = Transform::from_xyz(-76.0, 64.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y);
    let linear = |c: Color| { let l = c.to_linear(); [l.red, l.green, l.blue] };
    let (falls, reflects) = crate::skylight::light(linear(theme::sky_horizon()), linear(theme::sky_overhead()), linear(theme::yard()));
    let sky_light = EnvironmentMapLight { diffuse_map: images.add(falls), specular_map: images.add(reflects), intensity: SKY_LIGHT, ..default() };
    commands.insert_resource(SkyLight(sky_light.clone()));
    let camera_at = Transform::from_xyz(0.0, 40.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y);
    // The sky: a dome round the eye, pale at the horizon and blue overhead, that nothing
    // lights and nothing shades. It goes where the eye goes, so it is never reached.
    let mut dome = Sphere::new(1.0).mesh().uv(48, 24);
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(at)) = dome.attribute(Mesh::ATTRIBUTE_POSITION) {
        let (low, high) = (theme::sky_horizon().to_linear(), theme::sky_overhead().to_linear());
        let colours: Vec<[f32; 4]> = at.iter().map(|p| { let t = p[1].clamp(0.0, 1.0).powf(0.55); [low.red + (high.red - low.red) * t, low.green + (high.green - low.green) * t, low.blue + (high.blue - low.blue) * t, 1.0] }).collect();
        dome.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    }
    let mut over = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, cull_mode: None, fog_enabled: false, ..default() });
    let _ = &mut over;
    commands.spawn((Mesh3d(meshes.add(dome)), MeshMaterial3d(over), Transform::from_scale(Vec3::splat(15_000.0)), bevy::light::NotShadowCaster, Pickable::IGNORE, Sky));
    let fog = DistanceFog { color: theme::sky_horizon(), falloff: FogFalloff::Linear { start: 4000.0, end: 9000.0 }, ..default() };
    if graphics.full {
        // Ambient occlusion darkens where surfaces meet, and the picture is steadied over
        // several frames instead of being left jagged.
        // The camera takes in a range of light and folds it to the screen, as a camera does.
        commands.spawn((Camera3d::default(), Camera { order: 0, ..default() }, bevy::camera::Hdr, Msaa::Off, FOLD, DebandDither::Enabled, TemporalAntiAliasing::default(), ScreenSpaceAmbientOcclusion::default(), fog, sky_light, IsDefaultUiCamera, MainCamera, camera_at));
    } else {
        commands.spawn((Camera3d::default(), Camera { order: 0, ..default() }, bevy::camera::Hdr, Msaa::Sample4, FOLD, DebandDither::Enabled, fog, sky_light, IsDefaultUiCamera, MainCamera, camera_at));
    }
    commands.spawn((DirectionalLight { illuminance: SUN_LIGHT, color: SUN_COLOUR, shadow_maps_enabled: true, ..default() }, cascades, key_at, KeyLight));
    commands.insert_resource(GlobalAmbientLight { brightness: 0.0, ..default() });

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
    // What is selected is outlined finely: a heavy wire hid what it was round.
    highlight.line.width = 2.5;
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
            if press.button == PointerButton::Primary && doc.entry.is_none() && doc.draw.tool.is_none() && !doc.draw.swallow && !dragged() {
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
            let material = StandardMaterial { base_color: if doc.plan { theme::ground() } else { theme::yard() }, perceptual_roughness: 1.0, reflectance: 0.0, ..default() };
            commands
                .spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(24000.0, 24000.0))), MeshMaterial3d(materials.add(material)), Transform::from_xyz(0.0, -0.04, 0.0), Ground))
                .observe(|press: On<PointerClick>, mut doc: ResMut<Doc>| {
                    // A click on the ground lets go of the selection.
                    if press.button == PointerButton::Primary && doc.selected.is_some() && doc.entry.is_none() && doc.draw.tool.is_none() && !doc.draw.swallow && !dragged() {
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
        if show.storey.as_ref().is_some_and(|s| *s != l.storey) || show.off_levels.contains(&l.storey) {
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
    // Each outside wall as the roof's edge sees it: its line, the way its outside face
    // looks, half its thickness and its top. An eave that hangs past one gets a soffit.
    let mut outside_walls: Vec<(Vec3, Vec3, Vec3, f32, f32)> = Vec::new();

    // ---- Interior woodwork: the base in each room that has base priced on it ----
    // A board 5 1/4 in high and 5/8 in thick at the foot of the room's walls, along
    // the room's own outline (the faces of its walls), left out across each door.
    // It is finish: off with Finishes, and with its level.
    if show.finishes && show.architecture && !on_plan_view {
        let (tall, thick) = (5.25_f32 / 12.0, 0.625_f32 / 12.0);
        for (room, (storey, outline)) in &snap.dress.base {
            if show.storey.as_ref().is_some_and(|s| storey.as_ref() != Some(s)) || storey.as_ref().is_some_and(|s| show.off_levels.contains(s)) {
                continue;
            }
            let floor = storey.as_ref().and_then(|s| snap.storeys.iter().find(|x| x.path == *s)).and_then(|s| s.elevation).unwrap_or(0.0);
            // The doors of this level, each as its two ends on the plan and how far
            // either side of its wall's centre line the wall reaches.
            let doors: Vec<([f64; 2], [f64; 2], f64)> = snap.drawn.iter().filter(|p| p.kind == Kind::Wall && p.storey == *storey).flat_map(|p| match &p.shape {
                Shape::Wall { a, b, thickness, holes, .. } => {
                    let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt().max(1e-9);
                    let u = [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
                    holes.iter().filter(|h| h.1 == Kind::Door).map(|h| ([a[0] + u[0] * h.2[0], a[1] + u[1] * h.2[0]], [a[0] + u[0] * h.2[1], a[1] + u[1] * h.2[1]], thickness / 2.0 + 0.1)).collect::<Vec<_>>()
                }
                _ => Vec::new(),
            }).collect();
            for (i, e0) in outline.iter().enumerate() {
                let e1 = outline[(i + 1) % outline.len()];
                let len = ((e1[0] - e0[0]).powi(2) + (e1[1] - e0[1]).powi(2)).sqrt();
                if len < 0.05 {
                    continue;
                }
                let d = [(e1[0] - e0[0]) / len, (e1[1] - e0[1]) / len];
                // What of this edge a door takes: a door in a wall that runs along it.
                let mut cuts: Vec<(f64, f64)> = doors.iter().filter_map(|(p0, p1, reach)| {
                    let off = |p: &[f64; 2]| ((p[0] - e0[0]) * d[1] - (p[1] - e0[1]) * d[0]).abs();
                    let at = |p: &[f64; 2]| (p[0] - e0[0]) * d[0] + (p[1] - e0[1]) * d[1];
                    (off(p0) <= *reach && off(p1) <= *reach).then(|| (at(p0).min(at(p1)), at(p0).max(at(p1))))
                }).filter(|(t0, t1)| *t1 > 0.0 && *t0 < len).collect();
                cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut from = 0.0;
                let mut runs: Vec<(f64, f64)> = Vec::new();
                for (t0, t1) in cuts {
                    if t0 > from + 0.05 {
                        runs.push((from, t0));
                    }
                    from = from.max(t1);
                }
                if len > from + 0.05 {
                    runs.push((from, len));
                }
                // Inside the room is to the left of an outline that runs counter-clockwise.
                let turn = Quat::from_rotation_y(d[1].atan2(d[0]) as f32);
                for (t0, t1) in runs {
                    let mid = (t0 + t1) / 2.0;
                    let on_plan = [e0[0] + d[0] * mid - d[1] * f64::from(thick) / 2.0, e0[1] + d[1] * mid + d[0] * f64::from(thick) / 2.0];
                    let middle = to_view(origin, [on_plan[0], on_plan[1], floor]) + Vec3::Y * (tall / 2.0);
                    build.piece(room, None, solids.cube.clone(), Transform { translation: middle, rotation: turn, scale: Vec3::new((t1 - t0) as f32, tall, thick) }, theme::casework(), 0.6, 0.0, false);
                }
            }
        }
    }

    // ---- The rest of the building ----
    for p in &snap.drawn {
        // Its trade's switch, the level on show, its level's own switch, its system's.
        if !show.discipline(p.discipline) || show.storey.as_ref().is_some_and(|s| p.storey.as_ref() != Some(s)) || p.storey.as_ref().is_some_and(|s| show.off_levels.contains(s)) || p.system.as_ref().is_some_and(|s| show.off_systems.contains(s)) {
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
                // A strip footing: concrete, as wide and as deep as it is, and nothing
                // else. Where another footing ends at one of its ends it is carried on
                // half its width, so the corner closes instead of leaving a notch.
                if p.kind == Kind::Footing {
                    let meets = |at: &[f64; 2]| snap.drawn.iter().any(|q| q.kind == Kind::Footing && q.path != p.path && matches!(&q.shape, Shape::Wall { a: qa, b: qb, .. } if [qa, qb].iter().any(|e| (e[0] - at[0]).abs() < 0.01 && (e[1] - at[1]).abs() < 0.01)));
                    let (back, on) = (if meets(a) { t / 2.0 } else { 0.0 }, if meets(b) { t / 2.0 } else { 0.0 });
                    let middle = va + along * ((length + on - back) / 2.0) + Vec3::Y * (h / 2.0);
                    build.piece(&p.path, None, solids.cube.clone(), Transform { translation: middle, rotation: turn, scale: Vec3::new(length + back + on, h, t) }, theme::concrete(), 0.92, 0.0, true);
                    continue;
                }
                let half_t = t / 2.0;
                // What it is dressed in, read from what is priced on it (call 58): which
                // face is outside, the cladding on that face, a wainscot under it, and
                // how far past its ends it runs to close its corners.
                let wd = snap.dress.walls.get(&p.path);
                // With Framing switched on, a wall whose framing is priced is drawn as
                // its frame and nothing else: the plates, the studs on their centres,
                // and round each opening its kings, jacks, header, sill and cripples
                // (`building::framing`). On plan a wall stays a wall.
                if let (true, false, Some((deep, spacing))) = (show.framing, on_plan_view, wd.and_then(|d| d.framing)) {
                    let spans: Vec<[f64; 4]> = holes.iter().map(|hole| hole.2).collect();
                    for m in plumbline_world::building::framing::frame(f64::from(length), *height, &spans, spacing) {
                        let (x0, x1, y0, y1) = (m.along[0] as f32, m.along[1] as f32, m.up[0] as f32, m.up[1] as f32);
                        let middle = va + along * ((x0 + x1) / 2.0) + Vec3::Y * ((y0 + y1) / 2.0);
                        build.piece(&p.path, None, solids.cube.clone(), Transform { translation: middle, rotation: turn, scale: Vec3::new(x1 - x0, y1 - y0, deep as f32) }, theme::lumber(), 0.85, 0.0, false);
                    }
                    continue;
                }
                let outside = wd.map(|d| d.outside).unwrap_or(Outside::NotKnown);
                let has_out = matches!(outside, Outside::Left | Outside::Right);
                let out = wd.and_then(|d| d.out).map(|o| Vec3::new(o[0] as f32, 0.0, -(o[1] as f32))).unwrap_or(turn * Vec3::Z);
                let past = wd.map(|d| [d.past[0] as f32, d.past[1] as f32]).unwrap_or([0.0; 2]);
                // With Finishes switched off the cladding and the wainscot are taken off,
                // and the wall's outside face is its weather barrier where one is priced.
                let clad = wd.and_then(|d| d.cladding.as_ref()).filter(|_| outside != Outside::Neither && show.finishes);
                let low = wd.and_then(|d| d.wainscot.as_ref()).filter(|_| outside != Outside::Neither && show.finishes);
                let wrapped = !show.finishes && wd.is_some_and(|d| d.barrier);
                // A wall with both faces outdoors, or with no outline to say which face is
                // out, is dressed on both.
                let both = wd.is_some_and(|d| d.both()) && (clad.is_some() || low.is_some());
                let upper = clad.map(|c| patterns.get(&mut images, dress::Look::of_cladding(c.what), c.tone.map(|t| t.rgb)));
                let lower = low.map(|(c, to, _)| (patterns.get(&mut images, dress::Look::of_cladding(c.what), c.tone.map(|t| t.rgb)), *to as f32));
                let masonry = clad.is_some_and(|c| c.what.masonry());
                let part = dress::Part { o: va, x: along, y: Vec3::Y, z: out };
                if has_out {
                    outside_walls.push((va, vb, out, half_t, va.y + h));
                }
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
                        build.piece(&p.path, None, meshes.add(bare.mesh()), Transform::IDENTITY, if wrapped { theme::barrier() } else { theme::wall_outside() }, if wrapped { 0.55 } else { 0.92 }, 0.0, true);
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
                    // With the finish off, the flashing round an opening that has it
                    // priced: a strip at the head, down each side and under the sill, on
                    // the wall's outside face, as it goes on before the siding.
                    if !show.finishes && has_out && od.is_some_and(|d| d.flashed) {
                        let w = 4.0 / 12.0;
                        let mut strips = vec![(from - w, to + w, head, head + w), (from - w, from, sill, head), (to, to + w, sill, head)];
                        if sill > 0.1 {
                            strips.push((from - w, to + w, sill - w, sill));
                        }
                        for (x0, x1, y0, y1) in strips {
                            let middle = va + along * ((x0 + x1) / 2.0) + Vec3::Y * ((y0 + y1) / 2.0) + out * (half_t + 0.02);
                            build.piece(hole_path, None, solids.cube.clone(), Transform { translation: middle, rotation: turn, scale: Vec3::new(x1 - x0, y1 - y0, 0.03) }, theme::flashing(), 0.4, 0.3, false);
                        }
                    }
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
                    // The casing and sill are finish: with Finishes off they are off too,
                    // and the flashing they cover is seen.
                    if !unit.trim.is_empty() && show.finishes {
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
                        // Casework and furniture: wood, until a thing is drawn as what it is.
                        (Kind::Furnishing, _) => (theme::casework(), 0.6, 0.0),
                        (_, Discipline::Plumbing) => (theme::porcelain(), 0.2, 0.0),
                        _ => (mix(theme::porcelain(), colour, 0.35), 0.5, 0.1),
                    };
                    // Drawn as what it is, where its name says what that is: a vanity as a
                    // cabinet on its plinth under a worktop, a water closet as a cistern
                    // and a bowl (the forms are BIMCraft's; `building::forms`). Every block
                    // is inside the box, which is still what is tested for clashes and
                    // what is outlined when the thing is selected. A thing whose name
                    // names no form is the box it is.
                    match plumbline_world::building::forms::form_of(&p.name) {
                        Some(form) => {
                            use plumbline_world::building::forms::Made;
                            for b in plumbline_world::building::forms::blocks(form, *size) {
                                // The thing's own frame: x along its width, y from front to
                                // back, z up. The view's: x east, y up, z south.
                                let mid = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
                                let at = centre + turn * Vec3::new(mid[0] as f32, (mid[2] - size[2] / 2.0) as f32, -mid[1] as f32);
                                let big = Vec3::new((b.hi[0] - b.lo[0]) as f32, (b.hi[2] - b.lo[2]) as f32, (b.hi[1] - b.lo[1]) as f32);
                                let (ink, rough) = match b.made {
                                    Made::Timber => (theme::casework(), 0.6),
                                    Made::Worktop => (theme::worktop(), 0.35),
                                    Made::Porcelain => (theme::porcelain(), 0.2),
                                    Made::Fabric => (theme::fabric(), 0.95),
                                };
                                let solid = if b.round { solids.cylinder.clone() } else { solids.cube.clone() };
                                build.piece(&p.path, p.system.as_ref(), solid, Transform { translation: at, rotation: turn, scale: big }, ink, rough, 0.0, false);
                            }
                        }
                        None => build.piece(&p.path, p.system.as_ref(), solids.cube.clone(), Transform { translation: centre, rotation: turn, scale }, paint, rough, metal, false),
                    }
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
                // Where the eave hangs past an outside wall, a soffit closes it underneath:
                // level, from the foot of the fascia back to the wall's face. (Without
                // it the gap between the wall's top and the roof stood open and dark.)
                if line.kind == Kind::Eave && !show.ghost {
                    let mid = (a + b) / 2.0;
                    let hang = outside_walls
                        .iter()
                        .filter(|(_, _, out, _, top)| out.dot(normal) > 0.9 && (top - mid.y).abs() < 3.0)
                        .filter_map(|(wa, wb, out, half, _)| {
                            let run = (*wb - *wa).with_y(0.0);
                            let along = (mid - *wa).with_y(0.0).dot(run.normalize_or_zero());
                            let past = (mid - *wa).with_y(0.0).dot(*out) - half;
                            (along > -4.0 && along < run.length() + 4.0 && past > 0.05 && past < 4.0).then_some(past)
                        })
                        .min_by(|x, y| x.total_cmp(y));
                    if let Some(past) = hang {
                        let mut under = Faces::new();
                        under.quad([fa, fb, fb - normal * past, fa - normal * past], Vec3::NEG_Y);
                        let soffit = build.materials.add(StandardMaterial { base_color: theme::trim(), perceptual_roughness: 0.85, double_sided: true, cull_mode: None, ..default() });
                        build.commands.spawn((Mesh3d(meshes.add(under.mesh())), MeshMaterial3d(soffit), SceneMesh, NotShadowCaster, Pickable::IGNORE));
                    }
                }
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
pub fn tint(doc: Res<Doc>, snap: Res<Snapshot>, drawing: Res<Drawing>, mut materials: ResMut<Assets<StandardMaterial>>, mut shown: Local<(Option<String>, Option<String>, u64, u64)>) {
    if shown.0 == doc.hover && shown.1 == doc.selected && shown.2 == doc.geometry && shown.3 == snap.revision {
        return;
    }
    *shown = (doc.hover.clone(), doc.selected.clone(), doc.geometry, snap.revision);
    // A placed thing that is selected and in the same space as another is drawn red.
    let clashing = doc.selected.as_ref().filter(|p| snap.selected.as_ref().is_some_and(|s| stands(s.kind) && s.path == **p) && in_what(&snap, p).is_some());
    let a_system = doc.selected.as_ref().is_some_and(|s| drawing.surfaces.iter().any(|x| x.system.as_ref() == Some(s)));
    for surface in &drawing.surfaces {
        let chosen = doc.selected.as_deref() == Some(surface.path.as_str());
        let on_system = a_system && surface.system.as_ref() == doc.selected.as_ref();
        let pointed = doc.hover.as_deref() == Some(surface.path.as_str());
        // What is selected keeps the colour of what it is made of, a little lighter: the
        // outline drawn round it says it is selected, and copper stays copper.
        // (A surface with a pattern on it takes no glow: on brick the blue of it read
        // as a violet wall, which is not what the wall is made of.)
        let dressed = surface.colour == theme::dressed();
        let (colour, glow) = if chosen && clashing.is_some() {
            (mix(surface.colour, theme::clash(), 0.72), LinearRgba::from(theme::clash()) * 0.10)
        } else if chosen {
            (mix(surface.colour, Color::WHITE, 0.16).with_alpha(surface.colour.alpha().max(0.55)), if dressed { LinearRgba::BLACK } else { LinearRgba::from(theme::accent()) * 0.08 })
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

/// The ground is paper on plan and a yard in 3D, whichever the view is now: it is not
/// left the colour it was drawn in when the view changes without anything being redrawn.
pub fn ground_tone(doc: Res<Doc>, ground: Query<&MeshMaterial3d<StandardMaterial>, With<Ground>>, mut seen: Query<(&mut Visibility, &mut Transform), With<Ground>>, snap: Res<Snapshot>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // The ground has a switch: off, what is under it is seen. On plan it is the paper
    // the drawing is on, and lies under the lowest thing drawn, so the plan of a
    // foundation is not hidden by the ground over it.
    let lowest = snap.drawn.iter().filter_map(|p| match &p.shape { Shape::Wall { base, .. } => Some(*base as f32), _ => None }).fold(0.0_f32, f32::min);
    let (want_seen, want_y) = (if doc.show.ground { Visibility::Inherited } else { Visibility::Hidden }, if doc.plan { lowest - 0.5 } else { 0.0 } - 0.04);
    for (mut v, mut t) in &mut seen {
        if *v != want_seen {
            *v = want_seen;
        }
        if (t.translation.y - want_y).abs() > 1.0e-4 {
            t.translation.y = want_y;
        }
    }
    let want = if doc.plan { theme::ground() } else { theme::yard() };
    for handle in &ground {
        if materials.get(&handle.0).is_some_and(|m| m.base_color != want) {
            if let Some(mut material) = materials.get_mut(&handle.0) {
                material.base_color = want;
            }
        }
    }
}

/// Ease the camera toward where it is going, and set the lens for the view it is in.
pub fn place_camera(mut orbit: ResMut<Orbit>, doc: Res<Doc>, time: Res<Time>, mut camera: Query<(&mut Transform, &mut Projection, Option<&mut DistanceFog>, &mut Tonemapping), (With<MainCamera>, Without<Sky>)>, mut sun: Query<&mut DirectionalLight, With<KeyLight>>, mut sky: Query<(&mut Transform, &mut Visibility), (With<Sky>, Without<MainCamera>)>) {
    // A plan is a drawing, not a view: it has no shadows to hide what is on the floor.
    for mut light in &mut sun {
        if light.shadow_maps_enabled == doc.plan {
            light.shadow_maps_enabled = !doc.plan;
        }
    }
    let Ok((mut t, mut projection, fog, mut fold)) = camera.single_mut() else { return };
    // A plan is a drawing: its paper and inks go to the screen as they are.
    let wanted = if doc.plan { Tonemapping::None } else { FOLD };
    if *fold != wanted {
        *fold = wanted;
    }
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
    // The perspective is taken out only when the eye is all but overhead, and by then
    // the eye has come in to where the lens frames the ground exactly as the plan
    // will (`stand_off`): no step in size at the moment one becomes the other.
    let overhead = doc.plan && (pitch - 1.5607).abs() < 0.004 && yaw.abs() < 0.02;
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
        let (lens, back) = (LENS, stand_off(pitch, distance));
        let eye = at + Vec3::new(sy * cp, sp, cy * cp) * back;
        let up = if pitch > 1.5 { Vec3::NEG_Z } else { Vec3::Y };
        *t = Transform::from_translation(eye).looking_at(at, up);
        if !matches!(&*projection, Projection::Perspective(p) if (p.fov - lens).abs() < 1e-4) {
            *projection = Projection::from(PerspectiveProjection { fov: lens, near: 0.5, far: 40_000.0, ..default() });
        }
    }
    // The sky stays round the eye; a plan has none.
    for (mut dome, mut seen) in &mut sky {
        dome.translation = Vec3::new(t.translation.x, 0.0, t.translation.z);
        let want = if overhead { Visibility::Hidden } else { Visibility::Visible };
        if *seen != want {
            *seen = want;
        }
    }
}

/// Whether the pointer is over a panel, as picking sees it; and whether the left button's
/// last press became a drag (a drag is not a click). Kept where any system can ask.
pub static OVER_PANEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static DRAGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Did the left button's last press slide or turn the view? Then its release selects
/// nothing and lets go of nothing.
pub fn dragged() -> bool {
    DRAGGED.load(std::sync::atomic::Ordering::Relaxed)
}

/// The left button went down: whatever it does next, it has not dragged yet.
pub fn pressed(press: On<PointerPress>) {
    if press.button == PointerButton::Primary {
        DRAGGED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The left button is down and the pointer has gone more than a few dots: a drag. The
/// pointer's own account of it comes before the click that ends it, even when the
/// press, the drag and the release all land in one slow frame, so a quick flick that
/// starts on a wall slides the view and selects nothing (J19-11).
pub fn dragging(drag: On<PointerDrag>) {
    if drag.button == PointerButton::Primary && drag.distance.length() > 5.0 {
        DRAGGED.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Is this place in the window over the model, not over a panel? Panels float and can
/// be put away, so this is asked of what the pointer is over, not of fixed rectangles.
pub fn over_model_at(window: &Window, p: Vec2) -> bool {
    let (w, h) = (window.width(), window.height());
    p.x >= 0.0 && p.y >= 0.0 && p.x <= w && p.y <= h && !OVER_PANEL.load(std::sync::atomic::Ordering::Relaxed)
}

/// What the hand has hold of while a button is down.
#[derive(Clone, Copy, PartialEq)]
enum Hold {
    /// The ground: the place grabbed stays under the pointer.
    Ground,
    /// A turn about the place that was under the pointer at the press.
    Turn,
}

/// How the view feels under the hand: what a drag has hold of, and what carries on
/// after it is let go. (DECISIONS.md call 63 has where each figure comes from.)
#[derive(Resource, Default)]
pub struct Feel {
    hold: Option<(Hold, Vec3)>,
    /// Where the pointer was at the press, and how far it has gone since, in dots.
    pressed_at: Vec2,
    gone: f32,
    /// Carrying on after a release: the ground sliding (feet a second), and a turn
    /// (yaw and pitch, radians a second) about a place.
    slide: Vec3,
    spin: (Vec2, Vec3),
    /// When the left button last went down and where, for a double-click.
    last_press: Option<(f32, Vec2)>,
    /// Where the pointer was a frame ago.
    was: Option<Vec2>,
}

/// How far the eye stands from what it looks at. As far as the view says, until the eye
/// is nearly overhead; from there it comes in, to where the lens shows as much of the
/// ground as the plan does at that distance, so that a plan and a view from straight
/// above are the same size. (A turn in 3D stops short of where this begins.)
fn stand_off(pitch: f32, distance: f32) -> f32 {
    let t = ((pitch - 1.525) / (1.5567 - 1.525)).clamp(0.0, 1.0);
    let k = t * t * (3.0 - 2.0 * t);
    distance * (1.0 - k * (1.0 - PLAN_HEIGHT / (2.0 * (LENS / 2.0).tan())))
}

/// The ray under a place in the window, from where the view is going (not where it has
/// got to): its eye and its way.
fn ray_at(orbit: &Orbit, window: &Window, cursor: Vec2) -> (Vec3, Vec3) {
    let (sy, cy) = orbit.yaw.sin_cos();
    let (sp, cp) = orbit.pitch.sin_cos();
    let back = Vec3::new(sy * cp, sp, cy * cp);
    let eye = orbit.target + back * stand_off(orbit.pitch, orbit.distance);
    let right = Vec3::new(cy, 0.0, -sy);
    let up = right.cross(-back);
    let (w, h) = (window.width().max(1.0), window.height().max(1.0));
    let half = (LENS / 2.0).tan();
    let (nx, ny) = (cursor.x / w * 2.0 - 1.0, 1.0 - cursor.y / h * 2.0);
    (eye, (-back + right * (nx * half * w / h) + up * (ny * half)).normalize())
}

/// Where a ray meets the level plane at a height; none when it runs away from it or
/// nearly along it.
fn on_level(eye: Vec3, way: Vec3, height: f32) -> Option<Vec3> {
    if way.y.abs() < 0.035 {
        return None;
    }
    let t = (height - eye.y) / way.y;
    (t > 0.0 && t < 50_000.0).then(|| eye + way * t)
}

/// Turn the view about a place: the place stays where it is on the screen.
fn turn_about(orbit: &mut Orbit, pivot: Vec3, yaw: f32, pitch: f32) {
    let frame = |yaw: f32, pitch: f32| {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let back = Vec3::new(sy * cp, sp, cy * cp);
        let right = Vec3::new(cy, 0.0, -sy);
        (right, right.cross(-back), back)
    };
    let to = (orbit.pitch + pitch).clamp(0.06, 1.52);
    let (r0, u0, b0) = frame(orbit.yaw, orbit.pitch);
    let (r1, u1, b1) = frame(orbit.yaw + yaw, to);
    // The target, as the view sees it from the pivot, is carried round with the view.
    let v = orbit.target - pivot;
    orbit.target = pivot + r1 * v.dot(r0) + u1 * v.dot(u0) + b1 * v.dot(b0);
    orbit.yaw += yaw;
    orbit.pitch = to;
}

/// Whether a placed thing is in the hand: then the left button's drag moves it, not the view.
static CARRYING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A placed thing picked up: which, where on it the hand took hold, and how far it has
/// been carried from where it stood (feet east and north).
#[derive(Resource, Default)]
pub struct Carry {
    path: Option<String>,
    grabbed: Vec3,
    gone: [f64; 2],
    said: String,
    /// Whether the clashes were marked on the model before it was picked up: they are
    /// marked while it is carried, and put back as they were when it is let go (J19-28).
    marked: bool,
}

/// Is this a thing that stands where it is put and can be picked up?
fn stands(kind: Kind) -> bool {
    matches!(kind, Kind::Equipment | Kind::Fixture | Kind::Furnishing)
}

/// A short distance as a tape reads it: inches to the sixteenth under a foot ("1 11/16
/// in"), feet and inches from there up.
fn near(feet: f64) -> String {
    if feet.abs() < 1.0 {
        plumbline_world::units::show_length(&plumbline_world::units::Len::One((feet * 12.0 * 16.0).round() / 16.0), "in")
    } else {
        plumbline_world::units::feet_as_ft_in(feet, 16)
    }
}

/// What a thing is in the same space as, as the line along the bottom says it.
fn in_what(snap: &Snapshot, path: &str) -> Option<String> {
    let Some(Ok(found)) = &snap.clashes else { return None };
    let with: Vec<String> = found
        .found
        .iter()
        .filter(|c| c.how == plumbline_world::building::clash::ClashKind::Hard && !c.through && (c.a == path || c.b == path))
        .map(|c| format!("{}{}", if c.a == path { &c.b_name } else { &c.a_name }, c.by.map(|by| format!(" by {}", near(by))).unwrap_or_default()))
        .collect();
    (!with.is_empty()).then(|| with.join(", and "))
}

/// Picking a placed thing up and carrying it. Press the left button on the thing that
/// is selected (a fixture, a piece of equipment, a furnishing) and it goes with the
/// hand over the floor: the place on it that was grabbed stays under the pointer. What
/// it is in the same space as is said and marked as it goes, and clears as it clears.
/// Let go and it stands there; the whole carry is one step to undo. Where it stands is
/// then Assumed: a hand in a picture is not a tape.
#[allow(clippy::too_many_arguments)]
pub fn carry(mut carry: ResMut<Carry>, mut doc: ResMut<Doc>, snap: Res<Snapshot>, orbit: Res<Orbit>, buttons: Res<ButtonInput<MouseButton>>, window: Query<&Window, With<PrimaryWindow>>, hover: Res<bevy::picking::hover::HoverMap>, panels: Query<(), With<Node>>) {
    use std::sync::atomic::Ordering::Relaxed;
    let Ok(window) = window.single() else { return };
    let cursor = window.cursor_position();
    let hits = hover.get(&bevy::picking::pointer::PointerId::Mouse);
    let on_panel = hits.is_some_and(|h| h.keys().any(|e| panels.contains(*e)));
    let free = doc.entry.is_none() && doc.draw.tool.is_none() && !doc.choosing && !doc.walk;
    // An item in hand: a click on the model sets one down there, on the level on show.
    if let (true, Some(item), false) = (buttons.just_pressed(MouseButton::Left) && free && !on_panel, doc.placing.clone(), carry.path.is_some()) {
        let at = hits.and_then(|h| h.values().filter_map(|hit| hit.position.map(|p| (hit.depth, p))).min_by(|a, b| a.0.total_cmp(&b.0))).map(|(_, p)| p);
        let level = doc.show.storey.clone().or_else(|| plumbline_world::building::storey_totals(&mut doc.job).into_iter().min_by(|a, b| a.elevation.unwrap_or(f64::MAX).total_cmp(&b.elevation.unwrap_or(f64::MAX))).map(|s| s.path));
        let tidy = |x: f64| (x * 1000.0).round() / 1000.0;
        let made = match (at, level) {
            (None, _) => Err("click on the floor or the ground, where it is to stand".to_string()),
            (_, None) => Err("there is no level to stand it on: add a level first".to_string()),
            (Some(p), Some(level)) => {
                let o = doc.origin;
                plumbline_world::editing::place_item(&mut doc.job, &level, &item, [tidy(o[0] + f64::from(p.x)), tidy(o[1] - f64::from(p.z))], "0 ft", plumbline_world::editing::Entered { status: plumbline_world::value::Status::Assumed, source: "set down by hand in the model" })
            }
        };
        match made {
            Ok(path) => {
                doc.edit("Place");
                doc.placing = None;
                doc.selected = Some(path.clone());
                // One with a size that is not all known has no shape: that is said at
                // the moment it is set down, not left to be found (J19-25, J19-26).
                let shapeless = doc.job.find(&path).and_then(|e| doc.job.world.get::<plumbline_world::Placed>(e)).filter(|p| p.size_ft().is_none()).map(|p| p.size.source.clone());
                match shapeless {
                    Some(why) => doc.say(Tone::Warned, format!("Set one down, but it cannot be drawn or tested for clashes yet: {why}. Select it under Items and measure what is missing; this one will follow.")),
                    None => doc.say(Tone::Done, "Set one down. Pick it up to move it: press on it and carry it."),
                }
            }
            Err(why) => crate::tools::cant(&mut doc, "set it down", why),
        }
        // This press was the setting down: it selects nothing else and lets go of nothing.
        DRAGGED.store(true, Relaxed);
        doc.redraw();
        return;
    }
    if buttons.just_pressed(MouseButton::Left) && carry.path.is_none() && free && !on_panel {
        // The thing under the pointer, if it is one that stands where it is put. It need
        // not have been selected first: a person reaching for the vanity takes hold of
        // the vanity, and does not slide the world (J19-34). The press selects it.
        let under = doc.hover.clone().filter(|path| {
            let kind = doc.job.find(path).and_then(|e| doc.job.world.get::<Kind>(e).copied());
            kind.is_some_and(stands)
        });
        let hold = hits.and_then(|h| h.values().filter_map(|hit| hit.position.map(|p| (hit.depth, p))).min_by(|a, b| a.0.total_cmp(&b.0))).map(|(_, p)| p);
        if let (Some(path), Some(at)) = (under, hold) {
            if doc.selected.as_deref() != Some(path.as_str()) {
                doc.select(&path);
            }
            *carry = Carry { path: Some(path), grabbed: at, gone: [0.0, 0.0], said: String::new(), marked: doc.show.clashes };
            CARRYING.store(true, Relaxed);
            // What it is in is marked on the model while it is carried, and after.
            if !doc.show.clashes {
                doc.show.clashes = true;
                doc.touch();
            }
        }
    }
    let Some(path) = carry.path.clone() else { return };
    // (What it is called: the panel's own account of it, once the panel has caught up.)
    let name = snap.selected.as_ref().filter(|s| s.path == path).map(|s| s.name.clone()).or_else(|| doc.job.find(&path).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone()))).unwrap_or_else(|| path.clone());
    if buttons.pressed(MouseButton::Left) {
        // Where the place grabbed should be now: under the pointer, at the height it was.
        // On plan a dot covers the same ground everywhere. In 3D it is where the ray
        // under the pointer meets that height; unless something stands in the way
        // first (the pointer is on a wall's face, above that height): then it is where
        // the hand is, at the foot of what it is on, and not on the floor far behind
        // it (J19-32).
        let under = if doc.plan {
            cursor.map(|c| {
                let per_px = orbit.distance * PLAN_HEIGHT / window.height().max(1.0);
                let off = c - Vec2::new(window.width(), window.height()) / 2.0;
                Vec3::new(orbit.target.x + off.x * per_px, carry.grabbed.y, orbit.target.z + off.y * per_px)
            })
        } else {
            let in_the_way = doc.hover.as_deref().filter(|h| *h != path.as_str()).and(doc.pointer).map(|p| to_view(doc.origin, p)).filter(|p| p.y > carry.grabbed.y + 0.25);
            in_the_way.map(|p| Vec3::new(p.x, carry.grabbed.y, p.z)).or_else(|| cursor.and_then(|c| { let (eye, way) = ray_at(&orbit, window, c); on_level(eye, way, carry.grabbed.y) }))
        };
        if let Some(to) = under {
            // East is the view's x; north is against its z.
            // (To the thousandth of a foot: a hand in a picture is no finer than that.)
            let tidy = |x: f32| (f64::from(x) * 1000.0).round() / 1000.0;
            let want = [tidy(to.x - carry.grabbed.x), -tidy(to.z - carry.grabbed.z)];
            let step = [want[0] - carry.gone[0], want[1] - carry.gone[1]];
            if step[0].hypot(step[1]) > 0.01 && want[0].hypot(want[1]) < 500.0 {
                match plumbline_world::editing::move_by(&mut doc.job, &path, step[0], step[1], plumbline_world::editing::Entered { status: plumbline_world::value::Status::Assumed, source: "by hand in the model" }) {
                    Ok(()) => {
                        carry.gone = want;
                        DRAGGED.store(true, Relaxed);
                        doc.redraw();
                    }
                    Err(why) => {
                        doc.say(Tone::Refused, format!("Can't move {name} there: {why}"));
                        doc.touch();
                    }
                }
            }
        }
        // What it is in now, said as it changes.
        let said = match in_what(&snap, &path) {
            Some(what) => format!("{name}: in the same space as {what}. Keep going, or let go and it stands here."),
            None => format!("{name}: clear of everything. Let go and it stands here."),
        };
        if said != carry.said && carry.gone != [0.0, 0.0] {
            doc.say(if said.contains("in the same space") { Tone::Refused } else { Tone::Done }, said.clone());
            doc.touch();
            carry.said = said;
        }
    } else {
        // Let go: one step, however far it was carried.
        if carry.gone != [0.0, 0.0] {
            doc.edit("Move");
            let (east, north) = (carry.gone[0], carry.gone[1]);
            let went = format!("{} {}, {} {}", plumbline_world::units::feet_as_ft_in(east.abs(), 16), if east >= 0.0 { "east" } else { "west" }, plumbline_world::units::feet_as_ft_in(north.abs(), 16), if north >= 0.0 { "north" } else { "south" });
            let said = match in_what(&snap, &path) {
                Some(what) => format!("Moved {name} {went}. It is in the same space as {what}."),
                None => format!("Moved {name} {went}. It is clear of everything."),
            };
            doc.say(if said.contains("in the same space") { Tone::Refused } else { Tone::Done }, said);
            doc.redraw();
        }
        // The clash marks go back to how they were, unless it was left in something.
        if !carry.marked && doc.show.clashes && in_what(&snap, &path).is_none() {
            doc.show.clashes = false;
            doc.redraw();
        }
        *carry = Carry::default();
        CARRYING.store(false, Relaxed);
    }
}

/// Moving round the model. The wheel dives at what is under the pointer. A drag with
/// the left button (when no tool is in hand) or the middle one slides the ground with
/// the hand: the place grabbed stays under the pointer. A drag with the right button
/// turns the view about the place under the pointer. Let go while moving and it
/// carries on and eases to a stop. A double-click flies to the place clicked. On foot
/// the same keys walk and the right button looks round.
#[allow(clippy::too_many_arguments)]
pub fn orbit_input(mut orbit: ResMut<Orbit>, mut feel: ResMut<Feel>, buttons: Res<ButtonInput<MouseButton>>, scroll: Res<AccumulatedMouseScroll>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, doc: Res<Doc>, window: Query<&Window, With<PrimaryWindow>>, hover: Res<bevy::picking::hover::HoverMap>, panels: Query<(), With<Node>>, mut presses: MessageReader<bevy::input::mouse::MouseButtonInput>) {
    use std::sync::atomic::Ordering::Relaxed;
    // How many times the left button went down since the last frame: on a slow machine
    // both presses of a double-click can land in one frame.
    let downs = presses.read().filter(|p| p.button == MouseButton::Left && p.state == bevy::input::ButtonState::Pressed).count();
    let Ok(window) = window.single() else { return };
    // What the pointer is over: a panel, or a place on the model.
    let hits = hover.get(&bevy::picking::pointer::PointerId::Mouse);
    let on_panel = hits.is_some_and(|h| h.keys().any(|e| panels.contains(*e)));
    OVER_PANEL.store(on_panel, Relaxed);
    let cursor = window.cursor_position();
    // How far the pointer went since the last frame, from where it is: a pointer that
    // is put somewhere (a tablet, a remote desktop, a test rig) sends no "moved by", and
    // a drag made of such moves is still a drag.
    let moved = match (cursor, feel.was) {
        (Some(now), Some(then)) => now - then,
        _ => Vec2::ZERO,
    };
    feel.was = cursor;
    let inside = cursor.is_some() && !on_panel;
    let under = |orbit: &Orbit| -> Option<Vec3> {
        let nearest = hits.and_then(|h| h.values().filter_map(|hit| hit.position.map(|p| (hit.depth, p))).min_by(|a, b| a.0.total_cmp(&b.0))).map(|(_, p)| p);
        nearest.or_else(|| cursor.and_then(|c| { let (eye, way) = ray_at(orbit, window, c); on_level(eye, way, 0.0) }))
    };
    let dt = time.delta_secs().min(0.1);
    // A speed is what moved over how long the frame really took, however slow the frame.
    let took = time.delta_secs().max(1.0e-3);
    let free = doc.entry.is_none() && doc.draw.tool.is_none() && !doc.choosing && doc.placing.is_none();
    let left = MouseButton::Left;

    // A press takes hold; any press stops what was carrying on.
    if buttons.any_just_pressed([left, MouseButton::Middle, MouseButton::Right]) {
        feel.slide = Vec3::ZERO;
        feel.spin = (Vec2::ZERO, Vec3::ZERO);
        feel.pressed_at = cursor.unwrap_or_default();
        feel.gone = 0.0;
        feel.hold = None;
        if inside && !doc.walk {
            let grab = under(&orbit);
            if buttons.just_pressed(MouseButton::Right) && !doc.plan {
                feel.hold = grab.map(|g| (Hold::Turn, g));
            } else if buttons.just_pressed(MouseButton::Middle) || (buttons.just_pressed(MouseButton::Right) && doc.plan) || (buttons.just_pressed(left) && free && !CARRYING.load(Relaxed)) {
                feel.hold = grab.map(|g| (Hold::Ground, g));
            }
            // A second press of the left button in the same place flies to it.
            if buttons.just_pressed(left) && free && !CARRYING.load(Relaxed) {
                let now = time.elapsed_secs();
                // Within half a second (the usual double-click), or within a frame and a
                // half where frames are slower than that.
                let window = (time.delta_secs() * 1.5).max(0.5);
                let again = downs >= 2 || feel.last_press.is_some_and(|(then, at)| now - then < window && cursor.is_some_and(|c| c.distance(at) < 8.0));
                feel.last_press = cursor.map(|c| (now, c));
                // Only to a place where the job is: a double-click on the sky, or on
                // ground out at the horizon, flew the eye to nothing (J19-50).
                let near_the_job = |to: &Vec3| to.distance(orbit.home.0) < (orbit.home.1 * 4.0).max(120.0);
                if let (true, Some(to)) = (again, grab.filter(near_the_job)) {
                    orbit.target = to;
                    orbit.distance = (orbit.distance * 0.45).clamp(10.0, 6000.0);
                    feel.hold = None;
                    feel.last_press = None;
                }
            }
        }
    }
    let held = buttons.pressed(left) || buttons.pressed(MouseButton::Middle) || buttons.pressed(MouseButton::Right);
    // Let go this frame: what the pointer did before the button came up still counts,
    // or on a slow frame the end of a flick is lost with the flick.
    let let_go = !held && buttons.any_just_released([left, MouseButton::Middle, MouseButton::Right]);
    // What a dot on the screen covers of the ground where the view is looking.
    let dot = orbit.distance * 2.0 * (LENS / 2.0).tan() / window.height().max(1.0);
    feel.gone += moved.length();
    // The left button's drag is told from a click by how far it went.
    if buttons.pressed(left) && feel.gone > 5.0 && feel.hold.is_some() {
        DRAGGED.store(true, Relaxed);
    }
    let (sy, cy) = if doc.plan { (0.0, 1.0) } else { orbit.yaw.sin_cos() };
    let right = Vec3::new(cy, 0.0, -sy);
    let toward = Vec3::new(sy, 0.0, cy);
    if doc.walk {
        // On foot the right button looks round.
        if buttons.pressed(MouseButton::Right) {
            orbit.yaw += moved.x * 0.006;
            orbit.pitch = (orbit.pitch + moved.y * 0.006).clamp(-1.3, 1.3);
        }
    } else if let (true, Some((hold, grabbed))) = (held || let_go, feel.hold) {
        match hold {
            Hold::Turn => {
                // A full window height of drag is a full turn (three.js's rule).
                let per_px = std::f32::consts::TAU / window.height().max(1.0);
                let (yaw, pitch) = (-moved.x * per_px, moved.y * per_px);
                turn_about(&mut orbit, grabbed, yaw, pitch);
                feel.spin = (feel.spin.0.lerp(Vec2::new(yaw, pitch) / took, 0.5), grabbed);
            }
            Hold::Ground => {
                let shift = if doc.plan {
                    // On plan a dot covers the same ground everywhere: exact.
                    let per_px = orbit.distance * PLAN_HEIGHT / window.height().max(1.0);
                    right * (-moved.x * per_px) + toward * (-moved.y * per_px)
                } else {
                    // The place grabbed is put back under the pointer.
                    cursor.and_then(|c| { let (eye, way) = ray_at(&orbit, window, c); on_level(eye, way, grabbed.y) }).map(|now| grabbed - now).filter(|s| s.length() < orbit.distance * 4.0).unwrap_or(Vec3::ZERO)
                };
                orbit.target += shift;
                feel.slide = feel.slide.lerp(shift / took, 0.5);
            }
        }
    }
    if !held && !doc.walk {
        if feel.hold.take().is_some() {
            // Let go: what it was doing carries on only if it was still going, faster
            // than twenty dots a second.
            if feel.slide.length() < dot * 20.0 {
                feel.slide = Vec3::ZERO;
            }
            if feel.spin.0.length() < 0.15 {
                feel.spin.0 = Vec2::ZERO;
            }
        }
        // It dies away as Cesium's does: e to the minus t over 0.4 seconds.
        let fade = (-dt * 2.5).exp();
        if feel.slide != Vec3::ZERO {
            let step = feel.slide * dt;
            orbit.target += step;
            feel.slide *= fade;
            if feel.slide.length() < dot * 8.0 {
                feel.slide = Vec3::ZERO;
            }
        }
        if feel.spin.0 != Vec2::ZERO && !doc.plan {
            let (rate, about) = feel.spin;
            turn_about(&mut orbit, about, rate.x * dt, rate.y * dt);
            feel.spin.0 *= fade;
            if feel.spin.0.length() < 0.02 {
                feel.spin.0 = Vec2::ZERO;
            }
        }
    }
    // PLUMBLINE_TRACE=1 says what the wheel sent, for telling a mouse from a test rig.
    if scroll.delta.y != 0.0 && std::env::var_os("PLUMBLINE_TRACE").is_some() {
        eprintln!("wheel: {:?} {} ({} presses of the left button this frame)", scroll.unit, scroll.delta.y, downs);
    }
    if inside && scroll.delta.y != 0.0 && !doc.walk {
        // A notch of the wheel is a tenth nearer or further; a touchpad's dots are a
        // hundredth of a notch each (three.js's measure of a wheel).
        let notches = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y / 100.0 } else { scroll.delta.y }.clamp(-3.0, 3.0);
        let was = orbit.distance;
        // Out stops where the building is still a building on the screen, about a
        // sixth of the size F frames it at (J19-15: forty notches out it shrank to
        // nothing and only a name tag was left in an empty view).
        let furthest = (orbit.home.1 * 6.0).clamp(60.0, 6000.0).max(was);
        orbit.distance = (was * 0.9_f32.powf(notches)).clamp(3.0, furthest);
        let by = orbit.distance / was;
        if doc.plan {
            // On plan the place under the pointer stays under it, which is how a
            // dimension's end on a drawing is closed in on.
            if let Some(cursor) = cursor {
                let per_px = was * PLAN_HEIGHT / window.height().max(1.0);
                let off = cursor - Vec2::new(window.width(), window.height()) / 2.0;
                let place = orbit.target + right * (off.x * per_px) + toward * (off.y * per_px);
                orbit.target = place + (orbit.target - place) * by;
            }
        } else if let Some(place) = under(&orbit).filter(|p| p.distance(orbit.target) < was * 6.0) {
            // In 3D the same: the eye and what it looks at both close on the place
            // under the pointer, so the place stays under the pointer.
            orbit.target = place + (orbit.target - place) * by;
        } else if let Some(cursor) = cursor {
            // Nothing near is under the pointer (sky, or ground out at the horizon):
            // the view still goes the way the pointer points, over the ground, and
            // not at whatever happens to be in the middle (J19-49: it ran onto the
            // roof of the house beside the pointer).
            let (eye, way) = ray_at(&orbit, window, cursor);
            let flat = Vec3::new(way.x, 0.0, way.z).normalize_or_zero();
            let place = Vec3::new(eye.x, orbit.target.y, eye.z) + flat * was;
            orbit.target = place + (orbit.target - place) * by;
        }
        feel.slide = Vec3::ZERO;
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
                // How sure it is shows on the mark (J19-20: every mark was one red). Between
                // things that are Known it is a finding and is red; where either rests on
                // something not yet measured it is a question for the field, and amber.
                let colour = if c.status == plumbline_world::value::Status::Known { theme::clash() } else { theme::clash_to_verify() };
                for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                    highlight.line(at - axis * 0.75, at + axis * 0.75, colour);
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
