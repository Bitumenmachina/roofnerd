//! `plumbline find-walls`: the walls, doors, windows and standing things a scan shows,
//! said in full, and laid on the person's word (DECISIONS.md call 173).
//!
//!   plumbline find-walls <file> <scan.ply> --level <level> [--units m|ft] [--up z|y]
//!   plumbline find-walls <file> <scan.ply> --level <level> --lay [--outside "<wall type>"]
//!
//! Without --lay nothing is written: the job is not touched. With it, what is found is
//! laid with the wall, door and window the model already has, all Assumed, each with the
//! source "found in scan <name>".

use std::fmt::Write as _;
use std::io::Write as _;

use plumbline_plans::found::{self, Found};
use plumbline_plans::scan::{Units, Up};
use plumbline_world::units;
use plumbline_world::{file, plans, Kind};

/// A length to the nearest inch, as a tape reads it.
fn inch(feet: f64) -> String {
    units::feet_as_ft_in((feet * 12.0).round() / 12.0, 16)
}

/// A thickness to the nearest quarter inch.
fn thick(feet: f64) -> String {
    units::feet_as_ft_in((feet * 12.0 * 4.0).round() / 48.0, 16)
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Where along a wall its first end is, in words: the way the wall runs says which end.
fn end_of(w: &found::Wall) -> &'static str {
    let (dx, dy) = (w.to[0] - w.from[0], w.to[1] - w.from[1]);
    if dx.abs() >= dy.abs() {
        if dx >= 0.0 { "west" } else { "east" }
    } else if dy >= 0.0 {
        "south"
    } else {
        "north"
    }
}

fn place(p: [f64; 2]) -> String {
    format!("{} east, {} north", inch(p[0]), inch(p[1]))
}

/// The one thing of this kind a name says (a level by its name, a wall type by its name), by its path.
fn by_name(job: &mut plumbline_world::job::Job, name: &str, kind: Kind) -> Option<String> {
    let mut q = job.world.query::<(&plumbline_world::Node, &Kind)>();
    let hits: Vec<String> = q.iter(&job.world).filter(|(n, k)| **k == kind && n.name.eq_ignore_ascii_case(name.trim())).map(|(n, _)| n.path.clone()).collect();
    (hits.len() == 1).then(|| hits[0].clone())
}

/// What a thing is called, by its path.
fn name_of(job: &mut plumbline_world::job::Job, path: &str) -> String {
    job.find(path).and_then(|e| job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone())).unwrap_or_else(|| path.to_string())
}

/// What the level says its height is, how sure that is and where it came from.
fn level_height(job: &mut plumbline_world::job::Job, level: &str) -> Option<(f64, String, String)> {
    let e = job.find(level)?;
    let l = job.world.get::<plumbline_world::Level>(e)?;
    let feet = l.height.value.as_ref()?.feet(&l.height.unit).ok()?;
    Some((feet, format!("{:?}", l.height.status), l.height.source.clone()))
}

/// Whether the job already holds a sheet laid from this scan under the level (its place is where the scan's feet put it).
fn has_sheet(job: &mut plumbline_world::job::Job, level: &str, called: &str) -> bool {
    let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::plans::Drawing)>();
    q.iter(&job.world).any(|(n, d)| n.parent.as_deref() == Some(level) && d.file.source.starts_with(&format!("scan {called}.ply")))
}

/// A scan farther than this from zero (a house is not over 150 ft: call 179, A7) is not laid there.
const FAR_FROM_ZERO: f64 = 150.0;
/// The level and the scan disagree about how high it is where they differ by more than this (an inch).
const LEVEL_AGREES: f64 = 1.0 / 12.0;

