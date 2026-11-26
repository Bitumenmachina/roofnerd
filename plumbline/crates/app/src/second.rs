//! The second window as a plan (call 222, part C; call 224). The estimate is what it opens on. With "Plan" it shows a level of the job or a drawing
//! sheet from straight above, with its own wheel, drag, level and layers; with "Two plans", two side by side. The plans are drawn by what draws the
//! model window's plan (`scene::build_house` and `scene::draw_view`), each half on its own render layer so that its camera sees it and no other does;
//! its wheel, drag and flight are the model window's own figures (`scene::plan_wheel`, `plan_drag`, `plan_flight`); what the pointer catches in it is
//! what the model window's pointer catches (`tools::level_scene`, `tools::catch_at`). What the second window shows is the program's setting
//! (`screen::Settings`), never the job's: looking at plans changes nothing in the job.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, RenderTarget, ScalingMode, Viewport};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::NotShadowCaster;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::{PointerButton, PointerId};
use bevy::prelude::*;
use bevy::window::WindowRef;

use crate::model::{to_view, Doc, Show, Snapshot, Tone};
use crate::panels::{one_line, pretty, tally_chips, wordmark, Cameras};
use crate::scene::{self, Drawing, Orbit, Pen, SceneMesh, SlotMesh, Solids, View};
use crate::sheet::SheetWindow;
use crate::theme::{self, rounded, text, Fonts, Weight};
use plumbline_world::editing::snap;

/// What the second window shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    #[default]
    Estimate,
    Plan,
    Two,
}

impl Mode {
    /// How many plans stand in the window.
    pub fn halves(self) -> usize {
        match self {
            Mode::Estimate => 0,
            Mode::Plan => 1,
            Mode::Two => 2,
        }
    }
    /// As the settings file writes it.
    fn word(self) -> &'static str {
        match self {
            Mode::Estimate => "estimate",
            Mode::Plan => "plan",
            Mode::Two => "two plans",
        }
    }
    fn from_word(w: &str) -> Option<Mode> {
        [Mode::Estimate, Mode::Plan, Mode::Two].into_iter().find(|m| m.word() == w)
    }
}

/// What a half shows: a level of the job, or a drawing sheet, each by its path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Look {
    Level(String),
    Sheet(String),
}

impl Look {
    fn said(&self) -> String {
        match self {
            Look::Level(p) => format!("level {p}"),
            Look::Sheet(p) => format!("sheet {p}"),
        }
    }
    fn from_said(v: &str) -> Option<Look> {
        match v.split_once(' ') {
            Some(("level", p)) if !p.trim().is_empty() => Some(Look::Level(p.trim().to_string())),
            Some(("sheet", p)) if !p.trim().is_empty() => Some(Look::Sheet(p.trim().to_string())),
            _ => None,
        }
    }
}

/// The layers a plan has switches for, by the words the model window uses for the same.
pub const LAYERS: [&str; 5] = ["Walls and floors", "Roof", "Mechanical", "Electrical", "Plumbing"];

fn layer_on(show: &Show, i: usize) -> bool {
    [show.architecture, show.roof, show.mechanical, show.electrical, show.plumbing][i]
}

fn layer_set(show: &mut Show, i: usize, on: bool) {
    match i {
        0 => show.architecture = on,
        1 => show.roof = on,
        2 => show.mechanical = on,
        3 => show.electrical = on,
        _ => show.plumbing = on,
    }
}

/// What of the second window is kept between runs: what it shows, what each plan looks at, and each plan's layers put away (by their words).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Kept {
    pub mode: Mode,
    pub looks: [Option<Look>; 2],
    pub away: [Vec<String>; 2],
    /// Where each plan was left by a hand: its middle in the view's own feet (across, down) and its distance. None: it is fitted to what it looks at.
    pub views: [Option<[f64; 3]>; 2],
}

impl Kept {
    /// The lines of the settings file that are the second window's.
    pub fn text(&self) -> String {
        let mut out = format!("second window = {}\n", self.mode.word());
        for n in 0..2 {
            if let Some(look) = &self.looks[n] {
                out.push_str(&format!("plan {} shows = {}\n", n + 1, look.said()));
            }
            if !self.away[n].is_empty() {
                out.push_str(&format!("plan {} layers away = {}\n", n + 1, self.away[n].join(", ")));
            }
            if let Some(v) = &self.views[n] {
                out.push_str(&format!("plan {} view = {} {} {}\n", n + 1, v[0], v[1], v[2]));
            }
        }
        out
    }
    /// A settings line, if it is one of the second window's; false when it is not (or not understood: it is then passed over).
    pub fn read(&mut self, name: &str, value: &str) -> bool {
        if name == "second window" {
            if let Some(m) = Mode::from_word(value) {
                self.mode = m;
            }
            return true;
        }
        for n in 0..2 {
            if name == format!("plan {} shows", n + 1) {
                self.looks[n] = Look::from_said(value);
                return true;
            }
            if name == format!("plan {} view", n + 1) {
                let v: Vec<f64> = value.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                self.views[n] = (v.len() == 3 && v.iter().all(|x| x.is_finite() && x.abs() < 1.0e7) && v[2] > 0.0).then(|| [v[0], v[1], v[2]]);
                return true;
            }
            if name == format!("plan {} layers away", n + 1) {
                self.away[n] = value.split(',').map(|w| w.trim()).filter(|w| LAYERS.contains(w)).map(String::from).collect();
                return true;
            }
        }
        false
    }
}

/// A place on the window's face, in dots.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Area {
    fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }
}

/// The line between two halves, in dots.
pub const LINE: f32 = 3.0;

/// How much of the window's top the controls take, in the units the lettering scales (a bar, and a row for each plan's chooser).
pub const TOP: f32 = 96.0;

/// Where the plans stand in a window of this size, under `top` dots of controls.
pub fn half_rects(mode: Mode, size: Vec2, top: f32) -> Vec<Area> {
    let h = (size.y - top).max(0.0);
    match mode {
        Mode::Estimate => Vec::new(),
        Mode::Plan => vec![Area { x: 0.0, y: top, w: size.x, h }],
        Mode::Two => {
            let w = (size.x - LINE) / 2.0;
            vec![Area { x: 0.0, y: top, w, h }, Area { x: w + LINE, y: top, w, h }]
        }
    }
}

/// Which half a point of the window is in.
pub fn half_at(areas: &[Area], p: Vec2) -> Option<usize> {
    areas.iter().position(|a| a.contains(p))
}

