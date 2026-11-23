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
pub mod stretch;

pub use history::{History, UNDO_LIMIT};
pub use stretch::{move_wall, Followed, Moved};
pub use join::{connect, connect_at, fit_all, fitting, split, Fitted, PassedOver};
pub(crate) use measure::takes_among;
pub use measure::{measure, measure_on_line, takes, Measured, Reading};

use crate::building::{self, InSystem, SystemInfo};
use crate::components::*;
use crate::check;
use crate::file;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Status, Val};
use crate::{Stood, Item, Level, Opening, OpeningType, Placed, Segment, SegmentType, Wall, WallType, P3};

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
            // What was made on the way and is now taken back has no path to be known
            // by: a refusal that began "wall-0004: its type is ..." named a wall that is
            // not in the job (M21-10, J21-6). Such a path is said as "the new wall".
            let made: Vec<(String, Kind)> = {
                let mut q = job.world.query::<(&Node, &Kind)>();
                q.iter(&job.world).map(|(n, k)| (n.path.clone(), *k)).collect()
            };
            match file::load(&before) {
                Ok((back, _)) => *job = back,
                Err(worse) => return Err(format!("{why}; and the job could not be put back as it was ({worse})")),
            }
            let mut why = why;
            for (path, kind) in made {
                if why.contains(&path) && job.find(&path).is_none() {
                    // "footing-0006 footing depth: ..." is "the new footing, its footing
                    // depth: ...": the value keeps the name the screen gives it.
                    // "type-0006 footing type width: ..." is "the new footing type width: ...".
                    why = why.replace(&format!("{path} {} ", kind.said()), &format!("the new {0}, its {0} ", kind.said()));
                    if kind == Kind::Type {
                        why = why.replace(&format!("{path} "), "the new ");
                    }
                    why = why.replace(&path, &format!("the new {}", kind.said()));
                }
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
                let lower = text.to_ascii_lowercase();
                let number = ["lb/ft", "lbs/ft", "plf", "#/ft"].iter().find_map(|u| lower.strip_suffix(u)).ok_or_else(|| format!("{text:?} is a weight per foot with no unit: write it like \"18 lb/ft\""))?;
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
fn type_is_free_for(job: &mut Job, type_path: &str, wanted: Kind) -> Result<(), String> {
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

/// An item: a thing that can be placed any number of times, as its maker's sheet
/// describes it. It is kept with the job's types, under the name it goes by.
pub fn item(job: &mut Job, name: &str, item: Item) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        if name.trim().is_empty() {
            return Err("an item needs a name: what the thing is called".into());
        }
        let path = job.add(&root, Kind::Type, name.trim())?;
        building::check_item(&path, Kind::Type, &item)?;
        job.put(&path, item)?;
        Ok(path)
    })
}

/// What an item is drawn as, changed: a form by its name, "box" for the box of its
/// sizes, or nothing, which leaves it to what its name says. Every one of it already
/// placed is drawn the new way: what it is drawn as is the item's, not the thing's.
pub fn item_drawn_as(job: &mut Job, item: &str, said: Option<&str>) -> Result<(), String> {
    all_or_nothing(job, |job| {
        let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
        let mut it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{item} is not an item"))?;
        it.form = match said.map(str::trim).filter(|s| !s.is_empty()) {
            Some(said) => {
                building::forms::drawn_as(said)?;
                Some(said.to_lowercase())
            }
            None => None,
        };
        building::check_item(item, Kind::Type, &it)?;
        job.put(item, it)
    })
}

/// Which trade's item it is, changed: one of the sixteen divisions, or none.
pub fn item_trade(job: &mut Job, item: &str, division: Option<crate::components::Division>) -> Result<(), String> {
    all_or_nothing(job, |job| {
        let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
        let mut it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{item} is not an item"))?;
        it.division = division;
        job.put(item, it)
    })
}

