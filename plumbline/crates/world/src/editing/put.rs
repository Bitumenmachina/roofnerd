//! Cut from `editing.rs`. PORTED CODE in part, as `editing.rs` says: the rules of the verbs are BIMCraft's,
//! `crates/core/src/commands.rs` at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (MIT); `PORTMAP.md` has each item.

use super::*;

pub(super) fn storey_of(job: &mut Job, storey: &str) -> Result<(), String> {
    match job.kind(storey) {
        Some(Kind::Storey) => Ok(()),
        Some(other) => Err(format!("{storey} is a {other:?}, not a level")),
        None => Err(format!("no element {storey}")),
    }
}

/// A new storey: its floor's height above the job's datum, and how far up to the next.
pub fn storey(job: &mut Job, name: &str, elevation: &str, height: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let level = Level { elevation: how.length(elevation)?, height: how.length(height)?, computation_height: None };
        let path = job.add(&root, Kind::Storey, name)?;
        building::check_level(&path, Kind::Storey, &level)?;
        job.put(&path, level)?;
        Ok(path)
    })
}

/// A wall type: how thick walls of it are.
pub fn wall_type(job: &mut Job, name: &str, thickness: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let t = WallType { thickness: how.length(thickness)?, layers: Vec::new(), footing: false };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_wall_type(&path, Kind::Type, &t)?;
        job.put(&path, t)?;
        Ok(path)
    })
}

/// A strip footing's type: how wide footings of it are. Held as a wall's type is, and
/// marked as a footing's: only footings are drawn on it (call 87).
pub fn footing_type(job: &mut Job, name: &str, width: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let t = WallType { thickness: how.length(width)?, layers: Vec::new(), footing: true };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_wall_type(&path, Kind::Type, &t).map_err(|why| why.replace("wall type thickness", "footing type width"))?;
        job.put(&path, t)?;
        Ok(path)
    })
}

/// A floor type: how thick floors of it are.
pub fn floor_type(job: &mut Job, name: &str, thickness: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let t = crate::FloorType { thickness: how.length(thickness)?, layers: Vec::new() };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_floor_type(&path, Kind::Type, &t)?;
        job.put(&path, t)?;
        Ok(path)
    })
}

/// A door type or a window type: how wide and how tall openings of it are.
pub fn opening_type(job: &mut Job, name: &str, kind: Kind, width: &str, height: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let t = OpeningType { category: kind, width: how.length(width)?, height: how.length(height)? };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_opening_type(&path, Kind::Type, &t)?;
        job.put(&path, t)?;
        Ok(path)
    })
}

/// A type of duct, pipe, conduit or cable: round and so far across, or so wide by so
/// deep (`depth` given).
pub fn segment_type(job: &mut Job, name: &str, kind: Kind, width: &str, depth: Option<&str>, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let t = SegmentType { category: kind, width: how.length(width)?, depth: depth.map(|d| how.length(d)).transpose()?, round: depth.is_none(), weight_per_foot: None };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_segment_type(&path, Kind::Type, &t)?;
        job.put(&path, t)?;
        Ok(path)
    })
}

/// A type for a beam, a post or a pad footing (call 164): its section, round and so far
/// across or so wide by so deep, and, for a beam, what a foot of it weighs where that is
/// said (`weight`: "18 lb/ft"; a number with no unit is refused). Nothing here sizes
/// anything: every figure is typed, with how sure it is and where it came from.
pub fn framing_type(job: &mut Job, name: &str, kind: Kind, width: &str, depth: Option<&str>, weight: Option<&str>, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        if !kind.is_framing() {
            return Err(format!("{kind:#} is not a beam, a post or a pad footing"));
        }
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let weight_per_foot = match weight {
            None => None,
            Some(typed) => {
                let text = typed.trim();
                // The program's own word for a foot of length is LF (a beam is "16 LF"): "18 lb/LF" is read as "18 lb/ft" (call 179, C8).
                let lower = text.to_ascii_lowercase().replace(" / ", "/");
                let number = ["lb/ft", "lbs/ft", "lb/lf", "lbs/lf", "plf", "#/ft", "#/lf"].iter().find_map(|u| lower.strip_suffix(u)).ok_or_else(|| format!("{text:?} is a weight per foot with no unit: write it like \"18 lb/ft\" or \"18 lb/LF\""))?;
                Some(how.val(units::parse_number(number)?, "lb/ft"))
            }
        };
        let t = SegmentType { category: kind, width: how.length(width)?, depth: depth.map(|d| how.length(d)).transpose()?, round: depth.is_none(), weight_per_foot };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_segment_type(&path, Kind::Type, &t)?;
        job.put(&path, t)?;
        Ok(path)
    })
}

