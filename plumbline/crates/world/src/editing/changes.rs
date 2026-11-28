//! Cut from `editing.rs`. PORTED CODE in part, as `editing.rs` says: the rules of the verbs are BIMCraft's,
//! `crates/core/src/commands.rs` at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (MIT); `PORTMAP.md` has each item.

use super::*;

/// Move a wall, a run, a fixture, a piece of equipment, a fitting or a room's point by
/// so many feet east and north. A wall's doors and windows go with it: they are placed
/// along it. What was moved is no surer than the move: it takes `how`'s status if that
/// is worse than its own.
///
/// A wall moved takes with it the ends of the walls that meet it (call 170): see
/// `stretch::move_wall`, which this is for a wall, and which says what followed. To move
/// a wall and nothing else, `move_alone`.
pub fn move_by(job: &mut Job, path: &str, east: f64, north: f64, how: Entered) -> Result<(), String> {
    move_wall(job, path, east, north, how, false).map(|_| ())
}

/// A wall, a run, a fixture, a piece of equipment, a fitting or a room's point moved by
/// itself: whatever meets it is left where it was (a wall's doors and windows still go
/// with it). What `move_by` did to a wall before call 170.
pub fn move_alone(job: &mut Job, path: &str, east: f64, north: f64, how: Entered) -> Result<(), String> {
    all_or_nothing(job, |job| shift_one(job, path, east, north, how))
}

/// The move of one element and nothing else; the caller keeps it all or nothing.
pub(crate) fn shift_one(job: &mut Job, path: &str, east: f64, north: f64, how: Entered) -> Result<(), String> {
    {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = job.kind(path).ok_or_else(|| format!("{path} has no kind"))?;
        if !(east.is_finite() && north.is_finite()) {
            return Err("the distance to move by is not a figure".into());
        }
        let shift = |unit: &str, p: P2| -> Result<P2, String> {
            let d = units::coord_divisor(unit)?;
            Ok([p[0] + east * d, p[1] + north * d])
        };
        if east == 0.0 && north == 0.0 {
            return Err(format!("{path} is moved by nothing"));
        }
        let moved = |status: Status, source: &str| (status.max(how.status), after(source, "moved", how.source));
        if let Some(mut w) = job.world.get::<Wall>(e).cloned() {
            let line = w.line.value.ok_or_else(|| format!("{path}'s line is not known, so it cannot be moved"))?;
            let (status, source) = moved(w.line.status, &w.line.source);
            w.line = Val { value: Some([shift(&w.line.unit, line[0])?, shift(&w.line.unit, line[1])?]), unit: w.line.unit.clone(), status, source };
            building::check_wall(path, kind, &w)?;
            job.put(path, w)?;
        } else if let Some(mut s) = job.world.get::<Segment>(e).cloned() {
            let line = s.line.value.ok_or_else(|| format!("{path}'s line is not known, so it cannot be moved"))?;
            let (status, source) = moved(s.line.status, &s.line.source);
            s.line = Val { value: Some([shift(&s.line.unit, line[0])?, shift(&s.line.unit, line[1])?]), unit: s.line.unit.clone(), status, source };
            building::check_segment(path, kind, &s)?;
            job.put(path, s)?;
        } else if let Some(mut p) = job.world.get::<Placed>(e).cloned() {
            let at = p.at.value.ok_or_else(|| format!("where {path} is is not known, so it cannot be moved"))?;
            let (status, source) = moved(p.at.status, &p.at.source);
            p.at = Val { value: Some(shift(&p.at.unit, at)?), unit: p.at.unit.clone(), status, source };
            building::check_placed(path, kind, &p)?;
            job.put(path, p)?;
        } else if let Some(mut r) = job.world.get::<crate::RoomLine>(e).cloned() {
            let line = r.line.value.ok_or_else(|| format!("{path}'s line is not known, so it cannot be moved"))?;
            let (status, source) = moved(r.line.status, &r.line.source);
            r.line = Val { value: Some([shift(&r.line.unit, line[0])?, shift(&r.line.unit, line[1])?]), unit: r.line.unit.clone(), status, source };
            building::check_room_line(path, kind, &r)?;
            job.put(path, r)?;
        } else if let Some(mut r) = job.world.get::<crate::Room>(e).cloned() {
            let at = r.point.value.ok_or_else(|| format!("{path}'s point is not known, so it cannot be moved"))?;
            let (status, source) = moved(r.point.status, &r.point.source);
            r.point = Val { value: Some(shift(&r.point.unit, at)?), unit: r.point.unit.clone(), status, source };
            building::check_room(path, kind, &r)?;
            job.put(path, r)?;
        } else {
            return Err(format!("{kind:#} is not moved this way; a wall, a run, a fixture, equipment, a fitting, a room or a room line is"));
        }
        Ok(())
    }
}

