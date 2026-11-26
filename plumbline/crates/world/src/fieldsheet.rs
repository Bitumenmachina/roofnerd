//! The field sheet (call 234): the verify-in-field list as a sheet to fill on site, and the
//! measurements brought back by file.
//!
//! Out: a row for each VALUE to measure (not for each question: "check its size" is a
//! width, a depth and a height), by the dollars riding on its question, each with a
//! reference of its own that names one value of one thing, what the job has for it now and
//! how sure, and blanks: As found, Measured by, On.
//!
//! In: each row with something As found puts that value Known by the very functions a typed
//! measurement goes through (`editing::measure`, `editing::measure_on_line`); nothing about
//! what follows is worked out here. All of it or none of it: every row is tried, in order,
//! on a copy of the job and every refusal is named by its row; only when none is refused
//! are the rows put on the job itself.
//!
//! Ours. BIMCraft has no field sheet and no certainty on a value to bring back (a survey of
//! its crates for a field sheet, a punch list or a measured value finds nothing to port).

use std::collections::{BTreeMap, BTreeSet};

use crate::building::clash;
use crate::editing::{self, measure::AGREE_FT, Reading};
use crate::value::Status;
use crate::{file, units, view, vif, Cost, Job, Justify};

/// The headings of the sheet, in the order it is written. A sheet read back is read by these
/// headings, not by where its columns lie.
pub const HEADINGS: [&str; 10] = ["Where", "What", "What to measure", "The job has", "How sure", "Riding on its question", "As found", "Measured by", "On", "Reference"];

/// One value to measure.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub where_said: String,
    pub what: String,
    pub to_do: String,
    /// The value as the job holds it when the sheet goes out, as the screen writes it.
    pub had: String,
    /// How sure it is then: Assumed or Unknown.
    pub how_sure: Status,
    /// What rides on the question the value belongs to; None when nothing priced does.
    pub dollars: Option<f64>,
    /// Which value of which thing: `wall-0001 level height`, or for a quantity typed on a line
    /// `wall-0001 "Gypsum board" measured`.
    pub reference: String,
}

/// Something of the list that is not a row, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct Left {
    pub where_said: String,
    pub what: String,
    pub why: String,
    /// True when the whole question has no row; false for one value of a question that has others.
    pub whole: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    pub rows: Vec<Row>,
    pub left: Vec<Left>,
    /// How many questions the list has.
    pub questions: usize,
}

impl Sheet {
    /// How many questions are on no row.
    pub fn questions_without_rows(&self) -> usize {
        self.left.iter().filter(|l| l.whole).count()
    }

    /// What is written out: the headings, a row for each value, then a note for what is not a row
    /// (its Reference is empty, so it is never read back as a measurement).
    pub fn table(&self) -> Vec<Vec<String>> {
        let mut out = vec![HEADINGS.iter().map(|h| h.to_string()).collect::<Vec<_>>()];
        for r in &self.rows {
            out.push(vec![
                r.where_said.clone(),
                r.what.clone(),
                r.to_do.clone(),
                r.had.clone(),
                status_said(r.how_sure).to_string(),
                r.dollars.map(view::money).unwrap_or_else(|| "no dollars".to_string()),
                String::new(),
                String::new(),
                String::new(),
                r.reference.clone(),
            ]);
        }
        for l in &self.left {
            out.push(vec![l.where_said.clone(), l.what.clone(), format!("Not a row: {}", l.why), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new()]);
        }
        out
    }
}

fn status_said(s: Status) -> &'static str {
    match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    }
}

/// Which value of which thing a reference names.
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Value { path: String, label: String },
    Line { path: String, line: String, name: String },
}

impl Target {
    fn path(&self) -> &str {
        match self {
            Target::Value { path, .. } | Target::Line { path, .. } => path,
        }
    }
}

fn reference_of(t: &Target) -> String {
    match t {
        Target::Value { path, label } => format!("{path} {label}"),
        Target::Line { path, line, name } => format!("{path} \"{line}\" {name}"),
    }
}

fn parse_reference(text: &str) -> Result<Target, String> {
    let text = text.trim();
    let bad = || format!("{text:?} is not a reference this program wrote (a thing, then the value: wall-0001 level height)");
    let (path, rest) = text.split_once(' ').ok_or_else(bad)?;
    let rest = rest.trim();
    if path.is_empty() || rest.is_empty() {
        return Err(bad());
    }
    if let Some(inner) = rest.strip_prefix('"') {
        let end = inner.rfind("\" ").ok_or_else(bad)?;
        let (line, name) = (&inner[..end], inner[end + 2..].trim());
        if name.is_empty() {
            return Err(bad());
        }
        return Ok(Target::Line { path: path.to_string(), line: line.to_string(), name: name.to_string() });
    }
    Ok(Target::Value { path: path.to_string(), label: rest.to_string() })
}

