//! Clash detection: which solid things in the job are in the same space, and which stand
//! closer to each other than a distance asked for.
//!
//! Ours. BIMCraft has none: `grep -rni clash crates/core/src` finds two comments and, in
//! `import.rs`, a variable for parameter names that collide; nothing there tests one
//! solid against another. The words are the trade's: a **hard clash** is two things in the same space; a
//! **clearance clash** (a soft clash) is two things closer than a clearance that was asked
//! for. The solid geometry is not ours: it is `parry3d-f64` (Apache-2.0), which says for
//! two convex solids how far apart they are, or how deep one is in the other.
//!
//! What is tested is what `placements` gives a shape, and nothing else: the window draws
//! from the same list, so what clashes is what is seen. A wall is its solid pieces round
//! its doors and windows (a door or a window is a hole: a pipe through it is through
//! nothing). A floor is its outline cut into triangles, each a slab. A run is a cylinder,
//! or a box when it is rectangular. Equipment, a fixture and a fitting are boxes.
//!
//! What is not a clash, and is never tested:
//! * walls and floors against each other, wall against wall and floor against floor:
//!   they are built into one another;
//! * the joint itself where two things are joined (the job's network says where): two
//!   runs that meet there, a run and its fitting, a run and the thing its end is
//!   connected to. Only the joint is excused. Each run is left out for a little way
//!   either side of it (three times the half-width of the larger run; into a fixture,
//!   equipment or a fitting, as deep as the joint is in it) and what is left of the two
//!   is tested like anything else: a run laid over another, folded back along it, or
//!   carried on through the tank it is joined to is a clash. A stub shorter than that
//!   little way is not seen;
//! * anything with no shape, each listed with why. A run with no size is a line, not a
//!   solid. The roof's planes are not in `placements` and are not tested.
//!
//! A run, or a fitting on one, that meets a wall or a floor is reported apart, as passing
//! through it: it is a hole to cut and sleeve, which an estimator counts, not two trades
//! in one place. This model's walls are solid, so a pipe drawn inside a wall is through it.
//!
//! Two things are in the same space when they are in each other by more than a thing can
//! be placed to, the fit tolerance (a hundredth of a foot): each solid is drawn in by
//! half of that, square to its faces, and what is left is tested for overlap. Touching is
//! not a clash. That test is the library's yes-or-no one, which does not fail. Its
//! measure of how deep two solids are in each other can: it answers nothing, or zero,
//! when one solid is dead centre in another (a pipe through the middle of a duct, drawn
//! at one height, which is the commonest clash there is). So the depth is asked for more
//! than once, a billionth of a foot off centre, and a clash whose depth still cannot be
//! had is reported as a clash with no depth. It is never dropped. Two round runs are
//! not asked of the library at all where plain geometry answers: side by side, one in
//! the other, or crossing well inside both lengths, they are in each other by the sum
//! of their radii less the distance between their centre lines.
//!
//! A wall or a floor is several solids. The way out of one of them can be into the
//! next, which is no way out: so against a wall or a floor a thing's depth is the least
//! of the ways out that clear every piece, among them straight out through the wall's
//! face or the floor's top or bottom.
//!
//! A clash is as sure as the two shapes: Assumed when either is. A shape is no surer
//! than the floor height of its storey, and a wall with an opening that could not be
//! cut is not sure.

use std::collections::{BTreeMap, BTreeSet};

use parry3d_f64::bounding_volume::{Aabb, BoundingVolume};
use parry3d_f64::math::{Pose, Rotation, Vector};
use parry3d_f64::query;
use parry3d_f64::shape::{ConvexPolyhedron, Cuboid, Cylinder, Shape as Solid};

use super::{network, placements, Placed, Placement, Shape, P3};
use crate::check::FIT_TOLERANCE_FT;
use crate::components::{Kind, Node};
use crate::geom;
use crate::job::Job;
use crate::value::Status;

/// The widest clearance that can be asked for, in feet. Past this the answer is every
/// pair of things in a room.
pub const MAX_CLEARANCE_FT: f64 = 10.0;

