//! The formula on a cost line.
//!
//! A line's quantity is a formula the estimator can read and edit on the line:
//! `plan_length * stretch_out`, `plan_length * sides / spacing`. Its names are the
//! element's measures, the element's own entered dimensions, and the values entered on
//! the line. The four operations and brackets, the lesser or the greater of two or more
//! (`min(net_area, 100 SF)`, `max(a, b)`: call 198), and a fixed size written with its
//! unit (`4 in`, `100 mm`, `3 ft`, `20 LF`, `100 SF`).
//!
//! Three things a spreadsheet cell does not do, which is why this is ours:
//!
//! * **Units.** Every length comes in as feet, whatever it was entered in, and the worked
//!   formula says where (`stretch_out ÷ 12`). Lengths multiply to areas, and to volumes,
//!   which come out in cubic yards with the ÷ 27 written in (call 130). A formula that
//!   adds a length to an area is refused.
//! * **Status.** The result is as good as its worst operand.
//! * **No number from nothing.** An Unknown operand leaves no result. It is never zero.

use crate::value::Status;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Name(String),
    Neg(Box<Expr>),
    Bin(Box<Expr>, char, Box<Expr>),
    /// A fixed size written with its unit: `4 in` is the number and the unit as written,
    /// worked in feet (or, for an area, square feet).
    Sized(f64, &'static str),
    /// The lesser or the greater of two or more: `min(a, b)`, `max(a, b)`.
    Pick(Pick, Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    Min,
    Max,
}

impl Pick {
    fn word(self) -> &'static str {
        match self {
            Pick::Min => "min",
            Pick::Max => "max",
        }
    }
}

/// The units a fixed size is written in: the word, how many of it make a foot (or, for
/// an area, a square foot), and the dimension of what it measures (1 a length, 2 an area).
const UNITS: [(&str, f64, i32); 7] = [("in", 12.0, 1), ("ft", 1.0, 1), ("mm", 304.8, 1), ("cm", 30.48, 1), ("m", 0.3048, 1), ("lf", 1.0, 1), ("sf", 1.0, 2)];

fn unit_named(word: &str) -> Option<(&'static str, f64, i32)> {
    let lower = word.to_ascii_lowercase();
    UNITS.iter().find(|(u, _, _)| *u == lower).copied()
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Op(char),
    Open,
    Close,
    Comma,
}

fn tokens(text: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' => i += 1,
            '+' => {
                out.push(Tok::Op('+'));
                i += 1;
            }
            // The minus a worked formula is written with reads back as a minus.
            '-' | '\u{2212}' => {
                out.push(Tok::Op('-'));
                i += 1;
            }
            '*' | '\u{00D7}' => {
                out.push(Tok::Op('*'));
                i += 1;
            }
            '/' | '\u{00F7}' => {
                out.push(Tok::Op('/'));
                i += 1;
            }
            '(' => {
                out.push(Tok::Open);
                i += 1;
            }
            ')' => {
                out.push(Tok::Close);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if word.matches('.').count() > 1 || word == "." {
                    return Err(format!("{word:?} is not a number"));
                }
                let x: f64 = word.parse().map_err(|_| format!("{word:?} is not a number"))?;
                if !x.is_finite() {
                    return Err(format!("{word:?} is not a usable number"));
                }
                out.push(Tok::Num(x));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let after_a_figure = matches!(out.last(), Some(Tok::Num(_) | Tok::Name(_) | Tok::Close));
                match word.to_ascii_lowercase().as_str() {
                    // "face_area x 0.10": times, as a takeoff sheet writes it (call 124).
                    "x" if after_a_figure => out.push(Tok::Op('*')),
                    // Priced each: one.
                    "each" | "ea" => out.push(Tok::Num(1.0)),
                    _ => out.push(Tok::Name(word)),
                }
            }
            other => return Err(format!("{other:?} is not something a formula can hold; it takes names, numbers, + - * / brackets, min and max")),
        }
    }
    Ok(out)
}

/// The most names, numbers, signs and brackets a formula may hold.
pub const MAX_TOKENS: usize = 200;

