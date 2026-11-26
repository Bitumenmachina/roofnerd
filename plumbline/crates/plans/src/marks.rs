//! The marks on a marked-up PDF, read as quantities (call 227).
//!
//! An estimator's takeoff often already exists as measured marks on the plans. A PDF keeps such a mark as an ordinary
//! note of the page (an "annotation", PDF 32000-1 section 12.5): a line, a run of lines, a closed shape, each with its
//! points, its own text, a subject, a name of its own and, for a measured one, a measure dictionary (section 12.9: how
//! the sheet's scale was set when it was drawn). `research/pdf-marks.md` has each key read here, with where it is in the
//! standard, and what could not be confirmed. **No file written by a marking program was read to write this**: it is
//! written to the standard, and its tests read files the tests themselves write.
//!
//! What a mark gives:
//! - **what it says**: the figure in its own text (`Contents`), which is what the estimator saw and signed off;
//! - **what its points work out to** at the scale it carries (`Measure`), worked here as the standard says: the points
//!   in default user space, times the first conversion of the x array (and of y), then of the distance array (the
//!   area array for an area), to that array's unit.
//!
//! The two are never merged. Where they differ by more than a hundredth, `differs` is true and both are given.
//!
//! A mark that cannot be read (a curved edge, no scale, a scale for part of a sheet only, a figure in a unit not held,
//! no figure at all) is in the list with `unread` saying why; it is never given a figure it did not say.
//!
//! The standard has no mark for a count. A count is read by the convention in `research/pdf-marks.md`, which could not
//! be confirmed against any marking program's own file: a stamp, which the page shows as a symbol, labelled by its subject;
//! stamps of one label are one count.

use hayro::hayro_interpret::util::TransformExt;
use hayro::hayro_syntax::object::dict::keys::ANNOTS;
use hayro::hayro_syntax::object::{Array, Dict, Name, String as PdfString};
use hayro::hayro_syntax::Pdf;
use hayro::kurbo::Point;

/// A hundredth: the two figures of a mark differ when they are further apart than this, in LF or SF.
pub const AGREE: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// One length (a line).
    Length,
    /// A run of lengths: a perimeter, or lengths end to end.
    Run,
    /// An area.
    Area,
    /// A count of symbols of one label.
    Count,
    /// A note that measures nothing.
    Note,
}

impl Kind {
    /// As an estimator says it.
    pub fn said(self) -> &'static str {
        match self {
            Kind::Length => "length",
            Kind::Run => "run of lengths",
            Kind::Area => "area",
            Kind::Count => "count",
            Kind::Note => "note",
        }
    }
}

/// What a mark's own text says: a figure and the unit it is in (LF, SF or EA), and the text it was in.
#[derive(Clone, Debug, PartialEq)]
pub struct Says {
    pub value: f64,
    pub unit: &'static str,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub kind: Kind,
    pub label: String,
    /// The mark's own name in the file (its `NM`): what a second reading knows it by.
    pub name: Option<String>,
    pub says: Option<Says>,
    /// What its points work out to at the scale it carries, in the unit of `says` (or LF for a length and a run, SF for
    /// an area when it says nothing).
    pub worked: Option<f64>,
    /// Why its points could not be worked, where its scale is there and partly unreadable.
    pub not_worked: Option<String>,
    /// The scale as the mark writes it ("1/4\" = 1'-0\""), where it does.
    pub scale: Option<String>,
    /// The mark's points, in the page's units from its lower-left corner (a count: none).
    pub points: Vec<[f64; 2]>,
    /// Where it is on the page: its centre; a count: each symbol, in the page's order.
    pub places: Vec<[f64; 2]>,
    /// The mark's own text as written.
    pub text: String,
    /// Why it is not read, where it is not.
    pub unread: Option<String>,
}

impl Mark {
    /// Where the mark is on the page: its centre, or a count's first symbol.
    pub fn at(&self) -> [f64; 2] {
        self.places.first().copied().unwrap_or([0.0, 0.0])
    }
    /// The figure it says and the figure its points work out to are further apart than a hundredth.
    pub fn differs(&self) -> bool {
        match (&self.says, self.worked) {
            (Some(s), Some(w)) => (s.value - w).abs() > AGREE,
            _ => false,
        }
    }
}

