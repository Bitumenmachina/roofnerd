//! Made-up scans for the tests of `found` (DECISIONS.md call 179): a house of any size, thick
//! or thin, dense or sparse, as a scanner sees it, built from faces and boxes with a floor and
//! a ceiling. The same idea as `tools/made-up-scan.py --scene`, in Rust so that a test needs
//! no file: points scattered uniformly at so many to the square foot of each surface, each
//! a little out, one in fifty a stray, the whole scan leaning half a degree and then levelled
//! by its floor as the command line does it. Nothing here is from a real scan.

use crate::scan::Scan;

type P2 = [f64; 2];

pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }
    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn between(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.unit()
    }
    pub fn gauss(&mut self) -> f64 {
        let (u, v) = (self.unit().max(1e-12), self.unit());
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }
}

/// A ceiling over a rectangle: at `z0` at its south edge, rising to `z1` at its north edge.
struct Ceiling {
    rect: [f64; 4],
    z0: f64,
    z1: f64,
}

pub struct Scene {
    pub pts: Vec<[f64; 3]>,
    pub rng: Rng,
    pub dens: f64,
    pub noise: f64,
    floors: Vec<[f64; 4]>,
    ceilings: Vec<Ceiling>,
    /// Footprints that take the floor away: boxes, and (a, b, thickness) walls.
    boxes: Vec<[f64; 4]>,
    feet: Vec<(P2, P2, f64)>,
    /// A mirror on the wall at this east: the whole scan is again beyond it (call 183).
    pub mirror: Option<f64>,
}

/// A stretch left out of a face: from, to along it, low, high.
pub type Gap = [f64; 4];

impl Scene {
    /// `dens` points to the square foot of every surface; `noise` the scatter in feet (1.5 cm is 0.049).
    pub fn new(dens: f64, seed: u64) -> Scene {
        Scene { pts: Vec::new(), rng: Rng::new(seed), dens, noise: 0.015 / 0.3048, floors: Vec::new(), ceilings: Vec::new(), boxes: Vec::new(), feet: Vec::new(), mirror: None }
    }

    pub fn ceiling_at(&self, x: f64, y: f64) -> f64 {
        for c in &self.ceilings {
            let [x0, y0, x1, y1] = c.rect;
            if x >= x0 - 1e-9 && x <= x1 + 1e-9 && y >= y0 - 1e-9 && y <= y1 + 1e-9 {
                return c.z0 + (c.z1 - c.z0) * (y - y0) / (y1 - y0);
            }
        }
        self.ceilings.iter().map(|c| c.z0).fold(0.0, f64::max)
    }

    /// A floor, and (when `high` is given) a ceiling at it, over a rectangle.
    pub fn floor(&mut self, r: [f64; 4]) {
        self.floors.push(r);
    }

    pub fn ceiling(&mut self, r: [f64; 4], z0: f64, z1: f64) {
        self.ceilings.push(Ceiling { rect: r, z0, z1 });
    }

    /// The four outside walls, one face each, seen from indoors, with a floor and a ceiling `h` high.
    pub fn room(&mut self, w: f64, d: f64, h: f64) {
        self.room_hiding(w, d, h, &[]);
    }

    /// The same, with stretches of the walls not seen below a height: (side, from, to, height), the side
    /// 0 south, 1 east, 2 north, 3 west, from and to along it the way the walls are drawn (south and east
    /// toward the north and east, north toward the west, west toward the south).
    pub fn room_hiding(&mut self, w: f64, d: f64, h: f64, hidden: &[(usize, [f64; 3])]) {
        self.floor([0.0, 0.0, w, d]);
        self.ceiling([0.0, 0.0, w, d], h, h);
        for (i, (a, b)) in [([0.0, 0.0], [w, 0.0]), ([w, 0.0], [w, d]), ([w, d], [0.0, d]), ([0.0, d], [0.0, 0.0])].into_iter().enumerate() {
            let hide: Vec<[f64; 3]> = hidden.iter().filter(|(side, _)| *side == i).map(|(_, x)| *x).collect();
            self.face(a, b, None, &[], &hide);
        }
    }

