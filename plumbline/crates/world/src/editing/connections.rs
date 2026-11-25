//! Where an item connects, changed and read (call 211): the model's changes, for the window and the command line to say alike.
//!
//! An item holds rows (`building::connections`): each a place where a pipe, a duct, a conduit or power meets it. A row is added,
//! changed or taken away one at a time (`connection_add`, `connection_set`, `connection_take`), each all of it or none, each held
//! to the box of the item at its own size, each one step for Undo, each saying what follows. A thing set down from the item has
//! its connections from that moment (`set_down`), each where the item's sizes put it, no surer than Assumed, saying the item and
//! the connection it came from; they follow the item when its sizes or its rows change (`follow`), told by name; a connection a
//! run or a model brought in made (a name the item has not got) is left alone, and so is one somebody measured. A run's end is
//! never moved to follow: where a connection moves away from a run that reached it, the reply says how many runs now stand off
//! their connection, and the takeoff names them open.
//!
//! **Joined is the network's node, and nothing else.** The network (`building::network`) joins points of one kind within
//! the fit tolerance; a connection is joined when the node it is in holds a run, its end there or passing over the place. Nothing is stored to say so and no second
//! test is made: `connections_standing`, the refusal in `connect`, the stands-off count and the clash's added words all read it
//! from that network.
//!
//! Ours. BIMCraft holds the places on its family and never draws a pipe to one (`grep -rn connector` in its `plumbing.rs`,
//! `runs.rs`, `snap.rs`: nothing); nothing here is ported but the idea, credited in `building::connections`.

use std::collections::BTreeMap;

use super::join::run_info;
use super::{all_or_nothing, Entered};
use crate::building::connections::{check_item_connections, domain_word, flow_word, inches_of, kind_word, Facing, ItemConnection, ItemConnections};
use crate::building::network::{self, thing_to_world, Joint};
use crate::building::{self};
use crate::check::FIT_TOLERANCE_FT;
use crate::components::*;
use crate::job::Job;
use crate::units;
use crate::value::Status;
use crate::{Connector, ConnectorKind, Connectors, Domain, Flow, Item, Placed, P3};

/// A row of an item, made from what a person types: the figures are formulas of the item's `width`, `depth` and `height`
/// (`-4 in`, `height - 12 in`), the size a length ("1/2 in"). `how` says how sure the place is and where it is from, and the
/// size too; Assumed is the only sureness a row may have (`check_item_connections`).
pub fn connection_row(name: &str, takes: ConnectorKind, carries: Option<Domain>, goes: Flow, across: &str, back: &str, up: &str, faces: Facing, size: Option<&str>, how: Entered) -> Result<ItemConnection, String> {
    let size = match size {
        Some(typed) => Some(how.length(typed).map_err(|why| format!("the {name} connection's size: {why}"))?),
        None => None,
    };
    Ok(ItemConnection { name: name.to_string(), kind: takes, carries, flow: goes, across: across.trim().to_string(), back: back.trim().to_string(), up: up.trim().to_string(), faces, place: how.status, place_from: how.source.to_string(), size })
}

/// What a connection on a thing is doing now: where it stands in the world (feet), which runs end at it, and where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Standing {
    /// Its place in the thing's list, from 0.
    pub index: usize,
    pub name: String,
    pub kind: ConnectorKind,
    pub carries: Option<Domain>,
    /// Where it is in the world: east, north, height above the datum, in feet.
    pub at: P3,
    /// Where it is in the thing's own axes, in feet.
    pub local: P3,
    /// The runs that end at it or pass over it (joined, as the network says), by path. None: it is open.
    pub joined_to: Vec<String>,
    /// The row of the item it was made from, where it was.
    pub of_item: Option<String>,
}

/// Where a thing stands: the middle of its base in the world and how it is turned.
struct Frame {
    origin: P3,
    sin_cos: (f64, f64),
}

impl Frame {
    /// A point in the world in the thing's own axes (feet).
    fn local(&self, p: P3) -> P3 {
        let (sin, cos) = self.sin_cos;
        let (dx, dy) = (p[0] - self.origin[0], p[1] - self.origin[1]);
        [dx * cos + dy * sin, -dx * sin + dy * cos, p[2] - self.origin[2]]
    }
}

