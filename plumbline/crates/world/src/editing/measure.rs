//! A measurement taken on site, put on the job: one entered value replaced by what was
//! measured, or marked as looked at and found as drawn. Either way the value is Known and
//! says who measured. This is the last step of the loop: what was traced starts Assumed,
//! and a tape or a laser makes it Known.
//!
//! Ours. BIMCraft keeps no certainty on a value, so it has nothing that does this. What a
//! run's heights are held to (its top, its centre line, its bottom) is BIMCraft's
//! `justify`, which the model here already carries (`building/model.rs`).
//!
//! The value is named as the screen names it ("storey height", "sill", "height above its
//! storey"), because that is what the person measuring is looking at.

use bevy_ecs::entity::Entity;

use super::{after, all_or_nothing};
use crate::building;
use crate::components::*;
use crate::job::Job;
use crate::units::{self, Len};
use crate::value::{Status, Val};
use crate::{check, Connectors, Floor, FloorType, InSystem, Justify, Level, Opening, OpeningType, Placed, Room, Segment, SegmentType, Wall, WallType};

/// What the person on site has to say about a value.
#[derive(Clone, Copy)]
pub enum Reading<'a> {
    /// What the tape or the laser read, as typed: `8' 10 1/2"`, `8'10"1/2`, `106.5 in`.
    Length(&'a str),
    /// It was looked at and is as drawn.
    Same,
}

/// What `put` is asked to do with a value: one of the two readings, or only to say
/// whether the element has a value of that name and whether it is one length.
#[derive(Clone, Copy)]
enum Do<'a> {
    Length(&'a str),
    Same,
    Probe,
}

/// What a measurement did.
#[derive(Clone, Debug, PartialEq)]
pub struct Measured {
    /// The value measured: what it is now and what it was.
    pub said: String,
    /// What else moved because of it, and what was left alone and why.
    pub also: Vec<String>,
}

/// A value that can be marked as found as drawn, whatever it holds.
trait Looked {
    fn settled(&self) -> bool;
    fn mark(&mut self, by: &str);
}

impl<T> Looked for Val<T> {
    fn settled(&self) -> bool {
        self.value.is_some() && self.status != Status::Unknown
    }
    fn mark(&mut self, by: &str) {
        self.status = Status::Known;
        self.source = after(&self.source, "found as drawn", by);
    }
}

/// One entered value of an element: a single length, or anything else.
enum Slot<'a> {
    Length(&'a mut Val<Len>),
    Other(&'a mut dyn Looked),
}

fn said(v: &Val<Len>) -> String {
    match &v.value {
        Some(len) => units::show_length(len, &v.unit),
        None => "unknown".into(),
    }
}

fn feet_said(feet: f64) -> String {
    units::feet_as_ft_in(feet, 64)
}

fn status_said(s: Status) -> &'static str {
    match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    }
}

/// Put the reading on the value. Returns what to say of it.
fn apply(slot: Slot, name: &str, what: &str, reading: Do, source: &str) -> Result<String, String> {
    match (slot, reading) {
        (Slot::Length(_), Do::Probe) => Ok("length".into()),
        (Slot::Other(_), Do::Probe) => Ok("other".into()),
        (Slot::Length(v), Do::Length(typed)) => {
            let (len, unit) = units::parse_length(typed)?;
            let was = if v.value.is_some() { format!("it was {}, {}: {}", said(v), status_said(v.status), v.source) } else { format!("it was not known: {}", v.source) };
            *v = Val { value: Some(len), unit: unit.into(), status: Status::Known, source: source.into() };
            Ok(format!("{name}: {what} is {}, measured ({was})", said(v)))
        }
        (Slot::Other(_), Do::Length(_)) => Err(format!("{name}'s {what} is not one length, so one measurement does not give it; move it to where it is, then say \"same\"")),
        (Slot::Length(v), Do::Same) => {
            if !v.settled() {
                return Err(format!("{name}'s {what} is not known, so there is nothing for it to be the same as; measure it"));
            }
            v.mark(source);
            Ok(format!("{name}: {what} is {}, found as drawn", said(v)))
        }
        (Slot::Other(v), Do::Same) => {
            if !v.settled() {
                return Err(format!("{name}'s {what} is not known, so there is nothing for it to be the same as"));
            }
            v.mark(source);
            Ok(format!("{name}: {what} found as drawn"))
        }
    }
}

