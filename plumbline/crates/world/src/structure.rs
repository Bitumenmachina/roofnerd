//! What carries the building, for the engineer (call 164): each wall the work opens with
//! what was said of its bearing, each beam, each post and each pad footing, level by
//! level, each with its size, how sure that is and where it came from.
//!
//! Ours. Nothing here is sized and nothing is advised: every size is what was typed on a
//! type, and the list ends with how many members' sizes are not Known, which are to be
//! sized by an engineer before the building is built or priced firm. No span table is in
//! the program.

use std::collections::BTreeMap;

use crate::building::{Tree, Stood, Segment, SegmentType};
use crate::components::*;
use crate::job::Job;
use crate::units;
use crate::value::Status;

/// One beam, post or pad footing.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub kind: Kind,
    pub path: String,
    pub name: String,
    /// The type it is of, by name; where that is not known, why.
    pub type_called: String,
    /// What it is: its length or height, and where it stands, in a builder's words.
    pub what: String,
    /// Its section as its type says it.
    pub size: String,
    /// How sure its size is (the worst of the type's sizes) and where the size came from.
    pub status: Status,
    pub source: String,
}

/// A wall the work opens: how, and what was said of its bearing.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenedWall {
    pub path: String,
    pub name: String,
    pub how: String,
    pub bearing: String,
}

/// What is on one level.
#[derive(Clone, Debug, PartialEq)]
pub struct OnLevel {
    pub path: String,
    pub name: String,
    pub walls: Vec<OpenedWall>,
    pub members: Vec<Member>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Structure {
    pub levels: Vec<OnLevel>,
    /// How many members have a size that is not Known: to be sized by an engineer.
    pub to_size: usize,
    /// How many members there are in all.
    pub members: usize,
}

/// How sure, in a word.
pub fn word(s: Status) -> &'static str {
    match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    }
}

/// A height as a builder says it: "9 ft" where it is whole feet, as a tape reads it (8' 0 1/8") otherwise.
pub fn height_said(feet: f64) -> String {
    if (feet - feet.round()).abs() < 1.0e-9 {
        format!("{} ft", feet.round())
    } else {
        units::feet_as_ft_in(feet, 16)
    }
}

/// How far above or below its level's floor something is, in words.
pub fn at_height(feet: f64) -> String {
    if feet < 0.0 {
        format!("{} below the floor", height_said(-feet))
    } else {
        format!("{} above the floor", height_said(feet))
    }
}

