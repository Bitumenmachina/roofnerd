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
//!   either side of it (between two runs, the sum of their half-widths over the sine of
//!   the angle they meet at, and no more than that at fifteen degrees; into a fixture,
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
//! A run cut in two is still one length of duct or pipe: where a run carries on, in one
//! line and at one size, into a run joined to its end, the depth of what crosses it is
//! measured against the whole length. A pipe through a duct at a cut is in each piece
//! by as much as it was in the duct before the cut, not by the inch it reaches into the
//! end of a piece.
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

use super::{network, placements, Placed, Placement, Shape, P2, P3};
use crate::check::FIT_TOLERANCE_FT;
use crate::components::{Kind, Node};
use crate::geom;
use crate::job::Job;
use crate::units;
use crate::value::Status;
use crate::view;

/// How far a thing may be in a wall, or a run in what it is joined to at its end, and be set
/// against it and not in it: two inches (call 175).
pub const SET_AGAINST_FT: f64 = 2.0 / 12.0;

/// How far from a joint a clash may lie, in feet, and be at that joint: a termination is inches in, where a branch drawn a foot back up inside its main is a drawing fault and stays a clash (demos/clash.sh).
const JOINT_REACH_FT: f64 = 0.25;

/// The widest clearance that can be asked for, in feet. Past this the answer is every
/// pair of things in a room.
pub const MAX_CLEARANCE_FT: f64 = 10.0;

/// What is never tested against what, for a report to say.
pub const NEVER_TESTED: [&str; 4] = [
    "walls and floors are not tested against each other, nor wall against wall, nor floor against floor; a footing is tested against neither, nor against another footing",
    "a joint is not a clash: each run is left out a little way either side of where it is joined",
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
    /// A run that lies in a floor or a wall along its length (a conduit cast in a slab, a pipe
    /// in the thickness of a wall): it never comes out, so it is no hole to cut. It is a
    /// `through` clash that has no hole, listed and counted apart (call 179, C5).
    pub laid_in: bool,
    /// A thing standing against a wall, or a run at its joint, in each other by no more than
    /// `SET_AGAINST_FT` (call 175): counted and listed apart, and not in the same space.
    pub set_against: bool,
    /// A hole to cut, for a run through a wall or a floor: which of `Clashes::holes` it is a
    /// piece of. A riser of three pieces through one floor is one hole.
    pub hole: Option<usize>,
    /// Hard: how far one would have to move to clear the other, in feet; none for a run
    /// through a wall or a floor (it is meant to go through), and none where the depth
    /// could not be worked out. Clearance: how far apart they are, in feet.
    pub by: Option<f64>,
    /// Where, in feet: east, north, height above the datum.
    pub at: P3,
    pub status: Status,
}

/// One hole to cut in a wall or a floor, for a run (call 175): the pieces of the run that
/// pass through it are one hole.
#[derive(Clone, Debug, PartialEq)]
pub struct Hole {
    /// The wall or floor it is cut in.
    pub through: String,
    pub through_name: String,
    pub through_kind: Kind,
    /// The pieces of the run, and the fittings on it, that pass through, by path and name.
    pub pieces: Vec<(String, String)>,
    /// The widest of them, as a person says it ("6 in round", "12 in by 8 in"); empty where
    /// no piece has a size.
    pub size: String,
    /// The same size as a builder says it in a sentence ("14 in by 8 in", "4 in round"; call 226 point 2). `size` is the writing of the lists and
    /// does not change.
    pub size_said: String,
    /// Where, in feet: east, north, height above the datum.
    pub at: P3,
    pub status: Status,
}

impl Clashes {
    /// What is in the same space as what: two things in each other by more than they could be
    /// set to, and not a run through a wall or a floor (a hole) nor a thing set against one.
    /// The count a window shows is this one's (call 177).
    pub fn in_the_same_space(&self) -> Vec<&Clash> {
        self.found.iter().filter(|c| c.how == ClashKind::Hard && !c.through && !c.set_against).collect()
    }
    /// What stands against a wall, or is joined to it, by no more than `SET_AGAINST_FT`: not a
    /// clash, and listed apart (call 175).
    pub fn set_against(&self) -> Vec<&Clash> {
        self.found.iter().filter(|c| c.how == ClashKind::Hard && !c.through && c.set_against).collect()
    }
    /// What lies in a floor or a wall along its length: not holes (call 179, C5).
    pub fn laid_in(&self) -> Vec<&Clash> {
        self.found.iter().filter(|c| c.how == ClashKind::Hard && c.laid_in).collect()
    }
    /// The holes to cut, each once: the first piece of a run that passes through a wall or a
    /// floor stands for its hole (call 175).
    pub fn hole_rows(&self) -> Vec<&Clash> {
        let mut seen: Vec<usize> = Vec::new();
        self.found
            .iter()
            .filter(|c| c.how == ClashKind::Hard && c.through && !c.laid_in)
            .filter(|c| match c.hole {
                Some(h) if seen.contains(&h) => false,
                Some(h) => {
                    seen.push(h);
                    true
                }
                None => true,
            })
            .collect()
    }
}

