//! Cut from `building.rs`. `floor_outline` is PORTED from BIMCraft (commit f202abd85d1cf165325e2f7b250da0b5e51f5b71, MIT);
//! `PORTMAP.md` says what, and the rest of the file is this project's own.

use super::*;

/// Each room's outline at the faces of the walls round its point, in feet on the plan,
/// counter-clockwise: the same outline its floor area is the area of. A room no walls
/// close round has none and is not in the map.
pub fn room_outlines(job: &mut Job) -> HashMap<String, Vec<P2>> {
    let tree = Tree::of(&mut job.world);
    let mut outlines = HashMap::new();
    room_measures(&mut job.world, &tree, &mut HashMap::new(), &mut outlines);
    outlines
}

/// A wall's face on one room's side: where it is, which way the room lies from it, and
/// how much of it is clear to stand things against (call 155; BIMCraft hosts a thing on a
/// wall by the wall's path and a distance along it, `commands.rs` `host`: here the room's
/// outline at the faces of its walls gives the face and its corners).
#[derive(Clone, Debug, PartialEq)]
pub struct WallFace {
    /// The wall, and the room, as people call them.
    pub wall_called: String,
    pub room_called: String,
    /// The wall's line, first end and second, in feet on the plan.
    pub start: P2,
    pub end: P2,
    /// The way the wall runs, a unit vector, and the way the room lies from the face.
    pub along: P2,
    pub into: P2,
    /// How far the face is from the wall's line.
    pub off: f64,
    /// The stretches of the face along the wall's line, in feet from its first end, with
    /// no corner or cross wall in them, in order.
    pub clear: Vec<(f64, f64)>,
    /// The doors in the wall: path, and where each starts and ends along it.
    pub doors: Vec<(String, f64, f64)>,
    /// The windows in the wall that is to stay: path, what it is called, where it starts and
    /// ends along the wall, and its sill and head in feet above the wall's base (call 157).
    pub windows: Vec<(String, String, f64, f64, f64, f64)>,
    /// As sure as the wall's line and thickness.
    pub status: Status,
}