struct Parser {
    toks: Vec<Tok>,
    at: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }
    fn sum(&mut self) -> Result<Expr, String> {
        let mut left = self.product()?;
        while let Some(Tok::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.at += 1;
            left = Expr::Bin(Box::new(left), op, Box::new(self.product()?));
        }
        Ok(left)
    }
    fn product(&mut self) -> Result<Expr, String> {
        let mut left = self.atom()?;
        while let Some(Tok::Op(op @ ('*' | '/'))) = self.peek().cloned() {
            self.at += 1;
            left = Expr::Bin(Box::new(left), op, Box::new(self.atom()?));
        }
        Ok(left)
    }
    fn atom(&mut self) -> Result<Expr, String> {
        self.depth += 1;
        if self.depth > 64 {
            return Err("is nested too deeply to read".into());
        }
        let out = match self.peek().cloned() {
            Some(Tok::Num(x)) => {
                self.at += 1;
                // A fixed size with its unit. A name after a number was always an error,
                // so no formula written before reads differently.
                match self.peek().cloned() {
                    Some(Tok::Name(word)) => match unit_named(&word) {
                        Some((unit, _, _)) => {
                            self.at += 1;
                            Ok(Expr::Sized(x, unit))
                        }
                        None => Ok(Expr::Num(x)),
                    },
                    _ => Ok(Expr::Num(x)),
                }
            }
            Some(Tok::Name(n)) => {
                self.at += 1;
                let pick = match n.to_ascii_lowercase().as_str() {
                    "min" => Some(Pick::Min),
                    "max" => Some(Pick::Max),
                    _ => None,
                };
                match (pick, self.peek()) {
                    // `min` and `max` are the lesser and the greater only with a bracket after
                    // them; alone they are names, as they always were.
                    (Some(pick), Some(Tok::Open)) => {
                        self.at += 1;
                        let mut args = vec![self.sum()?];
                        while matches!(self.peek(), Some(Tok::Comma)) {
                            self.at += 1;
                            args.push(self.sum()?);
                        }
                        match self.peek() {
                            Some(Tok::Close) if args.len() >= 2 => {
                                self.at += 1;
                                Ok(Expr::Pick(pick, args))
                            }
                            Some(Tok::Close) => Err(format!("{}( takes two figures or more, with a comma between them", pick.word())),
                            _ => Err(format!("has {}( with its bracket never closed", pick.word())),
                        }
                    }
                    _ => Ok(Expr::Name(n)),
                }
            }
            Some(Tok::Op('-')) => {
                self.at += 1;
                Ok(Expr::Neg(Box::new(self.atom()?)))
            }
            Some(Tok::Open) => {
                self.at += 1;
                let inner = self.sum()?;
                match self.peek() {
                    Some(Tok::Close) => {
                        self.at += 1;
                        Ok(inner)
                    }
                    _ => Err("has a bracket that is never closed".into()),
                }
            }
            Some(Tok::Close) => Err("has a closing bracket with nothing open".into()),
            Some(Tok::Comma) => Err("has a comma outside min( ) or max( )".into()),
            Some(Tok::Op(op)) => Err(format!("has {op:?} with nothing before it")),
            None => Err("stops where a name or a number should be".into()),
        };
        self.depth -= 1;
        out
    }
}

/// Read a formula. The error is a clause that follows the formula's own text.
pub fn parse(text: &str) -> Result<Expr, String> {
    if text.trim().is_empty() {
        return Err("is blank".into());
    }
    // A name can be typed as its words: `plan length * stretch out` is `plan_length *
    // stretch_out`. Two names never stand side by side in a formula, so words in a row,
    // with only spaces between them, are one name.
    let chars: Vec<char> = text.chars().collect();
    let mut joined = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ' ' {
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let word = |c: char| c.is_ascii_alphabetic() || c == '_';
            // An x standing alone between two things is times, as a takeoff sheet
            // writes it ("face area x 0.10"), and is not a word of a name (call 124).
            if i > 0 && j + 1 < chars.len() && matches!(chars[j], 'x' | 'X') && chars[j + 1] == ' ' {
                joined.push('*');
                i = j + 1;
                continue;
            }
            if i > 0 && j < chars.len() && word(chars[i - 1]) && word(chars[j]) {
                joined.push('_');
                i = j;
                continue;
            }
        }
        joined.push(chars[i]);
        i += 1;
    }
    let toks = tokens(&joined)?;
    // A line's formula is something a person reads on the line. Past this it is not, and
    // working it would go deeper than the program can.
    if toks.len() > MAX_TOKENS {
        return Err(format!("is {} names, numbers and signs long; a line's formula holds at most {MAX_TOKENS}", toks.len()));
    }
    let mut p = Parser { toks, at: 0, depth: 0 };
    let expr = p.sum()?;
    match p.peek() {
        None => Ok(expr),
        Some(Tok::Close) => Err("has a closing bracket with nothing open".into()),
        Some(Tok::Comma) => Err("has a comma outside min( ) or max( )".into()),
        Some(_) => Err("has two things side by side with nothing between them: put +, -, x or / between them".into()),
    }
}

/// The start of a wall's figure that takes a height from the line: `net_area_below_wainscot`
/// is the wall's net area below the length the line enters as `wainscot` (call 179, C2).
pub const BELOW: &str = "net_area_below_";

/// The start of a wall's corner-board figure that takes a height from the line:
/// `corner_board_length_above_wainscot` is the corner boards' length above the length the line
/// enters as `wainscot` (call 194), as `net_area_below_wainscot` is the face below it.
pub const ABOVE: &str = "corner_board_length_above_";

