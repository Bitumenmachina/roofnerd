//! A wall moved, and the walls that meet it follow (call 170).
//!
//! PORTED CODE, in part. What BIMCraft's `move` does at a joint (`crates/core/src/commands.rs`
//! at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0); the notice is in NOTICE at the repository root) is
//! `relocate` (`commands.rs:11767`), under `move_elements` (`commands.rs:12419`):
//!
//! * a wall that stays and has an end on a taken wall's end goes with that end;
//! * one with an end on a taken wall's span slides along its own line onto the taken
//!   wall's new line (`geom::line_intersection`);
//! * a taken wall's end that was on a wall that stays slides along that wall to where
//!   the taken line now crosses it;
//! * a door or a window keeps its place in the house (its offset is worked out again
//!   from where it was) unless its wall is taken whole, when it goes with it;
//! * walls that do not join (`join` off) stretch for no one.
//!
//! The first, third and fourth are the rules here (call 170's 1, 3 and 4). What is ours:
//! * An end on the taken wall's span goes by the part of the move square to the taken
//!   wall (rule 2: it stays butted and keeps its place along); BIMCraft slides it along
//!   its own line, which is the same where it meets the wall square and differs where
//!   it does not.
//! * A move that would put a door or a window past the end of its wall is refused,
//!   naming it. BIMCraft lets it stand where it falls.
//! * The reply says what followed and by how much, what stands against the moved wall
//!   and was not moved, and which walls marked existing were moved (rules 5 and 6); the
//!   figures are worked in feet, each wall written back in its own unit.
//! * A wall moved also takes with it what lies on its line (call 179, B1): the walls of the
//!   levels above, the floor's edge, the footing under it (the footings that meet its ends
//!   go with it as walls do), the roof's edge over it, and the room lines that end on it.
//!   BIMCraft's relocate stretches strip footings and glued footings too; the rest is ours:
//!   what lies on the wall is found as `carries` finds what stands on it and what it stands
//!   on, and what cannot follow is said as left where it is.
//! * A move that would carry the wall across a wall it does not meet is refused, saying
//!   which wall is in the way and how far it can go (B3). Ours.

use std::collections::HashMap;

use super::{after, all_or_nothing, shift_one, Entered};
use crate::building;
use crate::check::FIT_TOLERANCE_FT as TOL;
use crate::components::*;
use crate::geom;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::Val;
use crate::{Opening, Wall};
use crate::carries;
use crate::solve;

/// A wall that followed the one moved: which, and by how much its length changed (feet,
/// negative when it shrank).
#[derive(Clone, Debug, PartialEq)]
pub struct Followed {
    pub path: String,
    pub grew_by: f64,
    /// It turned: its end did not go along it.
    pub turned: bool,
}

/// A floor whose outline moved with the wall: its plan area before and after, in SF.
#[derive(Clone, Debug, PartialEq)]
pub struct Resized {
    pub path: String,
    /// The wall whose move it went with.
    pub wall: String,
    pub before: f64,
    pub after: f64,
}

/// A roof whose footprint moved with the wall, and what that did to it: the plan area and the
/// surface (the plan area on its pitch), before and after, in SF.
#[derive(Clone, Debug, PartialEq)]
pub struct RoofMoved {
    pub path: String,
    /// The wall whose move it went with.
    pub wall: String,
    pub plan: (f64, f64),
    pub surface: (Option<f64>, Option<f64>),
}

/// What a wall's move did besides the wall.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Moved {
    /// Every wall that followed, in the order the job holds them.
    pub followed: Vec<Followed>,
    /// The moved wall's own length changed by this (feet): not 0 where an end was held on a wall.
    pub own_grew_by: f64,
    /// The walls an end of it was held on (it stayed on them).
    pub held_on: Vec<String>,
    /// What stands against the moved wall and was not moved: path and name.
    pub against: Vec<(String, String)>,
    /// Each wall marked existing that was moved or made to follow.
    pub existing: Vec<String>,
    /// It was moved alone, as asked.
    pub alone: bool,
    /// The moved wall.
    pub wall: String,
    /// What each wall named here is called ("wall 2"), by path.
    pub names: std::collections::BTreeMap<String, String>,
    /// The walls of the levels above that lay on its line and went with it (call 179, B1).
    pub went_with: Vec<String>,
    /// Each floor whose edge lay on it and went with it.
    pub floors: Vec<Resized>,
    /// Each footing that went with it: by how much its length changed.
    pub footings: Vec<Followed>,
    /// Each roof whose edge lay over it and went with it.
    pub roofs: Vec<RoofMoved>,
    /// Each room line that ended on it and went with it.
    /// Each room line that ended on it, with the wall it went with.
    pub room_lines: Vec<(String, String)>,
    /// What lay on its line and could not go with it, and why.
    pub left: Vec<String>,
    /// Moved alone: what would have gone with it, left where it was.
    pub left_where_it_was: Vec<String>,
    /// Moved alone: each corner it opened (call 180, B3): the wall that met it, how far its end is from it now.
    pub opened: Vec<(String, f64)>,
}

pub(crate) fn feet_said(x: f64) -> String {
    let x = x.abs();
    if (x - x.round()).abs() < 1.0e-6 {
        format!("{} ft", x.round())
    } else {
        // "11' 6\"" as a builder says it aloud and writes it on a takeoff: "11 ft 6 in".
        units::feet_as_ft_in(x, 16).replacen("' ", " ft ", 1).replacen('"', " in", 1).trim_start_matches("0 ft ").to_string()
    }
}

/// A move said as a direction, each part in the way a builder says it: "4 ft west", "3 ft
/// north and 4 ft 6 in east". Never a minus sign.
pub fn way_said(east: f64, north: f64) -> String {
    let mut parts = Vec::new();
    if east.abs() > 1.0e-9 {
        parts.push(format!("{} {}", feet_said(east), if east > 0.0 { "east" } else { "west" }));
    }
    if north.abs() > 1.0e-9 {
        parts.push(format!("{} {}", feet_said(north), if north > 0.0 { "north" } else { "south" }));
    }
    if parts.is_empty() {
        "by nothing".to_string()
    } else {
        parts.join(" and ")
    }
}

