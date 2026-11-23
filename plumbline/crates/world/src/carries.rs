//! What a wall carries and what it stands on (call 176): worked from the plan lines of what
//! is drawn on the level above and the level below, and said to be only that.
//!
//! Ours. Nothing here is sized and nothing is found structurally: the program says what is
//! drawn over a line and what is drawn under it, and that this is all it is. No load, no
//! span table, no "safe". Nothing is saved: the figures are worked from the job each time
//! they are asked for, so a job saved before opens and saves the same.
//!
//! The rules, each the call's:
//!
//! * **What stands on a wall**: each wall of the level above whose line lies over this
//!   wall's (its centre line within this wall's thickness, and within 1 degree of its
//!   direction), with the length they share; each eave of a roof plane that lies over it
//!   the same way (a rake does not bear on the wall under it and is not listed); and the
//!   floor above by its joists, where the floor's outline holds the wall's line across
//!   its inside (a wall along an edge of the floor carries the floor's edge whichever way
//!   the joists run, and says nothing of them): if the joists' way is said and they cross
//!   the wall (not within 1 degree of its direction), they cross it; if they run alongside
//!   it, nothing; if which way they run is not said, that is said.
//! * **What a wall stands on**: each wall of the level below under its line, each footing
//!   of its own level under it, each beam under it (the beam's top within a foot of the
//!   wall's base), each with the length they share; and the length where none is found.
//!   A wall of the level below marked to demolish is listed and is not counted: once it is
//!   down it is not there.
//!
//! Choices the call left open (DECISIONS.md call 176, as built): the level above is the
//! storey whose elevation is next above, and the level below the next under; an eave is
//! over a wall only where it is within two feet of the wall's top (so the eaves of a
//! second-floor roof are not on a first-floor wall); a beam is under a wall within a foot
//! of its base, whichever level it is drawn on; a beam's height is the mean of its two ends.

use std::collections::BTreeMap;

use crate::building::{self, plan_wall, Floor, JoistsSpan, Opening, Segment, Tree, Wall};
use crate::components::*;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::Status;
use crate::view;

/// What is said wherever these figures are shown.
pub const SAID: &str = "worked from what is drawn above and below this line; it is not an engineer's finding";

/// The sine of 1 degree: walls within it run the same way.
const SAME_WAY: f64 = 0.017_452_406_437_283_512;
const TOL: f64 = crate::check::FIT_TOLERANCE_FT;
/// How far from a wall's top an eave's height may be for the eave to be over that wall, in feet.
pub const EAVE_REACH_FT: f64 = 2.0;
/// How far below a wall's base a beam's top may be for the beam to be under the floor the wall stands on, in feet
/// (call 180, A2: it was a foot, either way). A beam through the floor, above the base, is not under it.
pub const BEAM_REACH_FT: f64 = 2.0;
/// How far clear of a lower wall's face an upper wall's line may be and still be "beside it, not on it", in feet
/// (call 180, A1; the call names no distance, and 12 in on 6 in walls reads beside).
pub const BESIDE_FT: f64 = 1.0;
/// The sine of 10 degrees: a wall within it of the way another runs can be beside it.
const BESIDE_ANGLE: f64 = 0.173_648_177_666_930_3;
/// How far outside a wall a floor's edge may be and go with it, in feet (call 180, B1).
pub const FLOOR_REACH_FT: f64 = 2.0;
/// How far outside a wall a roof's eave may be and go with it, in feet (call 180, B1).
pub const ROOF_REACH_FT: f64 = 4.0;

/// What stands on a wall.
#[derive(Clone, Debug, PartialEq)]
pub struct On {
    /// A wall, or the roof's eave.
    pub kind: Kind,
    pub path: String,
    pub name: String,
    /// The length they share, in feet along this wall.
    pub shared: f64,
    /// It is marked to demolish: it is listed and is not counted as standing there.
    pub comes_down: bool,
    /// How much of it would have nothing under it if this wall were taken out: what is
    /// drawn under it besides this wall is not counted out.
    pub left: f64,
    /// It is beside this wall, not on it (call 180, A1): said, and not counted as standing here.
    pub beside: bool,
    /// How many levels up it is: 1 is the next level, 2 has nothing drawn between (call 180, A3).
    pub up: usize,
}

/// What a wall stands on.
#[derive(Clone, Debug, PartialEq)]
pub struct Under {
    /// A wall, a footing or a beam.
    pub kind: Kind,
    pub path: String,
    pub name: String,
    pub shared: f64,
    /// It is marked to demolish: it is listed and is not counted.
    pub comes_down: bool,
    /// It is beside this wall, not under it (call 180, A1): said, and not counted as under it.
    pub beside: bool,
    /// How many levels down it is: 1 is the next level, 2 has nothing drawn between (call 180, A3).
    pub down: usize,
}

/// How the joists of the floor above lie to a wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoistWay {
    /// Which way they run is not said.
    NotSaid,
    /// They run across it.
    Cross,
    /// They run alongside it.
    Alongside,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Joists {
    pub way: JoistWay,
    /// The floor, by name.
    pub floor: String,
    /// How much of the wall lies across the inside of that floor, in feet.
    pub inside: f64,
    /// For `Cross`: how much of that would have nothing under it were this wall taken out.
    pub left: f64,
}

/// What a wall carries and stands on.
#[derive(Clone, Debug, PartialEq)]
pub struct Carry {
    pub path: String,
    pub name: String,
    /// The level it is on, by path and by name.
    pub storey: String,
    pub storey_name: String,
    /// Its length on the plan, in feet.
    pub length: f64,
    pub carries: Vec<On>,
    pub joists: Option<Joists>,
    pub stands_on: Vec<Under>,
    /// The length that nothing found is under, in feet.
    pub nothing_under: f64,
    /// There is no level below this one.
    pub lowest: bool,
    /// What was said of it: bearing or not, and how sure that is.
    pub bearing: Option<(Bearing, Status)>,
    /// How sure the figures are: the worst of the positions they were worked from.
    pub status: Status,
    /// The things whose positions are not Known and the figures rest on, by name.
    pub rests_on: Vec<String>,
    /// The roofs that are over this wall by the plan and have no eave on it: whether one bears here is not worked.
    pub roofs_over: Vec<String>,
}

fn fig(x: f64) -> String {
    format!("{} LF", view::figure(x, 2))
}

fn levels_word(n: usize) -> String {
    match n {
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        n => n.to_string(),
    }
}

