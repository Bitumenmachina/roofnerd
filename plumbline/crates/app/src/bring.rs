//! What people already have, brought into the job from the window: a supplier's price
//! list, an assembly library, an estimate made before, a model from another trade.
//!
//! Each is the same verb the command line calls (`tables::prices_in`,
//! `tables::assemblies_in`, `tables::prior_in`, `plumbline_ifc::import`), all of it or
//! none of it. What was set, what was not used and what was not read are said in full
//! over the model: nothing in the file goes missing without being said. What is brought
//! in here is Assumed, and its source says the file it came from, unless the file says
//! for itself how sure each row is (this program's own price list does).

use std::path::Path;

use plumbline_world::editing::Entered;
use plumbline_world::tables::{self, Brought};
use plumbline_world::value::Status;

use crate::model::{Bring, Doc, Tone};

/// A file's name as typed, found from the job's own folder first, then as it stands.
fn found(doc: &Doc, name: &str) -> std::path::PathBuf {
    let beside = doc.path.parent().map(|p| p.join(name));
    match beside {
        Some(p) if !Path::new(name).is_absolute() && p.exists() => p,
        _ => Path::new(name).to_path_buf(),
    }
}

fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// What a table brought in did, said in full.
fn said(file: &str, lost: &[String], b: &Brought) -> (String, bool) {
    let mut out = format!("Brought in {file}: {} read, {} set.", count(b.rows, "row", "rows"), b.done.len());
    for d in b.done.iter().take(6) {
        out.push_str(&format!("\n{d}"));
    }
    if b.done.len() > 6 {
        out.push_str(&format!("\nand {} more", b.done.len() - 6));
    }
    if b.same > 0 {
        out.push_str(&format!("\n{} the same as the job already has", count(b.same, "row is", "rows are")));
    }
    for (row, why) in &b.not_used {
        out.push_str(&format!("\nNot used, row {row}: {why}"));
    }
    for note in &b.notes {
        out.push_str(&format!("\nNote: {note}"));
    }
    if b.above > 0 {
        out.push_str(&format!("\nNot read: {} above the headings", count(b.above, "row", "rows")));
    }
    for c in &b.columns_not_used {
        out.push_str(&format!("\nNot read: the column {c:?}"));
    }
    for l in lost {
        out.push_str(&format!("\nNot read: {l}"));
    }
    let to_see = !b.not_used.is_empty() || !b.notes.is_empty() || !b.columns_not_used.is_empty() || !lost.is_empty() || b.above > 0;
    (out, to_see)
}

/// Bring a file in as what it is said to be. `typed` is the file's name; for a price
/// list a supplier's name can follow a comma ("lists/ridge.xlsx, Ridge Supply").
pub fn bring(doc: &mut Doc, what: Bring, typed: &str) {
    let (name, supplier) = match (what, typed.rsplit_once(',')) {
        (Bring::Prices, Some((file, who))) if !who.trim().is_empty() && !who.contains('.') => (file.trim(), Some(who.trim().to_string())),
        _ => (typed.trim(), None),
    };
    if name.is_empty() {
        crate::tools::cant(doc, "bring it in", "no file was named. Type the file's name, or drop the file on this window.");
        doc.touch();
        return;
    }
    let path = found(doc, name);
    let file = path.file_name().and_then(|f| f.to_str()).unwrap_or(name).to_string();
    let source = format!("{file}, brought in in the window");
    let how = Entered { status: Status::Assumed, source: &source };
    let done: Result<(String, bool), String> = (|| {
        let bytes = std::fs::read(&path).map_err(|e| format!("{file} {}", crate::plans::why_not_opened(&e)))?;
        match what {
            Bring::Model => {
                let b = plumbline_ifc::import(&mut doc.job, &bytes, &file, how)?;
                let mut out = format!("Brought in {file} ({}): {}.", b.schema, b.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { what.strip_suffix('s').unwrap_or(what) } else { what })).collect::<Vec<_>>().join(", "));
                let left: usize = b.left_out.iter().map(|l| l.1).sum();
                if left > 0 {
                    out.push_str(&format!("\nNot brought in: {}", count(left, "thing", "things")));
                    for (what, n, why) in &b.left_out {
                        out.push_str(&format!("\n{n} x {what}: {why}"));
                    }
                }
                for note in &b.notes {
                    out.push_str(&format!("\nNote: {note}"));
                }
                Ok((out, left > 0 || !b.notes.is_empty()))
            }
            _ => {
                let sheet = plumbline_sheets::read(&bytes, None).map_err(|why| format!("{file}: {why}"))?;
                let b = match what {
                    // The list's own words for how sure each price is, where it has
                    // them; otherwise Assumed, from this file.
                    Bring::Prices => match tables::prices_in(&mut doc.job, &sheet.rows, Some(&sheet.numbers), supplier.as_deref(), true, None) {
                        Err(why) if why.starts_with("say how sure") => tables::prices_in(&mut doc.job, &sheet.rows, Some(&sheet.numbers), supplier.as_deref(), true, Some(how)),
                        other => other,
                    },
                    Bring::Assemblies => tables::assemblies_in(&mut doc.job, &sheet.rows, Some(&sheet.numbers)),
                    _ => {
                        let called = path.file_stem().and_then(|s| s.to_str()).unwrap_or("prior estimate").to_string();
                        tables::prior_in(&mut doc.job, &called, &sheet.rows, Some(&sheet.numbers), how).map(|(_, b)| b)
                    }
                }
                .map_err(|why| format!("{file}: {why}"))?;
                Ok(said(&file, &sheet.lost, &b))
            }
        }
    })();
    match done {
        Ok((out, to_see)) => {
            doc.job.derive();
            doc.framed = false;
            doc.edit("Bring in");
            // What was not used or not read has to be seen: it is said over the model.
            doc.say(if to_see { Tone::Warned } else { Tone::Done }, out);
        }
        Err(why) => crate::tools::cant(doc, &format!("bring in the {}", what.name().to_lowercase()), why),
    }
    doc.redraw();
}

/// A file dropped on the window: a plan is added under the level on show, a model is
/// brought in, and a table (which could be a price list, assemblies or an estimate) is
/// kept until what it is has been said.
pub fn dropped(doc: &mut Doc, file: &str) {
    let lower = file.to_ascii_lowercase();
    if let Some((crate::model::Entry::BringIn(what), _)) = doc.entry.take() {
        // Dropped while a file was being asked for: this is that file.
        bring(doc, what, file);
    } else if lower.ends_with(".ifc") {
        bring(doc, Bring::Model, file);
    } else if lower.ends_with(".csv") || lower.ends_with(".xlsx") || lower.ends_with(".xlsm") || lower.ends_with(".xls") {
        doc.dropped = Some(file.to_string());
        doc.say(Tone::Warned, format!("{} is a table. Say what it is: click Price list, Assemblies or Prior estimate, under Bring in on the left.", Path::new(file).file_name().and_then(|f| f.to_str()).unwrap_or(file)));
        doc.touch();
    } else {
        crate::plans::add(doc, file);
    }
}
