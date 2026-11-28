use super::*;

/// A wall of the job as the rooms see it.
pub(super) struct RoomWall {
    pub(super) path: String,
    pub(super) wall: walls::PlanWall,
    /// What its line and thickness rest on.
    pub(super) shape: Status,
    /// What its base and top rest on.
    pub(super) span: Status,
    /// What its base alone rests on.
    pub(super) base: Status,
    /// Whether it bounds rooms (`room_bounding`).
    pub(super) bounds: bool,
}

/// What one room comes to against one set of walls: the floor inside the faces and the
/// distance round, and the area to centre lines, each with the walls it is measured to;
/// or why not.
#[derive(Clone, PartialEq)]
pub(super) struct RoomFigures {
    pub(super) finish: Result<(f64, f64, Vec<usize>), rooms::Unbound>,
    pub(super) centre: Result<(f64, Vec<usize>), rooms::Unbound>,
    pub(super) in_wall: bool,
    /// The corners of the room at its walls' faces, when it has any: where it is.
    pub(super) corners: Vec<crate::skeleton::P2>,
    /// The room's outline at the faces of its walls, counter-clockwise, without what
    /// stands free inside it; empty when no walls close round its point.
    pub(super) outline: Vec<crate::skeleton::P2>,
    /// What stands free inside it, at its walls' faces: a room within the room, a
    /// chase. Its sides are the room's walls too (judged 2026-11-19).
    pub(super) islands: Vec<Vec<crate::skeleton::P2>>,
    /// The room lines (by their place in the list given) with a side of the room along
    /// them: the room is measured to them (call 152).
    pub(super) along: Vec<usize>,
}

/// Every room of one storey measured against `walls`, of which those at `bounding` bound
/// the storey's rooms, and `room_lines`, which part them where no wall stands. Each of `walls` can join the others where they meet (BIMCraft
/// `rooms.rs:62` and `walls.rs:238`: a bounding wall's outline is cut against every wall
/// it meets, bounding or not).
pub(super) fn rooms_among(walls: &[walls::PlanWall], bounding: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], points: &[crate::skeleton::P2], tol: f64) -> Vec<RoomFigures> {
    // The rooms of a set of walls are a function of the walls, the room lines and the points alone, and the same set is asked for again
    // by every figure worked out after an edit that moved none of them: a model brought in whole made two hundred edits and worked
    // its ten rooms out two hundred times, three quarters of the twenty seconds it took (call 204, point 7). So what was worked out
    // is kept, by what it was worked out from (every number as it is written, so equal only where identical), and handed back as it
    // was. Nothing is kept past the run of the program, and nothing about the answer changes.
    thread_local! { static KEPT: std::cell::RefCell<HashMap<String, Vec<RoomFigures>>> = std::cell::RefCell::new(HashMap::new()); }
    let key = format!("{walls:?}|{bounding:?}|{room_lines:?}|{points:?}|{tol:?}");
    if let Some(found) = KEPT.with(|k| k.borrow().get(&key).cloned()) {
        return found;
    }
    let worked = rooms_among_worked(walls, bounding, room_lines, points, tol);
    KEPT.with(|k| {
        let mut k = k.borrow_mut();
        if k.len() >= 256 {
            k.clear();
        }
        k.insert(key, worked.clone());
    });
    worked
}