/// Where a view of this much ground (low and high corner, in feet) stands, and how far (`Orbit::distance`), to be whole in a half `w` by `h` dots,
/// with a margin of 8 in a hundred each side. Ground less than 8 ft each way is taken as 8 ft.
pub fn fit(lo: [f64; 2], hi: [f64; 2], w: f32, h: f32) -> ([f64; 2], f32) {
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let (ex, ey) = (((hi[0] - lo[0]) as f32).max(8.0), ((hi[1] - lo[1]) as f32).max(8.0));
    let tall = (ey * 1.16).max(ex * 1.16 * h.max(1.0) / w.max(1.0));
    (centre, tall / scene::PLAN_HEIGHT)
}

// ---------------------------------------------------------------- what the second window holds

/// Which list is open under a half's controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Open {
    #[default]
    Closed,
    Levels,
    Layers,
}

/// One plan of the second window: what it looks at, its own layers, its own view, and what it has drawn.
pub struct Half {
    pub look: Option<Look>,
    /// Its own layers: a copy of what the model window had on when it was opened, then its own.
    pub layers: Option<Show>,
    /// Layers kept as put away, to be applied when the half's layers are first made.
    pending_away: Vec<String>,
    /// A view kept from last time, to be put back when the plan is first fitted.
    pending_view: Option<[f64; 3]>,
    pub orbit: Orbit,
    pub drawing: Drawing,
    /// What the house was drawn for: the job's drawing, the drawings read, what it looks at, its layers.
    built: Option<(u64, usize, Look, Show)>,
    /// What the view was fitted for: a hand has not moved it since.
    fitted: Option<(Look, u32, u32)>,
    pub touched: bool,
    pub area: Area,
    pub open: Open,
    /// How many things are drawn in it.
    pub shown: usize,
    /// What the pointer catches in it, and what is said of it.
    pub catch: Option<snap::Found>,
    pub said: String,
    scene: Option<(u64, String, usize, snap::Scene)>,
    slide: Vec3,
    was: Option<Vec2>,
    gone: f32,
    last_press: Option<(f32, Vec2)>,
}

impl Default for Half {
    fn default() -> Half {
        Half { look: None, layers: None, pending_away: Vec::new(), pending_view: None, orbit: Orbit::new(), drawing: Drawing::default(), built: None, fitted: None, touched: false, area: Area::default(), open: Open::Closed, shown: 0, catch: None, said: String::new(), scene: None, slide: Vec3::ZERO, was: None, gone: 0.0, last_press: None }
    }
}

/// The second window's state. It lives in the `Doc` as a way of looking, and is never saved with the job.
#[derive(Default)]
pub struct Second {
    pub mode: Mode,
    pub halves: [Half; 2],
    /// Which half a hand has hold of.
    hold: Option<usize>,
    /// How many things the model window draws.
    pub model_shown: usize,
}

impl Second {
    /// What is kept of it.
    pub fn kept(&self) -> Kept {
        let mut k = Kept { mode: self.mode, ..Kept::default() };
        for n in 0..2 {
            k.looks[n] = self.halves[n].look.clone();
            // Where a hand has left the view, to a hundredth of a foot (and not each frame's ease): it is put back as it was left.
            let h = &self.halves[n];
            k.views[n] = if h.touched { Some([(h.orbit.target.x as f64 * 100.0).round() / 100.0, (h.orbit.target.z as f64 * 100.0).round() / 100.0, (h.orbit.distance as f64 * 100.0).round() / 100.0]) } else { h.pending_view };
            k.away[n] = match &self.halves[n].layers {
                Some(show) => LAYERS.iter().enumerate().filter(|(i, _)| !layer_on(show, *i)).map(|(_, w)| w.to_string()).collect(),
                None => self.halves[n].pending_away.clone(),
            };
        }
        k
    }
    /// What was kept, put back: each half's layers are made from the model window's when it is first drawn, less what was put away.
    pub fn load(&mut self, k: &Kept) {
        self.mode = k.mode;
        for n in 0..2 {
            let h = &mut self.halves[n];
            h.look = k.looks[n].clone();
            h.open = Open::Closed;
            h.layers = None;
            h.built = None;
            h.pending_away = k.away[n].clone();
            h.pending_view = k.views[n];
            h.fitted = None;
            h.touched = false;
        }
    }
}

// ---------------------------------------------------------------- what a click does

/// "Estimate" or "Plan" at the top of the window.
pub fn set_mode(doc: &mut Doc, mode: Mode) {
    if doc.second.mode != mode {
        doc.second.mode = mode;
        for h in &mut doc.second.halves {
            h.open = Open::Closed;
        }
        doc.touch();
    }
}

/// A half is set to a level or a sheet: its view is fitted again.
pub fn set_look(doc: &mut Doc, n: usize, look: Look) {
    let h = &mut doc.second.halves[n];
    h.look = Some(look);
    h.open = Open::Closed;
    h.touched = false;
    doc.touch();
}

fn toggle_open(doc: &mut Doc, n: usize, which: Open) {
    let h = &mut doc.second.halves[n];
    h.open = if h.open == which { Open::Closed } else { which };
    doc.touch();
}

fn toggle_layer(doc: &mut Doc, n: usize, i: usize) {
    if let Some(show) = &mut doc.second.halves[n].layers {
        let on = layer_on(show, i);
        layer_set(show, i, !on);
        doc.touch();
    }
}

/// What a half can look at, in the order the chooser lists it: the levels from the lowest, then the drawings.
pub fn choices(snap: &Snapshot) -> Vec<(Look, String)> {
    let mut levels: Vec<&plumbline_world::building::StoreyTotals> = snap.storeys.iter().collect();
    levels.sort_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0)));
    let mut out: Vec<(Look, String)> = levels.iter().map(|s| (Look::Level(s.path.clone()), pretty(&s.name))).collect();
    out.extend(snap.drawings.iter().map(|l| (Look::Sheet(l.path.clone()), l.name.clone())));
    out
}

fn name_of(snap: &Snapshot, look: &Look) -> String {
    choices(snap).into_iter().find(|(l, _)| l == look).map(|(_, n)| n).unwrap_or_default()
}

/// What the first plan looks at when nothing has been chosen: the level the model window is looking into, else the lowest level at or above the ground that
/// has walls, else the lowest. The second plan: the next level up, else the same.
fn default_look(snap: &Snapshot, model_level: Option<&str>, n: usize, first: Option<&Look>) -> Option<Look> {
    let mut levels: Vec<&plumbline_world::building::StoreyTotals> = snap.storeys.iter().collect();
    levels.sort_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0)));
    if n == 0 {
        let wanted = model_level.and_then(|p| levels.iter().find(|s| s.path == p)).map(|s| s.path.clone());
        let above: Vec<&&plumbline_world::building::StoreyTotals> = levels.iter().filter(|s| s.elevation.unwrap_or(0.0) >= -0.01).collect();
        let walled = above.iter().find(|s| s.walls.value.unwrap_or(0.0) > 0.0).or(above.first()).map(|s| s.path.clone());
        return wanted.or(walled).or_else(|| levels.first().map(|s| s.path.clone())).map(Look::Level);
    }
    let after = match first {
        Some(Look::Level(p)) => levels.iter().position(|s| s.path == *p).and_then(|i| levels.get(i + 1)).map(|s| s.path.clone()),
        _ => None,
    };
    after
        .or_else(|| match first {
            Some(Look::Level(p)) => Some(p.clone()),
            _ => levels.first().map(|s| s.path.clone()),
        })
        .map(Look::Level)
}

