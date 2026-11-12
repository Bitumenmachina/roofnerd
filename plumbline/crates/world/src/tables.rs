//! Tables brought in and written out: a price list, as rows of cells.
//!
//! Ours. BIMCraft has no price, and reads no table (DECISIONS.md call 45). The rows come
//! from `plumbline-sheets` (a CSV file, or a sheet of a workbook) and go back to it; what
//! they mean is decided here, and nothing here reads a disk.
//!
//! **A table this program wrote comes back as it went.** Every entered value has a
//! column for how sure it is and one for where it came from, and a figure is written in
//! full (4.1, not 4.10; 0.30000000000000004 if that is what is held).
//!
//! **Somebody else's table is read by its headings.** A heading is matched to what it
//! means from the short list of words a supplier or an estimator heads such a column
//! with. A column that is not used is named. A row that is not used is listed with why.
//! Nothing in the file is dropped without being said. A table with no column for what a
//! price list has to have (what, by what unit, how much) is refused whole.
//!
//! All of one table is brought in or none of it: the rows that can be used are, in one
//! step, and the job is checked whole after; if that fails the job is as it was.

use std::collections::BTreeMap;

use crate::check;
use crate::components::{Kind, Node, Order, Supply};
use crate::editing::{all_or_nothing, Entered};
use crate::job::Job;
use crate::units;
use crate::value::{Status, Val};

/// The headings of a price list as this program writes it.
pub const PRICE_HEADINGS: [&str; 10] = ["Material", "Bought by", "Price", "Supplier", "How sure", "Where from", "Ordered at", "Whole ones only", "Ordered at: how sure", "Ordered at: where from"];

/// What each column of a price list may be headed, by what it means. The first of each
/// is what this program writes.
const PRICE_COLUMNS: [(&str, &[&str]); 10] = [
    // A list with a part number and a description heads the number "Item": the words
    // that say what the thing is come first, and "item" last.
    ("material", &["material", "material description", "item description", "description", "product", "name", "item"]),
    ("unit", &["bought by", "unit", "uom", "u/m", "um", "per", "unit of measure", "sold by"]),
    ("price", &["price", "unit price", "cost", "unit cost", "each", "price each", "net price", "your price"]),
    ("supplier", &["supplier", "vendor", "quoted by"]),
    ("status", &["how sure", "status"]),
    ("source", &["where from", "source"]),
    ("order", &["ordered at", "order factor", "coverage"]),
    ("whole", &["whole ones only"]),
    ("order status", &["ordered at: how sure"]),
    ("order source", &["ordered at: where from"]),
];

/// How many rows down a table's headings are looked for: a supplier's list often has
/// its name and a date above them.
const HEADINGS_WITHIN: usize = 25;

/// A figure in full: reading it back gives the same number.
fn exact(x: f64) -> String {
    format!("{}", x + 0.0)
}

fn status_said(s: Status) -> &'static str {
    match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    }
}

fn status_read(text: &str) -> Option<Status> {
    match text.trim().to_lowercase().as_str() {
        "known" => Some(Status::Known),
        "assumed" => Some(Status::Assumed),
        "unknown" => Some(Status::Unknown),
        _ => None,
    }
}

/// A heading as it is compared: small letters, one space where there is any, no full
/// stop or colon at its end.
fn heading(text: &str) -> String {
    text.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ").trim_end_matches(['.', ':']).trim().to_string()
}

/// Which column of a table holds what.
struct Columns {
    at: BTreeMap<&'static str, usize>,
    not_used: Vec<String>,
    /// The row the headings are in, counted from 0.
    row: usize,
}

impl Columns {
    fn cell<'a>(&self, row: &'a [String], what: &str) -> &'a str {
        self.at.get(what).and_then(|i| row.get(*i)).map(|s| s.trim()).unwrap_or("")
    }
    fn has(&self, what: &str) -> bool {
        self.at.contains_key(what)
    }
}