/// The face of `wall` that `room` has, or why it has none. Both must be on `storey`.
pub fn wall_face(job: &mut Job, storey: &str, wall: &str, room: &str) -> Result<WallFace, String> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    if job.kind(wall) != Some(Kind::Wall) {
        return Err(format!("{wall} is not a wall"));
    }
    if job.kind(room) != Some(Kind::Room) {
        return Err(format!("{room} is not a room"));
    }
    let (wall_called, room_called) = (tree.called(wall), tree.called(room));
    if tree.storey_of(wall) != Some(storey) {
        return Err(format!("{wall_called} ({wall}) is not on {}", tree.called(storey)));
    }
    if tree.storey_of(room) != Some(storey) {
        return Err(format!("{room_called} ({room}) is not on {}", tree.called(storey)));
    }
    let e = job.find(wall).ok_or_else(|| format!("no element {wall}"))?;
    let w = job.world.get::<Wall>(e).cloned().ok_or_else(|| format!("{wall_called} has no wall line entered"))?;
    let (pw, shape, _, _) = plan_wall(&tree, wall, &w).map_err(|why| format!("{wall_called} ({wall}) cannot be placed: {why}"))?;
    let outline = room_outlines(job).remove(room).ok_or_else(|| format!("no walls close round {room_called}'s point, so it has no faces to stand things against"))?;
    let (a, b) = ([pw.start.0, pw.start.1], [pw.end.0, pw.end.1]);
    let length = geom::dist(a, b);
    if length <= tol {
        return Err(format!("{wall_called} has no length"));
    }
    let u = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
    let n = [-u[1], u[0]];
    let reach = pw.ext.abs() + pw.int.abs() + tol;
    // The room's sides that lie along this wall, on one side of its line.
    let mut found: Vec<(f64, f64, f64)> = Vec::new();
    for i in 0..outline.len() {
        let (p, q) = (outline[i], outline[(i + 1) % outline.len()]);
        let (dp, dq) = ((p[0] - a[0]) * n[0] + (p[1] - a[1]) * n[1], (q[0] - a[0]) * n[0] + (q[1] - a[1]) * n[1]);
        if (dp - dq).abs() > tol || dp.abs() > reach || dp.abs() <= tol {
            continue;
        }
        let (sp, sq) = ((p[0] - a[0]) * u[0] + (p[1] - a[1]) * u[1], (q[0] - a[0]) * u[0] + (q[1] - a[1]) * u[1]);
        let (lo, hi) = (sp.min(sq).max(0.0), sp.max(sq).min(length));
        if hi - lo > tol {
            found.push((dp, lo, hi));
        }
    }
    let Some(&(first, _, _)) = found.first() else {
        return Err(format!("{wall_called} ({wall}) is not one of the walls round {room_called}: it has no face on that room's side"));
    };
    // Its first side's side of the wall; any that lie on the other side are left alone.
    let mut sides: Vec<(f64, f64)> = found.iter().filter(|(d, _, _)| (d.signum() - first.signum()).abs() < 0.5).map(|&(_, lo, hi)| (lo, hi)).collect();
    sides.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut clear: Vec<(f64, f64)> = Vec::new();
    for (lo, hi) in sides {
        match clear.last_mut() {
            Some(last) if lo <= last.1 + tol => last.1 = last.1.max(hi),
            _ => clear.push((lo, hi)),
        }
    }
    let gone = crate::components::demolished(&mut job.world);
    let mut doors: Vec<(String, f64, f64)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    let mut mine: Vec<(String, f64, f64, u64)> = Vec::new();
    for (node, kind, o) in q.iter(&job.world) {
        if *kind == Kind::Door && node.parent.as_deref() == Some(wall) && !gone.contains(&node.path) {
            if let Some([from, to, _, _]) = hole_of(&tree, &node.path, o).0.span {
                mine.push((node.path.clone(), from, to, path_number(&node.path)));
            }
        }
    }
    mine.sort_by_key(|m| m.3);
    doors.extend(mine.into_iter().map(|(p, f, t, _)| (p, f, t)));
    let mut windows: Vec<(String, String, f64, f64, f64, f64, u64)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    for (node, kind, o) in q.iter(&job.world) {
        if *kind == Kind::Window && node.parent.as_deref() == Some(wall) && !gone.contains(&node.path) {
            if let Some([from, to, sill, head]) = hole_of(&tree, &node.path, o).0.span {
                windows.push((node.path.clone(), tree.called(&node.path), from, to, sill, head, path_number(&node.path)));
            }
        }
    }
    windows.sort_by_key(|m| m.6);
    let windows = windows.into_iter().map(|(p, c, f, t, s, h, _)| (p, c, f, t, s, h)).collect();
    Ok(WallFace { wall_called, room_called, start: a, end: b, along: u, into: [first.signum() * n[0], first.signum() * n[1]], off: first.abs(), clear, doors, windows, status: shape })
}

