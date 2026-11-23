//! What a plan set draws of one level, read from the job: its walls with their joins, the
//! doors and windows in them, the rooms the work leaves and the rooms as they stand.
//!
//! This file is ours: BIMCraft has no function that gives a level's walls as a plan sees
//! them (its plan pass walks the model's elements, `render.rs:2607` `draw_plan`, cutting
//! `walls::outline_in` at the view's height, `render.rs:568` `cut_outlines`), and it has
//! no rooms as they stood: `grep -rn 'Demolished' crates/core/src/rooms.rs` finds none, its
//! rooms are found among what stands in the phase being looked at (`phasing.rs`). What is
//! worked out here is read from the figures the model already has: a wall's length is its
//! `plan_length` (`measures`), a room the work leaves is its `floor_area`; only the rooms
//! as they stand are found here, by the model's own `rooms_among`, among the walls that
//! stand today.


use bevy_ecs::prelude::*;

use super::model::{Opening, Room, RoomLine, Wall};
use super::walls::PlanWall;
use super::{hole_of, measures, path_number, plan_wall, rooms_among, Tree};
use crate::components::*;
use crate::job::Job;
use crate::value::Status;

/// What the work does to a thing, as a place: 0 to build, 1 existing to remain, 2 to demolish
/// (`view::work_place`, call 158).
pub type WorkPlace = usize;

/// A wall of a level as a plan sees it.
#[derive(Clone, Debug)]
pub struct LevelWall {
    pub path: String,
    pub name: String,
    /// Its line, its faces, its heights, and whether it joins, as the model has them.
    pub wall: PlanWall,
    pub place: WorkPlace,
    /// Its length on the plan (the model's `plan_length`), feet, none when not known, and
    /// how sure it is.
    pub length: Option<f64>,
    pub length_status: Status,
}

/// A door or window of a level.
#[derive(Clone, Debug)]
pub struct LevelOpening {
    pub path: String,
    pub name: String,
    pub door: bool,
    /// Its wall, by its place in `LevelPlan::walls`.
    pub wall: usize,
    /// Where it starts and ends along its wall from the wall's first end, feet.
    pub span: [f64; 2],
    /// Its sill and its head above the wall's base.
    pub sill: f64,
    pub head: f64,
    /// The place its own mark puts it in; with no mark it is new (call 179 D1).
    pub place: WorkPlace,
    pub status: Status,
}

/// A cabinet, fixture, piece of equipment or furnishing set in place, as its outline (call 179 D3).
#[derive(Clone, Debug)]
pub struct LevelThing {
    pub path: String,
    pub name: String,
    /// The four corners of its footprint on the plan, feet, in order round it.
    pub ring: [V2; 4],
    pub place: WorkPlace,
    /// Whether it hangs wholly above the height the walls are cut at (a wall box): drawn
    /// dashed, as what is above the cut is.
    pub above_cut: bool,
    /// How sure its place and size are (the worst of the values it is drawn from).
    pub status: Status,
}

type V2 = crate::skeleton::P2;

/// A room, with its area at the walls' faces.
#[derive(Clone, Debug)]
pub struct LevelRoom {
    pub path: String,
    pub name: String,
    pub point: [f64; 2],
    /// Square feet, none when the model has no figure for it.
    pub area: Option<f64>,
    pub status: Status,
    /// Why there is no area, in the model's own words; empty when there is one (call 182).
    pub why: String,
}

/// A room line of a level: it parts rooms where no wall stands (call 182: drawn on the plan).
#[derive(Clone, Debug)]
pub struct LevelRoomLine {
    pub path: String,
    pub a: V2,
    pub b: V2,
    /// Whether it stands for a wall that is marked to demolish: on the plan as the walls
    /// stand today it is not a line.
    pub stands_for_demolished: bool,
}

/// Everything a plan set takes from the job for one level.
#[derive(Clone, Debug)]
pub struct LevelPlan {
    pub title: String,
    pub storey: String,
    pub level: String,
    /// How far above the floor the walls are cut, feet.
    pub cut: f64,
    /// The walls that stand at the cut, in the order they were drawn into the job.
    pub walls: Vec<LevelWall>,
    pub openings: Vec<LevelOpening>,
    /// What is set in place on the level, each as its outline (call 179 D3).
    pub things: Vec<LevelThing>,
    /// Things set in place that cannot be drawn, each with why.
    pub things_left_off: Vec<(String, String)>,
    /// The rooms the work leaves: the model's own figures.
    pub rooms_left: Vec<LevelRoom>,
    /// The rooms as they stand: found among the walls that stand today.
    pub rooms_stand: Vec<LevelRoom>,
    /// Rooms of the level that cannot be lettered as they stand, each with why in words when it is not
    /// simply that no wall that stands closes round its point (then the reason is empty).
    pub rooms_stand_missing: Vec<(String, String)>,
    /// The room lines of the level (call 182).
    pub room_lines: Vec<LevelRoomLine>,
    /// Walls of the level left off the plan, each with why: with no line, or not reaching the cut.
    pub walls_left_off: Vec<(String, String)>,
    /// Doors and windows left off the plan, each with why.
    pub openings_left_off: Vec<(String, String)>,
}