/// Is this name a figure of a wall that takes a height from the line, and what is the entry?
pub fn height_entry(name: &str) -> Option<&str> {
    name.strip_prefix(BELOW).or_else(|| name.strip_prefix(ABOVE)).filter(|i| !i.is_empty())
}

/// Every name the formula uses, once each, in the order they first appear. A figure that
/// takes a height from the line (`net_area_below_wainscot`) uses that entry (`wainscot`) too.
pub fn names(expr: &Expr) -> Vec<String> {
    fn walk(e: &Expr, out: &mut Vec<String>) {
        match e {
            Expr::Num(_) => {}
            Expr::Name(n) => {
                if !out.contains(n) {
                    out.push(n.clone());
                }
                if let Some(input) = height_entry(n) {
                    if !out.iter().any(|o| o == input) {
                        out.push(input.to_string());
                    }
                }
            }
            Expr::Neg(a) => walk(a, out),
            Expr::Bin(a, _, b) => {
                walk(a, out);
                walk(b, out);
            }
            Expr::Sized(..) => {}
            Expr::Pick(_, args) => args.iter().for_each(|a| walk(a, out)),
        }
    }
    let mut out = Vec::new();
    walk(expr, &mut out);
    out
}

/// What a name stands for when the formula is worked.
#[derive(Clone, Debug)]
pub struct Operand {
    /// In feet to the power `dim`. None when nobody has it.
    pub value: Option<f64>,
    /// 0 for a plain number or a count, 1 for a length, 2 for an area.
    pub dim: i32,
    pub status: Status,
    /// How it reads in the worked formula: its name, or its name with the conversion that
    /// brought it to feet (`stretch_out ÷ 12`).
    pub shown: String,
}

/// A formula worked out.
#[derive(Clone, Debug)]
pub struct Worked {
    pub value: Option<f64>,
    pub dim: i32,
    pub status: Status,
    /// The formula as worked, conversions written in.
    pub text: String,
}

fn num_text(x: f64) -> String {
    format!("{x}")
}

/// Work a formula out. `lookup` says what each name stands for, or why it stands for
/// nothing. An error is a reason the formula cannot be worked at all (a name that is not
/// there, a length added to an area); an Unknown operand is not an error, it gives a
/// result with no value.
pub fn work(expr: &Expr, lookup: &dyn Fn(&str) -> Result<Operand, String>) -> Result<Worked, String> {
    // `tight` is whether the text can sit beside × or ÷ without brackets, and `atom`
    // whether it can sit after ÷ or − without them.
    struct Part {
        value: Option<f64>,
        dim: i32,
        status: Status,
        text: String,
        tight: bool,
        atom: bool,
    }
    fn go(e: &Expr, lookup: &dyn Fn(&str) -> Result<Operand, String>) -> Result<Part, String> {
        match e {
            Expr::Num(x) => Ok(Part { value: Some(*x), dim: 0, status: Status::Known, text: num_text(*x), tight: true, atom: true }),
            Expr::Name(n) => {
                let o = lookup(n)?;
                let atom = o.shown == *n || o.shown.ends_with(')');
                Ok(Part { value: o.value, dim: o.dim, status: if o.value.is_none() { Status::Unknown } else { o.status }, text: o.shown, tight: true, atom })
            }
            Expr::Sized(x, unit) => {
                let (_, per, dim) = unit_named(unit).unwrap_or((unit, 1.0, 1));
                // Lengths come in as feet and the worked formula says where (`4 ÷ 12`).
                if per == 1.0 {
                    let shown = if dim == 2 { unit.to_ascii_uppercase() } else if *unit == "lf" { "LF".to_string() } else { unit.to_string() };
                    Ok(Part { value: Some(*x), dim, status: Status::Known, text: format!("{} {shown}", num_text(*x)), tight: true, atom: true })
                } else {
                    Ok(Part { value: Some(x / per), dim, status: Status::Known, text: format!("{} \u{00F7} {}", num_text(*x), num_text(per)), tight: true, atom: false })
                }
            }
            Expr::Pick(pick, args) => {
                let parts = args.iter().map(|a| go(a, lookup)).collect::<Result<Vec<_>, _>>()?;
                let first = &parts[0];
                if let Some(odd) = parts.iter().find(|p| p.dim != first.dim) {
                    return Err(format!("takes the {} of what cannot be compared: {} is {} and {} is {}", if *pick == Pick::Min { "lesser" } else { "greater" }, first.text, dim_word(first.dim), odd.text, dim_word(odd.dim)));
                }
                // As good as its worst operand, and no value where any operand has none.
                let status = parts.iter().map(|p| p.status).max().unwrap_or(Status::Known);
                let value = parts.iter().map(|p| p.value).collect::<Option<Vec<f64>>>().map(|xs| xs.into_iter().reduce(|a, b| if *pick == Pick::Min { a.min(b) } else { a.max(b) }).unwrap_or(0.0));
                let text = format!("{}({})", pick.word(), parts.iter().map(|p| p.text.clone()).collect::<Vec<_>>().join(", "));
                Ok(Part { value, dim: first.dim, status, text, tight: true, atom: true })
            }
            Expr::Neg(a) => {
                let a = go(a, lookup)?;
                let text = if a.atom { format!("-{}", a.text) } else { format!("-({})", a.text) };
                Ok(Part { value: a.value.map(|x| -x), dim: a.dim, status: a.status, text, tight: true, atom: false })
            }
            Expr::Bin(a, op, b) => {
                let (a, b) = (go(a, lookup)?, go(b, lookup)?);
                let status = a.status.max(b.status);
                let both = a.value.zip(b.value);
                match op {
                    '+' | '-' => {
                        if a.dim != b.dim {
                            return Err(format!("{} what cannot be {}: {} is {} and {} is {}", if *op == '+' { "adds" } else { "takes away" }, if *op == '+' { "added" } else { "taken away" }, a.text, dim_word(a.dim), b.text, dim_word(b.dim)));
                        }
                        let right = if *op == '-' && !b.tight { format!("({})", b.text) } else { b.text.clone() };
                        let sign = if *op == '+' { '+' } else { '\u{2212}' };
                        Ok(Part { value: both.map(|(x, y)| if *op == '+' { x + y } else { x - y }), dim: a.dim, status, text: format!("{} {sign} {right}", a.text), tight: false, atom: false })
                    }
                    '*' => {
                        let left = if a.tight { a.text.clone() } else { format!("({})", a.text) };
                        let right = if b.tight { b.text.clone() } else { format!("({})", b.text) };
                        Ok(Part { value: both.map(|(x, y)| x * y), dim: a.dim + b.dim, status, text: format!("{left} \u{00D7} {right}"), tight: true, atom: false })
                    }
                    _ => {
                        if b.value == Some(0.0) {
                            return Err(format!("divides by {}, which is nothing: a quantity cannot be divided by zero", b.text));
                        }
                        let left = if a.tight { a.text.clone() } else { format!("({})", a.text) };
                        let right = if b.atom { b.text.clone() } else { format!("({})", b.text) };
                        Ok(Part { value: both.map(|(x, y)| x / y), dim: a.dim - b.dim, status, text: format!("{left} \u{00F7} {right}"), tight: true, atom: false })
                    }
                }
            }
        }
    }
    let p = go(expr, lookup)?;
    Ok(Worked { value: p.value, dim: p.dim, status: p.status, text: p.text })
}