    /// A vertical face from `a` to `b`, to `top` or (None) to the ceiling over it; `gaps` are left
    /// empty (an opening) and `hide` stretches are not seen below the height given (something stands before them).
    pub fn face(&mut self, a: P2, b: P2, top: Option<f64>, gaps: &[Gap], hide: &[[f64; 3]]) {
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let tall = top.unwrap_or_else(|| self.ceiling_at(a[0], a[1]).max(self.ceiling_at(b[0], b[1])));
        let n = (len * tall * self.dens).round() as usize;
        for _ in 0..n {
            let (s, z) = (self.rng.between(0.0, len), self.rng.between(0.0, tall));
            let (x, y) = (a[0] + (b[0] - a[0]) * s / len, a[1] + (b[1] - a[1]) * s / len);
            if top.is_none() && z > self.ceiling_at(x, y) {
                continue;
            }
            if gaps.iter().any(|g| s >= g[0] && s <= g[1] && z >= g[2] && z <= g[3]) || hide.iter().any(|h| s >= h[0] && s <= h[1] && z <= h[2]) {
                continue;
            }
            self.pts.push([x, y, z]);
        }
    }

    /// A face as a phone's scan has it (call 188): `bow` deep in the middle against its ends, `lean` (a rise over a foot of height) off plumb,
    /// `extra` more scatter across it than the scan's own (feet), the whole face `shift` out, at `dens` to the square foot of it. The measures a
    /// phone's scan has are in `tools/made-up-scan.py` (scene phone), from three real scans.
    #[allow(clippy::too_many_arguments)]
    pub fn face_rough(&mut self, a: P2, b: P2, top: Option<f64>, bow: f64, lean: f64, extra: f64, shift: f64, dens: f64) {
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let tall = top.unwrap_or_else(|| self.ceiling_at(a[0], a[1]).max(self.ceiling_at(b[0], b[1])));
        let (nx, ny) = (-(b[1] - a[1]) / len, (b[0] - a[0]) / len);
        for _ in 0..(len * tall * dens).round() as usize {
            let (s, z) = (self.rng.between(0.0, len), self.rng.between(0.0, tall));
            let u = 2.0 * s / len - 1.0;
            let off = shift + lean * z + bow * (1.0 - u * u) + extra * self.rng.gauss();
            self.pts.push([a[0] + (b[0] - a[0]) * s / len + nx * off, a[1] + (b[1] - a[1]) * s / len + ny * off, z]);
        }
    }

    /// A face that hangs from the ceiling: from `low` up to the ceiling over it (a beam, a soffit).
    pub fn hung(&mut self, a: P2, b: P2, low: f64) {
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let tall = self.ceiling_at(a[0], a[1]).max(self.ceiling_at(b[0], b[1]));
        for _ in 0..(len * (tall - low) * self.dens).round() as usize {
            let (s, z) = (self.rng.between(0.0, len), self.rng.between(low, tall));
            self.pts.push([a[0] + (b[0] - a[0]) * s / len, a[1] + (b[1] - a[1]) * s / len, z]);
        }
    }

    /// A horizontal surface over a rectangle at height `z` (a second storey's floor): points at the density, to the square foot.
    pub fn plate(&mut self, r: [f64; 4], z: f64) {
        let n = ((r[2] - r[0]) * (r[3] - r[1]) * self.dens).round() as usize;
        for _ in 0..n {
            let p = [self.rng.between(r[0], r[2]), self.rng.between(r[1], r[3]), z];
            self.pts.push(p);
        }
    }

    /// The same at its own density to the square foot (a part of the house scanned thinner than the rest).
    pub fn plate_dens(&mut self, r: [f64; 4], z: f64, dens: f64) {
        let was = std::mem::replace(&mut self.dens, dens);
        self.plate(r, z);
        self.dens = was;
    }

    /// A curved face: an arc of radius `r` about `c` from angle `a0` to `a1` (degrees), to `top`.
    pub fn arc(&mut self, c: P2, r: f64, a0: f64, a1: f64, top: f64) {
        let n = (r * (a1 - a0).to_radians() * top * self.dens).round() as usize;
        for _ in 0..n {
            let (a, z) = (self.rng.between(a0, a1).to_radians(), self.rng.between(0.0, top));
            self.pts.push([c[0] + r * a.cos(), c[1] + r * a.sin(), z]);
        }
    }

    /// A wall `t` feet thick on the centre line `a` to `b`: both its faces, and no floor between them.
    pub fn thick(&mut self, a: P2, b: P2, t: f64, top: Option<f64>, gaps: &[Gap], hide: &[[f64; 3]]) {
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let (nx, ny) = (-(b[1] - a[1]) / len, (b[0] - a[0]) / len);
        self.feet.push((a, b, t));
        for o in [t / 2.0, -t / 2.0] {
            self.face([a[0] + nx * o, a[1] + ny * o], [b[0] + nx * o, b[1] + ny * o], top, gaps, hide);
        }
    }

