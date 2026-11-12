//! What each thing is finished with, read from what the estimate prices on it
//! (DECISIONS.md call 58). A wall, a roof plane, a window or a door is dressed in what
//! its own live cost lines, its material or its assembly name: the trade's words for a
//! cladding, a roofing, a window or a door. Nothing here is a number in the estimate;
//! it is one reading of the estimate, made once, that every picture of the house draws
//! from, so the picture and the price cannot disagree. A thing nothing names a finish
//! for is bare, and says so.

use crate::building::{self, Shape};
use crate::components::{Condition, Cost, Kind, MadeOf, Node};
use crate::job::Job;
use std::collections::HashMap;

/// What an outside wall is clad in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cladding {
    LapSiding,
    BoardAndBatten,
    Shingles,
    Brick,
    Stone,
    Stucco,
}

impl Cladding {
    pub fn said(self) -> &'static str {
        match self {
            Cladding::LapSiding => "lap siding",
            Cladding::BoardAndBatten => "board and batten",
            Cladding::Shingles => "shingles",
            Cladding::Brick => "brick veneer",
            Cladding::Stone => "stone veneer",
            Cladding::Stucco => "stucco",
        }
    }
    /// Unit masonry and stucco: an opening in it has a brickmould and a sill, not a flat
    /// casing, and its corners need no corner board.
    pub fn masonry(self) -> bool {
        matches!(self, Cladding::Brick | Cladding::Stone | Cladding::Stucco)
    }
}

/// What a roof plane is covered with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Roofing {
    AsphaltShingles,
    StandingSeam,
    Slate,
    Tile,
    Shakes,
    Membrane,
}

impl Roofing {
    pub fn said(self) -> &'static str {
        match self {
            Roofing::AsphaltShingles => "asphalt shingles",
            Roofing::StandingSeam => "standing seam",
            Roofing::Slate => "slate",
            Roofing::Tile => "tile",
            Roofing::Shakes => "wood shakes",
            Roofing::Membrane => "membrane",
        }
    }
}

/// How a window operates, which is how its sash is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sash {
    Fixed,
    DoubleHung,
    Casement,
    Slider,
}

impl Sash {
    pub fn said(self) -> &'static str {
        match self {
            Sash::Fixed => "fixed",
            Sash::DoubleHung => "double-hung",
            Sash::Casement => "casement",
            Sash::Slider => "slider",
        }
    }
}

/// How a door's leaf is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Leaf {
    Flush,
    Panel,
    Glazed,
    Overhead,
}

impl Leaf {
    pub fn said(self) -> &'static str {
        match self {
            Leaf::Flush => "flush",
            Leaf::Panel => "panel",
            Leaf::Glazed => "glazed",
            Leaf::Overhead => "overhead",
        }
    }
}

/// A color named in a line, and the color it is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tone {
    pub word: &'static str,
    pub rgb: [u8; 3],
}

/// A finish, the color named with it, and what on the element said so.
#[derive(Clone, Debug, PartialEq)]
pub struct Said<T> {
    pub what: T,
    pub tone: Option<Tone>,
    /// The line, material or assembly that names it, as it is written.
    pub by: String,
    /// Anything else on the element that names a different finish of the same kind:
    /// what it names and what says so. The first is the one drawn; these are priced
    /// and not drawn, and every reading says so.
    pub also: Vec<(String, String)>,
}

