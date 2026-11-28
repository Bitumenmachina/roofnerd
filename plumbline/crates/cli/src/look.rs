use super::*;

/// What is in the same space as what, what passes through a wall or a floor, and, when a
/// clearance is asked for, what is closer than it. Lengths are in inches, to two places.
/// The verify-in-field list as text: each question, where it is, what to measure or
/// check, and the dollars riding on it, largest first.
pub(super) fn vif(job: &mut plumbline_world::Job) -> String {
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
                if !to_do.is_empty() {
                    let _ = writeln!(out, "      {}", to_do.join("; "));
                }
                // What stands on a wall the work opens, and its headers (call 176).
                for said in &q.also {
                    let _ = writeln!(out, "      {said}");
                }
            }
        }
    }
    out
}

pub(super) fn clashes(job: &mut plumbline_world::Job, clearance: Option<f64>) -> Result<String, String> {
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
    let hard: Vec<&clash::Clash> = found.found.iter().filter(|c| c.how == ClashKind::Hard && !c.through && !c.set_against).collect();
    let against: Vec<&clash::Clash> = found.found.iter().filter(|c| c.how == ClashKind::Hard && !c.through && c.set_against).collect();
    // A hole is counted once (call 175): the first piece of a run that passes through a wall or
    // a floor stands for the hole.
    let through: Vec<&clash::Clash> = {
        let mut seen: Vec<usize> = Vec::new();
        found.found.iter().filter(|c| c.how == ClashKind::Hard && c.through && !c.laid_in).filter(|c| match c.hole {
            Some(h) if seen.contains(&h) => false,
            Some(h) => {
                seen.push(h);
                true
            }
            None => true,
        }).collect()
    };
    let close: Vec<&clash::Clash> = found.found.iter().filter(|c| c.how == ClashKind::Clearance).collect();
    let _ = writeln!(out, "In the same space: {}{}", hard.len(), assumed(&hard));
    for c in &hard {
        let by = c.depth_listed();
        // Where a run ends inside a thing that says where it connects: which connection is open and how far away (call 211).
        let connection = clash::connection_words(job, c).map(|words| format!("; {words}")).unwrap_or_default();
        let _ = writeln!(out, "  {}: {by}, at {}{connection}", pairs(c), place(c));
    }
    // A thing standing against a wall, hung on it or set in it, and a run at its joint, by no
    // more than two inches: not two things in one place, and counted apart (call 175).
    let _ = writeln!(out, "Set against (stands against it, or is joined to it, by no more than {}: not a clash): {}{}", inches(clash::SET_AGAINST_FT), against.len(), assumed(&against));
    for c in &against {
        let _ = writeln!(out, "  {}: set {} into it, at {}", pairs(c), inches(c.by.unwrap_or(0.0)), place(c));
    }
    let _ = writeln!(out, "Through a wall or a floor (a hole to cut): {}{}", through.len(), assumed(&through));
    for c in &through {
        match c.hole.and_then(|h| found.holes.get(h)) {
            Some(h) => {
                let mut names: Vec<String> = h.pieces.iter().take(3).map(|(p, n)| format!("{n} [{p}]")).collect();
                if h.pieces.len() > 3 {
                    names.push(format!("{} more", h.pieces.len() - 3));
                }
                let size = if h.size.is_empty() { String::new() } else { format!(", {}", h.size) };
                let of = if h.pieces.len() > 1 { format!(", through {} pieces of the run", h.pieces.len()) } else { String::new() };
                let _ = writeln!(out, "  {} {} and {} [{}]: 1 hole{size}{of}, at {}", h.status.flag(), names.join(", "), h.through_name, h.through, place(c));
            }
            None => {
                let _ = writeln!(out, "  {}, at {}", pairs(c), place(c));
            }
        }
    }
    // A run that lies in a floor or a wall along its length is no hole: it never comes out (call 179, C5).
    let laid = found.laid_in();
    let _ = writeln!(out, "Laid in a floor or a wall (it runs inside it, along its length: no hole to cut): {}{}", laid.len(), assumed(&laid));
    for c in &laid {
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
/// What the soffit taken off as length times overhang counts at its corners (call 191 items 12 and 17): corners where two runs meet are in
/// both, an inside corner is in neither, and the net is how much more the formula gives than the roof has.
pub(super) fn soffit_corner_figures(corners: &[plumbline_world::trim::Corner]) -> (usize, f64, usize, f64) {
    let (twice, none): (Vec<_>, Vec<_>) = corners.iter().partition(|c| !c.inside);
    (twice.len(), twice.iter().map(|c| c.area).sum(), none.len(), none.iter().map(|c| c.area).sum())
}

pub(super) fn corner_count(n: usize, what: &str) -> String {
    if n == 1 { format!("1 {what}") } else { format!("{n} {what}s") }
}

pub(super) fn soffit_corners_said(corners: &[plumbline_world::trim::Corner]) -> String {
    let (n, a, m, b) = soffit_corner_figures(corners);
    if m == 0 {
        return format!("each corner where two runs meet in both: {}, {} SF in all", corner_count(n, "corner"), view::figure2(a));
    }
    format!("each corner where two runs meet in both ({}, {} SF in two runs) and the square at each inside corner in neither ({}, {} SF), so it gives {} SF more than the roof has", corner_count(n, "corner"), view::figure2(a), corner_count(m, "inside corner"), view::figure2(b), view::figure2(a - b))
}

pub(super) fn soffit_corners_said_priced(corners: &[plumbline_world::trim::Corner]) -> String {
    let (n, a, m, b) = soffit_corner_figures(corners);
    if m == 0 {
        return format!("{} SF is corner counted in two runs ({})", view::figure2(a), corner_count(n, "corner"));
    }
    format!("{} SF is corner counted in two runs ({}) and {} SF at the {} is counted in neither: the lines price {} SF more than the roof has", view::figure2(a), corner_count(n, "corner"), view::figure2(b), corner_count(m, "inside corner"), view::figure2(a - b))
}

/// The outside trim's figures, each as the model works it out (call 185): the same figures a cost line's quantity reads.
pub(super) fn trim_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::trim;
    use plumbline_world::{Kind, Measure, Node};
    let figures = trim::figures(job);
    let corners = trim::soffit_corners(&mut job.world);
    let shown = |d: &plumbline_world::Derived, unit: &str| match d.value {
        Some(v) => format!("{}{} {unit}", d.status.flag().trim_end(), view::num(v)),
        None => format!("? no figure ({})", if d.note.is_empty() { "not worked out" } else { d.note.as_str() }),
    };
    // Each thing's name, its kind and, for an eave or a rake, its length.
    let mut info: std::collections::HashMap<String, (String, Kind, String)> = std::collections::HashMap::new();
    for f in &figures {
        if info.contains_key(&f.path) {
            continue;
        }
        let Some(e) = job.find(&f.path) else { continue };
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| f.path.clone());
        let kind = job.world.get::<Kind>(e).copied().unwrap_or(Kind::Wall);
        let length = match job.world.get::<Measure>(e).and_then(|m| m.0.get("true_length")) {
            Some(d) => shown(d, "LF"),
            None => "? no figure".to_string(),
        };
        info.insert(f.path.clone(), (name, kind, length));
    }
    let mut out = String::new();
    let _ = writeln!(out, "The outside trim, as it is worked out from what is drawn. Nothing is priced here: a cost line names the figure it is priced by.");
    let _ = writeln!(out, "(~ is assumed, ? is not known; a corner is counted on both its walls.)");
    let section = |out: &mut String, title: &str, names: &[&str], kinds: &[Kind]| {
        let mine: Vec<&trim::Figure> = figures.iter().filter(|f| names.contains(&f.name) && info.get(&f.path).is_some_and(|i| kinds.contains(&i.1))).collect();
        if mine.is_empty() {
            return;
        }
        let _ = writeln!(out, "{title}");
        let mut seen: Vec<&str> = Vec::new();
        for f in mine {
            if seen.contains(&f.path.as_str()) {
                continue;
            }
            seen.push(&f.path);
            let of = |name: &str| figures.iter().find(|g| g.path == f.path && g.name == name).map(|g| &g.figure);
            let said = match f.name {
                trim::OVERHANG => {
                    let d = &f.figure;
                    let under = d.inputs.iter().find(|i| i.starts_with("wall-")).and_then(|i| i.split('#').next()).map(|w| format!(", the outside face of {w}")).unwrap_or_default();
                    let name_of = |p: &str| info.get(p).map(|i| format!("{} [{p}]", i.0)).unwrap_or_else(|| p.to_string());
                    let at_corner: Vec<String> = corners
                        .iter()
                        .filter(|c| c.a == f.path || c.b == f.path)
                        .map(|c| {
                            let (other, sq) = (name_of(if c.a == f.path { &c.b } else { &c.a }), view::num((c.area * 10_000.0).round() / 10_000.0));
                            if c.inside {
                                format!("its soffit and {other}'s leave the square at the inside corner at ({}, {}) uncovered: {sq} SF is in neither run when soffit is taken as length times overhang", view::num(c.at[0]), view::num(c.at[1]))
                            } else {
                                format!("its soffit and {other}'s lie over one another at the corner at ({}, {}): {sq} SF is in both runs when soffit is taken as length times overhang", view::num(c.at[0]), view::num(c.at[1]))
                            }
                        })
                        .collect();
                    let corner = if at_corner.is_empty() { String::new() } else { format!("; {}", at_corner.join("; ")) };
                    format!("{} long; overhang {}{}{corner}", info[&f.path].2, shown(d, "ft"), if d.value.is_some() { under } else { String::new() })
                }
                trim::OUTSIDE_CORNERS => {
                    let (n, l) = (of(trim::OUTSIDE_CORNERS), of(trim::CORNER_BOARD_LENGTH));
                    format!("outside corners {}; corner board {}", n.map(|d| shown(d, "EA")).unwrap_or_default(), l.map(|d| shown(d, "LF")).unwrap_or_default())
                }
                _ => format!("casing {}", shown(&f.figure, "LF")),
            };
            let _ = writeln!(out, "  {} [{}]: {said}", info[&f.path].0, f.path);
        }
    };
    section(&mut out, "Eaves and rakes:", &[trim::OVERHANG], &[Kind::Eave, Kind::Rake]);
    if !corners.is_empty() {
        let _ = writeln!(out, "  Soffit is taken off as the length of the edges times the overhang, with waste on top, and that counts {} (the formula stays; this is how much of the soffit it is).", soffit_corners_said(&corners));
    }
    section(&mut out, "Walls:", &[trim::OUTSIDE_CORNERS], &[Kind::Wall]);
    section(&mut out, "Windows and doors:", &[trim::CASING_LENGTH], &[Kind::Window, Kind::Door]);
    if figures.is_empty() {
        let _ = writeln!(out, "Nothing drawn has outside trim to work out: no eave, rake, wall, window or door.");
    }
    out
}