/// Find a table's headings: the first row, among the first few, in which every one of
/// `needed` is found. None when no row has them all.
fn columns(rows: &[Vec<String>], known: &[(&'static str, &[&str])], needed: &[&str]) -> Option<Columns> {
    for (r, row) in rows.iter().take(HEADINGS_WITHIN).enumerate() {
        let mut at: BTreeMap<&'static str, usize> = BTreeMap::new();
        let mut not_used = Vec::new();
        for (i, cell) in row.iter().enumerate() {
            let h = heading(cell);
            if h.is_empty() {
                continue;
            }
            // Where two columns could both be the same thing ("Item" and "Description"),
            // the one whose heading stands earlier among the words for it is taken, and
            // the other is said not to be used.
            let rank = |what: &str, heading: &str| known.iter().find(|(w, _)| *w == what).and_then(|(_, words)| words.iter().position(|w| *w == heading)).unwrap_or(usize::MAX);
            match known.iter().find(|(_, words)| words.contains(&h.as_str())) {
                Some((what, _)) => match at.get(what).copied() {
                    None => {
                        at.insert(what, i);
                    }
                    Some(held) if rank(what, &h) < rank(what, &heading(&row[held])) => {
                        not_used.push(row[held].trim().to_string());
                        at.insert(what, i);
                    }
                    Some(_) => not_used.push(cell.trim().to_string()),
                },
                None => not_used.push(cell.trim().to_string()),
            }
        }
        if needed.iter().all(|n| at.contains_key(n)) {
            return Some(Columns { at, not_used, row: r });
        }
    }
    None
}

/// What bringing a table in did, for whoever brought it to read. Nothing in the file is
/// left out of this: a row is counted in `done`, in `same` or in `not_used`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Brought {
    /// Rows under the headings.
    pub rows: usize,
    /// What was set, each said in full.
    pub done: Vec<String>,
    /// Rows that say what the job already holds.
    pub same: usize,
    /// Rows that were not used: the row's number in the file, counted from 1, and why.
    pub not_used: Vec<(usize, String)>,
    /// Columns whose heading is not one this table has a use for.
    pub columns_not_used: Vec<String>,
    /// Rows above the headings, which were not read.
    pub above: usize,
    /// What whoever brought it in has to know that is not one row's fate: a column of
    /// the list that was overruled, a row read whose figures do not agree with each
    /// other, part of a row that was not read.
    pub notes: Vec<String>,
}

/// The number a row has in its file: what the reader of the file gave, or its place in
/// the rows counted from 1.
fn row_number(numbers: Option<&[usize]>, i: usize) -> usize {
    numbers.and_then(|n| n.get(i).copied()).unwrap_or(i + 1)
}

/// A figure as a list writes it: `4.25`, `1,204.50`, `$450`. A comma is read only where
/// it parts thousands, by the rule a typed price is read by. Anywhere else it may be the
/// decimal mark of a list written in another country, where `9,05` is nine and a bit and
/// `1.204,50` is twelve hundred: the figure is refused, not guessed at.
fn list_figure(text: &str) -> Result<f64, String> {
    let t = text.trim().trim_start_matches('$').trim();
    // The same rule a price typed at the keyboard is read by (`units::parse_price`).
    if t.contains(',') && !units::commas_part_thousands(t) {
        return Err(format!("\"{}\" has a comma that does not part thousands; if it is the decimal mark, write a point (9.05)", text.trim()));
    }
    let x: f64 = t.replace(',', "").parse().map_err(|_| format!("\"{}\" is not a figure", text.trim()))?;
    if !x.is_finite() {
        return Err(format!("\"{}\" is not a figure", text.trim()));
    }
    Ok(x)
}

/// The job's price list as rows: its headings, then for each material its own price and
/// how it is ordered, then each supplier's price for it.
pub fn prices_out(job: &mut Job) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = vec![PRICE_HEADINGS.iter().map(|h| h.to_string()).collect()];
    for e in job.order.clone() {
        let (Some(node), Some(supply)) = (job.world.get::<Node>(e), job.world.get::<Supply>(e)) else { continue };
        let by = supply.price.unit.trim_start_matches("$/").to_string();
        let priced = |v: &Val<f64>, supplier: &str| -> Vec<String> { vec![node.name.clone(), by.clone(), v.value.map(exact).unwrap_or_default(), supplier.to_string(), status_said(v.status).to_string(), v.source.clone()] };
        let mut own = priced(&supply.price, "");
        if let Some(o) = &supply.order {
            own.push(format!("{} {}", o.factor.value.map(exact).unwrap_or_else(|| "unknown".to_string()), o.factor.unit));
            own.push(if o.round_up { "yes" } else { "no" }.to_string());
            own.push(status_said(o.factor.status).to_string());
            own.push(o.factor.source.clone());
        }
        rows.push(own);
        for (supplier, quote) in &supply.quotes {
            rows.push(priced(quote, supplier));
        }
    }
    rows
}

/// A price as a list writes it: `4.25`, `$4.25`, `4.25/lb`, `$1,204.50 / sheet`. The
/// figure, and the unit after the stroke if there is one.
fn price_cell(text: &str) -> Result<(f64, Option<String>), String> {
    let t = text.trim();
    let (figure, unit) = match t.split_once('/') {
        Some((f, u)) => (f.trim(), Some(u.trim().to_string()).filter(|u| !u.is_empty())),
        None => (t, None),
    };
    let x = list_figure(figure).map_err(|why| if why.contains("comma") { format!("its price: {why}") } else { format!("its price, \"{}\", is not a figure", text.trim()) })?;
    if x < 0.0 {
        return Err(format!("its price, \"{}\", is less than nothing", text.trim()));
    }
    Ok((x, unit))
}

