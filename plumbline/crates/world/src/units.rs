//! Units. US measurements, entered the way the drawing or the tape gives them.
//!
//! A length is held exactly as it was entered, in one of three units:
//!
//! * `ft`: decimal feet. `25.5 ft`.
//! * `in`: inches. `306 in`, `19 1/2 in`.
//! * `ft-in`: feet and inches. `25' 6 1/2"`, held as the pair `[25, 6.5]`.
//!
//! Nothing is converted when a value is entered, shown or saved. A value is converted to
//! feet only inside arithmetic, and the derived figure's formula says so (`÷ 12`).
//! Halves down to sixty-fourths of an inch are exact in a 64-bit number, so `25' 6 1/2"`
//! and `306.5 in` are the same length to the last bit.
//!
//! A unit this program does not know is refused by name. It is never read as feet, and
//! neither is a number with no unit. Metric is not taken (PROPOSALS.md).

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const FT: &str = "ft";
pub const IN: &str = "in";
pub const FT_IN: &str = "ft-in";
/// Pitch: inches of rise for every 12 inches of run.
pub const PITCH: &str = "in/12";

/// The number part of an entered length. Which it is goes with the unit beside it:
/// one number for `ft` or `in`, a pair for `ft-in`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    /// A plain number of feet or of inches.
    One(f64),
    /// `[whole feet, inches]`. Both carry the sign of the whole length.
    FtIn([f64; 2]),
}