impl Carry {
    /// What stands on it, one sentence a thing, in the words the screens use.
    pub fn carries_said(&self) -> Vec<String> {
        let mut out = Vec::new();
        for o in &self.carries {
            let what = if o.kind == Kind::Eave { "the roof's eave".to_string() } else { format!("{} [{}]", o.name, o.path) };
            if o.beside {
                out.push(format!("{what} is beside it, not on it, for {}", fig(o.shared)));
            } else if o.up > 1 {
                out.push(if o.comes_down { format!("{what}, {} levels up, lies over this line with nothing drawn between them, for {}, which comes down: not counted", levels_word(o.up), fig(o.shared)) } else { format!("{what}, {} levels up, lies over this line with nothing drawn between them, for {}", levels_word(o.up), fig(o.shared)) });
            } else {
                out.push(if o.comes_down { format!("{what} for {}, which comes down: not counted", fig(o.shared)) } else { format!("{what} for {}", fig(o.shared)) });
            }
        }
        match &self.joists {
            Some(Joists { way: JoistWay::Cross, .. }) => out.push("the joists of the floor above cross it".to_string()),
            Some(Joists { way: JoistWay::NotSaid, .. }) => out.push("which way the joists of the floor above run is not said".to_string()),
            _ => {}
        }
        if !self.roofs_over.is_empty() {
            out.push("a roof is over this wall: whether it bears here is not worked (only eaves are)".to_string());
        }
        if out.is_empty() {
            out.push("nothing found standing on it".to_string());
        }
        out
    }

    /// What it stands on, one sentence a thing, and what is not found under it.
    pub fn stands_on_said(&self) -> Vec<String> {
        let mut out = Vec::new();
        for u in &self.stands_on {
            let what = format!("{} [{}] ({})", u.name, u.path, u.kind.said());
            if u.beside {
                out.push(format!("{what} is beside it, not on it, for {}", fig(u.shared)));
            } else if u.down > 1 {
                out.push(if u.comes_down { format!("{what}, {} levels down, is under this line with nothing drawn between them, for {}, which comes down: not counted", levels_word(u.down), fig(u.shared)) } else { format!("{what}, {} levels down, is under this line with nothing drawn between them, for {}", levels_word(u.down), fig(u.shared)) });
            } else {
                out.push(if u.comes_down { format!("{what} for {}, which comes down: not counted", fig(u.shared)) } else { format!("{what} for {}", fig(u.shared)) });
            }
        }
        if self.nothing_under > TOL {
            out.push(if self.lowest { format!("nothing found under it for {} (no footing)", fig(self.nothing_under)) } else { format!("nothing found under it for {}: it stands on the floor alone there, which is ordinary for a partition", fig(self.nothing_under)) });
        }
        if out.is_empty() {
            out.push("nothing is drawn under it, and it has no length".to_string());
        }
        out
    }

    /// How sure the figures are, in a sentence.
    pub fn status_said(&self) -> String {
        if self.rests_on.is_empty() {
            "Known: every position it was worked from is Known".to_string()
        } else {
            format!("{}: it rests on positions that are not Known: {}", crate::structure::word(self.status), self.rests_on.join(", "))
        }
    }

    /// Whether anything is found standing on it that does not come down.
    pub fn is_carrying(&self) -> bool {
        self.carries.iter().any(|o| !o.comes_down && !o.beside) || matches!(&self.joists, Some(j) if j.way == JoistWay::Cross)
    }
}

// ---------------------------------------------------------------- what is drawn

struct WallRec {
    path: String,
    name: String,
    storey: String,
    line: [P2; 2],
    thick: f64,
    base: f64,
    top: f64,
    status: Status,
    bearing: Option<(Bearing, Status)>,
    comes_down: bool,
}

struct FootRec {
    path: String,
    name: String,
    storey: String,
    line: [P2; 2],
    status: Status,
}

struct BeamRec {
    path: String,
    name: String,
    line: [P2; 2],
    top: f64,
    status: Status,
}

struct EaveRec {
    path: String,
    line: [P2; 2],
    /// The outline of the roof it is an eave of.
    ring: Vec<P2>,
    status: Status,
}

struct RoofRec {
    path: String,
    ring: Vec<P2>,
}

struct FloorRec {
    path: String,
    name: String,
    storey: String,
    ring: Vec<P2>,
    /// Which way the joists run (a vector), where it is said, and how sure that is.
    run: Option<(P2, Status)>,
    status: Status,
}

struct Drawn {
    /// Each storey that has an elevation, lowest first: elevation, path, name.
    storeys: Vec<(f64, String, String)>,
    walls: Vec<WallRec>,
    footings: Vec<FootRec>,
    beams: Vec<BeamRec>,
    eaves: Vec<EaveRec>,
    roofs: Vec<RoofRec>,
    floors: Vec<FloorRec>,
    bodies: Vec<Body>,
}

fn number(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

fn gather(job: &mut Job) -> Drawn {
    let tree = Tree::of(&mut job.world);
    let mut storeys: Vec<(f64, String, String)> = tree.levels.iter().filter_map(|(p, l)| l.elevation.feet().map(|e| (e, p.clone(), tree.called(p)))).collect();
    storeys.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let work: BTreeMap<String, WorkStatus> = {
        let mut q = job.world.query::<(&Node, &Work)>();
        q.iter(&job.world).filter_map(|(n, w)| w.status.value.map(|s| (n.path.clone(), s))).collect()
    };
    let hidden: BTreeMap<String, (Bearing, Status)> = {
        let mut q = job.world.query::<(&Node, &Hidden)>();
        q.iter(&job.world).filter_map(|(n, h)| h.bearing.as_ref().and_then(|b| b.value.map(|v| (n.path.clone(), (v, b.status))))).collect()
    };
    let mut walls = Vec::new();
    let mut footings = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Wall)>();
        let mut all: Vec<(String, Kind, Wall)> = q.iter(&job.world).map(|(n, k, w)| (n.path.clone(), *k, w.clone())).collect();
        all.sort_by_key(|(p, _, _)| number(p));
        for (path, kind, w) in all {
            let Some(storey) = tree.storey_of(&path).map(str::to_string) else { continue };
            match kind {
                Kind::Wall => {
                    let Ok((placed, status, _, _)) = plan_wall(&tree, &path, &w) else { continue };
                    walls.push(WallRec {
                        name: tree.called(&path),
                        storey,
                        line: [[placed.start.0, placed.start.1], [placed.end.0, placed.end.1]],
                        thick: placed.ext - placed.int,
                        base: placed.base,
                        top: placed.top,
                        status,
                        bearing: hidden.get(&path).copied(),
                        comes_down: work.get(&path) == Some(&WorkStatus::Demolish),
                        path,
                    });
                }
                Kind::Footing => {
                    if let Some(line) = w.line_ft() {
                        footings.push(FootRec { name: tree.called(&path), storey, line, status: w.line.status, path });
                    }
                }
                _ => {}
            }
        }
    }
    let mut beams = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Segment)>();
        let mut all: Vec<(String, Segment)> = q.iter(&job.world).filter(|(_, k, _)| **k == Kind::Beam).map(|(n, _, s)| (n.path.clone(), s.clone())).collect();
        all.sort_by_key(|(p, _)| number(p));
        for (path, s) in all {
            let (Some(line), Some((first, second)), Some(storey)) = (s.line_ft(), s.offsets_ft(), tree.storey_of(&path)) else { continue };
            let Some(elevation) = tree.levels.get(storey).and_then(|l| l.elevation.feet()) else { continue };
            let status = s.line.status.max(s.offset.status).max(s.end_offset.as_ref().map(|e| e.status).unwrap_or(Status::Known));
            beams.push(BeamRec { name: tree.called(&path), line, top: elevation + (first + second) / 2.0, status, path });
        }
    }
    let mut roofs: Vec<RoofRec> = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Footprint)>();
        let mut all: Vec<(String, Vec<P2>)> = q.iter(&job.world).filter_map(|(n, f)| Some((n.path.clone(), f.points_ft()?))).collect();
        all.sort_by_key(|(p, _)| number(p));
        roofs.extend(all.into_iter().map(|(path, ring)| RoofRec { path, ring }));
    }
    let mut eaves = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Run)>();
        let mut all: Vec<(String, Run)> = q.iter(&job.world).filter(|(_, k, _)| **k == Kind::Eave).map(|(n, _, r)| (n.path.clone(), r.clone())).collect();
        all.sort_by_key(|(p, _)| number(p));
        for (path, r) in all {
            let Some(line) = r.ends_ft() else { continue };
            // Its roof, by its plane: a roof is over a wall by the plan alone, so the eave's height is not asked (call 180, A2).
            let plane = r.planes.first().and_then(|p| job.find(p));
            let roof = plane.and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone());
            let Some(ring) = roof.and_then(|r| roofs.iter().find(|x| x.path == r)).map(|x| x.ring.clone()) else { continue };
            let height_status = plane.and_then(|e| job.world.get::<Outline>(e)).map(|o| o.eave_height.status).unwrap_or(Status::Unknown);
            eaves.push(EaveRec { path, line, ring, status: r.ends.status.max(height_status) });
        }
    }
    let mut floors = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Floor, Option<&JoistsSpan>)>();
        let mut all: Vec<(String, Floor, Option<JoistsSpan>)> = q.iter(&job.world).filter(|(_, k, _, _)| **k == Kind::Floor).map(|(n, _, f, j)| (n.path.clone(), f.clone(), j.cloned())).collect();
        all.sort_by_key(|(p, _, _)| number(p));
        for (path, f, span) in all {
            let (Some(ring), Some(storey)) = (f.boundary_ft(), tree.storey_of(&path)) else { continue };
            let run = building::joists_run(&ring, span.as_ref()).and_then(|(run, said)| said.map(|(_, status)| (run, status)));
            floors.push(FloorRec { name: tree.called(&path), path: path.clone(), storey: storey.to_string(), ring, run, status: f.boundary.status.max(span.as_ref().map(|s| s.0.status).unwrap_or(Status::Known)) });
        }
    }
    let bodies = walls.iter().map(|w| Body { path: w.path.clone(), storey: w.storey.clone(), line: w.line, thick: w.thick, base: w.base, top: w.top }).collect();
    Drawn { storeys, walls, footings, beams, eaves, roofs, floors, bodies }
}

