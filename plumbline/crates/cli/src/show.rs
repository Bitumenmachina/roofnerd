use super::*;

pub(super) fn show(job: &mut plumbline_world::Job, report: &file::LoadReport) -> String {
    let mut out = String::new();
    let elements = view::elements(job);
    let _ = writeln!(out, "{}", job.header.id);
    let _ = writeln!(out, "flags:  ~ Assumed    ? Unknown    (no mark) Known        = worked out from other values");
    let _ = writeln!(out);
    for el in &elements {
        let pad = "  ".repeat(el.depth);
        let _ = writeln!(out, "{pad}{}  {}  [{}]", el.path, el.name, el.kind.said());
        for n in &el.notes {
            let _ = writeln!(out, "{pad}      {n}");
        }
        for r in &el.rows {
            let eq = if r.derived { "=" } else { " " };
            // An assembly less a line reads so: "Kitchen without its line "Cabinets"" (call 167).
            let less = if r.label == "condition" && !r.derived && !r.note.is_empty() { format!(" {}", r.note) } else { String::new() };
            let _ = writeln!(out, "{pad}    {eq} {} {:<16} {}{less}", r.status.flag(), view::words(&r.label), r.text);
        }
    }
    let (known, assumed, unknown) = view::tally(&elements);
    let apart = view::tallies(job, &elements);
    let _ = writeln!(out);
    let _ = writeln!(out, "{known} Known, {assumed} Assumed, {unknown} Unknown");
    // The building and the tool chest apart, whose two make that one (call 209 point 8).
    let _ = writeln!(out, "  in the building: {} known, {} assumed, {} unknown", apart.building.0, apart.building.1, apart.building.2);
    let _ = writeln!(out, "  in the tool chest: {} known, {} assumed, {} unknown", apart.chest.0, apart.chest.1, apart.chest.2);
    let flagged: Vec<_> = elements.iter().flat_map(|e| e.rows.iter().map(move |r| (e, r))).filter(|(_, r)| r.status != Status::Known).collect();
    if !flagged.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "Flagged values:");
        for (el, r) in flagged {
            let why = if r.derived {
                if r.note.is_empty() { "follows from a flagged input".to_string() } else { r.note.clone() }
            } else {
                r.basis.clone()
            };
            let _ = writeln!(out, "  {} {} {} {}: {} ({})", r.status.flag(), el.path, el.name, view::words(&r.label), r.text, why);
        }
    }
    if !report.stale.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "WARNING: the file's figures did not agree with what is drawn for: {}. The figures above are worked out again from what is drawn.", report.stale.join(", "));
    }
    out
}

/// One thing as `show` lists it, with its rows; a wall also says what stands on it and what
/// it stands on (call 176).
pub(super) fn show_one(job: &mut plumbline_world::Job, thing: &str) -> Result<String, String> {
    let elements = view::elements(job);
    let el = elements.iter().find(|e| e.path == thing).ok_or_else(|| format!("{thing} is not in the job"))?;
    let mut out = String::new();
    let _ = writeln!(out, "{}  {}  [{}]", el.path, el.name, el.kind.said());
    for n in &el.notes {
        let _ = writeln!(out, "      {n}");
    }
    for r in &el.rows {
        let eq = if r.derived { "=" } else { " " };
        let less = if r.label == "condition" && !r.derived && !r.note.is_empty() { format!(" {}", r.note) } else { String::new() };
        let _ = writeln!(out, "    {eq} {} {:<16} {}{less}", r.status.flag(), view::words(&r.label), r.text);
    }
    if el.kind == plumbline_world::Kind::Wall {
        out.push_str(&carries_said(&plumbline_world::carries::of(job, thing)?));
    }
    Ok(out)
}

/// A wall's two figures, as a line each, and what they are.
pub(super) fn carries_said(c: &plumbline_world::carries::Carry) -> String {
    let mut out = String::new();
    let mut block = |label: &str, lines: Vec<String>| {
        for (i, l) in lines.iter().enumerate() {
            let _ = writeln!(out, "      {:<11} {l}", if i == 0 { label } else { "" });
        }
    };
    block("carries:", c.carries_said());
    block("stands on:", c.stands_on_said());
    let _ = writeln!(out, "      {:<11} {}", "", c.status_said());
    let _ = writeln!(out, "      {:<11} {}", "", plumbline_world::carries::SAID);
    out
}