/// What each mark of a page is known by when the same file is read again: its own name in the file where it has one, else
/// what kind of mark it is, its label and which of that label and kind it is on the page (a count: its label). Never holds
/// a double quote. Two marks never have one key: a name given twice is made two by its place on the page.
pub fn keys(marks: &[Mark]) -> Vec<String> {
    let clean = |s: &str| s.chars().map(|c| if c == '"' { '\'' } else if c.is_control() { ' ' } else { c }).collect::<String>().trim().to_string();
    let mut out: Vec<String> = Vec::new();
    for (i, m) in marks.iter().enumerate() {
        let base = match (&m.name, m.kind) {
            (Some(n), _) => clean(n),
            (None, Kind::Count) => format!("count {}", clean(&m.label)),
            (None, k) => {
                let nth = marks[..=i].iter().filter(|o| o.kind == k && o.label == m.label && o.name.is_none()).count();
                format!("{} {} #{nth}", k.said(), clean(&m.label))
            }
        };
        let mut key = base.clone();
        let mut n = 2;
        while out.contains(&key) {
            key = format!("{base} #{n}");
            n += 1;
        }
        out.push(key);
    }
    out
}

/// The marks of one page.
#[derive(Clone, Debug, PartialEq)]
pub struct Marks {
    pub page: usize,
    pub pages: usize,
    /// In the page's own order (a count at its first symbol).
    pub marks: Vec<Mark>,
    /// What was on the page and is not a mark: hidden marks, links, marks with no text.
    pub left: Vec<String>,
}

// ---- A figure in a mark's own text ----

#[derive(Clone, Debug, PartialEq)]
pub enum Figure {
    /// No figure with a unit in it.
    None,
    Said(Says),
    /// A figure in a unit this program does not hold, named as the text names it.
    Unheld { text: String, unit: String },
}

fn feet_unit(w: &str) -> bool {
    matches!(w, "ft" | "feet" | "foot" | "lf" | "linear ft" | "linear feet" | "linear foot" | "lin ft" | "lin feet")
}
fn inch_unit(w: &str) -> bool {
    matches!(w, "in" | "inch" | "inches")
}
fn area_unit(w: &str) -> bool {
    matches!(w, "sf" | "sq ft" | "sqft" | "ft2" | "ft\u{b2}" | "sq feet" | "sq foot" | "square feet" | "square foot" | "sf." | "sq. ft.")
}
fn unheld_unit(w: &str) -> bool {
    matches!(w, "m" | "mm" | "cm" | "km" | "m2" | "m\u{b2}" | "sqm" | "sq m" | "yd" | "yds" | "yard" | "yards" | "sy" | "sq yd" | "mi" | "cf" | "cy" | "cu ft" | "cu yd" | "metre" | "metres" | "meter" | "meters" | "sq in" | "in2" | "sq inches")
}

