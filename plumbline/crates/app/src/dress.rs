//! What the house is dressed in (DECISIONS.md call 58): the patterns of its claddings and
//! roofings, made here at their real sizes, and the parts of a window and a door.
//!
//! Nothing is a photograph and nothing is brought in from a file: each pattern is worked
//! out from the sizes the trade builds to (a modular brick is 8 in by 2 2/3 in with its
//! joint; a lap is its exposure; a seam is every 16 in) and tiles without a break. Each
//! is matte. What a thing is dressed in is read in `plumbline_world::finish`, from what
//! the estimate prices on it; this module only draws it.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};
use plumbline_world::finish::{Cladding, Leaf, Lites, OpeningDress, Roofing, Sash};
use std::collections::HashMap;

const IN: f32 = 1.0 / 12.0;

/// One pattern: which it is, and how big one repeat of it is on the building.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Lap,
    Batten,
    Shingle,
    Brick,
    Stone,
    Stucco,
    Asphalt,
    Seam,
    Slate,
    Tile,
    Shake,
    Membrane,
    Vertical,
    Sheet,
    Block,
}

impl Look {
    pub fn of_cladding(c: Cladding) -> Look {
        match c {
            Cladding::LapSiding => Look::Lap,
            Cladding::VerticalSiding => Look::Vertical,
            Cladding::BoardAndBatten => Look::Batten,
            Cladding::Shingles => Look::Shingle,
            Cladding::Panel => Look::Sheet,
            // Ribbed metal on a wall is the roof's panel stood up: its ribs run up the wall.
            Cladding::MetalPanel => Look::Seam,
            Cladding::Brick => Look::Brick,
            Cladding::Block => Look::Block,
            Cladding::Stone => Look::Stone,
            Cladding::Stucco => Look::Stucco,
        }
    }
    pub fn of_roofing(r: Roofing) -> Look {
        match r {
            Roofing::AsphaltShingles => Look::Asphalt,
            Roofing::StandingSeam | Roofing::MetalPanel => Look::Seam,
            // Metal shingles lie in courses of small pieces, as slate does.
            Roofing::MetalShingles | Roofing::Slate => Look::Slate,
            Roofing::Tile => Look::Tile,
            Roofing::Shakes => Look::Shake,
            Roofing::Membrane => Look::Membrane,
        }
    }
    /// The size of one repeat, in feet: across, and up the wall or up the slope.
    fn tile(self) -> [f32; 2] {
        match self {
            Look::Lap => [8.0, 2.0],
            Look::Batten => [16.0 * IN, 16.0 * IN],
            Look::Shingle => [4.0, 30.0 * IN],
            Look::Brick => [32.0 * IN, 32.0 * IN],
            Look::Stone => [4.0, 4.0],
            Look::Stucco => [4.0, 4.0],
            Look::Asphalt => [40.0 * IN, 45.0 * IN],
            Look::Seam => [16.0 * IN, 16.0 * IN],
            Look::Slate => [40.0 * IN, 45.0 * IN],
            Look::Tile => [32.0 * IN, 26.0 * IN],
            Look::Shake => [4.0, 40.0 * IN],
            Look::Membrane => [4.0, 4.0],
            Look::Vertical => [16.0 * IN, 16.0 * IN],
            Look::Sheet => [4.0, 8.0],
            Look::Block => [32.0 * IN, 16.0 * IN],
        }
    }
    /// The color it is when no line names one.
    fn tone(self) -> [u8; 3] {
        match self {
            Look::Lap | Look::Batten | Look::Vertical | Look::Sheet => [0xE6, 0xE1, 0xD6],
            Look::Block => [0xB4, 0xB2, 0xAC],
            Look::Shingle => [0xA8, 0x93, 0x7A],
            Look::Brick => [0x98, 0x50, 0x3C],
            Look::Stone => [0x9E, 0x98, 0x8C],
            Look::Stucco => [0xE2, 0xDA, 0xCB],
            Look::Asphalt => [0x5E, 0x5B, 0x57],
            Look::Seam => [0x5C, 0x61, 0x66],
            Look::Slate => [0x62, 0x68, 0x6E],
            Look::Tile => [0xB5, 0x65, 0x4A],
            Look::Shake => [0x8F, 0x75, 0x58],
            Look::Membrane => [0x4A, 0x4C, 0x4F],
        }
    }
    /// How rough and how much a metal it is: all matte but metal roofing.
    fn surface(self, tone: [u8; 3]) -> (f32, f32) {
        match self {
            // Bare copper is the one bright metal; painted panels are a satin.
            Look::Seam if tone == [0xB7, 0x73, 0x3B] => (0.42, 0.7),
            Look::Seam => (0.55, 0.3),
            Look::Slate => (0.8, 0.0),
            Look::Tile => (0.85, 0.0),
            _ => (0.95, 0.0),
        }
    }
    /// How deep its relief is drawn.
    fn relief(self) -> f32 {
        match self {
            Look::Lap => 2.2,
            Look::Batten => 2.6,
            Look::Shingle | Look::Shake => 2.4,
            Look::Brick => 2.0,
            Look::Stone => 3.2,
            Look::Stucco => 1.2,
            Look::Asphalt => 1.6,
            Look::Seam => 3.0,
            Look::Slate => 1.8,
            Look::Tile => 3.4,
            Look::Membrane => 0.3,
            Look::Vertical => 2.0,
            Look::Sheet => 1.4,
            Look::Block => 2.0,
        }
    }
}

/// A pattern ready to put on a surface.
#[derive(Clone)]
pub struct Pattern {
    pub colour: Handle<Image>,
    pub relief: Handle<Image>,
    pub tile: [f32; 2],
    pub rough: f32,
    pub metal: f32,
}

/// Every pattern made so far, by what it is and its color.
#[derive(Resource, Default)]
pub struct Patterns {
    made: HashMap<(Look, [u8; 3]), Pattern>,
}

