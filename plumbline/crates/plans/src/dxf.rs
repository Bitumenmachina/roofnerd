//! A CAD drawing (DXF), read as a sheet to trace over and written as the plan of a level (call 221).
//!
//! Written here from Autodesk's published DXF reference
//! (https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/index.htm): the file is a list of
//! pairs, a group code and its value, in ASCII (a line each) or in binary (the sentinel
//! "AutoCAD Binary DXF", then each code and value in bytes). No code is linked or copied: a reader
//! that dropped what it could not model without a name would break the rule that nothing is lost in
//! silence, so what the file holds and what is not carried is counted here, by kind.
//!
//! What is carried: LINE, LWPOLYLINE, POLYLINE (flat), ARC, CIRCLE, ELLIPSE, SPLINE as straight lines (a
//! curve is kept as straight pieces of at most 10 degrees, said with how many), TEXT, MTEXT and ATTRIB
//! as words, INSERT with its BLOCK (scaled, turned, in rows and columns), and the figure of a
//! DIMENSION. Layers are the file's own; what the file switches off is switched off. Everything is held
//! in the drawing's own unit, from the lower-left corner of what is drawn.

use std::collections::{BTreeMap, HashMap};

use crate::{Page, Picture, Sort, Text, LONG_SIDE, MOST_LINES};

/// The longest turn of one straight piece of a curve, in degrees.
pub const PIECE_DEGREES: f64 = 10.0;
/// The deepest a block may hold blocks.
const DEEPEST: usize = 16;
/// The most straight lines and words one insert of blocks may make: a file that repeats a block
/// without end is stopped here and says so.
const MOST_MADE: usize = 4_000_000;

// ---- What a file is, by how it begins ----

const BINARY: &[u8] = b"AutoCAD Binary DXF\r\n\x1a\0";

/// Whether these bytes are a DXF, in either form.
pub fn is_dxf(bytes: &[u8]) -> bool {
    if bytes.starts_with(BINARY) {
        return true;
    }
    // ASCII: pairs of lines. The first group is 0 "SECTION", after any comments (group 999).
    let text: &[u8] = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let head = &text[..text.len().min(4096)];
    let Ok(head) = std::str::from_utf8(head).or_else(|e| std::str::from_utf8(&head[..e.valid_up_to()])) else { return false };
    let mut lines = head.lines().map(str::trim);
    loop {
        let Some(code) = lines.next() else { return false };
        let Some(value) = lines.next() else { return false };
        match code {
            "999" => continue,
            "0" => return value.eq_ignore_ascii_case("SECTION"),
            _ => return false,
        }
    }
}

/// A closed format that is not a DXF: what it is, by how it begins or by its name, and what to do (the
/// words a person is told). None for a DXF and for anything this does not know.
///
/// DWG: the first six bytes are "AC10" and two digits (Library of Congress, fdd000445). SketchUp:
/// FF FE FF 0E and "SketchUp Model" in two-byte letters (PRONOM fmt/1265). Revit: a compound file
/// (D0 CF 11 E0 A1 B1 1A E1) holding a stream "BasicFileInfo" in two-byte letters (PRONOM fmt/1347).
pub fn closed_format(bytes: &[u8], name: &str) -> Option<String> {
    let low = name.to_ascii_lowercase();
    let ends = |e: &str| low.ends_with(e);
    let wide = |s: &str| -> Vec<u8> { s.bytes().flat_map(|c| [c, 0]).collect() };
    let has = |needle: &[u8]| bytes.windows(needle.len()).take(bytes.len().min(4_000_000)).any(|w| w == needle);
    if is_dxf(bytes) {
        return None;
    }
    let dwg = bytes.len() >= 6 && bytes.starts_with(b"AC10") && bytes[4].is_ascii_digit() && bytes[5].is_ascii_digit();
    if dwg || (ends(".dwg") && !bytes.starts_with(b"%PDF")) {
        let version = if dwg {
            let code = String::from_utf8_lossy(&bytes[..6]).to_string();
            let year = match code.as_str() {
                "AC1012" => "AutoCAD R13",
                "AC1014" => "AutoCAD R14",
                "AC1015" => "AutoCAD 2000",
                "AC1018" => "AutoCAD 2004",
                "AC1021" => "AutoCAD 2007",
                "AC1024" => "AutoCAD 2010",
                "AC1027" => "AutoCAD 2013",
                "AC1032" => "AutoCAD 2018",
                _ => "",
            };
            if year.is_empty() { format!(", version {code}") } else { format!(", {year}") }
        } else {
            String::new()
        };
        return Some(format!("{name} is an AutoCAD drawing (DWG{version}). DWG is a closed format this program cannot read. Open it in the CAD program and save it as a DXF (Save As, DXF), then bring that in"));
    }
    let ole = bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    if (ole && has(&wide("BasicFileInfo"))) || (ole && (ends(".rvt") || ends(".rfa") || ends(".rte"))) || ends(".rvt") || ends(".rfa") {
        return Some(format!("{name} is a Revit model. Revit's own file is closed and this program cannot read it. Export it from Revit as an IFC model and bring that in as a model, or export the plan as a DXF to trace over"));
    }
    if bytes.starts_with(&[0xFF, 0xFE, 0xFF]) && has(&wide("SketchUp")) || ends(".skp") {
        return Some(format!("{name} is a SketchUp model. SketchUp's own file is closed and this program cannot read it. Export it from SketchUp as a DXF, or as a model (IFC), and bring that in"));
    }
    None
}

// ---- Units ----

/// What a drawing says it is drawn in ($INSUNITS) and how long one of that unit is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Units {
    pub code: i64,
    pub name: &'static str,
    pub inches: f64,
}

/// The unit a $INSUNITS code names (Autodesk's table), when it is one a building is drawn in.
pub fn units_of(code: i64) -> Option<Units> {
    let (name, inches) = match code {
        1 => ("inches", 1.0),
        2 => ("feet", 12.0),
        3 => ("miles", 63_360.0),
        4 => ("millimeters", 1.0 / 25.4),
        5 => ("centimeters", 10.0 / 25.4),
        6 => ("meters", 1000.0 / 25.4),
        7 => ("kilometers", 1.0e6 / 25.4),
        8 => ("microinches", 1.0e-6),
        9 => ("mils", 0.001),
        10 => ("yards", 36.0),
        14 => ("decimeters", 100.0 / 25.4),
        15 => ("decameters", 10_000.0 / 25.4),
        16 => ("hectometers", 100_000.0 / 25.4),
        21 => ("US survey feet", 12.000024),
        22 => ("US survey inches", 1.000002),
        23 => ("US survey yards", 36.000072),
        24 => ("US survey miles", 63_360.12672),
        _ => return None,
    };
    Some(Units { code, name, inches })
}

// ---- The pairs ----

#[derive(Clone, Debug)]
struct Tag {
    code: i32,
    val: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    Str,
    Dbl,
    I16,
    I32,
    I64,
    Bool,
    Chunk,
}

/// How a group's value is held in the binary form, by its code (Autodesk's table of ranges).
fn ty_of(code: i32) -> Ty {
    match code {
        10..=59 | 110..=149 | 210..=239 | 460..=469 | 1010..=1059 => Ty::Dbl,
        60..=79 | 170..=179 | 270..=289 | 370..=389 | 400..=409 | 1060..=1070 => Ty::I16,
        90..=99 | 420..=429 | 440..=449 | 450..=459 | 1071 => Ty::I32,
        160..=169 => Ty::I64,
        290..=299 => Ty::Bool,
        310..=319 => Ty::Chunk,
        _ => Ty::Str,
    }
}

