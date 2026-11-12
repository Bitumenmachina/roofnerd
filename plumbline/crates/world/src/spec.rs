//! A manufacturer's sheet read into what a thing is: its name, its maker, its model
//! number, how wide, deep and high it is, and what it weighs.
//!
//! Ours. BIMCraft has no reader of cut sheets (`grep -rni "spec sheet\|cut sheet\|
//! submittal" crates/` finds nothing). There is no standard for how a cut sheet is laid
//! out, which is why this reads only what a sheet says in so many words and makes
//! nothing up: a size is taken when the sheet writes it with a unit and says which way
//! it runs (36" W, Depth: 21 in), or writes three in a row the way sheets do (24 x 18 x
//! 30 in), and the line it came from is kept, as written, to be the value's source.
//! Everything read is Assumed until someone checks it against the thing or the sheet;
//! everything not found is left for the caller to mark Unknown and ask for.
//!
//! The words are the trade's: width, depth and height are a cut sheet's own; a sheet
//! that gives length and width (furniture) has its length taken as the width along
//! the wall and its width as the depth out from it, and the note says so.

/// Something read off the sheet: its value, the line it was read from as written, and
/// anything a reader should know about how it was read.
#[derive(Clone, Debug, PartialEq)]
pub struct Found<T> {
    pub value: T,
    pub line: String,
    pub note: Option<String>,
}

/// What a sheet says. Sizes are in inches and the weight in pounds; a size written in
/// millimetres is turned to inches and its note gives the millimetres as written.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sheet {
    pub name: Option<Found<String>>,
    pub maker: Option<Found<String>>,
    pub model: Option<Found<String>>,
    pub width: Option<Found<f64>>,
    pub depth: Option<Found<f64>>,
    pub height: Option<Found<f64>>,
    pub weight: Option<Found<f64>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Unit {
    In,
    Ft,
    Mm,
    Cm,
}

impl Unit {
    fn inches(self, x: f64) -> f64 {
        match self {
            Unit::In => x,
            Unit::Ft => x * 12.0,
            Unit::Mm => x / 25.4,
            Unit::Cm => x / 2.54,
        }
    }
}

/// Which way a size runs, as a sheet's letter or word says it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Way {
    W,
    D,
    H,
    L,
}

/// One length written in a line.
#[derive(Clone, Debug)]
struct Length {
    number: f64,
    unit: Option<Unit>,
    /// Where it starts and ends in the line's characters.
    span: (usize, usize),
    /// As written, for a note.
    written: String,
}

/// The line with the marks a sheet may use for inches, feet and "by" made plain.
fn plain(line: &str) -> Vec<char> {
    line.chars()
        .map(|c| match c {
            '\u{2033}' | '\u{201d}' | '\u{201c}' => '"',
            '\u{2032}' | '\u{2019}' | '\u{2018}' => '\'',
            '\u{00d7}' | '\u{2715}' => 'x',
            '\u{2013}' | '\u{2014}' => '-',
            other => other,
        })
        .collect()
}

fn word_at(c: &[char], at: usize, word: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    at + w.len() <= c.len() && c[at..at + w.len()].iter().zip(&w).all(|(a, b)| a.eq_ignore_ascii_case(b)) && !c.get(at + w.len()).is_some_and(|n| n.is_ascii_alphabetic())
}

/// A unit starting at `at`, and where it ends.
fn unit_at(c: &[char], at: usize) -> Option<(Unit, usize)> {
    if c.get(at) == Some(&'"') {
        return Some((Unit::In, at + 1));
    }
    if c.get(at) == Some(&'\'') {
        // Two ticks are inches.
        return Some(if c.get(at + 1) == Some(&'\'') { (Unit::In, at + 2) } else { (Unit::Ft, at + 1) });
    }
    for (word, unit) in [("inches", Unit::In), ("inch", Unit::In), ("in.", Unit::In), ("in", Unit::In), ("feet", Unit::Ft), ("foot", Unit::Ft), ("ft.", Unit::Ft), ("ft", Unit::Ft), ("mm", Unit::Mm), ("cm", Unit::Cm)] {
        let bare = word.trim_end_matches('.');
        if word_at(c, at, bare) {
            let end = at + bare.len();
            return Some((unit, if word.ends_with('.') && c.get(end) == Some(&'.') { end + 1 } else { end }));
        }
    }
    None
}