impl Patterns {
    pub fn get(&mut self, images: &mut Assets<Image>, look: Look, tone: Option<[u8; 3]>) -> Pattern {
        let tone = tone.unwrap_or(look.tone());
        self.made
            .entry((look, tone))
            .or_insert_with(|| {
                let (colour, relief) = paint(look, tone);
                let (rough, metal) = look.surface(tone);
                Pattern { colour: images.add(colour), relief: images.add(relief), tile: look.tile(), rough, metal }
            })
            .clone()
    }
}

// ---- Noise that repeats, so a pattern tiles ----

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0x00FF_FFFF) as f32 / 16_777_215.0
}

/// Smooth noise over a lattice `px` by `py` cells that wraps at its edges.
fn noise(x: f32, y: f32, px: i32, py: i32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let at = |i: i32, j: i32| hash((x0 as i32 + i).rem_euclid(px), (y0 as i32 + j).rem_euclid(py), seed);
    let (a, b) = (at(0, 0) + (at(1, 0) - at(0, 0)) * sx, at(0, 1) + (at(1, 1) - at(0, 1)) * sx);
    a + (b - a) * sy
}

/// Several sizes of it laid over each other; `u` and `v` run 0 to 1 across the repeat.
fn cloud(u: f32, v: f32, cx: i32, cy: i32, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut fx, mut fy) = (0.0, 0.5, 0.0, cx, cy);
    for o in 0..octaves {
        sum += amp * noise(u * fx as f32, v * fy as f32, fx, fy, seed + o);
        norm += amp;
        amp *= 0.5;
        fx *= 2;
        fy *= 2;
    }
    sum / norm
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Pieces of random length laid end to end along a course `total` long, the last made to
/// close it: where each starts, for a course and a seed. The same course always cuts the same.
fn cuts(total: f32, least: f32, most: f32, course: i32, seed: u32) -> Vec<f32> {
    let mut at = vec![0.0];
    let mut x = 0.0;
    let mut k = 0;
    loop {
        let w = least + (most - least) * hash(k, course, seed);
        if x + w > total - least {
            break;
        }
        x += w;
        at.push(x);
        k += 1;
    }
    at
}

/// Which piece of a course a place falls in, how far into it, and how wide it is.
fn piece_at(cuts: &[f32], total: f32, x: f32) -> (usize, f32, f32) {
    let i = cuts.iter().rposition(|c| *c <= x).unwrap_or(0);
    let end = cuts.get(i + 1).copied().unwrap_or(total);
    (i, x - cuts[i], end - cuts[i])
}

const N: usize = 512;

