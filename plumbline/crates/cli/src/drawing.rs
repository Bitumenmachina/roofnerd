use super::*;

/// What a page is measured in until its drawing has a scale.
/// A drawing's scale in words (call 88). A sheet of paper says it as a drawing does:
/// `1/4" = 1'-0"`. A picture has no paper size: it says how many of its dots make a
/// foot, and what the sheet covers on the ground.
pub(super) fn scale_words(d: &plans::Drawing, feet_per_unit: f64) -> String {
    let dots = view::num((1.0e4 / feet_per_unit).round() / 1.0e4);
    match d.scale_said() {
        Some(said) if said.starts_with("sheet covers") => format!("{dots} dots to the foot; the {said}"),
        Some(said) => said,
        None => format!("{dots} to the foot"),
    }
}

pub(super) fn page_unit(sort: plumbline_plans::Sort) -> &'static str {
    match sort {
        plumbline_plans::Sort::Pdf => "points",
        plumbline_plans::Sort::Dxf => "drawing units",
        _ => "dots",
    }
}

/// A CAD drawing's curves are straight pieces here, so its count is of pieces, not of what the drawing holds.
pub(super) fn pieces_said(sort: plumbline_plans::Sort, n: usize) -> String {
    if sort == plumbline_plans::Sort::Dxf { format!("{n} straight pieces to snap to") } else { lines_said(n) }
}

pub(super) fn lines_said(n: usize) -> String {
    match n {
        0 => "no lines to snap to".to_string(),
        1 => "1 straight line to snap to".to_string(),
        n => format!("{n} straight lines to snap to"),
    }
}

