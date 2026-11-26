//! Trying a change and taking it back (call 219 point 2).
//!
//! BIMCraft tries a script and takes it back (`Session::script_dry_run`, `crates/core/src/engine.rs:602`, with
//! `dry_run_tests.rs`): it keeps what the project was (`Kept::of`), runs the script all of it or none, puts back what
//! was kept whatever the script did, and answers with what was made, changed and removed, and its warnings. That
//! shape is taken: keep the job's text, make the change by the same function a real change goes by, read what it
//! would make and what it would be in the same space as, and put the job back as it was kept. What is ours: one
//! change and not a script (`Change`), the reply in a builder's words (`Tried::said`), and the clashes and holes the
//! change would make, found with the clash test as it stands, so that what is said before the click is what
//! `clashes` says after it.
//!
//! The job is as it was after any try: the saved text byte for byte, the path the next thing made would be given,
//! and the undo history (it is not held here and is not touched).

use std::collections::HashSet;

use crate::building::clash;
use crate::components::*;
use crate::file;
use crate::job::Job;
use crate::units;
use crate::value::Status;
use crate::{Kind, P2, P3};

use super::copies::{copy_by, copy_row};
use super::slide::{by_name, move_opening, said_len, FaceChange, OpeningTo};
use super::stretch::{move_wall, names_of, way_said};
use super::{opening, place, place_item_said, runs, turn, walls, Entered, OnWall, Top};

/// How tall a wall is to be: so tall, or held to another level.
#[derive(Clone, Debug, PartialEq)]
pub enum HowTall {
    Height(String),
    Level(String),
}

/// Where along its wall a new door or window goes.
#[derive(Clone, Debug, PartialEq)]
pub enum AtOpening {
    /// Its middle so far from the wall's first end, typed as lengths are.
    Offset(String),
    /// Its middle at the place on the wall nearest a point on the plan.
    Point(P2),
}

/// One change to the job that makes or moves a thing: the one thing the window tries when the pointer rests, and does when it is clicked.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    /// A wall for each pair of points that follow one another.
    Walls { storey: String, type_path: String, points: Vec<P2>, closed: bool, tall: HowTall },
    /// A door or a window in a wall.
    Opening { wall: String, kind: Kind, type_path: String, at: AtOpening, sill: String },
    /// Straight pieces of duct, pipe, conduit or cable.
    Runs { storey: String, kind: Kind, type_path: String, points: Vec<P2>, height: String, system: Option<String> },
    /// One of an item.
    Item { storey: String, item: String, at: P2, offset: String },
    /// A fixture, a piece of equipment or a furnishing stood at a point.
    Place { storey: String, kind: Kind, what: String, at: P2, offset: String, size: Option<P3>, system: Option<String> },
    /// A wall, a run, a thing or a room moved; a wall takes the ends that meet it unless it is moved alone.
    Move { path: String, east: f64, north: f64, alone: bool },
    /// A thing turned that many degrees counter-clockwise.
    Turn { path: String, degrees: f64 },
    /// A door or a window moved along its wall or to another.
    MoveOpening { path: String, to: OpeningTo },
    /// A copy that far from the thing.
    Copy { path: String, east: f64, north: f64 },
    /// So many copies, each that far farther than the last.
    Row { path: String, times: u32, east: f64, north: f64 },
}

/// What a change did, or would do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangeDone {
    /// What was made, by path.
    pub made: Vec<String>,
    /// Everything that was made or moved, by path: what the clash test is asked about.
    pub touched: Vec<String>,
    /// What else it did, in words.
    pub notes: Vec<String>,
    /// Each wall whose face less openings changed.
    pub faces: Vec<FaceChange>,
}

