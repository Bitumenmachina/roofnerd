//! Drawing and editing the building: the verbs a window or a command line uses to put a
//! wall, an opening, a run or a fixture into the job, to move and remove them, and to
//! take any of it back.
//!
//! PORTED CODE, in part. The rule each verb follows (a wall for each pair of points, an
//! opening taken along its wall, a run for each pair of points, a thing stood at a point,
//! a wall's openings moved and removed with it, all of a change or none of it) is ported
//! from BIMCraft, `crates/core/src/commands.rs` and `crates/core/src/engine.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from. `editing/history.rs` is ported whole.
//!
//! What was changed in the port, and nothing else:
//! * BIMCraft's commands take JSON, in millimetres, and build its `Element`s. These take
//!   typed lengths with their units and feet on the plan, and build this program's
//!   elements and components, each value with a status and a source.
//! * Nothing is given a value nobody entered. BIMCraft's `wall` gives a wall with no
//!   height its default, or holds it to the level above; its `opening_of` puts an opening
//!   with no place in the middle of its wall; its window type has a sill of 900 mm when
//!   none is said (`commands.rs:11209`); its `runs_of` puts a run with no height at the
//!   floor. Here each of those has to be said.
//! * A point for an opening that falls past an end of its wall is refused. BIMCraft's
//!   `along` takes it to the end.
//! * Not taken: arcs, closed runs, a run's end height and justification, families,
//!   attaching a wall's top or base, moving one end, moving or removing several at once.

pub mod history;
pub mod join;
pub mod measure;
pub mod snap;

pub use history::{History, UNDO_LIMIT};
pub use join::{connect, connect_at, fit_all, fitting, split, Fitted, PassedOver};
pub(crate) use measure::takes_among;
pub use measure::{measure, takes, Measured, Reading};

use crate::building::{self, InSystem, SystemInfo};
use crate::components::*;
use crate::file;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Status, Val};
use crate::{Level, Opening, OpeningType, Placed, Segment, SegmentType, Wall, WallType, P3};

/// How sure a thing being drawn is, and who says so. Everything a verb writes carries it.
#[derive(Clone, Copy)]
pub struct Entered<'a> {
    pub status: Status,
    pub source: &'a str,
}

impl Entered<'_> {
    fn val<T>(&self, value: T, unit: &str) -> Val<T> {
        Val { value: Some(value), unit: unit.into(), status: self.status, source: self.source.into() }
    }
    fn length(&self, typed: &str) -> Result<Val<Len>, String> {
        let (len, unit) = units::parse_length(typed)?;
        Ok(self.val(len, unit))
    }
    fn path(&self, path: &str) -> Val<String> {
        self.val(path.to_string(), "")
    }
}

