//! What each thing is finished with, read from what the estimate prices on it
//! (DECISIONS.md call 58). A wall, a roof plane, a window or a door is dressed in what
//! its own live cost lines, its material or its assembly name: the trade's words for a
//! cladding, a roofing, a window or a door. Nothing here is a number in the estimate;
//! it is one reading of the estimate, made once, that every picture of the house draws
//! from, so the picture and the price cannot disagree. A thing nothing names a finish
//! for is bare, and says so.
//!
//! **A line names a finish only when the finish is what the line is** (the seventeenth
//! judgment, J17B-4, 5 and 7). "Stone sill at windows" is a sill, "#30 asphalt felt
//! underlayment" is an underlayment and "Lap siding with brick mold casing" is lap
//! siding: a finish's word somewhere in a line does not make the line that finish. A
//! line is read by its head, the words before its first comma and before "with",
//! "over", "on", "at" and the like. A head that holds a word for something else (a
//! sill, a flashing, nails, a removal, paint, trim) is not a finish. A head that names
//! a finish is that finish. A head that names only the kind of thing ("Siding",
//! "Roofing", "Window unit") is refined by what follows its commas, as estimators
//! write it: "Roofing, standing seam, charcoal".

use crate::building::{self, Shape};
use crate::components::{Condition, Cost, Kind, MadeOf, Node};
use crate::job::Job;
use std::collections::HashMap;

/// What an outside wall is clad in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cladding {
    LapSiding,
    VerticalSiding,
    BoardAndBatten,
    Shingles,
    Panel,
    MetalPanel,
    Brick,
    Block,
    Stone,
    Stucco,
}

impl Cladding {
    pub fn said(self) -> &'static str {
        match self {
            Cladding::LapSiding => "lap siding",
            Cladding::VerticalSiding => "vertical siding",
            Cladding::BoardAndBatten => "board and batten",
            Cladding::Shingles => "shingles",
            Cladding::Panel => "panel siding",
            Cladding::MetalPanel => "metal siding",
            Cladding::Brick => "brick veneer",
            Cladding::Block => "concrete block",
            Cladding::Stone => "stone veneer",
            Cladding::Stucco => "stucco",
        }
    }
    /// Unit masonry and stucco: an opening in it has a brickmould and a sill, not a flat
    /// casing, and its corners need no corner board.
    pub fn masonry(self) -> bool {
        matches!(self, Cladding::Brick | Cladding::Block | Cladding::Stone | Cladding::Stucco)
    }
}

/// What a roof plane is covered with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Roofing {
    AsphaltShingles,
    StandingSeam,
    MetalPanel,
    MetalShingles,
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
            Roofing::MetalPanel => "metal panels",
            Roofing::MetalShingles => "metal shingles",
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
    Awning,
    Hopper,
}

impl Sash {
    pub fn said(self) -> &'static str {
        match self {
            Sash::Fixed => "fixed",
            Sash::DoubleHung => "double-hung",
            Sash::Casement => "casement",
            Sash::Slider => "slider",
            Sash::Awning => "awning",
            Sash::Hopper => "hopper",
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
    /// With its article: "a panel door", "an overhead door".
    pub fn said(self) -> &'static str {
        match self {
            Leaf::Flush => "a flush door",
            Leaf::Panel => "a panel door",
            Leaf::Glazed => "a glazed door",
            Leaf::Overhead => "an overhead door",
        }
    }
}

/// Divided lites: how many in the upper sash (or the only one) and in the lower, when a
/// line gives a count ("6 over 1"); `None` is left to the size of the sash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lites {
    pub upper: Option<u32>,
    pub lower: Option<u32>,
}