/// The first figure with a unit in a mark's own text: `31'-0"`, `12' 6 1/2"`, `30 LF`, `600 SF`, `1,200 sq ft`, `9 m`.
/// A number with no unit after it (a detail's number, a size like 2x4) is not a figure and is passed over.
pub fn figure(text: &str) -> Figure {
    let c: Vec<char> = text.chars().collect();
    let mut i = 0;
    let skip = |i: &mut usize| {
        while *i < c.len() && c[*i].is_whitespace() {
            *i += 1;
        }
    };
    // A number at i: digits, thousands commas and a decimal point. Returns (value, next).
    let number = |mut i: usize| -> Option<(f64, usize)> {
        let start = i;
        let mut s = String::new();
        let mut dot = false;
        while i < c.len() {
            let ch = c[i];
            if ch.is_ascii_digit() {
                s.push(ch);
            } else if ch == ',' && i + 1 < c.len() && c[i + 1].is_ascii_digit() && !dot && i > start {
                // a thousands comma
            } else if ch == '.' && !dot && i + 1 < c.len() && c[i + 1].is_ascii_digit() {
                dot = true;
                s.push('.');
            } else {
                break;
            }
            i += 1;
        }
        if s.is_empty() {
            None
        } else {
            s.parse::<f64>().ok().map(|v| (v, i))
        }
    };
    // Letters (and the dot, the space between "sq" and "ft", a digit 2 or a superscript 2 straight after letters).
    let word = |mut i: usize| -> (String, usize) {
        let mut w = String::new();
        while i < c.len() {
            let ch = c[i];
            if ch.is_alphabetic() || ch == '\u{b2}' || (ch == '.' && !w.is_empty()) {
                w.push(ch.to_ascii_lowercase());
                i += 1;
            } else if ch == '2' && !w.is_empty() && !c.get(i + 1).is_some_and(|d| d.is_ascii_digit()) {
                w.push('2');
                i += 1;
            } else if ch == ' ' && !w.is_empty() && matches!(w.trim_end_matches('.'), "sq" | "linear" | "lin" | "cu" | "square") && c.get(i + 1).is_some_and(|d| d.is_alphabetic()) {
                w.push(' ');
                i += 1;
            } else {
                break;
            }
        }
        (w.trim().trim_matches('.').replace(". ", " ").replace('.', ""), i)
    };
    while i < c.len() {
        if !c[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let Some((n, mut j)) = number(i) else {
            i += 1;
            continue;
        };
        // A fraction right after (1/2), only for inches.
        let after_number = j;
        skip(&mut j);
        let here = j;
        // Feet, then inches.
        let (w, wend) = word(j);
        let is_foot_mark = c.get(j) == Some(&'\'') || c.get(j) == Some(&'\u{2032}');
        if is_foot_mark || feet_unit(&w) {
            let mut k = if is_foot_mark { j + 1 } else { wend };
            let mut feet = n;
            let mut said_to = k;
            // inches after the feet: `-`, then number, an optional fraction, then `"` or "in"
            let mut m = k;
            skip(&mut m);
            if m < c.len() && c[m] == '-' {
                m += 1;
                skip(&mut m);
            }
            if let Some((mut inch, mut p)) = number(m) {
                let mut q = p;
                skip(&mut q);
                // a mixed fraction: 6 1/2
                if let Some((top, p2)) = number(q) {
                    if p2 < c.len() && c[p2] == '/' {
                        if let Some((bottom, p3)) = number(p2 + 1) {
                            if bottom > 0.0 {
                                inch += top / bottom;
                                p = p3;
                            }
                        }
                    }
                } else if q < c.len() && c[q] == '/' {
                    // a bare fraction (6/8 is no inch: only whole-and-fraction is read)
                }
                let mut q = p;
                skip(&mut q);
                let (w2, w2end) = word(q);
                if c.get(q) == Some(&'"') || c.get(q) == Some(&'\u{2033}') {
                    feet += inch / 12.0;
                    said_to = q + 1;
                } else if inch_unit(&w2) {
                    feet += inch / 12.0;
                    said_to = w2end;
                }
            }
            k = said_to;
            let text: String = c[i..k].iter().collect();
            return Figure::Said(Says { value: feet, unit: "LF", text: text.trim().to_string() });
        }
        if c.get(j) == Some(&'"') || c.get(j) == Some(&'\u{2033}') {
            let text: String = c[i..j + 1].iter().collect();
            return Figure::Said(Says { value: n / 12.0, unit: "LF", text });
        }
        if inch_unit(&w) {
            let text: String = c[i..wend].iter().collect();
            return Figure::Said(Says { value: n / 12.0, unit: "LF", text });
        }
        if area_unit(&w) {
            let text: String = c[i..wend].iter().collect();
            return Figure::Said(Says { value: n, unit: "SF", text });
        }
        if matches!(w.as_str(), "ea" | "each") {
            let text: String = c[i..wend].iter().collect();
            return Figure::Said(Says { value: n, unit: "EA", text });
        }
        if unheld_unit(&w) {
            let text: String = c[i..wend].iter().collect();
            return Figure::Unheld { text, unit: w };
        }
        let _ = (after_number, here);
        i = j.max(i + 1);
    }
    Figure::None
}

// ---- The scale a mark carries ----

#[derive(Default)]
struct Scale {
    /// Conversion of default user space to the x array's unit, and the y array's where it has one.
    x: Option<f64>,
    y: Option<f64>,
    /// The distance array's first conversion and unit; the area array's.
    d: Option<(f64, String)>,
    a: Option<(f64, String)>,
    ratio: Option<String>,
    /// Not a rectilinear scale.
    other: bool,
}

fn text_of(s: &PdfString) -> String {
    let b = s.as_bytes();
    if let Some(rest) = b.strip_prefix(&[0xFE, 0xFF]) {
        let units: Vec<u16> = rest.chunks_exact(2).map(|p| u16::from_be_bytes([p[0], p[1]])).collect();
        String::from_utf16_lossy(&units)
    } else if let Some(rest) = b.strip_prefix(&[0xFF, 0xFE]) {
        let units: Vec<u16> = rest.chunks_exact(2).map(|p| u16::from_le_bytes([p[0], p[1]])).collect();
        String::from_utf16_lossy(&units)
    } else if let Some(rest) = b.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(rest).into_owned()
    } else {
        // The standard's own text encoding is close to Latin-1 for what a plan carries.
        b.iter().map(|&x| x as char).collect()
    }
}