/// `carries <file> [<a level or a wall>]`.
pub(super) fn carries_verb(args: &[String]) -> Result<(), String> {
    if args.len() > 3 {
        return Err(format!("carries takes a file and, if you like, one level or one wall; {:?} would be ignored", args[3]));
    }
    let path = args.get(1).ok_or_else(usage)?;
    let (mut job, _) = read(path)?;
    let mut all = plumbline_world::carries::all(&mut job);
    let mut out = String::new();
    let only = args.get(2);
    if let Some(only) = only {
        let by_path = all.iter().any(|c| &c.path == only);
        let by_level = all.iter().any(|c| &c.storey == only || c.storey_name.eq_ignore_ascii_case(only));
        if !by_path && !by_level {
            return Err(format!("{only} is not a wall or a level of this job that has walls"));
        }
        all.retain(|c| &c.path == only || &c.storey == only || c.storey_name.eq_ignore_ascii_case(only));
    }
    let mut at = String::new();
    for c in &all {
        if c.storey != at {
            at = c.storey.clone();
            let _ = writeln!(out, "Level {} [{}]", c.storey_name, c.storey);
        }
        let flag = match c.bearing {
            Some((b, s)) => format!("said {} ({})", b.said(), plumbline_world::structure::word(s)),
            None => "bearing not said".to_string(),
        };
        let _ = writeln!(out, "  {} [{}], {} LF, {flag}", c.name, c.path, view::figure(c.length, 2));
        out.push_str(&carries_said(c));
    }
    if all.is_empty() {
        let _ = writeln!(out, "No wall is drawn.");
    }
    if only.is_none() {
        let u = plumbline_world::carries::unsaid(&mut job);
        let _ = writeln!(out, "Walls whose bearing is not said, with something standing on them: {} in the job", u.job);
        for (_, name, n) in &u.levels {
            let _ = writeln!(out, "  {name}: {n}");
        }
    }
    let _ = std::io::stdout().write_all(out.as_bytes());
    Ok(())
}

/// The lines the job's planes imply, set against the lines that were entered. It reads
/// the job and changes nothing.
pub(super) fn lines(job: &mut plumbline_world::Job) -> String {
    let mut out = String::new();
    let (planes, left_out) = roof::solid_planes(job);
    let found = roof::lines_of(&planes);
    let cmp = roof::compare(job, &found);
    let point = |p: [f64; 2]| format!("({}, {})", p[0], p[1]);
    let _ = writeln!(out, "{} planes placed; where they meet and end they make {} eaves, rakes, hips, valleys and ridges (coordinates in feet):", planes.len(), found.lines.len());
    for l in &found.lines {
        let _ = writeln!(out, "  {} {:<7} {} to {}  on {}", l.status.flag(), capital(l.kind.said()), point(l.ends[0]), point(l.ends[1]), l.planes.join(" and "));
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "{} of those entered agree with the planes.", cmp.agree.len());
    if !cmp.entered_only.is_empty() {
        let _ = writeln!(out, "Entered, but not what the planes imply:");
        for e in &cmp.entered_only {
            let _ = writeln!(out, "  {e}");
        }
    }
    if !cmp.implied_only.is_empty() {
        let _ = writeln!(out, "Implied by the planes, but not entered:");
        for l in &cmp.implied_only {
            let _ = writeln!(out, "  {} {} to {} on {}", capital(l.kind.said()), point(l.ends[0]), point(l.ends[1]), l.planes.join(" and "));
        }
    }
    if !found.problems.is_empty() {
        let _ = writeln!(out, "How the planes meet, where it is not a plain ridge, hip or valley:");
        for p in &found.problems {
            let _ = writeln!(out, "  {p}");
        }
    }
    if !left_out.is_empty() {
        let _ = writeln!(out, "Planes left out:");
        for p in &left_out {
            let _ = writeln!(out, "  {p}");
        }
    }
    out
}

