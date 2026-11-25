//! A price set by name, and the price list as rows (call 207, points 4 and 5).
//!
//! Ours. BIMCraft carries no money (DECISIONS.md call 45, and call 196's survey of all of it).
//!
//! **One home, one road.** A price lives in the price list: on a material, or on an item (its own
//! entry, call 196), as the estimator's price and each supplier's. `set_price` sets or clears one of
//! them by the name of the material or the item, and it does it by the road a price list brought in
//! takes: it makes one row of a list and gives it to `tables::prices_in`, so the same rules hold
//! (the unit the entry is bought by, a price of $0.00 taken and named, a supplier's price standing
//! as the job's where the job has none, an item taking its first price, all or nothing). What it
//! adds is the words after it: the lines the price prices, on how many things, and the job cost
//! before and after, worked by the estimate and not again here.
//!
//! `price_list` is the price list as rows for whoever shows it. The window counts and sums nothing:
//! every figure on a row (the lines, the things, the dollars riding) is read off the estimate's own
//! lines, and each row says how many of its lines have no figure, so a sum that leaves something
//! out says so.
//!
//! A price cleared is no price, never $0.00. The model does not hold that a price was cleared as
//! apart from never set (both are a price with no value, Unknown); it holds the source that came
//! with it. A price of $0.00 is a price, and is marked.

use crate::check;
use crate::components::{Division, Kind, Node, Supply};
use crate::editing::{all_or_nothing, item_material_name, Entered};
use crate::job::Job;
use crate::tables;
use crate::value::{Status, Val};
use crate::view;
use std::collections::{BTreeSet, HashMap};

/// One price as a row of the price list holds it, with what rides on it.
#[derive(Clone, Debug, PartialEq)]
pub struct PriceCell {
    /// The price, in dollars for the unit the row is bought by. None: no price.
    pub value: Option<f64>,
    pub status: Status,
    /// Where it is from.
    pub source: String,
    /// A price of $0.00, which is a price, entered and marked, and not no price.
    pub at_nothing: bool,
    /// What the lines made of this entry come to at this price, to the cent: the sum of those
    /// that have a figure. None when none has.
    pub riding: Option<f64>,
    /// The least sure of the figures summed in `riding`.
    pub riding_status: Status,
    /// How many of the row's lines have a figure at this price (of `PriceRow::lines`).
    pub lines_at_it: usize,
}

/// A row of the price list: one material or one item.
#[derive(Clone, Debug, PartialEq)]
pub struct PriceRow {
    pub name: String,
    /// Its path in the job.
    pub path: String,
    /// An item (its own entry in the price list, call 196), not a material.
    pub item: bool,
    /// What it is, in words: "Material", or "Item, Mechanical".
    pub what: String,
    /// What it is bought by: EA, lb, sheet.
    pub unit: String,
    /// The estimator's price.
    pub price: PriceCell,
    /// Each supplier's, in the order of `PriceList::suppliers`; None where that supplier has not
    /// quoted it.
    pub suppliers: Vec<Option<PriceCell>>,
    /// The lines of the job made of it that are on.
    pub lines: usize,
    /// The lines made of it that are switched off.
    pub lines_off: usize,
    /// Of `lines`, how many have no figure (they wait on this price, or on something else, and
    /// say which on the sheet).
    pub waiting: usize,
    /// The things (elements) those lines are on.
    pub things: usize,
}

/// The price list: every supplier that has quoted anything, and a row for each material and item
/// that a line of the job is made of or that has a price. Rows with no estimator's price come
/// first (the estimate is waiting for them), the most lines waiting first; then the rest by the
/// dollars riding on them, the most first, then by name.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PriceList {
    pub suppliers: Vec<String>,
    pub rows: Vec<PriceRow>,
    /// Lines made of something the job has no material or item by: the name the line gives (empty:
    /// it does not say) and how many lines are made of it. Such a line is in no row.
    pub not_found: Vec<(String, usize)>,
}