/// The number after `prefix` in a label, and what follows it: "layer 2 thickness" gives
/// (2, " thickness").
fn numbered<'a>(label: &'a str, prefix: &str) -> Option<(usize, &'a str)> {
    let rest = label.strip_prefix(prefix)?;
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let n: usize = rest[..digits].parse().ok()?;
    (n >= 1).then_some((n, &rest[digits..]))
}

fn layer_slot<'a>(layers: &'a mut [crate::Layer], what: &str) -> Option<Slot<'a>> {
    let (n, rest) = numbered(what, "layer ")?;
    let layer = layers.get_mut(n - 1)?;
    if rest == " thickness" {
        Some(Slot::Length(&mut layer.thickness))
    } else if rest.starts_with(", ") {
        Some(Slot::Other(&mut layer.material))
    } else {
        None
    }
}

/// Put a measurement taken on site on one entered value of an element. `what` is the
/// value's name as the screen shows it. `to` is for a run's height only: which line of
/// its section the measurement was taken to. The value becomes Known, on `source`'s word.
pub fn measure(job: &mut Job, path: &str, what: &str, reading: Reading, to: Option<Justify>, source: &str) -> Result<Measured, String> {
    if source.trim().is_empty() {
        return Err("a measurement says who measured it, or with what; give its source".into());
    }
    let what = what.trim();
    let a_floor = job.kind(path) == Some(Kind::Storey);
    all_or_nothing(job, |job| {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = job.kind(path).ok_or_else(|| format!("{path} has no kind"))?;
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
        if to.is_some() && !kind.is_segment() {
            return Err(format!("{name} is not a run: only a run's height is measured to its bottom, its centre or its top"));
        }
        let reading = match reading {
            Reading::Length(typed) => Do::Length(typed),
            Reading::Same => Do::Same,
        };
        // What a run or a placed thing is joined to before it is measured: a height
        // that changes can pull it off its joints, and that has to be said.
        let joined_before = if kind.is_segment() || kind.is_placed() { joined_to(job, path) } else { Vec::new() };
        if let Some(mut done) = put(job, e, path, kind, &name, what, reading, to, source)? {
            if !joined_before.is_empty() {
                let still = joined_to(job, path);
                let parted: Vec<String> = joined_before.iter().filter(|(p, _)| !still.iter().any(|(q, _)| q == p)).map(|(_, n)| n.clone()).collect();
                if !parted.is_empty() {
                    done.also.push(format!(
                        "{name} no longer meets {}: {} where {} and did not move with it. Measure or move {} too, or the takeoff will count open ends there and the clash list will show them in each other",
                        parted.join(", "),
                        if parted.len() == 1 { "it is" } else { "they are" },
                        if parted.len() == 1 { "it was" } else { "they were" },
                        if parted.len() == 1 { "it" } else { "them" }
                    ));
                }
            }
            return Ok(done);
        }
        // Nothing of that name. Say what it does have.
        let has: Vec<String> = takes(job, path).into_iter().map(|t| t.0).collect();
        Err(if has.is_empty() { format!("{name} has no entered value to measure") } else { format!("{name} has no entered value called {what:?}; it has: {}", has.join(", ")) })
    })
    // A floor brought down onto walls that stand to their own height runs them into the
    // walls above, and the job's own check refuses that as one wall over another. What is
    // wrong is the walls' height, and that is what is said.
    .map_err(|why| {
        if a_floor && why.contains("one wall drawn over another") {
            format!("this measurement puts the floor above lower than walls on this storey that stand to their own height, so they would run up into the walls above ({why}). Measure those walls' heights first, or hold their tops to the storey above, and then measure this")
        } else {
            why
        }
    })
}