fn dim_word(dim: i32) -> &'static str {
    match dim {
        0 => "a plain number",
        1 => "a length",
        2 => "an area",
        3 => "a volume",
        _ => "neither a number, a length, an area nor a volume",
    }
}

/// The unit a result of this dimension is counted in, or why a line cannot be counted
/// in it. A volume is worked in cubic feet and counted in cubic yards, as concrete,
/// gravel, fill and excavation are bought (call 130): see `in_yards`.
pub fn unit_of(dim: i32) -> Result<&'static str, String> {
    match dim {
        0 => Ok("EA"),
        1 => Ok("LF"),
        2 => Ok("SF"),
        3 => Ok("CY"),
        d if d < 0 => Err("comes out as one over a length; a line is a count, a length, an area or a volume".into()),
        _ => Err("comes out as more than a volume; a line is a count, a length, an area or a volume".into()),
    }
}

/// Cubic feet to the cubic yard.
pub const CF_PER_CY: f64 = 27.0;

/// Does the formula divide by this number anywhere in it?
pub fn divides_by(expr: &Expr, n: f64) -> bool {
    match expr {
        Expr::Bin(a, op, b) => (*op == '/' && matches!(**b, Expr::Num(x) if x == n)) || divides_by(a, n) || divides_by(b, n),
        Expr::Neg(a) => divides_by(a, n),
        Expr::Pick(_, args) => args.iter().any(|a| divides_by(a, n)),
        _ => false,
    }
}

/// A formula worked to a volume, in cubic feet, given in cubic yards: the value divided
/// by 27 and the worked formula saying so. Any other result is given back as it is.
pub fn in_yards(expr: &Expr, worked: Worked) -> Worked {
    if worked.dim != 3 {
        return worked;
    }
    // A sum is bracketed before it is divided; a product or a single name is not.
    let text = if matches!(expr, Expr::Bin(_, '+' | '-', _)) { format!("({}) \u{00F7} 27", worked.text) } else { format!("{} \u{00F7} 27", worked.text) };
    Worked { value: worked.value.map(|v| v / CF_PER_CY), text, ..worked }
}