fn frame_of(job: &mut Job, thing: &str) -> Result<Frame, String> {
    let e = job.find(thing).ok_or_else(|| format!("no element {thing}"))?;
    let placed = job.world.get::<Placed>(e).cloned().ok_or_else(|| format!("{thing} has no position"))?;
    let tree = building::Tree::of(&mut job.world);
    let base = tree.storey_of(thing).and_then(|s| tree.levels.get(s)).and_then(|l| l.elevation.feet()).ok_or_else(|| format!("{thing} is under no level whose floor height is known"))?;
    let at = placed.at_ft().ok_or_else(|| format!("where {thing} stands is not known"))?;
    let offset = placed.offset.feet().ok_or_else(|| format!("{thing}'s height above its level is not known"))?;
    let angle = placed.angle_deg().ok_or_else(|| format!("how far {thing} is turned is not known"))?;
    Ok(Frame { origin: [at[0], at[1], base + offset], sin_cos: angle.to_radians().sin_cos() })
}

/// Every connection of a thing: where it stands and which runs end at it. Read from the network, which says what is joined to what
/// by where each end stands; nothing is stored.
pub fn connections_standing(job: &mut Job, thing: &str) -> Result<Vec<Standing>, String> {
    let frame = frame_of(job, thing)?;
    let e = job.find(thing).ok_or_else(|| format!("no element {thing}"))?;
    let list: Vec<Connector> = job.world.get::<Connectors>(e).map(|c| c.0.clone()).unwrap_or_default();
    let net = network::network(job, None, None, network::TOLERANCE_FT)?;
    let mut runs: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for node in &net.nodes {
        // Joined is what the network says: the node holds a run, whether the run ends there or passes over it.
        let ends: Vec<String> = node.joints.iter().filter_map(|j| if let Joint::RunEnd { run, .. } | Joint::RunSide { run } = j { Some(run.clone()) } else { None }).collect();
        for j in &node.joints {
            if let Joint::Connector { element, index, .. } = j {
                if element == thing {
                    runs.entry(*index).or_default().extend(ends.iter().cloned());
                }
            }
        }
    }
    let mut out = Vec::new();
    for (index, c) in list.iter().enumerate() {
        let Some(local) = c.at_ft() else { continue };
        out.push(Standing { index, name: c.name.clone(), kind: c.kind, carries: c.domain, at: thing_to_world(frame.origin, frame.sin_cos, local), local, joined_to: runs.remove(&index).unwrap_or_default(), of_item: c.of_item.clone() });
    }
    Ok(out)
}

/// The item named (its number or its name): its path, its name, the item and its rows.
fn item_of(job: &mut Job, typed: &str) -> Result<(String, String, Item, Vec<ItemConnection>), String> {
    let path = super::item_by_name(job, typed)?;
    let missing = || format!("{typed:?} is not an item of the job; `plumbline items <file>` lists them");
    let e = job.find(&path).ok_or_else(missing)?;
    let it = job.world.get::<Item>(e).cloned().ok_or_else(missing)?;
    let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| path.clone());
    let rows = job.world.get::<ItemConnections>(e).map(|r| r.0.clone()).unwrap_or_default();
    Ok((path, name, it, rows))
}

/// The rows an item holds, as they are kept.
pub fn item_rows(job: &mut Job, item: &str) -> Result<Vec<ItemConnection>, String> {
    item_of(job, item).map(|(_, _, _, rows)| rows)
}

/// One row worked for the item's sizes.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkedConnection {
    pub row: ItemConnection,
    /// Its place in the item's own axes, in feet.
    pub at: P3,
    /// What a person reads: its name, what it takes and carries, its place in inches, which way it faces and its size.
    pub said: String,
}

/// An item's rows, each worked at the item's own sizes. Refused where a size is not known (it has no place to work them at).
pub fn item_connections_worked(job: &mut Job, item: &str) -> Result<Vec<WorkedConnection>, String> {
    let (_, name, it, rows) = item_of(job, item)?;
    let (size, _, _) = it.size_ft().map_err(|why| format!("{name}: {why}, so where it connects cannot be worked"))?;
    rows.into_iter().map(|row| row.placed(&name, size).map(|at| WorkedConnection { said: row.said_at(at, size), at, row })).collect()
}

