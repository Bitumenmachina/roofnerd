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
//! * walls and floors against each other: they are built into one another;
//! * what is joined: two runs that meet at a joint, a run and its fitting, a run and the
//!   thing its end is connected to (the job's network says which);
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
//! had is reported as a clash with no depth. It is never dropped.
//!
//! A clash is as sure as the two shapes: Assumed when either is.

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
    "walls and floors are not tested against each other",
    "what is joined is not tested: runs at a joint, a run and its fitting, a run and the thing its end is connected to",
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
}

/// How far each solid is drawn in before two are tested for being in the same space.
const DRAWN_IN: f64 = FIT_TOLERANCE_FT / 2.0;

impl Piece {
    fn new(owner: usize, pose: Pose, solid: Box<dyn Solid>, core: Option<Box<dyn Solid>>) -> Piece {
        let aabb = solid.compute_aabb(&pose);
        Piece { owner, pose, solid, core, aabb }
    }
    fn cuboid(owner: usize, pose: Pose, half: Vector) -> Piece {
        let core: Option<Box<dyn Solid>> = (half.min_element() > DRAWN_IN).then(|| Box::new(Cuboid::new(half - Vector::splat(DRAWN_IN))) as Box<dyn Solid>);
        Piece::new(owner, pose, Box::new(Cuboid::new(half)), core)
    }
    fn cylinder(owner: usize, pose: Pose, half_length: f64, radius: f64) -> Piece {
        let core: Option<Box<dyn Solid>> = (half_length.min(radius) > DRAWN_IN).then(|| Box::new(Cylinder::new(half_length - DRAWN_IN, radius - DRAWN_IN)) as Box<dyn Solid>);
        Piece::new(owner, pose, Box::new(Cylinder::new(half_length, radius)), core)
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
            // A solid piece of wall from `x0` to `x1` along it and `z0` to `z1` up it.
            let mut slab = |x0: f64, x1: f64, z0: f64, z1: f64| {
                if x1 - x0 < 1.0e-6 || z1 - z0 < 1.0e-6 {
                    return;
                }
                let (mx, mz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
                let pose = Pose::from_parts(Vector::new(a[0] + ux * mx, a[1] + uy * mx, base + mz), turn);
                out.push(Piece::cuboid(owner, pose, Vector::new((x1 - x0) / 2.0, thickness / 2.0, (z1 - z0) / 2.0)));
            };
            let mut cuts: Vec<[f64; 4]> = holes.iter().map(|h| h.2).collect();
            cuts.sort_by(|x, y| x[0].total_cmp(&y[0]));
            let mut at = 0.0;
            for [from, to, sill, head] in cuts {
                slab(at, from, 0.0, *height);
                slab(from, to, 0.0, sill);
                slab(from, to, head, *height);
                at = at.max(to);
            }
            slab(at, length, 0.0, *height);
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
                out.push(Piece::new(owner, Pose::identity(), Box::new(whole), core));
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

/// How far one of two solids that are in each other would have to move to clear the
/// other, in feet; none when the library cannot say. It is asked with the second solid a
/// billionth of a foot off its place, three ways, because dead centre it answers zero.
fn depth(first: &Piece, second: &Piece) -> Option<f64> {
    const OFF: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [0.618, 0.382, 0.789], [-0.3, 0.9, 0.1]];
    let mut deepest: Option<f64> = None;
    for off in OFF {
        let pose = Pose::from_parts(second.pose.translation + Vector::new(off[0], off[1], off[2]) * 1.0e-9, second.pose.rotation);
        if let Ok(Some(c)) = query::contact(&first.pose, first.solid.as_ref(), &pose, second.solid.as_ref(), 0.0) {
            if c.dist < -FIT_TOLERANCE_FT && deepest.is_none_or(|d| -c.dist > d) {
                deepest = Some(-c.dist);
            }
        }
    }
    deepest
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
    // What is joined is not a clash: every two elements that meet at one joint.
    let mut joined: BTreeSet<(String, String)> = BTreeSet::new();
    for node in &net.nodes {
        for (i, x) in node.joints.iter().enumerate() {
            for y in &node.joints[i + 1..] {
                if x.element() != y.element() {
                    joined.insert(pair(x.element(), y.element()));
                }
            }
        }
    }
    let mut left_out: Vec<(String, String)> = not_drawn.into_iter().map(|(path, why)| (path, format!("it has no shape: {why}"))).collect();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut tested = 0;
    for (i, p) in drawn.iter().enumerate() {
        match pieces_of(p, i) {
            Ok(mut its) => {
                tested += 1;
                pieces.append(&mut its);
            }
            Err(why) => left_out.push((p.path.clone(), why)),
        }
    }
    // Pieces in order of where they start to the east; a piece is compared only with
    // those that start before it ends.
    let reach = clearance.unwrap_or(0.0);
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by(|&i, &j| pieces[i].aabb.mins.x.total_cmp(&pieces[j].aabb.mins.x));
    // For each two elements: whether any of their pieces are in each other, how deep at
    // the deepest, how near at the nearest, and where.
    let mut pairs: BTreeMap<(usize, usize), Met> = BTreeMap::new();
    let mut failed: BTreeSet<(usize, usize)> = BTreeSet::new();
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
            if (built_in(x.kind) && built_in(y.kind)) || joined.contains(&pair(&x.path, &y.path)) {
                continue;
            }
            let key = (first.owner, second.owner);
            // Where the two boxes round them overlap: its middle.
            let (lo, hi) = (wide.mins.max(q.aabb.mins), wide.maxs.min(q.aabb.maxs));
            let at = [(lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0, (lo.z + hi.z) / 2.0];
            match query::intersection_test(&first.pose, first.core(), &second.pose, second.core()) {
                Ok(hit) if hit.intersecting => {
                    let met = pairs.entry(key).or_insert(Met { hard: false, depth: None, gap: f64::INFINITY, at });
                    if !met.hard {
                        (met.hard, met.at) = (true, at);
                    }
                    if let Some(d) = depth(first, second) {
                        if met.depth.is_none_or(|was| d > was) {
                            (met.depth, met.at) = (Some(d), at);
                        }
                    }
                }
                Ok(_) if clearance.is_some() => match query::distance(&first.pose, first.solid.as_ref(), &second.pose, second.solid.as_ref()) {
                    Ok(gap) if gap.distance < reach => {
                        let met = pairs.entry(key).or_insert(Met { hard: false, depth: None, gap: f64::INFINITY, at });
                        if !met.hard && gap.distance < met.gap {
                            (met.gap, met.at) = (gap.distance, at);
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {
                        failed.insert(key);
                    }
                },
                Ok(_) => {}
                Err(_) => {
                    failed.insert(key);
                }
            }
        }
    }
    for (i, j) in failed {
        left_out.push((drawn[i].path.clone(), format!("could not be tested against {}", drawn[j].path)));
    }
    let mut found: Vec<Clash> = Vec::new();
    for ((i, j), met) in pairs {
        let (x, y) = (&drawn[i], &drawn[j]);
        // A fitting is part of its run: an elbow in a wall is the run through the wall.
        let of_a_run = |k: Kind| k.is_segment() || k == Kind::Fitting;
        let through = (of_a_run(x.kind) && built_in(y.kind)) || (of_a_run(y.kind) && built_in(x.kind));
        let (how, by) = if met.hard { (ClashKind::Hard, met.depth.filter(|_| !through)) } else { (ClashKind::Clearance, Some(met.gap.max(0.0))) };
        found.push(Clash { a: x.path.clone(), b: y.path.clone(), a_name: said(x, &what), b_name: said(y, &what), a_kind: x.kind, b_kind: y.kind, how, through, by, at: met.at, status: x.status.max(y.status) });
    }
    // Things in each other first, then holes to cut, then what is only close.
    found.sort_by_key(|c| (c.how, c.through && c.how == ClashKind::Hard));
    Ok(Clashes { found, tested, left_out })
}