/// What the job holds for one value now.
#[derive(Clone, Debug)]
struct Has {
    text: String,
    status: Status,
    source: String,
    /// The figure, in feet for a length and in its own unit for a quantity typed on a line.
    figure: Option<f64>,
    unit: String,
}

/// The values of every element as the screen has them, by thing and name, and where each lies in the job's own order.
struct Values(BTreeMap<(String, String), view::Row>, BTreeMap<String, usize>, BTreeMap<(String, String), usize>);

impl Values {
    fn of(job: &mut Job) -> Values {
        let mut m = BTreeMap::new();
        let (mut things, mut places) = (BTreeMap::new(), BTreeMap::new());
        for (i, el) in view::elements(job).into_iter().enumerate() {
            things.insert(el.path.clone(), i);
            for (j, r) in el.rows.into_iter().filter(|r| !r.derived).enumerate() {
                places.insert((el.path.clone(), r.label.clone()), j);
                m.insert((el.path.clone(), r.label.clone()), r);
            }
        }
        Values(m, things, places)
    }

    /// Where a value lies in the job's own order: the things in the order the job lists them (the levels from the bottom up, then what
    /// stands on them), a thing's values in the order the screen shows them, its cost lines after them. The floor above follows the floor
    /// below, so what a sheet measures is put on the job in this order, whatever order the sheet's rows were left in.
    fn rank(&self, job: &mut Job, t: &Target) -> (usize, usize) {
        let thing = self.1.get(t.path()).copied().unwrap_or(usize::MAX);
        match t {
            Target::Value { path, label } => (thing, self.2.get(&(path.clone(), label.clone())).copied().unwrap_or(usize::MAX)),
            Target::Line { path, line, .. } => {
                let at = job.find(path).and_then(|e| job.world.get::<Cost>(e)).and_then(|c| c.lines.iter().position(|l| &l.description == line)).unwrap_or(0);
                (thing, 1_000_000 + at)
            }
        }
    }

    fn has(&self, job: &mut Job, t: &Target) -> Option<Has> {
        match t {
            Target::Value { path, label } => {
                let r = self.0.get(&(path.clone(), label.clone()))?;
                let figure = units::parse_length(&r.text).ok().and_then(|(len, unit)| len.feet(unit).ok());
                Some(Has { text: r.text.clone(), status: r.status, source: r.basis.clone(), figure, unit: String::new() })
            }
            Target::Line { path, line, name } => {
                let e = job.find(path)?;
                let cost = job.world.get::<Cost>(e)?;
                let v = cost.lines.iter().find(|l| &l.description == line)?.inputs.get(name)?;
                let (text, figure) = match &v.value {
                    Some(len) => (units::show_length(len, &v.unit), if let units::Len::One(x) = len { Some(*x) } else { None }),
                    None => ("no figure yet".to_string(), None),
                };
                Some(Has { text, status: v.status, source: v.source.clone(), figure, unit: v.unit.clone() })
            }
        }
    }
}

