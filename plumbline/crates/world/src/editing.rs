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

pub mod carry;
pub mod connections;
pub mod copies;
pub mod find;
pub mod history;
pub mod join;
pub mod measure;
pub mod shapes;
pub mod slide;
pub mod snap;
pub mod stretch;
pub mod trying;

pub use connections::{connection_add, connection_row, connection_row_facing, connection_set, connection_set_how, connection_take, connections_outside_their_boxes, connections_standing, item_connections_worked, item_rows, ConnectionDone, ConnectionsFollowed, Near, Standing, StandsHow, StandsOff, WorkedConnection};
pub use copies::{copy_by, copy_row, row_said, Copied};
pub use find::{find_in_job, find_lines, find_names, nothing_said, NameFound};
pub use history::{History, UNDO_LIMIT};
pub use slide::{move_opening, wall_ends, FaceChange, OpeningMoved, OpeningTo};
pub use trying::{apply, try_change, ChangeDone, AtOpening, Change, HowTall, Tried, Trouble, WouldMake};
pub use shapes::{keep_shape_as, shape_block_add, shape_block_set, shape_block_take, BlockSet, ShapeChanged};
pub use stretch::{move_wall, read_move, way_said, Followed, Moved, Resized, RoofMoved};
pub use join::{connect, connect_at, connect_port, fit_all, fitting, split, Fitted, PassedOver, PortSaid};
pub(crate) use measure::takes_among;
pub use measure::{measure, measure_on_line, takes, Measured, Reading};

mod boxes;
mod changes;
mod floors;
mod items;
mod put;

pub use boxes::*;
pub use changes::*;
pub use floors::*;
pub use items::*;
pub use put::*;

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
/// What may not be drawn (`building::drawn_faults`), refused if the edit brought it in: a job
/// that already holds such a thing still opens, says it in `check`, and can still be edited
/// about anything else; what it cannot do is have one more drawn (call 179, B5 and B6).
fn new_faults(job: &mut Job, before: &str) -> Result<(), String> {
    let now = building::drawn_faults(job);
    if now.is_empty() {
        return Ok(());
    }
    let had: Vec<String> = match file::load(before) {
        Ok((mut was, _)) => building::drawn_faults(&mut was).into_iter().map(|f| f.0).collect(),
        Err(_) => Vec::new(),
    };
    match now.into_iter().find(|f| !had.contains(&f.0)) {
        Some((_, said)) => Err(said),
        None => Ok(()),
    }
}

pub fn all_or_nothing<T>(job: &mut Job, f: impl FnOnce(&mut Job) -> Result<T, String>) -> Result<T, String> {
    let before = file::save(job);
    match f(job).and_then(|v| building::check_whole(job).map(|()| v)).and_then(|v| new_faults(job, &before).map(|()| v)) {
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

/// Every name a thing under `parent` is called by on its level: the level's own things, so that two things on one level are
/// never called alike.
pub(crate) fn names_in_scope(job: &mut Job, parent: &str) -> std::collections::HashSet<String> {
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
    names.iter().filter(|(p, _)| up(p).contains(&scope)).map(|(_, n)| n.clone()).collect()
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

