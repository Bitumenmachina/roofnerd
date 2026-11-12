//! The light of the sky.
//!
//! The place the building stands in is a pale horizon, a blue overhead and a grey
//! ground (`theme::sky_horizon`, `sky_overhead`, `yard`; QUESTIONS.md Q22). Until now
//! that was a painted dome that lit nothing, and the house was lit by two lamps and a
//! flat light from everywhere. Here the same sky is made into the two pictures the
//! engine lights a scene from (`EnvironmentMapLight`): one of the light that falls on
//! a surface facing each way, one of what a surface reflects, sharp for a smooth
//! surface and blurred for a rough one. A wall facing the sky is lit by the sky, a
//! soffit by the ground, and glass and metal reflect the horizon.
//!
//! The pictures are worked out here, once, at start, in plain arithmetic: the sky is
//! the same in every direction round the compass, so what falls on a surface depends
//! only on how far up it faces, and a table of that is all there is to compute. The
//! engine can also filter a sky picture itself on the graphics card
//! (`GeneratedEnvironmentMapLight`), but that needs a compute pass every frame and a
//! card that has one; this needs neither and is the same on every machine.
//!
//! Ours: BIMCraft has no sky light. The convention for the two pictures (the diffuse
//! one holds the cosine-weighted mean of the sky; the reflection one is blurred more
//! at each smaller copy, and roughness picks the copy) is the engine's
//! (`bevy_pbr/src/light_probe/environment_map.wesl`).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDataOrder, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension};

/// How many heights the tables are worked at, from straight down to straight up.
const STEPS: usize = 65;
/// How many directions the sky is sampled in for each entry of a table.
const SAMPLES: usize = 1200;
/// The side of the reflection picture's largest copy, and how many copies it has.
const SIDE: u32 = 64;
const COPIES: u32 = 7;
/// The side of the picture of light that falls on a surface.
const FALLS_SIDE: u32 = 16;

/// What the sky sends from a direction, by how far up the direction points (-1 straight
/// down, 0 the horizon, 1 straight up): pale at the horizon, deeper overhead, and below
/// the horizon the ground. The join is eased over a few degrees so that nothing has an
/// edge reflected in it.
pub fn sky(up: f32, horizon: [f32; 3], overhead: [f32; 3], ground: [f32; 3]) -> [f32; 3] {
    let t = up.clamp(0.0, 1.0).powf(0.55);
    let above = [0, 1, 2].map(|i| horizon[i] + (overhead[i] - horizon[i]) * t);
    // From ground to sky over about four degrees either side of the horizon.
    let k = ((up + 0.07) / 0.14).clamp(0.0, 1.0);
    let k = k * k * (3.0 - 2.0 * k);
    [0, 1, 2].map(|i| ground[i] + (above[i] - ground[i]) * k)
}

/// The height each entry of a table is for.
fn height(step: usize) -> f32 {
    step as f32 / (STEPS - 1) as f32 * 2.0 - 1.0
}

/// A table by height: for a direction that far up, the mean of the sky over a lobe
/// about it. The lobe's weight is the cosine of the angle from the direction raised to
/// `power` (1 is what falls on a matte surface; larger is a tighter reflection).
fn table(power: f32, of: &dyn Fn(f32) -> [f32; 3]) -> Vec<[f32; 3]> {
    // Directions spread evenly over the sphere (a Fibonacci spiral), and the sky in each.
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let around: Vec<(Vec3, [f32; 3])> = (0..SAMPLES)
        .map(|i| {
            let y = 1.0 - (i as f32 + 0.5) / SAMPLES as f32 * 2.0;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let turn = golden * i as f32;
            (Vec3::new(turn.cos() * r, y, turn.sin() * r), of(y))
        })
        .collect();
    (0..STEPS)
        .map(|step| {
            let y = height(step);
            let way = Vec3::new((1.0 - y * y).max(0.0).sqrt(), y, 0.0);
            let (mut sum, mut weight) = ([0.0_f32; 3], 0.0_f32);
            for (to, light) in &around {
                let w = way.dot(*to).max(0.0).powf(power);
                weight += w;
                for i in 0..3 {
                    sum[i] += light[i] * w;
                }
            }
            sum.map(|s| s / weight.max(1.0e-6))
        })
        .collect()
}

