//! The verify-in-field list: each question the job still asks of the building, where it
//! is, what to measure or check to settle it, and the dollars riding on it.
//!
//! A question is one of two things (DECISIONS.md call 54):
//!
//! * a thing with a value that can be measured or checked on site and is not Known;
//! * a clash that rests on anything not Known (one that rests only on what is Known is
//!   a finding, and is not on this list).
//!
//! The dollars riding on a thing are what the estimate's lines come to whose quantity
//! is worked from a figure that rests on that thing's values, and whose amount is
//! itself not Known. The dollars riding on a clash are what the lines on the two things
//! in it come to. A question nothing priced rests on has no dollars, which is not
//! $0.00. Nothing here is kept beside the job: it is read from the figures the job
//! already works out and from what each says it rests on.

use std::collections::{BTreeMap, BTreeSet};

use crate::building::clash::{self, ClashKind};
use crate::value::Status;
use crate::{formula, view, Cost, Job, Kind, Measure, P3};

/// What two things in the same space are, for a question that is a clash.
#[derive(Clone, Debug, PartialEq)]
pub struct Between {
    pub other: String,
    pub other_name: String,
    pub how: ClashKind,
    /// How far one is into the other, or apart, in feet, when that is worked out.
    pub by: Option<f64>,
    /// Where, in feet: east, north, height above the datum.
    pub at: P3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    /// The thing to go to: the thing itself, or the first of the two in a clash.
    pub path: String,
    pub name: String,
    pub kind: Kind,
    /// The storey it is on, by name; a storey's own name for a storey.
    pub storey: String,
    /// For a clash: what it is with.
    pub clash: Option<Between>,
    /// What to measure (true) or check (false) to settle it: each value as the screen
    /// names it, with how sure it is now. Empty for a clash, whose two things each
    /// have a row of their own when they have such a value.
    pub values: Vec<(String, Status, bool)>,
    /// What rides on it, to the cent. None when nothing priced rests on it.
    pub dollars: Option<f64>,
    pub dollars_status: Status,
    /// How many of the estimate's lines that is.
    pub lines: usize,
    /// Lines that ride on it and have no figure yet (no price entered): they are in
    /// no dollar figure here, and are counted so that is said (J17A-7).
    pub unpriced: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vif {
    /// Largest first; ties in the job's order; those with no dollars last, clashes
    /// before things.
    pub questions: Vec<Question>,
    /// What rides on all of them together, each line counted once. None when nothing does.
    pub riding: Option<f64>,
    pub riding_status: Status,
    pub riding_lines: usize,
    /// Lines that ride on some question and have no figure yet, each counted once.
    pub riding_unpriced: usize,
    /// Why the clashes could not be worked out, when they could not: the list is then
    /// of things only, and says so.
    pub clashes_not_tested: Option<String>,
}

impl Default for Vif {
    fn default() -> Vif {
        Vif { questions: Vec::new(), riding: None, riding_status: Status::Known, riding_lines: 0, riding_unpriced: 0, clashes_not_tested: None }
    }
}

/// One line of the estimate as this list needs it: its place, what it comes to, and
/// the things its quantity is worked from.
struct Line {
    key: (String, String),
    on: String,
    /// None: the line has no figure yet.
    amount: Option<f64>,
    status: Status,
    rests: BTreeSet<String>,
}

pub fn vif(job: &mut Job) -> Vif {
    let elements = view::elements(job);
    let estimate = view::estimate(job);
    let storeys = crate::building::storeys_of(job);
    let names: BTreeMap<&str, &str> = elements.iter().map(|e| (e.path.as_str(), e.name.as_str())).collect();
    let storey_name = |path: &str, kind: Kind| -> String {
        if kind == Kind::Storey {
            return names.get(path).copied().unwrap_or(path).to_string();
        }
        storeys.get(path).and_then(|s| names.get(s.as_str()).copied()).unwrap_or("").to_string()
    };

    // Every live line that comes to something, with what its quantity is worked from:
    // the things the measures its formula names say they rest on.
    let mut lines: Vec<Line> = Vec::new();
    for sheet in estimate.lines.iter().filter(|l| !l.off) {
        let amount = sheet.amount.value;
        let Some(e) = job.find(&sheet.path) else { continue };
        let mut rests = BTreeSet::new();
        let formula = job.world.get::<Cost>(e).and_then(|c| c.lines.iter().find(|l| l.description == sheet.description)).map(|l| l.quantity.clone());
        if let (Some(Ok(expr)), Some(measure)) = (formula.map(|f| formula::parse(&f)), job.world.get::<Measure>(e)) {
            for name in formula::names(&expr) {
                if let Some(d) = measure.0.get(&name) {
                    rests.insert(sheet.path.clone());
                    for input in &d.inputs {
                        rests.insert(input.split('#').next().unwrap_or("").to_string());
                    }
                }
            }
        }
        lines.push(Line { key: (sheet.path.clone(), sheet.description.clone()), on: sheet.path.clone(), amount, status: if amount.is_some() { sheet.amount.status } else { sheet.priced.status }, rests });
    }
    let cents = crate::lines::cents;
    let sum = |picked: &[&Line]| -> (Option<f64>, Status, usize, usize) {
        let unpriced = picked.iter().filter(|l| l.amount.is_none()).count();
        let priced: Vec<&&Line> = picked.iter().filter(|l| l.amount.is_some()).collect();
        if priced.is_empty() {
            return (None, Status::Known, 0, unpriced);
        }
        (Some(cents(priced.iter().filter_map(|l| l.amount).sum())), priced.iter().map(|l| l.status).max().unwrap_or(Status::Known), priced.len(), unpriced)
    };

    let mut with: Vec<Question> = Vec::new();
    let mut without: Vec<Question> = Vec::new();
    let mut counted: BTreeSet<(String, String)> = BTreeSet::new();
    let place = |q: Question, with: &mut Vec<Question>, without: &mut Vec<Question>| {
        if q.dollars.is_some() {
            with.push(q);
        } else {
            without.push(q);
        }
    };

    // The clashes that are questions. What rides on one is what is priced on the two
    // things in it: what may have to move.
    let mut clashes_not_tested = None;
    match clash::clashes(job, None) {
        Err(why) => clashes_not_tested = Some(why),
        Ok(found) => {
            for c in found.found.iter().filter(|c| !c.through && c.status != Status::Known) {
                let picked: Vec<&Line> = lines.iter().filter(|l| l.on == c.a || l.on == c.b).collect();
                counted.extend(picked.iter().map(|l| l.key.clone()));
                let (dollars, dollars_status, n, unpriced) = sum(&picked);
                place(
                    Question {
                        path: c.a.clone(),
                        name: c.a_name.clone(),
                        kind: c.a_kind,
                        storey: storey_name(&c.a, c.a_kind),
                        clash: Some(Between { other: c.b.clone(), other_name: c.b_name.clone(), how: c.how, by: c.by, at: c.at }),
                        values: Vec::new(),
                        dollars,
                        dollars_status,
                        lines: n,
                        unpriced,
                    },
                    &mut with,
                    &mut without,
                );
            }
        }
    }

    // The things with a value that can be measured or checked and is not Known.
    for el in &elements {
        let labels: Vec<String> = el.rows.iter().filter(|r| !r.derived && !r.label.contains(": ") && r.status != Status::Known).map(|r| r.label.clone()).collect();
        if labels.is_empty() {
            continue;
        }
        let values: Vec<(String, Status, bool)> = crate::editing::takes_among(job, &el.path, &labels)
            .into_iter()
            .map(|(label, length)| {
                let status = el.rows.iter().find(|r| r.label == label).map(|r| r.status).unwrap_or(Status::Unknown);
                (label, status, length)
            })
            .collect();
        if values.is_empty() {
            continue;
        }
        // Only a line that is itself still in question has anything riding on a
        // measurement. But where what the thing is (its type, what it is, its size) is
        // itself to be checked, everything priced on it rides on that check, however
        // sure its quantity (call 59, J17A-14): a price for a 50 gal heater is a price
        // for nothing if the heater is not that.
        let identity = values.iter().any(|v| !v.2);
        let picked: Vec<&Line> = lines.iter().filter(|l| (l.status != Status::Known && l.rests.contains(&el.path)) || (identity && l.on == el.path)).collect();
        counted.extend(picked.iter().map(|l| l.key.clone()));
        let (dollars, dollars_status, n, unpriced) = sum(&picked);
        place(Question { path: el.path.clone(), name: el.name.clone(), kind: el.kind, storey: storey_name(&el.path, el.kind), clash: None, values, dollars, dollars_status, lines: n, unpriced }, &mut with, &mut without);
    }

    // Largest first; a tie keeps the order found, which is clashes and then the job's.
    with.sort_by(|a, b| b.dollars.unwrap_or(0.0).total_cmp(&a.dollars.unwrap_or(0.0)));
    let all: Vec<&Line> = lines.iter().filter(|l| counted.contains(&l.key)).collect();
    let (riding, riding_status, riding_lines, riding_unpriced) = sum(&all);
    with.extend(without);
    Vif { questions: with, riding, riding_status, riding_lines, riding_unpriced, clashes_not_tested }
}
