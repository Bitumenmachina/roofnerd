//! CAD drawings (DXF) at the command line (DECISIONS.md call 221): one brought in as a sheet to trace, its layers
//! switched on and off, and the plan of a level written out.
//!
//!   plumbline drawing <job> <level> "<name>" --file <a DXF> --status Assumed --source "<where from>"
//!   plumbline layers <job> <drawing>
//!   plumbline layer <job> <drawing> "<layer>" <on|off>
//!   plumbline export dxf <job> <out.dxf> --level <level>
//!
//! The reader and writer are `plumbline_plans::dxf`. This file is what a job needs of them: the sheet's place and scale,
//! which layers are off, and which of the job's things go on which layer.

use std::io::Write as _;

use plumbline_plans::dxf::{self, Cad, Writer};
use plumbline_world::building::planset::{level_named, level_plan};
use plumbline_world::building::{walls, Segment};
use plumbline_world::components::Node;
use plumbline_world::editing::Entered;
use plumbline_world::job::Job;
use plumbline_world::paper::cut;
use plumbline_world::skeleton::P2 as V;
use plumbline_world::value::Status;
use plumbline_world::{file, plans, units, view, Kind};

macro_rules! say {
    ($($arg:tt)*) => {{
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// A CAD drawing brought in under a level: its sheet at its own size, at true size where the file says its unit.
/// Returns what the person is told.
#[allow(clippy::too_many_arguments)]
pub fn bring_in(job: &mut Job, kept: &str, storey: &str, name: &str, bytes: &[u8], status: Status, source: &str) -> Result<String, String> {
    let cad = dxf::read(bytes)?;
    let made = plans::add(job, storey, name, plans::Read { file: kept, page: 1, bytes: bytes.len() as u64, checksum: &plumbline_plans::checksum(bytes), size: cad.size }, Entered { status, source })?;
    plans::set_layers_off(job, &made, &cad.off_by_the_file())?;
    let unit_said = match &cad.units {
        Some(u) => format!("drawn in {}", u.name),
        None => "its unit is not said".to_string(),
    };
    let mut out = format!(
        "{made}: drawing {name:?}: {kept} (CAD drawing (DXF)), {unit_said}, {} by {} {}, {}",
        view::num((cad.size[0] * 1.0e4).round() / 1.0e4),
        view::num((cad.size[1] * 1.0e4).round() / 1.0e4),
        cad.units.map_or("drawing units", |u| u.name),
        format!("holds {}", cad.held())
    );
    match cad.units {
        Some(u) => {
            // True size, from its own unit: the sheet's width, in inches, is the dimension.
            let (from, to, along) = if cad.size[0] >= cad.size[1] { ([0.0, 0.0], [cad.size[0], 0.0], cad.size[0]) } else { ([0.0, 0.0], [0.0, cad.size[1]], cad.size[1]) };
            let inches = along * u.inches;
            let how = format!("the file's own unit: {} (its $INSUNITS is {}) in {kept}", u.name, u.code);
            let done = plans::calibrate(job, &made, from, to, &format!("{inches} in"), Entered { status: Status::Assumed, source: &how })?;
            let width = cad.size[0] * u.inches / 12.0;
            let height = cad.size[1] * u.inches / 12.0;
            out.push_str(&format!("\n  at true size, nothing to calibrate: it covers {} by {}; its scale is Assumed, from the file's own unit ({}, {} to the foot)", units::feet_as_ft_in(width, 16), units::feet_as_ft_in(height, 16), u.name, view::num((1.0e6 / done.feet_per_unit).round() / 1.0e6)));
        }
        None => out.push_str(&format!("\n  no scale yet: {}. Give one dimension: plumbline calibrate <job> {made} --from \"x,y\" --to \"x,y\" --distance \"20 ft\"", cad.no_units)),
    }
    for l in cad.layers.iter().filter(|l| l.straight + l.curves + l.texts > 0) {
        out.push_str(&format!("\n  layer {}", cad.layer_said(l)));
    }
    if !cad.words(&[]).is_empty() {
        let w = cad.words(&[]);
        out.push_str(&format!("\n  words: {}", w.iter().take(8).map(|t| format!("{:?}", t.text)).collect::<Vec<_>>().join(", ") + &if w.len() > 8 { format!(" and {} more", w.len() - 8) } else { String::new() }));
    }
    for (b, n) in &cad.blocks {
        out.push_str(&format!("\n  block {b:?} is drawn where it is set, {}", count(*n, "time", "times")));
    }
    for lost in cad.lost() {
        out.push_str(&format!("\n  not read: {lost}"));
    }
    Ok(out)
}

/// The drawing's file, read from beside the job, and the drawing as the job has it.
fn load(path: &str, drawing: &str) -> Result<(Job, plans::Drawing, Cad), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let (mut job, _) = file::load(&text).map_err(|e| format!("{path}: {e}"))?;
    let e = job.find(drawing).ok_or_else(|| format!("no element {drawing}"))?;
    let d = job.world.get::<plans::Drawing>(e).cloned().ok_or_else(|| format!("{drawing} is not a drawing"))?;
    let file_name = d.file.value.clone().unwrap_or_default();
    let folder = std::path::Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new(".")).to_path_buf();
    let bytes = std::fs::read(folder.join(&file_name)).map_err(|e| format!("cannot read the sheet's file for {drawing} ({e}): it is kept beside the job"))?;
    if bytes.len() as u64 != d.bytes || plumbline_plans::checksum(&bytes) != d.checksum {
        return Err(format!("the file of {drawing} HAS CHANGED since the drawing was made from it: bring the sheet in again"));
    }
    if plumbline_plans::sort_of(&bytes) != Some(plumbline_plans::Sort::Dxf) {
        return Err(format!("{drawing} is not a CAD drawing (DXF): only a CAD drawing has layers"));
    }
    let cad = dxf::read(&bytes)?;
    Ok((job, d, cad))
}

/// `plumbline layers <job> <drawing>` and `plumbline layer <job> <drawing> "<layer>" <on|off>`.
pub fn layers(args: &[String]) -> Result<(), String> {
    let cmd = args[0].as_str();
    let (path, drawing) = match (args.get(1), args.get(2)) {
        (Some(p), Some(d)) => (p, d),
        _ => return Err(if cmd == "layers" { "layers takes the job's file and the drawing: plumbline layers <job> <drawing>".into() } else { "layer takes the job's file, the drawing, the layer and on or off: plumbline layer <job> <drawing> \"WALLS\" off".into() }),
    };
    let (mut job, d, cad) = load(path, drawing)?;
    if cmd == "layers" {
        if args.len() > 3 {
            return Err(format!("layers takes the job's file and the drawing; {:?} would be ignored", args[3]));
        }
        say!("{drawing}: {} in the CAD drawing {}", count(cad.layers.len(), "layer", "layers"), d.file.value.clone().unwrap_or_default());
        for l in &cad.layers {
            let off = d.layers_off.contains(&l.name);
            say!("  {} {}", if off { "off" } else { "on " }, cad.layer_said(l).replace(" (off in the file)", ""));
        }
        let page = cad.page(&d.layers_off);
        say!("  {} to snap to with these layers (curves count as their straight pieces)", count(page.lines.len(), "straight piece", "straight pieces"));
        return Ok(());
    }
    let [_, _, _, name, how] = args else {
        return Err("layer takes the job's file, the drawing, the layer and on or off: plumbline layer <job> <drawing> \"WALLS\" off".into());
    };
    let on = match how.as_str() {
        "on" => true,
        "off" => false,
        other => return Err(format!("{other:?} is not on or off: say whether the layer is to be on or off")),
    };
    let layer = match cad.layers.iter().find(|l| l.name == *name).or_else(|| {
        let same: Vec<&dxf::Layer> = cad.layers.iter().filter(|l| l.name.eq_ignore_ascii_case(name)).collect();
        if same.len() == 1 { Some(same[0]) } else { None }
    }) {
        Some(l) => l.name.clone(),
        None => return Err(format!("the drawing has no layer {name:?}; its layers are {}", cad.layers.iter().map(|l| l.name.as_str()).collect::<Vec<_>>().join(", "))),
    };
    let before = cad.page(&d.layers_off).lines.len();
    let off = plans::switch_layer(&mut job, drawing, &layer, on)?;
    let after = cad.page(&off).lines.len();
    std::fs::write(path, file::save(&mut job)).map_err(|e| format!("cannot write {path}: {e}"))?;
    say!("{drawing}: layer {layer} is {}; {} to snap to now ({} before)", if on { "on" } else { "off" }, count(after, "straight piece", "straight pieces"), before);
    if !off.is_empty() {
        say!("  off: {}", off.join(", "));
    }
    Ok(())
}

// ---- Out ----

/// The layers written, each with what it holds, where its name comes from, and its colour (AutoCAD's numbers).
/// A name carried by the published guidelines is said to be; one that is not found there is plain and says so.
const WALLS: &str = "A-WALL";
const DOORS: &str = "A-DOOR";
const WINDOWS: &str = "A-GLAZ";
const ROOMS: &str = "A-AREA-IDEN";
const FURNISHINGS: &str = "A-FURN";
const FIXTURES: &str = "P-FIXT";
const DUCTS: &str = "M-HVAC";
const POWER: &str = "E-POWR";
const EQUIPMENT: &str = "EQUIPMENT";
const PIPES: &str = "PIPES";
const OTHER_RUNS: &str = "RUNS";

/// What a layer name is, in words, for the reply: the AIA CAD Layer Guidelines' own, or a plain one.
fn layer_note(name: &str) -> &'static str {
    match name {
        WALLS => "walls (AIA CAD Layer Guidelines, A-WALL)",
        DOORS => "doors and their leaves (A-DOOR)",
        WINDOWS => "windows (A-GLAZ)",
        ROOMS => "room names and floor areas (A-AREA-IDEN, \"use for floor area calculations\")",
        FURNISHINGS => "furnishings and casework as outlines (A-FURN)",
        FIXTURES => "plumbing fixtures as outlines (P-FIXT)",
        DUCTS => "ducts, by their centerlines (M-HVAC, \"HVAC system, ductwork, equipment\")",
        POWER => "conduit and cable, by their centerlines (E-POWR, the nearest published name for them)",
        EQUIPMENT => "equipment as outlines (a plain name: no published name was found for it)",
        PIPES => "pipes, by their centerlines (a plain name: no published name was found for it)",
        _ => "runs of another kind (a plain name)",
    }
}