/// The list for the engineer: nothing is changed and nothing is stored.
pub fn structure(job: &mut Job) -> Structure {
    let tree = Tree::of(&mut job.world);
    let opened = crate::hidden::opened_walls(&mut job.world);
    let mut levels: BTreeMap<String, OnLevel> = BTreeMap::new();
    let mut order: Vec<(f64, String)> = tree.levels.iter().map(|(p, l)| (l.elevation.feet().unwrap_or(f64::INFINITY), p.clone())).collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    for (path, l) in &tree.levels {
        let _ = l;
        levels.insert(path.clone(), OnLevel { path: path.clone(), name: tree.called(path), walls: Vec::new(), members: Vec::new() });
    }
    // Each wall the work opens, and what was said of its bearing.
    for (path, how) in &opened {
        let Some(storey) = tree.storey_of(path).map(str::to_string) else { continue };
        let Some(level) = levels.get_mut(&storey) else { continue };
        let said = job.find(path).and_then(|e| job.world.get::<Hidden>(e)).and_then(|h| h.bearing.clone());
        let bearing = match said {
            Some(b) if b.value.is_some() => format!("{} ({}: {})", b.value.map(|v| v.said()).unwrap_or(""), word(b.status), b.source),
            _ => "not said: unknown until it is looked at".to_string(),
        };
        let how = match how {
            crate::hidden::Opened::Down => "comes down".to_string(),
            crate::hidden::Opened::Cut(by) => format!("is cut for {}", by.join(" and ")),
        };
        level.walls.push(OpenedWall { path: path.clone(), name: tree.called(path), how, bearing });
    }
    for level in levels.values_mut() {
        level.walls.sort_by_key(|w| w.path.clone());
    }
    // Each beam, post and pad footing.
    let mut found: Vec<(String, u64, Member)> = Vec::new();
    let paths: Vec<(String, Kind)> = tree.kinds.iter().filter(|(_, k)| k.is_framing()).map(|(p, k)| (p.clone(), *k)).collect();
    for (path, kind) in paths {
        let Some(e) = job.find(&path) else { continue };
        let Some(storey) = tree.storey_of(&path).map(str::to_string) else { continue };
        let figure = |name: &str| job.world.get::<Measure>(e).and_then(|m| m.0.get(name)).and_then(|d| d.value);
        let (type_ref, stands): (Option<String>, String) = match kind {
            Kind::Beam => match job.world.get::<Segment>(e) {
                Some(s) => {
                    let length = figure("length").map(|x| format!("{} LF", crate::view::num(x))).unwrap_or_else(|| "length not known".to_string());
                    let top = s.offset.value.as_ref().map(|l| units::show_length(l, &s.offset.unit)).unwrap_or_else(|| "not known".to_string());
                    let slope = if s.end_offset.is_some() { ", sloped" } else { "" };
                    (s.type_name.value.clone(), format!("{length}, top {top} above the floor{slope}"))
                }
                None => (None, "no line".to_string()),
            },
            Kind::Post => match job.world.get::<Stood>(e) {
                Some(c) => {
                    let tall = figure("height").map(|x| format!("{} LF tall", crate::view::num(x))).unwrap_or_else(|| "height not known".to_string());
                    let base = c.base_offset.feet().map(|b| format!(", its base {}", if b == 0.0 { "on the floor".to_string() } else { at_height(b) })).unwrap_or_default();
                    (c.type_name.value.clone(), format!("{tall}{base}"))
                }
                None => (None, "no position".to_string()),
            },
            _ => match job.world.get::<Stood>(e) {
                Some(c) => {
                    let thick = c.height.value.as_ref().map(|l| units::show_length(l, &c.height.unit)).unwrap_or_else(|| "not known".to_string());
                    let top = c.base_offset.feet().zip(c.height.feet()).map(|(b, h)| format!(", its top {}", at_height(b + h))).unwrap_or_default();
                    (c.type_name.value.clone(), format!("{thick} thick{top}"))
                }
                None => (None, "no position".to_string()),
            },
        };
        let (type_called, size, status, source) = match type_ref.as_ref().and_then(|t| job.find(t).map(|te| (t.clone(), te))) {
            Some((t, te)) => match job.world.get::<SegmentType>(te) {
                Some(ty) => {
                    let mut parts = vec![ty.width.source.clone()];
                    if let Some(d) = &ty.depth {
                        if d.source != ty.width.source {
                            parts.push(d.source.clone());
                        }
                    }
                    let mut size = ty.said();
                    if let Some(per) = &ty.weight_per_foot {
                        if let Some(x) = per.value {
                            size = format!("{size}, {} lb/ft ({}: {})", crate::view::num(x), word(per.status), per.source);
                        }
                    }
                    (tree.called(&t), size, ty.status(), parts.join("; "))
                }
                None => (tree.called(&t), "size unknown".to_string(), Status::Unknown, "its type has no size".to_string()),
            },
            None => ("no type".to_string(), "size unknown".to_string(), Status::Unknown, "it has no type".to_string()),
        };
        let number = path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX);
        let rank = match kind { Kind::Beam => 0, Kind::Post => 1, _ => 2 };
        found.push((storey, rank * 1_000_000_000 + number, Member { kind, path: path.clone(), name: tree.called(&path), type_called, what: stands, size, status, source }));
    }
    found.sort_by_key(|(_, n, _)| *n);
    let mut to_size = 0;
    let mut members = 0;
    for (storey, _, m) in found {
        members += 1;
        if m.status != Status::Known {
            to_size += 1;
        }
        if let Some(level) = levels.get_mut(&storey) {
            level.members.push(m);
        }
    }
    let levels = order.into_iter().filter_map(|(_, p)| levels.remove(&p)).collect();
    Structure { levels, to_size, members }
}