/// Where the ground a look covers lies: a level's is the job's, a sheet's is its page.
fn ground_of(snap: &Snapshot, look: &Look) -> ([f64; 2], [f64; 2]) {
    match look {
        Look::Sheet(p) => {
            if let Some(l) = snap.drawings.iter().find(|l| l.path == *p) {
                let [w, h] = l.drawing.size;
                let c = [l.on_plan([0.0, 0.0]), l.on_plan([w, 0.0]), l.on_plan([w, h]), l.on_plan([0.0, h])];
                let lo = [c.iter().map(|p| p[0]).fold(f64::MAX, f64::min), c.iter().map(|p| p[1]).fold(f64::MAX, f64::min)];
                let hi = [c.iter().map(|p| p[0]).fold(f64::MIN, f64::max), c.iter().map(|p| p[1]).fold(f64::MIN, f64::max)];
                return (lo, hi);
            }
            ([-30.0, -20.0], [30.0, 20.0])
        }
        Look::Level(path) => {
            // A level is fitted to its own walls (a drawing laid beside the house must not shrink it); with none, to everything the job has.
            let mut lo = [f64::MAX; 2];
            let mut hi = [f64::MIN; 2];
            for p in snap.drawn.iter().filter(|p| p.storey.as_deref() == Some(path.as_str())) {
                if let plumbline_world::building::Shape::Wall { a, b, .. } = &p.shape {
                    for q in [a, b] {
                        lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                        hi = [hi[0].max(q[0]), hi[1].max(q[1])];
                    }
                }
            }
            // (with four feet round it for what stands just outside: a condenser, a service)
            if lo[0] <= hi[0] { ([lo[0] - 4.0, lo[1] - 4.0], [hi[0] + 4.0, hi[1] + 4.0]) } else { snap.extent.map(|(lo, hi, _)| (lo, hi)).unwrap_or(([-30.0, -20.0], [30.0, 20.0])) }
        }
    }
}

fn elevation_of(snap: &Snapshot, look: &Look) -> f64 {
    match look {
        Look::Level(p) => snap.storeys.iter().find(|s| s.path == *p).and_then(|s| s.elevation).unwrap_or(0.0),
        Look::Sheet(p) => snap.drawings.iter().find(|l| l.path == *p).and_then(|l| snap.storeys.iter().find(|s| s.path == l.storey)).and_then(|s| s.elevation).unwrap_or(0.0),
    }
}

/// The layers a half starts with: what the model window has on, with its level, systems and see-through left out.
fn fresh_layers(doc: &Doc, away: &[String]) -> Show {
    let mut show = doc.show.clone();
    show.storey = None;
    show.off_levels.clear();
    show.off_systems.clear();
    show.ghost = false;
    show.certainty = false;
    show.clashes = false;
    show.away = [false; 3];
    for (i, word) in LAYERS.iter().enumerate() {
        if away.iter().any(|a| a == word) {
            layer_set(&mut show, i, false);
        }
    }
    show
}

/// The show a half is drawn with: its layers, its level, and no grid when it is a sheet alone.
fn shown_for(layers: &Show, look: &Look) -> (Show, Option<String>) {
    let mut show = layers.clone();
    match look {
        Look::Level(p) => {
            show.storey = Some(p.clone());
            (show, None)
        }
        Look::Sheet(p) => {
            show.ground = false;
            (show, Some(p.clone()))
        }
    }
}

// ---------------------------------------------------------------- the cameras

/// A half's camera: from straight above, no perspective, in the second window, on the half's own layer.
#[derive(Component)]
pub struct HalfCamera(pub u8);

/// The camera that draws the second window's controls, over the plans.
#[derive(Component)]
pub struct UiCamera;

/// What the pointer is over when it is over a control of the second window that stands over a plan (a list, a switch): not the plan.
#[derive(Component)]
pub struct PlanUi;

/// What the pointer has caught, said at the foot of a half.
#[derive(Component)]
pub struct HalfCaption(pub u8);

/// The two plans' cameras, made once; they draw only while their half is in the window.
pub fn spawn_cameras(commands: &mut Commands, window: Entity) {
    for n in 0..2u8 {
        commands.spawn((
            Camera3d::default(),
            Camera { order: 1 + n as isize, is_active: false, clear_color: if n == 0 { ClearColorConfig::Custom(theme::canvas()) } else { ClearColorConfig::None }, ..default() },
            RenderTarget::Window(WindowRef::Entity(window)),
            Projection::from(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: 50.0 }, far: 2000.0, ..OrthographicProjection::default_3d() }),
            Transform::from_xyz(0.0, 300.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
            Msaa::Off,
            Tonemapping::None,
            RenderLayers::layer(View::layer_of(n)),
            HalfCamera(n),
        ));
    }
}

/// Each camera to its half: where it stands on the window, what it looks at, how much ground it takes in. The controls' camera is drawn last and
/// clears nothing when a plan is under it.
#[allow(clippy::type_complexity)]
pub fn cameras(doc: Res<Doc>, windows: Query<&Window, With<SheetWindow>>, mut cams: Query<(&HalfCamera, &mut Camera, &mut Transform, &mut Projection), Without<UiCamera>>, mut ui: Query<&mut Camera, (With<UiCamera>, Without<HalfCamera>)>) {
    let Ok(window) = windows.single() else { return };
    let k = window.scale_factor().max(0.1);
    let (pw, ph) = (window.physical_width(), window.physical_height());
    let on = doc.second.mode.halves();
    for (which, mut camera, mut t, mut projection) in &mut cams {
        let n = which.0 as usize;
        let active = n < on;
        if camera.is_active != active {
            camera.is_active = active;
        }
        if !active {
            continue;
        }
        let h = &doc.second.halves[n];
        let (x, y) = (((h.area.x * k) as u32).min(pw.saturating_sub(1)), ((h.area.y * k) as u32).min(ph.saturating_sub(1)));
        let (w, hh) = (((h.area.w * k) as u32).min(pw - x).max(1), ((h.area.h * k) as u32).min(ph - y).max(1));
        let want = Viewport { physical_position: UVec2::new(x, y), physical_size: UVec2::new(w, hh), ..default() };
        if camera.viewport.as_ref().is_none_or(|v| v.physical_position != want.physical_position || v.physical_size != want.physical_size) {
            camera.viewport = Some(want);
        }
        let (at, distance) = (h.orbit.now.0, h.orbit.now.3);
        *t = Transform::from_translation(Vec3::new(at.x, 300.0, at.z)).looking_at(Vec3::new(at.x, 0.0, at.z), Vec3::NEG_Z);
        let height = distance * scene::PLAN_HEIGHT;
        let same = matches!(&*projection, Projection::Orthographic(o) if matches!(o.scaling_mode, ScalingMode::FixedVertical { viewport_height } if (viewport_height - height).abs() < 1e-3));
        if !same {
            *projection = Projection::from(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: height }, far: 2000.0, ..OrthographicProjection::default_3d() });
        }
    }
    for mut camera in &mut ui {
        let plain = matches!(camera.clear_color, ClearColorConfig::Default);
        if plain != (on == 0) {
            camera.clear_color = if on == 0 { ClearColorConfig::Default } else { ClearColorConfig::None };
        }
    }
}