/// The rows held to the item: whatever they are, and at the item's sizes the box.
fn held(name: &str, it: &Item, rows: &[ItemConnection]) -> Result<(), String> {
    check_item_connections(name, &ItemConnections(rows.to_vec()))?;
    if let Ok((size, _, _)) = it.size_ft() {
        for r in rows {
            r.placed(name, size)?;
        }
    }
    Ok(())
}

/// The item as it would be, held to its rows at its own sizes (call 211, point 3): a size typed that would put a connection outside
/// the box is refused, naming the item, the connection and how far, and nothing changes.
pub(crate) fn item_fits(job: &mut Job, item_path: &str, it: &Item) -> Result<(), String> {
    let Some(e) = job.find(item_path) else { return Ok(()) };
    let Some(rows) = job.world.get::<ItemConnections>(e).map(|r| r.0.clone()) else { return Ok(()) };
    let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| item_path.to_string());
    held(&name, it, &rows)
}

/// Every item of the job whose rows reach outside its box at its own sizes, with why: what `check` says of a job that holds one (it
/// still opens, as a job saved before does).
pub fn connections_outside_their_boxes(job: &mut Job) -> Vec<(String, String)> {
    let items: Vec<(String, String, Item, Vec<ItemConnection>)> = {
        let mut q = job.world.query::<(&Node, &Item, &ItemConnections)>();
        q.iter(&job.world).map(|(n, it, rows)| (n.path.clone(), n.name.clone(), it.clone(), rows.0.clone())).collect()
    };
    let mut out: Vec<(String, String)> = items.into_iter().filter_map(|(path, name, it, rows)| held(&name, &it, &rows).err().map(|why| (path, why))).collect();
    out.sort();
    out
}

fn put_rows(job: &mut Job, path: &str, rows: Vec<ItemConnection>) -> Result<(), String> {
    let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
    if rows.is_empty() {
        job.world.entity_mut(e).remove::<ItemConnections>();
        Ok(())
    } else {
        job.put(path, ItemConnections(rows))
    }
}

// ---------------------------------------------------------------- what follows

/// A run that stood at a connection and no longer does.
#[derive(Clone, Debug, PartialEq)]
pub struct StandsOff {
    pub run: String,
    pub run_name: String,
    pub thing: String,
    pub thing_name: String,
    pub connection: String,
    /// Where the run's end stands from the connection, in the thing's own words ("3 in in front of").
    pub from: String,
}

impl StandsOff {
    fn said(&self) -> String {
        format!("{} [{}] is {} {}'s {} connection", self.run_name, self.run, self.from, self.thing_name, self.connection)
    }
}

/// How a point stands from a connection, in the thing's own words: how far in inches, and which way. The thing's own axes: across
/// is to its right looking at its front, back is toward its wall, up is up.
fn way_from(delta: P3) -> (f64, String) {
    let far = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
    // An eighth of an inch is what two places are told apart by.
    let parts: Vec<String> = [(delta[0], "to the right of", "to the left of"), (delta[1], "behind", "in front of"), (delta[2], "above", "below")]
        .into_iter()
        .filter(|(d, _, _)| d.abs() * 12.0 >= 0.125)
        .map(|(d, plus, minus)| format!("{} {}", inches_of(d.abs()), if d > 0.0 { plus } else { minus }))
        .collect();
    (far, if parts.is_empty() { "at".to_string() } else { parts.join(" and ") })
}

/// What following did on the things set down from an item.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectionsFollowed {
    /// How many things are set down from the item.
    pub set_down: usize,
    /// Those whose connections changed, by path and name.
    pub things: Vec<(String, String)>,
    /// A connection somebody measured stays where it was measured: said by thing and connection.
    pub left_alone: Vec<String>,
    pub off: Vec<StandsOff>,
}

impl ConnectionsFollowed {
    /// What a reply adds about the connections, beyond the count of things that follow: the connections left where a person
    /// measured them, and how many runs now stand off their connection.
    pub fn lines(&self) -> Vec<String> {
        let mut out: Vec<String> = self.left_alone.clone();
        match self.off.len() {
            0 => {}
            1 => out.push(format!("1 run now stands off its connection: {}", self.off[0].said())),
            n => out.push(format!("{n} runs now stand off their connections: {}", self.off.iter().map(StandsOff::said).collect::<Vec<_>>().join("; "))),
        }
        out
    }
}