/// The color (a share of the tone, per channel) and the height of the surface at one
/// place of a pattern. `u` runs across the repeat and `v` down it, each 0 to 1.
fn at(look: Look, u: f32, v: f32, courses: &[Vec<f32>]) -> ([f32; 3], f32) {
    let [w, h] = look.tile();
    let (x, y) = (u * w * 12.0, v * h * 12.0); // inches across and down the repeat
    let flat = |s: f32| [s, s, s];
    match look {
        Look::Lap => {
            // Four courses at 6 in to the weather, each board 8 ft long.
            let c = (v * 4.0).floor();
            let f = v * 4.0 - c;
            let under = 0.58 + 0.42 * smooth(0.0, 0.10, f);
            let grain = 0.985 + 0.03 * noise(u * 14.0, v * 260.0, 14, 260, 7);
            let joint_x = hash(c as i32, 3, 11) * w * 12.0;
            let d = (x - joint_x).abs().min(w * 12.0 - (x - joint_x).abs());
            let joint = if d < 0.09 { 0.8 } else { 1.0 };
            (flat(under * grain * joint * (0.97 + 0.03 * f)), f)
        }
        Look::Batten => {
            // A 2 1/2 in batten over each joint, 16 in apart.
            let d = (u - 0.5).abs() * 16.0;
            let on = d < 1.25;
            let side = u - 0.5;
            let shadow = if !on && side > 0.0 && d < 2.1 { 0.70 + 0.30 * smooth(1.25, 2.1, d) } else if !on && d < 1.45 { 0.86 } else { 1.0 };
            let grain = 0.985 + 0.03 * noise(u * 90.0, v * 5.0, 90, 5, 9);
            (flat(shadow * grain * if on { 1.02 } else { 0.985 }), if on { 1.0 } else { 0.0 })
        }
        Look::Shingle | Look::Shake => {
            let n = if look == Look::Shingle { 6.0 } else { 4.0 };
            let c = (v * n).floor();
            let f = v * n - c;
            let (i, into, wide) = piece_at(&courses[c as usize], w * 12.0, x);
            let edge = into.min(wide - into);
            let rough: f32 = if look == Look::Shake { 1.6 } else { 1.0 };
            let shade = 0.82 + 0.30 * hash(i as i32, c as i32, 21) * rough.min(1.2);
            let warm = hash(i as i32, c as i32, 22) - 0.5;
            let under = 0.52 + 0.48 * smooth(0.0, 0.13, f);
            let gap = 0.45 + 0.55 * smooth(0.0, 0.16, edge);
            let grain = 0.94 + 0.12 * rough * noise(u * 150.0, v * 9.0, 150, 9, 23) * (0.6 + 0.4 * f);
            let s = shade * under * gap * grain;
            ([s * (1.0 + 0.06 * warm), s, s * (1.0 - 0.08 * warm)], f * 0.8 + 0.2 * hash(i as i32, c as i32, 24) + if edge < 0.12 { -0.5 } else { 0.0 })
        }
        Look::Brick => {
            // Modular brick in running bond: 8 in by 2 2/3 in with its 3/8 in joint.
            let c = (y / (8.0 / 3.0)).floor();
            let fy = y - c * 8.0 / 3.0;
            let bx = (x + if (c as i32) % 2 == 1 { 4.0 } else { 0.0 }).rem_euclid(32.0);
            let b = (bx / 8.0).floor();
            let fx = bx - b * 8.0;
            let joint = fx.min(fy);
            if joint < 0.375 {
                // The joint, struck back: its upper part is in the shadow of the brick over it.
                let m = 0.80 + 0.08 * cloud(u, v, 48, 48, 2, 31);
                let shaded = if fy < 0.2 && fx >= 0.375 { 0.78 } else { 1.0 };
                // Mortar keeps its own color whatever the brick: carried as a mark.
                ([-m * shaded, 0.0, 0.0], 0.0)
            } else {
                let k = hash(b as i32, c as i32, 33);
                let flashed = if hash(b as i32, c as i32, 34) < 0.12 { 0.74 } else { 1.0 };
                let face = 0.92 + 0.16 * cloud(u, v, 40, 40, 3, 35);
                let s = (0.86 + 0.26 * k) * flashed * face;
                let warm = hash(b as i32, c as i32, 36) - 0.5;
                ([s * (1.0 + 0.05 * warm), s * (1.0 - 0.03 * warm), s * (1.0 - 0.08 * warm)], smooth(0.375, 0.55, joint))
            }
        }
        Look::Stone => {
            // Coursed ashlar: courses of 4, 6 and 8 in, stones 8 to 22 in long.
            const RISE: [f32; 8] = [8.0, 6.0, 4.0, 6.0, 8.0, 4.0, 6.0, 6.0];
            let (mut c, mut top) = (0, 0.0);
            while c < 7 && y >= top + RISE[c] {
                top += RISE[c];
                c += 1;
            }
            let fy = y - top;
            let (i, into, wide) = piece_at(&courses[c], 48.0, x);
            let joint = into.min(wide - into).min(fy).min(RISE[c] - fy);
            let face = cloud(u, v, 24, 24, 4, 41);
            if joint < 0.22 {
                ([-(0.36 + 0.06 * face), 0.0, 0.0], 0.0)
            } else {
                let k = hash(i as i32, c as i32, 43);
                let warm = hash(i as i32, c as i32, 44) - 0.5;
                let s = (0.80 + 0.36 * k) * (0.86 + 0.28 * face) * (0.75 + 0.25 * smooth(0.22, 0.7, joint));
                ([s * (1.0 + 0.10 * warm), s, s * (1.0 - 0.12 * warm)], 0.45 + 0.55 * face * smooth(0.22, 0.9, joint))
            }
        }
        Look::Stucco | Look::Membrane => {
            let g = cloud(u, v, 64, 64, 4, 51);
            let amount = if look == Look::Stucco { 0.07 } else { 0.04 };
            (flat(1.0 - amount / 2.0 + amount * g), g)
        }
        Look::Asphalt => {
            // Laminated shingles, 5 5/8 in to the weather: tabs of uneven width, every
            // other one a second layer that stands up and casts a line of shadow.
            let c = (v * 8.0).floor();
            let f = v * 8.0 - c;
            let (i, into, wide) = piece_at(&courses[c as usize], w * 12.0, x);
            let raised = i % 2 == 0;
            let edge = into.min(wide - into);
            let grit = 0.90 + 0.20 * hash((u * N as f32) as i32, (v * N as f32) as i32, 61);
            let blend = 0.90 + 0.20 * cloud(u, v, 10, 12, 3, 62);
            let tab = if raised { 1.06 } else { 0.90 } * (0.94 + 0.12 * hash(i as i32, c as i32, 63));
            let under = 0.55 + 0.45 * smooth(0.0, 0.14, f);
            let step = if !raised && edge < 0.35 { 0.78 } else { 1.0 };
            (flat(grit * blend * tab * under * step), f * 0.7 + if raised { 0.3 } else { 0.0 })
        }
        Look::Seam => {
            // A 1 in seam every 16 in, standing an inch; the pan between is flat.
            let d = (u - 0.5) * 16.0;
            let on = d.abs() < 0.5;
            // The seam reads from far off as a darker line: its lit side is narrower than
            // its shaded side and the shadow it throws on the pan.
            let s = if on {
                if d < -0.25 { 1.06 } else { 0.62 }
            } else if d > 0.0 && d < 2.2 {
                0.66 + 0.34 * smooth(0.5, 2.2, d)
            } else {
                0.985 + 0.03 * cloud(u, v, 3, 5, 2, 71)
            };
            (flat(s), if on { 1.0 } else { 0.0 })
        }
        Look::Slate => {
            // 10 in slates, 7 1/2 in to the weather, each course broken half over the last.
            let c = (v * 6.0).floor();
            let f = v * 6.0 - c;
            let bx = (x + if (c as i32) % 2 == 1 { 5.0 } else { 0.0 }).rem_euclid(40.0);
            let b = (bx / 10.0).floor();
            let fx = bx - b * 10.0;
            let edge = fx.min(10.0 - fx);
            let k = hash(b as i32, c as i32, 81);
            let hue = hash(b as i32, c as i32, 82);
            let cleft = 0.93 + 0.14 * cloud(u, v, 20, 22, 3, 83);
            let under = 0.55 + 0.45 * smooth(0.0, 0.10, f);
            let gap = 0.5 + 0.5 * smooth(0.0, 0.14, edge);
            let s = (0.84 + 0.28 * k) * cleft * under * gap;
            // Most are gray; a few run to green and a few to plum, as a mixed lot does.
            let tint = if hue < 0.12 { [0.96, 1.04, 0.98] } else if hue > 0.90 { [1.05, 0.96, 1.02] } else { [1.0, 1.0, 1.0] };
            ([s * tint[0], s * tint[1], s * tint[2]], f * 0.7 + 0.3 * k)
        }
        Look::Vertical => {
            // Plywood or board siding with a groove every 8 in, running up the wall.
            let d = ((u * 2.0).fract() - 0.5).abs() * 8.0;
            let groove = d < 0.25;
            let grain = 0.985 + 0.03 * noise(u * 90.0, v * 5.0, 90, 5, 12);
            (flat(grain * if groove { 0.62 } else if d < 0.5 { 0.9 } else { 1.0 }), if groove { 0.0 } else { 1.0 })
        }
        Look::Sheet => {
            // Panels 4 ft by 8 ft with an open joint between them.
            let edge = (u.min(1.0 - u) * 48.0).min(v.min(1.0 - v) * 96.0);
            let face = 0.985 + 0.03 * cloud(u, v, 16, 32, 2, 13);
            (flat(face * if edge < 0.2 { 0.55 } else { 1.0 }), if edge < 0.2 { 0.0 } else { 1.0 })
        }
        Look::Block => {
            // Concrete block in running bond: 16 in by 8 in with its 3/8 in joint.
            let c = (y / 8.0).floor();
            let fy = y - c * 8.0;
            let bx = (x + if (c as i32) % 2 == 1 { 8.0 } else { 0.0 }).rem_euclid(32.0);
            let b = (bx / 16.0).floor();
            let fx = bx - b * 16.0;
            let joint = fx.min(fy);
            if joint < 0.375 {
                ([-(0.74 + 0.06 * cloud(u, v, 48, 24, 2, 14)), 0.0, 0.0], 0.0)
            } else {
                let s = (0.93 + 0.12 * hash(b as i32, c as i32, 15)) * (0.92 + 0.16 * cloud(u, v, 64, 32, 4, 16));
                (flat(s), smooth(0.375, 0.6, joint))
            }
        }
        Look::Tile => {
            // Barrel tile: 8 in across each, 13 in to the weather.
            let c = (v * 2.0).floor();
            let f = v * 2.0 - c;
            let col = (u * 4.0).floor();
            let fx = u * 4.0 - col;
            let round = 0.5 + 0.5 * (std::f32::consts::TAU * (fx - 0.5)).cos();
            let k = hash(col as i32, c as i32, 91);
            let under = 0.60 + 0.40 * smooth(0.0, 0.07, f);
            let s = (0.88 + 0.22 * k) * (0.66 + 0.34 * round) * under * (0.95 + 0.10 * cloud(u, v, 24, 24, 2, 92));
            (flat(s), round * 0.8 + f * 0.2)
        }
    }
}