// ---------------------------------------------------------------- what is drawn

/// Each half's house, drawn again when what is drawn, or what it looks at, or its layers, change.
#[allow(clippy::too_many_arguments)]
pub fn rebuild(
    mut commands: Commands,
    mut doc: ResMut<Doc>,
    snap: Res<Snapshot>,
    plans: Res<crate::plans::Plans>,
    solids: Res<Solids>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut patterns: ResMut<crate::dress::Patterns>,
    main_drawing: Res<Drawing>,
    old: Query<(Entity, &SlotMesh)>,
    model: Query<(), With<SceneMesh>>,
) {
    let mut second = std::mem::take(&mut doc.bypass_change_detection().second);
    second.model_shown = model.iter().count();
    if snap.revision == doc.revision {
        let on = second.mode.halves();
        for n in 0..2 {
            let h = &mut second.halves[n];
            let wanted = if n < on { h.look.clone().zip(h.layers.clone()) } else { None };
            let Some((look, layers)) = wanted else {
                if h.built.is_some() {
                    for (e, m) in &old {
                        if m.0 as usize == n {
                            commands.entity(e).despawn();
                        }
                    }
                    h.built = None;
                    h.drawing = Drawing::default();
                }
                h.shown = 0;
                continue;
            };
            let key = (doc.geometry, plans.read.len(), look.clone(), layers.clone());
            if h.built.as_ref() != Some(&key) {
                for (e, m) in &old {
                    if m.0 as usize == n {
                        commands.entity(e).despawn();
                    }
                }
                let (show, only_sheet) = shown_for(&layers, &look);
                let view = View { slot: Some(n as u8), show, plan: true, walk: false, bench: false, only_sheet };
                let mut drawing = Drawing { reach: main_drawing.reach, ..Drawing::default() };
                scene::build_house(&mut commands, &mut doc, &snap, &plans, &solids, &mut meshes, &mut materials, &mut images, &mut patterns, &mut drawing, &view);
                // The paper: white as it is given, under the lowest thing drawn, as the model window's ground is on plan.
                let lowest = snap.drawn.iter().filter_map(|p| match &p.shape { plumbline_world::building::Shape::Wall { base, .. } => Some(*base as f32), _ => None }).fold(0.0_f32, f32::min);
                let paper = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..default() });
                view.tag(commands.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(24000.0, 24000.0))), MeshMaterial3d(paper), Transform::from_xyz(0.0, lowest - 0.54, 0.0), NotShadowCaster, Pickable::IGNORE)));
                h.drawing = drawing;
                h.built = Some(key);
            }
            h.shown = old.iter().filter(|(_, m)| m.0 as usize == n).count();
        }
    }
    doc.bypass_change_detection().second = second;
}

/// Each half's lines, in its own pens: what draws the model window's plan, with the half's own layers. Where the pointer is caught, a small cross.
#[allow(clippy::too_many_arguments)]
pub fn draw<const S: u8>(doc: Res<Doc>, snap: Res<Snapshot>, mut lines: Gizmos<Pen<0, S>>, mut runs: Gizmos<Pen<1, S>>, mut broken: Gizmos<Pen<2, S>>, mut grid: Gizmos<Pen<3, S>>, mut highlight: Gizmos<Pen<4, S>>, mut room: Gizmos<Pen<5, S>>) {
    let n = S as usize;
    if n >= doc.second.mode.halves() {
        return;
    }
    let h = &doc.second.halves[n];
    let (Some(look), Some(layers)) = (&h.look, &h.layers) else { return };
    let (show, only_sheet) = shown_for(layers, look);
    let view = View { slot: Some(S), show, plan: true, walk: false, bench: false, only_sheet };
    scene::draw_view(&doc, &snap, &h.drawing, &view, &mut lines, &mut runs, &mut broken, &mut grid, &mut highlight, &mut room);
    if let Some(found) = &h.catch {
        let z = elevation_of(&snap, look);
        let at = to_view(doc.origin, [found.at[0], found.at[1], z]) + Vec3::Y * 0.2;
        let reach = scene::plan_feet_per_dot(h.orbit.now.3, h.area.h) * 6.0;
        for a in [Vec3::X, Vec3::Z] {
            highlight.line(at - a * reach, at + a * reach, theme::accent());
        }
    }
}

/// What a half says at its foot: where the pointer is, and what it caught.
pub fn captions(doc: Res<Doc>, mut out: Query<(&HalfCaption, &mut Text)>) {
    for (which, mut text) in &mut out {
        let said = doc.second.halves[which.0 as usize].said.clone();
        if text.0 != said {
            text.0 = said;
        }
    }
}

// ---------------------------------------------------------------- how it moves

/// The second window each frame: its title, where each plan stands, what each looks at, its fit, its flight and ease, what the hand does in it (the wheel,
/// the drag, a double-click, a click with a tool in hand) and what the pointer catches.
#[allow(clippy::too_many_arguments)]
pub fn step(
    mut doc: ResMut<Doc>,
    snap: Res<Snapshot>,
    plans: Res<crate::plans::Plans>,
    mut orbit: ResMut<Orbit>,
    time: Res<Time>,
    real: Res<Time<Real>>,
    buttons: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    hover: Res<HoverMap>,
    planui: Query<(), With<PlanUi>>,
    mut windows: Query<&mut Window, With<SheetWindow>>,
    ui: Res<UiScale>,
    mut trace: Local<Option<Option<String>>>,
    mut presses: MessageReader<bevy::input::mouse::MouseButtonInput>,
) {
    // How many times the left button went down since the last frame: on a slow machine both presses of a double-click can land in one frame.
    let downs = presses.read().filter(|p| p.button == MouseButton::Left && p.state == bevy::input::ButtonState::Pressed).count();
    let Ok(mut window) = windows.single_mut() else { return };
    let mut second = std::mem::take(&mut doc.bypass_change_detection().second);
    inner(&mut doc, &mut second, &snap, &plans, &mut orbit, &time, &real, &buttons, &scroll, &hover, &planui, &mut window, ui.0.max(0.1), &mut trace, downs);
    doc.bypass_change_detection().second = second;
}