/// What `set_price` did, in figures; `said` is it in words.
#[derive(Clone, Debug, PartialEq)]
pub struct PriceSet {
    /// The material's or item's name, as the job has it.
    pub name: String,
    pub path: String,
    pub item: bool,
    /// What it is bought by.
    pub unit: String,
    /// The supplier whose price it was; None: the estimator's.
    pub whose: Option<String>,
    /// A price cleared (and not set).
    pub cleared: bool,
    /// The job already held exactly this: nothing was changed.
    pub same: bool,
    /// A price of $0.00 was set.
    pub at_nothing: bool,
    /// A supplier the job had not yet had, now added by this price.
    pub supplier_added: Option<String>,
    /// The lines made of it that are on, after: what the price prices.
    pub lines: usize,
    /// The things those lines are on.
    pub things: usize,
    /// Of those lines, how many still have no figure.
    pub waiting: usize,
    /// What those lines come to at the supplier's price, when a supplier's price was set.
    pub at_supplier: Option<f64>,
    /// The job cost before and after, to the cent. None: no line came to a figure.
    pub cost_before: Option<f64>,
    pub cost_after: Option<f64>,
    /// How many lines the job cost leaves out, before and after.
    pub left_out_before: usize,
    pub left_out_after: usize,
    /// What the road the price took said besides (a line named for it that took the price, the job taking a
    /// supplier's price for its own), each in full.
    pub done: Vec<String>,
    /// The lines of the same name, not made of it, that this price priced (each as "<name>" on <thing>).
    pub named_lines: Vec<String>,
    /// What else it has to say, in full.
    pub notes: Vec<String>,
}

impl PriceSet {
    /// The change in words, a line each: what is now the price and what it was, what it prices,
    /// and the job cost before and after. The window and the command line say the same.
    pub fn said(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let whose = self.whose.as_ref().map(|s| format!("{s}'s price")).unwrap_or_else(|| "the estimator's price".to_string());
        if self.same {
            out.push(format!("nothing changed: {}", self.done.first().cloned().unwrap_or_else(|| format!("{} already has that as {whose}", self.name))));
        } else {
            out.extend(self.done.iter().cloned());
        }
        if let Some(s) = &self.supplier_added {
            out.push(format!("{s} is new to this job and is added as a supplier"));
        }
        if self.at_nothing {
            out.push(format!("that price is $0.00: it is taken, as a thing can be free; nothing is charged for {}{}, and the estimate marks each line made of it", self.name, self.whose.as_deref().map(|s| format!(" at {s}'s price")).unwrap_or_default()));
        }
        if self.lines == 0 {
            if self.named_lines.is_empty() {
                out.push(format!("{} prices nothing on the job yet: no line is made of it", self.name));
            } else {
                let n = self.named_lines.len();
                out.push(format!("no line is made of {}, and {} of that name {} priced by it: {}", self.name, view::so_many(n, "line"), if n == 1 { "is" } else { "are" }, self.named_lines.join(", ")));
            }
        } else {
            let mut priced = format!("{whose} for {} prices {} on {}", self.name, view::so_many(self.lines, "line"), view::so_many(self.things, "thing"));
            if self.waiting > 0 {
                priced.push_str(&format!("; {} of them still {} no figure", self.waiting, if self.waiting == 1 { "has" } else { "have" }));
            }
            out.push(priced);
            if let (Some(at), Some(s)) = (self.at_supplier, &self.whose) {
                out.push(format!("at {s}'s price those lines come to {}", view::money(at)));
            }
        }
        let cost = |c: Option<f64>, left: usize| match c {
            Some(x) => format!("{}{}", view::money(x), if left > 0 { format!(" ({} left out of it)", view::so_many(left, "line")) } else { String::new() }),
            None => "no figure".to_string(),
        };
        if self.cost_before == self.cost_after && self.left_out_before == self.left_out_after {
            out.push(format!("the job cost is as it was: {}", cost(self.cost_after, self.left_out_after)));
        } else {
            out.push(format!("the job cost was {} and is now {}", cost(self.cost_before, self.left_out_before), cost(self.cost_after, self.left_out_after)));
        }
        out.extend(self.notes.iter().cloned());
        out
    }
}

/// One thing the price list holds a price for: a material, or an item.
struct Entry {
    path: String,
    name: String,
    item: bool,
    division: Option<Division>,
    supply: Option<Supply>,
    /// The name as two names are compared, of what a line made of it says.
    key: String,
}