fn to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// Make one pattern in one color: its color picture and its relief, each with every
/// smaller size of itself so that it stays still and unspeckled from far off.
fn paint(look: Look, tone: [u8; 3]) -> (Image, Image) {
    let [w, _] = look.tile();
    // The pieces of each course, cut once.
    let courses: Vec<Vec<f32>> = match look {
        Look::Shingle => (0..6).map(|c| cuts(w * 12.0, 3.5, 7.5, c, 20)).collect(),
        Look::Shake => (0..4).map(|c| cuts(w * 12.0, 4.0, 10.0, c, 25)).collect(),
        Look::Stone => (0..8).map(|c| cuts(48.0, 8.0, 22.0, c, 40)).collect(),
        Look::Asphalt => (0..8).map(|c| cuts(w * 12.0, 3.5, 9.0, c, 60)).collect(),
        _ => Vec::new(),
    };
    let base = [f32::from(tone[0]) / 255.0, f32::from(tone[1]) / 255.0, f32::from(tone[2]) / 255.0];
    // Surfaces are drawn a little under white so that one picked or pointed at can lighten.
    let lift = 1.0 / 0.90;
    let mortar = [0.82, 0.80, 0.76];
    let mut rgb = vec![[0.0f32; 3]; N * N];
    let mut high = vec![0.0f32; N * N];
    for j in 0..N {
        for i in 0..N {
            let (share, h) = at(look, (i as f32 + 0.5) / N as f32, (j as f32 + 0.5) / N as f32, &courses);
            let c = if share[0] < 0.0 { [mortar[0] * -share[0], mortar[1] * -share[0], mortar[2] * -share[0]] } else { [base[0] * share[0], base[1] * share[1], base[2] * share[2]] };
            rgb[j * N + i] = [to_linear((c[0] * lift).clamp(0.0, 1.0)), to_linear((c[1] * lift).clamp(0.0, 1.0)), to_linear((c[2] * lift).clamp(0.0, 1.0))];
            high[j * N + i] = h;
        }
    }
    // The relief: which way the surface leans at each place, from its height. Toward the
    // top of the picture is up the wall, or up the slope.
    let k = look.relief();
    let mut lean = vec![[0.0f32; 3]; N * N];
    for j in 0..N {
        for i in 0..N {
            let h = |di: isize, dj: isize| high[((j as isize + dj).rem_euclid(N as isize) as usize) * N + (i as isize + di).rem_euclid(N as isize) as usize];
            let v = Vec3::new(-(h(1, 0) - h(-1, 0)) * k, -(h(0, -1) - h(0, 1)) * k, 1.0).normalize();
            lean[j * N + i] = [v.x, v.y, v.z];
        }
    }
    let (mut colour, mut relief) = (Vec::new(), Vec::new());
    let (mut size, mut levels) = (N, 0);
    loop {
        for p in &rgb {
            colour.extend([(to_srgb(p[0]) * 255.0).round() as u8, (to_srgb(p[1]) * 255.0).round() as u8, (to_srgb(p[2]) * 255.0).round() as u8, 255]);
        }
        for p in &lean {
            let v = Vec3::from_array(*p).normalize_or(Vec3::Z);
            relief.extend([((v.x * 0.5 + 0.5) * 255.0).round() as u8, ((v.y * 0.5 + 0.5) * 255.0).round() as u8, ((v.z * 0.5 + 0.5) * 255.0).round() as u8, 255]);
        }
        levels += 1;
        if size == 1 {
            break;
        }
        let half = size / 2;
        let shrink = |from: &Vec<[f32; 3]>| -> Vec<[f32; 3]> {
            (0..half * half)
                .map(|n| {
                    let (i, j) = (n % half, n / half);
                    let mut s = [0.0; 3];
                    for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let p = from[(j * 2 + dj) * size + i * 2 + di];
                        s = [s[0] + p[0] / 4.0, s[1] + p[1] / 4.0, s[2] + p[2] / 4.0];
                    }
                    s
                })
                .collect()
        };
        rgb = shrink(&rgb);
        lean = shrink(&lean);
        size = half;
    }
    let image = |data: Vec<u8>, format: TextureFormat| {
        let mut image = Image::new_uninit(Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 }, TextureDimension::D2, format, RenderAssetUsages::RENDER_WORLD);
        image.data = Some(data);
        image.texture_descriptor.mip_level_count = levels;
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            anisotropy_clamp: 16,
            ..default()
        });
        image
    };
    (image(colour, TextureFormat::Rgba8UnormSrgb), image(relief, TextureFormat::Rgba8Unorm))
}