/// Beams along `points` (in feet, on the plan): each pair of points that follow one
/// another is one beam (BIMCraft's `beam`, `commands.rs:577`, which makes one run for
/// each segment of its points), its heights held to its **top** as BIMCraft's beams are
/// (`runs.rs:57`), `top` above the level's floor. A beam that is too short, whose ends
/// are one over the other or that slopes more than 75 degrees is refused (`runs.rs:62-76`).
pub fn beams(job: &mut Job, storey: &str, type_path: &str, points: &[P2], top: &str, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if points.len() < 2 {
            return Err("a beam needs at least 2 points".into());
        }
        let mut made = Vec::new();
        for pair in points.windows(2) {
            let segment = Segment { type_name: how.path(type_path), line: how.val([pair[0], pair[1]], "ft"), offset: how.length(top)?, end_offset: None, justify: crate::Justify::Top };
            let path = add_numbered(job, storey, Kind::Beam)?;
            building::check_segment(&path, Kind::Beam, &segment)?;
            job.put(&path, segment)?;
            made.push(path);
        }
        Ok(made)
    })
}

/// A post stood at a point (BIMCraft's `stood_of`, `commands.rs:8662`, for a structural
/// column): `height` tall from `base` above the level's floor. With no height said it
/// stands to its level's height, Assumed, and the value says so (BIMCraft stands a column
/// to the level above); with no base said it stands on the floor, Assumed.
pub fn post(job: &mut Job, storey: &str, type_path: &str, at: P2, height: Option<&str>, base: Option<&str>, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let height = match height {
            Some(typed) => how.length(typed)?,
            None => {
                let level = job.find(storey).and_then(|e| job.world.get::<Level>(e)).map(|l| l.height.clone()).ok_or_else(|| format!("no level {storey}"))?;
                if level.value.is_none() {
                    return Err(format!("{storey}'s height is not known, so a post has nothing to stand to: say how tall it is with --height"));
                }
                Val { status: Status::Assumed.max(level.status), source: format!("stands to its level's height: nobody said how tall it is (the level's: {})", level.source), ..level }
            }
        };
        let base = match base {
            Some(typed) => how.length(typed)?,
            None => Val { value: Some(Len::One(0.0)), unit: "ft".into(), status: Status::Assumed, source: "stands on its level's floor: no base was given".into() },
        };
        let column = Stood { type_name: how.path(type_path), at: how.val(at, "ft"), base_offset: base, height };
        let path = add_numbered(job, storey, Kind::Post)?;
        building::check_column(&path, Kind::Post, &column)?;
        job.put(&path, column)?;
        Ok(path)
    })
}

/// A pad footing: a post laid flat, `thickness` thick, its **top** `top` above the level's
/// floor (below it, a minus height), so its base is the top less the thickness
/// (BIMCraft `commands.rs:8674-8686`). Unlike there, the thickness is always said: the
/// 400 mm it is given there is a thickness nobody gave.
pub fn pad(job: &mut Job, storey: &str, type_path: &str, at: P2, thickness: &str, top: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let thick = how.length(thickness)?;
        let up = how.length(top)?;
        let (Some(t), Some(u)) = (thick.feet(), up.feet()) else { return Err("a pad footing's thickness and top are lengths this program converts".into()) };
        // Its base in feet, from the two as they were worked (a thickness in inches and a
        // top in feet are added as feet).
        let base = how.val(Len::One(((u - t) * 1.0e9).round() / 1.0e9), "ft");
        let column = Stood { type_name: how.path(type_path), at: how.val(at, "ft"), base_offset: base, height: thick };
        let path = add_numbered(job, storey, Kind::PadFooting)?;
        building::check_column(&path, Kind::PadFooting, &column)?;
        job.put(&path, column)?;
        Ok(path)
    })
}