/// Which face of a wall is outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outside {
    /// To the left of the wall's line, going from its start to its end.
    Left,
    Right,
    /// Both faces are inside the outline its level's walls enclose: a partition.
    Neither,
    /// Its level's walls enclose nothing, so which face is out is not known.
    NotKnown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WallDress {
    pub outside: Outside,
    /// The way the outside face looks, on plan: a unit vector in feet east and north.
    pub out: Option<[f64; 2]>,
    pub cladding: Option<Said<Cladding>>,
    /// A cladding on the lowest part of the wall, how high it goes in feet, and whether
    /// that height was written in the line (false: a wainscot with no height written is
    /// drawn 3 ft high and says so).
    pub wainscot: Option<(Said<Cladding>, f64, bool)>,
    /// How far past each end of its centre line the wall is drawn so that it closes
    /// with the wall it corners with.
    pub past: [f64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpeningDress {
    pub sash: Option<Said<Sash>>,
    pub leaf: Option<Said<Leaf>>,
    /// Divided lites, when a line says so: across and up in each sash when it gives
    /// them ("6 over 6" is 3 across and 2 up in each), otherwise left to the size.
    pub divided: Option<Option<(u32, u32)>>,
    pub tone: Option<Tone>,
}

/// Every finish read from the job.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dress {
    pub walls: HashMap<String, WallDress>,
    pub roofs: HashMap<String, Said<Roofing>>,
    pub openings: HashMap<String, OpeningDress>,
    /// Eaves with a gutter priced on them, and the color named.
    pub gutters: HashMap<String, Said<()>>,
}

/// The words of a text, lower case, split wherever a letter or a digit stops.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

/// Whether the words hold this phrase, word for word.
fn has(words: &[String], phrase: &str) -> bool {
    let want: Vec<&str> = phrase.split(' ').collect();
    words.windows(want.len()).any(|w| w.iter().zip(&want).all(|(a, b)| a == b))
}

fn any(words: &[String], phrases: &[&str]) -> bool {
    phrases.iter().any(|p| has(words, p))
}

const TONES: &[(&str, [u8; 3])] = &[
    ("off white", [0xEC, 0xE7, 0xDA]),
    ("light gray", [0xC6, 0xC9, 0xCA]),
    ("light grey", [0xC6, 0xC9, 0xCA]),
    ("dark gray", [0x5C, 0x61, 0x66]),
    ("dark grey", [0x5C, 0x61, 0x66]),
    ("white", [0xF0, 0xEE, 0xE8]),
    ("cream", [0xEC, 0xE4, 0xCE]),
    ("ivory", [0xEE, 0xE8, 0xD6]),
    ("buff", [0xD6, 0xBE, 0x96]),
    ("tan", [0xC4, 0xAA, 0x84]),
    ("beige", [0xD6, 0xC9, 0xB2]),
    ("sand", [0xD4, 0xC5, 0xA4]),
    ("gray", [0x98, 0x9D, 0xA1]),
    ("grey", [0x98, 0x9D, 0xA1]),
    ("charcoal", [0x4A, 0x4E, 0x53]),
    ("black", [0x2C, 0x2E, 0x31]),
    ("red", [0x98, 0x4A, 0x3A]),
    ("brown", [0x6C, 0x4D, 0x37]),
    ("bronze", [0x58, 0x46, 0x36]),
    ("navy", [0x30, 0x3E, 0x56]),
    ("blue", [0x5C, 0x7A, 0x96]),
    ("sage", [0x9A, 0xA6, 0x8C]),
    ("green", [0x5A, 0x70, 0x56]),
    ("yellow", [0xD8, 0xC0, 0x78]),
    ("natural", [0xB4, 0x86, 0x5C]),
    ("cedar", [0xB4, 0x86, 0x5C]),
    ("redwood", [0x9A, 0x5C, 0x44]),
    ("mahogany", [0x6E, 0x3E, 0x2C]),
    ("walnut", [0x5E, 0x44, 0x30]),
    ("oak", [0xB8, 0x94, 0x64]),
    ("copper", [0xB7, 0x73, 0x3B]),
    ("zinc", [0x8C, 0x95, 0x9C]),
    ("galvalume", [0xB2, 0xB8, 0xBD]),
];

/// The color a text names, if it names one.
pub fn tone_in(text: &str) -> Option<Tone> {
    let w = words(text);
    TONES.iter().find(|(word, _)| has(&w, word)).map(|(word, rgb)| Tone { word, rgb: *rgb })
}

/// The cladding a text names, if it names one.
pub fn cladding_in(text: &str) -> Option<Cladding> {
    let w = words(text);
    if any(&w, &["board and batten", "batten", "battens"]) {
        Some(Cladding::BoardAndBatten)
    } else if any(&w, &["brick", "bricks"]) {
        Some(Cladding::Brick)
    } else if any(&w, &["stone", "ledgestone", "fieldstone", "limestone", "ashlar", "granite"]) {
        Some(Cladding::Stone)
    } else if any(&w, &["stucco", "eifs"]) {
        Some(Cladding::Stucco)
    } else if any(&w, &["shingle", "shingles", "shake", "shakes"]) {
        Some(Cladding::Shingles)
    } else if any(&w, &["siding", "clapboard", "clapboards", "lap", "bevel"]) {
        Some(Cladding::LapSiding)
    } else {
        None
    }
}

/// The roofing a text names, if it names one.
pub fn roofing_in(text: &str) -> Option<Roofing> {
    let w = words(text);
    if any(&w, &["standing seam", "metal roofing", "metal roof", "copper", "zinc", "seam panels"]) {
        Some(Roofing::StandingSeam)
    } else if has(&w, "slate") {
        Some(Roofing::Slate)
    } else if any(&w, &["tile", "tiles", "barrel"]) {
        Some(Roofing::Tile)
    } else if any(&w, &["shake", "shakes", "cedar shingle", "cedar shingles", "wood shingle", "wood shingles"]) {
        Some(Roofing::Shakes)
    } else if any(&w, &["asphalt", "shingle", "shingles", "composition", "architectural"]) {
        Some(Roofing::AsphaltShingles)
    } else if any(&w, &["membrane", "epdm", "tpo", "built up", "bitumen"]) {
        Some(Roofing::Membrane)
    } else {
        None
    }
}

/// How a window operates, if a text says.
pub fn sash_in(text: &str) -> Option<Sash> {
    let w = words(text);
    if any(&w, &["double hung", "single hung"]) {
        Some(Sash::DoubleHung)
    } else if any(&w, &["casement", "casements"]) {
        Some(Sash::Casement)
    } else if any(&w, &["slider", "sliding", "gliding"]) {
        Some(Sash::Slider)
    } else if any(&w, &["fixed", "picture", "awning"]) {
        Some(Sash::Fixed)
    } else {
        None
    }
}

/// How a door's leaf is built, if a text says.
pub fn leaf_in(text: &str) -> Option<Leaf> {
    let w = words(text);
    if any(&w, &["overhead", "garage"]) {
        Some(Leaf::Overhead)
    } else if any(&w, &["french", "glazed", "glass", "full lite", "full light"]) {
        Some(Leaf::Glazed)
    } else if any(&w, &["panel", "panels", "paneled", "panelled"]) {
        Some(Leaf::Panel)
    } else if has(&w, "flush") {
        Some(Leaf::Flush)
    } else {
        None
    }
}

/// Whether a text says the lites are divided, and how when it says "6 over 6".
pub fn divided_in(text: &str) -> Option<Option<(u32, u32)>> {
    let w = words(text);
    // "6 over 6": that many lites in each sash, laid out as near square as they go.
    if let Some(i) = w.iter().position(|x| x == "over") {
        if let (Some(a), Some(b)) = (i.checked_sub(1).and_then(|j| w[j].parse::<u32>().ok()), w.get(i + 1).and_then(|x| x.parse::<u32>().ok())) {
            if a == b && (2..=12).contains(&a) {
                let across = match a {
                    2 | 4 => 2,
                    6 | 9 => 3,
                    8 | 12 => 4,
                    _ => a,
                };
                return Some(Some((across, (a / across).max(1))));
            }
        }
    }
    any(&w, &["divided", "muntin", "muntins", "grille", "grilles", "colonial", "sdl", "tdl"]).then_some(None)
}

/// A height written in a text, in feet: the first part between commas that reads as a
/// length ("Stone veneer wainscot, 3 ft").
fn height_in(text: &str) -> Option<f64> {
    text.split(',').skip(1).find_map(|part| crate::units::parse_length(part.trim()).ok().and_then(|(len, unit)| len.feet(unit).ok())).filter(|h| *h > 0.0)
}

/// What on an element could name a finish: its live cost lines as they are written,
/// then its material, then its assembly. A line that is switched off is out of the
/// estimate, so it dresses nothing.
fn texts(job: &Job, e: bevy_ecs::entity::Entity) -> Vec<(String, String)> {
    let w = &job.world;
    let mut out = Vec::new();
    if let Some(cost) = w.get::<Cost>(e) {
        out.extend(cost.lines.iter().filter(|l| !l.off).map(|l| (l.description.clone(), format!("the line \"{}\"", l.description))));
    }
    if let Some(m) = w.get::<MadeOf>(e).and_then(|m| m.0.value.clone()) {
        out.push((m.clone(), format!("its material, {m}")));
    }
    if let Some(c) = w.get::<Condition>(e).and_then(|c| c.0.value.clone()) {
        out.push((c.clone(), format!("its assembly, {c}")));
    }
    out
}

fn inside(ring: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut odd = false;
    for i in 0..ring.len() {
        let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            odd = !odd;
        }
    }
    odd
}