impl Serialize for Len {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Len::One(x) => x.serialize(s),
            Len::FtIn(pair) => pair.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for Len {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Len;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a length: one number, or [feet, inches]")
            }
            fn visit_f64<E>(self, v: f64) -> Result<Len, E> {
                Ok(Len::One(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Len, E> {
                Ok(Len::One(v as f64))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Len, E> {
                Ok(Len::One(v as f64))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Len, A::Error> {
                let wrong = || de::Error::custom("a feet-and-inches length is exactly two numbers: [feet, inches]");
                let feet: f64 = seq.next_element()?.ok_or_else(wrong)?;
                let inches: f64 = seq.next_element()?.ok_or_else(wrong)?;
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(wrong());
                }
                Ok(Len::FtIn([feet, inches]))
            }
        }
        d.deserialize_any(V)
    }
}

/// Metric units by the names people give them. Named so the refusal can say what it is.
const METRIC: [&str; 20] = [
    "m", "mm", "cm", "dm", "km", "meter", "meters", "metre", "metres", "millimeter", "millimeters", "millimetre", "millimetres", "centimeter", "centimeters", "centimetre", "centimetres", "sqm", "m2", "m²",
];

/// The sentence refusing a unit a length cannot be in. `taken` is what the field takes.
pub fn refuse_length_unit(unit: &str, taken: &str) -> String {
    let u = unit.trim();
    if u.is_empty() {
        return format!("has no unit; say which it is ({taken}). A number with no unit is never read as feet");
    }
    if METRIC.contains(&u.to_lowercase().as_str()) {
        return format!("is in {unit:?}, which is metric; this program takes US measurements only ({taken}) and does not convert metric");
    }
    format!("is in {unit:?}, which is not a unit this program knows; it takes {taken} and reads nothing else as feet")
}

const SCALAR_UNITS: &str = "\"ft\", \"in\" or \"ft-in\"";
const COORD_UNITS: &str = "\"ft\" or \"in\"";

impl Len {
    /// This length in feet, for arithmetic. Fails with the reason if the unit is not one
    /// this program knows, or the number does not suit the unit.
    pub fn feet(&self, unit: &str) -> Result<f64, String> {
        match (unit, self) {
            (FT, Len::One(x)) => Ok(*x),
            (IN, Len::One(x)) => Ok(*x / 12.0),
            (FT_IN, Len::FtIn([feet, inches])) => {
                if !feet.is_finite() || !inches.is_finite() {
                    return Err("is not a usable number".into());
                }
                if feet.fract() != 0.0 {
                    return Err(format!("has {feet} for its feet; in a feet-and-inches length the feet are a whole number and the rest goes in the inches"));
                }
                if inches.abs() >= 12.0 {
                    return Err(format!("has {inches} for its inches; in a feet-and-inches length the inches are less than 12"));
                }
                if (*feet > 0.0 && *inches < 0.0) || (*feet < 0.0 && *inches > 0.0) {
                    return Err("has feet and inches of opposite signs; a length below zero carries the minus on both".into());
                }
                // Through inches, so 25' 6 1/2" and 306.5 in are the same number of feet.
                Ok((*feet * 12.0 + *inches) / 12.0)
            }
            (FT | IN, Len::FtIn(_)) => Err(format!("is a [feet, inches] pair but its unit is {unit:?}; the unit for a pair is \"ft-in\"")),
            (FT_IN, Len::One(_)) => Err("has the unit \"ft-in\" but is a single number; write it as [feet, inches]".into()),
            _ => Err(refuse_length_unit(unit, SCALAR_UNITS)),
        }
    }
}

/// Is this a unit a single length can be entered in? Checked even when the value itself
/// is Unknown, so the unit waiting for it is a real one.
pub fn scalar_unit(unit: &str) -> Result<(), String> {
    match unit {
        FT | IN | FT_IN => Ok(()),
        _ => Err(refuse_length_unit(unit, SCALAR_UNITS)),
    }
}

/// What plan coordinates in this unit are divided by to give feet. Coordinates are plain
/// numbers in feet or in inches; feet-and-inches is for a single dimension.
pub fn coord_divisor(unit: &str) -> Result<f64, String> {
    match unit {
        FT => Ok(1.0),
        IN => Ok(12.0),
        FT_IN => Err("is in \"ft-in\"; plan coordinates are plain numbers in \"ft\" or \"in\"".into()),
        _ => Err(refuse_length_unit(unit, COORD_UNITS)),
    }
}

/// How a derived figure's formula names a length it converts: nothing for feet, `÷ 12`
/// for inches, the sum for feet and inches.
pub fn to_feet_text(name: &str, unit: &str) -> String {
    match unit {
        IN => format!("{name} ÷ 12"),
        FT_IN => format!("{name} (feet + inches ÷ 12)"),
        _ => name.to_string(),
    }
}

// ---- Showing ----

/// Inches as a tape reads them: a whole number and a fraction when the figure is an exact
/// sixty-fourth, a decimal otherwise. No sign.
fn inches_text(x: f64) -> String {
    let whole = x.floor();
    let sixty_fourths = (x - whole) * 64.0;
    if sixty_fourths == 0.0 || sixty_fourths.fract() != 0.0 || whole >= 1.0e15 {
        return format!("{x}");
    }
    let (mut n, mut d) = (sixty_fourths as u32, 64u32);
    while n % 2 == 0 {
        n /= 2;
        d /= 2;
    }
    if whole == 0.0 {
        format!("{n}/{d}")
    } else {
        format!("{whole} {n}/{d}")
    }
}

/// An entered length as text, in the unit it was entered in.
pub fn show_length(len: &Len, unit: &str) -> String {
    match (unit, len) {
        (IN, Len::One(x)) => format!("{}{} in", if *x < 0.0 { "-" } else { "" }, inches_text(x.abs())),
        (FT_IN, Len::FtIn([feet, inches])) => {
            let minus = if *feet < 0.0 || *inches < 0.0 { "-" } else { "" };
            // 25' 0 1/2", not 25' 1/2": after feet, the whole inches are written even when none.
            let inch = inches_text(inches.abs());
            let inch = if inches.abs() > 0.0 && inches.abs() < 1.0 && inch.contains('/') { format!("0 {inch}") } else { inch };
            format!("{minus}{}' {inch}\"", feet.abs())
        }
        // Adding zero shows a "-0" from a file as plain 0.
        (_, Len::One(x)) => format!("{} {unit}", *x + 0.0).trim().to_string(),
        (_, Len::FtIn([feet, inches])) => format!("[{feet}, {inches}] {unit}").trim().to_string(),
    }
}

// ---- Reading what an estimator types ----

/// Marks and fractions as they arrive pasted from a drawing, turned into the plain ones.
fn plain(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().chars() {
        match c {
            '\u{2032}' | '\u{2019}' | '\u{2018}' => out.push('\''),
            '\u{2033}' | '\u{201D}' | '\u{201C}' => out.push('"'),
            '\u{00BD}' => out.push_str(" 1/2"),
            '\u{00BC}' => out.push_str(" 1/4"),
            '\u{00BE}' => out.push_str(" 3/4"),
            '\u{215B}' => out.push_str(" 1/8"),
            '\u{215C}' => out.push_str(" 3/8"),
            '\u{215D}' => out.push_str(" 5/8"),
            '\u{215E}' => out.push_str(" 7/8"),
            '\u{2212}' | '\u{2013}' => out.push('-'),
            c => out.extend(c.to_lowercase()),
        }
    }
    // Two apostrophes are an inch mark typed the long way.
    out.replace("''", "\"").trim().to_string()
}

struct Scan<'a> {
    s: &'a [u8],
    at: usize,
}

impl<'a> Scan<'a> {
    fn new(s: &'a str) -> Scan<'a> {
        Scan { s: s.as_bytes(), at: 0 }
    }
    fn peek(&self) -> Option<u8> {
        self.s.get(self.at).copied()
    }
    fn done(&self) -> bool {
        self.at >= self.s.len()
    }
    fn spaces(&mut self) {
        while self.peek() == Some(b' ') {
            self.at += 1;
        }
    }
    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn digits(&mut self) -> &'a str {
        let start = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.at += 1;
        }
        std::str::from_utf8(&self.s[start..self.at]).unwrap_or("")
    }
    fn letters(&mut self) -> &'a str {
        let start = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_alphabetic() || c == b'.') {
            self.at += 1;
        }
        std::str::from_utf8(&self.s[start..self.at]).unwrap_or("")
    }

    /// `n/d` where d is 2, 4, 8, 16, 32 or 64 and the fraction is less than one.
    /// Call with the scanner on the numerator's first digit.
    fn fraction(&mut self, numerator: &str) -> Result<f64, String> {
        let d = self.digits();
        let (n, d): (u32, u32) = match (numerator.parse(), d.parse()) {
            (Ok(n), Ok(d)) => (n, d),
            _ => return Err("has a fraction that is not two whole numbers".into()),
        };
        if ![2, 4, 8, 16, 32, 64].contains(&d) {
            return Err(format!("has the fraction {n}/{d}; a fraction is in halves, quarters, eighths, sixteenths, thirty-seconds or sixty-fourths. Write anything else as a decimal"));
        }
        if n == 0 || n >= d {
            return Err(format!("has the fraction {n}/{d}; a fraction is more than nothing and less than one"));
        }
        Ok(f64::from(n) / f64::from(d))
    }

    /// A plain number as an estimator writes one: `25`, `25.5`, `.5`, `1/2`, `6 1/2`,
    /// `6-1/2`. No exponents, no thousands commas. Returns the number and whether it
    /// was written with a decimal point.
    fn quantity(&mut self) -> Result<(f64, bool), String> {
        let start = self.at;
        let whole = self.digits();
        if self.peek() == Some(b'/') && !whole.is_empty() {
            self.at += 1;
            return Ok((self.fraction(whole)?, false));
        }
        let mut decimal = false;
        if self.peek() == Some(b'.') {
            self.at += 1;
            decimal = true;
            let after = self.digits();
            if whole.is_empty() && after.is_empty() {
                return Err("is not a number".into());
            }
        } else if whole.is_empty() {
            return Err("is not a number".into());
        }
        let text = std::str::from_utf8(&self.s[start..self.at]).unwrap_or("");
        let mut x: f64 = text.parse().map_err(|_| "is not a number".to_string())?;
        if !x.is_finite() {
            return Err("is not a usable number".into());
        }
        // Written as something, held as nothing: too small for a 64-bit number.
        if x == 0.0 && text.bytes().any(|c| (b'1'..=b'9').contains(&c)) {
            return Err("is too small a number to hold; it would be stored as zero".into());
        }
        // A fraction after a whole number: "6 1/2" or "6-1/2".
        let save = self.at;
        if self.eat(b' ') || self.eat(b'-') {
            self.spaces();
            let n = self.digits();
            if !n.is_empty() && self.peek() == Some(b'/') {
                if decimal {
                    return Err("has both a decimal point and a fraction; use one or the other".into());
                }
                self.at += 1;
                x += self.fraction(n)?;
                return Ok((x, false));
            }
            self.at = save;
        }
        Ok((x, decimal))
    }
}

