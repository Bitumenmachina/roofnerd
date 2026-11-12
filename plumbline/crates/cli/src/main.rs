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
use plumbline_world::{file, plans, roof, sample, solve, view, Footprint, Rise, Val};

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
    "usage:\n  plumbline sample <file>\n  plumbline house <file>\n  plumbline sixteen <file>   (the sample whole house with a made-up price on everything, in all sixteen divisions)\n  plumbline show <file>\n  plumbline set <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source \"<where from>\" [--line \"<cost line>\"] [--with \"name=value\"]\n  plumbline check <file>\n  plumbline lines <file>\n  plumbline takeoff <file>\n  plumbline drawings <file>\n  plumbline clashes <file> [--clearance \"2 in\"]\n  plumbline vif <file>   (what is still to be verified in the field, and the dollars riding on each)\n  plumbline price <file>\n  plumbline estimate <file>\n  plumbline priced <file>\n  plumbline material <file> \"<name>\" <unit it is bought by: lb, sheet, SF>\n  plumbline line <file> <path> \"<description>\" <quantity> [--cost-type <material|labor|...>] [--division <1 to 16>] [--of-material]\n  plumbline division <file> <path> <1 to 16, or its title> [--line \"<cost line>\"]\n  plumbline class <file> <path> <material|labor|fabrication|supervision|subcontract|general> [--line \"<cost line>\"]\n  plumbline labor <file> <path> <on|off> [--line \"<cost line>\"]\n  plumbline apply <file> <path> <condition> --source \"<who says>\"\n  plumbline export <prices|assemblies> <file> <out.csv>\n  plumbline export estimate <file> <out.xlsx|out.csv>   (.xlsx: a workbook whose amounts, recap and price are formulas; .csv: the lines as values)\n  plumbline export ifc <file> <out.ifc>   (the building as an IFC4 model, for another trade or a designer)\n  plumbline export <supplier|subcontractor|pm> <file> <out.xlsx|out.csv>   (one lens on the estimate: what to quote; the subcontract scope; the lines, budget and what is to verify)\n  plumbline import prices <file> <a price list: CSV or XLSX> [--sheet \"<name>\"] [--supplier \"<whose prices>\"] [--add] [--status <Known|Assumed> --source \"<where from>\"]\n  plumbline import assemblies <file> <an assembly library this program wrote: CSV or XLSX> [--sheet \"<name>\"]\n  plumbline import prior <file> <an estimate made before: CSV or XLSX> [--sheet \"<name>\"] [--name \"<what to call it>\"] --status <Known|Assumed> --source \"<where from>\"\n  plumbline import ifc <file> <a model: IFC2x3 or IFC4> --status <Known|Assumed> --source \"<where from>\"\n  plumbline compare <file>\n\ndrawing (each but new and system takes --status <Known|Assumed> --source \"<where from>\"; points are x,y in feet):\n  plumbline new <file> \"<title>\"\n  plumbline storey <file> \"<name>\" --elevation \"0 ft\" --height \"9 ft\"\n  plumbline type <file> wall \"<name>\" --thickness \"6 in\"\n  plumbline type <file> floor \"<name>\" --thickness \"4 in\"\n  plumbline type <file> <door|window> \"<name>\" --width \"3 ft\" --height \"7 ft\"\n  plumbline type <file> <duct|pipe|conduit|cable> \"<name>\" (--round \"6 in\" | --width \"12 in\" --depth \"8 in\")\n  plumbline system <file> \"<name>\" \"<Supply Air|Domestic Cold Water|...>\"\n  plumbline wall <file> <storey> <type> --points \"0,0 20,0 20,10\" [--closed] (--height \"9 ft\" | --top <storey>)\n  plumbline <door|window> <file> <wall> <type> (--offset \"5 ft\" | --at \"5,0\") --sill \"0 ft\"\n  plumbline floor <file> <storey> <type> (--walls | --points \"0,0 30,0 30,20 0,20\")   (--walls: the outline the storey's walls enclose)\n  plumbline room <file> <storey> \"<name>\" --at \"6,8\"\n  plumbline run <file> <storey> <duct|pipe|conduit|cable> <type> --points \"2,2 14,2\" --height \"8 ft\" [--system <system>]\n  plumbline place <file> <storey> <fixture|equipment> \"<what>\" --at \"14,7\" --offset \"0 ft\" [--size \"2,2,6\"] [--system <system>]\n  plumbline drawing <file> <storey> \"<name>\" --file <a PDF, PNG or JPEG of the plan> [--page 1]\n  plumbline calibrate <file> <drawing> --from \"126,90\" --to \"666,90\" --distance \"30 ft\"   (two places on the page and the dimension between them)\n  plumbline lineup <file> <drawing> --page-point \"126,126\" --plan-point \"0,0\"\n  tracing: wall, floor, door, window, room, run and place take --on <drawing> in place of --status and --source; their points are then places on the drawing's page\n  plumbline move <file> <path> --by \"3,2\"\n  plumbline measure <file> <path> \"<the value, as it is listed: storey height, sill, ...>\" <what was read: 8' 10 1/2\", or same> [--to <bottom|centre|top>] --source \"<who measured, or with what>\"\n  plumbline remove <file> <path>\n  plumbline fitting <file> <storey> --at \"10,2\" [--height \"8 ft\"] [--kind <bend|tee|cross|transition|coupling>]\n  plumbline fitting <file> --all [--system <system>]\n  plumbline connect <file> <fixture or equipment> <run>\n  plumbline split <file> <run> --at \"6,2\"\n\nfields: pitch (6, 6:12, 1/4:12), height, eave_height, stretch_out (2.5 ft, 30 in, 19 1/2\", 25' 6 1/2\"), condition, wbs, allowance (28, $28.00/SF), material (a material's name)\non the job: overhead (10), profit (8)\non a material: price ($4.10/lb), order (\"1.25 lb/SF\", \"30 SF/sheet\"), quote ($4.25/lb --supplier \"<name>\")\non a cost line (--line): stretch_out, quantity (\"plan_length * stretch_out\"), waste (10), order (\"30 SF/sheet\", \"1.25 lb/SF\"), unit_cost (4.25, $4.25/SF), or a value the formula names (spacing \"12 in\")".to_string()
}

/// What a page is measured in until its drawing has a scale.
fn page_unit(sort: plumbline_plans::Sort) -> &'static str {
    if sort == plumbline_plans::Sort::Pdf { "points" } else { "dots" }
}

fn lines_said(n: usize) -> String {
    match n {
        0 => "no lines to snap to".to_string(),
        1 => "1 straight line to snap to".to_string(),
        n => format!("{n} straight lines to snap to"),
    }
}

/// A plan file's path as the job keeps it: from the folder the job is in.
fn from_the_jobs_folder(job: &str, file: &str) -> Result<String, String> {
    use std::path::{Component, Path, PathBuf};
    let folder = Path::new(job).parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let folder = folder.canonicalize().map_err(|e| format!("cannot find the job's folder: {e}"))?;
    let file = Path::new(file).canonicalize().map_err(|e| format!("cannot find {file}: {e}"))?;
    let (a, b): (Vec<Component>, Vec<Component>) = (folder.components().collect(), file.components().collect());
    let shared = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut out = PathBuf::new();
    for _ in shared..a.len() {
        out.push("..");
    }
    for part in &b[shared..] {
        out.push(part);
    }
    out.to_str().map(str::to_string).ok_or_else(|| "the plan file's path is not text this program can keep".to_string())
}