/// Each wall's gable, as the model works it out (call 192): the same figure a cost line's quantity reads.
pub(super) fn gable_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::{gable, Node};
    let figures = gable::figures(job);
    let mut out = String::new();
    let _ = writeln!(out, "The gable ends, as they are worked out from what is drawn. A wall under a rake, or under the high edge of a shed roof, rises to the roof; its area is priced by a line that names it (\"net_area + gable_area\"), and the frame of a gable is not counted.");
    let _ = writeln!(out, "(~ is assumed, ? is not known; a top is how far along the wall's line from its first end, and how high above the datum.)");
    if figures.is_empty() {
        let _ = writeln!(out, "Nothing drawn has a wall.");
    }
    for g in &figures {
        let name = job.find(&g.path).and_then(|e| job.world.get::<Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| g.path.clone());
        let d = &g.area;
        match d.value {
            Some(v) if g.top.is_empty() => {
                let _ = writeln!(out, "  {name} [{}]: {}", g.path, if g.why.is_empty() { format!("{}{} SF", d.status.flag().trim_end(), view::num(v)) } else { format!("{}none: {}", match d.status.flag().trim_end() { "" => String::new(), flag => format!("{flag} ") }, g.why) });
            }
            Some(v) => {
                let top: Vec<String> = g.top.iter().map(|(s, z)| format!("({}, {})", view::num((s * 10_000.0).round() / 10_000.0), view::num((z * 10_000.0).round() / 10_000.0))).collect();
                let _ = writeln!(out, "  {name} [{}]: gable {}{} SF; top {} ft", g.path, d.status.flag().trim_end(), view::num(v), top.join(" "));
                if !d.note.is_empty() {
                    let _ = writeln!(out, "      ! {}", d.note);
                }
                let _ = writeln!(out, "      rests on: {}", d.inputs.join(", "));
            }
            None => {
                let _ = writeln!(out, "  {name} [{}]: ? no figure ({})", g.path, if d.note.is_empty() { "not worked out" } else { d.note.as_str() });
            }
        }
    }
    out
}

