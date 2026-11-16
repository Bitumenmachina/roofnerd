//! A flat reading of the world: every value on every element, entered or derived, with
//! its status. The command line prints this; the sheet and the 3D view will read the
//! same rows. Nothing here computes a quantity. It only reports what the world holds.

use std::collections::BTreeMap;
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

pub fn num(x: f64) -> String {
    // Adding zero shows a "-0" from a file as plain 0.
    let s = format!("{:.4}", x + 0.0);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

pub(crate) fn entered<T>(path: &str, label: &str, v: &Val<T>, show: impl Fn(&T) -> String) -> Row {
    let text = match &v.value {
        Some(x) => format!("{} {}", show(x), v.unit).trim().to_string(),
        None => "unknown".to_string(),
    };
    Row { path: path.into(), label: label.into(), text, status: v.status, basis: v.source.clone(), note: String::new(), derived: false, number: None, unit: String::new() }
}

/// An entered length, shown in the unit it was entered in: `30 in`, `2.5 ft`, `25' 6 1/2"`.
pub(crate) fn length(path: &str, label: &str, v: &Val<Len>) -> Row {
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
    Row { path: path.into(), label: label.into(), text, status: d.status, basis: words(&d.formula), note: d.note.clone(), derived: true, number: d.value, unit: d.unit.clone() }
}

/// Every element in tree order (each one, then what is under it), each with all of its
/// values.
pub fn elements(job: &mut Job) -> Vec<ElementRows> {
    let mut out = Vec::new();
    // Tree order: each element, then everything under it, with brothers in the order they
    // were added. A door is listed under its own wall, whenever it was added.
    let order = {
        let w = &job.world;
        let path_of = |e: &bevy_ecs::entity::Entity| w.get::<Node>(*e).map(|n| n.path.clone());
        let known: std::collections::HashSet<String> = job.order.iter().filter_map(path_of).collect();
        let mut under: std::collections::HashMap<Option<String>, Vec<bevy_ecs::entity::Entity>> = std::collections::HashMap::new();
        for e in &job.order {
            // An element whose parent is not in the job is listed at the top, not lost.
            let parent = w.get::<Node>(*e).and_then(|n| n.parent.clone()).filter(|p| known.contains(p));
            under.entry(parent).or_default().push(*e);
        }
        let mut out = Vec::with_capacity(job.order.len());
        let mut stack: Vec<bevy_ecs::entity::Entity> = under.get(&None).cloned().unwrap_or_default().into_iter().rev().collect();
        while let Some(e) = stack.pop() {
            out.push(e);
            if let Some(children) = path_of(&e).and_then(|p| under.get(&Some(p))) {
                stack.extend(children.iter().rev());
            }
        }
        // A loop of parents (which a loaded file cannot have) would leave some out.
        for e in &job.order {
            if !out.contains(e) {
                out.push(*e);
            }
        }
        out
    };
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
    let names: std::collections::HashMap<String, String> = order.iter().filter_map(|e| w.get::<Node>(*e)).map(|n| (n.path.clone(), n.name.clone())).collect();
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
                    // In-house work: how fast it goes. Anything else: how what is
                    // measured becomes what is bought. (Open items said "order factor:
                    // Hang the siding" of a labor line: J24B-56.)
                    let called = if o.unit == crate::components::HOURS { "production rate" } else { "order factor" };
                    rows.push(entered(p, &format!("{called}: {}", line.description), &o.factor, |x| num(*x)));
                }
                if !line.of_material {
                    rows.push(entered(p, &format!("unit cost: {}", line.description), &line.unit_cost, |x| rate(*x)));
                }
            }
        }
        if let Some(m) = w.get::<MadeOf>(e) {
            rows.push(entered(p, "material", &m.0, |s| s.clone()));
        }
        if let Some(supply) = w.get::<Supply>(e) {
            if let Some(o) = &supply.order {
                rows.push(entered(p, "order factor", &o.factor, |x| num(*x)));
            }
            rows.push(entered(p, "price", &supply.price, |x| rate(*x)));
            for (supplier, quote) in &supply.quotes {
                rows.push(entered(p, &format!("quote: {supplier}"), quote, |x| rate(*x)));
            }
        }
        if let Some(m) = w.get::<Markup>(e) {
            rows.push(entered(p, "overhead", &m.overhead, |x| num(*x)));
            rows.push(entered(p, "profit", &m.profit, |x| num(*x)));
        }
        rows.extend(crate::building::rows(w, e, p, &names));
        rows.extend(crate::plans::rows(w, e, p));
        rows.extend(crate::tables::rows(w, e, p));
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
                    // Hours are worked, not ordered.
                    let called = if d.unit == crate::components::HOURS { "hours" } else { "to order" };
                    rows.push(derived(p, &format!("{called}: {description}"), d));
                }
            }
        }
        // Money: what each line comes to (its unit cost is listed with the cost code,
        // above), the element's allowance, and what the element comes to both ways.
        if let Some(c) = w.get::<Cost>(e) {
            for line in &c.lines {
                let figures = w.get::<LineQuantities>(e).and_then(|q| q.0.get(&line.description));
                if let Some(d) = figures.and_then(|f| f.extension.as_ref()) {
                    rows.push(derived(p, &format!("extension: {}", line.description), d));
                }
                for (supplier, d) in figures.map(|f| &f.quotes).into_iter().flatten() {
                    rows.push(derived(p, &format!("extension at {supplier}: {}", line.description), d));
                }
            }
        }
        if let Some(a) = w.get::<Allowance>(e) {
            rows.push(entered(p, "allowance", &a.rate, |x| rate(*x)));
        }
        if let Some(priced) = w.get::<Priced>(e) {
            for name in ["allowance", "detail", "detail_rate", "difference", "difference_percent"] {
                if let Some(d) = priced.0.get(name) {
                    rows.push(derived(p, &format!("comes to, {}", name.replace('_', " ")), d));
                }
            }
        }
        // A condition's recipe is read, not counted: its blanks are not this job's unknowns.
        let mut notes = Vec::new();
        if let Some(t) = w.get::<Recipe>(e) {
            notes.push(format!("applies to: {}", t.applies_to.iter().map(|k| k.said().to_string()).collect::<Vec<_>>().join(", ")));
            for line in &t.cost.lines {
                // What the line takes, and what the recipe already states for it. A value a
                // recipe states goes onto an element with that status, so it is shown here.
                let stated = |status: Status, text: String| if status == Status::Unknown { "blank".to_string() } else { format!("{}{text}", if status == Status::Assumed { "~ " } else { "" }) };
                let mut extra = Vec::new();
                if line.stretch_out.is_some() {
                    extra.push("stretch-out: blank".to_string());
                }
                for (name, v) in &line.inputs {
                    extra.push(format!("{}: {}", words(name), stated(v.status, v.value.as_ref().map(|len| units::show_length(len, &v.unit)).unwrap_or_default())));
                }
                if let Some(w) = &line.waste {
                    extra.push(format!("waste: {}", stated(w.status, w.value.map(|x| format!("{} %", num(x))).unwrap_or_default())));
                }
                if let Some(o) = &line.order {
                    // In-house work is by the hour, and what an hour does is its
                    // production rate: "ordered by the hr" is how a sheet is bought.
                    let what = if o.unit == HOURS { "production rate".to_string() } else { format!("ordered by the {}", o.unit) };
                    extra.push(format!("{what}: {}", stated(o.factor.status, o.factor.value.map(|x| format!("{} {}", num(x), o.factor.unit)).unwrap_or_default())));
                }
                extra.push(format!("price: {}", stated(line.unit_cost.status, line.unit_cost.value.map(|x| format!("{} {}", num(x), line.unit_cost.unit)).unwrap_or_default())));
                notes.push(format!("line: {} = {}   ({})", line.description, words(&line.quantity), extra.join("; ")));
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

/// The same over the things of one kind that a level holds, where having none is itself
/// Known: a level with no wall has, for certain, no wall. (A roof with no planes is not
/// that: its surface is not known. J21-17: a level's zero walls were flagged Unknown at
/// the command line and written in red on the sheet.)
pub(crate) fn sum_of_what_is_there(unit: &'static str, figures: &[(String, Option<f64>, Status)]) -> Sum {
    let mut sum = sum_of(unit, figures);
    if figures.is_empty() {
        sum.status = Status::Known;
    }
    sum
}

pub(crate) fn sum_of(unit: &'static str, figures: &[(String, Option<f64>, Status)]) -> Sum {
    let missing: Vec<String> = figures.iter().filter(|f| f.1.is_none()).map(|f| f.0.clone()).collect();
    // A sum over nothing is not a Known zero: there is nothing there to total, and a view
    // has to say so (`members` is 0) instead of showing 0.00.
    if figures.is_empty() {
        return Sum { value: None, unit, status: Status::Unknown, members: 0, missing };
    }
    let status = figures.iter().map(|f| f.2).max().unwrap_or(Status::Unknown);
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
    /// worst status among the planes at that pitch. None when it cannot be said: a plane
    /// has no pitch or no surface (it might be the one that tips it), two pitches cover
    /// the same surface, or there are no planes. `pitch_note` says which.
    pub predominant_pitch: Option<(f64, Status)>,
    pub pitch_note: String,
}

/// A pitch as the trade says it, to the four places the model keeps: `6:12`, `6.125:12`.
pub fn pitch_said(rise: f64) -> String {
    format!("{}:12", num(rise))
}

/// A surface in SF, said in roofing squares: one square is 100 SF. The same surface in
/// the trade's other unit, carrying the same status and the same gaps.
pub fn in_squares(surface: &Sum) -> Sum {
    // A surface too small to show in squares at four places is not zero squares.
    let squares = surface.value.map(|sf| crate::value::round4(sf / 100.0));
    if surface.value.is_some_and(|sf| sf != 0.0) && squares == Some(0.0) {
        return Sum { value: None, unit: "SQ", status: Status::Unknown, members: surface.members, missing: surface.missing.clone() };
    }
    Sum { value: squares, unit: "SQ", status: surface.status, members: surface.members, missing: surface.missing.clone() }
}

/// A figure's name as a person reads it: its words. The job file and a formula's own
/// text join them with an underscore; nobody reads that, and either can be typed.
pub fn words(name: &str) -> String {
    name.replace('_', " ")
}

/// A figure as a person reads it on a sheet: thousands separated, to `decimals` places,
/// with zeros kept so a column lines up. Formatting only; nothing is worked out here.
pub fn figure(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return "not a number".into();
    }
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
/// zero: then it is shown to the four places the model keeps, exactly as the model has it.
/// A view may round; it may not round something away, or up into something else.
pub fn figure_shown(value: f64, decimals: usize) -> String {
    let text = figure(value, decimals);
    if value.is_finite() && value != 0.0 && !text.chars().any(|c| ('1'..='9').contains(&c)) {
        return num(value);
    }
    text
}

/// A rate (dollars for a unit, or a percentage) as it is written: to the cent, or to as
/// many places as it was entered with. Every place is shown, so a rate of 0.00004 is not
/// written as 0, and one price is written one way wherever it is printed.
/// How many decimal places a quantity that is priced is shown to: two, and four where
/// it has more than two, so that what is on the line times its unit cost is the
/// amount on the line (4.1338 SQ at $95.00 is $392.71; 4.13 SQ is $392.35, and an
/// estimator checking the line by hand finds it 36 cents out: J21-18, J22A-8). Hours
/// are shown to two, as a time sheet has them; money and counts are not quantities.
pub fn priced_places(quantity: f64, unit: &str) -> usize {
    let more = (quantity * 100.0 - (quantity * 100.0).round()).abs() > 1.0e-6;
    if more && unit != "$" && !unit.starts_with("$/") && unit != "EA" && unit != "hr" { 4 } else { 2 }
}

pub fn rate(value: f64) -> String {
    if !value.is_finite() {
        return "not a number".into();
    }
    // The shortest decimal that is this number, which is how it was typed.
    let shortest = format!("{}", value.abs());
    let places = shortest.split_once('.').map(|(_, part)| part.len()).unwrap_or(0);
    figure(value, places.max(2))
}

/// Total the elements directly under `roof_path`.
pub fn roof_totals(job: &mut Job, roof_path: &str) -> RoofTotals {
    let children: Vec<String> = job.children().get(roof_path).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let mut areas = Vec::new();
    // Surface by pitch, to find the pitch most of the roof is at.
    let mut by_pitch: Vec<(f64, f64, Status)> = Vec::new();
    let mut by_kind: Vec<(Kind, Vec<(String, Option<f64>, Status)>)> = Vec::new();
    // Planes with no pitch or no surface: any one of them might be what tips the balance.
    let mut unplaced: Vec<String> = Vec::new();
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
            let pitch_of = w.get::<Pitch>(e).and_then(|p| p.0.value.map(|x| (x, p.0.status)));
            if v.is_none() || pitch_of.is_none() {
                unplaced.push(node.path.clone());
            }
            if let (Some(area), Some(pitch)) = (v, pitch_of) {
                match by_pitch.iter_mut().find(|(p, _, _)| (*p - pitch.0).abs() < 1e-9) {
                    Some(entry) => {
                        entry.1 += area;
                        entry.2 = entry.2.max(s).max(pitch.1);
                    }
                    None => by_pitch.push((pitch.0, area, s.max(pitch.1))),
                }
            }
            areas.push((node.path.clone(), v, s));
        } else if kind.is_run() {
            // A parapet stands on the plan: its length is its plan length.
            let (v, s) = figure(if *kind == Kind::Parapet { "plan_length" } else { "true_length" });
            match by_kind.iter_mut().find(|(k, _)| k == kind) {
                Some((_, list)) => list.push((node.path.clone(), v, s)),
                None => by_kind.push((*kind, vec![(node.path.clone(), v, s)])),
            }
        }
    }
    by_pitch.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));
    let (predominant_pitch, pitch_note) = if areas.is_empty() {
        (None, "the roof has no planes".to_string())
    } else if !unplaced.is_empty() {
        (None, format!("{} has no pitch or no surface, and might be what tips it", unplaced.join(", ")))
    } else if by_pitch.len() > 1 && (by_pitch[0].1 - by_pitch[1].1).abs() < 0.00005 {
        (None, format!("{} and {} cover the same surface", pitch_said(by_pitch[0].0), pitch_said(by_pitch[1].0)))
    } else {
        (by_pitch.first().map(|(p, _, s)| (*p, *s)), String::new())
    };
    RoofTotals {
        facets: areas.len(),
        predominant_pitch,
        pitch_note,
        surface: sum_of("SF", &areas),
        lines: by_kind.into_iter().map(|(k, list)| (k, sum_of("LF", &list))).collect(),
    }
}