/// A unit as a list writes it, less what is only the list's way of writing: a full stop
/// after it (`lb.`), and the plural of the pound (`lbs`). Nothing else is changed.
fn unit_as_written(unit: &str) -> String {
    let u = unit.trim().trim_end_matches('.');
    if u.eq_ignore_ascii_case("lbs") {
        u[..2].to_string()
    } else {
        u.to_string()
    }
}

fn dollars(x: f64, by: &str) -> String {
    format!("${}/{by}", crate::view::figure(x, 2))
}

/// Bring a price list in. `supplier` says whose prices these are when the table has no
/// column for it (none: the job's own). `how` is whoever brings the list in saying how
/// sure its prices are and where they came from: it stands over the list's own columns
/// for that, and the result says so. With no `how`, the list's own columns are read; a
/// list that has neither is refused. With `add`, a row for a material the job does not
/// have adds the material. `numbers` gives each row's number in its file.
pub fn prices_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>, supplier: Option<&str>, add: bool, how: Option<Entered>) -> Result<Brought, String> {
    let cols = columns(rows, &PRICE_COLUMNS, &["material", "price"]).ok_or_else(|| {
        "this is not a price list this program can read: within its first rows it has no row of headings with a column for what is priced (Material, Item or Description) and one for the price (Price, Unit price or Cost)".to_string()
    })?;
    if !cols.has("unit") {
        return Err("the price list has no column for what each thing is bought by (Unit, UOM or Bought by): a price is for a unit, and this cannot tell a pound from a sheet".into());
    }
    // What the person bringing the list in says of it stands over what the list says of
    // itself: a supplier's flyer that calls its own prices Known does not make them so.
    let overruled = how.is_some() && cols.has("status") && cols.has("source");
    let own_words = how.is_none() && cols.has("status") && cols.has("source");
    if !own_words && how.is_none() {
        return Err("say how sure these prices are and where they came from (--status, --source): the list has no columns for it".into());
    }
    if let Some(s) = supplier {
        if s.trim().is_empty() {
            return Err("a supplier has a name".into());
        }
    }
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len().saturating_sub(cols.row + 1), columns_not_used: cols.not_used.clone(), above: cols.row, ..Brought::default() };
        // The materials the job has, and the suppliers, as the job names them.
        let mut materials: Vec<(String, String)> = {
            let mut q = job.world.query::<(&Node, &Kind)>();
            q.iter(&job.world).filter(|(_, k)| **k == Kind::Material).map(|(n, _)| (n.name.clone(), n.path.clone())).collect()
        };
        let mut suppliers: Vec<String> = {
            let mut q = job.world.query::<&Supply>();
            let mut all: Vec<String> = q.iter(&job.world).flat_map(|s| s.quotes.keys().cloned()).collect();
            all.sort();
            all.dedup();
            all
        };
        if overruled {
            let how = how.as_ref().expect("overruled only with a how");
            out.notes.push(format!("the list's own columns for how sure each price is and where it came from were not read: every price is {}, {}, as you said", status_said(how.status), how.source));
        }
        // What each row of this list has priced so far: a thing priced twice at two
        // prices is not taken a second time.
        let mut priced: Vec<(String, Option<String>, usize, Option<f64>)> = Vec::new();
        for (i, row) in rows.iter().enumerate().skip(cols.row + 1) {
            let number = row_number(numbers, i);
            let mut skip = |why: String| out.not_used.push((number, why));
            let name = cols.cell(row, "material");
            if name.is_empty() {
                skip("it names no material".into());
                continue;
            }
            // How sure, and where from: the row's own words, or whoever brought the list.
            let (status, source) = if own_words {
                match (status_read(cols.cell(row, "status")), cols.cell(row, "source")) {
                    (Some(status), source) if !source.is_empty() => (status, source.to_string()),
                    (None, _) => {
                        skip(format!("{name}: how sure it is reads {:?}, which is not Known, Assumed or Unknown", cols.cell(row, "status")));
                        continue;
                    }
                    (Some(_), _) => {
                        skip(format!("{name}: it does not say where its price came from"));
                        continue;
                    }
                }
            } else {
                let how = how.as_ref().expect("checked above");
                (how.status, how.source.to_string())
            };
            let typed = cols.cell(row, "price");
            let (price, unit_in_price) = if typed.is_empty() {
                if status != Status::Unknown {
                    skip(format!("{name}: it has no price"));
                    continue;
                }
                (None, None)
            } else {
                match price_cell(typed) {
                    Ok((x, unit)) => (Some(x), unit),
                    Err(why) => {
                        skip(format!("{name}: {why}"));
                        continue;
                    }
                }
            };
            if price.is_some() && status == Status::Unknown {
                skip(format!("{name}: it has a price and says the price is Unknown"));
                continue;
            }
            let unit_in_price = unit_in_price.map(|u| unit_as_written(&u));
            let unit_cell = unit_as_written(cols.cell(row, "unit"));
            let unit = match (unit_cell.as_str(), unit_in_price) {
                ("", Some(u)) => u,
                ("", None) => {
                    skip(format!("{name}: it does not say what it is bought by"));
                    continue;
                }
                (u, Some(other)) if !u.eq_ignore_ascii_case(&other) => {
                    skip(format!("{name}: it is bought by the {u} and priced by the {other}"));
                    continue;
                }
                (u, _) => u.to_string(),
            };
            // Whose price: the row's supplier, or the one named for the whole list.
            let whose = match (cols.cell(row, "supplier"), supplier) {
                ("", given) => given.map(str::to_string),
                (named, _) => Some(named.to_string()),
            };
            let whose = whose.map(|w| suppliers.iter().find(|known| check::same_name(known, &w)).cloned().unwrap_or(w));
            // The material, by its name however it is typed.
            let path = match materials.iter().find(|(known, _)| check::same_name(known, name)) {
                Some((_, path)) => path.clone(),
                None if add => match job.add_material(name, &unit) {
                    Ok(path) => {
                        out.done.push(format!("{name}: added, bought by the {unit}"));
                        materials.push((name.to_string(), path.clone()));
                        path
                    }
                    Err(why) => {
                        skip(format!("{name}: {why}"));
                        continue;
                    }
                },
                None => {
                    skip(format!("the job has no material called {name:?}"));
                    continue;
                }
            };
            let e = job.find(&path).ok_or_else(|| format!("no element {path}"))?;
            let known = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_default();
            let mut supply = job.world.get::<Supply>(e).cloned().ok_or_else(|| format!("{path} has no price to set"))?;
            let by = supply.price.unit.trim_start_matches("$/").to_string();
            if !unit.eq_ignore_ascii_case(&by) {
                skip(format!("{name}: priced by the {unit}; the job buys {known} by the {by}"));
                continue;
            }
            match priced.iter().find(|(p, w, _, _)| *p == path && *w == whose) {
                Some((_, _, first, at)) if *at != price => {
                    let said = |x: &Option<f64>| x.map(|x| dollars(x, &by)).unwrap_or_else(|| "no price".to_string());
                    skip(format!("{name}: row {first} of this list already prices it, at {}; this row says {}, and was not used", said(at), said(&price)));
                    continue;
                }
                Some(_) => {}
                None => priced.push((path.clone(), whose.clone(), number, price)),
            }
            let new = Val { value: price, unit: supply.price.unit.clone(), status, source: source.clone() };
            let was = match &whose {
                None => Some(supply.price.clone()),
                Some(s) => supply.quotes.get(s).cloned(),
            };
            let mut changed = was.as_ref() != Some(&new);
            let said = |v: &Val<f64>| v.value.map(|x| dollars(x, &by)).unwrap_or_else(|| "no price".to_string());
            let mut what = match (&whose, &was) {
                (None, Some(old)) if old.value.is_none() => format!("{known}: {}", said(&new)),
                (None, Some(old)) => format!("{known}: {} becomes {}", said(old), said(&new)),
                (Some(s), Some(old)) => format!("{known} from {s}: {} becomes {}", said(old), said(&new)),
                (Some(s), None) => format!("{known} from {s}: {}, which {s} had not quoted", said(&new)),
                (None, None) => format!("{known}: {}", said(&new)),
            };
            if was.as_ref().is_some_and(|old| old.value == new.value && old != &new) {
                what = format!("{known}{}: {} still, now {} ({})", whose.as_ref().map(|s| format!(" from {s}")).unwrap_or_default(), said(&new), status_said(status), source);
            }
            match &whose {
                None => supply.price = new,
                Some(s) => {
                    supply.quotes.insert(s.clone(), new);
                    if !suppliers.contains(s) {
                        suppliers.push(s.clone());
                    }
                }
            }
            // How it is ordered, where the row says: on the job's own row only.
            let order = cols.cell(row, "order");
            if !order.is_empty() && whose.is_some() {
                out.notes.push(format!("row {number}: {name}: how it is ordered ({order}) was not read: that is the job's to say of the material, not one supplier's row's"));
            }
            if !order.is_empty() && whose.is_none() {
                let (figure, per) = match order.split_once(' ') {
                    Some(parts) => parts,
                    None => {
                        skip(format!("{name}: it is ordered at {order:?}, which is not a figure and what per what (1.1 lb/SF)"));
                        continue;
                    }
                };
                let (order_status, order_source) = match (status_read(cols.cell(row, "order status")), cols.cell(row, "order source")) {
                    (Some(s), src) if !src.is_empty() => (s, src.to_string()),
                    _ if !own_words => (status, source.clone()),
                    _ => {
                        skip(format!("{name}: how it is ordered does not say how sure it is and where it came from"));
                        continue;
                    }
                };
                let value = if figure == "unknown" {
                    None
                } else {
                    match units::parse_number(figure) {
                        Ok(x) => Some(x),
                        Err(why) => {
                            skip(format!("{name}: it is ordered at {order:?}: {why}"));
                            continue;
                        }
                    }
                };
                let whole = matches!(cols.cell(row, "whole").to_lowercase().as_str(), "yes" | "y" | "true");
                let new = Order { unit: by.clone(), factor: Val { value, unit: per.trim().to_string(), status: order_status, source: order_source }, round_up: whole };
                if supply.order.as_ref() != Some(&new) {
                    changed = true;
                    what.push_str(&format!("; ordered at {order}{}", if whole { ", whole ones only" } else { "" }));
                }
                supply.order = Some(new);
            }
            if !changed {
                out.same += 1;
                continue;
            }
            if let Err(why) = check::supply(&path, Kind::Material, &supply) {
                skip(format!("{name}: {why}"));
                continue;
            }
            job.put(&path, supply)?;
            out.done.push(what);
        }
        Ok(out)
    })
}