/// What a run or a placed thing meets at its ends or its connectors, by path and name.
fn joined_to(job: &mut Job, path: &str) -> Vec<(String, String)> {
    let Ok(net) = building::network::network(job, None, None, building::network::TOLERANCE_FT) else { return Vec::new() };
    let mut met: Vec<String> = net.nodes.iter().filter(|n| n.joints.iter().any(|j| j.element() == path)).flat_map(|n| n.joints.iter().map(|j| j.element().to_string())).filter(|p| p != path).collect();
    met.sort();
    met.dedup();
    met.into_iter().map(|p| {
        let name = job.find(&p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap_or_else(|| p.clone());
        (p, name)
    }).collect()
}

/// How nearly two Known values that say the same thing have to agree: an eighth of an
/// inch. A tape and a laser on the same floor differ by that.
pub const AGREE_FT: f64 = 1.0 / 96.0;

/// The source of a floor's height worked from the floor below and its storey height.
/// A floor that carries it follows that storey height when it is measured again.
fn follows(below: &str) -> String {
    format!("{below}'s floor and its storey height")
}

/// So many feet as a length written the way `was` is written: in inches, in feet and
/// inches, or in feet.
fn written_like(was: &Val<Len>, feet: f64) -> (Len, String) {
    let tidy = |x: f64| (x * 1.0e6).round() / 1.0e6 + 0.0;
    match was.unit.as_str() {
        units::IN => (Len::One(tidy(feet * 12.0)), units::IN.to_string()),
        units::FT_IN if feet >= 0.0 => {
            let whole = (feet + 1.0e-9).floor();
            let inches = tidy((feet - whole) * 12.0);
            if inches >= 12.0 { (Len::FtIn([whole + 1.0, 0.0]), units::FT_IN.to_string()) } else { (Len::FtIn([whole, inches]), units::FT_IN.to_string()) }
        }
        _ => (Len::One((feet * 1.0e9).round() / 1.0e9), units::FT.to_string()),
    }
}

/// The entered values of an element that `measure` takes, each by the name the screen
/// shows it under, and whether it is one length (true) or can only be marked "same".
/// Nothing is changed.
pub fn takes(job: &mut Job, path: &str) -> Vec<(String, bool)> {
    let Some(e) = job.find(path) else { return Vec::new() };
    let Some(kind) = job.kind(path) else { return Vec::new() };
    let listed: Vec<String> = crate::view::elements(job).into_iter().filter(|el| el.path == path).flat_map(|el| el.rows.into_iter().filter(|r| !r.derived && !r.label.contains(": ")).map(|r| r.label)).collect();
    let mut has = Vec::new();
    for label in listed {
        if let Ok(Some(found)) = put(job, e, path, kind, "", &label, Do::Probe, None, "") {
            has.push((label, found.said == "length"));
        }
    }
    has
}

/// The same, of the values named: for a caller that has the element's rows to hand
/// and asks about many elements, so the job is not read through again for each.
pub(crate) fn takes_among(job: &mut Job, path: &str, labels: &[String]) -> Vec<(String, bool)> {
    let Some(e) = job.find(path) else { return Vec::new() };
    let Some(kind) = job.kind(path) else { return Vec::new() };
    let mut has = Vec::new();
    for label in labels {
        if let Ok(Some(found)) = put(job, e, path, kind, "", label, Do::Probe, None, "") {
            has.push((label.clone(), found.said == "length"));
        }
    }
    has
}

/// Do `reading` to the element's value of that name. None when it has no such value.
#[allow(clippy::too_many_arguments)]
fn put(job: &mut Job, e: Entity, path: &str, kind: Kind, name: &str, what: &str, reading: Do, to: Option<Justify>, source: &str) -> Result<Option<Measured>, String> {
    {
        let name = name.to_string();
        let mut also = Vec::new();
        // One component after another, until one has a value of this name. `$slot` picks
        // the value out of a copy; the copy is checked whole before it replaces what is
        // there.
        macro_rules! on {
            ($comp:ty, $check:expr, |$c:ident| $slot:expr) => {
                if let Some(mut $c) = job.world.get::<$comp>(e).cloned() {
                    let slot: Option<Slot> = $slot;
                    if let Some(slot) = slot {
                        let said = apply(slot, &name, what, reading, source)?;
                        #[allow(clippy::redundant_closure_call)]
                        ($check)(&$c)?;
                        job.put(path, $c)?;
                        return Ok(Some(Measured { said, also }));
                    }
                }
            };
        }
        // A storey: its height is the floor to floor a person shoots, and it says where
        // the floors above are.
        if let Some(mut c) = job.world.get::<Level>(e).cloned() {
            let slot = match what {
                "elevation" => Some(Slot::Length(&mut c.elevation)),
                "storey height" => Some(Slot::Length(&mut c.height)),
                "rooms find their walls at" => c.computation_height.as_mut().map(Slot::Length),
                _ => None,
            };
            if let Some(slot) = slot {
                let said = apply(slot, &name, what, reading, source)?;
                building::check_level(path, kind, &c)?;
                job.put(path, c.clone())?;
                // A storey's floor, its height and the floor above say one thing three
                // ways. Whichever is measured or checked, the three are made to agree,
                // or the measurement is refused: by a length or by "same" alike.
                if !matches!(reading, Do::Probe) {
                    match what {
                        "storey height" => floors_above(job, e, path, &name, &c, source, &mut also)?,
                        "elevation" => floor_set(job, e, path, &name, &c, source, &mut also)?,
                        _ => {}
                    }
                }
                return Ok(Some(Measured { said, also }));
            }
        }
        on!(WallType, |c: &WallType| building::check_wall_type(path, kind, c), |c| if what == "wall thickness" { Some(Slot::Length(&mut c.thickness)) } else { layer_slot(&mut c.layers, what) });
        on!(FloorType, |c: &FloorType| building::check_floor_type(path, kind, c), |c| if what == "floor thickness" { Some(Slot::Length(&mut c.thickness)) } else { layer_slot(&mut c.layers, what) });
        on!(OpeningType, |c: &OpeningType| building::check_opening_type(path, kind, c), |c| match what {
            "width" => Some(Slot::Length(&mut c.width)),
            "height" => Some(Slot::Length(&mut c.height)),
            _ => None,
        });
        on!(Wall, |c: &Wall| building::check_wall(path, kind, c), |c| match what {
            "type" => Some(Slot::Other(&mut c.type_name)),
            "wall line" => Some(Slot::Other(&mut c.line)),
            "top level" => c.top_level.as_mut().map(|v| Slot::Other(v)),
            "height" => c.height.as_mut().map(Slot::Length),
            "base offset" => c.base_offset.as_mut().map(Slot::Length),
            "top offset" => c.top_offset.as_mut().map(Slot::Length),
            _ => None,
        });
        on!(Opening, |c: &Opening| building::check_opening(path, kind, c), |c| match what {
            "type" => Some(Slot::Other(&mut c.type_name)),
            "centre, along the wall" => Some(Slot::Length(&mut c.offset)),
            "sill" => Some(Slot::Length(&mut c.sill)),
            _ => None,
        });
        on!(Floor, |c: &Floor| building::check_floor(path, kind, c), |c| match what {
            "type" => Some(Slot::Other(&mut c.type_name)),
            "boundary" => Some(Slot::Other(&mut c.boundary)),
            "offset" => c.offset.as_mut().map(Slot::Length),
            _ => None,
        });
        on!(SegmentType, |c: &SegmentType| building::check_segment_type(path, kind, c), |c| match what {
            "diameter" if c.round => Some(Slot::Length(&mut c.width)),
            "width" if !c.round => Some(Slot::Length(&mut c.width)),
            "depth" => c.depth.as_mut().map(Slot::Length),
            _ => None,
        });
        if let Some(mut c) = job.world.get::<Segment>(e).cloned() {
            let level = c.end_offset.is_none();
            let end = match what {
                "height above its storey" if level => Some(false),
                "height above its storey, first end" if !level => Some(false),
                "height above its storey, second end" if !level => Some(true),
                _ => None,
            };
            if let Some(second) = end {
                if let (Some(to), Do::Length(_)) = (to, reading) {
                    if to != c.justify {
                        if !level {
                            return Err(format!(
                                "{name} slopes, and its heights are held to its {}: a measurement to its {} at one end would leave the other end a guess. Measure to its {}, or work the height out and enter that",
                                line_said(c.justify),
                                line_said(to),
                                line_said(c.justify)
                            ));
                        }
                        if to != Justify::Centre && !sized(job, &c) {
                            return Err(format!("{name}'s size is not known, so its centre cannot be worked from its {}; give its type a size first, or measure to its centre", line_said(to)));
                        }
                        also.push(format!("{name}'s height is now held to its {} (it was held to its {})", line_said(to), line_said(c.justify)));
                        c.justify = to;
                    }
                } else if to.is_some_and(|to| to != c.justify) {
                    return Err(format!("\"same\" is said of {name}'s height as it is held, to its {}; leave the line out", line_said(c.justify)));
                }
                let slot = if second { c.end_offset.as_mut().map(Slot::Length) } else { Some(Slot::Length(&mut c.offset)) };
                let slot = slot.ok_or_else(|| format!("{name} has no second height"))?;
                let said = apply(slot, &name, what, reading, source)?;
                building::check_segment(path, kind, &c)?;
                job.put(path, c)?;
                return Ok(Some(Measured { said, also }));
            }
            if to.is_some() {
                return Err(format!("only {name}'s height is measured to its bottom, its centre or its top"));
            }
        }
        on!(Segment, |c: &Segment| building::check_segment(path, kind, c), |c| match what {
            "type" => Some(Slot::Other(&mut c.type_name)),
            "line on the plan" => Some(Slot::Other(&mut c.line)),
            _ => None,
        });
        on!(Placed, |c: &Placed| building::check_placed(path, kind, c), |c| match what {
            "what it is" => Some(Slot::Other(&mut c.what)),
            "position" => Some(Slot::Other(&mut c.at)),
            "height above its storey" => Some(Slot::Length(&mut c.offset)),
            "turned" => c.angle.as_mut().map(|v| Slot::Other(v)),
            "size" => Some(Slot::Other(&mut c.size)),
            _ => None,
        });
        on!(Connectors, |c: &Connectors| building::check_connectors(path, kind, c), |c| numbered(what, "connector ").and_then(|(n, rest)| {
            let k = c.0.get_mut(n - 1)?;
            if rest == " size" {
                k.size.as_mut().map(Slot::Length)
            } else if rest.starts_with(" (") || rest.starts_with(": ") {
                Some(Slot::Other(&mut k.at))
            } else {
                None
            }
        }));
        on!(Room, |c: &Room| building::check_room(path, kind, c), |c| match what {
            "a point inside it" => Some(Slot::Other(&mut c.point)),
            "number" => c.number.as_mut().map(|v| Slot::Other(v)),
            _ => None,
        });
        on!(InSystem, |c: &InSystem| building::check_in_system(path, kind, c), |c| if what == "system" { Some(Slot::Other(&mut c.0)) } else { None });
        // The roof's own lengths.
        on!(Height, |c: &Height| check::height(path, kind, c), |c| if what == "height" { Some(Slot::Length(&mut c.0)) } else { None });
        // A plane solved from its roof's footprint has the roof's eave height: what is put
        // on the plane is lost the next time the roof is solved. It is the roof's to set.
        if what == "eave height" && job.world.get::<Solved>(e).is_some() {
            if matches!(reading, Do::Probe) {
                return Ok(None);
            }
            return Err(format!("{name} is solved from its roof's footprint, and its eave height is the roof's: set the roof's with \"set\", which solves the roof again"));
        }
        on!(Outline, |c: &Outline| check::outline(path, kind, c), |c| if what == "eave height" { Some(Slot::Length(&mut c.eave_height)) } else { None });
        if job.world.get::<Footprint>(e).is_some() && what == "eave height" {
            return Err(format!("{name} is a roof solved from its footprint: its eave height is set with \"set\", which solves the roof again"));
        }
        Ok(None)
    }
}

fn line_said(j: Justify) -> &'static str {
    match j {
        Justify::Top => "top",
        Justify::Centre => "centre",
        Justify::Bottom => "bottom",
    }
}