/// A whole number or a decimal starting at `at`, and where it ends.
fn digits_at(c: &[char], at: usize) -> Option<(f64, usize)> {
    let mut i = at;
    while c.get(i).is_some_and(|d| d.is_ascii_digit()) {
        i += 1;
    }
    if i == at {
        return None;
    }
    if c.get(i) == Some(&'.') && c.get(i + 1).is_some_and(|d| d.is_ascii_digit()) {
        i += 1;
        while c.get(i).is_some_and(|d| d.is_ascii_digit()) {
            i += 1;
        }
    }
    c[at..i].iter().collect::<String>().parse().ok().map(|x| (x, i))
}

/// Every length written in a line: 36", 34-1/2", 34 1/2 in, 3/4", 3'-6", 914 mm, and a
/// bare number (its unit to come from the one it is written "by").
fn lengths(c: &[char]) -> Vec<Length> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let starts = c[i].is_ascii_digit() && (i == 0 || !(c[i - 1].is_ascii_alphanumeric() || c[i - 1] == '.' || c[i - 1] == '/'))
            // "NV-3621", "AH-24": a number hyphened to letters is part of a name.
            && !(i >= 2 && c[i - 1] == '-' && c[i - 2].is_ascii_alphabetic());
        let Some((whole, mut end)) = starts.then(|| digits_at(c, i)).flatten() else {
            i += 1;
            continue;
        };
        let mut number = whole;
        // A fraction on its own (3/4), or after a space or a hyphen (34-1/2, 34 1/2).
        let fraction = |from: usize| -> Option<(f64, usize)> {
            let (top, slash) = digits_at(c, from)?;
            if c.get(slash) != Some(&'/') {
                return None;
            }
            let (bottom, end) = digits_at(c, slash + 1)?;
            (bottom > 0.0 && top < bottom).then_some((top / bottom, end))
        };
        if c.get(end) == Some(&'/') {
            if let Some((bottom, e)) = digits_at(c, end + 1) {
                if bottom > 0.0 {
                    (number, end) = (whole / bottom, e);
                }
            }
        } else if matches!(c.get(end), Some(' ') | Some('-')) {
            if let Some((part, e)) = fraction(end + 1) {
                (number, end) = (whole + part, e);
            }
        }
        // Its unit, with or without a space before it.
        let after = if c.get(end) == Some(&' ') { end + 1 } else { end };
        let mut unit = None;
        if let Some((u, e)) = unit_at(c, after) {
            (unit, end) = (Some(u), e);
            // Feet and then inches: 3'-6", 3' 6".
            if u == Unit::Ft {
                let next = if matches!(c.get(end), Some(' ') | Some('-')) { end + 1 } else { end };
                if let Some((inches, e)) = digits_at(c, next) {
                    let (inches, e) = match fraction(e + 1).filter(|_| matches!(c.get(e), Some(' ') | Some('-'))) {
                        Some((part, e2)) => (inches + part, e2),
                        None => (inches, e),
                    };
                    if let Some((Unit::In, e2)) = unit_at(c, if c.get(e) == Some(&' ') { e + 1 } else { e }) {
                        (number, unit, end) = (number * 12.0 + inches, Some(Unit::In), e2);
                    }
                }
            }
        }
        out.push(Length { number, unit, span: (i, end), written: c[i..end].iter().collect() });
        i = end.max(i + 1);
    }
    out
}