// ---------------------------------------------------------------- an assembly library

/// The headings of an assembly library as this program writes it: one row for each line
/// an assembly puts on what it goes on. After them come three columns for each further
/// figure a line's quantity names (`Figure 1`, `Figure 1: how sure`, `Figure 1: where
/// from`), as many as the line with the most has.
pub const ASSEMBLY_HEADINGS: [&str; 26] = [
    "Assembly",
    "Goes on",
    "Cost code",
    "Code title",
    "Code scheme",
    "Code: how sure",
    "Code: where from",
    "Line",
    "Class",
    "Quantity",
    "Stretch-out",
    "Stretch-out: how sure",
    "Stretch-out: where from",
    "Waste %",
    "Waste: how sure",
    "Waste: where from",
    "Ordered by",
    "Ordered at",
    "Whole ones only",
    "Ordered at: how sure",
    "Ordered at: where from",
    "Unit cost",
    "Unit cost: how sure",
    "Unit cost: where from",
    "Made of the element's material",
    "Off",
];

/// A length, or a plain number, as a cell: `30 in`, `25' 6 1/2"`, `2`. What is not known
/// is `unknown` and its unit. Written so that reading it gives the same value: the way a
/// drawing writes it where that is exact, and in full otherwise.
fn length_out(v: &Val<units::Len>) -> String {
    let Some(len) = &v.value else { return format!("unknown {}", v.unit).trim().to_string() };
    let plain = match len {
        units::Len::One(x) => format!("{} {}", exact(*x), v.unit).trim().to_string(),
        units::Len::FtIn([feet, inches]) => format!("{}' {}\"", exact(*feet), exact(*inches)),
    };
    if v.unit.is_empty() {
        return plain;
    }
    let as_drawn = units::show_length(len, &v.unit);
    match units::parse_length(&as_drawn) {
        Ok((back, unit)) if back == *len && unit == v.unit => as_drawn,
        _ => plain,
    }
}

