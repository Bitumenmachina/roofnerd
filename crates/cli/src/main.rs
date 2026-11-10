//! The command line over the job world. It reads and edits the same saved job the
//! desktop application will.
//!
//!   plumbline sample <file>                      write the sample job
//!   plumbline show   <file>                      every element and value, flagged
//!   plumbline set    <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source "<where from>" [--line "<cost line>"]
//!
//! A value is typed the way the drawing or the tape gives it and is kept in that unit:
//! a length as `2.5 ft`, `30 in`, `19 1/2"` or `25' 6 1/2"`; a pitch as `6`, `6:12` or
//! `1/4:12`. A length with no unit is refused, never read as feet.
//!
//!   plumbline check  <file>                      load, re-derive, confirm the file is exactly what a save would write
//!   plumbline lines  <file>                      the lines the planes imply, set against the lines entered

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;

use plumbline_world::value::Status;
use plumbline_world::{file, roof, sample, view};

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
    "usage:\n  plumbline sample <file>\n  plumbline show <file>\n  plumbline set <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source \"<where from>\" [--line \"<cost line>\"]\n  plumbline check <file>\n  plumbline lines <file>\n\nfields: pitch (6, 6:12, 1/4:12), height, eave_height, stretch_out (2.5 ft, 30 in, 19 1/2\", 25' 6 1/2\"), condition, wbs".to_string()
}

fn read(path: &str) -> Result<(plumbline_world::Job, file::LoadReport), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    file::load(&text).map_err(|e| format!("{path}: {e}"))
}

fn run(args: &[String]) -> Result<(), String> {
    let cmd = args.first().map(String::as_str).ok_or_else(usage)?;
    match cmd {
        "sample" => {
            let out = args.get(1).ok_or_else(usage)?;
            let mut job = sample::sample();
            std::fs::write(out, file::save(&mut job)).map_err(|e| format!("cannot write {out}: {e}"))?;
            println!("wrote {out}");
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
            println!("{path}: loads, measures match the geometry, and saving it back writes the same bytes");
            Ok(())
        }
        "lines" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(lines(&mut job).as_bytes());
            Ok(())
        }
        "set" => {
            let [_, path, element, field, value, rest @ ..] = args else { return Err(usage()) };
            let mut status = None;
            let mut source = None;
            let mut line = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--status" | "--source" | "--line" if (a == "--status" && status.is_some()) || (a == "--source" && source.is_some()) || (a == "--line" && line.is_some()) => return Err(format!("{a} given twice")),
                    "--line" => line = Some(it.next().cloned().ok_or("--line needs the cost line's description")?),
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
                println!("WARNING: before this edit, the file's measures did not match its geometry for: {}. They have been recomputed.", report.stale.join(", "));
            }
            job.set_on_line(element, field, line.as_deref(), value, status, &source)?;
            let changed = job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            // Reported back as the job now holds it, in the unit it was entered in.
            let shown = job.shown(element, field, line.as_deref()).unwrap_or_else(|| value.unwrap_or("unknown").to_string());
            println!("set {element} {field} = {shown} ({status:?})");
            if changed.is_empty() && !report.stale.is_empty() {
                println!("no measure changed beyond those recomputed above");
            } else if changed.is_empty() {
                println!("no measure changed");
            } else {
                println!("measures re-derived on: {}", changed.join(", "));
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
        let _ = writeln!(out, "Planes that do not fit each other:");
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