/// Make the change, all of it or none, by the same functions the verbs go by.
pub fn apply(job: &mut Job, change: &Change, how: Entered) -> Result<ChangeDone, String> {
    let mut out = ChangeDone::default();
    match change {
        Change::Walls { storey, type_path, points, closed, tall } => {
            let top = match tall {
                HowTall::Height(h) => Top::Height(h),
                HowTall::Level(l) => Top::Level(l),
            };
            out.made = walls(job, storey, type_path, points, *closed, top, how)?;
        }
        Change::Opening { wall, kind, type_path, at, sill } => {
            let on = match at {
                AtOpening::Offset(o) => OnWall::Offset(o),
                AtOpening::Point(p) => OnWall::Point(*p),
            };
            out.made = vec![opening(job, wall, *kind, type_path, on, sill, how)?];
        }
        Change::Runs { storey, kind, type_path, points, height, system } => {
            out.made = runs(job, storey, *kind, type_path, points, height, system.as_deref(), how)?;
        }
        Change::Item { storey, item, at, offset } => {
            let (path, notes) = place_item_said(job, storey, item, *at, offset, how)?;
            out.made = vec![path];
            out.notes = notes;
        }
        Change::Place { storey, kind, what, at, offset, size, system } => {
            out.made = vec![place(job, storey, *kind, what, *at, offset, *size, system.as_deref(), how)?];
        }
        Change::Move { path, east, north, alone } => {
            let moved = move_wall(job, path, *east, *north, how, *alone)?;
            out.touched.push(path.clone());
            out.touched.extend(moved.followed.iter().map(|f| f.path.clone()));
            out.touched.extend(moved.went_with.iter().cloned());
            out.notes.push(format!("{} would be moved {}", moved.names.get(path).cloned().unwrap_or_else(|| path.clone()), way_said(*east, *north)));
            out.notes.extend(moved.said());
        }
        Change::Turn { path, degrees } => {
            let now = turn(job, path, *degrees, how)?;
            out.touched.push(path.clone());
            out.notes.push(format!("turned {} degrees from east", crate::view::num(super::turned_said(now))));
        }
        Change::MoveOpening { path, to } => {
            let moved = move_opening(job, path, to, how)?;
            out.touched.push(path.clone());
            out.touched.push(moved.from_wall.clone());
            out.touched.push(moved.to_wall.clone());
            out.notes = std::iter::once(moved.where_said()).chain(moved.notes.iter().cloned()).collect();
            out.faces = moved.faces;
        }
        Change::Copy { path, east, north } => {
            let c = copy_by(job, path, *east, *north, how)?;
            out.made = c.made.clone();
            out.notes.push(c.said());
            out.notes.extend(c.left_behind);
        }
        Change::Row { path, times, east, north } => {
            let all = copy_row(job, path, *times, *east, *north, how)?;
            for c in all {
                out.made.extend(c.made.clone());
                out.notes.push(c.said());
                out.notes.extend(c.left_behind);
            }
        }
    }
    let mut touched = out.made.clone();
    touched.extend(out.touched.drain(..));
    touched.sort();
    touched.dedup();
    out.touched = touched;
    Ok(out)
}

/// A thing a change would make, as the job would call it.
#[derive(Clone, Debug, PartialEq)]
pub struct WouldMake {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    /// The figures the model works out for it, as the estimate has them: (what it is, how much, its unit).
    pub figures: Vec<(String, f64, String)>,
    /// What it is, said ("wall 5, 12 ft long").
    pub said: String,
}

/// Something the change would be in the same space as, set against, or cut a hole in.
#[derive(Clone, Debug, PartialEq)]
pub struct Trouble {
    pub said: String,
    /// How sure it is: Assumed is a question for the field, Known a finding.
    pub status: Status,
    /// What the change would make or move that is in it, by path, and what it is in.
    pub paths: Vec<String>,
}

/// What a try found: what the change would make, what it would be in the same space as, what hole it would cut, whether the
/// model would refuse it and why. The job is as it was.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tried {
    /// The model's refusal, word for word as the real change gives it, said by the names of things.
    pub refused: Option<String>,
    pub made: Vec<WouldMake>,
    /// What else it would do, in words.
    pub notes: Vec<String>,
    /// In each other: two things in the same space.
    pub same_space: Vec<Trouble>,
    /// Standing against a wall, or a run at its joint, by no more than two inches: not a clash.
    pub set_against: Vec<Trouble>,
    /// A hole to cut, each once.
    pub holes: Vec<Trouble>,
    /// Lying inside a floor or a wall along its length: no hole.
    pub laid_in: Vec<Trouble>,
    /// Each wall whose face less openings would change.
    pub faces: Vec<FaceChange>,
}

impl Tried {
    /// Whether the change would be made and would be in the same space as nothing.
    pub fn fits(&self) -> bool {
        self.refused.is_none() && self.same_space.is_empty()
    }