/// Each drawing in the job: its file and page, its scale, and whether the file is still
/// the one the drawing was made from. The file is read again here, so what it cannot
/// give is said again.
fn drawings(job: &mut plumbline_world::Job, path: &str) -> String {
    let folder = std::path::Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new(".")).to_path_buf();
    let all: Vec<(String, String, plans::Drawing)> = {
        let mut q = job.world.query::<(&plumbline_world::Node, &plans::Drawing)>();
        let mut all: Vec<(String, String, plans::Drawing)> = q.iter(&job.world).map(|(n, d)| (n.path.clone(), n.name.clone(), d.clone())).collect();
        all.sort_by(|a, b| a.0.cmp(&b.0));
        all
    };
    let mut out = String::new();
    if all.is_empty() {
        let _ = writeln!(out, "No drawings: nothing is laid under the tracing.");
    }
    for (path, name, d) in all {
        let file = d.file.value.clone().unwrap_or_default();
        let _ = writeln!(out, "{} {name} [{path}]: page {} of {file}", d.status().flag(), d.page);
        match (&d.scale, d.feet_per_unit()) {
            (Some(s), Some(k)) => {
                let _ = writeln!(out, "    scale: {} between ({}, {}) and ({}, {}) on its page: {} to the foot  ({})", s.distance.value.as_ref().map(|l| units::show_length(l, &s.distance.unit)).unwrap_or_default(), view::num(s.from[0]), view::num(s.from[1]), view::num(s.to[0]), view::num(s.to[1]), view::num((1.0e4 / k).round() / 1.0e4), s.distance.source);
            }
            _ => {
                let _ = writeln!(out, "    scale: none. Nothing can be traced over it until it is calibrated from one dimension.");
            }
        }
        for then in plans::stale(job, &path).unwrap_or_default() {
            if then.rescaled {
                let _ = writeln!(out, "    TRACED AT AN EARLIER SCALE ({} to the foot) and not moved: {}", view::num((1.0e4 / then.feet_per_unit).round() / 1.0e4), then.traced.join(", "));
            } else {
                let _ = writeln!(out, "    TRACED BEFORE THE SHEET WAS MOVED and not moved with it: {}", then.traced.join(", "));
            }
        }
        match std::fs::read(folder.join(&file)) {
            Err(e) => {
                let _ = writeln!(out, "    file: cannot be read ({e}). The drawing cannot be shown.");
            }
            Ok(bytes) => {
                if bytes.len() as u64 != d.bytes || plumbline_plans::checksum(&bytes) != d.checksum {
                    let _ = writeln!(out, "    file: HAS CHANGED since this drawing was made from it. What was traced over it may no longer lie on it.");
                } else {
                    let _ = writeln!(out, "    file: as it was when the drawing was made");
                }
                match plumbline_plans::read(&bytes, d.page) {
                    Err(why) => {
                        let _ = writeln!(out, "    page: cannot be read: {why}");
                    }
                    Ok(page) => {
                        let _ = writeln!(out, "    page: {}, {} by {} {}, {}", page.sort.said(), view::num(page.size[0]), view::num(page.size[1]), page_unit(page.sort), lines_said(page.lines.len()));
                        if (page.size[0] - d.size[0]).abs() > 0.01 || (page.size[1] - d.size[1]).abs() > 0.01 {
                            let _ = writeln!(out, "    page: IS NOT THE SIZE THIS DRAWING WAS MADE AT, {} by {}. The sheet does not fit what was traced over it; it is not shown.", view::num(d.size[0]), view::num(d.size[1]));
                        }
                        for lost in &page.lost {
                            let _ = writeln!(out, "    not read: {lost}");
                        }
                    }
                }
            }
        }
    }
    out
}

/// What is in the same space as what, what passes through a wall or a floor, and, when a
/// clearance is asked for, what is closer than it. Lengths are in inches, to two places.
/// The verify-in-field list as text: each question, where it is, what to measure or
/// check, and the dollars riding on it, largest first.
fn vif(job: &mut plumbline_world::Job) -> String {
    let list = plumbline_world::vif::vif(job);
    let money = |dollars: Option<f64>, status: Status| match dollars {
        Some(x) => format!("{}${}", status.flag().trim(), view::figure(x, 2)),
        None => "no dollars".to_string(),
    };
    let lines = |n: usize| if n == 1 { "1 line".to_string() } else { format!("{n} lines") };
    let mut out = String::new();
    let n = list.questions.len();
    let _ = match (n, list.riding) {
        (0, _) => writeln!(out, "Verify in field: nothing. Every value that can be measured is Known, and no clash is a question."),
        (_, Some(x)) => writeln!(out, "Verify in field: {n} {}. {} rides on them ({} of the estimate, each counted once).", if n == 1 { "question" } else { "questions" }, money(Some(x), list.riding_status), lines(list.riding_lines)),
        (_, None) => writeln!(out, "Verify in field: {n} {}. Nothing priced rides on {}.", if n == 1 { "question" } else { "questions" }, if n == 1 { "it" } else { "them" }),
    };
    if let Some(why) = &list.clashes_not_tested {
        let _ = writeln!(out, "  The clashes are not on this list: they could not be worked out ({why}).");
    }
    for q in &list.questions {
        let on = if q.storey.is_empty() || q.kind == plumbline_world::Kind::Storey { String::new() } else { format!(", on {}", q.storey) };
        let riding = match q.dollars {
            Some(_) => format!("{}, {}", money(q.dollars, q.dollars_status), lines(q.lines)),
            None => "no dollars riding on it".to_string(),
        };
        match &q.clash {
            Some(c) => {
                let by = match (c.how, c.by) {
                    (plumbline_world::building::clash::ClashKind::Hard, Some(by)) => format!("in the same space by {:.2} in", by * 12.0 + 0.0),
                    (plumbline_world::building::clash::ClashKind::Hard, None) => "in the same space".to_string(),
                    (_, Some(by)) => format!("{:.2} in apart", by * 12.0 + 0.0),
                    (_, None) => "too close".to_string(),
                };
                let _ = writeln!(out, "  {} [{}] and {} [{}] {by}{on}, at ({}, {}, {}): {riding}", q.name, q.path, c.other_name, c.other, view::num(c.at[0]), view::num(c.at[1]), view::num(c.at[2]));
            }
            None => {
                let what = |length: bool| q.values.iter().filter(|v| v.2 == length).map(|v| format!("{}{}", v.1.flag().trim(), v.0)).collect::<Vec<_>>().join(", ");
                let (measure, check) = (what(true), what(false));
                let mut to_do = Vec::new();
                if !check.is_empty() {
                    to_do.push(format!("check: {check}"));
                }
                if !measure.is_empty() {
                    to_do.push(format!("measure: {measure}"));
                }
                let _ = writeln!(out, "  {} [{}] ({}){on}: {riding}", q.name, q.path, format!("{:?}", q.kind).to_lowercase());
                let _ = writeln!(out, "      {}", to_do.join("; "));
            }
        }
    }
    out
}

