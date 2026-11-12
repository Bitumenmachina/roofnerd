//! Joining runs: a run cut in two where a branch meets it, a fitting where runs' ends
//! meet, and a run's end joined to a fixture or a piece of equipment. After these the
//! job's network (`building/network.rs`) says what is joined and what is still open, and
//! the takeoff counts fittings by kind.
//!
//! PORTED CODE, in part. `split` follows the rule of BIMCraft's `split`
//! (`crates/core/src/commands.rs:12534`, of a wall): cut between the ends, the first piece
//! keeps its identity, the second is new, and what hangs on it goes with its piece. It is
//! from commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10),
//! https://github.com/IPedrax/bimcraft, Copyright (c) 2026 IPedrax and the BIMCraft
//! contributors, used under the MIT license (BIMCraft is MIT OR Apache-2.0). The notice is
//! in NOTICE at the repository root. `PORTMAP.md` gives the line.
//!
//! The rest is ours. BIMCraft places a pipe fitting by hand, as a component of a family
//! with its connectors drawn in the family (`fixtures.rs:203`, `family.rs:132`); it does
//! not work out which fitting a joint takes, and it has no verb that joins a run to a
//! thing. Here a fitting's kind and connectors are worked out from the runs that meet,
//! because an estimator counts fittings from the runs and has no family library to pick
//! from. What is not known is not made up: a fitting is as sure as the runs it joins, and
//! which part is bought is said to be not selected.

use std::collections::BTreeSet;

use super::{add_numbered, all_or_nothing, storey_of, Entered};
use crate::building::{self, network, InSystem, SystemInfo};
use crate::check::FIT_TOLERANCE_FT;
use crate::components::*;
use crate::formula;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Status, Val};
use crate::{Connector, ConnectorKind, Connectors, Domain, Fitting, FittingKind, Flow, Level, Placed, Segment, SegmentType, P3};

/// Two runs are in line when the angle between them is under a degree: the cosine of it.
const IN_LINE: f64 = 0.999_847_695_156_391_3;
/// The most runs one fitting cuts on its way in: a cross on two through runs.
const MOST_SPLITS: usize = 4;

/// A number as it is written to the file: no stray digits from the arithmetic.
fn tidy(x: f64) -> f64 {
    (x * 1.0e6).round() / 1.0e6 + 0.0
}

/// What is written by a verb here is never surer than what it was worked out from.
fn written(worst: Status) -> Status {
    if worst == Status::Known {
        Status::Known
    } else {
        Status::Assumed
    }
}

/// A run as these verbs need it: its line, its type when it has one, the system it is
/// on, its two ends in space, and the worst status among what places it.
struct RunInfo {
    kind: Kind,
    segment: Segment,
    section: Option<SegmentType>,
    system: Option<String>,
    a: P3,
    b: P3,
    status: Status,
}

fn run_info(job: &mut Job, path: &str) -> Result<RunInfo, String> {
    let index = job.index();
    let e = *index.get(path).ok_or_else(|| format!("no element {path}"))?;
    let kind = job.world.get::<Kind>(e).copied().ok_or_else(|| format!("{path} has no kind"))?;
    if !kind.is_segment() {
        return Err(format!("{path} is a {kind:?}, not a duct, a pipe, a conduit or a cable"));
    }
    let segment = job.world.get::<Segment>(e).cloned().ok_or_else(|| format!("{path} has no line"))?;
    let section = segment.type_name.value.as_ref().and_then(|t| index.get(t)).and_then(|t| job.world.get::<SegmentType>(*t)).cloned();
    let system = job.world.get::<InSystem>(e).and_then(|s| s.0.value.clone());
    let tree = building::Tree::of(&mut job.world);
    let level = tree.storey_of(path).and_then(|s| tree.levels.get(s));
    let base = level.and_then(|l| l.elevation.feet()).ok_or_else(|| format!("{path} is under no storey whose floor height is known"))?;
    // Where a run is in space is no surer than its storey's floor height.
    let floor_status = level.map(|l| l.elevation.status).unwrap_or(Status::Unknown);
    let [p, q] = segment.line_ft().ok_or_else(|| format!("{path}'s line is not known"))?;
    let (z0, z1) = segment.offsets_ft().ok_or_else(|| format!("{path}'s height above its storey is not known"))?;
    let mut status = segment.type_name.status.max(segment.line.status).max(segment.offset.status).max(floor_status);
    if let Some(end) = &segment.end_offset {
        status = status.max(end.status);
    }
    if let Some(t) = &section {
        status = status.max(t.status());
    }
    Ok(RunInfo { kind, segment, section, system, a: [p[0], p[1], base + z0], b: [q[0], q[1], base + z1], status })
}

