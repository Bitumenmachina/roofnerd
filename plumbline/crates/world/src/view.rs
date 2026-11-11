//! A flat reading of the world: every value on every element, entered or derived, with
//! its status. The command line prints this; the sheet and the 3D view will read the
//! same rows. Nothing here computes a quantity. It only reports what the world holds.

use crate::components::*;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Derived, Status, Val};

#[derive(Clone, Debug)]
pub struct Row {
    pub path: String,
    pub label: String,
    /// The value as text, or "unknown" when there is none.
    pub text: String,
    pub status: Status,
    /// For an entered value, its source. For a derived one, its formula.
    pub basis: String,
    /// Why a derived value is flagged, when it says.
    pub note: String,
    pub derived: bool,
    /// A derived figure as a number with its unit, for a view that sets its own decimal
    /// places. None for an entered value, and for a derived one that has no figure.
    pub number: Option<f64>,
    pub unit: String,
}

#[derive(Clone, Debug)]
pub struct ElementRows {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    pub depth: usize,
    pub rows: Vec<Row>,
    /// Things to read that are not values on the job: a condition's recipe.
    pub notes: Vec<String>,
}

fn num(x: f64) -> String {
    // Adding zero shows a "-0" from a file as plain 0.
    let s = format!("{:.4}", x + 0.0);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

fn entered<T>(path: &str, label: &str, v: &Val<T>, show: impl Fn(&T) -> String) -> Row {
    let text = match &v.value {
        Some(x) => format!("{} {}", show(x), v.unit).trim().to_string(),
        None => "unknown".to_string(),
    };
    Row { path: path.into(), label: label.into(), text, status: v.status, basis: v.source.clone(), note: String::new(), derived: false, number: None, unit: String::new() }
}

/// An entered length, shown in the unit it was entered in: `30 in`, `2.5 ft`, `25' 6 1/2"`.
fn length(path: &str, label: &str, v: &Val<Len>) -> Row {
    let text = match &v.value {
        Some(len) => units::show_length(len, &v.unit),
        None => "unknown".to_string(),
    };
    Row { path: path.into(), label: label.into(), text, status: v.status, basis: v.source.clone(), note: String::new(), derived: false, number: None, unit: String::new() }
}

fn derived(path: &str, label: &str, d: &Derived) -> Row {
    let text = match d.value {
        Some(x) => format!("{} {}", num(x), d.unit).trim().to_string(),
        None => "unknown".to_string(),
    };
    Row { path: path.into(), label: label.into(), text, status: d.status, basis: d.formula.clone(), note: d.note.clone(), derived: true, number: d.value, unit: d.unit.clone() }
}

/// Every element in job order, each with all of its values.
pub fn elements(job: &mut Job) -> Vec<ElementRows> {
    let mut out = Vec::new();
    let order = job.order.clone();
    let w = &job.world;
    // How deep each element sits. The tree was checked on load; the step limit is only
    // so that a world built wrongly in code cannot hang a view.
    let parents: std::collections::HashMap<String, Option<String>> = order.iter().filter_map(|e| w.get::<Node>(*e)).map(|n| (n.path.clone(), n.parent.clone())).collect();
    let depth_of = |mut parent: Option<String>| -> usize {
        let mut d = 0;
        while let Some(p) = parent {
            d += 1;
            if d > parents.len() {
                break;
            }
            parent = parents.get(&p).cloned().flatten();
        }
        d
    };
    for &e in &order {
        let node = w.get::<Node>(e).expect("every element has a node");
        let kind = *w.get::<Kind>(e).expect("every element has a kind");
        let p = &node.path;
        let mut rows = Vec::new();
        // A roof's footprint: what it is solved from, each value with its flag.
        if let Some(f) = w.get::<Footprint>(e) {
            rows.push(entered(p, "footprint", &f.points, |pts| format!("{} points", pts.len())));
            rows.push(length(p, "eave height", &f.eave_height));
            for (i, edge) in f.edges.iter().enumerate() {
                rows.push(entered(p, &format!("edge {}", i + 1), edge, |r| match r {
                    Rise::Pitch(x) => num(*x),
                    Rise::Gable => "gable".to_string(),
                }));
            }
        }
        if let Some(state) = w.get::<SolveState>(e) {
            if !state.not_solved.is_empty() {
                rows.push(Row { path: p.clone(), label: "solved roof".into(), text: "not solved".into(), status: Status::Unknown, basis: "the planes and lines under this roof".into(), note: state.not_solved.clone(), derived: true, number: None, unit: String::new() });
            }
            for note in &state.notes {
                rows.push(Row { path: p.clone(), label: "solved roof".into(), text: "see note".into(), status: Status::Assumed, basis: "the planes and lines under this roof".into(), note: note.clone(), derived: true, number: None, unit: String::new() });
            }
        }
        if let Some(o) = w.get::<Outline>(e) {
            rows.push(entered(p, "outline", &o.points, |pts| format!("{} points", pts.len())));
            rows.push(entered(p, "eave line", &o.eave, |[a, b]| format!("({}, {}) to ({}, {})", num(a[0]), num(a[1]), num(b[0]), num(b[1]))));
            rows.push(length(p, "eave height", &o.eave_height));
        }
        if let Some(c) = w.get::<Pitch>(e) {
            rows.push(entered(p, "pitch", &c.0, |x| num(*x)));
        }
        if let Some(c) = w.get::<Run>(e) {
            rows.push(entered(p, "line ends", &c.ends, |[a, b]| format!("({}, {}) to ({}, {})", num(a[0]), num(a[1]), num(b[0]), num(b[1]))));
        }
        if let Some(c) = w.get::<Height>(e) {
            rows.push(length(p, "height", &c.0));
        }
        if let Some(c) = w.get::<Condition>(e) {
            rows.push(entered(p, "condition", &c.0, |s| s.clone()));
        }
        if let Some(c) = w.get::<Cost>(e) {
            rows.push(entered(p, "cost code", &c.code, |k| format!("{} {} {}", k.scheme, k.code, k.title)));
            for line in &c.lines {
                if let Some(so) = &line.stretch_out {
                    rows.push(length(p, &format!("stretch-out: {}", line.description), so));
                }
                for (name, v) in &line.inputs {
                    rows.push(length(p, &format!("{name}: {}", line.description), v));
                }
                if let Some(w) = &line.waste {
                    rows.push(entered(p, &format!("waste: {}", line.description), w, |x| num(*x)));
                }
                if let Some(o) = &line.order {
                    rows.push(entered(p, &format!("order factor: {}", line.description), &o.factor, |x| num(*x)));
                }
                rows.push(entered(p, &format!("unit cost: {}", line.description), &line.unit_cost, |x| num(*x)));
            }
        }
        if let Some(c) = w.get::<WorkCode>(e) {
            rows.push(entered(p, "schedule code", &c.wbs, |s| s.clone()));
        }
        if let Some(m) = w.get::<Measure>(e) {
            for (name, d) in &m.0 {
                rows.push(derived(p, name, d));
            }
        }
        if let Some(q) = w.get::<LineQuantities>(e) {
            // In the order the lines are entered, each line's steps together.
            let order: Vec<&String> = match w.get::<Cost>(e) {
                Some(c) => c.lines.iter().map(|l| &l.description).collect(),
                None => q.0.keys().collect(),
            };
            for description in order {
                let Some(f) = q.0.get(description) else { continue };
                rows.push(derived(p, &format!("quantity: {description}"), &f.quantity));
                if let Some(d) = &f.with_waste {
                    rows.push(derived(p, &format!("with waste: {description}"), d));
                }
                if let Some(d) = &f.order {
                    rows.push(derived(p, &format!("to order: {description}"), d));
                }
            }
        }
        // A condition's recipe is read, not counted: its blanks are not this job's unknowns.
        let mut notes = Vec::new();
        if let Some(t) = w.get::<Recipe>(e) {
            notes.push(format!("applies to: {}", t.applies_to.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>().join(", ")));
            for line in &t.cost.lines {
                // What the line takes, and what the recipe already states for it. A value a
                // recipe states goes onto an element with that status, so it is shown here.
                let stated = |status: Status, text: String| if status == Status::Unknown { "blank".to_string() } else { format!("{}{text}", if status == Status::Assumed { "~ " } else { "" }) };
                let mut extra = Vec::new();
                if line.stretch_out.is_some() {
                    extra.push("stretch-out: blank".to_string());
                }
                for (name, v) in &line.inputs {
                    extra.push(format!("{name}: {}", stated(v.status, v.value.as_ref().map(|len| units::show_length(len, &v.unit)).unwrap_or_default())));
                }
                if let Some(w) = &line.waste {
                    extra.push(format!("waste: {}", stated(w.status, w.value.map(|x| format!("{} %", num(x))).unwrap_or_default())));
                }
                if let Some(o) = &line.order {
                    extra.push(format!("ordered by the {}: {}", o.unit, stated(o.factor.status, o.factor.value.map(|x| format!("{} {}", num(x), o.factor.unit)).unwrap_or_default())));
                }
                extra.push(format!("price: {}", stated(line.unit_cost.status, line.unit_cost.value.map(|x| format!("{} {}", num(x), line.unit_cost.unit)).unwrap_or_default())));
                notes.push(format!("line: {} = {}   ({})", line.description, line.quantity, extra.join("; ")));
            }
        }
        out.push(ElementRows { path: p.clone(), name: node.name.clone(), kind, depth: depth_of(node.parent.clone()), rows, notes });
    }
    out
}

/// How many values stand at each status: (Known, Assumed, Unknown).
pub fn tally(elements: &[ElementRows]) -> (usize, usize, usize) {
    let mut t = (0, 0, 0);
    for r in elements.iter().flat_map(|e| &e.rows) {
        match r.status {
            Status::Known => t.0 += 1,
            Status::Assumed => t.1 += 1,
            Status::Unknown => t.2 += 1,
        }
    }
    t
}

/// A sum over part of the job, honest about what it could not include.
#[derive(Clone, Debug, PartialEq)]
pub struct Sum {
    /// The total, when every member has a figure. None when any does not: a total that
    /// leaves something out is not a total.
    pub value: Option<f64>,
    pub unit: &'static str,
    /// The worst status among the members.
    pub status: Status,
    pub members: usize,
    /// The members that have no figure, by path.
    pub missing: Vec<String>,
}

fn sum_of(unit: &'static str, figures: &[(String, Option<f64>, Status)]) -> Sum {
    let missing: Vec<String> = figures.iter().filter(|f| f.1.is_none()).map(|f| f.0.clone()).collect();
    let status = if figures.is_empty() { Status::Known } else { figures.iter().map(|f| f.2).max().unwrap_or(Status::Unknown) };
    let value = if missing.is_empty() { Some(crate::value::round4(figures.iter().filter_map(|f| f.1).sum())) } else { None };
    Sum { value, unit, status: if missing.is_empty() { status } else { Status::Unknown }, members: figures.len(), missing }
}

/// What a roof comes to: its planes' surface in SF, and its lines' length in LF by kind.
#[derive(Clone, Debug)]
pub struct RoofTotals {
    pub surface: Sum,
    pub lines: Vec<(Kind, Sum)>,
    /// How many planes the roof has: its facets, as a roof report counts them.
    pub facets: usize,
    /// The pitch that covers the most of the roof's surface, in inches in 12, with the
    /// worst status among the planes at that pitch. None when no plane has both a pitch
    /// and a surface.
    pub predominant_pitch: Option<(f64, Status)>,
}

/// A surface in SF, said in roofing squares: one square is 100 SF. The same surface in
/// the trade's other unit, carrying the same status and the same gaps.
pub fn in_squares(surface: &Sum) -> Sum {
    Sum { value: surface.value.map(|sf| crate::value::round4(sf / 100.0)), unit: "SQ", status: surface.status, members: surface.members, missing: surface.missing.clone() }
}

/// A figure as a person reads it on a sheet: thousands separated, to `decimals` places,
/// with zeros kept so a column lines up. Formatting only; nothing is worked out here.
pub fn figure(value: f64, decimals: usize) -> String {
    let text = format!("{:.*}", decimals, value.abs());
    let (whole, part) = text.split_once('.').unwrap_or((text.as_str(), ""));
    let mut grouped = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    let minus = if value < 0.0 && text.chars().any(|c| ('1'..='9').contains(&c)) { "-" } else { "" };
    if part.is_empty() { format!("{minus}{grouped}") } else { format!("{minus}{grouped}.{part}") }
}

/// A figure to `decimals` places, unless that would print a figure that is not zero as
/// zero: then to as many places as it takes to show it, up to the four the model keeps.
/// A view may round; it may not round something away.
pub fn figure_shown(value: f64, decimals: usize) -> String {
    let is_zero = |text: &str| !text.chars().any(|c| ('1'..='9').contains(&c));
    let mut places = decimals;
    let mut text = figure(value, places);
    while value != 0.0 && is_zero(&text) && places < 4 {
        places += 1;
        text = figure(value, places);
    }
    text
}

/// Total the elements directly under `roof_path`.
pub fn roof_totals(job: &mut Job, roof_path: &str) -> RoofTotals {
    let children: Vec<String> = job.children().get(roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let mut areas = Vec::new();
    // Surface by pitch, to find the pitch most of the roof is at.
    let mut by_pitch: Vec<(f64, f64, Status)> = Vec::new();
    let mut by_kind: Vec<(Kind, Vec<(String, Option<f64>, Status)>)> = Vec::new();
    // In job order, so the sheet reads the same every time.
    for &e in &job.order {
        let w = &job.world;
        let (Some(node), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        if !children.contains(&node.path) {
            continue;
        }
        let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (d.value, d.status)).unwrap_or((None, Status::Unknown));
        if kind.is_area() {
            let (v, s) = figure("surface_area");
            if let (Some(area), Some(pitch)) = (v, w.get::<Pitch>(e).and_then(|p| p.0.value.map(|x| (x, p.0.status)))) {
                match by_pitch.iter_mut().find(|(p, _, _)| (*p - pitch.0).abs() < 1e-9) {
                    Some(entry) => {
                        entry.1 += area;
                        entry.2 = entry.2.max(s).max(pitch.1);
                    }
                    None => by_pitch.push((pitch.0, area, s.max(pitch.1))),
                }
            }
            areas.push((node.path.clone(), v, s));
        } else if kind.is_run() && *kind != Kind::Parapet {
            let (v, s) = figure("true_length");
            match by_kind.iter_mut().find(|(k, _)| k == kind) {
                Some((_, list)) => list.push((node.path.clone(), v, s)),
                None => by_kind.push((*kind, vec![(node.path.clone(), v, s)])),
            }
        }
    }
    by_pitch.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));
    RoofTotals {
        facets: areas.len(),
        predominant_pitch: by_pitch.first().map(|(p, _, s)| (*p, *s)),
        surface: sum_of("SF", &areas),
        lines: by_kind.into_iter().map(|(k, list)| (k, sum_of("LF", &list))).collect(),
    }
}