/// What is found, in a builder's words.
pub fn said(found: &Found, called: &str, points: usize, taken: &str, level_note: Option<&str>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "scan {called:?}: {} points, {taken}", plumbline_world::view::figure(points as f64, 0));
    if let Some(why) = &found.not_likely {
        let _ = writeln!(out, "  {why}");
        let _ = writeln!(out, "  nothing is laid from it unless you say --lay --anyway");
    }
    if let (Some(f), Some(c)) = (found.floor, found.ceiling) {
        let _ = writeln!(out, "  floor to ceiling reads about {}", inch(c - f));
    }
    if let Some(note) = level_note {
        let _ = writeln!(out, "  {note}");
    }
    if let Some(d) = found.density {
        let _ = writeln!(out, "  the scan holds about {} to the square foot of floor", plumbline_world::view::figure(d, 0).to_string() + " points");
    }
    if let Some(note) = &found.thin_note {
        let _ = writeln!(out, "  {note}");
    }
    if let Some(why) = &found.too_thin {
        let _ = writeln!(out, "  nothing found: {why}");
        return out;
    }
    if let Some(note) = &found.ceiling_note {
        let _ = writeln!(out, "  {note}");
    }
    for why in &found.left_out {
        let _ = writeln!(out, "  left out: {why}");
    }
    if found.no_ceiling {
        let _ = writeln!(out, "  nothing found: without a ceiling a wall cannot be told from a tall cabinet");
        return out;
    }
    let one_side = found.walls.iter().filter(|w| w.thickness.is_none()).count();
    let doubted = found.walls.iter().filter(|w| w.doubt.is_some()).count();
    let in_doubt = if doubted > 0 { format!(", {doubted} of them in doubt (a wall or something tall standing at a wall)") } else { String::new() };
    if found.walls.is_empty() {
        let _ = writeln!(out, "  no wall found: no upright face of 2 ft or more runs from the floor to the ceiling");
    }
    let _ = writeln!(out, "  walls found: {}, {} of them seen from one side only{in_doubt} (everything found is Assumed: it is a guess from points, to be checked)", found.walls.len(), one_side);
    for (i, w) in found.walls.iter().enumerate() {
        let how = match w.thickness {
            Some(t) => format!("{} thick, seen from both sides", thick(t)),
            None => "seen from one side (its thickness is not in the scan)".to_string(),
        };
        let high = match (found.ceiling_note.is_some(), w.height) {
            (true, Some(h)) => format!(", {} high", inch(h)),
            _ => String::new(),
        };
        let _ = writeln!(out, "    wall {}: from {} to {}, {} long, {how}{high}", i + 1, place(w.from), place(w.to), inch(w.length()));
        let end = end_of(w);
        if let Some(why) = &w.doubt {
            let _ = writeln!(out, "      {why}: it is not laid unless you say it is a wall (--take {})", i + 1);
        }
        for o in &w.openings {
            if o.window {
                let _ = writeln!(out, "      window: {} to {} from its {end} end, {} wide, sill {}, head {}", inch(o.from), inch(o.to), inch(o.to - o.from), inch(o.sill), inch(o.head));
            } else {
                let _ = writeln!(out, "      doorway: {} to {} from its {end} end, {} wide, head {}", inch(o.from), inch(o.to), inch(o.to - o.from), inch(o.head));
            }
        }
        for h in &w.hidden {
            let _ = writeln!(out, "      not seen for {} ({} to {} from its {end} end) behind what stands at it: taken as wall, to be checked", inch(h.to - h.from), inch(h.from), inch(h.to));
        }
    }
    if found.walls.iter().any(|w| w.doubt.is_none()) {
        if found.open_ends.is_empty() {
            let _ = writeln!(out, "  the walls found close: every end meets another wall");
        } else {
            let _ = writeln!(out, "  the walls found do not close: {} meet{} nothing, so the room is open there as far as the scan shows (a wall may be hidden, or missing):", plural(found.open_ends.len(), "end", "ends"), if found.open_ends.len() == 1 { "s" } else { "" });
            for (i, p) in &found.open_ends {
                let w = &found.walls[*i];
                let (dx, dy) = (w.to[0] - w.from[0], w.to[1] - w.from[1]);
                let first = (p[0] - w.from[0]).hypot(p[1] - w.from[1]) <= (p[0] - w.to[0]).hypot(p[1] - w.to[1]);
                let word = match (dx.abs() >= dy.abs(), first, dx >= 0.0, dy >= 0.0) {
                    (true, true, true, _) | (true, false, false, _) => "west",
                    (true, _, _, _) => "east",
                    (false, true, _, true) | (false, false, _, false) => "south",
                    _ => "north",
                };
                let _ = writeln!(out, "    wall {}, its {word} end, at {}", i + 1, place(*p));
            }
        }
    }
    if found.walls.iter().any(|w| w.openings.iter().any(|o| !o.window)) {
        let _ = writeln!(out, "  a doorway is a clear opening from the floor: the scan cannot tell a door from an open doorway, and each is laid as a door");
    }
    if let Some(main) = found.main {
        let _ = writeln!(out, "  squared: {} of {} wall faces were within 1 degree of the scan's two main directions ({:.1} and {:.1} degrees from east) and were taken as square to them; the house is laid as the scan lies, not turned to the page", found.squared.0, found.squared.1, main, main + 90.0);
    }
    if found.things.is_empty() {
        let _ = writeln!(out, "  nothing stood short of the ceiling");
    } else {
        let _ = writeln!(out, "  stood short of the ceiling, and not walls: {} (cabinets, furniture); the tallest is {}", plural(found.things.len(), "thing", "things"), inch(found.tallest().unwrap_or(0.0)));
    }
    out
}