/// Cut a run in two at a point on its line (in feet, on the plan). The first piece keeps
/// the run's path and name; the second is a new run of the same type, on the same system,
/// carrying what the first carries. Returns the second piece's path.
///
/// A run with a cost line whose quantity is a typed number is not cut: the number would
/// be counted on both pieces, and which piece it belongs to is not something this can
/// tell. A quantity that is a formula is worked out again for each piece.
pub fn split(job: &mut Job, run: &str, at: P2, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| split_at(job, run, at, how))
}

fn split_at(job: &mut Job, run: &str, at: P2, how: Entered) -> Result<String, String> {
    let info = run_info(job, run)?;
    let e = job.find(run).ok_or_else(|| format!("no element {run}"))?;
    let [a, b] = info.segment.line_ft().ok_or_else(|| format!("{run}'s line is not known, so it cannot be cut"))?;
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    if len <= FIT_TOLERANCE_FT {
        return Err(format!("{run} stands upright: it has no length on the plan to be cut along"));
    }
    let t = ((at[0] - a[0]) * dx + (at[1] - a[1]) * dy) / (len * len);
    let on = [a[0] + dx * t, a[1] + dy * t];
    if (at[0] - on[0]).hypot(at[1] - on[1]) > FIT_TOLERANCE_FT {
        return Err(format!("({}, {}) is not on {run}", tidy(at[0]), tidy(at[1])));
    }
    if t * len <= FIT_TOLERANCE_FT || t * len >= len - FIT_TOLERANCE_FT {
        return Err(format!("cut {run} between its ends"));
    }
    if let Some(cost) = job.world.get::<Cost>(e) {
        for line in &cost.lines {
            // A quantity that names no measure is a typed number however it is written:
            // `4`, `2 * 2`, `-(-4)`.
            if formula::parse(&line.quantity).is_ok_and(|q| formula::names(&q).is_empty()) {
                return Err(format!(
                    "{run} has a cost line, {:?}, whose quantity is a typed number ({}): cut in two it would be counted on both pieces. Take the line off or make its quantity a formula first",
                    line.description, line.quantity
                ));
            }
        }
    }
    let divisor = units::coord_divisor(&info.segment.line.unit)?;
    let raw = info.segment.line.value.ok_or_else(|| format!("{run}'s line is not known"))?;
    let cut = [tidy(on[0] * divisor), tidy(on[1] * divisor)];
    // The cut is no surer than whoever asked for it.
    let line = |ends: [P2; 2]| {
        let l = &info.segment.line;
        let (status, source) = (l.status.max(how.status), super::after(&l.source, "cut", how.source));
        Val { value: Some(ends), unit: l.unit.clone(), status, source }
    };
    let (mut first, mut second) = (info.segment.clone(), info.segment.clone());
    first.line = line([raw[0], cut]);
    second.line = line([cut, raw[1]]);
    // A run that climbs is as high at the cut as its line is there.
    let (z0, z1) = info.segment.offsets_ft().ok_or_else(|| format!("{run}'s height above its storey is not known, so it cannot be cut"))?;
    if let (Some(end), true) = (&info.segment.end_offset, (z1 - z0).abs() > 1.0e-9) {
        let status = written(info.segment.offset.status.max(end.status).max(info.segment.line.status).max(how.status));
        let at_cut = Val { value: Some(Len::One(tidy(z0 + (z1 - z0) * t))), unit: units::FT.into(), status, source: format!("worked out where {run} was cut ({})", how.source) };
        first.end_offset = Some(at_cut.clone());
        second.offset = at_cut;
    }
    let parent = job.world.get::<Node>(e).and_then(|n| n.parent.clone()).ok_or_else(|| format!("{run} is under nothing"))?;
    let new = add_numbered(job, &parent, info.kind)?;
    // A run somebody named keeps its name on both pieces: the second says it was cut off
    // the first. One with only its number for a name gets its own number.
    let was = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_default();
    let own_number = run.rsplit('-').next().and_then(|n| n.parse::<u32>().ok()).map(|n| format!("{} {n}", info.kind.stem()));
    if !was.is_empty() && Some(&was) != own_number.as_ref() {
        let number = new.rsplit('-').next().and_then(|n| n.parse::<u32>().ok()).unwrap_or(0);
        if let Some(mut node) = job.find(&new).and_then(|n| job.world.get_mut::<Node>(n)) {
            node.name = format!("{was}, cut at {number}");
        }
    }
    building::check_segment(run, info.kind, &first)?;
    building::check_segment(&new, info.kind, &second)?;
    job.put(run, first)?;
    job.put(&new, second)?;
    // What hangs on the run goes on both pieces: each is still that run.
    macro_rules! carry {
        ($($component:ty),*) => {$(
            if let Some(c) = job.world.get::<$component>(e).cloned() {
                job.put(&new, c)?;
            }
        )*};
    }
    carry!(InSystem, Condition, Cost, Allowance, MadeOf, WorkCode, Foreign);
    Ok(new)
}