fn way_of(word: &str) -> Option<Way> {
    match word.trim_matches(|c: char| !c.is_ascii_alphabetic()).to_ascii_lowercase().as_str() {
        "w" | "wide" | "width" => Some(Way::W),
        "d" | "deep" | "depth" => Some(Way::D),
        "h" | "high" | "height" | "tall" | "ht" => Some(Way::H),
        "l" | "long" | "length" | "lg" => Some(Way::L),
        _ => None,
    }
}

/// The word that stands right after a place in the line, and the one right before.
fn word_after(c: &[char], at: usize) -> String {
    let mut i = at;
    while c.get(i).is_some_and(|x| *x == ' ' || *x == '(') {
        i += 1;
    }
    c[i.min(c.len())..].iter().take_while(|x| x.is_ascii_alphabetic()).collect()
}

fn word_before(c: &[char], at: usize) -> String {
    let mut i = at;
    while i > 0 && matches!(c[i - 1], ' ' | ':' | '=' | ')' | '.') {
        i -= 1;
    }
    let end = i;
    while i > 0 && c[i - 1].is_ascii_alphabetic() {
        i -= 1;
    }
    c[i..end].iter().collect()
}

/// Whether what stands between two lengths is only "x" or "by": the two are written
/// as one size.
fn by_between(c: &[char], from: usize, to: usize) -> bool {
    let between: String = c[from.min(to)..to].iter().collect::<String>().to_ascii_lowercase();
    let mut words = between.split(|ch: char| !ch.is_ascii_alphabetic()).filter(|w| !w.is_empty());
    let mut by = false;
    for w in &mut words {
        if w == "x" || w == "by" {
            by = true;
        } else if way_of(w).is_none() {
            return false;
        }
    }
    by
}

/// The sizes a line gives, each with the way it runs, in inches; and whether the ways
/// were written or taken from the order.
fn sizes_in(line: &str) -> (Vec<(Way, f64, String)>, bool) {
    let c = plain(line);
    let mut found = lengths(&c);
    // In "24 x 18 x 30 in" the unit is written once, at the end.
    for i in (0..found.len().saturating_sub(1)).rev() {
        if found[i].unit.is_none() && found[i + 1].unit.is_some() && by_between(&c, found[i].span.1, found[i + 1].span.0) {
            found[i].unit = found[i + 1].unit;
        }
    }
    // What a size not in inches was on the sheet, with its unit even where the sheet
    // wrote the unit once for three sizes.
    let said = |l: &Length, u: Unit| match u {
        Unit::In => l.written.clone(),
        Unit::Ft => format!("{} on the sheet", l.written.trim()),
        Unit::Mm => format!("{} mm on the sheet", l.number),
        Unit::Cm => format!("{} cm on the sheet", l.number),
    };
    // Labels stand before every length ("Width 21 in") or after every one ("21 in W"):
    // whichever the first length with a unit shows.
    let with_unit: Vec<&Length> = found.iter().filter(|l| l.unit.is_some()).collect();
    let before_first = with_unit.first().is_some_and(|l| way_of(&word_before(&c, l.span.0)).is_some());
    let mut out: Vec<(Way, f64, String)> = Vec::new();
    for l in &with_unit {
        let way = if before_first { way_of(&word_before(&c, l.span.0)) } else { way_of(&word_after(&c, l.span.1)) };
        if let (Some(way), Some(u)) = (way, l.unit) {
            if !out.iter().any(|(w, ..)| *w == way) {
                out.push((way, u.inches(l.number), said(l, u)));
            }
        }
    }
    if !out.is_empty() {
        return (out, true);
    }
    // No letters: three in a row, written "by" each other, are width, depth and height.
    for run in with_unit.windows(3) {
        if by_between(&c, run[0].span.1, run[1].span.0) && by_between(&c, run[1].span.1, run[2].span.0) {
            let ways = [Way::W, Way::D, Way::H];
            return (run.iter().zip(ways).map(|(l, w)| (w, l.unit.expect("kept with a unit").inches(l.number), said(l, l.unit.expect("kept with a unit")))).collect(), false);
        }
    }
    (Vec::new(), true)
}