#[derive(PartialEq)]
enum Mark {
    Feet,
    Inches,
    None,
}

fn mark(scan: &mut Scan, whole_text: &str) -> Result<Mark, String> {
    scan.spaces();
    if scan.eat(b'\'') {
        return Ok(Mark::Feet);
    }
    if scan.eat(b'"') {
        return Ok(Mark::Inches);
    }
    let word = scan.letters();
    match word.trim_end_matches('.') {
        "" => Ok(Mark::None),
        "ft" | "foot" | "feet" => Ok(Mark::Feet),
        "in" | "inch" | "inches" => Ok(Mark::Inches),
        other => Err(format!("{whole_text:?} {}", refuse_length_unit(other, "feet (25.5 ft), inches (306 in) or feet and inches (25' 6 1/2\")"))),
    }
}

/// Read a length as typed: `25.5 ft`, `306 in`, `19 1/2"`, `25' 6"`, `25'-6 1/2"`,
/// `25 ft 6 in`. Gives the number and the unit it was entered in. A bare number is
/// refused: it is never read as feet.
pub fn parse_length(text: &str) -> Result<(Len, &'static str), String> {
    let typed = text.trim();
    let s = plain(text);
    if !s.is_ascii() {
        return Err(format!("{typed:?} has a character that is not part of a measurement"));
    }
    let fail = |why: String| format!("{typed:?} {why}");
    let mut scan = Scan::new(&s);
    let negative = scan.eat(b'-');
    if !negative {
        scan.eat(b'+');
    }
    scan.spaces();
    let sign = if negative { -1.0 } else { 1.0 };
    let (first, first_decimal) = scan.quantity().map_err(&fail)?;
    match mark(&mut scan, typed)? {
        Mark::None => {
            if scan.done() {
                Err(fail("has no unit; say which it is: 25.5 ft, 306 in, or 25' 6\". A number with no unit is never read as feet".into()))
            } else {
                Err(fail("is not a measurement this program can read; write it like 25.5 ft, 306 in, or 25' 6 1/2\", with no commas".into()))
            }
        }
        Mark::Inches => {
            scan.spaces();
            if !scan.done() {
                return Err(fail("has something after the inches".into()));
            }
            Ok((Len::One(sign * first + 0.0), IN))
        }
        Mark::Feet => {
            scan.spaces();
            if scan.done() {
                return Ok((Len::One(sign * first + 0.0), FT));
            }
            // 25'-6" and 25' - 6": the dash between feet and inches is a separator, not a
            // minus. A dash hard against the inches with a space before it (25' -6") reads
            // as a minus on the inches, which means nothing; it is refused, not dropped.
            let spaced = scan.at > 0 && scan.s[scan.at - 1] == b' ';
            if scan.eat(b'-') {
                if spaced && scan.peek() != Some(b' ') {
                    return Err(fail("has a minus on the inches; a length below zero carries its minus in front (-2' 6\"), and the dash in 25'-6\" goes hard against the feet mark".into()));
                }
                scan.spaces();
            }
            let (inches, _) = scan.quantity().map_err(&fail)?;
            match mark(&mut scan, typed)? {
                Mark::Feet => return Err(fail("gives feet twice".into())),
                Mark::Inches | Mark::None => {}
            }
            scan.spaces();
            if !scan.done() {
                return Err(fail("has something after the inches".into()));
            }
            if first_decimal || first.fract() != 0.0 {
                return Err(fail("has a part of a foot and inches as well; write the feet as a whole number, or the whole length as decimal feet".into()));
            }
            if inches >= 12.0 {
                return Err(fail(format!("has {inches} inches after the feet; the inches of a feet-and-inches length are less than 12")));
            }
            Ok((Len::FtIn([sign * first + 0.0, sign * inches + 0.0]), FT_IN))
        }
    }
}