/// What `fitting` did: the fitting it made, what kind the joint took, and the runs it cut
/// on the way (each with its new second piece).
#[derive(Clone, Debug, PartialEq)]
pub struct Fitted {
    pub path: String,
    pub kind: FittingKind,
    pub cut: Vec<(String, String)>,
}

/// As a part is asked for at a counter: an elbow, a tee.
fn part(kind: FittingKind) -> &'static str {
    match kind {
        FittingKind::Bend => "elbow",
        FittingKind::Tee => "tee",
        FittingKind::Cross => "cross",
        FittingKind::Transition => "transition",
        FittingKind::Coupling => "coupling",
    }
}

/// A fitting where runs meet on `storey`, at a point on the plan (in feet) and, when
/// more than one joint lies over that point, at `height` above the storey's floor.
///
/// What kind it is is worked out from what meets: two ends at an angle are a bend, two in
/// line of one size a coupling, two in line of two sizes a transition, three a tee, four
/// a cross. A `kind` given is checked against that. A run that passes through the point
/// is cut there first, so a branch drawn to a run's side becomes a tee of three ends.
pub fn fitting(job: &mut Job, storey: &str, at: P2, height: Option<&str>, kind: Option<FittingKind>, how: Entered) -> Result<Fitted, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let z = match height {
            None => None,
            Some(typed) => {
                let (len, unit) = units::parse_length(typed)?;
                Some(len.feet(unit)?)
            }
        };
        fit_at(job, storey, at, z, kind, how)
    })
}

fn floor_of(job: &mut Job, storey: &str) -> Result<f64, String> {
    let e = job.find(storey).ok_or_else(|| format!("no element {storey}"))?;
    job.world.get::<Level>(e).and_then(|l| l.elevation.feet()).ok_or_else(|| format!("{storey}'s floor height is not known"))
}

