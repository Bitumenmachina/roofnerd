//! Where an item connects (call 211, the bench's step D): the rows an item holds, each a place on it where a pipe, a duct, a
//! conduit or a cable meets it, so that a thing set down from the item has its connections from that moment and a run can be
//! drawn to them.
//!
//! A row says what the connection is called, what it takes (a pipe, a duct, a conduit, power), what a pipe carries (cold,
//! hot, waste, vent, storm, gas), which way it goes (in, out, both), where it is as three figures, **across, back and up, the
//! same three a block of a shape is written in** (across from the middle, back from the back, up from the foot, each a formula
//! of the item's `width`, `depth` and `height`), which way it faces, its size, and how sure and where from. A place worked
//! for a size is turned into the thing's own axes by the shape's own function (`shape::thing_axes`): a block and a connection
//! written with the same figures land on the same point, turned and not.
//!
//! Ported from BIMCraft, `crates/core/src/family.rs` at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), under its
//! MIT licence (NOTICE): which connections each starter fixture has and which way each goes and faces (`Connector`'s `flow` and
//! `direction`, the latter kept to the six words a tradesman says: `Facing`), and the rules `check_connectors` holds (only a
//! pipe carries something; a place and a size that are lengths; a bound on how many). Ours: rows on an item and not on a
//! family, a place as figures of the item's three sizes and not of a family's parameters, the box rule, and every place
//! Assumed, never Known on an item's or a file's say-so.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use super::model::{ConnectorKind, Connector, Domain, Flow, MAX_CONNECTORS, MAX_SECTION_FT};
use super::shape::thing_axes;
use super::P3;
use crate::formula::{self, Expr, Operand};
use crate::units::{self, Len};
use crate::value::{Status, Val};

/// Which way a connection faces, in the thing's own axes: the way the pipe leaves it. BIMCraft's `direction` is a vector; a
/// tradesman says one of these six. Across is to the right looking at the thing's front, back is toward its wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Facing {
    Back,
    Front,
    Left,
    Right,
    Up,
    Down,
}

impl Facing {
    pub const ALL: [Facing; 6] = [Facing::Back, Facing::Front, Facing::Left, Facing::Right, Facing::Up, Facing::Down];

    /// The word a table and a person say it by.
    pub fn word(self) -> &'static str {
        match self {
            Facing::Back => "back",
            Facing::Front => "front",
            Facing::Left => "left",
            Facing::Right => "right",
            Facing::Up => "up",
            Facing::Down => "down",
        }
    }

    /// The facing of that word, however it is capitalised.
    pub fn named(word: &str) -> Option<Facing> {
        Facing::ALL.into_iter().find(|f| f.word().eq_ignore_ascii_case(word.trim()))
    }

    /// The way it points, in the thing's own axes (x to its right, y to its back, z up): a unit vector, as BIMCraft's
    /// `direction` is.
    pub fn vector(self) -> P3 {
        match self {
            Facing::Back => [0.0, 1.0, 0.0],
            Facing::Front => [0.0, -1.0, 0.0],
            Facing::Left => [-1.0, 0.0, 0.0],
            Facing::Right => [1.0, 0.0, 0.0],
            Facing::Up => [0.0, 0.0, 1.0],
            Facing::Down => [0.0, 0.0, -1.0],
        }
    }
}

fn is_both(f: &Flow) -> bool {
    *f == Flow::Both
}

/// One row of an item: a place where something connects to it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemConnection {
    /// What it is called on the item (`Cold water`, `Waste`): one of its own, whatever the capitals.
    pub name: String,
    /// What it takes: a pipe, a duct, a conduit or power.
    pub kind: ConnectorKind,
    /// What a pipe carries; none on any other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carries: Option<Domain>,
    #[serde(default, skip_serializing_if = "is_both")]
    pub flow: Flow,
    /// Across from the middle of the item, back from its back, up from its foot: each a formula of `width`, `depth` and
    /// `height` and a length (`-4 in`, `height - 12 in`; a plain 0 stands without a unit).
    pub across: String,
    pub back: String,
    pub up: String,
    /// Which way it faces; none where nothing says (a model's connection point whose placement gives no axis). Absent in the saved job where there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faces: Option<Facing>,
    /// How sure the place is: Assumed. A place is Known only where somebody measures it on the thing.
    pub place: Status,
    /// Where the place is from.
    pub place_from: String,
    /// What it takes in size (a pipe's nominal diameter), with how sure and where from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Val<Len>>,
}