/// Where each piece of outside trim lies, one by one (call 184, step 1): only what a cost line prices.
pub(super) fn trim_pieces_said(job: &mut plumbline_world::Job) -> String {
    use plumbline_world::trim::{self, PieceKind};
    let laid = trim::lay(job);
    let flag = |s: Status| s.flag().trim_end();
    let point = |p: [f64; 3]| format!("({}, {}, {})", view::num(p[0]), view::num(p[1]), view::num(p[2]));
    let compass = |x: f64, y: f64| if x.abs() >= y.abs() { if x > 0.0 { "east" } else { "west" } } else if y > 0.0 { "north" } else { "south" };
    let looks = |f: [f64; 3]| {
        let (h, flat) = (f[0].hypot(f[1]), f[2].abs() < 1e-6);
        if flat {
            format!("looks {}", compass(f[0], f[1]))
        } else if h < 0.05 {
            if f[2] < 0.0 { "looks down".to_string() } else { "looks up".to_string() }
        } else {
            format!("looks {} and out toward the {}", if f[2] < 0.0 { "down" } else { "up" }, compass(f[0], f[1]))
        }
    };
    let mut out = String::new();
    let _ = writeln!(out, "The outside trim, piece by piece: where each lies. A piece is drawn only where a cost line prices it, and the pieces of a kind add up to what the line prices.");
    let _ = writeln!(out, "(~ is assumed, ? is not known. Places are in feet: east, north, up. A piece's from and to are the middle of its back, the side against what it is fixed to; width is across its face, thick is out from the wall or roof.)");
    for kind in PieceKind::ALL {
        let mine: Vec<&trim::Piece> = laid.pieces.iter().filter(|p| p.kind == kind).collect();
        if mine.is_empty() {
            continue;
        }
        let _ = writeln!(out, "{}:", { let w = kind.words(); let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() });
        for p in mine {
            let name = job.find(&p.of).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| p.of.clone());
            let long = ((p.to[0] - p.from[0]).powi(2) + (p.to[1] - p.from[1]).powi(2) + (p.to[2] - p.from[2]).powi(2)).sqrt();
            let _ = writeln!(out, "  {name} [{}], priced by \"{}\": {}{} LF from {} to {}; {}; {} wide, {} thick ({})", p.of, p.line, flag(p.status), view::num((long * 10_000.0).round() / 10_000.0), point(p.from), point(p.to), looks(p.face), trim::lay::inches_said(p.width), trim::lay::inches_said(p.thickness), p.sized_from);
        }
    }
    let mut any = false;
    for kind in PieceKind::ALL {
        let priced = laid.priced.iter().find(|(k, _)| *k == kind).and_then(|(_, q)| *q);
        let no_figure = laid.no_figure_of(kind);
        let mine: Vec<&trim::Piece> = laid.pieces.iter().filter(|p| p.kind == kind).collect();
        if priced.is_none() && no_figure == 0 {
            continue;
        }
        if !any {
            let _ = writeln!(out, "Each kind's pieces beside what its lines price:");
            any = true;
        }
        let area = kind.unit() == "SF";
        let sum: f64 = mine.iter().map(|p| ((p.to[0] - p.from[0]).powi(2) + (p.to[1] - p.from[1]).powi(2) + (p.to[2] - p.from[2]).powi(2)).sqrt() * if area { p.width } else { 1.0 }).sum();
        let drawn = format!("{} piece{} {} to {} {}", mine.len(), if mine.len() == 1 { "" } else { "s" }, if mine.len() == 1 { "comes" } else { "come" }, view::figure2(sum), kind.unit());
        let lacking = if no_figure == 0 { String::new() } else { format!("; {} of its lines {} no figure", no_figure, if no_figure == 1 { "has" } else { "have" }) };
        match priced {
            // The lines price it and every one has no figure: "no figure", not nothing (call 191 item 11).
            None => {
                let _ = writeln!(out, "  {}: {drawn}; the lines price no figure: {no_figure} of its lines {} no figure (see what is left out)", kind.words(), if no_figure == 1 { "has" } else { "have" });
            }
            Some(priced) => {
                // The same to the cent is the same: both are said as the estimate says them, to 2 places (call 191 item 13).
                let cents = |x: f64| (x * 100.0).round();
                let same = cents(sum) == cents(priced);
                let short = if !same && priced > sum { format!(": {} {} of what the lines price is not drawn", view::figure2(priced - sum), kind.unit()) } else { String::new() };
                let _ = writeln!(out, "  {}: {drawn}; the lines price {} {}{lacking}{}{short}", kind.words(), view::figure2(priced), kind.unit(), if same { "" } else { " (they differ: see what is left out)" });
            }
        }
    }
    // Where two runs of soffit meet at a corner, the soffit taken off as length times overhang counts the corner in both (call 191 item 12).
    {
        let drawn_soffit: std::collections::HashSet<&str> = laid.pieces.iter().filter(|p| matches!(p.kind, PieceKind::Soffit | PieceKind::RakeSoffit)).map(|p| p.of.as_str()).collect();
        let both: Vec<_> = trim::soffit_corners(&mut job.world).into_iter().filter(|c| drawn_soffit.contains(c.a.as_str()) && drawn_soffit.contains(c.b.as_str())).collect();
        if !both.is_empty() {
            let _ = writeln!(out, "  Of the soffit the lines price, {}: that is how soffit is taken off, the length of the edges times the overhang, and the pieces drawn lie over one another at the corners that are counted twice.", soffit_corners_said_priced(&both));
        }
    }
    if laid.pieces.is_empty() && laid.left_out.is_empty() {
        let _ = writeln!(out, "No cost line prices any outside trim here, so no piece is drawn.");
    }
    if !laid.left_out.is_empty() {
        let _ = writeln!(out, "Left out:");
        for l in &laid.left_out {
            let _ = writeln!(out, "  {l}");
        }
    }
    out
}

/// What carries the building, for the engineer: what the world says, shown.
pub(super) fn structure(job: &mut plumbline_world::Job) -> String {
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
