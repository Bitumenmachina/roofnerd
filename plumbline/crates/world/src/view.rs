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
    // The walls the work opens, and what the bearing ones say they need (call 160).
    let opened = crate::hidden::opened_walls(&mut job.world);
    let cautions: std::collections::HashMap<String, String> = crate::hidden::cautions(&mut job.world).into_iter().collect();
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
        if let Some(x) = w.get::<Work>(e) {
            rows.push(entered(p, "work", &x.status, |s| s.said().to_string()));
        }
        // What a wall the work opens may be hiding: what is said of it, and what is not
        // said and so is not known (call 160). A wall the work does not open is asked
        // nothing; a word said of it anyway is kept and shown.
        if kind == Kind::Wall {
            let said = w.get::<Hidden>(e);
            let asked = opened.contains_key(p);
            match said.and_then(|h| h.bearing.as_ref()) {
                Some(b) => rows.push(entered(p, crate::hidden::BEARING, b, |v| v.said().to_string())),
                None if asked => rows.push(Row { path: p.clone(), label: crate::hidden::BEARING.into(), text: "unknown".into(), status: Status::Unknown, basis: crate::hidden::BEARING_NOT_SEEN.into(), note: String::new(), derived: false, number: None, unit: String::new() }),
                None => {}
            }
            match said.and_then(|h| h.in_the_wall.as_ref()) {
                Some(t) => rows.push(entered(p, crate::hidden::IN_THE_WALL, t, |v| v.clone())),
                None if asked => rows.push(Row { path: p.clone(), label: crate::hidden::IN_THE_WALL.into(), text: "unknown".into(), status: Status::Unknown, basis: crate::hidden::IN_THE_WALL_NOT_SEEN.into(), note: String::new(), derived: false, number: None, unit: String::new() }),
                None => {}
            }
        }
        if let Some(t) = w.get::<Taper>(e) {
            rows.push(entered(p, "drains", &t.drains, |pts| so_many(pts.len(), "drain")));
            rows.push(length(p, "thickness at the drain", &t.at_drain));
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
        // What a bearing wall the work opens needs, and that it is not in the price.
        if let Some(said) = cautions.get(p) {
            notes.push(format!("! {said}"));
        }
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

/// What a thing's mark says it is to the work, as one of three places (call 158): 0 to
/// build (new, temporary, or no mark: a thing with no mark is new), 1 existing to remain,
/// 2 to demolish.
pub fn work_place(w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> usize {
    match w.get::<Work>(e).and_then(|k| k.status.value) {
        None | Some(WorkStatus::New) | Some(WorkStatus::Temporary) => 0,
        Some(WorkStatus::Existing) => 1,
        Some(WorkStatus::Demolish) => 2,
    }
}

/// A line's figures, given three ways (call 158): to build, existing to remain, to
/// demolish. Each is the sum of the same figures the line itself sums, so the three add
/// up to it.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkSplit {
    pub to_build: Sum,
    pub existing: Sum,
    pub demolish: Sum,
}

impl WorkSplit {
    /// The three, in order, with the words a builder says for each.
    pub fn parts(&self) -> [(&'static str, &Sum); 3] {
        [("to build", &self.to_build), ("existing to remain", &self.existing), ("to demolish", &self.demolish)]
    }
}

/// The same figures as `sum_of`, split by the place each is in. None where every one is
/// to build: a line with nothing marked prints as it always did.
pub(crate) fn split_by_work(unit: &'static str, figures: &[(String, Option<f64>, Status)], places: &[usize]) -> Option<WorkSplit> {
    if places.iter().all(|p| *p == 0) {
        return None;
    }
    let part = |at: usize| {
        let of: Vec<(String, Option<f64>, Status)> = figures.iter().zip(places).filter(|(_, p)| **p == at).map(|(f, _)| f.clone()).collect();
        sum_of(unit, &of)
    };
    Some(WorkSplit { to_build: part(0), existing: part(1), demolish: part(2) })
}

/// What a roof comes to: its planes' surface in SF, and its lines' length in LF by kind.
#[derive(Clone, Debug)]
pub struct RoofTotals {
    pub surface: Sum,
    pub lines: Vec<(Kind, Sum)>,
    /// The surface and each line given three ways, where a plane or a line is not new
    /// work (call 158): None where nothing is marked. `lines_by_work` is in step with
    /// `lines`.
    pub surface_by_work: Option<WorkSplit>,
    pub lines_by_work: Vec<Option<WorkSplit>>,
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

/// Dollars as they are written everywhere: to the cent, thousands separated, the minus
/// before the sign ("-$12.50", never "$-12.50"). One writer (R21-12: it was written
/// out in about forty places, and a negative came out two ways).
pub fn money(value: f64) -> String {
    if !value.is_finite() {
        return "not a number".into();
    }
    let text = figure(value.abs(), 2);
    let minus = if value < 0.0 && text.chars().any(|c| ('1'..='9').contains(&c)) { "-" } else { "" };
    format!("{minus}${text}")
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
    // Where each plane and each line is, to build, existing or to demolish (call 158).
    let mut area_places: Vec<usize> = Vec::new();
    let mut kind_places: Vec<Vec<usize>> = Vec::new();
    // Planes with no pitch or no surface: any one of them might be what tips the balance.
    let mut unplaced: Vec<String> = Vec::new();
    // In job order, so the sheet reads the same every time.
    for &e in &job.order {
        let w = &job.world;
        let (Some(node), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        if !children.contains(&node.path) {
            continue;
        }
        // Each figure as it was worked (call 133), so that the roof's totals are the
        // roof's own figures to the last place (call 134): one arithmetic.
        let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (d.worked(), d.status)).unwrap_or((None, Status::Unknown));
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
            area_places.push(work_place(w, e));
        } else if kind.is_run() {
            // A parapet stands on the plan: its length is its plan length.
            let (v, s) = figure(if *kind == Kind::Parapet { "plan_length" } else { "true_length" });
            match by_kind.iter().position(|(k, _)| k == kind) {
                Some(at) => {
                    by_kind[at].1.push((node.path.clone(), v, s));
                    kind_places[at].push(work_place(w, e));
                }
                None => {
                    by_kind.push((*kind, vec![(node.path.clone(), v, s)]));
                    kind_places.push(vec![work_place(w, e)]);
                }
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
        surface_by_work: split_by_work("SF", &areas, &area_places),
        lines_by_work: by_kind.iter().zip(&kind_places).map(|((_, list), places)| split_by_work("LF", list, places)).collect(),
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
    /// A roof priced whole (it carries cost lines) with planes that are priced on their
    /// own as well: (the roof, its planes that carry lines). The two assemblies name the
    /// same shingles differently ("Field shingles", "Asphalt shingles"), so it is the
    /// pricing both ways that is said, not a line's name (call 136).
    pub twice: Vec<(String, Vec<String>)>,
    /// Rooms whose walls' board is priced by the room's wall area, each with the walls
    /// round it that carry board of their own (call 146).
    pub board_twice: Vec<(String, Vec<String>)>,
    /// A lump line on a room (a plain number: "Cabinets" 1 EA) priced beside the lines of
    /// things set in that room in its division: both are in the price (call 162).
    pub lumps: Vec<LumpLine>,
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
    if x > 100.0 {
        return Some(format!("That is more than what it is a percentage of. If {} % was meant, enter it again.", figure(x / 10.0, 2)));
    }
    // 100 typed for 10 doubles a bid and was taken without a word (J27A-19).
    (x >= 50.0).then(|| format!("That is a large markup. If {} % was meant, enter it again.", figure(x / 10.0, 2)))
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

/// After any change: each thing whose allowance was carried in the price before and
/// is not now, though the thing is still there (it was given cost lines, and is priced
/// by them from then on). A price that falls by an allowance says so (J27A-28, call
/// 126). `before` is what was carried: each thing's path, name and what it came to.
pub fn allowance_dropped_said(before: &[(String, String, Option<f64>)], job: &mut Job) -> Option<String> {
    let now: Vec<String> = estimate(job).allowance_on.into_iter().map(|(path, _, _)| path).collect();
    let said: Vec<String> = before
        .iter()
        .filter(|(path, _, _)| !now.contains(path) && job.find(path).is_some())
        .map(|(_, name, value)| {
            let name = format!("{}{}", name[..1].to_uppercase(), &name[1..]);
            match value {
                Some(x) => format!("{name}'s allowance of {} is no longer carried in the price: it is priced by its cost lines now.", money(*x)),
                None => format!("{name}'s allowance is no longer carried in the price: it is priced by its cost lines now."),
            }
        })
        .collect();
    (!said.is_empty()).then(|| said.join(" "))
}

/// What is carried as an allowance now, as `allowance_dropped_said` takes it.
pub fn carried_now(job: &mut Job) -> Vec<(String, String, Option<f64>)> {
    estimate(job).allowance_on.into_iter().map(|(path, name, d)| (path, name, d.value)).collect()
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

/// After a roof change that dropped nothing (a pitch made steeper, an eave moved): what
/// the job cost was and is, when what is priced on the roof moved it. None when it did
/// not move, or when either has no figure.
pub fn job_cost_moved_said(before: &Estimate, job: &mut Job) -> Option<String> {
    let (was, is) = (before.job_cost.value?, estimate(job).job_cost.value?);
    ((was - is).abs() >= 0.005).then(|| format!("What is priced on the roof changed with it: the job cost was {} and is {}.", money(was), money(is)))
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
/// A roof priced whole and plane by plane, in one sentence: "Main roof is priced whole,
/// and plane 1 and plane 2 are priced on their own as well: what both carry (its sheathing,
/// its shingles) is in the price twice; take one of the two off". None when no roof is.
pub fn twice_said(twice: &[(String, Vec<String>)]) -> Option<String> {
    let (roof, planes) = twice.first()?;
    let named = match planes.len() {
        1 => format!("{} is", planes[0]),
        n if n <= 3 => format!("{} and {} are", planes[..n - 1].join(", "), planes[n - 1]),
        n => format!("{}, {} and {} more are", planes[0], planes[1], n - 2),
    };
    let more = if twice.len() > 1 { format!(" ({} more roofs the same)", twice.len() - 1) } else { String::new() };
    Some(format!("counted twice: {roof} is priced whole, and {named} priced on {} own as well: what both carry (its sheathing, its shingles) is in the price twice; take one of the two off{more}", if planes.len() == 1 { "its" } else { "their" }))
}

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
    /// their allowances that have a figure come to, to the cent. Each is carried in the
    /// job cost and the price as an allowance until the thing has cost lines (call 114).
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
        let money = sum.map(money).unwrap_or_else(|| "no figure".to_string());
        Some(format!("{n} thing{} priced by an allowance and no cost line ({money}): {} carried in the job cost and the price as {}, and noted as such, until {} given cost lines", if n == 1 { " is" } else { "s are" }, if n == 1 { "it is" } else { "they are" }, if n == 1 { "an allowance" } else { "allowances" }, if n == 1 { "it is" } else { "they are" }))
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
    let twice = on_the_roof_and_a_plane(job);
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
            // What is existing to remain is not bought: nothing is owed on it (call 148).
            if w.get::<Work>(e).is_some_and(|x| x.status.value == Some(WorkStatus::Existing)) {
                continue;
            }
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
    let board_twice = board_on_the_room_and_a_wall(job);
    let lumps = lump_lines(job);
    Pricing { allowance: money_total(&allowances), detail: money_total(&details), both, not_priced: bare, elements, twice, board_twice, lumps }
}

/// A line on a room whose quantity is a plain number, in a division where things set in
/// the room carry lines of their own (call 162): the design-phase allowance ("Cabinets",
/// 1 EA) and the things it stands for, both in the price.
#[derive(Clone, Debug, PartialEq)]
pub struct LumpLine {
    /// The room, by path and name.
    pub room: String,
    pub room_name: String,
    pub line: String,
    /// The plain number the line's quantity is.
    pub quantity: f64,
    pub division: Division,
    /// How many things set in the room carry lines in that division.
    pub things: usize,
}

impl LumpLine {
    /// "Kitchen's line "Cabinets" (1 EA, 12 Furnishings) is in the price beside the lines of
    /// 7 things set in Kitchen in that division: if it stands for them, take the line off."
    pub fn said(&self) -> String {
        let lines = if self.things == 1 { "the line of 1 thing".to_string() } else { format!("the lines of {} things", self.things) };
        format!("{room}'s line {line:?} ({} EA, {} {}) is in the price beside {lines} set in {room} in that division: if it stands for {them}, take the line off.", num(self.quantity), self.division.number(), self.division.title(), room = self.room_name, line = self.line, them = if self.things == 1 { "it" } else { "them" })
    }
}

/// Every such line, once, in the job's order. A thing is set in a room where it stands (its
/// plan's middle is inside the room's outline, on the room's storey); it counts where it
/// carries a line that is not switched off in the line's division. Nothing is taken off.
pub fn lump_lines(job: &mut Job) -> Vec<LumpLine> {
    let outlines = crate::building::room_outlines(job);
    let (drawn, _) = crate::building::placements(job);
    let w = &job.world;
    // The divisions each thing carries a live line in.
    let divisions: std::collections::HashMap<&str, Vec<Division>> = job.order.iter().filter_map(|&e| Some((w.get::<Node>(e)?.path.as_str(), w.get::<Cost>(e)?.lines.iter().filter(|l| !l.off).filter_map(|l| l.division).collect()))).collect();
    let mut out = Vec::new();
    for &e in &job.order {
        let (Some(node), Some(Kind::Room), Some(cost)) = (w.get::<Node>(e), w.get::<Kind>(e).copied(), w.get::<Cost>(e)) else { continue };
        let Some(outline) = outlines.get(&node.path) else { continue };
        for line in cost.lines.iter().filter(|l| !l.off) {
            let (Some(division), Ok(crate::formula::Expr::Num(quantity))) = (line.division, crate::formula::parse(&line.quantity)) else { continue };
            let things = drawn
                .iter()
                .filter(|p| matches!(p.kind, Kind::Furnishing | Kind::Equipment | Kind::Fixture) && p.storey.as_deref() == node.parent.as_deref())
                .filter(|p| matches!(p.shape, crate::building::Shape::Box { at, .. } if crate::vif::inside(outline, [at[0], at[1]])))
                .filter(|p| divisions.get(p.path.as_str()).is_some_and(|d| d.contains(&division)))
                .count();
            if things > 0 {
                out.push(LumpLine { room: node.path.clone(), room_name: node.name.clone(), line: line.description.clone(), quantity, division, things });
            }
        }
    }
    out
}

/// Each roof priced whole whose planes are priced on their own too: (the roof's name,
/// the names of those planes), in the job's order.
fn on_the_roof_and_a_plane(job: &mut Job) -> Vec<(String, Vec<String>)> {
    let children = job.children();
    let w = &job.world;
    // Each element's name and kind, and whether it carries a line that is switched on.
    let mut by_path: std::collections::HashMap<String, (String, Kind, bool)> = std::collections::HashMap::new();
    for &e in &job.order {
        let (Some(node), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        let priced = w.get::<Cost>(e).is_some_and(|c| c.lines.iter().any(|l| !l.off));
        by_path.insert(node.path.clone(), (node.name.clone(), *kind, priced));
    }
    let mut out = Vec::new();
    for &e in &job.order {
        let (Some(node), Some(Kind::Roof)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        let Some((roof, _, true)) = by_path.get(&node.path) else { continue };
        let mut planes: Vec<&String> = children.get(&node.path).map(|c| c.values().collect()).unwrap_or_default();
        planes.sort();
        let priced: Vec<String> = planes.into_iter().filter_map(|p| by_path.get(p)).filter(|(_, k, priced)| k.is_area() && *priced).map(|(n, _, _)| n.clone()).collect();
        if !priced.is_empty() {
            out.push((roof.clone(), priced));
        }
    }
    out
}

/// Does a line's text name wallboard?
fn names_board(text: &str) -> bool {
    let t = text.to_lowercase();
    ["gypsum", "drywall", "sheetrock", "wallboard", "plasterboard"].iter().any(|w| t.contains(w))
}

/// Rooms with a line priced by their walls' area (`wall_area`, call 144) whose walls
/// carry board lines of their own: the same board bought twice (call 146). Each room by
/// its name, with the names of those walls.
fn board_on_the_room_and_a_wall(job: &mut Job) -> Vec<(String, Vec<String>)> {
    let w = &job.world;
    let mut boarded: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for &e in &job.order {
        let (Some(node), Some(Kind::Wall)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        if w.get::<Cost>(e).is_some_and(|c| c.lines.iter().any(|l| !l.off && names_board(&l.description))) {
            boarded.insert(node.path.clone(), node.name.clone());
        }
    }
    let mut out = Vec::new();
    for &e in &job.order {
        let (Some(node), Some(Kind::Room)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
        let by_walls = w.get::<Cost>(e).is_some_and(|c| c.lines.iter().any(|l| !l.off && crate::formula::parse(&l.quantity).is_ok_and(|x| crate::formula::names(&x).iter().any(|n| n == "wall_area"))));
        if !by_walls {
            continue;
        }
        let cited: Vec<String> = w.get::<Measure>(e).and_then(|m| m.0.get("wall_area")).map(|d| d.inputs.iter().filter_map(|i| i.strip_suffix(&format!("#{}", crate::building::K_WALL))).map(str::to_string).collect()).unwrap_or_default();
        let mut walls: Vec<String> = cited.iter().filter_map(|p| boarded.get(p).cloned()).collect();
        walls.dedup();
        if !walls.is_empty() {
            out.push((node.name.clone(), walls));
        }
    }
    out
}

/// "counted twice: Kitchen's walls are priced by its wall area, and wall 1 and wall 2
/// carry board of their own: ...". None when no room is.
pub fn board_twice_said(twice: &[(String, Vec<String>)]) -> Option<String> {
    let (room, walls) = twice.first()?;
    let named = match walls.len() {
        1 => format!("{} carries", walls[0]),
        n if n <= 3 => format!("{} and {} carry", walls[..n - 1].join(", "), walls[n - 1]),
        n => format!("{}, {} and {} more carry", walls[0], walls[1], n - 2),
    };
    let more = if twice.len() > 1 { format!(" ({} more rooms the same)", twice.len() - 1) } else { String::new() };
    Some(format!("counted twice: {room}'s walls are priced by its wall area, and {named} board of {} own: the board on those walls is in the price twice; take one of the two off{more}", if walls.len() == 1 { "its" } else { "their" }))
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
    /// What is priced by an allowance and by no cost line: carried in the job cost and
    /// the price as an allowance, on its own row, until the thing is given cost lines
    /// (Patrick, 2026-11-17: "Allowances are allowed and noted as such"). Each by its
    /// path and name, with what its allowance comes to.
    pub allowances: MoneyTotal,
    pub allowance_on: Vec<(String, String, Derived)>,
    /// The sum of the classes and the allowances. Left out: every classed line with no
    /// figure, every line with no class, and every allowance with no figure.
    pub job_cost: MoneyTotal,
    /// The job cost read division by division: all sixteen, in order, whether or not
    /// anything is in them. Each is over the lines of the job cost in that division.
    pub divisions: Vec<DivisionTotal>,
    /// The lines of the job cost that are in no division. The sixteen, this and
    /// `allowances` come to the job cost.
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

/// One item of the estimate as an estimator writes it: every line of one description,
/// one cost type and one division, bought by one unit, added up. A house with four
/// footings has one line for its footings, with how many places it is made of.
#[derive(Clone, Debug)]
pub struct ItemTotal {
    pub division: Option<crate::components::Division>,
    pub class: Option<Class>,
    pub description: String,
    /// The unit the lines are priced in (CY, SF, hr).
    pub unit: String,
    /// How many lines of the sheet it is the sum of.
    pub places: usize,
    /// What they come to in that unit, when every one of them has a quantity.
    pub quantity: Option<f64>,
    pub quantity_status: Status,
    /// The price each is at, when every one of them is at the same price.
    pub unit_cost: Option<f64>,
    /// How many of its lines have a price at all.
    pub priced_places: usize,
    pub amount: MoneyTotal,
}

impl ItemTotal {
    /// What stands where the unit cost would when the lines are not all at one price:
    /// "no price" where none has one, how many have where some do, and "prices vary"
    /// only where every line is priced and they differ.
    pub fn no_one_price_said(&self) -> String {
        match self.priced_places {
            0 => "no price".to_string(),
            n if n < self.places => format!("{n} of {} priced", self.places),
            _ => "prices vary".to_string(),
        }
    }

    /// The quantity as it is written, for the window and the command line alike: a
    /// count of whole things whole; anything else to two places unless, at two, it
    /// would not multiply by its price to its amount on a calculator, and then to the
    /// four it is held to (the rule a line of the sheet follows, J17A-9; call 120).
    /// None where some line of the item has no quantity.
    pub fn quantity_said(&self) -> Option<String> {
        let q = self.quantity?;
        let places = if self.unit == "EA" && q.fract() == 0.0 {
            0
        } else {
            match (self.unit_cost, self.amount.value) {
                (Some(p), Some(a)) if (crate::lines::cents((q * 100.0).round() / 100.0 * p) - a).abs() > 0.005 => 4,
                _ => 2,
            }
        };
        Some(figure(q, places))
    }
}

impl Estimate {
    /// The lines that are on, read item by item: in the order of the divisions (what is
    /// in none comes last), and within one in the order each item first appears. It is
    /// the same lines and the same amounts as the sheet, added up; nothing is worked
    /// out again. (2026-11-17: the sheet of the sample house was 399 lines, four of
    /// them the same footing line, and read as a list of the model and not an estimate.)
    pub fn by_item(&self) -> Vec<ItemTotal> {
        let mut groups: Vec<((u8, String, String, Option<Class>), Vec<&SheetLine>)> = Vec::new();
        for l in self.lines.iter().filter(|l| !l.off) {
            let key = (l.division.map(|d| d.number()).unwrap_or(u8::MAX), l.description.clone(), l.priced.unit.clone(), l.class);
            match groups.iter_mut().find(|g| g.0 == key) {
                Some(g) => g.1.push(l),
                None => groups.push((key, vec![l])),
            }
        }
        groups.sort_by_key(|g| g.0 .0);
        groups
            .into_iter()
            .map(|((_, description, unit, class), lines)| {
                let parts: Vec<(String, &Derived)> = lines.iter().map(|l| (format!("{}: {:?}", l.path, l.description), &l.amount)).collect();
                let quantities: Vec<f64> = lines.iter().filter_map(|l| l.priced.value).collect();
                let costs: Vec<f64> = lines.iter().filter_map(|l| l.unit_cost.as_ref().and_then(|v| v.value)).collect();
                let one_price = costs.len() == lines.len() && costs.iter().all(|c| (c - costs[0]).abs() < 1.0e-9);
                ItemTotal {
                    division: lines[0].division,
                    class,
                    description,
                    unit,
                    places: lines.len(),
                    quantity: (quantities.len() == lines.len()).then(|| (quantities.iter().sum::<f64>() * 10000.0).round() / 10000.0),
                    quantity_status: lines.iter().map(|l| l.priced.status).max().unwrap_or(Status::Unknown),
                    unit_cost: one_price.then(|| costs[0]),
                    priced_places: costs.len(),
                    amount: total_of(&parts),
                }
            })
            .collect()
    }
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
    // What is priced by an allowance alone (the one rule for it is `Pricing`'s).
    let allowance_on: Vec<(String, String, Derived)> = pricing(job).elements.into_iter().filter(|e| e.detail.is_none()).filter_map(|e| e.allowance.map(|a| (e.path, e.name, a))).collect();
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
    let mut all: Vec<(String, &Derived)> = on.iter().map(|l| (label(l), if l.class.is_some() { &l.amount } else { &no_class })).collect();
    // And every allowance on a thing with no cost line: in the job cost, and so in the
    // price, as an allowance. It is in no cost type's row and no division: it has a
    // row of its own in the recap, and is counted with what the sixteen leave out.
    let carried: Vec<(String, &Derived)> = allowance_on.iter().map(|(path, _, d)| (format!("{path}: its allowance"), d)).collect();
    let allowances = total_of(&carried);
    all.extend(carried.iter().cloned());
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
    // What is carried as an allowance is in the job cost and in neither of these: it
    // is `allowances`, on a row of its own (call 123). The sixteen, the lines in no
    // division and the allowances come to the job cost.
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
    let no_cost = "there is no job cost: no line with a cost type and no allowance comes to a figure";
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
    Estimate { lines, columns, classes, not_classed, allowances, allowance_on, job_cost, divisions, no_division, markup, overhead, profit, price, suppliers, at_generic, at_supplier }
}

// ---------------------------------------------------------------------------------------
// Two or three layouts side by side (call 154).

/// One figure of one layout: what its own job's takeoff or estimate gives it.
#[derive(Clone, Debug, PartialEq)]
pub enum OptionFigure {
    /// The layout has nothing of this: no such room, no wall of this kind, no work in it.
    None,
    /// It has the thing and the thing has no figure.
    NoFigure,
    Is(f64, Status),
}

/// Which part of the table a row belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionGroup {
    Rooms,
    Walls,
    Cost,
    Divisions,
}

#[derive(Clone, Debug)]
pub struct OptionRow {
    pub group: OptionGroup,
    pub label: String,
    /// "SF", "LF", "$" or "lines".
    pub unit: &'static str,
    /// One figure for each layout, in the order they were given.
    pub figures: Vec<OptionFigure>,
    /// The last layout less the first; not worked where either has no value.
    pub difference: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Options {
    /// Each layout's job title, in the order given.
    pub titles: Vec<String>,
    pub rows: Vec<OptionRow>,
}

impl OptionFigure {
    /// As a figure is written beside its flag: "~368.125 SF", "$5,777.00", "none", "no figure".
    pub fn said(&self, unit: &str) -> String {
        match self {
            OptionFigure::None => "none".to_string(),
            OptionFigure::NoFigure => "no figure".to_string(),
            OptionFigure::Is(x, _) if unit == "lines" => match *x as usize {
                0 => "none".to_string(),
                1 => "1 line".to_string(),
                n => format!("{n} lines"),
            },
            OptionFigure::Is(x, status) => {
                let flag = status.flag().trim_end();
                if unit == "$" { format!("{flag}{}", money(*x)) } else { format!("{flag}{} {unit}", num(*x)) }
            }
        }
    }
}

impl OptionRow {
    /// The difference as a builder writes it: "+62 SF", "-8 LF", "+$1,360.00", "same";
    /// "not worked" where a layout has no value.
    pub fn difference_said(&self) -> String {
        let Some(d) = self.difference else { return "not worked".to_string() };
        let tiny = if self.unit == "$" { 0.005 } else { 0.00005 };
        if d.abs() < tiny {
            return "same".to_string();
        }
        let sign = if d < 0.0 { "-" } else { "+" };
        if self.unit == "lines" {
            return format!("{sign}{} line{}", num(d.abs()), if d.abs() == 1.0 { "" } else { "s" });
        }
        if self.unit == "$" { format!("{sign}{}", money(d.abs())) } else { format!("{sign}{} {}", num(d.abs()), self.unit) }
    }
}

/// The layouts set side by side. Every figure is the one the job's own takeoff and estimate
/// work out (a room's floor area and a wall's plan length as the takeoff adds them, the
/// job cost and each division as the estimate has them); this adds up nothing of its own
/// but the differences. Nothing in a job is changed.
pub fn options(jobs: &mut [&mut Job]) -> Options {
    type Fig = (String, Option<f64>, Status);
    let from_sum = |s: &Sum| match (s.members, s.value) {
        (0, _) => OptionFigure::None,
        (_, Some(x)) => OptionFigure::Is(x, s.status),
        (_, None) => OptionFigure::NoFigure,
    };
    let from_money = |t: &MoneyTotal| match t.value {
        Some(x) => OptionFigure::Is(x, t.status),
        None if t.left_out.is_empty() => OptionFigure::None,
        None => OptionFigure::NoFigure,
    };
    let mut titles = Vec::new();
    // Rooms by name, in the order each is first met; walls by what the work does.
    let mut room_names: Vec<String> = Vec::new();
    let mut rooms: Vec<(Vec<(String, Vec<Fig>)>, Vec<Fig>)> = Vec::new();
    let mut walls: Vec<[Vec<Fig>; 3]> = Vec::new();
    let mut estimates = Vec::new();
    for job in jobs.iter_mut() {
        let root = job.root();
        titles.push(job.world.get::<JobInfo>(root).map(|i| i.title.clone()).unwrap_or_default());
        let (mut by_name, mut all): (Vec<(String, Vec<Fig>)>, Vec<Fig>) = (Vec::new(), Vec::new());
        let mut by_work: [Vec<Fig>; 3] = Default::default();
        let w = &job.world;
        for &e in &job.order {
            let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
            let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (n.path.clone(), d.value, d.status)).unwrap_or((n.path.clone(), None, Status::Unknown));
            match kind {
                Kind::Room => {
                    let f = figure("floor_area");
                    if !room_names.contains(&n.name) {
                        room_names.push(n.name.clone());
                    }
                    match by_name.iter_mut().find(|(name, _)| *name == n.name) {
                        Some((_, v)) => v.push(f.clone()),
                        None => by_name.push((n.name.clone(), vec![f.clone()])),
                    }
                    all.push(f);
                }
                Kind::Wall => {
                    // A thing with no mark is new; a temporary wall is counted with the new.
                    let at = match w.get::<Work>(e).and_then(|k| k.status.value) {
                        None | Some(WorkStatus::New) | Some(WorkStatus::Temporary) => 0,
                        Some(WorkStatus::Existing) => 1,
                        Some(WorkStatus::Demolish) => 2,
                    };
                    by_work[at].push(figure("plan_length"));
                }
                _ => {}
            }
        }
        rooms.push((by_name, all));
        walls.push(by_work);
        estimates.push(estimate(job));
    }
    let mut rows: Vec<OptionRow> = Vec::new();
    let mut push = |group: OptionGroup, label: String, unit: &'static str, figures: Vec<OptionFigure>| {
        let difference = match (figures.first(), figures.last()) {
            (Some(OptionFigure::Is(a, _)), Some(OptionFigure::Is(b, _))) if figures.len() > 1 => Some(crate::value::round4(b - a)),
            _ => None,
        };
        rows.push(OptionRow { group, label, unit, figures, difference });
    };
    for name in &room_names {
        let figures = rooms.iter().map(|(by_name, _)| by_name.iter().find(|(n, _)| n == name).map(|(_, f)| from_sum(&sum_of("SF", f))).unwrap_or(OptionFigure::None)).collect();
        push(OptionGroup::Rooms, name.clone(), "SF", figures);
    }
    push(OptionGroup::Rooms, "all rooms".into(), "SF", rooms.iter().map(|(_, all)| from_sum(&sum_of("SF", all))).collect());
    for (at, label) in ["walls, new (with temporary)", "walls, existing to remain", "walls, to demolish"].into_iter().enumerate() {
        push(OptionGroup::Walls, label.into(), "LF", walls.iter().map(|w| from_sum(&sum_of("LF", &w[at]))).collect());
    }
    push(OptionGroup::Cost, "job cost".into(), "$", estimates.iter().map(|e| match e.job_cost.value { Some(x) => OptionFigure::Is(x, e.job_cost.status), None => OptionFigure::NoFigure }).collect());
    // What each job cost leaves out, so two job costs are not set side by side as if both
    // were whole (call 157): the lines the estimate has no figure for.
    push(OptionGroup::Cost, "left out of the job cost".into(), "lines", estimates.iter().map(|e| OptionFigure::Is(e.job_cost.left_out.len() as f64, Status::Known)).collect());
    for (i, d) in Division::ALL.iter().enumerate() {
        let totals: Vec<&MoneyTotal> = estimates.iter().map(|e| &e.divisions[i].total).collect();
        if totals.iter().any(|t| t.members > 0 || !t.left_out.is_empty()) {
            push(OptionGroup::Divisions, format!("{:>2} {}", d.number(), d.title()), "$", totals.into_iter().map(from_money).collect());
        }
    }
    // What the job cost holds that is in none of the sixteen is said, so the rows add up.
    let others: [(&str, fn(&Estimate) -> &MoneyTotal); 2] = [("Allowances", |e| &e.allowances), ("in no division", |e| &e.no_division)];
    for (label, of) in others {
        let totals: Vec<&MoneyTotal> = estimates.iter().map(of).collect();
        if totals.iter().any(|t| t.members > 0 || !t.left_out.is_empty()) {
            push(OptionGroup::Divisions, label.into(), "$", totals.into_iter().map(from_money).collect());
        }
    }
    Options { titles, rows }
}
