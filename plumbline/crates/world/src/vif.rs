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
    /// A thing standing against a wall, or a run at its joint, in the other by no more than
    /// two inches (call 175): not two things in one space.
    pub set_against: bool,
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
    /// For a thing in a wall that comes down (call 160): which wall, and where.
    pub in_the_way: Option<crate::hidden::InTheWay>,
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
    /// What the program says of a wall the work opens besides the values to check (call
    /// 176): what stands on it, and which opening's header is to be sized. Each is a
    /// sentence, with what it is worked from.
    pub also: Vec<String>,
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
    /// Its quantity is worked from a measure typed on the line that is not Known: a tape
    /// on the building settles it (call 167).
    typed: bool,
}

/// A measure typed on a cost line (a value entered in LF, SF, SQ, CY or EA, and named by
/// the line's formula) that is not Known: what to measure, as the list names it, and how
/// sure it is. An assembly's rule values (a strip's width, a swell) are not in these
/// units and are the estimator's allowances, not a question of the site.
fn typed_unsure(line: &crate::CostLine) -> Option<(String, Status)> {
    let names = formula::parse(&line.quantity).map(|e| formula::names(&e)).unwrap_or_default();
    let mut unsure: Vec<(&String, &crate::Val<crate::units::Len>)> = line.inputs.iter().filter(|(n, v)| names.contains(n) && crate::units::is_measure_unit(&v.unit) && v.status != Status::Known).collect();
    unsure.sort_by(|a, b| a.0.cmp(b.0));
    let (_, v) = unsure.first()?;
    let typed = match &v.value {
        Some(len) => crate::units::show_length(len, &v.unit),
        None => "no figure yet".to_string(),
    };
    Some((format!("\"{}\" ({typed})", line.description), v.status))
}