#[allow(clippy::too_many_arguments)]
fn inner(doc: &mut Doc, second: &mut Second, snap: &Snapshot, plans: &crate::plans::Plans, orbit: &mut Orbit, time: &Time, real: &Time<Real>, buttons: &ButtonInput<MouseButton>, scroll: &AccumulatedMouseScroll, hover: &HoverMap, planui: &Query<(), With<PlanUi>>, window: &mut Window, ui: f32, trace: &mut Option<Option<String>>, downs: usize) {
    // The window's title says what it shows.
    let on = second.mode.halves();
    let title = match second.mode {
        Mode::Estimate => "BuildOS: sheet".to_string(),
        Mode::Plan | Mode::Two => {
            let names: Vec<String> = (0..on).map(|n| second.halves[n].look.as_ref().map(|l| name_of(snap, l)).filter(|s| !s.is_empty()).unwrap_or_default()).collect();
            format!("BuildOS: sheet, {} {}", if on == 1 { "plan" } else { "plans" }, names.join(" and "))
        }
    };
    if window.title != title {
        window.title = title;
    }
    if on == 0 {
        for h in &mut second.halves {
            h.catch = None;
            h.said.clear();
        }
        second.hold = None;
        return;
    }
    let size = Vec2::new(window.width(), window.height());
    let areas = half_rects(second.mode, size, TOP * ui);
    let model_level = doc.show.storey.clone();
    // What each plan looks at, its own layers, where it stands and how it is fitted.
    for n in 0..on {
        let (first, valid) = {
            let first = second.halves[0].look.clone();
            let valid = second.halves[n].look.as_ref().is_some_and(|l| choices(snap).iter().any(|(c, _)| c == l));
            (first, valid)
        };
        if !valid && !snap.storeys.is_empty() {
            second.halves[n].look = default_look(snap, model_level.as_deref(), n, if n == 0 { None } else { first.as_ref() });
            second.halves[n].touched = false;
        }
        if second.halves[n].layers.is_none() {
            let away = second.halves[n].pending_away.clone();
            second.halves[n].layers = Some(fresh_layers(doc, &away));
            doc.touch();
        }
        let h = &mut second.halves[n];
        h.area = areas[n];
        let Some(look) = h.look.clone() else { continue };
        let key = (look.clone(), h.area.w as u32, h.area.h as u32);
        // A view kept from last time is put back as it was left, once; the plan is then the hand's until it is set to something else.
        if let Some(v) = h.pending_view.take() {
            let target = Vec3::new(v[0] as f32, 0.0, v[1] as f32);
            h.orbit.target = target;
            h.orbit.distance = v[2] as f32;
            h.orbit.home = (target, v[2] as f32);
            h.orbit.now = (target, h.orbit.yaw, h.orbit.pitch, v[2] as f32);
            h.touched = true;
            h.fitted = Some(key.clone());
        }
        if !h.touched && h.fitted.as_ref() != Some(&key) {
            let (lo, hi) = ground_of(snap, &look);
            let (centre, distance) = fit(lo, hi, h.area.w, h.area.h);
            let target = to_view(doc.origin, [centre[0], centre[1], 0.0]).with_y(0.0);
            let first_time = h.fitted.is_none();
            h.orbit.target = target;
            h.orbit.distance = distance;
            h.orbit.home = (target, distance);
            h.orbit.flight = None;
            if first_time || h.fitted.as_ref().is_some_and(|f| f.0 != look) {
                // A plan newly opened, or set to another level, comes up fitted and is not seen to travel there.
                h.orbit.now = (target, h.orbit.yaw, h.orbit.pitch, distance);
            }
            h.fitted = Some(key);
        }
    }
    // The hand.
    let dt = time.delta_secs().min(0.1);
    let took = time.delta_secs().max(1.0e-3);
    let cursor = window.cursor_position();
    let over_ui = hover.get(&PointerId::Mouse).is_some_and(|hits| hits.keys().any(|e| planui.contains(*e)));
    let at = cursor.and_then(|c| half_at(&areas, c)).filter(|_| !over_ui);
    let left = MouseButton::Left;
    for n in 0..on {
        let h = &mut second.halves[n];
        let ppd = |h: &Half| scene::plan_feet_per_dot(h.orbit.distance, h.area.h);
        // A flight carries the view until a hand takes it over.
        let by_hand = at == Some(n) && (buttons.any_just_pressed([left, MouseButton::Middle, MouseButton::Right]) || scroll.delta.y != 0.0);
        if h.orbit.flight.is_some() && by_hand {
            let now = h.orbit.pose_now();
            h.orbit.target = now.target;
            h.orbit.distance = now.distance;
            h.orbit.flight = None;
        }
        if h.orbit.flight.is_some() {
            scene::plan_flight(&mut h.orbit, time.delta_secs());
        } else {
            scene::plan_ease(&mut h.orbit, time.delta_secs());
        }
        let centre = Vec2::new(h.area.x + h.area.w / 2.0, h.area.y + h.area.h / 2.0);
        let moved = match (cursor, h.was) {
            (Some(now), Some(then)) => now - then,
            _ => Vec2::ZERO,
        };
        let moved = if buttons.any_just_pressed([left, MouseButton::Middle, MouseButton::Right]) { Vec2::ZERO } else { moved };
        h.was = cursor;
        // A press in a half takes hold of its ground (the left or the middle or the right button: on a plan all three slide it).
        if at == Some(n) && buttons.any_just_pressed([left, MouseButton::Middle, MouseButton::Right]) {
            second.hold = Some(n);
            h.slide = Vec3::ZERO;
            h.gone = 0.0;
            // A second press of the left button in the same place flies to what is selected.
            if buttons.just_pressed(left) {
                // The program's own clock, as the model window tells a double-click by: on a slow frame a frame is a quarter of a second of it, never more.
                let now = time.elapsed_secs();
                let again = downs >= 2 || h.last_press.is_some_and(|(then, p)| crate::nav::again(then, now, time.delta_secs()) && cursor.is_some_and(|c| c.distance(p) < 8.0));
                h.last_press = cursor.map(|c| (now, c));
                if again {
                    if let Some(path) = doc.selected.clone() {
                        if let Some((centre_of, radius)) = scene::sphere_of(&scene::corners_of(doc, snap, &path)) {
                            let to = crate::nav::Pose { target: centre_of.with_y(0.0), yaw: h.orbit.yaw, pitch: h.orbit.pitch, distance: scene::plan_distance_for(radius) };
                            h.orbit.fly(to);
                            h.touched = true;
                            scene::frame(doc, orbit, snap);
                        }
                    }
                    h.last_press = None;
                }
            }
        }
        let held = buttons.pressed(left) || buttons.pressed(MouseButton::Middle) || buttons.pressed(MouseButton::Right);
        let let_go = !held && buttons.any_just_released([left, MouseButton::Middle, MouseButton::Right]);
        if second.hold == Some(n) && (held || let_go) {
            h.gone += moved.length();
            if buttons.pressed(left) && h.gone > 5.0 {
                scene::mark_dragged();
            }
            // What the hand did this frame, held still or not: held still it is no speed, and what carries on after it lets go is nothing.
            let shift = scene::plan_drag(&h.orbit, moved, h.area.h);
            h.orbit.target += shift;
            if moved != Vec2::ZERO {
                h.orbit.flight = None;
                h.touched = true;
            }
            h.slide = h.slide.lerp(shift / took, 0.5);
        }
        if !held {
            if second.hold == Some(n) {
                second.hold = None;
                // Let go: it carries on only if it was still going, faster than twenty dots a second.
                let dot = ppd(h);
                if h.slide.length() < dot * 20.0 {
                    h.slide = Vec3::ZERO;
                }
                // A click with a drawing tool in hand, here, draws nothing; it says where to draw.
                if h.gone <= 5.0 && let_go && buttons.just_released(left) && doc.draw.tool.is_some() {
                    doc.say(Tone::Ask, "Draw in the model window. This window is for looking.");
                    doc.touch();
                }
            }
            let fade = (-dt * 2.5).exp();
            if h.slide != Vec3::ZERO {
                let step = h.slide * dt;
                h.orbit.target += step;
                h.slide *= fade;
                if h.slide.length() < ppd(h) * 8.0 {
                    h.slide = Vec3::ZERO;
                }
            }
        }
        // The wheel: the place under the pointer stays under it.
        if at == Some(n) && scroll.delta.y != 0.0 {
            let notches = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y / 100.0 } else { scroll.delta.y }.clamp(-3.0, 3.0);
            scene::plan_wheel(&mut h.orbit, notches, cursor.map(|c| c - centre), h.area.h);
            h.touched = true;
            h.slide = Vec3::ZERO;
        }
    }
    // What the pointer catches, in the words the model window uses.
    for n in 0..on {
        let (look, area, now) = {
            let h = &second.halves[n];
            (h.look.clone(), h.area, h.orbit.now)
        };
        second.halves[n].catch = None;
        second.halves[n].said.clear();
        let (Some(look), Some(c), true) = (look, cursor, at == Some(n) && second.hold.is_none()) else { continue };
        let centre = Vec2::new(area.x + area.w / 2.0, area.y + area.h / 2.0);
        let place = scene::plan_place(now.0, now.3, c - centre, area.h);
        let p = [doc.origin[0] + f64::from(place.x), doc.origin[1] - f64::from(place.z)];
        let mut said = format!("{}     {}", crate::panels::compass(p[0], 0), crate::panels::compass(p[1], 1));
        if let Look::Level(path) = &look {
            let h = &mut second.halves[n];
            let pages = plans.read.len();
            if !h.scene.as_ref().is_some_and(|(g, of, read, _)| *g == doc.geometry && of == path && *read == pages) {
                let (scene, _) = crate::tools::level_scene(&mut doc.job, snap, plans, path, None);
                h.scene = Some((doc.geometry, path.clone(), pages, scene));
            }
            if let Some((_, _, _, scene)) = &h.scene {
                let found = crate::tools::catch_at(scene, p, f64::from(scene::plan_feet_per_dot(now.3, area.h)));
                let name = found.kind.name();
                if !name.is_empty() {
                    said.push_str(&format!("     {name}"));
                    h.catch = Some(found);
                }
            }
        }
        second.halves[n].said = theme::spoken(&said);
    }
    // Frame times, to the map's own folder: one line a frame while a plan's view moves, so that a demonstration reads what a frame costs.
    let file = trace.get_or_insert_with(|| std::env::var("PLUMBLINE_MAP").ok().map(|f| format!("{f}.halfview")));
    if let Some(file) = file.as_ref() {
        use std::io::Write;
        for n in 0..on {
            let h = &second.halves[n];
            let moving = h.orbit.flight.is_some() || h.slide != Vec3::ZERO || (h.orbit.now.0 - h.orbit.target).length() > 1e-3 || (h.orbit.now.3 - h.orbit.distance).abs() > 1e-3 || scroll.delta.y != 0.0 || second.hold == Some(n);
            if moving {
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(file) {
                    let _ = writeln!(f, "half\t{}\t{:.3}\t{:.1}\t{:.3}\t{:.3}\t{:.3}", n + 1, real.elapsed_secs(), real.delta_secs() * 1000.0, doc.origin[0] + f64::from(h.orbit.now.0.x), doc.origin[1] - f64::from(h.orbit.now.0.z), h.area.h / (h.orbit.now.3 * scene::PLAN_HEIGHT));
                }
            }
        }
    }
}

