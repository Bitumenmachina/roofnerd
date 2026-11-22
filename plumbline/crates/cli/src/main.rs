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
            eprintln!("plumbline: {}", plainly(&e, &args));
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage:\n  plumbline sample <file>\n  plumbline house <file>\n  plumbline sixteen <file>   (the sample whole house with a made-up price on everything, in all sixteen divisions)\n  plumbline show <file>\n  plumbline set <file> <path> <field> <value|unknown> --status <Known|Assumed|Unknown> --source \"<where from>\" [--line \"<cost line>\"] [--with \"name=value\"]\n  plumbline check <file>\n  plumbline lines <file>\n  plumbline takeoff <file>\n  plumbline drawings <file>\n  plumbline clashes <file> [--clearance \"2 in\"]\n  plumbline vif <file>   (what is still to be verified in the field, and the dollars riding on each)\n  plumbline finishes <file>   (what each wall, roof plane, window and door is finished with, read from what is priced on it)\n  plumbline price <file>\n  plumbline estimate <file> [by-item]   (every line under the thing it is on; by-item: each item once, with its places, division by division)\n  plumbline priced <file>\n  plumbline material <file> \"<name>\" <unit it is bought by: lb, sheet, SF>\n  plumbline line <file> <path> \"<description>\" <quantity, or a measure like \"60 SF\" with --status and --source> [--cost-type <material|labor|...>] [--division <1 to 16>] [--of-material]\n  plumbline remove-line <file> <path> \"<cost line>\"   (takes that one line off the thing)\n  plumbline division <file> <path> <1 to 16, or its title> [--line \"<cost line>\"]\n  plumbline cost-type <file> <path> <material|labor|fabrication|supervision|subcontract|general> [--line \"<cost line>\"]\n  plumbline labor <file> <path> <on|off> [--line \"<cost line>\"]\n  plumbline apply <file> <path> <assembly> --source \"<who says>\" [--change]   (--change: in place of the assembly it has; lines the two share keep their prices)\n  plumbline export <prices|assemblies> <file> <out.csv|out.xlsx>\n  plumbline export estimate <file> <out.xlsx|out.csv>   (.xlsx: a workbook whose amounts, recap and price are formulas; .csv: the lines as values)\n  plumbline export ifc <file> <out.ifc>   (the building as an IFC4 model, for another trade or a designer)\n  plumbline export <supplier|subcontractor|pm> <file> <out.xlsx|out.csv> [--division <1 to 16>] [--system \"<name>\"]   (--division and --system cut the subcontractor's list to one trade's or one system's scope; one lens on the estimate: what to quote; the subcontract scope; the lines, budget and what is to verify)\n  plumbline import prices <file> <a price list: CSV or XLSX> [--sheet \"<name>\"] [--supplier \"<whose prices>\"] [--add] [--status <Known|Assumed> --source \"<where from>\"]\n  plumbline import assemblies <file> <an assembly library this program wrote: CSV or XLSX> [--sheet \"<name>\"]\n  plumbline import assemblies <file> starter   (the starter library the program ships with: whole assemblies for walls, floors, rooms, roofs, openings and runs, a line in every division, no prices)\n  plumbline import items <file> <an item library: CSV or XLSX, or the word starter for the one the program ships with> [--sheet \"<name>\"]   (the things that can be placed, by trade: each row an item, its kind, what it is drawn as and its sizes; a row whose item the job has changes it)\n  plumbline export items <file> <out.csv>   (the job's items as that table, to open, change and bring back in)\n  plumbline export vif <file> <out.xlsx|out.csv>   (what is to verify in the field, as a sheet to print and take to the job: a box to tick, the dollars riding, where, what, what to do, and a blank for what was found)\n  plumbline items <file>   (the job's items, trade by trade)\n  plumbline import prior <file> <an estimate made before: CSV or XLSX> [--sheet \"<name>\"] [--name \"<what to call it>\"] --status <Known|Assumed> --source \"<where from>\"\n  plumbline import scan <file> <a scan: a PLY point cloud> --level <level> [--units m|ft] [--up z|y] [--cut \"4 ft\"]   (a scan of the building, cut above its floor into a sheet to trace over: to scale, in the scan's own place, Assumed; taken as meters with z up unless said)\n  plumbline import spec <file> <a maker's sheet: PDF> [--page <n>] [--as <equipment|fixture|furnishing>] [--name \"<what it is called>\"]   (an item to place: its name, maker, model, width, depth, height and weight read off the sheet, each Assumed and sourced to its line; what is not there is Unknown)\n  plumbline import ifc <file> <a model: IFC2x3 or IFC4> --status <Known|Assumed> --source \"<where from>\"\n  plumbline plan-set <file> <level> <out.pdf> [--scale \"1/4\"] [--paper \"ARCH A\"] [--date <the day>] [--list]   (the plan of a level as it stands, the demolition plan and the proposed plan, a page each, as line work to print: walls joined, doors and windows, room names and areas, a dimension along each wall (VIF after a length that is not Known), a title box and notes; 1/4 in to the foot unless said (1/8, 3/16, 1/4, 3/8, 1/2); the smallest of ARCH A, B, C and D it fits unless said; a level with no mark on anything gets one page, FLOOR PLAN; --list says what is on each page, wall by wall; the job is not changed)\n  plumbline page <file> [--page <n>] [--lines [--longer-than <points>]] [--png <out.png>]   (what a page of a PDF holds as this program reads it: its size, the straight lines it draws; --png writes the page as a picture)\n  plumbline compare <file>\n  plumbline options <layout> <layout> [<layout>]   (two or three layouts of a job, each a job file, set side by side: each room's floor area, the walls by what the work does to them, the job cost and each division; the last less the first, and the lines each job cost leaves out; nothing is changed)\n  plumbline price-item <file> \"<item>\" '$180' --status <Known|Assumed|Unknown> --source \"<where from>\"   (one price on the line named for the item, on every one of it set in the job; it says how many; one set later takes none)\n\ndrawing (each but new and system takes --status <Known|Assumed> --source \"<where from>\"; points are x,y in feet):\n  plumbline new <file> \"<title>\"\n  plumbline level <file> \"<name>\" --elevation \"0 ft\" --height \"9 ft\"\n  plumbline type <file> wall \"<name>\" --thickness \"6 in\"\n  plumbline type <file> floor \"<name>\" --thickness \"4 in\"\n  plumbline type <file> <door|window> \"<name>\" --width \"3 ft\" --height \"7 ft\"\n  plumbline type <file> <duct|pipe|conduit|cable> \"<name>\" (--round \"6 in\" | --width \"12 in\" --depth \"8 in\")\n  plumbline system <file> \"<name>\" \"<Supply Air|Domestic Cold Water|...>\"\n  plumbline wall <file> <level> <type> --points \"0,0 20,0 20,10\" [--closed] (--height \"9 ft\" | --top <level>)\n  plumbline type <file> footing \"<name>\" --width \"24 in\"\n  plumbline type <file> <beam|post|pad> \"<name>\" (--round \"6 in\" | --width \"3 1/2 in\" --depth \"11 7/8 in\") [--weight \"18 lb/ft\"]   (a beam's, a post's or a pad footing's section, typed: nothing here sizes anything; --weight is a beam's, what a foot of it weighs)\n  plumbline beam <file> <level> <type> --points \"12,0 12,16\" --top \"9 ft\"   (a beam for each pair of points, its top that high above the level's floor)\n  plumbline post <file> <level> <type> --at \"12,0\" [--height \"8 ft\"] [--base \"0 ft\"]   (with no height it stands to its level's height, Assumed)\n  plumbline pad <file> <level> <type> --at \"12,0\" --thickness \"18 in\" --top \"-1 ft\"   (a pad footing: a post laid flat; its top above the level's floor, below it a minus height)\n  plumbline structure <file>   (what carries the building, for the engineer: the walls the work opens, each beam, post and pad footing with its size, how sure that is and where it came from; and how many are to be sized)\n  plumbline footing <file> <level> <type> --points \"0,0 40,0 40,28\" [--closed] --depth \"12 in\"   (strip footings under a level's floor; the type gives the width)\n  plumbline <door|window> <file> <wall> <type> (--offset \"5 ft\" | --at \"5,0\") --sill \"0 ft\"\n  plumbline floor <file> <level> <type> (--walls | --points \"0,0 30,0 30,20 0,20\")   (--walls: the outline the level's walls enclose)\n  plumbline room <file> <level> \"<name>\" --at \"6,8\"\n  plumbline room-line <file> <level> --points \"24,0 24,16\"   (parts the rooms of a level where no wall stands: one room line for each pair of points that follow one another; it is no wall, and is not priced or counted)\n  plumbline low-slope <file> \"<name>\" --points \"0,0 48,0 48,48 0,48\" --pitch 1/4:12 --height \"11 ft\"   (a low-slope roof: its outline, the slope it finishes at, how high its deck is)\n  plumbline drains <file> <low-slope area> --at \"24,24\" --at-drain \"1/2 in\"   (its roof drains; the tapered insulation is laid out to them at the area's pitch, NRCA's four-way layout)\n  plumbline roof <file> \"<name>\" --points \"0,0 30,0 30,20 0,20\" --pitch 6:12 --eave-height \"8 ft\"   (a hip roof over its footprint: every edge rises at the pitch)\n  plumbline run <file> <level> <duct|pipe|conduit|cable> <type> --points \"2,2 14,2\" --height \"8 ft\" [--system <system>]\n  plumbline place <file> <level> <fixture|equipment|furnishing> \"<what>\" --at \"14,7\" --offset \"0 ft\" [--size \"2,2,6\"] [--system <system>]\n  plumbline place <file> <level> <item> --at \"x,y\" [--offset \"0 ft\"] --status <Known|Assumed> --source \"<where from>\"   (one of an item made from a maker's sheet: its size is the item's)\n  plumbline cabinets <file> <level> <wall> --in <room> --from \"1 ft\" --boxes \"B18 SB36 B24\" [--countertop] --status <Known|Assumed> --source \"<where from>\"   (a run of cabinet boxes by their codes, side by side along the wall's face on the room's side, backs to the wall, from so far along the wall from its first end; base and tall boxes stand on the floor, wall boxes (W...) hang 54 in up; each box carries one line, 1 EA, Division 12, to be priced; any item of the job goes in by its name, a name with a space or a comma in square brackets (\"B18 [Dishwasher] SB36 [Range, 30 in]\"), standing on the floor like a base box; --countertop sets one over the base boxes and what stands no taller than they do, 25 1/2 in deep; a box that would stand in another thing, or across a window, is refused; it says the run's length and what is left to the corner)\n  plumbline drawing <file> <level> \"<name>\" --file <a PDF, PNG or JPEG of the plan> [--page 1]\n  plumbline calibrate <file> <drawing> --from \"126,90\" --to \"666,90\" --distance \"30 ft\"   (two places on the page and the dimension between them)\n  plumbline lineup <file> <drawing> --page-point \"126,126\" --plan-point \"0,0\"\n  tracing: wall, floor, door, window, room, run and place take --on <drawing> in place of --status and --source; their points are then places on the drawing's page\n  plumbline mark <file> <path> [<path> ...] <new|existing|demolish|temporary> [--clear|--keep]   (what the work does to a thing; --clear takes the lines for building it off, --keep leaves them priced as work done to it; what is to demolish bounds no room and is in nothing's way; a thing with no mark is new)\n  plumbline move <file> <path> --by \"3,2\" [--alone]   (a wall takes with it the ends of the walls that meet it, and says which followed; --alone moves it and nothing else)\n  plumbline measure <file> <path> \"<the value, as it is listed: level height, sill, ...>\" <what was read: 8' 10 1/2\", or same> [--to <bottom|center|top>] [--line \"<cost line>\", for a value entered on it]  --source \"<who measured, or with what>\"\n  plumbline remove <file> <path>\n  plumbline fitting <file> <level> --at \"10,2\" [--height \"8 ft\"] [--kind <bend|tee|cross|transition|coupling>]\n  plumbline fitting <file> --all [--system <system>]\n  plumbline connect <file> <fixture or equipment> <run>\n  plumbline split <file> <run> --at \"6,2\"\n\nfields: pitch (6, 6:12, 1/4:12), joists_span (\"the short way\" or \"the long way\": which way a floor's joists run), type (a beam's, a post's or a pad footing's, another type of its kind), height, eave_height, on a roof drawn from its footprint edge:<n> (a pitch, or gable) and move:<n> (how far that edge moves out, like \"6 in\"), stretch_out (2.5 ft, 30 in, 19 1/2\", 25' 6 1/2\"), assembly, wbs, allowance (28, $28.00/SF), material (a material's name)\non the job: overhead (10), profit (8)\non a material: price ($4.10/lb), order (\"1.25 lb/SF\", \"30 SF/sheet\"), quote ($4.25/lb --supplier \"<name>\")\non a cost line (--line): a stretch-out and the quantity that uses it go on together (set <file> <path> quantity \"plan_length * stretch_out\" --line \"<line>\" --with \"stretch_out=19 1/2 in\"); once it is on, stretch_out alone changes it; quantity (\"plan_length * stretch_out\"), waste (10), order (\"30 SF/sheet\", \"1.25 lb/SF\"), unit_cost (4.25, $4.25/SF), or a value the formula names (spacing \"12 in\")".to_string()
}