fn length_in(feet: f64) -> f64 {
    feet * 12.0
}

/// Where a segment of one of a wall's runs lies along that wall and across it.
fn along(w: &walls::PlanWall, p: V) -> (f64, f64) {
    let t = (w.end - w.start).unit();
    let d = p - w.start;
    (d.0 * t.0 + d.1 * t.1, d.0 * t.perp().0 + d.1 * t.perp().1)
}

/// The parts of the segment `a`-`b` that are not in a gap `s0`..`s1` of a wall's face lines.
fn outside_gap(w: &walls::PlanWall, a: V, b: V, gaps: &[[f64; 2]]) -> Vec<(V, V)> {
    let (pa, pb) = (along(w, a), along(w, b));
    let (lo, hi) = (w.int.min(w.ext) - 1.0e-4, w.int.max(w.ext) + 1.0e-4);
    let parallel = (pa.1 - pb.1).abs() < 1.0e-6 && pa.1 >= lo && pa.1 <= hi;
    if !parallel || gaps.is_empty() {
        return vec![(a, b)];
    }
    // Along the segment, from 0 at a to 1 at b: what the gaps leave.
    let len = pb.0 - pa.0;
    if len.abs() < 1.0e-12 {
        return vec![(a, b)];
    }
    let mut keep: Vec<[f64; 2]> = vec![[0.0, 1.0]];
    for g in gaps {
        let (g0, g1) = (((g[0] - pa.0) / len).min((g[1] - pa.0) / len), ((g[0] - pa.0) / len).max((g[1] - pa.0) / len));
        let mut next = Vec::new();
        for k in keep {
            if g1 <= k[0] || g0 >= k[1] {
                next.push(k);
                continue;
            }
            if g0 > k[0] {
                next.push([k[0], g0]);
            }
            if g1 < k[1] {
                next.push([g1, k[1]]);
            }
        }
        keep = next;
    }
    keep.into_iter().filter(|k| k[1] - k[0] > 1.0e-9).map(|k| (a + (b - a) * k[0], a + (b - a) * k[1])).collect()
}

