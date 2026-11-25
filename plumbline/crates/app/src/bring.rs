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
use plumbline_world::Kind;

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

/// A file named as a person names it: its title, with no folder and no ending
/// ("turned items", not "lists/turned items.csv"; call 178).
pub fn titled(file: &str) -> String {
    let name = Path::new(file).file_name().and_then(|n| n.to_str()).unwrap_or(file);
    match name.rsplit_once('.') {
        Some((stem, end)) if !stem.is_empty() && end.len() <= 5 && end.chars().all(|c| c.is_ascii_alphanumeric()) => stem.to_string(),
        _ => name.to_string(),
    }
}

/// What a table brought in did, said in full. Counted in what it holds (prices, items),
/// not in rows (call 178).
fn said(file: &str, what: Bring, lost: &[String], b: &Brought) -> (String, bool) {
    let (one, many) = match what {
        Bring::Prices => ("price", "prices"),
        Bring::Assemblies => ("assembly", "assemblies"),
        Bring::Items => ("item", "items"),
        _ => ("line", "lines"),
    };
    let mut out = format!("Brought in the {} \u{201c}{}\u{201d}: {} read, {} set on the job.", what.said(), titled(file), count(b.rows, one, many), b.done.len());
    for d in b.done.iter().take(6) {
        out.push_str(&format!("\n{d}"));
    }
    if b.done.len() > 6 {
        out.push_str(&format!("\nand {} more", b.done.len() - 6));
    }
    if b.same > 0 {
        out.push_str(&format!("\n{} the same as the job already has", count(b.same, &format!("{one} is"), &format!("{} are", many))));
    }
    for (row, why) in &b.not_used {
        out.push_str(&format!("\nNot used, row {row}: {why}"));
    }
    for note in &b.notes {
        out.push_str(&format!("\nNote: {note}"));
    }
    if b.above > 0 {
        out.push_str(&format!("\nNot read: {} above the headings", count(b.above, "line", "lines")));
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
        // What follows the last comma is the supplier, unless it is the end of the
        // file's own name (a name with a comma in it). A supplier's name can have a full
        // stop: "Ridge Supply Co." (R21-27: any full stop made it part of the file).
        (Bring::Prices, Some((file, who))) if !who.trim().is_empty() && !who.trim().rsplit_once('.').is_some_and(|(_, end)| matches!(end.to_ascii_lowercase().as_str(), "csv" | "xlsx" | "xlsm" | "xls")) => (file.trim(), Some(who.trim().to_string())),
        _ => (typed.trim(), None),
    };
    // A maker's sheet can say what kind of thing it is for, after a comma.
    let (name, kind) = match (what, name.rsplit_once(',')) {
        (Bring::Sheet, Some((file, word))) => match word.trim().to_lowercase().as_str() {
            "equipment" => (file.trim(), Some(Kind::Equipment)),
            "fixture" => (file.trim(), Some(Kind::Fixture)),
            "furnishing" => (file.trim(), Some(Kind::Furnishing)),
            _ => (name, None),
        },
        _ => (name, None),
    };
    if what == Bring::Scan {
        scan(doc, typed);
        return;
    }
    if name.is_empty() {
        crate::tools::cant(doc, "bring it in", "no file was named. Type the file's name, or drop the file on this window.");
        doc.touch();
        return;
    }
    let path = found(doc, name);
    let file = path.file_name().and_then(|f| f.to_str()).unwrap_or(name).to_string();
    let source = format!("{file}, brought in on screen");
    let how = Entered { status: Status::Assumed, source: &source };
    let done: Result<(String, bool), String> = (|| {
        let bytes = std::fs::read(&path).map_err(|e| format!("{file} {}", crate::plans::why_not_opened(&e)))?;
        match what {
            Bring::Sheet => {
                let words = plumbline_plans::words(&bytes, 1).map_err(|why| format!("{file}: {why}"))?;
                if words.is_empty() {
                    return Err(format!("{file}: there are no words on its first page to read. A scanned sheet is a picture of words: place the thing with Fixture or Equipment and type its sizes"));
                }
                let lines: Vec<(String, f64)> = words.into_iter().map(|w| (w.text, w.size)).collect();
                let sheet = plumbline_world::spec::read(&lines);
                let called = sheet.name.as_ref().map(|f| f.value.clone()).unwrap_or_else(|| file.clone());
                let kind = match kind {
                    Some(kind) => kind,
                    None => plumbline_world::spec::kind_said(&called).map(|k| k.0).ok_or_else(|| format!("{file}: \u{201c}{called}\u{201d} does not say what kind of thing it is. Type its kind after the file's name: {file}, fixture (or equipment, or furnishing)"))?,
                };
                let made = plumbline_world::spec::item_of(&sheet, &file, 1, kind);
                plumbline_world::editing::item(&mut doc.job, &called, made.item.clone()).map_err(|why| format!("{file}: {why}"))?;
                let mut out = format!("Made an item from {file}: {called}, {}.", kind.a());
                if !made.read.is_empty() {
                    out.push_str(&format!("\nRead off the sheet, each Assumed until it is checked: {}.", made.read.iter().map(|(what, value, ..)| format!("{what} {value}")).collect::<Vec<_>>().join("; ")));
                }
                if !made.missing.is_empty() {
                    out.push_str(&format!("\nNot on the sheet, and Unknown until entered: {}.", made.missing.join(", ")));
                }
                // The sheet does not say which trade's it is (call 197, point 15), as New item does not either.
                if made.item.division.is_none() {
                    out.push_str("\nIts trade is not said yet: pick it under Trade, on the right, with the item selected.");
                }
                out.push_str("\nIt is in the tool chest, on the left: click it there, then click where one stands.");
                Ok((out, !made.missing.is_empty()))
            }
            Bring::Model => {
                let b = plumbline_ifc::import(&mut doc.job, &bytes, &file, how)?;
                let mut out = format!("Brought in the designer's model \u{201c}{}\u{201d} ({}): {}.", titled(&file), b.schema, b.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { what.strip_suffix('s').unwrap_or(what) } else { what })).collect::<Vec<_>>().join(", "));
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
                    Bring::Items => tables::items_in(&mut doc.job, &sheet.rows, Some(&sheet.numbers)),
                    _ => {
                        let called = path.file_stem().and_then(|s| s.to_str()).unwrap_or("prior estimate").to_string();
                        tables::prior_in(&mut doc.job, &called, &sheet.rows, Some(&sheet.numbers), how).map(|(_, b)| b)
                    }
                }
                .map_err(|why| format!("{file}: {why}"))?;
                Ok(said(&file, what, &sheet.lost, &b))
            }
        }
    })();
    match done {
        Ok((out, to_see)) => {
            doc.job.derive();
            // Only a model brings a building to be framed: a price list, an assembly
            // list or an estimate leaves the view where it is (R21-21).
            if what == Bring::Model {
                doc.framed = false;
            }
            doc.edit("brought something in");
            // What was not used or not read has to be seen: it is said over the model.
            doc.say(if to_see { Tone::Warned } else { Tone::Done }, out);
        }
        Err(why) => crate::tools::cant(doc, &format!("bring in the {}", what.said()), why),
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
    } else if lower.ends_with(".ply") {
        bring(doc, Bring::Scan, file);
    } else if lower.ends_with(".csv") || lower.ends_with(".xlsx") || lower.ends_with(".xlsm") || lower.ends_with(".xls") {
        doc.dropped = Some(file.to_string());
        doc.say(Tone::Ask, format!("{} is a table. Say what it is: click Price list, Assemblies, Prior estimate or Tool chest file, under Bring in on the left.", Path::new(file).file_name().and_then(|f| f.to_str()).unwrap_or(file)));
        doc.touch();
    } else {
        crate::plans::add(doc, file);
    }
}

