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
    /// A run through a wall or a floor: a hole to cut, not two things in each other's
    /// way. Where it is is still to verify; nothing priced rides on the hole itself.
    pub through: bool,
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
    /// The room it is in, by name, where a room's outline on that storey holds the
    /// place it is at. None for a thing in no room (a wall, a thing outside, a roof).
    pub room: Option<String>,
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
    /// How sure its dollars are (its price and its quantity together).
    status: Status,
    /// How sure its quantity is: what a measurement can make surer.
    quantity: Status,
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

    // The planes a roof's footprint was solved into, each with its roof.
    let solved_from: BTreeMap<String, String> = {
        let mut q = job.world.query::<(&crate::components::Node, &crate::components::Solved)>();
        q.iter(&job.world).map(|(n, s)| (n.path.clone(), s.from.clone())).collect()
    };

    // What was traced over each drawing, each thing with its drawing: a figure traced
    // rests on the sheet's scale as well as on the thing it was traced as.
    let traced_over: BTreeMap<String, String> = {
        let drawings: Vec<String> = elements.iter().filter(|e| e.kind == Kind::Drawing).map(|e| e.path.clone()).collect();
        let mut over = BTreeMap::new();
        for drawing in drawings {
            for thing in crate::plans::traced_over(job, &drawing).unwrap_or_default() {
                over.insert(thing, drawing.clone());
            }
        }
        over
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
        // What rests on a plane solved from a roof's footprint rests on that roof: the
        // footprint and the pitches are the roof's, and it is at the roof that they
        // are checked (J22A-18).
        for roof in rests.iter().filter_map(|p| solved_from.get(p)).cloned().collect::<Vec<_>>() {
            rests.insert(roof);
        }
        // And what rests on a thing traced over a drawing rests on that drawing's scale.
        for drawing in rests.iter().filter_map(|p| traced_over.get(p)).cloned().collect::<Vec<_>>() {
            rests.insert(drawing);
        }
        lines.push(Line { key: (sheet.path.clone(), sheet.description.clone()), on: sheet.path.clone(), amount, status: if amount.is_some() { sheet.amount.status } else { sheet.priced.status }, quantity: sheet.quantity.status, rests });
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
            // A hole to cut that is still in question is a question too (J17C-2): it is
            // on the clash list as one to verify, and so it is here. Nothing rides on
            // it in dollars: neither thing has to move, a hole has to be cut.
            for c in found.found.iter().filter(|c| c.status != Status::Known) {
                let picked: Vec<&Line> = if c.through { Vec::new() } else { lines.iter().filter(|l| l.on == c.a || l.on == c.b).collect() };
                counted.extend(picked.iter().map(|l| l.key.clone()));
                let (dollars, dollars_status, n, unpriced) = sum(&picked);
                place(
                    Question {
                        path: c.a.clone(),
                        name: c.a_name.clone(),
                        kind: c.a_kind,
                        storey: storey_name(&c.a, c.a_kind),
                        room: None,
                        clash: Some(Between { other: c.b.clone(), other_name: c.b_name.clone(), how: c.how, through: c.through, by: c.by, at: c.at }),
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
        // What the thing is, where that is itself to be checked (see below). Worked out
        // before the next step adds values that are not of that kind.
        let identity = values.iter().any(|v| !v.2);
        let mut values = values;
        if values.is_empty() {
            // Nothing on it is a value `measure` takes, but priced lines whose quantity
            // is in question rest on it. It is a question all the same, to be checked,
            // and their dollars ride on it: left off, an estimator sorting his field
            // questions by dollars is never sent to it (J22A-18 was a roof drawn in the
            // window, which `measure` now reaches; this is for whatever it does not).
            // A plane solved from a roof's footprint is not one of those: what is left
            // in question on it is the roof's, and the roof is on the list with it. Nor
            // is a drawing whose scale has been checked: where the sheet sits on the
            // plan is not a thing to find on site.
            if solved_from.contains_key(&el.path) || el.kind == Kind::Drawing || !lines.iter().any(|l| l.quantity != Status::Known && l.rests.contains(&el.path)) {
                continue;
            }
            // What says what it is priced as, or where it is filed, is not checked on site.
            let not_on_site = |label: &str| {
                let l = label.to_lowercase().replace('_', " ");
                ["cost code", "assembly", "condition", "wbs", "material", "allowance"].iter().any(|w| l.contains(w))
            };
            values = labels.iter().filter(|l| !not_on_site(l)).map(|label| (label.clone(), el.rows.iter().find(|r| r.label == *label).map(|r| r.status).unwrap_or(Status::Unknown), false)).collect();
            if values.is_empty() {
                continue;
            }
        }
        // Only a line that is itself still in question has anything riding on a
        // measurement. But where what the thing is (its type, what it is, its size) is
        // itself to be checked, everything priced on it rides on that check, however
        // sure its quantity (call 59, J17A-14): a price for a 50 gal heater is a price
        // for nothing if the heater is not that.
        // What rides on a measurement is a line whose quantity is itself in question:
        // a line of a Known length at an Assumed price is not made surer by a tape,
        // and its dollars do not send anyone to measure (call 83, M21-2).
        let picked: Vec<&Line> = lines.iter().filter(|l| (l.quantity != Status::Known && l.rests.contains(&el.path)) || (identity && l.on == el.path)).collect();
        counted.extend(picked.iter().map(|l| l.key.clone()));
        let (dollars, dollars_status, n, unpriced) = sum(&picked);
        place(Question { path: el.path.clone(), name: el.name.clone(), kind: el.kind, storey: storey_name(&el.path, el.kind), room: None, clash: None, values, dollars, dollars_status, lines: n, unpriced }, &mut with, &mut without);
    }

    // Largest first; a tie keeps the order found, which is clashes and then the job's.
    with.sort_by(|a, b| b.dollars.unwrap_or(0.0).total_cmp(&a.dollars.unwrap_or(0.0)));
    let all: Vec<&Line> = lines.iter().filter(|l| counted.contains(&l.key)).collect();
    let (riding, riding_status, riding_lines, riding_unpriced) = sum(&all);
    with.extend(without);
    // Where each is, to the room: a list a person walks with says "first floor, Bath",
    // not "first floor" (call 117). A clash is where the two meet; a thing is where it
    // stands, or the middle of its run.
    let rooms: Vec<(String, String, Vec<[f64; 2]>)> = crate::building::room_outlines(job)
        .into_iter()
        .filter_map(|(path, outline)| {
            let e = job.find(&path)?;
            let n = job.world.get::<crate::Node>(e)?;
            Some((n.parent.clone()?, n.name.clone(), outline))
        })
        .collect();
    let placed = crate::building::placements(job).0;
    for q in with.iter_mut() {
        let Some(p) = placed.iter().find(|p| p.path == q.path) else { continue };
        let Some(storey) = &p.storey else { continue };
        let at = match (&q.clash, &p.shape) {
            (Some(c), _) => [c.at[0], c.at[1]],
            (None, crate::building::Shape::Box { at, .. }) => [at[0], at[1]],
            (None, crate::building::Shape::Run { a, b, .. }) => [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0],
            _ => continue,
        };
        q.room = rooms.iter().find(|(of, _, outline)| of == storey && inside(outline, at)).map(|r| r.1.clone());
    }
    Vif { questions: with, riding, riding_status, riding_lines, riding_unpriced, clashes_not_tested }
}

/// Whether a place lies inside an outline on plan.
fn inside(outline: &[[f64; 2]], at: [f64; 2]) -> bool {
    let mut odd = false;
    for i in 0..outline.len() {
        let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
        if (a[1] > at[1]) != (b[1] > at[1]) && at[0] < a[0] + (at[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            odd = !odd;
        }
    }
    odd
}

impl Question {
    /// What it is, as the list names it: the thing, or the two things of a clash.
    pub fn what(&self) -> String {
        // A run is drawn piece by piece and each piece is named "..., piece 2 of 3":
        // on the job it is the run a person goes to, so the piece is not said.
        let run = |name: &str| -> String {
            match name.rfind(", piece ") {
                Some(i) if name[i + 8..].split(" of ").count() == 2 && name[i + 8..].split(" of ").all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())) => name[..i].to_string(),
                _ => name.to_string(),
            }
        };
        match &self.clash {
            Some(c) => format!("{} and {}", run(&self.name), run(&c.other_name)),
            None => run(&self.name),
        }
    }

    /// Where it is, as a person finds it: the level, and the room where it is in one.
    pub fn where_said(&self) -> String {
        match (&self.room, self.storey.is_empty()) {
            (Some(room), false) => format!("{}, {room}", self.storey),
            (Some(room), true) => room.clone(),
            (None, _) => self.storey.clone(),
        }
    }

    /// What to do about it on site, in one sentence. Inches are written as a tape
    /// reads them (1/2 in, not 0.50 in). One wording for the window, the command line
    /// and the list that is written out.
    pub fn to_do(&self) -> String {
        match &self.clash {
            Some(c) if c.through => "One passes through the other: a hole to cut. Where is still to verify.".to_string(),
            Some(c) => {
                let by = c.by.map(|by| format!(", by {}", crate::units::show_length(&crate::units::Len::One((by * 12.0 * 16.0).round() / 16.0 + 0.0), "in"))).unwrap_or_default();
                format!("In the same space{by}. Measure either one.")
            }
            None => {
                // "Measure its width and its height; check what it is and its size."
                let said = |name: &str| match name {
                    "condition" => "its assembly".to_string(),
                    n if n.starts_with("what ") => n.to_string(),
                    n => format!("its {}", n.replace('_', " ")),
                };
                let listed = |length: bool| -> String {
                    let all: Vec<String> = self.values.iter().filter(|v| v.2 == length).map(|v| said(&v.0)).collect();
                    match all.as_slice() {
                        [] => String::new(),
                        [one] => one.clone(),
                        [most @ .., last] => format!("{} and {last}", most.join(", ")),
                    }
                };
                let (measure, check) = (listed(true), listed(false));
                let mut to_do = Vec::new();
                if !measure.is_empty() {
                    to_do.push(format!("Measure {measure}"));
                }
                if !check.is_empty() {
                    to_do.push(format!("{} {check}", if to_do.is_empty() { "Check" } else { "check" }));
                }
                format!("{}.", to_do.join("; "))
            }
        }
    }
}

/// The list as a sheet a person takes to the job: one row a question, largest dollars
/// first, with a box to tick and a blank for what was found. Text only: it is printed,
/// written on, and typed back in at the window.
pub fn field_list(job: &mut Job) -> Vec<Vec<String>> {
    let list = vif(job);
    let mut rows = vec![vec!["Done".to_string(), "Riding on it".to_string(), "Where".to_string(), "What".to_string(), "What to do".to_string(), "As found".to_string()]];
    for q in &list.questions {
        rows.push(vec![
            "[  ]".to_string(),
            q.dollars.map(view::money).unwrap_or_else(|| "no dollars".to_string()),
            q.where_said(),
            q.what(),
            q.to_do(),
            String::new(),
        ]);
    }
    rows
}
