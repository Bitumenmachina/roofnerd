use super::*;

pub(super) fn takeoff(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::building;
    let said = |sum: &view::Sum| match (sum.value, sum.members) {
        (_, 0) => "none".to_string(),
        (Some(x), _) => format!("{} {}", view::figure(x, if sum.unit == "EA" { 0 } else { 2 }), sum.unit),
        (None, n) => format!("no total, {} of {n} missing", sum.missing.len()),
    };
    // Where something is not new work, a line is given three ways under itself, each the
    // sum of the same figures the line sums (call 158): the line itself stays the whole,
    // as it stands. Where nothing is marked it is not given, and the line is as it was.
    let parts = |out: &mut String, by: &Option<view::WorkSplit>, noun: Option<(&str, &str)>| {
        if let Some(by) = by {
            for (label, sum) in by.parts() {
                let count = match noun {
                    Some((one, many)) if sum.members > 0 => format!("{} {}  ", sum.members, if sum.members == 1 { one } else { many }),
                    _ => String::new(),
                };
                let _ = writeln!(out, "      {label:<18}  {count}{}", said(sum));
            }
        }
    };
    let counted = |by: &[usize; 3], unit: &str| -> String {
        let one = |n: usize| if n == 0 { "none".to_string() } else { format!("{n} {unit}") };
        format!("to build {}; existing to remain {}; to demolish {}", one(by[0]), one(by[1]), one(by[2]))
    };
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    let mut gone: Vec<String> = plumbline_world::components::demolished(&mut job.world).into_iter().collect();
    gone.sort();
    let under = job.children();
    for s in building::storey_totals(job) {
        let _ = writeln!(out, "\n{}  ({})", s.name, s.path);
        let stands = |by: &Option<view::WorkSplit>| if by.is_some() { " as it stands" } else { "" };
        let b = &s.by_work;
        let _ = writeln!(out, "  {} walls {:<12} {}{}", s.walls.status.flag(), s.walls.members, said(&s.walls), stands(&b.walls));
        parts(&mut out, &b.walls, Some(("wall", "walls")));
        let _ = writeln!(out, "  {} wall face, gross   {}{}", s.wall_face.status.flag(), said(&s.wall_face), stands(&b.wall_face));
        parts(&mut out, &b.wall_face, None);
        let _ = writeln!(out, "  {} wall face, net     {}{}", s.wall_net.status.flag(), said(&s.wall_net), stands(&b.wall_net));
        parts(&mut out, &b.wall_net, None);
        // What is coming down is counted as it stands, and said (judged 2026-11-21).
        let down: Vec<String> = gone.iter().filter(|p| p.starts_with("wall-") && under.get(&s.path).is_some_and(|c| c.values().any(|x| x == *p))).cloned().collect();
        if !down.is_empty() {
            let _ = writeln!(out, "      of the walls, {} to demolish ({}): counted above as {} stand{}", down.len(), down.join(", "), if down.len() == 1 { "it" } else { "they" }, if down.len() == 1 { "s" } else { "" });
        }
        // By the type of wall, where a level has more than one.
        if s.walls_by_type.len() > 1 {
            for t in &s.walls_by_type {
                let _ = writeln!(out, "  {}   of them {}: {}, {}; face {}; net {}", t.length.status.max(t.net.status).flag(), t.name, if t.length.members == 1 { "1 wall".to_string() } else { format!("{} walls", t.length.members) }, said(&t.length), said(&t.face), said(&t.net));
            }
        }
        // A level's footings, where it has any: their length and the concrete in them.
        if s.footings.members > 0 {
            let _ = writeln!(out, "  {} footings {:<9} {}{}", s.footings.status.flag(), s.footings.members, said(&s.footings), stands(&b.footings));
            parts(&mut out, &b.footings, Some(("footing", "footings")));
            let _ = writeln!(out, "  {} footing concrete   {}{}", s.footing_volume.status.flag(), said(&s.footing_volume), stands(&b.footing_volume));
            parts(&mut out, &b.footing_volume, None);
        }
        let _ = writeln!(out, "  {} floors {:<11} {}{}", s.floors.status.flag(), s.floors.members, said(&s.floors), stands(&b.floors));
        parts(&mut out, &b.floors, Some(("floor", "floors")));
        if s.rooms.members > 0 {
            let _ = writeln!(out, "  {} rooms {:<12} {}  (the floor inside the walls)", s.rooms.status.flag(), s.rooms.members, said(&s.rooms));
        }
        let _ = writeln!(out, "    doors {}, windows {}{}", s.doors, s.windows, if b.doors.is_some() || b.windows.is_some() { " as they stand" } else { "" });
        if let Some(d) = &b.doors {
            let _ = writeln!(out, "      doors: {}", counted(d, "EA"));
        }
        if let Some(d) = &b.windows {
            let _ = writeln!(out, "      windows: {}", counted(d, "EA"));
        }
        // What is set down on the level, counted by the model (call 197 point 10), in the words the sheet window uses.
        let _ = writeln!(out, "  Set down on this level:");
        for line in s.set_down.said() {
            let _ = writeln!(out, "    {line}");
        }    }
    // Each roof: its planes, surface and squares, and its lines by kind (J27A-13: a
    // roof of 12 squares was not in the takeoff, and nothing said it was left out).
    let roofs: Vec<(String, String)> = view::elements(job).into_iter().filter(|e| e.kind == plumbline_world::Kind::Roof).map(|e| (e.path, e.name)).collect();
    for (path, name) in roofs {
        let t = view::roof_totals(job, &path);
        let squares = view::in_squares(&t.surface);
        let _ = writeln!(out, "\n{name}  ({path})");
        let _ = writeln!(out, "  {} roof planes {:<6} {}, {}{}", t.surface.status.flag(), t.facets, said(&t.surface), said(&squares), if t.surface_by_work.is_some() { " as it stands" } else { "" });
        parts(&mut out, &t.surface_by_work, Some(("plane", "planes")));
        match t.predominant_pitch {
            Some((pitch, status)) => {
                let _ = writeln!(out, "  {} predominant pitch  {}", status.flag(), view::pitch_said(pitch));
            }
            None => {
                let _ = writeln!(out, "  ? predominant pitch  not said: {}", t.pitch_note);
            }
        }
        for ((kind, sum), by) in t.lines.iter().zip(&t.lines_by_work) {
            let _ = writeln!(out, "  {} {:<8} {:<9} {}{}", sum.status.flag(), kind.said(), sum.members, said(sum), if by.is_some() { " as it stands" } else { "" });
            parts(&mut out, by, Some(("line", "lines")));
        }
    }
    for t in building::takeoff(job) {
        let what = if !t.classification.is_empty() { t.classification.as_str() } else if t.flag == Status::Unknown { "which system is not known" } else { "no system" };
        // The length is no surer than the runs it is the sum of, and is marked so.
        let pieces = |n: usize| format!("{n} straight piece{}", if n == 1 { "" } else { "s" });
        // A thing on no run has no length to be sure of: it says how many straight pieces and nothing after (call 213 point 12).
        if t.runs.members == 0 {
            let _ = writeln!(out, "\n{}{}  ({what}; {})", if t.flag == Status::Known { String::new() } else { format!("{} ", t.flag.flag()) }, t.name, pieces(t.runs.members));
        } else {
            let _ = writeln!(out, "\n{}{}  ({what}; {}, {}{}{})", if t.flag == Status::Known { String::new() } else { format!("{} ", t.flag.flag()) }, t.name, pieces(t.runs.members), t.runs.status.flag().trim_end(), said(&t.runs), if t.runs_by_work.is_some() { " as it stands" } else { "" });
        }
        parts(&mut out, &t.runs_by_work, Some(("straight piece", "straight pieces")));
        for line in &t.lines {
            let _ = writeln!(out, "  {} {:<8} {:<16} {:>2} {:<6}  {:>10}", line.size_status.max(line.length.status).flag(), capital(line.kind.said()), line.size, line.length.members, if line.length.members == 1 { "piece" } else { "pieces" }, said(&line.length));
            parts(&mut out, &line.length_by_work, Some(("piece", "pieces")));
        }
        // Each is counted as it stands, and given three ways where one is not new work.
        let marked = |by: &[usize; 3]| by[1] + by[2] > 0;
        for ((kind, what, count, status), by) in t.fittings.iter().zip(&t.fittings_by_work) {
            let _ = writeln!(out, "  {} {:<9} {what}: {count} EA{}", status.flag(), kind.map(|k| k.title()).unwrap_or("Fitting"), if marked(by) { " as it stands" } else { "" });
            if marked(by) {
                let _ = writeln!(out, "      {}", counted(by, "EA"));
            }
        }
        for ((kind, what, count, status), by) in t.placed.iter().zip(&t.placed_by_work) {
            let _ = writeln!(out, "  {} {:<9} {what}: {count} EA{}", status.flag(), capital(kind.said()), if marked(by) { " as it stands" } else { "" });
            if marked(by) {
                let _ = writeln!(out, "      {}", counted(by, "EA"));
            }
        }
        // How it hangs together: said only where there are runs to join.
        let j = &t.joined;
        if t.runs.members > 0 {
            let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
            let over = if j.over_connection.is_empty() { String::new() } else { format!("; {}", plural(j.over_connection.len(), "place where a run passes over a thing's connection (joined to it)", "places where a run passes over a thing's connection (joined to it)")) };
            let _ = writeln!(out, "    joined: {}; {}; {}{over}", plural(j.networks, "network", "networks"), plural(j.open.len(), "open end", "open ends"), plural(j.no_fitting.len(), "place where a run meets another along its length with no fitting", "places where a run meets another along its length with no fitting"));
        }
        for o in &j.open {
            let _ = writeln!(out, "      open: {} at ({}, {}, {})", o.said, view::num(o.at[0]), view::num(o.at[1]), view::num(o.at[2]));
        }
        for at in &j.no_fitting {
            let _ = writeln!(out, "      no fitting at ({}, {}, {})", view::num(at[0]), view::num(at[1]), view::num(at[2]));
        }
        for o in &j.over_connection {
            let _ = writeln!(out, "      {} at ({}, {}, {}): joined to it, with no fitting", o.said, view::num(o.at[0]), view::num(o.at[1]), view::num(o.at[2]));
        }
        for warning in &j.warnings {
            let _ = writeln!(out, "      {warning}");
        }
        for (path, why) in &j.left_out {
            let _ = writeln!(out, "    ? not checked: {path}: {why}");
        }
    }
    out
}