/// Read every finish in the job.
pub fn dress(job: &mut Job) -> Dress {
    let mut out = Dress::default();
    let (placed, _) = building::placements(job);
    // Each wall's centre line, thickness and level.
    struct W {
        path: String,
        storey: Option<String>,
        a: [f64; 2],
        b: [f64; 2],
        t: f64,
    }
    let walls: Vec<W> = placed
        .iter()
        .filter_map(|p| match &p.shape {
            Shape::Wall { a, b, thickness, .. } => Some(W { path: p.path.clone(), storey: p.storey.clone(), a: *a, b: *b, t: *thickness }),
            _ => None,
        })
        .collect();
    // The outline each level's walls enclose.
    let mut outlines: HashMap<String, Option<Vec<[f64; 2]>>> = HashMap::new();
    for w in &walls {
        if let Some(s) = &w.storey {
            if !outlines.contains_key(s) {
                let ring = building::floor_outline(job, s).ok().flatten().map(|o| o.boundary);
                outlines.insert(s.clone(), ring);
            }
        }
    }
    let index: HashMap<String, bevy_ecs::entity::Entity> = job.order.iter().filter_map(|e| job.world.get::<Node>(*e).map(|n| (n.path.clone(), *e))).collect();
    for (i, w) in walls.iter().enumerate() {
        let (dx, dy) = (w.b[0] - w.a[0], w.b[1] - w.a[1]);
        let length = (dx * dx + dy * dy).sqrt();
        if length < 1e-6 {
            continue;
        }
        let (u, left) = ([dx / length, dy / length], [-dy / length, dx / length]);
        let mid = [(w.a[0] + w.b[0]) / 2.0, (w.a[1] + w.b[1]) / 2.0];
        let reach = w.t / 2.0 + 0.25;
        let outside = match w.storey.as_ref().and_then(|s| outlines.get(s)).and_then(|r| r.as_ref()) {
            None => Outside::NotKnown,
            Some(ring) => {
                let l = inside(ring, [mid[0] + left[0] * reach, mid[1] + left[1] * reach]);
                let r = inside(ring, [mid[0] - left[0] * reach, mid[1] - left[1] * reach]);
                match (l, r) {
                    (false, true) => Outside::Left,
                    (true, false) => Outside::Right,
                    (true, true) => Outside::Neither,
                    (false, false) => Outside::NotKnown,
                }
            }
        };
        let out_way = match outside {
            Outside::Left => Some(left),
            Outside::Right => Some([-left[0], -left[1]]),
            _ => None,
        };
        // Where an end corners with another wall of its level, how far past the end it
        // is drawn: to where the two centre lines cross, and half the other's thickness
        // on (more at a corner that is not square).
        let mut past = [0.0, 0.0];
        for (k, (end, away)) in [(w.a, [-u[0], -u[1]]), (w.b, u)].into_iter().enumerate() {
            for (j, o) in walls.iter().enumerate() {
                if j == i || o.storey != w.storey {
                    continue;
                }
                let (ox, oy) = (o.b[0] - o.a[0], o.b[1] - o.a[1]);
                let ol = (ox * ox + oy * oy).sqrt();
                if ol < 1e-6 {
                    continue;
                }
                let ou = [ox / ol, oy / ol];
                let sin = (u[0] * ou[1] - u[1] * ou[0]).abs();
                if sin < 0.2 {
                    continue;
                }
                let near = (w.t + o.t) / 2.0 + 0.05;
                let meets = [o.a, o.b].iter().any(|q| ((q[0] - end[0]).powi(2) + (q[1] - end[1]).powi(2)).sqrt() <= near);
                if !meets {
                    continue;
                }
                // Where the two centre lines cross, measured out along this wall from its end.
                let d = [o.a[0] - end[0], o.a[1] - end[1]];
                let s = (d[0] * ou[1] - d[1] * ou[0]) / (away[0] * ou[1] - away[1] * ou[0]);
                let e = (s + o.t / 2.0 / sin.max(0.5)).clamp(-w.t, 2.0 * w.t.max(o.t));
                if e.abs() > past[k] {
                    past[k] = e;
                }
            }
        }
        let on = index.get(&w.path).map(|e| texts(job, *e)).unwrap_or_default();
        let mut cladding = None;
        let mut wainscot = None;
        for (text, by) in &on {
            let Some(what) = cladding_in(text) else { continue };
            let said = Said { what, tone: tone_in(text), by: by.clone(), also: Vec::new() };
            if has(&words(text), "wainscot") {
                if wainscot.is_none() {
                    let h = height_in(text);
                    wainscot = Some((said, h.unwrap_or(3.0), h.is_some()));
                }
            } else {
                match &mut cladding {
                    None => cladding = Some(said),
                    Some(first) if first.what != what => first.also.push((what.said().to_string(), by.clone())),
                    Some(_) => {}
                }
            }
        }
        out.walls.insert(w.path.clone(), WallDress { outside, out: out_way, cladding, wainscot, past });
    }
    // Roof planes, openings and gutters.
    for &e in &job.order.clone() {
        let (Some(node), Some(kind)) = (job.world.get::<Node>(e), job.world.get::<Kind>(e).copied()) else { continue };
        let path = node.path.clone();
        let on = texts(job, e);
        match kind {
            k if k.is_area() => {
                let mut named = on.iter().filter_map(|(text, by)| roofing_in(text).map(|r| (r, text, by)));
                if let Some((what, text, by)) = named.next() {
                    let also = named.filter(|(r, _, _)| *r != what).map(|(r, _, by)| (r.said().to_string(), by.clone())).collect();
                    out.roofs.insert(path, Said { what, tone: tone_in(text), by: by.clone(), also });
                }
            }
            Kind::Window | Kind::Door => {
                let mut d = OpeningDress { sash: None, leaf: None, divided: None, tone: None };
                for (text, by) in &on {
                    if kind == Kind::Window && d.sash.is_none() {
                        d.sash = sash_in(text).map(|what| Said { what, tone: None, by: by.clone(), also: Vec::new() });
                    }
                    if kind == Kind::Door && d.leaf.is_none() {
                        d.leaf = leaf_in(text).map(|what| Said { what, tone: None, by: by.clone(), also: Vec::new() });
                    }
                    if d.divided.is_none() {
                        d.divided = divided_in(text);
                    }
                    // A color belongs to the unit, not to its hardware or its flashing.
                    if d.tone.is_none() && any(&words(text), &["window", "door", "unit", "sash", "clad"]) {
                        d.tone = tone_in(text);
                    }
                }
                out.openings.insert(path, d);
            }
            Kind::Eave => {
                if let Some((text, by)) = on.iter().find(|(text, _)| any(&words(text), &["gutter", "gutters"])) {
                    out.gutters.insert(path, Said { what: (), tone: tone_in(text), by: by.clone(), also: Vec::new() });
                }
            }
            _ => {}
        }
    }
    out
}