/// A system: what its runs, equipment and fixtures carry (one of `CLASSIFICATIONS`).
pub fn system(job: &mut Job, name: &str, classification: &str) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let found = building::CLASSIFICATIONS.iter().find(|(c, _)| c.eq_ignore_ascii_case(classification.trim()));
        let (classification, discipline) = found.ok_or_else(|| format!("{classification:?} is not a kind of system; there are: {}", building::CLASSIFICATIONS.iter().map(|c| c.0).collect::<Vec<_>>().join(", ")))?;
        let info = SystemInfo { discipline: *discipline, classification: classification.to_string() };
        let path = job.add(&root, Kind::System, name)?;
        building::check_system(&path, Kind::System, &info)?;
        job.put(&path, info)?;
        Ok(path)
    })
}

/// Walls along `points` (in feet, on the plan): each pair of points that follow one
/// another is one wall, and `closed` adds one from the last back to the first (BIMCraft's
/// `wall` command, `commands.rs:7341`). Unlike there, how tall the walls are is always
/// said: nothing here is given a height nobody entered.
pub fn walls(job: &mut Job, storey: &str, type_path: &str, points: &[P2], closed: bool, top: Top, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let mut pts = points.to_vec();
        if pts.len() < 2 {
            return Err("a wall needs at least 2 points".into());
        }
        if closed {
            pts.push(pts[0]);
        }
        // A top held to a storey holds the wall up to it: the storey has to be above.
        if let Top::Level(level) = &top {
            let floor = |job: &mut Job, path: &str| job.find(path).and_then(|e| job.world.get::<Level>(e).and_then(|l| l.elevation.feet()));
            if let (Some(own), Some(held)) = (floor(job, storey), floor(job, level)) {
                if held <= own + crate::check::FIT_TOLERANCE_FT {
                    return Err(format!("{level}'s floor is not above {storey}'s: a wall on {storey} cannot have its top held to it"));
                }
            }
        }
        type_is_free_for(job, type_path, Kind::Wall)?;
        let mut made = Vec::new();
        for pair in pts.windows(2) {
            let line = how.val([pair[0], pair[1]], "ft");
            let wall = match &top {
                Top::Height(typed) => Wall::tall(how.path(type_path), line, how.length(typed)?),
                Top::Level(level) => Wall::up_to(how.path(type_path), line, how.path(level)),
            };
            let path = add_numbered(job, storey, Kind::Wall)?;
            building::check_wall(&path, Kind::Wall, &wall)?;
            job.put(&path, wall)?;
            made.push(path);
        }
        Ok(made)
    })
}