// ---- Surfaces with a pattern laid on them ----

/// Flat surfaces to be drawn in one material, each place on them knowing where in a
/// pattern it falls and which way the pattern runs.
#[derive(Default)]
pub struct Skin {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    places: Vec<[f32; 2]>,
    runs: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

/// How a pattern lies on a flat surface: the way it runs across, the way its top is, and
/// the size of one repeat.
#[derive(Clone, Copy)]
pub struct Lay {
    pub across: Vec3,
    pub up: Vec3,
    pub tile: [f32; 2],
}

impl Skin {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
    /// A flat piece: its corners, how it is cut into triangles, the way it faces, and how
    /// a pattern lies on it (None: it is plain).
    pub fn add(&mut self, points: &[Vec3], tris: &[[u32; 3]], normal: Vec3, lay: Option<Lay>) {
        let base = self.positions.len() as u32;
        let run = match lay {
            // The pattern's top is toward `up`: the third way of the three is that or its opposite.
            Some(l) => [l.across.x, l.across.y, l.across.z, if normal.cross(l.across).dot(l.up) < 0.0 { -1.0 } else { 1.0 }],
            None => {
                let t = normal.any_orthonormal_vector();
                [t.x, t.y, t.z, 1.0]
            }
        };
        for p in points {
            self.positions.push([p.x, p.y, p.z]);
            self.normals.push([normal.x, normal.y, normal.z]);
            self.places.push(lay.map(|l| [p.dot(l.across) / l.tile[0], -p.dot(l.up) / l.tile[1]]).unwrap_or([0.0, 0.0]));
            self.runs.push(run);
        }
        for t in tris {
            let (a, b, c) = (points[t[0] as usize], points[t[1] as usize], points[t[2] as usize]);
            let wound = if (b - a).cross(c - a).dot(normal) < 0.0 { [t[0], t[2], t[1]] } else { *t };
            self.indices.extend(wound.iter().map(|i| base + i));
        }
    }
    pub fn quad(&mut self, corners: [Vec3; 4], normal: Vec3, lay: Option<Lay>) {
        self.add(&corners, &[[0, 1, 2], [0, 2, 3]], normal, lay);
    }
    /// A plain block in a part's own directions: from and to along each of the three.
    pub fn block(&mut self, f: &Part, x: [f32; 2], y: [f32; 2], z: [f32; 2]) {
        if x[1] - x[0] < 1e-4 || y[1] - y[0] < 1e-4 || z[1] - z[0] < 1e-4 {
            return;
        }
        let p = |a: usize, b: usize, c: usize| f.at(x[a], y[b], z[c]);
        self.quad([p(0, 0, 1), p(1, 0, 1), p(1, 1, 1), p(0, 1, 1)], f.z, None);
        self.quad([p(0, 0, 0), p(1, 0, 0), p(1, 1, 0), p(0, 1, 0)], -f.z, None);
        self.quad([p(0, 0, 0), p(0, 0, 1), p(0, 1, 1), p(0, 1, 0)], -f.x, None);
        self.quad([p(1, 0, 0), p(1, 0, 1), p(1, 1, 1), p(1, 1, 0)], f.x, None);
        self.quad([p(0, 0, 0), p(1, 0, 0), p(1, 0, 1), p(0, 0, 1)], -f.y, None);
        self.quad([p(0, 1, 0), p(1, 1, 0), p(1, 1, 1), p(0, 1, 1)], f.y, None);
    }
    pub fn mesh(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.places)
            .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, self.runs)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// Where a part of the building stands and which way it is turned: a corner, and the
/// ways across it, up it and out through it.
pub struct Part {
    pub o: Vec3,
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Part {
    pub fn at(&self, x: f32, y: f32, z: f32) -> Vec3 {
        self.o + self.x * x + self.y * y + self.z * z
    }
}

/// The parts of a window or a door, each to be drawn in its own material.
#[derive(Default)]
pub struct Unit {
    /// Frame, sash and muntins of a window; the leaf of a door.
    pub unit: Skin,
    /// Casing, sill and a door's frame.
    pub trim: Skin,
    pub glass: Skin,
    /// A door's knobs: where each is, in the wall's own directions.
    pub knobs: Vec<Vec3>,
}

/// One sash: stiles, rails, its glass, and the bars across it when its lites are divided.
#[allow(clippy::too_many_arguments)]
fn sash(u: &mut Unit, f: &Part, x: [f32; 2], y: [f32; 2], zc: f32, stile: f32, bottom: f32, divided: Option<(u32, u32)>) {
    let deep = [zc - 0.75 * IN, zc + 0.75 * IN];
    u.unit.block(f, [x[0], x[0] + stile], y, deep);
    u.unit.block(f, [x[1] - stile, x[1]], y, deep);
    u.unit.block(f, [x[0] + stile, x[1] - stile], [y[0], y[0] + bottom], deep);
    u.unit.block(f, [x[0] + stile, x[1] - stile], [y[1] - stile, y[1]], deep);
    let (gx, gy) = ([x[0] + stile, x[1] - stile], [y[0] + bottom, y[1] - stile]);
    u.glass.block(f, gx, gy, [zc - 0.2 * IN, zc + 0.2 * IN]);
    if let Some((across, up)) = divided {
        let (gw, gh) = (gx[1] - gx[0], gy[1] - gy[0]);
        let bar = 0.44 * IN;
        let bars = [zc - 0.5 * IN, zc + 0.5 * IN];
        for i in 1..across {
            let at = gx[0] + gw * i as f32 / across as f32;
            u.unit.block(f, [at - bar, at + bar], gy, bars);
        }
        for j in 1..up {
            let at = gy[0] + gh * j as f32 / up as f32;
            u.unit.block(f, gx, [at - bar, at + bar], bars);
        }
    }
}

/// The casing round an opening on one face of its wall. `out` is 1 on the face toward
/// `f.z` and -1 on the other; `face` is how far that face is from the wall's middle.
#[allow(clippy::too_many_arguments)]
fn casing(u: &mut Unit, f: &Part, w: f32, h: f32, face: f32, out: f32, outside: bool, masonry: bool, sill: bool) {
    let z = |a: f32, b: f32| if out > 0.0 { [face + a, face + b] } else { [-face - b, -face - a] };
    if outside && masonry {
        // In masonry: a brickmould, and a sill that sheds water.
        let c = 2.0 * IN;
        u.trim.block(f, [-c, 0.0], [0.0, h], z(-0.5 * IN, 1.1 * IN));
        u.trim.block(f, [w, w + c], [0.0, h], z(-0.5 * IN, 1.1 * IN));
        u.trim.block(f, [-c, w + c], [h, h + c], z(-0.5 * IN, 1.1 * IN));
        if sill {
            u.trim.block(f, [-c - 1.0 * IN, w + c + 1.0 * IN], [-2.5 * IN, 0.0], z(-0.5 * IN, 2.0 * IN));
        }
    } else if outside {
        // In siding: flat casing, a head casing with a drip cap over it, and a sill.
        let c = 3.5 * IN;
        u.trim.block(f, [-c, 0.0], [0.0, h], z(-0.5 * IN, 1.0 * IN));
        u.trim.block(f, [w, w + c], [0.0, h], z(-0.5 * IN, 1.0 * IN));
        u.trim.block(f, [-c - 0.5 * IN, w + c + 0.5 * IN], [h, h + 4.5 * IN], z(-0.5 * IN, 1.25 * IN));
        u.trim.block(f, [-c - 1.0 * IN, w + c + 1.0 * IN], [h + 4.5 * IN, h + 5.25 * IN], z(-0.5 * IN, 2.0 * IN));
        if sill {
            u.trim.block(f, [-c - 1.0 * IN, w + c + 1.0 * IN], [-1.75 * IN, 0.0], z(-0.5 * IN, 2.0 * IN));
        }
    } else {
        // Inside: casing all round, and under a window a stool and an apron.
        let c = 2.5 * IN;
        u.trim.block(f, [-c, 0.0], [0.0, h], z(-0.25 * IN, 0.75 * IN));
        u.trim.block(f, [w, w + c], [0.0, h], z(-0.25 * IN, 0.75 * IN));
        u.trim.block(f, [-c, w + c], [h, h + c], z(-0.25 * IN, 0.75 * IN));
        if sill {
            u.trim.block(f, [-c - 0.75 * IN, w + c + 0.75 * IN], [-1.0 * IN, 0.0], z(-0.25 * IN, 1.75 * IN));
            u.trim.block(f, [-c, w + c], [-1.0 * IN - c, -1.0 * IN], z(-0.25 * IN, 0.75 * IN));
        }
    }
}

/// A window in its opening. `f` stands at the opening's lower corner in the middle of the
/// wall, across the opening, up it, and out through the wall (to the outside, when the
/// wall has one). `outside`: whether the face toward `f.z` is an outside face, and of
/// the other face the same.
pub fn window(f: &Part, w: f32, h: f32, t: f32, d: Option<&OpeningDress>, outside: [bool; 2], masonry: bool) -> Unit {
    let mut u = Unit::default();
    let frame = 1.25 * IN;
    let half = t / 2.0;
    u.unit.block(f, [0.0, frame], [0.0, h], [-half, half]);
    u.unit.block(f, [w - frame, w], [0.0, h], [-half, half]);
    u.unit.block(f, [frame, w - frame], [h - frame, h], [-half, half]);
    u.unit.block(f, [frame, w - frame], [0.0, frame], [-half, half]);
    // The sash sits toward the outside, 2 1/2 in back from the face (in the middle of a
    // wall too thin for that).
    let zc = if outside[0] { (half - 2.5 * IN).max(0.0) } else if outside[1] { -(half - 2.5 * IN).max(0.0) } else { 0.0 };
    let (x, y) = ([frame, w - frame], [frame, h - frame]);
    let stile = 2.0 * IN;
    let hung = matches!(d.and_then(|d| d.sash.as_ref()).map(|s| s.what), Some(Sash::DoubleHung));
    // The lites of a sash: so many across and up. A count a line gives ("6 over 1") is
    // laid out as near square as it goes; with none given a light is about 11 in wide
    // and 15 in tall, worked out once so that every sash of the window matches.
    let pair = matches!(d.and_then(|d| d.sash.as_ref()).map(|s| s.what), Some(Sash::Casement | Sash::Slider)) && w > 3.5;
    let sash_h = if hung { (y[1] - y[0]) / 2.0 } else { y[1] - y[0] } - 2.0 * stile;
    let sash_w = if pair { (x[1] - x[0]) / 2.0 } else { x[1] - x[0] } - 2.0 * stile;
    let by_size = (((sash_w / (11.0 * IN)).round() as u32).clamp(2, 8), ((sash_h / (15.0 * IN)).round() as u32).clamp(1, 8));
    let grid = |count: Option<u32>| -> Option<(u32, u32)> {
        match count {
            None => Some(by_size),
            Some(1) => None,
            Some(n) => {
                let across = match n {
                    2 | 4 => 2,
                    3 | 6 | 9 => 3,
                    8 | 12 | 16 => 4,
                    n => n,
                };
                Some((across, (n / across).max(1)))
            }
        }
    };
    let lites: Option<Lites> = d.and_then(|d| d.divided);
    let (divided, lower) = (lites.and_then(|l| grid(l.upper)), lites.and_then(|l| grid(l.lower)));
    match d.and_then(|d| d.sash.as_ref()).map(|s| s.what) {
        Some(Sash::DoubleHung) => {
            // Two sashes, the upper one outside the lower, meeting at the middle.
            let mid = (y[0] + y[1]) / 2.0;
            let step = if zc >= 0.0 { 0.8 * IN } else { -0.8 * IN };
            sash(&mut u, f, x, [mid - 0.6 * IN, y[1]], zc + step, stile, 1.6 * IN, divided);
            sash(&mut u, f, x, [y[0], mid + 0.6 * IN], zc - step, stile, 3.0 * IN, lower);
        }
        Some(Sash::Casement) if w > 3.5 => {
            // A pair, with a post between them.
            let mid = w / 2.0;
            u.unit.block(f, [mid - 1.0 * IN, mid + 1.0 * IN], y, [-half, half]);
            sash(&mut u, f, [x[0], mid - 1.0 * IN], y, zc, stile, 2.5 * IN, divided);
            sash(&mut u, f, [mid + 1.0 * IN, x[1]], y, zc, stile, 2.5 * IN, divided);
        }
        Some(Sash::Slider) => {
            let mid = w / 2.0;
            let step = if zc >= 0.0 { 0.8 * IN } else { -0.8 * IN };
            sash(&mut u, f, [x[0], mid + 0.6 * IN], y, zc + step, stile, 2.0 * IN, divided);
            sash(&mut u, f, [mid - 0.6 * IN, x[1]], y, zc - step, stile, 2.0 * IN, divided);
        }
        _ => sash(&mut u, f, x, y, zc, stile, 2.5 * IN, divided),
    }
    casing(&mut u, f, w, h, half, 1.0, outside[0], masonry, true);
    casing(&mut u, f, w, h, half, -1.0, outside[1], masonry, true);
    u
}

/// A door in its opening, stood as a window is.
pub fn door(f: &Part, w: f32, h: f32, t: f32, d: Option<&OpeningDress>, outside: [bool; 2], masonry: bool) -> Unit {
    let mut u = Unit::default();
    let frame = 1.25 * IN;
    let half = t / 2.0;
    u.trim.block(f, [0.0, frame], [0.0, h], [-half, half]);
    u.trim.block(f, [w - frame, w], [0.0, h], [-half, half]);
    u.trim.block(f, [frame, w - frame], [h - frame, h], [-half, half]);
    // The leaf is 1 3/4 in thick and hangs toward the outside of an outside wall.
    let thick = 0.875 * IN;
    let zc = if outside[0] { (half - 2.5 * IN).max(0.0) } else if outside[1] { -(half - 2.5 * IN).max(0.0) } else { 0.0 };
    let (x, y) = ([frame + 0.125 * IN, w - frame - 0.125 * IN], [0.5 * IN, h - frame - 0.125 * IN]);
    let full = [zc - thick, zc + thick];
    let (lw, lh) = (x[1] - x[0], y[1] - y[0]);
    match d.and_then(|d| d.leaf.as_ref()).map(|l| l.what) {
        Some(Leaf::Panel) => {
            // Stiles and rails, with a raised panel in each opening between them: two
            // wide, a short pair at the top, a tall pair, and a pair below the lock rail.
            let (stile, rail, bottom) = (4.5 * IN, 4.5 * IN, 9.0 * IN);
            let back = [zc - thick + 0.45 * IN, zc + thick - 0.45 * IN];
            let field = [zc - thick + 0.15 * IN, zc + thick - 0.15 * IN];
            u.unit.block(f, x, y, back);
            u.unit.block(f, [x[0], x[0] + stile], y, full);
            u.unit.block(f, [x[1] - stile, x[1]], y, full);
            let mid = (x[0] + x[1]) / 2.0;
            // A narrow door has one panel across, a wide one two.
            let columns: Vec<[f32; 2]> = if lw > 2.2 { vec![[x[0] + stile, mid - stile / 2.0], [mid + stile / 2.0, x[1] - stile]] } else { vec![[x[0] + stile, x[1] - stile]] };
            if lw > 2.2 {
                u.unit.block(f, [mid - stile / 2.0, mid + stile / 2.0], y, full);
            }
            // The rails, from the floor up: bottom, lock (centred 36 in up), one under the top panels, top.
            let top_panel = (lh * 0.13).max(8.0 * IN);
            let rails = [[y[0], y[0] + bottom], [y[0] + 36.0 * IN - 3.0 * IN, y[0] + 36.0 * IN + 3.0 * IN], [y[1] - rail - top_panel - rail, y[1] - rail - top_panel], [y[1] - rail, y[1]]];
            for r in rails {
                u.unit.block(f, [x[0] + stile, x[1] - stile], r, full);
            }
            for c in &columns {
                for k in 0..3 {
                    let (lo, hi) = (rails[k][1], rails[k + 1][0]);
                    let inset = 1.1 * IN;
                    u.unit.block(f, [c[0] + inset, c[1] - inset], [lo + inset, hi - inset], field);
                }
            }
        }
        Some(Leaf::Glazed) => {
            let (stile, rail, bottom) = (4.5 * IN, 4.5 * IN, 9.0 * IN);
            u.unit.block(f, [x[0], x[0] + stile], y, full);
            u.unit.block(f, [x[1] - stile, x[1]], y, full);
            u.unit.block(f, [x[0] + stile, x[1] - stile], [y[0], y[0] + bottom], full);
            u.unit.block(f, [x[0] + stile, x[1] - stile], [y[1] - rail, y[1]], full);
            let (gx, gy) = ([x[0] + stile, x[1] - stile], [y[0] + bottom, y[1] - rail]);
            u.glass.block(f, gx, gy, [zc - 0.2 * IN, zc + 0.2 * IN]);
            if d.and_then(|d| d.divided).is_some() {
                let (across, up) = (if lw > 2.6 { 3 } else { 2 }, 5);
                let bar = 0.44 * IN;
                for i in 1..across {
                    let at = gx[0] + (gx[1] - gx[0]) * i as f32 / across as f32;
                    u.unit.block(f, [at - bar, at + bar], gy, [zc - 0.5 * IN, zc + 0.5 * IN]);
                }
                for j in 1..up {
                    let at = gy[0] + (gy[1] - gy[0]) * j as f32 / up as f32;
                    u.unit.block(f, gx, [at - bar, at + bar], [zc - 0.5 * IN, zc + 0.5 * IN]);
                }
            }
        }
        Some(Leaf::Overhead) => {
            // Sections about 21 in tall, each of raised panels about 2 ft wide.
            let rows = ((lh / (21.0 * IN)).round() as u32).max(2);
            let cols = ((lw / 2.0).round() as u32).max(1);
            u.unit.block(f, x, y, [zc - thick + 0.4 * IN, zc + thick - 0.4 * IN]);
            for r in 0..rows {
                for c in 0..cols {
                    let (px, py) = (x[0] + lw * c as f32 / cols as f32, y[0] + lh * r as f32 / rows as f32);
                    let inset = 2.2 * IN;
                    u.unit.block(f, [px + inset, px + lw / cols as f32 - inset], [py + inset, py + lh / rows as f32 - inset], full);
                }
            }
        }
        _ => u.unit.block(f, x, y, full),
    }
    if !matches!(d.and_then(|d| d.leaf.as_ref()).map(|l| l.what), Some(Leaf::Overhead)) {
        // A knob each side, 36 in up and 2 3/4 in in from the edge.
        for side in [-1.0, 1.0] {
            u.knobs.push(f.at(x[1] - 2.75 * IN, y[0] + 36.0 * IN - 0.5 * IN, zc + side * (thick + 1.4 * IN)));
        }
    }
    casing(&mut u, f, w, h, half, 1.0, outside[0], masonry, false);
    casing(&mut u, f, w, h, half, -1.0, outside[1], masonry, false);
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every pattern repeats without a break: its left edge runs on from its right and
    /// its top from its bottom, to within what one step across the picture changes it.
    #[test]
    fn every_pattern_is_made_and_has_every_smaller_size_of_itself() {
        for look in [Look::Lap, Look::Batten, Look::Shingle, Look::Brick, Look::Stone, Look::Stucco, Look::Asphalt, Look::Seam, Look::Slate, Look::Tile, Look::Shake, Look::Membrane, Look::Vertical, Look::Sheet, Look::Block] {
            let (colour, relief) = paint(look, look.tone());
            // 512 squared and each half of it down to one: (4^10 - 1) / 3 * 4 bytes... in all.
            let want: usize = (0..10).map(|k| (N >> k) * (N >> k) * 4).sum();
            assert_eq!(colour.data.as_ref().map(Vec::len), Some(want), "{look:?}");
            assert_eq!(relief.data.as_ref().map(Vec::len), Some(want), "{look:?}");
            assert_eq!(colour.texture_descriptor.mip_level_count, 10);
        }
    }

    /// A brick is 8 in by 2 2/3 in with its joint: 4 to the repeat across and 12 up.
    #[test]
    fn brick_is_laid_to_the_modular_size() {
        let courses = Vec::new();
        // Along the middle of a course, the head joints: one every 8 in.
        let y = (4.0 / 3.0 + 0.4) / 32.0;
        let joints = (0..N).filter(|i| at(Look::Brick, (*i as f32 + 0.5) / N as f32, y, &courses).0[0] < 0.0).count();
        // Four joints of 3/8 in in 32 in, at 16 dots to the inch: 6 dots each.
        assert_eq!(joints, 24);
        // Down a brick's middle, the bed joints: twelve of them.
        let x = 2.0 / 32.0;
        let mut beds = 0;
        let mut was = false;
        for j in 0..N {
            let is = at(Look::Brick, x, (j as f32 + 0.5) / N as f32, &courses).0[0] < 0.0;
            if is && !was {
                beds += 1;
            }
            was = is;
        }
        assert_eq!(beds, 12);
    }

    /// The parts of a window stay in its opening and its wall, but for the casing and sill.
    #[test]
    fn a_double_hung_window_has_two_sashes_and_its_bars() {
        let f = Part { o: Vec3::ZERO, x: Vec3::X, y: Vec3::Y, z: Vec3::Z };
        let d = OpeningDress { sash: Some(plumbline_world::finish::Said { what: Sash::DoubleHung, tone: None, by: String::new(), also: Vec::new() }), leaf: None, divided: Some(Lites { upper: Some(6), lower: Some(6) }), tone: None, flashed: false };
        let plain = window(&f, 3.0, 5.0, 0.5, None, [true, false], false);
        let hung = window(&f, 3.0, 5.0, 0.5, Some(&d), [true, false], false);
        // One light against two.
        assert_eq!(plain.glass.indices.len(), 36);
        assert_eq!(hung.glass.indices.len(), 72);
        // 3 across and 2 up in each sash: 2 bars and 1 bar, twice, more than the plain one has,
        // and one more sash of four pieces.
        assert_eq!(hung.unit.indices.len() - plain.unit.indices.len(), (4 + 2 * 3) * 36);
    }
}