impl Drawn {
    /// A wall as another wall's figures name it: its name and its level, since every level
    /// has a "wall 1".
    fn called(&self, w: &WallRec) -> String {
        let level = self.storeys.iter().find(|s| s.1 == w.storey).map(|s| s.2.as_str()).unwrap_or("");
        if level.is_empty() { w.name.clone() } else { format!("{} on {level}", w.name) }
    }
    fn above(&self, storey: &str) -> Option<&str> {
        let at = self.storeys.iter().position(|s| s.1 == storey)?;
        self.storeys[at + 1..].iter().find(|s| s.0 > self.storeys[at].0 + TOL).map(|s| s.1.as_str())
    }
    fn below(&self, storey: &str) -> Option<&str> {
        let at = self.storeys.iter().position(|s| s.1 == storey)?;
        self.storeys[..at].iter().rev().find(|s| s.0 < self.storeys[at].0 - TOL).map(|s| s.1.as_str())
    }
    /// The levels above `storey`, nearest first, each with how many levels up it is (levels at one
    /// height are one level).
    fn levels_above(&self, storey: &str) -> Vec<(String, usize)> {
        let Some(at) = self.storeys.iter().position(|s| s.1 == storey) else { return Vec::new() };
        let (mut out, mut up, mut last) = (Vec::new(), 0, self.storeys[at].0);
        for s in &self.storeys[at + 1..] {
            if s.0 <= self.storeys[at].0 + TOL {
                continue;
            }
            if s.0 > last + TOL {
                up += 1;
                last = s.0;
            }
            out.push((s.1.clone(), up));
        }
        out
    }
    /// The levels below `storey`, nearest first, each with how many levels down it is.
    fn levels_below(&self, storey: &str) -> Vec<(String, usize)> {
        let Some(at) = self.storeys.iter().position(|s| s.1 == storey) else { return Vec::new() };
        let (mut out, mut down, mut last) = (Vec::new(), 0, self.storeys[at].0);
        for s in self.storeys[..at].iter().rev() {
            if s.0 >= self.storeys[at].0 - TOL {
                continue;
            }
            if s.0 < last - TOL {
                down += 1;
                last = s.0;
            }
            out.push((s.1.clone(), down));
        }
        out
    }
}

// ---------------------------------------------------------------- geometry along a wall

type Span = (f64, f64);

/// How far along `wall` (a line) the line `other` lies over it: where it runs within 1
/// degree of the wall's direction and each of its ends is within `thick` of the wall's
/// line. The stretch is in feet from the wall's first end.
pub(crate) fn over(wall: [P2; 2], thick: f64, other: [P2; 2]) -> Option<Span> {
    let (dx, dy) = (wall[1][0] - wall[0][0], wall[1][1] - wall[0][1]);
    let long = dx.hypot(dy);
    let (ox, oy) = (other[1][0] - other[0][0], other[1][1] - other[0][1]);
    let other_long = ox.hypot(oy);
    if long <= TOL || other_long <= TOL {
        return None;
    }
    let (ux, uy) = (dx / long, dy / long);
    if (ux * oy - uy * ox).abs() / other_long > SAME_WAY {
        return None;
    }
    let along = |p: P2| ux * (p[0] - wall[0][0]) + uy * (p[1] - wall[0][1]);
    let across = |p: P2| (ux * (p[1] - wall[0][1]) - uy * (p[0] - wall[0][0])).abs();
    if across(other[0]).max(across(other[1])) > thick + TOL {
        return None;
    }
    let (a, b) = (along(other[0]).min(along(other[1])).max(0.0), along(other[0]).max(along(other[1])).min(long));
    (b - a > TOL).then_some((a, b))
}

/// The values of s in `range` for which `lo <= a0 + a1 s <= hi`, as a stretch (call 180, A1).
fn lin(a0: f64, a1: f64, lo: f64, hi: f64, range: Span) -> Option<Span> {
    let (mut l, mut h) = range;
    if a1.abs() < 1.0e-12 {
        if a0 < lo - 1.0e-9 || a0 > hi + 1.0e-9 {
            return None;
        }
    } else {
        let (x, y) = ((lo - a0) / a1, (hi - a0) / a1);
        let (x, y) = if x <= y { (x, y) } else { (y, x) };
        l = l.max(x);
        h = h.min(y);
    }
    (h - l > 1.0e-9).then_some((l, h))
}