fn fit_at(job: &mut Job, storey: &str, at: P2, z: Option<f64>, kind: Option<FittingKind>, how: Entered) -> Result<Fitted, String> {
    let base = floor_of(job, storey)?;
    let place = format!("({}, {})", tidy(at[0]), tidy(at[1]));
    let mut cut: Vec<(String, String)> = Vec::new();
    // The joint: after any run passing through the point has been cut there.
    let node = loop {
        let net = network::network(job, None, Some(storey), network::TOLERANCE_FT)?;
        let here: Vec<&network::Node> =
            net.nodes.iter().filter(|n| (n.at[0] - at[0]).hypot(n.at[1] - at[1]) <= FIT_TOLERANCE_FT && z.is_none_or(|z| (n.at[2] - base - z).abs() <= FIT_TOLERANCE_FT)).collect();
        let node = match here.as_slice() {
            [] => return Err(format!("nothing meets at {place}: a fitting goes where runs meet")),
            [one] => (*one).clone(),
            many => {
                // Two kinds of run ending at one place and one height: a height does
                // not tell them apart, and asking for one would be asking for nothing.
                let level = many.iter().all(|n| (n.at[2] - many[0].at[2]).abs() <= FIT_TOLERANCE_FT);
                if level {
                    let kinds: Vec<String> = many.iter().map(|n| format!("a {}'s", n.kind.stem())).collect();
                    return Err(format!("{} joints are both at {place}, {} ft above the floor, and which is meant cannot be said here: put each fitting on before the other run is drawn to that place", kinds.join(" and "), tidy(many[0].at[2] - base)));
                }
                let heights: Vec<String> = many.iter().map(|n| format!("{} ft", tidy(n.at[2] - base))).collect();
                return Err(format!("{} joints lie over one another at {place}, at {} above the floor: say which with a height", many.len(), heights.join(" and ")));
            }
        };
        let through = node.joints.iter().find_map(|j| match j {
            network::Joint::RunSide { run } => Some(run.clone()),
            _ => None,
        });
        match through {
            None => break node,
            Some(run) => {
                if cut.len() >= MOST_SPLITS {
                    return Err(format!("more than {MOST_SPLITS} runs pass through {place}: that is not one fitting"));
                }
                let new = split_at(job, &run, [node.at[0], node.at[1]], how)?;
                cut.push((run, new));
            }
        }
    };
    if node.kind == Kind::Cable {
        return Err(format!("cables meet at {place}: a cable is not joined by a fitting"));
    }
    // What meets: runs' ends, and nothing else.
    let mut ends: Vec<(String, bool)> = Vec::new();
    for joint in &node.joints {
        match joint {
            network::Joint::RunEnd { run, end } => ends.push((run.clone(), *end)),
            network::Joint::Connector { element, .. } => {
                return Err(match job.kind(element) {
                    Some(Kind::Fitting) => format!("{element} is already the fitting at {place}"),
                    _ => format!("{element} is joined at {place} by its connector: a fitting joins runs' ends to each other"),
                });
            }
            network::Joint::RunSide { run } => return Err(format!("{run} still passes through {place}")),
        }
    }
    if !(2..=4).contains(&ends.len()) {
        return Err(match ends.as_slice() {
            [(run, _)] => format!("only {run} ends at {place}: a fitting joins two, three or four runs' ends"),
            _ => format!("{} runs' ends meet at {place}: a fitting joins two, three or four", ends.len()),
        });
    }
    let mut runs: Vec<RunInfo> = Vec::new();
    for (run, _) in &ends {
        runs.push(run_info(job, run)?);
    }
    // Which way each run leaves the joint.
    let away: Vec<[f64; 3]> = runs
        .iter()
        .zip(&ends)
        .map(|(r, (_, end))| {
            let (from, to) = if *end { (r.b, r.a) } else { (r.a, r.b) };
            let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1.0e-12);
            [d[0] / len, d[1] / len, d[2] / len]
        })
        .collect();
    // Each run's section, in feet: across, deep, and whether it is round.
    let sections: Vec<Option<(f64, f64, bool)>> = runs.iter().map(|r| r.section.as_ref().and_then(|t| t.round_ft().map(|d| (d, d, true)).or_else(|| t.rect_ft().map(|(w, d)| (w, d, false))))).collect();
    // No two of them leave it the same way: those lie one over the other, which is a
    // clash and not a joint.
    let dot_of = |i: usize, j: usize| away[i][0] * away[j][0] + away[i][1] * away[j][1] + away[i][2] * away[j][2];
    for i in 0..ends.len() {
        for j in i + 1..ends.len() {
            if dot_of(i, j) >= IN_LINE {
                return Err(format!("{} and {} leave {place} the same way, one over the other", ends[i].0, ends[j].0));
            }
        }
    }
    let worked = match ends.len() {
        3 => FittingKind::Tee,
        4 => FittingKind::Cross,
        _ => {
            if dot_of(0, 1) > -IN_LINE {
                FittingKind::Bend
            } else {
                match (sections[0], sections[1]) {
                    (Some(a), Some(b)) => {
                        if (a.0 - b.0).abs() <= 1.0e-6 && (a.1 - b.1).abs() <= 1.0e-6 && a.2 == b.2 {
                            FittingKind::Coupling
                        } else {
                            FittingKind::Transition
                        }
                    }
                    _ => match kind {
                        Some(k @ (FittingKind::Coupling | FittingKind::Transition)) => k,
                        _ => return Err(format!("{} and {} are in line at {place} and the size of one is not known, so this is a coupling or a transition and it cannot be told which: say which, or give the runs their size", ends[0].0, ends[1].0)),
                    },
                }
            }
        }
    };
    if let Some(asked) = kind {
        if asked != worked {
            return Err(format!("what meets at {place} makes a {}, not a {}: {} runs' ends meet there", worked.title().to_lowercase(), asked.title().to_lowercase(), ends.len()));
        }
    }
    let systems: BTreeSet<&String> = runs.iter().filter_map(|r| r.system.as_ref()).collect();
    if systems.len() > 1 {
        let names: Vec<String> = systems.iter().map(|s| s.to_string()).collect();
        return Err(format!("runs on the systems {} meet at {place}: a fitting is on one system", names.join(" and ")));
    }
    let system = systems.into_iter().next().cloned();

    let status = written(runs.iter().map(|r| r.status).max().unwrap_or(Status::Known).max(how.status));
    let joined: Vec<&str> = ends.iter().map(|(run, _)| run.as_str()).collect();
    let source = format!("{}; put where {} meet; which {} is bought is not specified", how.source, joined.join(", "), part(worked));
    let val = |value| Val { value: Some(value), unit: String::new(), status, source: source.clone() };

    // Its size is its largest run's; what it is called names every size it joins.
    let largest = sections.iter().flatten().copied().fold(None, |best: Option<(f64, f64, bool)>, s| match best {
        Some(b) if b.0 * b.1 >= s.0 * s.1 => Some(b),
        _ => Some(s),
    });
    let all_sized = sections.iter().all(Option::is_some);
    let (what, size, deep_ft) = match (largest, all_sized) {
        (Some((across, deep, _)), true) => {
            let mut said: Vec<String> = Vec::new();
            let mut order: Vec<usize> = (0..runs.len()).collect();
            order.sort_by(|&i, &j| {
                let area = |k: usize| sections[k].map(|s| s.0 * s.1).unwrap_or(0.0);
                area(j).total_cmp(&area(i))
            });
            for i in order {
                let Some(t) = &runs[i].section else { continue };
                // A pipe's or a conduit's size is its diameter and is said without "round".
                let one = if node.kind == Kind::Duct { t.said() } else { t.said().replace(" round", "") };
                if !said.contains(&one) {
                    said.push(one);
                }
            }
            let what = val(format!("{} {}", said.join(if worked == FittingKind::Transition { " to " } else { " x " }), part(worked)));
            let size = Val { value: Some([tidy(across * 12.0), tidy(across * 12.0), tidy(deep * 12.0)]), unit: units::IN.into(), status: Status::Assumed, source: "drawn at its run's size; the fitting is not specified".to_string() };
            (what, size, deep)
        }
        _ => (Val::unknown("", "a run it joins is drawn on the plan with no size"), Val::unknown(units::IN, "a run it joins is drawn on the plan with no size"), 0.0),
    };
    let of_run = ConnectorKind::for_run(node.kind).ok_or_else(|| format!("a {:?} has no connector that joins it", node.kind))?;
    let connectors: Vec<Connector> = runs
        .iter()
        .map(|r| Connector {
            name: String::new(),
            kind: of_run,
            domain: None,
            flow: Flow::Both,
            at: Val { value: Some([0.0, 0.0, tidy(deep_ft * 6.0)]), unit: units::IN.into(), status, source: "on the joint".to_string() },
            size: r.section.as_ref().map(|t| t.width.clone()),
        })
        .collect();
    let placed = Placed {
        what,
        at: Val { value: Some([tidy(node.at[0]), tidy(node.at[1])]), unit: units::FT.into(), status, source: source.clone() },
        offset: Val { value: Some(Len::One(tidy(node.at[2] - base - deep_ft / 2.0))), unit: units::FT.into(), status, source: source.clone() },
        // Turned to lie along the first of its runs that has a way on the plan, so its
        // box is square to that run and not to the page. None when every run is upright.
        angle: away.iter().find(|d| d[0].hypot(d[1]) > 1.0e-6).map(|d| tidy(d[1].atan2(d[0]).to_degrees())).filter(|a| a.abs() > 1.0e-9).map(|a| Val { value: Some(a), unit: "deg".into(), status, source: "turned to lie along the first run it joins".to_string() }),
        size,
    };
    let path = add_numbered(job, storey, Kind::Fitting)?;
    building::check_placed(&path, Kind::Fitting, &placed)?;
    let fitting = Fitting { kind: worked };
    building::check_fitting(&path, Kind::Fitting, &fitting)?;
    let connectors = Connectors(connectors);
    building::check_connectors(&path, Kind::Fitting, &connectors)?;
    job.put(&path, placed)?;
    job.put(&path, fitting)?;
    job.put(&path, connectors)?;
    if let Some(system) = system {
        let on = InSystem(val(system));
        building::check_in_system(&path, Kind::Fitting, &on)?;
        job.put(&path, on)?;
    }
    Ok(Fitted { path, kind: worked, cut })
}