/// The sheet for a job: its verify-in-field list, a row for each value a tape or a laser can answer.
pub fn make(job: &mut Job) -> Sheet {
    let list = vif::vif(job);
    let values = Values::of(job);
    let mut rows = Vec::new();
    let mut left = Vec::new();
    for q in &list.questions {
        let (where_said, what) = (q.where_said(), q.what());
        if let Some(c) = &q.clash {
            let why = if c.through {
                "a run through a wall or a floor is a hole to cut, and where it is is looked at: there is no length to type for it"
            } else if c.set_against {
                "a thing set against another is looked at: there is no length to type for it"
            } else {
                "two things in the same space are settled by measuring either one: each has a row of its own where it has a value to measure"
            };
            left.push(Left { where_said, what, why: why.to_string(), whole: true });
            continue;
        }
        if q.in_the_way.is_some() {
            left.push(Left { where_said, what, why: "what to do about a thing in the way of a wall that comes down (cap it, reroute it or take it out) is decided, not measured".to_string(), whole: true });
            continue;
        }
        let e = job.find(&q.path);
        let lines: Vec<crate::CostLine> = e.and_then(|e| job.world.get::<Cost>(e)).map(|c| c.lines.clone()).unwrap_or_default();
        let mut made = 0;
        let mut not_rows: Vec<String> = Vec::new();
        for (label, _, length) in &q.values {
            // A quantity typed on a cost line: measured with the line and the name it is entered under.
            let typed = lines.iter().find_map(|l| vif::typed_unsure_named(l).filter(|(_, shown, _)| shown == label).map(|(name, _, _)| Target::Line { path: q.path.clone(), line: l.description.clone(), name }));
            let target = match typed {
                Some(t) => t,
                None => {
                    if editing::takes_among(job, &q.path, std::slice::from_ref(label)).is_empty() {
                        not_rows.push(vif::to_do_word(label));
                        continue;
                    }
                    Target::Value { path: q.path.clone(), label: label.clone() }
                }
            };
            let Some(has) = values.has(job, &target) else {
                not_rows.push(vif::to_do_word(label));
                continue;
            };
            let to_do = match &target {
                Target::Line { line, name, .. } => format!("Measure the quantity of the line \"{line}\" ({}), with its unit, like 72 SF.", name.replace('_', " ")),
                Target::Value { label, .. } if *length => {
                    let mut said = format!("Measure {}, with its unit.", vif::to_do_word(label));
                    if label.starts_with("height above its level") && q.kind.is_segment() {
                        said.push_str(" Put bottom, center or top before the reading to say where it was shot to.");
                    }
                    said
                }
                Target::Value { label, .. } => format!("Check {}. Write same if it is as drawn.", vif::to_do_word(label)),
            };
            rows.push(Row { where_said: where_said.clone(), what: what.clone(), to_do, had: has.text, how_sure: has.status, dollars: q.dollars, reference: reference_of(&target) });
            made += 1;
        }
        if made == 0 {
            let why = if not_rows.is_empty() { "nothing on it is a value to measure; priced lines rest on it and it is looked at".to_string() } else { format!("{} looked at, not measured: no reading is typed for it", not_rows.join(", ")) };
            left.push(Left { where_said, what, why, whole: true });
        } else if !not_rows.is_empty() {
            left.push(Left { where_said, what, why: format!("{} looked at, not measured: no reading is typed for it", not_rows.join(", ")), whole: false });
        }
    }
    Sheet { rows, left, questions: list.questions.len() }
}

/// One row of a sheet read back, as the cells hold it.
struct Filled {
    number: usize,
    reference: String,
    target: Target,
    had: Option<String>,
    how: Option<String>,
    found: String,
    by: String,
    on: String,
}

/// What putting one row on the job did.
#[derive(Clone, Debug, PartialEq)]
pub struct Done {
    /// The row's number in the file.
    pub number: usize,
    pub reference: String,
    /// What a typed measurement says of it, and what else followed.
    pub said: String,
    pub also: Vec<String>,
    /// The price before and after: None where there is no price yet.
    pub price: (Option<f64>, Option<f64>),
    /// Which clash was confirmed (is now a finding) or cleared, and any new one.
    pub clashes: Vec<String>,
}

impl Done {
    /// The reply for this row, in the order a person reads it.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!("row {}: {}", self.number, self.said)];
        out.extend(self.also.iter().map(|a| format!("  {a}")));
        out.push(format!("  {}", match self.price {
            (Some(a), Some(b)) if (a - b).abs() < 0.005 => format!("the price is {}, as it was", view::money(b)),
            (Some(a), Some(b)) => format!("the price was {} and is {}", view::money(a), view::money(b)),
            (None, Some(b)) => format!("the price is {} (it had no figure before)", view::money(b)),
            (Some(a), None) => format!("the price was {} and has no figure now", view::money(a)),
            (None, None) => "the price has no figure yet".to_string(),
        }));
        out.extend(self.clashes.iter().map(|c| format!("  {c}")));
        out
    }
}

/// What was brought in.
#[derive(Clone, Debug, PartialEq)]
pub struct Brought {
    pub done: Vec<Done>,
    /// Rows with a reference and nothing As found: nothing was done with them.
    pub blank: usize,
    /// Questions still to verify in the field, and what rides on them.
    pub questions_left: usize,
    pub riding: Option<f64>,
    pub riding_status: Status,
}

