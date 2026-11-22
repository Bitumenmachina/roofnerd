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
//! * Footings, runs and every other element are not stretched: BIMCraft's relocate
//!   stretches strip footings and glued footings as well, which this program holds as a
//!   wall of their own (`Kind::Footing`) and does not move with a wall.

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

/// A wall that followed the one moved: which, and by how much its length changed (feet,
/// negative when it shrank).
#[derive(Clone, Debug, PartialEq)]
pub struct Followed {
    pub path: String,
    pub grew_by: f64,
    /// It turned: its end did not go along it.
    pub turned: bool,
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
}


fn feet_said(x: f64) -> String {
    let x = x.abs();
    if (x - x.round()).abs() < 1.0e-6 {
        format!("{} ft", x.round())
    } else {
        units::feet_as_ft_in(x, 16)
    }
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
        if !self.held_on.is_empty() {
            let on: Vec<String> = self.held_on.iter().map(|h| self.called(h)).collect();
            let how = if self.own_grew_by.abs() > 1.0e-9 { format!(", {}", by_said(self.own_grew_by)) } else { String::new() };
            out.push(format!("{me} stayed on {}{how}", on.join(" and ")));
        }
        // Said when asked for alone; and when nothing met it, so a person is told that
        // the walls round it were looked at.
        if self.followed.is_empty() && (self.alone || self.held_on.is_empty()) {
            out.push(if self.alone { "moved alone: nothing followed it".to_string() } else { "no wall meets it: nothing followed it".to_string() });
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
fn tidy(x: f64) -> f64 {
    (x * 1.0e9).round() / 1.0e9 + 0.0
}
fn number_of(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

/// A length in feet as a value of the unit it was held in.
fn len_in(feet: f64, unit: &str) -> Option<Len> {
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
/// Only a wall on the same level is followed or held; a footing, a run or anything else is
/// moved as it was. A wall that does not join (its `join` off) follows nobody and is
/// followed by nobody. What was moved is no surer than `how`.
pub fn move_wall(job: &mut Job, path: &str, east: f64, north: f64, how: Entered, alone: bool) -> Result<Moved, String> {
    all_or_nothing(job, |job| {
        let kind = job.kind(path).ok_or_else(|| format!("no element {path}"))?;
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let names: HashMap<String, String> = {
            let mut q = job.world.query::<&Node>();
            q.iter(&job.world).filter(|n| !n.name.is_empty()).map(|n| (n.path.clone(), n.name.clone())).collect()
        };
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
        if alone {
            shift_one(job, path, east, north, how)?;
            if marked_existing(job, path) {
                report.existing.push(path.to_string());
            }
            return Ok(report);
        }
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
        let d = [east, north];
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
                report.own_grew_by = grew;
            } else {
                report.followed.push(Followed { turned: turned.get(&p).copied().unwrap_or(false), path: p, grew_by: grew });
            }
        }
        report.held_on = held_on;
        Ok(report)
    })
}