/// Where the wall `up` (a line and a thickness) stands on the wall `down`, and how much of it is beside it (call
/// 180, A1): worked from the two bodies on the plan, at any angle. A point of `up`'s line stands on `down` while at
/// least half of `up`'s thickness lies over `down`'s body: with the two bodies' widths `t_up` and `t_down` that is
/// while the line is within `t_down / 2` of `down`'s line (and `t_down` is at least half of `t_up`), and over
/// `down`'s length. The stretch is in feet along `up` from its first end. The second figure is the length of `up`
/// that is not on `down` but is beside it: within `BESIDE_FT` of its face, where the walls run within 10 degrees of
/// the same way, over `down`'s length.
pub(crate) fn bearing(up: [P2; 2], t_up: f64, down: [P2; 2], t_down: f64) -> (Option<Span>, f64) {
    let (lu, ld) = (length(up), length(down));
    if lu <= TOL || ld <= TOL {
        return (None, 0.0);
    }
    let (u, v) = ([(up[1][0] - up[0][0]) / lu, (up[1][1] - up[0][1]) / lu], [(down[1][0] - down[0][0]) / ld, (down[1][1] - down[0][1]) / ld]);
    let from = [up[0][0] - down[0][0], up[0][1] - down[0][1]];
    let (d0, d1) = (v[0] * from[1] - v[1] * from[0], v[0] * u[1] - v[1] * u[0]);
    let (q0, q1) = (from[0] * v[0] + from[1] * v[1], u[0] * v[0] + u[1] * v[1]);
    // Where the walls run across each other, the upper wall's body only grazes the end of a lower wall at a corner: half its
    // thickness lies over the lower body there exactly, and it is not standing on it. The lower wall's length is
    // taken less half the upper's thickness at each end (a joint is not a bearing).
    let across = (u[0] * v[1] - u[1] * v[0]).abs() > BESIDE_ANGLE;
    let (qlo, qhi) = if across { (t_up / 2.0, ld - t_up / 2.0) } else { (0.0, ld) };
    if qhi <= qlo {
        return (None, 0.0);
    }
    let Some(range) = lin(q0, q1, qlo, qhi, (0.0, lu)) else { return (None, 0.0) };
    let a = t_down / 2.0 + 1.0e-7;
    // Across the lower wall the upper one stands on it only where it passes over the whole of it; a wall that
    // ends on the lower wall's line is a joint (half of the lower wall's thickness lies under its end).
    let stands = if t_down + TOL >= t_up / 2.0 { lin(d0, d1, -a, a, range).filter(|s| s.1 - s.0 > TOL && (!across || s.1 - s.0 >= t_down - 1.0e-6)) } else { None };
    let mut beside = 0.0;
    if (u[0] * v[1] - u[1] * v[0]).abs() <= BESIDE_ANGLE {
        for (lo, hi) in [(a, a + BESIDE_FT), (-a - BESIDE_FT, -a)] {
            if let Some(s) = lin(d0, d1, lo, hi, range) {
                beside += s.1 - s.0;
            }
        }
    }
    (stands, if beside > TOL { beside } else { 0.0 })
}

/// A stretch of wall `other`, given along `other`, as a stretch along `wall`, clipped to it.
fn along_wall(wall: [P2; 2], other: [P2; 2], span: Span) -> Option<Span> {
    let (lw, lo) = (length(wall), length(other));
    if lw <= TOL || lo <= TOL {
        return None;
    }
    let (w, o) = ([(wall[1][0] - wall[0][0]) / lw, (wall[1][1] - wall[0][1]) / lw], [(other[1][0] - other[0][0]) / lo, (other[1][1] - other[0][1]) / lo]);
    let at = |s: f64| ((other[0][0] + o[0] * s) - wall[0][0]) * w[0] + ((other[0][1] + o[1] * s) - wall[0][1]) * w[1];
    let (a, b) = (at(span.0).min(at(span.1)).max(0.0), at(span.0).max(at(span.1)).min(lw));
    (b - a > TOL).then_some((a, b))
}

fn merged(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<Span> = Vec::new();
    for s in spans {
        match out.last_mut() {
            Some(last) if s.0 <= last.1 + 1.0e-9 => last.1 = last.1.max(s.1),
            _ => out.push(s),
        }
    }
    out
}

fn length_of(spans: &[Span]) -> f64 {
    merged(spans.to_vec()).iter().map(|s| s.1 - s.0).sum()
}

/// How much of `span` lies outside every one of `by`.
fn left_of(span: Span, by: &[Span]) -> f64 {
    let covered: f64 = merged(by.to_vec()).iter().map(|b| (b.1.min(span.1) - b.0.max(span.0)).max(0.0)).sum();
    ((span.1 - span.0) - covered).max(0.0)
}

fn length(line: [P2; 2]) -> f64 {
    (line[1][0] - line[0][0]).hypot(line[1][1] - line[0][1])
}

/// The stretches of `wall` that lie across the inside of a floor's outline, leaving out
/// those along an edge of it (within the wall's thickness, and within 1 degree).
fn inside_floor(wall: [P2; 2], thick: f64, ring: &[P2]) -> Vec<Span> {
    let long = length(wall);
    if long <= TOL || ring.len() < 3 {
        return Vec::new();
    }
    let (dx, dy) = (wall[1][0] - wall[0][0], wall[1][1] - wall[0][1]);
    let mut at = vec![0.0, 1.0];
    for i in 0..ring.len() {
        let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
        let (ex, ey) = (q[0] - p[0], q[1] - p[1]);
        let den = dx * ey - dy * ex;
        if den.abs() < 1.0e-12 {
            continue;
        }
        let t = ((p[0] - wall[0][0]) * ey - (p[1] - wall[0][1]) * ex) / den;
        let s = ((p[0] - wall[0][0]) * dy - (p[1] - wall[0][1]) * dx) / den;
        if t > 1.0e-9 && t < 1.0 - 1.0e-9 && (-1.0e-9..=1.0 + 1.0e-9).contains(&s) {
            at.push(t);
        }
    }
    at.sort_by(f64::total_cmp);
    let mut out = Vec::new();
    for pair in at.windows(2) {
        if pair[1] - pair[0] < 1.0e-9 {
            continue;
        }
        let mid = [wall[0][0] + dx * (pair[0] + pair[1]) / 2.0, wall[0][1] + dy * (pair[0] + pair[1]) / 2.0];
        if !crate::vif::inside(ring, mid) {
            continue;
        }
        // Along an edge: an edge it runs with, within its thickness.
        let along_edge = (0..ring.len()).any(|i| {
            let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
            let (ex, ey) = (q[0] - p[0], q[1] - p[1]);
            let el = ex.hypot(ey);
            if el <= TOL || (dx * ey - dy * ex).abs() / (long * el) > SAME_WAY {
                return false;
            }
            let across = ((ex * (mid[1] - p[1]) - ey * (mid[0] - p[0])) / el).abs();
            let along = ((mid[0] - p[0]) * ex + (mid[1] - p[1]) * ey) / el;
            across <= thick + TOL && along >= -TOL && along <= el + TOL
        });
        if !along_edge {
            out.push((pair[0] * long, pair[1] * long));
        }
    }
    merged(out)
}

// ---------------------------------------------------------------- the figures