/// What a page is measured in until its drawing has a scale.
/// A drawing's scale in words (call 88). A sheet of paper says it as a drawing does:
/// `1/4" = 1'-0"`. A picture has no paper size: it says how many of its dots make a
/// foot, and what the sheet covers on the ground.
fn scale_words(d: &plans::Drawing, feet_per_unit: f64) -> String {
    let dots = view::num((1.0e4 / feet_per_unit).round() / 1.0e4);
    match d.scale_said() {
        Some(said) if said.starts_with("sheet covers") => format!("{dots} dots to the foot; the {said}"),
        Some(said) => said,
        None => format!("{dots} to the foot"),
    }
}

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
                let _ = writeln!(out, "    scale: {} between ({}, {}) and ({}, {}) on its page: {}  ({})", s.distance.value.as_ref().map(|l| units::show_length(l, &s.distance.unit)).unwrap_or_default(), view::num(s.from[0]), view::num(s.from[1]), view::num(s.to[0]), view::num(s.to[1]), scale_words(&d, k), s.distance.source);
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
    let lines = |n: usize| plumbline_world::view::so_many(n, "line");
    let mut out = String::new();
    let n = list.questions.len();
    let _ = match (n, list.riding) {
        (0, _) => writeln!(out, "Verify in field: nothing. Every value that can be measured is Known, and no clash is a question."),
        (_, Some(x)) => writeln!(out, "Verify in field: {n} {}. {} rides on them ({} of the estimate, each counted once).", if n == 1 { "question" } else { "questions" }, money(Some(x), list.riding_status), lines(list.riding_lines)),
        (_, None) => writeln!(out, "Verify in field: {n} {}. Nothing priced rides on {}.", if n == 1 { "question" } else { "questions" }, if n == 1 { "it" } else { "them" }),
    };
    // What rides on a question and has no price yet is in no figure above, and is said.
    if list.riding_unpriced > 0 {
        let _ = writeln!(out, "  Not in that: {} that {} on them and {} no price yet.", lines(list.riding_unpriced), if list.riding_unpriced == 1 { "rides" } else { "ride" }, if list.riding_unpriced == 1 { "has" } else { "have" });
    }
    if let Some(why) = &list.clashes_not_tested {
        let _ = writeln!(out, "  The clashes are not on this list: they could not be worked out ({why}).");
    }
    for q in &list.questions {
        let on = if q.storey.is_empty() || q.kind == plumbline_world::Kind::Storey { String::new() } else { format!(", on {}", q.storey) };
        let more = |n: usize| if n == 1 { "1 line has no price yet".to_string() } else { format!("{n} lines have no price yet") };
        let riding = match (q.dollars, q.unpriced) {
            (Some(_), 0) => format!("{}, {}", money(q.dollars, q.dollars_status), lines(q.lines)),
            (Some(_), u) => format!("{}, {}; of what else rides on it, {}", money(q.dollars, q.dollars_status), lines(q.lines), more(u)),
            (None, 0) => "no dollars riding on it".to_string(),
            (None, u) => format!("no dollars yet: of what rides on it, {}", more(u)),
        };
        if let Some(w) = &q.in_the_way {
            let _ = writeln!(out, "  {} [{}] ({}){on}: {riding}", q.name, q.path, q.kind.said());
            let _ = writeln!(out, "      {}", w.said());
            continue;
        }
        match &q.clash {
            Some(c) => {
                let by = match (c.how, c.by) {
                    _ if c.through => "passes through it (a hole to cut: where is still to verify)".to_string(),
                    (plumbline_world::building::clash::ClashKind::Hard, Some(by)) => format!("in the same space by {:.2} in", by * 12.0 + 0.0),
                    (plumbline_world::building::clash::ClashKind::Hard, None) => "in the same space".to_string(),
                    (_, Some(by)) => format!("{:.2} in apart", by * 12.0 + 0.0),
                    (_, None) => "too close".to_string(),
                };
                let riding = if c.through { "nothing priced rides on the hole itself".to_string() } else { riding };
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
                let _ = writeln!(out, "  {} [{}] ({}){on}: {riding}", q.name, q.path, q.kind.said());
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
    // What is in the way of a wall that comes down is not a clash (it is gone when the work
    // is done); it is counted apart, and only where a wall is to demolish (call 160).
    let in_the_way = clash::in_the_way(job)?;
    if !in_the_way.is_empty() {
        let _ = writeln!(out, "In the way of demolition: {}", in_the_way.len());
        for w in &in_the_way {
            let _ = writeln!(out, "  {} [{}] is in {} [{}], which comes down, at ({}, {}, {}): cap it, reroute it or take it out", w.thing_name, w.thing, w.wall_name, w.wall, view::num(w.at[0]), view::num(w.at[1]), view::num(w.at[2]));
        }
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
    if clash::has_framing(job) {
        let _ = writeln!(out, "  {}", clash::FRAMING_NOT_TESTED);
    }
    Ok(out)
}

/// What the building comes to: each storey's walls and floors, then each system's runs by
/// kind and size, its fittings by kind, and what is set on it. The same figures the sheet window shows.
/// What carries the building, for the engineer: what the world says, shown.
fn structure(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::structure::{self, word};
    let s = structure::structure(job);
    let mut out = String::new();
    let _ = writeln!(out, "What carries the building. Every size here is what was typed on a type: nothing in this program sizes a beam, a post or a footing.");
    for level in &s.levels {
        if level.walls.is_empty() && level.members.is_empty() {
            continue;
        }
        let _ = writeln!(out, "Level {} [{}]", level.name, level.path);
        for w in &level.walls {
            let _ = writeln!(out, "  {} [{}] {}; bearing: {}", w.name, w.path, w.how, w.bearing);
        }
        for m in &level.members {
            let _ = writeln!(out, "  {} [{}] ({}): {}", capital(&m.name), m.path, m.type_called, m.what);
            let _ = writeln!(out, "    size {} ({}: {})", m.size, word(m.status), m.source);
        }
    }
    if s.members == 0 {
        let _ = writeln!(out, "No beam, post or pad footing is in the job.");
    } else if s.to_size == 0 {
        let _ = writeln!(out, "Members whose size is not Known: none. All {} sizes are Known.", s.members);
    } else {
        let _ = writeln!(out, "Members whose size is not Known: {} of {}: to be sized by an engineer before it is built or priced firm.", s.to_size, s.members);
    }
    out
}

fn takeoff(job: &mut plumbline_world::Job) -> String {
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
        let _ = writeln!(out, "\n{}{}  ({what}; {}, {}{}{})", if t.flag == Status::Known { String::new() } else { format!("{} ", t.flag.flag()) }, t.name, pieces(t.runs.members), t.runs.status.flag().trim_end(), said(&t.runs), if t.runs_by_work.is_some() { " as it stands" } else { "" });
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
fn capital(word: &str) -> String {
    let mut c = word.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// The estimate sheet: every cost line under its element, in three money columns; the
/// recap; and the suppliers side by side.
/// A refusal as a builder reads it (J17A-16, J17B-21): without the machine's number
/// for why a file could not be read, and, where a thing was named that is not in the
/// job, with what to give instead: a thing is given by its path, and if what was typed
/// is the name of one thing, that thing's path is said.
fn plainly(why: &str, args: &[String]) -> String {
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
fn estimate_by_item(job: &mut plumbline_world::Job) -> String {
    let est = view::estimate(job);
    let money = view::money;
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n");
    let items = est.by_item();
    if items.is_empty() {
        out.push_str("\nNothing in this job has a cost line.\n");
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

fn estimate(job: &mut plumbline_world::Job) -> String {
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
        // A bearing wall the work opens says what that needs, priced or not (call 167).
        for (_, said) in plumbline_world::hidden::cautions(&mut job.world) {
            let _ = writeln!(out, "  ! {said}");
        }
        return out;
    }
    let _ = writeln!(out, "\n{:<58}{:>14} {:>16} {:>13} {:>13} {:>13}", "THE SHEET", "priced", "at", "MATERIAL", "LABOR", "GENERAL");
    let mut last = String::new();
    for l in &est.lines {
        if l.path != last {
            // The job itself is "the job", as the workbook has it, not its place in the file.
            let _ = writeln!(out, "{}  [{}]  ({})", if l.kind == plumbline_world::Kind::Job { "the job" } else { l.element.as_str() }, l.kind.said(), l.path);
            last = l.path.clone();
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
            f @ ("--sheet" | "--supplier" | "--status" | "--source" | "--name" | "--page" | "--as" | "--level" | "--units" | "--up" | "--cut" | "--division" | "--system") => {
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
    // What is written out is never written over a job: not the job it is written from,
    // and not any other file that is a job, whatever its name ends in (J17C-1).
    if cmd == "export" {
        let same = match (std::fs::canonicalize(path), std::fs::canonicalize(other)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        };
        if same {
            return Err(format!("{other} is the job itself: what is written out of a job is not written over it. Give the table a name of its own"));
        }
        if std::fs::read_to_string(other).ok().is_some_and(|text| file::load(&text).is_ok()) {
            return Err(format!("{other} is a job: what is written out is not written over a job. Give the table a name that is not in use"));
        }
    }
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
        // "starter" in place of an assembly library's file is the one the program ships with.
        let starter = matches!(*what, "assemblies" | "items") && *other == "starter" && !std::path::Path::new(other).exists();
        let shipped = if *what == "items" { plumbline_world::tables::STARTER_ITEMS } else { plumbline_world::tables::STARTER_LIBRARY };
        let bytes = if starter { shipped.as_bytes().to_vec() } else { std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))? };
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
        // The item library: the things that can be placed, by trade, as a table a
        // person can open, change and bring back in.
        // What is to verify in the field, as a sheet to print and take to the job.
        ("export", "vif") => {
            only(&[])?;
            let rows = plumbline_world::vif::field_list(&mut job);
            let bytes = if other.to_lowercase().ends_with(".xlsx") {
                let tab = plumbline_sheets::Tab { name: "Field list".into(), rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { plumbline_sheets::Cell::Empty } else { plumbline_sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 1, widths: vec![6.0, 14.0, 24.0, 44.0, 52.0, 30.0] };
                plumbline_sheets::xlsx(&[tab])?
            } else {
                plumbline_sheets::csv(&rows).into_bytes()
            };
            std::fs::write(other, bytes).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: what is to verify in the field in {path}, {} under its headings, the most dollars first", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("export", "items") => {
            only(&[])?;
            let rows = tables::items_out(&mut job);
            std::fs::write(other, plumbline_sheets::csv(&rows)).map_err(|e| format!("cannot write {other}: {e}"))?;
            say!("wrote {other}: the items of {path}, {} under its headings", count(rows.len() - 1, "row", "rows"));
            Ok(())
        }
        ("import", "items") => {
            only(&["--sheet"])?;
            let sheet = table(flag("--sheet"))?;
            // The library the program ships with adds what the job has not got and
            // writes over nothing the job has (J25A-18); a person's own file changes
            // the items it names.
            let shipped = *other == "starter" && !std::path::Path::new(other).exists();
            let (brought, kept) = if shipped {
                tables::items_in_missing(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?
            } else {
                (tables::items_in(&mut job, &sheet.rows, Some(&sheet.numbers)).map_err(|why| format!("{other}: {why}"))?, Vec::new())
            };
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let _ = std::io::stdout().write_all(brought_said(other, &sheet, &brought).as_bytes());
            if !kept.is_empty() {
                say!("  in the job already, and left as the job has them: {}", kept.len());
            }
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
            // One of a thing is said as one: "1 door", not "1 doors" (J17B-19).
            for (what, n) in &b.made {
                let _ = writeln!(out, "    {n} {}", if *n == 1 { what.strip_suffix('s').unwrap_or(what) } else { what });
            }
            let _ = writeln!(out, "  not brought in: {}", count(b.left_out.iter().map(|l| l.1).sum(), "thing", "things"));
            for (class, n, why) in &b.left_out {
                // "4 floor slabs", and each reason as it reads for that many.
                let plural = *n > 1 && !class.ends_with('s') && !class.contains('"') && !class.contains(" or ");
                let _ = writeln!(out, "    {n} {class}{}: {why}", if plural { "s" } else { "" });
            }
            for note in &b.notes {
                let _ = writeln!(out, "  note: {note}");
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        // A scan of the building as a point cloud, cut into a sheet to trace over
        // (DECISIONS.md call 128). The sheet is a drawing under the level, already to
        // scale and in the scan's own place; everything about it is Assumed.
        ("import", "scan") => {
            use plumbline_plans::scan::{Units, Up};
            only(&["--level", "--units", "--up", "--cut"])?;
            let level = flag("--level").ok_or("say which level the scan is of: --level <level>")?.to_string();
            let units = match flag("--units") {
                None | Some("m") => Units::Metres,
                Some("ft") => Units::Feet,
                Some(other) => return Err(format!("--units {other:?}: a scan is in m or ft")),
            };
            let up = match flag("--up") {
                None | Some("z") => Up::Z,
                Some("y") => Up::Y,
                Some(other) => return Err(format!("--up {other:?}: up is z or y")),
            };
            let cut_at = match flag("--cut") {
                None => 4.0,
                Some(text) => {
                    let (len, unit) = plumbline_world::units::parse_length(text).map_err(|why| format!("--cut {text:?}: {why}"))?;
                    len.feet(unit).map_err(|why| format!("--cut {text:?}: {why}"))?
                }
            };
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let called = std::path::Path::new(other).file_stem().and_then(|s| s.to_str()).unwrap_or("scan").to_string();
            let folder = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
            // The one way a scan is brought in, here and in the window (call 153).
            let brought = plumbline_plans::scan::bring_in(&bytes, other, &called, &plumbline_plans::scan::Said { units, up: Some(up), cut_at }, &mut |s| {
                plumbline_world::plans::lay_scan_sheet(&mut job, &level, &folder, s.called, s.taken, s.cut_at, s.png, s.checksum, s.size, s.cut.size, s.cut.dots, s.cut.corner)
            })?;
            let (made, taken, cut, floor, ceiling, leaned, scan_points) = (brought.laid.clone(), brought.taken.clone(), &brought.cut, brought.floor, brought.ceiling, brought.leaned, brought.points);
            let ft_in = |x: f64| plumbline_world::units::feet_as_ft_in(x, 16);
            let drawn = job.find(&made).ok_or("the drawing is not in the job")?;
            let kept = job.world.get::<plumbline_world::plans::Drawing>(drawn).and_then(|d| d.file.value.clone()).unwrap_or_default();
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let mut out = format!("{made}: scan {called:?}: {} points, {taken} (--units ft or --up y says otherwise)\n", view::figure(scan_points as f64, 0));
            // To the inch: a scan's points are each a little out, and its edge with them.
            let about = |x: f64| plumbline_world::units::feet_as_ft_in((x * 12.0).round() / 12.0, 16);
            let _ = writeln!(out, "  on plan, where it is cut, it is about {} by {}", about(cut.extent[0]), about(cut.extent[1]));
            if let Some(lean) = leaned.filter(|l| *l * 120.0 >= 0.125) {
                let _ = writeln!(out, "  it leaned {} in 10 ft and was levelled by its own floor (level it where it was cleaned for a truer sheet)", plumbline_world::units::show_length(&plumbline_world::units::Len::One((lean * 120.0 * 16.0).round() / 16.0), "in"));
            }
            match (floor, ceiling) {
                (Some(f), Some(c)) => {
                    let _ = writeln!(out, "  floor to ceiling reads about {} on the scan (it is not given to the level: measure the level's height with it if you take it)", about(c - f));
                }
                (Some(_), None) => {
                    let _ = writeln!(out, "  its floor shows and no ceiling does: no height read");
                }
                _ => {
                    let _ = writeln!(out, "  no floor shows in it: it is cut {} above its lowest point", ft_in(cut_at));
                }
            }
            let _ = writeln!(out, "  cut {} above the floor: {} points drawn on {kept}, {} by {} dots at {} to the foot", ft_in(cut_at), view::figure(cut.points as f64, 0), cut.size[0], cut.size[1], view::num(cut.dots));
            if cut.strays > 0 {
                let _ = writeln!(out, "  left off the sheet: {} stray points at that height, each alone in its square foot of plan", view::figure(cut.strays as f64, 0));
            }
            let _ = writeln!(out, "  it is a drawing under {level}, to scale and in the scan's own place, Assumed: trace over it as over any sheet");
            for lost in &brought.lost {
                let _ = writeln!(out, "  not kept: {lost}");
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        ("import", "spec") => {
            only(&["--page", "--as", "--name"])?;
            let page = match flag("--page") {
                Some(p) => p.parse::<usize>().ok().filter(|n| *n >= 1).ok_or_else(|| format!("--page {p:?} is not a page number"))?,
                None => 1,
            };
            let bytes = std::fs::read(other).map_err(|e| format!("cannot read {other}: {e}"))?;
            let words = plumbline_plans::words(&bytes, page).map_err(|why| format!("{other}: {why}"))?;
            let called = std::path::Path::new(other).file_name().and_then(|s| s.to_str()).unwrap_or(other).to_string();
            if words.is_empty() {
                return Err(format!("{other}, page {page}: there are no words on it to read. A scanned sheet is a picture of words: enter the item's sizes by hand"));
            }
            let lines: Vec<(String, f64)> = words.into_iter().map(|w| (w.text, w.size)).collect();
            let sheet = plumbline_world::spec::read(&lines);
            let name = flag("--name").map(str::to_string).or_else(|| sheet.name.as_ref().map(|f| f.value.clone())).unwrap_or_else(|| called.clone());
            let (kind, why) = match flag("--as") {
                Some("equipment") => (plumbline_world::Kind::Equipment, "as you said".to_string()),
                Some("fixture") => (plumbline_world::Kind::Fixture, "as you said".to_string()),
                Some("furnishing") => (plumbline_world::Kind::Furnishing, "as you said".to_string()),
                Some(other) => return Err(format!("--as {other:?}: an item is equipment, a fixture or a furnishing")),
                None => match plumbline_world::spec::kind_said(&name) {
                    Some((kind, word)) => (kind, format!("its name says \u{201c}{word}\u{201d}; --as says otherwise")),
                    None => return Err(format!("{other}: \u{201c}{name}\u{201d} does not say what kind of thing it is. Say which with --as equipment, --as fixture or --as furnishing")),
                },
            };
            let made = plumbline_world::spec::item_of(&sheet, &called, page, kind);
            let path_made = plumbline_world::editing::item(&mut job, &name, made.item.clone()).map_err(|why| format!("{other}: {why}"))?;
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{path_made}: {name}, {} ({why}), from {called}, page {page}", kind.a());
            if !made.read.is_empty() {
                say!("  read off the sheet, each Assumed until it is checked against the thing:");
                for (what, value, line, note) in &made.read {
                    say!("    ~ {what:<7} {value:<22} \u{201c}{line}\u{201d}{}", note.as_ref().map(|n| format!("  ({n})")).unwrap_or_default());
                }
            }
            if !made.missing.is_empty() {
                say!("  not on the sheet, and Unknown until entered:");
                for what in &made.missing {
                    say!("    ? {what}");
                }
                if made.missing.iter().any(|m| matches!(*m, "width" | "depth" | "height")) {
                    say!("  Without all three sizes it has no shape: placed, it is not drawn solid and not tested for clashes. Enter one with: plumbline measure {path} {path_made} <width|depth|height> \"34 in\" --source \"<where from>\"");
                }
            }
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
            only(if matches!(*what, "subcontractor" | "sub") { &["--division", "--system"][..] } else { &[] })?;
            let workbook = match std::path::Path::new(other).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
                Some("xlsx") => true,
                Some("csv") => false,
                _ => return Err(format!("{other}: what is written is a workbook (.xlsx) or rows of values (.csv); name the file with one of those")),
            };
            let (sheets, said) = match Lens::parse(what) {
                None => (export::estimate_out(&mut job, workbook), "the estimate"),
                Some(Lens::Supplier) => (export::lens_out(&mut job, Lens::Supplier), "the materials to quote, with none of our prices"),
                Some(Lens::Subcontractor) => (export::lens_out_for(&mut job, Lens::Subcontractor, flag("--division"), flag("--system"))?, "the subcontract scope, with none of our prices"),
                Some(Lens::Pm) => (export::lens_out(&mut job, Lens::Pm), "the lines, the budget by cost type and by division, and what is to verify, with no overhead, profit or price"),
            };
            let lines: usize = sheets.first().map(|s| s.rows.len().saturating_sub(s.head + 1)).unwrap_or(0);
            if workbook {
                let (bytes, formulas) = plumbline_sheets::workbook(&sheets)?;
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
        ("export", _) => Err(format!("export {what}: what is written out is prices, assemblies, items, estimate, supplier, subcontractor, pm or ifc\n{}", usage())),
        _ => Err(format!("import {what}: what is brought in is prices, assemblies, items, prior (an estimate made before) or ifc (a model)\n{}", usage())),
    }
}

/// Each prior estimate in the job set against the job's own lines.
/// Two or three layouts side by side, as `view::options` has them: a column for each, and the
/// last less the first. Every figure is the model's; this only sets them in columns.
fn options_said(table: &view::Options, files: &[String]) -> String {
    let mut out = String::from("flags:  ~ Assumed    ? Unknown    (no mark) Known\n\n");
    let last = table.titles.len() - 1;
    // Where two layouts have one title, each column is headed by its title and its file's
    // name (call 157): two job costs are not set side by side under the same words.
    let one_title = table.titles.iter().enumerate().any(|(i, t)| table.titles[..i].contains(t));
    let heads: Vec<String> = table
        .titles
        .iter()
        .zip(files)
        .map(|(t, f)| if one_title { format!("{t} ({})", std::path::Path::new(f).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| f.clone())) } else { t.clone() })
        .collect();
    let diff_head = format!("{} less {}", heads[last], heads[0]);
    let label_w = table.rows.iter().map(|r| r.label.chars().count()).max().unwrap_or(0).max(8);
    let widths: Vec<usize> = (0..table.titles.len())
        .map(|i| table.rows.iter().map(|r| r.figures[i].said(r.unit).chars().count()).max().unwrap_or(0).max(heads[i].chars().count()))
        .collect();
    let diff_w = table.rows.iter().map(|r| r.difference_said().chars().count()).max().unwrap_or(0).max(diff_head.chars().count());
    let mut head = format!("{:<label_w$}", "");
    for (i, t) in heads.iter().enumerate() {
        let _ = write!(head, "  {:>w$}", t, w = widths[i]);
    }
    let _ = write!(head, "  {:>diff_w$}", diff_head);
    let _ = writeln!(out, "{head}");
    let mut group = None;
    for r in &table.rows {
        if group.is_some() && group != Some(r.group) {
            out.push('\n');
        }
        group = Some(r.group);
        let mut line = format!("{:<label_w$}", r.label);
        for (i, f) in r.figures.iter().enumerate() {
            let _ = write!(line, "  {:>w$}", f.said(r.unit), w = widths[i]);
        }
        let _ = write!(line, "  {:>diff_w$}", r.difference_said());
        let _ = writeln!(out, "{line}");
    }
    out
}

/// `plumbline price-item <file> "<item>" '$180' --status <...> --source "<...>"`: one price on
/// the line named for an item, on every one of it set in the job (call 157).
fn price_item(args: &[String]) -> Result<(), String> {
    use plumbline_world::editing as ed;
    let [_, path, item, price, rest @ ..] = args else {
        return Err("price-item takes a file, an item's name, a price and then --status and --source: plumbline price-item <file> \"B18\" '$180' --status Assumed --source \"supplier quote\"".into());
    };
    let (mut status, mut source) = (None, None);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--status" if status.is_none() => status = Some(it.next().cloned().ok_or("--status needs a value")?),
            "--source" if source.is_none() => source = Some(it.next().cloned().ok_or("--source needs a value")?),
            "--status" | "--source" => return Err(format!("{a} given twice")),
            other => return Err(format!("price-item does not take {other:?}: it takes --status and --source")),
        }
    }
    let status = match status.as_deref() {
        Some("Known") => Status::Known,
        Some("Assumed") => Status::Assumed,
        Some("Unknown") => Status::Unknown,
        Some(other) => return Err(format!("{other:?} is not a status; use Known, Assumed or Unknown")),
        None => return Err("--status is required: say whether the price is Known, Assumed or Unknown".into()),
    };
    let source = source.ok_or("--source is required: say where the price came from")?;
    let (mut job, _) = read(path)?;
    let done = ed::price_item(&mut job, item, price, status, &source)?;
    if done.priced.is_empty() && done.without.is_empty() {
        say!("none of {} is set in the job yet, so nothing was priced: a price goes on each one set, and one set later takes none", done.item);
        return Ok(());
    }
    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
    say!("priced {} {} at {price} ({status:?}): {}", done.priced.len(), done.item, done.priced.join(", "));
    if !done.without.is_empty() {
        say!("  {} of {} {} no line of that name and took nothing: {}", done.without.len(), done.item, if done.without.len() == 1 { "has" } else { "have" }, done.without.join(", "));
    }
    say!("  one of {} set later takes no price: run this again then", done.item);
    Ok(())
}

fn compared_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::tables;
    let money = view::money;
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
            // A rate to the places it is held to: $12.345 is not said as $12.35.
            let rate = |x: f64| {
                let places = (2..=4).find(|p| ((x * 10f64.powi(*p)).round() / 10f64.powi(*p) - x).abs() < 1e-9).unwrap_or(4);
                format!("${}", view::figure(x, places as usize))
            };
            let prior_rate = p.unit_cost.map(|x| format!(" at {}", rate(x))).unwrap_or_default();
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
                        // One unit written two ways is one unit: SQFT is SF, LIN FT is LF.
                        let plain = |u: &str| {
                            let u: String = u.to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
                            match u.as_str() {
                                "SQFT" | "SQUAREFEET" | "SQUAREFOOT" | "SFT" => "SF".to_string(),
                                "LNFT" | "LINFT" | "LINEARFEET" | "LINEALFEET" | "LFT" => "LF".to_string(),
                                "EACH" => "EA".to_string(),
                                "SQUARE" | "SQUARES" => "SQ".to_string(),
                                "CUYD" | "CUBICYARD" | "CUBICYARDS" | "YD3" => "CY".to_string(),
                                "HRS" | "HOUR" | "HOURS" => "HR".to_string(),
                                _ => u,
                            }
                        };
                        if plain(&p.unit) == plain(&j.unit) {
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
    if cmd == "plan-set" {
        return plan_set_verb(args);
    }
    if cmd == "page" {
        return page_verb(args);
    }
    if cmd == "options" {
        let files = &args[1..];
        if files.len() < 2 || files.len() > 3 {
            return Err(format!("options sets two or three layouts side by side, each a job file; {} {} given", if files.is_empty() { "none".to_string() } else { files.len().to_string() }, if files.len() == 1 || files.is_empty() { "was" } else { "were" }));
        }
        let mut jobs = Vec::new();
        for f in files {
            jobs.push(read(f)?.0);
        }
        let mut refs: Vec<&mut plumbline_world::Job> = jobs.iter_mut().collect();
        let _ = std::io::stdout().write_all(options_said(&view::options(&mut refs), files).as_bytes());
        return Ok(());
    }
    if cmd == "price-item" {
        return price_item(args);
    }
    if cmd == "compare" {
        let [_, path] = args else {
            return Err(format!("compare takes one file and nothing else\n{}", usage()));
        };
        let (mut job, _) = read(path)?;
        let _ = std::io::stdout().write_all(compared_said(&mut job).as_bytes());
        return Ok(());
    }
    if matches!(cmd, "new" | "level" | "storey" | "type" | "system" | "wall" | "footing" | "low-slope" | "drains" | "mark" | "floor" | "door" | "window" | "room" | "room-line" | "run" | "place" | "cabinets" | "beam" | "post" | "pad" | "move" | "measure" | "remove" | "split" | "fitting" | "connect" | "drawing" | "calibrate" | "lineup") {
        return draw(args);
    }
    if matches!(cmd, "sample" | "house" | "sixteen" | "priced" | "show" | "check" | "lines" | "takeoff" | "price" | "estimate") && args.len() > 2 && !(cmd == "estimate" && args.len() == 3) {
        return Err(format!("{cmd} takes one file and nothing else; {:?} would be ignored\n{}", args[2], usage()));
    }
    match cmd {
        "sample" | "house" | "sixteen" | "priced" => {
            let out = args.get(1).ok_or_else(usage)?;
            // A made-up job is not written over a file, any more than a new job is
            // (J17A-16): the name typed may be the name of somebody's own job.
            if std::path::Path::new(out).exists() {
                return Err(format!("{out} is already there; a made-up job is not written over a file. Give it a name that is not in use"));
            }
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
                return Err(format!("figures in the file do not agree with what is drawn for: {}", report.stale.join(", ")));
            }
            let saved = file::save(&mut job);
            if saved != text {
                // Where: the first line the two part at, and what each has there.
                let (have, writes): (Vec<&str>, Vec<&str>) = (text.lines().collect(), saved.lines().collect());
                let n = have.iter().zip(&writes).position(|(a, b)| a != b).unwrap_or(have.len().min(writes.len()));
                let at = |lines: &[&str]| lines.get(n).map(|l| if l.trim().is_empty() { "an empty line".to_string() } else { format!("\u{201c}{}\u{201d}", l.trim()) }).unwrap_or_else(|| "nothing more".to_string());
                return Err(format!("{path} reads, but it is not what saving this job writes. They part at line {}: the file has {}; saving writes {}. Opened and saved, it is written the program's way", n + 1, at(&have), at(&writes)));
            }
            say!("{path}: reads, its figures agree with what is drawn, and saving it back writes the same file");
            // What a bearing wall that the work opens needs, and that it is not in the price (call 160).
            for said in plumbline_world::hidden::cautions(&mut job.world) {
                say!("! {}", said.1);
            }
            // A lump line and the things it stands for, both in the price (call 162).
            for lump in view::lump_lines(&mut job) {
                say!("! {}", lump.said());
            }
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
        // What carries the building, for the engineer (call 164).
        "structure" => {
            let path = args.get(1).ok_or_else(usage)?;
            if args.len() > 2 {
                return Err(format!("structure takes one file and nothing else; {:?} would be ignored", args[2]));
            }
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(structure(&mut job).as_bytes());
            Ok(())
        }
        // What is still to be verified in the field, and the dollars riding on each.
        "vif" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(vif(&mut job).as_bytes());
            Ok(())
        }
        // The job's items: the things that can be placed, trade by trade, each with
        // what it is drawn as, its sizes with how sure each is, and how many of it
        // are placed.
        "items" => {
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let _ = std::io::stdout().write_all(items_said(&mut job).as_bytes());
            Ok(())
        }
        "finishes" => {
            // What each wall, roof plane, window and door is finished with, read from
            // what is priced on it: what the model window dresses the house in.
            let path = args.get(1).ok_or_else(usage)?;
            let (mut job, _) = read(path)?;
            let lines = plumbline_world::finish::report(&mut job);
            let mut out = String::from("FINISHES: what each thing is dressed in, and what priced on it says so\n");
            if lines.is_empty() {
                out.push_str("  nothing in the job takes a finish yet\n");
            }
            for l in lines {
                out.push_str("  ");
                out.push_str(&l);
                out.push('\n');
            }
            let _ = std::io::stdout().write_all(out.as_bytes());
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
            let said = match args.get(2).map(String::as_str) {
                None => estimate(&mut job),
                Some("by-item") => estimate_by_item(&mut job),
                Some(other) => return Err(format!("{other:?} is not a way to read the estimate: `plumbline estimate <file>` is every line, `plumbline estimate <file> by-item` is each item once")),
            };
            let _ = std::io::stdout().write_all(said.as_bytes());
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
            let (mut status, mut source): (Option<String>, Option<String>) = (None, None);
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--status" | "--source" if (a == "--status" && status.is_some()) || (a == "--source" && source.is_some()) => return Err(format!("{a} given twice")),
                    "--status" => status = Some(it.next().cloned().ok_or("--status needs a value")?),
                    "--source" => source = Some(it.next().cloned().ok_or("--source needs a value")?),
                    "--class" | "--cost-type" => class = Some(plumbline_world::Class::parse(it.next().ok_or("--cost-type needs a cost type")?)?),
                    "--division" => division = Some(plumbline_world::Division::parse(it.next().ok_or("--division needs a division, 1 to 16")?)?),
                    "--of-material" => of_material = true,
                    other => return Err(format!("unexpected {other:?}\n{}", usage())),
                }
            }
            let (mut job, _) = read(path)?;
            // "60 SF": a measure typed outright, a value entered on the line (call 159). It
            // says how sure it is and where it came from, as every entered value does.
            // What is typed as a number and a word is read as a measure, as estimators write
            // one (1,200 SF, 60 sq ft, 24 ft); one that is not right is refused for what is
            // wrong, with the units a measure takes (call 167).
            let typed_measure = plumbline_world::units::looks_like_measure(quantity);
            let mut typed_said = quantity.trim().to_string();
            if typed_measure {
                let (x, unit) = plumbline_world::units::parse_measure_on_line(quantity)?;
                typed_said = plumbline_world::units::show_length(&plumbline_world::Len::One(x), unit);
                let status = match status.as_deref() {
                    Some("Known") => Status::Known,
                    Some("Assumed") => Status::Assumed,
                    Some("Unknown") => Status::Unknown,
                    Some(other) => return Err(format!("{other:?} is not a status; use Known, Assumed or Unknown")),
                    None => return Err(format!("{quantity:?} is a measure typed on the line, so --status is required: say whether it is Known or Assumed")),
                };
                let source = source.as_deref().ok_or_else(|| format!("{quantity:?} is a measure typed on the line, so --source is required: say where it came from"))?;
                job.add_line_measured(element, description, quantity, class, of_material, status, source)?;
            } else if status.is_some() || source.is_some() {
                return Err(format!("--status and --source go with a measure typed on the line, like \"60 SF\"; {quantity:?} is a formula or a count"));
            } else {
                job.add_line(element, description, quantity, class, of_material)?;
            }
            if let Some(division) = division {
                job.set_division(element, Some(description), division)?;
            }
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("added to {element}: {description:?} = {}{}{}", if typed_measure { format!("{} ({}, {})", typed_said, status.as_deref().unwrap_or_default(), source.as_deref().unwrap_or_default()) } else { quantity.to_string() }, if of_material { ", made of the element's material".to_string() } else { ", with no price yet".to_string() }, division.map(|d| format!("; {}", d.said())).unwrap_or_default());
            Ok(())
        }
        // One cost line taken off a thing, with whatever was entered on it (call 166).
        "remove-line" => {
            let [_, path, element, description] = args else {
                return Err("remove-line takes the thing and the line to take off it: plumbline remove-line <file> <thing> \"<line>\"".to_string());
            };
            let (mut job, _) = read(path)?;
            // What the line came to before it goes, as the estimate has it.
            job.derive();
            let before = view::estimate(&mut job);
            let was = before.lines.iter().find(|l| l.path == *element && l.description == *description).map(|l| (l.quantity.value, l.quantity.unit.clone(), l.amount.value));
            job.remove_line(element, description)?;
            job.derive();
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            let stood = match was {
                Some((Some(q), unit, Some(amount))) => format!("{} {unit}, {}", view::num(q), view::money(amount)),
                Some((Some(q), unit, None)) => format!("{} {unit}, with no price on it", view::num(q)),
                _ => "with no figure on it".to_string(),
            };
            say!("took {description:?} off {element}: {stood}");
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
        // A line's cost type; "class" is the word this verb had before and is still taken.
        "cost-type" | "class" | "labor" => {
            let [_, path, element, value, rest @ ..] = args else { return Err(usage()) };
            let line = match rest {
                [] => None,
                [flag, line] if flag == "--line" => Some(line.as_str()),
                _ => return Err(usage()),
            };
            let (mut job, _) = read(path)?;
            if cmd != "labor" {
                let class = plumbline_world::Class::parse(value)?;
                job.set_class(element, line, class)?;
                say!("{element}{}: cost type is {}", line.map(|l| format!(" {l:?}")).unwrap_or_default(), class.title());
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
            // --change: in place of the assembly the thing has, as the window's list does it.
            let (source, change) = match rest {
                [flag, source] if flag == "--source" => (source, false),
                [flag, source, change] | [change, flag, source] if flag == "--source" && change == "--change" => (source, true),
                _ => return Err("--source is required: say who says this thing is that assembly".into()),
            };
            let (mut job, report) = read(path)?;
            if !report.stale.is_empty() {
                say!("WARNING: before this change, the file's figures did not agree with what is drawn for: {}. They have been worked out again.", report.stale.join(", "));
            }
            let priced_before = view::estimate(&mut job);
            let (added, taken) = if change { job.change_assembly(element, condition, Status::Known, source)? } else { (job.apply(element, condition, source)?, Vec::new()) };
            job.derive();
            let came_off = view::came_off_said(&priced_before, &mut job);
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            say!("{element} is now {:?} and has gained {} cost {}:", job.shown(element, "condition", None).unwrap_or_default(), added.len(), if added.len() == 1 { "line" } else { "lines" });
            // On a thing that is existing or coming down these are not lines for building
            // it, and that is said where they go on (judged 2026-11-21).
            if let Some(w) = job.find(element).and_then(|e| job.world.get::<plumbline_world::components::Work>(e)).and_then(|w| w.status.value).filter(|w| matches!(w, plumbline_world::components::WorkStatus::Existing | plumbline_world::components::WorkStatus::Demolish)) {
                say!("! {element} is {}: an assembly's lines are for building a thing new, and these are now priced on it. Take off what is not done to it", w.said());
            }
            for d in added {
                say!("  {d}");
            }
            if !taken.is_empty() {
                say!("{} of the assembly it had came off, with whatever was priced on {}:", if taken.len() == 1 { "1 cost line".to_string() } else { format!("{} cost lines", taken.len()) }, if taken.len() == 1 { "it" } else { "them" });
                for d in taken {
                    say!("  {d}");
                }
                if let Some(said) = came_off {
                    say!("  {said}");
                }
            }
            Ok(())
        }
        "set" => {
            let [_, path, element, field, value, rest @ ..] = args else { return Err(usage()) };
            // A field is named by its words, joined or not: "unit cost" is unit_cost.
            let field = &field.trim().replace(' ', "_");
            let said = view::words(&field.to_lowercase());
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
                        // A name is given by its words, joined or not: "turn up" is turn_up.
                        let name = name.trim().replace(' ', "_");
                        if with.iter().any(|(have, _)| *have == name) {
                            return Err(format!("--with gives {:?} twice", view::words(&name)));
                        }
                        with.push((name, typed.trim().to_string()));
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
                say!("WARNING: before this change, the file's figures did not agree with what is drawn for: {}. They have been worked out again.", report.stale.join(", "));
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
                let priced_before = view::estimate(&mut job);
                let solved = solve::edit(&mut job, element, change)?;
                job.derive();
                let after = file::save(&mut job);
                std::fs::write(path, &after).map_err(|e| format!("cannot write {path}: {e}"))?;
                say!("set {element} {said} = {typed} ({status:?}) and solved the roof again");
                say_solve(&solved);
                // A plane that is gone took what was priced on it: said to the cent (J23-3).
                if let Some(went) = view::went_with_said(&priced_before, &mut job) {
                    say!("  {went}");
                    // And what the planes that stay did to it (call 103).
                    if let Some(whole) = view::job_cost_was_and_is_said(&priced_before, &mut job) {
                        say!("  {whole}");
                    }
                }
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
            // A material's order factor reaches every line made of it: which of them
            // come to a figure now, to see that none loses it (J17A-13).
            let had: Vec<(String, String)> = if field == "order" && line.is_none() { view::estimate(&mut job).lines.iter().filter(|l| !l.off && l.amount.value.is_some()).map(|l| (l.path.clone(), l.description.clone())).collect() } else { Vec::new() };
            let dropped = if field == "quantity" {
                let formula = value.ok_or("a quantity is a formula; it cannot be marked unknown")?;
                job.set_formula(element, line.as_deref(), formula, &with, status, &source)?
            } else {
                job.set_on_line(element, field, line.as_deref(), value, status, &source)?;
                Vec::new()
            };
            let changed = job.derive();
            if !had.is_empty() {
                let now = view::estimate(&mut job);
                let lost: Vec<String> = now.lines.iter().filter(|l| l.amount.value.is_none() && had.contains(&(l.path.clone(), l.description.clone()))).map(|l| format!("{:?} on {} ({})", l.description, l.element, l.amount.note)).collect();
                if !lost.is_empty() {
                    return Err(format!(
                        "{} does not fit how this material is bought and what is made of it is measured in: with it, {} that {} to a figure now would come to none: {}. An order factor turns what a line measures into what is bought (1.25 lb/SF, 32 SF/sheet); a material bought by what its lines already measure needs none. Nothing was changed",
                        value.unwrap_or("that"),
                        if lost.len() == 1 { "1 line".to_string() } else { format!("{} lines", lost.len()) },
                        if lost.len() == 1 { "comes" } else { "come" },
                        lost.join("; ")
                    ));
                }
            }
            std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
            // Reported back as the job now holds it, in the unit it was entered in.
            let shown = job.shown(element, field, line.as_deref()).unwrap_or_else(|| value.unwrap_or("unknown").to_string());
            if field == "quantity" {
                say!("set {element} {said} = {shown}");
                for (name, _) in &with {
                    say!("  with {name} = {} ({status:?})", job.shown(element, name, line.as_deref()).unwrap_or_default());
                }
                for name in &dropped {
                    say!("  {name} is no longer used by the formula and has been taken off the line");
                }
            } else {
                say!("set {element} {said} = {shown} ({status:?})");
                // Overhead or profit over 100 % is taken, and answered (J23-8).
                if let Some(large) = value.filter(|_| matches!(field.as_str(), "overhead" | "profit")).and_then(view::markup_large_said) {
                    say!("  {large}");
                }
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
            } else if changed.is_empty() && matches!(field.as_str(), "pitch" | "eave_height") && job.find(element).is_some_and(|e| job.world.get::<plumbline_world::Solved>(e).is_some()) {
                // Put on the roof this plane is solved from, which was solved again (J23-1).
                say!("it is held on the roof this plane is solved from, which was solved again: its planes and their eaves, hips, valleys and ridges follow");
            } else if changed.is_empty() {
                say!("no measure changed");
            } else {
                say!("figures worked out again on: {}", changed.join(", "));
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
            let _ = writeln!(out, "  {} {} {} {}: {} ({})", r.status.flag(), el.path, el.name, view::words(&r.label), r.text, why);
        }
    }
    if !report.stale.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "WARNING: the file's figures did not agree with what is drawn for: {}. The figures above are worked out again from what is drawn.", report.stale.join(", "));
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

fn items_said(job: &mut plumbline_world::Job) -> String {
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
            Some(said) => format!("a {said}"),
            None => match forms::form_of(&n.name) {
                Some(form) => format!("a {}, by its name", form.name()),
                None => "a box, its name saying nothing else".to_string(),
            },
        };
        let kind = match it.category {
            plumbline_world::Kind::Equipment => "equipment",
            plumbline_world::Kind::Fixture => "fixture",
            _ => "furnishing",
        };
        by_trade.entry(it.division.map(Division::number).unwrap_or(0)).or_default().push(format!("    {} [{}]: {kind}, drawn as {drawn}; {} wide, {} deep, {} high; {} placed", n.name, n.path, size(&it.width), size(&it.depth), size(&it.height), placed));
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
    out
}

fn say_solve(report: &solve::Report) {
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
        let value = if flag == "--closed" || flag == "--all" || flag == "--walls" || flag == "--countertop" || flag == "--clear" || flag == "--keep" || flag == "--alone" { String::new() } else { it.next().cloned().ok_or_else(|| format!("{flag} needs a value"))? };
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
        "furnishing" => Ok(Kind::Furnishing),
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
                return Err(format!("{d} is a plan of {its}, not of {onto}: what is traced over a drawing is on the level it is a plan of"));
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
            // The scale as a drawing writes it (call 88), from the model.
            let scale = job.find(drawing).and_then(|e| job.world.get::<plans::Drawing>(e)).map(|d| scale_words(d, k)).unwrap_or_default();
            let mut said = format!("{drawing}: {} is {} on its page: {scale}", distance.trim(), view::num(((to[0] - from[0]).hypot(to[1] - from[1]) * 1.0e4).round() / 1.0e4));
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
        // A level; "storey" is the word this verb had before and is still taken.
        ("level" | "storey", [name]) => {
            let (status, source) = how(&status, &source)?;
            let made = ed::storey(&mut job, name, &need(flag("--elevation"), "--elevation", "\"0 ft\"")?, &need(flag("--height"), "--height", "\"9 ft\"")?, ed::Entered { status, source: &source })?;
            format!("{made}: level {name:?}")
        }
        ("type", [what, name]) => {
            let (status, source) = how(&status, &source)?;
            let how = ed::Entered { status, source: &source };
            let made = match what.as_str() {
                "wall" => ed::wall_type(&mut job, name, &need(flag("--thickness"), "--thickness", "\"6 in\"")?, how)?,
                // A footing's type says how wide footings of it are.
                // (Held as a wall type is; what is wrong with it is said as a footing's.)
                "footing" => ed::footing_type(&mut job, name, &need(flag("--width"), "--width", "\"24 in\"")?, how)?,
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
                "beam" | "post" | "pad" => {
                    let kind = match what.as_str() {
                        "beam" => Kind::Beam,
                        "post" => Kind::Post,
                        _ => Kind::PadFooting,
                    };
                    let (round, width, depth, weight) = (flag("--round"), flag("--width"), flag("--depth"), flag("--weight"));
                    match (round, width, depth) {
                        (Some(across), None, None) => ed::framing_type(&mut job, name, kind, &across, None, weight.as_deref(), how)?,
                        (None, Some(width), Some(depth)) => ed::framing_type(&mut job, name, kind, &width, Some(&depth), weight.as_deref(), how)?,
                        _ => return Err(format!("a {what}'s type is --width \"3 1/2 in\" --depth \"11 7/8 in\"{}", if kind == Kind::PadFooting { "" } else { ", or --round \"6 in\"" })),
                    }
                }
                other => return Err(format!("{other:?} has no type here; a type is for a wall, floor, door, window, duct, pipe, conduit, cable, beam, post or pad")),
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
                _ => return Err("say how tall the walls are: --height \"9 ft\", or --top <the level their top is held to>; one of them".into()),
            };
            let made = ed::walls(&mut job, storey, type_path, &pts, closed, top, ed::Entered { status, source: &source })?;
            format!("{} wall{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
        }
        ("mark", [paths @ .., word]) if !paths.is_empty() => {
            let (status, source) = how(&status, &source)?;
            let what = plumbline_world::components::WorkStatus::read(word).ok_or_else(|| format!("{word:?} is not what the work does to a thing; say new, existing, demolish or temporary"))?;
            let its = match (flag("--clear").is_some(), flag("--keep").is_some()) {
                (true, true) => return Err("--clear and --keep are one or the other".into()),
                (true, false) => ed::ItsLines::Clear,
                (false, true) => ed::ItsLines::Keep,
                (false, false) => ed::ItsLines::Ask,
            };
            ed::mark(&mut job, paths, what, its, ed::Entered { status, source: &source }).map_err(|why| if why.contains("Say which:") { format!("{why}: --clear takes them off, --keep leaves them priced") } else { why })?.join("\n")
        }
        ("mark", _) => return Err("mark takes the things to mark and then what the work does to them, before any flag: plumbline mark <file> wall-0004 wall-0005 existing --status Known --source \"survey\" (new, existing, demolish or temporary)".into()),
        ("low-slope", [name]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"0,0 48,0 48,48 0,48\" (feet)")?)?;
            let pitch = need(flag("--pitch"), "--pitch", "1/4:12")?;
            let height = need(flag("--height"), "--height", "\"11 ft\" (how high its deck is)")?;
            let made = ed::low_slope(&mut job, name, &pts, &pitch, &height, ed::Entered { status, source: &source })?;
            format!("{made}: a low-slope area, {name:?}, at {pitch}")
        }
        ("drains", [area]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"24,24\", or \"12,24 36,24\" for two (feet)")?)?;
            let at_drain = need(flag("--at-drain"), "--at-drain", "\"1/2 in\" (how thick the insulation is at a drain)")?;
            let had = job.find(area).and_then(|e| job.world.get::<plumbline_world::components::Taper>(e)).and_then(|t| t.drains.value.as_ref().map(Vec::len));
            ed::drains(&mut job, area, &at, &at_drain, ed::Entered { status, source: &source })?;
            let given = job.find(area).and_then(|e| job.world.get::<plumbline_world::components::Taper>(e)).and_then(|t| t.at_drain.value.as_ref().map(|l| units::show_length(l, &t.at_drain.unit))).unwrap_or_else(|| at_drain.trim().to_string());
            job.derive();
            let figure = |job: &mut plumbline_world::Job, name: &str| -> String {
                let d = job.find(area).and_then(|e| job.world.get::<plumbline_world::components::Measure>(e)).and_then(|m| m.0.get(name).cloned());
                match d {
                    Some(d) => match d.value {
                        Some(v) => format!("{}{} {}", d.status.flag(), view::num(v), d.unit),
                        None => format!("none ({})", d.note),
                    },
                    None => "none".to_string(),
                }
            };
            format!(
                "{area}: {}{}, the insulation {} at a drain. Its taper: thickest {}, average {}, {}; valleys {}",
                plumbline_world::view::so_many(at.len(), "drain"),
                had.map(|n| format!(" in place of the {} it had", plumbline_world::view::so_many(n, "drain"))).unwrap_or_default(),
                given,
                figure(&mut job, "thickest"),
                figure(&mut job, "average_thickness"),
                figure(&mut job, "board_feet"),
                figure(&mut job, "valley_length"),
            )
        }
        ("footing", [storey, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"0,0 40,0 40,28\" (feet)")?)?;
            let depth = need(flag("--depth"), "--depth", "\"12 in\"")?;
            let made = ed::footings(&mut job, storey, type_path, &pts, flag("--closed").is_some(), &depth, ed::Entered { status, source: &source })?;
            format!("{} footing{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
        }
        ("beam", [storey, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"12,0 12,16\" (feet)")?)?;
            let top = need(flag("--top"), "--top", "\"9 ft\" (the top of the beam above the level's floor)")?;
            let made = ed::beams(&mut job, storey, type_path, &pts, &top, ed::Entered { status, source: &source })?;
            format!("{} beam{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
        }
        ("post", [storey, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"12,0\" (feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let made = ed::post(&mut job, storey, type_path, *at, flag("--height").as_deref(), flag("--base").as_deref(), ed::Entered { status, source: &source })?;
            let said = job.find(&made).and_then(|e| job.world.get::<plumbline_world::Stood>(e)).map(|c| if c.height.status == Status::Assumed && c.height.source.starts_with("stands to its level's height") { format!(": it stands to its level's height, {} (Assumed): say how tall it is with --height", c.height.value.as_ref().map(|l| units::show_length(l, &c.height.unit)).unwrap_or_default()) } else { String::new() }).unwrap_or_default();
            format!("{made}: a post{said}")
        }
        ("pad", [storey, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"12,0\" (feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let thickness = need(flag("--thickness"), "--thickness", "\"18 in\" (how thick the pad is: nobody's default is taken)")?;
            let top = need(flag("--top"), "--top", "\"-1 ft\" (its top above the level's floor; below it, a minus height)")?;
            let made = ed::pad(&mut job, storey, type_path, *at, &thickness, &top, ed::Entered { status, source: &source })?;
            format!("{made}: a pad footing")
        }
        ("door" | "window", [wall, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let (offset, at) = (flag("--offset"), flag("--at"));
            let point = at.as_deref().map(points).transpose()?;
            let place = match (&offset, &point) {
                (Some(o), None) => ed::OnWall::Offset(o),
                (None, Some(p)) if p.len() == 1 => ed::OnWall::Point(p[0]),
                _ => return Err("say where along the wall its center is: --offset \"5 ft\" from the wall's first end, or --at \"5,0\" a point on the plan; one of them".into()),
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
                _ => return Err("a floor is given by --walls (the outline its level's walls enclose) or by --points \"0,0 30,0 30,20 0,20\" (its corners, in feet): one of the two".into()),
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
        ("room-line", [storey]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"24,0 24,16\" (feet)")?)?;
            let made = ed::room_lines(&mut job, storey, &pts, ed::Entered { status, source: &source })?;
            let mut said = format!("{} room line{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "));
            for note in ed::room_lines_unreached_said(&mut job, &made) {
                said.push_str(&format!("\n  {note}"));
            }
            said
        }
        ("run", [storey, what, type_path]) => {
            let (status, source) = how(&status, &source)?;
            let pts = points(&need(flag("--points"), "--points", "\"2,2 14,2 14,7\" (feet)")?)?;
            let made = ed::runs(&mut job, storey, kind_of(what)?, type_path, &pts, &need(flag("--height"), "--height", "\"8 ft\" above the level's floor")?, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
            format!("{} straight piece{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
        }
        // One of an item (made from a maker's sheet): its size is the item's.
        ("place", [storey, item]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"14,7\" (feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let made = ed::place_item(&mut job, storey, item, *at, &flag("--offset").unwrap_or_else(|| "0 ft".to_string()), ed::Entered { status, source: &source })?;
            let size = job.find(&made).and_then(|e| job.world.get::<plumbline_world::Placed>(e)).map(|p| match p.size_ft() {
                Some([w, d, h]) => format!("{} wide, {} deep, {} high{}", units::feet_as_ft_in(w, 16), units::feet_as_ft_in(d, 16), units::feet_as_ft_in(h, 16), if p.size.status == Status::Known { "" } else { " (as its sheet says; not yet checked)" }),
                None => format!("no size yet, so it has no shape and is not tested for clashes: {}", p.size.source),
            });
            format!("{made}: one of {item}{}", size.map(|s| format!(", {s}")).unwrap_or_default())
        }
        // A run of cabinet boxes along the face a wall has on a room's side, by their codes (call 155).
        ("cabinets", [storey, wall]) => {
            let (status, source) = how(&status, &source)?;
            let room = need(flag("--in"), "--in", "room-1 (the room the boxes stand in)")?;
            let from = need(flag("--from"), "--from", "\"1 ft\" (how far along the wall from its first end the run starts)")?;
            let codes = need(flag("--boxes"), "--boxes", "\"B18 SB36 B24\" (the boxes' codes, in order along the wall)")?;
            let codes: Vec<String> = ed::box_list(&codes)?;
            let top = flag("--countertop").is_some();
            let made = ed::cabinets(&mut job, storey, wall, &room, &from, &codes, top, ed::Entered { status, source: &source })?;
            let mut out = format!(
                "set {} along {wall} in {room}: a run of {} ft, from {} ft to {} ft along the wall; {} ft left to the corner",
                if made.boxes.len() == 1 { "1 box".to_string() } else { format!("{} boxes", made.boxes.len()) },
                view::num(made.run),
                view::num(made.from),
                view::num(made.from + made.run),
                view::num(made.left)
            );
            for (path, code, at, width) in &made.boxes {
                let division = job.find(path).and_then(|e| job.world.get::<plumbline_world::components::Cost>(e)).and_then(|c| c.lines.first().and_then(|l| l.division)).map(|d| d.number()).unwrap_or(12);
                out.push_str(&format!("\n  {path}: {code}, {} wide, middle at {}, {}{}; one line, {code}, 1 EA, Division {division}, no price", units::feet_as_ft_in(*width, 16), view::num(at[0]), view::num(at[1]), made.hung.map(|up| format!(", hung {} up", units::feet_as_ft_in(up, 16))).unwrap_or_default()));
            }
            if let Some((path, at)) = &made.countertop {
                let mut figure = |name: &str| job.find(path).and_then(|e| job.world.get::<plumbline_world::components::Measure>(e)).and_then(|m| m.0.get(name).and_then(|d| d.worked()));
                let (long, area) = (figure("width"), figure("plan_area"));
                out.push_str(&format!(
                    "\n  {path}: countertop, {} long by 25 1/2 in deep, 34 1/2 in up, middle at {}, {}; {} SF, front edge {} LF",
                    long.map(|x| format!("{} ft", view::num(x))).unwrap_or_else(|| "unknown".into()),
                    view::num(at[0]),
                    view::num(at[1]),
                    area.map(view::num).unwrap_or_else(|| "unknown".into()),
                    long.map(view::num).unwrap_or_else(|| "unknown".into())
                ));
                out.push_str("; one line, Countertop, by its area, Division 12, no price");
                // What its area leaves out (the leg over a box that turns a corner), and
                // what is in the price twice (call 162).
                let note = job.find(path).and_then(|e| job.world.get::<plumbline_world::components::Measure>(e)).and_then(|m| m.0.get("plan_area").map(|d| d.note.clone())).unwrap_or_default();
                if !note.is_empty() {
                    out.push_str(&format!("\n  {path}: {note}"));
                }
            }
            for lump in view::lump_lines(&mut job).into_iter().filter(|l| l.room == room) {
                out.push_str(&format!("\n  ! {}", lump.said()));
            }
            out
        }
        ("place", [storey, what, name]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"14,7\" (feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let size = match flag("--size") {
                None => None,
                Some(text) => match text.split(',').map(units::parse_number).collect::<Result<Vec<f64>, String>>()?.as_slice() {
                    [a, b, c] => Some([*a, *b, *c]),
                    _ => return Err("--size is along,across,up in feet, like \"2,2,6\"".into()),
                },
            };
            let made = ed::place(&mut job, storey, kind_of(what)?, name, *at, &need(flag("--offset"), "--offset", "\"0 ft\" above the level's floor")?, size, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
            format!("{made}: {name:?}")
        }
        ("move", [element]) => {
            let (status, source) = how(&status, &source)?;
            let by = points(&need(flag("--by"), "--by", "\"3,2\" (feet east, feet north)")?)?;
            let [by] = by.as_slice() else { return Err("--by is one distance, east,north in feet".into()) };
            // A wall takes the ends that meet it with it unless moved alone (call 170).
            let moved = ed::move_wall(&mut job, element, by[0], by[1], ed::Entered { status, source: &source }, flag("--alone").is_some())?;
            let mut out = format!("{element} moved {} ft east and {} ft north", view::num(by[0]), view::num(by[1]));
            for line in moved.said() {
                out.push_str(&format!("\n  {line}"));
            }
            out
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
                Some(other) => return Err(format!("--to {other:?}: a run's height is measured to its bottom, its center or its top")),
            };
            let reading = if reading.trim().eq_ignore_ascii_case("same") { ed::Reading::Same } else { ed::Reading::Length(reading) };
            // A value entered on a cost line (call 159) is named with the line.
            let done = match flag("--line") {
                Some(line) => ed::measure_on_line(&mut job, element, &line, &what.trim().replace(' ', "_"), reading, &source)?,
                None => ed::measure(&mut job, element, what, reading, to, &source)?,
            };
            let mut out = done.said;
            for also in done.also {
                out.push_str(&format!("\n  {also}"));
            }
            out
        }
        ("remove", [element]) => {
            let before = view::estimate(&mut job);
            // What is set against a wall's faces still stands when the wall goes (call 157).
            let against = ed::set_against_said(&mut job, element);
            let gone = ed::remove(&mut job, element)?;
            let mut said = match view::went_with_said(&before, &mut job) {
                Some(went) => format!("removed {}\n  {went}", gone.join(", ")),
                None => format!("removed {}", gone.join(", ")),
            };
            if let Some(against) = against {
                said.push_str(&format!("\n  {against}"));
            }
            said
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
                return Err("say where: plumbline fitting <file> <level> --at \"10,2\", or --all for every joint that has none".into());
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
            format!("{thing} is joined to {run} by its connection {which}")
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

/// `plan-set <file> <level> <out.pdf> [--scale ..] [--paper ..] [--date ..] [--list]` (call 168).
fn plan_set_verb(args: &[String]) -> Result<(), String> {
    use plumbline_world::paper::size;
    let (mut plain, mut scale, mut paper, mut date, mut list) = (Vec::new(), None, None, None, false);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" => list = true,
            f @ ("--scale" | "--paper" | "--date") => {
                let v = args.get(i + 1).ok_or_else(|| format!("{f} needs a value after it"))?.clone();
                let slot = match f {
                    "--scale" => &mut scale,
                    "--paper" => &mut paper,
                    _ => &mut date,
                };
                if slot.is_some() {
                    return Err(format!("{f} is given twice"));
                }
                *slot = Some(v);
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("plan-set does not take {f}\n{}", usage())),
            a => plain.push(a.to_string()),
        }
        i += 1;
    }
    let [path, level, out] = plain.as_slice() else {
        return Err(format!("plan-set takes the job's file, the level and the PDF to write\n{}", usage()));
    };
    let opts = plumbline_world::plan_set::Options {
        scale: match scale {
            None => None,
            Some(t) => Some(size::scale(&t).ok_or_else(|| format!("{t:?} is not a scale this program draws a plan set at: 1/8, 3/16, 1/4, 3/8 or 1/2 inch to the foot"))?),
        },
        paper: match paper {
            None => None,
            Some(t) => Some(size::paper(&t).ok_or_else(|| format!("{t:?} is not a paper this program writes a plan set on: ARCH A, ARCH B, ARCH C or ARCH D"))?),
        },
        date: match date {
            None => None,
            Some(t) if t.trim().is_empty() || t.chars().count() > 24 => return Err("--date is the day to letter on the title box: a few characters, like 2026-11-22".into()),
            Some(t) => Some(t.trim().to_string()),
        },
    };
    // What is written out is never written over a job.
    if let (Ok(a), Ok(b)) = (std::fs::canonicalize(path), std::fs::canonicalize(out)) {
        if a == b {
            return Err(format!("{out} is the job itself: a plan set is not written over it. Give it a name of its own"));
        }
    }
    if std::fs::read_to_string(out).ok().is_some_and(|text| file::load(&text).is_ok()) {
        return Err(format!("{out} is a job: a plan set is not written over a job. Give it a name that is not in use"));
    }
    let (mut job, _) = read(path)?;
    let set = plumbline_world::plan_set::plan_set(&mut job, level, &opts)?;
    std::fs::write(out, set.pdf()).map_err(|e| format!("cannot write {out}: {e}"))?;
    say!("wrote {out}: {}, {}, {} paper, {}", count(set.pages.len(), "page", "pages"), set.pages.iter().map(|p| p.show.name()).collect::<Vec<_>>().join(", "), set.paper.name, plumbline_world::paper::size::scale_said(set.scale));
    for (show, why) in &set.not_written {
        say!("not written: {}: {why}", show.name());
    }
    if list {
        say!();
        let _ = std::io::stdout().write_all(set.list().as_bytes());
    }
    Ok(())
}

/// `page <file> [--page n] [--lines [--longer-than pt]] [--png out.png]`: what a page of a
/// PDF holds, as this program's own reader sees it.
fn page_verb(args: &[String]) -> Result<(), String> {
    let (mut plain, mut page, mut lines, mut longer, mut png) = (Vec::new(), 1usize, false, 0.0f64, None);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--lines" => lines = true,
            f @ ("--page" | "--longer-than" | "--png") => {
                let v = args.get(i + 1).ok_or_else(|| format!("{f} needs a value after it"))?;
                match f {
                    "--page" => page = v.parse().map_err(|_| format!("--page is a page number, not {v:?}"))?,
                    "--longer-than" => longer = units::parse_number(v)?,
                    _ => png = Some(v.clone()),
                }
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("page does not take {f}\n{}", usage())),
            a => plain.push(a.to_string()),
        }
        i += 1;
    }
    let [file] = plain.as_slice() else {
        return Err(format!("page takes one file\n{}", usage()));
    };
    let bytes = std::fs::read(file).map_err(|e| format!("cannot read {file}: {e}"))?;
    let got = plumbline_plans::read(&bytes, page)?;
    say!("{file}, page {page} of {}: {}, {} by {} {} ({} by {} in), {}", got.pages, got.sort.said(), view::num(got.size[0]), view::num(got.size[1]), page_unit(got.sort), view::num(got.size[0] / 72.0), view::num(got.size[1] / 72.0), lines_said(got.lines.len()));
    if lines {
        for [a, b] in got.lines.iter().filter(|[a, b]| (b[0] - a[0]).hypot(b[1] - a[1]) >= longer) {
            say!("line {:.2},{:.2} to {:.2},{:.2}: {:.2} points ({:.4} in)", a[0], a[1], b[0], b[1], (b[0] - a[0]).hypot(b[1] - a[1]), (b[0] - a[0]).hypot(b[1] - a[1]) / 72.0);
        }
    }
    if let Some(out) = png {
        std::fs::write(&out, plumbline_plans::png(&got.picture)?).map_err(|e| format!("cannot write {out}: {e}"))?;
        say!("wrote {out}: {} by {} dots", got.picture.width, got.picture.height);
    }
    Ok(())
}
