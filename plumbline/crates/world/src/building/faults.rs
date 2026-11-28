use super::*;

/// What can only be checked once every element is in: an opening is under a wall, a wall
/// and a floor are under a storey, a run and a placed thing are under a storey too, a type is under the job and says what it is a type of,
/// a type named is a type of the right sort, a top level named is a storey, and a system
/// named is a system in the job.
pub fn check_whole(job: &mut Job) -> Result<(), String> {
    // What is to be demolished is gone when the work is done: a new wall may stand
    // where it stood (call 148).
    let gone = crate::components::demolished(&mut job.world);
    let tree = Tree::of(&mut job.world);
    let kinds = &tree.kinds;
    let classed: HashMap<String, String> = {
        let mut q = job.world.query::<(&Node, &SystemInfo)>();
        q.iter(&job.world).map(|(n, s)| (n.path.clone(), s.classification.clone())).collect()
    };
    // Each wall's storey, its line, and how high it stands where that is entered outright.
    // Each wall with a line: its storey, its centre line, its thickness (none when its
    // type's is not known) and the heights it stands between, where both are known.
    let mut standing: Vec<(String, String, [P2; 2], f64, Option<(f64, f64)>)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, Option<&InSystem>, Option<&Wall>, Option<&Opening>, Option<&Floor>, Option<&Segment>, Option<&Stood>)>();
    for (node, kind, in_system, wall, opening, floor, segment, column) in q.iter(&job.world) {
        let me = &node.path;
        let parent = node.parent.as_ref().and_then(|p| kinds.get(p)).copied();
        if kind.is_opening() && parent != Some(Kind::Wall) {
            return Err(format!("{me}: {kind:#} is an opening in a wall and has to be under one"));
        }
        if *kind == Kind::Room && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a room is a space on a level and has to be directly under one"));
        }
        if *kind == Kind::RoomLine && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a room line parts the rooms of a level and has to be directly under one"));
        }
        if matches!(kind, Kind::Wall | Kind::Floor | Kind::Footing) && parent != Some(Kind::Storey) {
            return Err(format!("{me}: {kind:#} stands on a level and has to be under one"));
        }
        if *kind == Kind::Drawing && parent != Some(Kind::Storey) {
            return Err(format!("{me}: a drawing is a plan of a level and has to be directly under one"));
        }
        if *kind == Kind::Prior && parent != Some(Kind::Job) {
            return Err(format!("{me}: a prior estimate is set against the whole job and has to be directly under it"));
        }
        if *kind == Kind::Storey && parent != Some(Kind::Job) {
            return Err(format!("{me}: a level is a floor of the building and has to be directly under the job, not under {}", node.parent.as_deref().unwrap_or("nothing")));
        }
        if *kind == Kind::Type {
            if parent != Some(Kind::Job) {
                return Err(format!("{me}: a type belongs to the whole job and has to be directly under it, not under {}", node.parent.as_deref().unwrap_or("nothing")));
            }
            let sorts = [tree.wall_types.contains_key(me), tree.floor_types.contains_key(me), tree.opening_types.contains_key(me), tree.segment_types.contains_key(me), tree.items.contains(me)].into_iter().filter(|x| *x).count();
            if sorts != 1 {
                return Err(format!("{me}: a type is one sort of type (of a wall, a floor, a door or a window, or a run) or an item to place; this one carries {sorts}"));
            }
        }
        if let Some(named) = in_system.and_then(|s| s.0.value.as_ref()) {
            if kinds.get(named) != Some(&Kind::System) {
                return Err(format!("{me}: says it belongs to {named:?}, which is not a system in this job"));
            }
            // A duct does not carry water. A fixture, equipment or a fitting can be on any system.
            let goes_on = run_goes_on(*kind);
            if let Some(class) = classed.get(named).filter(|c| !goes_on.is_empty() && !goes_on.contains(&c.as_str())) {
                return Err(format!("{me}: a {} is on {named}, which is a {class} system; a {} goes on: {}", kind.stem(), kind.stem(), goes_on.join(", ")));
            }
        }
        if let Some(w) = wall {
            if let (Some(line), Some(storey)) = (w.line_ft(), node.parent.as_ref()) {
                // Its body, not the line it was drawn on: the centre line lies to the
                // left of the drawn line by what its `location` says.
                let (thick, centre) = match wall_section(&tree, w) {
                    Ok((thickness, layers, _)) => {
                        let off = walls::centre(thickness, &layers, w.location);
                        let length = geom::dist(line[0], line[1]).max(1.0e-12);
                        let left = [-(line[1][1] - line[0][1]) / length * off, (line[1][0] - line[0][0]) / length * off];
                        (thickness, [[line[0][0] + left[0], line[0][1] + left[1]], [line[1][0] + left[0], line[1][1] + left[1]]])
                    }
                    Err(_) => (0.0, line),
                };
                // Where it stands, however its top is said: by a height or by a level.
                let heights = wall_heights(&tree, me, w);
                // A footing lies below its base by its depth; a wall stands up from it. So a
                // wall on its footing, on the same line, is two things one over the other.
                let stands = heights.base.ok().zip(heights.height.ok()).map(|(b, h)| if *kind == Kind::Footing { (b - h, b) } else { (b, b + h) });
                standing.push((me.clone(), storey.clone(), centre, thick, stands));
            }
            if let Some(p) = &w.type_name.value {
                if !tree.wall_types.contains_key(p) {
                    return Err(format!("{me}: its type is {p:?}, which is not a {} type in this job", if *kind == Kind::Footing { "footing" } else { "wall" }));
                }
            }
            if let Some(p) = w.top_level.as_ref().and_then(|t| t.value.as_ref()) {
                if kinds.get(p) != Some(&Kind::Storey) {
                    return Err(format!("{me}: its top level is {p:?}, which is not a level in this job"));
                }
            }
        }
        if let Some(p) = opening.and_then(|o| o.type_name.value.as_ref()) {
            match tree.opening_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a door or window type in this job")),
                Some(t) if t.category != *kind => return Err(format!("{me}: is {kind:#}, but its type, {p}, is {:#}'s", t.category)),
                Some(_) => {}
            }
        }
        if let Some(p) = floor.and_then(|f| f.type_name.value.as_ref()) {
            if !tree.floor_types.contains_key(p) {
                return Err(format!("{me}: its type is {p:?}, which is not a floor type in this job"));
            }
        }
        if let Some(p) = segment.and_then(|r| r.type_name.value.as_ref()) {
            match tree.segment_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a run type in this job")),
                Some(t) if t.category != *kind => return Err(format!("{me}: is {kind:#}, but its type, {p}, is {:#}'s", t.category)),
                Some(_) => {}
            }
        }
        if let Some(p) = column.and_then(|c| c.type_name.value.as_ref()) {
            match tree.segment_types.get(p) {
                None => return Err(format!("{me}: its type is {p:?}, which is not a {} type in this job", kind.said())),
                Some(t) if t.category != *kind => return Err(format!("{me}: is {kind:#}, but its type, {p}, is {:#}'s", t.category)),
                Some(_) => {}
            }
        }
        // A run's heights and a placed thing's are above the storey it is under.
        if (kind.is_segment() || kind.is_placed() || kind.is_framing()) && tree.storey_of(me).is_none() {
            return Err(format!("{me}: {kind:#} is set at a height above a level and has to be under one"));
        }
    }
    // One wall drawn over another on a storey is the same wall counted twice. Two that
    // stand one over the other (a wall on a knee wall) are two walls.
    standing.sort_by(|x, y| x.0.cmp(&y.0));
    for (i, (a, storey, line, thick, stands)) in standing.iter().enumerate() {
        for (b, other, its, as_thick, theirs) in &standing[i + 1..] {
            let tol = crate::check::FIT_TOLERANCE_FT;
            if gone.contains(a) != gone.contains(b) {
                continue;
            }
            // Where each stands, above the datum: walls on two storeys are compared too
            // (a wall held up to the roof and another on the floor above, on one line).
            // Where that is not known, walls of one storey are taken to stand together
            // and walls of two storeys apart.
            match (stands, theirs) {
                (Some((lo, hi)), Some((lo2, hi2))) => {
                    if *hi <= lo2 + tol || *hi2 <= lo + tol {
                        continue;
                    }
                }
                _ if storey != other => continue,
                _ => {}
            }
            if let Some(shared) = bodies_overlap(*line, *thick, *its, *as_thick) {
                // Said in the words of what the two are (a footing is held as a wall is).
                let what = if b.starts_with("footing") { "footing" } else { "wall" };
                return Err(format!("{b}: lies along {a} for {}: one {what} drawn over another would be counted twice", crate::units::length_said(shared)));
            }
        }
    }
    Ok(())
}