/// What lies over a wall from the levels above, nearest level first (call 180, A1 and A3): each wall that
/// stands on it, with the stretch of that wall that does, and each that is beside it. The search stops at the
/// first level that has a wall standing on it: what is above that stands on that.
struct Hit<'a> {
    wall: &'a WallRec,
    span: Option<Span>,
    beside: f64,
    levels: usize,
}

fn walls_above<'a>(d: &'a Drawn, w: &WallRec) -> Vec<Hit<'a>> {
    let mut out = Vec::new();
    for (storey, up) in d.levels_above(&w.storey) {
        let here: Vec<Hit> = d.walls.iter().filter(|u| u.storey == storey).filter_map(|u| {
            let (span, beside) = bearing(u.line, u.thick, w.line, w.thick);
            (span.is_some() || beside > 0.0).then_some(Hit { wall: u, span, beside, levels: up })
        }).collect();
        let stood = here.iter().any(|h| h.span.is_some());
        out.extend(here);
        if stood {
            break;
        }
    }
    out
}

fn walls_below<'a>(d: &'a Drawn, w: &WallRec, leaving: Option<&str>) -> Vec<Hit<'a>> {
    let mut out = Vec::new();
    for (storey, down) in d.levels_below(&w.storey) {
        let here: Vec<Hit> = d.walls.iter().filter(|o| o.storey == storey && Some(o.path.as_str()) != leaving).filter_map(|o| {
            let (span, beside) = bearing(w.line, w.thick, o.line, o.thick);
            (span.is_some() || beside > 0.0).then_some(Hit { wall: o, span, beside, levels: down })
        }).collect();
        let stood = here.iter().any(|h| h.span.is_some() && !h.wall.comes_down);
        out.extend(here);
        if stood {
            break;
        }
    }
    out
}

/// What stands under `w` and is counted, as stretches along `w`; `leaving` is a wall taken
/// out of the count (the wall it would stand on, were it to come down).
fn supports(d: &Drawn, w: &WallRec, leaving: Option<&str>) -> (Vec<Under>, Vec<Span>) {
    let mut under = Vec::new();
    let mut counted: Vec<Span> = Vec::new();
    for h in walls_below(d, w, leaving) {
        let o = h.wall;
        if let Some(s) = h.span {
            if !o.comes_down {
                counted.push(s);
            }
            under.push(Under { kind: Kind::Wall, path: o.path.clone(), name: d.called(o), shared: s.1 - s.0, comes_down: o.comes_down, beside: false, down: h.levels });
        }
        if h.beside > 0.0 {
            under.push(Under { kind: Kind::Wall, path: o.path.clone(), name: d.called(o), shared: h.beside, comes_down: o.comes_down, beside: true, down: h.levels });
        }
    }
    for f in d.footings.iter().filter(|f| f.storey == w.storey) {
        if let Some(s) = over(w.line, w.thick, f.line) {
            counted.push(s);
            under.push(Under { kind: Kind::Footing, path: f.path.clone(), name: f.name.clone(), shared: s.1 - s.0, comes_down: false, beside: false, down: 1 });
        }
    }
    // A beam is under the floor the wall stands on: its top within 2 ft below the wall's base, and not above it (call 180, A2).
    for b in d.beams.iter().filter(|b| b.top <= w.base + TOL && b.top >= w.base - BEAM_REACH_FT - TOL) {
        if let Some(s) = over(w.line, w.thick, b.line) {
            counted.push(s);
            under.push(Under { kind: Kind::Beam, path: b.path.clone(), name: b.name.clone(), shared: s.1 - s.0, comes_down: false, beside: false, down: 1 });
        }
    }
    (under, merged(counted))
}

/// The stretches of `w` that a beam carries at `w`'s top: what a beam drawn in its line
/// answers, were `w` taken out.
fn beams_at_top(d: &Drawn, w: &WallRec) -> Vec<Span> {
    d.beams.iter().filter(|b| b.top <= w.top + TOL && b.top >= w.top - BEAM_REACH_FT - TOL).filter_map(|b| over(w.line, w.thick, b.line)).collect()
}

fn carry_of(d: &Drawn, w: &WallRec) -> Carry {
    let storey_name = d.storeys.iter().find(|s| s.1 == w.storey).map(|s| s.2.clone()).unwrap_or_else(|| w.storey.clone());
    let mut status = w.status;
    let mut rests: Vec<(String, Status)> = Vec::new();
    let mut rest = |name: &str, s: Status, status: &mut Status| {
        *status = (*status).max(s);
        if s != Status::Known && !rests.iter().any(|r| r.0 == name) {
            rests.push((name.to_string(), s));
        }
    };
    rest(&w.name, w.status, &mut status);
    let mut carries = Vec::new();
    // What stands on the wall above are stretches along the wall above; as stretches along this wall they are:
    let mut above_spans: Vec<Span> = Vec::new();
    for h in walls_above(d, w) {
        let u = h.wall;
        if let Some(span) = h.span {
            // What would have nothing under it if this wall came out: its own length over
            // this wall, less what else is drawn under it.
            let left = left_of(span, &supports(d, u, Some(&w.path)).1);
            rest(&d.called(u), u.status, &mut status);
            if !u.comes_down {
                above_spans.extend(along_wall(w.line, u.line, span));
            }
            carries.push(On { kind: Kind::Wall, path: u.path.clone(), name: d.called(u), shared: span.1 - span.0, comes_down: u.comes_down, left, beside: false, up: h.levels });
        }
        if h.beside > 0.0 {
            carries.push(On { kind: Kind::Wall, path: u.path.clone(), name: d.called(u), shared: h.beside, comes_down: u.comes_down, left: 0.0, beside: true, up: h.levels });
        }
    }
    let answered = beams_at_top(d, w);
    let others: Vec<&Body> = d.bodies.iter().filter(|b| b.path != w.path).collect();
    let me = d.bodies.iter().find(|b| b.path == w.path);
    let mut eave_roofs: Vec<&str> = Vec::new();
    for e in &d.eaves {
        // Over this wall by the plan alone (call 180, A2): the eave along it within its thickness, or outside it by no more
        // than 4 ft with no other wall between, whatever the height of the eave.
        let Some(me) = me else { continue };
        let Some(on) = edge_on(me, e.line, &others, &e.ring) else { continue };
        if !(on.across <= w.thick + TOL || (on.across <= ROOF_REACH_FT + TOL && on.outward)) {
            continue;
        }
        let Some(s) = along_wall(w.line, e.line, (0.0, length(e.line))) else { continue };
        rest("the roof's eave", e.status, &mut status);
        if let Some(roof) = d.roofs.iter().find(|r| r.ring == e.ring) {
            eave_roofs.push(roof.path.as_str());
        }
        let mut covered = answered.clone();
        covered.extend(above_spans.iter().copied());
        carries.push(On { kind: Kind::Eave, path: e.path.clone(), name: "the roof's eave".to_string(), shared: s.1 - s.0, comes_down: false, left: left_of(s, &covered), beside: false, up: 1 });
    }
    // A roof over the wall by the plan whose eave is not on it: whether it bears here is not worked (call 180, A4).
    let mid = [(w.line[0][0] + w.line[1][0]) / 2.0, (w.line[0][1] + w.line[1][1]) / 2.0];
    // (Not where a wall is drawn standing on this one: the roof is over that wall, and it is the one it is said of.)
    let walled_over = carries.iter().any(|o| o.kind == Kind::Wall && !o.beside);
    let roofs_over: Vec<String> = d.roofs.iter().filter(|r| !walled_over && !eave_roofs.contains(&r.path.as_str()) && (crate::vif::inside(&r.ring, mid) || crate::geom::point_to_outline(mid, &r.ring) <= w.thick + TOL)).map(|r| r.path.clone()).collect();
    // The joists of the floor above, where the wall lies across the inside of it.
    let mut joists: Option<Joists> = None;
    if let Some(above) = d.above(&w.storey) {
        let (dx, dy) = (w.line[1][0] - w.line[0][0], w.line[1][1] - w.line[0][1]);
        let long = dx.hypot(dy).max(1.0e-12);
        for f in d.floors.iter().filter(|f| f.storey == above) {
            let inside = inside_floor(w.line, w.thick, &f.ring);
            let total = length_of(&inside);
            if total <= TOL {
                continue;
            }
            let way = match f.run {
                None => JoistWay::NotSaid,
                Some((run, _)) => {
                    let rl = run[0].hypot(run[1]).max(1.0e-12);
                    if (dx * run[1] - dy * run[0]).abs() / (long * rl) <= SAME_WAY { JoistWay::Alongside } else { JoistWay::Cross }
                }
            };
            let rank = |w: JoistWay| match w {
                JoistWay::Cross => 2,
                JoistWay::NotSaid => 1,
                JoistWay::Alongside => 0,
            };
            if joists.as_ref().map(|j| rank(j.way) >= rank(way)).unwrap_or(false) {
                continue;
            }
            let left = inside.iter().map(|s| left_of(*s, &answered)).sum();
            rest(&f.name, f.status, &mut status);
            joists = Some(Joists { way, floor: f.name.clone(), inside: total, left });
        }
    }
    let (stands_on, counted) = supports(d, w, None);
    for u in &stands_on {
        let s = match u.kind {
            Kind::Wall => d.walls.iter().find(|o| o.path == u.path).map(|o| o.status),
            Kind::Footing => d.footings.iter().find(|o| o.path == u.path).map(|o| o.status),
            _ => d.beams.iter().find(|o| o.path == u.path).map(|o| o.status),
        };
        if let Some(s) = s {
            rest(&u.name, s, &mut status);
        }
    }
    let long = length(w.line);
    Carry {
        path: w.path.clone(),
        name: w.name.clone(),
        storey: w.storey.clone(),
        storey_name,
        length: long,
        carries,
        joists,
        stands_on,
        nothing_under: (long - length_of(&counted)).max(0.0),
        lowest: d.below(&w.storey).is_none(),
        bearing: w.bearing,
        status,
        rests_on: rests.into_iter().map(|r| r.0).collect(),
        roofs_over,
    }
}

