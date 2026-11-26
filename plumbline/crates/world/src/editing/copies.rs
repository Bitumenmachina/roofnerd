//! A copy of a thing, a row of copies, and a wall copied with its doors and windows (call 219 point 6).
//!
//! BIMCraft's `copy` command "copies elements, with the doors and windows of copied walls" and returns the copies
//! (`crates/core/src/commands.rs:1219`, `duplicate`, `12198`): a copied wall brings its openings, a copied opening
//! alone is not made (it goes with its wall). That rule is taken. A row of copies is ours: BIMCraft has no array
//! (`grep -rn '"array"' crates/core/src/commands.rs`: nothing); it is the one copy made again each time a step farther,
//! all of it or none.
//!
//! A copy is Assumed, whatever the thing copied was: its place is new and nobody has measured it. Every value the
//! copy holds that its original held as Known is Assumed, and every value of its place says what it was copied from
//! and on whose word. It carries the same type, sizes and cost lines, and its item and assembly. It does not carry a
//! path (it is given its own), what its original was marked (a copy is new work, as a second thing built; a
//! temporary thing's copy is temporary), what a run or a model made of its connections (a copy is joined to nothing),
//! or what is known to be inside its wall (that is learnt by opening it). What is not carried is said.

use crate::building;
use crate::components::*;
use crate::job::Job;
use crate::units;
use crate::value::{Status, Val};
use crate::building::InSystem;
use crate::{Connectors, Opening, Placed, Segment, Wall, P2};

use super::slide::{by_name, node_of, said_len};
use super::stretch::names_of;
use super::{add_called, add_numbered, after, all_or_nothing, Entered};

/// One thing copied: what it was copied from, what was made, and what was not carried.
#[derive(Clone, Debug, PartialEq)]
pub struct Copied {
    pub from: String,
    pub from_name: String,
    /// The copy first, then each door or window made with it.
    pub made: Vec<String>,
    pub names: Vec<String>,
    /// What was not carried, in a builder's words.
    pub left_behind: Vec<String>,
}

impl Copied {
    /// "Lavatory 2 is a copy of Lavatory, Assumed."
    pub fn said(&self) -> String {
        let with = if self.made.len() > 1 { format!(", with its {}", so_many_openings(self.made.len() - 1)) } else { String::new() };
        format!("{} is a copy of {}{with}; its place is Assumed, and says it was copied", self.names.first().cloned().unwrap_or_default(), self.from_name)
    }
}

fn so_many_openings(n: usize) -> String {
    if n == 1 { "1 door or window".to_string() } else { format!("{n} doors and windows") }
}

/// A value of the copy: no surer than Assumed, and saying what it was copied from.
fn lowered<T>(v: &mut Val<T>, from: &str, who: &str) {
    v.status = v.status.max(Status::Assumed);
    v.source = after(&v.source, &format!("copied from {from}"), who);
}

fn lowered_opt<T>(v: &mut Option<Val<T>>, from: &str, who: &str) {
    if let Some(v) = v {
        lowered(v, from, who);
    }
}

fn shifted(v: &mut Val<P2>, east: f64, north: f64) -> Result<(), String> {
    let d = units::coord_divisor(&v.unit)?;
    let p = v.value.ok_or("where it is is not known, so it cannot be copied to a new place")?;
    v.value = Some([p[0] + east * d, p[1] + north * d]);
    Ok(())
}

fn shifted_line(v: &mut Val<[P2; 2]>, east: f64, north: f64) -> Result<(), String> {
    let d = units::coord_divisor(&v.unit)?;
    let l = v.value.ok_or("its line is not known, so it cannot be copied to a new place")?;
    v.value = Some([[l[0][0] + east * d, l[0][1] + north * d], [l[1][0] + east * d, l[1][1] + north * d]]);
    Ok(())
}

