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
/// How far from a wall's base (or top) a beam's top may be for the beam to be under (or in line with) it, in feet.
pub const BEAM_REACH_FT: f64 = 1.0;

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
}

fn fig(x: f64) -> String {
    format!("{} LF", view::figure(x, 2))
}

impl Carry {
    /// What stands on it, one sentence a thing, in the words the screens use.
    pub fn carries_said(&self) -> Vec<String> {
        let mut out = Vec::new();
        for o in &self.carries {
            let what = if o.kind == Kind::Eave { "the roof's eave".to_string() } else { format!("{} [{}]", o.name, o.path) };
            out.push(if o.comes_down { format!("{what} for {}, which comes down: not counted", fig(o.shared)) } else { format!("{what} for {}", fig(o.shared)) });
        }
        match &self.joists {
            Some(Joists { way: JoistWay::Cross, .. }) => out.push("the joists of the floor above cross it".to_string()),
            Some(Joists { way: JoistWay::NotSaid, .. }) => out.push("which way the joists of the floor above run is not said".to_string()),
            _ => {}
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
            out.push(if u.comes_down { format!("{what} for {}, which comes down: not counted", fig(u.shared)) } else { format!("{what} for {}", fig(u.shared)) });
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
        self.carries.iter().any(|o| !o.comes_down) || matches!(&self.joists, Some(j) if j.way == JoistWay::Cross)
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
    height: Option<f64>,
    status: Status,
}

struct FloorRec {
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
    floors: Vec<FloorRec>,
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
    let mut eaves = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Run)>();
        let mut all: Vec<(String, Run)> = q.iter(&job.world).filter(|(_, k, _)| **k == Kind::Eave).map(|(n, _, r)| (n.path.clone(), r.clone())).collect();
        all.sort_by_key(|(p, _)| number(p));
        for (path, r) in all {
            let Some(line) = r.ends_ft() else { continue };
            // Its height: its plane's eave height.
            let plane = r.planes.first().and_then(|p| job.find(p)).and_then(|e| job.world.get::<Outline>(e));
            let height = plane.and_then(|o| o.eave_height.feet());
            let status = r.ends.status.max(plane.map(|o| o.eave_height.status).unwrap_or(Status::Unknown));
            eaves.push(EaveRec { path, line, height, status });
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
            floors.push(FloorRec { name: tree.called(&path), storey: storey.to_string(), ring, run, status: f.boundary.status.max(span.as_ref().map(|s| s.0.status).unwrap_or(Status::Known)) });
        }
    }
    Drawn { storeys, walls, footings, beams, eaves, floors }
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
}

// ---------------------------------------------------------------- geometry along a wall

type Span = (f64, f64);

