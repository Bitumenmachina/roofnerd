//! A door or a window moved: along its wall, or to another wall (call 219 point 5).
//!
//! BIMCraft's `move` says "doors and windows slide along their host wall" (`crates/core/src/commands.rs:1211`,
//! `relocate`): an opening is held by its host and its distance along it (`crates/core/src/walls/index.rs:107`,
//! `hosts`), and nothing there moves one to another wall. Here the opening keeps what it is (its type, its
//! cost lines, its work mark, its path) and changes where it is: its distance along its wall, or its wall.
//! What follows is worked from the model as it stands afterwards and not here: each wall's face less openings,
//! the casing and trim, the framing round it, the lines priced by any of them.
//!
//! It is refused where it would run past an end of its wall or lie over another opening in it, saying by how
//! much in feet and inches, and where the wall it goes to is coming down. A refusal names no coordinate.

use crate::building;
use crate::components::*;
use crate::geom;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::Val;
use crate::{Opening, OpeningType, Wall, P2};

use super::stretch::{feet_said, len_in, names_of, tidy};
use super::{after, all_or_nothing, names_in_scope, Entered};

/// Where an opening is to go.
#[derive(Clone, Debug, PartialEq)]
pub enum OpeningTo {
    /// Along its own wall by so many feet, toward the wall's second end when it is plus.
    Along(f64),
    /// Its middle so far along a wall (typed as lengths are: `6 ft`) from the wall's first end; no wall named is its own.
    Offset { wall: Option<String>, offset: String },
    /// Its middle at the point on a wall nearest a place on the plan; no wall named is its own.
    Point { wall: Option<String>, point: P2 },
}

/// A wall's face less its openings, before and after a change, in SF.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceChange {
    pub wall: String,
    pub name: String,
    pub was: Option<f64>,
    pub now: Option<f64>,
}

impl FaceChange {
    /// "wall 1: face less openings 220.00 SF, now 240.00 SF"
    pub fn said(&self) -> String {
        let sf = |x: Option<f64>| x.map(|x| format!("{} SF", crate::view::figure(x, 2))).unwrap_or_else(|| "no figure".to_string());
        match (self.was, self.now) {
            (Some(a), Some(b)) if (a - b).abs() < 0.005 => format!("{}: face less openings stays {}", self.name, sf(self.now)),
            _ => format!("{}: face less openings was {}, now {}", self.name, sf(self.was), sf(self.now)),
        }
    }
}

/// What moving an opening did.
#[derive(Clone, Debug, PartialEq)]
pub struct OpeningMoved {
    pub opening: String,
    pub name: String,
    pub from_wall: String,
    pub to_wall: String,
    pub to_wall_name: String,
    /// How far its middle is along the wall it is in now, from the wall's first end, in feet.
    pub offset_ft: f64,
    /// How far clear it is of each end of its wall, in feet: the first end's side, then the second's.
    pub clear_ft: (f64, f64),
    /// The ends of its wall, as a builder names them ("west end", "east end").
    pub ends: (String, String),
    /// Each wall whose face less openings changed.
    pub faces: Vec<FaceChange>,
    /// Said of what it was moved to, if it changed wall.
    pub notes: Vec<String>,
}

impl OpeningMoved {
    /// Where it stands now and how clear it is of each end of its wall.
    pub fn where_said(&self) -> String {
        format!("{} is now {} in {}: {} clear of the {} and {} clear of the {}", self.name, along_said(self.offset_ft, &self.ends.0), self.to_wall_name, feet_said(self.clear_ft.0), self.ends.0, feet_said(self.clear_ft.1), self.ends.1)
    }

    /// What a person is told: where it stands now, how clear it is of each end, and what each wall's face came to.
    pub fn said(&self) -> Vec<String> {
        let mut out = vec![self.where_said()];
        out.extend(self.faces.iter().map(FaceChange::said));
        out.extend(self.notes.iter().cloned());
        out
    }
}

fn along_said(offset: f64, end: &str) -> String {
    format!("{} from the {end}", feet_said(offset))
}

/// The two ends of a wall's line as a builder names them: a wall drawn west to east starts at its west end.
pub fn wall_ends(line: [P2; 2]) -> (String, String) {
    let (dx, dy) = (line[1][0] - line[0][0], line[1][1] - line[0][1]);
    let (first, second) = if dx.abs() >= dy.abs() { if dx >= 0.0 { ("west end", "east end") } else { ("east end", "west end") } } else if dy >= 0.0 { ("south end", "north end") } else { ("north end", "south end") };
    (first.to_string(), second.to_string())
}

/// A length as a builder says it aloud, never with a minus sign ("6 in", "2 ft 3 in").
pub(crate) fn said_len(feet: f64) -> String {
    feet_said(feet)
}