/// The job priced both ways and the two set against each other: each element that has an
/// allowance or a cost line, then the totals, each saying what it rests on and what it
/// leaves out. Every figure is worked out in the world; this only prints them.
pub(super) fn price(job: &mut plumbline_world::Job) -> String {
    let p = view::pricing(job);
    let money = view::money;
    let amount = |d: &Option<plumbline_world::Derived>| match d {
        None => "none entered".to_string(),
        Some(d) => match d.value {
            Some(x) if d.unit == "$" => format!("{}{}", d.status.flag().trim_end(), money(x)).trim().to_string(),
            Some(x) if d.unit == "%" => format!("{}{} %", d.status.flag().trim_end(), view::figure(x, 2)).trim().to_string(),
            Some(x) => format!("{}${} per {}", d.status.flag().trim_end(), view::rate(x), d.unit.trim_start_matches("$/")).trim().to_string(),
            None => "no figure".to_string(),
        },
    };
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if p.elements.is_empty() {
        out.push_str("\nNothing in this job has an allowance or a cost line.\n");
    }
    for e in &p.elements {
        let _ = writeln!(out, "\n{}  [{}]  ({})", e.name, e.kind.said(), e.path);
        if let Some((name, unit, d)) = &e.figure {
            let _ = writeln!(out, "    priced by its {}: {}", view::words(name), d.value.map(|x| format!("{}{} {unit}", d.status.flag().trim_end(), view::figure(x, 2))).unwrap_or_else(|| format!("no figure ({})", if d.note.is_empty() { "a value it is worked out from is not known" } else { d.note.as_str() })));
        }
        let rate = e.rate.as_ref().map(|r| match r.value {
            Some(x) => format!(" at {}${} per {}", r.status.flag().trim_end(), view::rate(x), r.unit.trim_start_matches("$/")),
            None => format!(" (rate not known: {})", r.source),
        });
        let _ = writeln!(out, "    allowance   {}{}", amount(&e.allowance), rate.unwrap_or_default());
        let detail = if e.all_off { "none: every cost line it has is switched off".to_string() } else { amount(&e.detail) };
        let _ = writeln!(out, "    detail      {detail}{}", e.detail_rate.as_ref().filter(|d| d.value.is_some()).map(|d| format!(", which is {}", amount(&Some(d.clone())))).unwrap_or_default());
        if let Some(d) = e.detail.as_ref().filter(|d| d.value.is_none() || !d.note.is_empty()) {
            let _ = writeln!(out, "      {} {}", if d.value.is_none() { "?" } else { "!" }, d.note);
        }
        // A detail with a figure and no rate: the rate was refused, and says why.
        if let (Some(d), Some(r)) = (&e.detail, &e.detail_rate) {
            if d.value.is_some() && r.value.is_none() {
                let _ = writeln!(out, "      ? it is not given as a rate: {}", r.note);
            }
        }
        if let Some(d) = e.allowance.as_ref().filter(|d| d.value.is_none()) {
            let _ = writeln!(out, "      ? {}", d.note);
        }
        if let (Some(diff), Some(pct)) = (&e.difference, &e.difference_percent) {
            if let Some(x) = diff.value {
                let flag = diff.status.flag().trim_end();
                if x == 0.0 {
                    let _ = writeln!(out, "    the detail is {flag}the same as the allowance");
                } else {
                    let word = if x > 0.0 { "over" } else { "under" };
                    let _ = writeln!(out, "    the detail is {flag}{} {word} the allowance{}", money(x.abs()), pct.value.map(|v| format!(" ({}{} %)", pct.status.flag().trim_end(), view::figure(v, 2))).unwrap_or_default());
                }
            }
        }
    }
    let total = |name: &str, t: &view::MoneyTotal, what: &str| {
        let mut s = String::new();
        match t.value {
            None => {
                let _ = writeln!(s, "  {name:<12} none: no element has {what}");
            }
            Some(x) => {
                let _ = writeln!(s, "  {name:<12} {}{} over {} element{}", t.status.flag().trim_end(), money(x), t.members, if t.members == 1 { "" } else { "s" });
                if t.assumed != 0.0 {
                    let _ = writeln!(s, "               of which {} rests on something assumed", money(t.assumed));
                }
            }
        }
        for (path, why) in &t.left_out {
            let _ = writeln!(s, "             ? left out: {path}: {why}");
        }
        s
    };
    out.push_str("\nThe job\n");
    out.push_str(&total("allowances", &p.allowance, "an allowance that comes to a figure"));
    // What each total does not cover at all, by name: a total says what it leaves out.
    let named = |list: Vec<&view::PricedElement>| list.iter().map(|e| e.path.clone()).collect::<Vec<_>>().join(", ");
    let no_allowance: Vec<&view::PricedElement> = p.elements.iter().filter(|e| e.rate.is_none()).collect();
    if !no_allowance.is_empty() {
        let _ = writeln!(out, "             ? no allowance entered on {} element{} with cost lines: {}", no_allowance.len(), if no_allowance.len() == 1 { "" } else { "s" }, named(no_allowance));
    }
    out.push_str(&total("detail", &p.detail, "a detail that is whole"));
    let no_lines: Vec<&view::PricedElement> = p.elements.iter().filter(|e| e.detail.is_none() && e.rate.is_some() && !e.all_off).collect();
    if !no_lines.is_empty() {
        let _ = writeln!(out, "             ? no cost line on {} element{} with an allowance: {}", no_lines.len(), if no_lines.len() == 1 { "" } else { "s" }, named(no_lines));
    }
    match p.both {
        None => out.push_str("  like for like: no element has both an allowance and a whole detail to compare\n"),
        Some((a, d, status, n)) => {
            // Each amount carries its own flag, and the difference and the percentage the
            // worse of the two.
            let (fa, fd) = (a.1.flag().trim_end().to_string(), d.1.flag().trim_end().to_string());
            let (a, d) = (a.0, d.0);
            // The difference and the percentage are the model's (`like_for_like`).
            let (diff, percent) = p.like_for_like().unwrap_or((0.0, None));
            let flag = status.flag().trim_end();
            let said = if diff == 0.0 {
                format!("the detail is {flag}the same")
            } else {
                let pct = percent.map(|x| format!(" ({flag}{} %)", view::figure(x, 2))).unwrap_or_default();
                format!("the detail is {flag}{} {}{pct}", money(diff.abs()), if diff > 0.0 { "over" } else { "under" })
            };
            let _ = writeln!(out, "  like for like, over the {n} element{} that have both: allowances {fa}{}, detail {fd}{}: {said}", if n == 1 { "" } else { "s" }, money(a), money(d));
        }
    }
    if !p.not_priced.is_empty() {
        let n: usize = p.not_priced.iter().map(|x| x.1).sum();
        let kinds: Vec<String> = p.not_priced.iter().map(|(k, n)| k.so_many(*n)).collect();
        let _ = writeln!(out, "  not priced at all: {n} thing{} that could be ({})", if n == 1 { "" } else { "s" }, kinds.join(", "));
    }
    out
}