    /// A box: its top and its four sides (south, east, north, west), those named in `unseen` left out.
    pub fn bx(&mut self, r: [f64; 4], top: f64, unseen: &[&str]) {
        let [x0, y0, x1, y1] = r;
        self.boxes.push(r);
        let n = ((x1 - x0) * (y1 - y0) * self.dens).round() as usize;
        for _ in 0..n {
            let p = [self.rng.between(x0, x1), self.rng.between(y0, y1), top];
            self.pts.push(p);
        }
        for (side, a, b) in [("south", [x0, y0], [x1, y0]), ("east", [x1, y0], [x1, y1]), ("north", [x0, y1], [x1, y1]), ("west", [x0, y0], [x0, y1])] {
            if unseen.contains(&side) {
                continue;
            }
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            for _ in 0..(len * top * self.dens).round() as usize {
                let (s, z) = (self.rng.unit(), self.rng.between(0.0, top));
                self.pts.push([a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, z]);
            }
        }
    }

    /// The scan as it comes in: floors and ceilings put in, scatter, strays, the house turned `turn`
    /// degrees about its southwest corner, half a degree of lean, and levelled by its floor.
    pub fn scan(mut self, turn: f64) -> Scan {
        let floors = std::mem::take(&mut self.floors);
        for r in floors {
            let n = ((r[2] - r[0]) * (r[3] - r[1]) * self.dens).round() as usize;
            for _ in 0..n {
                let (x, y) = (self.rng.between(r[0], r[2]), self.rng.between(r[1], r[3]));
                let under_box = self.boxes.iter().any(|b| x >= b[0] && x <= b[2] && y >= b[1] && y <= b[3]);
                let under_wall = self.feet.iter().any(|(a, b, t)| {
                    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
                    let (s, c) = (((x - a[0]) * (b[0] - a[0]) + (y - a[1]) * (b[1] - a[1])) / len, (-(x - a[0]) * (b[1] - a[1]) + (y - a[1]) * (b[0] - a[0])) / len);
                    s >= 0.0 && s <= len && c.abs() <= t / 2.0 + 0.05
                });
                if !under_box && !under_wall {
                    self.pts.push([x, y, 0.0]);
                }
            }
        }
        let ceilings = std::mem::take(&mut self.ceilings);
        for c in &ceilings {
            let n = ((c.rect[2] - c.rect[0]) * (c.rect[3] - c.rect[1]) * self.dens).round() as usize;
            for _ in 0..n {
                let (x, y) = (self.rng.between(c.rect[0], c.rect[2]), self.rng.between(c.rect[1], c.rect[3]));
                self.pts.push([x, y, c.z0 + (c.z1 - c.z0) * (y - c.rect[1]) / (c.rect[3] - c.rect[1])]);
            }
        }
        let noise = self.noise;
        let mut pts: Vec<[f64; 3]> = std::mem::take(&mut self.pts);
        if let Some(m) = self.mirror {
            let again: Vec<[f64; 3]> = pts.iter().map(|p| [2.0 * m - p[0], p[1], p[2]]).collect();
            pts.extend(again);
        }
        for p in pts.iter_mut() {
            for c in p.iter_mut() {
                *c += noise * self.rng.gauss();
            }
        }
        let (lo, hi) = pts.iter().fold(([f64::MAX; 3], [f64::MIN; 3]), |(l, h), p| ([l[0].min(p[0]), l[1].min(p[1]), l[2].min(p[2])], [h[0].max(p[0]), h[1].max(p[1]), h[2].max(p[2])]));
        for _ in 0..pts.len() / 50 {
            let p = [self.rng.between(lo[0] - 6.0, hi[0] + 6.0), self.rng.between(lo[1] - 6.0, hi[1] + 6.0), self.rng.between(-1.0, hi[2] + 2.0)];
            pts.push(p);
        }
        let (t, lean) = (turn.to_radians(), 0.5f64.to_radians());
        let points = pts
            .into_iter()
            .map(|p| {
                let (x, y) = (p[0] * t.cos() - p[1] * t.sin(), p[0] * t.sin() + p[1] * t.cos());
                [x * lean.cos() - p[2] * lean.sin(), y, x * lean.sin() + p[2] * lean.cos()]
            })
            .collect();
        let mut scan = Scan { points, lost: Vec::new() };
        scan.level_by_floor();
        scan
    }
}