/// Text as a file holds it: UTF-8 where it is, else one byte a letter (the older code page).
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_string(),
        Err(_) => bytes.iter().map(|&b| char::from(b)).collect(),
    }
}

fn tags(bytes: &[u8]) -> Result<Vec<Tag>, String> {
    let damaged = |at: String| format!("the drawing could not be read: it is damaged or cut short {at}. Save it again from the CAD program as a DXF");
    let mut out = Vec::new();
    if bytes.starts_with(BINARY) {
        let mut at = BINARY.len();
        let take = |at: &mut usize, n: usize| -> Option<&[u8]> {
            let s = bytes.get(*at..*at + n)?;
            *at += n;
            Some(s)
        };
        while at < bytes.len() {
            let code = i32::from(u16::from_le_bytes(take(&mut at, 2).ok_or_else(|| damaged("in its pairs".into()))?.try_into().unwrap_or([0, 0])));
            let cut = || damaged(format!("at group {code}"));
            let val = match ty_of(code) {
                Ty::Str => {
                    let end = bytes[at..].iter().position(|&b| b == 0).ok_or_else(cut)?;
                    let s = decode(&bytes[at..at + end]);
                    at += end + 1;
                    s
                }
                Ty::Dbl => f64::from_le_bytes(take(&mut at, 8).ok_or_else(cut)?.try_into().unwrap_or([0; 8])).to_string(),
                Ty::I16 => i16::from_le_bytes(take(&mut at, 2).ok_or_else(cut)?.try_into().unwrap_or([0; 2])).to_string(),
                Ty::I32 => i32::from_le_bytes(take(&mut at, 4).ok_or_else(cut)?.try_into().unwrap_or([0; 4])).to_string(),
                Ty::I64 => i64::from_le_bytes(take(&mut at, 8).ok_or_else(cut)?.try_into().unwrap_or([0; 8])).to_string(),
                Ty::Bool => take(&mut at, 1).ok_or_else(cut)?[0].to_string(),
                Ty::Chunk => {
                    let n = usize::from(take(&mut at, 1).ok_or_else(cut)?[0]);
                    take(&mut at, n).ok_or_else(cut)?;
                    String::new()
                }
            };
            let end = code == 0 && val == "EOF";
            out.push(Tag { code, val });
            if end {
                break;
            }
        }
        if out.iter().find(|t| t.code != 999).is_none_or(|t| t.code != 0) {
            return Err("this binary DXF is of a kind older than AutoCAD R14, which this program does not read. Save it again as a DXF in a newer version, or as the text form".into());
        }
        return Ok(out);
    }
    let text = decode(bytes);
    let mut lines = text.split('\n').map(|l| l.trim_end_matches('\r'));
    let mut n = 0usize;
    while let Some(c) = lines.next() {
        n += 1;
        if c.trim().is_empty() && out.is_empty() {
            continue;
        }
        let Ok(code) = c.trim().parse::<i32>() else {
            if c.trim().is_empty() {
                continue;
            }
            return Err(damaged(format!("near line {n} (a group code was wanted)")));
        };
        let Some(v) = lines.next() else { return Err(damaged(format!("after line {n}"))) };
        n += 1;
        let val = if ty_of(code) == Ty::Str { v.to_string() } else { v.trim().to_string() };
        let end = code == 0 && val.trim() == "EOF";
        out.push(Tag { code, val });
        if end {
            break;
        }
    }
    Ok(out)
}

// ---- Entities ----

#[derive(Clone, Debug, Default)]
struct Ent {
    kind: String,
    t: Vec<(i32, String)>,
    /// A POLYLINE's vertices.
    sub: Vec<Ent>,
}

impl Ent {
    fn s(&self, c: i32) -> Option<&str> {
        self.t.iter().find(|(k, _)| *k == c).map(|(_, v)| v.as_str())
    }
    fn f(&self, c: i32) -> Option<f64> {
        self.s(c).and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite())
    }
    fn fd(&self, c: i32, d: f64) -> f64 {
        self.f(c).unwrap_or(d)
    }
    fn i(&self, c: i32) -> Option<i64> {
        self.s(c).and_then(|v| v.trim().parse::<f64>().ok()).map(|v| v as i64)
    }
    fn all(&self, c: i32) -> Vec<f64> {
        self.t.iter().filter(|(k, _)| *k == c).filter_map(|(_, v)| v.trim().parse::<f64>().ok()).collect()
    }
    fn layer(&self) -> &str {
        self.s(8).map(str::trim).filter(|l| !l.is_empty()).unwrap_or("0")
    }
}

struct Block {
    base: [f64; 2],
    ents: Vec<Ent>,
}

/// Put an entity on a list; a vertex joins its polyline, and the end of one's vertices closes it.
fn push(list: &mut Vec<Ent>, open: &mut bool, e: Ent) {
    match e.kind.as_str() {
        "VERTEX" if *open => {
            if let Some(p) = list.last_mut() {
                p.sub.push(e);
            }
        }
        "SEQEND" => *open = false,
        "POLYLINE" => {
            *open = true;
            list.push(e);
        }
        _ => {
            *open = false;
            list.push(e);
        }
    }
}

// ---- Turning and placing ----

#[derive(Clone, Copy, Debug)]
struct M {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl M {
    const ONE: M = M { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };
    fn at(&self, p: [f64; 2]) -> [f64; 2] {
        [self.a * p[0] + self.c * p[1] + self.e, self.b * p[0] + self.d * p[1] + self.f]
    }
    /// This, then `outer`.
    fn then(&self, outer: &M) -> M {
        M {
            a: outer.a * self.a + outer.c * self.b,
            b: outer.b * self.a + outer.d * self.b,
            c: outer.a * self.c + outer.c * self.d,
            d: outer.b * self.c + outer.d * self.d,
            e: outer.a * self.e + outer.c * self.f + outer.e,
            f: outer.b * self.e + outer.d * self.f + outer.f,
        }
    }
    /// How much a length along x is scaled, and the way x points (radians).
    fn stretch(&self) -> (f64, f64) {
        (self.a.hypot(self.b), self.b.atan2(self.a))
    }
}

/// What a layer holds, for the list a person switches.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub name: String,
    /// Switched off or frozen in the file itself.
    pub off: bool,
    /// Straight lines drawn on it (a polyline's every side counts), and curves (a circle, an arc, an
    /// ellipse, a spline, a rounded polyline side), words, and the straight pieces the curves are kept as.
    pub straight: usize,
    pub curves: usize,
    pub pieces: usize,
    pub texts: usize,
    /// What the drawing holds on this layer, by kind, in this order: lines, polylines, arcs, circles, ellipses, splines.
    pub held: Vec<(&'static str, usize)>,
    linetype: String,
}

/// One straight piece of the drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seg {
    pub a: [f64; 2],
    pub b: [f64; 2],
    pub layer: usize,
    pub dashed: bool,
    /// A piece of a curve, not a line of its own.
    pub piece: bool,
}

/// A line of words on the drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub text: String,
    pub at: [f64; 2],
    pub height: f64,
    /// Radians counter-clockwise from east.
    pub angle: f64,
    pub layer: usize,
}