pub fn run(args: &[String]) -> Result<(), String> {
    let usage = "find-walls takes the job's file, a scan (a PLY point cloud) and --level <level>: plumbline find-walls <file> <scan.ply> --level <level> [--units m|ft] [--up z|y] [--lay [--outside \"<wall type>\"] [--take <n>] [--anyway]]";
    let (mut plain, mut flags): (Vec<&str>, Vec<(&str, Option<&str>)>) = (Vec::new(), Vec::new());
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--lay" => flags.push(("--lay", None)),
            "--take" => {
                let v = args.get(i + 1).ok_or("--take needs the number of a wall found after it")?;
                flags.push(("--take", Some(v.as_str())));
                i += 1;
            }
            "--anyway" => flags.push(("--anyway", None)),
            f @ ("--level" | "--units" | "--up" | "--outside") => {
                let v = args.get(i + 1).ok_or_else(|| format!("{f} needs a value after it"))?;
                if flags.iter().any(|(known, _)| *known == f) {
                    return Err(format!("{f} is given twice"));
                }
                flags.push((f, Some(v.as_str())));
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("find-walls does not take {f}\n{usage}")),
            a => plain.push(a),
        }
        i += 1;
    }
    let flag = |name: &str| flags.iter().find(|(f, _)| *f == name).and_then(|(_, v)| *v);
    let [path, scan_path] = plain.as_slice() else { return Err(usage.into()) };
    let lay = flags.iter().any(|(f, _)| *f == "--lay");
    let anyway = flags.iter().any(|(f, _)| *f == "--anyway");
    if anyway && !lay {
        return Err("--anyway says to lay what is found though it is not likely a house: it goes with --lay".into());
    }
    if flag("--outside").is_some() && !lay {
        return Err("--outside says what the walls seen from one side are laid as: it goes with --lay".into());
    }
    if flag("--take").is_some() && !lay {
        return Err("--take says a wall found is a wall, to be laid with the rest: it goes with --lay".into());
    }
    let level = flag("--level").ok_or("say which level the scan is of: --level <level>")?.to_string();
    let units_of = match flag("--units") {
        None | Some("m") => Units::Metres,
        Some("ft") => Units::Feet,
        Some(other) => return Err(format!("--units {other:?}: a scan is in m or ft")),
    };
    let up = match flag("--up") {
        None | Some("z") => Up::Z,
        Some("y") => Up::Y,
        Some(other) => return Err(format!("--up {other:?}: up is z or y")),
    };
    let (mut job, _) = super::read(path)?;
    // A level is given by its name as well as by its mark.
    let level = if job.kind(&level).is_none() { by_name(&mut job, &level, Kind::Storey).unwrap_or(level) } else { level };
    match job.kind(&level) {
        Some(Kind::Storey) => {}
        Some(other) => return Err(format!("{level} is a {other:?}, not a level")),
        None => return Err(format!("no element {level}")),
    }
    let bytes = std::fs::read(scan_path).map_err(|e| format!("cannot read {scan_path}: {e}"))?;
    let called = std::path::Path::new(scan_path).file_stem().and_then(|s| s.to_str()).unwrap_or("scan").to_string();
    let mut scan = plumbline_plans::scan::read_ply(&bytes, units_of, up).map_err(|why| format!("{scan_path}: {why}"))?;
    scan.level_by_floor();
    let taken = format!("read as {}{}", if units_of == Units::Metres { "meters" } else { "feet" }, if up == Up::Z { "" } else { ", with the file's other direction taken as up (--up y)" });
    let found = found::find(&scan);
    // Where the level says it is another height than the scan reads, that is said (call 179, A8).
    let level_name = name_of(&mut job, &level);
    let level_note = level_height(&mut job, &level).and_then(|(h, status, source)| {
        let (Some(f), Some(c)) = (found.floor, found.ceiling) else { return None };
        let reads = c - f;
        if found.ceiling_note.is_some() || (reads - h).abs() <= LEVEL_AGREES {
            return None;
        }
        Some(format!("the level {level_name} is {} high ({status}, {source}) and the scan reads {} from floor to ceiling, {} {}: the walls are laid as high as the scan reads, Assumed, from the scan, and the level's height is not changed; measure it to tell which is right", inch(h), inch(reads), inch((reads - h).abs()), if reads > h { "more" } else { "less" }))
    });
    let mut out = said(&found, &called, scan.points.len(), &taken, level_note.as_deref());
    if !lay {
        if !found.walls.is_empty() {
            let _ = writeln!(out, "  nothing is laid: say --lay to lay these walls, doors and windows on {level}");
            let one_side = found.walls.iter().filter(|w| w.thickness.is_none() && w.doubt.is_none()).count();
            if one_side > 0 {
                let _ = writeln!(out, "  the {} seen from one side {} laid only as a wall type you name for them (--outside \"<wall type>\"): the scan does not say how thick {} {}", plural(one_side, "wall", "walls"), if one_side == 1 { "is" } else { "are" }, if one_side == 1 { "it" } else { "they" }, if one_side == 1 { "is" } else { "are" });
            }
        }
        let _ = std::io::stdout().write_all(out.as_bytes());
        return Ok(());
    }
    // Laying.
    if let (Some(why), false) = (&found.not_likely, anyway) {
        let _ = std::io::stdout().write_all(out.as_bytes());
        return Err(format!("nothing is laid: {why}. If it is the house, say --lay --anyway"));
    }
    let outside = match flag("--outside") {
        Some(t) => {
            let t = if job.find(t).is_none() { by_name(&mut job, t, Kind::Type).unwrap_or_else(|| t.to_string()) } else { t.to_string() };
            let feet = plans::wall_type_feet(&mut job, &t)?;
            Some((t, feet))
        }
        None => None,
    };
    let mut take: Vec<usize> = Vec::new();
    for t in flags.iter().filter(|(f, _)| *f == "--take").filter_map(|(_, v)| *v).flat_map(|v| v.split(',')) {
        let n: usize = t.trim().parse().map_err(|_| format!("--take {t:?}: a wall found is named by its number, as \"wall 4\" says it"))?;
        if n == 0 || n > found.walls.len() {
            return Err(format!("--take {n}: {} found, numbered 1 to {}", plural(found.walls.len(), "wall was", "walls were"), found.walls.len()));
        }
        take.push(n - 1);
    }
    let (laid, not_laid) = found::lay_with(&found, outside.as_ref().map(|o| o.1), &take);
    let (Some(floor), Some(ceiling)) = (found.floor, found.ceiling) else {
        let _ = writeln!(out, "  nothing is laid");
        let _ = std::io::stdout().write_all(out.as_bytes());
        return Ok(());
    };
    // Where it is laid: where its sheet lies; with none, a scan far from zero at its own southwest corner (call 179, A9).
    let mut laid = laid;
    let mut where_laid: Option<String> = None;
    let corner = laid.iter().flat_map(|l| [found.walls[l.of].from, found.walls[l.of].to]).fold(None, |a: Option<[f64; 2]>, p| Some(a.map_or(p, |q| [q[0].min(p[0]), q[1].min(p[1])])));
    if let Some(c) = corner.filter(|c| c[0].abs() > FAR_FROM_ZERO || c[1].abs() > FAR_FROM_ZERO) {
        let (east, north) = (inch(c[0]), inch(c[1]));
        if has_sheet(&mut job, &level, &called) {
            where_laid = Some(format!("the scan lies {east} east and {north} north of zero; it is laid where its sheet on {level_name} lies"));
        } else {
            for l in laid.iter_mut() {
                l.from = [l.from[0] - c[0], l.from[1] - c[1]];
                l.to = [l.to[0] - c[0], l.to[1] - c[1]];
            }
            where_laid = Some(format!("the scan lies {east} east and {north} north of zero and has no sheet on {level_name} to lay it by: it is laid with its own southwest corner at zero"));
        }
    }
    let walls: Vec<plans::FoundWall> = laid
        .iter()
        .map(|l| plans::FoundWall {
            from: l.from,
            to: l.to,
            thickness: l.thickness,
            measured: l.measured,
            height: l.height,
            openings: l.openings.iter().map(|o| plans::FoundOpening { window: o.window, from: o.from, to: o.to, sill: o.sill, head: o.head }).collect(),
        })
        .collect();
    if !walls.is_empty() {
        let made = plans::lay_found(&mut job, &level, &called, ceiling - floor, outside.as_ref().map(|o| o.0.as_str()), &walls)?;
        std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
        if let Some(w) = &where_laid {
            let _ = writeln!(out, "  {w}");
        }
        let _ = writeln!(out, "  laid on {level}: {}, {}, {}, each Assumed, source \"found in scan {called}\"; none is Known", plural(made.walls.len(), "wall", "walls"), plural(made.doors.len(), "door", "doors"), plural(made.windows.len(), "window", "windows"));
        // A wall the model would not take (laid along another) is named as the find named it, and the rest are laid.
        let name_of = |p: &str| made.walls.iter().zip(&made.of).find(|(w, _)| w.as_str() == p).map(|(_, i)| format!("wall {} of what was found", laid[*i].of + 1));
        for (i, why) in &made.skipped {
            let mut said = why.replace("the new wall: ", "").replace("the new wall", "this wall");
            for (w, _) in made.walls.iter().zip(&made.of) {
                if let Some(n) = name_of(w) {
                    said = said.replace(w.as_str(), &n);
                }
            }
            let _ = writeln!(out, "  not laid: wall {} of what was found: {said}", laid[*i].of + 1);
        }
        for (path, i) in made.walls.iter().zip(&made.of) {
            let l = &laid[*i];
            let _ = writeln!(out, "    {path}: found wall {}, {} thick{}", l.of + 1, thick(l.thickness), if l.measured { " (from the scan)" } else { " (the type named for it)" });
        }
        for (path, name) in &made.types {
            let _ = path;
            let _ = writeln!(out, "    made for them, Assumed, from the scan: the type {name:?}");
        }
        for (wall, o) in laid.iter().enumerate().filter(|(i, _)| made.of.contains(i)).flat_map(|(_, l)| l.openings.iter().map(move |o| (l.of, o))) {
            let _ = writeln!(out, "    {} on found wall {}: {} wide, {}", if o.window { "window" } else { "door" }, wall + 1, inch(o.to - o.from), if o.window { format!("sill {}, head {}", inch(o.sill), inch(o.head)) } else { format!("head {}", inch(o.head)) });
        }
        for w in &found.walls {
            for h in &w.hidden {
                let _ = writeln!(out, "    not seen for {} behind what stands at it: laid as wall, not as an opening, to be checked", inch(h.to - h.from));
            }
        }
    } else {
        let _ = writeln!(out, "  nothing is laid");
    }
    let (in_doubt, one_sided): (Vec<usize>, Vec<usize>) = not_laid.iter().partition(|i| found.walls[**i].doubt.is_some());
    for i in &in_doubt {
        let _ = writeln!(out, "  not laid: wall {} of what was found, {}: say --take {} with --lay if it is a wall", i + 1, found.walls[*i].doubt.as_deref().unwrap_or(""), i + 1);
    }
    if !one_sided.is_empty() {
        let _ = writeln!(out, "  not laid: {} seen from one side only, since the scan does not give how thick they are; say which wall type they are with --outside \"<wall type>\" and lay again", plural(one_sided.len(), "wall", "walls"));
    }
    let _ = std::io::stdout().write_all(out.as_bytes());
    Ok(())
}