fn length_in(cell: &str, status: Status, source: &str) -> Result<Val<units::Len>, String> {
    let t = cell.trim();
    if let Some(unit) = t.strip_prefix("unknown") {
        return Ok(Val { value: None, unit: unit.trim().to_string(), status, source: source.to_string() });
    }
    if let Ok(x) = t.parse::<f64>() {
        return Ok(Val { value: Some(units::Len::One(x)), unit: String::new(), status, source: source.to_string() });
    }
    let (len, unit) = units::parse_length(t)?;
    Ok(Val { value: Some(len), unit: unit.to_string(), status, source: source.to_string() })
}

/// A figure and what it is in, as a cell: `30 SF/sheet`, `unknown lb/SF`.
fn figure_out(v: &Val<f64>) -> String {
    format!("{} {}", v.value.map(exact).unwrap_or_else(|| "unknown".to_string()), v.unit).trim().to_string()
}

fn figure_in(cell: &str, status: Status, source: &str) -> Result<Val<f64>, String> {
    let t = cell.trim();
    let (figure, unit) = t.split_once(' ').unwrap_or((t, ""));
    let value = if figure == "unknown" { None } else { Some(figure.parse::<f64>().map_err(|_| format!("{cell:?} is not a figure and what it is in"))?) };
    Ok(Val { value, unit: unit.trim().to_string(), status, source: source.to_string() })
}