/// Every wall's figures, the lowest level first and each level's walls in the order they
/// were drawn. A wall that cannot be placed has none.
pub fn all(job: &mut Job) -> Vec<Carry> {
    let d = gather(job);
    let mut out: Vec<Carry> = d.walls.iter().map(|w| carry_of(&d, w)).collect();
    let rank = |s: &str| d.storeys.iter().position(|x| x.1 == s).unwrap_or(usize::MAX);
    out.sort_by_key(|c| (rank(&c.storey), number(&c.path)));
    out
}

/// One wall's figures, or why it has none.
pub fn of(job: &mut Job, wall: &str) -> Result<Carry, String> {
    let kind = job.find(wall).and_then(|e| job.world.get::<Kind>(e).copied()).ok_or_else(|| format!("{wall} is not in the job"))?;
    if kind != Kind::Wall {
        return Err(format!("{wall} is {} {}, not a wall: only a wall carries or stands on anything", if matches!(kind.said().chars().next(), Some('a' | 'e' | 'i' | 'o' | 'u')) { "an" } else { "a" }, kind.said()));
    }
    all(job).into_iter().find(|c| c.path == wall).ok_or_else(|| format!("{wall} cannot be placed (its line, its type or its level is not known), so what stands on it and what it stands on are not worked out"))
}

// ---------------------------------------------------------------- where it speaks up