/// One element as the reconciliation lists it: what it is bought by, and what it comes to
/// from its allowance and from its cost lines.
#[derive(Clone, Debug)]
pub struct PricedElement {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    /// The figure it is bought by, with its name and unit (`surface_area`, `SF`).
    pub figure: Option<(String, String, Derived)>,
    /// The allowance rate as entered, when it has one.
    pub rate: Option<Val<f64>>,
    pub allowance: Option<Derived>,
    pub detail: Option<Derived>,
    pub detail_rate: Option<Derived>,
    pub difference: Option<Derived>,
    pub difference_percent: Option<Derived>,
    /// It has cost lines and every one of them is switched off: it has no detail, and
    /// that is why.
    pub all_off: bool,
}

/// A total of money that says what it rests on and what it leaves out.
#[derive(Clone, Debug, PartialEq)]
pub struct MoneyTotal {
    /// The sum of what is included, to the cent. None when nothing is included: a sum
    /// over nothing is not $0.00.
    pub value: Option<f64>,
    /// The worst status among what is included.
    pub status: Status,
    /// How many elements are included.
    pub members: usize,
    /// How many dollars of it are from elements whose figure is Assumed.
    pub assumed: f64,
    /// The elements left out, each with why.
    pub left_out: Vec<(String, String)>,
}

