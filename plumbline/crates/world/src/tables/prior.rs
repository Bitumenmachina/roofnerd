use super::*;

// ---------------------------------------------------------------- a prior estimate

pub const K_PRIOR: &str = "pl::prior";

/// One line of an estimate made before, as it was written: nothing here is worked out.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorLine {
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub class: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub code: String,
    /// What the line is on, its division, how sure the estimate says it is, and its
    /// note: the columns this program's own estimate writes beside each line, kept so
    /// that an estimate written out and read back as a prior estimate loses nothing
    /// (J17B-25).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub element: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub division: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sure: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

/// `pl::prior`: on a Prior element under the job, an estimate made before, brought in to
/// be set against this one. It is no part of the job's price: nothing adds its lines to
/// anything. How sure it is and where it came from are whoever brought it in's to say.
#[derive(bevy_ecs::component::Component, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prior {
    pub lines: Val<Vec<PriorLine>>,
}

/// The most lines one prior estimate may hold.
pub const MOST_PRIOR_LINES: usize = 50_000;

pub fn check_prior(path: &str, kind: Kind, p: &Prior) -> Result<(), String> {
    if kind != Kind::Prior {
        return Err(format!("{path}: a {kind:?} cannot carry a prior estimate's lines; only a Prior element can"));
    }
    p.lines.check_basics(&format!("{path} lines"))?;
    let lines = p.lines.value.as_deref().unwrap_or(&[]);
    if lines.len() > MOST_PRIOR_LINES {
        return Err(format!("{path}: holds {} lines; a prior estimate is no more than {MOST_PRIOR_LINES}", lines.len()));
    }
    for (i, l) in lines.iter().enumerate() {
        if l.description.trim().is_empty() {
            return Err(format!("{path}: its line {} has no description", i + 1));
        }
        for (what, x) in [("quantity", l.quantity), ("unit cost", l.unit_cost), ("amount", l.amount)] {
            if x.is_some_and(|x| !x.is_finite() || x.abs() > check::MAX_UNIT_COST) {
                return Err(format!("{path}: the {what} of its line {:?} is not a figure this program can hold", l.description));
            }
        }
    }
    Ok(())
}

pub(super) const PRIOR_COLUMNS: [(&str, &[&str]); 11] = [
    ("description", &["description", "line item", "scope", "work", "line", "item"]),
    ("quantity", &["quantity", "qty"]),
    ("unit", &["unit", "uom", "u/m", "um"]),
    ("unit cost", &["unit cost", "unit price", "rate", "price"]),
    ("amount", &["amount", "total", "extension", "ext", "line total", "extended"]),
    ("class", &["class", "category", "cost type"]),
    ("code", &["cost code", "code"]),
    ("element", &["element", "location", "area"]),
    ("division", &["division", "div"]),
    ("sure", &["how sure"]),
    ("note", &["note", "notes", "remarks"]),
];

/// A figure of money or of quantity as an estimate writes it: `1,204.50`, `$450`.
pub(super) fn money_cell(text: &str) -> Result<Option<f64>, String> {
    if text.trim().trim_start_matches('$').trim().is_empty() {
        return Ok(None);
    }
    list_figure(text).map(Some)
}

/// Whether a row of an estimate is a total of the rows above it and not a line: it is
/// called a total, or begins with the word and gives no quantity and no unit cost.
pub(super) fn is_total_row(description: &str, quantity: Option<f64>, unit_cost: Option<f64>) -> bool {
    let d = heading(description).replace('-', " ");
    let d = d.trim_end_matches(['$', ' ']);
    matches!(d, "total" | "totals" | "subtotal" | "sub total" | "grand total" | "sum" | "total cost" | "total price" | "job total" | "estimate total" | "total estimate" | "total bid" | "bid total")
        || (quantity.is_none() && unit_cost.is_none() && ["total ", "subtotal ", "sub total ", "grand total "].iter().any(|w| d.starts_with(w)))
}

