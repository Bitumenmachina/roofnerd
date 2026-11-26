//! The house as its solids (call 229): one part for each thing the BIM file's writer writes a body for, as triangles in the
//! job's own axes and feet, for a writer of a mesh format to take. **This is the one source of solids**: the parts are worked
//! from what the BIM file's writer works its shapes from (`building::placements` for each wall, floor, run and thing set down,
//! `roof::solid_planes` for the roof, `building::room_outlines` for each room), through the one corner rule
//! ([`corner_carry`], which the BIM file's writer calls too) and the one frame of a run ([`along`], likewise). Nothing about a
//! shape is worked a second way here: a wall is the rectangle on its line pushed up as high as it stands, cut for each opening
//! by the same cut the model's own pictures use (`geom::face_less_holes`); a floor and a room are their outline pushed up; a
//! run is its section pushed along its line; a thing set down is its box (as the BIM file writes it: the shape it is drawn as
//! in the window is not carried); a roof plane is the face the BIM file writes, a surface and no solid.
//!
//! What cannot be handed out is named, never approximated in silence (`Solids::not_carried`).

use crate::building::{self, Placement, Shape};
use crate::finish::{self, Cladding, Roofing};
use crate::{geom, roof, view, Job, Kind, Level, Node};
use std::collections::{BTreeMap, HashMap};

type P3 = [f64; 3];
type P2 = [f64; 2];

/// A body as triangles: places, and for each triangle three of them by number, counter-clockwise seen from outside.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub points: Vec<P3>,
    pub faces: Vec<[u32; 3]>,
}

impl Mesh {
    /// The box that holds it: lowest and highest of each axis.
    pub fn bounds(&self) -> Option<[P3; 2]> {
        let first = *self.points.first()?;
        let mut b = [first, first];
        for p in &self.points {
            for k in 0..3 {
                b[0][k] = b[0][k].min(p[k]);
                b[1][k] = b[1][k].max(p[k]);
            }
        }
        Some(b)
    }

    /// What it holds, in cubic feet, by the sum of the volumes under each triangle: true for a closed body whose triangles face out.
    pub fn volume(&self) -> f64 {
        let mut six = 0.0;
        for f in &self.faces {
            let (a, b, c) = (self.points[f[0] as usize], self.points[f[1] as usize], self.points[f[2] as usize]);
            six += a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
        six / 6.0
    }

    fn corner(&mut self, p: P3) -> u32 {
        self.points.push(p);
        (self.points.len() - 1) as u32
    }

    fn quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.faces.push([a, b, c]);
        self.faces.push([a, c, d]);
    }
}

/// What a part is made of, as the estimate names it: the reading of the priced lines that every picture of the house uses
/// (`finish.rs`). The writer chooses a plain colour for each.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Finish {
    /// Nothing priced on it names a finish.
    Bare,
    Cladding(Cladding),
    Roofing(Roofing),
    /// A wall or footing whose priced lines say it is concrete.
    Concrete,
}

/// What a part is as a body.
#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    /// A closed solid.
    Solid(Mesh),
    /// A face and no thickness: a roof plane, as the BIM file writes it.
    Surface(Mesh),
    /// A run drawn with no size: the line it is drawn on, and no body (the BIM file writes only that line).
    Line([P3; 2]),
}

impl Body {
    pub fn bounds(&self) -> Option<[P3; 2]> {
        match self {
            Body::Solid(m) | Body::Surface(m) => m.bounds(),
            Body::Line([a, b]) => Some([[a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])], [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]]),
        }
    }
}

/// One thing the BIM file writes a shape for.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub path: String,
    /// The name a builder knows it by: the one the clash list uses.
    pub name: String,
    pub kind: Kind,
    /// The level it stands on, by path (None: the building's own zero).
    pub level: Option<String>,
    /// The roof it is a plane of, by path.
    pub roof: Option<String>,
    pub body: Body,
    pub finish: Finish,
    /// A colour its lines name (a standing seam in charcoal), as the estimate says it.
    pub tone: Option<[u8; 3]>,
}

/// A level, lowest first.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelRow {
    pub path: String,
    pub name: String,
    pub elevation: f64,
}

/// A roof, and what it is called.
#[derive(Clone, Debug, PartialEq)]
pub struct RoofRow {
    pub path: String,
    pub name: String,
}