/// The path of a level typed as its path or its name (any capitals).
pub fn level_named(job: &mut Job, typed: &str) -> Result<String, String> {
    let mut q = job.world.query::<(&Node, &Kind)>();
    let storeys: Vec<(String, String)> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Storey).map(|(n, _)| (n.path.clone(), n.name.clone())).collect();
    let t = typed.trim();
    if let Some((p, _)) = storeys.iter().find(|(p, _)| p == t) {
        return Ok(p.clone());
    }
    let named: Vec<&(String, String)> = storeys.iter().filter(|(_, n)| n.eq_ignore_ascii_case(t)).collect();
    match named.as_slice() {
        [one] => Ok(one.0.clone()),
        [] => Err(format!("{typed:?} is not a level of this job{}", if storeys.is_empty() { ": the job has none".to_string() } else { format!("; its levels are {}", storeys.iter().map(|(p, n)| format!("{n} ({p})")).collect::<Vec<_>>().join(", ")) })),
        _ => Err(format!("{typed:?} names more than one level; give its path ({})", named.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>().join(", "))),
    }
}

/// The levels of the job, lowest first (by elevation, then as they were drawn in): the order a
/// plan set's sheets are numbered through (call 182).
pub fn levels_from_the_lowest(job: &mut Job) -> Vec<String> {
    let tree = Tree::of(&mut job.world);
    let mut q = job.world.query::<(&Node, &Kind)>();
    let mut all: Vec<(f64, u64, String)> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Storey).map(|(n, _)| (tree.levels.get(&n.path).and_then(|l| l.elevation.feet()).unwrap_or(f64::MAX), path_number(&n.path), n.path.clone())).collect();
    all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)));
    all.into_iter().map(|(_, _, p)| p).collect()
}