fn entries(job: &mut Job) -> Vec<Entry> {
    let mut out = Vec::new();
    for e in job.order.clone() {
        let (Some(node), Some(kind)) = (job.world.get::<Node>(e), job.world.get::<Kind>(e).copied()) else { continue };
        let item = job.world.get::<crate::Item>(e);
        if item.is_none() && kind != Kind::Material {
            continue;
        }
        let key = check::plain_name(&if item.is_some() { item_material_name(&node.name) } else { node.name.clone() });
        out.push(Entry { path: node.path.clone(), name: node.name.clone(), item: item.is_some(), division: item.and_then(|i| i.division), supply: job.world.get::<Supply>(e).cloned(), key });
    }
    out
}

fn cell(v: &Val<f64>) -> PriceCell {
    PriceCell { value: v.value, status: v.status, source: v.source.clone(), at_nothing: v.value == Some(0.0), riding: None, riding_status: Status::Known, lines_at_it: 0 }
}

/// The sum of the figures given, to the cent, and how many there were, and the least sure of them.
fn sum(parts: impl Iterator<Item = (f64, Status)>) -> (Option<f64>, usize, Status) {
    let (mut total, mut n, mut status) = (0.0, 0, Status::Known);
    for (x, s) in parts {
        total += x;
        n += 1;
        status = status.max(s);
    }
    (if n == 0 { None } else { Some(crate::lines::cents(total)) }, n, status)
}

/// The price list as rows. See `PriceList`.
pub fn price_list(job: &mut Job) -> PriceList {
    let est = view::estimate(job);
    let all = entries(job);
    // A line finds its row by the name it is made of, as names are compared everywhere: the first
    // entry of a name (a material before an item, as the price list brought in goes by).
    let mut by_key: HashMap<&str, usize> = HashMap::new();
    for (i, e) in all.iter().enumerate() {
        by_key.entry(e.key.as_str()).or_insert(i);
    }
    let mut of: Vec<Vec<&view::SheetLine>> = vec![Vec::new(); all.len()];
    let mut not_found: Vec<(String, usize)> = Vec::new();
    for l in est.lines.iter().filter(|l| l.material.is_some()) {
        let said = l.material.clone().unwrap_or_default();
        match by_key.get(check::plain_name(&said).as_str()) {
            Some(&i) if !said.is_empty() => of[i].push(l),
            _ => match not_found.iter_mut().find(|(n, _)| *n == said) {
                Some((_, n)) => *n += 1,
                None => not_found.push((said, 1)),
            },
        }
    }
    let suppliers = est.suppliers.clone();
    let mut rows: Vec<PriceRow> = Vec::new();
    for (i, e) in all.iter().enumerate() {
        let lines: Vec<&&view::SheetLine> = of[i].iter().filter(|l| !l.off).collect();
        let has_price = e.supply.as_ref().is_some_and(|s| s.price.value.is_some() || s.quotes.values().any(|q| q.value.is_some()));
        if lines.is_empty() && of[i].is_empty() && !has_price {
            continue;
        }
        let unit = e.supply.as_ref().map(|s| s.price.unit.trim_start_matches("$/").to_string()).unwrap_or_else(|| "EA".to_string());
        let mut price = e.supply.as_ref().map(|s| cell(&s.price)).unwrap_or(PriceCell { value: None, status: Status::Unknown, source: "no price entered".into(), at_nothing: false, riding: None, riding_status: Status::Known, lines_at_it: 0 });
        let (riding, n, status) = sum(lines.iter().filter_map(|l| l.amount.value.map(|x| (x, l.amount.status))));
        (price.riding, price.lines_at_it, price.riding_status) = (riding, n, status);
        let at: Vec<Option<PriceCell>> = suppliers
            .iter()
            .map(|s| {
                let quote = e.supply.as_ref().and_then(|sup| sup.quotes.get(s))?;
                let mut c = cell(quote);
                let (riding, n, status) = sum(lines.iter().filter_map(|l| l.quotes.get(s).and_then(|d| d.value.map(|x| (x, d.status)))));
                (c.riding, c.lines_at_it, c.riding_status) = (riding, n, status);
                Some(c)
            })
            .collect();
        let things: BTreeSet<&str> = lines.iter().map(|l| l.path.as_str()).collect();
        rows.push(PriceRow {
            name: e.name.clone(),
            path: e.path.clone(),
            item: e.item,
            what: if e.item { e.division.map(|d| format!("Item, {}", d.title())).unwrap_or_else(|| "Item".into()) } else { "Material".into() },
            unit,
            waiting: lines.iter().filter(|l| l.amount.value.is_none()).count(),
            lines: lines.len(),
            lines_off: of[i].len() - lines.len(),
            things: things.len(),
            price,
            suppliers: at,
        });
    }
    rows.sort_by(|a, b| {
        let key = |r: &PriceRow| (r.price.value.is_some(), std::cmp::Reverse(if r.price.value.is_some() { 0 } else { r.lines }));
        key(a).cmp(&key(b)).then_with(|| b.price.riding.unwrap_or(-1.0).partial_cmp(&a.price.riding.unwrap_or(-1.0)).unwrap_or(std::cmp::Ordering::Equal)).then_with(|| a.name.cmp(&b.name))
    });
    PriceList { suppliers, rows, not_found }
}