/// Every solid of the job, and what could not be handed out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solids {
    /// The job's title.
    pub title: String,
    pub levels: Vec<LevelRow>,
    pub roofs: Vec<RoofRow>,
    pub parts: Vec<Part>,
    /// What was not carried: what, how many, why. Said in a builder's words.
    pub not_carried: Vec<(String, usize, String)>,
}

/// Along a run: the unit way from `a` to `b`, a level way across it, and its length. For a riser, across is east.
pub fn along(a: P3, b: P3) -> Option<(P3, P3, f64)> {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if length < 1.0e-9 {
        return None;
    }
    let way = [d[0] / length, d[1] / length, d[2] / length];
    let flat = (way[0] * way[0] + way[1] * way[1]).sqrt();
    let across = if flat < 1.0e-9 { [1.0, 0.0, 0.0] } else { [-way[1] / flat, way[0] / flat, 0.0] };
    Some((way, across, length))
}

/// Where two walls, or two footings, end at one point square to each other (a plain corner), the one drawn first runs through to
/// the outside corner and the other stops at its face: together they are the L that is built, with no square left out at the
/// outside and none held twice at the inside. How much longer (or, below nought, shorter) each end of each is, by path: `[at its
/// first end, at its last]`. The BIM file's writer and the solids both take it from here (J20-6).
pub fn corner_carry(placed: &[Placement]) -> HashMap<String, [f64; 2]> {
    type Line<'a> = (usize, &'a Placement, [f64; 2], [f64; 2], f64);
    let lines: Vec<Line> = placed.iter().enumerate().filter_map(|(i, p)| match &p.shape { Shape::Wall { a, b, thickness, .. } => Some((i, p, *a, *b, *thickness)), _ => None }).collect();
    let same = |x: [f64; 2], y: [f64; 2]| (x[0] - y[0]).hypot(x[1] - y[1]) < 0.01;
    let mut out = HashMap::new();
    for (i, p, a, b, _) in &lines {
        let mut by = [0.0, 0.0];
        for (which, end) in [a, b].into_iter().enumerate() {
            let meeting: Vec<&Line> = lines.iter().filter(|o| o.0 != *i && o.1.kind == p.kind && o.1.storey == p.storey && (same(o.2, *end) || same(o.3, *end))).collect();
            let [other] = meeting.as_slice() else { continue };
            let (ux, uy) = (b[0] - a[0], b[1] - a[1]);
            let (vx, vy) = (other.3[0] - other.2[0], other.3[1] - other.2[1]);
            let turn = ((ux * vy - uy * vx) / (ux.hypot(uy) * vx.hypot(vy)).max(1.0e-12)).abs();
            if turn > 0.98 {
                by[which] = if *i < other.0 { other.4 / 2.0 } else { -other.4 / 2.0 };
            }
        }
        out.insert(p.path.clone(), by);
    }
    out
}

/// A level's floor-to-floor height, where one is entered and above nothing.
pub fn level_height(job: &mut Job, storey: &str) -> Option<f64> {
    job.find(storey).and_then(|e| job.world.get::<Level>(e)).and_then(|l| l.height.feet()).filter(|h| *h > 0.0)
}

/// A place and three ways, right-handed: `x` along, `y` across (`z` crossed with `x`), `z` up from the place.
struct Frame {
    o: P3,
    x: P3,
    y: P3,
    z: P3,
}

impl Frame {
    fn new(o: P3, z: P3, x: P3) -> Frame {
        let y = [z[1] * x[2] - z[2] * x[1], z[2] * x[0] - z[0] * x[2], z[0] * x[1] - z[1] * x[0]];
        Frame { o, x, y, z }
    }
    fn at(&self, u: f64, v: f64, w: f64) -> P3 {
        [0, 1, 2].map(|k| self.o[k] + self.x[k] * u + self.y[k] * v + self.z[k] * w)
    }
}