/// Read a pitch as typed: rise in inches for 12 of run. `6`, `6:12`, `6/12`, `6 in/12`,
/// `1/4:12`, `1/4 in/ft`. Anything else (degrees, percent, a run other than 12, a bare
/// fraction such as `3/4`) is refused.
pub fn parse_pitch(text: &str) -> Result<f64, String> {
    let typed = text.trim();
    let s: String = plain(text).replace('"', " in ").replace("in.", "in");
    // One space between words, none around the marks that join a rise to its run.
    let squeezed: String = s.split_whitespace().collect::<Vec<_>>().join(" ").replace(" /", "/").replace("/ ", "/").replace(" :", ":").replace(": ", ":");
    let how = "a pitch is the rise in inches for 12 of run: write 6, 6:12 or 1/4:12";
    if squeezed.contains("deg") || squeezed.contains('°') {
        return Err(format!("{typed:?} is in degrees; {how}"));
    }
    if squeezed.contains('%') || squeezed.contains("percent") {
        return Err(format!("{typed:?} is a percentage; {how}"));
    }
    let mut body = squeezed.as_str();
    let mut run_given = false;
    if !squeezed.is_ascii() {
        return Err(format!("{typed:?} has a character that is not part of a pitch; {how}"));
    }
    // The most specific ending first, so "6in/12" is not read as "6in" over 12.
    for suffix in [" in per ft", " in per foot", " in/ft", "in/ft", " in/12", "in/12", " in:12", "in:12", " in 12", ":12", "/12"] {
        if let Some(rest) = body.strip_suffix(suffix) {
            body = rest.trim_end();
            run_given = true;
            break;
        }
    }
    // A bare fraction is two things to two people: 3/4 is a rise of 3/4 in 12 to one and
    // 3 in 4 (9:12) to another, and "1/2 pitch" once meant 12:12. Only a/12 is read
    // without its run, as a in 12. Any other fraction has to say its run.
    let bare = body.trim_start_matches(['-', '+']);
    if !run_given && bare.contains('/') && !bare.contains(' ') && !bare.contains('-') {
        return Err(format!("{typed:?} could be a rise of {bare} in 12 or a rise of {} in {}; write the rise with its run, like {bare}:12; {how}", bare.split('/').next().unwrap_or(""), bare.split('/').nth(1).unwrap_or("")));
    }
    let mut scan = Scan::new(body);
    let negative = scan.eat(b'-');
    if !negative {
        scan.eat(b'+');
    }
    let (x, _) = scan.quantity().map_err(|why| format!("{typed:?} {why}; {how}"))?;
    if !scan.done() {
        return Err(format!("{typed:?} is not a pitch this program can read; {how}"));
    }
    let rise = if negative { -x } else { x };
    Ok(rise + 0.0)
}