/// What follows a label in a line: "Model: NV-3621" gives "NV-3621".
fn after_label(line: &str, labels: &[&str]) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    for label in labels {
        let Some(at) = lower.find(label) else { continue };
        // The label is a word of its own: "model", not "remodel".
        if at > 0 && lower.as_bytes()[at - 1].is_ascii_alphabetic() {
            continue;
        }
        let mut rest = line[at + label.len()..].trim_start();
        for lead in ["number", "Number", "no.", "No.", "NO.", "no", "No", "#", ":", "="] {
            if let Some(cut) = rest.strip_prefix(lead) {
                // "no" is a lead only as a word of its own ("Model No 12", not "Model Nova").
                if lead.chars().all(|ch| ch.is_ascii_alphabetic()) && cut.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                    continue;
                }
                rest = cut.trim_start();
            }
        }
        let rest = rest.trim_start_matches([':', '#', '=']).trim();
        // To the end of its column.
        let value = rest.split("   ").next().unwrap_or("").trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

const MODEL_LABELS: [&str; 9] = ["model", "catalog no", "catalog #", "cat. no", "cat no", "item no", "item #", "part no", "sku"];
const MAKER_LABELS: [&str; 5] = ["manufacturer", "manufactured by", "made by", "mfr.", "mfr"];

/// Read a sheet's lines, each with how tall its letters are (0 where that is not known).
pub fn read(lines: &[(String, f64)]) -> Sheet {
    let mut sheet = Sheet::default();
    let mut ways: Vec<(Way, f64, String, String, bool)> = Vec::new();
    let mut plain_lines: Vec<usize> = Vec::new();
    for (n, (line, _)) in lines.iter().enumerate() {
        let lower = line.to_ascii_lowercase();
        let mut used = false;
        if sheet.model.is_none() {
            if let Some(value) = after_label(line, &MODEL_LABELS) {
                sheet.model = Some(Found { value, line: line.clone(), note: None });
                continue;
            }
        }
        if sheet.maker.is_none() {
            if let Some(value) = after_label(line, &MAKER_LABELS) {
                sheet.maker = Some(Found { value, line: line.clone(), note: None });
                continue;
            }
        }
        if sheet.weight.is_none() && (lower.contains("weight") || lower.contains("wt.") || lower.contains(" wt ")) {
            let c = plain(line);
            for i in 0..c.len() {
                if !c[i].is_ascii_digit() || (i > 0 && (c[i - 1].is_ascii_alphanumeric() || c[i - 1] == '.')) {
                    continue;
                }
                let Some((x, end)) = digits_at(&c, i) else { continue };
                let at = if c.get(end) == Some(&' ') { end + 1 } else { end };
                let unit = [("lbs", 1.0), ("lb.", 1.0), ("lb", 1.0), ("pounds", 1.0), ("#", 1.0), ("kg", 2.204_622_621_8)].into_iter().find(|(w, _)| if *w == "#" { c.get(at) == Some(&'#') } else { word_at(&c, at, w.trim_end_matches('.')) });
                if let Some((word, k)) = unit {
                    let note = (word == "kg").then(|| format!("{x} kg on the sheet, at 2.2046 lb to the kilogram"));
                    sheet.weight = Some(Found { value: x * k, line: line.clone(), note });
                    used = true;
                    break;
                }
            }
        }
        let (sizes, written) = sizes_in(line);
        for (way, inches, said) in sizes {
            if !ways.iter().any(|(w, ..)| *w == way) {
                ways.push((way, inches, said, line.clone(), written));
                used = true;
            }
        }
        if !used {
            plain_lines.push(n);
        }
    }
    // Which of width, depth and height each is.
    let get = |way: Way| ways.iter().find(|(w, ..)| *w == way).cloned();
    let found = |got: Option<(Way, f64, String, String, bool)>, extra: Option<&str>| {
        got.map(|(_, inches, said, line, written)| {
            let mut notes: Vec<String> = Vec::new();
            if said.ends_with("on the sheet") {
                notes.push(said);
            }
            if !written {
                notes.push("the sheet does not say which size is which: read as width by depth by height, in the order written".into());
            }
            if let Some(extra) = extra {
                notes.push(extra.into());
            }
            Found { value: inches, line, note: (!notes.is_empty()).then(|| notes.join("; ")) }
        })
    };
    let (w, d, l) = (get(Way::W), get(Way::D), get(Way::L));
    match (w.is_some(), d.is_some(), l.is_some()) {
        // Length and width, as furniture is written: the length runs along the wall.
        (true, false, true) => {
            sheet.width = found(l, Some("the sheet's length, taken as the width along the wall"));
            sheet.depth = found(w, Some("the sheet's width, taken as the depth out from the wall"));
        }
        (false, _, true) => {
            sheet.width = found(l, Some("the sheet's length, taken as the width along the wall"));
            sheet.depth = found(d, None);
        }
        _ => {
            sheet.width = found(w, None);
            sheet.depth = found(d, None);
        }
    }
    sheet.height = found(get(Way::H), None);
    // The name: the tallest of the first lines that say nothing else; the first of them
    // when none is taller.
    let candidates: Vec<usize> = plain_lines.into_iter().filter(|&n| n < 12 && lines[n].0.chars().any(|c| c.is_alphabetic())).collect();
    let tallest = candidates.iter().map(|&n| lines[n].1).fold(0.0, f64::max);
    if let Some(&n) = candidates.iter().find(|&&n| lines[n].1 >= tallest - 0.25) {
        sheet.name = Some(Found { value: lines[n].0.trim().to_string(), line: lines[n].0.clone(), note: None });
    }
    sheet
}