/// One of an item, set down on a level: its middle at `at` on the plan, its underside
/// `offset` above the floor. It is as big as its item says and as sure as that; where
/// the item's size is not all known it is placed with no size, has no shape, and says
/// which of its sizes is missing. Where it stands is as sure as `how`.
pub fn place_item(job: &mut Job, storey: &str, item: &str, at: P2, offset: &str, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let offset = how.length(offset)?;
        let item = item_by_name(job, item)?;
        place_item_in(job, storey, &item, at, offset, how)
    })
}

/// An item by its number, or by its name in any capitals (call 167). Two items of one name are
/// refused with both; what names no item is left as typed, for the caller to refuse.
fn item_by_name(job: &mut Job, typed: &str) -> Result<String, String> {
    let mut q = job.world.query::<(&Node, &Item)>();
    let mut named: Vec<(String, String)> = q.iter(&job.world).filter(|(n, _)| n.path == typed || n.name.eq_ignore_ascii_case(typed.trim())).map(|(n, _)| (n.path.clone(), n.name.clone())).collect();
    named.sort();
    if let Some(one) = named.iter().find(|(path, _)| path == typed) {
        return Ok(one.0.clone());
    }
    match named.as_slice() {
        [] => Ok(typed.to_string()),
        [(path, _)] => Ok(path.clone()),
        many => Err(format!("{} items are called {:?}: {}; give the one to set down by its number", many.len(), typed.trim(), many.iter().map(|(path, name)| format!("{name} ({path})")).collect::<Vec<_>>().join(", "))),
    }
}

/// One of an item set down, within a change that is all or nothing already.
fn place_item_in(job: &mut Job, storey: &str, item: &str, at: P2, offset: Val<Len>, how: Entered) -> Result<String, String> {
    storey_of(job, storey)?;
    let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
    let (name, it) = match (job.world.get::<Node>(e), job.world.get::<Item>(e)) {
        (Some(n), Some(it)) => (n.name.clone(), it.clone()),
        _ => return Err(format!("{item} is not an item: an item is made from a maker's sheet, or entered by its sizes")),
    };
    let size = match it.size_ft() {
        Ok((feet, status, source)) => Val { value: Some(feet), unit: "ft".into(), status, source: format!("{name}: {source}") },
        Err(why) => Val::unknown("ft", &format!("{name}: {why}")),
    };
    let placed = Placed { what: Val { value: Some(name.clone()), unit: String::new(), status: Status::Known, source: format!("the item {item}") }, at: how.val(at, "ft"), offset, angle: None, size, item: Some(item.to_string()) };
    let path = add_called(job, storey, it.category, &name, 2, true)?;
    building::check_placed(&path, it.category, &placed)?;
    job.put(&path, placed)?;
    Ok(path)
}

/// What a run of cabinet boxes came to (call 155).
#[derive(Clone, Debug, PartialEq)]
pub struct Cabinets {
    /// Each box set: its path, its code, the middle of its base on the plan, and how wide it is.
    pub boxes: Vec<(String, String, P2, f64)>,
    /// The countertop over the base boxes, with the middle of its base on the plan, if one was asked for.
    pub countertop: Option<(String, P2)>,
    /// Where the run starts along the wall's line from its first end, how long it is, and
    /// how much of the wall's face is left beyond it to its corner; all in feet.
    pub from: f64,
    pub run: f64,
    pub left: f64,
    /// The boxes hang on the wall (they are wall boxes), this far up in feet.
    pub hung: Option<f64>,
}

/// The box codes a job's tool chest has: its furnishings of Division 12 named by one word of
/// letters then figures (B18, SB36, W3030, T1884, F3).
pub fn box_codes(job: &mut Job) -> Vec<(String, String)> {
    let mut q = job.world.query::<(&Node, &Item)>();
    let mut found: Vec<(u64, String, String)> = q
        .iter(&job.world)
        .filter(|(n, it)| {
            let (letters, rest) = n.name.split_at(n.name.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(n.name.len()));
            it.category == Kind::Furnishing && it.division == crate::components::Division::of_number(12) && !letters.is_empty() && !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
        })
        .map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone(), n.name.clone()))
        .collect();
    found.sort();
    found.into_iter().map(|(_, path, name)| (path, name)).collect()
}