/// What the job comes to both ways, and the two set against each other.
#[derive(Clone, Debug)]
pub struct Pricing {
    /// Every element that has an allowance or a cost line, in the job's order.
    pub elements: Vec<PricedElement>,
    /// Over the elements that have an allowance. Left out: those whose allowance has no
    /// figure.
    pub allowance: MoneyTotal,
    /// Over the elements whose detail is whole. Left out: those whose detail is not.
    pub detail: MoneyTotal,
    /// The allowance and the detail of the elements that have both figures, and only
    /// those: the like-for-like comparison.
    /// Each sum with its own status, then the worse of the two, then how many elements.
    pub both: Option<((f64, Status), (f64, Status), Status, usize)>,
    /// Elements that could carry a price (they have a figure to be bought by, or are
    /// bought by the each) and carry neither an allowance nor a cost line: how many of
    /// each kind.
    pub not_priced: Vec<(Kind, usize)>,
}

impl Estimate {
    /// The sixteen divisions together, to the cent: what the divisions that come to a
    /// figure add to (None where none does), no surer than the least sure of them, and
    /// how many lines the divisions leave out. Worked here and nowhere else: the
    /// command line and the workbook each added them up for themselves (M21-13).
    pub fn sixteen(&self) -> (Option<f64>, Status, usize) {
        let with: Vec<&DivisionTotal> = self.divisions.iter().filter(|d| d.total.value.is_some()).collect();
        let total = (!with.is_empty()).then(|| crate::lines::cents(with.iter().filter_map(|d| d.total.value).sum()));
        (total, with.iter().map(|d| d.total.status).max().unwrap_or(Status::Known), self.divisions.iter().map(|d| d.total.left_out.len()).sum())
    }
}