impl Brought {
    pub fn closing(&self) -> String {
        let measured = match self.done.len() {
            0 => "no value was measured".to_string(),
            1 => "1 value was measured".to_string(),
            n => format!("{n} values were measured"),
        };
        let blank = match self.blank {
            0 => String::new(),
            1 => "; 1 row was blank and left alone".to_string(),
            n => format!("; {n} rows were blank and left alone"),
        };
        let left = match self.questions_left {
            0 => "no question is left to verify in the field".to_string(),
            1 => "1 question is left to verify in the field".to_string(),
            n => format!("{n} questions are left to verify in the field"),
        };
        let them = if self.questions_left == 1 { "it" } else { "them" };
        let riding = match self.riding {
            Some(x) => format!(", with {} still riding on {them}{}", view::money(x), if self.riding_status == Status::Known { "" } else { " (it rests on something not yet measured)" }),
            None if self.questions_left == 0 => String::new(),
            None => format!(", and nothing priced rides on {them}"),
        };
        format!("{measured}{blank}; {left}{riding}.")
    }
}

/// A column of the sheet's headings, by its heading (capitals and spaces do not matter).
fn column(heading: &[String], name: &str) -> Option<usize> {
    heading.iter().position(|h| h.trim().eq_ignore_ascii_case(name))
}

/// The day a workbook turned a typed date into, written as a day; a date a person typed as text is left as it is.
fn day_said(cell: &str) -> String {
    let cell = cell.trim();
    let Some(inner) = cell.strip_prefix("a date or a time (").and_then(|s| s.strip_suffix(')')) else { return cell.to_string() };
    if let Some(n) = inner.strip_prefix("day ").and_then(|d| d.trim().parse::<f64>().ok()) {
        // A workbook counts days from the 30th of December 1899.
        return civil(n.floor() as i64 - 25569);
    }
    inner.chars().take(10).collect()
}

/// A day number (days since 1970-01-01) as YYYY-MM-DD.
fn civil(days: i64) -> String {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{m:02}-{d:02}", if m <= 2 { y + 1 } else { y })
}

/// What was found, as it was written on the row.
enum Found {
    Same,
    Length(String, Option<Justify>),
}

fn read_found(text: &str) -> Result<Found, String> {
    let text = text.trim();
    if text.starts_with("a date or a time") {
        return Err("the workbook turned what was written into a date. Write it as a length with its unit (8 ft 6 in), and format the cell as text".to_string());
    }
    if text.eq_ignore_ascii_case("same") || text.eq_ignore_ascii_case("as drawn") {
        return Ok(Found::Same);
    }
    // What a run's height was shot to, before the reading or after it, as the window takes it.
    let line = |w: &&str| match w.to_ascii_lowercase().as_str() {
        "bottom" => Some(Justify::Bottom),
        "top" => Some(Justify::Top),
        "centre" | "center" => Some(Justify::Centre),
        _ => None,
    };
    let mut words: Vec<&str> = text.split_whitespace().collect();
    let mut to = None;
    if let Some(j) = words.first().and_then(line) {
        to = Some(j);
        words.remove(0);
    } else if let Some(j) = words.last().and_then(line) {
        to = Some(j);
        words.pop();
    }
    let reading = words.join(" ");
    if reading.is_empty() {
        return Err("it says where a height was shot to and gives no length".to_string());
    }
    Ok(Found::Length(reading, to))
}

/// Whether a reading is the figure the job already has (to an eighth of an inch for a length).
fn agrees(has: &Has, found: &str) -> bool {
    match (has.figure, &has.unit[..]) {
        (Some(x), "") => units::parse_length(found).ok().and_then(|(len, unit)| len.feet(unit).ok()).is_some_and(|y| (x - y).abs() <= AGREE_FT),
        (Some(x), unit) => units::read_measure(found, true).and_then(|r| r.ok()).is_some_and(|(y, u)| u == unit && (x - y).abs() < 1.0e-9),
        (None, _) => false,
    }
}

/// How a row will be put on the job.
struct Plan {
    rank: (usize, usize),
    number: usize,
    reference: String,
    target: Target,
    reading: Option<(String, Option<Justify>)>,
    source: String,
}

fn put_on(job: &mut Job, plan: &Plan) -> Result<editing::Measured, String> {
    let reading = match &plan.reading {
        Some((text, _)) => Reading::Length(text),
        None => Reading::Same,
    };
    let to = plan.reading.as_ref().and_then(|(_, to)| *to);
    match &plan.target {
        Target::Value { path, label } => editing::measure(job, path, label, reading, to, &plan.source),
        Target::Line { path, line, name } => editing::measure_on_line(job, path, line, name, reading, &plan.source),
    }
}

