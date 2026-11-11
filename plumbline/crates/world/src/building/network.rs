//! The network of a job's runs, worked out from the job and never stored: where the ends
//! of ducts, pipes, conduit and cable and the connectors of equipment, fixtures and
//! fittings meet, which runs join which, which ends are open and what looks wrong. It is
//! rebuilt each time it is asked for, so it cannot go stale. A job being entered has open
//! ends and odd joins, so nothing here refuses a job: what looks wrong is a warning.
//!
//! Points within `tolerance` of each other are one node. A run's end on another run's
//! length (a branch drawn to its centre line, with no fitting) is a node too, splitting
//! that run in two edges without splitting the element. A fitting's connectors are joined
//! through it, so a system runs through it; a fixture's and a piece of equipment's are
//! not, since a fixture's waste and its water are networks of their own.
//!
//! PORTED CODE. This file is ported from BIMCraft, `crates/core/src/network.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from.
//!
//! What was changed in the port, and nothing else:
//! * Lengths are feet. BIMCraft's tolerance of 1 mm is this program's fit tolerance,
//!   0.01 ft; its largest, 50 mm, is 2 in.
//! * BIMCraft's network is of pipes. Here it is of every kind of run, and a point joins
//!   only points of its own kind: a duct does not join a pipe that passes within the
//!   tolerance of it. A node says which kind meets at it.
//! * An element is named by its path, and a run's system is the path of a system element,
//!   said in a message by the system's name.
//! * BIMCraft leaves out, without saying so, a pipe it cannot work out. Here each run and
//!   each connector that cannot be placed is listed in `left_out` with why: an answer
//!   that leaves something out says what it left out.
//! * A run with no size is still in the network (BIMCraft has no geometry for a run with
//!   no type); the narrowing check passes over it.
//! * The grid of the plan's squares that finds the pipes a node may lie along is not
//!   taken, nor its limits: every run of the node's kind is tried.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use bevy_ecs::prelude::*;

use super::model::{ConnectorKind, Connectors, Domain, Placed, Segment};
use super::{runs, typed, InSystem, Tree, P3};
use crate::check::FIT_TOLERANCE_FT;
use crate::components::{Kind, Node as Element};
use crate::job::Job;

/// A run: its path, its centre line's ends, its width where it has one, its system, where
/// its ends are among the points, and its kind.
type Run = (String, P3, P3, Option<f64>, Option<String>, usize, Kind);
/// A group being gathered: its elements, systems, domains and nodes.
type Gathered = (BTreeSet<String>, BTreeSet<String>, BTreeSet<Domain>, Vec<usize>);

/// How close two points must be to be one node, in feet, unless a query says otherwise.
pub const TOLERANCE_FT: f64 = FIT_TOLERANCE_FT;
/// The largest tolerance a query may ask for: 2 in.
pub const MAX_TOLERANCE_FT: f64 = 2.0 / 12.0;
/// The most points (runs' ends and connectors) one query works on.
pub const MAX_POINTS: usize = 200_000;

/// What meets at a node.
#[derive(Clone, Debug, PartialEq)]
pub enum Joint {
    /// A run's first end (`end: false`) or its second.
    RunEnd { run: String, end: bool },
    /// A run passing through: the node lies on its length.
    RunSide { run: String },
    /// A placed thing's connector, by its place in the thing's list.
    Connector { element: String, index: usize, domain: Option<Domain> },
}

impl Joint {
    pub fn element(&self) -> &str {
        match self {
            Joint::RunEnd { run, .. } | Joint::RunSide { run } => run,
            Joint::Connector { element, .. } => element,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub at: P3,
    /// The kind of run that meets here.
    pub kind: Kind,
    pub joints: Vec<Joint>,
}

/// A piece of a run between two nodes, or the way through a fitting from one of its
/// connectors to another.
#[derive(Clone, Debug)]
pub struct Edge {
    pub element: String,
    pub from: usize,
    pub to: usize,
    /// Feet; none through a fitting.
    pub length: Option<f64>,
}

/// Runs and placed things joined to each other, and what they carry.
#[derive(Clone, Debug)]
pub struct Group {
    pub elements: Vec<String>,
    /// The systems its runs are on, by path: those that have one.
    pub systems: Vec<String>,
    pub domains: Vec<Domain>,
}

#[derive(Clone, Debug)]
pub struct Warning {
    /// `systems_meet`, `domains_meet`, `joined_mid_run` or `smaller_downstream`.
    pub kind: &'static str,
    pub at: Option<P3>,
    pub elements: Vec<String>,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
    /// The nodes where one end or one connector meets nothing.
    pub open: Vec<usize>,
    pub warnings: Vec<Warning>,
    /// What could not be placed and so is in none of the above, each with why.
    pub left_out: Vec<(String, String)>,
}

fn dist(a: P3, b: P3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Where along segment `a`–`b` the point nearest `p` lies (0 to 1) and how far `p` is from it.
fn along(p: P3, a: P3, b: P3) -> (f64, f64) {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let len2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if len2 <= 0.0 {
        return (0.0, dist(p, a));
    }
    let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1] + (p[2] - a[2]) * d[2]) / len2).clamp(0.0, 1.0);
    (t, dist(p, [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t]))
}