/// Which runs end at which connection, by the thing's path and the connection's name, and where that was.
fn joins(job: &mut Job) -> BTreeMap<(String, String), Vec<(String, P3)>> {
    let Ok(net) = network::network(job, None, None, network::TOLERANCE_FT) else { return BTreeMap::new() };
    let mut out: BTreeMap<(String, String), Vec<(String, P3)>> = BTreeMap::new();
    for node in &net.nodes {
        let ends: Vec<(String, P3)> = node.joints.iter().filter_map(|j| if let Joint::RunEnd { run, .. } | Joint::RunSide { run } = j { Some((run.clone(), node.at)) } else { None }).collect();
        if ends.is_empty() {
            continue;
        }
        for j in &node.joints {
            if let Joint::Connector { element, index, .. } = j {
                let name = job.find(element).and_then(|e| job.world.get::<Connectors>(e)).and_then(|c| c.0.get(*index)).map(|c| c.name.clone());
                if let Some(name) = name {
                    out.entry((element.clone(), name)).or_default().extend(ends.iter().cloned());
                }
            }
        }
    }
    out
}

fn called(job: &mut Job, path: &str) -> String {
    job.find(path).and_then(|e| job.world.get::<Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string())
}

/// The things set down from an item, by path and name, in the job's order.
fn set_down_from(job: &mut Job, item_path: &str) -> Vec<(String, String)> {
    let mut q = job.world.query::<(&Node, &Placed)>();
    let mut found: Vec<(usize, String, String)> = q.iter(&job.world).filter(|(_, p)| p.item.as_deref() == Some(item_path)).map(|(n, _)| (0, n.path.clone(), n.name.clone())).collect();
    let order: BTreeMap<String, usize> = job.order.iter().enumerate().filter_map(|(i, e)| job.world.get::<Node>(*e).map(|n| (n.path.clone(), i))).collect();
    for f in &mut found {
        f.0 = order.get(&f.1).copied().unwrap_or(usize::MAX);
    }
    found.sort();
    found.into_iter().map(|(_, p, n)| (p, n)).collect()
}

/// Give a thing just set down its connections from its item (call 211, point 4): none where the item has no rows. Where the item's
/// sizes are not all known it has none, and the answer says why.
pub(crate) fn set_down(job: &mut Job, thing: &str, item_path: &str) -> Result<Option<String>, String> {
    let (name, it, rows) = {
        let e = job.find(item_path).ok_or_else(|| format!("{item_path} is not in the job"))?;
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| item_path.to_string());
        let Some(it) = job.world.get::<Item>(e).cloned() else { return Ok(None) };
        let rows = job.world.get::<ItemConnections>(e).map(|r| r.0.clone()).unwrap_or_default();
        (name, it, rows)
    };
    if rows.is_empty() {
        return Ok(None);
    }
    let (size, sizes, _) = match it.size_ft() {
        Ok(s) => s,
        Err(why) => return Ok(Some(format!("its {} {} not placed: {why}", crate::view::so_many(rows.len(), "connection"), if rows.len() == 1 { "is" } else { "are" }))),
    };
    let made: Vec<Connector> = rows.iter().map(|r| r.made(&name, size, sizes)).collect::<Result<_, _>>()?;
    let kind = job.kind(thing).ok_or_else(|| format!("no element {thing}"))?;
    let connectors = Connectors(made);
    building::check_connectors(thing, kind, &connectors)?;
    job.put(thing, connectors)?;
    Ok(None)
}