/// `plumbline export dxf <job> <out.dxf> --level <level>`: the plan of a level as a CAD drawing, in inches at true size.
pub fn export(args: &[String]) -> Result<(), String> {
    let mut plain: Vec<&String> = Vec::new();
    let mut level: Option<&String> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--level" => {
                level = Some(args.get(i + 1).ok_or("--level wants the level: its name or its path")?);
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("export dxf does not take {f}")),
            _ => plain.push(&args[i]),
        }
        i += 1;
    }
    let (path, out) = match plain.as_slice() {
        [p, o] => (p.as_str(), o.as_str()),
        _ => return Err("export dxf takes the job's file and the DXF to write: plumbline export dxf <job> <out.dxf> --level <level>".into()),
    };
    let level = level.ok_or("export dxf wants the level whose plan is written: --level \"First Floor\"")?;
    if !out.to_ascii_lowercase().ends_with(".dxf") {
        return Err(format!("{out}: a CAD drawing is written as a .dxf file; name the file with that"));
    }
    if let (Ok(a), Ok(b)) = (std::fs::canonicalize(path), std::fs::canonicalize(out)) {
        if a == b {
            return Err(format!("{out} is the job itself: what is written out of a job is not written over it. Give the drawing a name of its own"));
        }
    }
    if std::fs::read_to_string(out).ok().is_some_and(|text| file::load(&text).is_ok()) {
        return Err(format!("{out} is a job: what is written out is not written over a job. Give the drawing a name that is not in use"));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let (mut job, _) = file::load(&text).map_err(|e| format!("{path}: {e}"))?;
    let storey = level_named(&mut job, level)?;
    let plan = level_plan(&mut job, &storey)?;
    let tol = plumbline_world::check::FIT_TOLERANCE_FT;
    let mut w = Writer::new();
    let mut said: Vec<(&str, usize)> = Vec::new();
    let mut put = |said: &mut Vec<(&'static str, usize)>, layer: &'static str, n: usize| {
        if n > 0 {
            match said.iter_mut().find(|(l, _)| *l == layer) {
                Some(e) => e.1 += n,
                None => said.push((layer, n)),
            }
        }
    };
    // Walls: what stands and what is new; what is marked to come down is not on the plan written.
    let keep: Vec<usize> = (0..plan.walls.len()).filter(|&i| plan.walls[i].place != 2).collect();
    let demolished = plan.walls.len() - keep.len();
    let page_walls: Vec<walls::PlanWall> = keep.iter().map(|&i| plan.walls[i].wall.clone()).collect();
    let rings: Vec<Vec<V>> = (0..page_walls.len()).map(|k| walls::outline(&page_walls, k, tol)).collect();
    let usable: Vec<usize> = (0..rings.len()).filter(|&k| rings[k].len() >= 3).collect();
    let group: Vec<Vec<V>> = usable.iter().map(|&k| rings[k].clone()).collect();
    let mut wall_lines = 0usize;
    let gaps_of = |k: usize| -> Vec<[f64; 2]> { plan.openings.iter().filter(|o| keep.get(k) == Some(&o.wall) && o.place != 2).map(|o| o.span).collect() };
    for (g, runs) in cut::cut_edges(&group, 0.0, 0.003).into_iter().enumerate() {
        let k = usable[g];
        let gaps = gaps_of(k);
        for run in runs {
            let (pts, closed) = cut::ink_cut(&run);
            let n = pts.len();
            for j in 0..if closed { n } else { n.saturating_sub(1) } {
                for (a, b) in outside_gap(&page_walls[k], pts[j], pts[(j + 1) % n], &gaps) {
                    // Where two walls' runs meet the runs carry a point twice: that is no line.
                    if a.dist(b) < 1.0e-6 {
                        continue;
                    }
                    w.line(WALLS, [length_in(a.0), length_in(a.1)], [length_in(b.0), length_in(b.1)]);
                    wall_lines += 1;
                }
            }
        }
    }
    put(&mut said, WALLS, wall_lines);
    // Doors and windows: the jambs across the wall, and the leaf or the glass along it.
    for o in plan.openings.iter().filter(|o| o.place != 2) {
        let Some(k) = keep.iter().position(|&i| i == o.wall) else { continue };
        let wl = &page_walls[k];
        let t = (wl.end - wl.start).unit();
        let at = |s: f64, off: f64| wl.start + t * s + t.perp() * off;
        let to_in = |p: V| [length_in(p.0), length_in(p.1)];
        let layer: &'static str = if o.door { DOORS } else { WINDOWS };
        let [s0, s1] = o.span;
        for s in [s0, s1] {
            w.line(layer, to_in(at(s, wl.int)), to_in(at(s, wl.ext)));
        }
        let mid = (wl.ext + wl.int) / 2.0;
        let offs: Vec<f64> = if o.door { vec![mid] } else { vec![wl.int, mid, wl.ext] };
        for off in &offs {
            w.line(layer, to_in(at(s0, *off)), to_in(at(s1, *off)));
        }
        let n = 2 + offs.len();
        put(&mut said, layer, n);
    }
    // Rooms: the name, and the area as the model has it.
    let mut rooms_written = 0;
    let mut rooms_without = Vec::new();
    for r in &plan.rooms_left {
        let at = [length_in(r.point[0]), length_in(r.point[1])];
        w.text(ROOMS, at, 6.0, 0.0, &r.name);
        match r.area {
            Some(a) => {
                let sure = if r.status == Status::Known { "" } else { " VIF" };
                w.text(ROOMS, [at[0], at[1] - 9.0], 6.0, 0.0, &format!("{} SF{sure}", view::num(plumbline_world::value::round4(a))));
            }
            None => rooms_without.push(format!("{}: {}", r.name, r.why)),
        }
        rooms_written += 1;
    }
    put(&mut said, ROOMS, rooms_written);
    // Things set in place, as their outlines and their names.
    for t in &plan.things {
        let layer = match job.kind(&t.path) {
            Some(Kind::Fixture) => FIXTURES,
            Some(Kind::Equipment) => EQUIPMENT,
            _ => FURNISHINGS,
        };
        for j in 0..4 {
            let (a, b) = (t.ring[j], t.ring[(j + 1) % 4]);
            w.line(layer, [length_in(a.0), length_in(a.1)], [length_in(b.0), length_in(b.1)]);
        }
        let c = [(t.ring[0].0 + t.ring[2].0) / 2.0, (t.ring[0].1 + t.ring[2].1) / 2.0];
        w.text(layer, [length_in(c[0]) - 3.0 * t.name.chars().count() as f64 / 2.0, length_in(c[1])], 3.0, 0.0, &t.name);
        put(&mut said, if layer == FIXTURES { FIXTURES } else if layer == EQUIPMENT { EQUIPMENT } else { FURNISHINGS }, 5);
    }
    // Runs, by their centerlines, on this level.
    let mut runs: Vec<(Kind, [[f64; 2]; 2])> = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Segment)>();
        let mut all: Vec<(String, Kind, [[f64; 2]; 2])> = q.iter(&job.world).filter(|(n, k, _)| n.parent.as_deref() == Some(storey.as_str()) && matches!(**k, Kind::Duct | Kind::Pipe | Kind::Conduit | Kind::Cable)).filter_map(|(n, k, s)| s.line_ft().map(|l| (n.path.clone(), *k, l))).collect();
        all.sort_by(|a, b| a.0.cmp(&b.0));
        runs.extend(all.into_iter().map(|(_, k, l)| (k, l)));
    }
    let mut risers = 0;
    for (kind, [a, b]) in &runs {
        if (a[0] - b[0]).hypot(a[1] - b[1]) < 1.0e-9 {
            risers += 1;
            continue;
        }
        let layer = match kind {
            Kind::Duct => DUCTS,
            Kind::Pipe => PIPES,
            Kind::Conduit | Kind::Cable => POWER,
            _ => OTHER_RUNS,
        };
        w.line(layer, [length_in(a[0]), length_in(a[1])], [length_in(b[0]), length_in(b[1])]);
        put(&mut said, layer, 1);
    }
    let text = w.finish();
    std::fs::write(out, &text).map_err(|e| format!("cannot write {out}: {e}"))?;
    say!("wrote {out}: the plan of {} as a CAD drawing (DXF), in inches at true size", plan.level);
    for (layer, n) in &said {
        let what = match *layer {
            ROOMS => format!("{} written", count(*n, "room", "rooms")),
            _ => format!("{} drawn", count(*n, "line", "lines")),
        };
        say!("  layer {layer}: {what}: {}", layer_note(layer));
    }
    if demolished > 0 {
        say!("  not written: {} marked to demolish (the plan written is what is built and what is new)", count(demolished, "wall", "walls"));
    }
    if risers > 0 {
        say!("  not written: {} (a rise, with no length on the plan)", count(risers, "riser", "risers"));
    }
    for (what, why) in plan.walls_left_off.iter().chain(&plan.openings_left_off).chain(&plan.things_left_off) {
        say!("  not written: {what}: {why}");
    }
    for r in rooms_without {
        say!("  a room written without its area: {r}");
    }
    say!("  not in this drawing: floors, roofs, footings, beams, posts and the cost of any of it; the job file and the IFC model carry them");
    Ok(())
}