/// Dollars for a unit, as a cell: `$4.25/SF`; not known, `$?/SF`.
fn rate_out(v: &Val<f64>) -> String {
    format!("${}/{}", v.value.map(exact).unwrap_or_else(|| "?".to_string()), v.unit.trim_start_matches("$/"))
}

fn rate_in(cell: &str, status: Status, source: &str) -> Result<Val<f64>, String> {
    let t = cell.trim().trim_start_matches('$');
    let (figure, unit) = t.split_once('/').ok_or_else(|| format!("{cell:?} is not dollars for a unit, like $4.25/SF"))?;
    let value = if figure.trim() == "?" { None } else { Some(figure.trim().replace(',', "").parse::<f64>().map_err(|_| format!("{cell:?} is not dollars for a unit, like $4.25/SF"))?) };
    Ok(Val { value, unit: format!("$/{}", unit.trim()), status, source: source.to_string() })
}

fn yes(b: bool) -> String {
    if b { "yes" } else { "" }.to_string()
}

fn is_yes(cell: &str) -> bool {
    matches!(cell.trim().to_lowercase().as_str(), "yes" | "y" | "true")
}

/// One cost line as the cells after an assembly's (or an element's) own: from `Line` on.
fn line_out(l: &crate::components::CostLine) -> Vec<String> {
    let said = |s: Status| status_said(s).to_string();
    let mut row = vec![l.description.clone(), l.class.map(|c| c.title().to_string()).unwrap_or_default(), l.quantity.clone()];
    match &l.stretch_out {
        Some(v) => row.extend([length_out(v), said(v.status), v.source.clone()]),
        None => row.extend([String::new(), String::new(), String::new()]),
    }
    match &l.waste {
        Some(v) => row.extend([v.value.map(exact).unwrap_or_else(|| "unknown".to_string()), said(v.status), v.source.clone()]),
        None => row.extend([String::new(), String::new(), String::new()]),
    }
    match &l.order {
        Some(o) => row.extend([o.unit.clone(), figure_out(&o.factor), if o.round_up { "yes" } else { "no" }.to_string(), said(o.factor.status), o.factor.source.clone()]),
        None => row.extend(std::iter::repeat_n(String::new(), 5)),
    }
    if crate::components::CostLine::is_no_price_of_its_own(&l.unit_cost) {
        row.extend([String::new(), String::new(), String::new()]);
    } else {
        row.extend([rate_out(&l.unit_cost), said(l.unit_cost.status), l.unit_cost.source.clone()]);
    }
    row.push(yes(l.of_material));
    row.push(yes(l.off));
    for (name, v) in &l.inputs {
        row.extend([format!("{name} = {}", length_out(v)), said(v.status), v.source.clone()]);
    }
    row
}

/// The cells of `line_out`, read back. `cells` starts at `Line`.
fn line_in(cells: &[String]) -> Result<crate::components::CostLine, String> {
    use crate::components::{Class, CostLine};
    let cell = |i: usize| cells.get(i).map(|s| s.trim()).unwrap_or("");
    let how = |i: usize, what: &str| -> Result<(Status, String), String> {
        let status = status_read(cell(i)).ok_or_else(|| format!("its {what} does not say how sure it is (Known, Assumed or Unknown)"))?;
        if cell(i + 1).is_empty() {
            return Err(format!("its {what} does not say where it came from"));
        }
        Ok((status, cell(i + 1).to_string()))
    };
    let description = cell(0).to_string();
    let class = match cell(1) {
        "" => None,
        text => Some(Class::ALL.into_iter().find(|c| c.title().eq_ignore_ascii_case(text)).map(Ok).unwrap_or_else(|| Class::parse(text))?),
    };
    let stretch_out = if cell(3).is_empty() && cell(4).is_empty() {
        None
    } else {
        let (status, source) = how(4, "stretch-out")?;
        Some(length_in(cell(3), status, &source)?)
    };
    let waste = if cell(6).is_empty() && cell(7).is_empty() {
        None
    } else {
        let (status, source) = how(7, "waste")?;
        let value = if cell(6) == "unknown" { None } else { Some(cell(6).parse::<f64>().map_err(|_| format!("its waste, {:?}, is not a percentage", cell(6)))?) };
        Some(Val { value, unit: "%".into(), status, source })
    };
    let order = if cell(9).is_empty() && cell(10).is_empty() {
        None
    } else {
        let (status, source) = how(12, "order conversion")?;
        Some(Order { unit: cell(9).to_string(), factor: figure_in(cell(10), status, &source)?, round_up: is_yes(cell(11)) })
    };
    let unit_cost = if cell(14).is_empty() && cell(15).is_empty() {
        CostLine::no_price_of_its_own()
    } else {
        let (status, source) = how(15, "unit cost")?;
        rate_in(cell(14), status, &source)?
    };
    let mut inputs = BTreeMap::new();
    let mut i = 19;
    while i < cells.len() {
        if !cell(i).is_empty() {
            let (name, value) = cell(i).split_once('=').ok_or_else(|| format!("{:?} is not a figure by name, like spacing = 12 in", cell(i)))?;
            let (status, source) = how(i + 1, name.trim())?;
            inputs.insert(name.trim().to_string(), length_in(value, status, &source)?);
        }
        i += 3;
    }
    Ok(CostLine { description, quantity: cell(2).to_string(), stretch_out, inputs, waste, order, unit_cost, class, off: is_yes(cell(18)), of_material: is_yes(cell(17)) })
}

