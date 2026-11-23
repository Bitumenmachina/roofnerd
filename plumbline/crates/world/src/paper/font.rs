//! Single-stroke lettering for a plan's words and figures: capitals, digits and the marks a
//! plan uses, drawn on an 8 by 12 grid. Lower case is drawn as capitals, as architectural
//! lettering is. It needs no font file, so a page written from it holds none.
//!
//! PORTED CODE. Ported from BIMCraft, `crates/core/src/font.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//!
//! What was changed, and nothing else:
//! * The glyph table (`known`, `font.rs:18`) is taken for the Latin capitals, the digits and
//!   the marks `' " - + = _ / ( ) : ; . , # × ² ° ±` only, with the glyph that BIMCraft draws
//!   for each, point for point. Its Cyrillic capitals, its accents and its Æ are not taken:
//!   the words on a plan set are English.
//! * What there is no letter for is drawn as the question mark, as there (`glyph`,
//!   `font.rs:14`), except what call 182 adds (ours, not BIMCraft's): the ampersand, and a
//!   base letter with its mark (the accented Latin capitals: grave, acute, circumflex,
//!   tilde, diaeresis, ring, cedilla, stroke), drawn from the base letter's own strokes and
//!   a mark above it (or below, a cedilla); a letter with no glyph but a plain letter
//!   (a Polish or Czech letter, a curly quote) is lettered as its plain letter and counted
//!   (`unlike`), and only what has neither is the question mark.
//! * `width` (`font.rs:231`) is taken as it is.
//! * `strokes` (`font.rs:237`) puts the baseline at `y` and draws up the page from it
//!   (y up, as a PDF page is); BIMCraft's runs down a screen (y down).

type Glyph = &'static [&'static [(i8, i8)]];

const O: &[(i8, i8)] = &[(2, 0), (0, 2), (0, 10), (2, 12), (6, 12), (8, 10), (8, 2), (6, 0), (2, 0)];
const P: &[(i8, i8)] = &[(0, 0), (0, 12), (6, 12), (8, 10), (8, 8), (6, 6), (0, 6)];
const QUESTION: Glyph = &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (4, 5), (4, 3)], &[(4, 1), (4, 0)]];

/// Where a letter's mark lies: above the capitals (grid 13 to 16), or below the line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Grave,
    Acute,
    Circumflex,
    Tilde,
    Diaeresis,
    Ring,
    Cedilla,
    Slash,
}

impl Mark {
    fn strokes(self) -> Glyph {
        match self {
            Mark::Grave => &[&[(2, 16), (5, 13)]],
            Mark::Acute => &[&[(3, 13), (6, 16)]],
            Mark::Circumflex => &[&[(1, 13), (4, 16), (7, 13)]],
            Mark::Tilde => &[&[(0, 13), (2, 15), (4, 14), (6, 15), (8, 13)]],
            Mark::Diaeresis => &[&[(2, 14), (2, 15)], &[(6, 14), (6, 15)]],
            Mark::Ring => &[&[(3, 13), (5, 13), (6, 14), (5, 16), (3, 16), (2, 14), (3, 13)]],
            Mark::Cedilla => &[&[(4, 0), (5, -2), (4, -3), (2, -3)]],
            Mark::Slash => &[&[(0, -1), (8, 13)]],
        }
    }
}

/// A capital with a mark: its base letter and its mark.
fn accented(c: char) -> Option<(char, Mark)> {
    use Mark::*;
    Some(match c {
        'À' => ('A', Grave),
        'Á' => ('A', Acute),
        'Â' => ('A', Circumflex),
        'Ã' => ('A', Tilde),
        'Ä' => ('A', Diaeresis),
        'Å' => ('A', Ring),
        'Ç' => ('C', Cedilla),
        'È' => ('E', Grave),
        'É' => ('E', Acute),
        'Ê' => ('E', Circumflex),
        'Ë' => ('E', Diaeresis),
        'Ì' => ('I', Grave),
        'Í' => ('I', Acute),
        'Î' => ('I', Circumflex),
        'Ï' => ('I', Diaeresis),
        'Ñ' => ('N', Tilde),
        'Ò' => ('O', Grave),
        'Ó' => ('O', Acute),
        'Ô' => ('O', Circumflex),
        'Õ' => ('O', Tilde),
        'Ö' => ('O', Diaeresis),
        'Ø' => ('O', Slash),
        'Ù' => ('U', Grave),
        'Ú' => ('U', Acute),
        'Û' => ('U', Circumflex),
        'Ü' => ('U', Diaeresis),
        'Ý' => ('Y', Acute),
        'Ÿ' => ('Y', Diaeresis),
        _ => return None,
    })
}