/// Turn a placed thing about its own point, by `degrees` counter-clockwise seen from
/// above: a fixture, a piece of equipment or a furnishing set against a wall that runs
/// the other way. Its size is its own (width, depth, height) and does not change; what
/// it is turned to is as sure as `how`. Returns how far it is now turned, between
/// −180 and 180 degrees. A fitting is not turned by hand: it lies along the run it joins.
pub fn turn(job: &mut Job, path: &str, degrees: f64, how: Entered) -> Result<f64, String> {
    all_or_nothing(job, |job| {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = job.kind(path).ok_or_else(|| format!("{path} has no kind"))?;
        if !degrees.is_finite() {
            return Err("how far to turn it is not a figure".into());
        }
        // Nothing, or whole turns: it stands as it stood.
        if (degrees % 360.0).abs() < 1.0e-9 {
            return Err(format!("turning it by {} degrees changes nothing", crate::view::num(degrees)));
        }
        let Some(mut p) = job.world.get::<Placed>(e).cloned() else {
            return Err(format!("{kind:#} is not turned this way; a fixture, a piece of equipment or a furnishing is"));
        };
        if kind == Kind::Fitting {
            return Err("a fitting lies along the run it joins, and is not turned by hand".into());
        }
        let was = p.angle_deg().ok_or_else(|| format!("how far {path} is turned is not known, so it cannot be turned from there"))?;
        // Kept between −180 and 180, and tidy: four quarter turns are no turn at all.
        let mut now = (was + degrees) % 360.0;
        if now > 180.0 {
            now -= 360.0;
        } else if now <= -180.0 {
            now += 360.0;
        }
        let now = (now * 1.0e6).round() / 1.0e6;
        // Turned four times on one person's word, it says so once.
        let note = format!("turned ({})", how.source);
        let (status, source) = match &p.angle {
            Some(a) if a.source == note => (a.status.max(how.status), note),
            Some(a) => (a.status.max(how.status), after(&a.source, "turned", how.source)),
            None => (how.status, note),
        };
        p.angle = if now == 0.0 && status == Status::Known { None } else { Some(Val { value: Some(now), unit: "deg".into(), status, source }) };
        building::check_placed(path, kind, &p)?;
        job.put(path, p)?;
        Ok(now)
    })
}

/// What stands against a wall's faces and would be left standing when it goes, said in a
/// sentence: "3 things set against it still stand: B18 (furnishing-0059), ...; move or
/// remove them". None when nothing does. Nothing is moved or removed for the estimator
/// (call 157).
pub fn set_against_said(job: &mut Job, wall: &str) -> Option<String> {
    let found = building::things_against(job, wall).ok()?;
    if found.is_empty() {
        return None;
    }
    let named: Vec<String> = found.iter().take(5).map(|(p, n)| format!("{n} ({p})")).collect();
    let more = if found.len() > 5 { format!(" and {} more", found.len() - 5) } else { String::new() };
    let one = found.len() == 1;
    Some(format!(
        "{} set against it still {}: {}{more}; move or remove {}",
        if one { "1 thing".to_string() } else { format!("{} things", found.len()) },
        if one { "stands" } else { "stand" },
        named.join(", "),
        if one { "it" } else { "them" }
    ))
}