/// Read a level of the job as a plan set draws it, or why it cannot be drawn.
pub fn level_plan(job: &mut Job, storey: &str) -> Result<LevelPlan, String> {
    use crate::skeleton::P2 as V;
    let tol = crate::check::FIT_TOLERANCE_FT;
    let tree = Tree::of(&mut job.world);
    let level = tree.levels.get(storey).cloned().ok_or_else(|| format!("{storey} is not a level with a floor height and an elevation"))?;
    let elevation = level.elevation.feet().ok_or("its level's elevation is not known")?;
    let cut = match &level.computation_height {
        None => super::model::COMPUTATION_HEIGHT_FT,
        Some(c) => c.feet().ok_or("the height this level is cut at is not known")?,
    };
    let title = job.world.query::<&JobInfo>().iter(&job.world).next().map(|j| j.title.clone()).unwrap_or_default();
    let level_name = tree.called(storey);
    let figures = measures(&mut job.world);

    // The walls: placed, or why not.
    let mut all: Vec<(Entity, String, String)> = {
        let mut q = job.world.query::<(Entity, &Node, &Kind, &Wall)>();
        q.iter(&job.world).filter(|(_, n, k, _)| **k == Kind::Wall && tree.storey_of(&n.path) == Some(storey)).map(|(e, n, _, _)| (e, n.path.clone(), n.name.clone())).collect()
    };
    all.sort_by_key(|(_, p, _)| path_number(p));
    let mut walls: Vec<LevelWall> = Vec::new();
    let mut bounds: Vec<bool> = Vec::new();
    let mut shapes: Vec<(Status, Status)> = Vec::new();
    let mut walls_left_off: Vec<(String, String)> = Vec::new();
    for (e, path, name) in all {
        let Some(w) = job.world.get::<Wall>(e).cloned() else { continue };
        let place = crate::view::work_place(&job.world, e);
        match plan_wall(&tree, &path, &w) {
            Err(why) => walls_left_off.push((path, format!("it cannot be placed: {why}"))),
            Ok((pw, shape, span, _)) => {
                let at = elevation + cut;
                if !(pw.base <= at && at < pw.top) {
                    walls_left_off.push((path, format!("it does not stand at {} above the floor, where the walls are cut", plumbline_roofing::drawing::feet_written(cut))));
                    continue;
                }
                let (length, length_status) = figures.get(&path).and_then(|m| m.get("plan_length")).map_or((None, Status::Unknown), |d| (d.value, d.status));
                bounds.push(w.room_bounding);
                shapes.push((shape, span));
                walls.push(LevelWall { path, name, wall: pw, place, length, length_status });
            }
        }
    }

    // The doors and windows, each in a wall that is on the plan.
    let mut ops: Vec<(Entity, String, String, Kind, String)> = {
        let mut q = job.world.query::<(Entity, &Node, &Kind, &Opening)>();
        q.iter(&job.world).filter(|(_, _, k, _)| k.is_opening()).filter_map(|(e, n, k, _)| n.parent.clone().map(|p| (e, n.path.clone(), n.name.clone(), *k, p))).collect()
    };
    ops.sort_by_key(|(_, p, ..)| path_number(p));
    let mut openings: Vec<LevelOpening> = Vec::new();
    let mut openings_left_off: Vec<(String, String)> = Vec::new();
    for (e, path, name, kind, host) in ops {
        let Some(wi) = walls.iter().position(|w| w.path == host) else {
            // Its wall is not on the plan: said once there, by its wall, when it is a wall
            // that could not be placed or does not reach the cut.
            if tree.storey_of(&host) == Some(storey) {
                openings_left_off.push((path, format!("its wall, {}, is not on the plan", tree.called(&host))));
            }
            continue;
        };
        let Some(o) = job.world.get::<Opening>(e).cloned() else { continue };
        let (hole, _) = hole_of(&tree, &path, &o);
        let Some([s0, s1, sill, head]) = hole.span else {
            openings_left_off.push((path, "its size or where it is along its wall is not known".to_string()));
            continue;
        };
        // A wall's own heights: the opening is cut at the plan's height when it spans it.
        let at = elevation + cut;
        let base = walls[wi].wall.base;
        if !(base + sill <= at && at < base + head) {
            openings_left_off.push((path, format!("it does not reach {} above the floor, where the walls are cut", plumbline_roofing::drawing::feet_written(cut))));
            continue;
        }
        // An opening is on the sheets its own mark puts it on, and one with no mark is new
        // (call 179 D1): a new door in an existing wall is not on the existing plan.
        let place = crate::view::work_place(&job.world, e);
        openings.push(LevelOpening { path, name, door: kind == Kind::Door, wall: wi, span: [s0, s1], sill, head, place, status: hole.status });
    }

    // The rooms the work leaves: the model's figures.
    let mut rq: Vec<(String, String, Option<[f64; 2]>, Status)> = {
        let mut q = job.world.query::<(&Node, &Kind, &Room)>();
        q.iter(&job.world).filter(|(n, k, _)| **k == Kind::Room && tree.storey_of(&n.path) == Some(storey)).map(|(n, _, r)| (n.path.clone(), n.name.clone(), r.point_ft(), r.point.status)).collect()
    };
    rq.sort_by_key(|(p, ..)| path_number(p));
    let mut rooms_left = Vec::new();
    for (path, name, point, point_status) in &rq {
        let Some(point) = *point else { continue };
        let d = figures.get(path).and_then(|m| m.get("floor_area"));
        let (area, status) = d.map_or((None, *point_status), |d| (d.value, d.status));
        let why = match (area, d) {
            (Some(_), _) => String::new(),
            (None, Some(d)) if !d.note.is_empty() => d.note.clone(),
            _ => format!("{name} has no area: the job has no figure for it"),
        };
        rooms_left.push(LevelRoom { path: path.clone(), name: name.clone(), point, area, status, why });
    }

    // The rooms as they stand: the walls that stand today (existing and to demolish), their
    // joins as they are, and the room lines but those that stand for a wall taken down.
    let stand: Vec<usize> = (0..walls.len()).filter(|&i| walls[i].place >= 1).collect();
    let mut rooms_stand = Vec::new();
    let mut rooms_stand_missing: Vec<(String, String)> = Vec::new();
    let mut room_lines: Vec<LevelRoomLine> = Vec::new();
    if !stand.is_empty() {
        let plan_walls: Vec<PlanWall> = stand.iter().map(|&i| walls[i].wall.clone()).collect();
        let bounding: Vec<usize> = stand.iter().enumerate().filter(|(_, &i)| bounds[i]).map(|(k, _)| k).collect();
        let gone: Vec<&PlanWall> = stand.iter().filter(|&&i| walls[i].place == 2).map(|&i| &walls[i].wall).collect();
        let mut lines: Vec<(V, V)> = Vec::new();
        {
            let mut q = job.world.query::<(&Node, &Kind, &RoomLine)>();
            let mut all: Vec<(String, [[f64; 2]; 2])> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::RoomLine && tree.storey_of(&n.path) == Some(storey)).filter_map(|(n, _, r)| r.line_ft().map(|l| (n.path.clone(), l))).collect();
            all.sort_by_key(|(p, _)| path_number(p));
            for (_, [a, b]) in all {
                let (a, b) = (V(a[0], a[1]), V(b[0], b[1]));
                let stands_for = |w: &&PlanWall| {
                    let half = (w.ext - w.int).abs() / 2.0 + tol;
                    super::on_line(a, w.start, w.end, half) && super::on_line(b, w.start, w.end, half)
                };
                if !gone.iter().any(stands_for) {
                    lines.push((a, b));
                }
            }
        }
        let points: Vec<V> = rq.iter().map(|(_, _, p, _)| p.map_or(V(f64::NAN, f64::NAN), |p| V(p[0], p[1]))).collect();
        let found = rooms_among(&plan_walls, &bounding, &lines, &points, tol);
        for ((path, name, point, point_status), f) in rq.iter().zip(found) {
            let Some(point) = *point else { continue };
            match (&f.finish, f.in_wall) {
                (Ok((area, _, mine)), false) => {
                    let mut status = *point_status;
                    for &k in mine {
                        let (shape, span) = shapes[stand[k]];
                        status = status.max(shape).max(span);
                    }
                    rooms_stand.push(LevelRoom { path: path.clone(), name: name.clone(), point, area: Some(*area), status, why: String::new() });
                }
                (Err(super::rooms::Unbound::Shared(before)), false) => {
                    let other = rq.get(*before).map_or("another room", |r| r.1.as_str());
                    rooms_stand.push(LevelRoom { path: path.clone(), name: name.clone(), point, area: None, status: *point_status, why: format!("{name} has no area as the walls stand: its point is in the space {other} has. Part the two with a wall or a room line, and each gets its own.") });
                }
                (_, true) => rooms_stand_missing.push((name.clone(), format!("{name}'s point is inside a wall"))),
                _ => rooms_stand_missing.push((name.clone(), String::new())),
            }
        }
    } else {
        rooms_stand_missing = rq.iter().map(|(_, n, ..)| (n.clone(), String::new())).collect();
    }
    {
        // Every room line of the level, and whether it stands for a wall to demolish.
        let gone: Vec<PlanWall> = walls.iter().filter(|w| w.place == 2).map(|w| w.wall.clone()).collect();
        let mut q = job.world.query::<(&Node, &Kind, &RoomLine)>();
        let mut all: Vec<(String, [[f64; 2]; 2])> = q.iter(&job.world).filter(|(n, k, _)| **k == Kind::RoomLine && tree.storey_of(&n.path) == Some(storey)).filter_map(|(n, _, r)| r.line_ft().map(|l| (n.path.clone(), l))).collect();
        all.sort_by_key(|(p, _)| path_number(p));
        for (path, [a, b]) in all {
            let (a, b) = (V(a[0], a[1]), V(b[0], b[1]));
            let stands_for = |w: &PlanWall| {
                let half = (w.ext - w.int).abs() / 2.0 + tol;
                super::on_line(a, w.start, w.end, half) && super::on_line(b, w.start, w.end, half)
            };
            room_lines.push(LevelRoomLine { path, a, b, stands_for_demolished: gone.iter().any(stands_for) });
        }
    }
    // What is set in place: cabinets, fixtures, equipment and furnishings, each as the outline
    // of its footprint (the model's own size and place, turned as it is turned).
    let mut things = Vec::new();
    let (drawn, not_drawn) = super::placements(job);
    for p in drawn {
        if !matches!(p.kind, Kind::Furnishing | Kind::Equipment | Kind::Fixture) || p.storey.as_deref() != Some(storey) {
            continue;
        }
        let super::Shape::Box { at, size, angle } = p.shape else { continue };
        let (sin, cos) = angle.to_radians().sin_cos();
        let ring = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(sx, sy)| {
            let (lx, ly) = (sx * size[0] / 2.0, sy * size[1] / 2.0);
            V(at[0] + lx * cos - ly * sin, at[1] + lx * sin + ly * cos)
        });
        let place = job.find(&p.path).map_or(0, |e| crate::view::work_place(&job.world, e));
        things.push(LevelThing { path: p.path, name: p.name, ring, place, above_cut: at[2] >= elevation + cut, status: p.status });
    }
    let things_left_off: Vec<(String, String)> = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let ours: Vec<String> = q.iter(&job.world).filter(|(n, k)| matches!(**k, Kind::Furnishing | Kind::Equipment | Kind::Fixture) && tree.storey_of(&n.path) == Some(storey)).map(|(n, _)| n.path.clone()).collect();
        not_drawn.into_iter().filter(|(p, _)| ours.contains(p)).collect()
    };
    Ok(LevelPlan { title, storey: storey.to_string(), level: level_name, cut, walls, openings, things, things_left_off, rooms_left, rooms_stand, rooms_stand_missing, room_lines, walls_left_off, openings_left_off })
}
