//! "My database" (call 223): every table the program works from, written out as one and
//! brought back whole.
//!
//! Each tab is exactly what its table's own writing-out gives (`tables`), and each is brought
//! back by the reader that table already has; this module only puts them in the order they
//! depend on one another and holds them to all of it or none of it. Nothing here reads a
//! file: the command line and the window read a workbook or a folder into `Given` tabs.

use crate::editing::all_or_nothing;
use crate::tables::{self, Brought};
use crate::Job;

/// The tabs, in the order each depends on the ones before it: an item is drawn as a shape,
/// where things connect is a row of an item, a price is of an item or a material.
pub const TABS: [&str; 5] = ["Shapes", "Items", "Connections", "Assemblies", "Prices"];

/// One tab of a workbook, or one file of a folder, as it was read.
#[derive(Clone, Debug, Default)]
pub struct Given {
    /// The tab's name, or the file's name without what it ends in.
    pub name: String,
    pub rows: Vec<Vec<String>>,
    /// Each row's number in its file, counted from 1 as a spreadsheet counts them.
    pub numbers: Vec<usize>,
    /// What the file held that is not in the rows (as `plumbline_sheets::Sheet::lost`).
    pub lost: Vec<String>,
}

/// What one tab did.
#[derive(Clone, Debug, Default)]
pub struct TabIn {
    pub name: &'static str,
    pub brought: Brought,
    /// With the starter database: how many of a table's rows are for something the job has
    /// already, which keeps what it has.
    pub kept: usize,
    pub lost: Vec<String>,
}

/// What bringing a database in did, tab by tab, in the order they were brought in.
#[derive(Clone, Debug, Default)]
pub struct Whole {
    pub tabs: Vec<TabIn>,
    /// Tabs the program has no table for: named, and left alone.
    pub not_read: Vec<String>,
    /// Tabs of a database that this one did not have: the job's own are left as they are.
    pub absent: Vec<&'static str>,
}

/// Every table written out: its tab's name and its rows, headings first. The tables are
/// written by their own functions, called, not copied.
pub fn out(job: &mut Job) -> Vec<(&'static str, Vec<Vec<String>>)> {
    vec![
        ("Shapes", tables::shapes_out(job)),
        ("Items", tables::items_out(job)),
        ("Connections", tables::connections_out(job)),
        ("Assemblies", tables::assemblies_out(job)),
        ("Prices", tables::prices_out(job)),
    ]
}

/// The tables the program comes with, each as the text of its file: the starter database.
/// (There is no starter price list: prices are the person's.)
pub fn starter_files() -> [(&'static str, &'static str); 4] {
    [("Shapes", tables::STARTER_SHAPES), ("Items", tables::STARTER_ITEMS), ("Connections", tables::STARTER_CONNECTIONS), ("Assemblies", tables::STARTER_LIBRARY)]
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Bring a database in, whole. Every tab it has a table for is brought in by that table's own
/// reader, in the order they depend on one another, wherever the tabs lie in the file. If any
/// tab is refused, or any row of one cannot be used, nothing at all is brought in and the
/// reply names each tab and row and why. With `starter`, what the program ships with is
/// brought in as it is by the single tables: it adds what the job lacks and writes over
/// nothing the job has.
pub fn bring_in(job: &mut Job, given: &[Given], starter: bool) -> Result<Whole, String> {
    // Which of the tables each tab is.
    let mut found: Vec<Option<&Given>> = vec![None; TABS.len()];
    let mut not_read = Vec::new();
    for g in given {
        match TABS.iter().position(|t| t.eq_ignore_ascii_case(g.name.trim())) {
            Some(i) if found[i].is_some() => return Err(format!("the tab \"{}\" is in it twice, and there is no telling which is meant: nothing was brought in", TABS[i])),
            Some(i) => found[i] = Some(g),
            None => not_read.push(g.name.clone()),
        }
    }
    if found.iter().all(Option::is_none) {
        let has = if given.is_empty() { "it has no tabs".to_string() } else { format!("it has: {}", given.iter().map(|g| format!("\"{}\"", g.name)).collect::<Vec<_>>().join(", ")) };
        return Err(format!("this is not a database this program can read: it has none of the tabs of one ({}); {has}. Nothing was brought in", TABS.join(", ")));
    }
    let wanted: &[&str] = if starter { &TABS[..4] } else { &TABS };
    let absent: Vec<&'static str> = TABS.iter().enumerate().filter(|(i, t)| found[*i].is_none() && wanted.contains(t)).map(|(_, t)| *t).collect();

    let mut refused: Vec<String> = Vec::new();
    let done = all_or_nothing(job, |job| {
        let mut tabs = Vec::new();
        for (i, name) in TABS.iter().enumerate() {
            let Some(g) = found[i] else { continue };
            let numbers = Some(g.numbers.as_slice());
            if g.rows.is_empty() {
                let mut brought = Brought::default();
                brought.notes.push("the tab has nothing in it".to_string());
                tabs.push(TabIn { name, brought, kept: 0, lost: g.lost.clone() });
                continue;
            }
            let (brought, kept) = match (*name, starter) {
                ("Shapes", _) => (tables::shapes_in(job, &g.rows, numbers), 0),
                ("Items", false) => (tables::items_in(job, &g.rows, numbers), 0),
                ("Items", true) => match tables::items_in_missing(job, &g.rows, numbers) {
                    Ok((b, kept)) => (Ok(b), kept.len()),
                    Err(why) => (Err(why), 0),
                },
                ("Connections", false) => (tables::connections_in(job, &g.rows, numbers), 0),
                ("Connections", true) => match tables::connections_in_missing(job, &g.rows, numbers) {
                    Ok((b, kept)) => (Ok(b), kept.len()),
                    Err(why) => (Err(why), 0),
                },
                ("Assemblies", _) => (tables::assemblies_in(job, &g.rows, numbers), 0),
                // A price may be for a material only the price list knows: it is made, as it was when it was written out.
                _ => (tables::prices_in(job, &g.rows, numbers, None, true, None), 0),
            };
            match brought {
                Ok(brought) => {
                    for (row, why) in &brought.not_used {
                        refused.push(format!("  tab {name}, row {row}: {why}"));
                    }
                    tabs.push(TabIn { name, brought, kept, lost: g.lost.clone() });
                }
                Err(why) => refused.push(format!("  tab {name}: {why}")),
            }
        }
        if refused.is_empty() {
            Ok(tabs)
        } else {
            Err(String::new())
        }
    });
    match done {
        Ok(tabs) => Ok(Whole { tabs, not_read, absent }),
        Err(why) if refused.is_empty() => Err(why),
        Err(_) => Err(format!(
            "nothing was brought in: a database comes in whole or not at all, and {} cannot be used\n{}\nthe job is as it was",
            plural(refused.len(), "place in it", "places in it"),
            refused.join("\n")
        )),
    }
}