/// A compass word for a way on plan: where an outside face looks.
pub fn compass(way: [f64; 2]) -> &'static str {
    let deg = way[1].atan2(way[0]).to_degrees();
    let names = ["east", "northeast", "north", "northwest", "west", "southwest", "south", "southeast"];
    names[(((deg + 360.0 + 22.5) % 360.0) / 45.0) as usize % 8]
}

/// The reading written out, one line for each wall, roof plane, window and door, in the
/// job's order: what it is finished with and what said so, or that it is bare.
pub fn report(job: &mut Job) -> Vec<String> {
    let d = dress(job);
    let mut out = Vec::new();
    let tone = |t: &Option<Tone>| t.map(|t| format!(", {}", t.word)).unwrap_or_default();
    let also = |others: &[(String, String)]| others.iter().map(|(what, by)| format!("; also priced on it and not drawn: {what} ({by})")).collect::<String>();
    for &e in &job.order.clone() {
        let (Some(node), Some(kind)) = (job.world.get::<Node>(e), job.world.get::<Kind>(e).copied()) else { continue };
        let name = format!("{} ({})", node.name, node.path);
        match kind {
            Kind::Wall => {
                let Some(w) = d.walls.get(&node.path) else {
                    out.push(format!("{name}: not drawn: it cannot be placed"));
                    continue;
                };
                let face = match (w.outside, w.out) {
                    (_, Some(way)) => format!("on its {} face", compass(way)),
                    (Outside::Neither, _) => "a partition: both its faces are inside".to_string(),
                    _ => "which face is outside is not known (its level's walls enclose nothing)".to_string(),
                };
                let lower = w.wainscot.as_ref().map(|(s, h, written)| format!("; {}{} wainscot to {}{} ({})", s.what.said(), tone(&s.tone), crate::units::feet_as_ft_in(*h, 16), if *written { "" } else { ", a height no line gives" }, s.by)).unwrap_or_default();
                match (&w.cladding, w.outside) {
                    (Some(c), Outside::Neither) => out.push(format!("{name}: {face}, so no cladding is drawn on it; {} is priced on it ({})", c.what.said(), c.by)),
                    (Some(c), Outside::NotKnown) => out.push(format!("{name}: {}{}, drawn on both faces: {face} ({}){lower}{}", c.what.said(), tone(&c.tone), c.by, also(&c.also))),
                    (Some(c), _) => out.push(format!("{name}: {}{}, {face} ({}){lower}{}", c.what.said(), tone(&c.tone), c.by, also(&c.also))),
                    (None, Outside::Neither) => out.push(format!("{name}: {face}")),
                    (None, _) if w.wainscot.is_some() => out.push(format!("{name}: bare above, {face}{lower}")),
                    (None, _) => out.push(format!("{name}: bare, {face}: nothing priced on it names a cladding")),
                }
            }
            k if k.is_area() => match d.roofs.get(&node.path) {
                Some(r) => out.push(format!("{name}: {}{} ({}){}", r.what.said(), tone(&r.tone), r.by, also(&r.also))),
                None => out.push(format!("{name}: bare: nothing priced on it names a roofing")),
            },
            Kind::Window => {
                let o = d.openings.get(&node.path);
                let sash = o.and_then(|o| o.sash.as_ref());
                let divided = match o.and_then(|o| o.divided) {
                    Some(Some((a, u))) => format!(", divided lites {a} across and {u} up in each sash"),
                    Some(None) => ", divided lites".to_string(),
                    None => String::new(),
                };
                match sash {
                    Some(s) => out.push(format!("{name}: {}{divided}{} ({})", s.what.said(), tone(&o.and_then(|o| o.tone)), s.by)),
                    None => out.push(format!("{name}: a frame, one sash and one light{divided}{}: nothing priced on it says how it operates", tone(&o.and_then(|o| o.tone)))),
                }
            }
            Kind::Door => {
                let o = d.openings.get(&node.path);
                match o.and_then(|o| o.leaf.as_ref()) {
                    Some(l) => out.push(format!("{name}: a {} door{} ({})", l.what.said(), tone(&o.and_then(|o| o.tone)), l.by)),
                    None => out.push(format!("{name}: a frame and a plain leaf{}: nothing priced on it says how it is built", tone(&o.and_then(|o| o.tone)))),
                }
            }
            Kind::Eave => {
                if let Some(g) = d.gutters.get(&node.path) {
                    out.push(format!("{name}: a gutter{} ({})", tone(&g.tone), g.by));
                }
            }
            _ => {}
        }
    }
    out
}