pub(super) fn rooms_among_worked(walls: &[walls::PlanWall], bounding: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], points: &[crate::skeleton::P2], tol: f64) -> Vec<RoomFigures> {
    use crate::skeleton::P2 as V;
    use rooms::{Plan, RoomArea};
    let (centre, finish) = (Plan::of_walls(walls, bounding, room_lines, RoomArea::Centre, tol), Plan::of_walls(walls, bounding, room_lines, RoomArea::Finish, tol));
    let centre_lines: Vec<Vec<(V, V)>> = bounding.iter().map(|&i| walls::centre_line(walls, i, tol)).map(|[a, b]| vec![(a, b)]).collect();
    let outlines: Vec<Vec<V>> = bounding.iter().map(|&i| walls::outline(walls, i, tol)).collect();
    let outline_edges: Vec<Vec<(V, V)>> = outlines.iter().map(|ring| ring.iter().copied().zip(ring.iter().copied().cycle().skip(1)).collect()).collect();
    // The walls a region is measured to: those with a line along one of its edges.
    let touching = |g: &rooms::Region, lines: &[Vec<(V, V)>]| -> Vec<usize> {
        let mids: Vec<V> = std::iter::once(&g.outer).chain(g.holes.iter()).flat_map(|ring| ring.iter().zip(ring.iter().cycle().skip(1)).map(|(a, b)| (*a + *b) * 0.5)).collect();
        (0..bounding.len())
            .filter(|&k| {
                lines[k].iter().any(|(a, b)| {
                    mids.iter().any(|m| {
                        let d = *b - *a;
                        let t = if d.dot(d) > 0.0 { ((*m - *a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
                        m.dist(*a + d * t) <= tol
                    })
                })
            })
            .map(|k| bounding[k])
            .collect()
    };
    // The room lines a region has a side along.
    let along = |g: &rooms::Region| -> Vec<usize> {
        let mids: Vec<V> = std::iter::once(&g.outer).chain(g.holes.iter()).flat_map(|ring| ring.iter().zip(ring.iter().cycle().skip(1)).map(|(a, b)| (*a + *b) * 0.5)).collect();
        (0..room_lines.len()).filter(|&k| mids.iter().any(|m| on_line(*m, room_lines[k].0, room_lines[k].1, tol))).collect()
    };
    (0..points.len())
        .map(|i| RoomFigures {
            along: finish.bound(points, i, tol).map(|g| along(&g)).unwrap_or_default(),
            corners: finish.bound(points, i, tol).map(|g| std::iter::once(&g.outer).chain(g.holes.iter()).flatten().copied().collect()).unwrap_or_default(),
            outline: finish.bound(points, i, tol).map(|g| g.outer).unwrap_or_default(),
            islands: finish.bound(points, i, tol).map(|g| g.holes).unwrap_or_default(),
            finish: finish.bound(points, i, tol).map(|g| (g.area(), g.perimeter(), touching(&g, &outline_edges))),
            centre: centre.bound(points, i, tol).map(|g| (g.area(), touching(&g, &centre_lines))),
            in_wall: !points[i].0.is_nan() && outlines.iter().any(|ring| rooms::contains(ring, points[i])),
        })
        .collect()
}

/// Whether `m` lies on the line from `a` to `b`, to within `tol`.
pub(super) fn on_line(m: crate::skeleton::P2, a: crate::skeleton::P2, b: crate::skeleton::P2, tol: f64) -> bool {
    let d = b - a;
    let t = if d.dot(d) > 0.0 { ((m - a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
    m.dist(a + d * t) <= tol
}

/// A room's wall area (call 144): each side of its outline at its walls' faces, as long as
/// it is and as tall as the wall it lies against, less the part of each door and window
/// in that wall that lies along the side. A door between two rooms is taken off both:
/// both its faces are cut. A side along a room line is no wall and takes nothing. With the
/// statuses and inputs of the openings it took off.
pub(super) fn room_wall_area(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, tol: f64) -> Result<(f64, Vec<Status>, Vec<String>), String> {
    use crate::skeleton::P2 as V;
    if outline.len() < 3 {
        return Err("no loop of walls closes round its point".into());
    }
    // An opening that does not fit its wall (past an end, over its top) leaves the wall
    // with no net area, and the room is no surer of what it takes off (judged
    // 2026-11-19: the room's figure stood Known beside the wall's Unknown).
    for &k in mine {
        let (w, length) = (&walls[k], walls[k].start.dist(walls[k].end));
        for hole in holes.get(&placed[k].path).map(Vec::as_slice).unwrap_or(&[]) {
            if let Some([from, to, _, head]) = hole.span {
                if from < -tol || to > length + tol || head > w.top - w.base + tol {
                    return Err(format!("{} does not fit its wall, so what it takes off this room's walls is not known", hole.path));
                }
            }
        }
    }
    let rings: Vec<(usize, Vec<(V, V)>)> = mine.iter().map(|&k| {
        let ring = walls::outline(walls, k, tol);
        let edges = ring.iter().copied().zip(ring.iter().copied().cycle().skip(1)).collect();
        (k, edges)
    }).collect();
    let on = |m: V, a: V, b: V| {
        let d = b - a;
        let t = if d.dot(d) > 0.0 { ((m - a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
        m.dist(a + d * t) <= tol
    };
    let (mut total, mut statuses, mut cited) = (0.0, Vec::new(), Vec::new());
    let sides: Vec<(V, V)> = std::iter::once(outline).chain(islands.iter().map(Vec::as_slice)).flat_map(|ring| (0..ring.len()).map(move |i| (ring[i], ring[(i + 1) % ring.len()]))).collect();
    for (p, q) in sides {
        let length = p.dist(q);
        if length <= tol {
            continue;
        }
        let mid = (p + q) * 0.5;
        let Some(&(k, _)) = rings.iter().find(|(_, edges)| edges.iter().any(|(a, b)| on(mid, *a, *b))) else {
            // A side along a room line has no wall: it takes no wall area (call 152).
            if room_lines.iter().any(|(a, b)| on(mid, *a, *b)) {
                continue;
            }
            return Err(format!("the side of the room from ({}, {}) to ({}, {}) lies along no wall's face", view::num(p.0), view::num(p.1), view::num(q.0), view::num(q.1)));
        };
        let w = &walls[k];
        let tall = w.top - w.base;
        total += length * tall;
        let (s1, s2) = (w.project(p).0, w.project(q).0);
        let (lo, hi) = (s1.min(s2), s1.max(s2));
        for hole in holes.get(&placed[k].path).map(Vec::as_slice).unwrap_or(&[]) {
            let Some([from, to, sill, head]) = hole.span else {
                return Err(format!("where {} is in its wall is not known, so what it takes off this room's walls is not", hole.path));
            };
            let along = to.min(hi) - from.max(lo);
            if along > tol {
                total -= along * (head.min(tall) - sill.max(0.0)).max(0.0);
                statuses.push(hole.status);
                cited.extend(hole.inputs.iter().cloned());
            }
        }
    }
    Ok((total, statuses, cited))
}

/// The runs of wall along a room's floor line (calls 147 and 157): each ring of it (round
/// the room, and round what stands inside it) cut at its doors into runs; a side along a
/// room line cuts it as a door does (call 152); a ring with no door is one run all the way
/// round. A door to demolish is filled in and is no doorway. The length of each run, with
/// the statuses and inputs of the doors that cut it. Receptacle outlets (`room_receptacles`)
/// and the room's `wall_run` are both worked from these: one walk of the floor line.
#[allow(clippy::too_many_arguments)]
pub(super) fn room_runs(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, against: &[&Standing], tol: f64) -> Result<(Vec<f64>, Vec<Status>, Vec<String>, Vec<usize>), String> {
    use crate::skeleton::P2 as V;
    // Which of `against` cover some of the floor line (call 162).
    let mut touched: Vec<usize> = Vec::new();
    let rings: Vec<(usize, Vec<(V, V)>)> = mine.iter().map(|&k| {
        let ring = walls::outline(walls, k, tol);
        let edges = ring.iter().copied().zip(ring.iter().copied().cycle().skip(1)).collect();
        (k, edges)
    }).collect();
    let on = |m: V, a: V, b: V| {
        let d = b - a;
        let t = if d.dot(d) > 0.0 { ((m - a).dot(d) / d.dot(d)).clamp(0.0, 1.0) } else { 0.0 };
        m.dist(a + d * t) <= tol
    };
    let (mut runs, mut statuses, mut cited) = (Vec::new(), Vec::new(), Vec::new());
    for ring in std::iter::once(outline).chain(islands.iter().map(Vec::as_slice)) {
        // The doorways along this ring, as stretches of its length from its first corner.
        let mut round = 0.0;
        let mut cuts: Vec<(f64, f64)> = Vec::new();
        for i in 0..ring.len() {
            let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
            let length = p.dist(q);
            if length > tol {
                let mid = (p + q) * 0.5;
                if let Some(&(k, _)) = rings.iter().find(|(_, edges)| edges.iter().any(|(a, b)| on(mid, *a, *b))) {
                    let w = &walls[k];
                    let (s1, s2) = (w.project(p).0, w.project(q).0);
                    for hole in holes.get(&placed[k].path).map(Vec::as_slice).unwrap_or(&[]) {
                        if kinds.get(&hole.path) != Some(&Kind::Door) {
                            continue;
                        }
                        let Some([from, to, _, _]) = hole.span else {
                            return Err(format!("where {} is in its wall is not known, so where it cuts this room's floor line is not", hole.path));
                        };
                        let (a, b) = (from.max(s1.min(s2)), to.min(s1.max(s2)));
                        if b - a > tol {
                            let (x, y) = ((a - s1).abs(), (b - s1).abs());
                            cuts.push((round + x.min(y), round + x.max(y)));
                            statuses.push(hole.status);
                            cited.extend(hole.inputs.iter().cloned());
                        }
                    }
                    // What stands on the floor against this side covers it, by the length
                    // of the side its plan lies along (call 162): a cut, as a door is.
                    if !against.is_empty() && length > tol {
                        let u = (q - p) * (1.0 / length);
                        let mut n = V(-u.1, u.0);
                        let into_room = |at: V| rooms::contains(outline, at) && !islands.iter().any(|i| rooms::contains(i, at));
                        if !into_room(mid + n * 0.05) && into_room(mid - n * 0.05) {
                            n = V(-n.0, -n.1);
                        }
                        for (i, thing) in against.iter().enumerate() {
                            let (mut s_lo, mut s_hi, mut d_lo, mut d_hi) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
                            for c in thing.corners {
                                let (s, d) = ((c - p).dot(u), (c - p).dot(n));
                                (s_lo, s_hi, d_lo, d_hi) = (s_lo.min(s), s_hi.max(s), d_lo.min(d), d_hi.max(d));
                            }
                            let (lo, hi) = (s_lo.max(0.0), s_hi.min(length));
                            // Its plan lies against the face (within the fit tolerance) and
                            // reaches into the room.
                            if d_lo <= tol && d_hi > tol && hi - lo > tol {
                                cuts.push((round + lo, round + hi));
                                statuses.push(thing.status);
                                cited.push(format!("{}#{K_PLACED}", thing.path));
                                if !touched.contains(&i) {
                                    touched.push(i);
                                }
                            }
                        }
                    }
                } else if room_lines.iter().any(|(a, b)| on(mid, *a, *b)) {
                    // A side along a room line is no wall space, and cuts the floor line as a doorway does (call 152).
                    cuts.push((round, round + length));
                }
            }
            round += length;
        }
        if round <= tol {
            continue;
        }
        if cuts.is_empty() {
            runs.push(round);
            continue;
        }
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Doorways that touch or overlap are one.
        let mut merged: Vec<(f64, f64)> = Vec::new();
        for c in cuts {
            match merged.last_mut() {
                Some(last) if c.0 <= last.1 + tol => last.1 = last.1.max(c.1),
                _ => merged.push(c),
            }
        }
        // The runs between them, the last running on past the first corner to the first.
        for i in 0..merged.len() {
            let start = merged[i].1;
            let end = if i + 1 < merged.len() { merged[i + 1].0 } else { merged[0].0 + round };
            runs.push(end - start);
        }
    }
    Ok((runs, statuses, cited, touched))
}

/// What a countertop leaves out where a box under it is deeper than the top is (call 162): a
/// box that turns a corner (36 in on each wall) is taken as the top's depth, and the leg that
/// turns the corner, (the box's depth less the top's) by the top's depth, is not in the
/// area. Said, not guessed: the model holds a top as a rectangle. One sentence for each such
/// box under the top. The boxes are those on the same storey standing on the floor no taller
/// than a base box, square to the top, in its length and with their backs on its wall.
pub(super) fn counter_legs_said(me: &str, top: &Placed, size: [f64; 3], things: &[(String, String, Option<String>, Placed)]) -> Vec<String> {
    let tol = crate::check::FIT_TOLERANCE_FT;
    let (Some(at), Some(angle)) = (top.at_ft(), top.angle_deg()) else { return Vec::new() };
    let (long, deep) = (size[0], size[1]);
    let (sin, cos) = angle.to_radians().sin_cos();
    let storey = things.iter().find(|t| t.0 == me).and_then(|t| t.2.clone());
    let mut out = Vec::new();
    for (path, name, on, b) in things {
        let (Some(bat), Some([bw, bd, bh]), Some(bangle), Some(up)) = (b.at_ft(), b.size_ft(), b.angle_deg(), b.offset.feet()) else { continue };
        if path == me || *on != storey || up > tol || bh > 34.5 / 12.0 + tol || b.what.value.as_deref().is_some_and(|w| w.eq_ignore_ascii_case("Countertop")) {
            continue;
        }
        let (dx, dy) = (bat[0] - at[0], bat[1] - at[1]);
        let (lx, ly) = (dx * cos + dy * sin, -dx * sin + dy * cos);
        let turned = (bangle - angle).to_radians();
        let (w, d) = if turned.sin().abs() < 1.0e-6 {
            (bw, bd)
        } else if turned.cos().abs() < 1.0e-6 {
            (bd, bw)
        } else {
            continue;
        };
        let in_length = lx - w / 2.0 >= -long / 2.0 - tol && lx + w / 2.0 <= long / 2.0 + tol;
        let back_on_its_wall = ((ly + d / 2.0) - deep / 2.0).abs() <= tol;
        if in_length && back_on_its_wall && d > deep + tol {
            let inches = |feet: f64| units::show_length(&units::Len::One((feet * 12.0 * 64.0).round() / 64.0), units::IN);
            out.push(format!("the leg that turns the corner over {name}, {} by {} ({:.2} SF), is not in this area: add it where the top is priced by area.", inches(d - deep), inches(deep), (d - deep) * deep));
        }
    }
    out
}

/// A thing set in place that stands on the floor, by its plan: its four corners on the
/// plan, in feet, and how sure they are (call 162).
pub(super) struct Standing {
    pub(super) path: String,
    pub(super) corners: [crate::skeleton::P2; 4],
    pub(super) status: Status,
}

/// The fewest receptacle outlets a room's walls need (call 147, NEC 210.52(A)): each run of
/// its floor line (`room_runs`) of 2 ft or more takes one outlet for each 12 ft or part, a
/// shorter run none. With the statuses and inputs of the doors that cut it.
#[allow(clippy::too_many_arguments)]
pub(super) fn room_receptacles(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, tol: f64) -> Result<(f64, Vec<Status>, Vec<String>), String> {
    // How many outlets a run of this length needs: a run under 2 ft is no wall space.
    let need = |run: f64| if run < 2.0 - tol { 0.0 } else { (run / 12.0 - 1.0e-9).ceil().max(1.0) };
    let (runs, statuses, cited, _) = room_runs(outline, islands, mine, room_lines, walls, placed, holes, kinds, &[], tol)?;
    Ok((runs.into_iter().map(need).sum(), statuses, cited))
}

/// A room's run of wall (call 157): the length of its floor line along walls, each side
/// that lies along a wall less the doorways in it, nothing for a side along a room line,
/// islands standing in the room with it. The sum of `room_runs`.
#[allow(clippy::too_many_arguments)]
pub(super) fn room_wall_run(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, tol: f64) -> Result<(f64, Vec<Status>, Vec<String>), String> {
    let (runs, statuses, cited, _) = room_runs(outline, islands, mine, room_lines, walls, placed, holes, kinds, &[], tol)?;
    Ok((runs.into_iter().sum(), statuses, cited))
}

/// A room's run of base (call 162): its run of wall less the length of its walls' faces that
/// things set in place cover at the floor. A thing covers a face where its plan lies against
/// it and it stands on the floor; what it covers is the length of the face its plan lies
/// along, two things over the same length counting it once. The figure, the statuses and
/// inputs of what it rests on, and how many things covered something.
#[allow(clippy::too_many_arguments)]
pub(super) fn room_base_run(outline: &[crate::skeleton::P2], islands: &[Vec<crate::skeleton::P2>], mine: &[usize], room_lines: &[(crate::skeleton::P2, crate::skeleton::P2)], walls: &[walls::PlanWall], placed: &[RoomWall], holes: &HashMap<String, Vec<Hole>>, kinds: &HashMap<String, Kind>, against: &[&Standing], tol: f64) -> Result<(f64, Vec<Status>, Vec<String>, usize), String> {
    let (runs, statuses, cited, touched) = room_runs(outline, islands, mine, room_lines, walls, placed, holes, kinds, against, tol)?;
    Ok((runs.into_iter().sum(), statuses, cited, touched.len()))
}

/// Each room's figures: the floor inside the faces of the walls round its point, the area
/// to those walls' centre lines, and how far it is round the room at their faces.
///
/// A room is found among the walls that stand at its storey's computation height
/// (BIMCraft `Model::bounding`, `model.rs:3932`, and `Model::spans`, `model.rs:3960`) and
/// bound rooms. A wall under the room's own storey that cannot be placed leaves every
/// room on that storey with no figure: what bounds them is not known. A figure is no
/// surer than the walls it is measured to. And where a wall's base or top rests on
/// something Assumed, each room is measured again with that wall taken away and with it
/// standing full height: a room that comes out differently either way is no surer than
/// that wall's heights, and says so.
pub(super) fn room_measures(world: &mut World, tree: &Tree, out: &mut HashMap<String, BTreeMap<String, Derived>>, outlines: &mut HashMap<String, Vec<P2>>) {
    use crate::skeleton::P2 as V;
    use rooms::Unbound;
    let tol = crate::check::FIT_TOLERANCE_FT;
    // Every wall of the job: placed, or why not. A wall with nothing entered on it is a
    // wall that cannot be placed, not a wall that is not there.
    // A wall to be demolished bounds no room: rooms are the rooms the work leaves.
    let gone = crate::components::demolished(world);
    let mut placed: Vec<RoomWall> = Vec::new();
    let mut unplaced: Vec<(String, String)> = Vec::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Wall>)>();
    let mut all: Vec<(&Node, Option<&Wall>)> = q.iter(world).filter(|(_, k, _)| **k == Kind::Wall).map(|(n, _, w)| (n, w)).collect();
    all.sort_by_key(|(n, _)| path_number(&n.path));
    for (n, w) in all {
        match w.map(|w| (w, plan_wall(tree, &n.path, w))) {
            None => unplaced.push((n.path.clone(), "it has no wall line entered".into())),
            Some((w, Ok((mut wall, shape, span, base)))) => {
                // Nor do the walls it met end on it: they meet each other where it stood.
                let stands = !gone.contains(&n.path);
                wall.join = wall.join && stands;
                placed.push(RoomWall { path: n.path.clone(), wall, shape, span, base, bounds: w.room_bounding && stands });
            }
            Some((_, Err(why))) => unplaced.push((n.path.clone(), why)),
        }
    }
    // Every room line (call 152): its storey, where it lies, or why it cannot be placed.
    // Like a wall's, a room line with no figure leaves its storey's rooms with none.
    struct Parting {
        path: String,
        line: Option<[crate::skeleton::P2; 2]>,
        status: Status,
        why: String,
    }
    let mut partings: BTreeMap<String, Vec<Parting>> = BTreeMap::new();
    {
        let mut q = world.query::<(&Node, &Kind, Option<&RoomLine>)>();
        let mut all: Vec<(&Node, Option<&RoomLine>)> = q.iter(world).filter(|(_, k, _)| **k == Kind::RoomLine).map(|(n, _, r)| (n, r)).collect();
        all.sort_by_key(|(n, _)| path_number(&n.path));
        for (n, r) in all {
            let (line, status, why) = match r {
                None => (None, Status::Unknown, "it has no line entered".to_string()),
                Some(r) => match (r.line.value.is_some(), r.line_ft()) {
                    (_, Some([a, b])) => (Some([crate::skeleton::P2(a[0], a[1]), crate::skeleton::P2(b[0], b[1])]), r.line.status, String::new()),
                    (true, None) => (None, r.line.status, "its line is in a unit this program does not convert".to_string()),
                    (false, None) => (None, r.line.status, "its line is not known".to_string()),
                },
            };
            partings.entry(n.parent.clone().unwrap_or_default()).or_default().push(Parting { path: n.path.clone(), line, status, why });
        }
    }
    // The rooms, storey by storey, in the order they were added: the first to be placed
    // in a space has it.
    let mut q = world.query::<(&Node, &Kind, Option<&Room>)>();
    let mut by_storey: BTreeMap<String, Vec<(String, Option<Room>)>> = BTreeMap::new();
    for (n, kind, room) in q.iter(world) {
        if *kind == Kind::Room {
            by_storey.entry(n.parent.clone().unwrap_or_default()).or_default().push((n.path.clone(), room.cloned()));
        }
    }
    const F_FLOOR: &str = "the floor inside the faces of the walls round its point";
    const F_GROSS: &str = "the area to the center lines of the walls round its point";
    const F_ROUND: &str = "the distance round the room at the faces of its walls";
    const F_OUTLETS: &str = "round the room at its walls' faces, its floor line cut at each door into runs of wall; each run of 2 ft or more, one receptacle outlet for every 12 ft or part (no point more than 6 ft from one: NEC 2023 210.52(A)(1) and (A)(2), as Mike Holt's Understanding the 2023 NEC gives them); a room with no door is one run all the way round";
    const OUTLETS_NOTE: &str = "for the rooms NEC 210.52(A) covers (kitchen, family room, dining room, living room, sunroom, parlor, library, den, bedroom, recreation room and the like): fireplaces, stationary appliances and fixed cabinets also break wall space and are not drawn; a bathroom, a hallway, a foyer and a kitchen's counters have rules of their own (210.52(D), (H), (I), (C)), not worked here";
    const F_RUN: &str = "the room's floor line along its walls: each side of the room at its walls' faces that lies along a wall, less the doorways in it (a door to demolish is filled in and is no doorway), nothing for a side along a room line; what stands inside the room is taken with it";
    const F_BASE: &str = "the room's wall run, less the length of its walls' faces that things set in place cover at the floor (what is hung on a wall covers nothing, nor does what is to demolish; two things over the same length count it once)";
    const F_WALLS: &str = "each side of the room at its walls' faces × the height of the wall it is against, less the part of each door and window in that wall along the side";
    // Each wall's doors and windows, for the room's wall area (call 144).
    let holes_of: HashMap<String, Vec<Hole>> = {
        let mut q = world.query::<(&Node, &Kind, &Opening)>();
        let mut all: Vec<(String, Hole)> = q.iter(world).filter(|(n, k, _)| k.is_opening() && !gone.contains(&n.path)).filter_map(|(n, _, o)| n.parent.clone().map(|p| (p, hole_of(tree, &n.path, o).0))).collect();
        all.sort_by_key(|(_, h)| path_number(&h.path));
        let mut map: HashMap<String, Vec<Hole>> = HashMap::new();
        for (p, h) in all {
            map.entry(p).or_default().push(h);
        }
        map
    };
    // What is set in place and stands on the floor, by its plan: it covers the base of the
    // wall it is against (call 162). What is hung (up from the floor), what is to demolish
    // and what has no plan covers nothing.
    let standing: Vec<(Option<String>, Standing)> = {
        let mut q = world.query::<(&Node, &Kind, &Placed)>();
        let mut all: Vec<(u64, Option<String>, Standing)> = q
            .iter(world)
            .filter(|(n, k, _)| matches!(**k, Kind::Furnishing | Kind::Equipment | Kind::Fixture) && !gone.contains(&n.path))
            .filter_map(|(n, _, p)| {
                if p.offset.feet()? > tol {
                    return None;
                }
                let (at, size) = (p.at_ft()?, p.size_ft()?);
                let (sin, cos) = p.angle_deg()?.to_radians().sin_cos();
                let corner = |sx: f64, sy: f64| {
                    let (lx, ly) = (sx * size[0] / 2.0, sy * size[1] / 2.0);
                    crate::skeleton::P2(at[0] + lx * cos - ly * sin, at[1] + lx * sin + ly * cos)
                };
                let status = [p.at.status, p.size.status, p.offset.status, p.angle.as_ref().map(|a| a.status).unwrap_or(Status::Known)].into_iter().max().unwrap_or(Status::Known);
                Some((path_number(&n.path), tree.storey_of(&n.path).map(str::to_string), Standing { path: n.path.clone(), corners: [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)], status }))
            })
            .collect();
        all.sort_by_key(|(n, _, _)| *n);
        all.into_iter().map(|(_, storey, s)| (storey, s)).collect()
    };
    for (storey, mut list) in by_storey {
        list.sort_by_key(|(path, _)| path_number(path));
        let refuse_all = |out: &mut HashMap<String, BTreeMap<String, Derived>>, path: &str, inputs: &[String], why: &str| {
            let m = out.entry(path.to_string()).or_default();
            m.insert("floor_area".into(), Derived::refused("SF", F_FLOOR, inputs, why));
            m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, inputs, why));
            m.insert("perimeter".into(), Derived::refused("LF", F_ROUND, inputs, why));
            m.insert("wall_area".into(), Derived::refused("SF", F_WALLS, inputs, why));
            m.insert("wall_run".into(), Derived::refused("LF", F_RUN, inputs, why));
            m.insert("base_run".into(), Derived::refused("LF", F_BASE, inputs, why));
            m.insert("receptacles".into(), Derived::refused("EA", F_OUTLETS, inputs, why));
        };
        // Where the storey's rooms find their walls.
        let level = tree.levels.get(&storey);
        let cut: Result<(f64, Status), String> = match level {
            None => Err("it is under no level with a level".into()),
            Some(l) => match (l.elevation.feet(), &l.computation_height) {
                (None, _) => Err("its level's elevation is not known".into()),
                (Some(e), None) => Ok((e + COMPUTATION_HEIGHT_FT, l.elevation.status)),
                (Some(e), Some(c)) => c.feet().map(|up| (e + up, l.elevation.status.max(c.status))).ok_or_else(|| "the height its level's rooms find their walls at is not known".to_string()),
            },
        };
        let parting = partings.get(&storey).map(Vec::as_slice).unwrap_or(&[]);
        if let Some(p) = parting.iter().find(|p| p.line.is_none()) {
            for (path, _) in &list {
                refuse_all(out, path, &[format!("{path}#{K_ROOM}"), format!("{}#{K_ROOM_LINE}", p.path)], &format!("{} cannot be placed ({}), so what bounds the rooms of this level is not known", tree.called(&p.path), p.why));
            }
            continue;
        }
        let room_lines: Vec<(V, V)> = parting.iter().filter_map(|p| p.line).map(|[a, b]| (a, b)).collect();
        let stuck = unplaced.iter().find(|(path, _)| tree.storey_of(path) == Some(storey.as_str()));
        let (cut, cut_status) = match (cut, stuck) {
            (Err(why), _) => {
                for (path, _) in &list {
                    refuse_all(out, path, &[format!("{path}#{K_ROOM}")], &why);
                }
                continue;
            }
            (Ok(_), Some((wall, why))) => {
                for (path, _) in &list {
                    refuse_all(out, path, &[format!("{path}#{K_ROOM}"), format!("{wall}#{K_WALL}")], &format!("{} cannot be placed ({why}), so what bounds the rooms of this level is not known", tree.called(wall)));
                }
                continue;
            }
            (Ok(c), None) => c,
        };
        // A room with no point entered, or a point nobody has, still takes its place in
        // the order: it is given a point no region can hold.
        let nowhere = V(f64::NAN, f64::NAN);
        let points: Vec<V> = list.iter().map(|(_, r)| r.as_ref().and_then(|r| r.point_ft()).map(|p| V(p[0], p[1])).unwrap_or(nowhere)).collect();
        // The walls as they stand, and which of them bound this storey's rooms: those that
        // bound rooms and span the height the rooms are found at.
        let plan_walls: Vec<walls::PlanWall> = placed.iter().map(|w| w.wall.clone()).collect();
        let spans = |w: &walls::PlanWall| w.base <= cut && cut < w.top;
        let bounding: Vec<usize> = (0..placed.len()).filter(|&i| placed[i].bounds && spans(&placed[i].wall)).collect();
        let as_entered = rooms_among(&plan_walls, &bounding, &room_lines, &points, tol);
        for (i, (path, _)) in list.iter().enumerate() {
            if as_entered[i].outline.len() >= 3 {
                outlines.insert(path.clone(), as_entered[i].outline.iter().map(|p| [p.0, p.1]).collect());
            }
        }
        // Each wall whose base or top rests on something less than Known, tried both other
        // ways: not there at all, and standing through every height. A room that comes out
        // differently is no surer than that wall's heights.
        let mut rests_on: Vec<Vec<usize>> = vec![Vec::new(); list.len()];
        for k in (0..placed.len()).filter(|&k| placed[k].span != Status::Known) {
            let without: Vec<walls::PlanWall> = plan_walls.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, w)| w.clone()).collect();
            let shifted: Vec<usize> = bounding.iter().filter(|&&i| i != k).map(|&i| if i > k { i - 1 } else { i }).collect();
            let gone = rooms_among(&without, &shifted, &room_lines, &points, tol);
            // Standing as tall as it could: its top anywhere above, and its base anywhere
            // below only if its base is itself unsure. A wall that starts at a Known
            // height does not reach below it whatever its top turns out to be.
            let mut tall = plan_walls.clone();
            if placed[k].base != Status::Known {
                tall[k].base = f64::NEG_INFINITY;
            }
            tall[k].top = f64::INFINITY;
            let mut with: Vec<usize> = bounding.clone();
            if placed[k].bounds && spans(&tall[k]) && !with.contains(&k) {
                with.push(k);
                with.sort_unstable();
            }
            let standing = rooms_among(&tall, &with, &room_lines, &points, tol);
            // Compared by what a room comes to, not by which walls are listed (taking a
            // wall away renumbers the rest).
            let same = |a: &RoomFigures, b: &RoomFigures| {
                let near = |x: f64, y: f64| (x - y).abs() <= 1.0e-9;
                let finish = match (&a.finish, &b.finish) {
                    (Ok(x), Ok(y)) => near(x.0, y.0) && near(x.1, y.1),
                    (Err(x), Err(y)) => std::mem::discriminant(x) == std::mem::discriminant(y),
                    _ => false,
                };
                let centre = match (&a.centre, &b.centre) {
                    (Ok(x), Ok(y)) => near(x.0, y.0),
                    (Err(x), Err(y)) => std::mem::discriminant(x) == std::mem::discriminant(y),
                    _ => false,
                };
                finish && centre && a.in_wall == b.in_wall
            };
            for i in 0..list.len() {
                if !same(&as_entered[i], &gone[i]) || !same(&as_entered[i], &standing[i]) {
                    rests_on[i].push(k);
                }
            }
        }
        for (i, (path, room)) in list.iter().enumerate() {
            let input = format!("{path}#{K_ROOM}");
            let Some(room) = room else {
                refuse_all(out, path, &[input], "no point entered");
                continue;
            };
            if points[i].0.is_nan() {
                refuse_all(out, path, &[input], if room.point.value.is_some() { "its point is in a unit this program does not convert" } else { "its point is not known" });
                continue;
            }
            let found = &as_entered[i];
            let why = |u: Unbound| match u {
                // A point in the body of a wall is in no room, at its faces or at its
                // centre line: which side of the wall was meant is not something to guess.
                _ if found.in_wall => "its point is inside a wall".to_string(),
                Unbound::NotEnclosed => "no loop of walls closes round its point".to_string(),
                Unbound::Shared(before) => format!("{} has no area yet: its point is in the space {} has. Part the two with a wall or a room line, and each gets its own.", tree.called(path), tree.called(&list[before].0)),
            };
            let figure = |value: f64, mine: &[usize], unit: &str, formula: &str| -> Derived {
                let mut inputs = vec![input.clone()];
                inputs.extend(mine.iter().map(|&k| format!("{}#{K_WALL}", placed[k].path)));
                let mut statuses: Vec<Status> = mine.iter().map(|&k| placed[k].shape.max(placed[k].span)).collect();
                statuses.extend([room.point.status, cut_status]);
                // A room measured to a room line is no surer than the line (call 152).
                for &k in &found.along {
                    statuses.push(parting[k].status);
                    inputs.push(format!("{}#{K_ROOM_LINE}", parting[k].path));
                }
                let mut notes: Vec<String> = Vec::new();
                // Walls whose heights are equally sure are said in one sentence, not one
                // each: a room of four traced walls read as the same clause four times.
                let mut by_how_sure: Vec<(String, Vec<String>)> = Vec::new();
                for &k in &rests_on[i] {
                    statuses.push(placed[k].span);
                    if !inputs.contains(&format!("{}#{K_WALL}", placed[k].path)) {
                        inputs.push(format!("{}#{K_WALL}", placed[k].path));
                    }
                    let (how, wall) = (format!("{:?}", placed[k].span).to_lowercase(), format!("{} ({})", tree.called(&placed[k].path), placed[k].path));
                    match by_how_sure.iter_mut().find(|g| g.0 == how) {
                        Some(group) => group.1.push(wall),
                        None => by_how_sure.push((how, vec![wall])),
                    }
                }
                for (how, walls) in by_how_sure {
                    notes.push(match walls.as_slice() {
                        [one] => format!("its outline counts on {one}, whose height is {how}"),
                        [most @ .., last] => format!("its outline counts on {} and {last}, whose heights are {how}", most.join(", ")),
                        [] => continue,
                    });
                }
                // A wall's end is cut to the walls it meets there, bounding or not. Where
                // that end is at this room (a corner of the room lies within the walls'
                // thickness of it), the room is no surer than the line and thickness of
                // each of those. An end at the far side of the building is not this room's.
                let thick = |w: &walls::PlanWall| (w.ext - w.int).abs();
                for (k, at_end) in mine.iter().flat_map(|&k| [(k, false), (k, true)]) {
                    let met = walls::meets(&plan_walls, k, at_end, tol);
                    let node = if at_end { plan_walls[k].end } else { plan_walls[k].start };
                    let reach = met.iter().map(|&n| thick(&plan_walls[n])).fold(thick(&plan_walls[k]), f64::max) + tol;
                    if !found.corners.iter().any(|c| c.dist(node) <= reach) {
                        continue;
                    }
                    for n in met {
                        if placed[n].shape != Status::Known && !mine.contains(&n) {
                            statuses.push(placed[n].shape);
                            let cited = format!("{}#{K_WALL}", placed[n].path);
                            if !inputs.contains(&cited) {
                                inputs.push(cited);
                                notes.push(format!("{} ({}) is cut to {} ({}), whose line or thickness is {}", tree.called(&placed[k].path), placed[k].path, tree.called(&placed[n].path), placed[n].path, format!("{:?}", placed[n].shape).to_lowercase()));
                            }
                        }
                    }
                }
                let mut d = Derived::new(Some(value), unit, &statuses, formula, &inputs);
                // A wall elsewhere that could not be placed is assumed not to reach this
                // storey, and the figure says so.
                if let (Some((wall, why)), true) = (unplaced.first(), d.value.is_some()) {
                    d.status = d.status.max(Status::Assumed);
                    notes.push(format!("assumes {} does not bound this room: it cannot be placed ({why})", tree.called(wall)));
                }
                if d.value.is_some() {
                    d.note = notes.join("; ");
                }
                d
            };
            let m = out.entry(path.clone()).or_default();
            match (&found.finish, found.in_wall) {
                (Ok((area, round, mine)), false) => {
                    m.insert("floor_area".into(), figure(*area, mine, "SF", F_FLOOR));
                    m.insert("perimeter".into(), figure(*round, mine, "LF", F_ROUND));
                    let walls_d = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol) {
                        Ok((value, statuses, cited)) => {
                            let mut d = figure(value, mine, "SF", F_WALLS);
                            if d.value.is_some() {
                                d.status = statuses.into_iter().fold(d.status, Status::max);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                            }
                            d
                        }
                        Err(why) => Derived::refused("SF", F_WALLS, std::slice::from_ref(&input), &why),
                    };
                    // The run of wall: wall_area's status and inputs, and the doorways it is cut at (call 157).
                    let run_d = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol).and_then(|(_, area_statuses, area_cited)| room_wall_run(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, &tree.kinds, tol).map(|(v, s, c)| (v, area_statuses.into_iter().chain(s).collect::<Vec<_>>(), area_cited.into_iter().chain(c).collect::<Vec<_>>()))) {
                        Ok((value, statuses, cited)) => {
                            let mut d = figure(value, mine, "LF", F_RUN);
                            if d.value.is_some() {
                                d.status = statuses.into_iter().fold(d.status, Status::max);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                            }
                            d
                        }
                        Err(why) => Derived::refused("LF", F_RUN, std::slice::from_ref(&input), &why),
                    };
                    // The base: the run of wall less what stands against it on the floor (call 162).
                    let on_this_storey: Vec<&Standing> = standing.iter().filter(|(s, _)| s.as_deref() == Some(storey.as_str())).map(|(_, s)| s).collect();
                    let base_d = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol).and_then(|(_, area_statuses, area_cited)| room_base_run(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, &tree.kinds, &on_this_storey, tol).map(|(v, s, c, n)| (v, area_statuses.into_iter().chain(s).collect::<Vec<_>>(), area_cited.into_iter().chain(c).collect::<Vec<_>>(), n))) {
                        Ok((value, statuses, cited, things)) => {
                            let mut d = figure(value, mine, "LF", F_BASE);
                            if d.value.is_some() {
                                d.status = statuses.into_iter().fold(d.status, Status::max);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                                // What it took off the wall run is said (less what is set against the walls).
                                let wall_run = run_d.value;
                                if let (Some(run), true) = (wall_run, things > 0) {
                                    let took = format!("less {} ft behind {} set against the walls", crate::view::num(run - value), if things == 1 { "1 thing".to_string() } else { format!("{things} things") });
                                    d.note = if d.note.is_empty() { took } else { format!("{}; {took}", d.note) };
                                }
                            }
                            d
                        }
                        Err(why) => Derived::refused("LF", F_BASE, std::slice::from_ref(&input), &why),
                    };
                    m.insert("wall_area".into(), walls_d);
                    m.insert("wall_run".into(), run_d);
                    m.insert("base_run".into(), base_d);
                    let outlets = match room_wall_area(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, tol).and_then(|_| room_receptacles(&found.outline, &found.islands, mine, &room_lines, &plan_walls, &placed, &holes_of, &tree.kinds, tol)) {
                        Ok((count, statuses, cited)) => {
                            let mut d = figure(count, mine, "EA", F_OUTLETS);
                            if d.value.is_some() {
                                // What it leaves out is a fact of the room, not of the walls.
                                d.status = statuses.into_iter().fold(d.status, Status::max).max(Status::Assumed);
                                for c in cited {
                                    if !d.inputs.contains(&c) {
                                        d.inputs.push(c);
                                    }
                                }
                                d.note = if d.note.is_empty() { OUTLETS_NOTE.to_string() } else { format!("{}; {OUTLETS_NOTE}", d.note) };
                            }
                            d
                        }
                        Err(why) => Derived::refused("EA", F_OUTLETS, std::slice::from_ref(&input), &why),
                    };
                    m.insert("receptacles".into(), outlets);
                }
                (other, _) => {
                    let reason = why(other.as_ref().err().copied().unwrap_or(Unbound::NotEnclosed));
                    m.insert("floor_area".into(), Derived::refused("SF", F_FLOOR, std::slice::from_ref(&input), &reason));
                    m.insert("perimeter".into(), Derived::refused("LF", F_ROUND, std::slice::from_ref(&input), &reason));
                    m.insert("wall_area".into(), Derived::refused("SF", F_WALLS, std::slice::from_ref(&input), &reason));
                    m.insert("wall_run".into(), Derived::refused("LF", F_RUN, std::slice::from_ref(&input), &reason));
                    m.insert("base_run".into(), Derived::refused("LF", F_BASE, std::slice::from_ref(&input), &reason));
                    m.insert("receptacles".into(), Derived::refused("EA", F_OUTLETS, std::slice::from_ref(&input), &reason));
                }
            }
            // The faces can close a room that the wall lines do not: two walls drawn
            // to stop a couple of inches short of each other, inside each other's
            // thickness. The area inside the faces is then right, and the area to the
            // wall centers runs out through the gap and takes its neighbors (J22A-3: a
            // bedroom of 144 SF inside its walls and 865 SF to their centers, shown
            // as a figure). The centers cannot lie farther out than the widest wall
            // all the way round, so a figure past that is not this room's and is
            // refused.
            let widest = plan_walls.iter().map(|w| (w.ext - w.int).abs()).fold(0.0, f64::max);
            let runs_out = match (&found.centre, &found.finish) {
                (Ok((gross, _)), Ok((inside, round, _))) => *gross > inside + round * widest + 4.0 * widest * widest + 1.0,
                _ => false,
            };
            match (&found.centre, found.in_wall) {
                (Ok(_), false) if runs_out => {
                    let reason = "the walls round this room close at their faces and not on their lines: two of them stop short of each other by less than a wall's thickness, and the area to their centers runs out through the gap. Draw them to meet";
                    m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, std::slice::from_ref(&input), reason));
                }
                (Ok((area, mine)), false) => {
                    m.insert("gross_area".into(), figure(*area, mine, "SF", F_GROSS));
                }
                (other, _) => {
                    let reason = why(other.as_ref().err().copied().unwrap_or(Unbound::NotEnclosed));
                    m.insert("gross_area".into(), Derived::refused("SF", F_GROSS, std::slice::from_ref(&input), &reason));
                }
            }
        }
    }
}