/// A type gives a wall its thickness or a footing its width, never both: a type that
/// footings are of is not drawn as a wall, nor the other way (the twentieth judgment
/// drew a 24 in "wall" on a footing's type, and a footing 8 in wide on a wall's).
pub(super) fn type_is_free_for(job: &mut Job, type_path: &str, wanted: Kind) -> Result<(), String> {
    // The job's types of the kind wanted, to offer where the one named will not do.
    let offered = |job: &mut Job| {
        let mut q = job.world.query::<(&Node, &WallType)>();
        let mut used = job.world.query::<(&Kind, &Wall)>();
        let under_footings: Vec<String> = used.iter(&job.world).filter(|(k, _)| **k == Kind::Footing).filter_map(|(_, w)| w.type_name.value.clone()).collect();
        let mut names: Vec<String> = q.iter(&job.world).filter(|(n, t)| (t.footing || under_footings.contains(&n.path)) == (wanted == Kind::Footing)).map(|(n, _)| format!("{} {:?}", n.path, n.name)).collect();
        names.sort();
        let make = format!("make one with `type {}`", wanted.said());
        if names.is_empty() { format!("this job has none yet: {make}") } else { format!("this job has {}; or {make}", names.join(", ")) }
    };
    let Some(made_for_footings) = job.find(type_path).and_then(|e| job.world.get::<WallType>(e)).map(|t| t.footing) else {
        return Err(format!("{type_path} is not {} type: {}", if wanted == Kind::Footing { "a footing's" } else { "a wall's" }, offered(job)));
    };
    let mut q = job.world.query::<(&Node, &Kind, &Wall)>();
    let users: Vec<(String, Kind)> = q.iter(&job.world).filter(|(_, k, w)| matches!(**k, Kind::Wall | Kind::Footing) && w.type_name.value.as_deref() == Some(type_path)).map(|(n, k, _)| (n.path.clone(), *k)).collect();
    // What it was made as, or, for a type from a file written before types said so,
    // what is already drawn on it.
    if let Some((path, kind)) = users.iter().find(|(_, k)| *k != wanted) {
        return Err(format!("{type_path} is the type of {path}, {kind:#}: {wanted:#} needs a type of its own ({})", offered(job)));
    }
    let is_footings = made_for_footings || users.iter().any(|(_, k)| *k == Kind::Footing);
    if is_footings != (wanted == Kind::Footing) {
        return Err(format!("{type_path} was made as {} type: {wanted:#} needs a type of its own ({})", if is_footings { "a footing's" } else { "a wall's" }, offered(job)));
    }
    Ok(())
}

/// Strip footings along `points` (in feet, on the plan): each pair of points that follow
/// one another is one footing, and `closed` adds one from the last back to the first
/// (BIMCraft's `strip_footing`, `commands.rs:8707`, which makes one run for each
/// segment of its points as `runs_of` does, `commands.rs:8776`). Its type gives its
/// width; `depth` is how deep it is, and is always said. Its top is at its level's
/// floor, so it lies below it by its depth: a foundation wall on the same level stands
/// on it.
pub fn footings(job: &mut Job, storey: &str, type_path: &str, points: &[P2], closed: bool, depth: &str, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let mut pts = points.to_vec();
        if pts.len() < 2 {
            return Err("a footing needs at least 2 points".into());
        }
        if closed {
            pts.push(pts[0]);
        }
        let deep = how.length(depth)?;
        deep.feet().filter(|d| *d > 0.0).ok_or("a footing's depth has to be more than nothing")?;
        type_is_free_for(job, type_path, Kind::Footing)?;
        let mut made = Vec::new();
        for pair in pts.windows(2) {
            // Its top is at its level's floor; where it lies is worked out from its depth
            // (`building::placements`), so a depth measured later moves its underside.
            let footing = Wall::tall(how.path(type_path), how.val([pair[0], pair[1]], "ft"), deep.clone());
            let path = add_numbered(job, storey, Kind::Footing)?;
            building::check_wall(&path, Kind::Footing, &footing)?;
            job.put(&path, footing)?;
            made.push(path);
        }
        Ok(made)
    })
}