/// A DXF, read.
#[derive(Clone, Debug)]
pub struct Cad {
    /// The unit the file says it is drawn in; none when it does not say (or says one no building is drawn in).
    pub units: Option<Units>,
    /// Why there is no unit, in words, when there is none.
    pub no_units: String,
    pub layers: Vec<Layer>,
    pub segs: Vec<Seg>,
    pub notes: Vec<Note>,
    /// What is drawn spans this much, in the file's unit, from `origin` (the lower-left corner, in the file's own place).
    pub size: [f64; 2],
    pub origin: [f64; 2],
    /// What the file holds that is not brought in, each kind with its count, in words.
    pub left: BTreeMap<String, usize>,
    /// Other things said of what was read (a curve kept as pieces, lines seen from above), in words.
    pub notes_said: Vec<String>,
    /// Each block that was drawn, and how many times it was set.
    pub blocks: Vec<(String, usize)>,
    pub dimensions: usize,
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What a kind of thing is called in a drawing, one and many, for the words that say it was left out.
fn called(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "HATCH" => ("hatch (a filled or patterned area)", "hatches (filled or patterned areas)"),
        "IMAGE" => ("picture placed in the drawing", "pictures placed in the drawing"),
        "3DSOLID" | "BODY" | "REGION" => ("3D solid", "3D solids"),
        "SURFACE" | "MESH" => ("3D surface", "3D surfaces"),
        "SOLID" | "TRACE" => ("filled shape", "filled shapes"),
        "3DFACE" => ("3D face", "3D faces"),
        "POINT" => ("point", "points"),
        "LEADER" | "MLEADER" => ("leader", "leaders"),
        "MLINE" => ("multiline", "multilines"),
        "TOLERANCE" => ("tolerance mark", "tolerance marks"),
        "WIPEOUT" => ("blank-out shape", "blank-out shapes"),
        "XLINE" | "RAY" => ("line without an end", "lines without an end"),
        "OLEFRAME" | "OLE2FRAME" => ("object from another program", "objects from another program"),
        "UNDERLAY" => ("placed PDF or sheet", "placed PDFs or sheets"),
        "ATTDEF" => ("block text field", "block text fields"),
        "TABLE" => ("table", "tables"),
        "VIEWPORT" => ("sheet view", "sheet views"),
        "LIGHT" | "SUN" => ("light", "lights"),
        "SHAPE" => ("shape from a shape file", "shapes from a shape file"),
        "HELIX" => ("helix", "helices"),
        _ => ("thing of a kind this program does not know", "things of kinds this program does not know"),
    }
}

struct Build {
    layers: Vec<Layer>,
    index: HashMap<String, usize>,
    segs: Vec<Seg>,
    notes: Vec<Note>,
    left: BTreeMap<String, usize>,
    blocks: BTreeMap<String, usize>,
    paper: usize,
    tilted: usize,
    not_level: usize,
    nothing_long: usize,
    over: usize,
    cut_short: usize,
    missing: BTreeMap<String, usize>,
    dimensions: usize,
    dimensions_bare: usize,
    ellipse_odd: usize,
    made: usize,
    inches_per_unit: Option<f64>,
}

impl Build {
    fn layer(&mut self, name: &str) -> usize {
        if let Some(&i) = self.index.get(name) {
            return i;
        }
        let i = self.layers.len();
        self.layers.push(Layer { name: name.to_string(), off: false, straight: 0, curves: 0, pieces: 0, texts: 0, held: Vec::new(), linetype: String::new() });
        self.index.insert(name.to_string(), i);
        i
    }
    fn hold(&mut self, layer: usize, kind: &'static str) {
        let held = &mut self.layers[layer].held;
        match held.iter_mut().find(|(k, _)| *k == kind) {
            Some(e) => e.1 += 1,
            None => held.push((kind, 1)),
        }
    }
    fn not(&mut self, kind: &str) {
        *self.left.entry(kind.to_string()).or_default() += 1;
    }
    fn seg(&mut self, layer: usize, a: [f64; 2], b: [f64; 2], dashed: bool, piece: bool) {
        if ![a[0], a[1], b[0], b[1]].iter().all(|v| v.is_finite()) {
            return;
        }
        if (b[0] - a[0]).hypot(b[1] - a[1]) < 1.0e-12 {
            self.nothing_long += 1;
            return;
        }
        if self.segs.len() >= MOST_LINES {
            self.over += 1;
            return;
        }
        self.segs.push(Seg { a, b, layer, dashed, piece });
        if piece {
            self.layers[layer].pieces += 1;
        } else {
            self.layers[layer].straight += 1;
        }
    }
}

fn dashed_name(name: &str) -> bool {
    let n = name.to_ascii_uppercase();
    ["DASH", "HIDDEN", "CENTER", "PHANTOM", "DOT", "DIVIDE"].iter().any(|w| n.contains(w))
}

/// Whether an extrusion direction is (0, 0, 1), or (0, 0, -1) which mirrors x, or neither.
fn flip_of(e: &Ent) -> Option<bool> {
    let z = e.fd(230, 1.0);
    let (x, y) = (e.fd(210, 0.0), e.fd(220, 0.0));
    if x.abs() > 1.0e-9 || y.abs() > 1.0e-9 {
        return None;
    }
    if z > 0.0 { Some(false) } else { Some(true) }
}