/// Boxes named in a list: words separated by spaces, each an item's code (B18, SB36) or,
/// in square brackets, any item of the job by its name, which may have a space or a comma
/// in it: `B18 [Dishwasher] SB36 [Range, 30 in]` is four boxes (call 157).
pub fn box_list(text: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = text.trim_start();
    while !rest.is_empty() {
        if let Some(inner) = rest.strip_prefix('[') {
            let Some(close) = inner.find(']') else {
                return Err(format!("a name in square brackets is not closed: [{}", inner.trim_end()));
            };
            let name = inner[..close].trim();
            if name.is_empty() {
                return Err("a pair of square brackets has no name in it".into());
            }
            out.push(name.to_string());
            rest = inner[close + 1..].trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            if rest[..end].contains('[') || rest[..end].contains(']') {
                return Err(format!("{:?} has a square bracket in the middle of it: a name with a space or a comma goes whole in brackets, like [Range, 30 in]", &rest[..end]));
            }
            out.push(rest[..end].to_string());
            rest = rest[end..].trim_start();
        }
    }
    Ok(out)
}

/// The most a thing can stand and still have a countertop laid over it: a base box's height.
const COUNTER_HEIGHT_FT: f64 = 34.5 / 12.0;

/// A run of cabinet boxes set side by side along the face `wall` has on `room`'s side,
/// backs to the wall, from `from` along the wall's line from its first end, each turned to
/// the wall. Base and tall boxes stand on the floor; wall boxes hang 54 in up (Assumed).
/// An item that is no box (a dishwasher, a range, named in the list by its name) stands on
/// the floor, back to the wall, as a base box does. Each is one of its item, as sure as
/// the item, and carries one cost line (its item's name, 1 EA, no price). With
/// `countertop`, one countertop over the base boxes and whatever else stands no taller than
/// they do: as long as they are, 25 1/2 in deep, 1 1/2 in thick, 34 1/2 in up (Assumed).
/// Refused, the job as it was: a code the tool chest has not, a wall that is to demolish, a
/// run past the room's corner or starting inside it, a run across a door, a box across a
/// window it would stand in front of (a wall box or a tall box; a base box where the sill is
/// below its top), a box that would stand in another thing set in place (call 157).
#[allow(clippy::too_many_arguments)]
pub fn cabinets(job: &mut Job, storey: &str, wall: &str, room: &str, from: &str, codes: &[String], countertop: bool, how: Entered) -> Result<Cabinets, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if codes.is_empty() {
            return Err("a run needs boxes: give their codes, like \"B18 SB36 B24\"".into());
        }
        let tol = crate::check::FIT_TOLERANCE_FT;
        let ft = |x: f64| units::feet_as_ft_in(x, 16);
        let start = how.length(from)?.feet().ok_or("where the run starts is not a length")?;
        if start < 0.0 {
            return Err("a run starts so far along the wall from its first end: that is not less than nothing".into());
        }
        // The boxes, by their codes (or any item by its name), with their sizes.
        struct Boxed {
            path: String,
            name: String,
            size: P3,
            /// A box of the tool chest, named by a code: wall boxes start with W, tall ones with T.
            code: bool,
            division: Option<crate::components::Division>,
        }
        let chest = box_codes(job);
        let mut boxes: Vec<Boxed> = Vec::new();
        for code in codes {
            let named = chest.iter().find(|(_, name)| name.eq_ignore_ascii_case(code.trim())).map(|(p, n)| (p.clone(), n.clone(), true)).or_else(|| {
                let mut q = job.world.query::<(&Node, &Item)>();
                let mut all: Vec<(u64, String, String)> = q.iter(&job.world).filter(|(n, _)| n.name.eq_ignore_ascii_case(code.trim())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone(), n.name.clone())).collect();
                all.sort();
                all.into_iter().next().map(|(_, p, n)| (p, n, false))
            });
            let Some((path, name, code)) = named else {
                let have = chest.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join(" ");
                return Err(if have.is_empty() { format!("{code} is not in the tool chest, and it has no boxes: bring the starter items in first") } else { format!("{code} is not in the tool chest. It has: {have}") });
            };
            let e = job.find(&path).ok_or_else(|| format!("{path} is not in the job"))?;
            let it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
            let (size, _, _) = it.size_ft().map_err(|why| format!("{name}: {why}, so it cannot be set"))?;
            boxes.push(Boxed { path, name, size, code, division: it.division });
        }
        let hangs = |b: &Boxed| b.code && b.name.starts_with('W');
        // What a countertop goes over: what is not hung and stands no taller than a base box.
        let covered = |b: &Boxed| !hangs(b) && b.size[2] <= COUNTER_HEIGHT_FT + tol;
        let wall_boxes = boxes.iter().filter(|b| hangs(b)).count();
        if wall_boxes != 0 && wall_boxes != boxes.len() {
            return Err("wall boxes hang over the counter and base boxes stand under it: set them as two runs".into());
        }
        let hung = (wall_boxes != 0).then_some(54.0 / 12.0);
        if countertop && hung.is_some() {
            return Err("a countertop goes over base boxes, and a run of wall boxes has none".into());
        }
        if countertop && !boxes.iter().any(&covered) {
            return Err("a countertop goes over base boxes, and this run has none".into());
        }
        // A wall that is to demolish is no wall to set boxes against.
        if crate::components::demolished(&mut job.world).contains(wall) {
            let called = job.find(wall).and_then(|e| job.world.get::<Node>(e)).map(|n| format!("{} ({wall})", n.name)).unwrap_or_else(|| wall.to_string());
            return Err(format!("{called} is to demolish: boxes set against a wall that is coming down would be left standing with nothing behind them. Set the run along a wall that stays"));
        }
        let face = building::wall_face(job, storey, wall, room)?;
        let run: f64 = boxes.iter().map(|b| b.size[0]).sum();
        let end = start + run;
        let Some(&(_, hi)) = face.clear.iter().find(|(lo, hi)| start >= lo - tol && start <= hi + tol) else {
            let said = face.clear.iter().map(|(lo, hi)| format!("{} to {}", ft(*lo), ft(*hi))).collect::<Vec<_>>().join(", ");
            return Err(format!("{} does not have a clear face at {} along it: its face on {} runs clear from {said}", face.wall_called, ft(start), face.room_called));
        };
        if end > hi + tol {
            return Err(format!("a run of {} from {} reaches {} along {}, {} past the corner of {} at {}", ft(run), ft(start), ft(end), face.wall_called, ft(end - hi), face.room_called, ft(hi)));
        }
        if let Some((door, a, b)) = face.doors.iter().find(|(_, a, b)| start < *b - tol && *a < end - tol) {
            return Err(format!("a run of {} from {} crosses {door}, which is in {} from {} to {}: a door is no place for boxes", ft(run), ft(start), face.wall_called, ft(*a), ft(*b)));
        }
        // A window the box would stand in front of: it reaches from the floor (or from where
        // it hangs) up to its top, and the window from its sill up to its head.
        let mut at = start;
        for b in &boxes {
            let (low, high) = (hung.unwrap_or(0.0), hung.unwrap_or(0.0) + b.size[2]);
            for (window, called, from, to, sill, head) in &face.windows {
                if at < *to - tol && *from < at + b.size[0] - tol && low < *head - tol && *sill < high - tol {
                    let hangs_or_stands = if hung.is_some() { "hangs" } else { "stands" };
                    return Err(format!(
                        "{} {hangs_or_stands} from {} to {} along {} and from {} up to {} high, across {called} ({window}), which is in {} from {} to {} and from {} up to {} high{}",
                        b.name,
                        ft(at),
                        ft(at + b.size[0]),
                        face.wall_called,
                        ft(low),
                        ft(high),
                        face.wall_called,
                        ft(*from),
                        ft(*to),
                        ft(*sill),
                        ft(*head),
                        if hung.is_none() && covered(b) && *sill > low + tol { format!(": a base box fits under a window only where the sill is at or above its top ({}), and this sill is lower", ft(high)) } else { if hung.is_some() { ": a wall box does not hang across a window".to_string() } else { ": a tall box does not stand in front of a window".to_string() } }
                    ));
                }
            }
            at += b.size[0];
        }
        // Each box's middle on the plan: along the wall, then back from the face into the room.
        let (u, into) = (face.along, face.into);
        let spot = |along: f64, out: f64| -> P2 { [face.start[0] + u[0] * along + into[0] * out, face.start[1] + u[1] * along + into[1] * out] };
        // Turned so its back (the +y of its own frame) is against the wall.
        let turned = {
            let mut a = into[0].atan2(-into[1]).to_degrees();
            if a <= -180.0 {
                a += 360.0;
            }
            (a * 1.0e6).round() / 1.0e6
        };
        let sure = how.status.max(face.status);
        let by = format!("{}, {} along {} against {} ({})", "set in a run", ft(start), face.wall_called, face.room_called, how.source);
        let assumed = Entered { status: how.status.max(Status::Assumed), source: "" };
        let to_wall = |job: &mut Job, path: &str| -> Result<(), String> {
            let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
            let mut p = job.world.get::<Placed>(e).cloned().ok_or_else(|| format!("{path} has no position"))?;
            p.angle = if turned == 0.0 && sure == Status::Known { None } else { Some(Val { value: Some(turned), unit: "deg".into(), status: sure, source: format!("turned to {}, the wall it backs onto", face.wall_called) }) };
            building::check_placed(path, job.kind(path).unwrap_or(Kind::Furnishing), &p)?;
            job.put(path, p)
        };
        let mut set: Vec<(String, String, P2, f64)> = Vec::new();
        let mut along = start;
        let mut base_span: Option<(f64, f64)> = None;
        for b in &boxes {
            let (item, code, size) = (&b.path, &b.name, b.size);
            let mid = spot(along + size[0] / 2.0, face.off + size[1] / 2.0);
            let offset = match hung {
                Some(up) => Val { value: Some(units::Len::One(up)), unit: "ft".into(), status: assumed.status, source: "wall boxes hang with their bottoms 54 in up, 18 in over a 36 in counter; the maker's catalog decides".into() },
                None => how.length("0 ft")?,
            };
            let path = place_item_in(job, storey, item, mid, offset, Entered { status: sure, source: &by })?;
            to_wall(job, &path)?;
            job.add_line(&path, code, "1", Some(crate::Class::Material), false)?;
            job.set_division(&path, Some(code), b.division.or(crate::components::Division::of_number(12)).ok_or("Division 12 is not there")?)?;
            if covered(b) {
                base_span = Some(match base_span {
                    Some((a, _)) => (a, along + size[0]),
                    None => (along, along + size[0]),
                });
            }
            set.push((path, code.clone(), mid, size[0]));
            along += size[0];
        }
        // A box that stands in another thing set in place (a box of another run, an
        // appliance) is refused: the clash test is the one `clashes` reports with, asked of
        // the job as it now stands, and only what a box of this run is in is looked at.
        let mine: std::collections::HashSet<&str> = set.iter().map(|(p, _, _, _)| p.as_str()).collect();
        if let Ok(found) = building::clash::clashes(job, None) {
            let is_set_thing = |k: Kind| matches!(k, Kind::Furnishing | Kind::Equipment | Kind::Fixture);
            for c in found.found.iter().filter(|c| c.how == building::clash::ClashKind::Hard && !c.through) {
                let (this, other_name, other_kind, other_path) = match (mine.contains(c.a.as_str()), mine.contains(c.b.as_str())) {
                    (true, false) => (&c.a, &c.b_name, c.b_kind, &c.b),
                    (false, true) => (&c.b, &c.a_name, c.a_kind, &c.a),
                    _ => continue,
                };
                if !is_set_thing(other_kind) {
                    continue;
                }
                let name = set.iter().find(|(p, _, _, _)| p == this).map(|(_, code, _, _)| code.clone()).unwrap_or_default();
                return Err(format!(
                    "{name} would stand in {other_name} ({other_path}){}: set the run clear of it. A run that turns an inside corner starts off the corner by what the other run fills (24 in for base boxes)",
                    c.by.map(|d| format!(" by {} in", crate::view::num((d * 12.0 * 100.0).round() / 100.0))).unwrap_or_default()
                ));
            }
        }
        // The countertop over the base boxes.
        let top = if countertop {
            let (a, b) = base_span.ok_or("a countertop goes over base boxes, and this run has none")?;
            if boxes.iter().any(|x| !covered(x) && !hangs(x)) {
                let mut at = start;
                let mut between: Option<&Boxed> = None;
                for x in &boxes {
                    if !covered(x) && at > a + tol && at + x.size[0] < b - tol {
                        between = Some(x);
                    }
                    at += x.size[0];
                }
                if let Some(x) = between {
                    return Err(if x.code { "a tall box stands in the middle of the base boxes, and a countertop does not go over it: set the countertop by hand".to_string() } else { format!("{} stands taller than a countertop in the middle of the base boxes, and a countertop does not go over it: set the countertop by hand", x.name) });
                }
            }
            let (long, deep, thick) = (b - a, 25.5 / 12.0, 1.5 / 12.0);
            let mid = spot((a + b) / 2.0, face.off + deep / 2.0);
            let said = Entered { status: assumed.status, source: "a countertop over the base boxes: as long as they are, 25 1/2 in deep (a 1 1/2 in overhang at the front), 1 1/2 in thick, 34 1/2 in up; the maker's catalog decides" };
            let path = add_called(job, storey, Kind::Furnishing, "Countertop", 2, true)?;
            let placed = Placed { what: how.val("Countertop".to_string(), ""), at: Entered { status: sure, source: &by }.val(mid, "ft"), offset: Val { value: Some(units::Len::One(34.5 / 12.0)), unit: "ft".into(), status: said.status, source: said.source.into() }, angle: None, size: said.val([long, deep, thick], "ft"), item: None };
            building::check_placed(&path, Kind::Furnishing, &placed)?;
            job.put(&path, placed)?;
            to_wall(job, &path)?;
            // It carries its own line, by its area, to be priced (call 162); `price-item
            // Countertop` finds countertops by what they are.
            job.add_line(&path, "Countertop", "plan_area", Some(crate::Class::Material), false)?;
            job.set_division(&path, Some("Countertop"), crate::components::Division::of_number(12).ok_or("Division 12 is not there")?)?;
            Some((path, mid))
        } else {
            None
        };
        Ok(Cabinets { boxes: set, countertop: top, from: start, run, left: hi - end, hung })
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