/// A table read at a height between its entries.
fn read(table: &[[f32; 3]], up: f32) -> [f32; 3] {
    let at = ((up.clamp(-1.0, 1.0) + 1.0) / 2.0) * (STEPS - 1) as f32;
    let (lo, part) = ((at.floor() as usize).min(STEPS - 2), at - at.floor());
    let (a, b) = (table[lo], table[lo + 1]);
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * if at >= (STEPS - 1) as f32 { 1.0 } else { part })
}

/// How tight a reflection's lobe is for a roughness as the engine counts it (0 a
/// mirror, 1 matte): the usual match of a cosine lobe to a rough surface, 2/a^2 - 2
/// with a the roughness squared.
fn tightness(roughness: f32) -> f32 {
    let a = (roughness * roughness).max(0.02);
    (2.0 / (a * a) - 2.0).max(1.0)
}

/// How far up the middle of a dot of a cube picture's face points. The six faces are
/// in the engine's order: east, west, up, down, south, north (+X, -X, +Y, -Y, +Z, -Z).
fn up_of(face: usize, i: u32, j: u32, side: u32) -> f32 {
    let u = (i as f32 + 0.5) / side as f32 * 2.0 - 1.0;
    let v = (j as f32 + 0.5) / side as f32 * 2.0 - 1.0;
    let long = (1.0 + u * u + v * v).sqrt();
    match face {
        2 => 1.0 / long,
        3 => -1.0 / long,
        _ => -v / long,
    }
}

/// A number as the sixteen-bit float a picture of light is stored in. For what a sky
/// holds: zero or more, and not past the largest such float.
fn half(x: f32) -> u16 {
    let x = x.clamp(0.0, 65504.0);
    if x < 6.104e-5 {
        // Smaller than the smallest ordinary one: counted in steps of 2^-24.
        return (x * 16_777_216.0).round() as u16;
    }
    let bits = x.to_bits();
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mantissa = (bits >> 13) & 0x3ff;
    // Rounded to the nearest: the bit below the ten kept.
    let up = (bits >> 12) & 1;
    (((exponent as u32) << 10 | mantissa) + up) as u16
}

/// A cube picture: its six faces at each size, the largest first.
fn cube(side: u32, copies: u32, of: impl Fn(u32, f32) -> [f32; 3]) -> Image {
    let mut data: Vec<u8> = Vec::new();
    for copy in 0..copies {
        let s = (side >> copy).max(1);
        for face in 0..6 {
            for j in 0..s {
                for i in 0..s {
                    let [r, g, b] = of(copy, up_of(face, i, j, s));
                    for x in [r, g, b, 1.0] {
                        data.extend_from_slice(&half(x).to_le_bytes());
                    }
                }
            }
        }
    }
    let mut image = Image::default();
    image.data = Some(data);
    image.data_order = TextureDataOrder::MipMajor;
    image.texture_descriptor.size = Extent3d { width: side, height: side, depth_or_array_layers: 6 };
    image.texture_descriptor.dimension = TextureDimension::D2;
    image.texture_descriptor.format = TextureFormat::Rgba16Float;
    image.texture_descriptor.mip_level_count = copies;
    image.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::Cube), ..default() });
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image
}

