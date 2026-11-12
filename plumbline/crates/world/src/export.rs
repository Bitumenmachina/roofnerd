//! The estimate written out for other people: as a workbook whose figures are formulas,
//! as rows of values, and as three lenses on the one estimate.
//!
//! Everything here is laid out from `view::estimate` and `vif::vif`. A lens is the
//! estimate with what its reader is not shown left out, and it says at its head what
//! that is (QUESTIONS.md Q17):
//!
//! * **supplier**: the materials to quote, by what is bought and the unit it is bought
//!   by, with quantities and none of our prices;
//! * **subcontractor**: the scope and quantities of the lines classed Subcontract, with
//!   none of our prices;
//! * **PM**: every line with its hours, what is ordered, its class and its code; the
//!   budget by class; what is to verify in the field. No overhead, profit or price.
//!
//! In the workbook an amount is a formula (the quantity times the price, to the cent),
//! each class of the recap is a formula over the amounts, and overhead, profit and the
//! price are formulas on those: change a price in the workbook and the price follows.
//! Each formula is written only where it comes to the figure the job has; where it
//! would not, the job's figure is written as a figure and the row says so.

use crate::lines::cents;
use crate::value::Status;
use crate::view::{self, SheetLine};
use crate::{Class, Job, Kind, WorkCode};

/// One cell of what is written out.
#[derive(Clone, Debug, PartialEq)]
pub enum Out {
    None,
    Text(String),
    /// A quantity, a rate or a count: to every place the job holds.
    Number(f64),
    Dollars(f64),
    /// A formula as a workbook writes it, and what it comes to in the job (dollars).
    Formula { text: String, value: Option<f64> },
    /// A formula that, in the job as it stands, comes to a word or to nothing: "none"
    /// for a cost type with no priced line yet, nothing for an amount with no price
    /// yet. It turns to a figure when what it rests on is filled in on the sheet.
    FormulaText { text: String, shown: String },
}

impl Out {
    /// The cell as a file of values writes it: a formula is what it comes to.
    pub fn said(&self) -> String {
        match self {
            Out::None => String::new(),
            Out::Text(s) => s.clone(),
            Out::Number(x) => format!("{}", x + 0.0),
            Out::Dollars(x) | Out::Formula { value: Some(x), .. } => format!("{:.2}", x + 0.0),
            Out::Formula { value: None, .. } => String::new(),
            Out::FormulaText { shown, .. } => shown.clone(),
        }
    }
}

fn text(s: impl Into<String>) -> Out {
    Out::Text(s.into())
}

/// One sheet of what is written out. `head` is the row of headings, counted from 0.
#[derive(Clone, Debug, PartialEq)]
pub struct OutSheet {
    pub name: String,
    pub rows: Vec<Vec<Out>>,
    pub head: usize,
    /// How wide each column is best shown, in characters.
    pub widths: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lens {
    Supplier,
    Subcontractor,
    Pm,
}

impl Lens {
    pub fn parse(word: &str) -> Option<Lens> {
        match word {
            "supplier" => Some(Lens::Supplier),
            "subcontractor" | "sub" => Some(Lens::Subcontractor),
            "pm" => Some(Lens::Pm),
            _ => None,
        }
    }
}

fn sure(s: Status) -> &'static str {
    match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    }
}

fn title(job: &Job) -> String {
    job.world.get::<crate::JobInfo>(job.root()).map(|j| j.title.clone()).unwrap_or_default()
}

/// A column's letter: A, B, ... Z. No sheet here has more.
fn col(i: usize) -> char {
    (b'A' + i as u8) as char
}

/// What a line is on, as a reader outside the job is told: the job itself has no name
/// but its place in the file, and is "the job".
fn on(l: &SheetLine) -> String {
    if l.kind == Kind::Job && l.element == l.path {
        "the job".to_string()
    } else {
        l.element.clone()
    }
}