/// Every thing set down from the item has its connections as the item says (call 211, point 5): each connection that came from a
/// row is worked again at the item's sizes; one whose row is gone goes; a row with none on the thing gives one, at the end. A
/// connection a run or a model made (no row of the item's) is left alone, and so is one whose place somebody measured. A run's end
/// is never moved: where a connection moves away from a run that reached it, the run is counted as standing off.
pub(crate) fn follow(job: &mut Job, item_path: &str) -> Result<ConnectionsFollowed, String> {
    let things = set_down_from(job, item_path);
    let mut done = ConnectionsFollowed { set_down: things.len(), ..Default::default() };
    if things.is_empty() {
        return Ok(done);
    }
    let (name, it, rows) = {
        let e = job.find(item_path).ok_or_else(|| format!("{item_path} is not in the job"))?;
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| item_path.to_string());
        (name, job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{item_path} is not an item"))?, job.world.get::<ItemConnections>(e).map(|r| r.0.clone()).unwrap_or_default())
    };
    let has = |job: &mut Job, thing: &str| job.find(thing).and_then(|e| job.world.get::<Connectors>(e)).is_some_and(|c| c.0.iter().any(|k| k.of_item.is_some()));
    let any_before = !rows.is_empty() || things.iter().any(|(t, _)| has(job, t));
    if !any_before {
        return Ok(done);
    }
    let before = joins(job);
    let size = it.size_ft().ok().map(|(feet, status, _)| (feet, status));
    for (thing, thing_name) in &things {
        let e = job.find(thing).ok_or_else(|| format!("no element {thing}"))?;
        let existing: Vec<Connector> = job.world.get::<Connectors>(e).map(|c| c.0.clone()).unwrap_or_default();
        let mut next: Vec<Connector> = Vec::new();
        let mut present: Vec<String> = Vec::new();
        for c in &existing {
            let Some(row_name) = &c.of_item else {
                next.push(c.clone());
                continue;
            };
            let row = rows.iter().find(|r| r.name.eq_ignore_ascii_case(row_name));
            if c.at.status == Status::Known {
                // Somebody measured where it is on the thing: it stays, and the reply says so.
                done.left_alone.push(format!("{thing_name} [{thing}]: its {} connection was measured ({}) and stays where it was measured", c.name, c.at.source));
                if row.is_some() {
                    present.push(row_name.to_lowercase());
                }
                next.push(c.clone());
                continue;
            }
            match (row, size) {
                (Some(r), Some((feet, status))) => {
                    present.push(row_name.to_lowercase());
                    next.push(r.made(&name, feet, status)?);
                }
                // The item's sizes are not all known: nothing can be worked again, so it stays as it was.
                (Some(_), None) => {
                    present.push(row_name.to_lowercase());
                    next.push(c.clone());
                }
                // The row is gone: its connection goes with it.
                (None, _) => {}
            }
        }
        if let Some((feet, status)) = size {
            for r in rows.iter().filter(|r| !present.contains(&r.name.to_lowercase())) {
                next.push(r.made(&name, feet, status)?);
            }
        }
        if next == existing {
            continue;
        }
        let kind = job.kind(thing).ok_or_else(|| format!("no element {thing}"))?;
        if next.is_empty() {
            job.world.entity_mut(e).remove::<Connectors>();
        } else {
            let c = Connectors(next);
            building::check_connectors(thing, kind, &c)?;
            job.put(thing, c)?;
        }
        done.things.push((thing.clone(), thing_name.clone()));
    }
    if done.things.is_empty() {
        return Ok(done);
    }
    // Which runs stood at a connection and now stand off it.
    let after = joins(job);
    for ((thing, connection), ends) in before {
        let still: Vec<&String> = after.get(&(thing.clone(), connection.clone())).map(|v| v.iter().map(|(r, _)| r).collect()).unwrap_or_default();
        let now = connections_standing(job, &thing).unwrap_or_default();
        let frame = frame_of(job, &thing).ok();
        for (run, at) in ends {
            if still.contains(&&run) {
                continue;
            }
            let (Some(frame), Some(c)) = (&frame, now.iter().find(|s| s.name == connection)) else { continue };
            let (_, from) = way_from(sub(frame.local(at), c.local));
            let thing_name = called(job, &thing);
            let run_name = called(job, &run);
            done.off.push(StandsOff { run, run_name, thing: thing.clone(), thing_name, connection: connection.clone(), from });
        }
    }
    Ok(done)
}

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ---------------------------------------------------------------- a run that is not at a connection

/// The open connection of a thing, made from its item, that a run's end stands nearest to, and how it stands from it.
#[derive(Clone, Debug, PartialEq)]
pub struct Near {
    pub name: String,
    /// How far the run's end is from it, in feet.
    pub by: f64,
    /// Which way, in the thing's own words ("3 in in front of").
    pub from: String,
}