/// A joint that `fit_all` passed over, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct PassedOver {
    pub storey: String,
    pub at: P3,
    pub why: String,
}

/// A fitting at every joint that has none: on one system when `system` is given, else on
/// every run in the job. A joint that cannot take a fitting (five ends, runs of two
/// systems, two runs in line with no size) is passed over and listed with the reason, and
/// the job is left as it was at that joint. Cables are not joined by fittings and are not
/// listed. Joints are looked for a storey at a time: a riser that meets a run of the
/// storey above is not joined to it here.
pub fn fit_all(job: &mut Job, system: Option<&str>, how: Entered) -> Result<(Vec<Fitted>, Vec<PassedOver>), String> {
    all_or_nothing(job, |job| {
        if let Some(system) = system {
            let e = job.find(system).ok_or_else(|| format!("no element {system}"))?;
            if job.world.get::<SystemInfo>(e).is_none() {
                return Err(format!("{system} is not a system"));
            }
        }
        let storeys: Vec<String> = {
            let order = job.order.clone();
            order.iter().filter(|e| job.world.get::<Kind>(**e) == Some(&Kind::Storey)).filter_map(|e| job.world.get::<Node>(*e).map(|n| n.path.clone())).collect()
        };
        let (mut made, mut passed) = (Vec::new(), Vec::new());
        for storey in storeys {
            let base = match floor_of(job, &storey) {
                Ok(base) => base,
                Err(_) => continue,
            };
            let mut tried: Vec<P3> = Vec::new();
            loop {
                let net = network::network(job, system, Some(&storey), network::TOLERANCE_FT)?;
                let next = net.nodes.iter().find(|n| {
                    let ends = n.joints.iter().filter(|j| matches!(j, network::Joint::RunEnd { .. })).count();
                    let sides = n.joints.iter().filter(|j| matches!(j, network::Joint::RunSide { .. })).count();
                    let things = n.joints.iter().any(|j| matches!(j, network::Joint::Connector { .. }));
                    n.kind != Kind::Cable && !things && ends >= 1 && ends + sides >= 2 && !tried.iter().any(|t| (t[0] - n.at[0]).abs() + (t[1] - n.at[1]).abs() + (t[2] - n.at[2]).abs() <= FIT_TOLERANCE_FT)
                });
                let Some(node) = next else { break };
                let at = node.at;
                tried.push(at);
                // Two runs of one size in line are most often one run drawn in two picks: a
                // coupling is put only where it is asked for by name.
                let one = |job: &mut Job| {
                    fit_at(job, &storey, [at[0], at[1]], Some(at[2] - base), None, how).and_then(|f| match f.kind {
                        FittingKind::Coupling => Err("two runs of one size meet in line here, which is most often one run drawn in two pieces: a coupling is put only where it is asked for".to_string()),
                        _ => Ok(f),
                    })
                };
                match all_or_nothing(job, one) {
                    Ok(fitted) => made.push(fitted),
                    Err(why) => passed.push(PassedOver { storey: storey.clone(), at, why }),
                }
            }
        }
        Ok((made, passed))
    })
}