/// Take an element out of the job. A wall's doors and windows go with it. Anything else
/// with elements under it, or that something else still points at, is refused, and the
/// job is left as it was: a type in use, a system with runs on it, a material something
/// is made of, a condition something has, a roof plane a line names, and what a roof's
/// footprint solves (it would come back the next time the roof is solved).
pub fn remove(job: &mut Job, path: &str) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        let kind = job.kind(path).ok_or_else(|| format!("no element {path}"))?;
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        if let Some(solved) = job.world.get::<Solved>(e) {
            return Err(format!("{path} is solved from {}'s footprint and would come back: change the footprint, or remove {} (in the window, Whole roof on the plane's panel picks it)", solved.from, solved.from));
        }
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_default();
        let listed = |who: &[String]| format!("{}{}", who.iter().take(3).cloned().collect::<Vec<_>>().join(", "), if who.len() > 3 { format!(" and {} more", who.len() - 3) } else { String::new() });
        if kind == Kind::RoofPlane {
            let mut q = job.world.query::<(&Node, &Run)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(_, r)| r.planes.iter().any(|p| p == path)).map(|(n, _)| n.path.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is the plane {} lie on: remove those first", listed(&by)));
            }
        }
        // A type that things are of is not taken from under them, and the refusal says
        // whose type it is (J21-5: it said the type "is not a footing type in this job",
        // of a footing's type).
        if kind == Kind::Type {
            let mut by: Vec<String> = Vec::new();
            let mut walls = job.world.query::<(&Node, &Wall)>();
            by.extend(walls.iter(&job.world).filter(|(_, c)| c.type_name.value.as_deref() == Some(path)).map(|(n, _)| n.path.clone()));
            let mut openings = job.world.query::<(&Node, &Opening)>();
            by.extend(openings.iter(&job.world).filter(|(_, c)| c.type_name.value.as_deref() == Some(path)).map(|(n, _)| n.path.clone()));
            let mut floors = job.world.query::<(&Node, &crate::Floor)>();
            by.extend(floors.iter(&job.world).filter(|(_, c)| c.type_name.value.as_deref() == Some(path)).map(|(n, _)| n.path.clone()));
            let mut runs = job.world.query::<(&Node, &Segment)>();
            by.extend(runs.iter(&job.world).filter(|(_, c)| c.type_name.value.as_deref() == Some(path)).map(|(n, _)| n.path.clone()));
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is the type of {}: give {} another type, or remove {}, first", listed(&by), if by.len() == 1 { "it" } else { "them" }, if by.len() == 1 { "it" } else { "them" }));
            }
        }
        // Nor is an item that things are set down from: each would be left as big as
        // an item that is no longer there.
        if job.world.get::<crate::Item>(e).is_some() {
            let mut q = job.world.query::<(&Node, &Placed)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(_, p)| p.item.as_deref() == Some(path)).map(|(n, _)| n.path.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{} set down from {path}: take {} out of the job first", if by.len() == 1 { "one is".to_string() } else { format!("{} are", by.len()) }, if by.len() == 1 { "it" } else { "them" }));
            }
        }
        if kind == Kind::Material {
            let mut q = job.world.query::<(&Node, &MadeOf)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(_, m)| m.0.value.as_ref().is_some_and(|v| v == path || crate::check::same_name(v, &name))).map(|(n, _)| n.path.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is what {} {} made of: give {} another material first", listed(&by), if by.len() == 1 { "is" } else { "are" }, if by.len() == 1 { "it" } else { "them" }));
            }
        }
        // An assembly an item is priced as is not taken from under it (call 196).
        if kind == Kind::Condition {
            let mut q = job.world.query::<(&Node, &Item)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(_, it)| it.priced_as.as_ref().is_some_and(|a| a == &name || a == path || crate::check::same_name(a, &name))).map(|(n, _)| n.name.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is what {} {} priced as: price {} as something else first", listed(&by), if by.len() == 1 { "is" } else { "are" }, if by.len() == 1 { "it" } else { "them" }));
            }
        }
        if kind == Kind::Condition {
            let mut q = job.world.query::<(&Node, &Condition)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(n, c)| n.path != path && c.0.value.as_ref().is_some_and(|v| v == path || crate::check::same_name(v, &name))).map(|(n, _)| n.path.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is the assembly of {}: give {} another first", listed(&by), if by.len() == 1 { "it" } else { "them" }));
            }
        }
        let mut gone = Vec::new();
        // A roof solved from its footprint goes with everything solved from it: its
        // planes and the eaves, hips, valleys and ridges round and between them. None
        // of those can be taken out alone (they would come back), so until this was
        // here a roof drawn in the window could not be deleted at all (seen
        // 2026-11-16: "roof has elements under it; remove those first").
        if job.world.get::<Footprint>(e).is_some() {
            let under: Vec<String> = job.children().get(path).map(|c| c.values().cloned().collect()).unwrap_or_default();
            for child in under {
                let solved_here = job.find(&child).and_then(|c| job.world.get::<Solved>(c)).is_some_and(|s| s.from == path);
                if solved_here {
                    job.remove(&child)?;
                    gone.push(child);
                }
            }
        }
        if kind == Kind::Wall {
            let under: Vec<String> = job.children().get(path).map(|c| c.values().cloned().collect()).unwrap_or_default();
            for child in under {
                job.remove(&child)?;
                gone.push(child);
            }
        }
        job.remove(path)?;
        gone.push(path.to_string());
        Ok(gone)
    })
}

