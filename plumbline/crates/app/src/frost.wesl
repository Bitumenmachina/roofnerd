// Frosted glass behind a panel: the scene as a small picture, blurred, under a tint.
// (crates/app/src/glass.rs has what this is for.)
import bevy_render::view::View;
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

@group(0) @binding(0)
var<uniform> view: View;

struct Frost {
    // The glass's own colour, and in its alpha how thick it is: how much of the tint
    // is laid over what is behind.
    tint: vec4<f32>,
    // How far the blur reaches, as a share of the window's height.
    reach: f32,
}

@group(1) @binding(0)
var<uniform> frost: Frost;
@group(1) @binding(1)
var scene: texture_2d<f32>;
@group(1) @binding(2)
var scene_sampler: sampler;

// How far a place is outside a rectangle with round corners (below zero: inside).
fn outside(p: vec2<f32>, half: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Where this dot of the panel is in the window, 0 to 1.
    let at = in.position.xy / view.viewport.zw;
    // Thirteen looks at the small picture, in two rings round the place: near and far.
    let aspect = view.viewport.w / view.viewport.z;
    let step = vec2<f32>(frost.reach * aspect, frost.reach);
    var sum = textureSampleLevel(scene, scene_sampler, at, 0.0).rgb * 2.0;
    var weight = 2.0;
    for (var i = 0; i < 6; i++) {
        let turn = f32(i) * 1.0471976;
        let near = vec2<f32>(cos(turn), sin(turn)) * step * 0.5;
        let far = vec2<f32>(cos(turn + 0.5235988), sin(turn + 0.5235988)) * step;
        sum += textureSampleLevel(scene, scene_sampler, at + near, 0.0).rgb * 1.5;
        sum += textureSampleLevel(scene, scene_sampler, at + far, 0.0).rgb;
        weight += 2.5;
    }
    let behind = sum / weight;
    let glass = mix(behind, frost.tint.rgb, frost.tint.a);
    // Round at the corners, as the panel's own edge and shadow are.
    let p = (in.uv - vec2<f32>(0.5)) * in.size;
    let d = outside(p, in.size * 0.5, in.border_radius_x.x);
    let cover = 1.0 - smoothstep(-0.75, 0.75, d);
    return vec4<f32>(glass, cover);
}