/// Bring an estimate made before into the job, as an element called `name`, to be set
/// against the job's own lines by `compare`. Returns its path and what was read.
pub fn prior_in(job: &mut Job, name: &str, rows: &[Vec<String>], numbers: Option<&[usize]>, how: Entered) -> Result<(String, Brought), String> {
    let cols = columns(rows, &PRIOR_COLUMNS, &["description"]).ok_or_else(|| "this is not an estimate this program can read: within its first rows it has no row of headings with a column for what each line is (Description or Item)".to_string())?;
    if !(cols.has("amount") || (cols.has("quantity") && cols.has("unit cost"))) {
        return Err("the estimate has no column for what each line comes to (Amount or Total), nor both a quantity and a unit cost".into());
    }
    if name.trim().is_empty() {
        return Err("a prior estimate has a name".into());
    }
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len().saturating_sub(cols.row + 1), columns_not_used: cols.not_used.clone(), above: cols.row, ..Brought::default() };
        let mut lines = Vec::new();
        // What the estimate's own total rows say, to set against the lines read.
        let mut said_totals: Vec<(usize, String, f64)> = Vec::new();
        for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
            let description = cols.cell(row, "description");
            let number = row_number(numbers, i);
            if description.is_empty() {
                out.not_used.push((number, "it says nothing of what the line is".into()));
                continue;
            }
            let figures = (|| -> Result<(Option<f64>, Option<f64>, Option<f64>), String> { Ok((money_cell(cols.cell(row, "quantity"))?, money_cell(cols.cell(row, "unit cost"))?, money_cell(cols.cell(row, "amount"))?)) })();
            match figures {
                Err(why) => out.not_used.push((number, format!("{description}: {why}"))),
                Ok((None, None, None)) => out.not_used.push((number, format!("{description}: it has no figure at all (a heading or a note)"))),
                Ok((quantity, unit_cost, amount)) if is_total_row(description, quantity, unit_cost) => {
                    if let Some(a) = amount {
                        said_totals.push((number, description.to_string(), a));
                    }
                    out.not_used.push((number, format!("{description}: it is a total of rows above it, not a line{}", amount.map(|a| format!(" (it says ${})", crate::view::figure(a, 2))).unwrap_or_default())));
                }
                Ok((quantity, unit_cost, amount)) => {
                    // An amount that is not its quantity times its unit cost is kept as
                    // written, and said: one of the three was written wrong.
                    if let (Some(q), Some(c), Some(a)) = (quantity, unit_cost, amount) {
                        let worked = crate::lines::cents(q * c);
                        if (worked - a).abs() > 0.011 {
                            out.notes.push(format!(
                                "row {number}: {description}: its amount, ${}, is not its quantity times its unit cost ({} × ${} = ${}); the amount was kept as written",
                                crate::view::figure(a, 2),
                                crate::view::figure(q, 2),
                                crate::view::figure(c, 2),
                                crate::view::figure(worked, 2)
                            ));
                        }
                    }
                    lines.push(PriorLine { description: description.to_string(), quantity, unit: cols.cell(row, "unit").to_string(), unit_cost, amount, class: cols.cell(row, "class").to_string(), code: cols.cell(row, "code").to_string(), element: cols.cell(row, "element").to_string(), division: cols.cell(row, "division").to_string(), sure: cols.cell(row, "sure").to_string(), note: cols.cell(row, "note").to_string() });
                }
            }
        }
        if lines.is_empty() {
            return Err("no row of the estimate could be read as a line".into());
        }
        // The estimate's last total against what its lines add to: an estimate that
        // does not add up to its own total is said to (J17B-23). Only when every line
        // read has an amount, and only the last total row, which is the whole estimate's.
        if let (Some((number, what, said)), true) = (said_totals.last(), lines.iter().all(|l: &PriorLine| l.amount.is_some())) {
            let add: f64 = crate::lines::cents(lines.iter().filter_map(|l| l.amount).sum());
            if (add - said).abs() > 0.011 {
                out.notes.push(format!("row {number}: {what} says ${}, and the {} lines read add to ${}: ${} apart. Neither was changed; the lines are what is set against this job", crate::view::figure(*said, 2), lines.len(), crate::view::figure(add, 2), crate::view::figure((add - said).abs(), 2)));
            }
        }
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let count = lines.len();
        let prior = Prior { lines: Val { value: Some(lines), unit: String::new(), status: how.status, source: how.source.to_string() } };
        let path = job.add(&root, Kind::Prior, name.trim())?;
        check_prior(&path, Kind::Prior, &prior)?;
        job.put(&path, prior)?;
        out.done.push(format!("{}: a prior estimate of {count} line{}, kept to be set against this job; it is no part of this job's price", name.trim(), if count == 1 { "" } else { "s" }));
        Ok((path, out))
    })
}