/// Read what a move is typed as (call 179, B4): east and north as "4,2" (feet, as before) or
/// each of them as a length read as lengths are read everywhere ("4 ft, 2' 6\""), or as
/// directions ("4 ft west", "3 ft 6 in north and 2 ft east", "25' 6\" south"). One distance
/// along each way at most.
pub fn read_move(text: &str) -> Result<(f64, f64), String> {
    // Said as a builder says it aloud (call 180, B4): "4 ft to the east", "4 ft toward the east", "4 ft eastward".
    let mut said = format!(" {} ", text.trim());
    for filler in [" to the ", " toward the ", " towards the ", " toward ", " towards ", " going ", " heading "] {
        while let Some(at) = said.to_ascii_lowercase().find(filler) {
            said.replace_range(at..at + filler.len(), " ");
        }
    }
    for (long, short) in [("eastwards", "east"), ("westwards", "west"), ("northwards", "north"), ("southwards", "south"), ("eastward", "east"), ("westward", "west"), ("northward", "north"), ("southward", "south")] {
        while let Some(at) = said.to_ascii_lowercase().find(long) {
            said.replace_range(at..at + long.len(), short);
        }
    }
    let said = said.trim().to_string();
    let typed = if text.trim().is_empty() { text.trim() } else { said.as_str() };
    if typed.is_empty() {
        return Err("say how far, like \"4 ft west\", \"3 ft 6 in north and 2 ft east\", or \"4,0\" (feet east, feet north)".into());
    }
    let flat = typed.replace(" and ", ",");
    let parts: Vec<&str> = flat.split(',').map(str::trim).collect();
    let way_of = |part: &str| -> Option<(&'static str, String)> {
        let lower = part.to_ascii_lowercase();
        for w in ["east", "west", "north", "south"] {
            if let Some(rest) = lower.strip_suffix(w) {
                if rest.is_empty() || rest.ends_with(|c: char| c.is_whitespace() || c.is_ascii_digit() || c == '"' || c == '\'' || c == '.') {
                    return Some((w, part[..rest.len()].trim().to_string()));
                }
            }
        }
        None
    };
    let any_way = parts.iter().any(|p| way_of(p).is_some());
    let length = |text: &str| -> Result<f64, String> {
        let (len, unit) = units::parse_length(text)?;
        len.feet(unit).map_err(|why| format!("{text:?} {why}"))
    };
    if any_way {
        let (mut east, mut north): (Option<f64>, Option<f64>) = (None, None);
        for part in &parts {
            let (way, how_far) = way_of(part).ok_or_else(|| format!("{part:?} has no direction: give each distance a direction, like \"4 ft west\", or give both as feet, like \"4,0\""))?;
            let feet = length(&how_far)?;
            let (slot, sign) = match way {
                "east" => (&mut east, 1.0),
                "west" => (&mut east, -1.0),
                "north" => (&mut north, 1.0),
                _ => (&mut north, -1.0),
            };
            if slot.is_some() {
                return Err(format!("{typed:?} gives two distances along the one way (east and west, or north and south): give one"));
            }
            *slot = Some(sign * feet + 0.0);
        }
        return Ok((east.unwrap_or(0.0), north.unwrap_or(0.0)));
    }
    let [x, y] = parts.as_slice() else {
        return Err(format!("{typed:?} is not a move: say it as a direction, like \"4 ft west\", or as feet east and feet north, like \"4,0\""));
    };
    // Feet east, feet north: a bare number is feet, as it always was; with a unit it is read as a length is.
    let one = |t: &str| units::parse_number(t).or_else(|_| length(t));
    Ok((one(x)?, one(y)?))
}

fn by_said(grew: f64) -> String {
    if grew > 1.0e-9 {
        format!("{} longer", feet_said(grew))
    } else if grew < -1.0e-9 {
        format!("{} shorter", feet_said(grew))
    } else {
        "the same length".to_string()
    }
}