/// The components a copy carries besides its place: the assembly, the cost lines (each figure a person measured
/// is Assumed on the copy), the work breakdown code, the allowance, what it is made of, its system, its height.
/// Returns what was left behind.
fn carry(job: &mut Job, from: bevy_ecs::entity::Entity, to: &str, from_name: &str, who: &str) -> Result<Vec<String>, String> {
    let mut left = Vec::new();
    macro_rules! same {
        ($t:ty) => {
            if let Some(c) = job.world.get::<$t>(from).cloned() {
                job.put(to, c)?;
            }
        };
    }
    if let Some(mut c) = job.world.get::<Condition>(from).cloned() {
        lowered(&mut c.0, from_name, who);
        job.put(to, c)?;
    }
    if let Some(mut c) = job.world.get::<Cost>(from).cloned() {
        lowered(&mut c.code, from_name, who);
        for line in &mut c.lines {
            lowered_opt(&mut line.stretch_out, from_name, who);
            lowered_opt(&mut line.waste, from_name, who);
            for v in line.inputs.values_mut() {
                lowered(v, from_name, who);
            }
        }
        job.put(to, c)?;
    }
    if let Some(mut c) = job.world.get::<Height>(from).cloned() {
        lowered(&mut c.0, from_name, who);
        job.put(to, c)?;
    }
    if let Some(mut c) = job.world.get::<InSystem>(from).cloned() {
        lowered(&mut c.0, from_name, who);
        job.put(to, c)?;
    }
    same!(WorkCode);
    same!(Allowance);
    same!(MadeOf);
    // What the work does to it: a copy is new work; a temporary thing's copy is temporary too.
    if let Some(w) = job.world.get::<Work>(from).cloned() {
        match w.status.value {
            Some(WorkStatus::Temporary) => {
                let mut w = w;
                lowered(&mut w.status, from_name, who);
                job.put(to, w)?;
            }
            Some(other) if other != WorkStatus::New => left.push(format!("{from_name} is marked {}; the copy is new work", other.said())),
            _ => {}
        }
    }
    if job.world.get::<Hidden>(from).is_some() {
        left.push(format!("what is known to be inside {from_name} is not carried: the copy's inside is not known until it is opened"));
    }
    Ok(left)
}

/// A thing copied `east` and `north` feet. All of it or none.
pub fn copy_by(job: &mut Job, path: &str, east: f64, north: f64, how: Entered) -> Result<Copied, String> {
    let names = names_of(job);
    all_or_nothing(job, |job| copy_in(job, path, east, north, how)).map_err(|why| by_name(&names, &why))
}

/// A thing copied `times` times, each `east` and `north` feet farther than the last one: the first copy that far from the
/// thing, the second twice that far. All of it or none: where the model would refuse one it says which copy and makes none.
pub fn copy_row(job: &mut Job, path: &str, times: u32, east: f64, north: f64, how: Entered) -> Result<Vec<Copied>, String> {
    let names = names_of(job);
    if times == 0 {
        return Err("a row of no copies makes nothing: say how many".into());
    }
    if times > 500 {
        return Err(format!("{times} copies in a row is more than this makes at once (500)"));
    }
    all_or_nothing(job, |job| {
        let mut out = Vec::new();
        for k in 1..=times {
            let (e, n) = (east * f64::from(k), north * f64::from(k));
            let made = copy_in(job, path, e, n, how).map_err(|why| format!("copy {k} of {times}: {why}"))?;
            building::check_whole(job).map_err(|why| format!("copy {k} of {times}: {why}"))?;
            out.push(made);
        }
        Ok(out)
    })
    .map_err(|why| by_name(&names, &why))
}