impl Pricing {
    /// Like for like, worked here and nowhere else: over the elements that have both,
    /// the detail less the allowances to the cent (over is above nought), and that as
    /// a percentage of the allowances (None where they add to nothing). Each window
    /// worked these out for itself before (R21-10, M21-13).
    pub fn like_for_like(&self) -> Option<(f64, Option<f64>)> {
        let ((allowance, _), (detail, _), _, _) = self.both?;
        let difference = crate::lines::cents(detail - allowance);
        Some((difference, (allowance != 0.0).then(|| difference / allowance * 100.0)))
    }
}

/// A count with its thing, said as a person says it: "1 line", "3 lines", "0 doors".
/// For a thing whose plural only adds an s.
/// Names as a sentence lists them: "plane 1", "plane 1 and plane 2", "plane 1, plane 2
/// and plane 4".
pub fn listed(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

pub fn so_many(n: usize, one: &str) -> String {
    if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") }
}

/// Overhead or profit typed over 100 %: taken, and answered, since it is a slip (250
/// for 25) far more often than a markup, and the program does not know which
/// (J22A-19; J23-8: the command line took it without a word).
pub fn markup_large_said(typed: &str) -> Option<String> {
    let x = typed.trim().trim_end_matches('%').trim().parse::<f64>().ok()?;
    (x > 100.0).then(|| format!("That is more than what it is a percentage of. If {} % was meant, enter it again.", figure(x / 10.0, 2)))
}

/// What was priced on the things a step took out of the job, in the one sentence that
/// is said as they go: the cost lines that went with them, what those came to, and
/// how many had no figure. `before` is the estimate as it stood before the step. None
/// when nothing priced went. A priced wall was deleted with "Deleted wall 1." and the
/// price fell by what was on it without a word (J22A-17).
pub fn went_with_said(before: &Estimate, job: &mut Job) -> Option<String> {
    gone_said(before, job, false)
}

/// The same for cost lines taken off a thing that stays: an assembly changed for
/// another takes off the lines the new one does not have, and their dollars with
/// them (J23-5).
pub fn came_off_said(before: &Estimate, job: &mut Job) -> Option<String> {
    gone_said(before, job, true)
}

/// After a roof change that dropped a priced plane: what the job cost was and is. The
/// planes that stay change size when one goes, and what is priced on them changes
/// with it: a hip made a gable drops a plane's dollars and the two planes beside it
/// take nearly all of them back. "$71.55 is out of the job cost" is true of what went
/// and is not the whole of it. None when either job cost has no figure.
pub fn job_cost_was_and_is_said(before: &Estimate, job: &mut Job) -> Option<String> {
    let (was, is) = (before.job_cost.value?, estimate(job).job_cost.value?);
    Some(format!("The planes that stay changed with it: the job cost was ${} and is ${}.", figure(was, 2), figure(is, 2)))
}

fn gone_said(before: &Estimate, job: &mut Job, came_off: bool) -> Option<String> {
    let now: std::collections::BTreeSet<(String, String)> = estimate(job).lines.into_iter().map(|l| (l.path, l.description)).collect();
    let gone: Vec<&SheetLine> = before.lines.iter().filter(|l| !now.contains(&(l.path.clone(), l.description.clone()))).collect();
    if gone.is_empty() {
        return None;
    }
    let live: Vec<&&SheetLine> = gone.iter().filter(|l| !l.off).collect();
    let priced: Vec<f64> = live.iter().filter_map(|l| l.amount.value).collect();
    let waiting = live.len() - priced.len();
    let off = gone.len() - live.len();
    let went = if came_off {
        format!("{} came off", so_many(gone.len(), "cost line"))
    } else {
        format!("{} went with {}", so_many(gone.len(), "cost line"), if gone.iter().any(|l| l.path != gone[0].path) { "them" } else { "it" })
    };
    let mut said = if priced.is_empty() {
        format!("{went}; none of them had come to a figure, so the price is as it was")
    } else {
        format!("{went}: ${} is out of the job cost", figure(crate::lines::cents(priced.iter().sum()), 2))
    };
    // What else is true of them, in one bracket.
    let mut also = Vec::new();
    if !priced.is_empty() && waiting > 0 {
        also.push(format!("{} had no price yet", if waiting == 1 { "1 of them".to_string() } else { format!("{waiting} of them") }));
    }
    if off > 0 {
        also.push(format!("{} switched off, and in no total", if off == 1 { "1 was".to_string() } else { format!("{off} were") }));
    }
    if !also.is_empty() {
        said.push_str(&format!(" ({})", also.join("; ")));
    }
    Some(format!("{said}."))
}

/// What is built and has nothing priced on it, in the one sentence every view says it
/// with: "nothing is priced on 3 things that could be (2 doors, 1 window): they are in
/// no line and no total above". None when everything that could be priced is.
pub fn not_priced_said(not_priced: &[(Kind, usize)]) -> Option<String> {
    if not_priced.is_empty() {
        return None;
    }
    let n: usize = not_priced.iter().map(|x| x.1).sum();
    let kinds: Vec<String> = not_priced.iter().map(|(k, n)| k.so_many(*n)).collect();
    Some(format!("nothing is priced on {n} thing{} that could be ({}): {} in no line and no total above", if n == 1 { "" } else { "s" }, kinds.join(", "), if n == 1 { "it is" } else { "they are" }))
}

impl SheetLine {
    /// How many decimal places what is priced on this line is shown to: two, unless
    /// at two it would not multiply by the line's price to its amount on a
    /// calculator, and then the four it is held to (J17A-9, J22A-8). Hours are shown
    /// to `hours` places: four where they are added up, two as a time sheet has them.
    /// The command line worked this out for itself and the window showed two always.
    pub fn priced_places(&self, hours: usize) -> usize {
        let Some(x) = self.priced.value else { return 2 };
        if self.priced.unit == crate::HOURS {
            return hours;
        }
        match (self.unit_cost.as_ref().and_then(|v| v.value), self.amount.value) {
            (Some(p), Some(a)) if (crate::lines::cents((x * 100.0).round() / 100.0 * p) - a).abs() > 0.004 => 4,
            _ => 2,
        }
    }
}

impl Pricing {
    /// What is priced by an allowance and by no cost line: how many things, and what
    /// their allowances that have a figure come to, to the cent. An allowance is the
    /// budget's figure; the price is over the cost lines. These are in no line of it.
    pub fn allowances_alone(&self) -> Option<(usize, Option<f64>)> {
        let alone: Vec<&PricedElement> = self.elements.iter().filter(|e| e.allowance.is_some() && e.detail.is_none()).collect();
        if alone.is_empty() {
            return None;
        }
        let figures: Vec<f64> = alone.iter().filter_map(|e| e.allowance.as_ref().and_then(|d| d.value)).collect();
        Some((alone.len(), (!figures.is_empty()).then(|| crate::lines::cents(figures.iter().sum()))))
    }

    /// The same, in the one sentence every view says it with. None when nothing is
    /// priced by an allowance alone. (J22A-10: $8,580 of allowance on a room was in
    /// no line and no total of the price, the room was no longer counted as not
    /// priced, and the screen the price is read on said nothing.)
    pub fn allowances_alone_said(&self) -> Option<String> {
        let (n, sum) = self.allowances_alone()?;
        let money = sum.map(|x| format!("${}", figure(x, 2))).unwrap_or_else(|| "no figure".to_string());
        Some(format!("{n} thing{} priced by an allowance and no cost line ({money}): an allowance is the budget's figure and is not in this price. Give {} cost lines to carry {} into it", if n == 1 { " is" } else { "s are" }, if n == 1 { "it" } else { "them" }, if n == 1 { "it" } else { "them" }))
    }
}

fn money_total(parts: &[(&PricedElement, &Derived)]) -> MoneyTotal {
    let included: Vec<&(&PricedElement, &Derived)> = parts.iter().filter(|(_, d)| d.value.is_some()).collect();
    let left_out = parts.iter().filter(|(_, d)| d.value.is_none()).map(|(e, d)| (e.path.clone(), d.note.clone())).collect();
    let cents = crate::lines::cents;
    finish_total(MoneyTotal {
        value: if included.is_empty() { None } else { Some(cents(included.iter().filter_map(|(_, d)| d.value).sum())) },
        status: included.iter().map(|(_, d)| d.status).max().unwrap_or(Status::Unknown),
        members: included.len(),
        assumed: cents(included.iter().filter(|(_, d)| d.status == Status::Assumed).filter_map(|(_, d)| d.value).sum()),
        left_out,
    })
}

/// The two things every total of money is held to. A total that leaves something out is
/// never shown as Known: it is the sum of what is in it, and reads as that. And a total
/// too large to hold to the cent is not given.
fn finish_total(mut t: MoneyTotal) -> MoneyTotal {
    if t.value.is_some_and(|v| v.abs() > crate::lines::MAX_DOLLARS) {
        t.value = None;
        t.left_out.insert(0, ("the total".into(), "it comes to more than can be held to the cent".into()));
    }
    if t.value.is_some() && !t.left_out.is_empty() {
        t.status = t.status.max(Status::Assumed);
    }
    t
}

/// The job priced from its allowances and from its cost lines, and the two reconciled.
/// Nothing here is arithmetic of its own beyond adding up: each element's figures are
/// its `pl::priced`, worked out in `derive`.
pub fn pricing(job: &mut Job) -> Pricing {
    let order = job.order.clone();
    let w = &job.world;
    let mut elements = Vec::new();
    let mut bare: Vec<(Kind, usize)> = Vec::new();
    for &e in &order {
        let (Some(node), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e).copied()) else { continue };
        let priced = w.get::<Priced>(e);
        let has_lines = w.get::<Cost>(e).is_some_and(|c| !c.lines.is_empty());
        let all_off = has_lines && w.get::<Cost>(e).is_some_and(|c| c.lines.iter().all(|l| l.off));
        let rate = w.get::<Allowance>(e).map(|a| a.rate.clone());
        if rate.is_none() && !has_lines {
            // It could be priced and is not: by its area or length, or by the each.
            if kind.priced_by().is_some() || matches!(kind, Kind::Door | Kind::Window | Kind::Fitting | Kind::Fixture | Kind::Furnishing | Kind::Equipment) {
                match bare.iter_mut().find(|b| b.0 == kind) {
                    Some(b) => b.1 += 1,
                    None => bare.push((kind, 1)),
                }
            }
            continue;
        }
        let figure = kind.priced_by().map(|(name, unit)| {
            let d = w.get::<Measure>(e).and_then(|m| m.0.get(name)).cloned().unwrap_or_else(|| Derived::refused(unit, "", &[], &format!("it has no {name}")));
            (name.to_string(), unit.to_string(), d)
        });
        let get = |name: &str| priced.and_then(|p| p.0.get(name)).cloned();
        elements.push(PricedElement { path: node.path.clone(), name: node.name.clone(), kind, figure, rate, allowance: get("allowance"), detail: get("detail"), detail_rate: get("detail_rate"), difference: get("difference"), difference_percent: get("difference_percent"), all_off });
    }
    let allowances: Vec<(&PricedElement, &Derived)> = elements.iter().filter_map(|e| e.allowance.as_ref().map(|d| (e, d))).collect();
    let details: Vec<(&PricedElement, &Derived)> = elements.iter().filter_map(|e| e.detail.as_ref().map(|d| (e, d))).collect();
    let cents = crate::lines::cents;
    let pairs: Vec<((f64, Status), (f64, Status))> = elements.iter().filter_map(|e| e.allowance.as_ref().zip(e.detail.as_ref())).filter_map(|(a, d)| a.value.zip(d.value).map(|(x, y)| ((x, a.status), (y, d.status)))).collect();
    let worst = |side: &dyn Fn(&((f64, Status), (f64, Status))) -> Status| pairs.iter().map(side).max().unwrap_or(Status::Unknown);
    let (sa, sd) = (worst(&|p| p.0 .1), worst(&|p| p.1 .1));
    let both = if pairs.is_empty() { None } else { Some(((cents(pairs.iter().map(|p| p.0 .0).sum()), sa), (cents(pairs.iter().map(|p| p.1 .0).sum()), sd), sa.max(sd), pairs.len())) };
    Pricing { allowance: money_total(&allowances), detail: money_total(&details), both, not_priced: bare, elements }
}