/// "a water closet", "an outdoor unit" (call 197, point 7): the article a person would say.
pub(super) fn with_article(word: &str) -> String {
    // A single letter is said by its name: an x, an f, a b (call 206 point 12).
    let word_t = word.trim();
    let an = if word_t.chars().count() == 1 { word_t.chars().next().is_some_and(|c| "aefhilmnorsxAEFHILMNORSX".contains(c)) } else { word_t.chars().next().is_some_and(|c| "aeiouAEIOU".contains(c)) };
    format!("{} {word}", if an { "an" } else { "a" })
}

/// Is what was typed as a price $0.00: "0", "$0", "$0.00/EA".
pub(super) fn zero_price(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim().trim_start_matches('$').split('/').next().is_some_and(|n| n.trim().replace(',', "").parse::<f64>().is_ok_and(|x| x == 0.0)))
}

/// An item's shape worked at its own sizes (call 198): each block between its figures, in
/// inches, across from the middle, back from the back (the side that goes to a wall), up
/// from the foot. A shape is a drawing and carries no Known or Assumed; the sizes it is worked
/// from do, and are said.
/// The six figures of a block as `plumbline shape ... set` names them (lower case).
pub(super) const FIGURES_SAID: [&str; 6] = ["across from", "across to", "back from", "back to", "up from", "up to"];

/// An item of the job by its path or its name, however it is capitalised.
pub(super) fn item_path_of(job: &mut plumbline_world::Job, typed: &str) -> Result<String, String> {
    use plumbline_world::{Item, Node};
    let wanted = plumbline_world::check::plain_name(typed);
    let mut q = job.world.query::<(&Node, &Item)>();
    q.iter(&job.world).find(|(n, _)| n.path == typed || plumbline_world::check::plain_name(&n.name) == wanted).map(|(n, _)| n.path.clone()).ok_or_else(|| format!("{typed:?} is not an item of the job; `plumbline items <file>` lists them"))
}

/// The job's own shapes, or one of them with its blocks and who is drawn as it.
pub(super) fn shapes_said(job: &mut plumbline_world::Job, name: Option<&str>) -> Result<String, String> {
    use plumbline_world::building::shape;
    use plumbline_world::{Item, Node, Placed};
    let drawn_as = |job: &mut plumbline_world::Job, shape: &str| -> (Vec<String>, usize) {
        let items: Vec<(String, String)> = {
            let mut q = job.world.query::<(&Node, &Item)>();
            q.iter(&job.world).filter(|(_, it)| it.form.as_deref().is_some_and(|f| f.trim().eq_ignore_ascii_case(shape))).map(|(n, _)| (n.path.clone(), n.name.clone())).collect()
        };
        let mut q = job.world.query::<&Placed>();
        let set_down = q.iter(&job.world).filter(|p| p.item.as_ref().is_some_and(|i| items.iter().any(|(path, _)| path == i))).count();
        (items.into_iter().map(|(_, n)| n).collect(), set_down)
    };
    let mut out = String::new();
    match name {
        None => {
            let names: Vec<(String, usize)> = job.shapes.iter().map(|s| (s.name.clone(), s.blocks.len())).collect();
            if names.is_empty() {
                out.push_str("The job has no shape of its own: every item is drawn as a box or as one of the program's shapes. `plumbline shape <file> keep \"<item>\" \"<new name>\"` makes one.\n");
            }
            for (n, blocks) in names {
                let (items, set_down) = drawn_as(job, &n);
                out.push_str(&format!("{n}: {}; {} drawn as it, {} set down from them\n", view::so_many(blocks, "block"), view::so_many(items.len(), "item"), view::so_many(set_down, "thing")));
            }
        }
        Some(name) => {
            let Some(found) = shape::find(&job.shapes, name) else { return Err(format!("{name:?} is not a shape the job has or the program draws; `plumbline shape <file>` lists the job's own")) };
            let (own, name) = (job.shapes.iter().any(|s| s.name == found.name), found.name.clone());
            let blocks = found.blocks.clone();
            let (items, set_down) = drawn_as(job, &name);
            out.push_str(&format!("{name} ({}): {}\n", if own { "the job's own" } else { "a starter shape, which is not changed" }, view::so_many(blocks.len(), "block")));
            for (i, b) in blocks.iter().enumerate() {
                let figures = b.figures();
                out.push_str(&format!("  block {}, {}{}: across {} to {}; back {} to {}; up {} to {}{}\n", i + 1, b.made.word(), if b.round { ", round" } else { "" }, figures[0], figures[1], figures[2], figures[3], figures[4], figures[5], b.when.as_ref().map(|w| format!("; there when {w}")).unwrap_or_default()));
            }
            out.push_str(&format!("  {} drawn as it{}; {} set down from them\n", view::so_many(items.len(), "item"), if items.is_empty() { String::new() } else { format!(" ({})", items.join(", ")) }, view::so_many(set_down, "thing")));
        }
    }
    Ok(out)
}