/// A door or a window in a wall (BIMCraft's `opening_of`, `commands.rs:7473`). Given a
/// point, it goes where the point falls along the wall's line (`along`, `commands.rs:2426`);
/// a point that falls off either end is refused, not moved to the end. Unlike there, an
/// opening with no place said is not put in the middle, and a sill is always said.
pub fn opening(job: &mut Job, wall: &str, kind: Kind, type_path: &str, at: OnWall, sill: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        if !kind.is_opening() {
            return Err(format!("{kind:#} is not a door or a window"));
        }
        let host = job.find(wall).and_then(|e| job.world.get::<Wall>(e).cloned()).ok_or_else(|| format!("{wall} is not a wall with a line"))?;
        // A wall that is coming down takes no new door or window (judged 2026-11-21).
        if job.find(wall).and_then(|e| job.world.get::<Work>(e)).is_some_and(|w| w.status.value == Some(WorkStatus::Demolish)) {
            return Err(format!("{wall} is to demolish: {kind:#} is not set in a wall that is coming down. Mark the wall existing or new first, or draw the new wall it goes in"));
        }
        let offset = match at {
            OnWall::Offset(typed) => how.length(typed)?,
            OnWall::Point(p) => {
                let [a, b] = host.line_ft().ok_or_else(|| format!("{wall}'s line is not known, so a point cannot be placed along it"))?;
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let length = (dx * dx + dy * dy).sqrt();
                let along = ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length;
                if !(0.0..=length).contains(&along) {
                    return Err(format!("the point ({}, {}) falls {} along {wall}, which is {} long: it is not on the wall", p[0], p[1], crate::units::length_said(along), crate::units::length_said(length)));
                }
                // And it is in the wall, not out in the room: within half the wall's
                // thickness of its centre line, which lies to the left of the line it
                // was drawn on by what its `location` says.
                let section = {
                    let tree = crate::building::Tree::of(&mut job.world);
                    crate::building::wall_section(&tree, &host).ok()
                };
                let (half, centre) = match section {
                    Some((thickness, layers, _)) => (thickness / 2.0, crate::building::walls::centre(thickness, &layers, host.location)),
                    None => (0.0, 0.0),
                };
                let off = (((p[1] - a[1]) * dx - (p[0] - a[0]) * dy) / length - centre).abs();
                if off > half + crate::check::FIT_TOLERANCE_FT {
                    return Err(format!("the point ({}, {}) is {} off the middle of {wall}, which is {} thick: it is not on the wall", p[0], p[1], crate::units::length_said(off), crate::units::length_said(half * 2.0)));
                }
                how.val(Len::One((along * 10_000.0).round() / 10_000.0), "ft")
            }
        };
        let opening = Opening { type_name: how.path(type_path), offset, sill: how.length(sill)? };
        let path = add_numbered(job, wall, kind)?;
        building::check_opening(&path, kind, &opening)?;
        job.put(&path, opening)?;
        Ok(path)
    })
}

/// One straight piece of duct, pipe, conduit or cable that stands upright at `point` (in
/// feet), from `from` up to `to` above the storey's floor: a riser. Several, end to end, are
/// one riser of several pieces (call 175, for a demonstration of a hole counted once).
#[allow(clippy::too_many_arguments)]
pub fn riser(job: &mut Job, storey: &str, kind: Kind, type_path: &str, point: P2, from: &str, to: &str, system: Option<&str>, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if !kind.is_segment() {
            return Err(format!("{kind:#} is not a duct, a pipe, a conduit or a cable"));
        }
        let segment = Segment { type_name: how.path(type_path), line: how.val([point, point], "ft"), offset: how.length(from)?, end_offset: Some(how.length(to)?), justify: Default::default() };
        let path = add_numbered(job, storey, kind)?;
        building::check_segment(&path, kind, &segment)?;
        job.put(&path, segment)?;
        if let Some(system) = system {
            let on = InSystem(how.path(system));
            building::check_in_system(&path, kind, &on)?;
            job.put(&path, on)?;
        }
        Ok(path)
    })
}

/// Straight pieces of duct, pipe, conduit or cable along `points` (in feet), level at
/// `height` above the storey's floor, on `system` if one is named (BIMCraft's `runs_of`,
/// `commands.rs:8776`: one run for each pair of points that follow one another).
pub fn runs(job: &mut Job, storey: &str, kind: Kind, type_path: &str, points: &[P2], height: &str, system: Option<&str>, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if !kind.is_segment() {
            return Err(format!("{kind:#} is not a duct, a pipe, a conduit or a cable"));
        }
        if points.len() < 2 {
            return Err("a run needs at least 2 points".into());
        }
        let mut made = Vec::new();
        for pair in points.windows(2) {
            let segment = Segment { type_name: how.path(type_path), line: how.val([pair[0], pair[1]], "ft"), offset: how.length(height)?, end_offset: None, justify: Default::default() };
            let path = add_numbered(job, storey, kind)?;
            building::check_segment(&path, kind, &segment)?;
            job.put(&path, segment)?;
            if let Some(system) = system {
                let on = InSystem(how.path(system));
                building::check_in_system(&path, kind, &on)?;
                job.put(&path, on)?;
            }
            made.push(path);
        }
        Ok(made)
    })
}