fn copy_in(job: &mut Job, path: &str, east: f64, north: f64, how: Entered) -> Result<Copied, String> {
    if !(east.is_finite() && north.is_finite()) {
        return Err("how far to copy it is not a figure".into());
    }
    if east == 0.0 && north == 0.0 {
        return Err("a copy is set down where the thing is: say how far to copy it".into());
    }
    let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
    let kind = job.kind(path).ok_or_else(|| format!("{path} has no kind"))?;
    let node = job.world.get::<Node>(e).cloned().ok_or_else(|| format!("{path} has no place in the job"))?;
    let parent = node.parent.clone().ok_or("it has no level to copy into")?;
    let from_name = node.name.clone();
    let who = how.source;
    // A copy is Assumed whatever the word it was given with.
    let mut left = Vec::new();
    match kind {
        Kind::Fixture | Kind::Equipment | Kind::Furnishing => {
            let mut p = job.world.get::<Placed>(e).cloned().ok_or_else(|| format!("{from_name} has no place to copy"))?;
            shifted(&mut p.at, east, north)?;
            lowered(&mut p.what, &from_name, who);
            lowered(&mut p.at, &from_name, who);
            lowered(&mut p.offset, &from_name, who);
            lowered(&mut p.size, &from_name, who);
            lowered_opt(&mut p.angle, &from_name, who);
            let base = p.what.value.clone().unwrap_or_else(|| from_name.clone());
            let copy = add_called(job, &parent, kind, &base, 2, true)?;
            building::check_placed(&copy, kind, &p)?;
            job.put(&copy, p)?;
            left.extend(carry(job, e, &copy, &from_name, who)?);
            // Where the item says where it connects, the copy has the same connections; what a run or a model made on the
            // original is not the copy's.
            if let Some(mut c) = job.world.get::<Connectors>(e).cloned() {
                let all = c.0.len();
                c.0.retain(|k| k.of_item.is_some());
                let dropped = all - c.0.len();
                for k in &mut c.0 {
                    lowered(&mut k.at, &from_name, who);
                }
                if dropped > 0 {
                    left.push(format!("{from_name} has {dropped} {} made by a run or a model: the copy is joined to nothing", if dropped == 1 { "connection" } else { "connections" }));
                }
                if !c.0.is_empty() {
                    job.put(&copy, c)?;
                }
            }
            let name = node_of(job, &copy).map(|n| n.name).unwrap_or_default();
            Ok(Copied { from: path.to_string(), from_name, made: vec![copy], names: vec![name], left_behind: left })
        }
        k if k.is_segment() => {
            let mut s = job.world.get::<Segment>(e).cloned().ok_or_else(|| format!("{from_name} has no line to copy"))?;
            shifted_line(&mut s.line, east, north)?;
            lowered(&mut s.type_name, &from_name, who);
            lowered(&mut s.line, &from_name, who);
            lowered(&mut s.offset, &from_name, who);
            lowered_opt(&mut s.end_offset, &from_name, who);
            let copy = add_numbered(job, &parent, kind)?;
            building::check_segment(&copy, kind, &s)?;
            job.put(&copy, s)?;
            left.extend(carry(job, e, &copy, &from_name, who)?);
            left.push(format!("the copy of {from_name} is joined to nothing: fit it where it meets a run"));
            let name = node_of(job, &copy).map(|n| n.name).unwrap_or_default();
            Ok(Copied { from: path.to_string(), from_name, made: vec![copy], names: vec![name], left_behind: left })
        }
        Kind::Wall | Kind::Footing => {
            let mut w = job.world.get::<Wall>(e).cloned().ok_or_else(|| format!("{from_name} has no line to copy"))?;
            shifted_line(&mut w.line, east, north)?;
            lowered(&mut w.type_name, &from_name, who);
            lowered(&mut w.line, &from_name, who);
            lowered_opt(&mut w.top_level, &from_name, who);
            lowered_opt(&mut w.height, &from_name, who);
            lowered_opt(&mut w.base_offset, &from_name, who);
            lowered_opt(&mut w.top_offset, &from_name, who);
            let copy = add_numbered(job, &parent, kind)?;
            building::check_wall(&copy, kind, &w)?;
            job.put(&copy, w)?;
            left.extend(carry(job, e, &copy, &from_name, who)?);
            let mut made = vec![copy.clone()];
            // Its doors and windows come with it, each as far along its wall as before.
            let mut kids: Vec<String> = {
                let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
                q.iter(&job.world).filter(|(n, k, _)| k.is_opening() && n.parent.as_deref() == Some(path)).map(|(n, _, _)| n.path.clone()).collect()
            };
            kids.sort();
            for kid in kids {
                let ke = job.find(&kid).ok_or("a door or window of the wall is not there")?;
                let kkind = job.kind(&kid).ok_or("a door or window has no kind")?;
                let kname = job.world.get::<Node>(ke).map(|n| n.name.clone()).unwrap_or_default();
                let mut o = job.world.get::<Opening>(ke).cloned().ok_or("a door or window has no place in its wall")?;
                lowered(&mut o.type_name, &kname, who);
                lowered(&mut o.offset, &kname, who);
                lowered(&mut o.sill, &kname, who);
                let kc = add_numbered(job, &copy, kkind)?;
                building::check_opening(&kc, kkind, &o)?;
                job.put(&kc, o)?;
                left.extend(carry(job, ke, &kc, &kname, who)?);
                made.push(kc);
            }
            let names = made.iter().map(|p| job.find(p).and_then(|x| job.world.get::<Node>(x)).map(|n| n.name.clone()).unwrap_or_default()).collect();
            Ok(Copied { from: path.to_string(), from_name, made, names, left_behind: left })
        }
        k if k.is_opening() => Err(format!("{from_name} is a {} in a wall: it is copied with its wall, or moved to another wall; to put one more in a wall, make it there", k.said())),
        Kind::Fitting => Err(format!("{from_name} is a fitting, made where runs meet: copy the runs and fit them")),
        other => Err(format!("{other:#} is not copied; a wall, a footing, a run, a fixture, a piece of equipment or a furnishing is")),
    }
}

/// How far a row of copies reaches, said as the row is asked for: "4 copies, each 3 ft east of the last".
pub fn row_said(times: u32, east: f64, north: f64) -> String {
    let step = super::stretch::way_said(east, north);
    format!("{times} {}, each {} farther than the last ({} from the first to the last)", if times == 1 { "copy" } else { "copies" }, step, said_len((east * east + north * north).sqrt() * f64::from(times)))
}
