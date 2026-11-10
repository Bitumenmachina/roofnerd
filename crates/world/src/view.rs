//! A flat reading of the world: every value on every element, entered or derived, with
//! its status. The command line prints this; the sheet and the 3D view will read the
//! same rows. Nothing here computes a quantity. It only reports what the world holds.

use crate::components::*;
use crate::job::Job;
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
}

#[derive(Clone, Debug)]
pub struct ElementRows {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    pub depth: usize,
    pub rows: Vec<Row>,
}

fn num(x: f64) -> String {
    let s = format!("{x:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

fn entered<T>(path: &str, label: &str, v: &Val<T>, show: impl Fn(&T) -> String) -> Row {
    let text = match &v.value {
        Some(x) => format!("{} {}", show(x), v.unit).trim().to_string(),
        None => "unknown".to_string(),
    };
    Row { path: path.into(), label: label.into(), text, status: v.status, basis: v.source.clone(), note: String::new(), derived: false }
}

fn derived(path: &str, label: &str, d: &Derived) -> Row {
    let text = match d.value {
        Some(x) => format!("{} {}", num(x), d.unit).trim().to_string(),
        None => "unknown".to_string(),
    };
    Row { path: path.into(), label: label.into(), text, status: d.status, basis: d.formula.clone(), note: d.note.clone(), derived: true }
}

/// Every element in job order, each with all of its values.
pub fn elements(job: &mut Job) -> Vec<ElementRows> {
    let mut out = Vec::new();
    let order = job.order.clone();
    let w = &job.world;
    let depth_of = |mut parent: Option<String>| -> usize {
        let mut d = 0;
        while let Some(p) = parent {
            d += 1;
            parent = order.iter().filter_map(|e| w.get::<Node>(*e)).find(|n| n.path == p).and_then(|n| n.parent.clone());
        }
        d
    };
    for &e in &order {
        let node = w.get::<Node>(e).expect("every element has a node");
        let kind = *w.get::<Kind>(e).expect("every element has a kind");
        let p = &node.path;
        let mut rows = Vec::new();
        if let Some(o) = w.get::<Outline>(e) {
            rows.push(entered(p, "outline", &o.points, |pts| format!("{} points", pts.len())));
            rows.push(entered(p, "eave height", &o.eave_height, |x| num(*x)));
        }
        if let Some(c) = w.get::<Pitch>(e) {
            rows.push(entered(p, "pitch", &c.0, |x| num(*x)));
        }
        if let Some(c) = w.get::<Run>(e) {
            rows.push(entered(p, "line ends", &c.ends, |[a, b]| format!("({}, {}) to ({}, {})", num(a[0]), num(a[1]), num(b[0]), num(b[1]))));
        }
        if let Some(c) = w.get::<Height>(e) {
            rows.push(entered(p, "height", &c.0, |x| num(*x)));
        }
        if let Some(c) = w.get::<Condition>(e) {
            rows.push(entered(p, "condition", &c.0, |s| s.clone()));
        }
        if let Some(c) = w.get::<Cost>(e) {
            rows.push(entered(p, "cost code", &c.code, |k| format!("{} {} {}", k.scheme, k.code, k.title)));
            for line in &c.lines {
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
        out.push(ElementRows { path: p.clone(), name: node.name.clone(), kind, depth: depth_of(node.parent.clone()), rows });
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