/// A fixture or a piece of equipment stood at a point (in feet), `offset` above the
/// storey's floor (BIMCraft's `stood_of`, `commands.rs:8662`). Its size is along, across
/// and up, in feet; with none given it is not known, and the thing is listed as not drawn.
pub fn place(job: &mut Job, storey: &str, kind: Kind, what: &str, at: P2, offset: &str, size: Option<P3>, system: Option<&str>, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if !matches!(kind, Kind::Fixture | Kind::Equipment | Kind::Furnishing) {
            return Err(format!("{kind:#} is not a fixture, a piece of equipment or a furnishing"));
        }
        let size = match size {
            Some(s) => how.val(s, "ft"),
            None => Val::unknown("ft", "size not entered"),
        };
        let placed = Placed { what: how.val(what.to_string(), ""), at: how.val(at, "ft"), offset: how.length(offset)?, angle: None, size, item: None };
        // A fixture or a piece of equipment is called what it is: "Lavatory", and the
        // next one "Lavatory 2".
        let path = add_called(job, storey, kind, what, 2, true)?;
        building::check_placed(&path, kind, &placed)?;
        job.put(&path, placed)?;
        if let Some(system) = system {
            let on = InSystem(how.path(system));
            building::check_in_system(&path, kind, &on)?;
            job.put(&path, on)?;
        }
        Ok(path)
    })
}

/// A name no other type in the job goes by: this one, or this and a number after it ("6 in wall 2"),
/// where the job has a type of the name already at another size (call 180, C3: a wall is never
/// dropped for its type's name).
pub(super) fn free_type_name(job: &mut Job, wanted: &str) -> String {
    let taken = |job: &mut Job, name: &str| -> bool {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).unwrap_or_default();
        job.children().get(&root).is_some_and(|c| c.contains_key(name))
    };
    if !taken(job, wanted) {
        return wanted.to_string();
    }
    (2..).map(|n| format!("{wanted} {n}")).find(|name| !taken(job, name)).unwrap_or_else(|| wanted.to_string())
}

/// The wall type of this thickness with no layers, or a new one named for it ("6 in wall": a name reads as a name, call 178).
/// A window draws with a thickness the person typed; this is the type that thickness is.
pub fn wall_type_for(job: &mut Job, thickness: &str, how: Entered) -> Result<String, String> {
    let wanted = how.length(thickness)?.feet().ok_or("the thickness is not a length this program converts")?;
    let mut q = job.world.query::<(&Node, &WallType)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => {
            let name = free_type_name(job, &format!("{} wall", thickness.trim()));
            wall_type(job, &name, thickness, how)
        }
    }
}

/// The wall type of this name and this thickness with no layers, or a new one of that name:
/// what a model this program wrote says each wall's type was called (call 175). Where the
/// job has a type of the name at another thickness, the name is not taken, and it is the
/// type of this thickness (`wall_type_for`) that is given.
pub fn wall_type_named(job: &mut Job, name: &str, thickness: &str, how: Entered) -> Result<String, String> {
    let wanted = how.length(thickness)?.feet().ok_or("the thickness is not a length this program converts")?;
    let mut q = job.world.query::<(&Node, &WallType)>();
    let same: Vec<(String, bool)> = q.iter(&job.world).filter(|(n, t)| n.name == name && !t.footing).map(|(n, t)| (n.path.clone(), t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9))).collect();
    match same.iter().find(|(_, fits)| *fits) {
        Some((path, _)) => Ok(path.clone()),
        None if same.is_empty() => wall_type(job, name, thickness, how),
        None => wall_type_for(job, thickness, how),
    }
}