/// What a clash says of a thing's connections (call 211, point 7): where a run ends inside a thing whose item says where it
/// connects, and the thing has an open connection of the run's sort, which connection and how far away it is ("Test lavatory's Hot
/// water connection is 3 in away"). Nothing else about the clash moves: not a count, not a kind. None where the clash is not a run
/// ending inside a fixture or a piece of equipment, or the thing has no such connection open.
pub fn connection_words(job: &mut Job, c: &Clash) -> Option<String> {
    if c.how != ClashKind::Hard || c.through {
        return None;
    }
    let placed = |k: Kind| matches!(k, Kind::Fixture | Kind::Equipment);
    let (run, thing, name) = if c.a_kind.is_segment() && placed(c.b_kind) {
        (&c.a, &c.b, &c.b_name)
    } else if c.b_kind.is_segment() && placed(c.a_kind) {
        (&c.b, &c.a, &c.a_name)
    } else {
        return None;
    };
    // A run joined to a connection of the thing that goes on inside it: said as that, so that the clash and `takeoff` read as one fact
    // (call 213 point 3).
    if let Ok(Some((connection, inside))) = crate::editing::connections::run_joined_and_inside(job, thing, run) {
        return Some(format!("joined to {name}'s {connection} connection and goes on {} inside it", crate::units::inches_said(inside)));
    }
    let near = crate::editing::connections::run_end_near_connection(job, thing, run).ok()??;
    Some(format!("{name}'s {} connection is {}", near.name, near.away()))
}

/// The storey a clash is on, by path: the storey of whichever of the two things is not a wall, a floor or a footing (a wall holds the
/// thing, the thing is where the question is), else the first's. One rule, for the room a clash is in (`rooms_of`) and the level
/// the issue list says (call 228).
pub fn storey_of_pair(placed: &[Placement], a: &str, b: &str) -> Option<String> {
    let storey_of = |path: &str| placed.iter().find(|p| p.path == path).and_then(|p| p.storey.clone());
    let kind_of = |path: &str| placed.iter().find(|p| p.path == path).map(|p| p.kind);
    let solid = |k: Option<Kind>| !matches!(k, Some(Kind::Wall | Kind::Floor | Kind::Footing));
    if solid(kind_of(a)) { storey_of(a) } else if solid(kind_of(b)) { storey_of(b) } else { storey_of(a) }
}

/// The box that holds a placed thing's solids, as the clash test holds them: its smallest and its largest corner, in feet (east,
/// north, height above the datum). None for a thing with no solid. The issue list's view is worked from it (call 228).
pub fn extent(p: &Placement) -> Option<(P3, P3)> {
    let pieces = pieces_of(p, 0).ok()?;
    let mut it = pieces.iter();
    let first = it.next()?;
    let (mut lo, mut hi) = ([first.aabb.mins.x, first.aabb.mins.y, first.aabb.mins.z], [first.aabb.maxs.x, first.aabb.maxs.y, first.aabb.maxs.z]);
    for q in it {
        for (k, (a, b)) in [(q.aabb.mins.x, q.aabb.maxs.x), (q.aabb.mins.y, q.aabb.maxs.y), (q.aabb.mins.z, q.aabb.maxs.z)].into_iter().enumerate() {
            lo[k] = lo[k].min(a);
            hi[k] = hi[k].max(b);
        }
    }
    Some((lo, hi))
}

impl Clash {
    /// How far they are in each other, as the clash list writes it ("in each other by 3.50 in"): a line of a list, its quantity to
    /// two places. The command line's list and the issue list's title say it by this one function.
    pub fn depth_listed(&self) -> String {
        match self.by {
            Some(by) => format!("in each other by {:.2} in", by * 12.0 + 0.0),
            None => "in each other; by how much could not be worked out".to_string(),
        }
    }
}

