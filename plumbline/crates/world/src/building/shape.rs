//! An item's shape as data (call 198): a named list of blocks, each a box (or an upright
//! round that fills its box) between six figures, every figure a formula in the item's
//! `width`, `depth` and `height`, in the language a cost line's quantity is written in
//! (`min(100 mm, 0.14 * height)`), read by the same reader (`formula.rs`). A block may carry a
//! "There when": one comparison of two such formulas (`width >= 30 in`), read here and nowhere else:
//! the formula language has no comparison, and a cost line typed `net_area > 100 SF` is refused.
//!
//! Across is from the middle of the thing (a plus is to its right, looking at its front);
//! back is from the back, the side that goes to a wall, toward the front; up is from the
//! foot. Every figure comes out as a length (worked in feet). A block's two figures on an
//! axis may be written in either order: the lesser is its start, as BIMCraft's `put` has it.
//!
//! Each block says what it is made of, from the materials the window already draws
//! (`forms::Made`). A shape is a drawing and nothing else: no price rests on it, and a
//! clash still tests the item's box, so a block carries no Known or Assumed (call 198,
//! point 5). A shape that cannot be worked says which block and which figure.
//!
//! The thirteen the program draws today are rows of `library/shapes.csv` (`STARTER`),
//! as the starter assemblies are rows of theirs: they are in the program, and a job that
//! names one carries nothing for it. A job carries the shapes of its own (`Job::shapes`).
//! Ported from BIMCraft's `Family` forms in `forms.rs`, whose header says from where.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::forms::{Block, Made, BOX};
use super::P3;
use crate::formula::{self, Expr, Operand};
use crate::value::Status;

/// What the six figures of a block are called, as the table's headings say them.
pub const FIGURES: [&str; 6] = ["Across from", "Across to", "Back from", "Back to", "Up from", "Up to"];

/// A block as its figures are written: formulas in `width`, `depth` and `height`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub made: Made,
    /// An upright round that fills the box the figures make.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub round: bool,
    pub across: [String; 2],
    pub back: [String; 2],
    pub up: [String; 2],
    /// "There when": two formulas with `>`, `>=`, `<` or `<=` between them (`width >= 30 in`). The
    /// block is there at the sizes where it holds. None: always there. BIMCraft's `visible`, narrowed
    /// to a comparison; it is read here and nowhere else (call 198, ruling 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

impl Piece {
    /// The six figures in the order `FIGURES` names them.
    pub fn figures(&self) -> [&str; 6] {
        [&self.across[0], &self.across[1], &self.back[0], &self.back[1], &self.up[0], &self.up[1]]
    }

    /// A box the whole size of the item, of that material, always there: what a block added starts as (call 205).
    pub fn whole(made: Made) -> Piece {
        Piece { made, round: false, across: ["-width / 2".into(), "width / 2".into()], back: ["0".into(), "depth".into()], up: ["0".into(), "height".into()], when: None }
    }
}

/// A named list of blocks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub name: String,
    pub blocks: Vec<Piece>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cmp {
    Gt,
    Ge,
    Lt,
    Le,
}

/// A block's condition, read.
#[derive(Clone, Debug)]
struct Cond {
    left: Expr,
    op: Cmp,
    right: Expr,
}

/// A block with its figures read, ready to be worked for a size.
#[derive(Clone, Debug)]
pub struct Compiled {
    made: Made,
    round: bool,
    figures: [Expr; 6],
    when: Option<Cond>,
}

/// The most blocks a shape may have, and the most shapes a job may carry.
pub const MOST_BLOCKS: usize = 64;
pub const MOST_SHAPES: usize = 500;

/// The three sizes a shape is worked at when it is brought in (small, typical, large): at each, every
/// block that is there has to be inside the box of the item's sizes (call 198, ruling 3). The typical
/// is the starter 36 in vanity.
pub const CHECK_SIZES: [(&str, P3); 3] = [("small", [0.4, 0.3, 0.5]), ("typical", [3.0, 1.75, 2.875]), ("large", [6.5, 3.0, 2.6])];

/// How far a block may reach past the box, in feet.
const REACH: f64 = 1.0e-8;