/// Read a plain number as typed: `2`, `10`, `1.25`, `1/2`. No unit, no exponent, no comma.
/// A price as it is typed: `4.25`, `$4.25`, `$4.25/SF`, `4.25 $/SF`. `per` is the unit the
/// price is for (`$/SF`); a unit that is typed has to be that one. Thousands may be
/// written with commas.
pub fn parse_price(text: &str, per: &str) -> Result<f64, String> {
    let t = text.trim();
    let wanted = per.trim_start_matches('$').trim_start_matches('/');
    let (number, unit) = match t.find('/') {
        Some(i) => (t[..i].trim(), Some(t[i + 1..].trim())),
        None => (t, None),
    };
    let number = number.trim_end_matches('$').trim().trim_start_matches('$').trim();
    if let Some(unit) = unit {
        if unit != wanted {
            return Err(format!("{text:?} is a price per {unit}, and this is priced per {wanted}; a price does not carry from one unit to another"));
        }
    }
    // A comma is a thousands mark and nothing else: 1,250 and 12,500.75, never 1,25
    // (which somewhere means one and a quarter, and would be read a hundred times out).
    if number.contains(',') {
        let whole = number.split('.').next().unwrap_or("");
        let groups: Vec<&str> = whole.split(',').collect();
        let digits = |g: &str| !g.is_empty() && g.chars().all(|c| c.is_ascii_digit());
        let ok = number.matches('.').count() <= 1 && !number.split('.').nth(1).is_some_and(|f| f.contains(',')) && groups.len() > 1 && digits(groups[0]) && groups[0].len() <= 3 && groups[1..].iter().all(|g| g.len() == 3 && digits(g));
        if !ok {
            return Err(format!("{text:?} has a comma that is not a thousands mark; write thousands as 1,250 or 1250, and a decimal with a point, like 1.25"));
        }
    }
    let plain: String = number.chars().filter(|c| *c != ',').collect();
    let value = parse_number(&plain).map_err(|_| format!("{text:?} is not a price; write it as a number of dollars, like 4.25 or $4.25/{wanted}"))?;
    if value < 0.0 {
        return Err(format!("{text:?} is below zero; a price is zero or more"));
    }
    Ok(value)
}

pub fn parse_number(text: &str) -> Result<f64, String> {
    let typed = text.trim();
    let s = plain(text);
    if !s.is_ascii() {
        return Err(format!("{typed:?} has a character that is not part of a number"));
    }
    let mut scan = Scan::new(&s);
    let negative = scan.eat(b'-');
    if !negative {
        scan.eat(b'+');
    }
    let (x, _) = scan.quantity().map_err(|why| format!("{typed:?} {why}"))?;
    if !scan.done() {
        return Err(format!("{typed:?} is not a plain number"));
    }
    let x = if negative { -x } else { x };
    Ok(x + 0.0)
}

/// A number of feet written as feet and inches to the nearest `denominator`th of an inch
/// (16 for a sixteenth), the way a tape reads: `25' 6 3/16"`. For showing a position; an
/// entered value is shown as it was entered.
pub fn feet_as_ft_in(feet: f64, denominator: u32) -> String {
    if !feet.is_finite() {
        return "unknown".into();
    }
    let parts = (feet.abs() * 12.0 * f64::from(denominator)).round();
    let per_foot = 12.0 * f64::from(denominator);
    let whole_feet = (parts / per_foot).floor();
    let inch_parts = parts - whole_feet * per_foot;
    let inches = inch_parts / f64::from(denominator);
    let minus = if feet < 0.0 && parts > 0.0 { "-" } else { "" };
    show_length(&Len::FtIn([whole_feet, inches]), FT_IN).replacen("", minus, 1)
}