/// Whether the run's type gives it a size, so its centre can be worked from an edge.
fn sized(job: &mut Job, run: &Segment) -> bool {
    let Some(t) = run.type_name.value.as_ref().and_then(|p| job.find(p)) else { return false };
    job.world.get::<SegmentType>(t).is_some_and(|t| t.round_ft().is_some() || t.rect_ft().is_some())
}

/// The storeys beside this one (under the same parent), each with its floor's height in
/// feet where it has one, lowest first.
fn storeys_beside(job: &mut Job, me: Entity) -> Vec<(String, String, Level, Option<f64>)> {
    let parent = job.world.get::<Node>(me).and_then(|n| n.parent.clone());
    let mut q = job.world.query::<(&Node, &Level)>();
    let mut all: Vec<(String, String, Level, Option<f64>)> = q.iter(&job.world).filter(|(n, _)| n.parent == parent).map(|(n, l)| (n.path.clone(), n.name.clone(), l.clone(), l.elevation.feet())).collect();
    all.sort_by(|a, b| a.3.unwrap_or(f64::INFINITY).total_cmp(&b.3.unwrap_or(f64::INFINITY)));
    all
}

/// Walls on a storey that stand to their own height, taller than the storey now is:
/// each said, since nothing moves them.
fn walls_through(job: &mut Job, storey: &str, tall: f64, above: &str, also: &mut Vec<String>) {
    let mut q = job.world.query::<(&Node, &Wall)>();
    for (n, w) in q.iter(&job.world) {
        if n.parent.as_deref() != Some(storey) || w.top_level.is_some() {
            continue;
        }
        let Some(height) = w.height.as_ref().and_then(|h| h.feet()) else { continue };
        let top = w.base_offset.as_ref().and_then(|o| o.feet()).unwrap_or(0.0) + height;
        if top > tall + AGREE_FT {
            also.push(format!("{} stands {} by its own height: that is {} above {above}'s floor now. Measure its height, or hold its top to {above}", n.name, feet_said(top), feet_said(top - tall)));
        }
    }
}