fn str_of(d: &Dict<'_>, key: &str) -> Option<String> {
    d.get::<PdfString>(key).map(|s| text_of(&s)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn first_format(d: &Dict<'_>, key: &str) -> Option<(f64, String)> {
    let a = d.get::<Array<'_>>(key)?;
    let first = a.iter::<Dict<'_>>().next()?;
    Some((first.get::<f64>("C")?, str_of(&first, "U").unwrap_or_default()))
}

fn scale_of(m: &Dict<'_>) -> Scale {
    let sub = m.get::<Name<'_>>("Subtype");
    Scale { x: first_format(m, "X").map(|x| x.0), y: first_format(m, "Y").map(|y| y.0), d: first_format(m, "D"), a: first_format(m, "A"), ratio: str_of(m, "R"), other: sub.is_some_and(|n| n.as_str() != "RL") }
}

fn to_feet(unit: &str) -> Option<f64> {
    let u = unit.trim().to_lowercase().replace('.', "");
    if matches!(u.as_str(), "ft" | "feet" | "foot" | "'") {
        Some(1.0)
    } else if matches!(u.as_str(), "in" | "inch" | "inches" | "\"") {
        Some(1.0 / 12.0)
    } else {
        None
    }
}

fn to_square_feet(unit: &str) -> Option<f64> {
    let u = unit.trim().to_lowercase().replace('.', "");
    if matches!(u.as_str(), "sq ft" | "sf" | "ft2" | "ft\u{b2}" | "square feet" | "sqft") {
        Some(1.0)
    } else if matches!(u.as_str(), "sq in" | "in2" | "in\u{b2}" | "square inches" | "sqin") {
        Some(1.0 / 144.0)
    } else {
        None
    }
}

fn path_length(p: &[[f64; 2]], closed: bool) -> f64 {
    let mut total: f64 = p.windows(2).map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1])).sum();
    if closed && p.len() > 2 {
        let (a, b) = (p[p.len() - 1], p[0]);
        total += (b[0] - a[0]).hypot(b[1] - a[1]);
    }
    total
}

fn shoelace(p: &[[f64; 2]]) -> f64 {
    let mut s = 0.0;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        s += a[0] * b[1] - b[0] * a[1];
    }
    s.abs() / 2.0
}

/// What the points work out to at the scale, in LF (`area` false) or SF (true): a figure, or why none.
fn worked(s: &Scale, points: &[[f64; 2]], closed: bool, area: bool) -> Result<f64, String> {
    if s.other {
        return Err("its scale is of a kind this program does not read".into());
    }
    let x = s.x.ok_or("its scale gives no conversion along the sheet")?;
    let y = s.y.unwrap_or(x);
    if (x - y).abs() > x.abs() * 1e-9 {
        return Err("its scale differs along and across the sheet".into());
    }
    if area {
        let (c, u) = s.a.as_ref().ok_or("its scale has no unit for an area")?;
        let k = to_square_feet(u).ok_or_else(|| format!("its scale is in {u:?}, a unit of area this program does not hold"))?;
        Ok(shoelace(points) * x * y * c * k)
    } else {
        let (c, u) = s.d.as_ref().ok_or("its scale has no unit for a length")?;
        let k = to_feet(u).ok_or_else(|| format!("its scale is in {u:?}, a unit of length this program does not hold"))?;
        Ok(path_length(points, closed) * x * c * k)
    }
}

// ---- Reading a page ----

const NOT_A_PDF: &str = "only a PDF has marks that can be read; a picture of a sheet has none";