/// The outline `ring` (in the frame's x and y) pushed from `w0` to `w1` up the frame's z: a closed solid. None where the outline
/// cannot be cut into triangles cleanly.
fn prism(f: &Frame, ring: &[P2], w0: f64, w1: f64) -> Option<Mesh> {
    let n = ring.len();
    if n < 3 || w1 <= w0 {
        return None;
    }
    let (tris, clean) = geom::triangles(ring);
    if !clean {
        return None;
    }
    // Counter-clockwise, so the caps and the sides face out.
    let area: f64 = (0..n).map(|i| ring[i][0] * ring[(i + 1) % n][1] - ring[(i + 1) % n][0] * ring[i][1]).sum();
    let order: Vec<usize> = if area >= 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
    let mut m = Mesh::default();
    let bottom: Vec<u32> = ring.iter().map(|p| m.corner(f.at(p[0], p[1], w0))).collect();
    let top: Vec<u32> = ring.iter().map(|p| m.corner(f.at(p[0], p[1], w1))).collect();
    for t in &tris {
        m.faces.push([top[t[0]], top[t[1]], top[t[2]]]);
        m.faces.push([bottom[t[0]], bottom[t[2]], bottom[t[1]]]);
    }
    for k in 0..n {
        let (i, j) = (order[k], order[(k + 1) % n]);
        m.quad(bottom[i], bottom[j], top[j], top[i]);
    }
    Some(m)
}

/// A rectangle `x0..x1` by `y0..y1` pushed from `w0` to `w1`.
fn block(f: &Frame, x0: f64, x1: f64, y0: f64, y1: f64, w0: f64, w1: f64) -> Option<Mesh> {
    prism(f, &[[x0, y0], [x1, y0], [x1, y1], [x0, y1]], w0, w1)
}

/// The sides of the round a run of a diameter has: a multiple of four, so the extremes on its own axes are vertices.
pub const ROUND_SIDES: usize = 24;

fn join(parts: Vec<Mesh>) -> Mesh {
    let mut out = Mesh::default();
    for m in parts {
        let at = out.points.len() as u32;
        out.points.extend(m.points);
        out.faces.extend(m.faces.into_iter().map(|f| f.map(|i| i + at)));
    }
    out
}