fn clashes(job: &mut plumbline_world::Job, clearance: Option<f64>) -> Result<String, String> {
    use plumbline_world::building::clash::{self, ClashKind};
    let found = clash::clashes(job, clearance)?;
    let inches = |feet: f64| format!("{:.2} in", feet * 12.0 + 0.0);
    let place = |c: &clash::Clash| format!("({}, {}, {})", view::num(c.at[0]), view::num(c.at[1]), view::num(c.at[2]));
    let pairs = |c: &clash::Clash| format!("{} {} [{}] and {} [{}]", c.status.flag(), c.a_name, c.a, c.b_name, c.b);
    // Between things that are Known a clash is a finding; where either rests on something
    // assumed it is a question for the field.
    let assumed = |list: &[&clash::Clash]| {
        let field = list.iter().filter(|c| c.status != Status::Known).count();
        let found = list.len() - field;
        let mut parts = Vec::new();
        if found > 0 {
            parts.push(format!("{found} {}", if found == 1 { "finding" } else { "findings" }));
        }
        if field > 0 {
            parts.push(format!("{field} to verify in the field"));
        }
        if parts.is_empty() { String::new() } else { format!(" ({})", parts.join(", ")) }
    };
    let mut out = String::new();
    let hard: Vec<&clash::Clash> = found.found.iter().filter(|c| c.how == ClashKind::Hard && !c.through).collect();
    let through: Vec<&clash::Clash> = found.found.iter().filter(|c| c.how == ClashKind::Hard && c.through).collect();
    let close: Vec<&clash::Clash> = found.found.iter().filter(|c| c.how == ClashKind::Clearance).collect();
    let _ = writeln!(out, "In the same space: {}{}", hard.len(), assumed(&hard));
    for c in &hard {
        let by = match c.by {
            Some(by) => format!("in each other by {}", inches(by)),
            None => "in each other; by how much could not be worked out".to_string(),
        };
        let _ = writeln!(out, "  {}: {by}, at {}", pairs(c), place(c));
    }
    let _ = writeln!(out, "Through a wall or a floor (a hole to cut): {}{}", through.len(), assumed(&through));
    for c in &through {
        let _ = writeln!(out, "  {}, at {}", pairs(c), place(c));
    }
    match clearance {
        None => {
            let _ = writeln!(out, "Closer than a clearance: not asked for (--clearance \"2 in\")");
        }
        Some(asked) => {
            let _ = writeln!(out, "Closer than {}: {}{}", inches(asked), close.len(), assumed(&close));
            for c in &close {
                let _ = writeln!(out, "  {}: {} apart, at {}", pairs(c), inches(c.by.unwrap_or(0.0)), place(c));
            }
        }
    }
    let _ = writeln!(out, "Tested: {} element{}", found.tested, if found.tested == 1 { "" } else { "s" });
    if !found.left_out.is_empty() {
        let _ = writeln!(out, "Not tested: {}", found.left_out.len());
        for (path, why) in &found.left_out {
            let _ = writeln!(out, "  {path}: {why}");
        }
    }
    let _ = writeln!(out, "Never tested:");
    for note in clash::NEVER_TESTED {
        let _ = writeln!(out, "  {note}");
    }
    Ok(out)
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
            Some(x) => format!("{}${} per {}", d.status.flag().trim_end(), view::rate(x), d.unit.trim_start_matches("$/")).trim().to_string(),
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
            let _ = writeln!(out, "    priced by its {name}: {}", d.value.map(|x| format!("{}{} {unit}", d.status.flag().trim_end(), view::figure(x, 2))).unwrap_or_else(|| format!("no figure ({})", if d.note.is_empty() { "a value it is worked out from is not known" } else { d.note.as_str() })));
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
            let diff = plumbline_world::lines::cents(d - a);
            let flag = status.flag().trim_end();
            let said = if diff == 0.0 {
                format!("the detail is {flag}the same")
            } else {
                let pct = if a != 0.0 { format!(" ({flag}{} %)", view::figure(diff / a * 100.0, 2)) } else { String::new() };
                format!("the detail is {flag}{} {}{pct}", money(diff.abs()), if diff > 0.0 { "over" } else { "under" })
            };
            let _ = writeln!(out, "  like for like, over the {n} element{} that have both: allowances {fa}{}, detail {fd}{}: {said}", if n == 1 { "" } else { "s" }, money(a), money(d));
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
        // Hours are shown to the four places they are held and added to: at two, a line's
        // hours would not multiply to its amount, or add to its class.
        let places = |unit: &str| if unit == plumbline_world::HOURS { 4 } else { 2 };
        let priced = match l.priced.value {
            Some(x) => format!("{}{} {}", l.priced.status.flag().trim_end(), view::figure(x, places(&l.priced.unit)), l.priced.unit),
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
            let _ = writeln!(out, "      ? no class: it is in no column and no row of the recap{}", l.amount.value.map(|_| format!(" (it comes to {})", flagged(&l.amount))).unwrap_or_default());
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
    let rate = |v: Option<&plumbline_world::Val<f64>>| v.and_then(|v| v.value.map(|x| format!(" at {}{} %", v.status.flag().trim_end(), view::rate(x)))).unwrap_or_default();
    out.push_str(&line(&format!("Overhead{}", rate(est.markup.as_ref().map(|m| &m.overhead))), &est.overhead));
    out.push_str(&line(&format!("Profit{}", rate(est.markup.as_ref().map(|m| &m.profit))), &est.profit));
    out.push_str(&line("PRICE", &est.price));
    if est.price.value.is_some() && !est.price.note.is_empty() {
        let _ = writeln!(out, "  {:<20}{:>14}   ! {}", "", "", est.price.note);
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
    if est.divisions.iter().any(|d| d.total.members > 0) || est.no_division.members > 0 {
        let _ = writeln!(out, "\nBY DIVISION");
        let lines = |n: usize| format!("{n} line{}", if n == 1 { "" } else { "s" });
        for d in &est.divisions {
            let said = match d.total.value {
                Some(x) => format!("{}{}", d.total.status.flag().trim_end(), money(x)),
                None => "none".to_string(),
            };
            let name = format!("{:>2} {}", d.division.number(), d.division.title());
            let _ = writeln!(out, "  {name:<36}{said:>14}   {}", if d.total.members == 0 { String::new() } else { lines(d.total.members) });
            for (who, why) in &d.total.left_out {
                let _ = writeln!(out, "      ? left out: {who}: {why}");
            }
        }
        let in_all: f64 = est.divisions.iter().filter_map(|d| d.total.value).sum();
        let _ = writeln!(out, "  {:<36}{:>14}", "the sixteen divisions", money(plumbline_world::lines::cents(in_all)));
        if est.no_division.members > 0 {
            let said = est.no_division.value.map(money).unwrap_or_else(|| "no figure".to_string());
            let _ = writeln!(out, "  in no division: {}, {said}: in the job cost, and in none of the sixteen", lines(est.no_division.members));
            for l in est.lines.iter().filter(|l| !l.off && l.class.is_some() && l.division.is_none()) {
                let _ = writeln!(out, "    ? {}: {:?}", l.path, l.description);
            }
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

/// A table written out of the job, or brought into it: `export prices <file> <out.csv>`,
/// `import prices <file> <list.csv or .xlsx> [--sheet "<name>"] [--supplier "<name>"]
/// [--add] [--status <Known|Assumed> --source "<where from>"]`.
fn tables(args: &[String]) -> Result<(), String> {
    use plumbline_world::tables;
    let cmd = args[0].as_str();
    let (mut plain, mut flags): (Vec<&str>, Vec<(&str, Option<&str>)>) = (Vec::new(), Vec::new());
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--add" => flags.push(("--add", None)),
            f @ ("--sheet" | "--supplier" | "--status" | "--source" | "--name") => {
                let v = args.get(i + 1).ok_or_else(|| format!("{f} needs a value after it"))?;
                if flags.iter().any(|(known, _)| *known == f) {
                    return Err(format!("{f} is given twice"));
                }
                flags.push((f, Some(v.as_str())));
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("{cmd} does not take {f}\n{}", usage())),
            a => plain.push(a),
        }
        i += 1;
    }
    let flag = |name: &str| flags.iter().find(|(f, _)| *f == name).and_then(|(_, v)| *v);
    let [what, path, other] = plain.as_slice() else {
        return Err(format!("{cmd} takes what, the job's file and the table's file\n{}", usage()));
    };
    let (mut job, _) = read(path)?;
    // How sure what is brought in is, and where it came from, when the table does not say.
    let how = match (flag("--status"), flag("--source")) {
        (None, None) => None,
        (Some(status), Some(source)) => {
            let status = match status {
                "Known" => Status::Known,
                "Assumed" => Status::Assumed,
                other => return Err(format!("--status {other:?}: what is brought in is Known or Assumed")),
            };
            if source.trim().is_empty() {
                return Err("--source says where it came from; it is empty".into());
            }
            Some((status, source.to_string()))
        }
        _ => return Err("--status and --source go together: how sure it is, and where it came from".into()),
    };
    let table = |sheet: Option<&str>| -> Result<plumbline_sheets::Sheet, String> {
        let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
        plumbline_sheets::read(&bytes, sheet).map_err(|why| format!("{other}: {why}"))
    };
    let only = |allowed: &[&str]| -> Result<(), String> {
        match flags.iter().find(|(f, _)| !allowed.contains(f)) {
            Some((f, _)) => Err(format!("{cmd} {what} does not take {f}")),
            None => Ok(()),
        }
    };
    match (cmd, *what) {
        ("export", "assemblies") => {
            only(&[])?;
            let rows = tables::assemblies_out(&mut job);
            std::fs::write(other, plumbline_sheets::csv(&rows)).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the assemblies of {path}, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "assemblies") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            let brought = tables::assemblies_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        ("import", "ifc") => {
            only(&["--status", "--source"])?;
            let (status, source) = how.clone().ok_or("say how sure the model is and where it came from: --status, --source")?;
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let called = std::path::Path::new(other).file_name().and_then(|s| s.to_str()).unwrap_or("the model");
            let b = plumbline_ifc::import(&mut job, &bytes, called, plumbline_world::editing::Entered { status, source: &source }).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let mut out = format!("{other} ({}): brought in\n", b.schema);
            for (what, n) in &b.made {
                let _ = writeln!(out, "    {n} {what}");
            }
            let _ = writeln!(out, "  not brought in: {}", count(b.left_out.iter().map(|l| l.1).sum(), "thing", "things"));
            for (class, n, why) in &b.left_out {
                let _ = writeln!(out, "    {n} {class}: {why}");
            }
            for note in &b.notes {
                let _ = writeln!(out, "  note: {note}");
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        ("import", "prior") => {
            only(&["--sheet", "--name", "--status", "--source"])?;
            let (status, source) = how.clone().ok_or("say how sure the prior estimate is and where it came from: --status, --source")?;
            let sheet = table(flag("--sheet"))?;
            let name = match flag("--name") {
                Some(n) => n.to_string(),
                None => std::path::Path::new(other).file_stem().and_then(|s| s.to_str()).unwrap_or("prior estimate").to_string(),
            };
            let (made, brought) = tables::prior_in(&mut job, &name, &sheet.rows, Some(&sheet.numbers), plumbline_world::editing::Entered { status, source: &source }).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{made}: {name}");
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        ("export", "prices") => {
            only(&[])?;
            let rows = tables::prices_out(&mut job);
            std::fs::write(other, plumbline_sheets::csv(&rows)).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the price list of {path}, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "prices") => {
            only(&["--sheet", "--supplier", "--add", "--status", "--source"])?;
            let sheet = table(flag("--sheet"))?;
            let entered = how.as_ref().map(|(status, source)| plumbline_world::editing::Entered { status: *status, source });
            let brought = tables::prices_in(&mut job, &sheet.rows, Some(&sheet.numbers), flag("--supplier"), flags.iter().any(|(f, _)| *f == "--add"), entered).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            Ok(())
        }
        // The building as a model another trade's or a designer's program reads.
        ("export", "ifc") => {
            only(&[])?;
            if !other.to_ascii_lowercase().ends_with(".ifc") {
                return Err(format!("{other}: a model is written as an .ifc file; name the file with that"));
            }
            let file = std::path::Path::new(other).file_name().and_then(|f| f.to_str()).unwrap_or("model.ifc");
            let written = plumbline_ifc::write::export(&mut job, file, &plumbline_world::job::now())?;
            std::fs::write(other, &written.text).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the building of {path} as an IFC4 model, lengths in feet");
            // One of a thing is said as one: "1 roof", "1 piece of equipment".
            let one = |what: &str| what.strip_prefix("pieces ").map(|rest| format!("piece {rest}")).unwrap_or_else(|| what.strip_suffix('s').unwrap_or(what).to_string());
            say!("  written: {}", written.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { one(what) } else { what.clone() })).collect::<Vec<_>>().join(", "));
            say!("  every value entered on each goes with it, with how sure it is and where it came from");
            if !written.not_written.is_empty() {
                say!("  not written (a building model has no place for it, or it has no shape):");
                for (what, n, why) in &written.not_written {
                    say!("    {n} x {what}: {why}");
                }
            }
            Ok(())
        }
        // The estimate, or one lens on it, as a workbook (its figures formulas) or as
        // rows of values: by the name of the file asked for.
        ("export", "estimate" | "supplier" | "subcontractor" | "sub" | "pm") => {
            use plumbline_world::export::{self, Lens, Out};
            only(&[])?;
            let workbook = match std::path::Path::new(other).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
                Some("xlsx") => true,
                Some("csv") => false,
                _ => return Err(format!("{other}: what is written is a workbook (.xlsx) or rows of values (.csv); name the file with one of those")),
            };
            let (sheets, said) = match Lens::parse(what) {
                None => (export::estimate_out(&mut job, workbook), "the estimate"),
                Some(Lens::Supplier) => (export::lens_out(&mut job, Lens::Supplier), "the materials to quote, with none of our prices"),
                Some(Lens::Subcontractor) => (export::lens_out(&mut job, Lens::Subcontractor), "the subcontract scope, with none of our prices"),
                Some(Lens::Pm) => (export::lens_out(&mut job, Lens::Pm), "the lines, the budget by class and what is to verify, with no overhead, profit or price"),
            };
            let lines: usize = sheets.first().map(|s| s.rows.len().saturating_sub(s.head + 1)).unwrap_or(0);
            if workbook {
                let tabs: Vec<plumbline_sheets::Tab> = sheets
                    .iter()
                    .map(|s| plumbline_sheets::Tab {
                        name: s.name.clone(),
                        head: s.head,
                        widths: s.widths.clone(),
                        rows: s
                            .rows
                            .iter()
                            .map(|r| {
                                r.iter()
                                    .map(|c| match c {
                                        Out::None => plumbline_sheets::Cell::Empty,
                                        Out::Text(t) => plumbline_sheets::Cell::Text(t.clone()),
                                        Out::Number(x) => plumbline_sheets::Cell::Number(*x),
                                        Out::Dollars(x) => plumbline_sheets::Cell::Dollars(*x),
                                        Out::Formula { text, value } => plumbline_sheets::Cell::Formula { text: text.clone(), value: *value },
                                    })
                                    .collect()
                            })
                            .collect(),
                    })
                    .collect();
                let formulas = sheets.iter().flat_map(|s| s.rows.iter().flatten()).filter(|c| matches!(c, Out::Formula { .. })).count();
                let bytes = plumbline_sheets::xlsx(&tabs)?;
                std::fs::write(other, bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
                say!("wrote {other}: {said} of {path}; {}: {}; {}", count(sheets.len(), "sheet", "sheets"), sheets.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "), if formulas == 0 { "figures, no formulas".to_string() } else { count(formulas, "figure that is a formula", "figures that are formulas") });
            } else {
                // The estimate as values is its lines under their headings and nothing
                // else, so it reads back as an estimate made before.
                let rows = match Lens::parse(what) {
                    None => sheets[0].rows[sheets[0].head..].iter().map(|r| r.iter().map(Out::said).collect()).collect(),
                    Some(_) => export::values(&sheets),
                };
                std::fs::write(other, plumbline_sheets::csv(&rows)).map_err(|e| format!("cannot write {other}: {e}"))?;
                say!("wrote {other}: {said} of {path}, {} under its headings", count(lines, "row", "rows"));
            }
            Ok(())
        }
        ("export", _) => Err(format!("export {what}: what is written out is prices, assemblies, estimate, supplier, subcontractor, pm or ifc\n{}", usage())),
        _ => Err(format!("import {what}: what is brought in is prices, assemblies, prior (an estimate made before) or ifc (a model)\n{}", usage())),
    }
}

/// Each prior estimate in the job set against the job's own lines.
fn compared_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::tables;
    let money = |x: f64| format!("${}", view::figure(x, 2));
    let signed = |x: f64| format!("{}${}", if x < 0.0 { "-" } else { "+" }, view::figure(x.abs(), 2));
    let est = view::estimate(job);
    let all = tables::compare(job);
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    if all.is_empty() {
        out.push_str("No prior estimate has been brought into this job: plumbline import prior <file> <estimate.csv> --status Assumed --source \"...\"\n");
    }
    for c in all {
        let _ = writeln!(out, "\n{}{} [{}], set against this job  ({})", match c.status { Status::Known => "", Status::Assumed => "~ ", Status::Unknown => "? " }, c.name, c.path, c.source);
        let _ = writeln!(out, "  LINE BY LINE (each prior line beside every line of this job called the same)");
        for l in &c.lines {
            let p = &l.prior;
            let prior_qty = p.quantity.map(|q| format!("{} {}", view::figure(q, 2), p.unit).trim().to_string()).unwrap_or_else(|| "no quantity".into());
            let prior_rate = p.unit_cost.map(|x| format!(" at {}", money(x))).unwrap_or_default();
            let prior_amount = l.prior_amount.map(|a| format!("{}{}", money(a), if l.worked { " (its quantity times its unit cost)" } else { "" })).unwrap_or_else(|| "no amount".into());
            let _ = writeln!(out, "  {}{}", p.description, if l.of > 1 { format!("  ({} lines of the prior estimate go by this name; they are added together)", l.of) } else { String::new() });
            let _ = writeln!(out, "      prior:     {prior_qty}{prior_rate}: {prior_amount}");
            match &l.job {
                None => {
                    let _ = writeln!(out, "      this job:  has no line called that");
                }
                Some(j) => {
                    let flag = match j.status { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" };
                    let qty = match j.quantity {
                        Some(q) => format!("{flag}{} {}", view::figure(q, 2), j.unit),
                        None if j.unit.is_empty() => "its lines are in more than one unit and do not add".to_string(),
                        None => "no quantity".to_string(),
                    };
                    let amount = j.amount.map(|a| format!("{flag}{}", money(a))).unwrap_or_else(|| format!("no figure: {}", j.why));
                    let _ = writeln!(out, "      this job:  {qty}: {amount}  ({} on {})", count(j.on.len(), "line", "lines"), j.on.join(", "));
                    // A difference is as sure as the less sure of the two it is between.
                    let flag = match j.status.max(c.status) { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" };
                    let mut differs = Vec::new();
                    if let (Some(pq), Some(jq)) = (p.quantity, j.quantity) {
                        if p.unit.eq_ignore_ascii_case(&j.unit) {
                            differs.push(format!("quantity {flag}{}{} {}", if jq - pq < 0.0 { "-" } else { "+" }, view::figure((jq - pq).abs(), 2), j.unit));
                        } else {
                            differs.push(format!("the prior line is in {} and this job's in {}: the quantities are not compared", if p.unit.is_empty() { "no unit" } else { &p.unit }, j.unit));
                        }
                    }
                    if let (Some(pa), Some(ja)) = (l.prior_amount, j.amount) {
                        differs.push(format!("amount {flag}{}", signed(ja - pa)));
                    }
                    if !differs.is_empty() {
                        let _ = writeln!(out, "      this job less the prior: {}", differs.join("; "));
                    }
                }
            }
        }
        let _ = writeln!(out, "  IN THIS JOB AND NOT IN THE PRIOR ESTIMATE: {}", c.only_in_job.len());
        for (description, on, amount, status) in &c.only_in_job {
            let flag = match status { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" };
            let _ = writeln!(out, "    {description}: {}  ({})", amount.map(|a| format!("{flag}{}", money(a))).unwrap_or_else(|| "no figure".into()), on.join(", "));
        }
        let unmatched: Vec<&tables::ComparedLine> = c.lines.iter().filter(|l| l.job.is_none()).collect();
        let _ = writeln!(out, "  IN THE PRIOR ESTIMATE AND NOT IN THIS JOB: {}", unmatched.len());
        for l in unmatched {
            let _ = writeln!(out, "    {}: {}", l.prior.description, l.prior_amount.map(money).unwrap_or_else(|| "no amount".into()));
        }
        let _ = writeln!(out, "  the prior estimate's lines add to {}{}", money(c.prior_total), if c.prior_left_out > 0 { format!(", leaving out {} with no amount", count(c.prior_left_out, "line", "lines")) } else { String::new() });
        let job_cost = match est.job_cost.value {
            Some(x) => format!("{}{}", match est.job_cost.status { Status::Known => "", Status::Assumed => "~", Status::Unknown => "?" }, money(x)),
            None => "no figure".to_string(),
        };
        let _ = writeln!(out, "  this job's cost is {job_cost}; nothing of the prior estimate is in it");
    }
    out
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What bringing a table in did, said so that every row and column of the file is
/// accounted for: used, the same as the job already had, or not used and why.
fn brought_said(file: &str, sheet: &plumbline_sheets::Sheet, b: &plumbline_world::tables::Brought) -> String {
    let mut out = String::new();
    let from = if sheet.name.is_empty() { sheet.sort.said().to_string() } else { format!("{}, sheet {:?}", sheet.sort.said(), sheet.name) };
    let _ = writeln!(out, "{file} ({from}): {} under its headings", count(b.rows, "row", "rows"));
    let _ = writeln!(out, "  set: {}", b.done.len());
    for d in &b.done {
        let _ = writeln!(out, "    {d}");
    }
    let _ = writeln!(out, "  the same as the job already has: {}", b.same);
    let _ = writeln!(out, "  not used: {}", b.not_used.len());
    for (row, why) in &b.not_used {
        let _ = writeln!(out, "    row {row}: {why}");
    }
    for note in &b.notes {
        let _ = writeln!(out, "  NOTE: {note}");
    }
    if b.above > 0 {
        let _ = writeln!(out, "  not read: {} above the headings", count(b.above, "row", "rows"));
    }
    for c in &b.columns_not_used {
        let _ = writeln!(out, "  not read: the column {c:?}");
    }
    for lost in &sheet.lost {
        let _ = writeln!(out, "  not read: {lost}");
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
    if matches!(cmd, "export" | "import") {
        return tables(args);
    }
    if cmd == "compare" {
        let [_, path] = args else {
            return Err(format!("compare takes one file and nothing else\n{}", usage()));
        };
        let (mut job, _) = read(path)?;
        let _ = std::io::stdout().write_all(compared_said(&mut job).as_bytes());
        return Ok(());
    }
    if matches!(cmd, "new" | "storey" | "type" | "system" | "wall" | "floor" | "door" | "window" | "room" | "run" | "place" | "move" | "measure" | "remove" | "split" | "fitting" | "connect" | "drawing" | "calibrate" | "lineup") {
        return draw(args);
    }
    if matches!(cmd, "sample" | "house" | "sixteen" | "priced" | "show" | "check" | "lines" | "takeoff" | "price" | "estimate") && args.len() > 2 {
        return Err(format!("{cmd} takes one file and nothing else; {:?} would be ignored\n{}", args[2], usage()));
    }
    match cmd {
        "sample" | "house" | "sixteen" | "priced" => {
            let out = args.get(1).ok_or_else(usage)?;
            let mut job = match cmd {
                "house" => sample::house(),
                "sixteen" => sample::house_priced(),
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
            let mut out = takeoff(&mut job);
            // What was traced before its sheet was rescaled or moved is counted above
            // as it lies. It is said here too, not only where the drawings are listed.
            let drawings: Vec<(String, String)> = {
                let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::Kind)>();
                q.iter(&job.world).filter(|(_, k)| **k == plumbline_world::Kind::Drawing).map(|(n, _)| (n.path.clone(), n.name.clone())).collect()
            };
            for (path, name) in drawings {
                let left: Vec<String> = plumbline_world::plans::stale(&mut job, &path).unwrap_or_default().into_iter().flat_map(|s| s.traced).collect();
                if !left.is_empty() {
                    let _ = writeln!(out, "NOTE: {} traced over {name} before its scale or its place was changed, and not moved: {}. The figures above count {} as {} lie{}; check {} against the sheet (plumbline drawings).", count(left.len(), "thing was", "things were"), left.join(", "), if left.len() == 1 { "it" } else { "them" }, if left.len() == 1 { "it" } else { "they" }, if left.len() == 1 { "s" } else { "" }, if left.len() == 1 { "it" } else { "each" });
                }
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        "drawings" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(drawings(&mut job, path).as_bytes());
            Ok(())
        }
        "clashes" => {
            let path = args.get(1).ok_or_else(usage)?;
            let clearance = match &args[2..] {
                [] => None,
                [flag, typed] if flag == "--clearance" => {
                    let (len, unit) = units::parse_length(typed)?;
                    Some(len.feet(unit)?)
                }
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(clashes(&mut job, clearance)?.as_bytes());
            Ok(())
        }
        // What is still to be verified in the field, and the dollars riding on each.
        "vif" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(vif(&mut job).as_bytes());
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
            let (mut class, mut of_material, mut division) = (None, false, None);
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--class" | "--cost-type" => class = Some(plumbline_world::Class::parse(it.next().ok_or("--cost-type needs a cost type")?)?),
                    "--division" => division = Some(plumbline_world::Division::parse(it.next().ok_or("--division needs a division, 1 to 16")?)?),
                    "--of-material" => of_material = true,
                    other => return Err(format!("unexpected {other:?}\n{}", usage())),
                }
            }
            let (mut job, _) = read(path)?;
            job.add_line(element, description, quantity, class, of_material)?;
            if let Some(division) = division {
                job.set_division(element, Some(description), division)?;
            }
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("added to {element}: {description:?} = {quantity}{}{}", if of_material { ", made of the element's material".to_string() } else { ", with no price yet".to_string() }, division.map(|d| format!("; {}", d.said())).unwrap_or_default());
            Ok(())
        }
        // Which row of the recap a line lands in; and a Labor line switched off or on.
        // Which of the sixteen divisions a line's work is.
        "division" => {
            let [_, path, element, value, rest @ ..] = args else { return Err(usage()) };
            let line = match rest {
                [] => None,
                [flag, line] if flag == "--line" => Some(line.as_str()),
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            let division = plumbline_world::Division::parse(value)?;
            job.set_division(element, line, division)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{element}{}: {}", line.map(|l| format!(" {l:?}")).unwrap_or_default(), division.said());
            Ok(())
        }
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
/// The drawing verbs: a job started from nothing, and a building put into it a storey, a
/// type, a wall, an opening, a run and a fixture at a time. Each is one call into
/// `plumbline_world::editing`, which the window calls too.
fn draw(args: &[String]) -> Result<(), String> {
    use plumbline_world::editing as ed;
    use plumbline_world::Kind;
    let cmd = args[0].as_str();
    let path = args.get(1).ok_or_else(usage)?;
    // What stands before the first flag, and the flags, each given once.
    let rest = &args[2..];
    let split = rest.iter().position(|a| a.starts_with("--")).unwrap_or(rest.len());
    let (plain, flags) = rest.split_at(split);
    let mut given: Vec<(String, String)> = Vec::new();
    let mut it = flags.iter();
    while let Some(flag) = it.next() {
        if given.iter().any(|(f, _)| f == flag) {
            return Err(format!("{flag} given twice"));
        }
        let value = if flag == "--closed" || flag == "--all" || flag == "--walls" { String::new() } else { it.next().cloned().ok_or_else(|| format!("{flag} needs a value"))? };
        given.push((flag.clone(), value));
    }
    let mut used: Vec<&str> = Vec::new();
    let mut flag = |name: &'static str| -> Option<String> {
        used.push(name);
        given.iter().find(|(f, _)| f == name).map(|(_, v)| v.clone())
    };
    let need = |what: Option<String>, name: &str, like: &str| what.ok_or_else(|| format!("{name} is required, like {like}"));
    let points = |text: &str| -> Result<Vec<[f64; 2]>, String> {
        text.split_whitespace()
            .map(|pair| {
                let (x, y) = pair.split_once(',').ok_or_else(|| format!("{pair:?} is not a point; write it x,y in feet"))?;
                Ok([units::parse_number(x)?, units::parse_number(y)?])
            })
            .collect()
    };
    let kind_of = |word: &str| match word {
        "door" => Ok(Kind::Door),
        "window" => Ok(Kind::Window),
        "duct" => Ok(Kind::Duct),
        "pipe" => Ok(Kind::Pipe),
        "conduit" => Ok(Kind::Conduit),
        "cable" => Ok(Kind::Cable),
        "fixture" => Ok(Kind::Fixture),
        "equipment" => Ok(Kind::Equipment),
        other => Err(format!("{other:?} is not a kind this verb takes")),
    };
    if cmd == "new" {
        let [title] = plain else { return Err(usage()) };
        if std::path::Path::new(path).exists() {
            return Err(format!("{path} is already there; a new job is not written over a file"));
        }
        // The day it was started, from the machine's clock.
        let mut job = plumbline_world::Job::new(title, "plumbline", &plumbline_world::job::now()[..10]);
        std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
        say!("wrote {path}: a new job, {title:?}, with nothing in it");
        return Ok(());
    }
    // Every other verb says how sure what it enters is, and who says so.
    let (status, source) = (flag("--status"), flag("--source"));
    // A system is named, and a thing is removed: neither is entered with a status, and a
    // flag that would be ignored is refused, as any other is.
    if matches!(cmd, "system" | "remove") && (status.is_some() || source.is_some()) {
        return Err(format!("--status and --source are not something {cmd} takes; they would be ignored\n{}", usage()));
    }
    let how = |status: &Option<String>, source: &Option<String>| -> Result<(Status, String), String> {
        let status = match status.as_deref() {
            Some("Known") => Status::Known,
            Some("Assumed") => Status::Assumed,
            Some(other) => return Err(format!("{other:?} is not a status a thing can be drawn with; use Known or Assumed")),
            None => return Err("--status is required: say whether what is drawn is Known or Assumed".into()),
        };
        Ok((status, source.clone().ok_or("--source is required: say where it came from")?))
    };
    let (mut job, _) = read(path)?;
    // What is traced over a drawing is Assumed and says which drawing. `--on <drawing>`
    // stands in place of --status and --source, and the points given are then places on
    // the drawing's page, turned into places on the plan by its scale.
    let on = if matches!(cmd, "wall" | "door" | "window" | "room" | "run" | "place" | "floor") { flag("--on") } else { None };
    let traced = match &on {
        None => None,
        Some(d) => {
            if status.is_some() || source.is_some() {
                return Err("with --on, what is drawn is Assumed and its source is the drawing: --status and --source are not given".into());
            }
            let source = plans::traced_from(&mut job, d)?;
            let drawing = job.find(d).and_then(|e| job.world.get::<plans::Drawing>(e).cloned()).ok_or_else(|| format!("{d} is not a drawing"))?;
            // A drawing is a plan of one storey: what is traced over it is on that storey.
            let its = plans::storey_of(&mut job, d)?;
            let onto = match (cmd, plain.first()) {
                ("door" | "window", Some(wall)) => job.find(wall).and_then(|e| job.world.get::<plumbline_world::Node>(e).and_then(|n| n.parent.clone())),
                (_, Some(storey)) => Some(storey.to_string()),
                _ => None,
            };
            if let Some(onto) = onto.filter(|s| *s != its) {
                return Err(format!("{d} is a plan of {its}, not of {onto}: what is traced over a drawing is on the storey it is a plan of"));
            }
            Some((drawing, source))
        }
    };
    let how = |status: &Option<String>, source: &Option<String>| -> Result<(Status, String), String> {
        match &traced {
            Some((_, source)) => Ok((Status::Assumed, source.clone())),
            None => how(status, source),
        }
    };
    let points = |text: &str| -> Result<Vec<[f64; 2]>, String> {
        let pts = points(text)?;
        match &traced {
            None => Ok(pts),
            Some((drawing, _)) => pts.into_iter().map(|p| drawing.to_plan(p).map(|q| q.map(|x| (x * 1.0e4).round() / 1.0e4 + 0.0)).ok_or_else(|| "the drawing's scale does not work out".to_string())).collect(),
        }
    };
    let said: String = match (cmd, plain) {
        ("drawing", [storey, name]) => {
            let (status, source) = how(&status, &source)?;
            let given = need(flag("--file"), "--file", "plans.pdf")?;
            let page = match flag("--page") {
                None => 1,
                Some(p) => p.parse::<usize>().map_err(|_| format!("--page {p:?} is not a page number"))?,
            };
            let bytes = std::fs::read(&given).map_err(|e| format!("cannot read {given}: {e}"))?;
            let read = plumbline_plans::read(&bytes, page).map_err(|why| format!("{given}: {why}"))?;
            // Kept in the job by its path from the job's folder, so the two can be moved together.
            let kept = from_the_jobs_folder(path, &given)?;
            let made = plans::add(&mut job, storey, name, plans::Read { file: &kept, page, bytes: bytes.len() as u64, checksum: &plumbline_plans::checksum(&bytes), size: read.size }, ed::Entered { status, source: &source })?;
            let mut out = format!("{made}: drawing {name:?}: page {page} of {} of {kept} ({}), {} by {} {}, {}; no scale yet", read.pages, read.sort.said(), view::num(read.size[0]), view::num(read.size[1]), page_unit(read.sort), lines_said(read.lines.len()));
            for lost in &read.lost {
                out.push_str(&format!("\n  not read: {lost}"));
            }
            out
        }
        ("calibrate", [drawing]) => {
            let (status, source) = how(&status, &source)?;
            let one = |name: &'static str, text: Option<String>| -> Result<[f64; 2], String> {
                match points(&need(text, name, "\"126,90\" (a place on the page)")?)?.as_slice() {
                    [p] => Ok(*p),
                    _ => Err(format!("{name} is one point, x,y on the page")),
                }
            };
            let (from, to) = (one("--from", flag("--from"))?, one("--to", flag("--to"))?);
            let distance = need(flag("--distance"), "--distance", "\"30 ft\" (the dimension between the two points)")?;
            let done = plans::calibrate(&mut job, drawing, from, to, &distance, ed::Entered { status, source: &source })?;
            let k = done.feet_per_unit;
            let mut said = format!("{drawing}: {} is {} on its page: {} to the foot", distance.trim(), view::num(((to[0] - from[0]).hypot(to[1] - from[1]) * 1.0e4).round() / 1.0e4), view::num((1.0e4_f64 / k).round() / 1.0e4));
            if done.lined_up_again {
                said.push_str("\n  it was lined up, and the place it was lined up by is where it was");
            }
            if !done.left_at_earlier_scale.is_empty() {
                said.push_str(&format!("\n  NOT MOVED: {} traced over it at the scale it had before: {}. Each is where that scale put it: check it against the sheet, or remove it and trace it again", count(done.left_at_earlier_scale.len(), "thing was", "things were"), done.left_at_earlier_scale.join(", ")));
            }
            said
        }
        ("lineup", [drawing]) => {
            let (status, source) = how(&status, &source)?;
            let one = |name: &'static str, text: Option<String>, like: &str| -> Result<[f64; 2], String> {
                match points(&need(text, name, like)?)?.as_slice() {
                    [p] => Ok(*p),
                    _ => Err(format!("{name} is one point, x,y")),
                }
            };
            let on_page = one("--page-point", flag("--page-point"), "\"126,126\" (a place on the page)")?;
            let on_plan = one("--plan-point", flag("--plan-point"), "\"0,0\" (the place on the plan it is, in feet)")?;
            let left = plans::line_up(&mut job, drawing, on_page, on_plan, ed::Entered { status, source: &source })?;
            let mut said = format!("{drawing}: ({}, {}) on its page is ({}, {}) on the plan", view::num(on_page[0]), view::num(on_page[1]), view::num(on_plan[0]), view::num(on_plan[1]));
            if !left.is_empty() {
                said.push_str(&format!("\n  NOT MOVED: {} traced over it where it lay before: {}. The sheet has moved and {} not; check {} against it", count(left.len(), "thing was", "things were"), left.join(", "), if left.len() == 1 { "it has" } else { "they have" }, if left.len() == 1 { "it" } else { "each" }));
            }
            said
        }
        ("storey", [name]) => {
            let (status, source) = how(&status, &source)?;
            let made = ed::storey(&mut job, name, &need(flag("--elevation"), "--elevation", "\"0 ft\"")?, &need(flag("--height"), "--height", "\"9 ft\"")?, ed::Entered { status, source: &source })?;
            format!("{made}: storey {name:?}")
        }
        ("type", [what, name]) => {
            let (status, source) = how(&status, &source)?;
            let how = ed::Entered { status, source: &source };
            let made = match what.as_str() {
                "wall" => ed::wall_type(&mut job, name, &need(flag("--thickness"), "--thickness", "\"6 in\"")?, how)?,
                "floor" => ed::floor_type(&mut job, name, &need(flag("--thickness"), "--thickness", "\"4 in\"")?, how)?,
                "door" | "window" => ed::opening_type(&mut job, name, kind_of(what)?, &need(flag("--width"), "--width", "\"3 ft\"")?, &need(flag("--height"), "--height", "\"7 ft\"")?, how)?,
                "duct" | "pipe" | "conduit" | "cable" => {
                    let (round, width, depth) = (flag("--round"), flag("--width"), flag("--depth"));
                    match (round, width, depth) {
                        (Some(across), None, None) => ed::segment_type(&mut job, name, kind_of(what)?, &across, None, how)?,
                        (None, Some(width), Some(depth)) => ed::segment_type(&mut job, name, kind_of(what)?, &width, Some(&depth), how)?,
                        _ => return Err("a run's type is --round \"6 in\", or --width \"12 in\" --depth \"8 in\"".into()),
                    }
                }
                other => return Err(format!("{other:?} has no type here; a type is for a wall, floor, door, window, duct, pipe, conduit or cable")),
            };
            format!("{made}: {what} type {name:?}")
        }
        ("system", [name, classification]) => {
            let made = ed::system(&mut job, name, classification)?;
            format!("{made}: system {name:?}")
        }
        ("wall", [storey, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"0,0 20,0 20,10\" (feet)")?)?;
            let (height, top, closed) = (flag("--height"), flag("--top"), flag("--closed").is_some());
            let top = match (&height, &top) {
                (Some(h), None) => ed::Top::Height(h),
                (None, Some(level)) => ed::Top::Level(level),
                _ => return Err("say how tall the walls are: --height \"9 ft\", or --top <the storey their top is held to>; one of them".into()),
            };
            let made = ed::walls(&mut job, storey, type_path, &pts, closed, top, ed::Entered { status, source: &source })?;
            format!("{} wall{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
        }
        ("door" | "window", [wall, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let (offset, at) = (flag("--offset"), flag("--at"));
            let point = at.as_deref().map(points).transpose()?;
            let place = match (&offset, &point) {
                (Some(o), None) => ed::OnWall::Offset(o),
                (None, Some(p)) if p.len() == 1 => ed::OnWall::Point(p[0]),
                _ => return Err("say where along the wall its centre is: --offset \"5 ft\" from the wall's first end, or --at \"5,0\" a point on the plan; one of them".into()),
            };
            let made = ed::opening(&mut job, wall, kind_of(cmd)?, type_path, place, &need(flag("--sill"), "--sill", "\"0 ft\" for a door, \"3 ft\" for a window")?, ed::Entered { status, source: &source })?;
            format!("{made}: a {cmd} in {wall}")
        }
        ("floor", [storey, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let entered = ed::Entered { status, source: &source };
            let by_walls = given.iter().any(|(f, _)| f == "--walls");
            let made = match (by_walls, flag("--points")) {
                (true, None) => {
                    let _ = flag("--walls");
                    ed::floor(&mut job, storey, type_path, ed::FloorBy::Walls, entered)?
                }
                (false, Some(text)) => ed::floor(&mut job, storey, type_path, ed::FloorBy::Points(&points(&text)?), entered)?,
                _ => return Err("a floor is given by --walls (the outline its storey's walls enclose) or by --points \"0,0 30,0 30,20 0,20\" (its corners, in feet): one of the two".into()),
            };
            let mut out = format!("{}: a floor on {storey}", made.path);
            for note in made.notes {
                out.push_str(&format!("\n  NOTE: {note}"));
            }
            out
        }
        ("room", [storey, name]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"6,8\" (a point inside it, in feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let made = ed::room(&mut job, storey, name, *at, ed::Entered { status, source: &source })?;
            format!("{made}: room {name:?}")
        }
        ("run", [storey, what, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"2,2 14,2 14,7\" (feet)")?)?;
            let made = ed::runs(&mut job, storey, kind_of(what)?, type_path, &pts, &need(flag("--height"), "--height", "\"8 ft\" above the storey's floor")?, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
            format!("{} straight piece{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
        }
        ("place", [storey, what, name]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"14,7\" (feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let size = match flag("--size") {
                None => None,
                Some(text) => match text.split(',').map(|x| units::parse_number(x)).collect::<Result<Vec<f64>, String>>()?.as_slice() {
                    [a, b, c] => Some([*a, *b, *c]),
                    _ => return Err("--size is along,across,up in feet, like \"2,2,6\"".into()),
                },
            };
            let made = ed::place(&mut job, storey, kind_of(what)?, name, *at, &need(flag("--offset"), "--offset", "\"0 ft\" above the storey's floor")?, size, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
            format!("{made}: {name:?}")
        }
        ("move", [element]) => {
            let (status, source) = how(&status, &source)?;
            let by = points(&need(flag("--by"), "--by", "\"3,2\" (feet east, feet north)")?)?;
            let [by] = by.as_slice() else { return Err("--by is one distance, east,north in feet".into()) };
            ed::move_by(&mut job, element, by[0], by[1], ed::Entered { status, source: &source })?;
            format!("{element} moved {} ft east and {} ft north", view::num(by[0]), view::num(by[1]))
        }
        ("measure", [element, what, reading]) => {
            // What is measured is Known: that is what a measurement is. It still says who
            // measured, or with what.
            if status.is_some() {
                return Err("a measurement is Known; --status is not given. Say who measured, or with what, with --source".into());
            }
            let source = source.clone().ok_or("--source is required: say who measured it, or with what")?;
            let to = match flag("--to").as_deref() {
                None => None,
                Some("bottom") => Some(plumbline_world::Justify::Bottom),
                Some("centre" | "center") => Some(plumbline_world::Justify::Centre),
                Some("top") => Some(plumbline_world::Justify::Top),
                Some(other) => return Err(format!("--to {other:?}: a run's height is measured to its bottom, its centre or its top")),
            };
            let reading = if reading.trim().eq_ignore_ascii_case("same") { ed::Reading::Same } else { ed::Reading::Length(reading) };
            let done = ed::measure(&mut job, element, what, reading, to, &source)?;
            let mut out = done.said;
            for also in done.also {
                out.push_str(&format!("\n  {also}"));
            }
            out
        }
        ("remove", [element]) => {
            let gone = ed::remove(&mut job, element)?;
            format!("removed {}", gone.join(", "))
        }
        ("split", [run]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"6,2\" (a point on the run, in feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let new = ed::split(&mut job, run, *at, ed::Entered { status, source: &source })?;
            format!("{run} cut at ({}, {}); its second piece is {new}", view::num(at[0]), view::num(at[1]))
        }
        ("fitting", []) => {
            let (status, source) = how(&status, &source)?;
            if flag("--all").is_none() {
                return Err("say where: plumbline fitting <file> <storey> --at \"10,2\", or --all for every joint that has none".into());
            }
            let (made, passed) = ed::fit_all(&mut job, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
            let mut out = format!("{} fitting{}", made.len(), if made.len() == 1 { "" } else { "s" });
            for f in &made {
                out.push_str(&format!("\n  {}: a {}{}", f.path, f.kind.title().to_lowercase(), f.cut.iter().map(|(run, new)| format!("; {run} was cut there, its second piece is {new}")).collect::<String>()));
            }
            for p in &passed {
                out.push_str(&format!("\n  passed over on {} at ({}, {}, {}): {}", p.storey, view::num(p.at[0]), view::num(p.at[1]), view::num(p.at[2]), p.why));
            }
            out
        }
        ("fitting", [storey]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"10,2\" (where the runs meet, in feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let kind = match flag("--kind").as_deref() {
                None => None,
                Some("bend") => Some(plumbline_world::FittingKind::Bend),
                Some("tee") => Some(plumbline_world::FittingKind::Tee),
                Some("cross") => Some(plumbline_world::FittingKind::Cross),
                Some("transition") => Some(plumbline_world::FittingKind::Transition),
                Some("coupling") => Some(plumbline_world::FittingKind::Coupling),
                Some(other) => return Err(format!("{other:?} is not a kind of fitting; there are: bend, tee, cross, transition, coupling")),
            };
            let f = ed::fitting(&mut job, storey, *at, flag("--height").as_deref(), kind, ed::Entered { status, source: &source })?;
            format!("{}: a {} at ({}, {}){}", f.path, f.kind.title().to_lowercase(), view::num(at[0]), view::num(at[1]), f.cut.iter().map(|(run, new)| format!("; {run} was cut there, its second piece is {new}")).collect::<String>())
        }
        ("connect", [thing, run]) => {
            let (status, source) = how(&status, &source)?;
            let which = ed::connect(&mut job, thing, run, ed::Entered { status, source: &source })?;
            format!("{thing} is joined to {run} by its connector {which}")
        }
        _ => return Err(usage()),
    };
    if let Some((unused, _)) = given.iter().find(|(f, _)| !used.contains(&f.as_str())) {
        return Err(format!("{unused} is not something {cmd} takes; it would be ignored\n{}", usage()));
    }
    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
    say!("{said}");
    Ok(())
}

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