/// What a letter with no glyph of its own is lettered as, when it has a plain letter.
fn plain(c: char) -> Option<char> {
    Some(match c {
        'Ā' | 'Ă' | 'Ą' => 'A',
        'Ć' | 'Ĉ' | 'Ċ' | 'Č' => 'C',
        'Ď' | 'Đ' => 'D',
        'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => 'E',
        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => 'G',
        'Ĥ' | 'Ħ' => 'H',
        'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => 'I',
        'Ĵ' => 'J',
        'Ķ' => 'K',
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ł' => 'L',
        'Ń' | 'Ņ' | 'Ň' => 'N',
        'Ō' | 'Ŏ' | 'Ő' => 'O',
        'Ŕ' | 'Ŗ' | 'Ř' => 'R',
        'Ś' | 'Ŝ' | 'Ş' | 'Š' => 'S',
        'Ţ' | 'Ť' | 'Ŧ' => 'T',
        'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => 'U',
        'Ŵ' => 'W',
        'Ŷ' => 'Y',
        'Ź' | 'Ż' | 'Ž' => 'Z',
        '\u{2018}' | '\u{2019}' => '\'',
        '\u{201C}' | '\u{201D}' => '"',
        '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
        '\u{00A0}' => ' ',
        _ => return None,
    })
}

/// The strokes drawing `c` (a capital): its own, a base letter with its mark, the stroke of a
/// plain letter, or the question mark.
fn strokes_of(c: char) -> Vec<&'static [(i8, i8)]> {
    if let Some(g) = known(c) {
        return g.to_vec();
    }
    if let Some((base, mark)) = accented(c) {
        let mut g = known(base).map(|g| g.to_vec()).unwrap_or_default();
        g.extend_from_slice(mark.strokes());
        return g;
    }
    if let Some(b) = plain(c).and_then(known) {
        return b.to_vec();
    }
    QUESTION.to_vec()
}

/// The capitals `text` is lettered in: lower case as capitals, and a letter that capitals
/// make two of (a German sharp s) as its two.
fn capitals(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars().flat_map(char::to_uppercase)
}

/// What of `text` is not lettered as written: each such letter and what it is lettered as
/// (its plain letter, or `None` for the question mark).
pub fn unlike(text: &str) -> Vec<(char, Option<char>)> {
    capitals(text).filter(|&c| known(c).is_none() && accented(c).is_none()).map(|c| (c, plain(c).filter(|&p| known(p).is_some()))).collect()
}

/// How far above the capitals' height (in twelfths of it) and below the baseline the mark of
/// any letter of `text` reaches, for the box the lettering lies in.
pub fn reach(text: &str) -> (f32, f32) {
    let mut up = 0.0f32;
    let mut down = 0.0f32;
    for c in capitals(text) {
        if let Some((_, m)) = accented(c) {
            match m {
                Mark::Cedilla => down = down.max(3.0),
                Mark::Slash => {
                    up = up.max(1.0);
                    down = down.max(1.0);
                }
                _ => up = up.max(4.0),
            }
        }
    }
    (up / 12.0, down / 12.0)
}

