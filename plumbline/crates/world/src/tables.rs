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
use crate::building::shape::Shape;
use crate::value::{Status, Val};

mod assemblies;
mod cabinet_rules;
mod connections;
mod items;
mod prices;
mod prior;
mod shapes;

pub use assemblies::*;
pub use cabinet_rules::*;
pub use connections::*;
pub use items::*;
pub use prices::*;
pub use prior::*;
pub use shapes::*;

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

/// The item library the program ships with: a few things of each trade that has any,
/// at typical sizes, every size Assumed and saying so, no maker and no model.
pub const STARTER_ITEMS: &str = include_str!("../../../library/items.csv");

// ---------------------------------------------------------------- an assembly library

/// The headings of an assembly library as this program writes it: one row for each line
/// an assembly puts on what it goes on. After them come three columns for each further
/// figure a line's quantity names (`Figure 1`, `Figure 1: how sure`, `Figure 1: where
/// from`), as many as the line with the most has.
/// The starter library of assemblies the program ships with (`library/sixteen-divisions.csv`):
/// whole assemblies for a house's walls, floors, rooms, roof, openings and runs, with a
/// line in every one of the sixteen divisions and no price in it. It is the file as it
/// stands in the repository, read by the same reader as any other library.
pub const STARTER_LIBRARY: &str = include_str!("../../../library/sixteen-divisions.csv");

/// The shapes the program ships with (`library/shapes.csv`): the forms it draws a vanity, a
/// water closet or a table as, one row a block.
pub const STARTER_SHAPES: &str = include_str!("../../../library/shapes.csv");

// ---------------------------------------------------------------- how a cabinet's code is read

/// The cabinet rules the program ships with (`library/cabinet-rules.csv`): how the letters of a code (B, SB, W, T, F) and the
/// number after them make a box, as a table a person can open, change and bring back (call 215). Every row Assumed.
pub const STARTER_CABINET_RULES: &str = include_str!("../../../library/cabinet-rules.csv");

/// The connections the program ships with (`library/connections.csv`): for the starter items a published rough-in gives a
/// place for, one row a connection, every place and size Assumed, "a typical rough-in; the product is not chosen". The other
/// starter items have none, and `items` counts them.
pub const STARTER_CONNECTIONS: &str = include_str!("../../../library/connections.csv");