/// A roof's eave height is its own value and does not follow a floor: said when a
/// floor has moved.
fn roofs_stay(job: &mut Job, also: &mut Vec<String>) {
    let mut q = job.world.query::<(&Node, &Kind)>();
    let roofs: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Roof).map(|(n, _)| n.name.clone()).collect();
    if !roofs.is_empty() {
        also.push(format!("the eave height of {} is its own value and did not move with the floor: check it against the walls under it", roofs.join(", ")));
    }
}

/// A storey's height was measured floor to floor, or checked: the floors above are where
/// that puts them. The next storey up sits at this floor plus the height, as sure as the
/// less sure of the two. Storeys above that move by the same amount and stay as sure as
/// they were. A floor that is Known in its own right is not moved: if it is the next one
/// up and this puts it more than an eighth of an inch elsewhere, the three values
/// disagree and the measurement is refused. A floor that is Known only because this
/// storey's height was measured before follows it when it is measured again.
fn floors_above(job: &mut Job, me: Entity, path: &str, name: &str, level: &Level, source: &str, also: &mut Vec<String>) -> Result<(), String> {
    let measured = level.height.feet().ok_or("the storey height is not a length this program can use")?;
    let Some(floor) = level.elevation.feet() else {
        also.push(format!("no floor above was moved: {name}'s own floor is not known"));
        return Ok(());
    };
    let beside = storeys_beside(job, me);
    for (_, n, _, z) in &beside {
        if z.is_none() {
            also.push(format!("{n}'s floor is not known and was left so"));
        }
    }
    let above: Vec<(String, String, Level, f64)> = beside.into_iter().filter(|(p, ..)| p != path).filter_map(|(p, n, l, z)| z.filter(|z| *z > floor + 1.0e-9).map(|z| (p, n, l, z))).collect();
    let Some((next_path, next_name, next, was)) = above.first().cloned() else { return Ok(()) };
    let target = ((floor + measured) * 1.0e9).round() / 1.0e9;
    // Known in its own right: not a floor that is Known only because this storey's
    // height was measured before. One that followed and was then checked on site has
    // had that said of it ("...; found as drawn"), and is its own.
    let followed = next.elevation.source.starts_with(&follows(name)) && !next.elevation.source.contains("; ");
    let its_own = next.elevation.status == Status::Known && !followed;
    if its_own {
        if (target - was).abs() > AGREE_FT {
            return Err(format!(
                "{name}'s floor is at {} ({}) and {next_name}'s floor is Known at {} ({}): that is {} floor to floor, and this measurement says {}. One of the three is wrong, and the program does not pick",
                feet_said(floor),
                level.elevation.source,
                feet_said(was),
                next.elevation.source,
                feet_said(was - floor),
                feet_said(measured)
            ));
        }
        if (target - was).abs() > 1.0e-6 {
            also.push(format!("{next_name}'s floor is Known at {}; this puts it at {}, within an eighth of an inch, and it was left where it is", feet_said(was), feet_said(target)));
        }
        walls_through(job, path, was - floor, &next_name, also);
        return Ok(());
    }
    let status = level.elevation.status.max(level.height.status);
    let mut moved = next.clone();
    let (value, unit) = written_like(&next.elevation, target);
    moved.elevation = Val { value: Some(value), unit, status, source: format!("{} ({source})", follows(name)) };
    building::check_level(&next_path, Kind::Storey, &moved)?;
    job.put(&next_path, moved)?;
    let by = target - was;
    if by.abs() > 1.0e-9 || status != next.elevation.status {
        also.push(format!("{next_name}'s floor is now at {}, {}; it was at {}", feet_said(target), status_said(status), feet_said(was)));
    }
    walls_through(job, path, measured, &next_name, also);
    if by.abs() < 1.0e-9 {
        return Ok(());
    }
    let how = if by > 0.0 { format!("moved up {}", feet_said(by)) } else { format!("moved down {}", feet_said(-by)) };
    let mut top = target;
    for (p, n, l, z) in above.into_iter().skip(1) {
        if l.elevation.status == Status::Known {
            if z <= top + 1.0e-9 {
                return Err(format!("{n}'s floor is Known at {} ({}), and this measurement puts the floor below it at {}: one of them is wrong, and the program does not pick", feet_said(z), l.elevation.source, feet_said(top)));
            }
            also.push(format!("{n}'s floor is Known at {} and was left there, with every floor above it", feet_said(z)));
            break;
        }
        let mut shifted = l.clone();
        top = ((z + by) * 1.0e9).round() / 1.0e9;
        let (value, unit) = written_like(&l.elevation, top);
        shifted.elevation = Val { value: Some(value), unit, status: l.elevation.status, source: after(&l.elevation.source, &format!("{how} with the floor below"), source) };
        building::check_level(&p, Kind::Storey, &shifted)?;
        job.put(&p, shifted)?;
        also.push(format!("{n}'s floor is now at {}; it was at {}", feet_said(top), feet_said(z)));
    }
    roofs_stay(job, also);
    Ok(())
}