/// Read a condition: two formulas with one of `>=`, `<=`, `>`, `<` between them.
fn read_cond(text: &str) -> Result<Cond, String> {
    let at = text.find(['<', '>']).ok_or_else(|| format!("{text:?} has no comparison: it takes two figures with one of > >= < <= between them (width >= 30 in)"))?;
    let rest = &text[at..];
    let (op, len) = if rest.starts_with(">=") {
        (Cmp::Ge, 2)
    } else if rest.starts_with("<=") {
        (Cmp::Le, 2)
    } else if rest.starts_with('>') {
        (Cmp::Gt, 1)
    } else {
        (Cmp::Lt, 1)
    };
    let (l, r) = (&text[..at], &text[at + len..]);
    if r.contains(['<', '>', '=']) || l.contains('=') {
        return Err(format!("{text:?} has more than one comparison: it takes two figures with one of > >= < <= between them"));
    }
    let side = |t: &str| formula::parse(t).map_err(|why| format!("{:?} {why}", t.trim()));
    Ok(Cond { left: side(l)?, op, right: side(r)? })
}

fn lookup(name: &str, size: P3) -> Result<Operand, String> {
    let i = match name.to_ascii_lowercase().as_str() {
        "width" => 0,
        "depth" => 1,
        "height" => 2,
        _ => return Err(format!("{name:?} is not something a shape can be worked from: it has its width, depth and height")),
    };
    Ok(Operand { value: Some(size[i]), dim: 1, status: Status::Known, shown: name.to_ascii_lowercase() })
}

fn dim_said(dim: i32) -> &'static str {
    match dim {
        0 => "a plain number",
        1 => "a length",
        2 => "an area",
        3 => "a volume",
        _ => "not a length",
    }
}

/// A condition worked for a size: whether it holds.
fn holds(shape: &str, n: usize, c: &Cond, size: P3) -> Result<bool, String> {
    let at = |why: String| format!("shape {shape:?}, block {}, There when: {why}", n + 1);
    let work = |e: &Expr| formula::work(e, &|name| lookup(name, size)).map_err(&at);
    let (l, r) = (work(&c.left)?, work(&c.right)?);
    let zero = |e: &Expr| matches!(e, Expr::Num(x) if *x == 0.0);
    if l.dim != r.dim && !(zero(&c.left) || zero(&c.right)) {
        return Err(at(format!("{} is {} and {} is {}: both sides are lengths or both are plain numbers", l.text, dim_said(l.dim), r.text, dim_said(r.dim))));
    }
    if !(l.dim == r.dim && l.dim <= 1) && !(zero(&c.left) || zero(&c.right)) {
        return Err(at(format!("{} is {}: both sides are lengths or both are plain numbers", l.text, dim_said(l.dim))));
    }
    match (l.value, r.value) {
        (Some(x), Some(y)) if x.is_finite() && y.is_finite() => Ok(match c.op {
            Cmp::Gt => x > y,
            Cmp::Ge => x >= y,
            Cmp::Lt => x < y,
            Cmp::Le => x <= y,
        }),
        _ => Err(at("comes to no figure".into())),
    }
}

impl Shape {
    fn shape_fault(&self) -> Option<String> {
        if self.name.trim().is_empty() || self.name.trim() != self.name {
            return Some("a shape has to be called something, with no space at either end of its name".into());
        }
        if self.name.eq_ignore_ascii_case(BOX) {
            return Some(format!("{BOX:?} is the plain box and is not a shape to write over"));
        }
        if self.blocks.is_empty() {
            return Some(format!("shape {:?} has no blocks", self.name));
        }
        if self.blocks.len() > MOST_BLOCKS {
            return Some(format!("shape {:?} has {} blocks; a shape has at most {MOST_BLOCKS}", self.name, self.blocks.len()));
        }
        None
    }