/// `pl::item_connections`: on an item, where it connects.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemConnections(pub Vec<ItemConnection>);

/// What the three words of a place are called, in a refusal.
pub const PLACE_WORDS: [&str; 3] = ["Across", "Back", "Up"];

/// A length in feet as inches, to the thousandth, no trailing zeros ("22 in").
pub fn inches_of(ft: f64) -> String {
    // To the sixteenth, as a tape reads and as every length in a reply is written (call 213 point 8): 1/2 in, 5 1/2 in, 21 in.
    let parts = (ft.abs() * 12.0 * 16.0).round() as u64;
    if parts == 0 {
        return "0 in".to_string();
    }
    let (whole, mut top, mut bottom) = (parts / 16, parts % 16, 16);
    while top > 0 && top % 2 == 0 {
        top /= 2;
        bottom /= 2;
    }
    let sign = if ft < 0.0 { "-" } else { "" };
    match (whole, top) {
        (w, 0) => format!("{sign}{w} in"),
        (0, t) => format!("{sign}{t}/{bottom} in"),
        (w, t) => format!("{sign}{w} {t}/{bottom} in"),
    }
}

/// The item's sizes as a refusal says them: "24 in wide, 18 in deep, 34 in high".
pub fn size_in_inches(size: P3) -> String {
    format!("{} wide, {} deep, {} high", inches_of(size[0]), inches_of(size[1]), inches_of(size[2]))
}

fn lookup(name: &str, size: P3) -> Result<Operand, String> {
    let i = match name.to_ascii_lowercase().as_str() {
        "width" => 0,
        "depth" => 1,
        "height" => 2,
        _ => return Err(format!("{name:?} is not something a connection's place can be worked from: it has the item's width, depth and height")),
    };
    Ok(Operand { value: Some(size[i]), dim: 1, status: Status::Known, shown: name.to_ascii_lowercase() })
}

fn dim_said(dim: i32) -> &'static str {
    match dim {
        0 => "a plain number",
        1 => "a length",
        2 => "an area",
        3 => "a volume",
        _ => "not a length",
    }
}

/// One figure of a place worked for an item of `size`: a length in feet.
fn figure(item: &str, name: &str, which: &str, text: &str, size: P3) -> Result<f64, String> {
    let who = format!("{item:?}: the {name} connection's {which} figure");
    let expr = formula::parse(text).map_err(|why| format!("{who}, {text:?} {why}"))?;
    let worked = formula::work(&expr, &|n| lookup(n, size)).map_err(|why| format!("{who}, {text:?}: {why}"))?;
    // A plain 0 is no distance in any unit and is the one number that stands without a unit.
    let nothing = matches!(expr, Expr::Num(x) if x == 0.0);
    if worked.dim != 1 && !nothing {
        return Err(format!("{who}, {:?}, comes out as {}; a figure of a place is a length (write a fixed length with its unit: 4 in, 100 mm; only 0 stands without one)", formula::typed(&expr), dim_said(worked.dim)));
    }
    match worked.value {
        Some(x) if x.is_finite() => Ok(x),
        _ => Err(format!("{who}, {text:?}, comes to no figure")),
    }
}

/// A number as it is written to the file: no stray digits from the arithmetic (to a millionth).
fn tidy(x: f64) -> f64 {
    (x * 1.0e6).round() / 1.0e6 + 0.0
}