impl Near {
    /// How far, as a person says it: "3 in away".
    pub fn away(&self) -> String {
        format!("{} away", inches_of(self.by))
    }
}

/// Of the connections of `thing` that came from its item, the open ones that take what a run of `kind` carries (and, where the
/// run says what it carries, that carry it where any does), the one `end` (in the thing's own axes, feet) is nearest to. None
/// where the thing has no such connection open: what a run does there is what it did before.
pub(crate) fn nearest_open(job: &mut Job, thing: &str, end: P3, kind: ConnectorKind, domain: Option<Domain>) -> Result<Option<Near>, String> {
    let open: Vec<Standing> = connections_standing(job, thing)?.into_iter().filter(|s| s.of_item.is_some() && s.kind == kind && s.joined_to.is_empty()).collect();
    let by_domain: Vec<&Standing> = match domain {
        Some(d) if open.iter().any(|s| s.carries == Some(d)) => open.iter().filter(|s| s.carries == Some(d)).collect(),
        _ => open.iter().collect(),
    };
    let nearest = by_domain.into_iter().map(|s| (s, way_from(sub(end, s.local)))).min_by(|a, b| a.1 .0.total_cmp(&b.1 .0));
    Ok(nearest.map(|(s, (by, from))| Near { name: s.name.clone(), by, from }))
}

/// What a run that ends inside a thing says of the thing's open connections: for the clash's line (call 211, point 7). The run's end
/// that is inside the thing's box, and the connection of the run's sort that is open and nearest it. None where the run does not end
/// inside the thing, or the thing has no such connection open.
pub fn run_end_near_connection(job: &mut Job, thing: &str, run: &str) -> Result<Option<Near>, String> {
    let r = run_info(job, run)?;
    let Some(of_run) = ConnectorKind::for_run(r.kind) else { return Ok(None) };
    let frame = frame_of(job, thing)?;
    let e = job.find(thing).ok_or_else(|| format!("no element {thing}"))?;
    let size = job.world.get::<Placed>(e).and_then(|p| p.size_ft()).ok_or_else(|| format!("{thing}'s size is not known"))?;
    let tol = FIT_TOLERANCE_FT;
    let inside = |q: P3| q[0].abs() <= size[0] / 2.0 + tol && q[1].abs() <= size[1] / 2.0 + tol && q[2] >= -tol && q[2] <= size[2] + tol;
    let (qa, qb) = (frame.local(r.a), frame.local(r.b));
    let end = match (inside(qa), inside(qb)) {
        (true, false) => qa,
        (false, true) => qb,
        _ => return Ok(None),
    };
    let domain = run_domain(job, &r);
    nearest_open(job, thing, end, of_run, domain)
}

/// What a run carries, where its system says: a pipe on a water system.
pub(super) fn run_domain(job: &mut Job, r: &super::join::RunInfo) -> Option<Domain> {
    if r.kind != Kind::Pipe {
        return None;
    }
    let class = r.system.as_ref().and_then(|s| job.find(s)).and_then(|s| job.world.get::<building::SystemInfo>(s)).map(|s| s.classification.clone())?;
    Domain::of_system(&class)
}

/// `connect` refuses a run whose end is not at an open connection of the thing's item, saying which connection and how far, and
/// which way (call 211, point 6). `end` is the run's end in the thing's own axes.
pub(crate) fn refuse_if_off(job: &mut Job, thing: &str, run: &str, end: P3, kind: ConnectorKind, domain: Option<Domain>) -> Result<(), String> {
    match nearest_open(job, thing, end, kind, domain)? {
        None => Ok(()),
        Some(n) => {
            let thing_name = called(job, thing);
            Err(format!("{run} does not end at a connection of {thing_name}: its end is {} from the nearest one that is open, the {} connection ({} it). A run is joined where it stands at the connection (its end there, or passing over it), not where it is near: draw its end to the connection", n.away(), n.name, n.from))
        }
    }
}

// ---------------------------------------------------------------- the changes

/// What a change to an item's rows did, for a window and a command line to say alike.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionDone {
    /// The item's name.
    pub item: String,
    /// What was done, in a few words ("the Waste connection added").
    pub did: String,
    /// The row it was to, worked at the item's sizes and said, where there is one and the item has its sizes.
    pub shown: Option<String>,
    /// How many connections the item has now.
    pub rows: usize,
    /// How many things are set down from it.
    pub set_down: usize,
    /// The things (by name) whose connections now follow.
    pub followed: Vec<String>,
    pub left_alone: Vec<String>,
    pub off: Vec<StandsOff>,
}