    /// One block read, worked at the three sizes and held to the box (rulings 2 and 3).
    fn piece(&self, n: usize) -> Result<Compiled, String> {
        let piece = &self.blocks[n];
        let mut read: Vec<Expr> = Vec::new();
        for (what, text) in FIGURES.iter().zip(piece.figures()) {
            read.push(formula::parse(text).map_err(|why| format!("shape {:?}, block {}, {what}: {text:?} {why}", self.name, n + 1))?);
        }
        let figures: [Expr; 6] = read.try_into().map_err(|_| "six figures".to_string())?;
        let when = match piece.when.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
            Some(text) => Some(read_cond(text).map_err(|why| format!("shape {:?}, block {}, There when: {why}", self.name, n + 1))?),
            None => None,
        };
        let c = Compiled { made: piece.made, round: piece.round, figures, when };
        for (word, size) in CHECK_SIZES {
            let Some(b) = run_one(&self.name, n, &c, size)? else { continue };
            for (i, axis) in ["across", "back", "up"].into_iter().enumerate() {
                let (lo, hi) = if i == 2 { (0.0, size[2]) } else { (-size[i] / 2.0, size[i] / 2.0) };
                let out = (lo - b.lo[i]).max(b.hi[i] - hi);
                if out > REACH {
                    // Which way, so a person knows which figure to pull in (call 205).
                    let (low, high) = [("to the left", "to the right"), ("in front", "behind"), ("below", "above")][i];
                    let way = if lo - b.lo[i] >= b.hi[i] - hi { low } else { high };
                    let inches = format!("{:.3}", out * 12.0);
                    let inches = inches.trim_end_matches('0').trim_end_matches('.');
                    return Err(format!("shape {:?}, block {}: at the {word} size ({} x {} x {} ft) it reaches {out:.4} ft outside the box of the item's sizes, {axis} ({inches} in {way} the box); a shape is drawn inside its box", self.name, n + 1, size[0], size[1], size[2]));
                }
            }
        }
        Ok(c)
    }

    /// Every block that cannot be used, by its place in the shape (from 0) and why.
    pub fn faults(&self) -> Vec<(usize, String)> {
        if self.shape_fault().is_some() {
            return Vec::new();
        }
        (0..self.blocks.len()).filter_map(|n| self.piece(n).err().map(|why| (n, why))).collect()
    }

    /// Read every figure and condition. The error says which block and which figure.
    pub fn compile(&self) -> Result<Vec<Compiled>, String> {
        if let Some(why) = self.shape_fault() {
            return Err(why);
        }
        (0..self.blocks.len()).map(|n| self.piece(n)).collect()
    }

    /// Worked for a thing `size` feet wide, deep and high. A block whose condition does not hold at
    /// that size is not there.
    pub fn worked(&self, size: P3) -> Result<Vec<Block>, String> {
        let compiled = self.compile()?;
        run(&self.name, &compiled, size)
    }

    /// One block (by its place from 0) worked for a size: None where its "There when" does not hold at it.
    pub fn worked_block(&self, n: usize, size: P3) -> Result<Option<Block>, String> {
        let compiled = self.compile()?;
        let c = compiled.get(n).ok_or_else(|| format!("shape {:?} has no block {}", self.name, n + 1))?;
        run_one(&self.name, n, c, size)
    }

    /// Refuse what cannot be drawn.
    pub fn check(&self) -> Result<(), String> {
        self.compile().map(|_| ())
    }
}

/// One figure worked: a length in feet.
fn figure(shape: &str, n: usize, what: &str, expr: &Expr, size: P3) -> Result<f64, String> {
    let worked = formula::work(expr, &|name| lookup(name, size)).map_err(|why| format!("shape {shape:?}, block {}, {what}: {why}", n + 1))?;
    // A plain 0 is no distance in any unit, and is the one number that stands without a unit (ruling 2).
    let nothing = matches!(expr, Expr::Num(x) if *x == 0.0);
    if worked.dim != 1 && !nothing {
        return Err(format!("shape {shape:?}, block {}, {what}: {} comes out as {}; a figure of a shape is a length (write a fixed size with its unit: 4 in, 100 mm; only 0 stands without one)", n + 1, worked.text, dim_said(worked.dim)));
    }
    match worked.value {
        Some(x) if x.is_finite() => Ok(x),
        _ => Err(format!("shape {shape:?}, block {}, {what}: comes to no figure", n + 1)),
    }
}

/// One block worked for a size, or None where its condition does not hold.
fn run_one(name: &str, n: usize, c: &Compiled, size: P3) -> Result<Option<Block>, String> {
    if let Some(cond) = &c.when {
        if !holds(name, n, cond, size)? {
            return Ok(None);
        }
    }
    let mut f = [0.0; 6];
    for i in 0..6 {
        f[i] = figure(name, n, FIGURES[i], &c.figures[i], size)?;
    }
    // The thing's own frame: y from its front (less) to its back (more), from the
    // middle. A figure "back" is from the back toward the front.
    let y = |b: f64| size[1] / 2.0 - b;
    let (y0, y1) = (y(f[2]), y(f[3]));
    Ok(Some(Block { lo: [f[0].min(f[1]), y0.min(y1), f[4].min(f[5])], hi: [f[0].max(f[1]), y0.max(y1), f[4].max(f[5])], round: c.round, made: c.made }))
}

