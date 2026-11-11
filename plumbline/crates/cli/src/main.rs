//! The command line over the job world. It reads and edits the same saved job the
//! desktop application will.
//!
//!   plumbline sample <file>                      write the sample job
//!   plumbline house <file>                       write the sample whole house, with its MEP
//!   plumbline takeoff <file>                     each storey's walls and floors, and each system's runs by size
//!   plumbline show   <file>                      every element and value, flagged
//!   plumbline set    <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source "<where from>" [--line "<cost line>"]
//!
//! A value is typed the way the drawing or the tape gives it and is kept in that unit:
//! a length as `2.5 ft`, `30 in`, `19 1/2"` or `25' 6 1/2"`; a pitch as `6`, `6:12` or
//! `1/4:12`. A length with no unit is refused, never read as feet.
//!
//!   plumbline check  <file>                      load, re-derive, confirm the file is exactly what a save would write
//!   plumbline lines  <file>                      the lines the planes imply, set against the lines entered
//!   plumbline apply  <file> <path> <condition> --source "<who says>"   give an element a condition's cost lines
//!   plumbline roof   <file> <name> --points "0,0 40,0 40,30 0,30" --pitch 6:12 --eave-height "10 ft" --status <S> --source "<where from>"
//!                                                a new roof solved from a footprint (coordinates in feet)
//!
//! On a roof solved from a footprint, `set` also takes `edge:<n>` (a pitch, or `gable`) and
//! `move:<n>` (a length; the edge moves out by it, in if it is negative), and solves again.

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;

use plumbline_world::value::Status;
use plumbline_world::units::{self, Len};
use plumbline_world::{file, roof, sample, solve, view, Footprint, Rise, Val};