// ---------------------------------------------------------------- what is on the window

/// The map's lines for the second window (new kinds only): which of Estimate and Plan it shows; for each plan what it shows, the place on the ground at its middle,
/// the dots to a foot and where it stands on the screen; what is marked in it; how many things are drawn in it and in the model window; what the pointer caught.
pub fn map_lines(doc: &Doc, snap: &Snapshot, at: IVec2) -> String {
    let second = &doc.second;
    let mut out = format!("second\t{}\n", second.mode.word());
    for n in 0..second.mode.halves() {
        let h = &second.halves[n];
        let shows = match &h.look {
            Some(Look::Level(_)) => format!("level {}", h.look.as_ref().map(|l| name_of(snap, l)).unwrap_or_default()),
            Some(Look::Sheet(_)) => format!("drawing {}", h.look.as_ref().map(|l| name_of(snap, l)).unwrap_or_default()),
            None => "nothing".to_string(),
        };
        let (east, north) = (doc.origin[0] + f64::from(h.orbit.now.0.x), doc.origin[1] - f64::from(h.orbit.now.0.z));
        let dots = h.area.h / (h.orbit.now.3 * scene::PLAN_HEIGHT).max(1e-3);
        out.push_str(&format!("half\t{}\t{}\t{:.2}\t{:.2}\t{:.3}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\n", n + 1, shows, east, north, dots, h.area.x + at.x as f32, h.area.y + at.y as f32, h.area.w, h.area.h));
        let marked = doc.selected.as_ref().filter(|s| h.drawing.frames.iter().any(|(p, _)| p == *s)).cloned().unwrap_or_else(|| "none".to_string());
        out.push_str(&format!("marked\t{}\t{}\n", n + 1, marked));
        out.push_str(&format!("shown\t{}\t{}\n", n + 1, h.shown));
        out.push_str(&format!("catch\t{}\t{}\n", n + 1, if h.said.is_empty() { "none" } else { &h.said }));
    }
    if second.mode.halves() > 0 {
        out.push_str(&format!("shown\tmodel\t{}\n", second.model_shown));
        out.push_str(&format!("marked\tmodel\t{}\n", doc.selected.as_deref().unwrap_or("none")));
    }
    out
}

