//! Frosted glass: what is behind a panel, blurred, under the panel's tint.
//!
//! The engine has no way to blur what is behind a panel (DECISIONS.md call 63). So the
//! scene is drawn a second time, small, into a picture; and a panel's fill is a
//! material that looks at that picture where the panel is, at thirteen places round
//! each dot, and lays the glass's tint over the result. Small is what makes it cheap
//! and is half of the blur: a picture a sixth of the window's size has already
//! averaged thirty-six dots into one.
//!
//! With PLUMBLINE_GRAPHICS=plain there is no second picture and a panel is its tint.

use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, TextureFormat};
use bevy::shader::ShaderRef;

use crate::scene::MainCamera;
use crate::theme;

/// How many times smaller than the window the picture behind the glass is, each way.
const SMALLER: u32 = 6;

/// A panel's fill: the scene behind it, blurred, under a tint.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct Frost {
    #[uniform(0)]
    pub tint: LinearRgba,
    #[uniform(0)]
    pub reach: f32,
    #[texture(1)]
    #[sampler(2)]
    pub scene: Handle<Image>,
}

impl UiMaterial for Frost {
    fn fragment_shader() -> ShaderRef {
        "embedded://plumbline_app/frost.wesl".into()
    }
}

/// The glass every panel is made of, for whoever builds a panel. None: no frost, and a
/// panel is its tint.
static PANEL: std::sync::OnceLock<Handle<Frost>> = std::sync::OnceLock::new();

pub fn panel() -> Option<Handle<Frost>> {
    PANEL.get().cloned()
}

/// The camera that draws the small picture.
#[derive(Component)]
pub struct Behind;

pub struct GlassPlugin;

impl Plugin for GlassPlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "frost.wesl");
        app.add_plugins(UiMaterialPlugin::<Frost>::default());
    }
}

/// Make the small picture, the camera that draws it, and the glass that looks at it.
pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut glass: ResMut<Assets<Frost>>, window: Query<&Window, With<bevy::window::PrimaryWindow>>, graphics: Res<crate::scene::Graphics>, sky: Res<crate::scene::SkyLight>) {
    if !graphics.full {
        return;
    }
    let Ok(window) = window.single() else { return };
    let (w, h) = ((window.physical_width() / SMALLER).max(16), (window.physical_height() / SMALLER).max(16));
    let picture = images.add(Image::new_target_texture(w, h, TextureFormat::Bgra8UnormSrgb, None));
    // It draws before the window's own camera, and plainly: no steadying over frames,
    // no shade where surfaces meet. What it draws is only ever seen blurred.
    commands.spawn((Camera3d::default(), Camera { order: -1, ..default() }, RenderTarget::Image(ImageRenderTarget { handle: picture.clone(), scale_factor: 1.0 }), bevy::camera::Hdr, Msaa::Off, crate::scene::FOLD, DebandDither::Disabled, sky.0.clone(), Behind));
    let tint = theme::frost().to_linear();
    let made = glass.add(Frost { tint: LinearRgba::new(tint.red, tint.green, tint.blue, theme::FROST_THICK), reach: 0.028, scene: picture });
    let _ = PANEL.set(made);
}

/// The small picture is of what the window shows: its camera goes where the window's goes.
pub fn follow(main: Query<(&Transform, &Projection, &Tonemapping), (With<MainCamera>, Without<Behind>)>, mut behind: Query<(&mut Transform, &mut Projection, &mut Tonemapping), (With<Behind>, Without<MainCamera>)>, mut glass: ResMut<Assets<Frost>>) {
    let (Ok((at, lens, fold)), Ok((mut t, mut p, mut f))) = (main.single(), behind.single_mut()) else { return };
    // Over a plan the glass is a shade cooler and denser than over the model: white
    // frost over white paper was no panel at all, only words lying on the drawing
    // (seen in the clash demonstration's plan). Over the model it is as it was.
    let on_plan = *fold == Tonemapping::None;
    let tint = if on_plan { theme::frost_on_paper() } else { theme::frost() }.to_linear();
    let want = LinearRgba::new(tint.red, tint.green, tint.blue, if on_plan { 0.86 } else { theme::FROST_THICK });
    if let Some(handle) = PANEL.get() {
        if glass.get(handle).is_some_and(|m| m.tint != want) {
            if let Some(mut m) = glass.get_mut(handle) {
                m.tint = want;
            }
        }
    }
    *t = *at;
    *p = lens.clone();
    // Lit and folded to the screen as the window is, so the blur matches what is round it.
    if *f != *fold {
        *f = *fold;
    }
}