/// A plan file's path as the job keeps it: from the folder the job is in.
pub(super) fn from_the_jobs_folder(job: &str, file: &str) -> Result<String, String> {
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
pub(super) fn drawings(job: &mut plumbline_world::Job, path: &str) -> String {
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
                match plumbline_plans::read_with(&bytes, d.page, Some(&d.layers_off)) {
                    Err(why) => {
                        let _ = writeln!(out, "    page: cannot be read: {why}");
                    }
                    Ok(page) => {
                        let _ = writeln!(out, "    page: {}, {} by {} {}, {}", page.sort.said(), view::num(page.size[0]), view::num(page.size[1]), page_unit(page.sort), pieces_said(page.sort, page.lines.len()));
                        if (page.size[0] - d.size[0]).abs() > 0.01 || (page.size[1] - d.size[1]).abs() > 0.01 {
                            let _ = writeln!(out, "    page: IS NOT THE SIZE THIS DRAWING WAS MADE AT, {} by {}. The sheet does not fit what was traced over it; it is not shown.", view::num(d.size[0]), view::num(d.size[1]));
                        }
                        if !d.layers_off.is_empty() {
                            let _ = writeln!(out, "    layers off: {}", d.layers_off.join(", "));
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

/// `plumbline roof <file> <name> --points "x,y x,y ..." --pitch <p> --eave-height <len> --status <S> --source <where>`
/// The drawing verbs: a job started from nothing, and a building put into it a storey, a
/// type, a wall, an opening, a run and a fixture at a time. Each is one call into
/// `plumbline_world::editing`, which the window calls too.
pub(super) fn draw(args: &[String]) -> Result<(), String> {
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
            // A closed format is refused by name, whatever the file is called (call 221 point 7).
            if let Some(why) = plumbline_plans::dxf::closed_format(&bytes, &given) {
                return Err(why);
            }
            // Kept in the job by its path from the job's folder, so the two can be moved together.
            let kept = from_the_jobs_folder(path, &given)?;
            if plumbline_plans::sort_of(&bytes) == Some(plumbline_plans::Sort::Dxf) {
                if page != 1 {
                    return Err(format!("{given}: a CAD drawing has one page; page {page} was asked for"));
                }
                cad::bring_in(&mut job, &kept, storey, name, &bytes, status, &source).map_err(|why| format!("{given}: {why}"))?
            } else {
            let read = plumbline_plans::read(&bytes, page).map_err(|why| format!("{given}: {why}"))?;
            let made = plans::add(&mut job, storey, name, plans::Read { file: &kept, page, bytes: bytes.len() as u64, checksum: &plumbline_plans::checksum(&bytes), size: read.size }, ed::Entered { status, source: &source })?;
            let mut out = format!("{made}: drawing {name:?}: page {page} of {} of {kept} ({}), {} by {} {}, {}; no scale yet", read.pages, read.sort.said(), view::num(read.size[0]), view::num(read.size[1]), page_unit(read.sort), lines_said(read.lines.len()));
            for lost in &read.lost {
                out.push_str(&format!("\n  not read: {lost}"));
            }
            // The marks of a marked-up PDF are kept with the sheet (call 227 point 7), and said; a page with none is as it was.
            if read.sort == plumbline_plans::Sort::Pdf {
                match plumbline_plans::marks::read(&bytes, page) {
                    Ok(got) if !got.marks.is_empty() => {
                        plumbline_world::marks::set_shapes(&mut job, &made, marks::shapes(&marks::file_name(&given), page, &got))?;
                        out.push_str(&marks::kept_said(&got));
                    }
                    Ok(got) => out.push_str(&marks::kept_said(&got)),
                    Err(why) => out.push_str(&format!("\n  not read: its marks could not be read: {why}")),
                }
            }
            out
            }
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
            if let Some(caution) = job.find(drawing).and_then(|e| job.world.get::<plans::Drawing>(e)).and_then(|d| d.scale_caution()) {
                said.push_str(&format!("\n  caution: {caution}"));
            }
            if done.lined_up_again {
                said.push_str("\n  it was lined up, and the place it was lined up by is where it was");
            }
            if !done.left_at_earlier_scale.is_empty() {
                // Walls laid from the sheet are named in the same words (call 191 item 15); where none was, the old words stand.
                let laid = plans::laid_from(&mut job, drawing).unwrap_or_default();
                let any_laid = done.left_at_earlier_scale.iter().any(|p| laid.contains(p));
                let (how_made, again) = if any_laid { ("traced over it or laid from its sheet", "trace or lay it again") } else { ("traced over it", "trace it again") };
                said.push_str(&format!("\n  NOT MOVED: {} {how_made} at the scale it had before: {}. Each is where that scale put it: check it against the sheet, or remove it and {again}", count(done.left_at_earlier_scale.len(), "thing was", "things were"), done.left_at_earlier_scale.join(", ")));
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
                let laid = plans::laid_from(&mut job, drawing).unwrap_or_default();
                let how_made = if left.iter().any(|p| laid.contains(p)) { "traced over it or laid from its sheet" } else { "traced over it" };
                said.push_str(&format!("\n  NOT MOVED: {} {how_made} where it lay before: {}. The sheet has moved and {} not; check {} against it", count(left.len(), "thing was", "things were"), left.join(", "), if left.len() == 1 { "it has" } else { "they have" }, if left.len() == 1 { "it" } else { "each" }));
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
            // A riser: one piece standing upright at one point, from its height up to --rises-to.
            if let Some(to) = flag("--rises-to") {
                let [at] = pts.as_slice() else { return Err("a riser stands at one point: --points \"6,5\"".into()) };
                let height = need(flag("--height"), "--height", "\"6 ft\" above the level's floor, where the riser starts")?;
                let made = ed::riser(&mut job, storey, kind_of(what)?, type_path, *at, &height, &to, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
                format!("1 upright piece: {made}")
            } else {
            let made = ed::runs(&mut job, storey, kind_of(what)?, type_path, &pts, &need(flag("--height"), "--height", "\"8 ft\" above the level's floor")?, flag("--system").as_deref(), ed::Entered { status, source: &source })?;
            format!("{} straight piece{}: {}", made.len(), if made.len() == 1 { "" } else { "s" }, made.join(", "))
            }
        }
        // One of an item (made from a maker's sheet): its size is the item's.
        ("place", [storey, item]) => {
            let (status, source) = how(&status, &source)?;
            let at = points(&need(flag("--at"), "--at", "\"14,7\" (feet)")?)?;
            let [at] = at.as_slice() else { return Err("--at is one point, x,y in feet".into()) };
            let (made, notes) = ed::place_item_said(&mut job, storey, item, *at, &flag("--offset").unwrap_or_else(|| "0 ft".to_string()), ed::Entered { status, source: &source })?;
            let size = job.find(&made).and_then(|e| job.world.get::<plumbline_world::Placed>(e)).map(|p| match p.size_ft() {
                Some([w, d, h]) => format!("{} wide, {} deep, {} high{}", units::feet_as_ft_in(w, 16), units::feet_as_ft_in(d, 16), units::feet_as_ft_in(h, 16), if p.size.status == Status::Known { "" } else { " (as its sheet says; not yet checked)" }),
                None => format!("no size yet, so it has no shape and is not tested for clashes: {}", p.size.source),
            });
            // The assembly its item is priced as, which it took (call 196).
            let priced = job.find(&made).and_then(|e| job.world.get::<plumbline_world::components::Condition>(e)).and_then(|c| c.0.value.clone()).map(|a| format!("; priced as {a:?}")).unwrap_or_default();
            let notes: String = notes.iter().map(|n| format!("; {n}")).collect();
            format!("{made}: one of {item}{}{priced}{notes}", size.map(|s| format!(", {s}")).unwrap_or_default())
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
            // Each box a cabinet rule made because the tool chest had not its code (call 215).
            for said in &made.made {
                out.push_str(&format!("\n  {said}"));
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
        // A thing turned (call 211 needed it at the command line; the window turns one with a key): so many degrees further,
        // counter-clockwise from east. What is joined to its connections is not moved: the reply is what the model says.
        ("turn", [element]) => {
            let (status, source) = how(&status, &source)?;
            let by: f64 = need(flag("--by"), "--by", "\"90\" (degrees counter-clockwise from east; 90 is a quarter turn)")?.trim().trim_end_matches("deg").trim().parse().map_err(|_| "--by is a number of degrees, like 90 or -45".to_string())?;
            let now = ed::turn(&mut job, element, by, ed::Entered { status, source: &source })?;
            // Said from 0 up to 360, as everywhere (call 213 point 7).
            format!("{element} is turned {} degrees from east now", view::num(ed::turned_said(now)))
        }
        ("move", [element]) => {
            let (status, source) = how(&status, &source)?;
            let (east, north) = ed::read_move(&need(flag("--by"), "--by", "\"4 ft west\", \"3 ft 6 in north and 2 ft east\", or \"3,2\" (feet east, feet north)")?)?;
            // A wall takes the ends that meet it with it unless moved alone (call 170).
            let moved = ed::move_wall(&mut job, element, east, north, ed::Entered { status, source: &source }, flag("--alone").is_some())?;
            let mut out = format!("{element} moved {}", ed::way_said(east, north));
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
            let priced = ed::prices_that_go(&mut job, element);
            let gone = ed::remove(&mut job, element)?;
            let mut said = match view::went_with_said(&before, &mut job) {
                Some(went) => format!("removed {}\n  {went}", gone.join(", ")),
                None => format!("removed {}", gone.join(", ")),
            };
            if let Some(against) = against {
                said.push_str(&format!("\n  {against}"));
            }
            if let Some(priced) = priced {
                said.push_str(&format!("\n  {priced}"));
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

pub(super) fn new_roof(args: &[String]) -> Result<(), String> {
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
pub(super) fn plan_set_verb(args: &[String]) -> Result<(), String> {
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
            Some(t) => Some(plumbline_world::plan_set::day(&t)?),
        },
    };
    // A plan set is a PDF, and its name ends as a PDF's does (call 179 D6).
    if !out.to_ascii_lowercase().ends_with(".pdf") {
        return Err(format!("{out} does not end in .pdf: a plan set is written as a PDF. Name it {out}.pdf, or any name that ends .pdf"));
    }
    // What is written out is never written over a job.
    if let (Ok(a), Ok(b)) = (std::fs::canonicalize(path), std::fs::canonicalize(out)) {
        if a == b {
            return Err(format!("{out} is the job itself: a plan set is not written over it. Give it a name of its own"));
        }
    }
    if std::fs::read_to_string(out).ok().is_some_and(|text| file::load(&text).is_ok()) {
        return Err(format!("{out} is a job: a plan set is not written over a job. Give it a name that is not in use"));
    }
    // A file that is there is written over only if it is a plan set or another PDF, and that is said.
    let over = match std::fs::read(out) {
        Ok(old) if old.starts_with(b"%PDF") => Some(()),
        Ok(_) => return Err(format!("{out} is there and is not a PDF: it is not written over. Give the plan set a name that is not in use")),
        Err(_) => None,
    };
    let (mut job, _) = read(path)?;
    let set = plumbline_world::plan_set::plan_set(&mut job, level, &opts)?;
    std::fs::write(out, set.pdf()).map_err(|e| format!("cannot write {out}: {e}"))?;
    if over.is_some() {
        say!("wrote over the plan set that was there");
    }
    say!("wrote {out}: {}, {}, {} paper, {}", count(set.pages.len(), "page", "pages"), set.pages.iter().map(|p| p.show.name()).collect::<Vec<_>>().join(", "), set.paper.name, plumbline_world::paper::size::scale_said(set.scale));
    for (show, why) in &set.not_written {
        say!("not written: {}: {why}", show.name());
    }
    let unlettered = set.unlettered();
    if !unlettered.is_empty() {
        let n: usize = unlettered.iter().map(|u| u.2).sum();
        say!("{} could not be lettered as written: {}", count(n, "letter", "letters"), unlettered.iter().map(|(c, p, _)| format!("{c} as {}", p.map_or("?".to_string(), |p| p.to_string()))).collect::<Vec<_>>().join(", "));
    }
    let faults = set.faults();
    if !faults.is_empty() {
        say!("{} not right on the pages as written:", count(faults.len(), "thing is", "things are"));
        for f in &faults {
            say!("  {f}");
        }
    }
    if list {
        say!();
        let _ = std::io::stdout().write_all(set.list().as_bytes());
    }
    Ok(())
}

/// `page <file> [--page n] [--lines [--longer-than pt]] [--png out.png]`: what a page of a
/// PDF holds, as this program's own reader sees it.
pub(super) fn page_verb(args: &[String]) -> Result<(), String> {
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
    if let Some(why) = plumbline_plans::dxf::closed_format(&bytes, file) {
        return Err(why);
    }
    let got = plumbline_plans::read(&bytes, page)?;
    if got.sort == plumbline_plans::Sort::Dxf {
        say!("{file}: {}, {} by {} {}, {}", got.sort.said(), view::num(got.size[0]), view::num(got.size[1]), page_unit(got.sort), pieces_said(got.sort, got.lines.len()));
    } else {
    say!("{file}, page {page} of {}: {}, {} by {} {} ({} by {} in), {}", got.pages, got.sort.said(), view::num(got.size[0]), view::num(got.size[1]), page_unit(got.sort), view::num(got.size[0] / 72.0), view::num(got.size[1] / 72.0), lines_said(got.lines.len()));
    }
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