/// What is set against a wall's faces and would stand when the wall is gone: each thing
/// set in place (furnishing, equipment, fixture) on the wall's storey that is not itself to
/// demolish, whose plan is along the wall and touches the wall's thickness or a face of it.
/// Path and name, in the order the job holds them (call 157). Nothing is moved or removed.
pub fn things_against(job: &mut Job, wall: &str) -> Result<Vec<(String, String)>, String> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    if job.kind(wall) != Some(Kind::Wall) {
        return Err(format!("{wall} is not a wall"));
    }
    let e = job.find(wall).ok_or_else(|| format!("no element {wall}"))?;
    let w = job.world.get::<Wall>(e).cloned().ok_or_else(|| format!("{wall} has no wall line entered"))?;
    let Ok((pw, _, _, _)) = plan_wall(&tree, wall, &w) else { return Ok(Vec::new()) };
    let storey = tree.storey_of(wall).map(str::to_string);
    let (a, b) = ([pw.start.0, pw.start.1], [pw.end.0, pw.end.1]);
    let length = geom::dist(a, b);
    if length <= tol {
        return Ok(Vec::new());
    }
    let u = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
    let n = [-u[1], u[0]];
    let band = pw.ext.abs().max(pw.int.abs()) + tol;
    let gone = crate::components::demolished(&mut job.world);
    let (drawn, _) = placements(job);
    let mut found = Vec::new();
    for p in drawn {
        let Shape::Box { at, size, angle } = p.shape else { continue };
        if !matches!(p.kind, Kind::Furnishing | Kind::Equipment | Kind::Fixture) || p.storey != storey || gone.contains(&p.path) {
            continue;
        }
        let (sin, cos) = angle.to_radians().sin_cos();
        let (mut s_lo, mut s_hi, mut d_lo, mut d_hi) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let (lx, ly) = (sx * size[0] / 2.0, sy * size[1] / 2.0);
            let (x, y) = (at[0] + lx * cos - ly * sin - a[0], at[1] + lx * sin + ly * cos - a[1]);
            let (s, d) = (x * u[0] + y * u[1], x * n[0] + y * n[1]);
            (s_lo, s_hi, d_lo, d_hi) = (s_lo.min(s), s_hi.max(s), d_lo.min(d), d_hi.max(d));
        }
        if s_hi.min(length) - s_lo.max(0.0) > tol && d_hi >= -band && d_lo <= band {
            found.push((p.path.clone(), p.name.clone()));
        }
    }
    Ok(found)
}

/// The ends of a room line that meet no wall and no other room line of its storey, in feet
/// on the plan: an end meets a wall where it lies in the wall's outline or on its edge, and
/// another room line where it lies on it. Such a line parts nothing until it does (call 157).
pub fn room_line_unreached(job: &mut Job, line: &str) -> Vec<P2> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let Some(storey) = tree.storey_of(line).map(str::to_string) else { return Vec::new() };
    let gone = crate::components::demolished(&mut job.world);
    let mine = job.find(line).and_then(|e| job.world.get::<RoomLine>(e).and_then(|r| r.line_ft()));
    let Some([a, b]) = mine else { return Vec::new() };
    let mut others: Vec<(V, V)> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &RoomLine)>();
    for (n, k, r) in q.iter(&job.world) {
        if *k == Kind::RoomLine && n.path != line && tree.storey_of(&n.path) == Some(storey.as_str()) {
            if let Some([c, d]) = r.line_ft() {
                others.push((V(c[0], c[1]), V(d[0], d[1])));
            }
        }
    }
    let mut pws: Vec<walls::PlanWall> = Vec::new();
    let mut q = job.world.query::<(&Node, &Kind, &Wall)>();
    for (n, k, w) in q.iter(&job.world) {
        if *k == Kind::Wall && !gone.contains(&n.path) && tree.storey_of(&n.path) == Some(storey.as_str()) {
            if let Ok((pw, _, _, _)) = plan_wall(&tree, &n.path, w) {
                pws.push(pw);
            }
        }
    }
    let rings: Vec<Vec<V>> = (0..pws.len()).map(|i| walls::outline(&pws, i, tol)).collect();
    let reaches = |p: V| {
        rings.iter().any(|ring| rooms::contains(ring, p) || ring.iter().zip(ring.iter().cycle().skip(1)).any(|(x, y)| on_line(p, *x, *y, tol))) || others.iter().any(|(x, y)| on_line(p, *x, *y, tol))
    };
    [a, b].into_iter().filter(|p| !reaches(V(p[0], p[1]))).collect()
}

/// The number at the end of a path: paths are numbered in the order elements were added,
/// and a number is never given twice.
pub(super) fn path_number(path: &str) -> u64 {
    path.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
}

/// A wall as the walls round it see it on the plan; the worst status among what its line
/// and its thickness were worked out from; and the worst among what its base and top rest
/// on. Or why it cannot be placed.
pub(crate) fn plan_wall(tree: &Tree, me: &str, w: &Wall) -> Result<(walls::PlanWall, Status, Status, Status), String> {
    use crate::skeleton::P2 as V;
    let [a, b] = w.line_ft().ok_or("its line is not known")?;
    let heights = wall_heights(tree, me, w);
    let base = heights.base.clone()?;
    let height = heights.height.clone()?;
    let (thickness, layers, section_status) = wall_section(tree, w)?;
    let (ext, int) = walls::wall_faces(thickness, &layers, w.location);
    Ok((walls::PlanWall { start: V(a[0], a[1]), end: V(b[0], b[1]), ext, int, base, top: base + height, join: w.join }, w.line.status.max(section_status), heights.span, heights.base_status))
}