/// What is never tested against what, for a report to say.
pub const NEVER_TESTED: [&str; 4] = [
    "walls and floors are not tested against each other, nor wall against wall, nor floor against floor",
    "a joint is not a clash: where runs meet, where a run meets its fitting or the thing it is connected to, each run is left out for a little way either side",
    "a door or a window is a hole in its wall: nothing is tested against what fills it",
    "the roof's planes are not tested",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClashKind {
    /// Two things in the same space.
    Hard,
    /// Two things closer than the clearance asked for.
    Clearance,
}

/// Two elements that clash.
#[derive(Clone, Debug, PartialEq)]
pub struct Clash {
    /// The two elements, by path, in the order the job holds them.
    pub a: String,
    pub b: String,
    pub a_name: String,
    pub b_name: String,
    pub a_kind: Kind,
    pub b_kind: Kind,
    pub how: ClashKind,
    /// A run meeting a wall or a floor: a hole to cut, listed apart.
    pub through: bool,
    /// Hard: how far one would have to move to clear the other, in feet; none for a run
    /// through a wall or a floor (it is meant to go through), and none where the depth
    /// could not be worked out. Clearance: how far apart they are, in feet.
    pub by: Option<f64>,
    /// Where, in feet: east, north, height above the datum.
    pub at: P3,
    pub status: Status,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clashes {
    /// Clashes between things, then runs through walls and floors, then clearances.
    pub found: Vec<Clash>,
    /// How many elements had a solid and were tested.
    pub tested: usize,
    /// What was not tested, each with why.
    pub left_out: Vec<(String, String)>,
}

/// One convex solid of one element, placed. `core` is the same solid drawn in by half the
/// fit tolerance, where it is thick enough to be.
struct Piece {
    owner: usize,
    pose: Pose,
    solid: Box<dyn Solid>,
    core: Option<Box<dyn Solid>>,
    aabb: Aabb,
    /// A round run: the two ends of its centre line and its radius.
    round: Option<(Vector, Vector, f64)>,
    /// A piece of a wall or a floor: the way it is thin, and how far the whole wall or
    /// floor reaches that way, from and to.
    thin: Option<(Vector, f64, f64)>,
}

/// How far each solid is drawn in before two are tested for being in the same space.
const DRAWN_IN: f64 = FIT_TOLERANCE_FT / 2.0;

impl Piece {
    fn new(owner: usize, pose: Pose, solid: Box<dyn Solid>, core: Option<Box<dyn Solid>>) -> Piece {
        let aabb = solid.compute_aabb(&pose);
        Piece { owner, pose, solid, core, aabb, round: None, thin: None }
    }
    fn cuboid(owner: usize, pose: Pose, half: Vector) -> Piece {
        let core: Option<Box<dyn Solid>> = (half.min_element() > DRAWN_IN).then(|| Box::new(Cuboid::new(half - Vector::splat(DRAWN_IN))) as Box<dyn Solid>);
        Piece::new(owner, pose, Box::new(Cuboid::new(half)), core)
    }
    fn cylinder(owner: usize, pose: Pose, half_length: f64, radius: f64) -> Piece {
        let core: Option<Box<dyn Solid>> = (half_length.min(radius) > DRAWN_IN).then(|| Box::new(Cylinder::new(half_length - DRAWN_IN, radius - DRAWN_IN)) as Box<dyn Solid>);
        let mut piece = Piece::new(owner, pose, Box::new(Cylinder::new(half_length, radius)), core);
        let half = pose.rotation * Vector::Y * half_length;
        piece.round = Some((pose.translation - half, pose.translation + half, radius));
        piece
    }
    fn core(&self) -> &dyn Solid {
        self.core.as_deref().unwrap_or(self.solid.as_ref())
    }
}

/// The solids of one placed element, or why it has none.
fn pieces_of(p: &Placement, owner: usize) -> Result<Vec<Piece>, String> {
    let mut out = Vec::new();
    match &p.shape {
        Shape::Wall { a, b, base, height, thickness, holes } => {
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let length = dx.hypot(dy);
            if length <= 1.0e-9 {
                return Err("it has no length".into());
            }
            let turn = Rotation::from_rotation_z(dy.atan2(dx));
            let (ux, uy) = (dx / length, dy / length);
            // What is left of the wall round its doors and windows, each piece a solid.
            // All of them are thin the same way, square to the wall's face.
            let across = Vector::new(-uy, ux, 0.0);
            let middle = a[0] * across.x + a[1] * across.y;
            let cuts: Vec<[f64; 4]> = holes.iter().map(|h| h.2).collect();
            for [x0, x1, z0, z1] in geom::face_less_holes(length, *height, &cuts) {
                let (mx, mz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
                let pose = Pose::from_parts(Vector::new(a[0] + ux * mx, a[1] + uy * mx, base + mz), turn);
                let mut piece = Piece::cuboid(owner, pose, Vector::new((x1 - x0) / 2.0, thickness / 2.0, (z1 - z0) / 2.0));
                piece.thin = Some((across, middle - thickness / 2.0, middle + thickness / 2.0));
                out.push(piece);
            }
        }
        Shape::Floor { boundary, top, thickness } => {
            let (triangles, clean) = geom::triangles(boundary);
            if !clean || triangles.is_empty() {
                return Err("its outline could not be cut into triangles".into());
            }
            // A triangle of the outline, as a slab from `high` down to `low`.
            let slab = |t: &[usize; 3], high: f64, low: f64| {
                let corners: Vec<Vector> = t.iter().flat_map(|&i| [Vector::new(boundary[i][0], boundary[i][1], high), Vector::new(boundary[i][0], boundary[i][1], low)]).collect();
                ConvexPolyhedron::from_convex_hull(&corners)
            };
            for t in &triangles {
                let whole = slab(t, *top, top - thickness).ok_or("a piece of it is too thin to be a solid")?;
                // Drawn in at its top and its bottom only: its other faces are cuts between
                // its own triangles, or its edge, where it meets the walls.
                let core: Option<Box<dyn Solid>> = if *thickness > 2.0 * DRAWN_IN { slab(t, top - DRAWN_IN, top - thickness + DRAWN_IN).map(|c| Box::new(c) as Box<dyn Solid>) } else { None };
                let mut piece = Piece::new(owner, Pose::identity(), Box::new(whole), core);
                piece.thin = Some((Vector::Z, top - thickness, *top));
                out.push(piece);
            }
        }
        Shape::Run { a, b, round, rect } => {
            let d = Vector::new(b[0] - a[0], b[1] - a[1], b[2] - a[2]);
            let length = d.length();
            if length <= 1.0e-9 {
                return Err("it has no length".into());
            }
            let middle = Vector::new((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0);
            match (round, rect) {
                // A cylinder's own axis is its y.
                (Some(across), _) => out.push(Piece::cylinder(owner, Pose::from_parts(middle, Rotation::from_rotation_arc(Vector::Y, d / length)), length / 2.0, across / 2.0)),
                // A box along its x, its width level and its depth square to the run.
                (None, Some((width, depth))) => {
                    let yaw = if d.x.hypot(d.y) > 1.0e-9 { d.y.atan2(d.x) } else { 0.0 };
                    let pitch = (d.z / length).clamp(-1.0, 1.0).asin();
                    let turn = Rotation::from_rotation_z(yaw) * Rotation::from_rotation_y(-pitch);
                    out.push(Piece::cuboid(owner, Pose::from_parts(middle, turn), Vector::new(length / 2.0, width / 2.0, depth / 2.0)));
                }
                (None, None) => return Err("its size is not known: it is a line, not a solid".into()),
            }
        }
        Shape::Box { at, size, angle } => {
            let pose = Pose::from_parts(Vector::new(at[0], at[1], at[2] + size[2] / 2.0), Rotation::from_rotation_z(angle.to_radians()));
            out.push(Piece::cuboid(owner, pose, Vector::new(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0)));
        }
    }
    if out.is_empty() {
        return Err("nothing of it is solid".into());
    }
    Ok(out)
}

fn built_in(kind: Kind) -> bool {
    matches!(kind, Kind::Wall | Kind::Floor)
}

fn pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

/// What two elements' pieces came to.
struct Met {
    hard: bool,
    depth: Option<f64>,
    gap: f64,
    at: P3,
}

/// How deep two solids are in each other: how far the second would have to move to clear
/// the first, which way, and a place that is in both.
struct Deep {
    by: f64,
    way: Vector,
    at: Vector,
}

/// Where two lengths of line come nearest: how far along each, from 0 to 1.
fn nearest(a0: Vector, u: Vector, b0: Vector, v: Vector) -> (f64, f64) {
    let r = a0 - b0;
    let (a, e, f) = (u.dot(u), v.dot(v), v.dot(r));
    let c = u.dot(r);
    let b = u.dot(v);
    let denom = a * e - b * b;
    let mut s = if denom > 1.0e-12 { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
    let mut t = (b * s + f) / e;
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b - c) / a).clamp(0.0, 1.0);
    }
    (s, t)
}

/// Two round runs in each other, where plain geometry says by how much: side by side or
/// one inside the other, or crossing well inside both lengths. Otherwise none, and the
/// library is asked.
fn round_in_round((a0, a1, r1): (Vector, Vector, f64), (b0, b1, r2): (Vector, Vector, f64)) -> Option<Deep> {
    let (u, v) = (a1 - a0, b1 - b0);
    let (lu, lv) = (u.length(), v.length());
    if lu < 1.0e-9 || lv < 1.0e-9 {
        return None;
    }
    let (du, dv) = (u / lu, v / lv);
    let sine = du.cross(dv).length();
    let (s, t) = nearest(a0, u, b0, v);
    let (p, q) = (a0 + u * s, b0 + v * t);
    let between = q - p;
    let apart = between.length();
    let side = r1 + r2 - apart;
    if side <= 0.0 {
        return None;
    }
    // A place in both: on the line between the two centre lines, in proportion.
    let at = p + between * (r1 / (r1 + r2));
    let sideways = if apart > 1.0e-9 {
        between / apart
    } else if sine > 1.0e-6 {
        du.cross(dv).normalize()
    } else {
        du.any_orthonormal_vector()
    };
    if sine < 1.0e-6 {
        // Along one another: out sideways, or out endways where that is less.
        let (x0, x1) = ((b0 - a0).dot(du), (b1 - a0).dot(du));
        let over = lu.min(x0.max(x1)) - x0.min(x1).max(0.0);
        if over <= 0.0 {
            return None;
        }
        // Sideways only if they are truly side by side there, not end to end.
        let level = ((b0 - a0) - du * (b0 - a0).dot(du)).length();
        let side = r1 + r2 - level;
        if side <= 0.0 {
            return None;
        }
        let at = a0 + du * ((lu.min(x0.max(x1)) + x0.min(x1).max(0.0)) / 2.0) + ((b0 - a0) - du * (b0 - a0).dot(du)) * (r1 / (r1 + r2));
        return Some(if over < side { Deep { by: over, way: if (x0 + x1) / 2.0 >= lu / 2.0 { du } else { -du }, at } } else { Deep { by: side, way: sideways, at } });
    }
    // Crossing: exact when each nearest place is further from its run's ends than the
    // other run is wide there.
    let room = (r1 + r2) / sine;
    let inside = |x: f64, len: f64| x * len >= room && (1.0 - x) * len >= room;
    (inside(s, lu) && inside(t, lv)).then_some(Deep { by: side, way: sideways, at })
}

/// How far one of two solids that are in each other would have to move to clear the
/// other, in feet; none when it cannot be said. The library is asked with the second
/// solid a billionth of a foot off its place, three ways, because dead centre it answers
/// zero.
fn depth(first: &Piece, second: &Piece) -> Option<Deep> {
    if let (Some(a), Some(b)) = (first.round, second.round) {
        if let Some(deep) = round_in_round(a, b) {
            return (deep.by > FIT_TOLERANCE_FT).then_some(deep);
        }
    }
    const OFF: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [0.618, 0.382, 0.789], [-0.3, 0.9, 0.1]];
    let mut deepest: Option<Deep> = None;
    for off in OFF {
        let pose = Pose::from_parts(second.pose.translation + Vector::new(off[0], off[1], off[2]) * 1.0e-9, second.pose.rotation);
        if let Ok(Some(c)) = query::contact(&first.pose, first.solid.as_ref(), &pose, second.solid.as_ref(), 0.0) {
            if c.dist < -FIT_TOLERANCE_FT && deepest.as_ref().is_none_or(|d| -c.dist > d.by) {
                deepest = Some(Deep { by: -c.dist, way: c.normal1, at: (c.point1 + c.point2) / 2.0 });
            }
        }
    }
    deepest
}

/// Whether two pieces are in the same space: what is left of each, drawn in by half the
/// fit tolerance, overlaps. None when the library could not say.
fn in_each_other(first: &Piece, at: &Pose, second: &Piece, other: &Pose) -> Option<bool> {
    query::intersection_test(at, first.core(), other, second.core()).ok().map(|hit| hit.intersecting)
}

/// How far a run reaches from its centre line: half its diameter, or half its diagonal.
fn half_across(shape: &Shape) -> f64 {
    match shape {
        Shape::Run { round: Some(across), .. } => across / 2.0,
        Shape::Run { rect: Some((width, depth)), .. } => width.hypot(*depth) / 2.0,
        _ => 0.0,
    }
}

/// How deep a place is inside a box: how far to its nearest face, or nothing outside it.
fn deep_in_box(at: &P3, size: &P3, angle: f64, place: P3) -> f64 {
    let (sin, cos) = (-angle.to_radians()).sin_cos();
    let (dx, dy) = (place[0] - at[0], place[1] - at[1]);
    let (x, y, z) = (dx * cos - dy * sin, dx * sin + dy * cos, place[2] - at[2]);
    (size[0] / 2.0 - x.abs()).min(size[1] / 2.0 - y.abs()).min(z).min(size[2] - z).max(0.0)
}

/// The solids of an element that is joined to another, with each joint taken out of it:
/// a run is left out either side of each place it is joined at, as far as the joint
/// excuses (see the head of this file). Anything else is whole.
fn away_from_joints(p: &Placement, other: &Placement, places: &[P3], owner: usize) -> Vec<Piece> {
    let Shape::Run { a, b, round, rect } = &p.shape else { return pieces_of(p, owner).unwrap_or_default() };
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if length <= 1.0e-9 {
        return Vec::new();
    }
    let along = [d[0] / length, d[1] / length, d[2] / length];
    let mine = half_across(&p.shape);
    // What each joint takes out of the run: from and to, along it.
    let mut out_of_it: Vec<[f64; 2]> = places
        .iter()
        .map(|n| {
            let t = ((n[0] - a[0]) * along[0] + (n[1] - a[1]) * along[1] + (n[2] - a[2]) * along[2]).clamp(0.0, length);
            let excused = match &other.shape {
                Shape::Run { .. } => 3.0 * mine.max(half_across(&other.shape)),
                Shape::Box { at, size, angle } => deep_in_box(at, size, *angle, *n) + mine,
                _ => 0.0,
            } + FIT_TOLERANCE_FT;
            [t - excused, t + excused]
        })
        .collect();
    out_of_it.sort_by(|x, y| x[0].total_cmp(&y[0]));
    let mut kept: Vec<[f64; 2]> = Vec::new();
    let mut at = 0.0;
    for [from, to] in out_of_it {
        if from - at > FIT_TOLERANCE_FT {
            kept.push([at, from]);
        }
        at = at.max(to);
    }
    if length - at > FIT_TOLERANCE_FT {
        kept.push([at, length]);
    }
    let mut pieces = Vec::new();
    for [from, to] in kept {
        let point = |t: f64| [a[0] + along[0] * t, a[1] + along[1] * t, a[2] + along[2] * t];
        let piece = Placement { shape: Shape::Run { a: point(from), b: point(to), round: *round, rect: *rect }, ..p.clone() };
        pieces.append(&mut pieces_of(&piece, owner).unwrap_or_default());
    }
    pieces
}

/// What an element is called in a report: a placed thing by what it is, with its name.
fn said(p: &Placement, what: &BTreeMap<String, String>) -> String {
    match what.get(&p.path) {
        Some(what) => format!("{what} ({})", p.name),
        None => p.name.clone(),
    }
}

/// Every clash in the job. With a `clearance` (in feet), also every two things closer
/// than that. Nothing is changed and nothing is stored: the answer is worked out from the
/// job as it stands, so it cannot go stale.
pub fn clashes(job: &mut Job, clearance: Option<f64>) -> Result<Clashes, String> {
    if let Some(c) = clearance {
        if !(c.is_finite() && c > 0.0) {
            return Err("a clearance is a length of more than nothing".into());
        }
        if c > MAX_CLEARANCE_FT {
            return Err(format!("a clearance of more than {MAX_CLEARANCE_FT} ft is every two things in a room: ask for less"));
        }
    }
    let (drawn, not_drawn) = placements(job);
    let what: BTreeMap<String, String> = {
        let mut q = job.world.query::<(&Node, &Placed)>();
        q.iter(&job.world).filter_map(|(n, p)| p.what.value.clone().map(|w| (n.path.clone(), w))).collect()
    };
    let net = network::network(job, None, None, network::TOLERANCE_FT)?;
    // What is joined, and where: every two elements that meet at one joint.
    let mut joined: BTreeMap<(String, String), Vec<P3>> = BTreeMap::new();
    for node in &net.nodes {
        for (i, x) in node.joints.iter().enumerate() {
            for y in &node.joints[i + 1..] {
                if x.element() != y.element() {
                    joined.entry(pair(x.element(), y.element())).or_default().push(node.at);
                }
            }
        }
    }
    let mut left_out: Vec<(String, String)> = not_drawn.into_iter().map(|(path, why)| (path, format!("it has no shape: {why}"))).collect();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut solid: BTreeMap<&str, usize> = BTreeMap::new();
    let mut tested = 0;
    for (i, p) in drawn.iter().enumerate() {
        match pieces_of(p, i) {
            Ok(mut its) => {
                tested += 1;
                solid.insert(p.path.as_str(), i);
                pieces.append(&mut its);
            }
            Err(why) => left_out.push((p.path.clone(), why)),
        }
    }
    // For each two elements: whether any of their pieces are in each other, how deep at
    // the deepest, how near at the nearest, and where.
    let mut pairs: BTreeMap<(usize, usize), Met> = BTreeMap::new();
    let mut failed: BTreeSet<(usize, usize)> = BTreeSet::new();
    // Two pieces of two elements, the first of them the element the job holds first.
    let mut meet = |first: &Piece, second: &Piece, middle: P3, reach: Option<f64>| {
        let key = (first.owner, second.owner);
        match in_each_other(first, &first.pose, second, &second.pose) {
            Some(true) => {
                let deep = depth(first, second);
                // Where: the middle of where the boxes round them overlap, if that is in
                // both; else a place the measure of depth found in both.
                let inside = |p: &Piece, at: P3| p.solid.contains_point(&p.pose, Vector::new(at[0], at[1], at[2]));
                let at = match &deep {
                    Some(d) if !(inside(first, middle) && inside(second, middle)) => [d.at.x, d.at.y, d.at.z],
                    _ => middle,
                };
                let met = pairs.entry(key).or_insert(Met { hard: false, depth: None, gap: f64::INFINITY, at });
                if !met.hard {
                    (met.hard, met.at) = (true, at);
                }
                if let Some(d) = deep {
                    if met.depth.is_none_or(|was| d.by > was) {
                        (met.depth, met.at) = (Some(d.by), at);
                    }
                }
            }
            Some(false) => {
                if let Some(reach) = reach {
                    match query::distance(&first.pose, first.solid.as_ref(), &second.pose, second.solid.as_ref()) {
                        Ok(gap) if gap.distance < reach => {
                            let met = pairs.entry(key).or_insert(Met { hard: false, depth: None, gap: f64::INFINITY, at: middle });
                            if !met.hard && gap.distance < met.gap {
                                (met.gap, met.at) = (gap.distance, middle);
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {
                            failed.insert(key);
                        }
                    }
                }
            }
            None => {
                failed.insert(key);
            }
        }
    };
    // Where the boxes round two pieces overlap: its middle.
    let middle_of = |wide: &Aabb, q: &Aabb| {
        let (lo, hi) = (wide.mins.max(q.mins), wide.maxs.min(q.maxs));
        [(lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0, (lo.z + hi.z) / 2.0]
    };
    // Pieces in order of where they start to the east; a piece is compared only with
    // those that start before it ends.
    let reach = clearance.unwrap_or(0.0);
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by(|&i, &j| pieces[i].aabb.mins.x.total_cmp(&pieces[j].aabb.mins.x));
    for (n, &i) in order.iter().enumerate() {
        let wide = pieces[i].aabb.loosened(reach);
        for &j in &order[n + 1..] {
            if pieces[j].aabb.mins.x > wide.maxs.x {
                break;
            }
            let (p, q) = (&pieces[i], &pieces[j]);
            if p.owner == q.owner || !wide.intersects(&q.aabb) {
                continue;
            }
            let (first, second) = if p.owner < q.owner { (p, q) } else { (q, p) };
            let (x, y) = (&drawn[first.owner], &drawn[second.owner]);
            // What is joined is tested apart, below, with its joints taken out.
            if (built_in(x.kind) && built_in(y.kind)) || joined.contains_key(&pair(&x.path, &y.path)) {
                continue;
            }
            meet(first, second, middle_of(&wide, &q.aabb), clearance);
        }
    }
    // What is joined: each of the two with its joints taken out, and what is left of one
    // against what is left of the other. Two fixtures or fittings joined to each other
    // have no run to leave out, and are not tested. No clearance is asked between things
    // that are joined: they are meant to touch.
    for ((a, b), places) in &joined {
        let (Some(&i), Some(&j)) = (solid.get(a.as_str()), solid.get(b.as_str())) else { continue };
        let (i, j) = (i.min(j), i.max(j));
        let (x, y) = (&drawn[i], &drawn[j]);
        if !(matches!(x.shape, Shape::Run { .. }) || matches!(y.shape, Shape::Run { .. })) {
            continue;
        }
        let (firsts, seconds) = (away_from_joints(x, y, places, i), away_from_joints(y, x, places, j));
        for first in &firsts {
            for second in &seconds {
                if first.aabb.intersects(&second.aabb) {
                    meet(first, second, middle_of(&first.aabb, &second.aabb), None);
                }
            }
        }
    }
    // Against a wall or a floor, which is several solids, the way out of one piece can
    // be into the next. The depth is the least of the ways out that clear every piece:
    // each piece's own, where it does, and straight out through the face.
    let through = |x: &Placement, y: &Placement| {
        // A fitting is part of its run: an elbow in a wall is the run through the wall.
        let of_a_run = |k: Kind| k.is_segment() || k == Kind::Fitting;
        (of_a_run(x.kind) && built_in(y.kind)) || (of_a_run(y.kind) && built_in(x.kind))
    };
    for (&(i, j), met) in pairs.iter_mut() {
        let (x, y) = (&drawn[i], &drawn[j]);
        if !met.hard || through(x, y) || built_in(x.kind) == built_in(y.kind) {
            continue;
        }
        let (whole, thing) = if built_in(x.kind) { (i, j) } else { (j, i) };
        let of_whole: Vec<&Piece> = pieces.iter().filter(|p| p.owner == whole).collect();
        let of_thing: Vec<&Piece> = pieces.iter().filter(|p| p.owner == thing).collect();
        if of_whole.len() < 2 {
            continue;
        }
        // Whether the thing, moved so far that way, is clear of every piece.
        let clears = |way: Vector, by: f64| {
            of_thing.iter().all(|t| {
                let moved = Pose::from_parts(t.pose.translation + way * (by + FIT_TOLERANCE_FT), t.pose.rotation);
                of_whole.iter().all(|w| in_each_other(w, &w.pose, t, &moved) == Some(false))
            })
        };
        let mut least: Option<f64> = None;
        let mut take = |by: f64| {
            if by > FIT_TOLERANCE_FT && least.is_none_or(|was| by < was) {
                least = Some(by);
            }
        };
        if let Some((way, from, to)) = of_whole[0].thin {
            // How far the thing reaches each way along the wall's thickness.
            let reach = |t: &Piece, way: Vector| t.solid.as_support_map().map(|s| s.support_point(&t.pose, way).dot(way));
            let low = of_thing.iter().filter_map(|t| reach(t, -way)).map(|x| -x).fold(f64::INFINITY, f64::min);
            let high = of_thing.iter().filter_map(|t| reach(t, way)).fold(f64::NEG_INFINITY, f64::max);
            if low.is_finite() && high.is_finite() {
                for (way, by) in [(way, to - low), (-way, high - from)] {
                    if clears(way, by) {
                        take(by);
                    }
                }
            }
        }
        for w in &of_whole {
            for t in &of_thing {
                if in_each_other(w, &w.pose, t, &t.pose) != Some(true) {
                    continue;
                }
                // `depth` says which way its second solid moves; the thing is moved.
                if let Some(deep) = depth(w, t) {
                    if clears(deep.way, deep.by) {
                        take(deep.by);
                    }
                }
            }
        }
        met.depth = least;
    }
    for (i, j) in failed {
        left_out.push((drawn[i].path.clone(), format!("could not be tested against {}", drawn[j].path)));
    }
    let mut found: Vec<Clash> = Vec::new();
    for ((i, j), met) in pairs {
        let (x, y) = (&drawn[i], &drawn[j]);
        let through = through(x, y);
        let (how, by) = if met.hard { (ClashKind::Hard, met.depth.filter(|_| !through)) } else { (ClashKind::Clearance, Some(met.gap.max(0.0))) };
        found.push(Clash { a: x.path.clone(), b: y.path.clone(), a_name: said(x, &what), b_name: said(y, &what), a_kind: x.kind, b_kind: y.kind, how, through, by, at: met.at, status: x.status.max(y.status) });
    }
    // Things in each other first, then holes to cut, then what is only close.
    found.sort_by_key(|c| (c.how, c.through && c.how == ClashKind::Hard));
    Ok(Clashes { found, tested, left_out })
}