/// The material or item a typed name or path is; refused when it is neither (naming what the job has
/// that is near it) or is more than one.
fn find(job: &mut Job, typed: &str) -> Result<Entry, String> {
    let typed = typed.trim();
    let mut all = entries(job);
    let plain = check::plain_name(typed);
    let found: Vec<usize> = (0..all.len()).filter(|&i| all[i].path == typed || check::plain_name(&all[i].name) == plain).collect();
    match found.as_slice() {
        [one] => Ok(all.swap_remove(*one)),
        [] => {
            let near: Vec<&str> = all.iter().filter(|e| { let n = check::plain_name(&e.name); n.contains(&plain) || plain.contains(&n) || plain.split(' ').any(|w| w.len() > 2 && n.split(' ').any(|x| x == w)) }).map(|e| e.name.as_str()).take(5).collect();
            let shown: Vec<&str> = if near.is_empty() { all.iter().map(|e| e.name.as_str()).take(5).collect() } else { near };
            Err(format!(
                "{typed:?} is neither a material nor an item of this job{}",
                if shown.is_empty() { "; the job has none to price".to_string() } else { format!("; the job has {}{}", shown.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join(", "), if near_is_all(&all, &shown) { "" } else { ", and more" }) }
            ))
        }
        many => Err(format!("{typed:?} is {} things of this job ({}); give the path of the one", many.len(), many.iter().map(|&i| format!("{} {}", if all[i].item { "the item" } else { "the material" }, all[i].path)).collect::<Vec<_>>().join(", "))),
    }
}

fn near_is_all(all: &[Entry], shown: &[&str]) -> bool {
    all.len() <= shown.len()
}

