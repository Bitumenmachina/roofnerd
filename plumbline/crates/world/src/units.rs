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

/// The fraction a laser's display writes after the inch mark: `2'2"13/32`, `26"13/32`.
/// Nothing when the text ends at the mark. The inches before the mark are then whole.
/// `by_mark` says the inches were closed with `"` and not with the word.
fn fraction_after_mark(scan: &mut Scan, inches: f64, by_mark: bool) -> Result<f64, String> {
    scan.spaces();
    if scan.done() {
        return Ok(0.0);
    }
    let n = scan.digits();
    if !by_mark || n.is_empty() || !scan.eat(b'/') {
        return Err("has something after the inches".into());
    }
    if inches.fract() != 0.0 {
        return Err("has a part of an inch before the inch mark and a fraction after it; write one or the other".into());
    }
    let fraction = scan.fraction(n)?;
    scan.spaces();
    if !scan.done() {
        return Err("has something after the inches".into());
    }
    Ok(fraction)
}

/// What is said of a thing that is no measurement at all. It names the way the trades write feet and inches, and says nothing
/// of commas: there are none in most of what lands here (call 209 point 5).
fn not_a_measure_said() -> String {
    "is not a measurement this program can read; write it like 25.5 ft, 306 in, 12-6 (twelve feet six) or 25' 6 1/2\"".into()
}

/// The dash after a number with a space before it ("12 - 6") is a range or a subtraction, not feet and inches.
fn spaced_dash(scan: &Scan) -> bool {
    scan.at > 0 && scan.s[scan.at - 1] == b' '
}