/// Print a line. A reader that has stopped listening (`| head -1`) is not an error and
/// must not be a panic, which is what `println!` makes of it.
macro_rules! say {
    ($($arg:tt)*) => {{
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

fn main() -> ExitCode {
    // An argument that is not text is refused, not a panic.
    let mut args: Vec<String> = Vec::new();
    for a in std::env::args_os().skip(1) {
        match a.into_string() {
            Ok(s) => args.push(s),
            Err(bad) => {
                eprintln!("plumbline: the argument {:?} is not readable text", bad.to_string_lossy());
                return ExitCode::FAILURE;
            }
        }
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("plumbline: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage:\n  plumbline sample <file>\n  plumbline house <file>\n  plumbline show <file>\n  plumbline set <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source \"<where from>\" [--line \"<cost line>\"] [--with \"name=value\"]\n  plumbline check <file>\n  plumbline lines <file>\n  plumbline takeoff <file>\n  plumbline price <file>\n  plumbline estimate <file>\n  plumbline priced <file>\n  plumbline material <file> \"<name>\" <unit it is bought by: lb, sheet, SF>\n  plumbline line <file> <path> \"<description>\" <quantity> [--class <class>] [--of-material]\n  plumbline class <file> <path> <material|labor|fabrication|supervision|subcontract|general> [--line \"<cost line>\"]\n  plumbline labor <file> <path> <on|off> [--line \"<cost line>\"]\n  plumbline apply <file> <path> <condition> --source \"<who says>\"\n\nfields: pitch (6, 6:12, 1/4:12), height, eave_height, stretch_out (2.5 ft, 30 in, 19 1/2\", 25' 6 1/2\"), condition, wbs, allowance (28, $28.00/SF), material (a material's name)\non the job: overhead (10), profit (8)\non a material: price ($4.10/lb), order (\"1.25 lb/SF\", \"30 SF/sheet\"), quote ($4.25/lb --supplier \"<name>\")\non a cost line (--line): stretch_out, quantity (\"plan_length * stretch_out\"), waste (10), order (\"30 SF/sheet\", \"1.25 lb/SF\"), unit_cost (4.25, $4.25/SF), or a value the formula names (spacing \"12 in\")".to_string()
}

/// What the building comes to: each storey's walls and floors, then each system's runs by
/// kind and size, its fittings by kind, and what is set on it. The same figures the sheet window shows.
fn takeoff(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::building;
    let said = |sum: &view::Sum| match (sum.value, sum.members) {
        (_, 0) => "none".to_string(),
        (Some(x), _) => format!("{} {}", view::figure(x, if sum.unit == "EA" { 0 } else { 2 }), sum.unit),
        (None, n) => format!("no total, {} of {n} missing", sum.missing.len()),
    };
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    for s in building::storey_totals(job) {
        let _ = writeln!(out, "\n{}  ({})", s.name, s.path);
        let _ = writeln!(out, "  {} walls {:<12} {}", s.walls.status.flag(), s.walls.members, said(&s.walls));
        let _ = writeln!(out, "  {} wall face, gross   {}", s.wall_face.status.flag(), said(&s.wall_face));
        let _ = writeln!(out, "  {} wall face, net     {}", s.wall_net.status.flag(), said(&s.wall_net));
        let _ = writeln!(out, "  {} floors {:<11} {}", s.floors.status.flag(), s.floors.members, said(&s.floors));
        if s.rooms.members > 0 {
            let _ = writeln!(out, "  {} rooms {:<12} {}  (the floor inside the walls)", s.rooms.status.flag(), s.rooms.members, said(&s.rooms));
        }
        let _ = writeln!(out, "    doors {}, windows {}", s.doors, s.windows);
    }
    for t in building::takeoff(job) {
        let what = if !t.classification.is_empty() { t.classification.as_str() } else if t.flag == Status::Unknown { "which system is not known" } else { "no system" };
        // The length is no surer than the runs it is the sum of, and is marked so.
        let pieces = |n: usize| format!("{n} straight piece{}", if n == 1 { "" } else { "s" });
        let _ = writeln!(out, "\n{}{}  ({what}; {}, {}{})", if t.flag == Status::Known { String::new() } else { format!("{} ", t.flag.flag()) }, t.name, pieces(t.runs.members), t.runs.status.flag().trim_end(), said(&t.runs));
        for line in &t.lines {
            let _ = writeln!(out, "  {} {:<8} {:<16} {:>2} {:<6}  {:>10}", line.size_status.max(line.length.status).flag(), format!("{:?}", line.kind), line.size, line.length.members, if line.length.members == 1 { "piece" } else { "pieces" }, said(&line.length));
        }
        for (kind, what, count, status) in &t.fittings {
            let _ = writeln!(out, "  {} {:<9} {what}: {count} EA", status.flag(), kind.map(|k| k.title()).unwrap_or("Fitting"));
        }
        for (kind, what, count, status) in &t.placed {
            let _ = writeln!(out, "  {} {:<9} {what}: {count} EA", status.flag(), format!("{kind:?}"));
        }
        // How it hangs together: said only where there are runs to join.
        let j = &t.joined;
        if t.runs.members > 0 {
            let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
            let _ = writeln!(out, "    joined: {}; {}; {}", plural(j.networks, "network", "networks"), plural(j.open.len(), "open end", "open ends"), plural(j.no_fitting.len(), "place where a run meets another along its length with no fitting", "places where a run meets another along its length with no fitting"));
        }
        for o in &j.open {
            let _ = writeln!(out, "      open: {} at ({}, {}, {})", o.said, view::num(o.at[0]), view::num(o.at[1]), view::num(o.at[2]));
        }
        for at in &j.no_fitting {
            let _ = writeln!(out, "      no fitting at ({}, {}, {})", view::num(at[0]), view::num(at[1]), view::num(at[2]));
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
fn price(job: &mut plumbline_world::Job) -> String {
    let p = view::pricing(job);
    let money = |x: f64| format!("{}${}", if x < 0.0 { "-" } else { "" }, view::figure(x.abs(), 2));
    let amount = |d: &Option<plumbline_world::Derived>| match d {
        None => "none entered".to_string(),
        Some(d) => match d.value {
            Some(x) if d.unit == "$" => format!("{}{}", d.status.flag().trim_end(), money(x)).trim().to_string(),
            Some(x) if d.unit == "%" => format!("{}{} %", d.status.flag().trim_end(), view::figure(x, 2)).trim().to_string(),
            Some(x) => format!("{}${} per {}", d.status.flag().trim_end(), view::figure(x, 2), d.unit.trim_start_matches("$/")).trim().to_string(),
            None => "no figure".to_string(),
        },
    };
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if p.elements.is_empty() {
        out.push_str("\nNothing in this job has an allowance or a cost line.\n");
    }
    for e in &p.elements {
        let _ = writeln!(out, "\n{}  [{:?}]  ({})", e.name, e.kind, e.path);
        if let Some((name, unit, d)) = &e.figure {
            let _ = writeln!(out, "    priced by its {name}: {}", d.value.map(|x| format!("{}{} {unit}", d.status.flag().trim_end(), view::figure(x, 2))).unwrap_or_else(|| format!("no figure ({})", d.note)));
        }
        let rate = e.rate.as_ref().map(|r| match r.value {
            Some(x) => format!(" at {}${} per {}", r.status.flag().trim_end(), view::figure(x, 2), r.unit.trim_start_matches("$/")),
            None => format!(" (rate not known: {})", r.source),
        });
        let _ = writeln!(out, "    allowance   {}{}", amount(&e.allowance), rate.unwrap_or_default());
        let _ = writeln!(out, "    detail      {}{}", amount(&e.detail), e.detail_rate.as_ref().filter(|d| d.value.is_some()).map(|d| format!(", which is {}", amount(&Some(d.clone())))).unwrap_or_default());
        if let Some(d) = e.detail.as_ref().filter(|d| d.value.is_none()) {
            let _ = writeln!(out, "      ? {}", d.note);
        }
        if let Some(d) = e.allowance.as_ref().filter(|d| d.value.is_none()) {
            let _ = writeln!(out, "      ? {}", d.note);
        }
        if let (Some(diff), Some(pct)) = (&e.difference, &e.difference_percent) {
            if let Some(x) = diff.value {
                let word = if x > 0.0 { "over" } else if x < 0.0 { "under" } else { "the same as" };
                let _ = writeln!(out, "    the detail is {}{} {word} the allowance{}", diff.status.flag().trim_end(), money(x.abs()), pct.value.map(|v| format!(" ({} %)", view::figure(v, 2))).unwrap_or_default());
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
    out.push_str(&total("detail", &p.detail, "a detail that is whole"));
    match p.both {
        None => out.push_str("  like for like: no element has both an allowance and a whole detail to compare\n"),
        Some((a, d, status, n)) => {
            let diff = d - a;
            let word = if diff > 0.0 { "over" } else if diff < 0.0 { "under" } else { "the same as" };
            let pct = if a != 0.0 { format!(" ({} %)", view::figure(diff / a * 100.0, 2)) } else { String::new() };
            let _ = writeln!(out, "  like for like, over the {n} element{} that have both: allowances {}, detail {}: the detail is {}{} {word}{pct}", if n == 1 { "" } else { "s" }, money(a), money(d), status.flag().trim_end(), money(diff.abs()));
        }
    }
    if !p.not_priced.is_empty() {
        let n: usize = p.not_priced.iter().map(|x| x.1).sum();
        let kinds: Vec<String> = p.not_priced.iter().map(|(k, n)| format!("{n} {k:?}")).collect();
        let _ = writeln!(out, "  not priced at all: {n} element{} that could be ({})", if n == 1 { "" } else { "s" }, kinds.join(", "));
    }
    out
}

/// The estimate sheet: every cost line under its element, in three money columns; the
/// recap; and the suppliers side by side.
fn estimate(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::{Class, Column, Derived};
    let est = view::estimate(job);
    let money = |x: f64| format!("{}${}", if x < 0.0 { "-" } else { "" }, view::figure(x.abs(), 2));
    let flagged = |d: &Derived| match d.value {
        Some(x) => format!("{}{}", d.status.flag().trim_end(), money(x)),
        None => "no figure".to_string(),
    };
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if est.lines.is_empty() {
        out.push_str("\nNothing in this job has a cost line.\n");
        return out;
    }
    let _ = writeln!(out, "\n{:<58}{:>14} {:>16} {:>13} {:>13} {:>13}", "THE SHEET", "priced", "at", "MATERIAL", "LABOR", "GENERAL");
    let mut last = String::new();
    for l in &est.lines {
        if l.path != last {
            let _ = writeln!(out, "{}  [{:?}]  ({})", l.element, l.kind, l.path);
            last = l.path.clone();
        }
        let priced = match l.priced.value {
            Some(x) => format!("{}{} {}", l.priced.status.flag().trim_end(), view::figure(x, 2), l.priced.unit),
            None => "no figure".to_string(),
        };
        let at = match l.unit_cost.as_ref() {
            Some(v) => match v.value {
                Some(x) => format!("{}${}/{}", v.status.flag().trim_end(), view::figure(x, 2), v.unit.trim_start_matches("$/")),
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
            None => g = format!("no class ({})", flagged(&l.amount)),
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
        }
    }
    let col = |c: Column| est.columns.iter().find(|x| x.0 == c).map(|x| &x.1);
    let shown = |t: Option<&view::MoneyTotal>| t.and_then(|t| t.value.map(|x| format!("{}{}", t.status.flag().trim_end(), money(x)))).unwrap_or_else(|| "none".to_string());
    let _ = writeln!(out, "  {:<56}{:>14} {:>16} {:>13} {:>13} {:>13}", "column totals", "", "", shown(col(Column::Material)), shown(col(Column::Labor)), shown(col(Column::General)));

    let _ = writeln!(out, "\nTHE RECAP");
    for c in &est.classes {
        let hours = c.hours.filter(|_| c.total.value.is_some()).map(|(h, s)| format!("   {}{} hr", s.flag().trim_end(), view::figure(h, 2))).unwrap_or_default();
        match c.total.value {
            Some(x) => {
                let _ = writeln!(out, "  {:<20}{:>14}   {} line{}{hours}", c.class.title(), format!("{}{}", c.total.status.flag().trim_end(), money(x)), c.total.members, if c.total.members == 1 { "" } else { "s" });
                if c.total.assumed != 0.0 {
                    let _ = writeln!(out, "  {:<20}{:>14}   of it rests on something assumed", "", money(c.total.assumed));
                }
            }
            None => {
                let _ = writeln!(out, "  {:<20}{:>14}", c.class.title(), "none");
            }
        }
    }
    let line = |name: &str, d: &Derived| {
        let mut s = String::new();
        let _ = writeln!(s, "  {:<20}{:>14}{}", name, flagged(d), if d.value.is_none() { format!("   ? {}", d.note) } else { String::new() });
        s
    };
    match est.job_cost.value {
        Some(x) => {
            let _ = writeln!(out, "  {:<20}{:>14}   {} line{}", "Job cost", format!("{}{}", est.job_cost.status.flag().trim_end(), money(x)), est.job_cost.members, if est.job_cost.members == 1 { "" } else { "s" });
        }
        None => {
            let _ = writeln!(out, "  {:<20}{:>14}   no classed line comes to a figure", "Job cost", "none");
        }
    }
    let rate = |v: Option<&plumbline_world::Val<f64>>| v.and_then(|v| v.value.map(|x| format!(" at {}{} %", v.status.flag().trim_end(), view::figure(x, 2)))).unwrap_or_default();
    out.push_str(&line(&format!("Overhead{}", rate(est.markup.as_ref().map(|m| &m.overhead))), &est.overhead));
    out.push_str(&line(&format!("Profit{}", rate(est.markup.as_ref().map(|m| &m.profit))), &est.profit));
    out.push_str(&line("PRICE", &est.price));
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

fn read(path: &str) -> Result<(plumbline_world::Job, file::LoadReport), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    file::load(&text).map_err(|e| format!("{path}: {e}"))
}

fn run(args: &[String]) -> Result<(), String> {
    let cmd = args.first().map(String::as_str).ok_or_else(usage)?;
    // A command that takes one file takes nothing after it: an argument that would be
    // ignored is refused instead.
    if cmd == "roof" {
        return new_roof(args);
    }
    if matches!(cmd, "sample" | "house" | "priced" | "show" | "check" | "lines" | "takeoff" | "price" | "estimate") && args.len() > 2 {
        return Err(format!("{cmd} takes one file and nothing else; {:?} would be ignored\n{}", args[2], usage()));
    }
    match cmd {
        "sample" | "house" | "priced" => {
            let out = args.get(1).ok_or_else(usage)?;
            let mut job = match cmd {
                "house" => sample::house(),
                "priced" => sample::priced(),
                _ => sample::sample(),
            };
            std::fs::write(out, file::save(&mut job)).map_err(|e| format!("cannot write {out}: {e}"))?;
            say!("wrote {out}");
            Ok(())
        }
        "show" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, report) = read(path)?;
            // Written in one piece, and a closed pipe (`| head`) is not an error.
            let _ = std::io::stdout().write_all(show(&mut job, &report).as_bytes());
            Ok(())
        }
        "check" => {
            let path = args.get(1).ok_or_else(usage)?;
            let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
            let (mut job, report) = file::load(&text).map_err(|e| format!("{path}: {e}"))?;
            if !report.stale.is_empty() {
                return Err(format!("measures in the file do not match the geometry for: {}", report.stale.join(", ")));
            }
            if file::save(&mut job) != text {
                return Err("the file is not byte-for-byte what saving this job writes".into());
            }
            say!("{path}: loads, measures match the geometry, and saving it back writes the same bytes");
            Ok(())
        }
        "lines" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(lines(&mut job).as_bytes());
            Ok(())
        }
        "takeoff" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(takeoff(&mut job).as_bytes());
            Ok(())
        }
        "price" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(price(&mut job).as_bytes());
            Ok(())
        }
        "estimate" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(estimate(&mut job).as_bytes());
            Ok(())
        }
        // A material the job can be made of: its name and the unit it is bought and priced by.
        "material" => {
            let [_, path, name, unit] = args else { return Err(usage()) };
            let (mut job, _) = read(path)?;
            let made = job.add_material(name, unit)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("added material {name:?} as {made}, priced by the {unit}; its price is not entered yet");
            Ok(())
        }
        // A cost line typed by hand onto an element (or onto the job, for general conditions).
        "line" => {
            let [_, path, element, description, quantity, rest @ ..] = args else { return Err(usage()) };
            let (mut class, mut of_material) = (None, false);
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--class" => class = Some(plumbline_world::Class::parse(it.next().ok_or("--class needs a class")?)?),
                    "--of-material" => of_material = true,
                    other => return Err(format!("unexpected {other:?}\n{}", usage())),
                }
            }
            let (mut job, _) = read(path)?;
            job.add_line(element, description, quantity, class, of_material)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("added to {element}: {description:?} = {quantity}{}", if of_material { ", made of the element's material".to_string() } else { ", with no price yet".to_string() });
            Ok(())
        }
        // Which row of the recap a line lands in; and a Labor line switched off or on.
        "class" | "labor" => {
            let [_, path, element, value, rest @ ..] = args else { return Err(usage()) };
            let line = match rest {
                [] => None,
                [flag, line] if flag == "--line" => Some(line.as_str()),
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            if cmd == "class" {
                let class = plumbline_world::Class::parse(value)?;
                job.set_class(element, line, class)?;
                say!("{element}{}: class is {}", line.map(|l| format!(" {l:?}")).unwrap_or_default(), class.title());
            } else {
                let off = match value.as_str() {
                    "off" => true,
                    "on" => false,
                    other => return Err(format!("{other:?} is not on or off")),
                };
                job.set_off(element, line, off)?;
                say!("{element}{}: labor is {value}", line.map(|l| format!(" {l:?}")).unwrap_or_default());
            }
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            Ok(())
        }
        "apply" => {
            let [_, path, element, condition, rest @ ..] = args else { return Err(usage()) };
            let source = match rest {
                [flag, source] if flag == "--source" => source,
                _ => return Err("--source is required: say who says this element is that condition".into()),
            };
            let (mut job, report) = read(path)?;
            if !report.stale.is_empty() {
                say!("WARNING: before this edit, the file's measures did not match its geometry for: {}. They have been recomputed.", report.stale.join(", "));
            }
            let added = job.apply(element, condition, source)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{element} is now {:?} and has gained {} cost lines:", job.shown(element, "condition", None).unwrap_or_default(), added.len());
            for d in added {
                say!("  {d}");
            }
            Ok(())
        }
        "set" => {
            let [_, path, element, field, value, rest @ ..] = args else { return Err(usage()) };
            let mut status = None;
            let mut source = None;
            let mut line = None;
            let mut supplier: Option<String> = None;
            let mut with: Vec<(String, String)> = Vec::new();
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--status" | "--source" | "--line" if (a == "--status" && status.is_some()) || (a == "--source" && source.is_some()) || (a == "--line" && line.is_some()) => return Err(format!("{a} given twice")),
                    "--line" => line = Some(it.next().cloned().ok_or("--line needs the cost line's description")?),
                    "--supplier" => supplier = Some(it.next().cloned().ok_or("--supplier needs the supplier's name")?),
                    "--with" => {
                        let pair = it.next().ok_or("--with needs name=value, like --with \"turn_up=8 in\"")?;
                        let (name, typed) = pair.split_once('=').ok_or_else(|| format!("--with {pair:?} is not name=value"))?;
                        if with.iter().any(|(have, _)| have == name.trim()) {
                            return Err(format!("--with gives {:?} twice", name.trim()));
                        }
                        with.push((name.trim().to_string(), typed.trim().to_string()));
                    }
                    "--status" => status = Some(it.next().cloned().ok_or("--status needs a value")?),
                    "--source" => source = Some(it.next().cloned().ok_or("--source needs a value")?),
                    other => return Err(format!("unexpected {other:?}\n{}", usage())),
                }
            }
            let status = match status.as_deref() {
                Some("Known") => Status::Known,
                Some("Assumed") => Status::Assumed,
                Some("Unknown") => Status::Unknown,
                Some(other) => return Err(format!("{other:?} is not a status; use Known, Assumed or Unknown")),
                None => return Err("--status is required: say whether the value is Known, Assumed or Unknown".into()),
            };
            let source = source.ok_or("--source is required: say where the value came from")?;
            let value = if value == "unknown" { None } else { Some(value.as_str()) };
            let (mut job, report) = read(path)?;
            if !report.stale.is_empty() {
                say!("WARNING: before this edit, the file's measures did not match its geometry for: {}. They have been recomputed.", report.stale.join(", "));
            }
            // A roof solved from a footprint: change the footprint and solve again.
            let on_footprint = field.starts_with("edge:") || field.starts_with("move:") || (field == "eave_height" && job.find(element).is_some_and(|e| job.world.get::<Footprint>(e).is_some()));
            if on_footprint {
                let typed = value.ok_or("mark an edge unknown in the file; a roof with an edge of no pitch cannot be solved")?;
                let number = |text: &str| text.parse::<usize>().map_err(|_| format!("{text:?} is not an edge number"));
                let change = if let Some(n) = field.strip_prefix("edge:") {
                    let rise = if typed.trim().eq_ignore_ascii_case("gable") { Val { value: Some(Rise::Gable), unit: String::new(), status, source: source.clone() } } else { Val { value: Some(Rise::Pitch(units::parse_pitch(typed)?)), unit: units::PITCH.into(), status, source: source.clone() } };
                    solve::Change::Rise { edge: number(n)?, rise }
                } else if let Some(n) = field.strip_prefix("move:") {
                    let (len, unit) = units::parse_length(typed)?;
                    solve::Change::MoveEdge { edge: number(n)?, by_ft: len.feet(unit)?, status, source: source.clone() }
                } else {
                    let (len, unit) = units::parse_length(typed)?;
                    solve::Change::EaveHeight(Val { value: Some(len), unit: unit.into(), status, source: source.clone() })
                };
                let before = file::save(&mut job);
                let solved = solve::edit(&mut job, element, change)?;
                let after = file::save(&mut job);
                std::fs::write(path, &after).map_err(|e| format!("cannot write {path}: {e}"))?;
                say!("set {element} {field} = {typed} ({status:?}) and solved the roof again");
                say_solve(&solved);
                if before == after {
                    say!("nothing changed");
                }
                return Ok(());
            }
            if !with.is_empty() && field != "quantity" {
                return Err("--with goes with a new quantity formula: it gives a value to a name the formula brings in".into());
            }
            if (field == "quote") != supplier.is_some() {
                return Err("a quote is a supplier's price for a material: set <material> quote <price> --supplier \"<name>\"".into());
            }
            if let Some(supplier) = &supplier {
                job.set_quote(element, supplier, value, status, &source)?;
                job.derive();
                std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
                say!("set {element} quote from {supplier} = {} ({status:?})", value.unwrap_or("unknown"));
                return Ok(());
            }
            let units_before = job.line_units(element, line.as_deref());
            let dropped = if field == "quantity" {
                let formula = value.ok_or("a quantity is a formula; it cannot be marked unknown")?;
                job.set_formula(element, line.as_deref(), formula, &with, status, &source)?
            } else {
                job.set_on_line(element, field, line.as_deref(), value, status, &source)?;
                Vec::new()
            };
            let changed = job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            // Reported back as the job now holds it, in the unit it was entered in.
            let shown = job.shown(element, field, line.as_deref()).unwrap_or_else(|| value.unwrap_or("unknown").to_string());
            if field == "quantity" {
                say!("set {element} {field} = {shown}");
                for (name, _) in &with {
                    say!("  with {name} = {} ({status:?})", job.shown(element, name, line.as_deref()).unwrap_or_default());
                }
                for name in &dropped {
                    say!("  {name} is no longer used by the formula and has been taken off the line");
                }
            } else {
                say!("set {element} {field} = {shown} ({status:?})");
            }
            // A line that now comes out in another unit, or is priced by another, says so.
            if let (Some((was, was_priced)), Some((now, now_priced))) = (units_before, job.line_units(element, line.as_deref())) {
                if was != now {
                    say!("  the line now comes out in {now}; it came out in {was}");
                }
                if was_priced != now_priced {
                    say!("  it is now priced in {now_priced}; it was priced in {was_priced}");
                }
            }
            if changed.is_empty() && !report.stale.is_empty() {
                say!("no measure changed beyond those recomputed above");
            } else if changed.is_empty() {
                say!("no measure changed");
            } else {
                say!("measures re-derived on: {}", changed.join(", "));
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn show(job: &mut plumbline_world::Job, report: &file::LoadReport) -> String {
    let mut out = String::new();
    let elements = view::elements(job);
    let _ = writeln!(out, "{}", job.header.id);
    let _ = writeln!(out, "flags:  ~ Assumed    ? Unknown    (no mark) Known        = worked out from other values");
    let _ = writeln!(out);
    for el in &elements {
        let pad = "  ".repeat(el.depth);
        let _ = writeln!(out, "{pad}{}  {}  [{:?}]", el.path, el.name, el.kind);
        for n in &el.notes {
            let _ = writeln!(out, "{pad}      {n}");
        }
        for r in &el.rows {
            let eq = if r.derived { "=" } else { " " };
            let _ = writeln!(out, "{pad}    {eq} {} {:<16} {}", r.status.flag(), r.label, r.text);
        }
    }
    let (known, assumed, unknown) = view::tally(&elements);
    let _ = writeln!(out);
    let _ = writeln!(out, "{known} Known, {assumed} Assumed, {unknown} Unknown");
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
            let _ = writeln!(out, "  {} {} {} {}: {} ({})", r.status.flag(), el.path, el.name, r.label, r.text, why);
        }
    }
    if !report.stale.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "WARNING: the file's measures did not match its geometry for: {}. The figures above are the recomputed ones.", report.stale.join(", "));
    }
    out
}

/// The lines the job's planes imply, set against the lines that were entered. It reads
/// the job and changes nothing.
fn lines(job: &mut plumbline_world::Job) -> String {
    let mut out = String::new();
    let (planes, left_out) = roof::solid_planes(job);
    let found = roof::lines_of(&planes);
    let cmp = roof::compare(job, &found);
    let point = |p: [f64; 2]| format!("({}, {})", p[0], p[1]);
    let _ = writeln!(out, "{} planes placed; they imply {} lines (coordinates in feet):", planes.len(), found.lines.len());
    for l in &found.lines {
        let _ = writeln!(out, "  {} {:<7} {} to {}  on {}", l.status.flag(), format!("{:?}", l.kind), point(l.ends[0]), point(l.ends[1]), l.planes.join(" and "));
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "{} entered lines agree with the planes.", cmp.agree.len());
    if !cmp.entered_only.is_empty() {
        let _ = writeln!(out, "Entered, but not what the planes imply:");
        for e in &cmp.entered_only {
            let _ = writeln!(out, "  {e}");
        }
    }
    if !cmp.implied_only.is_empty() {
        let _ = writeln!(out, "Implied by the planes, but not entered:");
        for l in &cmp.implied_only {
            let _ = writeln!(out, "  {:?} {} to {} on {}", l.kind, point(l.ends[0]), point(l.ends[1]), l.planes.join(" and "));
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

fn say_solve(report: &solve::Report) {
    say!("  {} planes, {} lines", report.planes, report.lines);
    if !report.added.is_empty() {
        say!("  new: {}", report.added.join(", "));
    }
    for moved in &report.renumbered {
        say!("  {moved}");
    }
    for (path, carried) in &report.removed {
        say!("  gone: {path}{}", if *carried { "  (it carried a condition or cost lines, which went with it)" } else { "" });
    }
    for note in &report.notes {
        say!("  note: {note}");
    }
}

/// `plumbline roof <file> <name> --points "x,y x,y ..." --pitch <p> --eave-height <len> --status <S> --source <where>`
fn new_roof(args: &[String]) -> Result<(), String> {
    let [_, path, name, rest @ ..] = args else { return Err(usage()) };
    let (mut points, mut pitch, mut eave, mut status, mut source) = (None, None, None, None, None);
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        let slot = match flag.as_str() {
            "--points" => &mut points,
            "--pitch" => &mut pitch,
            "--eave-height" => &mut eave,
            "--status" => &mut status,
            "--source" => &mut source,
            other => return Err(format!("unexpected {other:?}\n{}", usage())),
        };
        if slot.is_some() {
            return Err(format!("{flag} given twice"));
        }
        *slot = Some(it.next().cloned().ok_or_else(|| format!("{flag} needs a value"))?);
    }
    let status = match status.as_deref() {
        Some("Known") => Status::Known,
        Some("Assumed") => Status::Assumed,
        Some(other) => return Err(format!("{other:?} is not a status a roof can be entered with; use Known or Assumed")),
        None => return Err("--status is required: say whether the footprint is Known or Assumed".into()),
    };
    let source = source.ok_or("--source is required: say where the footprint came from")?;
    let mut pts = Vec::new();
    for pair in points.ok_or("--points is required: the footprint's corners in feet, like \"0,0 40,0 40,30 0,30\"")?.split_whitespace() {
        let (x, y) = pair.split_once(',').ok_or_else(|| format!("{pair:?} is not a corner; write it x,y in feet"))?;
        pts.push([units::parse_number(x)?, units::parse_number(y)?]);
    }
    let rise = units::parse_pitch(&pitch.ok_or("--pitch is required: the pitch every edge starts at, like 6:12")?)?;
    let (len, unit) = units::parse_length(&eave.ok_or("--eave-height is required, like \"10 ft\"")?)?;
    let footprint = Footprint {
        edges: pts.iter().map(|_| Val { value: Some(Rise::Pitch(rise)), unit: units::PITCH.into(), status, source: source.clone() }).collect(),
        points: Val { value: Some(pts), unit: units::FT.into(), status, source: source.clone() },
        eave_height: Val { value: Some(len), unit: unit.into(), status, source: source.clone() },
    };
    let (mut job, _) = read(path)?;
    let (roof_path, report) = solve::new_roof(&mut job, name, footprint)?;
    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
    say!("{roof_path} {name:?} solved from its footprint");
    say_solve(&report);
    let _: Option<Len> = None;
    Ok(())
}