fn list_said(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

fn sf(x: f64) -> String {
    format!("{} SF", crate::view::figure(x, 2))
}

impl Moved {
    fn called(&self, path: &str) -> String {
        self.names.get(path).cloned().unwrap_or_else(|| path.to_string())
    }
    /// What to tell the builder of the move, a line each, in the words of the trade.
    pub fn said(&self) -> Vec<String> {
        let mut out = Vec::new();
        let me = self.called(&self.wall);
        for f in &self.followed {
            out.push(format!("{} followed: {}{}", self.called(&f.path), by_said(f.grew_by), if f.turned { ", and turned" } else { "" }));
        }
        for w in &self.went_with {
            out.push(format!("{} went with it: it stands over {me}", self.called(w)));
        }
        if !self.held_on.is_empty() {
            let on: Vec<String> = self.held_on.iter().map(|h| self.called(h)).collect();
            let how = if self.own_grew_by.abs() > 1.0e-9 { format!(", {}", by_said(self.own_grew_by)) } else { String::new() };
            out.push(format!("{me} stayed on {}{how}", on.join(" and ")));
        }
        for f in &self.floors {
            out.push(format!("{} followed: its edge on {} went with it, {} now {}", self.called(&f.path), self.called(&f.wall), sf(f.before), sf(f.after)));
        }
        for f in &self.footings {
            let how = if f.grew_by.abs() > 1.0e-9 { by_said(f.grew_by) } else { "moved with it, the same length".to_string() };
            out.push(format!("{} followed: {how}", self.called(&f.path)));
        }
        for r in &self.roofs {
            let surface = match r.surface {
                (Some(a), Some(b)) => format!(", surface {} now {}", sf(a), sf(b)),
                _ => String::new(),
            };
            out.push(format!("{} followed: its edge over {} went with it, plan area {} now {}{surface}", self.called(&r.path), self.called(&r.wall), sf(r.plan.0), sf(r.plan.1)));
        }
        for (l, w) in &self.room_lines {
            out.push(format!("{} followed: its end on {} went with it", self.called(l), self.called(w)));
        }
        for l in &self.left {
            out.push(l.clone());
        }
        // Said when asked for alone; and when nothing met it, so a person is told that
        // the walls round it were looked at.
        let nothing = self.followed.is_empty() && self.went_with.is_empty() && self.floors.is_empty() && self.footings.is_empty() && self.roofs.is_empty() && self.room_lines.is_empty();
        if nothing && (self.alone || self.held_on.is_empty()) {
            out.push(if self.alone { "moved alone: nothing followed it".to_string() } else { "no wall meets it: nothing followed it".to_string() });
        }
        for (w, apart) in &self.opened {
            out.push(format!("the corner of {} and {me} is open, {} apart", self.called(w), feet_said(*apart)));
        }
        if self.alone && !self.left_where_it_was.is_empty() {
            out.push(format!("it left where they were: {}", list_said(&self.left_where_it_was)));
        }
        if !self.against.is_empty() {
            let named: Vec<String> = self.against.iter().take(5).map(|(p, n)| format!("{n} ({p})")).collect();
            let more = if self.against.len() > 5 { format!(" and {} more", self.against.len() - 5) } else { String::new() };
            let one = self.against.len() == 1;
            out.push(format!(
                "{} {} against {me} and {} not moved: {}{more}",
                if one { "1 thing".to_string() } else { format!("{} things", self.against.len()) },
                if one { "stands" } else { "stand" },
                if one { "was" } else { "were" },
                named.join(", ")
            ));
        }
        for w in &self.existing {
            out.push(format!("{} is marked existing: as moved, it is not as it stands.", self.called(w)));
        }
        out
    }
}

type P = [f64; 2];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1]]
}
fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: P, b: P) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn scale(a: P, k: f64) -> P {
    [a[0] * k, a[1] * k]
}
/// Where the two lines through (a, b) and (c, d) cross; none when they run the same way.
fn crossing(a: P, b: P, c: P, d: P) -> Option<P> {
    let (r, s) = (sub(b, a), sub(d, c));
    let denom = cross(r, s);
    if denom.abs() <= 1.0e-9 * geom::dist(a, b) * geom::dist(c, d) {
        return None;
    }
    let t = cross(sub(c, a), s) / denom;
    Some(add(a, scale(r, t)))
}
/// A figure as the file holds it: no stray digits from the arithmetic.
pub(crate) fn tidy(x: f64) -> f64 {
    (x * 1.0e9).round() / 1.0e9 + 0.0
}
fn number_of(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

/// A length in feet as a value of the unit it was held in.
pub(crate) fn len_in(feet: f64, unit: &str) -> Option<Len> {
    match unit {
        "ft" => Some(Len::One(tidy(feet))),
        "in" => Some(Len::One(tidy(feet * 12.0))),
        "ft-in" => {
            let inches = tidy(feet * 12.0);
            let whole = (inches / 12.0).trunc();
            Some(Len::FtIn([whole, tidy(inches - whole * 12.0)]))
        }
        _ => None,
    }
}

struct Standing {
    path: String,
    wall: Wall,
    /// Its line in feet.
    line: [P; 2],
}

/// Where a point goes when a wall is moved, by call 170's rules: one at the wall's first or
/// second end goes where that end goes; one along it, between its ends, goes by the part of
/// the move square to it. Any other does not move.
#[derive(Clone, Copy)]
struct Map {
    a: P,
    b: P,
    fin: [P; 2],
    square: P,
}

impl Map {
    fn at(&self, p: P) -> Option<P> {
        if geom::dist(p, self.a) <= TOL {
            Some(self.fin[0])
        } else if geom::dist(p, self.b) <= TOL {
            Some(self.fin[1])
        } else if geom::point_to_segment(p, self.a, self.b) <= TOL {
            Some(add(p, self.square))
        } else {
            None
        }
    }
}

impl Map {
    /// Where a corner of a floor's outline or a roof's footprint that lies along the wall, but off its
    /// line (an edge drawn outside the wall), goes (call 180, B1): the same distance along the new
    /// line and the same distance off it, and where it is past the wall's end, past its new end.
    /// A point on the wall's line goes as `at` has it.
    fn carry(&self, p: P) -> P {
        if let Some(q) = self.at(p) {
            return q;
        }
        let len = geom::dist(self.a, self.b);
        let new_len = geom::dist(self.fin[0], self.fin[1]);
        if len <= 1.0e-12 || new_len <= 1.0e-12 {
            return add(p, self.square);
        }
        let (u, nu) = (scale(sub(self.b, self.a), 1.0 / len), scale(sub(self.fin[1], self.fin[0]), 1.0 / new_len));
        let (s, n) = (dot(sub(p, self.a), u), cross(u, sub(p, self.a)));
        let base = if s <= 0.0 {
            add(self.fin[0], scale(nu, s))
        } else if s >= len {
            add(self.fin[1], scale(nu, s - len))
        } else {
            add(self.fin[0], scale(sub(self.fin[1], self.fin[0]), s / len))
        };
        add(base, scale([-nu[1], nu[0]], n))
    }
}

/// One wall that was moved: its old line and thickness, where its ends went, and the level it is on.
struct Motion {
    path: String,
    level: Option<String>,
    thick: f64,
    map: Map,
}

// ---------------------------------------------------------------- B3: a wall may not cross another

/// A wall's body on the plan, as four corners, less half its thickness at each end: where
/// two walls join at a corner or a tee their bodies overlap by that much, and that is a
/// joint, not one wall run through another.
fn rect(line: [P; 2], thick: f64) -> [P; 4] {
    let (d, len) = (sub(line[1], line[0]), geom::dist(line[0], line[1]));
    let u = if len > 1.0e-12 { scale(d, 1.0 / len) } else { [1.0, 0.0] };
    let (a, b) = if len > thick + 1.0e-9 { (add(line[0], scale(u, thick / 2.0)), sub(line[1], scale(u, thick / 2.0))) } else { (line[0], line[1]) };
    let n = scale([-u[1], u[0]], thick / 2.0);
    [add(a, n), add(b, n), sub(b, n), sub(a, n)]
}

/// Do two bodies share area? Faces that only touch do not.
fn bodies_cross(a: &[P; 4], b: &[P; 4]) -> bool {
    for r in [a, b] {
        for i in 0..2 {
            let e = sub(r[i + 1], r[i]);
            let axis = [-e[1], e[0]];
            let n = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt();
            if n < 1.0e-12 {
                continue;
            }
            let axis = scale(axis, 1.0 / n);
            let span = |q: &[P; 4]| q.iter().map(|p| dot(*p, axis)).fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
            let (sa, sb) = (span(a), span(b));
            if sa.1.min(sb.1) - sa.0.max(sb.0) <= 1.0e-7 {
                return false;
            }
        }
    }
    true
}

/// Do two lines on the plan touch or cross?
fn lines_meet(l: [P; 2], m: [P; 2]) -> bool {
    if (0..2).any(|i| geom::point_to_segment(m[i], l[0], l[1]) <= TOL || geom::point_to_segment(l[i], m[0], m[1]) <= TOL) {
        return true;
    }
    let side = |p: P, q: P, r: P| cross(sub(q, p), sub(r, p));
    side(l[0], l[1], m[0]) * side(l[0], l[1], m[1]) < 0.0 && side(m[0], m[1], l[0]) * side(m[0], m[1], l[1]) < 0.0
}

/// The first wall `path` would run into if moved by `d`, and the part of the move (0 to 1) it
/// can make before its body meets that wall's. A wall it meets (touching or crossing it, as a
/// neighbour, a partition that butts it, or a wall it butts) is not in the way: those are the
/// walls that follow it or hold its ends. Nor is one it already overlapped.
fn in_the_way(bodies: &[carries::Body], path: &str, d: P) -> Option<(String, f64)> {
    let m = bodies.iter().find(|b| b.path == path)?;
    let size = geom::dist([0.0, 0.0], d);
    let steps = ((size / 0.05).ceil() as usize).clamp(200, 20_000);
    let at = |t: f64| rect([add(m.line[0], scale(d, t)), add(m.line[1], scale(d, t))], m.thick);
    let mut best: Option<(String, f64)> = None;
    for o in bodies.iter().filter(|o| o.path != path && o.storey == m.storey) {
        if lines_meet(m.line, o.line) {
            continue;
        }
        let theirs = rect(o.line, o.thick);
        if bodies_cross(&at(0.0), &theirs) {
            continue;
        }
        let Some(first) = (1..=steps).map(|i| i as f64 / steps as f64).find(|t| bodies_cross(&at(*t), &theirs)) else { continue };
        let (mut lo, mut hi) = (first - 1.0 / steps as f64, first);
        for _ in 0..60 {
            let mid = (lo + hi) / 2.0;
            if bodies_cross(&at(mid), &theirs) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        if best.as_ref().map(|(_, t)| lo < *t).unwrap_or(true) {
            best = Some((o.path.clone(), lo));
        }
    }
    best
}

/// A distance to the sixteenth of an inch it was worked from: the nearest if it is within a
/// hair of one, else the one below, so a wall is never given more room than it has.
fn to_sixteenth(feet: f64) -> f64 {
    let n = feet * 192.0;
    (if (n - n.round()).abs() < 1.0e-4 { n.round() } else { n.floor() }) / 192.0
}

fn refuse_crossing(bodies: &[carries::Body], path: &str, d: P, called: &dyn Fn(&str) -> String) -> Result<(), String> {
    if let Some((other, t)) = in_the_way(bodies, path, d) {
        let (e, n) = (to_sixteenth((d[0] * t).abs()) * d[0].signum(), to_sixteenth((d[1] * t).abs()) * d[1].signum());
        let can = if e.abs() < 1.0e-9 && n.abs() < 1.0e-9 { "it cannot go that way at all".to_string() } else { format!("it can go {}", way_said(e, n)) };
        return Err(format!(
            "{} would run into {}, a wall it does not meet, if moved {}: {can}. Move it that far or less, or take {} out of the way first",
            called(path),
            called(&other),
            way_said(d[0], d[1]),
            called(&other)
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------- the move of one wall (call 170)

/// What each thing is called ("wall 2"), and where more than one level has things of a kind,
/// with its level ("wall 2 on second floor"): every level has a wall 2.
pub(crate) fn names_of(job: &mut Job) -> HashMap<String, String> {
    let tree = building::Tree::of(&mut job.world);
    let mut names: HashMap<String, String> = tree.names.iter().filter(|(_, n)| !n.is_empty()).map(|(p, n)| (p.clone(), n.clone())).collect();
    for kind in [Kind::Wall, Kind::Floor, Kind::Footing, Kind::RoomLine] {
        let things: Vec<(String, String)> = tree.kinds.iter().filter(|(_, k)| **k == kind).filter_map(|(p, _)| Some((p.clone(), tree.storey_of(p)?.to_string()))).collect();
        let mut levels: Vec<&String> = things.iter().map(|(_, s)| s).collect();
        levels.sort();
        levels.dedup();
        if levels.len() > 1 {
            for (p, s) in &things {
                if let (Some(name), Some(level)) = (names.get(p).cloned(), tree.names.get(s).filter(|l| !l.is_empty())) {
                    names.insert(p.clone(), format!("{name} on {level}"));
                }
            }
        }
    }
    names
}

/// One wall moved by call 170's rules, and the walls that meet it taken with it. Fails with
/// the reason; it is called inside a transaction, which puts the job back. `top` is the wall
/// the person moved: its figures are the report's own.
fn step(job: &mut Job, path: &str, d: P, how: Entered, names: &HashMap<String, String>, bodies: &[carries::Body], report: &mut Moved, top: bool) -> Result<Motion, String> {
    let (east, north) = (d[0], d[1]);
    let called = |p: &str| names.get(p).cloned().unwrap_or_else(|| p.to_string());
    let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
    let marked_existing = |job: &mut Job, p: &str| -> bool {
        job.find(p).and_then(|e| job.world.get::<Work>(e).map(|w| w.status.value == Some(WorkStatus::Existing))).unwrap_or(false)
    };
    // Every wall on this wall's level, with its line in feet.
    let parent = job.world.get::<Node>(e).and_then(|n| n.parent.clone());
    let mut all: Vec<Standing> = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Wall)>();
        for (n, k, w) in q.iter(&job.world) {
            if *k == Kind::Wall && n.parent == parent {
                if let Some(line) = w.line_ft() {
                    all.push(Standing { path: n.path.clone(), wall: w.clone(), line });
                }
            }
        }
    }
    all.sort_by_key(|s| number_of(&s.path));
    let m = all.iter().find(|s| s.path == path).ok_or_else(|| format!("{path}'s line is not known, so it cannot be moved"))?;
    let others: Vec<&Standing> = all.iter().filter(|s| s.path != path).collect();
    let [a, b] = m.line;
    let (a2, b2) = (add(a, d), add(b, d));
    let length = geom::dist(a, b);
    let u = if length > 1.0e-12 { scale(sub(b, a), 1.0 / length) } else { [1.0, 0.0] };
    let square = sub(d, scale(u, dot(d, u)));

    // Rule 3: an end of it that butts into a wall that is not moving stays on that wall.
    let mut fin = [a2, b2];
    let mut held_on: Vec<String> = Vec::new();
    let mut held = [false, false];
    if m.wall.join {
        for (i, p) in [a, b].into_iter().enumerate() {
            let host = others.iter().find(|o| {
                o.wall.join
                    && geom::dist(o.line[0], o.line[1]) > TOL
                    && geom::dist(p, o.line[0]) > TOL
                    && geom::dist(p, o.line[1]) > TOL
                    && geom::point_to_segment(p, o.line[0], o.line[1]) <= TOL
            });
            if let Some(h) = host {
                let x = crossing(a2, b2, h.line[0], h.line[1]).ok_or_else(|| {
                    format!(
                        "{} cannot stay on {}, which runs the same way it does: move it alone, or move {} first",
                        called(path),
                        called(&h.path),
                        called(&h.path)
                    )
                })?;
                if geom::point_to_segment(x, h.line[0], h.line[1]) > TOL {
                    return Err(format!(
                        "{}'s end would no longer meet {}: that far, it runs past the end of it. Move it alone, or move {} as well",
                        called(path),
                        called(&h.path),
                        called(&h.path)
                    ));
                }
                fin[i] = x;
                held[i] = true;
                held_on.push(h.path.clone());
            }
        }
    }
    if geom::dist(fin[0], fin[1]) <= TOL {
        return Err(format!("{} would shrink to nothing", called(path)));
    }
    if geom::dist(fin[0], a) <= 1.0e-9 && geom::dist(fin[1], b) <= 1.0e-9 {
        let on: Vec<String> = held_on.iter().map(|h| called(h)).collect();
        return Err(format!(
            "{} does not move: both its ends are held by the walls they meet ({}), and it is pushed along its own length. Move it alone to take its ends off them",
            called(path),
            on.join(" and ")
        ));
    }

    // Rules 1 and 2: the walls that meet it.
    let mut lines: HashMap<String, [P; 2]> = HashMap::new();
    lines.insert(path.to_string(), fin);
    let mut turned: HashMap<String, bool> = HashMap::new();
    if m.wall.join {
        for o in &others {
            if !o.wall.join {
                continue;
            }
            let mut n = o.line;
            for (i, p) in o.line.into_iter().enumerate() {
                if geom::dist(p, a) <= TOL {
                    n[i] = fin[0];
                } else if geom::dist(p, b) <= TOL {
                    n[i] = fin[1];
                } else if geom::point_to_segment(p, a, b) <= TOL {
                    n[i] = add(p, square);
                }
            }
            if n != o.line {
                if geom::dist(n[0], n[1]) <= TOL {
                    return Err(format!("{} would shrink to nothing", called(&o.path)));
                }
                let (old_way, new_way) = (sub(o.line[1], o.line[0]), sub(n[1], n[0]));
                let straight = cross(old_way, new_way).abs() <= 1.0e-9 * geom::dist(o.line[0], o.line[1]) * geom::dist(n[0], n[1]) && dot(old_way, new_way) > 0.0;
                turned.insert(o.path.clone(), !straight);
                lines.insert(o.path.clone(), n);
            }
        }
    }

    // Rule 4: doors and windows stay where they are in the house.
    let widths = building::opening_widths(job);
    let mut openings: Vec<(String, String, Opening)> = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
        for (n, k, o) in q.iter(&job.world) {
            if k.is_opening() {
                if let Some(p) = n.parent.as_ref().filter(|p| lines.contains_key(*p)) {
                    openings.push((n.path.clone(), p.clone(), o.clone()));
                }
            }
        }
    }
    openings.sort_by_key(|o| number_of(&o.0));
    let mut faults: Vec<String> = Vec::new();
    let mut new_offsets: Vec<(String, f64)> = Vec::new();
    for (opath, host, o) in &openings {
        let Some(was) = o.offset.feet() else { continue };
        let Some(was_line) = all.iter().find(|s| s.path == *host).map(|s| s.line) else { continue };
        let now = lines[host];
        let wl = geom::dist(was_line[0], was_line[1]);
        let wu = if wl > 1.0e-12 { scale(sub(was_line[1], was_line[0]), 1.0 / wl) } else { [1.0, 0.0] };
        let nl = geom::dist(now[0], now[1]);
        let nu = scale(sub(now[1], now[0]), 1.0 / nl);
        // Where its centre stood, and where that is now (one in the moved wall goes with it).
        let spot = add(add(was_line[0], scale(wu, was)), if host == path { d } else { [0.0, 0.0] });
        let offset = (dot(sub(spot, now[0]), nu) * 1.0e6).round() / 1.0e6 + 0.0;
        if let Some(w) = widths.get(opath) {
            let (from, to) = (offset - w / 2.0, offset + w / 2.0);
            if from < -TOL || to > nl + TOL {
                let how_far = if from < -TOL { format!("{} before the start of it", feet_said(-from)) } else { format!("{} past the end of it", feet_said(to - nl)) };
                faults.push(format!("{} [{opath}] would no longer lie within {}, which would be {} long ({how_far})", called(opath), called(host), feet_said(nl)));
            }
        }
        if (offset - was).abs() > 1.0e-6 {
            new_offsets.push((opath.clone(), offset));
        }
    }
    if !faults.is_empty() {
        return Err(format!("the move is not made: {}. Move it alone, or move the opening first", faults.join("; ")));
    }

    // Not into a wall it does not meet (call 179, B3), after the refusals above that say more.
    refuse_crossing(bodies, path, d, &called)?;
    // Nor a wall that follows it and turns (call 180, B2): every wall the move changes is tested, as it
    // would stand, against every wall of its level it did not meet and that would stand where it is.
    for (follower, now) in lines.iter().filter(|(p, _)| p.as_str() != path) {
        let Some(x) = bodies.iter().find(|b| b.path == *follower) else { continue };
        for y in bodies.iter().filter(|y| y.storey == x.storey && y.path != *follower && y.path != path) {
            if lines_meet(x.line, y.line) {
                continue;
            }
            let y_now = lines.get(&y.path).copied().unwrap_or(y.line);
            if bodies_cross(&rect(*now, x.thick), &rect(y_now, y.thick)) {
                return Err(format!(
                    "{} follows {} and would cross {}, a wall it does not meet, if {} were moved {}: move it less, or take {} out of the way first",
                    called(follower),
                    called(path),
                    called(&y.path),
                    called(path),
                    way_said(d[0], d[1]),
                    called(&y.path)
                ));
            }
        }
    }

    // Do it. Each wall written back in its own unit; the moved wall's own figures are
    // what a plain move writes, but for an end that was held.
    let mut changed: Vec<(String, f64)> = Vec::new();
    for s in all.iter() {
        let Some(line) = lines.get(&s.path) else { continue };
        let mut w = s.wall.clone();
        let unit = w.line.unit.clone();
        let div = units::coord_divisor(&unit)?;
        let was = w.line.value.ok_or_else(|| format!("{}'s line is not known, so it cannot be moved", s.path))?;
        let mut put = was;
        let (status, source);
        if s.path == path {
            for i in 0..2 {
                put[i] = if held[i] { [tidy(fin[i][0] * div), tidy(fin[i][1] * div)] } else { [was[i][0] + east * div, was[i][1] + north * div] };
            }
            (status, source) = (w.line.status.max(how.status), after(&w.line.source, "moved", how.source));
        } else {
            for i in 0..2 {
                if line[i] != s.line[i] {
                    put[i] = [tidy(line[i][0] * div), tidy(line[i][1] * div)];
                }
            }
            let note = format!("moved with {}", called(path));
            (status, source) = (w.line.status.max(how.status), after(&w.line.source, &note, how.source));
        }
        w.line = Val { value: Some(put), unit, status, source };
        building::check_wall(&s.path, Kind::Wall, &w)?;
        job.put(&s.path, w)?;
        changed.push((s.path.clone(), geom::dist(line[0], line[1]) - geom::dist(s.line[0], s.line[1])));
    }
    for (opath, offset) in new_offsets {
        let e = job.find(&opath).ok_or_else(|| format!("no element {opath}"))?;
        let mut o = job.world.get::<Opening>(e).cloned().ok_or_else(|| format!("{opath} is not an opening"))?;
        if let Some(len) = len_in(offset, &o.offset.unit) {
            let note = format!("kept in place as {} was moved", called(path));
            o.offset = Val { value: Some(len), unit: o.offset.unit.clone(), status: o.offset.status, source: after(&o.offset.source, &note, how.source) };
            job.put(&opath, o)?;
        }
    }
    for (p, grew) in changed {
        if marked_existing(job, &p) {
            report.existing.push(p.clone());
        }
        if p == path {
            if top {
                report.own_grew_by = grew;
            } else {
                report.went_with.push(p);
            }
        } else {
            report.followed.push(Followed { turned: turned.get(&p).copied().unwrap_or(false), path: p, grew_by: grew });
        }
    }
    if top {
        report.held_on = held_on;
    }
    let thick = carries::bodies(job).into_iter().find(|b| b.path == path).map(|b| b.thick).unwrap_or(0.0);
    Ok(Motion { path: path.to_string(), level: parent, thick, map: Map { a, b, fin, square } })
}

// ---------------------------------------------------------------- B1 and B2: what lies on the wall goes with it

fn roof_figures(job: &mut Job, roof: &str) -> (f64, Option<f64>) {
    let planes: Vec<String> = job.children().get(roof).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let (mut plan, mut surface, mut sure) = (0.0, 0.0, true);
    for p in planes {
        let Some(e) = job.find(&p) else { continue };
        if job.world.get::<Kind>(e) != Some(&Kind::RoofPlane) {
            continue;
        }
        let Some(m) = job.world.get::<Measure>(e) else { continue };
        plan += m.0.get("plan_area").and_then(|d| d.worked()).unwrap_or(0.0);
        match m.0.get("surface_area").and_then(|d| d.worked()) {
            Some(x) => surface += x,
            None => sure = false,
        }
    }
    (plan, sure.then_some(surface))
}

/// A line's ends carried by `map`: the line, and whether either end moved.
fn carried(map: &Map, line: [P; 2]) -> ([P; 2], [bool; 2]) {
    let m = [map.at(line[0]), map.at(line[1])];
    ([m[0].unwrap_or(line[0]), m[1].unwrap_or(line[1])], [m[0].is_some(), m[1].is_some()])
}

/// What lies on a wall that was moved goes with it (call 179, B1 and B2), by the same rules the
/// walls that meet it follow by: a point at one end of the wall goes where that end went, one
/// along it by the move's square part. When not `write`, nothing is changed and what would
/// have gone is named in `would`. What lies along the wall but cannot go with it (it runs
/// past the wall's end, so moving it would turn it) is said in `report.left`.
fn bring_along(job: &mut Job, mo: &Motion, goes: &carries::GoesWith, bodies: &[carries::Body], how: Entered, names: &HashMap<String, String>, write: bool, report: &mut Moved, would: &mut Vec<String>) -> Result<(), String> {
    let called = |p: &str| names.get(p).cloned().unwrap_or_else(|| p.to_string());
    let wall = called(&mo.path);
    let map = &mo.map;
    let old = [map.a, map.b];
    let on_line = |line: [P; 2]| carries::over(old, mo.thick, line).is_some();
    let note = format!("moved with {wall}");
    let past = |what: &str| format!("{what} lies along {wall} and runs past the end of it: it was left where it is, and would turn if moved");

    // Footings of this level: the one under it, and the ones that meet its ends or lie against it.
    let footings: Vec<(String, Wall)> = {
        let mut q = job.world.query::<(&Node, &Kind, &Wall)>();
        let mut v: Vec<(String, Wall)> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Footing && n.parent == mo.level).map(|(n, _, w)| (n.path.clone(), w.clone())).collect();
        v.sort_by_key(|(p, _)| number_of(p));
        v
    };
    for (fp, f) in footings {
        let Some(line) = f.line_ft() else { continue };
        let (now, moved) = carried(map, line);
        if !moved[0] && !moved[1] {
            continue;
        }
        if (on_line(line) || goes.footings.contains(&fp)) && !(moved[0] && moved[1]) {
            report.left.push(past(&called(&fp)));
            continue;
        }
        if geom::dist(now[0], now[1]) <= TOL {
            return Err(format!("{} would shrink to nothing", called(&fp)));
        }
        if !write {
            would.push(called(&fp));
            continue;
        }
        let mut w = f.clone();
        let unit = w.line.unit.clone();
        let div = units::coord_divisor(&unit)?;
        let mut put = w.line.value.ok_or_else(|| format!("{fp}'s line is not known, so it cannot be moved"))?;
        for i in 0..2 {
            if moved[i] {
                put[i] = [tidy(now[i][0] * div), tidy(now[i][1] * div)];
            }
        }
        w.line = Val { value: Some(put), unit, status: w.line.status.max(how.status), source: after(&w.line.source, &note, how.source) };
        building::check_wall(&fp, Kind::Footing, &w)?;
        job.put(&fp, w)?;
        report.footings.push(Followed { path: fp.clone(), grew_by: geom::dist(now[0], now[1]) - geom::dist(line[0], line[1]), turned: false });
    }

    // Floors of this level with an edge along it (within its thickness, or within 2 ft of it: a slab drawn
    // a little outside the walls), as the walls' bodies lie.
    let me = bodies.iter().find(|b| b.path == mo.path);
    let floors: Vec<(String, crate::Floor)> = {
        let mut q = job.world.query::<(&Node, &Kind, &crate::Floor)>();
        let mut v: Vec<(String, crate::Floor)> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Floor && n.parent == mo.level).map(|(n, _, f)| (n.path.clone(), f.clone())).collect();
        v.sort_by_key(|(p, _)| number_of(p));
        v
    };
    for (fp, f) in floors {
        let Some(ring) = f.boundary_ft() else { continue };
        let n = ring.len();
        let Some(me) = me else { continue };
        let edges = carries::edges_with(me, &ring, bodies, carries::FLOOR_REACH_FT, false);
        if edges.is_empty() {
            if write {
                if let Some((_, e)) = carries::edges_beyond(me, &ring, bodies, carries::FLOOR_REACH_FT, false).first() {
                    report.left.push(format!("{} was left where it was: its edge along {wall} is {} off it, more than {} ft", called(&fp), feet_said(e.across), carries::FLOOR_REACH_FT));
                }
            }
            continue;
        }
        if edges.iter().any(|(_, e)| e.runs_past) {
            report.left.push(format!("{} has an edge along {wall} that runs past the end of it: it was left where it is, and would turn if moved", called(&fp)));
            continue;
        }
        let mut goes_with_it: Vec<bool> = (0..n).map(|i| map.at(ring[i]).is_some()).collect();
        for (i, _) in &edges {
            goes_with_it[*i] = true;
            goes_with_it[(*i + 1) % n] = true;
        }
        let now: Vec<P> = (0..n).map(|i| if goes_with_it[i] { map.carry(ring[i]) } else { ring[i] }).collect();
        if !write {
            would.push(called(&fp));
            continue;
        }
        let (unit, was) = (f.boundary.unit.clone(), f.boundary.value.clone().ok_or_else(|| format!("{fp}'s outline is not known"))?);
        let div = units::coord_divisor(&unit)?;
        let put: Vec<P> = (0..n).map(|i| if goes_with_it[i] { [tidy(now[i][0] * div), tidy(now[i][1] * div)] } else { was[i] }).collect();
        let mut g = f.clone();
        g.boundary = Val { value: Some(put), unit, status: f.boundary.status.max(how.status), source: after(&f.boundary.source, &note, how.source) };
        building::check_floor(&fp, Kind::Floor, &g)?;
        job.put(&fp, g)?;
        report.floors.push(Resized { path: fp.clone(), wall: mo.path.clone(), before: geom::plan_area(&ring), after: geom::plan_area(&now) });
    }

    // Roofs with an edge over it, by the plan alone (call 180, A2 and B1): the edge outside it by no more than
    // 4 ft with no other wall between, whatever the height of the eave.
    let mut seen: Vec<String> = Vec::new();
    let all_roofs: Vec<(String, Vec<P>)> = {
        let mut q = job.world.query::<(&Node, &Footprint)>();
        let mut v: Vec<(String, Vec<P>)> = q.iter(&job.world).filter_map(|(n, f)| Some((n.path.clone(), f.points_ft()?))).collect();
        v.sort_by_key(|(p, _)| number_of(p));
        v
    };
    for (rp, ring) in all_roofs {
        if seen.contains(&rp) || report.roofs.iter().any(|r| r.path == rp) {
            continue;
        }
        seen.push(rp.clone());
        let Some(me) = me else { continue };
        let n = ring.len();
        let edges = carries::edges_with(me, &ring, bodies, carries::ROOF_REACH_FT, true);
        if edges.is_empty() {
            if write {
                if let Some((_, e)) = carries::edges_beyond(me, &ring, bodies, carries::ROOF_REACH_FT, true).first() {
                    report.left.push(format!("{} was left where it was: its edge over {wall} is {} outside it, more than {} ft", called(&rp), feet_said(e.across), carries::ROOF_REACH_FT));
                }
            }
            continue;
        }
        if edges.iter().any(|(_, e)| e.runs_past) {
            report.left.push(format!("{} has an edge over {wall} that runs past the end of it: it was left where it is, and would turn if moved", called(&rp)));
            continue;
        }
        let mut goes_with_it: Vec<bool> = (0..n).map(|i| map.at(ring[i]).is_some()).collect();
        for (i, _) in &edges {
            goes_with_it[*i] = true;
            goes_with_it[(*i + 1) % n] = true;
        }
        let now: Vec<P> = (0..n).map(|i| if goes_with_it[i] { map.carry(ring[i]) } else { ring[i] }).collect();
        if !write {
            would.push(called(&rp));
            continue;
        }
        let Some(e) = job.find(&rp) else { continue };
        let Some(fp) = job.world.get::<Footprint>(e).cloned() else { continue };
        let before = roof_figures(job, &rp);
        let (unit, was) = (fp.points.unit.clone(), fp.points.value.clone().ok_or_else(|| format!("{rp}'s footprint is not known"))?);
        let div = units::coord_divisor(&unit)?;
        let put: Vec<P> = (0..n).map(|i| if goes_with_it[i] { [tidy(now[i][0] * div), tidy(now[i][1] * div)] } else { was[i] }).collect();
        let mut g = fp.clone();
        g.points = Val { value: Some(put), unit, status: fp.points.status.max(how.status), source: after(&fp.points.source, &note, how.source) };
        let kind = job.kind(&rp).ok_or_else(|| format!("no element {rp}"))?;
        crate::check::footprint(&rp, kind, &g)?;
        job.put(&rp, g)?;
        solve::solve(job, &rp)?;
        let figures = roof_figures(job, &rp);
        report.roofs.push(RoofMoved { path: rp.clone(), wall: mo.path.clone(), plan: (before.0, figures.0), surface: (before.1, figures.1) });
    }

    // Room lines of this level that end on it.
    let lines: Vec<(String, crate::RoomLine)> = {
        let mut q = job.world.query::<(&Node, &crate::RoomLine)>();
        let mut v: Vec<(String, crate::RoomLine)> = q.iter(&job.world).filter(|(n, _)| n.parent == mo.level).map(|(n, r)| (n.path.clone(), r.clone())).collect();
        v.sort_by_key(|(p, _)| number_of(p));
        v
    };
    for (rp, r) in lines {
        let Some(line) = r.line_ft() else { continue };
        let (now, moved) = carried(map, line);
        if !moved[0] && !moved[1] {
            continue;
        }
        if on_line(line) && !(moved[0] && moved[1]) {
            report.left.push(past(&called(&rp)));
            continue;
        }
        if !write {
            would.push(called(&rp));
            continue;
        }
        let unit = r.line.unit.clone();
        let div = units::coord_divisor(&unit)?;
        let mut put = r.line.value.ok_or_else(|| format!("{rp}'s line is not known"))?;
        for i in 0..2 {
            if moved[i] {
                put[i] = [tidy(now[i][0] * div), tidy(now[i][1] * div)];
            }
        }
        let g = crate::RoomLine { line: Val { value: Some(put), unit, status: r.line.status.max(how.status), source: after(&r.line.source, &note, how.source) } };
        building::check_room_line(&rp, Kind::RoomLine, &g)?;
        job.put(&rp, g)?;
        report.room_lines.push((rp, mo.path.clone()));
    }
    Ok(())
}

/// The walls that lie on `path`'s line on the levels above it, each with what lies on its own
/// line; `path` first.
fn chain_of(job: &mut Job, path: &str) -> Vec<(String, carries::GoesWith)> {
    let mut chain: Vec<(String, carries::GoesWith)> = Vec::new();
    let mut next = vec![path.to_string()];
    while let Some(p) = next.pop() {
        if chain.iter().any(|(c, _)| *c == p) {
            continue;
        }
        let goes = carries::goes_with(job, &p).unwrap_or_default();
        next.extend(goes.walls.iter().rev().cloned());
        chain.push((p, goes));
    }
    chain
}

/// Move a wall by so many feet east and north (call 170). Unless `alone`, it takes with
/// it the ends of the walls that meet it, by these rules:
///
/// 1. every other wall's end at one of its ends goes as that end goes;
/// 2. every other wall's end that lies on it, between its ends, goes by the part of the
///    move square to it, so it stays butted and keeps its place along;
/// 3. an end of it that butts into a wall that is not moving stays on that wall: it goes
///    where its new line meets that wall's line. A move that leaves it where it was is
///    refused;
/// 4. doors and windows stay where they are in the house: those in the moved wall go
///    with it, and one in a wall whose first end moved has its offset changed by as much;
///    one that would no longer lie within its wall refuses the whole move, naming it;
/// 5. what stands against the moved wall is not moved, and is told;
/// 6. the moved wall and each that followed are no surer than the move; each marked
///    existing is told;
/// 7. all of it is done or none of it.
///
/// And, from call 179:
///
/// 8. a move that would carry the wall into a wall it does not meet is refused, saying which
///    wall is in the way and how far it can go;
/// 9. what lies on its line goes with it: the walls of the levels above (and the walls that
///    meet them), the floor's edge, the footing under it and the footings that meet its ends,
///    the roof's edge over it, and the room lines that end on it; each is told, and what cannot
///    go (it runs past the wall's end) is told as left where it is. Moved `alone`, none of it
///    goes, and what would have is told as left where it was.
///
/// Only a wall on the same level is followed or held; a run or anything else is moved as it
/// was. A wall that does not join (its `join` off) follows nobody and is followed by nobody.
/// What was moved is no surer than `how`.
pub fn move_wall(job: &mut Job, path: &str, east: f64, north: f64, how: Entered, alone: bool) -> Result<Moved, String> {
    all_or_nothing(job, |job| {
        let kind = job.kind(path).ok_or_else(|| format!("no element {path}"))?;
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let names = names_of(job);
        let called = |p: &str| names.get(p).cloned().unwrap_or_else(|| p.to_string());
        let mut report = Moved { alone, wall: path.to_string(), names: names.iter().map(|(k, v)| (k.clone(), v.clone())).collect(), ..Moved::default() };
        if kind != Kind::Wall || job.world.get::<Wall>(e).is_none() {
            shift_one(job, path, east, north, how)?;
            return Ok(report);
        }
        // The same refusals as ever, before anything is looked for.
        if !(east.is_finite() && north.is_finite()) {
            return Err("the distance to move by is not a figure".into());
        }
        if east == 0.0 && north == 0.0 {
            return Err(format!("{path} is moved by nothing"));
        }
        report.against = building::things_against(job, path).unwrap_or_default();
        let marked_existing = |job: &mut Job, p: &str| -> bool {
            job.find(p).and_then(|e| job.world.get::<Work>(e).map(|w| w.status.value == Some(WorkStatus::Existing))).unwrap_or(false)
        };
        let d = [east, north];
        let bodies = carries::bodies(job);
        // What lies on its line: found before anything moves.
        let chain = chain_of(job, path);
        if alone {
            // B3: not into a wall it does not meet.
            refuse_crossing(&bodies, path, d, &called)?;
            // The wall alone: and what would have gone with it, said as left where it was.
            let mut left: Vec<String> = Vec::new();
            if let Some(b) = bodies.iter().find(|b| b.path == path) {
                // Each corner it opens: a wall of its level with an end on it, now that end is off it (call 180, B3).
                let (a2, b2) = (add(b.line[0], d), add(b.line[1], d));
                for o in bodies.iter().filter(|o| o.storey == b.storey && o.path != path) {
                    for p in o.line {
                        if geom::point_to_segment(p, b.line[0], b.line[1]) <= TOL {
                            let apart = geom::point_to_segment(p, a2, b2);
                            if apart > TOL && !report.opened.iter().any(|(w, _)| *w == o.path) {
                                report.opened.push((o.path.clone(), apart));
                            }
                        }
                    }
                }
                // The walls on its level that meet it would follow.
                for o in bodies.iter().filter(|o| o.storey == b.storey && o.path != path) {
                    if (0..2).any(|i| geom::point_to_segment(o.line[i], b.line[0], b.line[1]) <= TOL) {
                        left.push(called(&o.path));
                    }
                }
                let along = sub(b.line[1], b.line[0]);
                let l2 = dot(along, along).max(1.0e-18);
                let square = sub(d, scale(along, dot(d, along) / l2));
                let level = job.world.get::<Node>(e).and_then(|n| n.parent.clone());
                let mo = Motion { path: path.to_string(), level, thick: b.thick, map: Map { a: b.line[0], b: b.line[1], fin: [add(b.line[0], d), add(b.line[1], d)], square } };
                let mut would = Vec::new();
                for (i, (_, goes)) in chain.iter().enumerate() {
                    if i == 0 {
                        bring_along(job, &mo, goes, &bodies, how, &names, false, &mut report, &mut would)?;
                    }
                }
                left.extend(would);
            }
            for (p, _) in chain.iter().skip(1) {
                left.push(called(p));
            }
            shift_one(job, path, east, north, how)?;
            if marked_existing(job, path) {
                report.existing.push(path.to_string());
            }
            report.left_where_it_was = left;
            return Ok(report);
        }
        let mut motions: Vec<(Motion, carries::GoesWith)> = Vec::new();
        for (i, (p, goes)) in chain.into_iter().enumerate() {
            let mo = step(job, &p, d, how, &names, &bodies, &mut report, i == 0)?;
            motions.push((mo, goes));
        }
        let mut would = Vec::new();
        for (mo, goes) in &motions {
            bring_along(job, mo, goes, &bodies, how, &names, true, &mut report, &mut would)?;
        }
        Ok(report)
    })
}