pub fn vif(job: &mut Job) -> Vif {
    let elements = view::elements(job);
    let estimate = view::estimate(job);
    let storeys = crate::building::storeys_of(job);
    let opened = crate::hidden::opened_walls(&mut job.world);
    // What stands on a wall the work opens, and the headers of the openings it cuts (call 176).
    let mut carried = crate::carries::asked(job);
    for (wall, mut sentences) in crate::carries::headers_asked(job) {
        carried.entry(wall).or_default().append(&mut sentences);
    }
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
        let typed = job.world.get::<Cost>(e).and_then(|c| c.lines.iter().find(|l| l.description == sheet.description)).is_some_and(|l| typed_unsure(l).is_some());
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
        lines.push(Line { key: (sheet.path.clone(), sheet.description.clone()), on: sheet.path.clone(), amount, status: if amount.is_some() { sheet.amount.status } else { sheet.priced.status }, quantity: sheet.quantity.status, rests, typed });
    }
    // What is carried as an allowance (a thing priced by an allowance and by no cost
    // line) rides as a line does: it is an area or a length times a rate, and where
    // the thing it is on is not Known a tape can move it (call 123; J27A-11: the
    // largest number in a job was in no dollar figure of this list).
    for (path, _, d) in &estimate.allowance_on {
        let Some(e) = job.find(path) else { continue };
        let mut rests: BTreeSet<String> = BTreeSet::from([path.clone()]);
        let mut quantity = Status::Known;
        for input in &d.inputs {
            let (of, name) = input.split_once('#').unwrap_or((input.as_str(), ""));
            rests.insert(of.to_string());
            // How sure what it is measured by is: the rate is not a thing to tape.
            if let Some(m) = job.find(of).and_then(|x| job.world.get::<Measure>(x)).and_then(|m| m.0.get(name)) {
                quantity = quantity.max(m.status);
                rests.extend(m.inputs.iter().map(|i| i.split('#').next().unwrap_or("").to_string()));
            }
        }
        if d.inputs.is_empty() {
            quantity = job.world.get::<Measure>(e).map(|m| m.0.values().map(|x| x.status).max().unwrap_or(Status::Known)).unwrap_or(d.status);
        }
        for roof in rests.iter().filter_map(|p| solved_from.get(p)).cloned().collect::<Vec<_>>() {
            rests.insert(roof);
        }
        for drawing in rests.iter().filter_map(|p| traced_over.get(p)).cloned().collect::<Vec<_>>() {
            rests.insert(drawing);
        }
        lines.push(Line { key: (path.clone(), "its allowance".to_string()), on: path.clone(), amount: d.value, status: d.status, quantity, rests, typed: false });
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
            // (A hole is one question however many pieces of the run pass through it.)
            let mut holes_seen: BTreeSet<usize> = BTreeSet::new();
            // (A run laid in a floor or a wall is no hole and no clash: it is not a question, call 179 C5.)
            for c in found.found.iter().filter(|c| c.status != Status::Known && !c.laid_in) {
                if let Some(h) = c.hole.filter(|_| c.through) {
                    if !holes_seen.insert(h) {
                        continue;
                    }
                }
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
                        clash: Some(Between { other: c.b.clone(), other_name: c.b_name.clone(), how: c.how, through: c.through, set_against: c.set_against, by: c.by, at: c.at }),
                        in_the_way: None,
                        values: Vec::new(),
                        dollars,
                        dollars_status,
                        lines: n,
                        unpriced,
                        also: Vec::new(),
                    },
                    &mut with,
                    &mut without,
                );
            }
        }
    }

    // The things with a value that can be measured or checked and is not Known.
    for el in &elements {
        // An item is a thing that can be placed, not a thing in the building: it is
        // on this list only through what is set down from it (J27A-27: a carpenter
        // was sent to measure a sofa nobody had set down).
        if job.find(&el.path).is_some_and(|e| job.world.get::<crate::Item>(e).is_some()) {
            continue;
        }
        let mut labels: Vec<String> = el.rows.iter().filter(|r| !r.derived && !r.label.contains(": ") && r.status != Status::Known).map(|r| r.label.clone()).collect();
        // What a wall the work opens may hide is asked of it, as things to check; of a
        // wall the work does not open nothing is asked (call 160).
        let is_hidden = |l: &String| l == crate::hidden::BEARING || l == crate::hidden::IN_THE_WALL;
        let asked_hidden: Vec<String> = if opened.contains_key(&el.path) { labels.iter().filter(|l| is_hidden(l)).cloned().collect() } else { Vec::new() };
        labels.retain(|l| !is_hidden(l));
        // A measure typed on a line that is not Known is a measurement of the building,
        // and a field question under its thing, by the line and what was typed (call 167).
        let typed: Vec<(String, Status)> = match job.find(&el.path) {
            Some(e) => job.world.get::<Cost>(e).map(|c| c.lines.iter().filter_map(typed_unsure).collect()).unwrap_or_default(),
            None => Vec::new(),
        };
        let also: Vec<String> = carried.get(&el.path).cloned().unwrap_or_default();
        if labels.is_empty() && asked_hidden.is_empty() && typed.is_empty() && also.is_empty() {
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
        let identity = values.iter().any(|v| !v.2) || !asked_hidden.is_empty() || !also.is_empty();
        let mut values = values;
        if values.is_empty() && asked_hidden.is_empty() && typed.is_empty() && also.is_empty() {
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
        // What was typed on a line to measure: that line's dollars ride on it.
        for (label, status) in &typed {
            values.push((label.clone(), *status, true));
        }
        // Everything priced on the wall rides on what it is, as on what any thing is.
        for label in &asked_hidden {
            let status = el.rows.iter().find(|r| r.label == *label).map(|r| r.status).unwrap_or(Status::Unknown);
            values.push((label.clone(), status, false));
        }
        // Only a line that is itself still in question has anything riding on a
        // measurement. But where what the thing is (its type, what it is, its size) is
        // itself to be checked, everything priced on it rides on that check, however
        // sure its quantity (call 59, J17A-14): a price for a 50 gal heater is a price
        // for nothing if the heater is not that.
        // What rides on a measurement is a line whose quantity is itself in question:
        // a line of a Known length at an Assumed price is not made surer by a tape,
        // and its dollars do not send anyone to measure (call 83, M21-2).
        let picked: Vec<&Line> = lines.iter().filter(|l| (l.quantity != Status::Known && l.rests.contains(&el.path)) || (identity && l.on == el.path) || (l.typed && l.on == el.path)).collect();
        counted.extend(picked.iter().map(|l| l.key.clone()));
        let (dollars, dollars_status, n, unpriced) = sum(&picked);
        place(Question { path: el.path.clone(), name: el.name.clone(), kind: el.kind, storey: storey_name(&el.path, el.kind), room: None, clash: None, in_the_way: None, values, dollars, dollars_status, lines: n, unpriced, also }, &mut with, &mut without);
    }

    // What is in the way of a wall that comes down: cap it, reroute it or take it out.
    // What rides on it is what is priced on that thing (call 160).
    match clash::in_the_way(job) {
        Err(why) => clashes_not_tested = clashes_not_tested.or(Some(why)),
        Ok(found) => {
            for w in found {
                let picked: Vec<&Line> = lines.iter().filter(|l| l.on == w.thing).collect();
                counted.extend(picked.iter().map(|l| l.key.clone()));
                let (dollars, dollars_status, n, unpriced) = sum(&picked);
                place(
                    Question { path: w.thing.clone(), name: w.thing_name.clone(), kind: w.thing_kind, storey: storey_name(&w.thing, w.thing_kind), room: None, clash: None, in_the_way: Some(w), values: Vec::new(), dollars, dollars_status, lines: n, unpriced, also: Vec::new() },
                    &mut with,
                    &mut without,
                );
            }
        }
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
pub(crate) fn inside(outline: &[[f64; 2]], at: [f64; 2]) -> bool {
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
            // A roof is on no level: where it is is the roof (it was a blank).
            (None, true) if matches!(self.kind, Kind::Roof | Kind::RoofPlane) => "Roof".to_string(),
            (None, _) => self.storey.clone(),
        }
    }

    /// What to do about it on site, in one sentence. Inches are written as a tape
    /// reads them (1/2 in, not 0.50 in). One wording for the window, the command line
    /// and the list that is written out.
    pub fn to_do(&self) -> String {
        if let Some(w) = &self.in_the_way {
            return w.said();
        }
        match &self.clash {
            Some(c) if c.through => "One passes through the other: a hole to cut. Where is still to verify.".to_string(),
            Some(c) if c.set_against => "Set against the other, not in its way. Where it stands is still to verify.".to_string(),
            Some(c) => {
                let by = c.by.map(|by| format!(", by {}", crate::units::show_length(&crate::units::Len::One((by * 12.0 * 16.0).round() / 16.0 + 0.0), "in"))).unwrap_or_default();
                format!("In the same space{by}. Measure either one.")
            }
            None => {
                // "Measure its width and its height; check what it is and its size."
                let said = |name: &str| match name {
                    "condition" => "its assembly".to_string(),
                    n if n == crate::hidden::BEARING => "whether it is bearing".to_string(),
                    n if n == crate::hidden::IN_THE_WALL => "what is in the wall".to_string(),
                    n if n.starts_with("what ") => n.to_string(),
                    // A measure typed on a line: the line and what was typed, as they read.
                    n if n.starts_with('"') => n.to_string(),
                    // A room is kept by a point inside it: on site that is where it is.
                    "a point inside it" => "where it is".to_string(),
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
                let mut said = if to_do.is_empty() { String::new() } else { format!("{}.", to_do.join("; ")) };
                // What the program says of what stands on it and of its headers (call 176).
                for s in &self.also {
                    if !said.is_empty() {
                        said.push(' ');
                    }
                    said.push_str(s);
                }
                said
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
