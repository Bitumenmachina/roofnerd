//! The paper a plan set is written on, and the scale it is drawn at.
//!
//! PORTED CODE. Ported from BIMCraft at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71
//! (2026-11-10), https://github.com/IPedrax/bimcraft, Copyright (c) 2026 IPedrax and the
//! BIMCraft contributors, used under the MIT license (BIMCraft is MIT OR Apache-2.0). The
//! notice is in NOTICE at the repository root.
//!
//! What was changed, and nothing else:
//! * The US architectural sizes ARCH A to D (`model::paper`, `crates/core/src/model.rs:1966`)
//!   are taken, as BIMCraft gives them, landscape, in millimetres: 304.8 by 228.6, 457.2 by
//!   304.8, 609.6 by 457.2, 914.4 by 609.6. The ISO, ANSI and the larger sizes, and a custom
//!   size, are not taken: a plan set is written on these four. A size is named as BIMCraft
//!   reads it ("ARCH A", "arch a", "ARCHA").
//! * The scale's wording (`Units::scale`, `crates/core/src/units.rs:123`) is taken for the
//!   architect's scales (an inch or a fraction of one to the foot): `1/4" = 1'-0"`. The
//!   engineer's scales are not taken. Of the architect's, these five are the ones a plan
//!   set is drawn at: 1/8, 3/16, 1/4, 3/8 and 1/2 in to the foot (what BIMCraft's
//!   `IMPERIAL_SCALES` offers of them).

/// A paper size: its name as it is lettered, and its width and height in inches (landscape).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paper {
    pub name: &'static str,
    pub width_in: f64,
    pub height_in: f64,
}

impl Paper {
    /// Its width and height in points (1/72 in), as BIMCraft's millimetres come to.
    pub fn points(&self) -> (f64, f64) {
        (self.width_in * 72.0, self.height_in * 72.0)
    }
}

/// ARCH A, B, C and D, smallest first.
pub const PAPERS: [Paper; 4] = [
    Paper { name: "ARCH A", width_in: 12.0, height_in: 9.0 },
    Paper { name: "ARCH B", width_in: 18.0, height_in: 12.0 },
    Paper { name: "ARCH C", width_in: 24.0, height_in: 18.0 },
    Paper { name: "ARCH D", width_in: 36.0, height_in: 24.0 },
];

/// A paper size as typed: "ARCH A", "arch a", "archa".
pub fn paper(name: &str) -> Option<Paper> {
    let squeezed: String = name.trim().to_ascii_uppercase().chars().filter(|c| !c.is_whitespace()).collect();
    PAPERS.iter().copied().find(|p| p.name.replace(' ', "") == squeezed)
}

/// The scales a plan set is drawn at, finest print last: inches of paper to the foot, as the
/// drawing says them (numerator, denominator).
pub const SCALES: [(u32, u32); 5] = [(1, 8), (3, 16), (1, 4), (3, 8), (1, 2)];

/// Inches of paper to the foot at scale `(num, den)`.
pub fn inches_to_the_foot(s: (u32, u32)) -> f64 {
    f64::from(s.0) / f64::from(s.1)
}

/// 1 : n for scale `(num, den)`: 12 / inches to the foot (1/4 in is 1 : 48).
pub fn ratio(s: (u32, u32)) -> f64 {
    12.0 / inches_to_the_foot(s)
}

/// A scale as a drawing says it: `1/4" = 1'-0"`.
pub fn scale_said(s: (u32, u32)) -> String {
    format!("{}/{}\" = 1'-0\"", s.0, s.1)
}

/// A scale as typed: `1/4`, `1/4 in`, `1/4"`, `1/4" = 1'-0"`; one of the five, or none.
pub fn scale(text: &str) -> Option<(u32, u32)> {
    let t = text.trim().to_ascii_lowercase();
    let t = t.split('=').next().unwrap_or("").trim();
    let t = t.trim_end_matches("in").trim_end_matches('"').trim();
    let (a, b) = t.split_once('/')?;
    let s = (a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?);
    SCALES.contains(&s).then_some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sizes_are_bimcrafts_in_points() {
        assert_eq!(paper("ARCH A").map(|p| p.points()), Some((864.0, 648.0)));
        assert_eq!(paper("arch c").map(|p| p.points()), Some((1728.0, 1296.0)));
        assert_eq!(paper("ArchD").map(|p| p.name), Some("ARCH D"));
        assert_eq!(paper("A4"), None);
    }

    #[test]
    fn a_scale_is_said_as_a_drawing_says_it_and_read_as_typed() {
        assert_eq!(scale_said((1, 4)), "1/4\" = 1'-0\"");
        assert_eq!(ratio((1, 4)), 48.0);
        assert_eq!(ratio((1, 2)), 24.0);
        for typed in ["1/2 in", "1/2", "1/2\"", "1/2\" = 1'-0\"", " 1/2 IN "] {
            assert_eq!(scale(typed), Some((1, 2)), "{typed}");
        }
        assert_eq!(scale("3/16"), Some((3, 16)));
        assert_eq!(scale("1/3"), None);
        assert_eq!(scale("1 in"), None);
    }
}