fn capital(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// What a wall the work opens has standing on it that is left with nothing under it, or
/// is carried by a header: one sentence for each thing, without the sentence of what
/// these figures are.
fn standing_on_opened(c: &Carry, how: &crate::hidden::Opened) -> Vec<String> {
    let mut out = Vec::new();
    for o in c.carries.iter().filter(|o| !o.comes_down && !o.beside) {
        let what = if o.kind == Kind::Eave { "the roof's eave".to_string() } else { o.name.clone() };
        let lead = if o.up > 1 { format!("{what}, {} levels up, lies over this line with nothing drawn between them,", levels_word(o.up)) } else { format!("{what} stands on it") };
        match how {
            crate::hidden::Opened::Down if o.left > TOL => out.push(format!("{lead} for {}: once it is down, nothing is found under {} for {}", fig(o.shared), if o.kind == Kind::Eave { "the roof there" } else { "that wall" }, fig(o.left))),
            crate::hidden::Opened::Down => {}
            crate::hidden::Opened::Cut(by) => out.push(format!("{lead} for {} and the work cuts it for {}: the header over each opening carries what stands on it", fig(o.shared), by.join(" and "))),
        }
    }
    if let Some(j) = c.joists.as_ref().filter(|j| j.way == JoistWay::Cross) {
        match how {
            crate::hidden::Opened::Down if j.left > TOL => out.push(format!("the joists of the floor above cross it: once it is down, nothing is found under them for {}", fig(j.left))),
            crate::hidden::Opened::Down => {}
            crate::hidden::Opened::Cut(by) => out.push(format!("the joists of the floor above cross it and the work cuts it for {}: the header over each opening carries them", by.join(" and "))),
        }
    }
    if matches!(how, crate::hidden::Opened::Down) && !c.roofs_over.is_empty() {
        out.push("a roof is over it: whether it bears here is not worked (only eaves are), so what it carries is not said".to_string());
    }
    out
}

/// A wall the work opens with something standing on it (call 176, 1): what stands on it,
/// worked as `Carry` works it, each as a sentence that ends with what the figures are.
/// A beam drawn under the wall above (new work) answers a wall that comes down: what it
/// carries is no longer left with nothing under it, and nothing is said of it.
pub fn asked(job: &mut Job) -> BTreeMap<String, Vec<String>> {
    let opened = crate::hidden::opened_walls(&mut job.world);
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in all(job) {
        let mut said: Vec<String> = Vec::new();
        if let Some(how) = opened.get(&c.path) {
            said.extend(standing_on_opened(&c, how).into_iter().map(|s| format!("{} ({SAID}).", capital(&s))));
        }
        if let Some(s) = bearing_on_nothing(&c) {
            said.push(format!("Is anything under it? It is {s} ({SAID})."));
        }
        if !said.is_empty() {
            out.insert(c.path.clone(), said);
        }
    }
    out
}

/// A wall said bearing with nothing under it (call 180, A5): the sentence, or none.
fn bearing_on_nothing(c: &Carry) -> Option<String> {
    match c.bearing {
        Some((Bearing::Bearing, _)) if c.nothing_under > TOL => Some(format!("said bearing, and nothing is found under it for {}{}", fig(c.nothing_under), if c.lowest { " (no footing)" } else { "" })),
        _ => None,
    }
}

/// The cautions of call 176, each with its wall's path: a wall the work opens with
/// something standing on it, and a wall said not bearing with a wall or an eave standing
/// on it.
pub fn cautions(job: &mut Job) -> Vec<(String, String)> {
    let opened = crate::hidden::opened_walls(&mut job.world);
    let mut out = Vec::new();
    for c in all(job) {
        // Two walls named alike in one sentence are named with their levels (call 180, A6).
        let bare = |n: &str| n.split(" on ").next().unwrap_or(n).to_string();
        let me = if c.carries.iter().any(|o| o.kind == Kind::Wall && bare(&o.name) == c.name) { format!("{} on {}", c.name, c.storey_name) } else { c.name.clone() };
        if let Some(how) = opened.get(&c.path) {
            let is = match how {
                crate::hidden::Opened::Down => "is to demolish".to_string(),
                crate::hidden::Opened::Cut(by) => format!("is cut for {}", by.join(" and ")),
            };
            for s in standing_on_opened(&c, how) {
                out.push((c.path.clone(), format!("{} {is}, and {s} ({SAID}).", capital(&me))));
            }
        }
        if matches!(c.bearing, Some((Bearing::NotBearing, _))) {
            for o in c.carries.iter().filter(|o| !o.comes_down && !o.beside) {
                let what = if o.kind == Kind::Eave { "the roof's eave".to_string() } else { o.name.clone() };
                out.push((c.path.clone(), format!("{} is said not bearing, and {what} stands on it for {} ({SAID}).", capital(&me), fig(o.shared))));
            }
        }
        if let Some(s) = bearing_on_nothing(&c) {
            out.push((c.path.clone(), format!("{} is {s} ({SAID}).", capital(&me))));
        }
    }
    out
}

/// Walls whose bearing is not said with something standing on them (call 176, 3): by
/// level, and in the job. A wall or an eave found standing on it, or joists found
/// crossing it, is something; a wall above that comes down is not.
#[derive(Clone, Debug, PartialEq)]
pub struct Unsaid {
    /// Each level that has walls, lowest first: path, name, how many of its walls.
    pub levels: Vec<(String, String, usize)>,
    pub job: usize,
}

pub fn unsaid(job: &mut Job) -> Unsaid {
    let mut levels: Vec<(String, String, usize)> = Vec::new();
    let mut total = 0;
    for c in all(job) {
        let n = usize::from(c.bearing.is_none() && c.is_carrying());
        match levels.iter_mut().find(|l| l.0 == c.storey) {
            Some(l) => l.2 += n,
            None => levels.push((c.storey.clone(), c.storey_name.clone(), n)),
        }
        total += n;
    }
    Unsaid { levels, job: total }
}

/// Each opening the work cuts in an old wall that is bearing or not said, and how deep its
/// header is drawn (call 175, as built): by the wall's path, a sentence for each opening.
/// The depth is a rule of thumb to lay the cripples out by and is said not to be a size;
/// what is asked is the header's size. None for a wall said not bearing.
pub fn headers_asked(job: &mut Job) -> BTreeMap<String, Vec<String>> {
    let opened = crate::hidden::opened_walls(&mut job.world);
    let mut out = BTreeMap::new();
    if opened.is_empty() {
        return out;
    }
    let tree = Tree::of(&mut job.world);
    let work: BTreeMap<String, WorkStatus> = {
        let mut q = job.world.query::<(&Node, &Work)>();
        q.iter(&job.world).filter_map(|(n, w)| w.status.value.map(|s| (n.path.clone(), s))).collect()
    };
    let said: BTreeMap<String, Bearing> = {
        let mut q = job.world.query::<(&Node, &Hidden)>();
        q.iter(&job.world).filter_map(|(n, h)| h.bearing.as_ref().and_then(|b| b.value).map(|v| (n.path.clone(), v))).collect()
    };
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    let mut cut: Vec<(String, String, String, String)> = q
        .iter(&job.world)
        .filter(|(_, k, _)| k.is_opening())
        .filter(|(n, _, _)| !matches!(work.get(&n.path), Some(WorkStatus::Existing | WorkStatus::Demolish)))
        .filter_map(|(n, _, o)| Some((n.parent.clone()?, n.path.clone(), n.name.clone(), o.type_name.value.clone()?)))
        .collect();
    cut.sort_by_key(|(_, p, _, _)| number(p));
    for (wall, _, name, type_path) in cut {
        if !matches!(opened.get(&wall), Some(crate::hidden::Opened::Cut(_))) || said.get(&wall) == Some(&Bearing::NotBearing) {
            continue;
        }
        let Some(span) = tree.opening_types.get(&type_path).and_then(|t| t.width.feet()) else { continue };
        let depth = building::framing::header_depth(span);
        let drawn = units::show_length(&Len::One((depth * 12.0 * 16.0).round() / 16.0 + 0.0), "in");
        out.entry(wall).or_insert_with(Vec::new).push(format!("What size is the header over {name}? It is drawn {drawn} deep ({}).", building::framing::HEADER_DEPTH_IS));
    }
    out
}

// ---------------------------------------------------------------- what goes with a wall that is moved (call 179, B1)

/// A wall as a body on the plan: where it is, how thick, how high.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub path: String,
    pub storey: String,
    /// Its line in feet, first end and second.
    pub line: [P2; 2],
    pub thick: f64,
    pub base: f64,
    pub top: f64,
}

/// Every wall that can be placed, with its level, line, thickness and height.
pub fn bodies(job: &mut Job) -> Vec<Body> {
    gather(job).walls.into_iter().map(|w| Body { path: w.path, storey: w.storey, line: w.line, thick: w.thick, base: w.base, top: w.top }).collect()
}

/// What lies on a wall's line, found as what stands on it and what it stands on are found
/// here (`over`), so that a wall moved can take it along: the walls of the level above whose
/// lines lie on it, the footings of its level under it, the floors of its level with an edge
/// on it, and the roofs with an edge over it whose eave is about as high as its top.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoesWith {
    pub walls: Vec<String>,
    pub footings: Vec<String>,
    pub floors: Vec<String>,
    /// A roof, and which of its edges (counted from 1).
    pub roofs: Vec<(String, usize)>,
}

/// An edge of a floor's outline or a roof's footprint that lies along a wall (call 180, B1): parallel to it
/// within 1 degree, over some of its length, with no other wall between them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeOn {
    /// How far the edge is from the wall's line, in feet.
    pub across: f64,
    /// It extends past either end of the wall by more than it is from the wall's line: moving it with
    /// the wall's end would turn it.
    pub runs_past: bool,
    /// It lies on the outer side of the wall: the wall is inside the outline and the edge is beyond it.
    pub outward: bool,
    /// The wall is inside the outline (or on it).
    pub wall_inside: bool,
}