    /// What a person is told, a line each, in a builder's words.
    pub fn said(&self) -> Vec<String> {
        if let Some(why) = &self.refused {
            return vec![format!("it would be refused: {why}")];
        }
        let mut out = Vec::new();
        if self.made.is_empty() {
            // A thing moved, turned or slid makes nothing: what it does is in the notes.
            if self.notes.is_empty() && self.faces.is_empty() {
                out.push("it would make nothing new".to_string());
            }
        } else {
            out.push(format!("it would make {}", self.made.iter().map(|m| m.said.clone()).collect::<Vec<_>>().join("; ")));
        }
        out.extend(self.notes.iter().cloned());
        out.extend(self.faces.iter().map(FaceChange::said));
        let flag = |t: &Trouble| match t.status {
            Status::Known => " (Known: a finding)",
            _ => " (Assumed: a question for the field)",
        };
        if !self.holes.is_empty() {
            out.push(format!("{} to cut", crate::view::so_many(self.holes.len(), "hole")));
            out.extend(self.holes.iter().map(|t| format!("  {}{}", t.said, flag(t))));
        }
        if !self.same_space.is_empty() {
            out.push(format!("in the same space as {}", crate::view::so_many(self.same_space.len(), "thing")));
            out.extend(self.same_space.iter().map(|t| format!("  {}{}", t.said, flag(t))));
        }
        if !self.set_against.is_empty() {
            out.push(format!("set against: {}", self.set_against.iter().map(|t| t.said.clone()).collect::<Vec<_>>().join("; ")));
        }
        if !self.laid_in.is_empty() {
            out.push(format!("lies inside: {}", self.laid_in.iter().map(|t| t.said.clone()).collect::<Vec<_>>().join("; ")));
        }
        if self.holes.is_empty() && self.same_space.is_empty() {
            out.push("it fits: in the same space as nothing, and no hole to cut".to_string());
        }
        out
    }
}

/// How a made thing is said: its name and what the model works out for it.
fn made_of(job: &mut Job, path: &str) -> Option<WouldMake> {
    let e = job.find(path)?;
    let node = job.world.get::<Node>(e)?.clone();
    let kind = *job.world.get::<Kind>(e)?;
    let m = job.world.get::<Measure>(e).cloned();
    let fig = |name: &str| m.as_ref().and_then(|m| m.0.get(name)).and_then(|d| d.worked().map(|v| (v, d.unit.clone())));
    let mut figures = Vec::new();
    let mut said = node.name.clone();
    if kind == Kind::Wall || kind == Kind::Footing {
        if let Some((v, unit)) = fig("plan_length") {
            said = format!("{said}, {} long", said_len(v));
            figures.push(("plan_length".to_string(), v, unit));
        }
        if let Some((v, unit)) = fig("net_area") {
            figures.push(("net_area".to_string(), v, unit));
        }
    } else if kind.is_segment() {
        if let Some((v, unit)) = fig("true_length").or_else(|| fig("length")) {
            said = format!("{said}, {} long", said_len(v));
            figures.push(("length".to_string(), v, unit));
        }
    } else if kind.is_opening() {
        if let Some(parent) = node.parent.as_deref().and_then(|p| job.find(p)).and_then(|p| job.world.get::<Node>(p)) {
            said = format!("{said} in {}", parent.name);
        }
    } else if let Some(p) = job.world.get::<crate::Placed>(e) {
        if let Some(size) = p.size_ft() {
            said = format!("{said}, {}", units::size_said(size));
        }
    }
    Some(WouldMake { path: path.to_string(), name: node.name, kind, figures, said })
}