/// How far along `wall` (a line) the line `other` lies over it: where it runs within 1
/// degree of the wall's direction and each of its ends is within `thick` of the wall's
/// line. The stretch is in feet from the wall's first end.
fn over(wall: [P2; 2], thick: f64, other: [P2; 2]) -> Option<Span> {
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

/// What stands under `w` and is counted, as stretches along `w`; `leaving` is a wall taken
/// out of the count (the wall it would stand on, were it to come down).
fn supports(d: &Drawn, w: &WallRec, leaving: Option<&str>) -> (Vec<Under>, Vec<Span>) {
    let mut under = Vec::new();
    let mut counted: Vec<Span> = Vec::new();
    if let Some(below) = d.below(&w.storey) {
        for o in d.walls.iter().filter(|o| o.storey == below && Some(o.path.as_str()) != leaving) {
            if let Some(s) = over(w.line, w.thick, o.line) {
                if !o.comes_down {
                    counted.push(s);
                }
                under.push(Under { kind: Kind::Wall, path: o.path.clone(), name: d.called(o), shared: s.1 - s.0, comes_down: o.comes_down });
            }
        }
    }
    for f in d.footings.iter().filter(|f| f.storey == w.storey) {
        if let Some(s) = over(w.line, w.thick, f.line) {
            counted.push(s);
            under.push(Under { kind: Kind::Footing, path: f.path.clone(), name: f.name.clone(), shared: s.1 - s.0, comes_down: false });
        }
    }
    for b in d.beams.iter().filter(|b| (b.top - w.base).abs() <= BEAM_REACH_FT + 1.0e-9) {
        if let Some(s) = over(w.line, w.thick, b.line) {
            counted.push(s);
            under.push(Under { kind: Kind::Beam, path: b.path.clone(), name: b.name.clone(), shared: s.1 - s.0, comes_down: false });
        }
    }
    (under, merged(counted))
}

/// The stretches of `w` that a beam carries at `w`'s top: what a beam drawn in its line
/// answers, were `w` taken out.
fn beams_at_top(d: &Drawn, w: &WallRec) -> Vec<Span> {
    d.beams.iter().filter(|b| (b.top - w.top).abs() <= BEAM_REACH_FT + 1.0e-9).filter_map(|b| over(w.line, w.thick, b.line)).collect()
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
    if let Some(above) = d.above(&w.storey) {
        for u in d.walls.iter().filter(|u| u.storey == above) {
            let Some(s) = over(w.line, w.thick, u.line) else { continue };
            // What would have nothing under it if this wall came out: its own length over
            // this wall, less what else is drawn under it.
            let left = match over(u.line, u.thick, w.line) {
                Some(mine) => left_of(mine, &supports(d, u, Some(&w.path)).1),
                None => 0.0,
            };
            rest(&d.called(u), u.status, &mut status);
            carries.push(On { kind: Kind::Wall, path: u.path.clone(), name: d.called(u), shared: s.1 - s.0, comes_down: u.comes_down, left });
        }
    }
    let answered = beams_at_top(d, w);
    for e in &d.eaves {
        // Over this wall where it is about as high as the wall's top; one whose height is not known is taken as over it, Unknown.
        if !e.height.map(|h| (h - w.top).abs() <= EAVE_REACH_FT + 1.0e-9).unwrap_or(true) {
            continue;
        }
        let Some(s) = over(w.line, w.thick, e.line) else { continue };
        rest("the roof's eave", e.status, &mut status);
        carries.push(On { kind: Kind::Eave, path: e.path.clone(), name: "the roof's eave".to_string(), shared: s.1 - s.0, comes_down: false, left: left_of(s, &answered) });
    }
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
    for o in c.carries.iter().filter(|o| !o.comes_down) {
        let what = if o.kind == Kind::Eave { "the roof's eave".to_string() } else { o.name.clone() };
        match how {
            crate::hidden::Opened::Down if o.left > TOL => out.push(format!("{what} stands on it for {}: once it is down, nothing is found under {} for {}", fig(o.shared), if o.kind == Kind::Eave { "the roof there" } else { "that wall" }, fig(o.left))),
            crate::hidden::Opened::Down => {}
            crate::hidden::Opened::Cut(by) => out.push(format!("{what} stands on it for {} and the work cuts it for {}: the header over each opening carries what stands on it", fig(o.shared), by.join(" and "))),
        }
    }
    if let Some(j) = c.joists.as_ref().filter(|j| j.way == JoistWay::Cross) {
        match how {
            crate::hidden::Opened::Down if j.left > TOL => out.push(format!("the joists of the floor above cross it: once it is down, nothing is found under them for {}", fig(j.left))),
            crate::hidden::Opened::Down => {}
            crate::hidden::Opened::Cut(by) => out.push(format!("the joists of the floor above cross it and the work cuts it for {}: the header over each opening carries them", by.join(" and "))),
        }
    }
    out
}

/// A wall the work opens with something standing on it (call 176, 1): what stands on it,
/// worked as `Carry` works it, each as a sentence that ends with what the figures are.
/// A beam drawn under the wall above (new work) answers a wall that comes down: what it
/// carries is no longer left with nothing under it, and nothing is said of it.
pub fn asked(job: &mut Job) -> BTreeMap<String, Vec<String>> {
    let opened = crate::hidden::opened_walls(&mut job.world);
    let mut out = BTreeMap::new();
    if opened.is_empty() {
        return out;
    }
    for c in all(job) {
        let Some(how) = opened.get(&c.path) else { continue };
        let said = standing_on_opened(&c, how);
        if !said.is_empty() {
            out.insert(c.path.clone(), said.into_iter().map(|s| format!("{} ({SAID}).", capital(&s))).collect());
        }
    }
    out
}

/// The cautions of call 176, each with its wall's path: a wall the work opens with
/// something standing on it, and a wall said not bearing with a wall or an eave standing
/// on it.
pub fn cautions(job: &mut Job) -> Vec<(String, String)> {
    let opened = crate::hidden::opened_walls(&mut job.world);
    let mut out = Vec::new();
    for c in all(job) {
        if let Some(how) = opened.get(&c.path) {
            let is = match how {
                crate::hidden::Opened::Down => "is to demolish".to_string(),
                crate::hidden::Opened::Cut(by) => format!("is cut for {}", by.join(" and ")),
            };
            for s in standing_on_opened(&c, how) {
                out.push((c.path.clone(), format!("{} {is}, and {s} ({SAID}).", capital(&c.name))));
            }
        }
        if matches!(c.bearing, Some((Bearing::NotBearing, _))) {
            for o in c.carries.iter().filter(|o| !o.comes_down) {
                let what = if o.kind == Kind::Eave { "the roof's eave".to_string() } else { o.name.clone() };
                out.push((c.path.clone(), format!("{} is said not bearing, and {what} stands on it for {} ({SAID}).", capital(&c.name), fig(o.shared))));
            }
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