/// What a line's row says of it beside its figures.
fn note_of(l: &SheetLine) -> String {
    let mut said = Vec::new();
    if l.off {
        said.push("switched off: in no total".to_string());
    }
    if let Some(m) = &l.material {
        said.push(format!("made of {m:?}"));
    }
    if l.amount.value.is_none() && !l.amount.note.is_empty() {
        said.push(l.amount.note.clone());
    }
    if l.class.is_none() {
        said.push("no cost type: in no row of the recap".to_string());
    }
    said.join("; ")
}

/// The estimate: its sheet, its recap and its suppliers side by side. With `formulas`
/// an amount, a total, overhead, profit and the price are formulas; without, figures.
pub fn estimate_out(job: &mut Job, formulas: bool) -> Vec<OutSheet> {
    let est = view::estimate(job);
    let title = title(job);
    const HEAD: usize = 3;
    let first = HEAD + 2; // the first line's row, as a workbook counts rows (from 1)
    let last = (first + est.lines.len()).saturating_sub(1).max(first);

    let mut rows: Vec<Vec<Out>> = vec![
        vec![text(format!("{title}: the estimate"))],
        vec![text(if formulas { "Each amount is a formula: change a quantity or a price here and the amounts, the recap and the price follow." } else { "Each line of the estimate, as figures." })],
        vec![],
        ["Element", "Line", "Cost type", "Quantity", "Unit", "Unit cost", "Amount", "How sure", "Note", "Division"].iter().map(|h| text(*h)).collect(),
    ];
    // A division as a row of the sheet names it: "7 Thermal and Moisture Protection".
    let named = |d: crate::components::Division| format!("{} {}", d.number(), d.title());
    for (i, l) in est.lines.iter().enumerate() {
        let n = first + i;
        let price = l.unit_cost.as_ref().and_then(|v| v.value);
        // The formula only where it is the job's own arithmetic to the cent.
        let amount = match (l.off, l.amount.value, l.priced.value, price) {
            (true, _, _, _) => Out::None,
            // No amount yet, for want of a quantity or a price: the formula is there all
            // the same and comes to nothing until both are filled in (J17A-6). A line
            // with no cost type is in no total, so it gets none.
            (_, None, _, _) if formulas && l.class.is_some() => Out::FormulaText { text: format!("=IF(OR(D{n}=\"\",F{n}=\"\"),\"\",ROUND(D{n}*F{n},2))"), shown: String::new() },
            (_, None, _, _) => Out::None,
            (_, Some(a), Some(q), Some(p)) if formulas && (cents(q * p) - a).abs() < 0.005 => Out::Formula { text: format!("=ROUND(D{n}*F{n},2)"), value: Some(a) },
            (_, Some(a), _, _) => Out::Dollars(a),
        };
        let mut note = note_of(l);
        if formulas && matches!(amount, Out::Dollars(_)) {
            note = [note, "this amount is the job's figure, not a formula: it is not this quantity times this price".to_string()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("; ");
        }
        rows.push(vec![
            text(on(l)),
            text(l.description.clone()),
            text(l.class.map(Class::title).unwrap_or("")),
            l.priced.value.map(Out::Number).unwrap_or(Out::None),
            text(l.priced.unit.clone()),
            price.map(Out::Number).unwrap_or(Out::None),
            amount,
            text(if l.off { "" } else { sure(l.amount.status.max(l.priced.status)) }),
            text(note),
            // Its division. A line that is in no total (switched off, or with no cost
            // type) says so beside it, so no sum by division takes it up.
            text(match (l.division, l.off || l.class.is_none()) {
                (None, _) => String::new(),
                (Some(d), false) => named(d),
                (Some(d), true) => format!("{} (in no total)", named(d)),
            }),
        ]);
    }
    let sheet = OutSheet { name: "Estimate".into(), rows, head: HEAD, widths: vec![22.0, 44.0, 18.0, 12.0, 7.0, 11.0, 13.0, 10.0, 60.0, 34.0] };
    if !formulas {
        return vec![sheet];
    }

    // The recap: each class over the sheet's amounts, then the job cost and what is taken on it.
    let mut recap: Vec<Vec<Out>> = vec![vec![text(format!("{title}: the recap"))], vec![], ["", "Amount", "Lines", "Hours", "Note"].iter().map(|h| text(*h)).collect()];
    let class_first = recap.len() + 1;
    for c in &est.classes {
        // Each cost type is a formula over the sheet's lines of that type, whether or not
        // it has any yet (J17A-6): "none" while no line of it has an amount, since
        // nothing is not $0.00, and a figure as soon as one does.
        let rule = format!(
            "=IF(COUNTIFS(Estimate!$C${first}:$C${last},\"{0}\",Estimate!$G${first}:$G${last},\">=0\")=0,\"none\",SUMIF(Estimate!$C${first}:$C${last},\"{0}\",Estimate!$G${first}:$G${last}))",
            c.class.title()
        );
        let amount = match c.total.value {
            None => Out::FormulaText { text: rule, shown: "none".into() },
            Some(v) => Out::Formula { text: rule, value: Some(v) },
        };
        let note = match (c.total.value, c.total.status, c.total.left_out.len()) {
            (_, _, n) if n > 0 => format!("leaves out {n} with no figure"),
            (None, _, _) | (_, Status::Known, _) => String::new(),
            (_, s, _) => format!("{}: {} of it rests on something assumed", sure(s), format_args!("${:.2}", c.total.assumed)),
        };
        recap.push(vec![text(c.class.title()), amount, Out::Number(c.total.members as f64), c.hours.filter(|_| c.total.members > 0).map(|h| Out::Number(h.0 + 0.0)).unwrap_or(Out::None), text(note)]);
    }
    let class_last = recap.len();
    let cost_row = recap.len() + 1;
    recap.push(vec![text("Job cost"), Out::Formula { text: format!("=SUM(B{class_first}:B{class_last})"), value: est.job_cost.value }, Out::Number(est.job_cost.members as f64), Out::None, text(if est.job_cost.status == Status::Known { "" } else { sure(est.job_cost.status) })]);
    let pct = |v: Option<&crate::Val<f64>>| v.and_then(|v| v.value);
    let (overhead_pct, profit_pct) = (pct(est.markup.as_ref().map(|m| &m.overhead)), pct(est.markup.as_ref().map(|m| &m.profit)));
    match (overhead_pct, profit_pct, est.price.value) {
        (Some(o), Some(p), Some(_)) => {
            let (o_pct, o_row, p_pct, p_row) = (cost_row + 1, cost_row + 2, cost_row + 3, cost_row + 4);
            recap.push(vec![text("Overhead, percent of the job cost"), Out::Number(o)]);
            recap.push(vec![text("Overhead"), Out::Formula { text: format!("=ROUND(B{cost_row}*B{o_pct}/100,2)"), value: est.overhead.value }]);
            recap.push(vec![text("Profit, percent of cost and overhead"), Out::Number(p)]);
            recap.push(vec![text("Profit"), Out::Formula { text: format!("=ROUND((B{cost_row}+B{o_row})*B{p_pct}/100,2)"), value: est.profit.value }]);
            recap.push(vec![text("Price"), Out::Formula { text: format!("=B{cost_row}+B{o_row}+B{p_row}"), value: est.price.value }, Out::None, Out::None, text(if est.price.status == Status::Known { "" } else { sure(est.price.status) })]);
        }
        _ => {
            // No figure is made up for what was not entered or could not be worked out.
            for (what, d) in [("Overhead", &est.overhead), ("Profit", &est.profit), ("Price", &est.price)] {
                recap.push(vec![text(what), d.value.map(Out::Dollars).unwrap_or_else(|| text("no figure")), Out::None, Out::None, text(d.note.clone())]);
            }
        }
    }
    // What the price leaves out is said under it, by name.
    // (The job cost's own list holds every line it leaves out: those with no figure and
    // those with no class.)
    let left: Vec<(String, String)> = est.job_cost.left_out.clone();
    if !left.is_empty() {
        recap.push(vec![]);
        recap.push(vec![text(format!("Left out of the job cost, and of everything taken on it: {}", if left.len() == 1 { "1 line".to_string() } else { format!("{} lines", left.len()) }))]);
        for (who, why) in &left {
            recap.push(vec![text(who.clone()), Out::None, Out::None, Out::None, text(why.clone())]);
        }
    }
    // The job cost again, division by division: each a formula over the sheet's amounts.
    let mut by_division: Vec<Vec<Out>> = vec![vec![text(format!("{title}: the job cost by division"))], vec![], ["Division", "Amount", "Lines", "Note"].iter().map(|h| text(*h)).collect()];
    let division_first = by_division.len() + 1;
    for d in &est.divisions {
        let rule = format!(
            "=IF(COUNTIFS(Estimate!$J${first}:$J${last},\"{0}\",Estimate!$G${first}:$G${last},\">=0\")=0,\"none\",SUMIF(Estimate!$J${first}:$J${last},\"{0}\",Estimate!$G${first}:$G${last}))",
            named(d.division)
        );
        let amount = match d.total.value {
            None => Out::FormulaText { text: rule, shown: "none".into() },
            Some(v) => Out::Formula { text: rule, value: Some(v) },
        };
        let note = match (d.total.left_out.len(), d.total.status) {
            (n, _) if n > 0 => format!("leaves out {n} with no figure: {}", d.total.left_out.iter().map(|(who, _)| who.as_str()).collect::<Vec<_>>().join("; ")),
            (_, Status::Known) => String::new(),
            (_, s) => sure(s).to_string(),
        };
        by_division.push(vec![text(named(d.division)), amount, Out::Number(d.total.members as f64), text(if d.total.members == 0 && d.total.left_out.is_empty() { String::new() } else { note })]);
    }
    let division_last = by_division.len();
    let in_all: f64 = est.divisions.iter().filter_map(|d| d.total.value).sum();
    let sum_row = by_division.len() + 1;
    let worst = est.divisions.iter().filter(|d| d.total.value.is_some()).map(|d| d.total.status).max().unwrap_or(Status::Known);
    let out_of_them: usize = est.divisions.iter().map(|d| d.total.left_out.len()).sum();
    let sum_note = [if worst == Status::Known { String::new() } else { sure(worst).to_string() }, if out_of_them == 0 { String::new() } else { format!("leaves out {out_of_them} with no figure, named above") }].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("; ");
    by_division.push(vec![text("The sixteen divisions"), Out::Formula { text: format!("=SUM(B{division_first}:B{division_last})"), value: Some(cents(in_all)) }, Out::None, text(sum_note)]);
    // What is in the job cost and in none of the sixteen: the job cost less the sixteen,
    // so the two still come to the job cost after any change on the sheet (J17A-4).
    {
        let none = &est.no_division;
        let rule = format!("=IF(ROUND(Recap!B{cost_row}-B{sum_row},2)=0,\"none\",ROUND(Recap!B{cost_row}-B{sum_row},2))");
        let amount = match none.value {
            Some(v) if v.abs() >= 0.005 => Out::Formula { text: rule, value: Some(v) },
            _ => Out::FormulaText { text: rule, shown: "none".into() },
        };
        let mut note = if none.members > 0 || !none.left_out.is_empty() { "in the job cost, and in none of the sixteen: each such line has no division on the Estimate sheet".to_string() } else { String::new() };
        if none.value.is_some() && none.status != Status::Known {
            note = format!("{}; {note}", sure(none.status));
        }
        if !none.left_out.is_empty() {
            note = format!("{note}; leaves out {} with no figure: {}", none.left_out.len(), none.left_out.iter().map(|(who, _)| who.as_str()).collect::<Vec<_>>().join("; "));
        }
        by_division.push(vec![text("In no division"), amount, Out::Number(none.members as f64), text(note)]);
    }
    let mut out = vec![
        sheet,
        OutSheet { name: "Recap".into(), rows: recap, head: 2, widths: vec![46.0, 14.0, 8.0, 10.0, 60.0] },
        OutSheet { name: "Divisions".into(), rows: by_division, head: 2, widths: vec![40.0, 14.0, 8.0, 60.0] },
    ];

    // The suppliers side by side: each line made of a material at each one's price.
    if !est.suppliers.is_empty() {
        let mut rows: Vec<Vec<Out>> = vec![vec![text(format!("{title}: suppliers, side by side (the lines made of a material)"))], vec![]];
        let mut head = vec![text("Line"), text("Generic")];
        head.extend(est.suppliers.iter().map(|s| text(s.clone())));
        rows.push(head);
        let from = rows.len() + 1;
        for l in est.lines.iter().filter(|l| !l.off && l.material.is_some()) {
            let mut row = vec![text(format!("{}: {}", on(l), l.description)), l.amount.value.map(Out::Dollars).unwrap_or_else(|| text("no figure"))];
            for s in &est.suppliers {
                row.push(l.quotes.get(s).and_then(|d| d.value).map(Out::Dollars).unwrap_or_else(|| text("no quote")));
            }
            rows.push(row);
        }
        let to = rows.len();
        if to >= from {
            let mut total = vec![text("Total of what each has quoted"), Out::Formula { text: format!("=SUM(B{from}:B{to})"), value: est.at_generic.value }];
            for (i, (_, t)) in est.at_supplier.iter().enumerate() {
                let c = col(i + 2);
                total.push(Out::Formula { text: format!("=SUM({c}{from}:{c}{to})"), value: t.value });
            }
            rows.push(total);
        }
        let mut widths = vec![52.0, 14.0];
        widths.extend(est.suppliers.iter().map(|_| 16.0));
        out.push(OutSheet { name: "Suppliers".into(), rows, head: 2, widths });
    }
    out
}

/// One lens on the estimate.
pub fn lens_out(job: &mut Job, lens: Lens) -> Vec<OutSheet> {
    let est = view::estimate(job);
    let title = title(job);
    let live: Vec<&SheetLine> = est.lines.iter().filter(|l| !l.off).collect();
    let unclassed: Vec<&&SheetLine> = live.iter().filter(|l| l.class.is_none()).collect();
    let left_out = |rows: &mut Vec<Vec<Out>>| {
        if !unclassed.is_empty() {
            rows.push(vec![]);
            rows.push(vec![text(format!("Not in this list: {} with no cost type, so the estimate does not say whose it is", if unclassed.len() == 1 { "1 line".to_string() } else { format!("{} lines", unclassed.len()) }))]);
            for l in &unclassed {
                rows.push(vec![text(format!("{}: {}", on(l), l.description))]);
            }
        }
    };
    match lens {
        Lens::Supplier => {
            let mut rows: Vec<Vec<Out>> = vec![
                vec![text(format!("{title}: materials to quote"))],
                vec![text("Quantities only, by what is bought and the unit it is bought by. None of our prices are in this list.")],
                vec![],
                ["What to quote", "Quantity", "Unit", "Lines", "How sure", "Note"].iter().map(|h| text(*h)).collect(),
            ];
            // By what is bought and its unit, in the order the estimate first names each.
            let mut groups: Vec<(String, String, Vec<&SheetLine>)> = Vec::new();
            for l in live.iter().filter(|l| l.class == Some(Class::Material)) {
                let what = l.material.clone().filter(|m| !m.is_empty()).unwrap_or_else(|| l.description.clone());
                match groups.iter_mut().find(|g| g.0 == what && g.1 == l.priced.unit) {
                    Some(g) => g.2.push(l),
                    None => groups.push((what, l.priced.unit.clone(), vec![l])),
                }
            }
            for (what, unit, lines) in &groups {
                let with: Vec<f64> = lines.iter().filter_map(|l| l.priced.value).collect();
                let without = lines.len() - with.len();
                // A quantity that is not all there is not a smaller quantity: it is none.
                // To the four places a quantity is shown to, as the recap's hours are.
                let quantity = if without == 0 { Out::Number((with.iter().sum::<f64>() * 10000.0).round() / 10000.0) } else { text("no quantity") };
                let note = if without == 0 { String::new() } else { format!("{without} of its lines {} no quantity: {}", if without == 1 { "has" } else { "have" }, lines.iter().filter(|l| l.priced.value.is_none()).map(|l| format!("{} ({})", on(l), l.priced.note)).collect::<Vec<_>>().join("; ")) };
                rows.push(vec![text(what.clone()), quantity, text(unit.clone()), Out::Number(lines.len() as f64), text(sure(lines.iter().map(|l| l.priced.status).max().unwrap_or(Status::Known))), text(note)]);
            }
            left_out(&mut rows);
            vec![OutSheet { name: "To quote".into(), rows, head: 3, widths: vec![40.0, 14.0, 8.0, 8.0, 10.0, 60.0] }]
        }
        Lens::Subcontractor => {
            let mut rows: Vec<Vec<Out>> = vec![
                vec![text(format!("{title}: scope for subcontractors"))],
                vec![text("Scope and quantities only: the lines of the estimate whose cost type is Subcontract. None of our prices are in this list.")],
                vec![],
                ["Where", "Scope", "Quantity", "Unit", "How sure", "Note"].iter().map(|h| text(*h)).collect(),
            ];
            for l in live.iter().filter(|l| l.class == Some(Class::Subcontract)) {
                rows.push(vec![text(on(l)), text(l.description.clone()), l.priced.value.map(Out::Number).unwrap_or_else(|| text("no quantity")), text(l.priced.unit.clone()), text(sure(l.priced.status)), text(if l.priced.value.is_none() { l.priced.note.clone() } else { String::new() })]);
            }
            left_out(&mut rows);
            vec![OutSheet { name: "Scope".into(), rows, head: 3, widths: vec![22.0, 48.0, 14.0, 8.0, 10.0, 60.0] }]
        }
        Lens::Pm => {
            let mut code = |path: &str| job.find(path).and_then(|e| job.world.get::<WorkCode>(e)).and_then(|c| c.wbs.value.clone()).unwrap_or_default();
            let mut lines: Vec<Vec<Out>> = vec![
                vec![text(format!("{title}: for the project manager"))],
                vec![text("Every line of the estimate with its hours, what is ordered, its cost type, its division and its code. No overhead, no profit and no price are in this list.")],
                vec![],
                ["Element", "Line", "Cost type", "Code", "Quantity", "Unit", "Hours", "Amount", "How sure", "Note", "Division"].iter().map(|h| text(*h)).collect(),
            ];
            for l in &est.lines {
                let hours = l.class.filter(|c| c.in_hours() && !l.off).and(l.priced.value);
                lines.push(vec![
                    text(on(l)),
                    text(l.description.clone()),
                    text(l.class.map(Class::title).unwrap_or("")),
                    text(code(&l.path)),
                    l.priced.value.map(Out::Number).unwrap_or(Out::None),
                    text(l.priced.unit.clone()),
                    hours.map(Out::Number).unwrap_or(Out::None),
                    if l.off { Out::None } else { l.amount.value.map(Out::Dollars).unwrap_or(Out::None) },
                    text(if l.off { "" } else { sure(l.amount.status.max(l.priced.status)) }),
                    text(note_of(l)),
                    text(l.division.map(|d| format!("{} {}", d.number(), d.title())).unwrap_or_default()),
                ]);
            }
            let mut budget: Vec<Vec<Out>> = vec![vec![text(format!("{title}: the budget by cost type"))], vec![], ["Cost type", "Amount", "Hours", "Lines", "How sure"].iter().map(|h| text(*h)).collect()];
            for c in &est.classes {
                budget.push(vec![text(c.class.title()), c.total.value.map(Out::Dollars).unwrap_or_else(|| text("none")), c.hours.filter(|_| c.total.members > 0).map(|h| Out::Number(h.0)).unwrap_or(Out::None), Out::Number(c.total.members as f64), text(if c.total.members == 0 { "" } else { sure(c.total.status) })]);
            }
            budget.push(vec![text("Job cost"), est.job_cost.value.map(Out::Dollars).unwrap_or_else(|| text("none")), Out::None, Out::Number(est.job_cost.members as f64), text(sure(est.job_cost.status))]);
            let left = est.job_cost.left_out.len();
            if left > 0 {
                budget.push(vec![text(format!("The job cost leaves out {}: those with no figure or no cost type, each said on its line", if left == 1 { "1 line".to_string() } else { format!("{left} lines") }))]);
            }
            // The same budget, division by division.
            budget.push(vec![]);
            budget.push(["Division", "Amount", "", "Lines", "How sure"].iter().map(|h| text(*h)).collect());
            // A division says what it leaves out, by name, as the job cost does (J17A-10):
            // one whose only work has no figure does not read as a division with no work.
            let left = |t: &view::MoneyTotal| if t.left_out.is_empty() { String::new() } else { format!("leaves out {} with no figure: {}", t.left_out.len(), t.left_out.iter().map(|(who, why)| format!("{who} ({why})")).collect::<Vec<_>>().join("; ")) };
            let how = |t: &view::MoneyTotal| [if t.members == 0 { String::new() } else { sure(t.status).to_string() }, left(t)].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("; ");
            for d in &est.divisions {
                let amount = match (d.total.value, d.total.left_out.is_empty()) {
                    (Some(v), _) => Out::Dollars(v),
                    (None, true) => text("none"),
                    (None, false) => text("no figure"),
                };
                budget.push(vec![text(format!("{} {}", d.division.number(), d.division.title())), amount, Out::None, Out::Number(d.total.members as f64), text(how(&d.total))]);
            }
            if est.no_division.members > 0 || !est.no_division.left_out.is_empty() {
                budget.push(vec![text("In no division"), est.no_division.value.map(Out::Dollars).unwrap_or_else(|| text("no figure")), Out::None, Out::Number(est.no_division.members as f64), text(how(&est.no_division))]);
            }
            // What is to verify in the field, by the dollars riding on each.
            let list = crate::vif::vif(job);
            let mut verify: Vec<Vec<Out>> = vec![vec![text(format!("{title}: to verify in the field"))], vec![], ["Riding on it", "What", "Level", "To do", "Lines"].iter().map(|h| text(*h)).collect()];
            for q in &list.questions {
                let (what, to_do) = match &q.clash {
                    Some(c) => (format!("{} and {}", q.name, c.other_name), format!("in the same space{}: measure either one", c.by.map(|by| format!(" by {:.2} in", by * 12.0 + 0.0)).unwrap_or_default())),
                    None => {
                        let named = |length: bool| q.values.iter().filter(|v| v.2 == length).map(|v| v.0.clone()).collect::<Vec<_>>().join(", ");
                        let (measure, check) = (named(true), named(false));
                        let mut to_do = Vec::new();
                        if !measure.is_empty() {
                            to_do.push(format!("measure: {measure}"));
                        }
                        if !check.is_empty() {
                            to_do.push(format!("check: {check}"));
                        }
                        (format!("{} ({})", q.name, if q.kind == Kind::Storey { "level".to_string() } else { format!("{:?}", q.kind).to_lowercase() }), to_do.join("; "))
                    }
                };
                verify.push(vec![q.dollars.map(Out::Dollars).unwrap_or_else(|| text("no dollars")), text(what), text(q.storey.clone()), text(to_do), Out::Number(q.lines as f64)]);
            }
            if list.questions.is_empty() {
                verify.push(vec![text("Nothing: every value that can be measured is Known, and no clash is a question.")]);
            }
            vec![
                OutSheet { name: "Lines".into(), rows: lines, head: 3, widths: vec![22.0, 44.0, 18.0, 12.0, 12.0, 7.0, 10.0, 13.0, 10.0, 60.0] },
                OutSheet { name: "Budget".into(), rows: budget, head: 2, widths: vec![26.0, 14.0, 10.0, 8.0, 10.0] },
                OutSheet { name: "Verify in field".into(), rows: verify, head: 2, widths: vec![14.0, 44.0, 14.0, 60.0, 8.0] },
            ]
        }
    }
}

/// What is written out as rows of values, for a CSV file: each sheet after the last,
/// a blank row between, and from the second on its name on a row of its own.
pub fn values(sheets: &[OutSheet]) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for (i, sheet) in sheets.iter().enumerate() {
        if i > 0 {
            rows.push(Vec::new());
        }
        rows.extend(sheet.rows.iter().map(|r| r.iter().map(Out::said).collect()));
    }
    rows
}
