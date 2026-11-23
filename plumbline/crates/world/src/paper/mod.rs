//! A page of line work: fills and lines in points on a sheet of paper, the lettering of
//! strokes, and the PDF they are written out as. It is the display list of BIMCraft's plan
//! pass, cut to what a plan set draws.
//!
//! PORTED CODE, in the files of this folder, each of which says what it was ported from
//! and what was changed. This file takes the shape of BIMCraft's display list
//! (`crates/core/src/render.rs:88`, `Prim`; `render.rs:215`, `Drawing`) at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//!
//! What was changed, and nothing else:
//! * A page's unit is the point (1/72 in) with y up, where BIMCraft's is a pixel of its view
//!   with y down; there is no view, so no scale between pixels and millimetres: a line's
//!   width is already in points (`pen::points`).
//! * Only a fill (`Prim::Fill`) and a line (`Prim::Line`) are taken. BIMCraft's third
//!   primitive, a picture, a page of a plan set has none. Of a primitive's owner, its part
//!   and its pen (for picking and overrides) only the one thing the PDF needs of its part
//!   is kept: whether a line's corners are mitred (`Part::mitred`, `render.rs:67`: all but
//!   lettering, whose strokes turn round so a V's point does not stick out).
//! * `Drawing::text` and `text_left` also record where each piece of lettering lies (`letters`, ours, call 179 D2).
//! * `Drawing::text` (`render.rs:428`, `text_turned`, `render.rs:449`) letters text centred
//!   on a point, turned an angle, in strokes a tenth of its height wide; here a stroke is
//!   never under half a point.

pub mod cut;
pub mod font;
pub mod letters;
pub mod pdf;
pub mod pen;
pub mod size;
pub mod strings;

/// A colour, 0 to 255 each of red, green and blue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// The paper (`render.rs:44`).
pub const PAPER: Rgb = Rgb(255, 255, 255);
/// A cut wall's fill (`render.rs:46`).
pub const POCHE: Rgb = Rgb(168, 168, 168);
/// Ink (`render.rs:49`).
pub const INK: Rgb = Rgb(25, 25, 25);
/// What is seen and not cut, or stays (`render.rs:50`).
pub const FAINT: Rgb = Rgb(120, 120, 120);

/// One thing drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum Prim {
    /// Rings filled in one colour, as a single shape: where rings overlap they are one.
    Fill { rings: Vec<Vec<(f32, f32)>>, color: Rgb },
    /// A line through points, `width` points wide; dashed (its on and off lengths in points,
    /// again and again) or solid; closed or open; its corners turned round when `round`.
    Line { pts: Vec<(f32, f32)>, width: f32, color: Rgb, dash: Option<Vec<f32>>, closed: bool, round: bool },
}

/// A page: its size in points and what is drawn on it, in the order drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct Drawing {
    pub width: f32,
    pub height: f32,
    pub prims: Vec<Prim>,
    /// Where each piece of lettering lies, as it was drawn (`letters`).
    pub letters: Vec<letters::Letter>,
    /// Each letter written that is not lettered as it is (its plain letter, or the
    /// question mark), as `font::unlike` has it (call 182).
    pub unlettered: Vec<(char, Option<char>)>,
}

impl Drawing {
    pub fn new(width: f32, height: f32) -> Drawing {
        Drawing { width, height, prims: Vec::new(), letters: Vec::new(), unlettered: Vec::new() }
    }

    pub fn fill(&mut self, rings: Vec<Vec<(f32, f32)>>, color: Rgb) {
        if !rings.is_empty() {
            self.prims.push(Prim::Fill { rings, color });
        }
    }

    /// A line, corners mitred (a cut wall's corner is square).
    pub fn line(&mut self, pts: Vec<(f32, f32)>, width: f32, color: Rgb, dash: Option<Vec<f32>>, closed: bool) {
        if pts.len() >= 2 {
            self.prims.push(Prim::Line { pts, width, color, dash, closed, round: false });
        }
    }

    /// `text` centred on `(x, y)`, capitals `height` points tall, turned `angle` radians
    /// counter-clockwise; each stroke turns round at its corners.
    pub fn text(&mut self, text: &str, (x, y): (f32, f32), height: f32, angle: f32, color: Rgb) {
        let (s, c) = angle.sin_cos();
        let w = font::width(text, height);
        self.unlettered.extend(font::unlike(text));
        if !text.trim().is_empty() {
            self.letters.push(letters::Letter::centred(text, (x, y), height, angle));
        }
        for pts in font::strokes(text, -w / 2.0, -height / 2.0, height) {
            let pts = pts.into_iter().map(|(a, b)| (x + a * c - b * s, y + a * s + b * c)).collect();
            self.prims.push(Prim::Line { pts, width: (height / 10.0).max(0.5), color, dash: None, closed: false, round: true });
        }
    }

    /// `text` with its left end at `(x, y)` on the baseline, level.
    pub fn text_left(&mut self, text: &str, (x, y): (f32, f32), height: f32, color: Rgb) {
        self.unlettered.extend(font::unlike(text));
        if !text.trim().is_empty() {
            self.letters.push(letters::Letter::centred(text, (x + font::width(text, height) / 2.0, y + height / 2.0), height, 0.0));
        }
        for pts in font::strokes(text, x, y, height) {
            self.prims.push(Prim::Line { pts, width: (height / 10.0).max(0.5), color, dash: None, closed: false, round: true });
        }
    }
}