/// One cost line as the estimate sheet lists it.
#[derive(Clone, Debug)]
pub struct SheetLine {
    pub path: String,
    pub element: String,
    pub kind: Kind,
    pub description: String,
    pub class: Option<Class>,
    /// Which of the sixteen divisions its work is.
    pub division: Option<crate::components::Division>,
    /// Switched off: on the sheet, out of every total.
    pub off: bool,
    /// The material it is made of, when it is made of its element's.
    pub material: Option<String>,
    pub quantity: Derived,
    /// What is priced: its order quantity, else its quantity with waste, else its quantity.
    pub priced: Derived,
    /// The price it is extended at: its own, or its material's. None when its material
    /// cannot be found.
    pub unit_cost: Option<Val<f64>>,
    /// What it comes to.
    pub amount: Derived,
    /// What it comes to at each supplier's price, when it is made of a material.
    pub quotes: BTreeMap<String, Derived>,
    /// Its stretch-out, when its detail has one: the girth of the piece the line is
    /// cut from. It belongs to the line, and is changed on the line.
    pub stretch_out: Option<Val<crate::Len>>,
    /// For in-house work: how fast it goes, as its order factor says it ("40 SF/hr" or
    /// "0.025 hr/SF"). None for a line that is not in hours. Its value is None where a
    /// library brought the line with no rate: the estimator's crew is the estimator's.
    pub rate: Option<Val<f64>>,
}