impl ConnectionDone {
    /// What happened and what follows, in the words a window and the command line both use.
    pub fn said(&self) -> String {
        let has = match self.rows {
            0 => "it has none now".to_string(),
            1 => "it has 1 connection now".to_string(),
            n => format!("it has {n} connections now"),
        };
        let mut said = format!("{}: {}; {has}.", self.item, self.did);
        if let Some(shown) = &self.shown {
            said.push_str(&format!("\n  {shown}"));
        }
        said.push('\n');
        said.push_str(&match (self.set_down, self.followed.len()) {
            (0, _) => "Nothing is set down from it.".to_string(),
            (_, 0) => format!("{} set down from it {} as {} {}.", crate::view::so_many(self.set_down, "thing"), if self.set_down == 1 { "stands" } else { "stand" }, if self.set_down == 1 { "it" } else { "they" }, "should"),
            (_, 1) => "1 thing set down from it follows.".to_string(),
            (_, n) => format!("{n} things set down from it follow."),
        });
        let tail = ConnectionsFollowed { set_down: self.set_down, things: Vec::new(), left_alone: self.left_alone.clone(), off: self.off.clone() };
        for line in tail.lines() {
            said.push_str(&format!("\n{line}"));
        }
        said
    }
}

/// The rows an item holds, changed to `rows` as one change: held (each a row that can be read, the box at the item's own sizes),
/// kept, and every thing set down from the item followed.
fn rewrite(job: &mut Job, path: &str, name: &str, it: &Item, rows: Vec<ItemConnection>, did: String, shown: Option<&str>) -> Result<ConnectionDone, String> {
    held(name, it, &rows)?;
    let shown_row = shown.and_then(|n| rows.iter().find(|r| r.name.eq_ignore_ascii_case(n)).cloned());
    let count = rows.len();
    put_rows(job, path, rows)?;
    let followed = follow(job, path)?;
    let shown = shown_row.map(|r| match it.size_ft() {
        Ok((size, _, _)) => r.placed(name, size).map(|at| r.said_at(at, size)).unwrap_or_default(),
        Err(why) => format!("{}: {why}, so where it is cannot be worked yet", r.name),
    });
    Ok(ConnectionDone { item: name.to_string(), did, shown, rows: count, set_down: followed.set_down, followed: followed.things.iter().map(|(_, n)| n.clone()).collect(), left_alone: followed.left_alone, off: followed.off })
}

/// A connection added to an item: the row is held as a row is (a name of its own, only a pipe carrying something, a place that is
/// Assumed, a box it is on or inside at the item's own size) and every thing set down from the item has it.
pub fn connection_add(job: &mut Job, item: &str, row: ItemConnection) -> Result<ConnectionDone, String> {
    all_or_nothing(job, |job| {
        let (path, name, it, mut rows) = item_of(job, item)?;
        let called = row.name.clone();
        rows.push(row);
        rewrite(job, &path, &name, &it, rows, format!("the {called} connection added"), Some(&called))
    })
}

/// What a row's fields are called by hand.
pub const FIELDS: [&str; 8] = ["takes", "carries", "goes", "across", "back", "up", "faces", "size"];