/// The outline a floor on a storey has when none is given, and what else the walls
/// enclose there.
#[derive(Clone, Debug, PartialEq)]
pub struct FloorOutline {
    /// The outer outline of the largest piece the storey's walls make, along their
    /// centre lines, counter-clockwise from its south-west corner, in feet.
    pub boundary: Vec<P2>,
    /// Loops of walls standing free inside it, joined to none of the outer walls and
    /// with no room in them: courtyards, which a floor would be cut round.
    pub courtyards: usize,
    /// Other pieces the storey's walls enclose, apart from this one.
    pub others: usize,
    /// The worst status among what the walls' lines and thicknesses rest on.
    pub status: Status,
}

/// The outline `floor` gives a floor on `storey` by default (BIMCraft `floor_outline`,
/// `commands.rs:2467`): along the centre lines of the walls that bound its rooms, the
/// outer outline of the largest piece they make. None when no walls enclose an area.
/// A wall of the storey that cannot be placed is why not.
pub fn floor_outline(job: &mut Job, storey: &str) -> Result<Option<FloorOutline>, String> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let mut q = job.world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Wall && n.parent.as_deref() == Some(storey)).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    let (mut placed, mut bounding, mut status) = (Vec::new(), Vec::new(), Status::Known);
    for (n, w) in all {
        let w = w.ok_or_else(|| format!("{} has no wall line entered, so what the walls of this level enclose is not known", tree.called(&n.path)))?;
        let (wall, shape, _, _) = plan_wall(&tree, &n.path, w).map_err(|why| format!("{} cannot be placed ({why}), so what the walls of this level enclose is not known", tree.called(&n.path)))?;
        if w.room_bounding {
            bounding.push(placed.len());
            status = status.max(shape);
        }
        placed.push(wall);
    }
    let lines: Vec<(V, V)> = bounding.iter().map(|&i| walls::centre_line(&placed, i, tol)).map(|[a, b]| (a, b)).collect();
    // Each piece's outline, clockwise, the largest first.
    let mut pieces: Vec<Vec<V>> = rooms::faces(&lines, tol).into_iter().filter(|f| rooms::area(f) < -tol * tol).collect();
    pieces.sort_by(|a, b| rooms::area(a).total_cmp(&rooms::area(b)));
    let Some((outer, inner)) = pieces.split_first() else { return Ok(None) };
    // The loops standing free inside the outline: the other pieces, inside it.
    let loops: Vec<&Vec<V>> = inner.iter().filter(|f| f.iter().all(|q| rooms::contains(outer, *q))).collect();
    let nested = |a: usize| (0..loops.len()).any(|b| b != a && loops[a].iter().all(|q| rooms::contains(loops[b], *q)));
    let around = |p: V| (0..loops.len()).filter(|&a| rooms::contains(loops[a], p)).max_by(|&a, &b| rooms::area(loops[a]).total_cmp(&rooms::area(loops[b])));
    let mut rq = job.world.query::<(&Node, &Room)>();
    let indoors: Vec<usize> = rq.iter(&job.world).filter(|(n, _)| n.parent.as_deref() == Some(storey)).filter_map(|(_, r)| r.point_ft()).filter_map(|p| around(V(p[0], p[1]))).collect();
    let courtyards = (0..loops.len()).filter(|&a| !nested(a) && !indoors.contains(&a)).count();
    // Counter-clockwise, its corners only, kept to a ten-thousandth of a foot, from its
    // south-west corner.
    let ccw: Vec<V> = outer.iter().rev().copied().collect();
    let mut ring: Vec<P2> = rooms::corners(&ccw).into_iter().map(|p| [(p.0 * 1.0e4).round() / 1.0e4 + 0.0, (p.1 * 1.0e4).round() / 1.0e4 + 0.0]).collect();
    if let Some(first) = ring.iter().enumerate().min_by(|(_, a), (_, b)| (a[1], a[0]).partial_cmp(&(b[1], b[0])).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i) {
        ring.rotate_left(first);
    }
    Ok(Some(FloorOutline { boundary: ring, courtyards, others: inner.len() - loops.len(), status }))
}