/// Its letter for capital `c`, if it has one.
fn known(c: char) -> Option<Glyph> {
    Some(match c {
        'A' => &[&[(0, 0), (4, 12), (8, 0)], &[(1, 3), (7, 3)]],
        'B' => &[&[(0, 0), (0, 12), (6, 12), (8, 10), (8, 8), (6, 6), (0, 6)], &[(6, 6), (8, 4), (8, 2), (6, 0), (0, 0)]],
        'C' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2)]],
        'D' => &[&[(0, 0), (0, 12), (5, 12), (8, 9), (8, 3), (5, 0), (0, 0)]],
        'E' => &[&[(8, 12), (0, 12), (0, 0), (8, 0)], &[(0, 6), (6, 6)]],
        'F' => &[&[(8, 12), (0, 12), (0, 0)], &[(0, 6), (6, 6)]],
        'G' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2), (8, 5), (5, 5)]],
        'H' => &[&[(0, 0), (0, 12)], &[(8, 0), (8, 12)], &[(0, 6), (8, 6)]],
        'I' => &[&[(2, 12), (6, 12)], &[(4, 12), (4, 0)], &[(2, 0), (6, 0)]],
        'J' => &[&[(8, 12), (8, 2), (6, 0), (2, 0), (0, 2)]],
        'K' => &[&[(0, 0), (0, 12)], &[(8, 12), (0, 4)], &[(3, 7), (8, 0)]],
        'L' => &[&[(0, 12), (0, 0), (8, 0)]],
        'M' => &[&[(0, 0), (0, 12), (4, 6), (8, 12), (8, 0)]],
        'N' => &[&[(0, 0), (0, 12), (8, 0), (8, 12)]],
        'O' => &[O],
        'P' => &[P],
        'Q' => &[O, &[(5, 3), (8, 0)]],
        'R' => &[P, &[(4, 6), (8, 0)]],
        'S' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 8), (2, 6), (6, 6), (8, 4), (8, 2), (6, 0), (2, 0), (0, 2)]],
        'T' => &[&[(0, 12), (8, 12)], &[(4, 12), (4, 0)]],
        'U' => &[&[(0, 12), (0, 2), (2, 0), (6, 0), (8, 2), (8, 12)]],
        'V' => &[&[(0, 12), (4, 0), (8, 12)]],
        'W' => &[&[(0, 12), (2, 0), (4, 6), (6, 0), (8, 12)]],
        'X' => &[&[(0, 0), (8, 12)], &[(0, 12), (8, 0)]],
        'Y' => &[&[(0, 12), (4, 6), (8, 12)], &[(4, 6), (4, 0)]],
        'Z' => &[&[(0, 12), (8, 12), (0, 0), (8, 0)]],
        '0' => &[&[(2, 0), (1, 2), (1, 10), (2, 12), (6, 12), (7, 10), (7, 2), (6, 0), (2, 0)]],
        '1' => &[&[(2, 10), (4, 12), (4, 0)], &[(2, 0), (6, 0)]],
        '2' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (0, 0), (8, 0)]],
        '3' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (6, 6), (8, 4), (8, 2), (6, 0), (2, 0), (0, 2)], &[(3, 6), (6, 6)]],
        '4' => &[&[(6, 0), (6, 12), (0, 4), (8, 4)]],
        '5' => &[&[(8, 12), (0, 12), (0, 7), (6, 7), (8, 5), (8, 2), (6, 0), (2, 0), (0, 2)]],
        '6' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2), (8, 5), (6, 7), (2, 7), (0, 5)]],
        '7' => &[&[(0, 12), (8, 12), (3, 0)]],
        '8' => &[&[(2, 6), (0, 8), (0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (6, 6), (2, 6), (0, 4), (0, 2), (2, 0), (6, 0), (8, 2), (8, 4), (6, 6)]],
        '9' => &[&[(0, 2), (2, 0), (6, 0), (8, 2), (8, 10), (6, 12), (2, 12), (0, 10), (0, 7), (2, 5), (6, 5), (8, 7)]],
        ' ' => &[],
        '.' => &[&[(3, 0), (4, 0), (4, 1), (3, 1), (3, 0)]],
        ',' => &[&[(4, 1), (4, 0), (3, -2)]],
        '-' => &[&[(1, 6), (7, 6)]],
        '+' => &[&[(4, 2), (4, 10)], &[(0, 6), (8, 6)]],
        '=' => &[&[(1, 4), (7, 4)], &[(1, 8), (7, 8)]],
        '&' => &[&[(8, 0), (2, 7), (1, 9), (1, 10), (3, 12), (5, 12), (7, 10), (7, 9), (0, 3), (0, 1), (2, 0), (5, 0), (8, 4)]],
        '_' => &[&[(0, -1), (8, -1)]],
        '/' => &[&[(0, 0), (8, 12)]],
        '(' => &[&[(6, 12), (4, 9), (4, 3), (6, 0)]],
        ')' => &[&[(2, 12), (4, 9), (4, 3), (2, 0)]],
        ':' => &[&[(4, 2), (4, 3)], &[(4, 8), (4, 9)]],
        ';' => &[&[(4, 8), (4, 9)], &[(4, 3), (4, 1), (3, -1)]],
        '\'' => &[&[(4, 12), (4, 9)]],
        '"' => &[&[(3, 12), (3, 9)], &[(5, 12), (5, 9)]],
        '#' => &[&[(2, 0), (3, 12)], &[(5, 0), (6, 12)], &[(0, 4), (8, 4)], &[(0, 8), (8, 8)]],
        '×' => &[&[(1, 2), (7, 10)], &[(1, 10), (7, 2)]],
        '²' => &[&[(2, 11), (3, 12), (5, 12), (6, 11), (6, 10), (2, 7), (6, 7)]],
        '°' => &[&[(3, 12), (5, 12), (6, 11), (6, 9), (5, 8), (3, 8), (2, 9), (2, 11), (3, 12)]],
        '±' => &[&[(4, 5), (4, 11)], &[(1, 8), (7, 8)], &[(1, 2), (7, 2)]],
        _ => return None,
    })
}