impl SheetLine {
    /// The production rate as it is said and typed: "40 SF/hr". None when the line is
    /// not in hours, or its rate has not been entered.
    pub fn rate_said(&self) -> Option<String> {
        let rate = self.rate.as_ref()?;
        rate.value.map(|x| format!("{} {}", num(x), rate.unit))
    }
}

/// One division of the estimate: what the lines of the job cost in it come to.
#[derive(Clone, Debug)]
pub struct DivisionTotal {
    pub division: crate::components::Division,
    pub total: MoneyTotal,
}

/// One row of the recap.
#[derive(Clone, Debug)]
pub struct ClassTotal {
    pub class: Class,
    pub total: MoneyTotal,
    /// For in-house work: the hours of the lines in the total, and how sure they are.
    /// None where no line is in the total.
    pub hours: Option<(f64, Status)>,
    /// For in-house work: the hours of the lines of this cost type that have hours and
    /// no price, and how many such lines. They are in neither the dollars nor `hours`,
    /// and whoever shows the hours says them beside (call 82).
    pub hours_unpriced: Option<(f64, usize)>,
}

impl ClassTotal {
    /// The hours that wait for a price, as they are said beside the hours: "18.00 hr
    /// more on 1 line with no price". None where there are none.
    pub fn hours_unpriced_said(&self) -> Option<String> {
        self.hours_unpriced.map(|(h, n)| format!("{} hr more on {} with no price", figure(h, 2), so_many(n, "line")))
    }
}

/// The estimate: every cost line, the three columns, the recap, and the suppliers side
/// by side. Nothing here is arithmetic beyond adding up and the two percentages: each
/// line's figures are its `pl::line_quantities`, worked out in `derive`.
#[derive(Clone, Debug)]
pub struct Estimate {
    pub lines: Vec<SheetLine>,
    /// Material, Labor, General: each over the lines whose class stands in that column.
    pub columns: Vec<(Column, MoneyTotal)>,
    /// All six classes, in order, whether or not anything is in them.
    pub classes: Vec<ClassTotal>,
    /// Lines with no class. They are in no column and no row of the recap.
    pub not_classed: Vec<(String, String)>,
    /// The sum of the classes. Left out: every classed line with no figure, and every
    /// line with no class.
    pub job_cost: MoneyTotal,
    /// The job cost read division by division: all sixteen, in order, whether or not
    /// anything is in them. Each is over the lines of the job cost in that division.
    pub divisions: Vec<DivisionTotal>,
    /// The lines of the job cost that are in no division: what the sixteen leave out.
    /// The sixteen and this come to the job cost.
    pub no_division: MoneyTotal,
    pub markup: Option<Markup>,
    pub overhead: Derived,
    pub profit: Derived,
    pub price: Derived,
    /// The suppliers that have quoted any material of the job, by name.
    pub suppliers: Vec<String>,
    /// Over the lines made of a material: at the generic price, then at each supplier's.
    pub at_generic: MoneyTotal,
    pub at_supplier: Vec<(String, MoneyTotal)>,
}

