# Bevy 0.20.1 survey for the plumbline window rebuild

Read-only survey from source on disk. Nothing was built or run in /home/user/roofnerd. I downloaded `bevy_ui_widgets` and `bevy_feathers` 0.20.1 from crates.io into the scratchpad only, to read them. "Registry" below means `/root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. Behaviour of effects on lavapipe is inferred from the source's own capability checks, not tested.

## 0. The three findings that change your plan

1. **BIMCraft is not a Bevy program.** `/home/user/ipedrax/bimcraft` is egui 0.36 / eframe plus its own wgpu and CPU renderer. `Cargo.lock` has 0 `bevy` crates. Licence is MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `Cargo.toml:8`). Only math, UX and ideas port; there is no Bevy code to lift (section 5).
2. **Bevy ships a complete pan/orbit/zoom camera controller, but with no input plugin.** The crate is `bevy_camera_controller` (feature `pan_orbit_camera`), on disk in the registry and not yet in your `Cargo.lock`. It has smoothing, momentum, anchor-under-cursor orbit/zoom/pan, and look-to and dolly-zoom extensions. The docs say it wires pointer input itself, but the crate contains no input system. You call `start_orbit`, `start_pan`, `send_screenspace_input`, `send_zoom_input` and `end_move` yourself.
3. **Bevy 0.20 has a transform gizmo and a ready widget set, but part of the widget set is missing on disk.**
   - `bevy_gizmos::transform_gizmo` is real and is in your tree today.
   - `bevy_ui_widgets` (scroll area, scrollbar, slider, checkbox, radio, text input, menu, dialog, tabs) and `bevy_feathers` (including a toggle switch) are not in the registry and not in `Cargo.lock`. Both exist on crates.io as 0.20.1, MIT OR Apache-2.0. Enabling them needs one network fetch.
   - `bevy_feathers` also needs `bevy_scene` (the `bsn!` macro), which is not on disk either.

## 1. bevy_ui and widgets

Paths are under the registry, in `bevy_ui-0.20.1/src/` (`bevy_ui`).

**Styling components.** These are plain components on a `Node`:
- **Rounded corners.** `Node.border_radius: BorderRadius` (`ui_node.rs:731`). `BorderRadius` is `{top_left, top_right, bottom_right, bottom_left}` of `CornerRadius` (`:2959`). Build it with `BorderRadius::all(px(10.))` (`:3005`) or `BorderRadius::px(a,b,c,d)` (`:3032`). Elliptical corners are supported.
- **Borders.** `Node.border: UiRect` (`:692`). `BorderColor { top, right, bottom, left }` (`:2603`); `BorderColor::all(c)` (`:2627`).
- **Fill.** `BackgroundColor(pub Color)` (`:2576`). Alpha in the colour works and blends normally, e.g. `Color::srgba(1.,1.,1.,0.12)`. There is no separate opacity component.
- **Shadows.** `BoxShadow(pub Vec<ShadowStyle>)` (`:3311`). Several shadows are allowed, drawn back to front. `ShadowStyle { color, x_offset: Val, y_offset: Val, spread_radius: Val, blur_radius: Val }` (`:3345`). Single shadow: `BoxShadow::new(color, x, y, spread, blur)`. Quality is the camera component `BoxShadowSamples(u32)` (default 4; `bevy_ui_render/src/lib.rs:200`).
- **Gradients** (`gradients.rs`). `BackgroundGradient(pub Vec<Gradient>)` (`:544`) and `BorderGradient(pub Vec<Gradient>)` (`:560`). `Gradient` is `Linear(LinearGradient)` (`:235`; fields `color_space`, `angle`, `stops`; consts such as `TO_BOTTOM`), `Radial(RadialGradient)` (`:369`; `position: UiPosition`, `shape: RadialGradientShape`, `stops`) or `Conic(ConicGradient)` (`:418`; `start`, `position`, `stops: Vec<AngularColorStop>`). Stops are `ColorStop::new(color, Val)`, `::auto`, `::px`, `::percent` (`:18`; has a `hint` midpoint). Interpolation space is `InterpolationColorSpace` (`:670`).
- **Outline.** `Outline { width, offset, color }` (`:2717`). Set the colour to `Color::NONE` to hide it, which avoids table moves.
- **Stacking.** `ZIndex(i32)` (`:2868`) is local to the parent. `GlobalZIndex(i32)` (`:2886`) is across the whole UI.
- **Transforms.** `UiTransform { translation: Val2, scale: Vec2, rotation: Rot2 }` (`ui_transform.rs:144`). It inserts `UiGlobalTransform` and is honoured by picking. Hover animation is: you write `scale` each frame; there is no built-in tween.
- **Scrolling and clipping.**
  - `Overflow { x, y }` (`ui_node.rs:1545`) with `Overflow::clip()`, `::scroll_y()` (`:1635`); `OverflowAxis::Scroll`.
  - `ScrollPosition(pub Vec2)` (`:411`) is in logical px, and writing it moves the content immediately.
  - `Node.scrollbar_width: f32` (`:516`).
  - `IgnoreScroll(BVec2)` (`:430`) pins a child.
  - The built-in `ScrollArea` is instant: no easing or inertia.

**Input and picking.**
- **Interaction is deprecated.** `Interaction` is an alias of `DeprecatedInteraction` (`focus.rs:50-65`), deprecated since 0.20. Use `bevy::picking::hover::Hovered(bool)` (`bevy_picking/src/hover.rs:417`; `DirectlyHovered` at `:435`) plus `bevy_ui::Pressed`, `Checked`, `Checkable`, `Selected`, `InteractionDisabled` (`interaction_states.rs:31-106`).
- **FocusPolicy is ignored.** The UI picking backend says so (`picking_backend.rs:10`). Use `bevy::picking::Pickable { should_block_lower, is_hoverable }` (`bevy_picking/src/lib.rs:199`). A node with no `Pickable` blocks everything below it. This is what the app already uses with `Pickable::IGNORE`.
- **Pointer events carry bevy_picking names, not `Pointer<Over>`.** In 0.20.1 the event types are `PointerOver` (`events.rs:192`), `PointerOut` (`:256`), `PointerClick` (`:353`), `PointerPress`, `PointerRelease`, `PointerMove` (`:374`), `PointerDragStart` (`:396`), `PointerDrag` (`:413`, fields `button`, `distance`, `delta`), `PointerDragEnd`, `PointerScroll` (`:540`, fields `unit`, `x`, `y`, `hit`, `phase`), plus the `Enter`, `Leave` and `DragEnter`, `DragOver`, `DragLeave`, `DragDrop` family. All are `EntityEvent`s that bubble up `ChildOf` to the window. Use them as observers: `e.observe(|ev: On<PointerOver>, ...| ..)`. The app already does this (`scene.rs:307-325`).
- **Wheel over UI.** `PointerScroll` fires on every entity in the hover map, so with a blocking panel only the panel gets it. An observer can call `scroll.propagate(false)`, as the built-in `ScrollArea` does (`bevy_ui_widgets` `scrollarea.rs:23-50`). A wheel observer on your ground or a catch-all node gets only the events no panel took. If no mesh is under the cursor there is no hover and no event, so keep `AccumulatedMouseScroll` as a fallback.

**Widgets.** Feature names are in `bevy-0.20.1/Cargo.toml:2796` (`bevy_feathers`, which also pulls `bevy_ui_widgets`) and `:2830` (`bevy_ui_widgets`, which pulls `bevy_input_focus` and `bevy_ui`).
- **Present in your lock and registry:** `bevy_ui`, `bevy_input_focus` (already enabled), `bevy_picking`, `bevy_text` (`EditableText`, `bevy_text/src/editing.rs:109`).
- **Not in the lock or registry:** `bevy_ui_widgets`, `bevy_feathers`, `bevy_scene`. Enabling needs `cargo fetch` through the proxy. The deps that `bevy_ui_widgets` adds (`parley`, `smol_str`, `accesskit`) are probably already in the lock; I did not check.
- **`bevy_ui_widgets` 0.20.1** (read from the downloaded crate). Plugin group `UiWidgetsPlugins`, which you add yourself. Components and events: `Button` and `ActivateOnPress` (event `Activate`), `Checkbox` (`SetChecked`, `ToggleChecked`; state is the `Checked` marker), `RadioGroup` and `RadioButton`, `Slider` (`SliderValue`, `SliderRange`, `SliderStep`, `SliderThumb`; event `ValueChange<f32>`), `ScrollArea`, `Scrollbar` and `ScrollbarThumb` (`ControlOrientation`), `TextInput` (needs `EditableText`), `ListBox`, `MenuPopup`, `Popover`, `Dialog`, `ModalDialog`, `TabList`/`Tab`/`SelectedTab`.
- **Toggle switch.** There is none in `bevy_ui_widgets`. `bevy_feathers` has `FeathersToggleSwitch` (`controls/toggle_switch.rs`), which is just a `Checkbox` with a sliding child, built in a `bsn!` scene. Feathers also has themed slider, scrollbar, radio, number input and so on, but it is themed by its own tokens and needs `bevy_scene`.
- **Least effort for a toggle.** Take `Checkbox` from `bevy_ui_widgets`, or write your own using `PointerClick` plus a sliding child node driven by `UiTransform` or `Node.left`, as the feathers source does.

**Custom UI material (no backdrop blur exists).**
- A search for "backdrop" in `bevy_ui`, `bevy_ui_render`, `bevy_core_pipeline` and `bevy_post_process` finds nothing. There is no backdrop-filter, no per-node blur and no frosted-glass node.
- **The material trait** is `bevy_ui_render::UiMaterial` (`ui_material.rs:103`): `AsBindGroup + Asset + Clone`, with optional `vertex_shader()`, `fragment_shader()`, `stack_z_offset()`, `specialize()`.
  - Plugin: `UiMaterialPlugin::<M>::default()` (`ui_material_pipeline.rs:37`).
  - Component: `MaterialNode<M>(Handle<M>)` (`ui_material.rs:182`), which requires `Node`.
- **Binding an image** is by the `AsBindGroup` derive:
  ```rust
  #[derive(AsBindGroup, Asset, TypePath, Clone)]
  struct Frost { #[uniform(0)] tint: LinearRgba, #[texture(1)] #[sampler(2)] scene: Handle<Image> }
  impl UiMaterial for Frost { fn fragment_shader() -> ShaderRef { "shaders/frost.wesl".into() } }
  ```
- **What the fragment shader receives** is `UiVertexOutput` from `bevy_ui_render/src/ui_vertex_output.wesl`: `uv` (node-local 0..1), `size` (node px), `border_widths`, `border_radius_x`, `border_radius_y` (px) and `@builtin(position)`. Bind group 0 is the `View` and `Globals` uniforms; your material is group 1.
- **Screen UV** is `in.position.xy / view.viewport.zw`. `view.wesl:82` has `viewport: vec4<f32>`. You write the rounded-rect mask from `border_radius_*` and `size` yourself.
- **Constraint.** The UI pass runs after post-processing, in the same view target as its camera (`bevy_ui_render/src/lib.rs:297-302`). A panel cannot sample the camera it is drawn on. So:
  1. The 3D camera renders to an `Image`.
  2. A second camera (`Camera2d`, higher `order`, `RenderTarget::Window`) draws that image full-window and also draws the panels, whose `UiMaterial` samples it (or a blurred copy of it).
  3. Pointer picking and `UiTargetCamera` follow the UI camera.
- **Render to an image.** The component is `bevy::camera::RenderTarget::Image(ImageRenderTarget { handle, scale_factor: 1.0 })` (`bevy_camera/src/camera.rs:893`, `:987`; `RenderTarget` is a component in 0.20). Make the image with `Image::new_target_texture(w, h, format, view_format)` (`bevy_image/src/image.rs:1253`). Resize it on window resize. Show it full-window with an `ImageNode` or `Sprite`.
- **Blurred copy.** There is no built-in way to hand a blurred copy to a shader as a texture. Options, cheapest first:
  - **(a) Sample the sharp image with many taps** (13-tap Kawase or Gaussian, linear filtering) inside the frost shader. No extra passes. Fine for a panel, and the UI pass is cheap even on lavapipe.
  - **(b) Second low-res 3D camera** on the same `RenderLayers`, rendering to a quarter-size `Image` with `RenderTarget::Image`. It doubles scene cost. On software Vulkan, where STATUS.md reports about a frame a second, this is the expensive one.
  - **(c) Mip chain.** `bevy_core_pipeline::mip_generation::MipGenerationJobs` (`mip_generation/mod.rs:234`) and `generate_mips_for_phase` generate mips for an `Image`, so `textureSampleLevel` at a high LOD gives a blur. It is render-world, manual, and uses a compute single-pass downsample (FidelityFX SPD).
  - **(d) Bloom's mip chain** (`bevy_post_process` `bloom/mod.rs`) is internal to the view and is not exposed as an asset. `bevy_post_process`'s `gaussian_blur.wesl` is a shader library (`gaussian_blur(...)`) you can import into a custom pass.
- **Same frame or one frame late.** With (a) and (c) the sample is the same frame, because the camera that makes the image runs before the UI camera (set `order`). A readback to the CPU would be late and pointless.
- **Other.** `FullscreenMaterial` (`bevy_core_pipeline/src/fullscreen_material.rs`) is for custom post-processing of the camera's own view, not for blurring into an asset.

## 2. Rendering realism in 0.20.1

Everything below is inside `PbrPlugin` or `bevy_core_pipeline`, so it needs only the cargo features you already have (`bevy_pbr`, `bevy_post_process`, `bevy_anti_alias`), except where I say otherwise. Camera HDR is the component `Hdr` (`bevy_camera/src/components.rs:89`), not a camera field. The cargo feature named `hdr` is for loading `.hdr` files.

**Missing cargo features in your `Cargo.toml` that matter** (`bevy-0.20.1/Cargo.toml:2758, 2838, 2907, 2976`):
- `smaa_luts`: SMAA's lookup tables. The SMAA plugin has a `#[cfg(not(feature = "smaa_luts"))]` branch (`bevy_anti_alias/src/smaa/mod.rs:294-323`), so SMAA without it uses placeholder LUTs. Enable it before using `Smaa`.
- `dfg_lut`: without it the PBR shader builds without `DFG_LUT` (`bevy_pbr/src/render/mesh.rs:3485`). Specular and environment lighting then use an analytic fallback; add the feature for correct environment BRDF.
- `bluenoise_texture`: without it a placeholder is used (`bevy_pbr/src/lib.rs:271-292`). Used by SSR and environment filtering.
- `area_light_luts`: only for `RectLight`.
- `experimental_pbr_pcss`: soft shadows (`DirectionalLight::soft_shadow_size`, `directional_light.rs:115`).

**Per item:**
- **SSAO.** `bevy::pbr::ScreenSpaceAmbientOcclusion { quality_level, radius, constant_object_thickness }` (`bevy_pbr/src/ssao/mod.rs:117`). It requires `DepthPrepass` and `NormalPrepass` (auto-added) and `Msaa::Off` (`:506`). It is compute-based and is skipped with a warning if `max_storage_textures_per_shader_stage < 5` (`:64-68`). The docs strongly recommend TAA with it. Your app already runs it with TAA on lavapipe.
- **Anti-aliasing.**
  - `Msaa` is a camera component (`bevy_render/src/view/mod.rs:248`: `Off`, `Sample2`, `Sample4` (default), `Sample8`).
  - `TemporalAntiAliasing` (`bevy_anti_alias/src/taa/mod.rs:115`) requires `Msaa::Off` and auto-adds `TemporalJitter`, `MipBias`, `DepthPrepass` and `MotionVectorPrepass`. It ghosts on alpha-blended meshes, which matters for your glass.
  - `Smaa { preset }` (`smaa/mod.rs:93`) and `Fxaa { enabled, edge_threshold, .. }` (`fxaa/mod.rs:58`) are post-process AA; use them with `Msaa::Off`.
  - `ContrastAdaptiveSharpening` exists (`contrast_adaptive_sharpening/mod.rs:39`).
  - OIT, SSAO and TAA all need MSAA off (OIT panics, `oit/mod.rs:139`).
- **Bloom.** `bevy::post_process::bloom::Bloom { intensity, .. }` (`bloom/settings.rs:33`; `#[require(Hdr)]`).
- **Tonemapping.** `Tonemapping` enum (`bevy_render/src/view/mod.rs:283`): `None`, `Linear`, `Reinhard`, `ReinhardLuminance`, `AcesFitted`, `AgX`, `SomewhatBoringDisplayTransform`, `TonyMcMapface` (default), `BlenderFilmic`, `KhronosPbrNeutral`. `AgX`, `TonyMcMapface` and `BlenderFilmic` need `tonemapping_luts`, which you have. The app sets `Tonemapping::None` (`scene.rs:164`) and no `Hdr`. Related: `DebandDither` (`:351`) and `ColorGrading` (`:490`).
- **Auto exposure.** `AutoExposure` (`bevy_post_process/src/auto_exposure/settings.rs:31`) requires `Hdr` and compute shaders.
- **Fog.**
  - `DistanceFog { color, directional_light_color, directional_light_exponent, falloff: FogFalloff }` (`bevy_pbr/src/fog.rs:56`). You already use it.
  - Volumetric: `VolumetricFog` on the camera (`bevy_light/src/volumetric.rs:25`), `VolumetricLight` on the directional light (`:16`) and a `FogVolume` entity (`:78`).
- **Atmosphere.** `bevy::light::Atmosphere` (`bevy_light/src/atmosphere.rs:37`; `Atmosphere::earth(medium_handle)`) plus the asset `ScatteringMedium` (`:135`; `ScatteringMedium::earth`) on an entity, and `AtmosphereSettings` (`bevy_pbr/src/atmosphere/mod.rs:297`, `#[require(Hdr)]`) on the camera. It is compute-based and does not load without compute support or `Rgba16Float` storage (`:135-157`). It is a sky. Your CLAUDE.md says no sky, sun or grass; if you want it only for lighting, that is `AtmosphereEnvironmentMapLight`.
- **Environment maps without an HDR file.**
  - `AtmosphereEnvironmentMapLight { intensity, size, .. }` (`bevy_light/src/probe.rs:306`) generates its own environment map from the atmosphere and inserts a `GeneratedEnvironmentMapLight`.
  - `GeneratedEnvironmentMapLight { environment_map: Handle<Image>, intensity, rotation, .. }` (`probe.rs:269`) filters a source cubemap you give it. You need a cubemap image (make one procedurally), and it needs compute with at least 6 storage textures (`bevy_pbr/src/light_probe/generate.rs:113-123`).
  - Other probes: `EnvironmentMapLight` (`probe.rs:107`, prefiltered cubemaps), `Skybox` (`:235`), `IrradianceVolume` and `LightProbe` (`:338`, `:75`).
  - No studio-light environment ships with Bevy.
- **Screen-space reflections.** `ScreenSpaceReflections` (`bevy_pbr/src/ssr/mod.rs:82`) requires `DepthPrepass` and `DeferredPrepass` (deferred only). Not recommended with TAA and blended glass.
- **Shadows.**
  - `ShadowFilteringMethod` on the camera (`bevy_light/src/lib.rs:291`): `Hardware2x2`, `Gaussian` (default), `Temporal` (pair with TAA).
  - `DirectionalLight::shadow_maps_enabled` and `contact_shadows_enabled` (`directional_light.rs:93, 96`).
  - Camera component `ContactShadows { linear_steps, thickness, length }` (`bevy_pbr/src/contact_shadows.rs:37`; requires `DepthPrepass`).
  - `CascadeShadowConfig` and `CascadeShadowConfigBuilder` (`bevy_light/src/cascade.rs:26, 60`), already used at `scene.rs:154`.
  - Soft shadows are `soft_shadow_size` and need the `experimental_pbr_pcss` feature.
- **Depth of field and motion blur.** `DepthOfField { mode, focal_distance, sensor_height, aperture_f_stops, .. }` (`bevy_post_process/src/dof/mod.rs:78`; it needs `Msaa::Off`) and `MotionBlur` (`motion_blur/mod.rs:76`, requires `MotionVectorPrepass`). Neither suits a drafting view. Also `Vignette`, `ChromaticAberration` and `LensDistortion` under `effect_stack/`.
- **Glass and materials.** `StandardMaterial` (`bevy_pbr/src/pbr_material.rs`) has `specular_transmission` (`:267`), `diffuse_transmission` (`:222`), `thickness` (`:297`), `ior` (`:345`), `attenuation_distance` and `attenuation_color` (`:358`, `:370`), `clearcoat` and `clearcoat_perceptual_roughness` (`:505`, `:531`), `anisotropy_strength` and `anisotropy_rotation` (`:590`, `:609`), `reflectance`, `specular_tint`, `alpha_mode`, `unlit`, `double_sided`. Transmission renders through the transmission pass (`bevy_pbr/src/transmission/`). `ExtendedMaterial<B, E>` (`extended_material.rs:151`) lets you add a shader to the standard material. Textured anisotropy and clearcoat need the `pbr_*_texture` features.
- **Order-independent transparency.** `OrderIndependentTransparencySettings { sorted_fragment_max_count, fragments_per_pixel_average, alpha_threshold }` (`bevy_core_pipeline/src/oit/mod.rs:44`). The plugin is added by `bevy_core_pipeline` (`lib.rs:57`). It needs `Msaa::Off`.
- **Mesh picking.** `MeshPickingPlugin` (`bevy_picking/src/mesh_picking/mod.rs:67`, settings at `:40`: `require_markers`, `ray_cast_visibility`) is already added at `main.rs:157`. Dragging meshes works with `On<PointerDragStart/PointerDrag/PointerDragEnd>` observers; `PointerDrag.delta` and `.distance` are screen pixels, so you do the plane projection yourself.
- **Outlines or selection highlight.** There is none for meshes: no outline pass, no stencil. Only `Wireframe`, `WireframeColor`, `NoWireframe` and `WireframeConfig` (`bevy_pbr/src/wireframe.rs:75, 203, 856, 881, 904`; needs `WireframePlugin`), which is a triangle wireframe and not a silhouette. Least-effort permissive route: keep what you have (gizmo line frames plus emissive tint), or write an inverted-hull outline (a back-face-extruded duplicate mesh with an unlit `StandardMaterial`).
- **Gizmos.**
  - Immediate-mode `Gizmos<Group>` (`bevy_gizmos`, with `GizmoConfigStore` and `depth_bias`, `line.width`, `render_layers`) is in use.
  - **Interactive transform gizmo:** `bevy::gizmos::transform_gizmo` (`bevy_gizmos/src/transform_gizmo.rs`). It has `TransformGizmoPlugin` (`:230`), `TransformGizmoFocus` (marker on the entity, `:89`), `TransformGizmoCamera` (`:99`), resource `TransformGizmoSettings` (`:139`: `mode` Translate, Rotate or Scale; `space`; `snap_translate`, `snap_rotate`, `snap_scale`; `axis_hit_distance`; `confine_cursor`) and `TransformGizmoState` (`:185`: `hovered_axis`, `active`, ..).
  - It does not read the keyboard. It writes `Transform` directly while dragging, reading `ButtonInput<MouseButton>` and the cursor, not through picking. It is not UI-aware, so you must stop it from starting a drag when the pointer is over a panel, for example by removing `TransformGizmoFocus`.
  - It draws with a separate overlay camera on its own `RenderLayers` (`bevy_gizmos_render/src/transform_gizmo_render.rs:78`; added at `lib.rs:119`). That interacts with your `UiTargetCamera` and image-target camera plan.
  - Your element positions live in the job world (`ed::move_by` in feet), so you must read the dragged `Transform` back into a job edit.

## 3. Camera helpers, easing, wheel

- **`bevy_camera_controller`.** Features (`bevy_internal-0.20.1/Cargo.toml:84, 273, 336-344`): `bevy_camera_controller`, `free_camera`, `pan_camera`, `pan_orbit_camera`, `pan_orbit_gizmo` (adds an anchor indicator). Re-exported as `bevy::camera_controller`. Enabling any feature changes `Cargo.lock` (new entry), but the source is already on disk.
  - **`free_camera`**: `FreeCameraPlugin`, `FreeCamera`, `FreeCameraState` (`free_camera.rs:40, 76, 224`). A first-person fly camera (keyboard and mouse look, `VerticalMovementAxis`). It is a 3D fly cam, not an orbit cam.
  - **`pan_camera`**: `PanCameraPlugin`, `PanCamera` (`pan_camera.rs:29, 51`). 2D, for orthographic and plan use. Has `zoom_target: ZoomTarget::{Center, Cursor}`, `min_zoom`, `max_zoom`, mouse-pan button settings, pan and rotate keys.
  - **`pan_orbit_camera`** (`pan_orbit_camera/`): `DefaultPanOrbitCameraPlugins` (`mod.rs`, with `MinimalPanOrbitCameraPlugin`, `DollyZoomPlugin`, `LookToPlugin`, optional anchor indicator and independent skybox), component `PanOrbitCamera` (`controller/component.rs:51`). Fields: `enabled_motion {pan, orbit, zoom}`, `orbit_constraint` (`OrbitConstraint::Fixed{up, can_pass_tdc}`/`Free`, `:672`), `zoom_limits`, `smoothing`, `sensitivity`, `momentum`, `perspective`, `orthographic`, `current_motion`.
    - Defaults. `Smoothing` is 10 ms pan, 30 ms orbit, 60 ms zoom (`smoothing.rs:25-33`). `Momentum` has damping 160 and friction 0.2 (`momentum.rs:29-40`). `ZoomLimits` (`zoom.rs:9`) set pixel-size bounds (`min_size_per_pixel` and so on) and `zoom_through_objects`.
    - API: `start_orbit(anchor: Option<DVec3>)` (`:207`), `start_pan` (`:222`), `start_zoom` (`:237`), `send_screenspace_input(Vec2)` (`:259`), `send_zoom_input(f32)` (`:280`), `end_move()` (`:290`). The anchor is a view-space point, which you get from your own hit test: mesh picking (`HitData.position`) or your ground-plane ray.
    - It works in f64 view space, uses the last anchor depth when nothing is under the cursor, keeps the grabbed point stuck to the pointer, and swaps perspective and orthographic without a jump. It needs `bevy_picking`.
    - Extensions are triggers: `LookToTrigger` (`extensions/look_to.rs:36`) snaps to a view, and `DollyZoomTrigger` (`extensions/dolly_zoom.rs:44`) switches projection.
    - **You write the input system.** `controller/mod.rs` says `MinimalPanOrbitCameraPlugin` "requires an input plugin to function"; the crate has none. The upstream example is `examples/camera/pan_orbit_camera_custom_input_plugin.rs`, not in the crates.
  - **Mapping for Google-Earth feel** with what exists: left-drag pans (`start_pan`), right-drag or Alt-drag orbits (`start_orbit`) about the point under the cursor, wheel calls `start_zoom` then `send_zoom_input`, and you convert Line to Pixel first.
  - **Your own camera.** It already eases with the framerate-independent exponential `1 - exp(-9 dt)` (`scene.rs:845`), which is the same thing as `StableInterpolate::smooth_nudge`. Your Plan (orthographic) and Walk (eye-level) modes are not expressed by `PanOrbitCamera`, so you would keep custom code for them.
- **Easing.**
  - `bevy::curve::{EasingCurve, EaseFunction, Ease}` live in the `bevy_curve` crate (`easing.rs:92, 335, 473`; re-exported at `bevy_internal/src/lib.rs:40`, so `bevy::curve`).
  - `EasingCurve::new(start, end, EaseFunction::CubicInOut).sample(t)` works for `f32`, `Vec2`, `Vec3`, `Quat`, `Rot2`, `Dir3` and `Isometry3d`; `EaseFunction` includes `SmoothStep` and `SmootherStep`.
  - `bevy_math::StableInterpolate::smooth_nudge(&mut self, target, decay_rate, dt)` (`bevy_math/src/common_traits.rs:426`) is the exponential follower.
  - `bevy_animation` (in `common_api`) has `AnimationPlayer`, `AnimatableCurve` and `animated_field!(Transform::translation)` (`bevy_animation/src/lib.rs:748, 363`), but it is clip-based and heavy for UI tweens. For hover and panel slides, the least effort is a timer plus `EasingCurve::sample`, or `smooth_nudge`, in your own system. There is no ready UI tween component.
- **Wheel input.** `MouseWheel { unit: MouseScrollUnit::{Line, Pixel}, x, y, window, phase }` (`bevy_input/src/mouse.rs:214`). `AccumulatedMouseScroll { unit, delta: Vec2 }` (`:285`). `MouseScrollPixelsPerLine` resource (`:165`, default 100.0); `PointerScroll::to_pixels/to_lines` convert. X11 notch wheels typically give `Line`; smooth-scroll touchpads give `Pixel`. You already branch on this (`scene.rs:~935`, `panels.rs:1494`).
- **Gestures.** `PinchGesture`, `RotationGesture`, `DoubleTapGesture`, `PanGesture` exist (`bevy_input/src/gestures.rs:33-92`, feature `gestures`; `bevy_winit` forwards them at `state.rs:334, 343`). winit emits these on macOS and iOS only, so on X11 there are no pinch or pan events; a two-finger touchpad scroll arrives as `MouseWheel`.

## 4. The app today (`/home/user/roofnerd/plumbline/crates/app/src/`)

**Camera control** (`scene.rs`):
- **State.** `Orbit` resource `{target, yaw, pitch, distance, now, home}` (`:72`). `orbit_input` (`:910`) writes the goal; `place_camera` (`:841`) eases `now` toward it each frame with `ease = 1 - exp(-9 dt)` (`:845`) and builds the `Transform` and `Projection`. Units are feet.
- **Inputs.**
  - Right drag turns, yaw and pitch at 0.006 per px (`:914`).
  - Middle drag pans on the ground plane (`:925`). Right drag also pans on plan (`:921`).
  - Wheel zooms by `distance * (1 - steps*0.12)`, clamped 4..6000 (`:941`); on plan it zooms about the pointer.
  - W A S D move the target; Q and E go down and up; Shift is 2.5 times faster; arrows turn.
  - T toggles plan, V walk and F frame (`main.rs:~755-770`).
- **Smoothing.** Only the exponential ease on `now`. There is no momentum and no orbit about the point under the cursor; the pivot is always `orbit.target`.
- **Modes.** Plan is orthographic (`FixedVertical`, `PLAN_HEIGHT` 0.62; `:870-881`). Walk uses a 68 degree lens with eye height `EYE_FT` 5.5. Perspective lens is `LENS` 0.698 rad (40 degrees).

**Wheel routing.**
- **Zoom.** `orbit_input` zooms only when `over_model(window)` (`scene.rs:897-906`), a pure rectangle test against constants: `p.y > TOP_H && p.y < h-STATUS_H-BUILD_H && p.x > LEFT_W && p.x < w-RIGHT_W`. The constants are `TOP_H` 44, `STATUS_H` 26, `LEFT_W` 252, `RIGHT_W` 344, `BUILD_H` 104 (`:35-39`).
- **Panel scroll.** `panels::scroll_panels` (`panels.rs:1487-1519`) hand-routes the wheel by the same pixel rectangles (`x > w-RIGHT_W && y > TOP_H` goes to the inspector, `x < LEFT_W && y > TOP_H` to the browser; otherwise the sheet window). It writes `ScrollPosition` and keeps the value in `Scrolls` (`:50`), because the whole UI tree is despawned and rebuilt on each snapshot revision. Line is 36 px per notch.

**Panel layout.**
- All panels are `position_type: Absolute` nodes made by `docked()` (`panels.rs:160`): opaque `theme::panel()` white fill, a one-sided 1 px hairline.
  - Top bar: height `TOP_H`, full width (`:199`).
  - Status line: height `STATUS_H` (`:261`).
  - Left: `LEFT_W` 252, scroll_y (`:374`).
  - Right: `RIGHT_W` 344, scroll_y (`:1058`).
  - Build bar: between the side panels, `BUILD_H` 104 (`:693`).
  - A transient message overlay is centred in the free rectangle (`:921`).
- **The 3D camera already fills the whole window.** There is no `Camera::viewport`; the primary window is 1600x960 at (0,0) (`main.rs:154`). The "viewport rect" is only the constants above, used for hit testing. So the panels already float over a full-window 3D view; they are just opaque.
- The UI is drawn by the 3D camera itself (`IsDefaultUiCamera`, `scene.rs:164`; root node `UiTargetCamera(cameras.ui)`, `panels.rs:186`).
- The sheet is a second OS window: 880x960 at (1610,0) (`main.rs:186`), a `Camera2d` on `RenderLayers::layer(1)`, with its UI targeted by `UiTargetCamera(cameras.sheet)` (`sheet.rs:161`).

**"Show" list toggles.**
- `model.rs:105` `Show {architecture, roof, mechanical, electrical, plumbing, ghost, storey, certainty, clashes}`.
- `panels.rs:~512-545`: `layer(...)` makes a `row` with a 10 px `tick` box (`:278`) and a label; a click mutates `doc.show.*` and calls `doc.redraw()`, which bumps `revision` and `geometry`. `scene::rebuild` then respawns the meshes, and `panels::rebuild` the whole UI.
- Roughly eleven rows (four trades, roof, ghost, certainty, clashes); the sub-list is the part to restyle as switches.

**Selection in 3D.**
- Each mesh entity gets observers on `PointerClick`, `PointerMove`, `PointerOver` and `PointerOut` (`scene.rs:307-326`). Click calls `doc.select(path)` (only if no tool is in hand, no entry open, not swallowed). Over and Out set `doc.hover`. The ground plane's click clears the selection (`:379`). Glass ("see through") gets `Pickable::IGNORE`.
- **Drawn as** (a) gizmo line frames in the `Highlight` group, accent blue, width 4.5, `depth_bias` -0.2 (`scene.rs:187-195`; `draw_lines` selected block, `:1052-1075`), and (b) `scene::tint` (`:805-840`) changing `base_color` (mixed 16% white) and `emissive` accent*0.08 on `StandardMaterial`. Hover is a 22% lighten.

**Lights, tonemapping, AA, shadows** (`scene.rs:151-195`):
- `DirectionalLightShadowMap {size: 4096}` (`:153`).
- Cascades: `first_cascade_far_bound` 180, `maximum_distance` 1600 (`:154`).
- Key light: 1080 lux with shadows at (-76,64,-8), disabled on plan (`:168`, `:844`).
- Fill: 260 lux, no shadows (`:169`).
- `GlobalAmbientLight` 1350 (`:170`).
- Camera (full): `Msaa::Off`, `Tonemapping::None`, `DebandDither::Disabled`, `TemporalAntiAliasing`, `ScreenSpaceAmbientOcclusion::default()`, `DistanceFog` (canvas colour, linear, retuned each frame; `:164`). No `Hdr`, no `Bloom`.
- `PLUMBLINE_GRAPHICS=plain` swaps to `Msaa::Sample4` and no SSAO or TAA (`:166`).
- Ground: 24000 x 24000 plane, roughness 1.0, plus a distance-faded gizmo grid (`:376`, `:975-995`).
- Materials: `StandardMaterial`, roughness about 0.8-0.95, `reflectance` 0.3, pattern textures and normal maps from `dress.rs`, `AlphaMode::Blend` for glass and ghost. No transmission, clearcoat or environment light.

**Dragging placed things and the Move tool.**
- **No mouse dragging of 3D objects exists.**
- The Move tool (`tools.rs:475-490`) is two clicks and works on plan only: `clicks` gates on `doc.plan` (`:795-808`) and `pointer` (`:728`) snaps a ground-plane ray, using `snap::snap`. Pick one is the "from" point, pick two the "to", then `ed::move_by(job, sel, east, north)`. It is plan-only and east/north only, with no vertical movement and no free 3D drag. The selection must exist first (`:265`).
- Nothing in the window uses `MeshRayCast`; picking is through the mesh-picking observers, and the plan ray is `camera.viewport_to_world` (`:746`).
- `PointerDrag` and the transform gizmo are the two ways to add 3D dragging; both need a mapping back into `ed::move_by` feet.

**Exports from the window.** None. A search for export, IFC, xlsx, csv and `Screenshot` finds only F12 screenshots (`main.rs:310-323`, `PLUMBLINE_SHOTS`) and imports: file drop (`plans.rs:318`, `bring.rs:142`: `.ifc`, `.csv`, `.xlsx`). Exports are the CLI and `demos/export.sh`.

**Structural obstacles to a floating, full-window view.**
1. **UI shares the 3D camera's view target.** The panels are drawn by the 3D camera (`IsDefaultUiCamera`, `Cameras.ui = 3D camera`). Frosted glass needs the 3D camera to render to an `Image` and a separate UI camera (`order` > 0) on the window; then `Cameras.ui` must point at the new camera, and the image must be resized with the window. `place_labels` (`panels.rs:1428`, `world_to_viewport`) and `tools::pointer` (`viewport_to_world`) then operate in image pixels, which equal window pixels only if sizes match. UI picking and mesh picking are per camera and window; you must re-check that `Pickable` blocking still works across two cameras.
2. **Hand-written pixel rectangles are the layout, and they are used four ways.** `over_model_at` (`scene.rs:902`) for orbit and wheel, `scroll_panels` for the wheel, `map.rs:60` for off-screen clipping, and `panels.rs:921` for the overlay. Floating, resizable or collapsible panels break all of these. Replace them with "the pointer is over a UI node" (`Hovered` or the hover map), use `PointerScroll` with `propagate(false)` for panels, and define the camera's visible rectangle from the panels' `ComputedNode` if you want the model centred in free space. Today the model is centred in the whole window, so panel-covered areas hide it.
3. **The whole UI is despawned and respawned whenever `snap.revision` changes** (`panels.rs:177-196`). Hover animation, a smooth scroll, a transition or a toggle's slide would restart or snap on any edit; the `Scrolls` resource is the workaround. Any animated control needs persistent nodes, or a diff-based update, or state kept in a resource keyed by name.

Also worth knowing: text entry is not Bevy widgets. It is a hand-rolled `doc.entry` and `Entry` state machine over `KeyboardInput` (`main.rs:310-` with `Burst` laser detection, `BURST_GAP` 0.12 s), so `TextInput` from `bevy_ui_widgets` would not drop in without redoing the laser path.

**What tests and scripts depend on.**
- **`PLUMBLINE_MAP` mechanism** (`map.rs:1-98`; documented in the file header, `main.rs` header). Env var gives a file path. `map::write` writes `<window>\t<name>\t<x>\t<y>` for every `theme::Named` UI node with `InheritedVisibility`. Position is `UiGlobalTransform.translation * inverse_scale_factor` plus the window origin.
  - Coupling to layout: `target.get() == Some(cameras.ui)` decides "model" vs "sheet" (`map.rs:~56`), so the UI camera entity must stay recorded in `Cameras`. `name.scrolls` rows are dropped when `at.y < TOP_H || at.y > h - STATUS_H` (`:60`), a hard-coded constant that must change with the layout. The sheet uses `sheet::HEAD_H` and `FOOT_H`.
  - Duplicate names get `#2`, `#3` from the top of the screen down. The point reported is the node's centre.
  - The `plan\tx0\ty0\tk` line (`map.rs:84-90`) is the plan zero and px per foot from the main camera, so it moves with any camera or framing change; the `at()` and `plan_px` helpers in `demos/window.sh` use it.
- **Button names (`theme::named`, used by `button()` at `theme.rs:406`, by `row` as "Show X" and by `named("To verify")`).** Scripts click by label: `Wall`, `Save`, `Undo`, `Redo`, `3D`, `Plan`, `Walk`, `Frame`, `Starter assemblies`, `To verify`, `Show Roof` and so on. Renaming or restyling a label as an icon or switch breaks every `on "Label"` call in the `*w.sh` demos. Keep a `Named` with the same text on each control.
- **`demos/window.sh`** drives the real program under xvfb with xdotool (about 16 uses of `PLUMBLINE_MAP`/xdotool there; 131 occurrences across 23 files in the plumbline tree, from my search, which timed out and may be incomplete): `focus`, `click` (1.5 s hover, 0.7 s press), `park` = `mousemove 1000 800` ("clear of every panel and of the building", so a floating panel there breaks it), `shot` = F12 then 12 s, `on_at` = `mousemove 120 600` then wheel (`click 5`) until a row is mapped (the left panel must scroll with a real X wheel event), `sheet_at` = `mousemove 2000 600` (the sheet window), `typed`, `len`, `laser`. It also assumes `sleep 40` for the first frames on lavapipe.
- **Hard-coded pixels in demos**, tied to the plan being centred on the 1600x960 window with no panel offset: `buildw.sh:50-79` (`click 543 691`, `1057 691`, `542 650`, `1058 656`, `1064 308`, `536 302`, `900 420`, `643 479`, `904 479`), `clashw.sh:64-79` (`736 672`, `608 672`, `480 672`, and `click 2000 205` on the sheet), `draww.sh:47-70` (`600 600`, `603 597`, `760 602`, `680 600`, `760 500`, `919 500`, `680 387`, `860 387`, `630 570`), `d2.sh:42-56` (`940 470`, `946 474`, `2050 500`, `1400 500`), `d5w.sh:45-69` (`2000 600`, `800 860`, `900 470`). If the new layout moves the plan's centre, shrinks or reorders the window, or changes the camera framing, every one of these must be converted to `at()` / `plan_px` before they can be trusted. The wheel at `1400 500` (inspector) and `2000 600` (sheet) assume the right panel and sheet window positions.
- `tools/installs.sh` starts the built window and reads the libraries it opens; unaffected unless you add GPU features.

## 5. BIMCraft (`/home/user/ipedrax/bimcraft`), what is worth taking

Licence: dual MIT / Apache-2.0 (checked: `LICENSE-MIT`, `LICENSE-APACHE`, `Cargo.toml:8`). `crates/rvt/NOTICE` carries a separate Apache notice for ported rvt-rs code; irrelevant here. UI is egui, rendering is its own CPU/GPU renderer in `crates/core` and `crates/ui-egui/src/gpu.rs` (wgpu). No Bevy.

- **Camera math, most portable.** `crates/core/src/view3d.rs:3205` `Camera {azimuth, elevation, scale, target, width, height, lens}` with `orbit` (`:3466`), `pan(from_px, to_px)` (`:3473`), `zoom(factor, at_px)` (`:3479`), `orbit_about(around, up, pivot)` (`:3487`) and `zoom_about(factor, pivot)` (`:3497`) (turn or walk toward a model point so it stays on the same pixel). Plain f64 and arrays, no dependencies; about 80 lines to translate to a Bevy `Transform`. It does what `PanOrbitCamera` already does, so port only if you reject the Bevy controller's feel. `Camera::along` (`:~3510`) is a drag-along-axis solver (perspective-correct movement along an axis from two pixels), the piece you would need for dragging an element along X, Y or Z without Bevy's gizmo.
- **Interaction rules** in the `crates/ui-egui/src/orbit.rs` header (`:1-15`): a quick press that moved is treated as a click and the camera turn is undone (`Canvas::quick_press`); orbit about the point under the pointer; wheel zooms toward the cursor. Specs, not code; they apply to your input layer.
- **Navigation triad.** `crates/ui-egui/src/triad.rs` (196 lines) draws a corner axis widget and picks a view; `crates/core/src/triad.rs` has the model. The core part is self-contained; the egui drawing is not.
- **Section box and grips.** `crates/ui-egui/src/section_box.rs` (242 lines) and `view3d::box_grips` (`:2978`): face grips dragged along an axis. This is the closest thing to a drag-move in the tree; its logic is in `core`, the interaction in egui.
- **Selection outline.** In the CPU picture (`view3d.rs`, `outline`, around `:1364`) and `gpu.rs`; it is a renderer feature, not portable to Bevy.
- **Procedural textures.** `crates/core/src/texture.rs` (768 lines): a `Texture`, `Sampler` and `swatch` that draw bricks, boards and concrete at real size. Your own `dress.rs` already does this for the house, so it is only a cross-check.
- **UI theme and look.** `crates/ui-egui/src/style.rs` (751 lines): `Palette` with light, dark and several schemes, a "Floating layout" of cards with `CARD_RADIUS` 8, `WINDOW_RADIUS` 10, `GAP` 7, hairline border and soft shadow (`:1-8, :327-330`), and WCAG-style `contrast()` (`:314`) with accent-ink stepping. The idea and numbers port; the egui `Frame` and `Shadow` code does not.
- Nothing there for a Bevy transform gizmo or drag-move; use Bevy's.

## 6. Recommended build order (least effort, all permissive)

1. Enable `bevy_camera_controller` with `pan_orbit_camera` on the main camera (source on disk) and write the input system (right-drag orbit, middle or left-drag pan, wheel zoom). Keep your own Plan and Walk modes outside it, or switch the controller's `enabled_motion` off while in them.
2. Add the `Hdr` component, `Bloom` (low) and `Tonemapping::AgX` or `TonyMcMapface`. Add features `smaa_luts` and `dfg_lut` if you want SMAA or correct specular, and `ShadowFilteringMethod::Temporal` with the TAA you already have. These are the effects with the best look-to-cost ratio on software Vulkan; skip atmosphere, volumetric fog, SSR, DoF and motion blur, which are compute or deferred heavy and fight TAA and glass.
3. Frosted panels: 3D to `RenderTarget::Image`, second UI camera on the window, a `UiMaterial` with a multi-tap blur of that image, rounded mask from `border_radius_*`, plus `BoxShadow` and a `BorderGradient` hairline. Move `Cameras.ui` to the new camera.
4. Toggles, scroll and sliders: fetch `bevy_ui_widgets` (`ScrollArea`, `Checkbox`, `Slider`) for correct behaviour, add easing on top via `ScrollPosition` and `smooth_nudge` yourself; or skip the fetch and write the switch from `PointerClick` plus `UiTransform`. Skip `bevy_feathers` (needs `bevy_scene` and its own theme).
5. 3D drag-move: `TransformGizmoPlugin` and `TransformGizmoFocus` on the selected element's entity, with a system that copies the dragged `Transform` into `ed::move_by`, or `PointerDrag` observers with your ground-plane ray. Block it when the pointer is over a panel.
6. Before restructuring layout, decide how `over_model_at`, `scroll_panels` and the demos' hard-coded pixels (section 4) will be replaced; that is the larger cost, not the rendering.