/// What a line of a prior estimate comes to: its amount as written, or, where none is
/// written, its quantity times its unit cost (and that it was worked here). The one
/// place this is worked: what a prior adds to is this, wherever it is said.
pub(super) fn comes_to(l: &PriorLine) -> (Option<f64>, bool) {
    match (l.amount, l.quantity, l.unit_cost) {
        (Some(a), _, _) => (Some(a), false),
        (None, Some(q), Some(c)) => (Some(crate::lines::cents(q * c)), true),
        _ => (None, false),
    }
}

/// The rows a flat reading of the job shows for a prior estimate.
pub fn rows(w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity, path: &str) -> Vec<crate::view::Row> {
    let Some(p) = w.get::<Prior>(e) else { return Vec::new() };
    vec![crate::view::entered(path, "lines of a prior estimate", &p.lines, |l| {
        // What each line comes to is worked in one place (`comes_to`): a line with a
        // quantity and a unit cost and no amount written comes to their product here
        // as it does where the prior is set against the job (M21-7: the two added to
        // different totals). A line that comes to nothing is said, not dropped.
        let total = crate::lines::cents(l.iter().filter_map(|x| comes_to(x).0).sum());
        let none = l.iter().filter(|x| comes_to(x).0.is_none()).count();
        let left = if none == 0 { String::new() } else { format!(" ({} with no figure, not in it)", crate::view::so_many(none, "line")) };
        format!("{}, adding to ${}{left}", l.len(), crate::view::figure(total, 2))
    })]
}

