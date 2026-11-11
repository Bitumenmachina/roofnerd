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
    "usage:\n  plumbline sample <file>\n  plumbline house <file>\n  plumbline show <file>\n  plumbline set <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source \"<where from>\" [--line \"<cost line>\"] [--with \"name=value\"]\n  plumbline check <file>\n  plumbline lines <file>\n  plumbline takeoff <file>\n  plumbline apply <file> <path> <condition> --source \"<who says>\"\n\nfields: pitch (6, 6:12, 1/4:12), height, eave_height, stretch_out (2.5 ft, 30 in, 19 1/2\", 25' 6 1/2\"), condition, wbs\non a cost line (--line): stretch_out, quantity (\"plan_length * stretch_out\"), waste (10), order (\"30 SF/sheet\", \"1.25 lb/SF\"), or a value the formula names (spacing \"12 in\")".to_string()
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
        let _ = writeln!(out, "\n{}{}  ({what}; {} runs, {})", if t.flag == Status::Known { String::new() } else { format!("{} ", t.flag.flag()) }, t.name, t.runs.members, said(&t.runs));
        for line in &t.lines {
            let _ = writeln!(out, "  {} {:<8} {:<16} {:>2} runs  {:>12}", line.size_status.max(line.length.status).flag(), format!("{:?}", line.kind), line.size, line.length.members, said(&line.length));
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
    if matches!(cmd, "sample" | "house" | "show" | "check" | "lines" | "takeoff") && args.len() > 2 {
        return Err(format!("{cmd} takes one file and nothing else; {:?} would be ignored\n{}", args[2], usage()));
    }
    match cmd {
        "sample" | "house" => {
            let out = args.get(1).ok_or_else(usage)?;
            let mut job = if cmd == "house" { sample::house() } else { sample::sample() };
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
            let mut with: Vec<(String, String)> = Vec::new();
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--status" | "--source" | "--line" if (a == "--status" && status.is_some()) || (a == "--source" && source.is_some()) || (a == "--line" && line.is_some()) => return Err(format!("{a} given twice")),
                    "--line" => line = Some(it.next().cloned().ok_or("--line needs the cost line's description")?),
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