fn total_of(parts: &[(String, &Derived)]) -> MoneyTotal {
    let included: Vec<&Derived> = parts.iter().map(|(_, d)| *d).filter(|d| d.value.is_some()).collect();
    let cents = crate::lines::cents;
    finish_total(MoneyTotal {
        value: if included.is_empty() { None } else { Some(cents(included.iter().filter_map(|d| d.value).sum())) },
        status: included.iter().map(|d| d.status).max().unwrap_or(Status::Unknown),
        members: included.len(),
        assumed: cents(included.iter().filter(|d| d.status == Status::Assumed).filter_map(|d| d.value).sum()),
        left_out: parts.iter().filter(|(_, d)| d.value.is_none()).map(|(who, d)| (who.clone(), d.note.clone())).collect(),
    })
}

/// The estimate sheet and its recap.
pub fn estimate(job: &mut Job) -> Estimate {
    let order = job.order.clone();
    let root = job.root();
    let w = &job.world;
    // The job's materials, to show the price a line made of one is extended at.
    let mut materials: Vec<(String, String, Supply)> = Vec::new();
    for &e in &order {
        if let (Some(node), Some(Kind::Material), Some(supply)) = (w.get::<Node>(e), w.get::<Kind>(e).copied(), w.get::<Supply>(e)) {
            materials.push((node.name.clone(), node.path.clone(), supply.clone()));
        }
    }
    let mut suppliers: Vec<String> = materials.iter().flat_map(|m| m.2.quotes.keys().cloned()).collect();
    suppliers.sort();
    suppliers.dedup();

    let mut lines = Vec::new();
    for &e in &order {
        let (Some(node), Some(kind), Some(cost)) = (w.get::<Node>(e), w.get::<Kind>(e).copied(), w.get::<Cost>(e)) else { continue };
        // A condition's recipe is not on the job's sheet; it carries its lines in `pl::recipe`.
        let made_of = w.get::<MadeOf>(e).and_then(|m| m.0.value.clone());
        let found = made_of.as_ref().and_then(|name| {
            let found: Vec<&(String, String, Supply)> = materials.iter().filter(|(n, p, _)| p == name || crate::check::same_name(n, name)).collect();
            (found.len() == 1).then(|| found[0].clone())
        });
        // The material by its name, however the element names it (it may name its path).
        let made_of = found.as_ref().map(|f| f.0.clone()).or(made_of);
        let supply = found.map(|f| f.2);
        for line in &cost.lines {
            let Some(f) = w.get::<LineQuantities>(e).and_then(|q| q.0.get(&line.description)) else { continue };
            let priced = f.order.as_ref().or(f.with_waste.as_ref()).unwrap_or(&f.quantity).clone();
            let amount = f.extension.clone().unwrap_or_else(|| Derived::refused("$", "", &[], "it has no extension"));
            lines.push(SheetLine {
                path: node.path.clone(),
                element: node.name.clone(),
                kind,
                description: line.description.clone(),
                class: line.class,
                division: line.division,
                off: line.off,
                material: if line.of_material { Some(made_of.clone().unwrap_or_default()) } else { None },
                quantity: f.quantity.clone(),
                priced,
                unit_cost: if line.of_material { supply.as_ref().map(|s| s.price.clone()) } else { Some(line.unit_cost.clone()) },
                amount,
                quotes: f.quotes.clone(),
                stretch_out: line.stretch_out.clone(),
                rate: line.order.as_ref().filter(|o| o.unit == crate::components::HOURS).map(|o| o.factor.clone()),
            });
        }
    }

    let label = |l: &SheetLine| format!("{}: {:?}", l.path, l.description);
    let on: Vec<&SheetLine> = lines.iter().filter(|l| !l.off).collect();
    let classed = |c: Class| -> Vec<(String, &Derived)> { on.iter().filter(|l| l.class == Some(c)).map(|l| (label(l), &l.amount)).collect() };
    let classes: Vec<ClassTotal> = Class::ALL
        .iter()
        .map(|&class| {
            let total = total_of(&classed(class));
            let four = |x: f64| (x * 10000.0).round() / 10000.0;
            let of_class = |priced: bool| -> Vec<&Derived> { on.iter().filter(|l| l.class == Some(class) && l.amount.value.is_some() == priced).map(|l| &l.priced).collect() };
            let hours = class.in_hours().then(|| of_class(true)).filter(|counted| !counted.is_empty()).map(|counted| (four(counted.iter().filter_map(|d| d.value).sum()), counted.iter().map(|d| d.status).max().unwrap_or(Status::Unknown)));
            let waiting: Vec<f64> = if class.in_hours() { of_class(false).iter().filter_map(|d| d.value).collect() } else { Vec::new() };
            let hours_unpriced = (!waiting.is_empty()).then(|| (four(waiting.iter().sum()), waiting.len()));
            ClassTotal { class, total, hours, hours_unpriced }
        })
        .collect();
    let columns = [Column::Material, Column::Labor, Column::General]
        .iter()
        .map(|&col| {
            let parts: Vec<(String, &Derived)> = on.iter().filter(|l| l.class.is_some_and(|c| c.column() == col)).map(|l| (label(l), &l.amount)).collect();
            (col, total_of(&parts))
        })
        .collect();
    let not_classed: Vec<(String, String)> = on.iter().filter(|l| l.class.is_none()).map(|l| (l.path.clone(), l.description.clone())).collect();
    // The job cost: every classed line that is on. A line with no class is left out and
    // named with the rest.
    let no_class = Derived::refused("$", "", &[], "the line has no cost type, so it is in no row of the recap");
    let all: Vec<(String, &Derived)> = on.iter().map(|l| (label(l), if l.class.is_some() { &l.amount } else { &no_class })).collect();
    let job_cost = total_of(&all);
    // The same lines, division by division. A line with no cost type is out of the job
    // cost and so out of every division; one with a cost type and no division is in
    // the job cost and in no division, and is counted apart so the two can be squared.
    // A line given a division and no cost type is out of its division's total as it is
    // out of the job cost, and the division says so by name (J17A-3): a total that
    // leaves something out says what it left out.
    let in_division = |d: Option<crate::components::Division>| -> Vec<(String, &Derived)> {
        on.iter().filter(|l| l.division == d && (l.class.is_some() || d.is_some())).map(|l| (label(l), if l.class.is_some() { &l.amount } else { &no_class })).collect()
    };
    let divisions: Vec<DivisionTotal> = crate::components::Division::ALL.iter().map(|&division| DivisionTotal { division, total: total_of(&in_division(Some(division))) }).collect();
    let no_division = total_of(&in_division(None));

    let markup = w.get::<Markup>(root).cloned();
    let root_path = w.get::<Node>(root).map(|n| n.path.clone()).unwrap_or_default();
    let inputs = [format!("{root_path}#{K_MARKUP}")];
    // A percentage of a base. Neither has a default: one not entered gives no figure.
    let percent_of = |base: Option<(f64, Status)>, no_base: &str, rate: Option<&Val<f64>>, what: &str, formula: &str| -> Derived {
        match (rate, base) {
            (None, _) => Derived::refused("$", formula, &inputs, &format!("{what} has not been entered; enter a percentage, 0 if there is none")),
            (Some(r), _) if r.value.is_none() => Derived::refused("$", formula, &inputs, &format!("{what} is not known ({})", r.source)),
            (Some(_), None) => Derived::refused("$", formula, &inputs, no_base),
            (Some(r), Some((b, status))) => crate::lines::dollars(Some(b * r.value.unwrap_or(0.0) / 100.0), &[status, r.status], formula, &inputs),
        }
    };
    let no_cost = "there is no job cost: no line with a cost type comes to a figure";
    let cost = job_cost.value.map(|v| (v, job_cost.status));
    let overhead = percent_of(cost, no_cost, markup.as_ref().map(|m| &m.overhead), "overhead", "job cost × overhead ÷ 100");
    let with_overhead = cost.zip(overhead.value).map(|((c, s), o)| (crate::lines::cents(c + o), s.max(overhead.status)));
    let no_base = if cost.is_none() { no_cost.to_string() } else { format!("profit is taken on the job cost and the overhead together, and the overhead has no figure: {}", overhead.note) };
    let profit = percent_of(with_overhead, &no_base, markup.as_ref().map(|m| &m.profit), "profit", "(job cost + overhead) × profit ÷ 100");
    let price = match (cost, overhead.value, profit.value) {
        (Some((c, s)), Some(o), Some(p)) => crate::lines::dollars(Some(c + o + p), &[s, overhead.status, profit.status], "job cost + overhead + profit", &inputs),
        (None, _, _) => Derived::refused("$", "job cost + overhead + profit", &inputs, no_cost),
        (_, None, _) => Derived::refused("$", "job cost + overhead + profit", &inputs, &overhead.note),
        (_, _, None) => Derived::refused("$", "job cost + overhead + profit", &inputs, &profit.note),
    };

    // A price over a job cost that leaves lines out is the price of what is in it, and
    // says so on its own line: it is not the price of the job.
    let mut price = price;
    if let (Some(_), n @ 1..) = (price.value, job_cost.left_out.len()) {
        price.note = format!("it leaves out {n} line{}, named below: it is the price of the lines in the job cost, not of the job", if n == 1 { "" } else { "s" });
    }
    // The suppliers, side by side, over the lines made of a material.
    let of_material: Vec<&&SheetLine> = on.iter().filter(|l| l.material.is_some()).collect();
    let at_generic = total_of(&of_material.iter().map(|l| (label(l), &l.amount)).collect::<Vec<_>>());
    let at_supplier = suppliers
        .iter()
        .map(|s| {
            // Why a line has no figure at this supplier: the line has none at all (and says
            // why itself), or this supplier has not quoted what it is made of.
            let none: Vec<Derived> = of_material
                .iter()
                .map(|l| match l.amount.value {
                    None => Derived::refused("$", "", &[], &format!("the line has no figure at any price: {}", l.amount.note)),
                    Some(_) => Derived::refused("$", "", &[], &format!("{s} has not quoted {}", l.material.as_ref().filter(|m| !m.is_empty()).map(|m| format!("{m:?}")).unwrap_or_else(|| "its material".into()))),
                })
                .collect();
            let parts: Vec<(String, &Derived)> = of_material.iter().zip(&none).map(|(l, none)| (label(l), l.quotes.get(s).unwrap_or(none))).collect();
            (s.clone(), total_of(&parts))
        })
        .collect();
    Estimate { lines, columns, classes, not_classed, job_cost, divisions, no_division, markup, overhead, profit, price, suppliers, at_generic, at_supplier }
}