/// What one thing is finished with, for a panel: each a few words of what, and under it
/// what says so. Nothing for a thing that takes no finish (a partition with nothing on it).
pub fn said_of(d: &Dress, path: &str, kind: Kind) -> Vec<(String, String)> {
    let tone = |t: &Option<Tone>| t.map(|t| format!(", {}", t.word)).unwrap_or_default();
    let cap = |s: &str| {
        let mut c = s.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    };
    let mut out = Vec::new();
    match kind {
        Kind::Wall => {
            let Some(w) = d.walls.get(path) else { return out };
            let face = match (w.outside, w.out) {
                (_, Some(way)) => format!("On its {} face", compass(way)),
                (Outside::Neither, _) => "A partition: both faces are inside, so none is drawn".to_string(),
                _ => "On both faces: its level's walls enclose nothing, so which is outside is not known".to_string(),
            };
            match (&w.cladding, w.outside) {
                (Some(c), _) => {
                    out.push((cap(&format!("{}{}", c.what.said(), tone(&c.tone))), format!("{face}. From {}", c.by)));
                    out.extend(c.also.iter().map(|(what, by)| (format!("Also priced, not drawn: {what}"), format!("From {by}"))));
                }
                (None, Outside::Neither) => {}
                (None, _) => out.push(("Bare".to_string(), "Nothing priced on it names a cladding".to_string())),
            }
            if let Some((s, h, written)) = &w.wainscot {
                out.push((cap(&format!("{}{} wainscot to {}", s.what.said(), tone(&s.tone), crate::units::feet_as_ft_in(*h, 16))), format!("From {}{}", s.by, if *written { "" } else { ". No line gives its height; drawn 3 ft" })));
            }
        }
        k if k.is_area() => match d.roofs.get(path) {
            Some(r) => {
                out.push((cap(&format!("{}{}", r.what.said(), tone(&r.tone))), format!("From {}", r.by)));
                out.extend(r.also.iter().map(|(what, by)| (format!("Also priced, not drawn: {what}"), format!("From {by}"))));
            }
            None => out.push(("Bare".to_string(), "Nothing priced on it names a roofing".to_string())),
        },
        Kind::Window => {
            let o = d.openings.get(path);
            let divided = if o.and_then(|o| o.divided).is_some() { ", divided lites" } else { "" };
            match o.and_then(|o| o.sash.as_ref()) {
                Some(s) => out.push((cap(&format!("{}{divided}{}", s.what.said(), tone(&o.and_then(|o| o.tone)))), format!("From {}", s.by))),
                None => out.push((format!("A frame, one sash, one light{divided}"), "Nothing priced on it says how it operates".to_string())),
            }
        }
        Kind::Door => {
            let o = d.openings.get(path);
            match o.and_then(|o| o.leaf.as_ref()) {
                Some(l) => out.push((cap(&format!("{} door{}", l.what.said(), tone(&o.and_then(|o| o.tone)))), format!("From {}", l.by))),
                None => out.push(("A frame and a plain leaf".to_string(), "Nothing priced on it says how it is built".to_string())),
            }
        }
        Kind::Eave => {
            if let Some(g) = d.gutters.get(path) {
                out.push((format!("Gutter{}", tone(&g.tone)), format!("From {}", g.by)));
            }
        }
        _ => {}
    }
    out
}