/// One field of a row changed: takes (pipe, duct, conduit, power), carries (cold, hot, waste, vent, storm, gas, or none), goes (in,
/// out, both), across, back, up (formulas of the item's width, depth and height), faces (back, front, left, right, up, down) or
/// size (a length, or none). A place changed is Assumed, `how` saying who changed it.
pub fn connection_set(job: &mut Job, item: &str, connection: &str, field: &str, value: &str) -> Result<ConnectionDone, String> {
    all_or_nothing(job, |job| {
        let (path, name, it, mut rows) = item_of(job, item)?;
        let at = rows.iter().position(|r| r.name.eq_ignore_ascii_case(connection.trim())).ok_or_else(|| {
            if rows.is_empty() {
                format!("{name:?} has no connection called {:?}: it has none yet", connection.trim())
            } else {
                format!("{name:?} has no connection called {:?}: its connections are {}", connection.trim(), rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>().join(", "))
            }
        })?;
        let value = value.trim();
        let none = value.eq_ignore_ascii_case("none") || value.is_empty();
        let r = &mut rows[at];
        let shown_name = r.name.clone();
        let field = field.trim().to_lowercase();
        let did = match field.as_str() {
            "takes" => {
                r.kind = [ConnectorKind::Pipe, ConnectorKind::Duct, ConnectorKind::Conduit, ConnectorKind::Power].into_iter().find(|k| kind_word(*k).eq_ignore_ascii_case(value)).ok_or_else(|| format!("{value:?} is not something a connection takes: pipe, duct, conduit or power"))?;
                format!("the {shown_name} connection takes a {} now", kind_word(r.kind))
            }
            "carries" => {
                r.carries = if none { None } else { Some([Domain::Cold, Domain::Hot, Domain::Waste, Domain::Vent, Domain::Storm, Domain::Gas].into_iter().find(|d| domain_word(*d).eq_ignore_ascii_case(value)).ok_or_else(|| format!("{value:?} is not something a pipe carries: cold, hot, waste, vent, storm or gas"))?) };
                format!("the {shown_name} connection carries {} now", r.carries.map(domain_word).unwrap_or("nothing said"))
            }
            "goes" => {
                r.flow = [Flow::In, Flow::Out, Flow::Both].into_iter().find(|f| flow_word(*f).eq_ignore_ascii_case(value)).ok_or_else(|| format!("{value:?} is not a way a connection goes: in, out or both"))?;
                format!("the {shown_name} connection goes {} now", flow_word(r.flow))
            }
            "across" | "back" | "up" => {
                match field.as_str() {
                    "across" => r.across = value.to_string(),
                    "back" => r.back = value.to_string(),
                    _ => r.up = value.to_string(),
                }
                r.place = Status::Assumed;
                format!("the {shown_name} connection's {} is {value} now", field.to_lowercase())
            }
            "faces" => {
                r.faces = Facing::named(value).ok_or_else(|| format!("{value:?} is not a way a connection faces: back, front, left, right, up or down"))?;
                format!("the {shown_name} connection faces {} now", r.faces.word())
            }
            "size" => {
                if none {
                    r.size = None;
                } else {
                    let (len, unit) = units::parse_length(value).map_err(|why| format!("the {shown_name} connection's size: {why}"))?;
                    let source = r.size.as_ref().map(|s| s.source.clone()).unwrap_or_else(|| r.place_from.clone());
                    r.size = Some(crate::value::Val { value: Some(len), unit: unit.to_string(), status: Status::Assumed, source });
                }
                format!("the {shown_name} connection's size is {} now", if none { "not said".to_string() } else { value.to_string() })
            }
            other => return Err(format!("{other:?} is not something a connection has: {}", FIELDS.join(", "))),
        };
        rewrite(job, &path, &name, &it, rows, did, Some(&shown_name))
    })
}

/// A connection taken away: every thing set down from the item loses the connection that came from it (one a run or a model made
/// stays).
pub fn connection_take(job: &mut Job, item: &str, connection: &str) -> Result<ConnectionDone, String> {
    all_or_nothing(job, |job| {
        let (path, name, it, mut rows) = item_of(job, item)?;
        let at = rows.iter().position(|r| r.name.eq_ignore_ascii_case(connection.trim())).ok_or_else(|| format!("{name:?} has no connection called {:?}{}", connection.trim(), if rows.is_empty() { ": it has none".to_string() } else { format!(": its connections are {}", rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>().join(", ")) }))?;
        let gone = rows.remove(at);
        rewrite(job, &path, &name, &it, rows, format!("the {} connection taken away", gone.name), None)
    })
}

/// Make the rows an item holds the rows given (a table brought in): the same as changing each by hand, in one step.
pub(crate) fn connections_replace(job: &mut Job, item_path: &str, rows: Vec<ItemConnection>) -> Result<ConnectionDone, String> {
    all_or_nothing(job, |job| {
        let (path, name, it, _) = item_of(job, item_path)?;
        let did = format!("its connections are {} now", rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>().join(", "));
        rewrite(job, &path, &name, &it, rows, did, None)
    })
}