fn run(name: &str, compiled: &[Compiled], size: P3) -> Result<Vec<Block>, String> {
    let mut out = Vec::with_capacity(compiled.len());
    for (n, c) in compiled.iter().enumerate() {
        out.extend(run_one(name, n, c, size)?);
    }
    Ok(out)
}

/// The starter shapes, in the order the table gives them.
pub fn starter() -> &'static [Shape] {
    static SHAPES: OnceLock<Vec<Shape>> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let rows = csv_rows(crate::tables::STARTER_SHAPES);
        crate::tables::shapes_from_rows(&rows, None).map(|(shapes, unused)| {
            assert!(unused.is_empty(), "the starter shapes have rows that cannot be read: {unused:?}");
            shapes
        }).unwrap_or_else(|why| panic!("the starter shapes cannot be read: {why}"))
    })
}

fn starter_compiled() -> &'static [(String, Vec<Compiled>)] {
    static COMPILED: OnceLock<Vec<(String, Vec<Compiled>)>> = OnceLock::new();
    COMPILED.get_or_init(|| starter().iter().map(|s| (s.name.clone(), s.compile().unwrap_or_else(|why| panic!("a starter shape cannot be worked: {why}")))).collect())
}

/// The starter shape of that name (however it is capitalised), if there is one.
pub fn starter_named(name: &str) -> Option<&'static Shape> {
    starter().iter().find(|s| s.name.eq_ignore_ascii_case(name.trim()))
}

/// The blocks of a starter shape for a thing of `size`, worked from its rows. None where
/// the program has no starter shape of that name.
pub fn starter_blocks(name: &str, size: P3) -> Option<Vec<Block>> {
    let (shape, compiled) = starter_compiled().iter().find(|(n, _)| n.eq_ignore_ascii_case(name.trim()))?;
    Some(run(shape, compiled, size).unwrap_or_else(|why| panic!("a starter shape cannot be worked: {why}")))
}

/// A shape by its name: the job's own first, then the starter's.
pub fn find<'a>(job: &'a [Shape], name: &str) -> Option<&'a Shape> {
    let name = name.trim();
    job.iter().find(|s| s.name.eq_ignore_ascii_case(name)).or_else(|| starter_named(name))
}

/// What an item says it is drawn as, checked against the shapes there are: "box", one of
/// the thirteen the program draws, or a shape the job carries. Anything else is refused,
/// naming the shape and what there is.
pub fn check_drawn_as(job: &[Shape], said: &str) -> Result<(), String> {
    let said = said.trim();
    if said.eq_ignore_ascii_case(BOX) || super::forms::Form::named(said).is_some() || find(job, said).is_some() {
        return Ok(());
    }
    let mut there: Vec<String> = vec![BOX.to_string()];
    there.extend(super::forms::Form::ALL.map(|f| f.name().to_string()));
    for s in job {
        if !there.iter().any(|t| t.eq_ignore_ascii_case(&s.name)) {
            there.push(s.name.clone());
        }
    }
    Err(format!("{said:?} is not something a thing can be drawn as. There are: {}", there.join(", ")))
}

/// Rows of a CSV table, as the starter table is written: a cell with a comma is in quotes,
/// a quote inside is doubled, a cell a spreadsheet would run as a formula has an apostrophe
/// in front (`'-width / 2`), which comes off here as the table reader takes it off.
pub fn csv_rows(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let (mut row, mut cell, mut quoted) = (Vec::<String>::new(), String::new(), false);
    let mut chars = text.chars().peekable();
    let finish = |row: &mut Vec<String>, cell: &mut String| {
        let c = std::mem::take(cell);
        let runs = |t: &str| t.starts_with(['=', '+', '-', '@']) && t.trim().parse::<f64>().is_err();
        let c = match c.strip_prefix('\'') {
            Some(rest) if runs(rest) => rest.to_string(),
            _ => c,
        };
        row.push(c);
    };
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                chars.next();
                cell.push('"');
            }
            ('"', true) => quoted = false,
            ('"', false) if cell.is_empty() => quoted = true,
            (',', false) => finish(&mut row, &mut cell),
            ('\r', false) => {}
            ('\n', false) => {
                finish(&mut row, &mut cell);
                if row.iter().any(|c| !c.is_empty()) {
                    rows.push(std::mem::take(&mut row));
                } else {
                    row.clear();
                }
            }
            (c, _) => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() {
        finish(&mut row, &mut cell);
        rows.push(row);
    }
    rows
}