impl Lites {
    fn said(self) -> String {
        match (self.upper, self.lower) {
            (Some(a), Some(b)) => format!("divided lites, {a} over {b}"),
            _ => "divided lites".to_string(),
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
    /// Anything else on the element that names another finish of the same kind, or the
    /// same one in another color: what it names and what says so. The first is the one
    /// drawn; these are priced and not drawn, and every reading says so.
    pub also: Vec<(String, String)>,
}

/// Which face of a wall is outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outside {
    /// To the left of the wall's line, going from its start to its end.
    Left,
    Right,
    /// Both faces are indoors: a partition.
    Neither,
    /// Both faces are outdoors: it stands outside everything its level's walls enclose
    /// (a garden wall), and what is priced on it is drawn on both faces.
    Both,
    /// Its level's walls enclose nothing, so which face is out is not known.
    NotKnown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WallDress {
    pub outside: Outside,
    /// The way the outside face looks, on plan: a unit vector in feet east and north.
    pub out: Option<[f64; 2]>,
    /// How tall it stands, in feet.
    pub height: f64,
    pub cladding: Option<Said<Cladding>>,
    /// A cladding on the lowest part of the wall, how high it goes in feet, and whether
    /// that height was written in the line (false: a wainscot with no height written is
    /// drawn 3 ft high and says so).
    pub wainscot: Option<(Said<Cladding>, f64, bool)>,
    /// How far past each end of its centre line the wall is drawn so that it closes
    /// with the wall it corners with.
    pub past: [f64; 2],
}

impl WallDress {
    /// Whether anything is drawn on a face of it: a partition takes no cladding.
    pub fn dressed(&self) -> bool {
        self.outside != Outside::Neither
    }
    /// Whether what is priced is drawn on both faces.
    pub fn both(&self) -> bool {
        matches!(self.outside, Outside::Both | Outside::NotKnown)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpeningDress {
    pub sash: Option<Said<Sash>>,
    pub leaf: Option<Said<Leaf>>,
    pub divided: Option<Lites>,
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

// ---- Reading a line ----

/// The words of a text, lower case, split wherever a letter or a digit stops.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

/// Where the words first hold this phrase, word for word.
fn at(words: &[String], phrase: &str) -> Option<usize> {
    let want: Vec<&str> = phrase.split(' ').collect();
    words.windows(want.len()).position(|w| w.iter().zip(&want).all(|(a, b)| a == b))
}

fn any(words: &[String], phrases: &[&str]) -> bool {
    phrases.iter().any(|p| at(words, p).is_some())
}

/// A line's parts between its commas, each cut where it turns to something else ("with
/// brick mold casing", "over felt", "at windows"): the first is its head.
fn parts(text: &str) -> Vec<Vec<String>> {
    const TURNS: [&str; 14] = ["with", "over", "on", "at", "for", "from", "behind", "under", "below", "above", "against", "to", "w", "into"];
    text.split([',', ';', '(']).map(|part| words(part).into_iter().take_while(|w| !TURNS.contains(&w.as_str())).collect::<Vec<String>>()).filter(|w: &Vec<String>| !w.is_empty()).collect()
}

/// The one of `table` a line names (see the top of this file): by its head, or, when its
/// head only names the kind of thing, by what follows. Among several in the same part
/// the one named first wins, and of two at the same word the one earlier in the table.
fn named<T: Copy>(text: &str, table: &[(T, &[&str])], kind: &[&str], not: &[&str]) -> Option<T> {
    let parts = parts(text);
    let head = parts.first()?;
    if any(head, not) {
        return None;
    }
    let first_in = |part: &Vec<String>| table.iter().filter_map(|(what, phrases)| phrases.iter().filter_map(|p| at(part, p)).min().map(|i| (i, *what))).min_by_key(|(i, _)| *i).map(|(_, what)| what);
    if let Some(what) = first_in(head) {
        return Some(what);
    }
    if any(head, kind) {
        return parts[1..].iter().find_map(first_in);
    }
    None
}

/// Whether a line is about the thing itself: its head names it or its kind, and holds no
/// word for something else.
fn about<T: Copy>(text: &str, table: &[(T, &[&str])], kind: &[&str], not: &[&str]) -> bool {
    parts(text).first().is_some_and(|head| !any(head, not) && (any(head, kind) || table.iter().any(|(_, p)| any(head, p))))
}

const CLADDINGS: &[(Cladding, &[&str])] = &[
    (Cladding::BoardAndBatten, &["board and batten", "board batten", "batten siding"]),
    (Cladding::VerticalSiding, &["t1 11", "t111", "vertical siding", "vertical board", "vertical boards", "channel siding"]),
    (Cladding::MetalPanel, &["corrugated metal", "metal siding", "metal panel", "metal panels", "corrugated"]),
    (Cladding::Panel, &["panel siding", "fiber cement panel", "fiber cement panels", "cement panel", "cement panels", "hardie panel", "composite panel", "composite panels"]),
    (Cladding::Brick, &["brick", "bricks"]),
    (Cladding::Block, &["cmu", "concrete block", "split face", "block"]),
    (Cladding::Stone, &["stone", "ledgestone", "fieldstone", "limestone", "ashlar", "granite"]),
    (Cladding::Stucco, &["stucco", "eifs"]),
    (Cladding::Shingles, &["shingle", "shingles", "shake", "shakes"]),
    (Cladding::LapSiding, &["lap siding", "lap", "clapboard", "clapboards", "bevel", "hardie plank", "hardieplank", "plank siding"]),
];
const CLADDING_KIND: &[&str] = &["siding", "cladding", "veneer", "exterior finish"];
const NOT_CLADDING: &[&str] = &[
    "sill", "sills", "flashing", "nails", "nail", "fasteners", "screws", "removal", "remove", "demolition", "demo", "tear", "haul", "molding", "moulding", "mold", "mould", "casing", "trim", "paint",
    "painting", "primer", "caulk", "caulking", "sealant", "sealer", "countertop", "countertops", "backsplash", "surround", "interior", "ties", "tie", "tape", "apron", "ledge", "lintel", "hearth",
    "mantel", "backer", "flooring", "floor", "paver", "pavers", "baseboard", "furring", "wrap", "repair", "patch", "cleaning", "wash", "coping", "accessories", "allowance",
];

const ROOFINGS: &[(Roofing, &[&str])] = &[
    (Roofing::StandingSeam, &["standing seam", "seam panels", "copper", "zinc", "metal roofing", "metal roof"]),
    (Roofing::MetalPanel, &["5v", "crimp", "corrugated", "metal panels", "metal panel", "r panel", "ribbed"]),
    (Roofing::MetalShingles, &["metal shingles", "metal shingle", "stone coated"]),
    (Roofing::Slate, &["slate"]),
    (Roofing::Tile, &["tile", "tiles", "barrel"]),
    (Roofing::Shakes, &["shake", "shakes", "cedar shingle", "cedar shingles", "wood shingle", "wood shingles"]),
    (Roofing::Membrane, &["membrane", "epdm", "tpo", "built up", "bitumen", "torch down", "torchdown", "rubber roof", "rubber roofing", "rolled roofing", "roll roofing"]),
    (Roofing::AsphaltShingles, &["asphalt", "shingle", "shingles", "composition", "architectural", "dimensional", "three tab", "3 tab"]),
];
const ROOFING_KIND: &[&str] = &["roofing", "roof", "roof covering"];
const NOT_ROOFING: &[&str] = &[
    "underlayment", "felt", "flashing", "gutter", "gutters", "drip", "strip", "tear", "tearoff", "removal", "remove", "vent", "vents", "nails", "sheathing", "decking", "deck", "insulation", "ice",
    "edge", "repair", "patch", "coating", "sealant", "snow", "guard", "guards", "cricket", "skylight", "boot", "boots", "framing", "trusses", "rafters", "allowance",
];

const SASHES: &[(Sash, &[&str])] = &[
    (Sash::DoubleHung, &["double hung", "single hung", "hung"]),
    (Sash::Casement, &["casement", "casements"]),
    (Sash::Slider, &["slider", "sliding", "gliding", "glider"]),
    (Sash::Awning, &["awning"]),
    (Sash::Hopper, &["hopper"]),
    (Sash::Fixed, &["fixed", "picture", "direct set"]),
];
const WINDOW_KIND: &[&str] = &["window", "windows", "unit", "sash"];
const NOT_WINDOW: &[&str] = &[
    "flashing", "tape", "trim", "casing", "screen", "screens", "treatment", "treatments", "blind", "blinds", "shade", "shades", "sealant", "hardware", "sill", "stool", "apron", "film", "well",
    "removal", "remove", "shutter", "shutters", "curtain", "curtains", "drapes", "paint", "caulk", "allowance",
];

const LEAVES: &[(Leaf, &[&str])] = &[
    (Leaf::Overhead, &["overhead door", "garage door", "overhead", "sectional", "roll up"]),
    (Leaf::Glazed, &["french", "glazed", "glass", "full lite", "full light", "half lite", "half light", "full view"]),
    (Leaf::Panel, &["panel", "panels", "paneled", "panelled"]),
    (Leaf::Flush, &["flush", "slab"]),
];
const DOOR_KIND: &[&str] = &["door", "doors", "unit"];
const NOT_DOOR: &[&str] = &[
    "hardware", "knob", "lockset", "lock", "deadbolt", "hinge", "hinges", "casing", "trim", "threshold", "opener", "stop", "sweep", "weatherstrip", "weatherstripping", "closer", "removal", "remove",
    "bell", "mat", "flashing", "sealant", "paint", "allowance",
];

const GUTTERS: &[((), &[&str])] = &[((), &["gutter", "gutters", "eavestrough", "eavestroughs"])];
const NOT_GUTTER: &[&str] = &["apron", "flashing", "remove", "removal", "guard", "guards", "cleaning", "screen", "screens", "repair"];

const TONES: &[(&str, [u8; 3])] = &[
    // A species named with a color in it is the species: red cedar is cedar.
    ("western red cedar", [0xB4, 0x86, 0x5C]),
    ("red cedar", [0xB4, 0x86, 0x5C]),
    ("white cedar", [0xC2, 0xA8, 0x84]),
    ("white oak", [0xB8, 0x94, 0x64]),
    ("red oak", [0xB0, 0x82, 0x5C]),
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

/// The color a text names, if it names one: the one named first, a longer name before a
/// shorter one that starts at the same word.
pub fn tone_in(text: &str) -> Option<Tone> {
    let w = words(text);
    TONES.iter().enumerate().filter_map(|(k, (word, rgb))| at(&w, word).map(|i| (i, k, Tone { word, rgb: *rgb }))).min_by_key(|(i, k, _)| (*i, *k)).map(|(_, _, t)| t)
}

/// The cladding a line names, if it is a line for one.
pub fn cladding_in(text: &str) -> Option<Cladding> {
    named(text, CLADDINGS, CLADDING_KIND, NOT_CLADDING).or_else(|| {
        // "Siding" and nothing more particular is lap siding, which is what the word means alone.
        parts(text).first().filter(|head| !any(head, NOT_CLADDING) && any(head, &["siding"])).map(|_| Cladding::LapSiding)
    })
}

/// The roofing a line names, if it is a line for one.
pub fn roofing_in(text: &str) -> Option<Roofing> {
    named(text, ROOFINGS, ROOFING_KIND, NOT_ROOFING)
}

/// How a window operates, if a line for the window says.
pub fn sash_in(text: &str) -> Option<Sash> {
    named(text, SASHES, WINDOW_KIND, NOT_WINDOW)
}

/// How a door's leaf is built, if a line for the door says.
pub fn leaf_in(text: &str) -> Option<Leaf> {
    named(text, LEAVES, DOOR_KIND, NOT_DOOR)
}

/// Whether a text says the lites are divided, and how many when it gives a count: "6
/// over 1", "6/6", "9 lite".
pub fn divided_in(text: &str) -> Option<Lites> {
    let w = words(text);
    let count = |x: Option<&String>| x.and_then(|x| x.parse::<u32>().ok()).filter(|n| (1..=16).contains(n));
    if let Some(i) = w.iter().position(|x| x == "over") {
        if let (Some(a), Some(b)) = (i.checked_sub(1).and_then(|j| count(w.get(j))), count(w.get(i + 1))) {
            return Some(Lites { upper: Some(a), lower: Some(b) });
        }
    }
    // "6/6", written tight, on a line that says the window is hung.
    if any(&w, &["hung"]) {
        let lower = text.to_lowercase();
        let bytes = lower.as_bytes();
        for (i, c) in bytes.iter().enumerate() {
            if *c == b'/' {
                let before: String = lower[..i].chars().rev().take_while(char::is_ascii_digit).collect::<String>().chars().rev().collect();
                let after: String = lower[i + 1..].chars().take_while(char::is_ascii_digit).collect();
                if let (Ok(a), Ok(b)) = (before.parse::<u32>(), after.parse::<u32>()) {
                    if (1..=16).contains(&a) && (1..=16).contains(&b) {
                        return Some(Lites { upper: Some(a), lower: Some(b) });
                    }
                }
            }
        }
    }
    // "9 lite": that many in each sash. ("3/4 lite" is a door's glass, not a count.)
    if let Some(i) = w.iter().position(|x| matches!(x.as_str(), "lite" | "lites" | "light" | "lights")) {
        let fraction = i >= 2 && w[i - 2].parse::<u32>().is_ok();
        if let Some(n) = i.checked_sub(1).and_then(|j| count(w.get(j))).filter(|n| *n >= 2 && !fraction) {
            return Some(Lites { upper: Some(n), lower: Some(n) });
        }
    }
    any(&w, &["divided", "muntin", "muntins", "grille", "grilles", "colonial", "sdl", "tdl"]).then_some(Lites { upper: None, lower: None })
}

/// A wainscot's height written in a line, in feet: the first length in it that is not
/// a thickness ("Stone wainscot 36 in", "to 3'-0\"", "1 1/2 in thick, 42 in high").
fn height_in(text: &str) -> Option<f64> {
    for part in text.split([',', ';']) {
        let Some(start) = part.find(|c: char| c.is_ascii_digit()) else { continue };
        let mut said = part[start..].trim().to_lowercase();
        if said.ends_with("thick") || said.contains(" thick") || said.contains("exposure") || said.contains(" o.c") || said.contains(" oc") {
            continue;
        }
        for tail in [" high", " tall", " height", " ht", " a.f.f", " aff"] {
            if let Some(cut) = said.strip_suffix(tail) {
                said = cut.trim().to_string();
            }
        }
        if let Some(h) = crate::units::parse_length(&said).ok().and_then(|(len, unit)| len.feet(unit).ok()).filter(|h| *h > 0.0) {
            return Some(h);
        }
    }
    None
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

/// The first of what is named, with every later one that differs (in what it is or in
/// its color) kept as also priced.
fn first_and_also<T: Copy + PartialEq>(named: impl Iterator<Item = (T, Option<Tone>, String)>, said: impl Fn(T) -> &'static str) -> Option<Said<T>> {
    let mut out: Option<Said<T>> = None;
    for (what, tone, by) in named {
        match &mut out {
            None => out = Some(Said { what, tone, by, also: Vec::new() }),
            Some(first) if first.what != what || (tone.is_some() && tone != first.tone) => {
                first.also.push((format!("{}{}", said(what), tone.map(|t| format!(", {}", t.word)).unwrap_or_default()), by));
            }
            Some(_) => {}
        }
    }
    out
}

/// Read every finish in the job.
pub fn dress(job: &mut Job) -> Dress {
    let mut out = Dress::default();
    let (placed, _) = building::placements(job);
    // Each wall's centre line, thickness, height and level.
    struct W {
        path: String,
        storey: Option<String>,
        a: [f64; 2],
        b: [f64; 2],
        t: f64,
        h: f64,
    }
    let walls: Vec<W> = placed
        .iter()
        .filter_map(|p| match &p.shape {
            Shape::Wall { a, b, thickness, height, .. } => Some(W { path: p.path.clone(), storey: p.storey.clone(), a: *a, b: *b, t: *thickness, h: *height }),
            _ => None,
        })
        .collect();
    // What each level's walls enclose: every piece, and the courtyards in them.
    let mut enclosed: HashMap<String, Option<building::Enclosed>> = HashMap::new();
    for w in &walls {
        if let Some(s) = &w.storey {
            if !enclosed.contains_key(s) {
                let found = building::enclosed(job, s).ok().filter(|e| !e.pieces.is_empty());
                enclosed.insert(s.clone(), found);
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
        let outside = match w.storey.as_ref().and_then(|s| enclosed.get(s)).and_then(|r| r.as_ref()) {
            None => Outside::NotKnown,
            Some(e) => {
                let l = e.indoors([mid[0] + left[0] * reach, mid[1] + left[1] * reach]);
                let r = e.indoors([mid[0] - left[0] * reach, mid[1] - left[1] * reach]);
                match (l, r) {
                    (false, true) => Outside::Left,
                    (true, false) => Outside::Right,
                    (true, true) => Outside::Neither,
                    (false, false) => Outside::Both,
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
        let named: Vec<(Cladding, Option<Tone>, String, bool, Option<f64>)> =
            on.iter().filter_map(|(text, by)| cladding_in(text).map(|what| (what, tone_in(text), by.clone(), at(&words(text), "wainscot").is_some(), height_in(text)))).collect();
        let cladding = first_and_also(named.iter().filter(|n| !n.3).map(|n| (n.0, n.1, n.2.clone())), Cladding::said);
        let low = first_and_also(named.iter().filter(|n| n.3).map(|n| (n.0, n.1, n.2.clone())), Cladding::said);
        let wainscot = low.map(|said| {
            let h = named.iter().find(|n| n.3 && n.2 == said.by).and_then(|n| n.4);
            (said, h.unwrap_or(3.0), h.is_some())
        });
        out.walls.insert(w.path.clone(), WallDress { outside, out: out_way, height: w.h, cladding, wainscot, past });
    }
    // Roof planes, openings and gutters.
    for &e in &job.order.clone() {
        let (Some(node), Some(kind)) = (job.world.get::<Node>(e), job.world.get::<Kind>(e).copied()) else { continue };
        let path = node.path.clone();
        let on = texts(job, e);
        match kind {
            k if k.is_area() => {
                if let Some(said) = first_and_also(on.iter().filter_map(|(text, by)| roofing_in(text).map(|r| (r, tone_in(text), by.clone()))), Roofing::said) {
                    out.roofs.insert(path, said);
                }
            }
            Kind::Window => {
                // Only a line about the window itself is read: not its flashing, its trim or its screen.
                let unit: Vec<&(String, String)> = on.iter().filter(|(text, _)| about(text, SASHES, WINDOW_KIND, NOT_WINDOW)).collect();
                let sash = first_and_also(unit.iter().filter_map(|(text, by)| sash_in(text).map(|s| (s, tone_in(text), by.clone()))), Sash::said);
                // Its color is the one its own line gives, or failing that the first line about the unit.
                let tone = sash.as_ref().map(|s| s.tone).unwrap_or_else(|| unit.iter().find_map(|(text, _)| tone_in(text)));
                let divided = unit.iter().find_map(|(text, _)| divided_in(text));
                out.openings.insert(path, OpeningDress { sash, leaf: None, divided, tone });
            }
            Kind::Door => {
                let unit: Vec<&(String, String)> = on.iter().filter(|(text, _)| about(text, LEAVES, DOOR_KIND, NOT_DOOR)).collect();
                let leaf = first_and_also(unit.iter().filter_map(|(text, by)| leaf_in(text).map(|l| (l, tone_in(text), by.clone()))), Leaf::said);
                let tone = leaf.as_ref().map(|l| l.tone).unwrap_or_else(|| unit.iter().find_map(|(text, _)| tone_in(text)));
                let divided = unit.iter().find_map(|(text, _)| divided_in(text));
                out.openings.insert(path, OpeningDress { sash: None, leaf, divided, tone });
            }
            Kind::Eave => {
                if let Some((text, by)) = on.iter().find(|(text, _)| named(text, GUTTERS, &[], NOT_GUTTER).is_some()) {
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

fn toned(t: &Option<Tone>) -> String {
    t.map(|t| format!(", {}", t.word)).unwrap_or_default()
}

fn also(others: &[(String, String)]) -> String {
    others.iter().map(|(what, by)| format!("; also priced on it and not drawn: {what} ({by})")).collect()
}

/// What a wall's reading says, for the report and for the panel: its first sentence and
/// what follows it.
fn wall_said(w: &WallDress) -> (String, Vec<String>) {
    let ft = |h: f64| crate::units::feet_as_ft_in(h, 16);
    let face = match (w.outside, w.out) {
        (_, Some(way)) => format!("on its {} face", compass(way)),
        (Outside::Neither, _) => "a partition: both its faces are inside".to_string(),
        (Outside::Both, _) => "on both faces: it stands outside what its level's walls enclose".to_string(),
        _ => "on both faces: which is outside is not known (its level's walls enclose nothing)".to_string(),
    };
    let mut more = Vec::new();
    if !w.dressed() {
        // A partition: whatever cladding is priced on it is priced and not drawn, and said.
        let mut priced: Vec<String> = Vec::new();
        if let Some(c) = &w.cladding {
            priced.push(format!("{} ({})", c.what.said(), c.by));
            priced.extend(c.also.iter().map(|(what, by)| format!("{what} ({by})")));
        }
        if let Some((s, _, _)) = &w.wainscot {
            priced.push(format!("{} wainscot ({})", s.what.said(), s.by));
            priced.extend(s.also.iter().map(|(what, by)| format!("{what} wainscot ({by})")));
        }
        return if priced.is_empty() { (face, more) } else { (format!("{face}, so no cladding is drawn on it; priced on it and not drawn: {}", priced.join("; ")), more) };
    }
    // A wainscot as tall as the wall, or taller, is the whole face.
    let whole = w.wainscot.as_ref().is_some_and(|(_, to, _)| *to >= w.height - 1e-6);
    let first = match &w.cladding {
        Some(c) if whole => format!("{}{} is priced on it ({}) and not drawn: the wainscot covers the wall{}", c.what.said(), toned(&c.tone), c.by, also(&c.also)),
        Some(c) => format!("{}{}, {face} ({}){}", c.what.said(), toned(&c.tone), c.by, also(&c.also)),
        None if w.wainscot.is_some() && whole => face.clone(),
        None if w.wainscot.is_some() => format!("bare above its wainscot, {face}: nothing priced on it names a cladding"),
        None => format!("bare, {face}: nothing priced on it names a cladding"),
    };
    if let Some((s, to, written)) = &w.wainscot {
        let how_high = match (whole, written) {
            (true, _) => format!("to {}, which is the whole of this {} wall, {face}", ft(*to), ft(w.height)),
            (false, true) => format!("to {}", ft(*to)),
            (false, false) => format!("to {}, a height no line gives in feet and inches", ft(*to)),
        };
        more.push(format!("{}{} wainscot {how_high} ({}){}", s.what.said(), toned(&s.tone), s.by, also(&s.also)));
    }
    (first, more)
}

fn window_said(o: Option<&OpeningDress>) -> String {
    let divided = o.and_then(|o| o.divided).map(|l| format!(", {}", l.said())).unwrap_or_default();
    let tone = toned(&o.and_then(|o| o.tone));
    match o.and_then(|o| o.sash.as_ref()) {
        Some(s) => format!("{}{divided}{tone} ({}){}", s.what.said(), s.by, also(&s.also)),
        None => format!("a frame, one sash and one light{divided}{tone}: nothing priced on it says how it operates"),
    }
}

fn door_said(o: Option<&OpeningDress>) -> String {
    let tone = toned(&o.and_then(|o| o.tone));
    match o.and_then(|o| o.leaf.as_ref()) {
        Some(l) => format!("{}{tone} ({}){}", l.what.said(), l.by, also(&l.also)),
        None => format!("a frame and a plain leaf{tone}: nothing priced on it says how it is built"),
    }
}

/// The reading written out, one line for each wall, roof plane, window and door, and
/// for each eave with a gutter priced on it, in the job's order: what it is finished
/// with and what said so, or that it is bare.
pub fn report(job: &mut Job) -> Vec<String> {
    let d = dress(job);
    let mut out = Vec::new();
    for &e in &job.order.clone() {
        let (Some(node), Some(kind)) = (job.world.get::<Node>(e), job.world.get::<Kind>(e).copied()) else { continue };
        let name = format!("{} ({})", node.name, node.path);
        match kind {
            Kind::Wall => match d.walls.get(&node.path) {
                None => out.push(format!("{name}: not drawn: it cannot be placed")),
                Some(w) => {
                    let (first, more) = wall_said(w);
                    out.push(std::iter::once(format!("{name}: {first}")).chain(more).collect::<Vec<_>>().join("; "));
                }
            },
            k if k.is_area() => match d.roofs.get(&node.path) {
                Some(r) => out.push(format!("{name}: {}{} ({}){}", r.what.said(), toned(&r.tone), r.by, also(&r.also))),
                None => out.push(format!("{name}: bare: nothing priced on it names a roofing")),
            },
            Kind::Window => out.push(format!("{name}: {}", window_said(d.openings.get(&node.path)))),
            Kind::Door => out.push(format!("{name}: {}", door_said(d.openings.get(&node.path)))),
            Kind::Eave => {
                if let Some(g) = d.gutters.get(&node.path) {
                    out.push(format!("{name}: a gutter{} ({})", toned(&g.tone), g.by));
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
    let cap = |s: &str| {
        let mut c = s.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    };
    // A sentence of the report, cut into what and why at its first "(" or ":".
    let cut = |s: &str| -> (String, String) {
        match s.find(" (").into_iter().chain(s.find(": ")).min() {
            Some(i) => (cap(s[..i].trim()), cap(s[i..].trim_start_matches([' ', '(', ':']).trim_end_matches(')').trim())),
            None => (cap(s), String::new()),
        }
    };
    let mut out = Vec::new();
    match kind {
        Kind::Wall => {
            let Some(w) = d.walls.get(path) else { return out };
            if !w.dressed() && w.cladding.is_none() && w.wainscot.is_none() {
                return out;
            }
            let (first, more) = wall_said(w);
            out.push(cut(&first));
            out.extend(more.iter().map(|m| cut(m)));
        }
        k if k.is_area() => match d.roofs.get(path) {
            Some(r) => {
                out.push((cap(&format!("{}{}", r.what.said(), toned(&r.tone))), format!("From {}", r.by)));
                out.extend(r.also.iter().map(|(what, by)| (format!("Also priced, not drawn: {what}"), format!("From {by}"))));
            }
            None => out.push(("Bare".to_string(), "Nothing priced on it names a roofing".to_string())),
        },
        Kind::Window => out.push(cut(&window_said(d.openings.get(path)))),
        Kind::Door => out.push(cut(&door_said(d.openings.get(path)))),
        Kind::Eave => {
            if let Some(g) = d.gutters.get(path) {
                out.push((format!("Gutter{}", toned(&g.tone)), format!("From {}", g.by)));
            }
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The seventeenth judgment's lines, each as a veteran wrote it (J17B-3 to 10).
    #[test]
    fn a_line_names_a_finish_only_when_the_finish_is_what_the_line_is() {
        // Not claddings: a sill, a flashing, nails, a removal, a molding, paint, a counter.
        for text in [
            "Stone sill at windows",
            "Brick ledge flashing",
            "Siding nails, 8d stainless",
            "Siding removal and haul off",
            "Shingle molding",
            "Paint, siding, white",
            "Granite countertop",
            "Kitchen backsplash, brick",
            "Fireplace surround, stacked stone, interior",
            "Interior wainscot, stone tile, 42 in",
            "Brick ties",
            "Wall framing, 2x6 at 16 in",
            "Gypsum board, hung and finished",
        ] {
            assert_eq!(cladding_in(text), None, "{text}");
        }
        // The head decides, not a word further on.
        for (text, want) in [
            ("Lap siding with brick mold casing", Cladding::LapSiding),
            ("Lap siding, Cobble Stone", Cladding::LapSiding),
            ("Lap siding over 1x3 rainscreen battens", Cladding::LapSiding),
            ("Shingle siding, white, on a brick base", Cladding::Shingles),
            ("T1-11 plywood siding", Cladding::VerticalSiding),
            ("Vertical board siding", Cladding::VerticalSiding),
            ("Corrugated metal siding", Cladding::MetalPanel),
            ("Hardie plank, 7 in exposure", Cladding::LapSiding),
            ("CMU, 8 in", Cladding::Block),
            ("Split face block", Cladding::Block),
            ("Fiber cement panel, smooth", Cladding::Panel),
            ("Western red cedar bevel siding", Cladding::LapSiding),
            ("Siding, fiber cement, lap, white", Cladding::LapSiding),
            ("Siding, cedar shingles", Cladding::Shingles),
            ("Veneer, brick, red", Cladding::Brick),
            ("Install brick veneer", Cladding::Brick),
            ("Stone veneer wainscot, 3 ft", Cladding::Stone),
            ("Siding, vinyl", Cladding::LapSiding),
        ] {
            assert_eq!(cladding_in(text), Some(want), "{text}");
        }
        // Not roofings: what goes under, beside or off a roof.
        for text in [
            "#30 asphalt felt underlayment",
            "Ice and water membrane, eaves 6 ft",
            "Copper gutter, 6 in half round",
            "Copper flashing at chimney",
            "Drip edge, metal roof edge",
            "Zinc strip at ridge",
            "Roof tile underlayment",
            "15 lb asphalt felt",
            "Shingle tear-off, 1 layer",
            "Roof sheathing",
        ] {
            assert_eq!(roofing_in(text), None, "{text}");
        }
        for (text, want) in [
            ("Architectural shingles, Slate", Roofing::AsphaltShingles),
            ("Architectural shingles, Weathered Wood", Roofing::AsphaltShingles),
            ("Metal shingles, stone coated", Roofing::MetalShingles),
            ("Torch-down roofing", Roofing::Membrane),
            ("Rubber roof", Roofing::Membrane),
            ("5V crimp, galvalume", Roofing::MetalPanel),
            ("Corrugated metal panels", Roofing::MetalPanel),
            ("Red cedar shakes", Roofing::Shakes),
            ("Roofing, standing seam, charcoal", Roofing::StandingSeam),
            ("Copper, 20 oz", Roofing::StandingSeam),
            ("Standing seam roofing over ice and water membrane", Roofing::StandingSeam),
        ] {
            assert_eq!(roofing_in(text), Some(want), "{text}");
        }
        // Windows and doors: the unit, not its tape, trim, screen, opener or knob.
        assert_eq!(sash_in("Awning window, 36x24"), Some(Sash::Awning));
        assert_eq!(sash_in("Hopper window, basement"), Some(Sash::Hopper));
        assert_eq!(sash_in("Casement, black, 30x48"), Some(Sash::Casement));
        assert_eq!(sash_in("Window flashing tape, black"), None);
        assert_eq!(sash_in("Bay window unit"), None);
        assert_eq!(leaf_in("Door from garage to house, 20 min fire rated, steel"), None);
        assert_eq!(leaf_in("Garage door opener"), None);
        assert_eq!(leaf_in("Door hardware, glass knob"), None);
        assert_eq!(leaf_in("Garage door, 16 ft, insulated"), Some(Leaf::Overhead));
        assert_eq!(leaf_in("Entry door, fiberglass, half lite"), Some(Leaf::Glazed));
        assert_eq!(leaf_in("Storm door, full view"), Some(Leaf::Glazed));
        assert_eq!(leaf_in("Interior door, six panel, prehung"), Some(Leaf::Panel));
        assert_eq!(Leaf::Overhead.said(), "an overhead door");
    }

    #[test]
    fn colors_lites_and_wainscot_heights_are_read_as_written() {
        assert_eq!(tone_in("Western red cedar bevel siding").map(|t| t.word), Some("western red cedar"));
        assert_eq!(tone_in("Red cedar shakes").map(|t| t.word), Some("red cedar"));
        assert_eq!(tone_in("Brick veneer, red").map(|t| t.word), Some("red"));
        assert_eq!(tone_in("Standing seam roofing"), None);
        assert_eq!(divided_in("Window, double-hung, 6 over 1"), Some(Lites { upper: Some(6), lower: Some(1) }));
        assert_eq!(divided_in("Window, double hung, 6/6"), Some(Lites { upper: Some(6), lower: Some(6) }));
        assert_eq!(divided_in("Window, 9 lite casement"), Some(Lites { upper: Some(9), lower: Some(9) }));
        assert_eq!(divided_in("Entry door, 3/4 lite"), None);
        assert_eq!(divided_in("divided lites"), Some(Lites { upper: None, lower: None }));
        assert_eq!(divided_in("double-hung"), None);
        for (text, want) in [
            ("Stone veneer wainscot, 3 ft", Some(3.0)),
            ("Stone veneer wainscot 36 in", Some(3.0)),
            ("Stone veneer wainscot to 3'-0\"", Some(3.0)),
            ("Stone wainscot, 42 in high", Some(3.5)),
            ("Stone wainscot, 1 1/2 in thick, 42 in high", Some(3.5)),
            ("Stone wainscot, 0 in", None),
            ("Stone wainscot, 1.5 m", None),
            ("Stone wainscot", None),
        ] {
            assert_eq!(height_in(text), want, "{text}");
        }
    }
}