/// How far a place may reach past the item's box, in feet (a billionth: what working it leaves behind).
const REACH: f64 = 1.0e-8;

impl ItemConnection {
    /// The place worked for an item of `size` (feet), in the thing's own axes: x along its width from the middle, y to its
    /// back, z up from its foot, in feet. Refused where a figure is not a length of the item's three sizes. The box is not
    /// held here (`placed` does).
    pub fn worked(&self, item: &str, size: P3) -> Result<P3, String> {
        let across = figure(item, &self.name, PLACE_WORDS[0], &self.across, size)?;
        let back = figure(item, &self.name, PLACE_WORDS[1], &self.back, size)?;
        let up = figure(item, &self.name, PLACE_WORDS[2], &self.up, size)?;
        Ok(thing_axes(size, across, back, up))
    }

    /// The place worked and held to the item's box at its own size: on it or inside it. The refusal names the item, the
    /// connection and the figure, and says how far it reaches in inches, at the item's own size.
    pub fn placed(&self, item: &str, size: P3) -> Result<P3, String> {
        let at = self.worked(item, size)?;
        let figures = [&self.across, &self.back, &self.up];
        for (i, word) in PLACE_WORDS.iter().enumerate() {
            let (lo, hi) = if i == 2 { (0.0, size[2]) } else { (-size[i] / 2.0, size[i] / 2.0) };
            let (below, above) = (lo - at[i], at[i] - hi);
            if below.max(above) > REACH {
                // The back figure counts from the back, so past the back (y over the box) is behind it.
                let way = match (i, below > above) {
                    (0, true) => "to the left of",
                    (0, false) => "to the right of",
                    (1, true) => "in front of",
                    (1, false) => "behind",
                    (_, true) => "below",
                    (_, false) => "above",
                };
                return Err(format!("{item:?}: the {} connection's {word} figure, {}, puts it {} {way} the item ({}); a connection is on the item or inside it", self.name, figures[i], inches_of(below.max(above)), size_in_inches(size)));
            }
        }
        Ok(at)
    }

    /// The connection a thing set down from the item has: where the item's sizes put it, in inches in the thing's own axes,
    /// no surer than Assumed and no surer than the row and the sizes it is worked from (`sizes`), saying the item and the
    /// connection it came from.
    pub fn made(&self, item: &str, size: P3, sizes: Status) -> Result<Connector, String> {
        let at = self.placed(item, size)?;
        let status = Status::Assumed.max(self.place).max(sizes);
        Ok(Connector {
            name: self.name.clone(),
            kind: self.kind,
            domain: self.carries,
            flow: self.flow,
            at: Val { value: Some([tidy(at[0] * 12.0), tidy(at[1] * 12.0), tidy(at[2] * 12.0)]), unit: units::IN.into(), status, source: format!("the item {item}, its {} connection: {}", self.name, self.place_from) },
            size: self.size.clone().map(|mut s| {
                s.status = Status::Assumed.max(s.status);
                s
            }),
            facing: self.faces,
            of_item: Some(self.name.clone()),
        })
    }

    /// One line a person reads: its name, what it takes, carries and which way it goes, then its place in inches at the item's
    /// sizes, which way it faces and its size.
    pub fn said_at(&self, at: P3, size: P3) -> String {
        let what = [Some(kind_word(self.kind).to_string()), self.carries.map(|d| domain_word(d).to_string()), (self.flow != Flow::Both).then(|| flow_word(self.flow).to_string())].into_iter().flatten().collect::<Vec<_>>().join(", ");
        let across = at[0];
        let back = size[1] / 2.0 - at[1];
        // A size says where it is from where that is not where the place is from (a size typed on a connection is not the rough-in sheet's).
        let sized = self.size.as_ref().and_then(|s| s.value.as_ref().map(|l| format!("; size {}{}", units::show_length(l, &s.unit), if s.source != self.place_from || s.status != self.place { format!(" ({}: {})", status_word(s.status), s.source) } else { String::new() }))).unwrap_or_default();
        let faces = self.faces.map(|f| format!("faces {}", f.word())).unwrap_or_else(|| "which way it faces is not said".to_string());
        format!("{}: {what}; across {}, back {}, up {}; {faces}{sized}; {}: {}", self.name, inches_of(across), inches_of(back), inches_of(at[2]), status_word(self.place), self.place_from)
    }
}