/// The marks of one page (counted from 1) of a PDF, in the page's own order.
pub fn read(bytes: &[u8], page: usize) -> Result<Marks, String> {
    if crate::sort_of(bytes) != Some(crate::Sort::Pdf) {
        return Err(NOT_A_PDF.into());
    }
    let pdf = Pdf::new(bytes.to_vec()).map_err(|_| "the PDF could not be read: it is damaged, cut short, locked with a password, or not a PDF at all. Print it to a new PDF, or send it again".to_string())?;
    let pages = pdf.pages();
    let count = pages.len();
    let (_, height) = crate::page_size(pages, page)?;
    let p = &pages[page - 1];
    let to_page = p.initial_transform(true).to_kurbo();
    let h = f64::from(height);
    let place = |q: [f64; 2]| {
        let r = to_page * Point::new(q[0], q[1]);
        [r.x, h - r.y]
    };
    let viewports = p.raw().get::<Array<'_>>("VP").is_some_and(|a| a.raw_iter().next().is_some());
    let mut out = Marks { page, pages: count, marks: Vec::new(), left: Vec::new() };
    let Some(annots) = p.raw().get::<Array<'_>>(ANNOTS) else { return Ok(out) };
    // Stamps are counted by label, at the first of each.
    let mut counts: Vec<(String, usize)> = Vec::new();
    let (mut hidden, mut links, mut bare) = (0usize, 0usize, 0usize);
    for annot in annots.iter::<Dict<'_>>() {
        let sub = annot.get::<Name<'_>>("Subtype").map(|n| n.as_str().to_string()).unwrap_or_default();
        if matches!(sub.as_str(), "Popup" | "Link" | "Widget" | "Watermark" | "TrapNet" | "Screen" | "Movie" | "Sound" | "3D" | "RichMedia" | "") {
            if sub == "Link" {
                links += 1;
            }
            continue;
        }
        let flags = annot.get::<u32>("F").unwrap_or(0);
        if flags & (2 | 32) != 0 {
            hidden += 1;
            continue;
        }
        let contents = str_of(&annot, "Contents").unwrap_or_default();
        let subject = str_of(&annot, "Subj");
        let name = str_of(&annot, "NM");
        let rect = annot.get::<[f64; 4]>("Rect");
        let centre = rect.map(|r| place([(r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0])).unwrap_or([0.0, 0.0]);
        let user_points = |key: &str| -> Vec<[f64; 2]> {
            let v: Vec<f64> = annot.get::<Array<'_>>(key).map(|a| a.iter::<f64>().collect()).unwrap_or_default();
            v.chunks_exact(2).map(|c| [c[0], c[1]]).collect()
        };
        let measure = annot.get::<Dict<'_>>("Measure");
        let curved = matches!(sub.as_str(), "Circle" | "Ink") || (matches!(sub.as_str(), "Polygon" | "PolyLine") && annot.contains_key("Path"));
        let shape = matches!(sub.as_str(), "Line" | "PolyLine" | "Polygon");
        let mut mark = Mark { kind: Kind::Note, label: String::new(), name, says: None, worked: None, not_worked: None, scale: None, points: Vec::new(), places: vec![centre], text: contents.clone(), unread: None };
        let fig = figure(&contents);
        // A stamp is a symbol set on the sheet: counted by its label.
        if sub == "Stamp" {
            let label = subject.clone().or_else(|| Some(contents.clone()).filter(|c| !c.is_empty()));
            match label {
                None => bare += 1,
                Some(label) => match counts.iter_mut().find(|(l, _)| *l == label) {
                    Some((_, n)) => {
                        *n += 1;
                        if let Some(m) = out.marks.iter_mut().find(|m| m.kind == Kind::Count && m.label == label) {
                            m.places.push(centre);
                            m.says = Some(Says { value: *n as f64, unit: "EA", text: format!("{n} symbols") });
                        }
                    }
                    None => {
                        counts.push((label.clone(), 1));
                        mark.kind = Kind::Count;
                        mark.label = label;
                        mark.text = String::new();
                        mark.says = Some(Says { value: 1.0, unit: "EA", text: "1 symbol".into() });
                        out.marks.push(mark);
                    }
                },
            }
            continue;
        }
        if curved && (measure.is_some() || fig != Figure::None) {
            mark.kind = if sub == "Ink" { Kind::Length } else { Kind::Area };
            mark.label = subject.clone().unwrap_or_else(|| first_words(&contents, &fig, mark.kind));
            mark.unread = Some("its edge is a curve, and a curve is not read as a measure".into());
            out.marks.push(mark);
            continue;
        }
        if shape && (measure.is_some() || fig != Figure::None) {
            let pts = if sub == "Line" { annot.get::<[f64; 4]>("L").map(|l| vec![[l[0], l[1]], [l[2], l[3]]]).unwrap_or_default() } else { user_points("Vertices") };
            // What kind of mark it is: by its shape, and by the unit it says (a closed shape that says LF is a perimeter).
            let kind = match (sub.as_str(), &fig) {
                ("Line", _) => Kind::Length,
                ("PolyLine", _) => Kind::Run,
                ("Polygon", Figure::Said(s)) if s.unit == "LF" => Kind::Run,
                _ => Kind::Area,
            };
            mark.kind = kind;
            mark.label = subject.clone().unwrap_or_else(|| first_words(&contents, &fig, kind));
            let min = if kind == Kind::Area || sub == "Polygon" { 3 } else { 2 };
            mark.points = pts.iter().map(|q| place(*q)).collect();
            if pts.len() >= min {
                let xs: Vec<f64> = mark.points.iter().map(|q| q[0]).collect();
                let ys: Vec<f64> = mark.points.iter().map(|q| q[1]).collect();
                mark.places = vec![[(xs.iter().cloned().fold(f64::MAX, f64::min) + xs.iter().cloned().fold(f64::MIN, f64::max)) / 2.0, (ys.iter().cloned().fold(f64::MAX, f64::min) + ys.iter().cloned().fold(f64::MIN, f64::max)) / 2.0]];
            }
            let wanted = if kind == Kind::Area { "SF" } else { "LF" };
            // Why it is not read, first found.
            let mut unread: Option<String> = None;
            if pts.len() < min {
                unread = Some(format!("it gives {} point{}, too few for {}", pts.len(), if pts.len() == 1 { "" } else { "s" }, if kind == Kind::Area { "an area" } else { "a length" }));
            }
            let scale = measure.as_ref().map(scale_of);
            if unread.is_none() && scale.is_none() {
                unread = Some(if viewports { "its scale is set for a part of the sheet, which is not read".into() } else { "it has no scale".into() });
            }
            if unread.is_none() {
                match &fig {
                    Figure::Unheld { text, unit } => unread = Some(format!("it says {text:?}, a figure in {unit}, a unit this program does not hold")),
                    Figure::None => unread = Some("it says no figure".into()),
                    Figure::Said(s) if s.unit != wanted => unread = Some(format!("it says {}, {} on {}", s.text, if s.unit == "SF" { "an area" } else if s.unit == "LF" { "a length" } else { "a count" }, if kind == Kind::Area { "an area" } else if kind == Kind::Run { "a run of lengths" } else { "a length" })),
                    Figure::Said(s) if !(s.value > 0.0 && s.value.is_finite()) => unread = Some(format!("it says {}, which is no quantity", s.text)),
                    Figure::Said(_) => {}
                }
            }
            if let Some(s) = &scale {
                mark.scale = s.ratio.clone();
                match worked(s, &pts, sub == "Polygon", kind == Kind::Area) {
                    Ok(w) => mark.worked = Some(w),
                    Err(why) => mark.not_worked = Some(why),
                }
            }
            if let Figure::Said(s) = fig {
                mark.says = Some(s);
            }
            mark.unread = unread;
            out.marks.push(mark);
            continue;
        }
        if shape && viewports && measure.is_none() && !contents.is_empty() {
            // falls to a note below
        }
        // A note that measures nothing.
        if contents.is_empty() && subject.is_none() {
            bare += 1;
            continue;
        }
        mark.kind = Kind::Note;
        mark.label = subject.unwrap_or_else(|| contents.clone());
        out.marks.push(mark);
    }
    let some = |n: usize, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
    if hidden > 0 {
        out.left.push(format!("{} not on the sheet as shown", some(hidden, "hidden mark is", "hidden marks are")));
    }
    if bare > 0 {
        out.left.push(format!("{} with no words and no measure", some(bare, "drawn mark", "drawn marks")));
    }
    if links > 0 {
        out.left.push(format!("{} on the page", some(links, "link", "links")));
    }
    Ok(out)
}

/// A mark's label where it has no subject: the words of its text round the figure, or what kind of mark it is.
fn first_words(contents: &str, fig: &Figure, kind: Kind) -> String {
    let said = match fig {
        Figure::Said(s) => Some(s.text.as_str()),
        Figure::Unheld { text, .. } => Some(text.as_str()),
        Figure::None => None,
    };
    let rest = match said {
        Some(t) => contents.replacen(t, "", 1),
        None => contents.to_string(),
    };
    let rest = rest.split_whitespace().collect::<Vec<_>>().join(" ");
    let rest = rest.trim_matches(|c: char| c == ':' || c == '-' || c == '=' || c.is_whitespace()).to_string();
    if rest.is_empty() {
        let w = kind.said();
        let mut cs = w.chars();
        cs.next().map(|f| f.to_uppercase().collect::<String>() + cs.as_str()).unwrap_or_default()
    } else {
        rest
    }
}