/// A word with its first letter a capital, for the head of a column or a row.
pub(super) fn capital(word: &str) -> String {
    let mut c = word.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// The estimate sheet: every cost line under its element, in three money columns; the
/// recap; and the suppliers side by side.
/// A refusal as a builder reads it (J17A-16, J17B-21): without the machine's number
/// for why a file could not be read, and, where a thing was named that is not in the
/// job, with what to give instead: a thing is given by its path, and if what was typed
/// is the name of one thing, that thing's path is said.
pub(super) fn plainly(why: &str, args: &[String]) -> String {
    let mut said = why.to_string();
    while let Some(i) = said.find(" (os error ") {
        let end = said[i..].find(')').map(|j| i + j + 1).unwrap_or(said.len());
        said.replace_range(i..end, "");
    }
    if let Some(i) = said.find("no element ") {
        let typed: String = said[i + "no element ".len()..].chars().take_while(|c| !c.is_whitespace() && *c != ':' && *c != ',').collect();
        let job = args.get(1).and_then(|file| std::fs::read_to_string(file).ok()).and_then(|text| file::load(&text).ok());
        if let (false, Some((mut job, _))) = (typed.is_empty(), job) {
            let named: Vec<String> = view::elements(&mut job).into_iter().filter(|e| e.name.eq_ignore_ascii_case(&typed)).map(|e| e.path).collect();
            let hint = match named.as_slice() {
                [one] => format!("\n  a thing is given by its path, and {typed:?} is the name of {one}: give {one}"),
                [] => "\n  a thing is given by its path, like wall-0004; `plumbline show <file>` lists everything in the job with its path".to_string(),
                many => format!("\n  a thing is given by its path, and {} things are called {typed:?}: {}", many.len(), many.join(", ")),
            };
            said.push_str(&hint);
        }
    }
    said
}

/// The estimate read item by item: every line of one description, one cost type, one
/// unit and one division on one row, division by division (`Estimate::by_item`; the
/// estimate window's "The estimate, by item"). The same lines and the same money as
/// `estimate`, added up; nothing is worked out here.
pub(super) fn estimate_by_item(job: &mut plumbline_world::Job) -> String {
    let est = view::estimate(job);
    let money = view::money;
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    let items = est.by_item();
    if items.is_empty() {
        out.push_str("\nNothing in this job has a cost line.\n");
        // What is set down and has nothing priced on it is in no total, and is said so (call 196).
        if let Some(said) = view::not_priced_said(&view::pricing(job).not_priced) {
            let _ = writeln!(out, "  ! {said}");
        }
        // A bearing wall the work opens says what that needs, priced or not (call 167).
        for (_, said) in plumbline_world::hidden::cautions(&mut job.world) {
            let _ = writeln!(out, "  ! {said}");
        }
        return out;
    }
    let on = est.lines.iter().filter(|l| !l.off).count();
    let _ = writeln!(out, "\nTHE ESTIMATE, BY ITEM: {} from {}", view::so_many(items.len(), "item"), view::so_many(on, "line"));
    let _ = writeln!(out, "{:<50}{:<20}{:>6} {:>16} {:>14} {:>15}", "", "cost type", "places", "quantity", "unit cost", "amount");
    let total = |t: &view::MoneyTotal| match t.value {
        Some(x) => format!("{}{}", t.status.flag().trim_end(), money(x)),
        None => "no figure".to_string(),
    };
    let mut division: Option<Option<u8>> = None;
    for it in &items {
        let d = it.division.map(|d| d.number());
        if division != Some(d) {
            division = Some(d);
            let (heading, sum) = match it.division {
                Some(d) => (format!("Division {}  {}", d.number(), d.title()), est.divisions.iter().find(|x| x.division == d).map(|x| total(&x.total)).unwrap_or_default()),
                None => ("No division".to_string(), total(&est.no_division)),
            };
            let _ = writeln!(out, "{heading:<108}{sum:>15}");
        }
        let quantity = match it.quantity_said() {
            Some(q) => format!("{}{q} {}", it.quantity_status.flag().trim_end(), it.unit),
            None => "no figure".to_string(),
        };
        let at = match it.unit_cost {
            Some(x) => format!("${}/{}", view::rate(x), it.unit),
            None => it.no_one_price_said(),
        };
        let class = it.class.map(plumbline_world::Class::title).unwrap_or("no cost type");
        // A long name is cut and says it is cut; `plumbline estimate` has it whole.
        let name: String = if it.description.chars().count() > 46 { format!("{}...", it.description.chars().take(43).collect::<String>()) } else { it.description.clone() };
        let _ = writeln!(out, "  {name:<48}{class:<20}{:>6} {quantity:>16} {at:>14} {:>15}", it.places, total(&it.amount));
    }
    let _ = writeln!(out, "{:<108}{:>15}", "Job cost, as `plumbline estimate` has it (with what is carried as allowances)", total(&est.job_cost));
    out
}

pub(super) fn estimate(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::{Class, Column, Derived};
    let est = view::estimate(job);
    let money = view::money;
    let flagged = |d: &Derived| match d.value {
        Some(x) => format!("{}{}", d.status.flag().trim_end(), money(x)),
        None => "no figure".to_string(),
    };
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if est.lines.is_empty() && est.allowance_on.is_empty() {
        out.push_str("\nNothing in this job has a cost line.\n");
        // What is set down and has nothing priced on it is in no total, and is said so (call 196).
        if let Some(said) = view::not_priced_said(&view::pricing(job).not_priced) {
            let _ = writeln!(out, "  ! {said}");
        }
        // A bearing wall the work opens says what that needs, priced or not (call 167).
        for (_, said) in plumbline_world::hidden::cautions(&mut job.world) {
            let _ = writeln!(out, "  ! {said}");
        }
        return out;
    }
    let _ = writeln!(out, "\n{:<58}{:>14} {:>16} {:>13} {:>13} {:>13}", "THE SHEET", "priced", "at", "MATERIAL", "LABOR", "GENERAL");
    let mut last = String::new();
    // The lines the marks of a marked-up PDF made stand under the file's name (call 227 point 4).
    let from_marks: std::collections::HashMap<String, (String, usize)> = plumbline_world::marks::lines_from_marks(job).into_iter().map(|(d, _, s)| (d, (s.file, s.page))).collect();
    let mut last_marks: Option<(String, usize)> = None;
    for l in &est.lines {
        if l.path != last {
            // The job itself is "the job", as the workbook has it, not its place in the file.
            let _ = writeln!(out, "{}  [{}]  ({})", if l.kind == plumbline_world::Kind::Job { "the job" } else { l.element.as_str() }, l.kind.said(), l.path);
            last = l.path.clone();
            last_marks = None;
        }
        let src = if l.kind == plumbline_world::Kind::Job { from_marks.get(&l.description).cloned() } else { None };
        if src != last_marks {
            if let Some((file, page)) = &src {
                let n = est.lines.iter().filter(|o| o.kind == plumbline_world::Kind::Job && from_marks.get(&o.description) == src.as_ref()).count();
                let _ = writeln!(out, "  from {file}, page {page}: {} of its marks, each Assumed, not priced, no division set", count(n, "line", "lines"));
            }
            last_marks = src;
        }
        // Hours are shown to the four places they are held and added to: at two, a line's
        // hours would not multiply to its amount, or add to its class.
        // Anything else is shown to two places unless, at two, it would not multiply by
        // its price to its amount on a calculator (J17A-9): then to the four it is held to.
        // (The rule is the model's, `SheetLine::priced_places`: the window shows the same.)
        let places = |_: &str, _: f64| l.priced_places(4);
        let priced = match l.priced.value {
            Some(x) => format!("{}{} {}", l.priced.status.flag().trim_end(), view::figure(x, places(&l.priced.unit, x)), l.priced.unit),
            None => "no figure".to_string(),
        };
        let at = match l.unit_cost.as_ref() {
            Some(v) => match v.value {
                Some(x) => format!("{}${}/{}", v.status.flag().trim_end(), view::rate(x), v.unit.trim_start_matches("$/")),
                None => "no price".to_string(),
            },
            None => "no price".to_string(),
        };
        let amount = if l.off { format!("off ({})", flagged(&l.amount)) } else { flagged(&l.amount) };
        let (mut m, mut b, mut g) = (String::new(), String::new(), String::new());
        match l.class.map(Class::column) {
            Some(Column::Material) => m = amount,
            Some(Column::Labor) => b = amount,
            Some(Column::General) => g = amount,
            // A line with no class stands in no column: its amount is said under it.
            None => {}
        }
        let mut what = l.description.clone();
        if let Some(c) = l.class.filter(|c| !matches!(c, Class::Material | Class::Labor)) {
            what.push_str(&format!(" [{}]", c.title()));
        }
        let _ = writeln!(out, "  {:<56}{:>14} {:>16} {:>13} {:>13} {:>13}", what, priced, at, m, b, g);
        if let Some(material) = &l.material {
            let _ = writeln!(out, "      made of {}", if material.is_empty() { "the element's material, which is not said".to_string() } else { format!("{material:?}") });
        }
        if l.amount.value.is_none() {
            let _ = writeln!(out, "      ? {}", l.amount.note);
        } else if !l.amount.note.is_empty() {
            let _ = writeln!(out, "      ! {}", l.amount.note);
        }
        if l.class.is_none() && !l.off {
            let _ = writeln!(out, "      ? no cost type: it is in no column and no row of the recap{}", l.amount.value.map(|_| format!(" (it comes to {})", flagged(&l.amount))).unwrap_or_default());
        }
        if !l.quantity.note.is_empty() && l.quantity.value.is_some() {
            let _ = writeln!(out, "      ! its quantity: {}", l.quantity.note);
        }
    }
    let col = |c: Column| est.columns.iter().find(|x| x.0 == c).map(|x| &x.1);
    let shown = |t: Option<&view::MoneyTotal>| t.and_then(|t| t.value.map(|x| format!("{}{}", t.status.flag().trim_end(), money(x)))).unwrap_or_else(|| "none".to_string());
    let _ = writeln!(out, "  {:<56}{:>14} {:>16} {:>13} {:>13} {:>13}", "column totals", "", "", shown(col(Column::Material)), shown(col(Column::Labor)), shown(col(Column::General)));

    let _ = writeln!(out, "\nTHE RECAP");
    for c in &est.classes {
        let hours = c.hours.filter(|_| c.total.value.is_some()).map(|(h, s)| format!("   {}{} hr", s.flag().trim_end(), view::figure(h, 4))).unwrap_or_default();
        match c.total.value {
            Some(x) => {
                let _ = writeln!(out, "  {:<20}{:>14}   {} line{}{hours}", c.class.title(), format!("{}{}", c.total.status.flag().trim_end(), money(x)), c.total.members, if c.total.members == 1 { "" } else { "s" });
                if c.total.assumed != 0.0 {
                    let _ = writeln!(out, "  {:<20}{:>14}   of it rests on something assumed", "", money(c.total.assumed));
                }
            }
            // Lines of this cost type that come to no figure are not "none": a labor
            // row read "none" over four labor lines with no production rate (J23-4).
            None if !c.total.left_out.is_empty() => {
                let _ = writeln!(out, "  {:<20}{:>14}   ? {} to price", c.class.title(), "no figure", view::so_many(c.total.left_out.len(), "line"));
            }
            None => {
                let _ = writeln!(out, "  {:<20}{:>14}", c.class.title(), "none");
            }
        }
        if c.total.value.is_some() && !c.total.left_out.is_empty() {
            let _ = writeln!(out, "  {:<20}{:>14}   ? leaves out {} of this cost type with no figure yet", "", "", view::so_many(c.total.left_out.len(), "line"));
        }
        if let Some(said) = c.hours_unpriced_said() {
            let _ = writeln!(out, "  {:<20}{:>14}   ? {said}: in neither the dollars nor the hours above", "", "");
        }
    }
    let line = |name: &str, d: &Derived| {
        let mut s = String::new();
        let _ = writeln!(s, "  {:<20}{:>14}{}", name, flagged(d), if d.value.is_none() { format!("   ? {}", d.note) } else { String::new() });
        s
    };
    // Allowances carried in the price, on a row of their own (call 114).
    if !est.allowance_on.is_empty() {
        let things = view::so_many(est.allowance_on.len(), "thing");
        match est.allowances.value {
            Some(x) => {
                let _ = writeln!(out, "  {:<20}{:>14}   {things} priced by an allowance and no cost line", "Allowances", format!("{}{}", est.allowances.status.flag().trim_end(), money(x)));
            }
            None => {
                let _ = writeln!(out, "  {:<20}{:>14}   ? {things} priced by an allowance, none with a figure", "Allowances", "no figure");
            }
        }
    }
    match est.job_cost.value {
        Some(x) => {
            // An allowance carried in it is not a line, and is counted as what it is.
            let lines = est.job_cost.members - est.allowances.members;
            let carried = match est.allowances.members {
                0 => String::new(),
                1 => " and 1 allowance".to_string(),
                n => format!(" and {n} allowances"),
            };
            let _ = writeln!(out, "  {:<20}{:>14}   {} line{}{carried}", "Job cost", format!("{}{}", est.job_cost.status.flag().trim_end(), money(x)), lines, if lines == 1 { "" } else { "s" });
        }
        None => {
            let _ = writeln!(out, "  {:<20}{:>14}   no line with a cost type comes to a figure", "Job cost", "none");
        }
    }
    let rate = |v: Option<&plumbline_world::Val<f64>>| v.and_then(|v| v.value.map(|x| format!(" at {}{} %", v.status.flag().trim_end(), view::rate(x)))).unwrap_or_default();
    out.push_str(&line(&format!("Overhead{}", rate(est.markup.as_ref().map(|m| &m.overhead))), &est.overhead));
    out.push_str(&line(&format!("Profit{}", rate(est.markup.as_ref().map(|m| &m.profit))), &est.profit));
    out.push_str(&line("PRICE", &est.price));
    if est.price.value.is_some() && !est.price.note.is_empty() {
        let _ = writeln!(out, "  {:<20}{:>14}   ! {}", "", "", est.price.note);
    }
    // What is built and has nothing priced on it is in no line above and no total:
    // the price is not the price of the building until it is, and says so (J17A-19).
    let pricing = view::pricing(job);
    if let Some(said) = view::not_priced_said(&pricing.not_priced) {
        let _ = writeln!(out, "  ! {said}");
    }
    if let Some(said) = view::twice_said(&pricing.twice) {
        let _ = writeln!(out, "  ! {said}");
    }
    if let Some(said) = view::board_twice_said(&pricing.board_twice) {
        let _ = writeln!(out, "  ! {said}");
    }
    if let Some(said) = pricing.allowances_alone_said() {
        let _ = writeln!(out, "  ! {said}");
    }
    for (_, said) in plumbline_world::hidden::cautions(&mut job.world) {
        let _ = writeln!(out, "  ! {said}");
    }
    for lump in &pricing.lumps {
        let _ = writeln!(out, "  ! {}", lump.said());
    }
    if !est.job_cost.left_out.is_empty() {
        let _ = writeln!(out, "  left out of the job cost, and of everything taken on it: {} line{}", est.job_cost.left_out.len(), if est.job_cost.left_out.len() == 1 { "" } else { "s" });
        for (who, why) in &est.job_cost.left_out {
            let _ = writeln!(out, "    ? {who}: {why}");
        }
    }
    let off: Vec<&view::SheetLine> = est.lines.iter().filter(|l| l.off).collect();
    if !off.is_empty() {
        let _ = writeln!(out, "  switched off, on the sheet and out of every total: {}", off.iter().map(|l| format!("{}: {:?} ({})", l.path, l.description, flagged(&l.amount))).collect::<Vec<_>>().join("; "));
    }

    // The job cost again, division by division: all sixteen, and what is in none.
    if est.divisions.iter().any(|d| d.total.members > 0) || !est.allowance_on.is_empty() || est.no_division.members > 0 || !est.no_division.left_out.is_empty() || est.divisions.iter().any(|d| !d.total.left_out.is_empty()) {
        let _ = writeln!(out, "\nBY DIVISION");
        let lines = |n: usize| plumbline_world::view::so_many(n, "line");
        for d in &est.divisions {
            // "none": no work in it. "no figure": work in it, and none of it priced
            // yet (the same two words the workbook and the PM's lens use, J17C-7).
            let said = match (d.total.value, d.total.left_out.is_empty()) {
                (Some(x), _) => format!("{}{}", d.total.status.flag().trim_end(), money(x)),
                (None, true) => "none".to_string(),
                (None, false) => "no figure".to_string(),
            };
            let name = format!("{:>2} {}", d.division.number(), d.division.title());
            let _ = writeln!(out, "  {name:<36}{said:>14}   {}", if d.total.members == 0 { String::new() } else { lines(d.total.members) });
            for (who, why) in &d.total.left_out {
                let _ = writeln!(out, "      ? left out: {who}: {why}");
            }
        }
        // The sixteen together: no surer than the least sure division, and saying how
        // many lines the divisions above leave out (J17A-2).
        let (in_all, status, out_of_them) = est.sixteen();
        let said = match in_all {
            None => "none".to_string(),
            Some(x) => format!("{}{}", status.flag().trim_end(), money(x)),
        };
        let _ = writeln!(out, "  {:<36}{said:>14}{}", "the sixteen divisions", if out_of_them == 0 { String::new() } else { format!("   leaves out {}, named above", lines(out_of_them)) });
        let none = &est.no_division;
        if none.members > 0 || !none.left_out.is_empty() {
            let said = none.value.map(|x| format!("{}{}", none.status.flag().trim_end(), money(x))).unwrap_or_else(|| "no figure".to_string());
            let _ = writeln!(out, "  in no division: {}, {said}: in the job cost, and in none of the sixteen", lines(none.members));
            for l in est.lines.iter().filter(|l| !l.off && l.class.is_some() && l.division.is_none()) {
                match l.amount.value {
                    Some(x) => {
                        let _ = writeln!(out, "    {}{}  {}: {:?}", l.amount.status.flag().trim_end(), money(x), l.path, l.description);
                    }
                    None => {
                        let _ = writeln!(out, "    ? left out: {}: {:?}: {}", l.path, l.description, l.amount.note);
                    }
                }
            }
        }
        // What is carried as an allowance, named as one: the sixteen, what is in no
        // division and this come to the job cost (call 123).
        if !est.allowance_on.is_empty() {
            let a = &est.allowances;
            let said = a.value.map(|x| format!("{}{}", a.status.flag().trim_end(), money(x))).unwrap_or_else(|| "no figure".to_string());
            let _ = writeln!(out, "  allowances: {}, {said}: carried in the job cost as allowances, and in none of the sixteen", plumbline_world::view::so_many(est.allowance_on.len(), "allowance"));
        }
    }

    let made: Vec<&view::SheetLine> = est.lines.iter().filter(|l| l.material.is_some() && !l.off).collect();
    if !made.is_empty() {
        let _ = writeln!(out, "\nSUPPLIERS, SIDE BY SIDE (the lines made of a material)");
        let mut head = format!("  {:<42}{:>14}", "", "generic");
        for s in &est.suppliers {
            let _ = write!(head, " {:>14}", s);
        }
        let _ = writeln!(out, "{head}");
        for l in &made {
            let mut row = format!("  {:<42}{:>14}", format!("{}: {}", l.element, l.description), flagged(&l.amount));
            for s in &est.suppliers {
                let _ = write!(row, " {:>14}", l.quotes.get(s).map(&flagged).unwrap_or_else(|| "no quote".to_string()));
            }
            let _ = writeln!(out, "{row}");
        }
        let mut row = format!("  {:<42}{:>14}", "total", shown(Some(&est.at_generic)));
        for (_, t) in &est.at_supplier {
            let _ = write!(row, " {:>14}", shown(Some(t)));
        }
        let _ = writeln!(out, "{row}");
        // How many of the lines each total is over: a total over fewer is not the same job.
        let mut row = format!("  {:<42}{:>14}", "lines in the total", format!("{} of {}", est.at_generic.members, made.len()));
        for (_, t) in &est.at_supplier {
            let _ = write!(row, " {:>14}", format!("{} of {}", t.members, made.len()));
        }
        let _ = writeln!(out, "{row}");
        for (s, t) in std::iter::once((&"generic".to_string(), &est.at_generic)).chain(est.at_supplier.iter().map(|(s, t)| (s, t))) {
            for (who, why) in &t.left_out {
                let _ = writeln!(out, "    ? {s} leaves out {who}: {why}");
            }
        }
        if est.suppliers.is_empty() {
            let _ = writeln!(out, "  no supplier has quoted a material of this job");
        }
    }
    out
}