/// Join a run's end to a fixture or a piece of equipment: the thing is given a connector
/// where the run's end is, of the run's kind and size, carrying what the run's system
/// carries. The end has to be at the thing: inside its box, or within the fit tolerance
/// of it. Returns which of the thing's connectors it is, counted from one.
pub fn connect(job: &mut Job, thing: &str, run: &str, how: Entered) -> Result<usize, String> {
    all_or_nothing(job, |job| {
        let kind = job.kind(thing).ok_or_else(|| format!("no element {thing}"))?;
        if !matches!(kind, Kind::Fixture | Kind::Equipment) {
            return Err(format!("{thing} is a {kind:?}: a run is joined to a fixture or a piece of equipment, and runs to each other by a fitting"));
        }
        let r = run_info(job, run)?;
        let e = job.find(thing).ok_or_else(|| format!("no element {thing}"))?;
        // A duct is not joined to a thing on a water system: the thing's system has to be
        // one a run of this kind goes on.
        if let Some(on) = job.world.get::<InSystem>(e).and_then(|s| s.0.value.clone()) {
            let class = job.find(&on).and_then(|s| job.world.get::<SystemInfo>(s)).map(|s| s.classification.clone());
            let goes_on = building::run_goes_on(r.kind);
            if let Some(class) = class.filter(|c| !goes_on.is_empty() && !goes_on.contains(&c.as_str())) {
                return Err(format!("{thing} is on {on}, a {class} system: a {} is not joined to it (a {} goes on: {})", r.kind.stem(), r.kind.stem(), goes_on.join(", ")));
            }
        }
        let placed = job.world.get::<Placed>(e).cloned().ok_or_else(|| format!("{thing} has no position"))?;
        let tree = building::Tree::of(&mut job.world);
        let base = tree.storey_of(thing).and_then(|s| tree.levels.get(s)).and_then(|l| l.elevation.feet()).ok_or_else(|| format!("{thing} is under no storey whose floor height is known"))?;
        let at = placed.at_ft().ok_or_else(|| format!("where {thing} stands is not known"))?;
        let offset = placed.offset.feet().ok_or_else(|| format!("{thing}'s height above its storey is not known"))?;
        let angle = placed.angle_deg().ok_or_else(|| format!("how far {thing} is turned is not known"))?;
        let size = placed.size_ft().ok_or_else(|| format!("{thing}'s size is not known, so whether {run} reaches it cannot be told"))?;
        let (sin, cos) = angle.to_radians().sin_cos();
        // A point in space, in the thing's own axes: along it, to its left, up from its base.
        let local = |p: P3| {
            let (dx, dy) = (p[0] - at[0], p[1] - at[1]);
            [dx * cos + dy * sin, -dx * sin + dy * cos, p[2] - base - offset]
        };
        let tol = FIT_TOLERANCE_FT;
        let inside = |q: [f64; 3]| q[0].abs() <= size[0] / 2.0 + tol && q[1].abs() <= size[1] / 2.0 + tol && q[2] >= -tol && q[2] <= size[2] + tol;
        let (qa, qb) = (local(r.a), local(r.b));
        let q = match (inside(qa), inside(qb)) {
            (true, false) => qa,
            (false, true) => qb,
            (true, true) => return Err(format!("both ends of {run} are inside {thing}")),
            (false, false) => return Err(format!("neither end of {run} is at {thing}: its ends are at ({}, {}) and ({}, {})", tidy(r.a[0]), tidy(r.a[1]), tidy(r.b[0]), tidy(r.b[1]))),
        };
        let of_run = ConnectorKind::for_run(r.kind).ok_or_else(|| format!("a {:?} has no connector that joins it", r.kind))?;
        let mut connectors = job.world.get::<Connectors>(e).cloned().unwrap_or(Connectors(Vec::new()));
        if connectors.0.iter().any(|c| c.kind == of_run && c.at_ft().is_some_and(|c| (c[0] - q[0]).abs() + (c[1] - q[1]).abs() + (c[2] - q[2]).abs() <= tol)) {
            return Err(format!("{thing} already has a connector where {run} ends"));
        }
        // What a pipe connector carries is what the pipe's system carries, where that is one
        // of the things a connector can say.
        let its_system = r.system.as_ref().and_then(|s| job.find(s)).and_then(|s| job.world.get::<SystemInfo>(s).cloned());
        let domain = match (r.kind, &its_system) {
            (Kind::Pipe, Some(s)) => Domain::of_system(&s.classification),
            _ => None,
        };
        let name = its_system.map(|s| s.classification).unwrap_or_default();
        let mut status = r.status.max(placed.at.status).max(placed.offset.status).max(placed.size.status).max(how.status);
        if let Some(a) = &placed.angle {
            status = status.max(a.status);
        }
        connectors.0.push(Connector {
            name,
            kind: of_run,
            domain,
            flow: Flow::Both,
            at: Val { value: Some([tidy(q[0] * 12.0), tidy(q[1] * 12.0), tidy(q[2] * 12.0)]), unit: units::IN.into(), status: written(status), source: format!("{}; where {run} ends", how.source) },
            size: r.section.as_ref().map(|t| t.width.clone()),
        });
        building::check_connectors(thing, kind, &connectors)?;
        let which = connectors.0.len();
        job.put(thing, connectors)?;
        Ok(which)
    })
}