/// Read a length as typed: `25.5 ft`, `306 in`, `19 1/2"`, `25' 6"`, `25'-6 1/2"`,
/// `25 ft 6 in`, `12-6` (twelve feet six, as the trades write it), and as a laser types its display: `2'2"13/32`, `26"13/32`, `23.46ft`.
/// Gives the number and the unit it was entered in. A bare number is refused: it is
/// never read as feet.
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
        // Nothing is nothing in any unit: "0" needs none (J18-5).
        Mark::None if scan.done() && first == 0.0 => Ok((Len::One(0.0), FT)),
        Mark::None => {
            if scan.done() {
                // Asked of the number that was typed: "9" was answered with "25.5 ft,
                // 306 in, or 25' 6"", a thickness of twenty-five feet (J22B-36, -41).
                let t = typed.trim();
                Err(fail(format!("has no unit: is it {t} ft or {t} in? Type the unit with it. A number with no unit is never read as feet (a laser has to be set to send its unit with the reading)")))
            } else if scan.peek() == Some(b',') {
                Err(fail("has a comma, and 1,234 is a thousand to one reader and one and a bit to another; write the decimal with a point, like 25.5 ft, and no comma in the thousands (a laser can be set to send a point)".into()))
            } else if scan.peek() == Some(b'-') && !spaced_dash(&scan) {
                // 12-6, 12-6 1/2, 12-6-1/2: feet and inches, written as the trades write them (call 209 point 5). Only
                // with the dash hard against the feet; "12 - 6" and "12-6 ft" are ranges, and are not read.
                scan.eat(b'-');
                if !scan.peek().is_some_and(|c| c.is_ascii_digit() || c == b'.') {
                    return Err(fail(not_a_measure_said()));
                }
                let (mut inches, _) = scan.quantity().map_err(&fail)?;
                match mark(&mut scan, typed)? {
                    Mark::Feet => return Err(fail("has feet after the inches; write twelve feet six as 12-6, 12' 6\" or 12 ft 6 in".into())),
                    Mark::Inches => {
                        let by_mark = scan.s[scan.at - 1] == b'"';
                        inches += fraction_after_mark(&mut scan, inches, by_mark).map_err(&fail)?;
                    }
                    Mark::None => {
                        scan.spaces();
                        if !scan.done() {
                            return Err(fail("has something after the inches".into()));
                        }
                    }
                }
                if first_decimal || first.fract() != 0.0 {
                    return Err(fail("has a part of a foot and inches as well; write the feet as a whole number, or the whole length as decimal feet".into()));
                }
                if inches >= 12.0 {
                    return Err(fail(format!("has {inches} inches after the feet; the inches of a feet-and-inches length are less than 12")));
                }
                Ok((Len::FtIn([sign * first + 0.0, sign * inches + 0.0]), FT_IN))
            } else {
                Err(fail(not_a_measure_said()))
            }
        }
        Mark::Inches => {
            let by_mark = scan.s[scan.at - 1] == b'"';
            let fraction = fraction_after_mark(&mut scan, first, by_mark).map_err(&fail)?;
            Ok((Len::One(sign * (first + fraction) + 0.0), IN))
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
            let (mut inches, _) = scan.quantity().map_err(&fail)?;
            match mark(&mut scan, typed)? {
                Mark::Feet => return Err(fail("gives feet twice".into())),
                Mark::Inches => {
                    let by_mark = scan.s[scan.at - 1] == b'"';
                    inches += fraction_after_mark(&mut scan, inches, by_mark).map_err(&fail)?;
                }
                Mark::None => {
                    scan.spaces();
                    if !scan.done() {
                        return Err(fail("has something after the inches".into()));
                    }
                }
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

/// The units a typed measure (a value entered on a line that is not a length of the
/// model's own) is given in: a length, an area, a roofer's square, a volume, a count.
pub const MEASURE_UNITS: [&str; 5] = ["LF", "SF", "SQ", "CY", "EA"];

/// Is this one of the units of a typed measure?
pub fn is_measure_unit(unit: &str) -> bool {
    MEASURE_UNITS.contains(&unit)
}

/// What a typed measure counts in, as a formula works it: its dimension (a count 0, a
/// length 1, an area 2, a volume 3) and what one of its unit is in feet to that power (a
/// square is 100 SF, a cubic yard 27 CF).
pub fn measure_dimension(unit: &str) -> Option<(i32, f64)> {
    match unit {
        "EA" => Some((0, 1.0)),
        "LF" => Some((1, 1.0)),
        "SF" => Some((2, 1.0)),
        "SQ" => Some((2, 100.0)),
        "CY" => Some((3, 27.0)),
        _ => None,
    }
}

/// The unit a measure's last word is, as estimators write it: sf, sq ft, sqft; lf, lin ft;
/// sq, square, squares; cy, cu yd; ea, each; and, where a length typed on a line is a
/// length of the line (`ft_is_lf`), ft. In any capitals, with or without stops.
fn measure_unit_word(word: &str, ft_is_lf: bool) -> Option<&'static str> {
    let said: String = word.to_lowercase().replace('.', " ").split_whitespace().collect::<Vec<_>>().join(" ");
    match said.as_str() {
        "sf" | "sq ft" | "sqft" => Some("SF"),
        "lf" | "lin ft" | "linft" => Some("LF"),
        "ft" if ft_is_lf => Some("LF"),
        "sq" | "square" | "squares" => Some("SQ"),
        "cy" | "cu yd" | "cuyd" => Some("CY"),
        "ea" | "each" => Some("EA"),
        _ => None,
    }
}

/// What a typed measure is refused with when it is not one: the units it takes.
fn not_a_measure(typed: &str) -> String {
    format!("{typed:?} is not a measure; a measure is a number and one of {}, like 60 SF, 24 LF, 2 CY, 3 SQ or 4 EA", MEASURE_UNITS.join(", "))
}

/// Read what is typed as a measure, where it is one: a number (with a comma for the
/// thousands, `1,200`) and a unit as estimators write it. None when it is not a number
/// and a unit at all; Some(Err) when it is one and is not right (below zero, a comma
/// that is no thousands mark); `ft_is_lf` reads "ft" as LF, for a length typed on a line.
pub fn read_measure(text: &str, ft_is_lf: bool) -> Option<Result<(f64, &'static str), String>> {
    let typed = text.trim();
    let split = typed.rfind(|c: char| c.is_ascii_digit())? + 1;
    let (number, unit) = (typed[..split].trim(), typed[split..].trim());
    let unit = measure_unit_word(unit, ft_is_lf)?;
    let plain = if number.contains(',') {
        if !commas_part_thousands(number) {
            return Some(Err(format!("{typed:?} has a comma that is not a thousands mark; write 1,200 or 1200, and a part of a unit with a point")));
        }
        number.replace(',', "")
    } else {
        number.to_string()
    };
    let x = match parse_number(&plain) {
        Ok(x) => x,
        Err(_) => return Some(Err(not_a_measure(typed))),
    };
    if x < 0.0 {
        return Some(Err(format!("{typed:?} is below zero; a measure is zero or more")));
    }
    Some(Ok((x, unit)))
}

/// Read a measure as typed: a number and one of LF, SF, SQ, CY or EA (`60 SF`, `2 CY`,
/// `24 LF`, `1,200 SF`, `60 sq ft`), in any capitals. A number alone is a count and is not
/// read here; a length in feet or inches is `parse_length`'s.
pub fn parse_measure(text: &str) -> Result<(f64, &'static str), String> {
    read_measure(text, false).unwrap_or_else(|| Err(not_a_measure(text.trim())))
}

/// As `parse_measure`, for a figure typed on a line to be bought by (a length is LF).
pub fn parse_measure_on_line(text: &str) -> Result<(f64, &'static str), String> {
    read_measure(text, true).unwrap_or_else(|| Err(not_a_measure(text.trim())))
}

/// Does what is typed as a quantity look like a measure someone meant to write, a number
/// and then words (`60 boards`, `-3 SF`, `24 ft`), and so is to be read as one or refused
/// with the units a measure takes? A formula has an operator or a name of its own.
pub fn looks_like_measure(text: &str) -> bool {
    let t = text.trim();
    let digits_then_words = |t: &str| {
        let split = t.find(|c: char| !(c.is_ascii_digit() || c == ',' || c == '.' || c == ' ')).unwrap_or(t.len());
        let (number, words) = (&t[..split], &t[split..]);
        number.chars().any(|c| c.is_ascii_digit()) && !words.is_empty() && words.chars().all(|c| c.is_ascii_alphabetic() || c == '.' || c == ' ') && words.chars().any(|c| c.is_ascii_alphabetic())
    };
    digits_then_words(t.strip_prefix('-').unwrap_or(t).trim_start())
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

/// Whether every comma in a figure is a thousands mark: one to three figures, then
/// threes, and none after the point. `1,250` and `12,500.75` are; `1,25`, `9,05` and
/// `1.204,50` are not (each is how a decimal is written somewhere, and would be read a
/// hundred or a thousand times out). The one rule for a comma in a figure, wherever a
/// figure is read: a price typed, and a figure in a list brought in.
pub fn commas_part_thousands(number: &str) -> bool {
    let number = number.trim().trim_start_matches(['-', '+']);
    let whole = number.split('.').next().unwrap_or("");
    let groups: Vec<&str> = whole.split(',').collect();
    let digits = |g: &str| !g.is_empty() && g.chars().all(|c| c.is_ascii_digit());
    number.matches('.').count() <= 1 && !number.split('.').nth(1).is_some_and(|f| f.contains(',')) && groups.len() > 1 && digits(groups[0]) && groups[0].len() <= 3 && groups[1..].iter().all(|g| g.len() == 3 && digits(g))
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
    if number.contains(',') && !commas_part_thousands(number) {
        return Err(format!("{text:?} has a comma that is not a thousands mark; write thousands as 1,250 or 1250, and a decimal with a point, like 1.25"));
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

/// A thing's size, wide by deep by high, given in feet and written the way its maker's
/// sheet and the trade write it: in inches while every side is under 8 ft (`16 × 16 ×
/// 8 in`, `24 × 30 × 60 in`), to the sixteenth; in feet and inches when a side is longer.
/// A length in feet as inches to the sixteenth, no unit written: `4 13/16`, `6`, `3 5/8`.
fn sixteenths(feet: f64) -> String {
    let parts = (feet.abs() * 12.0 * 16.0).round() as u64;
    let (whole, mut top, mut bottom) = (parts / 16, parts % 16, 16);
    while top > 0 && top % 2 == 0 {
        top /= 2;
        bottom /= 2;
    }
    match (whole, top) {
        (w, 0) => format!("{w}"),
        (0, t) => format!("{t}/{bottom}"),
        (w, t) => format!("{w} {t}/{bottom}"),
    }
}

/// A length in feet as a person says inches in a sentence, to the sixteenth: "4 13/16 in", "6 in". A distance too small to reach
/// a sixteenth is said so, and never as nothing.
pub fn inches_said(feet: f64) -> String {
    if feet.is_finite() && feet.abs() > 0.0 && (feet.abs() * 12.0 * 16.0).round() < 1.0 {
        return "less than 1/16 in".to_string();
    }
    format!("{} in", sixteenths(feet))
}

pub fn size_said(sides: [f64; 3]) -> String {
    if sides.iter().any(|x| !x.is_finite()) {
        return "unknown".into();
    }
    if sides.iter().any(|x| x.abs() >= 8.0) {
        return sides.map(|x| feet_as_ft_in(x, 16)).join(" × ");
    }
    format!("{} in", sides.map(sixteenths).join(" × "))
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