/// The job's side of one line of a prior estimate: every line of the job by that name.
#[derive(Clone, Debug, PartialEq)]
pub struct JobSide {
    /// How many of the job's lines have the name, and on what.
    pub on: Vec<String>,
    /// What they add to, where they are all in one unit.
    pub quantity: Option<f64>,
    pub unit: String,
    /// What they come to; none when any of them has no figure, and `why` says which.
    pub amount: Option<f64>,
    pub why: String,
    pub status: Status,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComparedLine {
    pub prior: PriorLine,
    /// What the prior line comes to: its amount, or its quantity times its unit cost
    /// where it gives no amount (`worked` is then true).
    pub prior_amount: Option<f64>,
    pub worked: bool,
    /// How many lines of the prior estimate go by this name and are added together
    /// here: one, unless the prior estimate names a line more than once.
    pub of: usize,
    pub job: Option<JobSide>,
}

/// A prior estimate set against the job.
#[derive(Clone, Debug, PartialEq)]
pub struct Compared {
    pub path: String,
    pub name: String,
    pub status: Status,
    pub source: String,
    pub lines: Vec<ComparedLine>,
    /// Lines of the job no line of the prior estimate is called: description, where, and
    /// what they come to.
    pub only_in_job: Vec<(String, Vec<String>, Option<f64>, Status)>,
    pub prior_total: f64,
    /// How many prior lines have no amount and are not in that total.
    pub prior_left_out: usize,
}

/// Each prior estimate in the job set against the job's own lines, by what each line is
/// called (capitals and spacing aside). Lines that are switched off are not the job's.
pub fn compare(job: &mut Job) -> Vec<Compared> {
    let est = crate::view::estimate(job);
    let live: Vec<&crate::view::SheetLine> = est.lines.iter().filter(|l| !l.off).collect();
    let mut out = Vec::new();
    for e in job.order.clone() {
        let (Some(node), Some(prior)) = (job.world.get::<Node>(e), job.world.get::<Prior>(e)) else { continue };
        let lines = prior.lines.value.clone().unwrap_or_default();
        let side = |description: &str| -> Option<JobSide> {
            let its: Vec<&&crate::view::SheetLine> = live.iter().filter(|l| check::same_name(&l.description, description)).collect();
            if its.is_empty() {
                return None;
            }
            let unit = its[0].priced.unit.clone();
            let one_unit = its.iter().all(|l| l.priced.unit == unit);
            let quantity = if one_unit { its.iter().map(|l| l.priced.value).sum::<Option<f64>>() } else { None };
            let missing: Vec<String> = its.iter().filter(|l| l.amount.value.is_none()).map(|l| format!("{} has no figure ({})", l.element, l.amount.note)).collect();
            let amount = if missing.is_empty() { Some(crate::lines::cents(its.iter().filter_map(|l| l.amount.value).sum())) } else { None };
            Some(JobSide {
                on: its.iter().map(|l| l.element.clone()).collect(),
                quantity,
                unit: if one_unit { unit } else { String::new() },
                amount,
                why: missing.join("; "),
                status: its.iter().map(|l| l.amount.status.max(l.priced.status)).max().unwrap_or(Status::Unknown),
            })
        };
        // Lines of the prior estimate that go by one name are one line here, added
        // together: the job's lines of that name are set against all of them at once,
        // not against each in turn.
        let mut compared: Vec<ComparedLine> = Vec::new();
        for l in &lines {
            let (amount, worked) = comes_to(l);
            match compared.iter_mut().find(|c| check::same_name(&c.prior.description, &l.description)) {
                None => compared.push(ComparedLine { prior: l.clone(), prior_amount: amount, worked, of: 1, job: side(&l.description) }),
                Some(c) => {
                    c.prior.quantity = match (c.prior.quantity, l.quantity) {
                        (Some(a), Some(b)) if c.prior.unit.eq_ignore_ascii_case(&l.unit) => Some(a + b),
                        _ => None,
                    };
                    if c.prior.unit_cost != l.unit_cost {
                        c.prior.unit_cost = None;
                    }
                    c.prior.amount = c.prior.amount.zip(l.amount).map(|(a, b)| crate::lines::cents(a + b));
                    c.prior_amount = c.prior_amount.zip(amount).map(|(a, b)| crate::lines::cents(a + b));
                    c.worked |= worked;
                    c.of += 1;
                }
            }
        }
        let mut only: Vec<(String, Vec<String>, Option<f64>, Status)> = Vec::new();
        for l in &live {
            if lines.iter().any(|p| check::same_name(&p.description, &l.description)) || only.iter().any(|(d, ..)| check::same_name(d, &l.description)) {
                continue;
            }
            if let Some(s) = side(&l.description) {
                only.push((l.description.clone(), s.on, s.amount, s.status));
            }
        }
        out.push(Compared {
            path: node.path.clone(),
            name: node.name.clone(),
            status: prior.lines.status,
            source: prior.lines.source.clone(),
            prior_total: crate::lines::cents(lines.iter().filter_map(|l| comes_to(l).0).sum()),
            prior_left_out: lines.iter().filter(|l| comes_to(l).0.is_none()).count(),
            lines: compared,
            only_in_job: only,
        });
    }
    out
}