// ---------------------------------------------------------------- the controls

/// The second window's bar: the wordmark, the job, "Estimate" and "Plan" (and "Two plans" while it is a plan), and the counts. One bar for both.
pub fn title_card(root: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    // A round card lifted off the page, as the model window's bar is, with the job's name in it.
    root.spawn((rounded(Node { height: px(44.0), flex_shrink: 0.0, margin: UiRect { left: px(theme::GAP), right: px(theme::GAP), top: px(theme::GAP), bottom: px(0.0) }, padding: UiRect::horizontal(px(14.0)), align_items: AlignItems::Center, column_gap: px(12.0), border: UiRect::all(px(1.0)), overflow: Overflow::clip(), ..default() }, theme::PANEL_RADIUS), BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()), theme::card_shadow())).with_children(|bar| {
        wordmark(bar, fonts);
        bar.spawn((Node { width: px(1.0), height: px(22.0), margin: UiRect::horizontal(px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::hairline())));
        bar.spawn(Node { flex_shrink: 1.0, min_width: px(0.0), overflow: Overflow::clip(), ..default() }).with_child((theme::name(fonts, one_line(&snap.title, 30), 13.5, Weight::Semibold, theme::ink()), TextLayout::no_wrap()));
        bar.spawn(Node { flex_grow: 1.0, ..default() });
        mode_buttons(bar, fonts, doc);
        if doc.second.mode == Mode::Estimate {
            tally_chips(bar, fonts, snap.tally);
        } else {
            // With plans in the window the price is kept in sight (the recap is the estimate's own): what a change in the model window does to it is seen here.
            let priced = snap.estimate.as_ref().is_some_and(|e| !e.lines.is_empty() || !e.allowance_on.is_empty());
            if let (Some(e), true) = (&snap.estimate, priced) {
                let (price, status) = crate::sheet::dollars(Some(&e.price));
                bar.spawn((text(fonts, "PRICE", 11.0, Weight::Semibold, theme::ink_faint()), TextLayout::no_wrap()));
                bar.spawn((theme::name(fonts, price, 14.0, Weight::Semibold, theme::status_ink(status)), TextLayout::no_wrap()));
            }
        }
    });
}

fn click(e: &mut EntityCommands, act: impl Fn(&mut Doc) + Send + Sync + 'static) {
    e.observe(move |mut press: On<PointerClick>, mut doc: ResMut<Doc>| {
        press.propagate(false);
        if press.button == PointerButton::Primary {
            act(&mut doc);
        }
    });
}

/// "Estimate" and "Plan", one on at a glance; and "Two plans" or "One plan" while it is a plan.
fn mode_buttons(bar: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc) {
    let mode = doc.second.mode;
    let mut b = theme::button(bar, fonts, "Estimate", "", mode == Mode::Estimate);
    b.insert(theme::named("Window Estimate"));
    click(&mut b, |doc| set_mode(doc, Mode::Estimate));
    let mut b = theme::button(bar, fonts, "Plan", "", mode != Mode::Estimate);
    b.insert(theme::named("Window Plan"));
    click(&mut b, |doc| {
        if doc.second.mode == Mode::Estimate {
            set_mode(doc, Mode::Plan)
        }
    });
    if mode != Mode::Estimate {
        let two = mode == Mode::Two;
        let mut b = theme::button(bar, fonts, if two { "One plan" } else { "Two plans" }, "", false);
        click(&mut b, move |doc| set_mode(doc, if two { Mode::Plan } else { Mode::Two }));
    }
}