/// Try a change: what it would make, what it would be in the same space as and what hole it would cut, and whether the
/// model would refuse it; and leave the job exactly as it was (the saved text byte for byte, the next path the job would give
/// out, and so everything else). It is as slow as the clash test is, which is what the window has to bear when the pointer rests.
pub fn try_change(job: &mut Job, change: &Change, how: Entered) -> Tried {
    let kept = file::save(job);
    let before_names = names_of(job);
    let before: HashSet<String> = {
        let mut q = job.world.query::<&Node>();
        q.iter(&job.world).map(|n| n.path.clone()).collect()
    };
    let done = apply(job, change, how);
    let tried = match done {
        Err(why) => Tried { refused: Some(by_name(&before_names, &why)), ..Tried::default() },
        Ok(applied) => {
            // The job as the change would leave it: what it made, and the figures of each.
            let mut tried = Tried { notes: applied.notes.clone(), faces: applied.faces.clone(), ..Tried::default() };
            let new: Vec<String> = {
                let mut q = job.world.query::<&Node>();
                let mut p: Vec<String> = q.iter(&job.world).filter(|n| !before.contains(&n.path)).map(|n| n.path.clone()).collect();
                p.sort();
                p
            };
            // What it made, in the order it made it (a wall, then its doors and windows): not what the job only made again to follow it
            // (a roof's planes, when a wall they stand on is moved).
            for p in &applied.made {
                if let Some(m) = made_of(job, p) {
                    tried.made.push(m);
                }
            }
            let mut touched: Vec<String> = applied.touched.clone();
            touched.extend(new.iter().cloned());
            let touched: HashSet<String> = touched.into_iter().collect();
            match clash::clashes(job, None) {
                Err(why) => tried.notes.push(format!("whether it clashes could not be worked out: {why}")),
                Ok(found) => {
                    let names = names_of(job);
                    let nm = |p: &str, own: &str| names.get(p).cloned().unwrap_or_else(|| own.to_string());
                    let inches = |x: f64| units::inches_said(x);
                    let mine = |c: &clash::Clash| touched.contains(&c.a) || touched.contains(&c.b);
                    for c in found.found.iter().filter(|c| mine(c)) {
                        // Said of the thing the change made or moved: it would be in the same space as what was there.
                        let (a, b) = if touched.contains(&c.a) { (nm(&c.a, &c.a_name), nm(&c.b, &c.b_name)) } else { (nm(&c.b, &c.b_name), nm(&c.a, &c.a_name)) };
                        let paths: Vec<String> = [&c.a, &c.b].into_iter().filter(|p| touched.contains(*p)).cloned().collect();
                        if c.how != clash::ClashKind::Hard {
                            continue;
                        }
                        if c.through {
                            continue;
                        }
                        if c.set_against {
                            tried.set_against.push(Trouble { said: format!("{a} stands against {b}, {} into it", inches(c.by.unwrap_or(0.0))), status: c.status, paths });
                        } else {
                            let by = c.by.map(|x| format!(", in each other by {}", inches(x))).unwrap_or_default();
                            tried.same_space.push(Trouble { said: format!("{a} would be in the same space as {b}{by}"), status: c.status, paths });
                        }
                    }
                    for c in found.laid_in().into_iter().filter(|c| mine(c)) {
                        tried.laid_in.push(Trouble { said: format!("{} is laid in {}", nm(&c.a, &c.a_name), nm(&c.b, &c.b_name)), status: c.status, paths: vec![c.a.clone(), c.b.clone()] });
                    }
                    let mut seen: Vec<usize> = Vec::new();
                    for c in found.found.iter().filter(|c| mine(c) && c.how == clash::ClashKind::Hard && c.through && !c.laid_in) {
                        let Some(h) = c.hole else { continue };
                        if seen.contains(&h) {
                            continue;
                        }
                        seen.push(h);
                        let Some(hole) = found.holes.get(h) else { continue };
                        let pieces: Vec<String> = hole.pieces.iter().map(|(p, n)| nm(p, n)).collect();
                        let size = if hole.size_said.is_empty() { String::new() } else { format!(", {}", hole.size_said) };
                        let paths: Vec<String> = hole.pieces.iter().map(|(p, _)| p.clone()).filter(|p| touched.contains(p)).collect();
                        tried.holes.push(Trouble { said: format!("{} passes through {}{size}", pieces.join(", "), nm(&hole.through, &hole.through_name)), status: hole.status, paths });
                    }
                }
            }
            tried
        }
    };
    // The job as it was kept. Where the change was refused it has put itself back already, from the same text.
    if tried.refused.is_none() {
        match file::load(&kept) {
            Ok((back, _)) => *job = back,
            Err(why) => return Tried { refused: Some(format!("the job could not be put back as it was ({why})")), ..Tried::default() },
        }
    }
    tried
}