pub(super) fn item_blocks_said(job: &mut plumbline_world::Job, item: &str) -> Result<String, String> {
    use plumbline_world::building::{forms, shape};
    use plumbline_world::{Item, Node};
    let wanted = plumbline_world::check::plain_name(item);
    let (name, it) = {
        let w = &job.world;
        let found = job.order.iter().find_map(|e| match (w.get::<Node>(*e), w.get::<Item>(*e)) {
            (Some(n), Some(it)) if plumbline_world::check::plain_name(&n.name) == wanted || n.path == item => Some((n.name.clone(), it.clone())),
            _ => None,
        });
        found.ok_or_else(|| format!("{item:?} is not an item of the job; `plumbline items <file>` lists them"))?
    };
    let mut out = format!("{name}\n");
    let (size, status, source) = match it.size_ft() {
        Ok(s) => s,
        Err(why) => {
            out.push_str(&format!("  {why}: no blocks can be worked for it\n"));
            return Ok(out);
        }
    };
    let inches = |ft: f64| {
        let t = format!("{:.3}", ft * 12.0 + 0.0);
        let t = t.trim_end_matches('0').trim_end_matches('.');
        format!("{} in", if t == "-0" { "0" } else { t })
    };
    out.push_str(&format!("  sizes: {} wide, {} deep, {} high; {}{}\n", inches(size[0]), inches(size[1]), inches(size[2]), status.flag().trim_end(), source));
    // What it is drawn as: what it says, else what its name says, else its box.
    let (as_word, blocks): (String, Option<Result<Vec<forms::Block>, String>>) = match it.form.as_deref() {
        Some(said) if said.eq_ignore_ascii_case(forms::BOX) => ("a box".into(), None),
        Some(said) => match shape::find(&job.shapes, said) {
            Some(shape) => (with_article(said), Some(shape.worked(size))),
            None => match forms::Form::named(said) {
                Some(form) => (with_article(said), Some(Ok(forms::blocks(form, size)))),
                None => return Err(format!("{said:?} is not a shape the job has")),
            },
        },
        None => match forms::form_of(&name) {
            Some(form) => (format!("{}, by its name", with_article(form.name())), Some(Ok(forms::blocks(form, size)))),
            None => ("a box, its name saying nothing else".into(), None),
        },
    };
    out.push_str(&format!("  drawn as {as_word}\n"));
    match blocks {
        None => out.push_str("  a box is one block, its sizes\n"),
        Some(Err(why)) => out.push_str(&format!("  its shape cannot be worked: {why}\n")),
        Some(Ok(blocks)) => {
            out.push_str(&format!("  its shape is {} (a drawing: nothing is priced by it, a clash tests the item's box, and a block is neither Known nor Assumed; the sizes above are):\n", if blocks.len() == 1 { "1 block".to_string() } else { format!("{} blocks", blocks.len()) }));
            for (n, b) in blocks.iter().enumerate() {
                out.push_str(&format!("    block {}, {}\n", n + 1, plumbline_world::editing::shapes::block_said(b, size)));
            }
        }
    }
    // Where it connects (call 211), each worked at its sizes: across from the middle, back from the back, up from the foot.
    if let Ok(worked) = plumbline_world::editing::item_connections_worked(job, item) {
        if !worked.is_empty() {
            out.push_str(&format!("  it connects at {} (each is on the item or inside it at its sizes; Assumed unless a person measured it on the thing):\n", view::so_many(worked.len(), "place")));
            for (n, w) in worked.iter().enumerate() {
                out.push_str(&format!("    connection {}, {}\n", n + 1, w.said));
            }
        }
    }
    Ok(out)
}

