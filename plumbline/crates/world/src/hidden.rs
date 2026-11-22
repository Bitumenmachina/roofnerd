//! What a wall the work opens may be hiding (call 160): whether it is bearing and what runs
//! inside it, asked in the field; what a bearing wall that comes down or is cut needs; and
//! what is in the way of a wall that comes down.
//!
//! Ours. Nothing here is sized or priced: the program says what must be settled and that
//! it is not in the price, as a flag and never a fabrication. No span table is in the
//! program (their terms do not allow it); a beam is an engineer's. What is in the way of a
//! wall is found with the clash test's own shapes (`building::clash::in_the_way`): there
//! is no second geometry here.

use std::collections::BTreeMap;

use bevy_ecs::world::World;

use crate::components::*;

/// Why bearing reads Unknown on a wall the work opens, until it is said.
pub const BEARING_NOT_SEEN: &str = "not seen until it is looked at: which way the joists above run and what bears on it";
/// Why what is in the wall reads Unknown on a wall the work opens, until it is said.
pub const IN_THE_WALL_NOT_SEEN: &str = "pipes, ducts and wiring in an old wall are not seen until it is opened";
/// What a question on the field list is called, on a wall.
pub const BEARING: &str = "bearing";
pub const IN_THE_WALL: &str = "in the wall";

/// How the work opens a wall.
#[derive(Clone, Debug, PartialEq)]
pub enum Opened {
    /// It is marked to demolish.
    Down,
    /// It is marked existing and a door or window that is new or temporary is in it: a
    /// new opening cut in an old wall, by the name of each. One marked existing was there,
    /// and one marked to demolish is taken out and filled in: neither cuts the wall.
    Cut(Vec<String>),
}

/// Every wall the work opens, by path: a wall marked to demolish, or an existing wall with
/// a door or window in it that is new or temporary (not marked existing or to demolish). A wall the work does not
/// open is not in the list.
pub fn opened_walls(world: &mut World) -> BTreeMap<String, Opened> {
    let nodes: Vec<(String, String, Kind, Option<String>)> = {
        let mut q = world.query::<(&Node, &Kind)>();
        q.iter(world).map(|(n, k)| (n.path.clone(), n.name.clone(), *k, n.parent.clone())).collect()
    };
    let work: BTreeMap<String, WorkStatus> = {
        let mut q = world.query::<(&Node, &Work)>();
        q.iter(world).filter_map(|(n, w)| w.status.value.map(|s| (n.path.clone(), s))).collect()
    };
    let mut out = BTreeMap::new();
    for (path, _, kind, _) in &nodes {
        if *kind != Kind::Wall {
            continue;
        }
        match work.get(path) {
            Some(WorkStatus::Demolish) => {
                out.insert(path.clone(), Opened::Down);
            }
            Some(WorkStatus::Existing) => {
                let cut: Vec<String> = nodes.iter().filter(|(p, _, k, parent)| k.is_opening() && parent.as_deref() == Some(path.as_str()) && !matches!(work.get(p), Some(WorkStatus::Existing | WorkStatus::Demolish))).map(|(_, name, _, _)| name.clone()).collect();
                if !cut.is_empty() {
                    out.insert(path.clone(), Opened::Cut(cut));
                }
            }
            _ => {}
        }
    }
    out
}

fn capital(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

fn list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => capital(one),
        [most @ .., last] => format!("{} and {}", most.iter().map(|n| capital(n)).collect::<Vec<_>>().join(", "), capital(last)),
    }
}

/// What a wall that is said to be bearing, and that the work opens, needs, and that it is
/// not in the price: one sentence for each such wall, with the wall's path. Said Assumed
/// or Known alike.
pub fn cautions(world: &mut World) -> Vec<(String, String)> {
    let opened = opened_walls(world);
    let mut out = Vec::new();
    let mut q = world.query::<(&Node, &Hidden)>();
    let mut said: Vec<(String, String, Bearing)> = q.iter(world).filter_map(|(n, h)| h.bearing.as_ref().and_then(|b| b.value).map(|b| (n.path.clone(), n.name.clone(), b))).collect();
    said.sort_by(|a, b| a.0.cmp(&b.0));
    for (path, name, bearing) in said {
        if bearing != Bearing::Bearing {
            continue;
        }
        match opened.get(&path) {
            Some(Opened::Down) => out.push((path.clone(), format!("{} is bearing and comes down: what it carries needs a beam, its posts and their footings, sized by an engineer, and shoring while it is out. None of that is in this price unless a line of this job says so.", capital(&name)))),
            Some(Opened::Cut(by)) => out.push((path.clone(), format!("{} is bearing and is cut for {}: the opening needs a header sized by an engineer, and shoring while it is cut. None of that is in this price unless a line of this job says so.", capital(&name), list(by)))),
            None => {}
        }
    }
    out
}

/// What is in the way of a wall that comes down.
#[derive(Clone, Debug, PartialEq)]
pub struct InTheWay {
    /// The thing in the wall (a run, a fitting, a thing set in place), by path and name.
    pub thing: String,
    pub thing_name: String,
    pub thing_kind: Kind,
    pub wall: String,
    pub wall_name: String,
    /// Where, in feet: east, north, height above the datum.
    pub at: [f64; 3],
}

impl InTheWay {
    /// What to do about it, in one sentence.
    pub fn said(&self) -> String {
        format!("{} is in {}, which comes down: cap it, reroute it or take it out.", capital(&self.thing_name), self.wall_name)
    }
}