/// The door or window type of this name, width and height, or a new one of that name
/// (as `wall_type_named`).
pub fn opening_type_named(job: &mut Job, kind: Kind, name: &str, width: &str, height: &str, how: Entered) -> Result<String, String> {
    let (w, h) = (how.length(width)?.feet().ok_or("the width is not a length this program converts")?, how.length(height)?.feet().ok_or("the height is not a length this program converts")?);
    let mut q = job.world.query::<(&Node, &OpeningType)>();
    let same: Vec<(String, bool)> = q.iter(&job.world).filter(|(n, t)| n.name == name && t.category == kind).map(|(n, t)| (n.path.clone(), t.width.feet().is_some_and(|x| (x - w).abs() < 1e-9) && t.height.feet().is_some_and(|x| (x - h).abs() < 1e-9))).collect();
    match same.iter().find(|(_, fits)| *fits) {
        Some((path, _)) => Ok(path.clone()),
        None if same.is_empty() => opening_type(job, name, kind, width, height, how),
        None => opening_type_for(job, kind, width, height, how),
    }
}

/// The floor type of this thickness with no layers, or a new one named for it ("floor 4 in").
pub fn floor_type_for(job: &mut Job, thickness: &str, how: Entered) -> Result<String, String> {
    let wanted = how.length(thickness)?.feet().ok_or("the thickness is not a length this program converts")?;
    let mut q = job.world.query::<(&Node, &crate::FloorType)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => {
            let name = free_type_name(job, &format!("floor {}", thickness.trim()));
            floor_type(job, &name, thickness, how)
        }
    }
}

/// The door or window type of this width and height, or a new one named for them.
pub fn opening_type_for(job: &mut Job, kind: Kind, width: &str, height: &str, how: Entered) -> Result<String, String> {
    let (w, h) = (how.length(width)?.feet().ok_or("the width is not a length this program converts")?, how.length(height)?.feet().ok_or("the height is not a length this program converts")?);
    let mut q = job.world.query::<(&Node, &OpeningType)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.category == kind && t.width.feet().is_some_and(|x| (x - w).abs() < 1e-9) && t.height.feet().is_some_and(|x| (x - h).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => {
            let name = free_type_name(job, &format!("{} {} by {}", kind.stem(), width.trim(), height.trim()));
            opening_type(job, &name, kind, width, height, how)
        }
    }
}

/// The round type of this kind of run and this size across, or a new one named for it.
pub fn segment_type_for(job: &mut Job, kind: Kind, across: &str, how: Entered) -> Result<String, String> {
    let wanted = how.length(across)?.feet().ok_or("the size is not a length this program converts")?;
    let mut q = job.world.query::<(&Node, &SegmentType)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.category == kind && t.round && t.width.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => segment_type(job, &format!("{} {} round", kind.stem(), across.trim()), kind, across, None, how),
    }
}

/// The job's system of this classification, or a new one named for it.
pub fn system_of_class(job: &mut Job, classification: &str) -> Result<String, String> {
    let mut q = job.world.query::<(&Node, &SystemInfo)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, s)| s.classification.eq_ignore_ascii_case(classification.trim())).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => {
            let name = building::CLASSIFICATIONS.iter().find(|(c, _)| c.eq_ignore_ascii_case(classification.trim())).map(|c| c.0).unwrap_or(classification.trim()).to_string();
            system(job, &name, classification)
        }
    }
}