fn bodies_of(d: &Drawn) -> Vec<Body> {
    d.walls.iter().map(|w| Body { path: w.path.clone(), storey: w.storey.clone(), line: w.line, thick: w.thick, base: w.base, top: w.top }).collect()
}

/// `edge` against `wall`, with `others` the walls that may lie between them and `ring` the outline it is an edge of.
pub fn edge_on(wall: &Body, edge: [P2; 2], others: &[&Body], ring: &[P2]) -> Option<EdgeOn> {
    let (long, elong) = (length(wall.line), length(edge));
    if long <= TOL || elong <= TOL {
        return None;
    }
    let u = [(wall.line[1][0] - wall.line[0][0]) / long, (wall.line[1][1] - wall.line[0][1]) / long];
    let e = [(edge[1][0] - edge[0][0]) / elong, (edge[1][1] - edge[0][1]) / elong];
    if (u[0] * e[1] - u[1] * e[0]).abs() > SAME_WAY {
        return None;
    }
    let n = |p: P2| u[0] * (p[1] - wall.line[0][1]) - u[1] * (p[0] - wall.line[0][0]);
    let s = |p: P2| u[0] * (p[0] - wall.line[0][0]) + u[1] * (p[1] - wall.line[0][1]);
    let lateral = (n(edge[0]) + n(edge[1])) / 2.0;
    let across = lateral.abs();
    let (lo, hi) = (s(edge[0]).min(s(edge[1])), s(edge[0]).max(s(edge[1])));
    if hi.min(long) - lo.max(0.0) <= TOL {
        return None;
    }
    if across > wall.thick + TOL {
        for o in others {
            let ol = length(o.line);
            if ol <= TOL || (u[0] * (o.line[1][1] - o.line[0][1]) - u[1] * (o.line[1][0] - o.line[0][0])).abs() / ol > SAME_WAY {
                continue;
            }
            let ln = (n(o.line[0]) + n(o.line[1])) / 2.0;
            if ln * lateral <= 0.0 || ln.abs() <= wall.thick / 2.0 + TOL || ln.abs() - o.thick / 2.0 > across + TOL {
                continue;
            }
            let (a, b) = (s(o.line[0]).min(s(o.line[1])), s(o.line[0]).max(s(o.line[1])));
            if hi.min(b).min(long) - lo.max(a).max(0.0) > TOL {
                return None;
            }
        }
    }
    let mid = [(wall.line[0][0] + wall.line[1][0]) / 2.0, (wall.line[0][1] + wall.line[1][1]) / 2.0];
    let toward = if lateral >= 0.0 { 1.0 } else { -1.0 };
    let probe = [mid[0] - u[1] * toward * (across + 0.1), mid[1] + u[0] * toward * (across + 0.1)];
    let wall_in = crate::vif::inside(ring, mid) || crate::geom::point_to_outline(mid, ring) <= wall.thick + TOL;
    let outward = across <= wall.thick + TOL || (wall_in && !crate::vif::inside(ring, probe));
    let runs_past = lo < -(across + TOL) || hi > long + across + TOL;
    Some(EdgeOn { across, runs_past, outward, wall_inside: wall_in })
}

/// The edges of `ring` that go with `wall` when it is moved: those lying along it within its
/// thickness, and those a floor has within `reach` of it (2 ft), or a roof outside it within
/// `reach` (4 ft: the eave), with no other wall between (call 180, B1). A roof is over a wall by
/// the plan alone, whatever its eave's height (A2).
pub fn edges_with(wall: &Body, ring: &[P2], bodies: &[Body], reach: f64, roof: bool) -> Vec<(usize, EdgeOn)> {
    let others: Vec<&Body> = bodies.iter().filter(|o| o.path != wall.path && (roof || o.storey == wall.storey)).collect();
    (0..ring.len())
        .filter_map(|i| edge_on(wall, [ring[i], ring[(i + 1) % ring.len()]], &others, ring).map(|e| (i, e)))
        .filter(|(_, e)| e.across <= wall.thick + TOL || (e.across <= reach + TOL && (!roof || e.outward)))
        .collect()
}

/// The edges that lie along `wall` with no other wall between and are farther than the wall's
/// thickness off but not within `reach`: what a move leaves where it was, and what `check` says is
/// drawn out of reach of the wall (call 180, B1).
pub fn edges_all(wall: &Body, ring: &[P2], bodies: &[Body], roof: bool) -> Vec<(usize, EdgeOn)> {
    let others: Vec<&Body> = bodies.iter().filter(|o| o.path != wall.path && (roof || o.storey == wall.storey)).collect();
    (0..ring.len()).filter_map(|i| edge_on(wall, [ring[i], ring[(i + 1) % ring.len()]], &others, ring).map(|e| (i, e))).collect()
}

pub fn edges_beyond(wall: &Body, ring: &[P2], bodies: &[Body], reach: f64, roof: bool) -> Vec<(usize, EdgeOn)> {
    let others: Vec<&Body> = bodies.iter().filter(|o| o.path != wall.path && (roof || o.storey == wall.storey)).collect();
    (0..ring.len())
        .filter_map(|i| edge_on(wall, [ring[i], ring[(i + 1) % ring.len()]], &others, ring).map(|e| (i, e)))
        .filter(|(_, e)| e.across > reach + TOL && e.outward)
        .collect()
}

/// What lies on `wall`'s line, as the job is now drawn.
pub fn goes_with(job: &mut Job, wall: &str) -> Result<GoesWith, String> {
    let roofs: Vec<(String, Vec<P2>)> = {
        let mut q = job.world.query::<(&Node, &Footprint)>();
        let mut all: Vec<(String, Vec<P2>)> = q.iter(&job.world).filter_map(|(n, f)| Some((n.path.clone(), f.points_ft()?))).collect();
        all.sort_by_key(|(p, _)| number(p));
        all
    };
    let d = gather(job);
    let all = bodies_of(&d);
    let me = all.iter().find(|w| w.path == wall).ok_or_else(|| format!("{wall} cannot be placed (its line, its type or its level is not known)"))?;
    let w = d.walls.iter().find(|w| w.path == wall).ok_or_else(|| format!("{wall} cannot be placed (its line, its type or its level is not known)"))?;
    let mut out = GoesWith::default();
    out.walls = walls_above(&d, w).into_iter().filter(|h| h.span.is_some()).map(|h| h.wall.path.clone()).collect();
    out.footings = d.footings.iter().filter(|f| f.storey == w.storey && over(w.line, w.thick, f.line).is_some()).map(|f| f.path.clone()).collect();
    out.floors = d.floors.iter().filter(|f| f.storey == w.storey && !edges_with(me, &f.ring, &all, FLOOR_REACH_FT, false).is_empty()).map(|f| f.path.clone()).collect();
    for (path, ring) in roofs {
        for (i, _) in edges_with(me, &ring, &all, ROOF_REACH_FT, true) {
            out.roofs.push((path.clone(), i + 1));
        }
    }
    Ok(out)
}