/// Points round an arc from `a0` to `a1` (radians, counter-clockwise, `a1` > `a0`), the ends included.
fn arc_points(c: [f64; 2], r: f64, a0: f64, a1: f64) -> Vec<[f64; 2]> {
    let sweep = a1 - a0;
    let n = ((sweep / PIECE_DEGREES.to_radians()).ceil() as usize).max(1);
    (0..=n)
        .map(|i| {
            let a = a0 + sweep * (i as f64) / (n as f64);
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

/// The points of the side from `p` to `q` that has a bulge `b` (the tangent of a quarter of the turn,
/// negative clockwise), the ends included.
fn bulge_points(p: [f64; 2], q: [f64; 2], b: f64) -> Vec<[f64; 2]> {
    let chord = (q[0] - p[0]).hypot(q[1] - p[1]);
    if b.abs() < 1.0e-12 || chord < 1.0e-12 {
        return vec![p, q];
    }
    let turn = 4.0 * b.atan();
    let s = b * chord / 2.0;
    let r = chord * (1.0 + b * b) / (4.0 * b);
    let mid = [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0];
    let left = [-(q[1] - p[1]) / chord, (q[0] - p[0]) / chord];
    let c = [mid[0] + left[0] * (r - s), mid[1] + left[1] * (r - s)];
    let a0 = (p[1] - c[1]).atan2(p[0] - c[0]);
    let n = ((turn.abs() / PIECE_DEGREES.to_radians()).ceil() as usize).max(1);
    let mut out: Vec<[f64; 2]> = (0..=n)
        .map(|i| {
            let a = a0 + turn * (i as f64) / (n as f64);
            [c[0] + r.abs() * a.cos(), c[1] + r.abs() * a.sin()]
        })
        .collect();
    // The ends are the ends: no rounding off them.
    out[0] = p;
    out[n] = q;
    out
}

/// A B-spline's point at `t`, by Cox and de Boor, with weights.
fn spline_at(degree: usize, knots: &[f64], ctrl: &[[f64; 2]], weights: &[f64], t: f64) -> [f64; 2] {
    fn basis(i: usize, p: usize, t: f64, k: &[f64], last: usize) -> f64 {
        if p == 0 {
            let inside = (k[i] <= t && t < k[i + 1]) || (t >= k[last] && i + 1 == last && k[i] < k[i + 1]);
            return if inside { 1.0 } else { 0.0 };
        }
        let mut v = 0.0;
        let d1 = k[i + p] - k[i];
        if d1 > 0.0 {
            v += (t - k[i]) / d1 * basis(i, p - 1, t, k, last);
        }
        let d2 = k[i + p + 1] - k[i + 1];
        if d2 > 0.0 {
            v += (k[i + p + 1] - t) / d2 * basis(i + 1, p - 1, t, k, last);
        }
        v
    }
    let last = knots.len() - degree - 1;
    let (mut x, mut y, mut w) = (0.0, 0.0, 0.0);
    for (i, c) in ctrl.iter().enumerate() {
        let n = basis(i, degree, t, knots, last) * weights[i];
        x += n * c[0];
        y += n * c[1];
        w += n;
    }
    if w.abs() < 1.0e-12 { ctrl[0] } else { [x / w, y / w] }
}

/// Words of a drawing with its codes taken out: `\P` is a break (said as a space), `{\fArial;..}` and the like are
/// formatting, `\U+00B0` is a letter, `%%d` a degree sign, `%%p` plus or minus, `%%c` a diameter sign.
pub fn plain_words(raw: &str, formatted: bool) -> String {
    let mut out = String::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '%' && chars.get(i + 1) == Some(&'%') {
            match chars.get(i + 2).map(|c| c.to_ascii_lowercase()) {
                Some('d') => out.push('\u{b0}'),
                Some('p') => out.push('\u{b1}'),
                Some('c') => out.push('\u{2300}'),
                Some('%') => out.push('%'),
                _ => {}
            }
            i += 3;
            continue;
        }
        if c == '\\' {
            let n = chars.get(i + 1).copied();
            match n {
                Some('U') if chars.get(i + 2) == Some(&'+') => {
                    let hex: String = chars.get(i + 3..).unwrap_or(&[]).iter().take(4).collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        out.push(ch);
                    }
                    i += 7;
                    continue;
                }
                Some('P') | Some('X') if formatted => {
                    if !out.ends_with(' ') && !out.is_empty() {
                        out.push(' ');
                    }
                    i += 2;
                    continue;
                }
                Some('\\') | Some('{') | Some('}') if formatted => {
                    out.push(n.unwrap_or(' '));
                    i += 2;
                    continue;
                }
                Some('~') if formatted => {
                    out.push(' ');
                    i += 2;
                    continue;
                }
                Some('L') | Some('l') | Some('O') | Some('o') | Some('K') | Some('k') if formatted => {
                    i += 2;
                    continue;
                }
                // \f, \H, \C, \Q, \W, \A, \T, \p, \S: a code up to its semicolon.
                Some('f') | Some('F') | Some('H') | Some('C') | Some('c') | Some('Q') | Some('W') | Some('A') | Some('T') | Some('p') if formatted => {
                    while i < chars.len() && chars[i] != ';' {
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
                // A stack: \S1/2; is read as 1/2.
                Some('S') if formatted => {
                    i += 2;
                    while i < chars.len() && chars[i] != ';' {
                        out.push(if chars[i] == '^' || chars[i] == '#' { '/' } else { chars[i] });
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }
        if formatted && (c == '{' || c == '}') {
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out.trim().to_string()
}

struct Walk<'a> {
    b: &'a mut Build,
    blocks: &'a HashMap<String, Block>,
}

impl Walk<'_> {
    fn go(&mut self, ents: &[Ent], m: &M, parent: Option<(usize, bool)>, depth: usize) {
        for e in ents {
            if self.b.made > MOST_MADE {
                self.b.cut_short += 1;
                return;
            }
            self.one(e, m, parent, depth);
        }
    }

    /// The layer an entity is on: a block's own layer "0" is the layer of the insert that sets it.
    fn layer_of(&mut self, e: &Ent, parent: Option<(usize, bool)>) -> usize {
        let name = e.layer();
        match parent {
            Some((p, _)) if name == "0" => p,
            _ => self.b.layer(name),
        }
    }

    fn dashed(&self, e: &Ent, layer: usize, parent: Option<(usize, bool)>) -> bool {
        match e.s(6).map(str::trim) {
            Some(l) if l.eq_ignore_ascii_case("BYBLOCK") => parent.is_some_and(|p| p.1),
            Some(l) if !l.is_empty() && !l.eq_ignore_ascii_case("BYLAYER") => dashed_name(l),
            _ => dashed_name(&self.b.layers[layer].linetype),
        }
    }

    fn put_points(&mut self, pts: &[[f64; 2]], layer: usize, dashed: bool, piece: bool, m: &M) {
        for w in pts.windows(2) {
            self.b.made += 1;
            self.b.seg(layer, m.at(w[0]), m.at(w[1]), dashed, piece);
        }
    }

    fn one(&mut self, e: &Ent, m: &M, parent: Option<(usize, bool)>, depth: usize) {
        if e.i(67) == Some(1) {
            self.b.paper += 1;
            return;
        }
        let kind = e.kind.as_str();
        match kind {
            "LINE" => {
                let layer = self.layer_of(e, parent);
                let dashed = self.dashed(e, layer, parent);
                let (a, b) = ([e.fd(10, 0.0), e.fd(20, 0.0)], [e.fd(11, 0.0), e.fd(21, 0.0)]);
                if (e.fd(30, 0.0) - e.fd(31, 0.0)).abs() > 1.0e-9 {
                    self.b.not_level += 1;
                }
                self.b.hold(layer, "line");
                self.put_points(&[a, b], layer, dashed, false, m);
            }
            "LWPOLYLINE" => {
                let layer = self.layer_of(e, parent);
                let dashed = self.dashed(e, layer, parent);
                let Some(flip) = flip_of(e) else {
                    self.b.tilted += 1;
                    return;
                };
                // Bulges belong to the vertex before them: read in order, one per vertex when present.
                let mut verts: Vec<([f64; 2], f64)> = Vec::new();
                for (k, v) in &e.t {
                    match k {
                        10 => verts.push(([v.trim().parse().unwrap_or(0.0), 0.0], 0.0)),
                        20 => {
                            if let Some(l) = verts.last_mut() {
                                l.0[1] = v.trim().parse().unwrap_or(0.0);
                            }
                        }
                        42 => {
                            if let Some(l) = verts.last_mut() {
                                l.1 = v.trim().parse().unwrap_or(0.0);
                            }
                        }
                        _ => {}
                    }
                }
                let closed = e.i(70).unwrap_or(0) & 1 == 1;
                self.polyline(&verts, closed, flip, layer, dashed, m);
            }
            "POLYLINE" => {
                let layer = self.layer_of(e, parent);
                let dashed = self.dashed(e, layer, parent);
                let flags = e.i(70).unwrap_or(0);
                if flags & 16 != 0 || flags & 64 != 0 {
                    self.b.not("3DFACE");
                    return;
                }
                let Some(flip) = flip_of(e) else {
                    self.b.tilted += 1;
                    return;
                };
                let mut verts: Vec<([f64; 2], f64)> = Vec::new();
                let mut level = true;
                let z0 = e.sub.first().map(|v| v.fd(30, 0.0));
                for v in &e.sub {
                    let vf = v.i(70).unwrap_or(0);
                    // Spline frame points are the control cage, not the curve: the curve's own points follow.
                    if vf & 16 != 0 || vf & 128 != 0 {
                        continue;
                    }
                    if z0.is_some_and(|z| (v.fd(30, 0.0) - z).abs() > 1.0e-9) {
                        level = false;
                    }
                    verts.push(([v.fd(10, 0.0), v.fd(20, 0.0)], v.fd(42, 0.0)));
                }
                if !level {
                    self.b.not_level += 1;
                }
                self.polyline(&verts, flags & 1 == 1, flip, layer, dashed, m);
            }
            "CIRCLE" | "ARC" => {
                let layer = self.layer_of(e, parent);
                let dashed = self.dashed(e, layer, parent);
                let Some(flip) = flip_of(e) else {
                    self.b.tilted += 1;
                    return;
                };
                let (c, r) = ([e.fd(10, 0.0), e.fd(20, 0.0)], e.fd(40, 0.0));
                if r <= 0.0 {
                    self.b.not("POINT");
                    return;
                }
                let (a0, a1) = if kind == "CIRCLE" {
                    (0.0, std::f64::consts::TAU)
                } else {
                    let (s, t) = (e.fd(50, 0.0).to_radians(), e.fd(51, 360.0).to_radians());
                    (s, if t > s { t } else { t + std::f64::consts::TAU })
                };
                let pts: Vec<[f64; 2]> = arc_points(c, r, a0, a1).into_iter().map(|p| if flip { [-p[0], p[1]] } else { p }).collect();
                self.b.layers[layer].curves += 1;
                self.b.hold(layer, if kind == "CIRCLE" { "circle" } else { "arc" });
                self.put_points(&pts, layer, dashed, true, m);
            }
            "ELLIPSE" => {
                let layer = self.layer_of(e, parent);
                let dashed = self.dashed(e, layer, parent);
                if e.fd(210, 0.0).abs() > 1.0e-9 || e.fd(220, 0.0).abs() > 1.0e-9 || e.fd(230, 1.0) < 0.0 {
                    self.b.ellipse_odd += 1;
                    return;
                }
                let (c, major, ratio) = ([e.fd(10, 0.0), e.fd(20, 0.0)], [e.fd(11, 0.0), e.fd(21, 0.0)], e.fd(40, 1.0));
                let (t0, t1) = (e.fd(41, 0.0), e.fd(42, std::f64::consts::TAU));
                let t1 = if t1 > t0 { t1 } else { t1 + std::f64::consts::TAU };
                let minor = [-major[1] * ratio, major[0] * ratio];
                let n = (((t1 - t0) / PIECE_DEGREES.to_radians()).ceil() as usize).max(1);
                let pts: Vec<[f64; 2]> = (0..=n)
                    .map(|i| {
                        let t = t0 + (t1 - t0) * (i as f64) / (n as f64);
                        [c[0] + major[0] * t.cos() + minor[0] * t.sin(), c[1] + major[1] * t.cos() + minor[1] * t.sin()]
                    })
                    .collect();
                self.b.layers[layer].curves += 1;
                self.b.hold(layer, "ellipse");
                self.put_points(&pts, layer, dashed, true, m);
            }
            "SPLINE" => {
                let layer = self.layer_of(e, parent);
                let dashed = self.dashed(e, layer, parent);
                let ctrl: Vec<[f64; 2]> = {
                    let (xs, ys) = (e.all(10), e.all(20));
                    xs.into_iter().zip(ys).map(|(x, y)| [x, y]).collect()
                };
                let fit: Vec<[f64; 2]> = {
                    let (xs, ys) = (e.all(11), e.all(21));
                    xs.into_iter().zip(ys).map(|(x, y)| [x, y]).collect()
                };
                let degree = e.i(71).unwrap_or(3).max(1) as usize;
                let knots = e.all(40);
                let mut weights = e.all(41);
                let pts: Vec<[f64; 2]> = if ctrl.len() > degree && knots.len() == ctrl.len() + degree + 1 {
                    if weights.len() != ctrl.len() {
                        weights = vec![1.0; ctrl.len()];
                    }
                    let (t0, t1) = (knots[degree], knots[ctrl.len()]);
                    let n = (ctrl.len() * 8).max(16);
                    (0..=n).map(|i| spline_at(degree, &knots, &ctrl, &weights, t0 + (t1 - t0) * (i as f64) / (n as f64))).collect()
                } else if fit.len() >= 2 {
                    fit
                } else {
                    ctrl
                };
                if pts.len() < 2 {
                    self.b.not("SPLINE");
                    return;
                }
                self.b.layers[layer].curves += 1;
                self.b.hold(layer, "spline");
                self.put_points(&pts, layer, dashed, true, m);
            }
            "TEXT" | "ATTRIB" | "MTEXT" => {
                let layer = self.layer_of(e, parent);
                let (raw, formatted) = if kind == "MTEXT" {
                    let mut s: String = e.t.iter().filter(|(k, _)| *k == 3).map(|(_, v)| v.as_str()).collect();
                    s.push_str(e.s(1).unwrap_or(""));
                    (s, true)
                } else {
                    (e.s(1).unwrap_or("").to_string(), false)
                };
                let text = plain_words(&raw, formatted);
                if text.is_empty() {
                    return;
                }
                let aligned = kind != "MTEXT" && (e.i(72).unwrap_or(0) != 0 || e.i(73).unwrap_or(0) != 0) && e.f(11).is_some() && e.i(72) != Some(3) && e.i(72) != Some(5);
                let at = if aligned { [e.fd(11, 0.0), e.fd(21, 0.0)] } else { [e.fd(10, 0.0), e.fd(20, 0.0)] };
                let flip = flip_of(e).unwrap_or(false);
                let at = if flip { [-at[0], at[1]] } else { at };
                let local = if kind == "MTEXT" {
                    match (e.f(11), e.f(21)) {
                        (Some(x), Some(y)) if x.hypot(y) > 0.0 => y.atan2(x),
                        _ => e.fd(50, 0.0),
                    }
                } else {
                    e.fd(50, 0.0).to_radians()
                };
                let (k, turn) = m.stretch();
                self.b.layers[layer].texts += 1;
                self.b.made += 1;
                self.b.notes.push(Note { text, at: m.at(at), height: e.fd(40, 0.0) * k, angle: local + turn, layer });
            }
            "DIMENSION" => {
                // Its figure comes, as words, where it is lettered; its lines and arrows do not.
                let layer = self.layer_of(e, parent);
                self.b.dimensions += 1;
                let measured = e.f(42);
                let typed = e.s(1).map(str::trim).unwrap_or("");
                let text = if typed.is_empty() || typed == "<>" {
                    measured.map(|v| {
                        let s = format!("{}", (v * 1.0e4).round() / 1.0e4);
                        match self.b.inches_per_unit {
                            Some(k) if (k - 1.0).abs() < 1.0e-12 => format!("{s} in"),
                            Some(k) if (k - 12.0).abs() < 1.0e-12 => format!("{s} ft"),
                            _ => s,
                        }
                    })
                } else {
                    Some(plain_words(&typed.replace("<>", &measured.map(|v| format!("{}", (v * 1.0e4).round() / 1.0e4)).unwrap_or_default()), true))
                };
                match (text, e.f(11), e.f(21)) {
                    (Some(text), Some(x), Some(y)) if !text.is_empty() => {
                        let (_, turn) = m.stretch();
                        self.b.layers[layer].texts += 1;
                        self.b.made += 1;
                        self.b.notes.push(Note { text, at: m.at([x, y]), height: 0.0, angle: turn, layer });
                    }
                    _ => self.b.dimensions_bare += 1,
                }
            }
            "INSERT" => {
                let Some(name) = e.s(2).map(|s| s.trim().to_string()) else { return };
                let Some(block) = self.blocks.get(&name) else {
                    *self.b.missing.entry(name).or_default() += 1;
                    return;
                };
                if depth >= DEEPEST {
                    self.b.cut_short += 1;
                    return;
                }
                let layer = self.layer_of(e, parent);
                let own_dashed = self.dashed(e, layer, parent);
                let Some(flip) = flip_of(e) else {
                    self.b.tilted += 1;
                    return;
                };
                let (sx, sy) = (e.fd(41, 1.0), e.fd(42, 1.0));
                let turn = e.fd(50, 0.0).to_radians();
                let (cols, rows) = (e.i(70).unwrap_or(1).max(1) as usize, e.i(71).unwrap_or(1).max(1) as usize);
                let (dc, dr) = (e.fd(44, 0.0), e.fd(45, 0.0));
                let at = [e.fd(10, 0.0), e.fd(20, 0.0)];
                let (s, c) = turn.sin_cos();
                let mut count = 0;
                for r in 0..rows.min(1_000) {
                    for k in 0..cols.min(1_000) {
                        // The block's own point is its base; one in an array is shifted along the insert's turned axes.
                        let (ox, oy) = (k as f64 * dc, r as f64 * dr);
                        let shift = [at[0] + c * ox - s * oy, at[1] + s * ox + c * oy];
                        let local = M { a: c * sx, b: s * sx, c: -s * sy, d: c * sy, e: shift[0], f: shift[1] };
                        // What the insert says is in its own plane; a plane facing down mirrors x to reach the drawing.
                        let ocs = if flip { M { a: -1.0, ..M::ONE } } else { M::ONE };
                        let base = M { e: -block.base[0], f: -block.base[1], ..M::ONE };
                        let all = base.then(&local).then(&ocs).then(m);
                        self.go(&block.ents, &all, Some((layer, own_dashed)), depth + 1);
                        count += 1;
                    }
                }
                *self.b.blocks.entry(name).or_default() += count;
            }
            "SEQEND" | "VERTEX" | "ENDBLK" | "BLOCK" => {}
            other => self.b.not(other),
        }
    }

    fn polyline(&mut self, verts: &[([f64; 2], f64)], closed: bool, flip: bool, layer: usize, dashed: bool, m: &M) {
        let n = verts.len();
        if n < 2 {
            return;
        }
        self.b.hold(layer, "polyline");
        let p = |i: usize| if flip { [-verts[i].0[0], verts[i].0[1]] } else { verts[i].0 };
        let sides = if closed { n } else { n - 1 };
        for i in 0..sides {
            let (a, b) = (p(i), p((i + 1) % n));
            // Mirrored, a bulge turns the other way.
            let bulge = if flip { -verts[i].1 } else { verts[i].1 };
            if bulge.abs() < 1.0e-12 {
                self.put_points(&[a, b], layer, dashed, false, m);
            } else {
                self.b.layers[layer].curves += 1;
                let pts = bulge_points(a, b, bulge);
                self.put_points(&pts, layer, dashed, true, m);
            }
        }
    }
}

/// A DXF read as a drawing.
pub fn read(bytes: &[u8]) -> Result<Cad, String> {
    if !is_dxf(bytes) {
        return Err("it is not a CAD drawing (DXF)".into());
    }
    let all = tags(bytes)?;
    // Entities, split at each group 0.
    let mut ents: Vec<Ent> = Vec::new();
    for t in all {
        if t.code == 0 {
            ents.push(Ent { kind: t.val.trim().to_string(), t: Vec::new(), sub: Vec::new() });
        } else if let Some(last) = ents.last_mut() {
            last.t.push((t.code, t.val));
        }
    }
    let (mut insunits, mut measurement): (Option<i64>, Option<i64>) = (None, None);
    let mut layer_table: Vec<(String, bool, String)> = Vec::new();
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut model: Vec<Ent> = Vec::new();
    let (mut section, mut current): (String, Option<String>) = (String::new(), None);
    let mut open = false;
    let mut model_open = false;
    for e in ents {
        match e.kind.as_str() {
            "SECTION" => {
                section = e.s(2).unwrap_or("").trim().to_ascii_uppercase();
                open = false;
                if section == "HEADER" {
                    let mut var = String::new();
                    for (k, v) in &e.t {
                        if *k == 9 {
                            var = v.trim().to_ascii_uppercase();
                        } else if *k == 70 {
                            match var.as_str() {
                                "$INSUNITS" => insunits = v.trim().parse().ok(),
                                "$MEASUREMENT" => measurement = v.trim().parse().ok(),
                                _ => {}
                            }
                        }
                    }
                }
            }
            "ENDSEC" => {
                section.clear();
                current = None;
            }
            "EOF" => break,
            _ => match section.as_str() {
                "TABLES" if e.kind == "LAYER" => {
                    let off = e.i(70).unwrap_or(0) & 1 == 1 || e.f(62).is_some_and(|c| c < 0.0);
                    layer_table.push((e.s(2).unwrap_or("0").trim().to_string(), off, e.s(6).unwrap_or("").trim().to_string()));
                }
                "BLOCKS" => match e.kind.as_str() {
                    "BLOCK" => {
                        let name = e.s(2).or_else(|| e.s(3)).unwrap_or("").trim().to_string();
                        blocks.insert(name.clone(), Block { base: [e.fd(10, 0.0), e.fd(20, 0.0)], ents: Vec::new() });
                        current = Some(name);
                        open = false;
                    }
                    "ENDBLK" => {
                        current = None;
                        open = false;
                    }
                    _ => {
                        if let Some(b) = current.as_ref().and_then(|n| blocks.get_mut(n)) {
                            push(&mut b.ents, &mut open, e);
                        }
                    }
                },
                "ENTITIES" => push(&mut model, &mut model_open, e),
                _ => {}
            },
        }
    }
    let (units, no_units) = match insunits {
        Some(c) if c != 0 => match units_of(c) {
            Some(u) => (Some(u), String::new()),
            None => (None, format!("the drawing says it is drawn in a unit (code {c}) that no building is drawn in")),
        },
        _ => (
            None,
            match measurement {
                Some(0) => "the drawing does not say what it is drawn in (it says only that its measure is English, feet and inches, which does not say which)".to_string(),
                Some(1) => "the drawing does not say what it is drawn in (it says only that its measure is metric, which does not say which)".to_string(),
                _ => "the drawing does not say what it is drawn in".to_string(),
            },
        ),
    };
    let mut b = Build {
        layers: Vec::new(),
        index: HashMap::new(),
        segs: Vec::new(),
        notes: Vec::new(),
        left: BTreeMap::new(),
        blocks: BTreeMap::new(),
        paper: 0,
        tilted: 0,
        not_level: 0,
        nothing_long: 0,
        over: 0,
        cut_short: 0,
        missing: BTreeMap::new(),
        dimensions: 0,
        dimensions_bare: 0,
        ellipse_odd: 0,
        made: 0,
        inches_per_unit: units.map(|u| u.inches),
    };
    for (name, off, lt) in &layer_table {
        let i = b.layer(name);
        b.layers[i].off = *off;
        b.layers[i].linetype = lt.clone();
    }
    {
        let mut w = Walk { b: &mut b, blocks: &blocks };
        w.go(&model, &M::ONE, None, 0);
    }
    if b.segs.is_empty() && b.notes.is_empty() {
        let mut why = "the drawing holds nothing that is drawn as lines or words, so there is nothing to trace over".to_string();
        if !b.left.is_empty() {
            why.push_str(&format!(" (it holds {})", left_said(&b.left)));
        }
        return Err(why);
    }
    // What is drawn spans so much, from its lower-left corner.
    let mut lo = [f64::MAX, f64::MAX];
    let mut hi = [f64::MIN, f64::MIN];
    let mut grow = |p: [f64; 2]| {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    };
    for s in &b.segs {
        grow(s.a);
        grow(s.b);
    }
    for n in &b.notes {
        grow(n.at);
    }
    let size = [(hi[0] - lo[0]).max(1.0), (hi[1] - lo[1]).max(1.0)];
    for s in &mut b.segs {
        s.a = [s.a[0] - lo[0], s.a[1] - lo[1]];
        s.b = [s.b[0] - lo[0], s.b[1] - lo[1]];
    }
    for n in &mut b.notes {
        n.at = [n.at[0] - lo[0], n.at[1] - lo[1]];
    }
    let mut said: Vec<String> = Vec::new();
    let (curves, pieces): (usize, usize) = (b.layers.iter().map(|l| l.curves).sum(), b.layers.iter().map(|l| l.pieces).sum());
    if curves > 0 {
        said.push(format!("its {} drawn as {} of at most {PIECE_DEGREES} degrees (none leaves its curve by more than 0.4 per cent of its radius)", plural(curves, "curve is", "curves are"), plural(pieces, "straight piece", "straight pieces")));
    }
    if b.not_level > 0 {
        said.push(format!("{} not level (drawn at different heights): kept as seen from above, as a plan", plural(b.not_level, "line is", "lines are")));
    }
    if b.nothing_long > 0 {
        said.push(format!("{} of no length not kept", plural(b.nothing_long, "line is", "lines are")));
    }
    if b.dimensions > 0 {
        said.push(format!("{} in the drawing: the figure of each is brought in as words where it is lettered; its lines and arrows are not", plural(b.dimensions, "dimension is", "dimensions are")));
    }
    if b.dimensions_bare > 0 {
        said.push(format!("{} has no figure to bring in", plural(b.dimensions_bare, "dimension", "dimensions")));
    }
    if b.paper > 0 {
        said.push(format!("{} drawn on a sheet layout (paper space) and not brought in: the drawing itself is what is on the model", plural(b.paper, "thing is", "things are")));
    }
    if b.tilted > 0 {
        said.push(format!("{} drawn on a tilted plane, not a plan: not brought in", plural(b.tilted, "thing is", "things are")));
    }
    if b.ellipse_odd > 0 {
        said.push(format!("{} drawn on a tilted plane: not brought in", plural(b.ellipse_odd, "ellipse is", "ellipses are")));
    }
    for (name, n) in &b.missing {
        said.push(format!("{} of the block {name:?}, which the drawing does not hold: not brought in", plural(*n, "insert", "inserts")));
    }
    if b.over > 0 {
        said.push(format!("{} beyond the first {MOST_LINES} are not kept", plural(b.over, "line", "lines")));
    }
    if b.cut_short > 0 {
        said.push(format!("{} not drawn: the blocks hold blocks too deep, or too many, to follow", plural(b.cut_short, "thing was", "things were")));
    }
    Ok(Cad {
        units,
        no_units,
        layers: b.layers,
        segs: b.segs,
        notes: b.notes,
        size,
        origin: lo,
        left: b.left,
        notes_said: said,
        blocks: b.blocks.into_iter().collect(),
        dimensions: b.dimensions,
    })
}

/// "2 hatches, 1 picture placed in the drawing", by kind.
pub fn left_said(left: &BTreeMap<String, usize>) -> String {
    // Kinds that read as one thing are put together: a 3D solid, a body and a region are 3D solids.
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for (k, n) in left {
        *by.entry(called(k).0).or_default() += n;
    }
    let mut parts: Vec<String> = Vec::new();
    for (one, n) in by {
        let many = left.keys().map(|k| called(k)).find(|c| c.0 == one).map(|c| c.1).unwrap_or(one);
        parts.push(plural(n, one, many));
    }
    parts.join(", ")
}

/// "4 lines, 1 circle": what is held, by kind, each in its order.
fn held_said(held: &[(&'static str, usize)]) -> Vec<String> {
    const ORDER: [(&str, &str, &str); 6] = [("line", "line", "lines"), ("polyline", "polyline", "polylines"), ("arc", "arc", "arcs"), ("circle", "circle", "circles"), ("ellipse", "ellipse", "ellipses"), ("spline", "spline", "splines")];
    ORDER.iter().filter_map(|(k, one, many)| held.iter().find(|(h, _)| h == k).map(|(_, n)| plural(*n, one, many))).collect()
}

impl Cad {
    /// What the whole drawing holds, by kind: "8 lines, 1 circle".
    pub fn held(&self) -> String {
        let mut all: Vec<(&'static str, usize)> = Vec::new();
        for l in &self.layers {
            for (k, n) in &l.held {
                match all.iter_mut().find(|(h, _)| h == k) {
                    Some(e) => e.1 += n,
                    None => all.push((k, *n)),
                }
            }
        }
        held_said(&all).join(", ")
    }
    /// The layers the file itself has off.
    pub fn off_by_the_file(&self) -> Vec<String> {
        self.layers.iter().filter(|l| l.off).map(|l| l.name.clone()).collect()
    }
    fn is_off(&self, layer: usize, off: &[String]) -> bool {
        off.iter().any(|n| *n == self.layers[layer].name)
    }
    /// What the file holds that is not brought in, in words, one sentence a kind; and what else is to be said.
    pub fn lost(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut by: BTreeMap<&str, usize> = BTreeMap::new();
        for (k, n) in &self.left {
            *by.entry(called(k).0).or_default() += n;
        }
        for (one, n) in by {
            let many = self.left.keys().map(|k| called(k)).find(|c| c.0 == one).map(|c| c.1).unwrap_or(one);
            out.push(format!("{} in the drawing and not brought in", plural(n, &format!("{one} is"), &format!("{many} are"))));
        }
        out.extend(self.notes_said.iter().cloned());
        out
    }
    /// The page of the drawing with the layers named in `off` left off: its straight lines to snap to and its picture.
    pub fn page(&self, off: &[String]) -> Page {
        let lines: Vec<[[f64; 2]; 2]> = self.segs.iter().filter(|s| !self.is_off(s.layer, off)).map(|s| [s.a, s.b]).collect();
        let dashed: Vec<usize> = self.segs.iter().filter(|s| !self.is_off(s.layer, off)).enumerate().filter(|(_, s)| s.dashed).map(|(i, _)| i).collect();
        let picture = draw(&lines, self.size);
        Page { sort: Sort::Dxf, pages: 1, size: self.size, picture, lines, dashed, cuts: Vec::new(), pictures: 0, lost: self.lost() }
    }
    /// The words of the drawing, with the layers named in `off` left off.
    pub fn words(&self, off: &[String]) -> Vec<Text> {
        self.notes
            .iter()
            .filter(|n| !self.is_off(n.layer, off))
            .map(|n| Text { text: n.text.clone(), at: n.at, tall: n.height, dir: [n.angle.cos(), n.angle.sin()] })
            .collect()
    }
    /// Each layer in one line of words: what it holds.
    pub fn layer_said(&self, l: &Layer) -> String {
        let mut parts: Vec<String> = held_said(&l.held);
        if l.texts > 0 {
            parts.push(plural(l.texts, "line of words", "lines of words"));
        }
        if parts.is_empty() {
            parts.push("nothing".to_string());
        }
        format!("{}: {}{}", l.name, parts.join(", "), if l.off { " (off in the file)" } else { "" })
    }
}

/// The lines drawn as dark ink on white, the longer side at most `LONG_SIDE` dots (never more than 2048 here).
fn draw(lines: &[[[f64; 2]; 2]], size: [f64; 2]) -> Picture {
    let k = (f64::from(LONG_SIDE.min(2048)) / size[0].max(size[1])).min(1.0e6);
    let (w, h) = (((size[0] * k).ceil() as u32 + 1).max(2), ((size[1] * k).ceil() as u32 + 1).max(2));
    let mut rgba = vec![255u8; (w as usize) * (h as usize) * 4];
    let mut dot = |x: i64, y: i64| {
        if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h {
            // y runs up the sheet; the picture's rows run down.
            let row = i64::from(h) - 1 - y;
            let at = ((row as usize) * (w as usize) + x as usize) * 4;
            rgba[at..at + 3].copy_from_slice(&[30, 30, 30]);
        }
    };
    for [a, b] in lines {
        let (x0, y0, x1, y1) = ((a[0] * k).round() as i64, (a[1] * k).round() as i64, (b[0] * k).round() as i64, (b[1] * k).round() as i64);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            dot(x, y);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    Picture { width: w, height: h, rgba }
}

// ---- Written out ----

/// A DXF being written: inches at true size, in the R12 form every CAD program reads (LINE, ARC, CIRCLE and
/// TEXT, a table of layers), $INSUNITS and $MEASUREMENT set, extents given.
pub struct Writer {
    layers: Vec<(String, i32)>,
    body: String,
    lo: [f64; 2],
    hi: [f64; 2],
}

fn num(v: f64) -> String {
    let v = if v.abs() < 1.0e-12 { 0.0 } else { (v * 1.0e6).round() / 1.0e6 };
    format!("{}", v + 0.0)
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer {
    pub fn new() -> Writer {
        Writer { layers: Vec::new(), body: String::new(), lo: [f64::MAX; 2], hi: [f64::MIN; 2] }
    }
    /// A layer with a colour (AutoCAD's colour numbers). A layer used and not declared is declared in white.
    pub fn layer(&mut self, name: &str, colour: i32) {
        if !self.layers.iter().any(|(n, _)| n == name) {
            self.layers.push((name.to_string(), colour));
        }
    }
    fn on(&mut self, layer: &str) {
        self.layer(layer, 7);
    }
    fn grow(&mut self, p: [f64; 2]) {
        for k in 0..2 {
            self.lo[k] = self.lo[k].min(p[k]);
            self.hi[k] = self.hi[k].max(p[k]);
        }
    }
    fn pair(&mut self, code: i32, v: &str) {
        self.body.push_str(&format!("{code:>3}\n{v}\n"));
    }
    pub fn line(&mut self, layer: &str, a: [f64; 2], b: [f64; 2]) {
        self.on(layer);
        self.grow(a);
        self.grow(b);
        self.pair(0, "LINE");
        self.pair(8, layer);
        self.pair(10, &num(a[0]));
        self.pair(20, &num(a[1]));
        self.pair(30, "0");
        self.pair(11, &num(b[0]));
        self.pair(21, &num(b[1]));
        self.pair(31, "0");
    }
    pub fn circle(&mut self, layer: &str, c: [f64; 2], r: f64) {
        self.on(layer);
        self.grow([c[0] - r, c[1] - r]);
        self.grow([c[0] + r, c[1] + r]);
        self.pair(0, "CIRCLE");
        self.pair(8, layer);
        self.pair(10, &num(c[0]));
        self.pair(20, &num(c[1]));
        self.pair(30, "0");
        self.pair(40, &num(r));
    }
    /// An arc from `a0` to `a1`, degrees counter-clockwise from east.
    pub fn arc(&mut self, layer: &str, c: [f64; 2], r: f64, a0: f64, a1: f64) {
        self.on(layer);
        self.grow([c[0] - r, c[1] - r]);
        self.grow([c[0] + r, c[1] + r]);
        self.pair(0, "ARC");
        self.pair(8, layer);
        self.pair(10, &num(c[0]));
        self.pair(20, &num(c[1]));
        self.pair(30, "0");
        self.pair(40, &num(r));
        self.pair(50, &num(a0));
        self.pair(51, &num(a1));
    }
    /// Words at a point, `rot` degrees counter-clockwise from east, `height` tall.
    pub fn text(&mut self, layer: &str, at: [f64; 2], height: f64, rot: f64, words: &str) {
        self.on(layer);
        self.grow(at);
        self.pair(0, "TEXT");
        self.pair(8, layer);
        self.pair(10, &num(at[0]));
        self.pair(20, &num(at[1]));
        self.pair(30, "0");
        self.pair(40, &num(height));
        self.pair(1, &words.replace(['\r', '\n'], " "));
        if rot.abs() > 1.0e-9 {
            self.pair(50, &num(rot));
        }
    }
    pub fn finish(mut self) -> String {
        if self.lo[0] > self.hi[0] {
            self.lo = [0.0; 2];
            self.hi = [0.0; 2];
        }
        self.layer("0", 7);
        let mut o = String::new();
        let mut p = |code: i32, v: &str| o.push_str(&format!("{code:>3}\n{v}\n"));
        p(0, "SECTION");
        p(2, "HEADER");
        p(9, "$ACADVER");
        p(1, "AC1009");
        p(9, "$INSUNITS");
        p(70, "1");
        p(9, "$MEASUREMENT");
        p(70, "0");
        p(9, "$EXTMIN");
        p(10, &num(self.lo[0]));
        p(20, &num(self.lo[1]));
        p(30, "0");
        p(9, "$EXTMAX");
        p(10, &num(self.hi[0]));
        p(20, &num(self.hi[1]));
        p(30, "0");
        p(0, "ENDSEC");
        p(0, "SECTION");
        p(2, "TABLES");
        p(0, "TABLE");
        p(2, "LTYPE");
        p(70, "1");
        p(0, "LTYPE");
        p(2, "CONTINUOUS");
        p(70, "0");
        p(3, "Solid line");
        p(72, "65");
        p(73, "0");
        p(40, "0.0");
        p(0, "ENDTAB");
        p(0, "TABLE");
        p(2, "LAYER");
        p(70, &self.layers.len().to_string());
        for (name, colour) in &self.layers {
            p(0, "LAYER");
            p(2, name);
            p(70, "0");
            p(62, &colour.to_string());
            p(6, "CONTINUOUS");
        }
        p(0, "ENDTAB");
        p(0, "TABLE");
        p(2, "STYLE");
        p(70, "1");
        p(0, "STYLE");
        p(2, "STANDARD");
        p(70, "0");
        p(40, "0.0");
        p(41, "1.0");
        p(50, "0.0");
        p(71, "0");
        p(42, "0.2");
        p(3, "txt");
        p(4, "");
        p(0, "ENDTAB");
        p(0, "ENDSEC");
        p(0, "SECTION");
        p(2, "BLOCKS");
        p(0, "ENDSEC");
        p(0, "SECTION");
        p(2, "ENTITIES");
        o.push_str(&self.body);
        let mut p = |code: i32, v: &str| o.push_str(&format!("{code:>3}\n{v}\n"));
        p(0, "ENDSEC");
        p(0, "EOF");
        o
    }
}