/// A storey's floor was measured, or checked. The storey below's height is the way up
/// to this floor, and this storey's own height is the way up to the next: each is made
/// to agree with the floor, as sure as what it is worked from, or, where it is Known in
/// its own right and more than an eighth of an inch out, the measurement is refused.
fn floor_set(job: &mut Job, me: Entity, path: &str, name: &str, level: &Level, source: &str, also: &mut Vec<String>) -> Result<(), String> {
    let Some(floor) = level.elevation.feet() else { return Ok(()) };
    let beside = storeys_beside(job, me);
    // The storey below: the highest whose floor is under this one.
    let below = beside.iter().filter(|(p, ..)| p != path).filter(|(.., z)| z.is_some_and(|z| z < floor - 1.0e-9)).next_back().cloned();
    if let Some((below_path, below_name, below_level, Some(under))) = below {
        let way_up = ((floor - under) * 1.0e9).round() / 1.0e9;
        match below_level.height.feet() {
            Some(h) if (h - way_up).abs() <= AGREE_FT => {}
            Some(h) if below_level.height.status == Status::Known && below_level.elevation.status == Status::Known => {
                return Err(format!(
                    "{below_name}'s floor is Known at {} ({}) and its storey height is Known at {} ({}): that puts {name}'s floor at {}, and this says {}. One of the three is wrong, and the program does not pick",
                    feet_said(under),
                    below_level.elevation.source,
                    feet_said(h),
                    below_level.height.source,
                    feet_said(under + h),
                    feet_said(floor)
                ));
            }
            was => {
                let mut fitted = below_level.clone();
                let status = below_level.elevation.status.max(level.elevation.status);
                let (value, unit) = written_like(&below_level.height, way_up);
                fitted.height = Val { value: Some(value), unit, status, source: format!("{below_name}'s floor and {name}'s floor ({source})") };
                building::check_level(&below_path, Kind::Storey, &fitted)?;
                job.put(&below_path, fitted)?;
                also.push(format!("{below_name}'s storey height is now {}, {}: the way up from its floor to this one; it was {}", feet_said(way_up), status_said(status), was.map(feet_said).unwrap_or_else(|| "not known".to_string())));
                walls_through(job, &below_path, way_up, name, also);
            }
        }
    }
    // The storey above: this storey's own height is the way up to it.
    let above = beside.iter().filter(|(p, ..)| p != path).find(|(.., z)| z.is_some_and(|z| z > floor + 1.0e-9)).cloned();
    if let Some((_, above_name, above_level, Some(over))) = above {
        let way_up = ((over - floor) * 1.0e9).round() / 1.0e9;
        match level.height.feet() {
            Some(h) if (h - way_up).abs() <= AGREE_FT => {}
            // This storey's height is Known: the floor above follows it, or, Known in
            // its own right and elsewhere, refuses the measurement.
            Some(_) if level.height.status == Status::Known => floors_above(job, me, path, name, level, source, also)?,
            was => {
                let mut fitted = level.clone();
                let status = level.elevation.status.max(above_level.elevation.status);
                let (value, unit) = written_like(&level.height, way_up);
                fitted.height = Val { value: Some(value), unit, status, source: format!("{name}'s floor ({source}) and {above_name}'s floor") };
                building::check_level(path, Kind::Storey, &fitted)?;
                job.put(path, fitted)?;
                also.push(format!("{name}'s storey height is now {}, {}: the way up from this floor to {above_name}'s; it was {}", feet_said(way_up), status_said(status), was.map(feet_said).unwrap_or_else(|| "not known".to_string())));
                walls_through(job, path, way_up, &above_name, also);
            }
        }
    }
    Ok(())
}