/// Every solid of the job. The BIM file's writer writes a body for each thing in the list that is not a `not_carried` row.
pub fn solids(job: &mut Job) -> Result<Solids, String> {
    let title = job.world.get::<crate::JobInfo>(job.root()).map(|j| j.title.clone()).unwrap_or_default();
    let elements = view::elements(job);
    let names: HashMap<String, String> = elements.iter().map(|e| (e.path.clone(), e.name.clone())).collect();
    let name_of = |path: &str, fallback: &str| names.get(path).cloned().unwrap_or_else(|| fallback.to_string());
    let storeys = building::storey_totals(job);
    if storeys.is_empty() {
        return Err("the job has no level: there is no building to write a model of".into());
    }
    let (placed, not_drawn) = building::placements(job);
    // A thing drawn goes by the name the clash list gives it (one rule: `clash::called`); a door, a window, a room and a roof plane by
    // the name the model gives it, as the BIM file does.
    let called = building::clash::called(job, &placed);
    let outlines = building::room_outlines(job);
    let (planes, planes_out) = roof::solid_planes(job);
    let dress = finish::dress(job);
    let mut out = Solids { title, ..Solids::default() };
    let mut not_carried: Vec<(String, usize, String)> = Vec::new();

    let mut ordered: Vec<&building::StoreyTotals> = storeys.iter().collect();
    ordered.sort_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0)));
    let mut elevation_of: HashMap<String, f64> = HashMap::new();
    let mut no_elevation = 0usize;
    for s in ordered {
        match s.elevation {
            Some(elevation) => {
                elevation_of.insert(s.path.clone(), elevation);
                out.levels.push(LevelRow { path: s.path.clone(), name: s.name.clone(), elevation });
            }
            None => no_elevation += 1,
        }
    }
    if no_elevation > 0 {
        not_carried.push(("level".into(), no_elevation, "its floor's elevation is not known, so nothing on it can be placed".into()));
    }
    if out.levels.is_empty() {
        return Err("no level of the job has an elevation: nothing can be placed".into());
    }

    let carried = corner_carry(&placed);
    let up = [0.0, 0.0, 1.0];
    let tone_of = |t: &Option<finish::Tone>| t.as_ref().map(|t| t.rgb);
    for p in &placed {
        let level = p.storey.clone().filter(|s| elevation_of.contains_key(s));
        let mut push = |path: &str, name: &str, kind: Kind, body: Body, finish: Finish, tone: Option<[u8; 3]>| {
            out.parts.push(Part { path: path.to_string(), name: if kind.is_opening() { name.to_string() } else { called.get(path).cloned().unwrap_or_else(|| name.to_string()) }, kind, level: level.clone(), roof: None, body, finish, tone });
        };
        match &p.shape {
            Shape::Wall { a, b, base, height, thickness, holes } => {
                let Some((way, _, length)) = along([a[0], a[1], 0.0], [b[0], b[1], 0.0]) else {
                    not_carried.push((format!("wall {}", p.name), 1, "it has no length".into()));
                    continue;
                };
                let frame = Frame::new([a[0], a[1], *base], up, way);
                let [before, after] = carried.get(&p.path).copied().unwrap_or([0.0, 0.0]);
                // The body runs from so far before its first end to so far past its last; the openings are cut out of that.
                let whole = length + before + after;
                let cuts: Vec<[f64; 4]> = holes.iter().filter(|h| h.2[1] > h.2[0] && h.2[3] > h.2[2]).map(|h| [h.2[0] + before, h.2[1] + before, h.2[2], h.2[3]]).collect();
                let pieces: Vec<Mesh> = geom::face_less_holes(whole, *height, &cuts).iter().filter_map(|[x0, x1, z0, z1]| block(&frame, x0 - before, x1 - before, -thickness / 2.0, thickness / 2.0, *z0, *z1)).collect();
                let (finish, tone) = if p.kind == Kind::Footing {
                    (Finish::Concrete, None)
                } else {
                    match dress.walls.get(&p.path) {
                        Some(w) => match &w.cladding {
                            Some(c) => (Finish::Cladding(c.what), tone_of(&c.tone)),
                            None if w.concrete => (Finish::Concrete, None),
                            None => (Finish::Bare, None),
                        },
                        None => (Finish::Bare, None),
                    }
                };
                push(&p.path, &p.name, p.kind, Body::Solid(join(pieces)), finish, tone);
                // Each door and window: the space it takes up in the wall.
                for (path, kind, [from, to, sill, head]) in holes {
                    if to - from <= 0.0 || head - sill <= 0.0 {
                        continue;
                    }
                    let Some(m) = block(&frame, *from, *to, -thickness / 2.0, thickness / 2.0, *sill, *head) else { continue };
                    push(path, &name_of(path, ""), *kind, Body::Solid(m), Finish::Bare, None);
                }
            }
            Shape::Floor { boundary, top, thickness } => {
                let frame = Frame::new([0.0, 0.0, top - thickness], up, [1.0, 0.0, 0.0]);
                match prism(&frame, boundary, 0.0, *thickness) {
                    Some(m) => push(&p.path, &p.name, p.kind, Body::Solid(m), Finish::Bare, None),
                    None => not_carried.push((format!("floor {}", p.name), 1, "its outline cannot be cut into faces cleanly".into())),
                }
            }
            Shape::Run { a, b, round, rect } => {
                let Some((way, across, length)) = along(*a, *b) else {
                    not_carried.push((format!("{} {}", p.kind.stem(), p.name), 1, "it has no length".into()));
                    continue;
                };
                let frame = Frame::new(*a, way, across);
                let ring: Option<Vec<P2>> = match (round, rect) {
                    (Some(d), _) => Some((0..ROUND_SIDES).map(|i| { let a = std::f64::consts::TAU * i as f64 / ROUND_SIDES as f64; [d / 2.0 * a.cos(), d / 2.0 * a.sin()] }).collect()),
                    (None, Some((w, d))) => Some(vec![[-w / 2.0, -d / 2.0], [w / 2.0, -d / 2.0], [w / 2.0, d / 2.0], [-w / 2.0, d / 2.0]]),
                    (None, None) => None,
                };
                match ring {
                    Some(r) => match prism(&frame, &r, 0.0, length) {
                        Some(m) => push(&p.path, &p.name, p.kind, Body::Solid(m), Finish::Bare, None),
                        None => not_carried.push((format!("{} {}", p.kind.stem(), p.name), 1, "its section cannot be cut into faces".into())),
                    },
                    None if matches!(p.kind, Kind::Duct | Kind::Pipe | Kind::Conduit | Kind::Cable) => push(&p.path, &p.name, p.kind, Body::Line([*a, *b]), Finish::Bare, None),
                    None => not_carried.push((format!("{} {}", p.kind.stem(), p.name), 1, "its size is not known, so it has no shape to write".into())),
                }
            }
            Shape::Box { at, size, angle } => {
                let turn = angle.to_radians();
                let frame = Frame::new(*at, up, [turn.cos(), turn.sin(), 0.0]);
                let finish = if p.kind == Kind::PadFooting { Finish::Concrete } else { Finish::Bare };
                match block(&frame, -size[0] / 2.0, size[0] / 2.0, -size[1] / 2.0, size[1] / 2.0, 0.0, size[2]) {
                    Some(m) => push(&p.path, &p.name, p.kind, Body::Solid(m), finish, None),
                    None => not_carried.push((format!("{} {}", p.kind.stem(), p.name), 1, "its box has no size".into())),
                }
            }
        }
    }

    // Rooms: each as the outline its floor area is the area of, as high as its level is to the next.
    let (mut no_walls, mut no_height) = (0usize, 0usize);
    for el in elements.iter().filter(|e| e.kind == Kind::Room) {
        let Some(storey) = job.find(&el.path).and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone()) else { continue };
        let Some(&elevation) = elevation_of.get(&storey) else { continue };
        let Some(ring) = outlines.get(&el.path) else {
            no_walls += 1;
            continue;
        };
        let Some(high) = level_height(job, &storey) else {
            no_height += 1;
            continue;
        };
        let frame = Frame::new([0.0, 0.0, elevation], up, [1.0, 0.0, 0.0]);
        match prism(&frame, ring, 0.0, high) {
            Some(m) => out.parts.push(Part { path: el.path.clone(), name: el.name.clone(), kind: Kind::Room, level: Some(storey), roof: None, body: Body::Solid(m), finish: Finish::Bare, tone: None }),
            None => not_carried.push((format!("room {}", el.name), 1, "its outline cannot be cut into faces cleanly".into())),
        }
    }
    if no_walls > 0 {
        not_carried.push(("room".into(), no_walls, "no walls close round its point, so it has no outline".into()));
    }
    if no_height > 0 {
        not_carried.push(("room".into(), no_height, "its level has no floor-to-floor height, so it has no body".into()));
    }

    // The roof: each plane as it lies, under its roof.
    let mut roofs: BTreeMap<String, ()> = BTreeMap::new();
    for plane in &planes {
        if plane.points.len() < 3 {
            continue;
        }
        let corners: Vec<P3> = plane.points.iter().map(|q| [q[0], q[1], plane.height_at(*q)]).collect();
        let (tris, clean) = geom::triangles(&plane.points);
        if !clean {
            not_carried.push((format!("roof plane {}", name_of(&plane.path, "")), 1, "its outline cannot be cut into faces cleanly".into()));
            continue;
        }
        let mesh = Mesh { points: corners, faces: tris.iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect() };
        let roof = job.find(&plane.path).and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone()).unwrap_or_default();
        roofs.insert(roof.clone(), ());
        let kind = job.find(&plane.path).and_then(|e| job.world.get::<Kind>(e)).copied().unwrap_or(Kind::RoofPlane);
        let said = dress.roofs.get(&plane.path);
        out.parts.push(Part {
            path: plane.path.clone(),
            name: name_of(&plane.path, ""),
            kind,
            level: None,
            roof: Some(roof),
            body: Body::Surface(mesh),
            finish: said.map(|s| Finish::Roofing(s.what)).unwrap_or(Finish::Bare),
            tone: said.and_then(|s| tone_of(&s.tone)),
        });
    }
    for roof in roofs.keys() {
        out.roofs.push(RoofRow { path: roof.clone(), name: name_of(roof, "roof") });
    }
    if !planes_out.is_empty() {
        not_carried.push(("roof plane".into(), planes_out.len(), "it cannot be placed in space: a value its shape needs is not known".into()));
    }
    for (path, why) in &not_drawn {
        not_carried.push((names.get(path).cloned().unwrap_or_else(|| path.clone()), 1, format!("it has no shape: {why}")));
    }
    out.not_carried = not_carried;
    Ok(out)
}