/// Room lines along `points` (in feet, on the plan): each pair of points that follow one
/// another is one room line. A room line bounds the rooms of its level where no wall
/// stands (call 152; BIMCraft's `room_separator` command, `commands.rs:6938`, one line
/// for each pair of points).
pub fn room_lines(job: &mut Job, storey: &str, points: &[P2], how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if points.len() < 2 {
            return Err("a room line needs at least 2 points".into());
        }
        let mut made = Vec::new();
        for pair in points.windows(2) {
            let line = crate::RoomLine { line: how.val([pair[0], pair[1]], "ft") };
            let path = add_called(job, storey, Kind::RoomLine, "room line", 1, false)?;
            building::check_room_line(&path, Kind::RoomLine, &line)?;
            job.put(&path, line)?;
            made.push(path);
        }
        Ok(made)
    })
}

/// What to say of room lines just drawn, each end that meets no wall and no other room
/// line of its level: "its end at x,y meets no wall and no room line: it parts nothing
/// until it does". One sentence for each such end; none when every end meets something.
pub fn room_lines_unreached_said(job: &mut Job, made: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for path in made {
        for p in building::room_line_unreached(job, path) {
            out.push(format!("{path}: its end at {},{} meets no wall and no room line: it parts nothing until it does", crate::view::num(p[0]), crate::view::num(p[1])));
        }
    }
    out
}

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
        if !degrees.is_finite() || degrees == 0.0 {
            return Err("how far to turn it is not a figure, or is nothing".into());
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

/// Call an item something else. Two items cannot share a name, however it is typed;
/// each thing set down from it is called what the item is. Returns how many were.
pub fn rename_item(job: &mut Job, path: &str, name: &str) -> Result<usize, String> {
    all_or_nothing(job, |job| {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        if job.world.get::<crate::Item>(e).is_none() {
            return Err(format!("{path} is not an item"));
        }
        let name = name.trim();
        if name.is_empty() {
            return Err("an item needs a name: what the thing is called".into());
        }
        let was = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_default();
        if was == name {
            return Err(format!("it is called {name:?} already"));
        }
        let taken = {
            let mut q = job.world.query::<(&Node, &crate::Item)>();
            q.iter(&job.world).any(|(n, _)| n.path != path && crate::check::same_name(&n.name, name))
        };
        if taken {
            return Err(format!("another item is called {name:?} already"));
        }
        if let Some(mut node) = job.world.get_mut::<Node>(e) {
            node.name = name.to_string();
        }
        let placed: Vec<(String, Kind, Placed)> = {
            let mut q = job.world.query::<(&Node, &Kind, &Placed)>();
            q.iter(&job.world).filter(|(_, _, p)| p.item.as_deref() == Some(path)).map(|(n, k, p)| (n.path.clone(), *k, p.clone())).collect()
        };
        let count = placed.len();
        for (at, k, mut p) in placed {
            p.what.value = Some(name.to_string());
            building::check_placed(&at, k, &p)?;
            job.put(&at, p)?;
            // One still called by the item's old name is called by the new one: it was
            // told "the one set down is called that too" while every list went on
            // naming it as before (the pictures of `itemneww`, read by a worker). One a
            // person has given a name of its own keeps it.
            if let Some(mut node) = job.find(&at).and_then(|e| job.world.get_mut::<Node>(e)) {
                if node.name == was {
                    node.name = name.to_string();
                }
            }
        }
        Ok(count)
    })
}

/// Every thing set down from an item is as big as the item says: after the item's
/// sizes change, each follows. Returns the path of each that changed size.
pub(crate) fn placed_follow(job: &mut Job, item_path: &str, item_name: &str, item: &crate::Item) -> Result<Vec<String>, String> {
    let placed: Vec<(String, Kind, Placed)> = {
        let mut q = job.world.query::<(&crate::Node, &Kind, &Placed)>();
        q.iter(&job.world).filter(|(_, _, p)| p.item.as_deref() == Some(item_path)).map(|(n, k, p)| (n.path.clone(), *k, p.clone())).collect()
    };
    let mut followed = Vec::new();
    for (at, k, mut p) in placed {
        let size = match item.size_ft() {
            Ok((feet, status, from)) => Val { value: Some(feet), unit: "ft".into(), status, source: format!("{item_name}: {from}") },
            Err(why) => Val::unknown("ft", &format!("{item_name}: {why}")),
        };
        if p.size == size {
            continue;
        }
        p.size = size;
        building::check_placed(&at, k, &p)?;
        job.put(&at, p)?;
        followed.push(at);
    }
    Ok(followed)
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

/// What `price_item` priced.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemPriced {
    /// The item's name.
    pub item: String,
    /// Each thing set down from the item whose line named for it was priced.
    pub priced: Vec<String>,
    /// Each that has no line of that name, and so took nothing.
    pub without: Vec<String>,
}

/// A unit cost on the cost line named for an item, on every one of the item set in the job
/// (a box set in a run carries one line named for its item): one price for one code
/// (call 157). `item` is the item's name or its path. Things set from it that have no line
/// of that name take nothing, and are listed. Nothing is set where none of the item has
/// been set down yet; that is for the caller to say.
pub fn price_item(job: &mut Job, item: &str, price: &str, status: Status, source: &str) -> Result<ItemPriced, String> {
    all_or_nothing(job, |job| {
        let found: Vec<(u64, String, String)> = {
            let mut q = job.world.query::<(&Node, &Item)>();
            q.iter(&job.world).filter(|(n, _)| n.path == item || n.name.eq_ignore_ascii_case(item.trim())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone(), n.name.clone())).collect()
        };
        // What is cut to its run and is no item of the tool chest (a countertop) is found by
        // what it is: every thing set in place of that name that is of no item (call 162).
        let by_what: Vec<(u64, String)> = {
            let mut q = job.world.query::<(&Node, &Placed)>();
            q.iter(&job.world).filter(|(_, p)| p.item.is_none() && p.what.value.as_deref().is_some_and(|w| w.eq_ignore_ascii_case(item.trim()))).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone())).collect()
        };
        let found_by_what = found.is_empty() && !by_what.is_empty();
        let (name, set): (String, Vec<String>) = match found.into_iter().min() {
            Some((_, path, name)) => {
                let mut q = job.world.query::<(&Node, &Placed)>();
                let mut all: Vec<(u64, String)> = q.iter(&job.world).filter(|(_, p)| p.item.as_deref() == Some(path.as_str())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone())).collect();
                all.sort();
                (name, all.into_iter().map(|(_, p)| p).collect())
            }
            None if !by_what.is_empty() => {
                let mut all = by_what;
                all.sort();
                let name = job.world.query::<&Placed>().iter(&job.world).find_map(|p| p.what.value.clone().filter(|w| w.eq_ignore_ascii_case(item.trim()))).unwrap_or_else(|| item.trim().to_string());
                (name, all.into_iter().map(|(_, p)| p).collect())
            }
            None => return Err(format!("{item} is not an item of this job: price takes an item's name, like B18")),
        };
        // A countertop is bought by the LF of its front edge (its width) or by the SF of its
        // top (its plan area): a price that says which puts the line on that figure; one that
        // says another unit is refused with the two it takes (call 167).
        let countertop = found_by_what;
        let figure: Option<&str> = match (countertop, price.rsplit_once('/')) {
            (true, Some((_, per))) => match per.trim().to_uppercase().as_str() {
                "LF" => Some("width"),
                "SF" => Some("plan_area"),
                _ => return Err(format!("{price:?} is a price per {}, and a countertop is priced by the LF of its front edge or by the SF of its top: write it like $80/LF or $62/SF", per.trim())),
            },
            _ => None,
        };
        let (mut priced, mut without) = (Vec::new(), Vec::new());
        for thing in set {
            let has = job.find(&thing).and_then(|e| job.world.get::<Cost>(e)).is_some_and(|c| c.lines.iter().any(|l| l.description == name));
            if has {
                if let Some(figure) = figure {
                    let on = job.find(&thing).and_then(|e| job.world.get::<Cost>(e)).and_then(|c| c.lines.iter().find(|l| l.description == name)).map(|l| l.quantity.clone());
                    if on.as_deref() != Some(figure) {
                        // A price does not carry from one unit to another: it is cleared, the
                        // line is put on the figure, and the price is entered in its unit.
                        job.set_on_line(&thing, "unit_cost", Some(&name), None, Status::Unknown, "the line changed units")?;
                        job.set_formula(&thing, Some(&name), figure, &[], status, source)?;
                    }
                }
                job.set_on_line(&thing, "unit_cost", Some(&name), Some(price), status, source)?;
                priced.push(thing);
            } else {
                without.push(thing);
            }
        }
        Ok(ItemPriced { item: name, priced, without })
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

/// The wall type of this thickness with no layers, or a new one named for it ("6 in wall").
/// A window draws with a thickness the person typed; this is the type that thickness is.
pub fn wall_type_for(job: &mut Job, thickness: &str, how: Entered) -> Result<String, String> {
    let wanted = how.length(thickness)?.feet().ok_or("the thickness is not a length this program converts")?;
    let mut q = job.world.query::<(&Node, &WallType)>();
    let mut found: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
    found.sort();
    match found.into_iter().next() {
        Some(path) => Ok(path),
        None => wall_type(job, &format!("{} wall", thickness.trim()), thickness, how),
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
                            job.world.entity_mut(e).remove::<Cost>();
                            line.push_str(&format!("; its {} taken off: {why}", crate::view::so_many(carried, "cost line")));
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