/// Where each clash and each hole of `found` is, to the room: the name of the room whose
/// outline holds the place on the storey the thing in it is on, or "outside any room"
/// where none does (call 177). One for each of `found.found`, then one for each of
/// `found.holes`. The storey is the one of whichever of the two is not a wall or a floor.
pub fn rooms_of(job: &mut Job, found: &Clashes) -> (Vec<String>, Vec<String>) {
    const NONE: &str = "outside any room";
    let rooms: Vec<(String, String, Vec<P2>)> = super::room_outlines(job)
        .into_iter()
        .filter_map(|(path, outline)| {
            let e = job.find(&path)?;
            let n = job.world.get::<Node>(e)?;
            Some((n.parent.clone()?, n.name.clone(), outline))
        })
        .collect();
    let placed = placements(job).0;
    let storey_of = |path: &str| placed.iter().find(|p| p.path == path).and_then(|p| p.storey.clone());
    let room = |storey: Option<String>, at: P3| -> String {
        let Some(storey) = storey else { return NONE.to_string() };
        rooms.iter().find(|(of, _, outline)| *of == storey && crate::vif::inside(outline, [at[0], at[1]])).map(|r| r.1.clone()).unwrap_or_else(|| NONE.to_string())
    };
    let thing = |a: &str, b: &str| -> Option<String> { storey_of_pair(&placed, a, b) };
    let clashes = found.found.iter().map(|c| room(thing(&c.a, &c.b), c.at)).collect();
    let holes = found.holes.iter().map(|h| room(h.pieces.first().and_then(|(p, _)| storey_of(p)).or_else(|| storey_of(&h.through)), h.at)).collect();
    (clashes, holes)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clashes {
    /// Clashes between things, then runs through walls and floors, then clearances.
    pub found: Vec<Clash>,
    /// The holes to cut, each once (call 175).
    pub holes: Vec<Hole>,
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
    /// A run that carries on, in one line and at one size, into a run joined to its
    /// end: the whole length of it, which is what a thing across it is deep in.
    whole: Option<Box<Piece>>,
}

/// How far each solid is drawn in before two are tested for being in the same space.
const DRAWN_IN: f64 = FIT_TOLERANCE_FT / 2.0;

impl Piece {
    fn new(owner: usize, pose: Pose, solid: Box<dyn Solid>, core: Option<Box<dyn Solid>>) -> Piece {
        let aabb = solid.compute_aabb(&pose);
        Piece { owner, pose, solid, core, aabb, round: None, thin: None, whole: None }
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
                (None, None) => return Err("its size is not known, so it has no shape to test".into()),
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

/// What the building is made of: walls, floors and footings meet each other by design
/// (a wall stands on its footing, two footings meet at a corner) and are not tested
/// against each other.
fn built_in(kind: Kind) -> bool {
    matches!(kind, Kind::Wall | Kind::Floor | Kind::Footing)
}

/// What carries the building: a beam, a post, a pad footing (call 164). Framing bears on
/// framing, so none of it is tested against walls, floors, footings or each other; what
/// it is tested against is every run and every thing set in place.
fn bears(x: Kind, y: Kind) -> bool {
    (x.is_framing() && (built_in(y) || y.is_framing())) || (y.is_framing() && built_in(x))
}

/// What a report says of framing, once, where the job has any (call 164).
pub const FRAMING_NOT_TESTED: &str = "a beam, a post and a pad footing are not tested against walls, floors, footings or each other: framing bears on framing (a post under a beam, a beam in a wall's line, a pad under a post); each is tested against every run and every thing set in place";

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
    // A run cut in two is one length of duct to what crosses it at the cut (J17A-20).
    let (first, second) = (first.whole.as_deref().unwrap_or(first), second.whole.as_deref().unwrap_or(second));
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
                // Two runs meeting at an angle are in each other, along either, for the
                // sum of their half-widths over the sine of the angle: that much is the
                // joint. Runs within fifteen degrees of one line are excused no more
                // than runs at fifteen degrees: past that they lie along each other.
                Shape::Run { a: oa, b: ob, .. } => {
                    let od = [ob[0] - oa[0], ob[1] - oa[1], ob[2] - oa[2]];
                    let ol = (od[0] * od[0] + od[1] * od[1] + od[2] * od[2]).sqrt().max(1.0e-12);
                    let cross = [along[1] * od[2] - along[2] * od[1], along[2] * od[0] - along[0] * od[2], along[0] * od[1] - along[1] * od[0]];
                    let sine = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() / ol;
                    (mine + half_across(&other.shape)) / sine.max(0.258_819)
                }
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

/// Each run's centre line carried on through the runs joined to its ends in one line and
/// at one size, for the runs that have such a neighbour: by where it stands in `drawn`.
/// A cut is bookkeeping. It moves nothing, and what crosses a run at a cut is in the
/// whole run, not in the last inch of one piece.
/// Whether a straight run, from `a` to `b`, lies in a wall or a floor along its length: both its
/// ends within the thickness (across a wall, down a floor), so it never comes out (call 179, C5).
fn lies_in(a: P3, b: P3, wall: &Placement) -> bool {
    let tol = FIT_TOLERANCE_FT;
    match &wall.shape {
        Shape::Wall { a: wa, b: wb, thickness, .. } => {
            let (e, long) = ([wb[0] - wa[0], wb[1] - wa[1]], (wb[0] - wa[0]).hypot(wb[1] - wa[1]));
            if long < 1.0e-9 {
                return false;
            }
            [a, b].iter().all(|p| ((p[0] - wa[0]) * e[1] - (p[1] - wa[1]) * e[0]).abs() / long <= thickness / 2.0 + tol)
        }
        Shape::Floor { top, thickness, .. } => [a, b].iter().all(|p| p[2] <= top + tol && p[2] >= top - thickness - tol),
        _ => false,
    }
}

/// Where a straight run, from `a` to `b`, passes through a wall (the middle of its thickness) or a
/// floor (the middle of its depth): None where it does not cross it between its two ends, or lies
/// along it.
fn cross_at(a: P3, b: P3, wall: &Placement) -> Option<P3> {
    let t = match &wall.shape {
        Shape::Wall { a: wa, b: wb, .. } => {
            let (d, e) = ([b[0] - a[0], b[1] - a[1]], [wb[0] - wa[0], wb[1] - wa[1]]);
            let denom = d[0] * e[1] - d[1] * e[0];
            if denom.abs() < 1.0e-9 {
                return None;
            }
            ((wa[0] - a[0]) * e[1] - (wa[1] - a[1]) * e[0]) / denom
        }
        Shape::Floor { top, thickness, .. } => {
            let dz = b[2] - a[2];
            if dz.abs() < 1.0e-9 {
                return None;
            }
            (top - thickness / 2.0 - a[2]) / dz
        }
        _ => return None,
    };
    (-1.0e-9..=1.0 + 1.0e-9).contains(&t).then(|| [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]), a[2] + t * (b[2] - a[2])])
}

fn carried_on(drawn: &[Placement], net: &network::Network) -> BTreeMap<usize, (P3, P3)> {
    let index: BTreeMap<&str, usize> = drawn.iter().enumerate().filter(|(_, p)| matches!(p.shape, Shape::Run { .. })).map(|(i, p)| (p.path.as_str(), i)).collect();
    // Which run ends meet at each joint: (run, whether it is its far end).
    let mut at_end: BTreeMap<(usize, bool), Vec<(usize, bool)>> = BTreeMap::new();
    for node in &net.nodes {
        let ends: Vec<(usize, bool)> = node.joints.iter().filter_map(|j| match j {
            network::Joint::RunEnd { run, end } => index.get(run.as_str()).map(|&i| (i, *end)),
            _ => None,
        }).collect();
        for &e in &ends {
            at_end.insert(e, ends.iter().copied().filter(|&o| o.0 != e.0).collect());
        }
    }
    let line = |i: usize| match &drawn[i].shape {
        Shape::Run { a, b, round, rect } => (*a, *b, *round, *rect),
        _ => unreachable!("only runs are indexed"),
    };
    let unit = |from: P3, to: P3| {
        let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        (l > 1.0e-9).then(|| [d[0] / l, d[1] / l, d[2] / l])
    };
    let same = |x: Option<f64>, y: Option<f64>| match (x, y) {
        (Some(x), Some(y)) => (x - y).abs() < 1.0e-9,
        (None, None) => true,
        _ => false,
    };
    // From one end of a run, on through each run that carries on from it: where the
    // length ends.
    let reach = |start: usize, far: bool| -> Option<P3> {
        let (a, b, round, rect) = line(start);
        let way = unit(if far { a } else { b }, if far { b } else { a })?;
        let (mut at, mut end, mut reached) = (start, far, None);
        for _ in 0..drawn.len() {
            let next = at_end.get(&(at, end)).into_iter().flatten().find(|&&(o, o_end)| {
                let (oa, ob, o_round, o_rect) = line(o);
                let on = unit(if o_end { ob } else { oa }, if o_end { oa } else { ob });
                // Within about half a degree of one line, and going on, not back.
                o != start && same(round, o_round) && same(rect.map(|r| r.0), o_rect.map(|r| r.0)) && same(rect.map(|r| r.1), o_rect.map(|r| r.1)) && on.is_some_and(|on| on[0] * way[0] + on[1] * way[1] + on[2] * way[2] > 0.99996)
            });
            let Some(&(o, o_end)) = next else { break };
            let (oa, ob, ..) = line(o);
            reached = Some(if o_end { oa } else { ob });
            (at, end) = (o, !o_end);
        }
        reached
    };
    let mut out = BTreeMap::new();
    for &i in index.values() {
        let (a, b, ..) = line(i);
        let (back, on) = (reach(i, false), reach(i, true));
        if back.is_some() || on.is_some() {
            out.insert(i, (back.unwrap_or(a), on.unwrap_or(b)));
        }
    }
    out
}

/// What each thing set down is, by its path: the name of its item, where it has one.
fn what_of(job: &mut Job) -> BTreeMap<String, String> {
    let mut q = job.world.query::<(&Node, &Placed)>();
    q.iter(&job.world).filter_map(|(n, p)| p.what.value.clone().map(|w| (n.path.clone(), w))).collect()
}

/// The name each drawn thing goes by in the clash list and everything that names a clash, by path (call 229: the parts of the house
/// written as glTF are named by it). One rule: the same `said` the clashes themselves are named with.
pub fn called(job: &mut Job, drawn: &[Placement]) -> std::collections::HashMap<String, String> {
    let what = what_of(job);
    drawn.iter().map(|p| (p.path.clone(), said(p, &what))).collect()
}

/// What an element is called in a report: a placed thing by what it is, with its name.
fn said(p: &Placement, what: &BTreeMap<String, String>) -> String {
    match what.get(&p.path) {
        // A thing called what it is ("Boiler", "Lavatory 2") is said once.
        Some(what) if p.name == *what || p.name.strip_prefix(what.as_str()).is_some_and(|rest| rest.strip_prefix(' ').is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))) => p.name.clone(),
        // What it is already begins with its name ("Kitchen sink, double bowl", named
        // "kitchen sink"): said once, not "Kitchen sink, double bowl (kitchen sink)"
        // (J24B-33).
        Some(what) if what.to_lowercase().starts_with(&p.name.to_lowercase()) => what.clone(),
        Some(what) => format!("{what} ({})", p.name),
        None => p.name.clone(),
    }
}