/// The second window as plans: its bar, a row of controls for each plan (what it looks at, its layers), and a line between two. The plans themselves are drawn
/// by their cameras, in the window under these.
pub fn spawn_ui(commands: &mut Commands, doc: &Doc, snap: &Snapshot, fonts: &Fonts, cameras: &Cameras) {
    let mode = doc.second.mode;
    let on = mode.halves();
    commands.spawn((Node { width: percent(100.0), max_width: percent(100.0), min_width: px(0.0), height: percent(100.0), flex_direction: FlexDirection::Column, overflow: Overflow::clip(), ..default() }, UiTargetCamera(cameras.sheet), crate::sheet::SheetUi, Pickable::IGNORE)).with_children(|root| {
        title_card(root, fonts, doc, snap);
        // A row of controls for each plan, over its own half.
        root.spawn((Node { height: px(36.0), flex_shrink: 0.0, margin: UiRect { top: px(6.0), left: px(theme::GAP), right: px(theme::GAP), ..default() }, column_gap: px(LINE + 2.0 * theme::GAP), ..default() }, Pickable::IGNORE)).with_children(|row| {
            for n in 0..on {
                let h = &doc.second.halves[n];
                row.spawn((Node { flex_basis: px(0.0), flex_grow: 1.0, min_width: px(0.0), align_items: AlignItems::Center, column_gap: px(8.0), ..default() }, Pickable::IGNORE)).with_children(|half| {
                    let said = h.look.as_ref().map(|l| name_of(snap, l)).filter(|s| !s.is_empty()).unwrap_or_else(|| "Choose".to_string());
                    let mut b = theme::button(half, fonts, &one_line(&said, 22), "", h.open == Open::Levels);
                    b.insert(theme::named(format!("Plan {} shows", n + 1)));
                    click(&mut b, move |doc| toggle_open(doc, n, Open::Levels));
                    let mut b = theme::button(half, fonts, "Layers", "", h.open == Open::Layers);
                    b.insert(theme::named(format!("Plan {} layers", n + 1)));
                    click(&mut b, move |doc| toggle_open(doc, n, Open::Layers));
                    // What a plan's chooser lists: the levels, then the drawings, each by its name.
                    if h.open == Open::Levels {
                        half.spawn((crate::panels::glass_card(Node { position_type: PositionType::Absolute, top: px(34.0), left: px(0.0), flex_direction: FlexDirection::Column, row_gap: px(3.0), padding: UiRect::all(px(8.0)), min_width: px(170.0), ..default() }), GlobalZIndex(20), PlanUi)).with_children(|list| {
                            for (look, name) in choices(snap) {
                                let on = h.look.as_ref() == Some(&look);
                                let mut b = theme::button(list, fonts, &one_line(&name, 30), "", on);
                                b.insert(theme::named(format!("Plan {} shows {}", n + 1, name)));
                                click(&mut b, move |doc| set_look(doc, n, look.clone()));
                            }
                        });
                    }
                    if h.open == Open::Layers {
                        if let Some(layers) = &h.layers {
                            half.spawn((crate::panels::glass_card(Node { position_type: PositionType::Absolute, top: px(34.0), left: px(0.0), flex_direction: FlexDirection::Column, row_gap: px(4.0), padding: UiRect::all(px(10.0)), min_width: px(190.0), ..default() }), GlobalZIndex(20), PlanUi)).with_children(|list| {
                                for (i, word) in LAYERS.iter().enumerate() {
                                    list.spawn(Node { align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, column_gap: px(12.0), ..default() }).with_children(|r| {
                                        r.spawn((text(fonts, *word, 12.0, Weight::Medium, theme::ink()), TextLayout::no_wrap(), Pickable::IGNORE));
                                        crate::panels::switch_at(r, layer_on(layers, i), format!("Plan {} {}", n + 1, word), false, move |doc| toggle_layer(doc, n, i));
                                    });
                                }
                            });
                        }
                    }
                });
            }
        });
        root.spawn((Node { flex_grow: 1.0, min_height: px(0.0), ..default() }, Pickable::IGNORE));
        crate::sheet::foot(root, fonts, doc);
    });
    // Over the plans: the line between two, and what the pointer has caught, at the foot of each.
    commands.spawn((Node { position_type: PositionType::Absolute, left: percent(50.0), margin: UiRect::left(px(-(LINE / 2.0))), top: px(TOP), bottom: px(0.0), width: px(LINE), ..default() }, BackgroundColor(if mode == Mode::Two { theme::hairline() } else { Color::NONE }), UiTargetCamera(cameras.sheet), crate::sheet::SheetUi, Pickable::IGNORE));
    for n in 0..on {
        commands.spawn((
            Node { position_type: PositionType::Absolute, bottom: px(50.0), left: if on == 2 && n == 1 { percent(50.0) } else { px(0.0) }, margin: UiRect::left(px(theme::GAP + 2.0)), ..default() },
            text(fonts, "", 12.0, Weight::Medium, theme::ink_soft()),
            TextLayout::no_wrap(),
            HalfCaption(n as u8),
            UiTargetCamera(cameras.sheet),
            crate::sheet::SheetUi,
            Pickable::IGNORE,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_plan_fills_the_window_under_its_controls_and_two_stand_side_by_side_with_a_line_between() {
        let size = Vec2::new(880.0, 960.0);
        assert_eq!(Mode::Estimate.halves(), 0);
        assert_eq!(Mode::Plan.halves(), 1);
        assert_eq!(Mode::Two.halves(), 2);
        assert!(half_rects(Mode::Estimate, size, 96.0).is_empty());
        let one = half_rects(Mode::Plan, size, 96.0);
        assert_eq!(one, vec![Area { x: 0.0, y: 96.0, w: 880.0, h: 864.0 }]);
        let two = half_rects(Mode::Two, size, 96.0);
        assert_eq!(two.len(), 2);
        assert_eq!(two[0].x, 0.0);
        assert_eq!(two[0].w, (880.0 - LINE) / 2.0);
        assert_eq!(two[1].x, two[0].w + LINE);
        assert_eq!(two[1].x + two[1].w, 880.0);
        assert!(two.iter().all(|a| a.y == 96.0 && a.h == 864.0));
    }

    #[test]
    fn a_point_is_in_the_half_it_lies_in_and_in_none_on_the_line_or_over_the_controls() {
        let two = half_rects(Mode::Two, Vec2::new(880.0, 960.0), 96.0);
        assert_eq!(half_at(&two, Vec2::new(100.0, 500.0)), Some(0));
        assert_eq!(half_at(&two, Vec2::new(700.0, 500.0)), Some(1));
        assert_eq!(half_at(&two, Vec2::new(439.0, 500.0)), None, "on the line between them");
        assert_eq!(half_at(&two, Vec2::new(100.0, 40.0)), None, "over the controls");
        assert_eq!(half_at(&two, Vec2::new(100.0, 960.0)), None, "past the foot");
        assert_eq!(half_at(&[], Vec2::new(1.0, 1.0)), None);
    }

    #[test]
    fn a_house_forty_by_twenty_eight_is_whole_in_a_half_with_a_margin_at_fifteen_to_twenty_dots_a_foot_in_a_whole_window() {
        let (lo, hi) = ([0.0, 0.0], [40.0, 28.0]);
        let (w, h) = (880.0_f32, 864.0_f32);
        let (centre, distance) = fit(lo, hi, w, h);
        assert_eq!(centre, [20.0, 14.0]);
        // What a dot covers is the distance's own figure (`scene::plan_feet_per_dot`): the view is `distance * 0.62` feet tall.
        let dots_a_foot = h / (distance * crate::scene::PLAN_HEIGHT);
        assert!((15.0..=20.0).contains(&dots_a_foot), "{dots_a_foot} dots a foot");
        // Whole: the house's width and height in dots are inside the half, with a margin each side.
        assert!(40.0 * dots_a_foot <= w * 0.96 && 28.0 * dots_a_foot <= h * 0.96);
    }

    #[test]
    fn a_tall_narrow_half_is_fitted_by_its_width_and_a_wide_one_by_its_height() {
        let (lo, hi) = ([0.0, 0.0], [40.0, 28.0]);
        let narrow = fit(lo, hi, 300.0, 900.0).1;
        let wide = fit(lo, hi, 900.0, 300.0).1;
        for (w, h, d) in [(300.0_f32, 900.0_f32, narrow), (900.0, 300.0, wide)] {
            let per_foot = h / (d * crate::scene::PLAN_HEIGHT);
            assert!(40.0 * per_foot <= w && 28.0 * per_foot <= h, "{w}x{h}: {per_foot} dots a foot is not whole");
        }
        assert!(narrow > 0.0 && wide > 0.0);
    }

    #[test]
    fn a_job_with_nothing_in_it_is_still_fitted_to_a_few_feet() {
        let (_, d) = fit([0.0, 0.0], [0.0, 0.0], 440.0, 400.0);
        assert!(d.is_finite() && d > 0.0);
    }
}