pub fn kind_word(kind: ConnectorKind) -> &'static str {
    match kind {
        ConnectorKind::Power => "power",
        ConnectorKind::Pipe => "pipe",
        ConnectorKind::Duct => "duct",
        ConnectorKind::Conduit => "conduit",
    }
}

pub fn domain_word(domain: Domain) -> &'static str {
    match domain {
        Domain::Cold => "cold",
        Domain::Hot => "hot",
        Domain::Waste => "waste",
        Domain::Vent => "vent",
        Domain::Storm => "storm",
        Domain::Gas => "gas",
    }
}

pub fn flow_word(flow: Flow) -> &'static str {
    match flow {
        Flow::In => "in",
        Flow::Out => "out",
        Flow::Both => "both",
    }
}

fn status_word(s: Status) -> &'static str {
    match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    }
}

/// What an item's rows have to be, whatever the item's sizes: at most 64, each with a name of its own, only a pipe carrying
/// something, a place that is Assumed and says where it is from, a size that is more than nothing and no more than a duct or
/// pipe. `item` is how the item is called in a refusal. A place's figures are held at the item's sizes (`ItemConnection::placed`).
pub fn check_item_connections(item: &str, rows: &ItemConnections) -> Result<(), String> {
    if rows.0.len() > MAX_CONNECTORS {
        return Err(format!("{item:?} has {} connections; an item takes up to {MAX_CONNECTORS}", rows.0.len()));
    }
    for (i, r) in rows.0.iter().enumerate() {
        if r.name.trim().is_empty() || r.name.trim() != r.name {
            return Err(format!("{item:?}: a connection has to be called something, with no space at either end of its name (connection {})", i + 1));
        }
        if let Some(earlier) = rows.0[..i].iter().find(|e| e.name.eq_ignore_ascii_case(&r.name)) {
            return Err(format!("{item:?} has a connection called {:?} already: each connection of an item has a name of its own", earlier.name));
        }
        if let Some(d) = r.carries {
            if r.kind != ConnectorKind::Pipe {
                return Err(format!("{item:?}: the {} connection takes a {} and says it carries {}; only a pipe connection says what it carries (cold, hot, waste, vent, storm or gas)", r.name, kind_word(r.kind), domain_word(d)));
            }
        }
        match r.place {
            Status::Known => return Err(format!("{item:?}: the {} connection's place is said to be Known; a place is Known only where somebody measures it on the thing", r.name)),
            Status::Unknown => return Err(format!("{item:?}: the {} connection's place is said to be Unknown; a place nobody has is no row: give it, or take the connection away", r.name)),
            Status::Assumed => {}
        }
        if r.place_from.trim().is_empty() {
            return Err(format!("{item:?}: the {} connection does not say where its place is from; every place does", r.name));
        }
        if let Some(s) = &r.size {
            s.check_basics(&format!("{item:?}: the {} connection's size", r.name))?;
            let Some(ft) = s.feet() else {
                return Err(format!("{item:?}: the {} connection's size is Unknown; leave the size out where nobody has it", r.name));
            };
            if s.status == Status::Known {
                return Err(format!("{item:?}: the {} connection's size is said to be Known; a size is Known only where somebody measures it on the thing", r.name));
            }
            if !(ft.is_finite() && ft > 0.0) {
                return Err(format!("{item:?}: the {} connection's size has to be more than nothing", r.name));
            }
            if ft > MAX_SECTION_FT {
                return Err(format!("{item:?}: the {} connection's size is over {MAX_SECTION_FT} ft across", r.name));
            }
        }
    }
    Ok(())
}