/// Every clash in the job. With a `clearance` (in feet), also every two things closer
/// than that. Nothing is changed and nothing is stored: the answer is worked out from the
/// job as it stands, so it cannot go stale.
/// Whether the job has a beam, a post or a pad footing, so a report says what is not
/// tested of them (a job with none prints what it always printed).
pub fn has_framing(job: &mut Job) -> bool {
    let mut q = job.world.query::<&Kind>();
    q.iter(&job.world).any(|k| k.is_framing())
}

pub fn clashes(job: &mut Job, clearance: Option<f64>) -> Result<Clashes, String> {
    find(job, clearance, false)
}

/// What is in the way of a wall that comes down (call 160): a run, a fitting or a thing set
/// in place that is in a wall marked to demolish and is not itself to demolish, found with
/// the same shapes the clash test uses (a wall that is to demolish is in nothing's way
/// once the work is done, and so is not in `clashes`; it is here, with its own meaning:
/// what has to be capped, rerouted or taken out when the wall comes down). Nothing is
/// returned for a job in which no wall is marked to demolish, and nothing is tested.
pub fn in_the_way(job: &mut Job) -> Result<Vec<crate::hidden::InTheWay>, String> {
    let gone = crate::components::demolished(&mut job.world);
    let walls: BTreeSet<String> = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).filter(|(n, k)| **k == Kind::Wall && gone.contains(&n.path)).map(|(n, _)| n.path.clone()).collect()
    };
    if walls.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for c in find(job, None, true)?.found {
        if c.how != ClashKind::Hard {
            continue;
        }
        let (wall, wall_name, thing, thing_name, thing_kind) = match (walls.contains(&c.a), walls.contains(&c.b)) {
            (true, false) => (c.a, c.a_name, c.b, c.b_name, c.b_kind),
            (false, true) => (c.b, c.b_name, c.a, c.a_name, c.a_kind),
            _ => continue,
        };
        if !(thing_kind.is_segment() || thing_kind == Kind::Fitting || thing_kind.is_placed()) || gone.contains(&thing) {
            continue;
        }
        out.push(crate::hidden::InTheWay { thing, thing_name, thing_kind, wall, wall_name, at: c.at });
    }
    // Where each is, for a person with a bar in the wall: on the thing and inside the wall,
    // not where the clash test found it deepest (a run that carries on past the wall, or
    // meets others at a joint, was given a place a foot off it: call 167).
    let (drawn, _) = placements(job);
    for w in out.iter_mut() {
        let shape = |path: &str| drawn.iter().find(|p| p.path == path).map(|p| &p.shape);
        if let (Some(Shape::Wall { a, b, base, height, thickness, .. }), Some(thing)) = (shape(&w.wall), shape(&w.thing)) {
            if let Some(at) = inside_wall(*a, *b, (*base, base + height), *thickness, thing, w.at) {
                w.at = at;
            }
        }
    }
    Ok(out)
}