/// What a level's walls enclose, for telling an outside face from an inside one: the
/// outline of every separate piece the walls make (a house, and a garage that stands
/// apart from it), and the courtyards inside them (a loop of walls standing free inside
/// a piece, with no room in it). Each ring is in feet. A place is indoors when it is
/// inside a piece and in none of its courtyards. Walls that cannot be placed are why not.
pub struct Enclosed {
    pub pieces: Vec<Vec<P2>>,
    pub courtyards: Vec<Vec<P2>>,
}

impl Enclosed {
    pub fn indoors(&self, p: P2) -> bool {
        let within = |ring: &Vec<P2>| {
            let mut odd = false;
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
                    odd = !odd;
                }
            }
            odd
        };
        // In more pieces than courtyards: a shed standing in a courtyard is indoors again.
        self.pieces.iter().filter(|r| within(r)).count() > self.courtyards.iter().filter(|r| within(r)).count()
    }
}

/// See [`Enclosed`]. The pieces and loops are found as `floor_outline` finds them, and
/// a loop is a courtyard by the same test: it stands free inside a piece, inside no
/// other loop, and no room of the level is in it.
pub fn enclosed(job: &mut Job, storey: &str) -> Result<Enclosed, String> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let mut q = job.world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Wall && n.parent.as_deref() == Some(storey)).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    let (mut placed, mut bounding) = (Vec::new(), Vec::new());
    for (n, w) in all {
        let w = w.ok_or_else(|| format!("{} has no wall line entered", tree.called(&n.path)))?;
        let (wall, _, _, _) = plan_wall(&tree, &n.path, w).map_err(|why| format!("{} cannot be placed ({why})", tree.called(&n.path)))?;
        if w.room_bounding {
            bounding.push(placed.len());
        }
        placed.push(wall);
    }
    let lines: Vec<(V, V)> = bounding.iter().map(|&i| walls::centre_line(&placed, i, tol)).map(|[a, b]| (a, b)).collect();
    let outlines: Vec<Vec<V>> = rooms::faces(&lines, tol).into_iter().filter(|f| rooms::area(f) < -tol * tol).collect();
    // How many other outlines each lies wholly inside: none, it is a piece; one, it is a
    // loop standing free inside a piece.
    let inside_of = |a: usize| (0..outlines.len()).filter(|&b| b != a && outlines[a].iter().all(|q| rooms::contains(&outlines[b], *q))).count();
    let depth: Vec<usize> = (0..outlines.len()).map(inside_of).collect();
    let mut rq = job.world.query::<(&Node, &Room)>();
    let rooms_at: Vec<V> = rq.iter(&job.world).filter(|(n, _)| n.parent.as_deref() == Some(storey)).filter_map(|(_, r)| r.point_ft()).map(|p| V(p[0], p[1])).collect();
    let ring = |f: &Vec<V>| -> Vec<P2> { f.iter().map(|p| [p.0, p.1]).collect() };
    let mut out = Enclosed { pieces: Vec::new(), courtyards: Vec::new() };
    for (i, f) in outlines.iter().enumerate() {
        // A loop standing free inside a piece with no room in it is a courtyard, as the
        // floor's outline takes it; what stands free inside a courtyard is a piece again.
        let roomless = !rooms_at.iter().any(|p| rooms::contains(f, *p));
        match depth[i] {
            0 => out.pieces.push(ring(f)),
            1 if roomless => out.courtyards.push(ring(f)),
            2 if outlines.iter().enumerate().any(|(j, o)| j != i && depth[j] == 1 && f.iter().all(|q| rooms::contains(o, *q)) && !rooms_at.iter().any(|p| rooms::contains(o, *p))) => out.pieces.push(ring(f)),
            _ => {}
        }
    }
    Ok(out)
}