/// The starter assemblies the program ships with, brought into the job: whole assemblies
/// for a house's walls, floors, rooms, roof, openings and runs, with a line in every one
/// of the sixteen divisions and no price on any of them.
pub fn starter(doc: &mut Doc) {
    let done = plumbline_sheets::read(tables::STARTER_LIBRARY.as_bytes(), None).and_then(|sheet| tables::assemblies_in(&mut doc.job, &sheet.rows, Some(&sheet.numbers)));
    match done {
        Ok(b) if b.done.is_empty() => doc.say(Tone::Done, "The starter assemblies are already in this job. Click a wall, a floor, a room or a roof plane, then Assembly."),
        Ok(b) => {
            doc.job.derive();
            doc.edit("brought in the starter assemblies");
            doc.say(Tone::Done, format!("Brought in {} starter assemblies: walls, floors, rooms, roofs, doors, windows and runs, with no prices. Click one of those on the model, then Assembly.", b.done.len()));
        }
        Err(why) => crate::tools::cant(doc, "bring in the starter assemblies", why),
    }
    doc.touch();
}

/// The item library the program ships with, brought into the job: a few things of each
/// trade that has any, at typical sizes, every size Assumed and saying so.
pub fn starter_items(doc: &mut Doc) {
    let done = plumbline_sheets::read(tables::STARTER_ITEMS.as_bytes(), None).and_then(|sheet| tables::items_in_missing(&mut doc.job, &sheet.rows, Some(&sheet.numbers)));
    // An item the job already has is left as it is: what was changed here is kept.
    let left = |kept: &[String]| match kept.len() {
        0 => String::new(),
        1 => format!(" {} is in the job already and was left as it is here.", kept[0]),
        n => format!(" {n} are in the job already and were left as they are here: what was changed on them is kept."),
    };
    match done {
        Ok((b, _)) if b.done.is_empty() => doc.say(Tone::Done, "The starter tool chest is already in this job, and each item was left as it is here. They are under Tool chest, on the left: click one, then click where one stands."),
        Ok((b, kept)) => {
            doc.job.derive();
            doc.edit("brought in the starter tool chest");
            doc.say(Tone::Done, format!("Brought in {} starter items, trade by trade, at typical sizes: each size is Assumed until the product is chosen.{} They are in the tool chest, on the left: click one, then click where one stands; Change opens it to be changed.", b.done.len(), left(&kept)));
        }
        Err(why) => crate::tools::cant(doc, "bring in the starter tool chest", why),
    }
    doc.touch();
}