/// Grid units from one character to the next.
const ADVANCE: f32 = 11.0;

/// How wide `text` is at capitals `height` tall (any one unit).
pub fn width(text: &str, height: f32) -> f32 {
    let n = capitals(text).count() as f32;
    if n == 0.0 {
        0.0
    } else {
        (n * ADVANCE - 3.0) * height / 12.0
    }
}

/// Polylines drawing `text` with capitals `height` tall, its first letter's left end at
/// `(x, y)` on the baseline, the page's y up.
pub fn strokes(text: &str, x: f32, y: f32, height: f32) -> Vec<Vec<(f32, f32)>> {
    let k = height / 12.0;
    let mut out = Vec::new();
    for (i, c) in capitals(text).enumerate() {
        let x0 = x + i as f32 * ADVANCE * k;
        for line in strokes_of(c) {
            out.push(line.iter().map(|&(gx, gy)| (x0 + f32::from(gx) * k, y + f32::from(gy) * k)).collect());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_case_is_drawn_as_capitals_and_what_it_has_no_letter_for_as_a_question_mark() {
        assert_eq!(strokes("living", 0.0, 0.0, 12.0), strokes("LIVING", 0.0, 0.0, 12.0));
        assert_eq!(strokes("ω", 0.0, 0.0, 12.0), strokes("?", 0.0, 0.0, 12.0));
        assert!(strokes(" ", 0.0, 0.0, 12.0).is_empty());
    }

    #[test]
    fn text_is_scaled_to_its_height_from_the_baseline_up_the_page() {
        let s = strokes("I", 10.0, 100.0, 24.0);
        let ys: Vec<f32> = s.iter().flatten().map(|p| p.1).collect();
        assert_eq!(ys.iter().copied().fold(f32::MAX, f32::min), 100.0);
        assert_eq!(ys.iter().copied().fold(f32::MIN, f32::max), 124.0);
        assert_eq!(width("II", 12.0), 19.0);
    }

    #[test]
    fn every_mark_a_plan_set_letters_has_a_letter() {
        for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,-+=_/():;'\"#×²°±&".chars() {
            assert!(known(c).is_some(), "{c}");
        }
    }

    #[test]
    fn an_ampersand_and_the_accented_letters_are_lettered_and_what_is_not_is_counted() {
        // The ampersand is a letter of its own, not the question mark.
        assert_ne!(strokes("&", 0.0, 0.0, 12.0), strokes("?", 0.0, 0.0, 12.0));
        // An accented letter is its base letter's strokes and one mark more, in the same width.
        let e = strokes("E", 0.0, 0.0, 12.0);
        let e_acute = strokes("é", 0.0, 0.0, 12.0);
        assert_eq!(e_acute.len(), e.len() + 1);
        assert_eq!(&e_acute[..e.len()], &e[..]);
        assert_ne!(e_acute, strokes("?", 0.0, 0.0, 12.0));
        assert_eq!(width("É", 12.0), width("E", 12.0));
        // Every Latin-1 capital with a mark is lettered as itself.
        assert!(unlike("ÀÁÂÃÄÅÇÈÉÊËÌÍÎÏÑÒÓÔÕÖØÙÚÛÜÝŸàéîõü").is_empty());
        assert!(unlike("Café & \"Bar\" Ünderstairs").is_empty());
        // A letter with a plain letter is lettered as it and counted; one with none is the question mark, counted.
        assert_eq!(unlike("Łódź"), vec![('Ł', Some('L')), ('Ź', Some('Z'))]);
        assert_eq!(unlike("ωx"), vec![('Ω', None)]);
        assert_eq!(strokes("Ł", 0.0, 0.0, 12.0), strokes("L", 0.0, 0.0, 12.0));
        // The sharp s is lettered as the two capitals it makes, and is as wide as two.
        assert_eq!(width("ß", 12.0), width("SS", 12.0));
        assert_eq!(strokes("ß", 0.0, 0.0, 12.0), strokes("SS", 0.0, 0.0, 12.0));
        // A mark reaches above the capitals, or below the line for a cedilla.
        assert!(reach("É").0 > 0.0 && reach("Ç").1 > 0.0 && reach("E") == (0.0, 0.0));
    }
}