/// Where a wall's top is: so tall, or held to another storey.
pub enum Top<'a> {
    Height(&'a str),
    Level(&'a str),
}

/// Where along its wall an opening's centre is: so far from the wall's first end, or the
/// point on the plan nearest which it goes.
pub enum OnWall<'a> {
    Offset(&'a str),
    Point(P2),
}

/// Do `f` to the job, all of it or none: if it fails, or leaves the job with a reference
/// that does not hold (a type that is not there, an opening off its wall's storey), the
/// job is put back as it was (BIMCraft's `transaction`, `engine.rs:648`: "all or
/// nothing"). The figures are worked out again before it returns.
pub fn all_or_nothing<T>(job: &mut Job, f: impl FnOnce(&mut Job) -> Result<T, String>) -> Result<T, String> {
    let before = file::save(job);
    match f(job).and_then(|v| building::check_whole(job).map(|()| v)) {
        Ok(v) => {
            job.derive();
            Ok(v)
        }
        Err(why) => {
            match file::load(&before) {
                Ok((back, _)) => *job = back,
                Err(worse) => return Err(format!("{why}; and the job could not be put back as it was ({worse})")),
            }
            Err(why)
        }
    }
}

/// A new element named for its kind, numbered from 1 among its kind on its storey:
/// "wall 1", "duct 3". (It was named by its number in the job's file, "wall 31", which
/// is how the file tells things apart and not how a builder does.)
pub(crate) fn add_numbered(job: &mut Job, parent: &str, kind: Kind) -> Result<String, String> {
    add_called(job, parent, kind, kind.stem(), 1, false)
}

/// A new element called `base`, told apart from every other thing on its storey by a
/// number counted from `first`. With `bare`, the first of a name has it with no number
/// ("Lavatory", then "Lavatory 2"); without, every one is numbered ("wall 1", "wall 2";
/// "trunk, piece 2").
pub(crate) fn add_called(job: &mut Job, parent: &str, kind: Kind, base: &str, first: u32, bare: bool) -> Result<String, String> {
    let base = base.trim();
    if base.is_empty() {
        return Err("a thing needs a name".into());
    }
    // Every element's parent, to find the storey a thing is on: the parent itself, or
    // the nearest storey above it, or failing that the parent.
    let (parents, names): (std::collections::HashMap<String, Option<String>>, Vec<(String, String)>) = {
        let mut q = job.world.query::<&Node>();
        let all: Vec<(String, Option<String>, String)> = q.iter(&job.world).map(|n| (n.path.clone(), n.parent.clone(), n.name.clone())).collect();
        (all.iter().map(|(p, up, _)| (p.clone(), up.clone())).collect(), all.into_iter().map(|(p, _, n)| (p, n)).collect())
    };
    let up = |from: &str| -> Vec<String> {
        let mut chain = vec![from.to_string()];
        while let Some(Some(next)) = parents.get(chain.last().map(String::as_str).unwrap_or("")) {
            if chain.len() > 64 {
                break;
            }
            chain.push(next.clone());
        }
        chain
    };
    let scope = up(parent).into_iter().find(|p| job.kind(p) == Some(Kind::Storey)).unwrap_or_else(|| parent.to_string());
    let taken: std::collections::HashSet<String> = names.iter().filter(|(p, _)| up(p).contains(&scope)).map(|(_, n)| n.clone()).collect();
    let name = if bare && !taken.contains(base) { base.to_string() } else { (first..).map(|n| format!("{base} {n}")).find(|n| !taken.contains(n)).unwrap_or_else(|| base.to_string()) };
    job.add(parent, kind, &name)
}

fn storey_of(job: &mut Job, storey: &str) -> Result<(), String> {
    match job.kind(storey) {
        Some(Kind::Storey) => Ok(()),
        Some(other) => Err(format!("{storey} is a {other:?}, not a storey")),
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
        let t = WallType { thickness: how.length(thickness)?, layers: Vec::new() };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_wall_type(&path, Kind::Type, &t)?;
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
        let t = SegmentType { category: kind, width: how.length(width)?, depth: depth.map(|d| how.length(d)).transpose()?, round: depth.is_none() };
        let path = job.add(&root, Kind::Type, name)?;
        building::check_segment_type(&path, Kind::Type, &t)?;
        job.put(&path, t)?;
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

/// A door or a window in a wall (BIMCraft's `opening_of`, `commands.rs:7473`). Given a
/// point, it goes where the point falls along the wall's line (`along`, `commands.rs:2426`);
/// a point that falls off either end is refused, not moved to the end. Unlike there, an
/// opening with no place said is not put in the middle, and a sill is always said.
pub fn opening(job: &mut Job, wall: &str, kind: Kind, type_path: &str, at: OnWall, sill: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        if !kind.is_opening() {
            return Err(format!("a {kind:?} is not a door or a window"));
        }
        let host = job.find(wall).and_then(|e| job.world.get::<Wall>(e).cloned()).ok_or_else(|| format!("{wall} is not a wall with a line"))?;
        let offset = match at {
            OnWall::Offset(typed) => how.length(typed)?,
            OnWall::Point(p) => {
                let [a, b] = host.line_ft().ok_or_else(|| format!("{wall}'s line is not known, so a point cannot be placed along it"))?;
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let length = (dx * dx + dy * dy).sqrt();
                let along = ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length;
                if !(0.0..=length).contains(&along) {
                    return Err(format!("the point ({}, {}) falls {} ft along {wall}, which is {} ft long: it is not on the wall", p[0], p[1], crate::view::figure(along, 2), crate::view::figure(length, 2)));
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
                    return Err(format!("the point ({}, {}) is {} ft off the middle of {wall}, which is {} ft thick: it is not on the wall", p[0], p[1], crate::view::figure(off, 2), crate::view::figure(half * 2.0, 2)));
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

/// Straight pieces of duct, pipe, conduit or cable along `points` (in feet), level at
/// `height` above the storey's floor, on `system` if one is named (BIMCraft's `runs_of`,
/// `commands.rs:8776`: one run for each pair of points that follow one another).
pub fn runs(job: &mut Job, storey: &str, kind: Kind, type_path: &str, points: &[P2], height: &str, system: Option<&str>, how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if !kind.is_segment() {
            return Err(format!("a {kind:?} is not a duct, a pipe, a conduit or a cable"));
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
        if !matches!(kind, Kind::Fixture | Kind::Equipment) {
            return Err(format!("a {kind:?} is not a fixture or a piece of equipment"));
        }
        let size = match size {
            Some(s) => how.val(s, "ft"),
            None => Val::unknown("ft", "size not entered"),
        };
        let placed = Placed { what: how.val(what.to_string(), ""), at: how.val(at, "ft"), offset: how.length(offset)?, angle: None, size };
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

/// How a floor's outline is given.
pub enum FloorBy<'a> {
    /// The outline the storey's walls enclose.
    Walls,
    /// Its corners, in feet on the plan.
    Points(&'a [P2]),
}

/// What `floor` made, and what whoever asked for it should know.
#[derive(Clone, Debug, PartialEq)]
pub struct Floored {
    pub path: String,
    pub notes: Vec<String>,
}

/// A floor of a type on a storey (BIMCraft `floor`, `commands.rs:7556`): by its corners,
/// or by the outline the storey's walls enclose. By its walls it is no surer than the
/// least sure of them, and its source says it is their outline. A floor on an outline a
/// floor of the storey already has is refused: it would be counted twice.
pub fn floor(job: &mut Job, storey: &str, type_path: &str, by: FloorBy, how: Entered) -> Result<Floored, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let called = job.find(storey).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap_or_else(|| storey.to_string());
        if job.find(type_path).and_then(|e| job.world.get::<crate::FloorType>(e)).is_none() {
            return Err(format!("{type_path} is not a floor type"));
        }
        let mut notes = Vec::new();
        let boundary = match by {
            FloorBy::Points(points) => {
                if points.len() < 3 {
                    return Err("a floor's outline has at least three corners".into());
                }
                how.val(points.to_vec(), "ft")
            }
            FloorBy::Walls => {
                let found = building::floor_outline(job, storey)?.ok_or_else(|| format!("no walls enclose an area on {called}: draw the walls round the floor first, or give the floor by its corners"))?;
                if found.courtyards > 0 {
                    return Err(format!(
                        "the walls on {called} enclose {} inside the floor's outline, and a floor here has no hole yet: give the floor by its corners, round {}",
                        if found.courtyards == 1 { "a courtyard".to_string() } else { format!("{} courtyards", found.courtyards) },
                        if found.courtyards == 1 { "it" } else { "them" }
                    ));
                }
                if found.others > 0 {
                    notes.push(format!("the walls on {called} enclose {} apart from this one, and this floor is the largest only: each of the others is a floor of its own, given by its corners", if found.others == 1 { "another outline".to_string() } else { format!("{} other outlines", found.others) }));
                }
                Val { value: Some(found.boundary), unit: "ft".into(), status: how.status.max(found.status), source: format!("the outline of the walls on {called} ({})", how.source) }
            }
        };
        // A floor already on this very outline.
        let same = |a: &[P2], b: &[P2]| a.len() == b.len() && (0..a.len()).any(|shift| (0..a.len()).all(|i| (a[i][0] - b[(i + shift) % b.len()][0]).abs() < 1.0e-3 && (a[i][1] - b[(i + shift) % b.len()][1]).abs() < 1.0e-3));
        let mine = boundary.value.clone().unwrap_or_default();
        let there = {
            let mut q = job.world.query::<(&Node, &crate::Floor)>();
            q.iter(&job.world).find(|(n, f)| n.parent.as_deref() == Some(storey) && f.boundary_ft().is_some_and(|b| same(&b, &mine))).map(|(n, _)| n.name.clone())
        };
        if let Some(there) = there {
            return Err(format!("{there} is already on this outline on {called}: a second floor there would be counted twice"));
        }
        let floor = crate::Floor { type_name: how.path(type_path), boundary, offset: None };
        let path = add_numbered(job, storey, Kind::Floor)?;
        building::check_floor(&path, Kind::Floor, &floor)?;
        job.put(&path, floor)?;
        Ok(Floored { path, notes })
    })
}

/// A room: a point inside it (in feet). Its boundary is found from the walls round the
/// point (BIMCraft's `room` command, `commands.rs:7909`: a room is placed by a point).
pub fn room(job: &mut Job, storey: &str, name: &str, at: P2, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let room = crate::Room { point: how.val(at, "ft"), number: None };
        let path = job.add(storey, Kind::Room, name)?;
        building::check_room(&path, Kind::Room, &room)?;
        job.put(&path, room)?;
        Ok(path)
    })
}

/// Move a wall, a run, a fixture, a piece of equipment, a fitting or a room's point by
/// so many feet east and north. A wall's doors and windows go with it: they are placed
/// along it. What was moved is no surer than the move: it takes `how`'s status if that
/// is worse than its own.
pub fn move_by(job: &mut Job, path: &str, east: f64, north: f64, how: Entered) -> Result<(), String> {
    all_or_nothing(job, |job| {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = job.kind(path).ok_or_else(|| format!("{path} has no kind"))?;
        if !(east.is_finite() && north.is_finite()) {
            return Err("the distance to move by is not a usable number".into());
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
        } else if let Some(mut r) = job.world.get::<crate::Room>(e).cloned() {
            let at = r.point.value.ok_or_else(|| format!("{path}'s point is not known, so it cannot be moved"))?;
            let (status, source) = moved(r.point.status, &r.point.source);
            r.point = Val { value: Some(shift(&r.point.unit, at)?), unit: r.point.unit.clone(), status, source };
            building::check_room(path, kind, &r)?;
            job.put(path, r)?;
        } else {
            return Err(format!("a {kind:?} is not moved this way; a wall, a run, a fixture, equipment, a fitting or a room is"));
        }
        Ok(())
    })
}

/// A source after something was done to the value it stands on: where the value came
/// from, then each thing done to it since and on whose word. It loses nothing.
pub(crate) fn after(source: &str, done: &str, by: &str) -> String {
    // Nothing a source says is taken out of it: what a person wrote and a note this
    // wrote look alike, and a guess at which is which would cut a person's words. The
    // one thing not done is saying the same thing twice running: a wall moved four
    // times on one person's word says so once.
    let note = format!("; {done} ({by})");
    if source.ends_with(&note) {
        source.to_string()
    } else {
        format!("{source}{note}")
    }
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
            return Err(format!("{path} is solved from {}'s footprint and would come back: change the footprint, or remove {}", solved.from, solved.from));
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
        if kind == Kind::Material {
            let mut q = job.world.query::<(&Node, &MadeOf)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(_, m)| m.0.value.as_ref().is_some_and(|v| v == path || crate::check::same_name(v, &name))).map(|(n, _)| n.path.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is what {} {} made of: give {} another material first", listed(&by), if by.len() == 1 { "is" } else { "are" }, if by.len() == 1 { "it" } else { "them" }));
            }
        }
        if kind == Kind::Condition {
            let mut q = job.world.query::<(&Node, &Condition)>();
            let mut by: Vec<String> = q.iter(&job.world).filter(|(n, c)| n.path != path && c.0.value.as_ref().is_some_and(|v| v == path || crate::check::same_name(v, &name))).map(|(n, _)| n.path.clone()).collect();
            by.sort();
            if !by.is_empty() {
                return Err(format!("{path} is the condition of {}: give {} another first", listed(&by), if by.len() == 1 { "it" } else { "them" }));
            }
        }
        let mut gone = Vec::new();
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

/// The wall type of this thickness with no layers, or a new one named for it ("wall 6 in").
/// A window draws with a thickness the person typed; this is the type that thickness is.
pub fn wall_type_for(job: &mut Job, thickness: &str, how: Entered) -> Result<String, String> {
    let wanted = how.length(thickness)?.feet().ok_or("the thickness is not a length this program converts")?;
    let mut q = job.world.query::<(&Node, &WallType)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => wall_type(job, &format!("wall {}", thickness.trim()), thickness, how),
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
        None => floor_type(job, &format!("floor {}", thickness.trim()), thickness, how),
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
        None => opening_type(job, &format!("{} {} by {}", kind.stem(), width.trim(), height.trim()), kind, width, height, how),
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
    let round = |x: f64| (x * 1e4).round() / 1e4 + 0.0;
    [round(last[0] + ux * length_ft), round(last[1] + uy * length_ft)]
}