/// What may not be drawn, and is said of a job that already has it (call 179, B5 and B6): an
/// opening past the end of its wall or over another opening in it; two floors of one level in
/// each other; two beams, two posts or two pad footings in one place. Each is a thing counted
/// twice, or hung where there is no wall, as two walls on one line are. Each comes as a key
/// (the thing and which fault) and the sentence. A job saved before that holds one still opens
/// and still says it (`check` prints these); what an edit may not do is bring one in
/// (`editing::all_or_nothing` refuses a fault the job did not have).
pub fn drawn_faults(job: &mut Job) -> Vec<(String, String)> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let gone = crate::components::demolished(&mut job.world);
    let tree = Tree::of(&mut job.world);
    let w = &job.world;
    let mut out: Vec<(String, String)> = Vec::new();
    let mut nodes: Vec<(&Node, Entity)> = job.order.iter().filter_map(|&e| w.get::<Node>(e).map(|n| (n, e))).collect();
    nodes.sort_by(|x, y| x.0.path.cmp(&y.0.path));
    // Openings: within the wall, clear of one another.
    for (n, e) in &nodes {
        if w.get::<Kind>(*e) != Some(&Kind::Wall) {
            continue;
        }
        let Some(line) = w.get::<Wall>(*e).and_then(|c| c.line_ft()) else { continue };
        let length = geom::dist(line[0], line[1]);
        let wall_high = w.get::<Wall>(*e).and_then(|c| plan_wall(&tree, &n.path, c).ok()).map(|(placed, _, _, _)| placed.top - placed.base);
        let mut spans: Vec<(&str, [f64; 4])> = Vec::new();
        for (c, ce) in &nodes {
            if c.parent.as_deref() != Some(n.path.as_str()) || !w.get::<Kind>(*ce).is_some_and(|k| k.is_opening()) {
                continue;
            }
            // (An opening to demolish is no hole in the wall that is built: the wall's figures leave it out,
            // and a new one may stand where it stood.)
            if gone.contains(&c.path) {
                continue;
            }
            let Some(o) = w.get::<Opening>(*ce) else { continue };
            let Some(span) = hole_of(&tree, &c.path, o).0.span else { continue };
            let [from, to, _, head] = span;
            // A door or a window does not go above the top of its wall (call 180, C5).
            if let Some(high) = wall_high {
                if head > high + tol {
                    out.push((format!("{} over the top", c.path), format!("{} goes up {} from the floor and its wall is only {} high: an opening lies within the height of its wall", c.path, ft_in_said(head), ft_in_said(high))));
                }
            }
            if from < -tol {
                out.push((format!("{} off the start", c.path), format!("{} starts {} before the start of its wall: an opening lies within its wall", c.path, crate::units::length_said(-from))));
            } else if to > length + tol {
                out.push((format!("{} off the end", c.path), format!("{} ends {} along a wall {} long: an opening lies within its wall", c.path, crate::units::length_said(to), crate::units::length_said(length))));
            }
            for (other, s) in &spans {
                if from < s[1] && s[0] < to && span[2] < s[3] && s[2] < span[3] {
                    out.push((format!("{} over {other}", c.path), format!("{} and {other} lie over each other: an opening is clear of every other opening in its wall", c.path)));
                }
            }
            spans.push((c.path.as_str(), span));
        }
    }
    // Floors of one level: none in another.
    let floors: Vec<(&str, Option<&str>, Vec<P2>)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Floor)).filter_map(|(n, e)| Some((n.path.as_str(), n.parent.as_deref(), w.get::<Floor>(*e)?.boundary_ft()?))).collect();
    for (i, (a, level, ring)) in floors.iter().enumerate() {
        for (b, other, its) in &floors[i + 1..] {
            if level == other && gone.contains(*a) == gone.contains(*b) && rings_overlap(ring, its) {
                out.push((format!("{b} over {a}"), format!("{b} and {a} cover the same ground on one level: two floors in one place would be counted twice")));
            }
        }
    }
    // Beams, posts and pad footings: not two in one place.
    // (Call 180, C4: for the length they share a line, within the width of the beam, and not only where one lies exactly on the other.)
    let beams: Vec<(&str, Option<&str>, [P2; 2], (f64, f64), f64)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Beam)).filter_map(|(n, e)| {
        let s = w.get::<Segment>(*e)?;
        let wide = tree.segment_types.get(s.type_name.value.as_deref()?).and_then(|t| t.width.feet()).unwrap_or(0.0);
        Some((n.path.as_str(), n.parent.as_deref(), s.line_ft()?, s.offsets_ft()?, wide))
    }).collect();
    for (i, (a, level, line, top, wide_a)) in beams.iter().enumerate() {
        for (b, other, its, their_top, wide_b) in &beams[i + 1..] {
            if level != other || gone.contains(*a) != gone.contains(*b) {
                continue;
            }
            let (la, lb) = (geom::dist(line[0], line[1]), geom::dist(its[0], its[1]));
            if la <= tol || lb <= tol {
                continue;
            }
            let (u, v) = ([(line[1][0] - line[0][0]) / la, (line[1][1] - line[0][1]) / la], [(its[1][0] - its[0][0]) / lb, (its[1][1] - its[0][1]) / lb]);
            // On one line: running within a degree of the same way, their lines within a beam's width of one another.
            if (u[0] * v[1] - u[1] * v[0]).abs() > 0.017_452_406_437_283_512 {
                continue;
            }
            let (along, across) = (|p: P2| (p[0] - line[0][0]) * u[0] + (p[1] - line[0][1]) * u[1], |p: P2| ((p[1] - line[0][1]) * u[0] - (p[0] - line[0][0]) * u[1]).abs());
            if across(its[0]).max(across(its[1])) > wide_a.max(*wide_b).max(tol) + tol {
                continue;
            }
            let (lo, hi) = (along(its[0]).min(along(its[1])).max(0.0), along(its[0]).max(along(its[1])).min(la));
            if hi - lo <= tol {
                continue;
            }
            // And at one height over what they share: the top of each at the two ends of the stretch.
            let height = |t: (f64, f64), len: f64, at: f64| t.0 + (t.1 - t.0) * (at / len);
            let flip = along(its[0]) > along(its[1]);
            let (theirs_at_lo, theirs_at_hi) = {
                let (s0, s1) = (along(its[0]), along(its[1]));
                let (first, second) = if flip { (their_top.1, their_top.0) } else { (their_top.0, their_top.1) };
                let (s_lo, s_hi) = (s0.min(s1), s0.max(s1));
                let on = |at: f64| first + (second - first) * ((at - s_lo) / (s_hi - s_lo).max(1.0e-12));
                (on(lo), on(hi))
            };
            if (height(*top, la, lo) - theirs_at_lo).abs() > tol || (height(*top, la, hi) - theirs_at_hi).abs() > tol {
                continue;
            }
            out.push((format!("{b} over {a}"), format!("{b} shares a line with {a} for {}: two beams in one place would be counted twice", ft_in_said(hi - lo))));
        }
    }
    for kind in [Kind::Post, Kind::PadFooting] {
        let columns: Vec<(&str, Option<&str>, P2, (f64, f64), [f64; 2])> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&kind)).filter_map(|(n, e)| {
            let c = w.get::<Stood>(*e)?;
            let (base, height) = (c.base_offset.feet()?, c.height.feet()?);
            // Its footprint: its type's width by its depth, or across for a round one or one with no depth.
            let t = tree.segment_types.get(c.type_name.value.as_deref()?);
            let across = t.and_then(|t| t.width.feet()).unwrap_or(0.0);
            let deep = t.and_then(|t| t.depth.as_ref().and_then(|d| d.feet())).unwrap_or(across);
            Some((n.path.as_str(), n.parent.as_deref(), c.at_ft()?, (base, base + height), [across, deep]))
        }).collect();
        for (i, (a, level, at, span, size)) in columns.iter().enumerate() {
            for (b, other, its, theirs, their_size) in &columns[i + 1..] {
                if level != other || gone.contains(*a) != gone.contains(*b) || !(span.0 < theirs.1 - tol && theirs.0 < span.1 - tol) {
                    continue;
                }
                // (Call 180, C4: pad footings whose plan areas overlap, not only the same point.)
                let overlap = if kind == Kind::PadFooting { (at[0] - its[0]).abs() < (size[0] + their_size[0]) / 2.0 - tol && (at[1] - its[1]).abs() < (size[1] + their_size[1]) / 2.0 - tol } else { false };
                if geom::dist(*at, *its) <= tol || overlap {
                    out.push((format!("{b} over {a}"), format!("{b} lies where {a} is: two {} in one place would be counted twice", if kind == Kind::Post { "posts" } else { "pad footings" })));
                }
            }
        }
    }
    out
}