/// The two pictures of the sky's light: what falls on a surface, and what one reflects.
pub fn light(horizon: [f32; 3], overhead: [f32; 3], ground: [f32; 3]) -> (Image, Image) {
    let of = move |up: f32| sky(up, horizon, overhead, ground);
    let falls = table(1.0, &of);
    // The sharpest copy is the sky itself; each smaller one is for a rougher surface.
    let blurred: Vec<Vec<[f32; 3]>> = (1..COPIES).map(|copy| table(tightness(copy as f32 / (COPIES - 1) as f32), &of)).collect();
    let falls = cube(FALLS_SIDE, 1, |_, up| read(&falls, up));
    let reflects = cube(SIDE, COPIES, |copy, up| if copy == 0 { of(up) } else { read(&blurred[copy as usize - 1], up) });
    (falls, reflects)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HORIZON: [f32; 3] = [0.9, 0.9, 0.9];
    const OVERHEAD: [f32; 3] = [0.3, 0.5, 0.9];
    const GROUND: [f32; 3] = [0.4, 0.4, 0.4];

    #[test]
    fn a_sky_the_same_all_round_lights_every_surface_the_same() {
        // Under an even sky of 0.5 a surface facing any way receives 0.5: the picture of
        // what falls holds the mean of the sky, not its sum.
        let even = |_: f32| [0.5, 0.5, 0.5];
        for power in [1.0, 8.0, 200.0] {
            for entry in table(power, &even) {
                for c in entry {
                    assert!((c - 0.5).abs() < 1.0e-3, "{c} under an even sky of 0.5 at lobe {power}");
                }
            }
        }
    }

    #[test]
    fn a_wall_is_lit_by_half_sky_and_half_ground_and_a_soffit_by_the_ground() {
        let of = |up: f32| sky(up, HORIZON, OVERHEAD, GROUND);
        let falls = table(1.0, &of);
        let (down, level, up) = (read(&falls, -1.0), read(&falls, 0.0), read(&falls, 1.0));
        // Facing down: the ground and nothing else, but for the eased join at the horizon,
        // which a surface facing straight down sees edge on.
        assert!((down[0] - 0.4).abs() < 0.02, "a soffit is lit by the ground: {down:?}");
        // Facing up: all sky, between the horizon's 0.9 and the overhead's 0.3 in red.
        assert!(up[0] > 0.45 && up[0] < 0.75, "a flat roof is lit by the sky: {up:?}");
        // Facing level: between the two, and bluer than the ground.
        assert!(level[0] > down[0] && level[0] < up[0], "{down:?} {level:?} {up:?}");
        assert!(level[2] > level[0], "a wall takes blue from the sky: {level:?}");
    }

    #[test]
    fn a_smooth_surface_reflects_the_sky_itself_and_a_rough_one_a_blur_of_it() {
        let of = |up: f32| sky(up, HORIZON, OVERHEAD, GROUND);
        let tight = table(tightness(1.0 / 6.0), &of);
        let loose = table(tightness(1.0), &of);
        // Just above the horizon the sky is at its palest; a tight lobe there is close to
        // it, a loose one has the ground and the overhead mixed in and is darker.
        let (exact, near, far) = (of(0.15)[0], read(&tight, 0.15)[0], read(&loose, 0.15)[0]);
        assert!((near - exact).abs() < 0.08, "tight {near} against the sky's {exact}");
        assert!(far < near - 0.1, "loose {far} against tight {near}");
    }

    #[test]
    fn sixteen_bit_floats_are_the_standard_ones() {
        assert_eq!(half(0.0), 0x0000);
        assert_eq!(half(1.0), 0x3c00);
        assert_eq!(half(0.5), 0x3800);
        assert_eq!(half(2.0), 0x4000);
        assert_eq!(half(65504.0), 0x7bff);
        // 1/3 is 0x3555 to the nearest.
        assert_eq!(half(1.0 / 3.0), 0x3555);
    }

    #[test]
    fn the_pictures_are_whole() {
        let (falls, reflects) = light(HORIZON, OVERHEAD, GROUND);
        // Six faces, four numbers of two bytes to a dot.
        assert_eq!(falls.data.as_ref().map(Vec::len), Some((FALLS_SIDE * FALLS_SIDE * 6 * 8) as usize));
        let dots: u32 = (0..COPIES).map(|c| (SIDE >> c) * (SIDE >> c) * 6).sum();
        assert_eq!(reflects.data.as_ref().map(Vec::len), Some((dots * 8) as usize));
        // The up face of the sharpest copy is the overhead's blue, the down face the ground.
        assert!((up_of(2, 32, 32, 64) - 1.0).abs() < 0.01 && (up_of(3, 32, 32, 64) + 1.0).abs() < 0.01);
        assert!(up_of(0, 10, 0, 64) > 0.5 && up_of(5, 10, 63, 64) < -0.5, "the top row of a side face looks up");
    }
}