fn kind_said(k: Kind) -> String {
    serde_json::to_value(k).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
}

fn kind_read(text: &str) -> Option<Kind> {
    serde_json::from_value(serde_json::Value::String(text.trim().to_string())).ok()
}

/// The job's assemblies (its conditions, each with the lines it puts on what it goes on)
/// as rows.
pub fn assemblies_out(job: &mut Job) -> Vec<Vec<String>> {
    use crate::components::Recipe;
    let mut body: Vec<Vec<String>> = Vec::new();
    for e in job.order.clone() {
        let (Some(node), Some(recipe)) = (job.world.get::<Node>(e), job.world.get::<Recipe>(e)) else { continue };
        let code = &recipe.cost.code;
        let (scheme, number, title) = code.value.as_ref().map(|c| (c.scheme.clone(), c.code.clone(), c.title.clone())).unwrap_or_default();
        let own = vec![node.name.clone(), recipe.applies_to.iter().map(|k| kind_said(*k)).collect::<Vec<_>>().join(", "), number, title, scheme, status_said(code.status).to_string(), code.source.clone()];
        if recipe.cost.lines.is_empty() {
            body.push(own.clone());
        }
        for line in &recipe.cost.lines {
            let mut row = own.clone();
            row.extend(line_out(line));
            body.push(row);
        }
    }
    let widest = body.iter().map(Vec::len).max().unwrap_or(0).max(ASSEMBLY_HEADINGS.len());
    let mut headings: Vec<String> = ASSEMBLY_HEADINGS.iter().map(|h| h.to_string()).collect();
    for k in 1..=(widest - ASSEMBLY_HEADINGS.len()) / 3 {
        headings.extend([format!("Figure {k}"), format!("Figure {k}: how sure"), format!("Figure {k}: where from")]);
    }
    let mut rows = vec![headings];
    rows.extend(body);
    rows
}