/// What price a thing carries that goes with it when it is taken out (an item's, a material's), said in the words the price
/// list is read in: its own and each supplier's (call 206 point 5). None where it has none.
pub fn prices_that_go(job: &mut Job, path: &str) -> Option<String> {
    let e = job.find(path)?;
    let supply = job.world.get::<crate::components::Supply>(e)?;
    let say = |v: &Val<f64>| v.value.map(|x| format!("{}/{}", crate::view::money(x), v.unit.trim_start_matches("$/")));
    let mut each: Vec<String> = Vec::new();
    if let Some(p) = say(&supply.price) {
        each.push(format!("the price {p}"));
    }
    for (supplier, v) in &supply.quotes {
        if let Some(p) = say(v) {
            each.push(format!("{supplier}'s {p}"));
        }
    }
    if each.is_empty() {
        return None;
    }
    let n = each.len();
    Some(format!("its {} in the price list {} with it: {}; bring {} in again to price it", if n == 1 { "1 price".to_string() } else { format!("{n} prices") }, if n == 1 { "went" } else { "went" }, each.join(", "), if n == 1 { "it" } else { "them" }))
}

/// What to do with the cost lines a thing carries when it is marked existing or to
/// demolish: they are lines for building it, and it is not being built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItsLines {
    /// Not said: a thing that carries lines is refused, and told the two choices.
    Ask,
    /// Take them off the thing.
    Clear,
    /// Leave them priced: they are work done to it (paint, a patch, taking it down).
    Keep,
}