/// A thing's place in the job, by path.
pub(crate) fn node_of(job: &mut Job, path: &str) -> Option<Node> {
    let e = job.find(path)?;
    job.world.get::<Node>(e).cloned()
}

fn net_area(job: &mut Job, wall: &str) -> Option<f64> {
    let e = job.find(wall)?;
    job.world.get::<Measure>(e)?.0.get("net_area")?.worked()
}

/// Move a door or a window. All of it or none: the job is as it was where this refuses.
pub fn move_opening(job: &mut Job, path: &str, to: &OpeningTo, how: Entered) -> Result<OpeningMoved, String> {
    let names = names_of(job);
    let said_by_name = |why: String| by_name(&names, &why);
    all_or_nothing(job, |job| move_in(job, path, to, how, &names)).map_err(said_by_name)
}

/// A refusal with each path in it said as the thing is called ("wall 1", "door 2").
pub(crate) fn by_name(names: &std::collections::HashMap<String, String>, why: &str) -> String {
    let mut out = why.to_string();
    let mut paths: Vec<&String> = names.keys().filter(|p| out.contains(p.as_str())).collect();
    paths.sort_by_key(|p| std::cmp::Reverse(p.len()));
    for p in paths {
        out = out.replace(p.as_str(), &names[p]);
    }
    out
}