/// What kind of thing a name says it is, and the word that says so: a vanity is a
/// furnishing, a lavatory a fixture, an air handler a piece of equipment. None when the
/// name does not say; then whoever brings the sheet in says which.
pub fn kind_said(name: &str) -> Option<(crate::Kind, &'static str)> {
    use crate::Kind;
    const WORDS: [(&str, Kind); 46] = [
        ("air handler", Kind::Equipment), ("furnace", Kind::Equipment), ("boiler", Kind::Equipment), ("water heater", Kind::Equipment), ("heat pump", Kind::Equipment),
        ("condenser", Kind::Equipment), ("condensing unit", Kind::Equipment), ("pump", Kind::Equipment), ("fan", Kind::Equipment), ("panelboard", Kind::Equipment),
        ("load center", Kind::Equipment), ("range", Kind::Equipment), ("refrigerator", Kind::Equipment), ("dishwasher", Kind::Equipment), ("washer", Kind::Equipment),
        ("dryer", Kind::Equipment), ("heater", Kind::Equipment), ("ventilator", Kind::Equipment),
        ("lavatory", Kind::Fixture), ("water closet", Kind::Fixture), ("toilet", Kind::Fixture), ("sink", Kind::Fixture), ("bathtub", Kind::Fixture), ("tub", Kind::Fixture),
        ("shower", Kind::Fixture), ("faucet", Kind::Fixture), ("register", Kind::Fixture), ("grille", Kind::Fixture), ("diffuser", Kind::Fixture), ("receptacle", Kind::Fixture),
        ("luminaire", Kind::Fixture), ("light", Kind::Fixture),
        ("vanity", Kind::Furnishing), ("cabinet", Kind::Furnishing), ("casework", Kind::Furnishing), ("bookcase", Kind::Furnishing), ("shelving", Kind::Furnishing),
        ("shelf", Kind::Furnishing), ("table", Kind::Furnishing), ("desk", Kind::Furnishing), ("chair", Kind::Furnishing), ("sofa", Kind::Furnishing), ("bed", Kind::Furnishing),
        ("bench", Kind::Furnishing), ("wardrobe", Kind::Furnishing), ("locker", Kind::Furnishing),
    ];
    let lower = name.to_ascii_lowercase();
    let words: Vec<&str> = lower.split(|c: char| !c.is_ascii_alphabetic()).filter(|w| !w.is_empty()).collect();
    // The earliest word in the name wins: a "vanity sink top" is a vanity.
    let mut best: Option<(usize, crate::Kind, &'static str)> = None;
    for (word, kind) in WORDS {
        let parts: Vec<&str> = word.split(' ').collect();
        if let Some(at) = words.windows(parts.len()).position(|w| w == parts.as_slice()) {
            if best.is_none_or(|(was, ..)| at < was) {
                best = Some((at, kind, word));
            }
        }
    }
    best.map(|(_, kind, word)| (kind, word))
}

/// An item made from a sheet, and an account of the making: what was read and from
/// which line, and what the sheet did not say.
pub struct Made {
    pub name: String,
    pub item: crate::Item,
    /// What was read: what it is, its value as read, the line it was read from, a note.
    pub read: Vec<(&'static str, String, String, Option<String>)>,
    /// What was not on the sheet, and is Unknown until someone enters it.
    pub missing: Vec<&'static str>,
}

/// The item a sheet describes. `called` is what the sheet is called (its file's name)
/// and `page` the page read: together with the line as written they are each value's
/// source. Everything read is Assumed: a sheet says what the maker meant to make, not
/// what is on the truck.
pub fn item_of(sheet: &Sheet, called: &str, page: usize, category: crate::Kind) -> Made {
    use crate::units::Len;
    use crate::value::{Status, Val};
    let from = |line: &str, note: &Option<String>| format!("{called}, page {page}: \u{201c}{line}\u{201d}{}", note.as_ref().map(|n| format!(" ({n})")).unwrap_or_default());
    let not_found = |what: &str| format!("{what} is not on {called}, page {page}");
    let mut read = Vec::new();
    let mut missing = Vec::new();
    let mut words = |what: &'static str, found: &Option<Found<String>>| match found {
        Some(f) => {
            read.push((what, f.value.clone(), f.line.clone(), f.note.clone()));
            Val { value: Some(f.value.clone()), unit: String::new(), status: Status::Assumed, source: from(&f.line, &f.note) }
        }
        None => {
            missing.push(what);
            Val::unknown("", &not_found(what))
        }
    };
    let maker = words("maker", &sheet.maker);
    let model = words("model", &sheet.model);
    let mut length = |what: &'static str, found: &Option<Found<f64>>| match found {
        Some(f) => {
            // Shown to the ten-thousandth of an inch; held as worked.
            read.push((what, crate::units::show_length(&Len::One((f.value * 1.0e4).round() / 1.0e4), "in"), f.line.clone(), f.note.clone()));
            Val { value: Some(Len::One(f.value)), unit: "in".to_string(), status: Status::Assumed, source: from(&f.line, &f.note) }
        }
        None => {
            missing.push(what);
            Val::unknown("in", &not_found(what))
        }
    };
    let (width, depth, height) = (length("width", &sheet.width), length("depth", &sheet.depth), length("height", &sheet.height));
    let weight = match &sheet.weight {
        Some(f) => {
            read.push(("weight", format!("{} lb", crate::view::num((f.value * 100.0).round() / 100.0)), f.line.clone(), f.note.clone()));
            Some(Val { value: Some(f.value), unit: "lb".to_string(), status: Status::Assumed, source: from(&f.line, &f.note) })
        }
        None => {
            missing.push("weight");
            Some(Val::unknown("lb", &not_found("weight")))
        }
    };
    let name = sheet.name.as_ref().map(|f| f.value.clone()).unwrap_or_else(|| called.to_string());
    Made { name, item: crate::Item { category, maker, model, width, depth, height, weight, sheet: format!("{called}, page {page}") }, read, missing }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(lines: &[(&str, f64)]) -> Sheet {
        read(&lines.iter().map(|(t, s)| (t.to_string(), *s)).collect::<Vec<_>>())
    }

    fn size(s: &Sheet) -> (Option<f64>, Option<f64>, Option<f64>) {
        let r = |f: &Option<Found<f64>>| f.as_ref().map(|f| (f.value * 1.0e4).round() / 1.0e4);
        (r(&s.width), r(&s.depth), r(&s.height))
    }

    /// Every sheet here is made up.
    #[test]
    fn a_vanity_with_its_sizes_on_one_line() {
        let s = sheet(&[("Harbor Vanity 36", 24.0), ("Manufacturer: Northfield Millwork", 12.0), ("Model: NV-3621", 11.0), ("Overall dimensions: 36\" W x 21\" D x 34-1/2\" H", 11.0), ("Shipping weight: 96 lb", 11.0)]);
        assert_eq!(s.name.as_ref().map(|f| f.value.as_str()), Some("Harbor Vanity 36"));
        assert_eq!(s.maker.as_ref().map(|f| f.value.as_str()), Some("Northfield Millwork"));
        assert_eq!(s.model.as_ref().map(|f| f.value.as_str()), Some("NV-3621"));
        assert_eq!(size(&s), (Some(36.0), Some(21.0), Some(34.5)));
        assert_eq!(s.weight.as_ref().map(|f| f.value), Some(96.0));
        // Each keeps the line it was read from, as written, and nothing was guessed.
        assert_eq!(s.height.as_ref().unwrap().line, "Overall dimensions: 36\" W x 21\" D x 34-1/2\" H");
        assert_eq!(s.height.as_ref().unwrap().note, None);
    }

    #[test]
    fn sizes_on_lines_of_their_own_with_the_word_first() {
        let s = sheet(&[("AH-24 Air Handler", 20.0), ("Model No. AH-24-B", 10.0), ("Width 21 in", 10.0), ("Depth 24 in", 10.0), ("Height 49 in", 10.0), ("Weight 135 lbs", 10.0), ("Electrical: 120 V, 15 A", 10.0)]);
        assert_eq!(s.name.as_ref().map(|f| f.value.as_str()), Some("AH-24 Air Handler"));
        assert_eq!(s.model.as_ref().map(|f| f.value.as_str()), Some("AH-24-B"));
        assert_eq!(size(&s), (Some(21.0), Some(24.0), Some(49.0)));
        assert_eq!(s.weight.as_ref().map(|f| f.value), Some(135.0));
        assert_eq!(s.maker, None);
        // The words first, all on one line.
        let one = sheet(&[("Cabinet", 0.0), ("Width 21 in  Depth 24 in  Height 49 in", 0.0)]);
        assert_eq!(size(&one), (Some(21.0), Some(24.0), Some(49.0)));
    }

    #[test]
    fn what_a_sheet_does_not_say_is_not_made_up() {
        // No height anywhere.
        let s = sheet(&[("Bookcase BK-30", 18.0), ("30 in wide, 12 in deep", 10.0)]);
        assert_eq!(size(&s), (Some(30.0), Some(12.0), None));
        assert_eq!((s.model.clone(), s.maker.clone(), s.weight.clone()), (None, None, None));
        // Two sizes with no letters could be width and depth or width and height: not taken.
        assert_eq!(size(&sheet(&[("Shelf", 0.0), ("30 x 12 in", 0.0)])), (None, None, None));
        // A number with no unit is not a size, and a model number is not one either.
        assert_eq!(size(&sheet(&[("Unit 36", 0.0), ("Series 2400, 3 speeds", 0.0), ("Model: X-36-21-34", 0.0)])), (None, None, None));
    }

    #[test]
    fn three_sizes_with_no_letters_are_read_in_the_order_written_and_say_so() {
        let s = sheet(&[("Storage cube", 0.0), ("24 x 18 x 30 in", 0.0)]);
        assert_eq!(size(&s), (Some(24.0), Some(18.0), Some(30.0)));
        assert!(s.width.as_ref().unwrap().note.as_deref().unwrap().contains("in the order written"));
        let by = sheet(&[("Storage cube", 0.0), ("24\" by 18\" by 30\"", 0.0)]);
        assert_eq!(size(&by), (Some(24.0), Some(18.0), Some(30.0)));
    }

    #[test]
    fn metric_is_turned_to_inches_and_the_millimetres_are_kept_in_the_note() {
        // 600 / 25.4 = 23.6220, 450 / 25.4 = 17.7165, 850 / 25.4 = 33.4646.
        let s = sheet(&[("Base unit", 0.0), ("600 x 450 x 850 mm", 0.0), ("Weight: 40 kg", 0.0)]);
        assert_eq!(size(&s), (Some(23.622), Some(17.7165), Some(33.4646)));
        assert!(s.width.as_ref().unwrap().note.as_deref().unwrap().contains("600 mm on the sheet"), "{:?}", s.width);
        // 40 kg at 2.2046 lb each: 88.18 lb.
        assert!((s.weight.as_ref().unwrap().value - 88.1849).abs() < 1.0e-3);
        // Inches first with millimetres after in brackets: the inches are taken.
        let both = sheet(&[("Base unit", 0.0), ("24\" W x 18\" D x 34\" H (610 x 457 x 864 mm)", 0.0)]);
        assert_eq!(size(&both), (Some(24.0), Some(18.0), Some(34.0)));
        assert_eq!(both.width.as_ref().unwrap().note, None);
    }

    #[test]
    fn fractions_feet_and_inches_and_a_sheets_own_marks() {
        let s = sheet(&[("Tub", 0.0), ("5'-0\" L x 2'-8\" W x 1' 7 1/2\" H", 0.0)]);
        // Length along the wall, width out from it: 60 by 32 by 19.5 in.
        assert_eq!(size(&s), (Some(60.0), Some(32.0), Some(19.5)));
        assert!(s.width.as_ref().unwrap().note.as_deref().unwrap().contains("length, taken as the width"));
        let marks = sheet(&[("Sink", 0.0), ("33\u{2033} W \u{00d7} 22\u{2033} D \u{00d7} 9 3/4\u{2033} H", 0.0)]);
        assert_eq!(size(&marks), (Some(33.0), Some(22.0), Some(9.75)));
        let small = sheet(&[("Bracket", 0.0), ("3/4 in W x 1.5 in D x 6 in H", 0.0)]);
        assert_eq!(size(&small), (Some(0.75), Some(1.5), Some(6.0)));
    }

    #[test]
    fn the_name_is_the_tallest_line_that_says_nothing_else() {
        let s = sheet(&[("SUBMITTAL DATA", 9.0), ("Model: Q-7", 9.0), ("Quiet Fan 80", 22.0), ("Width 9 in", 9.0)]);
        assert_eq!(s.name.as_ref().map(|f| f.value.as_str()), Some("Quiet Fan 80"));
        // With no sizes of letters to go by, the first line.
        let flat = sheet(&[("Quiet Fan 80", 0.0), ("Ceiling exhaust fan", 0.0)]);
        assert_eq!(flat.name.as_ref().map(|f| f.value.as_str()), Some("Quiet Fan 80"));
        assert_eq!(sheet(&[]).name, None);
    }

    #[test]
    fn a_name_says_what_kind_of_thing_it_is_or_does_not() {
        use crate::Kind;
        assert_eq!(kind_said("Harbor Vanity 36"), Some((Kind::Furnishing, "vanity")));
        assert_eq!(kind_said("AH-24 Air Handler"), Some((Kind::Equipment, "air handler")));
        assert_eq!(kind_said("Wall-hung lavatory"), Some((Kind::Fixture, "lavatory")));
        // The earliest word wins; a word inside another is not it.
        assert_eq!(kind_said("Vanity with sink top"), Some((Kind::Furnishing, "vanity")));
        assert_eq!(kind_said("Stable door"), None);
        assert_eq!(kind_said("Model 4400"), None);
    }
}