/// Set the estimator's price (`whose` None) or a supplier's for a material or an item, by its name or its
/// path; `price` None clears it. The price is typed as a price list writes it (`1850`, `$1,850.00`,
/// `4.25/lb`); `how` says how sure it is and where it came from (a cleared price is Unknown whatever
/// `how` says, and keeps `how`'s source). All or nothing: refused, the job is as it was. A supplier the
/// job has not had is added, and said. One call is one step for Undo.
pub fn set_price(job: &mut Job, name: &str, whose: Option<&str>, price: Option<&str>, how: Entered) -> Result<PriceSet, String> {
    if whose.is_some_and(|w| w.trim().is_empty()) {
        return Err("a supplier has a name: say whose price it is".into());
    }
    all_or_nothing(job, |job| {
        let entry = find(job, name)?;
        let known_suppliers: Vec<String> = {
            let mut q = job.world.query::<&Supply>();
            let mut all: Vec<String> = q.iter(&job.world).flat_map(|s| s.quotes.keys().cloned()).collect();
            all.sort();
            all.dedup();
            all
        };
        let resolved: Option<String> = whose.map(|w| known_suppliers.iter().find(|k| check::same_name(k, w)).cloned().unwrap_or_else(|| w.trim().to_string()));
        let supplier_added = resolved.clone().filter(|r| !known_suppliers.contains(r));
        let unit = entry.supply.as_ref().map(|s| s.price.unit.trim_start_matches("$/").to_string()).unwrap_or_else(|| "EA".to_string());
        let est_before = view::estimate(job);
        let before = (est_before.job_cost.value, est_before.job_cost.left_out.len());
        let held: Option<Val<f64>> = entry.supply.as_ref().and_then(|s| match &resolved {
            None => Some(s.price.clone()),
            Some(r) => s.quotes.get(r).cloned(),
        });
        let mut result = PriceSet {
            name: entry.name.clone(),
            path: entry.path.clone(),
            item: entry.item,
            unit: unit.clone(),
            whose: resolved.clone(),
            cleared: price.is_none(),
            same: false,
            at_nothing: false,
            supplier_added: supplier_added.clone(),
            lines: 0,
            things: 0,
            waiting: 0,
            at_supplier: None,
            cost_before: before.0,
            cost_after: before.0,
            left_out_before: before.1,
            left_out_after: before.1,
            done: Vec::new(),
            named_lines: Vec::new(),
            notes: Vec::new(),
        };
        // A supplier written another way from the job's own is taken as the job's spelling, and said which (call 208).
        if let (Some(typed), Some(job_has)) = (whose, resolved.as_deref()) {
            if typed.trim() != job_has {
                result.notes.push(format!("{:?} is {job_has:?} written another way: a supplier has one name in a job, and the job's is taken", typed.trim()));
            }
        }
        // Clearing what was never there changes nothing, and gives an item no entry it had not.
        let nothing_there = price.is_none() && held.as_ref().map_or(true, |v| v.value.is_none());
        if nothing_there {
            result.same = true;
            result.supplier_added = None;
            result.done = vec![match &resolved {
                None => format!("{} has no price of the estimator's to clear", entry.name),
                Some(s) => format!("{s} has not quoted {}, so there is no price of theirs to clear", entry.name),
            }];
            (result.lines, result.things, result.waiting, result.at_supplier) = follow(job, &entry, resolved.as_deref());
            return Ok(result);
        }
        let (status, source) = if price.is_some() { (how.status, how.source) } else { (Status::Unknown, how.source) };
        let supplier_cell = resolved.clone().unwrap_or_default();
        let list = vec![
            vec!["Material".to_string(), "Bought by".into(), "Price".into(), "Supplier".into()],
            vec![entry.name.clone(), unit.clone(), price.unwrap_or("").to_string(), supplier_cell],
        ];
        let brought = tables::prices_in(job, &list, None, None, false, Some(Entered { status, source }))?;
        if !brought.not_used.is_empty() {
            return Err(brought.not_used.iter().map(|(_, why)| why.clone()).collect::<Vec<_>>().join("; "));
        }
        result.same = brought.done.is_empty() && brought.same > 0;
        result.at_nothing = price.is_some() && brought.notes.iter().any(|n| n.contains("priced at $0.00"));
        result.named_lines = brought.done.iter().filter_map(|d| d.starts_with('"').then(|| d.split_once(": priced at ").map(|(who, _)| who.to_string())).flatten()).collect();
        result.done = brought.done;
        // What the price list's own wording says of a whole list is not said of one price: it is said here, of this one.
        result.notes.extend(brought.notes.into_iter().filter(|n| !n.contains("as the list says") && !n.contains("of this list's items")));
        if result.same {
            result.done = vec![format!("{} already has {} as {}", entry.name, held.as_ref().map(|v| v.value.map(|x| format!("${}/{unit}", view::rate(x))).unwrap_or_else(|| "no price".into())).unwrap_or_default(), match &resolved { None => "the estimator's price".to_string(), Some(s) => format!("{s}'s price") })];
        }
        job.derive();
        let est = view::estimate(job);
        result.cost_after = est.job_cost.value;
        result.left_out_after = est.job_cost.left_out.len();
        (result.lines, result.things, result.waiting, result.at_supplier) = follow(job, &entry, resolved.as_deref());
        Ok(result)
    })
}

/// The lines made of an entry that are on, as they now stand: how many, on how many things, how many
/// without a figure, and (given a supplier) what they come to at that supplier's price.
fn follow(job: &mut Job, entry: &Entry, supplier: Option<&str>) -> (usize, usize, usize, Option<f64>) {
    let est = view::estimate(job);
    let made: Vec<&view::SheetLine> = est.lines.iter().filter(|l| !l.off && l.material.as_deref().is_some_and(|m| check::plain_name(m) == entry.key)).collect();
    let things: BTreeSet<&str> = made.iter().map(|l| l.path.as_str()).collect();
    let at = supplier.and_then(|s| {
        let (total, n, _) = sum(made.iter().filter_map(|l| l.quotes.get(s).and_then(|d| d.value.map(|x| (x, d.status)))));
        total.filter(|_| n > 0)
    });
    (made.len(), things.len(), made.iter().filter(|l| l.amount.value.is_none()).count(), at)
}