fn move_in(job: &mut Job, path: &str, to: &OpeningTo, how: Entered, names: &std::collections::HashMap<String, String>) -> Result<OpeningMoved, String> {
    let called = |p: &str| names.get(p).cloned().unwrap_or_else(|| p.to_string());
    let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
    let kind = job.kind(path).ok_or_else(|| format!("{path} has no kind"))?;
    if !kind.is_opening() {
        return Err(format!("{} is not a door or a window", called(path)));
    }
    let node = job.world.get::<Node>(e).cloned().ok_or_else(|| format!("{path} has no place in the job"))?;
    let mut op = job.world.get::<Opening>(e).cloned().ok_or_else(|| format!("{} has no place in a wall", called(path)))?;
    let from_wall = node.parent.clone().ok_or_else(|| format!("{} is in no wall", called(path)))?;
    let name = called(path);
    let which_wall = |wall: &Option<String>| wall.clone().unwrap_or_else(|| from_wall.clone());
    let target = match to {
        OpeningTo::Along(_) => from_wall.clone(),
        OpeningTo::Offset { wall, .. } | OpeningTo::Point { wall, .. } => which_wall(wall),
    };
    let tw = job.find(&target).and_then(|x| job.world.get::<Wall>(x).cloned()).filter(|_| job.kind(&target) == Some(Kind::Wall)).ok_or_else(|| format!("{} is not a wall: a door or a window goes in a wall", called(&target)))?;
    let line = tw.line_ft().ok_or_else(|| format!("{}'s line is not known, so nothing can be placed along it", called(&target)))?;
    let length = geom::dist(line[0], line[1]);
    if length <= 1.0e-9 {
        return Err(format!("{} has no length to place anything along", called(&target)));
    }
    if job.find(&target).and_then(|x| job.world.get::<Work>(x)).is_some_and(|w| w.status.value == Some(WorkStatus::Demolish)) && target != from_wall {
        return Err(format!("{} is to demolish: {} is not moved into a wall that is coming down. Mark the wall existing or new first", called(&target), kind.said()));
    }
    // Where its middle is to be, as a value that says how sure that is.
    let offset: Val<Len> = match to {
        OpeningTo::Along(d) => {
            if !d.is_finite() {
                return Err("the distance to move it by is not a figure".into());
            }
            if d.abs() < 1.0e-9 {
                return Err(format!("{name} is moved by nothing along its wall"));
            }
            let was = op.offset.feet().ok_or_else(|| format!("where {name} is along its wall is not known, so it cannot be moved from there"))?;
            let len = len_in(tidy(was + d), &op.offset.unit).ok_or_else(|| format!("{name}'s distance along its wall is held in a unit this cannot move it in"))?;
            Val { value: Some(len), unit: op.offset.unit.clone(), status: op.offset.status.max(how.status), source: after(&op.offset.source, "moved", how.source) }
        }
        OpeningTo::Offset { offset, .. } => {
            let (len, unit) = units::parse_length(offset)?;
            Val { value: Some(len), unit: unit.to_string(), status: how.status, source: how.source.to_string() }
        }
        OpeningTo::Point { point, .. } => {
            let (dx, dy) = (line[1][0] - line[0][0], line[1][1] - line[0][1]);
            let along = ((point[0] - line[0][0]) * dx + (point[1] - line[0][1]) * dy) / length;
            Val { value: Some(Len::One((along * 10_000.0).round() / 10_000.0)), unit: "ft".to_string(), status: how.status, source: how.source.to_string() }
        }
    };
    let middle = offset.feet().ok_or("where it is to go is not a length")?;
    // Its width is its type's: where that is not known nothing can be said of how it fits.
    let width = building::opening_widths(job).get(path).copied();
    let ends = wall_ends(line);
    if let Some(w) = width {
        let (from, to_end) = (middle - w / 2.0, middle + w / 2.0);
        let tol = crate::check::FIT_TOLERANCE_FT;
        let sizes = format!("the wall is {} long, {name} is {} wide and its middle would be {} from the {}", said_len(length), said_len(w), said_len(middle), ends.0);
        if from < -tol {
            return Err(format!("{name} would run {} past the {} of {}: {sizes}", said_len(-from), ends.0, called(&target)));
        }
        if to_end > length + tol {
            return Err(format!("{name} would run {} past the {} of {}: {sizes}", said_len(to_end - length), ends.1, called(&target)));
        }
        // Over another opening in that wall: where both their widths and their heights overlap.
        let mut mine = None;
        let others: Vec<(String, Opening)> = {
            let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
            let gone = crate::components::demolished(&mut job.world);
            let mut q2 = q.iter(&job.world).filter(|(n, k, _)| k.is_opening() && n.parent.as_deref() == Some(target.as_str()) && n.path != path && !gone.contains(&n.path)).map(|(n, _, o)| (n.path.clone(), o.clone())).collect::<Vec<_>>();
            q2.sort_by(|a, b| a.0.cmp(&b.0));
            q2
        };
        let heights = |job: &mut Job, p: &str, o: &Opening| -> Option<(f64, f64, f64)> {
            let w = building::opening_widths(job).get(p).copied()?;
            let tall = job.find(&o.type_name.value.clone()?).and_then(|x| job.world.get::<OpeningType>(x)).and_then(|t| t.height.feet())?;
            Some((w, o.sill.feet()?, o.sill.feet()? + tall))
        };
        if let Some(own_tall) = job.find(&op.type_name.value.clone().unwrap_or_default()).and_then(|x| job.world.get::<OpeningType>(x)).and_then(|t| t.height.feet()) {
            mine = op.sill.feet().map(|s| (s, s + own_tall));
        }
        for (other, o) in &others {
            let (Some((ow, s0, s1)), Some((m0, m1)), Some(centre)) = (heights(job, other, o), mine, o.offset.feet()) else { continue };
            let (a, b) = (centre - ow / 2.0, centre + ow / 2.0);
            let shared = to_end.min(b) - from.max(a);
            if shared > tol && m0 < s1 && s0 < m1 {
                return Err(format!("{name} would lie over {} in {} by {}: an opening is clear of every other opening in its wall", called(other), called(&target), said_len(shared)));
            }
        }
    }
    // Before: what the faces are.
    job.derive();
    let before: Vec<(String, Option<f64>)> = if target == from_wall { vec![(from_wall.clone(), net_area(job, &from_wall))] } else { vec![(from_wall.clone(), net_area(job, &from_wall)), (target.clone(), net_area(job, &target))] };
    // Do it: the opening is under its new wall, called what no other thing on that level is called.
    let mut notes = Vec::new();
    if target != from_wall {
        let from_level = node_of(job, &from_wall).and_then(|n| n.parent);
        let to_level = node_of(job, &target).and_then(|n| n.parent);
        let mut new_name = node.name.clone();
        if from_level != to_level {
            let taken = names_in_scope(job, &target);
            if taken.contains(&new_name) {
                let base = new_name.trim_end_matches(|c: char| c.is_ascii_digit()).trim_end().to_string();
                new_name = (1..).map(|n| format!("{base} {n}")).find(|n| !taken.contains(n)).unwrap_or(new_name);
                notes.push(format!("it is called {new_name} on its new level: {} is called {} there", node.name, node.name));
            }
        }
        job.name_free(&target, &new_name, Some(path))?;
        let mut n = job.world.get_mut::<Node>(e).ok_or("its place in the job is gone")?;
        n.parent = Some(target.clone());
        n.name = new_name;
        notes.insert(0, format!("it is in {} now, and was in {}", called(&target), called(&from_wall)));
    }
    op.offset = offset;
    building::check_opening(path, kind, &op)?;
    job.put(path, op)?;
    job.derive();
    let mut faces = Vec::new();
    for (wall, was) in before {
        faces.push(FaceChange { name: called(&wall), now: net_area(job, &wall), was, wall });
    }
    let shown = middle;
    let w = width.unwrap_or(0.0);
    Ok(OpeningMoved { opening: path.to_string(), name: called(path), from_wall, to_wall_name: called(&target), to_wall: target, offset_ft: shown, clear_ft: (tidy(shown - w / 2.0), tidy(length - shown - w / 2.0)), ends, faces, notes })
}
