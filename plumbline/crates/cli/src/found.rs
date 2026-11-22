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

/// What is found, in a builder's words.
pub fn said(found: &Found, called: &str, points: usize, taken: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "scan {called:?}: {} points, {taken}", plumbline_world::view::figure(points as f64, 0));
    if let (Some(f), Some(c)) = (found.floor, found.ceiling) {
        let _ = writeln!(out, "  floor to ceiling reads about {}", inch(c - f));
    }
    for why in &found.left_out {
        let _ = writeln!(out, "  left out: {why}");
    }
    if found.no_ceiling {
        let _ = writeln!(out, "  nothing found: without a ceiling a wall cannot be told from a tall cabinet");
        return out;
    }
    let one_side = found.walls.iter().filter(|w| w.thickness.is_none()).count();
    let _ = writeln!(out, "  walls found: {}, {} of them seen from one side only (everything found is Assumed: it is a guess from points, to be checked)", found.walls.len(), one_side);
    for (i, w) in found.walls.iter().enumerate() {
        let how = match w.thickness {
            Some(t) => format!("{} thick, seen from both sides", thick(t)),
            None => "seen from one side (its thickness is not in the scan)".to_string(),
        };
        let _ = writeln!(out, "    wall {}: from {} to {}, {} long, {how}", i + 1, place(w.from), place(w.to), inch(w.length()));
        let end = end_of(w);
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
    let usage = "find-walls takes the job's file, a scan (a PLY point cloud) and --level <level>: plumbline find-walls <file> <scan.ply> --level <level> [--units m|ft] [--up z|y] [--lay [--outside \"<wall type>\"]]";
    let (mut plain, mut flags): (Vec<&str>, Vec<(&str, Option<&str>)>) = (Vec::new(), Vec::new());
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--lay" => flags.push(("--lay", None)),
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
    if flag("--outside").is_some() && !lay {
        return Err("--outside says what the walls seen from one side are laid as: it goes with --lay".into());
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
    match job.kind(&level) {
        Some(Kind::Storey) => {}
        Some(other) => return Err(format!("{level} is a {other:?}, not a level")),
        None => return Err(format!("no element {level}")),
    }
    let bytes = std::fs::read(scan_path).map_err(|e| format!("cannot read {scan_path}: {e}"))?;
    let called = std::path::Path::new(scan_path).file_stem().and_then(|s| s.to_str()).unwrap_or("scan").to_string();
    let mut scan = plumbline_plans::scan::read_ply(&bytes, units_of, up).map_err(|why| format!("{scan_path}: {why}"))?;
    scan.level_by_floor();
    let taken = format!("taken as {} with {} up", if units_of == Units::Metres { "meters" } else { "feet" }, if up == Up::Z { "z" } else { "y" });
    let found = found::find(&scan);
    let mut out = said(&found, &called, scan.points.len(), &taken);
    if !lay {
        if !found.walls.is_empty() {
            let _ = writeln!(out, "  nothing is laid: say --lay to lay these walls, doors and windows on {level}");
            let one_side = found.walls.iter().filter(|w| w.thickness.is_none()).count();
            if one_side > 0 {
                let _ = writeln!(out, "  the {} seen from one side are laid only as a wall type you name for them (--outside \"<wall type>\"): the scan does not say how thick they are", plural(one_side, "wall", "walls"));
            }
        }
        let _ = std::io::stdout().write_all(out.as_bytes());
        return Ok(());
    }
    // Laying.
    let outside = match flag("--outside") {
        Some(t) => Some((t.to_string(), plans::wall_type_feet(&mut job, t)?)),
        None => None,
    };
    let (laid, not_laid) = found::lay(&found, outside.as_ref().map(|o| o.1));
    let (Some(floor), Some(ceiling)) = (found.floor, found.ceiling) else {
        let _ = writeln!(out, "  nothing is laid");
        let _ = std::io::stdout().write_all(out.as_bytes());
        return Ok(());
    };
    let walls: Vec<plans::FoundWall> = laid
        .iter()
        .map(|l| plans::FoundWall {
            from: l.from,
            to: l.to,
            thickness: l.thickness,
            measured: l.measured,
            openings: l.openings.iter().map(|o| plans::FoundOpening { window: o.window, from: o.from, to: o.to, sill: o.sill, head: o.head }).collect(),
        })
        .collect();
    if !walls.is_empty() {
        let made = plans::lay_found(&mut job, &level, &called, ceiling - floor, outside.as_ref().map(|o| o.0.as_str()), &walls)?;
        std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
        let _ = writeln!(out, "  laid on {level}: {}, {}, {}, each Assumed, source \"found in scan {called}\"; none is Known", plural(made.walls.len(), "wall", "walls"), plural(made.doors.len(), "door", "doors"), plural(made.windows.len(), "window", "windows"));
        for (l, path) in laid.iter().zip(&made.walls) {
            let _ = writeln!(out, "    {path}: found wall {}, {} thick{}", l.of + 1, thick(l.thickness), if l.measured { " (from the scan)" } else { " (the type named for it)" });
        }
        for (path, name) in &made.types {
            let _ = writeln!(out, "    made for them, Assumed, from the scan: {path}, {name:?}");
        }
        for (wall, o) in laid.iter().flat_map(|l| l.openings.iter().map(move |o| (l.of, o))) {
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
    if !not_laid.is_empty() {
        let _ = writeln!(out, "  not laid: {} seen from one side only, since the scan does not give how thick they are; say which wall type they are with --outside \"<wall type>\" and lay again", plural(not_laid.len(), "wall", "walls"));
    }
    let _ = std::io::stdout().write_all(out.as_bytes());
    Ok(())
}