/// A place on a thing that lies inside a wall: for a run, the middle of the length of its
/// centre line that is in the wall (the wall's thickness, widened by the run's own, if the
/// centre line is not itself in it); for a thing in a box, its middle held inside the wall.
/// `near` is where the clash was found, used when nothing better can be said.
fn inside_wall(a: P2, b: P2, up: (f64, f64), thickness: f64, thing: &Shape, near: P3) -> Option<P3> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx.hypot(dy);
    if length <= 1.0e-9 {
        return None;
    }
    let (ux, uy) = (dx / length, dy / length);
    // A place in the wall's own directions: along it, across it, and up.
    let local = |p: P3| ((p[0] - a[0]) * ux + (p[1] - a[1]) * uy, -(p[0] - a[0]) * uy + (p[1] - a[1]) * ux, p[2]);
    let world = |u: f64, v: f64, z: f64| [a[0] + ux * u - uy * v, a[1] + uy * u + ux * v, z];
    let held = |u: f64, v: f64, z: f64| world(u.clamp(0.0, length), v.clamp(-thickness / 2.0, thickness / 2.0), z.clamp(up.0, up.1));
    match thing {
        Shape::Run { a: from, b: to, round, rect } => {
            let (p, q) = (local(*from), local(*to));
            // How far the run's own body reaches beyond its centre line, across the wall.
            let reach = round.map(|d| d / 2.0).or(rect.map(|(w, h)| w.max(h) / 2.0)).unwrap_or(0.0);
            // The part of the centre line, from p to q, that is within the wall's length and
            // within half its thickness to either side; if none is, within that and the run's
            // own reach.
            let span = |half: f64| -> Option<(f64, f64)> {
                let (mut s0, mut s1) = (0.0_f64, 1.0_f64);
                for (from, to, low, high) in [(p.0, q.0, 0.0, length), (p.1, q.1, -half, half)] {
                    let d = to - from;
                    if d.abs() < 1.0e-12 {
                        if from < low - 1.0e-9 || from > high + 1.0e-9 {
                            return None;
                        }
                        continue;
                    }
                    let (t0, t1) = ((low - from) / d, (high - from) / d);
                    s0 = s0.max(t0.min(t1));
                    s1 = s1.min(t0.max(t1));
                }
                (s0 <= s1).then_some((s0, s1))
            };
            let Some((s0, s1)) = span(thickness / 2.0).or_else(|| span(thickness / 2.0 + reach)) else {
                return Some(held_at(near, &local, &held));
            };
            let s = (s0 + s1) / 2.0;
            let (u, v, z) = (p.0 + (q.0 - p.0) * s, p.1 + (q.1 - p.1) * s, p.2 + (q.2 - p.2) * s);
            Some(held(u, v, z))
        }
        Shape::Box { at, size, .. } => {
            let (u, v, z) = local([at[0], at[1], at[2] + size[2] / 2.0]);
            Some(held(u, v, z))
        }
        _ => None,
    }
}