/// The clashes that are questions, by the two things in each, as they stand.
fn clash_state(job: &mut Job) -> Option<Vec<clash::Clash>> {
    clash::clashes(job, None).ok().map(|c| c.found.into_iter().filter(|c| !c.laid_in).collect())
}

fn price_of(job: &mut Job) -> Option<f64> {
    view::estimate(job).price.value
}

/// What a measurement did to the clashes: which was cleared, which is now a finding, which is new.
fn clash_said(before: &Option<Vec<clash::Clash>>, after: &Option<Vec<clash::Clash>>) -> Vec<String> {
    let (Some(before), Some(after)) = (before, after) else { return Vec::new() };
    let key = |c: &clash::Clash| (c.a.clone(), c.b.clone(), c.through, c.set_against);
    let now: BTreeMap<_, _> = after.iter().map(|c| (key(c), c)).collect();
    let was: BTreeSet<_> = before.iter().map(key).collect();
    let by = |c: &clash::Clash| match (c.how, c.by) {
        (clash::ClashKind::Hard, Some(by)) if !c.through => format!(", in each other by {}", units::length_said(by)),
        _ => String::new(),
    };
    let mut out = Vec::new();
    for c in before.iter().filter(|c| c.status != Status::Known) {
        match now.get(&key(c)) {
            None => out.push(format!("clash cleared: {} and {} are no longer in each other's way", c.a_name, c.b_name)),
            Some(n) if n.status == Status::Known => out.push(format!("clash confirmed, now a finding: {} and {}{}; everything it rests on is Known", n.a_name, n.b_name, by(n))),
            Some(_) => {}
        }
    }
    for c in after.iter().filter(|c| !was.contains(&key(c)) && c.status != Status::Known && c.how == clash::ClashKind::Hard && !c.through && !c.set_against) {
        out.push(format!("a new clash to verify in the field: {} and {}{}", c.a_name, c.b_name, by(c)));
    }
    out
}