/// Mark what the work does to things (call 148): existing to remain, to demolish, new,
/// or temporary. A wall's doors and windows are marked with it. A thing marked existing
/// or to demolish that carries cost lines is not bought again or built: its lines are
/// cleared or kept as `lines` says, and where that is not said it is refused with the
/// choice. Returns what to say, thing by thing.
pub fn mark(job: &mut Job, paths: &[String], status: WorkStatus, lines: ItsLines, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        let mut said = Vec::new();
        for path in paths {
            let kind = job.kind(path).ok_or_else(|| format!("there is no {path} in this job"))?;
            if !check::takes_work(kind) {
                return Err(format!("{path} is {kind:#}, which is not a thing that is built: walls, doors, windows, floors, footings, roofs, runs and things set in place are marked"));
            }
            // What the thing itself was before this mark: its doors and windows follow it
            // only where they were what it was. One with a mark of its own (a new door
            // hung in an existing wall, a door to demolish) keeps it, and that is said.
            let status_of = |job: &mut Job, p: &str| job.find(p).and_then(|e| job.world.get::<Work>(e)).and_then(|w| w.status.value).unwrap_or(WorkStatus::New);
            let was = status_of(job, path);
            let mut all = vec![path.clone()];
            if kind == Kind::Wall {
                let children = job.children().get(path).cloned().unwrap_or_default();
                let openings: Vec<String> = children.into_values().filter(|c| job.kind(c).is_some_and(|k| k.is_opening())).collect();
                for c in openings {
                    let its = status_of(job, &c);
                    if its == was {
                        all.push(c);
                    } else {
                        said.push(format!("  {c}, in it: stays {}", its.said()));
                    }
                }
            }
            for (i, p) in all.iter().enumerate() {
                let k = job.kind(p).ok_or_else(|| format!("there is no {p} in this job"))?;
                let before = status_of(job, p);
                let w = Work { status: how.val(status, "") };
                check::work(p, k, &w)?;
                job.put(p, w)?;
                let e = job.find(p).ok_or_else(|| format!("there is no {p} in this job"))?;
                let mut line = if i == 0 { format!("{p}: {}", status.said()) } else { format!("  {p}, in it: {}", status.said()) };
                let carried = job.world.get::<Cost>(e).map(|c| c.lines.len()).unwrap_or(0);
                if matches!(status, WorkStatus::Existing | WorkStatus::Demolish) && carried > 0 {
                    let why = if status == WorkStatus::Existing { "what stands is not bought again" } else { "what comes down is not built" };
                    match lines {
                        ItsLines::Ask => {
                            return Err(format!("{p} carries {}, and {why}. Say which: take its lines off, or keep them priced as work done to it (paint, a patch, taking it down)", crate::view::so_many(carried, "cost line")));
                        }
                        ItsLines::Clear => {
                            // What was entered on each, and what it came to, is said before they go (call 206 point 1).
                            let had: Vec<crate::components::CostLine> = job.world.get::<Cost>(e).map(|c| c.lines.clone()).unwrap_or_default();
                            let estimate = crate::view::estimate(job);
                            let mut named = Vec::new();
                            for l in &had {
                                let mut t = job.took_of(p, l);
                                t.amount = estimate.lines.iter().find(|x| x.path == *p && x.description == l.description && !x.off).and_then(|x| x.amount.value);
                                if !t.entered.is_empty() || t.amount.is_some() {
                                    named.push(format!("{:?}{}", t.line, t.tail()));
                                }
                            }
                            job.world.entity_mut(e).remove::<Cost>();
                            line.push_str(&format!("; its {} taken off: {why}", crate::view::so_many(carried, "cost line")));
                            if !named.is_empty() {
                                line.push_str(&format!(". They had {}", named.join("; ")));
                            }
                        }
                        ItsLines::Keep => line.push_str(&format!("; its {} kept and priced, as work done to it", crate::view::so_many(carried, "cost line"))),
                    }
                }
                // New work with nothing priced on it, that was existing or coming down:
                // whatever was taken off it then is not back (judged 2026-11-21).
                if matches!(status, WorkStatus::New | WorkStatus::Temporary) && matches!(before, WorkStatus::Existing | WorkStatus::Demolish) && carried == 0 {
                    line.push_str("; it carries no cost lines: give it its assembly or its lines again");
                }
                said.push(line);
                if i == 0 && kind == Kind::Wall && status == WorkStatus::Demolish {
                    if let Some(against) = set_against_said(job, p) {
                        said.push(format!("  {against}"));
                    }
                }
            }
        }
        // A wall that comes down may be bearing and may have pipes and wiring in it: not
        // seen until it is opened. Both are asked in the field (call 160).
        // Only what has not been said is asked: a wall whose bearing was said is not asked
        // that again (call 167).
        if status == WorkStatus::Demolish {
            let (mut bearing, mut inside) = (false, false);
            for p in paths {
                if job.kind(p) != Some(Kind::Wall) {
                    continue;
                }
                let found = job.find(p);
                let said_of = found.and_then(|e| job.world.get::<Hidden>(e)).cloned();
                bearing |= said_of.as_ref().and_then(|h| h.bearing.as_ref()).is_none_or(|b| b.value.is_none());
                inside |= said_of.as_ref().and_then(|h| h.in_the_wall.as_ref()).is_none_or(|b| b.value.is_none());
            }
            match (bearing, inside) {
                (true, true) => said.push("Is it bearing, and what is in it? Both are on the field list.".to_string()),
                (true, false) => said.push("Is it bearing? That is on the field list.".to_string()),
                (false, true) => said.push("What is in it? That is on the field list.".to_string()),
                (false, false) => {}
            }
        }
        Ok(said)
    })
}