/// Write something out of the job, from the window: beside the job, under the job's
/// own name with an ending that says what it is. The same writers the command line
/// uses. Nothing is written over a job.
pub fn write_out(doc: &mut Doc, what: crate::model::WriteOut) {
    use crate::model::WriteOut;
    use plumbline_world::export::{self, Lens};
    let stem = doc.path.file_stem().and_then(|s| s.to_str()).unwrap_or("job").to_string();
    let name = format!("{stem}{}", what.ending());
    let shown = format!("\u{201c}{}\u{201d}", titled(&name));
    let to = doc.path.with_file_name(&name);
    // Written again, it takes the place of the one written before, and says so (J19-38).
    let again = to.exists();
    // A job with nothing priced writes headings and nothing under them, and says so
    // (J19-40: it wrote the file without a word that nothing was in it).
    let bare = !matches!(what, WriteOut::Model | WriteOut::Items | WriteOut::Field) && plumbline_world::view::estimate(&mut doc.job).lines.is_empty();
    let done: Result<String, String> = (|| {
        if std::fs::read_to_string(&to).ok().is_some_and(|text| plumbline_world::file::load(&text).is_ok()) {
            return Err(format!("{shown} is a job: what is written out is not written over a job"));
        }
        let wrote = |bytes: &[u8]| std::fs::write(&to, bytes).map_err(|e| format!("{shown} {}", crate::plans::why_not_opened(&e)));
        match what {
            WriteOut::Model => {
                let written = plumbline_ifc::write::export(&mut doc.job, &name, &plumbline_world::job::now())?;
                wrote(written.text.as_bytes())?;
                let one = |what: &str| what.strip_prefix("pieces ").map(|rest| format!("piece {rest}")).unwrap_or_else(|| what.strip_suffix('s').unwrap_or(what).to_string());
                let mut out = format!("Wrote {shown}, beside the job: the building as a BIM model (IFC4, lengths in feet). In it: {}.", written.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { one(what) } else { what.clone() })).collect::<Vec<_>>().join(", "));
                if !written.not_written.is_empty() {
                    out.push_str(&format!("\nNot in it: {}.", written.not_written.iter().map(|(what, n, why)| format!("{n} x {what} ({why})")).collect::<Vec<_>>().join("; ")));
                }
                Ok(out)
            }
            // What is to verify in the field, as a sheet to print: a box to tick, the
            // dollars riding, where (level and room), what, what to do, and a blank
            // for what was found (call 117: the list could not leave the window).
            WriteOut::Field => {
                let rows = plumbline_world::vif::field_list(&mut doc.job);
                let n = rows.len() - 1;
                let tab = plumbline_sheets::Tab { name: "Field list".into(), rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { plumbline_sheets::Cell::Empty } else { plumbline_sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 1, widths: vec![6.0, 14.0, 24.0, 44.0, 52.0, 30.0] };
                wrote(&plumbline_sheets::xlsx(&[tab])?)?;
                Ok(if n == 0 { format!("Wrote {shown}, beside the job: nothing is to verify in the field, so it has its headings and no rows.") } else { format!("Wrote {shown}, beside the job: {n} to verify in the field, the most dollars first, each with where it is, what to do, a box to tick and a blank for what was found. Print it, take it to the job, and type what was measured back in here.") })
            }
            WriteOut::Items => {
                let rows = tables::items_out(&mut doc.job);
                wrote(plumbline_sheets::csv(&rows).as_bytes())?;
                Ok(format!("Wrote {shown}, beside the job: the job's tool chest, {} under their headings. Open it, change a size or add an item, and bring it back in with Tool chest file, under Bring in.", count(rows.len().saturating_sub(1), "item", "items")))
            }
            WriteOut::Rows => {
                let sheets = export::estimate_out(&mut doc.job, false);
                let rows: Vec<Vec<String>> = sheets[0].rows[sheets[0].head..].iter().map(|r| r.iter().map(export::Out::said).collect()).collect();
                wrote(plumbline_sheets::csv(&rows).as_bytes())?;
                Ok(format!("Wrote {shown}, beside the job: the estimate's lines as values, {} under their headings.", count(rows.len().saturating_sub(1), "line", "lines")))
            }
            _ => {
                let (sheets, said) = match what {
                    WriteOut::Supplier => (export::lens_out(&mut doc.job, Lens::Supplier), "the materials to quote, with none of our prices"),
                    WriteOut::Subcontractor => (export::lens_out(&mut doc.job, Lens::Subcontractor), "the subcontract scope, with none of our prices"),
                    WriteOut::Pm => (export::lens_out(&mut doc.job, Lens::Pm), "the lines, the budget and what is to verify, with no overhead, profit or price"),
                    _ => (export::estimate_out(&mut doc.job, true), "the estimate, its amounts, recap and price as formulas"),
                };
                let (bytes, formulas) = plumbline_sheets::workbook(&sheets)?;
                wrote(&bytes)?;
                Ok(format!("Wrote {shown}, beside the job: {said}. {}: {}{}.", count(sheets.len(), "sheet", "sheets"), sheets.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "), if formulas > 0 { format!("; {formulas} of its figures are formulas") } else { String::new() }))
            }
        }
    })();
    match done {
        Ok(mut out) => {
            if bare {
                out.push_str(" The job has no lines yet: the file has its headings and nothing under them.");
            }
            if again {
                out.push_str(&format!(" It takes the place of the {shown} that was there."));
            }
            doc.say(Tone::Done, out)
        }
        Err(why) => crate::tools::cant(doc, &format!("write out the {}", what.said()), why),
    }
    doc.touch();
}

/// A scan of the house as it stands, laid under the level on show as a sheet to trace
/// over: to scale, in the scan's own place, Assumed. It is the command line's own
/// (`plumbline import scan`): the same reader, the same levelling by its floor, the same
/// cut 4 ft above the floor at 24 dots to the foot, the same sheet, scale and place.
/// `typed` is the file's name. What the file does not say is what the list's two choices
/// say (meters or feet, `Doc::scan_feet`); which way is up is read from the scan. A comma
/// and a word after the name (`feet`, `y up`, `z up`) still say it for whoever types.
fn scan(doc: &mut Doc, typed: &str) {
    use plumbline_plans::scan::{Units, Up};
    let (name, said_so) = typed.rsplit_once(',').map_or((typed.trim(), String::new()), |(n, rest)| (n.trim(), rest.to_lowercase()));
    let units = if said_so.contains("feet") || said_so.contains("ft") || (doc.scan_feet && !said_so.contains("meters") && !said_so.contains("metres")) { Units::Feet } else { Units::Metres };
    let up = if said_so.contains("y up") {
        Some(Up::Y)
    } else if said_so.contains("z up") {
        Some(Up::Z)
    } else {
        None
    };
    scan_file(doc, name, units, up, false);
}

/// The button "It is lying on its side: stand it up" (call 161): the step that brought the
/// scan in is taken back, and the same file is brought in again with the other way up.
pub fn stand_it_up(doc: &mut Doc) {
    use plumbline_plans::scan::{Units, Up};
    let Some((file, was_z, step)) = doc.scan_side.clone() else { return };
    if doc.draw.edits != step {
        doc.scan_side = None;
        doc.touch();
        return;
    }
    crate::tools::step(doc, false);
    let units = if doc.scan_feet { Units::Feet } else { Units::Metres };
    scan_file(doc, &file, units, Some(if was_z { Up::Y } else { Up::Z }), true);
}

/// What every scan's message ends with (call 169): the size is the first thing a builder
/// can check, and the measure at the head of the list is the one thing to change.
const OTHER_MEASURE: &str = "If that is not the size of the house, choose the other measure at the head of the list and bring it in again.";

fn scan_file(doc: &mut Doc, name: &str, units: plumbline_plans::scan::Units, up: Option<plumbline_plans::scan::Up>, stood: bool) {
    use plumbline_plans::scan::Said;
    doc.scan_side = None;
    if name.is_empty() {
        crate::tools::cant(doc, "bring in the scan", "no file was named. Pick the scan from the list, or drop the file on this window.");
        doc.touch();
        return;
    }
    // With the whole house on show, the sheet goes under its lowest level, as a plan does.
    if doc.show.storey.is_none() {
        doc.show.storey = plumbline_world::building::storey_totals(&mut doc.job).into_iter().min_by(|a, b| a.elevation.unwrap_or(f64::MAX).total_cmp(&b.elevation.unwrap_or(f64::MAX))).map(|s| s.path);
    }
    let Some(level) = doc.show.storey.clone() else {
        crate::tools::cant(doc, "bring in the scan", "this job has no level for it to be a scan of. Add a level first.");
        doc.touch();
        return;
    };
    let path = found(doc, name);
    let file = path.file_name().and_then(|f| f.to_str()).unwrap_or(name).to_string();
    let mut laid: Option<String> = None;
    let mut up_taken: Option<bool> = None;
    let mut doubtful = false;
    let mut wrong = false;
    let done = (|| -> Result<(String, bool), String> {
        let bytes = std::fs::read(&path).map_err(|e| format!("{file} {}", crate::plans::why_not_opened(&e)))?;
        let called = path.file_stem().and_then(|s| s.to_str()).unwrap_or("scan").to_string();
        let cut_at = 4.0;
        let folder = crate::plans::folder_of(&doc.path);
        // The one way a scan is brought in, here and at the command line (call 153).
        let brought = plumbline_plans::scan::bring_in(&bytes, &file, &called, &Said { units, up, cut_at }, &mut |s| {
            plumbline_world::plans::lay_scan_sheet(&mut doc.job, &level, &folder, s.called, s.taken, s.cut_at, s.png, s.checksum, s.size, s.cut.size, s.cut.dots, s.cut.corner)
        })?;
        laid = Some(brought.laid.clone());
        up_taken = Some(brought.taken.contains("z up"));
        let (why_up, cut, floor, ceiling, leaned) = (brought.why_up, &brought.cut, brought.floor, brought.ceiling, brought.leaned);
        let ft_in = |x: f64| plumbline_world::units::feet_as_ft_in(x, 16);
        let figure = |n: usize| plumbline_world::view::figure(n as f64, 0);
        let about = |x: f64| plumbline_world::units::feet_as_ft_in((x * 12.0).round() / 12.0, 16);
        // A builder's words: the way the scan was measured, and that which way is up was
        // read from it. Nobody is asked for an axis (call 161).
        // The card is four lines: what was brought in, its size, what to do if the size is
        // wrong, and (when it is in doubt) which way is up; the rest is under More (call 178).
        let mut head = format!("Brought in the scan {called:?}: {} points, measured in {}, Assumed.\n", figure(brought.points), if units == plumbline_plans::scan::Units::Feet { "feet" } else { "meters" });
        let _ = std::fmt::Write::write_fmt(&mut head, format_args!("On plan, where it is cut, it is about {} by {}.", about(cut.extent[0]), about(cut.extent[1])));
        let mut out = String::new();
        let mut to_see = false;
        let told = why_up == "as you said";
        if !told {
            if why_up == "the way the house is shortest" {
                out.push_str("\nWhich way is up was read from the scan.");
            } else {
                head.push_str("\nWhich way is up is not clear from the scan. If the house looks to be lying on its side, press the button below.");
                doubtful = true;
                to_see = true;
            }
        }
        // A house over 150 ft long, or a floor to ceiling under 6 ft or over 15 ft, is
        // not likely to be the house: the message is a warning and says so first.
        let long = cut.extent[0].max(cut.extent[1]);
        let storey = match (floor, ceiling) { (Some(f), Some(c)) => Some(c - f), _ => None };
        if long > 150.0 || long < 6.0 || storey.is_some_and(|h| !(6.0..=15.0).contains(&h)) {
            to_see = true;
            doubtful = !told;
            wrong = true;
        }
        if let Some(lean) = leaned.filter(|l| *l * 120.0 >= 0.125) {
            out.push_str(&format!("\nIt leaned {} in 10 ft and was levelled by its own floor.", plumbline_world::units::show_length(&plumbline_world::units::Len::One((lean * 120.0 * 16.0).round() / 16.0), "in")));
        }
        match (floor, ceiling) {
            (Some(f), Some(c)) => {
                out.push_str(&format!("\nFloor to ceiling reads about {} on the scan. It is not given to the level: measure the level's height if you take it.", about(c - f)));
                if !told && !(6.0..=20.0).contains(&(c - f)) {
                    doubtful = true;
                }
            }
            (Some(_), None) => out.push_str("\nIts floor shows and no ceiling does: no height read."),
            _ => out.push_str(&format!("\nNo floor shows in it: it is cut {} above its lowest point.", ft_in(cut_at))),
        }
        // The sheet's size in dots and the file's column names are the command line's to
        // say; the window says what a builder reads (call 163).
        out.push_str(&format!("\nIt is cut {} above the floor, as a plan is.", ft_in(cut_at)));
        if cut.strays > 0 {
            out.push_str(&format!("\nLeft off the sheet: {} stray points at that height, each alone in its square foot of plan.", figure(cut.strays)));
        }
        out.push_str("\nIt is a drawing under the level on show, to scale and in the scan's own place: trace over it as over any plan.");
        if !brought.lost.is_empty() {
            out.push_str("\nOnly where its points are is kept: not its colors, and not the surface drawn over them.");
        }
        // The sentence about the other measure is said once: first where the size is not
        // likely the house's, and else after the size.
        out = if wrong { format!("{OTHER_MEASURE}\n{head}{out}") } else { format!("{head}\n{OTHER_MEASURE}{out}") };
        if stood {
            out = format!("Stood up. {out}");
        }
        Ok((out, to_see))
    })();
    match done {
        Ok((out, to_see)) => {
            doc.job.derive();
            if !doc.plan {
                doc.go = Some((true, false));
            }
            doc.framed = false;
            doc.frame_on = laid.and_then(|path| crate::plans::corners_of(&mut doc.job, &path));
            doc.edit("brought in a scan");
            // The button is there only when the way up was guessed and may be wrong, and
            // only while this is the last thing done.
            if doubtful && up.is_none() {
                doc.scan_side = up_taken.map(|z| (name.to_string(), z, doc.draw.edits));
            }
            doc.say(if to_see { Tone::Warned } else { Tone::Done }, out);
        }
        Err(why) if why.contains("there is already something called") => {
            doc.say(Tone::Warned, "It is already in, under the plan. Undo takes it out.");
        }
        Err(why) if why.contains("no point of it is at that height") => crate::tools::cant(doc, "bring in the scan", format!("{why}. {OTHER_MEASURE}")),
        Err(why) => crate::tools::cant(doc, "bring in the scan", why),
    }
    doc.redraw();
}