/// Bring an assembly library in: a table this program wrote. An assembly the job already
/// has by that name is left as it is: counted as the same if it is, listed if it differs.
pub fn assemblies_in(job: &mut Job, rows: &[Vec<String>], numbers: Option<&[usize]>) -> Result<Brought, String> {
    use crate::components::{Cost, CostCode, Recipe};
    let head = rows.first().ok_or("the table is empty")?;
    let wanted: Vec<String> = ASSEMBLY_HEADINGS.iter().map(|h| heading(h)).collect();
    let got: Vec<String> = head.iter().take(wanted.len()).map(|h| heading(h)).collect();
    if got != wanted {
        return Err(format!("this is not an assembly library this program wrote: its headings have to be, in order: {}", ASSEMBLY_HEADINGS.join(", ")));
    }
    all_or_nothing(job, |job| {
        let mut out = Brought { rows: rows.len() - 1, ..Brought::default() };
        // Rows gathered by assembly, in the order each is first named.
        let mut gathered: Vec<(String, Vec<(usize, &Vec<String>)>)> = Vec::new();
        for (i, row) in rows.iter().enumerate().skip(1) {
            let name = row.first().map(|s| s.trim()).unwrap_or("");
            let number = row_number(numbers, i);
            if name.is_empty() {
                out.not_used.push((number, "it names no assembly".into()));
                continue;
            }
            match gathered.iter_mut().find(|(known, _)| known == name) {
                Some((_, its)) => its.push((number, row)),
                None => gathered.push((name.to_string(), vec![(number, row)])),
            }
        }
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        for (name, its) in gathered {
            let first = its[0].1;
            let cell = |i: usize| first.get(i).map(|s| s.trim()).unwrap_or("");
            let made = (|| -> Result<Recipe, String> {
                // Every row of an assembly says the same of the assembly itself: what it
                // goes on and its cost code. Two that differ are two assemblies under one
                // name, and neither is taken for the other.
                if let Some((n, _)) = its.iter().find(|(_, row)| (1..7).any(|c| row.get(c).map(|s| s.trim()).unwrap_or("") != cell(c))) {
                    return Err(format!("its rows do not agree on what it goes on or on its cost code (row {} and row {n}): two assemblies cannot have one name", its[0].0));
                }
                let applies_to: Vec<Kind> = cell(1).split(',').filter(|s| !s.trim().is_empty()).map(|s| kind_read(s).ok_or_else(|| format!("it goes on {:?}, which is not a kind of element", s.trim()))).collect::<Result<_, _>>()?;
                let status = status_read(cell(5)).ok_or("its cost code does not say how sure it is")?;
                let value = if cell(2).is_empty() && cell(3).is_empty() && cell(4).is_empty() { None } else { Some(CostCode { scheme: cell(4).to_string(), code: cell(2).to_string(), title: cell(3).to_string() }) };
                let code = Val { value, unit: String::new(), status, source: cell(6).to_string() };
                let mut lines = Vec::new();
                for (_, row) in &its {
                    if row.len() > 7 && row[7..].iter().any(|c| !c.trim().is_empty()) {
                        lines.push(line_in(&row[7..]).map_err(|why| format!("its line {:?}: {why}", row.get(7).map(|s| s.as_str()).unwrap_or("")))?);
                    }
                }
                Ok(Recipe { applies_to, cost: Cost { code, lines } })
            })();
            let numbers: Vec<usize> = its.iter().map(|(n, _)| *n).collect();
            let recipe = match made {
                Ok(r) => r,
                Err(why) => {
                    out.not_used.extend(numbers.iter().map(|n| (*n, format!("{name}: {why}"))));
                    continue;
                }
            };
            // An assembly the job has by this name.
            let known: Option<(String, Option<Recipe>)> = {
                let mut q = job.world.query::<(&Node, &Kind, Option<&Recipe>)>();
                q.iter(&job.world).find(|(n, k, _)| **k == Kind::Condition && check::same_name(&n.name, &name)).map(|(n, _, r)| (n.path.clone(), r.cloned()))
            };
            match known {
                Some((_, Some(has))) if has == recipe => out.same += its.len(),
                Some((path, _)) => out.not_used.extend(numbers.iter().map(|n| (*n, format!("the job already has an assembly called {name:?} ({path}) and it is not the same: it was left as it is")))),
                None => {
                    if let Err(why) = check::recipe(&format!("the assembly {name:?}"), Kind::Condition, &recipe) {
                        out.not_used.extend(numbers.iter().map(|n| (*n, why.clone())));
                        continue;
                    }
                    let count = recipe.cost.lines.len();
                    let path = job.add(&root, Kind::Condition, &name)?;
                    job.put(&path, recipe)?;
                    out.done.push(format!("{name}: added, with {count} line{}", if count == 1 { "" } else { "s" }));
                }
            }
        }
        Ok(out)
    })
}

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

const PRIOR_COLUMNS: [(&str, &[&str]); 7] = [
    ("description", &["description", "line item", "scope", "work", "line", "item"]),
    ("quantity", &["quantity", "qty"]),
    ("unit", &["unit", "uom", "u/m", "um"]),
    ("unit cost", &["unit cost", "unit price", "rate", "price"]),
    ("amount", &["amount", "total", "extension", "ext", "line total", "extended"]),
    ("class", &["class", "category"]),
    ("code", &["cost code", "code"]),
];

/// A figure of money or of quantity as an estimate writes it: `1,204.50`, `$450`.
fn money_cell(text: &str) -> Result<Option<f64>, String> {
    if text.trim().trim_start_matches('$').trim().is_empty() {
        return Ok(None);
    }
    list_figure(text).map(Some)
}

/// Whether a row of an estimate is a total of the rows above it and not a line: it is
/// called a total, or begins with the word and gives no quantity and no unit cost.
fn is_total_row(description: &str, quantity: Option<f64>, unit_cost: Option<f64>) -> bool {
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
                    lines.push(PriorLine { description: description.to_string(), quantity, unit: cols.cell(row, "unit").to_string(), unit_cost, amount, class: cols.cell(row, "class").to_string(), code: cols.cell(row, "code").to_string() });
                }
            }
        }
        if lines.is_empty() {
            return Err("no row of the estimate could be read as a line".into());
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

/// The rows a flat reading of the job shows for a prior estimate.
pub fn rows(w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity, path: &str) -> Vec<crate::view::Row> {
    let Some(p) = w.get::<Prior>(e) else { return Vec::new() };
    vec![crate::view::entered(path, "lines of a prior estimate", &p.lines, |l| {
        let total: f64 = l.iter().filter_map(|x| x.amount).sum();
        format!("{}, adding to ${}", l.len(), crate::view::figure(total, 2))
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
        // What each prior line comes to: its amount, or its quantity times its unit cost.
        let comes_to = |l: &PriorLine| match (l.amount, l.quantity, l.unit_cost) {
            (Some(a), _, _) => (Some(a), false),
            (None, Some(q), Some(c)) => (Some(crate::lines::cents(q * c)), true),
            _ => (None, false),
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
