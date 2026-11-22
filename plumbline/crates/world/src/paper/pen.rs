//! Pens: how wide a line is on paper. A line's weight is a pen, 1 to 16 as in Revit, each a
//! width in paper millimetres, so it prints the same on any paper.
//!
//! PORTED CODE. Ported from BIMCraft, `crates/core/src/pen.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//!
//! What was changed, and nothing else:
//! * `WIDTHS` (`pen.rs:34`) and the pens a drawing uses (`Pen::CUT`, `BORDER`, `HEAVY`,
//!   `MEDIUM`, `PROJECTION`, `FINE`; `pen.rs:40` to `pen.rs:55`) are taken as they are, as
//!   constants (pen numbers).
//! * `LineWeights::default_for` and `LineWeights::at` (`pen.rs:136`, `pen.rs:154`) are taken
//!   for the imperial scales (12, 24, 48, 96, 192, 384) and as two functions: the model's
//!   pens widen at finer scales and thin at coarser ones by the fourth root of how many
//!   times finer, to the hundredth of a millimetre, taking the column of the scale nearest
//!   (by how many times, the finer of two as near); the annotation's pens are the widths
//!   themselves. A project's own line weights (`LineWeights` as a saved thing) are not taken:
//!   a plan set has the starting ones.
//! * A pen is a `u8`, where BIMCraft's is a type of its own with a reader that refuses a
//!   fraction.

pub type Pen = u8;

/// What a cut passes through, outlined: walls in plan.
pub const CUT: Pen = 10;
/// A sheet's border and its title block's frame.
pub const BORDER: Pen = 8;
/// Heavy: an opening's jambs, a dimension's ticks.
pub const HEAVY: Pen = 7;
/// Medium: a door's leaf.
pub const MEDIUM: Pen = 6;
/// What is seen beyond the cut, and annotation's lines: dimensions, lettering's strokes.
pub const PROJECTION: Pen = 4;
/// The finest.
pub const FINE: Pen = 1;

/// The widest pen.
pub const MAX_PEN: u8 = 16;

/// Each pen's width on paper at 1 : 100, mm, from pen 0 (a hairline, no width of its own) up.
const WIDTHS: [f64; MAX_PEN as usize + 1] = [0.0, 0.10, 0.13, 0.15, 0.18, 0.20, 0.22, 0.25, 0.28, 0.30, 0.35, 0.40, 0.50, 0.70, 1.00, 1.40, 2.00];

/// The scales a project in feet and inches draws at, 1 : n, as BIMCraft starts it
/// (`pen.rs:140`).
const SCALES: [f64; 6] = [12.0, 24.0, 48.0, 96.0, 192.0, 384.0];

/// How wide a pen draws a line of the annotation (lettering, dimensions), mm on paper.
pub fn annotation_mm(pen: Pen) -> f64 {
    WIDTHS.get(usize::from(pen)).copied().unwrap_or(0.0)
}

/// How wide a pen draws the model (walls, openings) on a plan at 1 : `n`, mm on paper.
pub fn model_mm(pen: Pen, n: f64) -> f64 {
    let off = |s: &f64| (s / n).ln().abs();
    let nearest = SCALES.iter().copied().min_by(|a, b| off(a).total_cmp(&off(b)).then(a.total_cmp(b))).unwrap_or(96.0);
    let k = (100.0 / nearest).powf(0.25);
    match WIDTHS.get(usize::from(pen)) {
        Some(&w) if pen > 0 => ((w * k * 100.0).round() / 100.0).clamp(0.01, 20.0),
        _ => 0.0,
    }
}

/// Millimetres of paper to points (a point is 1/72 in).
pub fn points(mm: f64) -> f32 {
    (mm * 72.0 / 25.4) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pens_widen_from_a_hairline() {
        assert_eq!(annotation_mm(0), 0.0);
        assert!((1..=MAX_PEN).all(|p| annotation_mm(p) > annotation_mm(p - 1)));
        assert_eq!(annotation_mm(MAX_PEN + 1), 0.0);
    }

    #[test]
    fn the_cuts_pen_is_wider_at_a_finer_scale() {
        // 1/4" = 1'-0" is 1 : 48, a column of its own: (100 / 48) to the fourth root.
        assert_eq!(model_mm(CUT, 48.0), 0.42);
        assert_eq!(model_mm(CUT, 96.0), 0.35);
        assert!(model_mm(CUT, 24.0) > model_mm(CUT, 48.0));
        // Between two, the nearer by how many times.
        assert_eq!(model_mm(CUT, 64.0), model_mm(CUT, 48.0));
        assert_eq!(annotation_mm(PROJECTION), 0.18);
    }
}