/// Feet as a builder says them aloud: "4 ft", "11 ft 6 in".
pub(super) fn ft_in_said(feet: f64) -> String {
    if (feet - feet.round()).abs() < 1.0e-6 {
        format!("{} ft", feet.round())
    } else {
        units::feet_as_ft_in(feet, 16).replacen("' ", " ft ", 1).replacen('"', " in", 1).trim_start_matches("0 ft ").to_string()
    }
}

/// What is drawn out of reach of what should carry it (call 179, B1): a room that reaches
/// past the floor of its level, and a wall that stands off a footing that has no wall on it.
/// Cautions, never refusals: the drawing may be what is meant, and `move --alone` leaves
/// exactly this. Each comes as a key and the sentence. A level with no floor drawn says
/// nothing of its rooms, and a footing with no wall near it says nothing.
pub fn reach_cautions(job: &mut Job) -> Vec<(String, String)> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let gone = crate::components::demolished(&mut job.world);
    let tree = Tree::of(&mut job.world);
    let outlines = room_outlines(job);
    let bodies = crate::carries::bodies(job);
    let w = &job.world;
    let mut out: Vec<(String, String)> = Vec::new();
    let mut nodes: Vec<(&Node, Entity)> = job.order.iter().filter_map(|&e| w.get::<Node>(e).map(|n| (n, e))).collect();
    nodes.sort_by(|x, y| x.0.path.cmp(&y.0.path));
    // Rooms and floors of one level.
    let floors: Vec<(&Node, Vec<P2>)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Floor)).filter_map(|(n, e)| Some((*n, w.get::<Floor>(*e)?.boundary_ft()?))).filter(|(n, _)| !gone.contains(&n.path)).collect();
    let inside = |p: P2, ring: &[P2]| crate::vif::inside(ring, p) || geom::point_to_outline(p, ring) <= tol;
    for (n, _) in nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Room)) {
        let Some(poly) = outlines.get(&n.path) else { continue };
        let there: Vec<&(&Node, Vec<P2>)> = floors.iter().filter(|(f, _)| f.parent == n.parent).collect();
        if there.is_empty() || gone.contains(&n.path) {
            continue;
        }
        // How far each corner of the room is outside every floor of its level.
        let reach = |ring: Option<&Vec<P2>>| -> f64 {
            poly.iter()
                .map(|p| {
                    let rings: Vec<&Vec<P2>> = match ring {
                        Some(r) => vec![r],
                        None => there.iter().map(|(_, r)| r).collect(),
                    };
                    if rings.iter().any(|r| inside(*p, r)) { 0.0 } else { rings.iter().map(|r| geom::point_to_outline(*p, r)).fold(f64::MAX, f64::min) }
                })
                .fold(0.0, f64::max)
        };
        let past = reach(None);
        if past > tol {
            let nearest = there.iter().min_by(|a, b| reach(Some(&a.1)).total_cmp(&reach(Some(&b.1)))).map(|(f, _)| f.name.clone()).unwrap_or_default();
            out.push((format!("{} past its floor", n.path), format!("{} reaches {} past {nearest}: the floor is drawn smaller than the room (a wall was moved, or the floor was drawn short)", if n.name.is_empty() { n.path.clone() } else { n.name.clone() }, ft_in_said(past))));
        }
    }
    // A floor or a roof that no longer reaches the walls under it, or reaches past them (call 180, B1): an edge
    // along a wall with no other wall between, the wall outside it by more than its thickness, or the edge
    // beyond a wall inside it by more than a floor's 2 ft or an eave's 4 ft. A floor is looked at against the
    // walls of its own level, a roof against every wall (a roof is over a wall by the plan alone).
    {
        let roofs: Vec<(&Node, Vec<P2>)> = nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Roof)).filter_map(|(n, e)| Some((*n, w.get::<Footprint>(*e)?.points_ft()?))).filter(|(n, _)| !gone.contains(&n.path)).collect();
        let several = bodies.iter().map(|b| b.storey.as_str()).collect::<std::collections::BTreeSet<_>>().len() > 1;
        let called = |path: &str, storey: &str| match (tree.names.get(path), tree.names.get(storey)) {
            (Some(n), Some(l)) if several && !l.is_empty() => format!("{n} on {l}"),
            (Some(n), _) => n.clone(),
            _ => path.to_string(),
        };
        let things: Vec<(&Node, &Vec<P2>, bool)> = floors.iter().map(|(n, r)| (*n, r, false)).chain(roofs.iter().map(|(n, r)| (*n, r, true))).collect();
        for (thing, ring, is_roof) in things {
            let reach = if is_roof { crate::carries::ROOF_REACH_FT } else { crate::carries::FLOOR_REACH_FT };
            for b in bodies.iter().filter(|b| !gone.contains(&b.path) && (is_roof || Some(b.storey.as_str()) == thing.parent.as_deref())) {
                let found = crate::carries::edges_all(b, ring, &bodies.iter().filter(|o| !gone.contains(&o.path)).cloned().collect::<Vec<_>>(), is_roof);
                let Some((_, e)) = found.iter().filter(|(_, e)| (!e.wall_inside && e.across > b.thick + tol) || (e.wall_inside && e.outward && e.across > reach + tol)).min_by(|x, y| x.1.across.total_cmp(&y.1.across)) else { continue };
                let name = if thing.name.is_empty() { thing.path.clone() } else { thing.name.clone() };
                let wall = called(&b.path, &b.storey);
                out.push((format!("{} off {}", b.path, thing.path), if e.wall_inside {
                    format!("{name} reaches {} past {wall}: more than {} ft, the most a {} is drawn out from a wall that goes with it", ft_in_said(e.across), reach, if is_roof { "roof's eave" } else { "floor's edge" })
                } else {
                    format!("{wall} stands {} outside {name}: {name} no longer reaches it", ft_in_said(e.across))
                }));
            }
        }
    }
    // Footings with no wall on them, and a wall that stands off one.
    let several = bodies.iter().map(|b| b.storey.as_str()).collect::<std::collections::BTreeSet<_>>().len() > 1;
    let called = |path: &str, storey: &str| match (tree.names.get(path), tree.names.get(storey)) {
        (Some(n), Some(l)) if several && !l.is_empty() => format!("{n} on {l}"),
        (Some(n), _) => n.clone(),
        _ => path.to_string(),
    };
    for (n, e) in nodes.iter().filter(|(_, e)| w.get::<Kind>(*e) == Some(&Kind::Footing)) {
        let Some(line) = w.get::<Wall>(*e).and_then(|c| c.line_ft()) else { continue };
        let Some(level) = n.parent.as_deref() else { continue };
        if gone.contains(&n.path) {
            continue;
        }
        let flen = geom::dist(line[0], line[1]);
        if flen <= tol {
            continue;
        }
        let on_it = bodies.iter().any(|b| b.storey == level && !gone.contains(&b.path) && crate::carries::over(b.line, b.thick, line).is_some());
        if on_it {
            continue;
        }
        let (ux, uy) = ((line[1][0] - line[0][0]) / flen, (line[1][1] - line[0][1]) / flen);
        // The nearest wall of its level that runs along it, within 10 ft, and overlaps it.
        let mut near: Option<(f64, &crate::carries::Body)> = None;
        for b in bodies.iter().filter(|b| b.storey == level && !gone.contains(&b.path)) {
            let (bx, by) = (b.line[1][0] - b.line[0][0], b.line[1][1] - b.line[0][1]);
            let blen = bx.hypot(by);
            if blen <= tol || (ux * by - uy * bx).abs() / blen > 0.017_452_406_437_283_512 {
                continue;
            }
            let along = |p: P2| ux * (p[0] - line[0][0]) + uy * (p[1] - line[0][1]);
            let (lo, hi) = (along(b.line[0]).min(along(b.line[1])).max(0.0), along(b.line[0]).max(along(b.line[1])).min(flen));
            if hi - lo <= tol {
                continue;
            }
            let lateral = (ux * (b.line[0][1] - line[0][1]) - uy * (b.line[0][0] - line[0][0])).abs();
            if lateral <= 10.0 && near.map(|(l, _)| lateral < l).unwrap_or(true) {
                near = Some((lateral, b));
            }
        }
        if let Some((lateral, b)) = near {
            out.push((format!("{} off {}", b.path, n.path), format!("{} stands {} off {}, which has no wall on it: the footing was left where it was", called(&b.path, &b.storey), ft_in_said(lateral), n.name)));
        }
    }
    out
}