/// Where the clash was found, held inside the wall.
fn held_at(near: P3, local: &dyn Fn(P3) -> (f64, f64, f64), held: &dyn Fn(f64, f64, f64) -> P3) -> P3 {
    let (u, v, z) = local(near);
    held(u, v, z)
}

/// `with_walls_coming_down`: a wall that is to demolish is tested like any other (the
/// question of what is in its way); otherwise it is left out, as call 148 has it.
fn find(job: &mut Job, clearance: Option<f64>, with_walls_coming_down: bool) -> Result<Clashes, String> {
    if let Some(c) = clearance {
        if !(c.is_finite() && c > 0.0) {
            return Err("a clearance is a length of more than nothing".into());
        }
        if c > MAX_CLEARANCE_FT {
            return Err(format!("a clearance of more than {MAX_CLEARANCE_FT} ft is every two things in a room: ask for less"));
        }
    }
    let (drawn, not_drawn) = placements(job);
    let what = what_of(job);
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
    let carried = carried_on(&drawn, &net);
    // What is to be demolished is in nothing's way once the work is done (call 148).
    let gone = crate::components::demolished(&mut job.world);
    for (i, p) in drawn.iter().enumerate() {
        if gone.contains(&p.path) && !(with_walls_coming_down && p.kind == Kind::Wall) {
            left_out.push((p.path.clone(), "it is to demolish, and in nothing's way once the work is done".into()));
            continue;
        }
        match pieces_of(p, i) {
            Ok(mut its) => {
                if let (Some(&(a, b)), Shape::Run { round, rect, .. }) = (carried.get(&i), &p.shape) {
                    let whole = Placement { shape: Shape::Run { a, b, round: *round, rect: *rect }, ..p.clone() };
                    if let (Some(piece), Ok(mut all)) = (its.first_mut(), pieces_of(&whole, i)) {
                        piece.whole = all.pop().map(Box::new);
                    }
                }
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
                    // (A run that carries on past this piece: where it is deepest in the
                    // whole length, not the middle of what is in this piece.)
                    Some(d) if first.whole.is_some() || second.whole.is_some() || !(inside(first, middle) && inside(second, middle)) => [d.at.x, d.at.y, d.at.z],
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
            if (built_in(x.kind) && built_in(y.kind)) || bears(x.kind, y.kind) || joined.contains_key(&pair(&x.path, &y.path)) {
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
        // A wall or a floor is cut for a run to pass. A footing is not: a pipe in a
        // footing is a clash to be settled (a sleeve, or the footing stepped), and is
        // listed as one, not as a hole to cut (the twentieth judgment found it under
        // "Through a wall or a floor").
        let cut_for_a_run = |k: Kind| matches!(k, Kind::Wall | Kind::Floor);
        (of_a_run(x.kind) && cut_for_a_run(y.kind)) || (of_a_run(y.kind) && cut_for_a_run(x.kind))
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
        // A hole is where the run passes through the wall or the floor: where its whole line, carried
        // on through the pieces it is split into, crosses the wall's middle or the floor's middle (call
        // 179, C4). The deepest point of one piece was off the wall once a run was split.
        let at = if through && met.hard {
            let (run, wall, run_at) = if matches!(x.kind, Kind::Wall | Kind::Floor) { (y, x, j) } else { (x, y, i) };
            match &run.shape {
                Shape::Run { a, b, .. } => {
                    let (a, b) = carried.get(&run_at).copied().unwrap_or((*a, *b));
                    cross_at(a, b, wall).unwrap_or(met.at)
                }
                _ => met.at,
            }
        } else {
            met.at
        };
        let laid_in = through && met.hard && {
            let (run, wall, run_at) = if matches!(x.kind, Kind::Wall | Kind::Floor) { (y, x, j) } else { (x, y, i) };
            match &run.shape {
                Shape::Run { a, b, .. } => {
                    let (a, b) = carried.get(&run_at).copied().unwrap_or((*a, *b));
                    lies_in(a, b, wall)
                }
                _ => false,
            }
        };
        found.push(Clash { a: x.path.clone(), b: y.path.clone(), a_name: said(x, &what), b_name: said(y, &what), a_kind: x.kind, b_kind: y.kind, how, through, laid_in, set_against: false, hole: None, by, at, status: x.status.max(y.status) });
    }
    // Set against (call 175). A placed thing and a wall in each other by no more than two
    // inches: a thing that stands against a wall, or hangs on it, or is set in it (the
    // wall's own list of what is set against it, `things_against`, says which things). A run
    // and the thing or run it is joined to (the job's network, `joined`), in each other by
    // no more than that, at that joint. The shapes are what they were; only the name changes.
    let mut against: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in found.iter_mut() {
        if c.how != ClashKind::Hard || c.through || c.by.is_none_or(|by| by > SET_AGAINST_FT + 1.0e-9) {
            continue;
        }
        let is_thing = |k: Kind| matches!(k, Kind::Furnishing | Kind::Equipment | Kind::Fixture);
        let (thing, wall) = match (c.a_kind, c.b_kind) {
            (k, Kind::Wall) if is_thing(k) => (Some(&c.a), Some(&c.b)),
            (Kind::Wall, k) if is_thing(k) => (Some(&c.b), Some(&c.a)),
            _ => (None, None),
        };
        if let (Some(thing), Some(wall)) = (thing, wall) {
            let list = against.entry(wall.clone()).or_insert_with(|| super::things_against(job, wall).unwrap_or_default().into_iter().map(|(p, _)| p).collect());
            if list.contains(thing) {
                c.set_against = true;
                continue;
            }
        }
        if let Some(places) = joined.get(&pair(&c.a, &c.b)) {
            if places.iter().any(|p| ((p[0] - c.at[0]).powi(2) + (p[1] - c.at[1]).powi(2) + (p[2] - c.at[2]).powi(2)).sqrt() <= JOINT_REACH_FT) {
                c.set_against = true;
            }
        }
    }
    // Things in each other first, then holes to cut, then what is only close.
    found.sort_by_key(|c| (c.how, c.through && c.how == ClashKind::Hard));
    // A hole is counted once (call 175): the pieces of a run that pass through one wall or
    // floor, joined end to end (or to a fitting on it), are one hole to cut.
    let mut holes: Vec<Hole> = Vec::new();
    let mut hole_of: Vec<Option<usize>> = vec![None; found.len()];
    let sides = |c: &Clash| if matches!(c.b_kind, Kind::Wall | Kind::Floor) { (c.a.clone(), c.b.clone()) } else { (c.b.clone(), c.a.clone()) };
    let cut: Vec<usize> = (0..found.len()).filter(|&i| found[i].how == ClashKind::Hard && found[i].through && !found[i].laid_in).collect();
    // Which pieces are one run: the same piece, or two joined, through the same wall or floor.
    let mut root: Vec<usize> = (0..cut.len()).collect();
    fn top(root: &mut [usize], mut i: usize) -> usize {
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    for x in 0..cut.len() {
        for y in 0..x {
            let ((run_x, whole_x), (run_y, whole_y)) = (sides(&found[cut[x]]), sides(&found[cut[y]]));
            // One hole holds pieces of one run that pass through at one place: joined, and no
            // further apart where they pass than their widths and a little over (a branch that
            // joins a stack a foot away has a hole of its own).
            let across = |path: &str| drawn.iter().find(|p| p.path == path).map(|p| match &p.shape {
                Shape::Run { round: Some(d), .. } => *d,
                Shape::Run { rect: Some((w, h)), .. } => w.max(*h),
                Shape::Box { size, .. } => size[0].max(size[1]),
                _ => 0.0,
            }).unwrap_or(0.0);
            let (px, py) = (found[cut[x]].at, found[cut[y]].at);
            let near = (px[0] - py[0]).hypot(px[1] - py[1]) <= (across(&run_x) + across(&run_y)) / 2.0 + 0.25;
            if whole_x == whole_y && near && (run_x == run_y || joined.contains_key(&pair(&run_x, &run_y))) {
                let (rx, ry) = (top(&mut root, x), top(&mut root, y));
                root[rx.max(ry)] = rx.min(ry);
            }
        }
    }
    let mut hole_at_root: BTreeMap<usize, usize> = BTreeMap::new();
    for x in 0..cut.len() {
        let i = cut[x];
        let (run, whole) = sides(&found[i]);
        let at = match hole_at_root.get(&top(&mut root, x)) {
            Some(&h) => h,
            None => {
                let on_b = found[i].b == whole;
                holes.push(Hole { through: whole, through_name: if on_b { found[i].b_name.clone() } else { found[i].a_name.clone() }, through_kind: if on_b { found[i].b_kind } else { found[i].a_kind }, pieces: Vec::new(), size: String::new(), size_said: String::new(), at: found[i].at, status: found[i].status });
                hole_at_root.insert(top(&mut root, x), holes.len() - 1);
                holes.len() - 1
            }
        };
        hole_of[i] = Some(at);
        let name = if found[i].a == run { found[i].a_name.clone() } else { found[i].b_name.clone() };
        holes[at].pieces.push((run, name));
        holes[at].status = holes[at].status.max(found[i].status);
    }
    for (c, h) in found.iter_mut().zip(&hole_of) {
        c.hole = *h;
    }
    // Each hole's size: the widest run through it, as it is said.
    let as_said = |inches: f64| view::figure_shown(inches, 2);
    for hole in holes.iter_mut() {
        let mut widest: Option<(f64, String, String)> = None;
        for (path, _) in &hole.pieces {
            let Some(p) = drawn.iter().find(|p| &p.path == path) else { continue };
            if let Shape::Run { round, rect, .. } = &p.shape {
                let (across, text, said) = match (round, rect) {
                    (Some(d), _) => (*d, format!("{} in round", as_said(d * 12.0)), format!("{} round", units::across_said(*d))),
                    (None, Some((w, h))) => (w.max(*h), format!("{} in by {} in", as_said(w * 12.0), as_said(h * 12.0)), format!("{} by {}", units::across_said(*w), units::across_said(*h))),
                    _ => continue,
                };
                if widest.as_ref().is_none_or(|(was, _, _)| across > *was) {
                    widest = Some((across, text, said));
                }
            }
        }
        let (text, said) = widest.map(|(_, t, s)| (t, s)).unwrap_or_default();
        hole.size = text;
        hole.size_said = said;
    }
    Ok(Clashes { found, holes, tested, left_out })
}