pub(super) fn items_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::building::forms;
    use plumbline_world::{Division, Item, Node, Placed};
    let w = &job.world;
    let mut by_trade: std::collections::BTreeMap<u8, Vec<String>> = std::collections::BTreeMap::new();
    let mut count = 0;
    for e in &job.order {
        let (Some(n), Some(it)) = (w.get::<Node>(*e), w.get::<Item>(*e)) else { continue };
        count += 1;
        let placed = job.order.iter().filter(|p| w.get::<Placed>(**p).is_some_and(|c| c.item.as_deref() == Some(n.path.as_str()))).count();
        let size = |v: &plumbline_world::value::Val<units::Len>| match &v.value {
            Some(len) => format!("{}{}", v.status.flag().trim_end(), units::show_length(len, &v.unit)),
            None => "? unknown".to_string(),
        };
        let drawn = match it.form.as_deref() {
            Some(said) if said.eq_ignore_ascii_case(forms::BOX) => "a box".to_string(),
            Some(said) => with_article(said),
            None => match forms::form_of(&n.name) {
                Some(form) => format!("{}, by its name", with_article(form.name())),
                None => "a box, its name saying nothing else".to_string(),
            },
        };
        let kind = match it.category {
            plumbline_world::Kind::Equipment => "equipment",
            plumbline_world::Kind::Fixture => "fixture",
            _ => "furnishing",
        };
        let priced = it.priced_as.as_ref().map(|a| format!("; priced as {a:?}")).unwrap_or_default();
        // How many places it says it connects at (call 211): said where it has any.
        let connects = w.get::<plumbline_world::ItemConnections>(*e).map(|c| format!("; {}", view::so_many(c.0.len(), "connection"))).unwrap_or_default();
        by_trade.entry(it.division.map(Division::number).unwrap_or(0)).or_default().push(format!("    {} [{}]: {kind}, drawn as {drawn}; {} wide, {} deep, {} high; {} placed{priced}{connects}", n.name, n.path, size(&it.width), size(&it.depth), size(&it.height), placed));
    }
    let mut out = format!("ITEMS: the things that can be placed, by trade ({count})\n");
    if count == 0 {
        out.push_str("  none. Bring in an item library (import items <file> starter), or a maker's sheet (import spec)\n");
    }
    for (number, lines) in by_trade {
        match Division::of_number(number) {
            Some(d) => out.push_str(&format!("  Division {} {}\n", d.number(), d.title())),
            None => out.push_str("  No trade said\n"),
        }
        for line in lines {
            out.push_str(&line);
            out.push('\n');
        }
    }
    // How many of the starter items say where they connect (call 211): the program holds a place for the few a published
    // rough-in gives one for, and the rest have none.
    let starter: Vec<String> = plumbline_world::building::shape::csv_rows(plumbline_world::tables::STARTER_ITEMS).into_iter().skip(1).filter_map(|r| r.into_iter().next()).map(|n| plumbline_world::check::plain_name(&n)).collect();
    let (mut present, mut without) = (0usize, 0usize);
    for e in &job.order {
        let (Some(n), Some(_)) = (job.world.get::<Node>(*e), job.world.get::<Item>(*e)) else { continue };
        if starter.contains(&plumbline_world::check::plain_name(&n.name)) {
            present += 1;
            if job.world.get::<plumbline_world::ItemConnections>(*e).is_none() {
                without += 1;
            }
        }
    }
    if present > 0 {
        out.push_str(&format!("Where they connect: of the job's {}, {without} have none (`plumbline import connections <file> starter` gives each the program holds a published place for)\n", view::so_many(present, "starter item")));
    }
    out
}

pub(super) fn say_solve(report: &solve::Report) {
    say!("  {}; {} eaves, rakes, hips, valleys and ridges round and between them", view::so_many(report.planes, "plane"), report.lines);
    if !report.added.is_empty() {
        say!("  new: {}", report.added.join(", "));
    }
    for moved in &report.renumbered {
        say!("  {moved}");
    }
    for (path, carried) in &report.removed {
        say!("  gone: {path}{}", if *carried { "  (it carried an assembly or cost lines, which went with it)" } else { "" });
    }
    for note in &report.notes {
        say!("  note: {note}");
    }
}