/// Whether two outlines have any area in common: their edges cross, one has a corner strictly
/// inside the other, or a piece of one (the middle of a triangle of it) is strictly inside the
/// other. Outlines that only touch along an edge, or at a corner, do not.
pub(super) fn rings_overlap(a: &[P2], b: &[P2]) -> bool {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let strictly_in = |p: P2, ring: &[P2]| {
        if geom::point_to_outline(p, ring) <= tol {
            return false;
        }
        let mut inside = false;
        for i in 0..ring.len() {
            let (u, v) = (ring[i], ring[(i + 1) % ring.len()]);
            if (u[1] > p[1]) != (v[1] > p[1]) && p[0] < u[0] + (p[1] - u[1]) / (v[1] - u[1]) * (v[0] - u[0]) {
                inside = !inside;
            }
        }
        inside
    };
    let side = |p: P2, q: P2, r: P2| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    for i in 0..a.len() {
        let (p, q) = (a[i], a[(i + 1) % a.len()]);
        for j in 0..b.len() {
            let (r, t) = (b[j], b[(j + 1) % b.len()]);
            // Crossing, each edge clear of the other's ends by more than the tolerance.
            let (d1, d2, d3, d4) = (side(p, q, r), side(p, q, t), side(r, t, p), side(r, t, q));
            let (lp, lr) = (geom::dist(p, q).max(1.0e-9), geom::dist(r, t).max(1.0e-9));
            if d1 * d2 < 0.0 && d3 * d4 < 0.0 && d1.abs() / lp > tol && d2.abs() / lp > tol && d3.abs() / lr > tol && d4.abs() / lr > tol {
                return true;
            }
        }
    }
    let middles = |ring: &[P2]| -> Vec<P2> { geom::triangles(ring).0.iter().map(|t| [(ring[t[0]][0] + ring[t[1]][0] + ring[t[2]][0]) / 3.0, (ring[t[0]][1] + ring[t[1]][1] + ring[t[2]][1]) / 3.0]).collect() };
    a.iter().any(|p| strictly_in(*p, b)) || b.iter().any(|p| strictly_in(*p, a)) || middles(a).into_iter().any(|p| strictly_in(p, b)) || middles(b).into_iter().any(|p| strictly_in(p, a))
}