/// Where a length typed while a line is being drawn puts the next point: that far from
/// the last point toward the pointer, east when the pointer is on the point (BIMCraft
/// `draw.rs:304`, `toward`).
pub fn toward(last: P2, pointer: P2, length_ft: f64) -> P2 {
    let (dx, dy) = (pointer[0] - last[0], pointer[1] - last[1]);
    let d = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if d > crate::check::FIT_TOLERANCE_FT { (dx / d, dy / d) } else { (1.0, 0.0) };
    // A direction that is east, west, north or south to within a hundredth of a
    // percent is that direction exactly. The pointer held "on the angle" is on it to
    // a dot of the screen and no better; kept to four places the difference was
    // rounded away with everything else, and kept finely (below) it would lean a wall
    // typed 23 ft 11 in north by a few hundred-thousandths of a foot and move its
    // face area in the fourth place (seen in `demos/planw.sh` on the first build
    // with the finer place).
    let square = |u: f64| if u.abs() < 1.0e-4 { 0.0 } else { u };
    let (ux, uy) = (square(ux), square(uy));
    let n = (ux * ux + uy * uy).sqrt();
    let (ux, uy) = if n > 0.0 { (ux / n, uy / n) } else { (1.0, 0.0) };
    // Kept to a hundred-millionth of a foot. It was kept to four places, and 28 ft 4 in
    // became 28.3333: a floor 41 ft 3 in by 28 ft 4 in was shown as 1,168.75 SF, held as
    // 1,168.7486, and at $6.00 came to $7,012.49 where a calculator says $7,012.50
    // (J26A-3). A place is tidied so that 0.1 + 0.2 is not 0.30000000000000004, and no
    // further than that: what is typed has to multiply to the cent.
    let round = |x: f64| (x * 1e8).round() / 1e8 + 0.0;
    [round(last[0] + ux * length_ft), round(last[1] + uy * length_ft)]
}

/// A low-slope area drawn from its outline (call 139): a roof of its own under the job,
/// and under it the area, its outline traced on the plan, its pitch, and how high its
/// deck is. Its first edge is the edge its pitch is measured from.
pub fn low_slope(job: &mut Job, name: &str, points: &[P2], pitch: &str, deck_height: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        if points.len() < 3 {
            return Err(format!("a low-slope area is an outline of 3 corners or more; {} given", points.len()));
        }
        let roof = job.add(&root, Kind::Roof, name)?;
        let area = job.add(&roof, Kind::LowSlopeArea, "field")?;
        let outline = Outline { points: how.val(points.to_vec(), units::FT), eave: how.val([points[0], points[1]], units::FT), eave_height: how.length(deck_height)? };
        check::outline(&area, Kind::LowSlopeArea, &outline)?;
        let p = Pitch(how.val(units::parse_pitch(pitch)?, units::PITCH));
        check::pitch(&area, Kind::LowSlopeArea, &p)?;
        if p.0.value.is_some_and(|x| x > 2.0) {
            return Err(format!("{area}: a pitch of {pitch} is a steep roof, not a low-slope one (a low-slope roof is 2:12 or less); draw it with `roof`"));
        }
        job.put(&area, outline)?;
        job.put(&area, p)?;
        Ok(area)
    })
}

/// The roof drains on a low-slope area and how thick its tapered insulation is at them
/// (call 139). Given again, they replace the drains it had. Each has to be on the area.
pub fn drains(job: &mut Job, area: &str, at: &[P2], at_drain: &str, how: Entered) -> Result<(), String> {
    all_or_nothing(job, |job| {
        let kind = job.kind(area).ok_or_else(|| format!("there is no {area} in this job"))?;
        if kind != Kind::LowSlopeArea {
            return Err(format!("{area} is {kind:#}; only a low-slope area takes roof drains"));
        }
        let e = job.find(area).ok_or_else(|| format!("there is no {area} in this job"))?;
        let ring = job.world.get::<Outline>(e).and_then(|o| o.points_ft()).ok_or_else(|| format!("{area} has no outline to put drains on"))?;
        let ring: Vec<plumbline_roofing::Point> = ring.iter().map(|p| plumbline_roofing::Point::new(p[0], p[1])).collect();
        let corners: Vec<P2> = ring.iter().map(|p| [p.x, p.y]).collect();
        for d in at {
            if crate::taper::on_edge(*d, &corners) {
                return Err(format!("the drain at {},{} is on the edge of {area}: {}", d[0], d[1], crate::taper::ON_EDGE));
            }
            if !plumbline_roofing::inside(plumbline_roofing::Point::new(d[0], d[1]), &ring) {
                return Err(format!("the drain at {},{} is not on {area}: a drain goes inside the area's outline", d[0], d[1]));
            }
        }
        let t = Taper { drains: how.val(at.to_vec(), units::FT), at_drain: how.length(at_drain)? };
        check::taper(area, kind, &t)?;
        job.put(area, t)?;
        Ok(())
    })
}