/// Bring the rows of a filled sheet in. `table` is the sheet as read (`plumbline_sheets::Sheet::rows`), `numbers` each row's
/// number in its file, `name` what the file is called (it is written into each measurement's source). All of it or none of it:
/// on any refusal the job is as it was and every refusal comes back, each named by its row.
pub fn bring(job: &mut Job, table: &[Vec<String>], numbers: &[usize], name: &str, anyway: bool) -> Result<Brought, Vec<String>> {
    let head = table
        .iter()
        .position(|r| column(r, "Reference").is_some() && column(r, "As found").is_some())
        .ok_or_else(|| vec!["this is not a field sheet: no row of it has the headings Reference and As found".to_string()])?;
    let heading = &table[head];
    let (c_ref, c_found) = (column(heading, "Reference").unwrap_or(0), column(heading, "As found").unwrap_or(0));
    let (c_by, c_on, c_had, c_how) = (column(heading, "Measured by"), column(heading, "On"), column(heading, "The job has"), column(heading, "How sure"));
    let cell = |row: &Vec<String>, c: Option<usize>| c.and_then(|c| row.get(c)).map(|s| s.trim().to_string()).unwrap_or_default();

    let mut refused: Vec<String> = Vec::new();
    let mut blank = 0;
    let mut filled: Vec<Filled> = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (i, row) in table.iter().enumerate().skip(head + 1) {
        let number = numbers.get(i).copied().unwrap_or(i + 1);
        let (reference, found) = (cell(row, Some(c_ref)), cell(row, Some(c_found)));
        if found.is_empty() {
            if !reference.is_empty() {
                blank += 1;
            }
            continue;
        }
        if reference.is_empty() {
            refused.push(format!("row {number}: has something as found ({found}) and no reference: it cannot be told which value it measures"));
            continue;
        }
        let target = match parse_reference(&reference) {
            Ok(t) => t,
            Err(why) => {
                refused.push(format!("row {number}: {why}"));
                continue;
            }
        };
        let key = reference_of(&target);
        if let Some(first) = seen.get(&key) {
            refused.push(format!("row {number}: measures the same value as row {first} ({key}); one row for each value"));
            continue;
        }
        seen.insert(key, number);
        let had = c_had.map(|_| cell(row, c_had));
        let how = c_how.map(|_| cell(row, c_how));
        filled.push(Filled { number, reference, target, had, how, found, by: cell(row, c_by), on: day_said(&cell(row, c_on)), });
    }

    // What each row says, read against the job as it stands now, before any row is put on it.
    let values = Values::of(job);
    let mut plans: Vec<Plan> = Vec::new();
    for f in &filled {
        let who = &f.reference;
        let Some(has) = values.has(job, &f.target) else {
            let why = if job.find(f.target.path()).is_none() { format!("the job has no {}", f.target.path()) } else { format!("{} has no value like that to measure", f.target.path()) };
            refused.push(format!("row {} ({who}): {why}", f.number));
            continue;
        };
        let found = match read_found(&f.found) {
            Ok(x) => x,
            Err(why) => {
                refused.push(format!("row {} ({who}): {why}", f.number));
                continue;
            }
        };
        // Is it the same value the sheet went out with?
        let stamp = match (&f.had, &f.how) {
            (Some(h), Some(k)) => Some((h.clone(), k.clone())),
            _ => None,
        };
        let status = status_said(has.status);
        let unchanged = stamp.as_ref().is_some_and(|(h, k)| h.split_whitespace().collect::<Vec<_>>() == has.text.split_whitespace().collect::<Vec<_>>() && k.eq_ignore_ascii_case(status));
        let (reading, agrees_now) = match &found {
            Found::Same => (None, false),
            Found::Length(text, to) => (Some((text.clone(), *to)), agrees(&has, text)),
        };
        let mut reading = reading;
        // A value already Known that the reading agrees with is found the same again; it keeps who measured it first.
        if has.status == Status::Known && agrees_now && !matches!(found, Found::Same) {
            reading = None;
        }
        let how_said = |had: &str| format!("the sheet had {had} and the job has {}, {status} ({})", has.text, has.source);
        if !(unchanged || agrees_now || anyway) {
            let had = match &stamp {
                Some((h, k)) => format!("{h}, {k}"),
                None => "no figure of its own (it has no The job has and How sure columns)".to_string(),
            };
            let since = if has.status == Status::Known { " It has been measured since the sheet went out." } else { "" };
            refused.push(format!("row {} ({who}): {}, and it says {}.{since} It is not put over what the job has without --anyway", f.number, how_said(&had), f.found.trim()));
            continue;
        }
        if has.status == Status::Known && !agrees_now && !anyway {
            refused.push(format!("row {} ({who}): the job has {}, Known ({}); this row says {}. A second measurement that differs is not put over it without --anyway", f.number, has.text, has.source, f.found.trim()));
            continue;
        }
        let by = if f.by.is_empty() { "measured in the field (the row names no one)".to_string() } else { format!("measured in the field by {}", f.by) };
        let on = if f.on.is_empty() { String::new() } else { format!(" on {}", f.on) };
        plans.push(Plan { rank: values.rank(job, &f.target), number: f.number, reference: f.reference.clone(), target: f.target.clone(), reading, source: format!("{by}{on}: field sheet {name:?}, row {}", f.number) });
    }

    // In the job's own order, not the sheet's: the same rows come to the same job however the sheet was sorted or cut down.
    plans.sort_by_key(|p| (p.rank, p.number));
    // Each row tried, in order, on a copy; every refusal is kept. Nothing of the job is touched.
    if refused.is_empty() && !plans.is_empty() {
        match file::load(&file::save(job)) {
            Ok((mut trial, _)) => {
                for p in &plans {
                    if let Err(why) = put_on(&mut trial, p) {
                        refused.push(format!("row {} ({}): {why}", p.number, p.reference));
                    }
                }
            }
            Err(why) => refused.push(format!("the job could not be tried before it was changed: {why}")),
        }
    }
    if !refused.is_empty() {
        refused.push(format!("nothing was brought in: {}", if refused.len() == 1 { "the row above is refused" } else { "the rows above are refused" }));
        return Err(refused);
    }

    // And now on the job itself, all of it or none of it.
    let done = editing::all_or_nothing(job, |job| {
        let mut done = Vec::new();
        let mut clashes = clash_state(job);
        let mut price = price_of(job);
        for p in &plans {
            let m = put_on(job, p).map_err(|why| format!("row {} ({}): {why}", p.number, p.reference))?;
            let (now_clashes, now_price) = (clash_state(job), price_of(job));
            done.push(Done { number: p.number, reference: p.reference.clone(), said: m.said, also: m.also, price: (price, now_price), clashes: clash_said(&clashes, &now_clashes) });
            clashes = now_clashes;
            price = now_price;
        }
        Ok(done)
    })
    .map_err(|why| vec![why])?;
    let list = vif::vif(job);
    Ok(Brought { done, blank, questions_left: list.questions.len(), riding: list.riding, riding_status: list.riding_status })
}
