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
    /// Part of its length bounds what its level's walls enclose and part runs on
    /// outside it: it is one wall where two are built, and is dressed on both faces
    /// until it is split where it leaves the house.
    Mixed,
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
    /// Its framing, where a line priced on it names one ("Wall framing, 2x6 at 16 in"):
    /// how deep its lumber is through the wall and how far apart its studs are, in
    /// feet. None where nothing priced on it says how it is framed: it is not framed
    /// on a guess.
    pub framing: Option<(f64, f64)>,
    /// How many lines priced on it name a framing. More than one: the first is the one
    /// drawn, and the report says the others are there.
    pub framing_lines: usize,
    /// How many lines on it name framing and do not say a lumber size and a spacing: it
    /// is not drawn as a frame for them, and the report says so.
    pub framing_unread: usize,
    /// Whether a line priced on it names a weather barrier under its finish ("Sheathing
    /// and weather barrier", "house wrap", "building paper").
    pub barrier: bool,
    /// Whether a line priced on it says it is concrete and none names a cladding: a
    /// foundation wall as placed. It is drawn as concrete, not as a bare framed wall.
    pub concrete: bool,
}

impl WallDress {
    /// Whether anything is drawn on a face of it: a partition takes no cladding.
    pub fn dressed(&self) -> bool {
        self.outside != Outside::Neither
    }
    /// Whether what is priced is drawn on both faces.
    pub fn both(&self) -> bool {
        matches!(self.outside, Outside::Both | Outside::NotKnown | Outside::Mixed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpeningDress {
    pub sash: Option<Said<Sash>>,
    pub leaf: Option<Said<Leaf>>,
    pub divided: Option<Lites>,
    pub tone: Option<Tone>,
    /// Whether a line priced on it names its flashing ("Flashing and sealant").
    pub flashed: bool,
}

/// Every finish read from the job.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dress {
    pub walls: HashMap<String, WallDress>,
    pub roofs: HashMap<String, Said<Roofing>>,
    pub openings: HashMap<String, OpeningDress>,
    /// Eaves with a gutter priced on them, and the color named.
    pub gutters: HashMap<String, Said<()>>,
    /// Rooms with base priced on them ("Base and trim", "baseboard"): the level each
    /// is on, and its outline at the faces of its walls, counter-clockwise, in feet.
    /// The base runs along that outline.
    pub base: HashMap<String, Base>,
}

// ---- Reading a line ----

/// Whether a line priced on a room names the base at the foot of its walls: "Base and
/// trim", "baseboard", "wood base". Not a base cabinet, a gravel base or a base coat.
pub fn base_in(text: &str) -> bool {
    if not_in_the_work(text) {
        return false;
    }
    let w = words(text);
    let has = |list: &[&str]| w.iter().any(|x| list.contains(&x.as_str()));
    if has(&["baseboard", "baseboards"]) {
        return true;
    }
    // "Base" alone is too many things (a vanity base, a shower base, a gravel base, a
    // base coat): it is the base at the foot of a wall only when the line says so by
    // what goes with it, and never when it names one of the others.
    let with = ["trim", "molding", "moulding", "shoe", "wood", "mdf", "pine", "oak", "poplar", "rubber", "vinyl", "cove", "wall"];
    let other = ["cabinet", "cabinets", "vanity", "shower", "tub", "gravel", "coat", "course", "plate", "sheet", "column", "post"];
    has(&["base"]) && has(&with) && !has(&other)
}

/// Whether a line says the wall itself is concrete ("Foundation wall, 8 in concrete,
/// formed and placed", "Poured concrete wall"), and not something fixed to concrete
/// ("Anchor bolts into concrete").
pub fn concrete_wall_in(text: &str) -> bool {
    if not_in_the_work(text) {
        return false;
    }
    let w = words(text);
    let has = |list: &[&str]| w.iter().any(|x| list.contains(&x.as_str()));
    has(&["concrete"]) && has(&["wall", "walls", "foundation", "stem", "formed", "poured", "cast"]) && !has(&["anchor", "anchors", "bolt", "bolts", "into", "fastener", "fasteners", "nails", "screws"])
}

/// Whether a line names a weather barrier: the sheet that keeps water out of the wall
/// behind its finish. By the words the trade uses for it.
pub fn barrier_in(text: &str) -> bool {
    if not_in_the_work(text) {
        return false;
    }
    let w = words(text);
    // Its names, the makes a line is most often written by, and a sheathing that
    // carries its barrier on its face. (An air barrier is taken for one, as call 72
    // decided: on an outside wall's line it is the membrane behind the finish. The
    // twenty-first judgment would not: J21-8.) A vapor barrier and ice and water shield
    // are not.
    const NAMES: &[&str] = &["weather barrier", "weather resistive", "water resistive", "air barrier", "house wrap", "building paper", "felt paper", "tar paper", "asphalt felt", "housewrap", "wrb", "tyvek", "typar", "zip system", "zip sheathing"];
    NAMES.iter().any(|name| {
        let Some(found) = at(&w, name) else { return false };
        // The tape, staples or caps that go with a wrap are not the wrap ("Housewrap
        // tape at seams" drew the whole wall wrapped: J21-8).
        !w.get(found + name.split(' ').count()).is_some_and(|next| ["tape", "tapes", "staples", "caps", "fasteners"].contains(&next.as_str()))
    })
}

/// Whether a line says the thing is taken out, or is not in the work at all: "Remove
/// house wrap", "Demo existing concrete wall", "Tear off felt paper", "Wall framing, 2x6
/// at 16 in - NOT INCLUDED", "by others". A remodel's estimate is full of such lines,
/// and what they name is not drawn as if it were being put in (J21-7).
pub fn not_in_the_work(text: &str) -> bool {
    let w = words(text);
    const OUT: &[&str] = &["remove", "removal", "removing", "demo", "demolish", "demolition", "tear off", "tear out", "strip off", "strip out", "haul off", "haul away", "dispose", "disposal", "not included", "not in contract", "excluded", "exclude", "by others", "by owner", "nic", "omit", "omitted", "deduct", "credit for"];
    any(&w, OUT)
}

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
/// brick mold casing", "over felt", "at windows", "and flashing"): the first is its
/// head. What the work is done with comes first in a line as often as not ("Labor to
/// install lap siding", "Furnish and install ..."), and is not part of what it is.
fn parts(text: &str, not: &[&str]) -> Vec<Vec<String>> {
    const TURNS: [&str; 15] = ["with", "over", "on", "at", "for", "from", "behind", "under", "below", "above", "against", "to", "w", "into", "per"];
    const LEAD_INS: [&str; 9] = ["labor to install", "furnish and install", "supply and install", "installation of", "labor to", "labor for", "install", "installed", "labor"];
    let mut out = Vec::new();
    for (k, part) in text.split([',', ';', '(']).enumerate() {
        let mut w = words(part);
        if k == 0 {
            if let Some(lead) = LEAD_INS.iter().find(|l| at(&w, l) == Some(0)) {
                // The lead-in alone is a head of its own kind ("Labor, lap siding"): kept empty.
                w.drain(..lead.split(' ').count());
            }
        }
        // Cut at a turn, and at "and" when what follows it is something else ("and flashing").
        let cut = w.iter().enumerate().position(|(i, x)| TURNS.contains(&x.as_str()) || (x == "and" && w.get(i + 1).is_some_and(|next| not.contains(&next.as_str()))));
        w.truncate(cut.unwrap_or(w.len()));
        if !w.is_empty() || k == 0 {
            out.push(w);
        }
    }
    out
}

/// The one of `table` a line names (see the top of this file). A word for something
/// else anywhere in what the line is (not in what it is laid over or set with) and it
/// is not a finish. Its head names it; a head that names none is refined by what
/// follows its commas ("Roofing, standing seam", "North wall, lap siding"). Among
/// several in one part the one named first wins, and of two at the same word the one
/// earlier in the table.
fn named<T: Copy>(text: &str, table: &[(T, &[&str])], not: &[&str]) -> Option<T> {
    let parts = parts(text, not);
    if parts.iter().any(|part| any(part, not)) {
        return None;
    }
    let first_in = |part: &Vec<String>| table.iter().filter_map(|(what, phrases)| phrases.iter().filter_map(|p| at(part, p)).min().map(|i| (i, *what))).min_by_key(|(i, _)| *i).map(|(_, what)| what);
    parts.iter().find_map(first_in)
}

/// Whether a line is about the thing itself: its head names it or its kind, and nothing
/// in the line is a word for something else.
fn about<T: Copy>(text: &str, table: &[(T, &[&str])], kind: &[&str], not: &[&str]) -> bool {
    let parts = parts(text, not);
    !parts.iter().any(|part| any(part, not)) && parts.iter().any(|part| any(part, kind) || table.iter().any(|(_, p)| any(part, p)))
}

/// The words of a line that say what it is, less what it is laid over or set with: a
/// color is read from these ("Brick veneer with gray mortar" is not gray brick).
fn its_own(text: &str, not: &[&str]) -> String {
    parts(text, not).iter().map(|p| p.join(" ")).collect::<Vec<_>>().join(", ")
}

const CLADDINGS: &[(Cladding, &[&str])] = &[
    (Cladding::BoardAndBatten, &["board and batten", "board batten", "batten siding"]),
    (Cladding::VerticalSiding, &["t1 11", "t111", "vertical siding", "vertical board", "vertical boards", "channel siding"]),
    (Cladding::MetalPanel, &["corrugated metal", "metal siding", "metal panel", "metal panels", "corrugated"]),
    (Cladding::Panel, &["panel siding", "fiber cement panel", "fiber cement panels", "cement panel", "cement panels", "hardie panel", "composite panel", "composite panels"]),
    (Cladding::Brick, &["brick", "bricks"]),
    (Cladding::Block, &["cmu", "concrete block", "split face", "masonry block", "block veneer"]),
    (Cladding::Stone, &["stone", "stones", "ledgestone", "fieldstone", "limestone", "ashlar", "granite"]),
    (Cladding::Stucco, &["stucco", "eifs"]),
    (Cladding::Shingles, &["shingle", "shingles", "shake", "shakes"]),
    (Cladding::LapSiding, &["lap siding", "dutch lap", "clapboard", "clapboards", "bevel", "hardie plank", "hardieplank", "plank siding"]),
];
const NOT_CLADDING: &[&str] = &[
    "sill", "sills", "flashing", "nails", "nail", "fasteners", "screws", "removal", "remove", "demolition", "demo", "tear", "haul", "molding", "moulding", "mold", "mould", "casing", "trim", "paint",
    "painting", "primer", "caulk", "caulking", "sealant", "sealer", "countertop", "countertops", "backsplash", "surround", "interior", "ties", "tie", "tape", "apron", "ledge", "lintel", "hearth",
    "mantel", "backer", "flooring", "floor", "paver", "pavers", "baseboard", "furring", "wrap", "repair", "patch", "cleaning", "wash", "coping", "accessories", "allowance", "vanity", "counter", "top", "splices", "splice", "rebar", "mounting", "fire", "glass", "fireplace",
];

const ROOFINGS: &[(Roofing, &[&str])] = &[
    (Roofing::StandingSeam, &["standing seam", "seam panels", "copper", "zinc", "metal roofing", "metal roof"]),
    (Roofing::MetalPanel, &["5v", "crimp", "corrugated", "metal panels", "metal panel", "r panel", "ribbed", "galvalume"]),
    (Roofing::MetalShingles, &["metal shingles", "metal shingle", "stone coated"]),
    (Roofing::Slate, &["slate", "slates"]),
    (Roofing::Tile, &["tile", "tiles", "barrel"]),
    (Roofing::Shakes, &["shake", "shakes", "cedar shingle", "cedar shingles", "wood shingle", "wood shingles"]),
    (Roofing::Membrane, &["membrane", "epdm", "tpo", "built up", "bitumen", "torch down", "torchdown", "rubber roof", "rubber roofing", "rolled roofing", "roll roofing"]),
    (Roofing::AsphaltShingles, &["asphalt", "shingle", "shingles", "composition", "architectural", "dimensional", "three tab", "3 tab"]),
];
const NOT_ROOFING: &[&str] = &[
    "underlayment", "felt", "flashing", "gutter", "gutters", "drip", "strip", "tear", "tearoff", "removal", "remove", "vent", "vents", "nails", "sheathing", "decking", "deck", "insulation", "ice",
    "edge", "repair", "patch", "coating", "sealant", "snow", "guard", "guards", "cricket", "skylight", "boot", "boots", "framing", "trusses", "rafters", "allowance", "starter", "hip", "ridge", "cap", "caps", "batten", "battens",
];

const SASHES: &[(Sash, &[&str])] = &[
    (Sash::DoubleHung, &["double hung", "single hung", "hung"]),
    (Sash::Casement, &["casement", "casements"]),
    (Sash::Slider, &["slider", "sliding", "gliding", "glider"]),
    (Sash::Awning, &["awning"]),
    (Sash::Hopper, &["hopper"]),
    (Sash::Fixed, &["fixed", "picture window", "picture unit", "direct set"]),
];
const WINDOW_KIND: &[&str] = &["window", "windows", "unit", "sash"];
const NOT_WINDOW: &[&str] = &[
    "flashing", "tape", "trim", "casing", "screen", "screens", "treatment", "treatments", "blind", "blinds", "shade", "shades", "sealant", "hardware", "sill", "stool", "apron", "film", "well",
    "removal", "remove", "shutter", "shutters", "curtain", "curtains", "drapes", "paint", "caulk", "allowance", "molding", "moulding",
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
const NOT_GUTTER: &[&str] = &["apron", "flashing", "remove", "removal", "guard", "guards", "cleaning", "screen", "screens", "repair", "hanger", "hangers", "cap", "caps", "bracket", "brackets", "outlet", "outlets", "miter", "miters", "screws", "sealant"];

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
    // What a line is laid over or set with has a color too, and it is not the line's.
    let w = words(&its_own(text, &[]));
    TONES.iter().enumerate().filter_map(|(k, (word, rgb))| at(&w, word).map(|i| (i, k, Tone { word, rgb: *rgb }))).min_by_key(|(i, k, _)| (*i, *k)).map(|(_, _, t)| t)
}

/// The cladding a line names, if it is a line for one.
pub fn cladding_in(text: &str) -> Option<Cladding> {
    named(text, CLADDINGS, NOT_CLADDING).or_else(|| {
        // "Siding" and nothing more particular is lap siding, which is what the word means alone.
        let parts = parts(text, NOT_CLADDING);
        (!parts.iter().any(|p| any(p, NOT_CLADDING)) && parts.iter().any(|p| any(p, &["siding", "lap"]))).then_some(Cladding::LapSiding)
    })
}

/// The roofing a line names, if it is a line for one.
pub fn roofing_in(text: &str) -> Option<Roofing> {
    named(text, ROOFINGS, NOT_ROOFING)
}

/// How a window operates, if a line for the window says.
pub fn sash_in(text: &str) -> Option<Sash> {
    named(text, SASHES, NOT_WINDOW)
}

/// How a door's leaf is built, if a line for the door says.
pub fn leaf_in(text: &str) -> Option<Leaf> {
    named(text, LEAVES, NOT_DOOR)
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
/// The base in one room: the level it is on, the room's outline at the faces of its
/// walls (counter-clockwise, in feet), and the lengths of base along it, each from one
/// point on the plan to another, with nothing across a door. Worked here, so that what
/// is drawn and what is said of it are one figure (R21-26: the window worked the cuts
/// at doors out for itself).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Base {
    pub storey: Option<String>,
    pub outline: Vec<[f64; 2]>,
    pub runs: Vec<[[f64; 2]; 2]>,
}

impl Base {
    /// How much base is drawn, in LF.
    pub fn length(&self) -> f64 {
        self.runs.iter().map(|[a, b]| (b[0] - a[0]).hypot(b[1] - a[1])).sum()
    }
}

/// The base along a room's outline, left out across each door: `doors` are the doors of
/// the room's level, each as its two ends on the plan (on its wall's centre line) and how
/// far either side of that line its wall reaches. A door takes what lies between its
/// ends out of an edge of the outline that runs along its wall.
pub fn base_runs(outline: &[[f64; 2]], doors: &[([f64; 2], [f64; 2], f64)]) -> Vec<[[f64; 2]; 2]> {
    let mut out = Vec::new();
    for (i, e0) in outline.iter().enumerate() {
        let e1 = outline[(i + 1) % outline.len()];
        let len = (e1[0] - e0[0]).hypot(e1[1] - e0[1]);
        if len < 0.05 {
            continue;
        }
        let d = [(e1[0] - e0[0]) / len, (e1[1] - e0[1]) / len];
        let mut cuts: Vec<(f64, f64)> = doors
            .iter()
            .filter_map(|(p0, p1, reach)| {
                let off = |p: &[f64; 2]| ((p[0] - e0[0]) * d[1] - (p[1] - e0[1]) * d[0]).abs();
                let at = |p: &[f64; 2]| (p[0] - e0[0]) * d[0] + (p[1] - e0[1]) * d[1];
                (off(p0) <= *reach && off(p1) <= *reach).then(|| (at(p0).min(at(p1)), at(p0).max(at(p1))))
            })
            .filter(|(t0, t1)| *t1 > 0.0 && *t0 < len)
            .collect();
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut from = 0.0;
        let at = |t: f64| [e0[0] + d[0] * t, e0[1] + d[1] * t];
        for (t0, t1) in cuts {
            if t0 > from + 0.05 {
                out.push([at(from), at(t0)]);
            }
            from = from.max(t1);
        }
        if len > from + 0.05 {
            out.push([at(from), at(len)]);
        }
    }
    out
}

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
    // Walls only: a footing is placed as a wall is and is not one (a wall is not
    // carried past its end to meet the footing under it).
    let walls: Vec<W> = placed
        .iter()
        .filter(|p| p.kind == Kind::Wall)
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
        let reach = w.t / 2.0 + 0.25;
        let outside = match w.storey.as_ref().and_then(|s| enclosed.get(s)).and_then(|r| r.as_ref()) {
            None => Outside::NotKnown,
            Some(e) => {
                // Looked at in five places along it, not at its middle alone: a wall
                // that is the house's side for part of its length and a garden wall for
                // the rest is not either one (J17C-12).
                let at_each: Vec<Outside> = [0.1, 0.3, 0.5, 0.7, 0.9]
                    .iter()
                    .map(|f| {
                        let p = [w.a[0] + dx * f, w.a[1] + dy * f];
                        let l = e.indoors([p[0] + left[0] * reach, p[1] + left[1] * reach]);
                        let r = e.indoors([p[0] - left[0] * reach, p[1] - left[1] * reach]);
                        match (l, r) {
                            (false, true) => Outside::Left,
                            (true, false) => Outside::Right,
                            (true, true) => Outside::Neither,
                            (false, false) => Outside::Both,
                        }
                    })
                    .collect();
                if at_each.iter().all(|o| *o == at_each[0]) { at_each[0] } else { Outside::Mixed }
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
            on.iter().filter_map(|(text, by)| cladding_in(text).map(|what| (what, tone_in(text), by.clone(), at(&words(&its_own(text, NOT_CLADDING)), "wainscot").is_some(), height_in(text)))).collect();
        let cladding = first_and_also(named.iter().filter(|n| !n.3).map(|n| (n.0, n.1, n.2.clone())), Cladding::said);
        let low = first_and_also(named.iter().filter(|n| n.3).map(|n| (n.0, n.1, n.2.clone())), Cladding::said);
        let wainscot = low.map(|said| {
            let h = named.iter().find(|n| n.3 && n.2 == said.by).and_then(|n| n.4);
            (said, h.unwrap_or(3.0), h.is_some())
        });
        // A framing line that is taken out or not in the work frames nothing.
        let framed_by = |text: &str| if not_in_the_work(text) { None } else { crate::building::framing::named(text) };
        let framing = on.iter().find_map(|(text, _)| framed_by(text));
        let framing_lines = on.iter().filter(|(text, _)| framed_by(text).is_some()).count();
        // Lines that name framing and give no lumber size and spacing to draw it by:
        // not drawn, and said (J21-8: "2x4 framing" was passed over without a word).
        let framing_unread = on.iter().filter(|(text, _)| !not_in_the_work(text) && crate::building::framing::named(text).is_none() && words(text).iter().any(|x| x == "framing" || x == "studs")).count();
        let barrier = on.iter().any(|(text, _)| barrier_in(text));
        let concrete = cladding.is_none() && wainscot.is_none() && on.iter().any(|(text, _)| concrete_wall_in(text));
        out.walls.insert(w.path.clone(), WallDress { outside, out: out_way, height: w.h, cladding, wainscot, past, framing, framing_lines, framing_unread, barrier, concrete });
    }
    // Roof planes, openings, gutters, and the base in rooms.
    let outlines = building::room_outlines(job);
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
                let flashed = on.iter().any(|(text, _)| !not_in_the_work(text) && words(text).iter().any(|w| w == "flashing" || w == "flashed"));
                out.openings.insert(path, OpeningDress { sash, leaf: None, divided, tone, flashed });
            }
            Kind::Room => {
                if on.iter().any(|(text, _)| base_in(text)) {
                    if let Some(outline) = outlines.get(&path) {
                        let storey = job.world.get::<Node>(e).and_then(|n| n.parent.clone());
                        // The doors of the room's level: each one's two ends on the plan,
                        // and half its wall's thickness and a little, either side.
                        let doors: Vec<([f64; 2], [f64; 2], f64)> = placed
                            .iter()
                            .filter(|p| p.kind == Kind::Wall && p.storey == storey)
                            .flat_map(|p| match &p.shape {
                                building::Shape::Wall { a, b, thickness, holes, .. } => {
                                    let l = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-9);
                                    let u = [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
                                    holes.iter().filter(|h| h.1 == Kind::Door).map(|h| ([a[0] + u[0] * h.2[0], a[1] + u[1] * h.2[0]], [a[0] + u[0] * h.2[1], a[1] + u[1] * h.2[1]], thickness / 2.0 + 0.1)).collect::<Vec<_>>()
                                }
                                _ => Vec::new(),
                            })
                            .collect();
                        let runs = base_runs(outline, &doors);
                        out.base.insert(path, Base { storey, outline: outline.clone(), runs });
                    }
                }
            }
            Kind::Door => {
                let unit: Vec<&(String, String)> = on.iter().filter(|(text, _)| about(text, LEAVES, DOOR_KIND, NOT_DOOR)).collect();
                let leaf = first_and_also(unit.iter().filter_map(|(text, by)| leaf_in(text).map(|l| (l, tone_in(text), by.clone()))), Leaf::said);
                let tone = leaf.as_ref().map(|l| l.tone).unwrap_or_else(|| unit.iter().find_map(|(text, _)| tone_in(text)));
                let divided = unit.iter().find_map(|(text, _)| divided_in(text));
                let flashed = on.iter().any(|(text, _)| !not_in_the_work(text) && words(text).iter().any(|w| w == "flashing" || w == "flashed"));
                out.openings.insert(path, OpeningDress { sash: None, leaf, divided, tone, flashed });
            }
            Kind::Eave => {
                if let Some((text, by)) = on.iter().find(|(text, _)| named(text, GUTTERS, NOT_GUTTER).is_some()) {
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
        (Outside::Mixed, _) => "on both faces: part of it bounds what its level's walls enclose and part runs on outside; split it where it leaves the house and each part is dressed as it is".to_string(),
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
    if let (true, Some((s, to, _))) = (whole, &w.wainscot) {
        let taller = if *to > w.height + 1e-6 { format!("written to {}, taller than this {} wall, so it covers the wall", ft(*to), ft(w.height)) } else { format!("to {}, the whole of this wall", ft(*to)) };
        let first = format!("{}{} wainscot {taller}, {face} ({}){}", s.what.said(), toned(&s.tone), s.by, also_wainscot(&s.also));
        if let Some(c) = &w.cladding {
            more.push(format!("{}{} is priced on it ({}) and not drawn: the wainscot covers the wall{}", c.what.said(), toned(&c.tone), c.by, also(&c.also)));
        }
        return (first, more);
    }
    let first = match &w.cladding {
        Some(c) => format!("{}{}, {face} ({}){}", c.what.said(), toned(&c.tone), c.by, also(&c.also)),
        None if w.wainscot.is_some() => format!("bare above its wainscot, {face}: nothing priced on it names a cladding"),
        // A concrete wall is not bare: it is what it is made of (J21-9: "bare ...;
        // concrete, as a line priced on it says" in one sentence).
        None if w.concrete => format!("no cladding, {face}"),
        None => format!("bare, {face}: nothing priced on it names a cladding"),
    };
    if let Some((s, to, written)) = &w.wainscot {
        let how_high = if *written { format!("to {}", ft(*to)) } else { format!("drawn to {}, since its line gives no height to draw it to (one is written like 36 in or 3 ft)", ft(*to)) };
        more.push(format!("{}{} wainscot {how_high} ({}){}", s.what.said(), toned(&s.tone), s.by, also_wainscot(&s.also)));
    }
    (first, more)
}

/// A second wainscot priced on a wall is said to be a wainscot.
fn also_wainscot(others: &[(String, String)]) -> String {
    others.iter().map(|(what, by)| format!("; also priced on it and not drawn: {what} wainscot ({by})")).collect()
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

/// What else of a wall is drawn from what is priced on it, under or instead of its
/// finish: that it is concrete, its frame, lines that name a framing and are not drawn,
/// the weather barrier. Said in the report and on the wall's own panel (J21-20: a
/// framed wall's panel said "Bare" and nothing of its frame).
pub fn wall_under(w: &WallDress) -> Vec<String> {
    let mut under: Vec<String> = Vec::new();
    if w.concrete {
        under.push("concrete, as a line priced on it says".into());
    }
    if let Some((deep, spacing)) = w.framing {
        let lumber = [(3.5, "2x4"), (5.5, "2x6"), (7.25, "2x8"), (9.25, "2x10"), (11.25, "2x12")].iter().find(|(d, _)| (d / 12.0 - deep).abs() < 1.0e-6).map(|(_, n)| *n).unwrap_or("lumber");
        under.push(format!("framed in {lumber} at {} in on center", crate::view::num(spacing * 12.0)));
        // Two framings priced on one wall: one is drawn, and it is said
        // which (it was the first, without a word).
        if w.framing_lines > 1 {
            under.push(format!("{} lines on it name a framing: the first is the one drawn", w.framing_lines));
        }
    }
    if w.framing_unread > 0 {
        under.push(format!("{} on it {} framing and {} no lumber size and spacing (like 2x6 at 16 in): not drawn as a frame", crate::view::so_many(w.framing_unread, "line"), if w.framing_unread == 1 { "names" } else { "name" }, if w.framing_unread == 1 { "gives" } else { "give" }));
    }
    if w.barrier {
        under.push("a weather barrier under its finish".into());
    }
    under
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
                    // What else is drawn from what is priced on it: its frame, the
                    // weather barrier under its finish, that it is concrete (J20-7: the
                    // window drew these and this report said nothing of them).
                    let under = wall_under(w);
                    out.push(std::iter::once(format!("{name}: {first}")).chain(more).chain(under).collect::<Vec<_>>().join("; "));
                }
            },
            Kind::Room => {
                if let Some(base) = d.base.get(&node.path) {
                    // How much of it is drawn: the room's outline less its doors.
                    out.push(format!("{name}: base at the foot of its walls, {} LF of it, with none across a door", crate::view::figure(base.length(), 2)));
                }
            }
            k if k.is_area() => match d.roofs.get(&node.path) {
                Some(r) => out.push(format!("{name}: {}{} ({}){}", r.what.said(), toned(&r.tone), r.by, also(&r.also))),
                None => out.push(format!("{name}: bare: nothing priced on it names a roofing")),
            },
            Kind::Window => out.push(format!("{name}: {}{}", window_said(d.openings.get(&node.path)), if d.openings.get(&node.path).is_some_and(|o| o.flashed) { "; flashed" } else { "" })),
            Kind::Door => out.push(format!("{name}: {}{}", door_said(d.openings.get(&node.path)), if d.openings.get(&node.path).is_some_and(|o| o.flashed) { "; flashed" } else { "" })),
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
            let under = wall_under(w);
            if !w.dressed() && w.cladding.is_none() && w.wainscot.is_none() && under.is_empty() {
                return out;
            }
            let (first, more) = wall_said(w);
            out.push(cut(&first));
            out.extend(more.iter().map(|m| cut(m)));
            // What is under its finish, each as a line of its own.
            out.extend(under.iter().map(|u| (cap(u), String::new())));
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

    #[test]
    fn base_is_read_by_its_own_words() {
        for said in ["Base and trim", "Baseboard, 5 1/4 in MDF", "Wood base, stained", "Rubber cove base", "Base shoe molding"] {
            assert!(base_in(said), "{said}");
        }
        // The twentieth judgment's: a bathroom with these priced must not get baseboard.
        for said in ["Base cabinets", "Excavation and gravel base", "Stucco base coat", "Casing and trim", "Cabinets and countertops", "Vanity base, 36 in", "Shower base, 32 x 48", "Base"] {
            assert!(!base_in(said), "{said}");
        }
    }

    #[test]
    fn a_concrete_wall_is_one_the_line_says_is_concrete() {
        for said in ["Foundation wall, 8 in concrete, formed and placed", "Poured concrete wall", "Cast-in-place concrete stem wall"] {
            assert!(concrete_wall_in(said), "{said}");
        }
        for said in ["Anchor bolts into concrete", "Concrete nails", "Partition framing, 2x4 at 16 in", "Concrete slab on grade, 4 in"] {
            assert!(!concrete_wall_in(said), "{said}");
        }
    }

    /// A weather barrier is read by the words the trade uses for it, and from nothing
    /// else: a line about sheathing alone, or about flashing, names no barrier.
    #[test]
    fn a_weather_barrier_is_read_by_its_own_words() {
        for said in ["Sheathing and weather barrier", "House wrap, taped", "15 lb building paper", "Housewrap", "Air barrier membrane", "WRB, fluid applied", "Water-resistive barrier", "Tyvek HomeWrap", "Felt paper, 15 lb", "Typar"] {
            assert!(barrier_in(said), "{said}");
        }
        for said in ["Roof sheathing", "Flashing and sealant", "Batt insulation", "Cedar lap siding, installed", "Wrap the columns in cedar", "Vapor retarder", "Vapor barrier, 6 mil poly", "Roofing felt"] {
            assert!(!barrier_in(said), "{said}");
        }
        // The twenty-first judgment's (J21-8): three it missed, and the tape that is not the wrap.
        for said in ["Weather-resistive barrier", "Asphalt felt, 30 lb", "Zip System sheathing, taped", "Tyvek, taped seams"] {
            assert!(barrier_in(said), "{said}");
        }
        for said in ["Housewrap tape at seams", "Tyvek tape", "Moisture barrier", "Ice and water shield"] {
            assert!(!barrier_in(said), "{said}");
        }
    }

    /// What a line takes out, or says is not in the work, is not drawn as if it were
    /// being put in (J21-7): a remodel's estimate is full of such lines.
    #[test]
    fn what_is_taken_out_or_not_in_the_work_is_not_drawn() {
        for said in ["Remove house wrap", "Demo existing house wrap", "Tear off felt paper", "Remove existing concrete wall", "Wall framing, 2x6 at 16 in - NOT INCLUDED", "Wood base, by others", "Demolition of concrete foundation wall", "Haul away baseboard"] {
            assert!(not_in_the_work(said), "{said}");
            assert!(!barrier_in(said) && !concrete_wall_in(said) && !base_in(said), "{said}");
        }
        // Words that only look like them are left alone.
        for said in ["House wrap, taped", "Foundation wall, 8 in concrete, formed and poured", "Wood base, 3 1/4 in, primed", "Demountable partition", "Stripping and sealing"] {
            assert!(!not_in_the_work(said), "{said}");
        }
    }

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

    /// The judge of the fixes (J17C-9, 10, 13, 15, 16): the honest lines the head rule
    /// dropped, and the ones that are not a finish and still dressed.
    #[test]
    fn the_nearest_lines_to_the_first_fix_read_as_what_they_are() {
        for (text, want) in [
            ("North wall, lap siding", Some(Cladding::LapSiding)),
            ("Brick veneer and flashing", Some(Cladding::Brick)),
            ("LABOR, LAP SIDING", Some(Cladding::LapSiding)),
            ("Labor to install lap siding", Some(Cladding::LapSiding)),
            ("Furnish and install brick veneer", Some(Cladding::Brick)),
            ("Stones veneer", Some(Cladding::Stone)),
            ("Wainscot, stone, 36 in", Some(Cladding::Stone)),
            ("Lap siding above stone wainscot", Some(Cladding::LapSiding)),
            ("Fire block, 2x4", None),
            ("Siding mounting block", None),
            ("Glass block", None),
            ("Rebar lap splices", None),
            ("Granite vanity top", None),
            ("Stacked stone veneer, interior fireplace wall", None),
            ("Board and batten siding", Some(Cladding::BoardAndBatten)),
        ] {
            assert_eq!(cladding_in(text), want, "{text}");
        }
        for (text, want) in [
            ("Galvalume roofing", Some(Roofing::MetalPanel)),
            ("Slates, Vermont gray", Some(Roofing::Slate)),
            ("Labor, architectural shingles", Some(Roofing::AsphaltShingles)),
            ("Reroof, asphalt shingles", Some(Roofing::AsphaltShingles)),
            ("Asphalt shingles and felt", Some(Roofing::AsphaltShingles)),
            ("Starter shingles", None),
            ("Hip and ridge shingles", None),
            ("Tile battens", None),
            ("Copper ridge cap", None),
        ] {
            assert_eq!(roofing_in(text), want, "{text}");
        }
        assert_eq!(sash_in("Labor, casement window"), Some(Sash::Casement));
        assert_eq!(sash_in("Window and flashing, casement"), Some(Sash::Casement));
        assert_eq!(sash_in("Picture frame molding"), None);
        assert_eq!(sash_in("Picture window, 6 ft"), Some(Sash::Fixed));
        // A color is the line's own, not that of what it is laid over or set with.
        assert_eq!(tone_in("Brick veneer with gray mortar"), None);
        assert_eq!(tone_in("Lap siding over black building paper"), None);
        assert_eq!(tone_in("Standing seam roofing over black underlayment"), None);
        assert_eq!(tone_in("Casement windows w/ white interior, black exterior").map(|t| t.word), Some("black"));
        assert_eq!(tone_in("Brick veneer, red").map(|t| t.word), Some("red"));
        // A wainscot is a line that is one, not a line laid above one.
        assert!(at(&words(&its_own("Stone veneer wainscot, 3 ft", NOT_CLADDING)), "wainscot").is_some());
        assert!(at(&words(&its_own("Lap siding above stone wainscot", NOT_CLADDING)), "wainscot").is_none());
        for (text, gutter) in [("Gutter hangers", false), ("Gutter end caps", false), ("Seamless aluminum eavestrough", true), ("Copper gutter, 6 in half round", true)] {
            assert_eq!(named(text, GUTTERS, NOT_GUTTER).is_some(), gutter, "{text}");
        }
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
    /// The base runs along a room's outline and is left out across a door (worked here
    /// and drawn from this: R21-26). A room 10 ft by 10 ft inside, a 3 ft door in its
    /// south wall from 4 ft to 7 ft: 4 + 3 on the south edge and three whole edges,
    /// 37 LF, in five lengths. A door in a wall that is not the room's takes nothing.
    #[test]
    fn base_is_left_out_across_a_door() {
        let room = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        // The door's ends on its wall's centre line, 3 in outside the room's face.
        let south_door = ([4.0, -0.25], [7.0, -0.25], 0.35);
        let runs = base_runs(&room, &[south_door]);
        assert_eq!(runs.len(), 5, "{runs:?}");
        let base = Base { storey: None, outline: room.to_vec(), runs };
        assert!((base.length() - 37.0).abs() < 1.0e-9, "{}", base.length());
        // No door: the whole outline, 40 LF.
        assert!((Base { storey: None, outline: room.to_vec(), runs: base_runs(&room, &[]) }.length() - 40.0).abs() < 1.0e-9);
        // A door in another room's wall, 6 ft away: nothing is taken out.
        let far = ([4.0, -6.0], [7.0, -6.0], 0.35);
        assert_eq!(base_runs(&room, &[far]).len(), 4);
    }
}