struct Sets(Vec<usize>);

impl Sets {
    fn new(n: usize) -> Sets {
        Sets((0..n).collect())
    }
    fn find(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }
    fn join(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        if a != b {
            self.0[a.max(b)] = a.min(b);
        }
    }
}

/// The cell of a grid of `size` a point is in.
fn cell(p: P3, size: f64) -> (i64, i64, i64) {
    let k = |x: f64| (x / size).floor() as i64;
    (k(p[0]), k(p[1]), k(p[2]))
}

/// The network of the job's runs and connectors: of `system`'s runs only (and what they
/// join), of `storey`'s elements only, as asked; points within `tolerance` feet are one
/// node.
pub fn network(job: &mut Job, system: Option<&str>, storey: Option<&str>, tolerance: f64) -> Result<Network, String> {
    let tol = tolerance.clamp(1e-9, MAX_TOLERANCE_FT);
    let order = job.order.clone();
    let tree = Tree::of(&mut job.world);
    let w: &World = &job.world;
    // The runs, their centre lines; the placed things' connectors, placed.
    let mut all: Vec<Run> = Vec::new();
    let mut points: Vec<(P3, Joint, Kind)> = Vec::new();
    let mut through: Vec<Vec<usize>> = Vec::new();
    let mut left_out: Vec<(String, String)> = Vec::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (w.get::<Element>(e), w.get::<Kind>(e).copied()) else { continue };
        if !(kind.is_segment() || kind.is_placed()) {
            continue;
        }
        let on = tree.storey_of(&n.path);
        if storey.is_some_and(|s| on != Some(s)) {
            continue;
        }
        // Where its storey's floor is, or why that is not to be had.
        let base: Result<f64, String> = match on {
            None => Err("it is under no storey".into()),
            Some(s) => tree.levels.get(s).and_then(|l| l.elevation.feet()).ok_or_else(|| "its storey's elevation is not known".to_string()),
        };
        let its_system = w.get::<InSystem>(e).and_then(|s| s.0.value.clone());
        if kind.is_segment() {
            let placed: Result<(P3, P3, Option<f64>), String> = (|| {
                let r = w.get::<Segment>(e).ok_or("it has no line")?;
                let [p, q] = r.line_ft().ok_or("its line is not known")?;
                let (z0, z1) = r.offsets_ft().ok_or("its height above its storey is not known")?;
                let t = typed(&tree.segment_types, &r.type_name, "a run").ok().map(|x| x.1);
                let section = t.and_then(|t| t.round_ft().map(|d| (d, d, true)).or_else(|| t.rect_ft().map(|(w, d)| (w, d, false))));
                if r.justify != super::Justify::Centre && section.is_none() {
                    return Err("its size is not known and its heights are those of its top or its bottom, so its centre line cannot be placed".into());
                }
                let (width, depth, round) = section.unwrap_or((0.0, 0.0, false));
                let g = runs::Geom::of(base.clone()?, p, q, z0, z1, r.justify, width, depth, round);
                Ok((g.a, g.b, section.map(|s| s.0)))
            })();
            match placed {
                Err(why) => left_out.push((n.path.clone(), why)),
                Ok((a, b, width)) => {
                    all.push((n.path.clone(), a, b, width, its_system, points.len(), kind));
                    points.push((a, Joint::RunEnd { run: n.path.clone(), end: false }, kind));
                    points.push((b, Joint::RunEnd { run: n.path.clone(), end: true }, kind));
                }
            }
        } else if let Some(connectors) = w.get::<Connectors>(e) {
            // Where it stands: turned and moved to its place (BIMCraft `model.rs:3313`).
            let stands: Result<(P3, f64), String> = (|| {
                let c = w.get::<Placed>(e).ok_or("it has connectors and no position")?;
                let at = c.at_ft().ok_or("its position is not known")?;
                let offset = c.offset.feet().ok_or("its height above its storey is not known")?;
                let angle = c.angle_deg().ok_or("how far it is turned is not known")?;
                Ok(([at[0], at[1], base.clone()? + offset], angle))
            })();
            let (origin, angle) = match stands {
                Ok(x) => x,
                Err(why) => {
                    left_out.push((n.path.clone(), why));
                    continue;
                }
            };
            let (sin, cos) = angle.to_radians().sin_cos();
            let mut mine: BTreeMap<ConnectorKind, Vec<usize>> = BTreeMap::new();
            for (index, k) in connectors.0.iter().enumerate() {
                let Some(q) = k.at_ft() else {
                    left_out.push((n.path.clone(), format!("where its connector {} is, is not known", index + 1)));
                    continue;
                };
                let at = [origin[0] + q[0] * cos - q[1] * sin, origin[1] + q[0] * sin + q[1] * cos, origin[2] + q[2]];
                mine.entry(k.kind).or_default().push(points.len());
                points.push((at, Joint::Connector { element: n.path.clone(), index, domain: k.domain }, k.kind.joins()));
            }
            if kind == Kind::Fitting {
                through.extend(mine.into_values().filter(|list| list.len() > 1));
            }
        }
        if points.len() > MAX_POINTS {
            return Err(format!("the job has more than {MAX_POINTS} run ends and connectors to join: ask for one storey or one system"));
        }
    }
    // Points within the tolerance are one node: each is compared with those in its cell and the cells round it.
    let mut sets = Sets::new(points.len());
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    for (i, (p, _, kind)) in points.iter().enumerate() {
        let (x, y, z) = cell(*p, tol);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(others) = grid.get(&(x + dx, y + dy, z + dz)) {
                        for &j in others {
                            if points[j].2 == *kind && dist(points[j].0, *p) <= tol {
                                sets.join(i, j);
                            }
                        }
                    }
                }
            }
        }
        grid.entry((x, y, z)).or_default().push(i);
    }
    let mut node_of: BTreeMap<usize, usize> = BTreeMap::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut at_point = vec![0; points.len()];
    for (i, (p, j, kind)) in points.iter().enumerate() {
        let root = sets.find(i);
        let n = *node_of.entry(root).or_insert_with(|| {
            nodes.push(Node { at: *p, kind: *kind, joints: Vec::new() });
            nodes.len() - 1
        });
        nodes[n].joints.push(j.clone());
        at_point[i] = n;
    }
    // A node on a run's length, away from its ends, splits it.
    let mut sides: Vec<Vec<(f64, usize)>> = vec![Vec::new(); all.len()];
    for (n, node) in nodes.iter().enumerate() {
        let p = node.at;
        for (k, (path, a, b, _, _, _, kind)) in all.iter().enumerate() {
            if *kind != node.kind || node.joints.iter().any(|j| matches!(j, Joint::RunEnd { run, .. } if run == path)) {
                continue;
            }
            let (t, d) = along(p, *a, *b);
            let len = dist(*a, *b);
            if d <= tol && t * len > tol && (1.0 - t) * len > tol {
                sides[k].push((t, n));
            }
        }
    }
    for (k, list) in sides.iter().enumerate() {
        for &(_, n) in list {
            nodes[n].joints.push(Joint::RunSide { run: all[k].0.clone() });
        }
    }
    // Each run from its start through the nodes on it to its end; each fitting from its first connector to the others.
    let mut edges = Vec::new();
    for (k, (path, a, b, _, _, ends, _)) in all.iter().enumerate() {
        let (first, last) = (at_point[*ends], at_point[*ends + 1]);
        let mut stops: Vec<(f64, usize)> = sides[k].clone();
        stops.sort_by(|x, y| x.0.total_cmp(&y.0));
        let len = dist(*a, *b);
        let mut prev = (0.0, first);
        for s in stops.into_iter().chain(std::iter::once((1.0, last))) {
            edges.push(Edge { element: path.clone(), from: prev.1, to: s.1, length: Some((s.0 - prev.0) * len) });
            prev = s;
        }
    }
    for group in &through {
        for &i in &group[1..] {
            edges.push(Edge { element: points[i].1.element().to_string(), from: at_point[group[0]], to: at_point[i], length: None });
        }
    }
    // The groups: nodes joined by edges.
    let mut joined = Sets::new(nodes.len());
    for e in &edges {
        joined.join(e.from, e.to);
    }
    let mut group_of: BTreeMap<usize, usize> = BTreeMap::new();
    let mut members: Vec<Gathered> = Vec::new();
    let system_of: HashMap<&str, &str> = all.iter().filter_map(|(path, _, _, _, s, _, _)| s.as_deref().map(|s| (path.as_str(), s))).collect();
    for (n, node) in nodes.iter().enumerate() {
        let root = joined.find(n);
        let g = *group_of.entry(root).or_insert_with(|| {
            members.push(Default::default());
            members.len() - 1
        });
        let entry = &mut members[g];
        entry.3.push(n);
        for j in &node.joints {
            entry.0.insert(j.element().to_string());
            if let Some(s) = system_of.get(j.element()).filter(|s| !s.is_empty()) {
                entry.1.insert(s.to_string());
            }
            if let Joint::Connector { domain: Some(d), .. } = j {
                entry.2.insert(*d);
            }
        }
    }
    // A system asked for keeps only the groups its runs are in.
    let keep: Vec<bool> = members.iter().map(|(_, systems, _, _)| system.is_none_or(|s| systems.contains(s))).collect();
    let mut warnings = Vec::new();
    for (g, (elements, systems, domains, ns)) in members.iter().enumerate() {
        if !keep[g] {
            continue;
        }
        let at = ns.first().map(|&n| nodes[n].at);
        if systems.len() > 1 {
            let names: Vec<String> = systems.iter().map(|s| tree.called(s)).collect();
            warnings.push(Warning { kind: "systems_meet", at, elements: elements.iter().cloned().collect(), message: format!("runs of the systems {} are joined in one network", names.join(", ")) });
        }
        if domains.len() > 1 {
            let names: Vec<String> = domains.iter().map(|d| format!("{d:?}").to_lowercase()).collect();
            warnings.push(Warning { kind: "domains_meet", at, elements: elements.iter().cloned().collect(), message: format!("connectors of {} are joined in one network", names.join(", ")) });
        }
    }
    for (n, node) in nodes.iter().enumerate() {
        if !keep[group_of[&joined.find(n)]] {
            continue;
        }
        if node.joints.iter().any(|j| matches!(j, Joint::RunSide { .. })) {
            let mut elements: Vec<String> = node.joints.iter().map(|j| j.element().to_string()).collect();
            elements.dedup();
            warnings.push(Warning { kind: "joined_mid_run", at: Some(node.at), elements, message: "a run or a connector meets another run along its length, with no fitting there".into() });
        }
    }
    // A drain (a group with a waste connector) flows down: no pipe leaving a node downhill is narrower than one coming down to it.
    let width: HashMap<&str, f64> = all.iter().filter_map(|(path, _, _, w, _, _, _)| w.map(|w| (path.as_str(), w))).collect();
    // Each node's run pieces, in the edges' order, found once for all the nodes.
    let mut pieces: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (i, e) in edges.iter().enumerate() {
        if e.length.is_some() {
            pieces[e.from].push(i);
            if e.to != e.from {
                pieces[e.to].push(i);
            }
        }
    }
    let mut seen_drop = BTreeSet::new();
    for (n, node) in nodes.iter().enumerate() {
        let g = group_of[&joined.find(n)];
        if !keep[g] || !members[g].2.contains(&Domain::Waste) {
            continue;
        }
        let mut down: Vec<(&str, f64)> = Vec::new();
        let mut up: Vec<(&str, f64)> = Vec::new();
        for e in pieces[n].iter().map(|&i| &edges[i]) {
            let other = if e.from == n { e.to } else { e.from };
            let dz = nodes[other].at[2] - node.at[2];
            // A run nobody has sized is not compared.
            let Some(w) = width.get(e.element.as_str()).copied() else { continue };
            if dz < -tol {
                down.push((&e.element, w));
            } else if dz > tol {
                up.push((&e.element, w));
            }
        }
        let widest_in = up.iter().map(|x| x.1).fold(0.0, f64::max);
        for (path, w) in &down {
            if *w + 1e-9 < widest_in && seen_drop.insert((n, path.to_string())) {
                let mut elements: Vec<String> = up.iter().map(|x| x.0.to_string()).collect();
                elements.push(path.to_string());
                let inches = |feet: f64| crate::view::num((feet * 12.0 * 1.0e4).round() / 1.0e4);
                warnings.push(Warning { kind: "smaller_downstream", at: Some(node.at), elements, message: format!("a drain narrows downstream: {} in leaving where {} in comes in", inches(*w), inches(widest_in)) });
            }
        }
    }
    let open = (0..nodes.len()).filter(|&n| keep[group_of[&joined.find(n)]] && nodes[n].joints.len() == 1).collect();
    let groups = members
        .into_iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|((elements, systems, domains, _), _)| Group { elements: elements.into_iter().collect(), systems: systems.into_iter().collect(), domains: domains.into_iter().collect() })
        .collect();
    Ok(Network { nodes, edges, groups, open, warnings, left_out })
}