/// Join the run that ends at a point of the plan (in feet) to the fixture or the piece of
/// equipment that stands there: `connect`, for a pointer that picks the place and not
/// the two names. Returns the thing, the run, and which of the thing's connectors it is.
/// More than one run ending there, or more than one thing standing there, is refused:
/// which is meant cannot be told from a point.
pub fn connect_at(job: &mut Job, storey: &str, at: P2, how: Entered) -> Result<(String, String, usize), String> {
    storey_of(job, storey)?;
    let tree = building::Tree::of(&mut job.world);
    let order = job.order.clone();
    let tol = FIT_TOLERANCE_FT;
    let place = format!("({}, {})", tidy(at[0]), tidy(at[1]));
    let (mut runs, mut things): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for e in order {
        let (Some(n), Some(kind)) = (job.world.get::<Node>(e), job.world.get::<Kind>(e).copied()) else { continue };
        if tree.storey_of(&n.path) != Some(storey) {
            continue;
        }
        if kind.is_segment() {
            let ends = job.world.get::<Segment>(e).and_then(|s| s.line_ft());
            if ends.is_some_and(|[p, q]| (p[0] - at[0]).hypot(p[1] - at[1]) <= tol || (q[0] - at[0]).hypot(q[1] - at[1]) <= tol) {
                runs.push(n.path.clone());
            }
        } else if matches!(kind, Kind::Fixture | Kind::Equipment) {
            let Some(placed) = job.world.get::<Placed>(e) else { continue };
            let (Some(c), Some(size), Some(angle)) = (placed.at_ft(), placed.size_ft(), placed.angle_deg()) else { continue };
            let (sin, cos) = angle.to_radians().sin_cos();
            let (dx, dy) = (at[0] - c[0], at[1] - c[1]);
            let (along, across) = (dx * cos + dy * sin, -dx * sin + dy * cos);
            if along.abs() <= size[0] / 2.0 + tol && across.abs() <= size[1] / 2.0 + tol {
                things.push(n.path.clone());
            }
        }
    }
    match (runs.as_slice(), things.as_slice()) {
        ([run], [thing]) => connect(job, thing, run, how).map(|which| (thing.clone(), run.clone(), which)),
        ([], _) => Err(format!("no run ends at {place}")),
        (_, []) => Err(format!("no fixture or equipment of a known size stands at {place}")),
        _ => Err(format!("{} runs end and {} things stand at {place}: which is meant cannot be told from a point; they are joined by naming the thing and the run", runs.len(), things.len())),
    }
}
