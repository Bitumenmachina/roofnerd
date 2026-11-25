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
    /// A length written with its unit as a person writes it: `1/2 in`, `5 1/2 in`, `3'`, `6"` (call 213 point 8). The unit is one of `UNITS`.
    Len(f64, &'static str),
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
            // The marks a builder writes feet and inches with, after a number: 3' is 3 ft and 6" is 6 in (call 213 point 8).
            '\'' | '"' if matches!(out.last(), Some(Tok::Num(_))) => {
                out.push(Tok::Name(if c == '\'' { "ft" } else { "in" }.to_string()));
                i += 1;
            }
            other => return Err(format!("{other} is not something a formula can hold; it takes names, numbers, + - * / brackets, min and max; a length is written with its unit, like 4 in, 1/2 in, 3 ft 6 in or 100 mm")),
        }
    }
    Ok(lengths(out))
}

/// A length as it is written by hand, read from the tokens (call 213 point 8). A number, a slash, a number and a unit is a fraction of
/// that unit (`1/2 in`); a whole number before one is added (`5 1/2 in`); a number and a unit is a length. Nothing that read before reads
/// otherwise: a fraction with a `*` or a `/` beside it (`plan_length * 1 / 2 in`, which was a number over a length) is left as it was,
/// and a number and a unit with nothing between them is what it always was.
fn lengths(toks: Vec<Tok>) -> Vec<Tok> {
    let mut out: Vec<Tok> = Vec::with_capacity(toks.len());
    let mut i = 0;
    let is_unit = |t: Option<&Tok>| match t {
        Some(Tok::Name(w)) => unit_named(w).map(|(u, _, _)| u),
        _ => None,
    };
    let times_or_over = |t: Option<&Tok>| matches!(t, Some(Tok::Op('*' | '/')));
    while i < toks.len() {
        // A whole number, a fraction and a unit: 5 1/2 in. (Two numbers side by side were never a figure.)
        if let (Tok::Num(w), Some(Tok::Num(a)), Some(Tok::Op('/')), Some(Tok::Num(b))) = (&toks[i], toks.get(i + 1), toks.get(i + 2), toks.get(i + 3)) {
            if let Some(unit) = is_unit(toks.get(i + 4)) {
                if *b != 0.0 {
                    out.push(Tok::Len(w + a / b, unit));
                    i += 5;
                    continue;
                }
            }
        }
        // A fraction of a unit: 1/2 in.
        if let (Tok::Num(a), Some(Tok::Op('/')), Some(Tok::Num(b))) = (&toks[i], toks.get(i + 1), toks.get(i + 2)) {
            if let Some(unit) = is_unit(toks.get(i + 3)) {
                let before = out.last().map(|t| matches!(t, Tok::Op('*' | '/'))).unwrap_or(false);
                if *b != 0.0 && !before && !times_or_over(toks.get(i + 4)) {
                    out.push(Tok::Len(a / b, unit));
                    i += 4;
                    continue;
                }
            }
        }
        // A number and a unit: a length (what the parser read before, said once here).
        if let Tok::Num(x) = &toks[i] {
            if let Some(unit) = is_unit(toks.get(i + 1)) {
                out.push(Tok::Len(*x, unit));
                i += 2;
                continue;
            }
        }
        out.push(toks[i].clone());
        i += 1;
    }
    out
}

/// Whether `next` is a smaller unit of the same kind of length than `last`, so that `3 ft 6 in` is 3 ft and 6 in: feet then inches; meters
/// then centimeters or millimeters; centimeters then millimeters.
fn smaller_than(last: &str, next: &str) -> bool {
    matches!((last, next), ("ft", "in") | ("m", "cm") | ("m", "mm") | ("cm", "mm"))
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
            Some(Tok::Len(x, unit)) => {
                self.at += 1;
                // Feet and inches side by side add (3 ft 6 in); a minus between them is a minus.
                let mut sum = Expr::Sized(x, unit);
                let mut last = unit;
                while let Some(Tok::Len(y, next)) = self.peek().cloned() {
                    if !smaller_than(last, next) {
                        break;
                    }
                    self.at += 1;
                    sum = Expr::Bin(Box::new(sum), '+', Box::new(Expr::Sized(y, next)));
                    last = next;
                }
                Ok(sum)
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
                        // Nothing in it, or nothing after a comma: said as what it takes (call 206 point 11).
                        let empty = |p: &Parser| matches!(p.peek(), Some(Tok::Close));
                        if empty(self) {
                            return Err(format!("{}( takes two figures or more, with a comma between them, like {}(4 in, height); there is nothing in it", pick.word(), pick.word()));
                        }
                        let mut args = vec![self.sum()?];
                        while matches!(self.peek(), Some(Tok::Comma)) {
                            self.at += 1;
                            if empty(self) {
                                return Err(format!("{}( takes two figures or more, with a comma between them, like {}(4 in, height); there is nothing after the last comma", pick.word(), pick.word()));
                            }
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

/// A formula as it is typed: names, numbers with their units, `+ - * /` and `min( )`, with the brackets it needs. Refusals quote
/// this and not the worked text (`height \u{00D7} 0.5 \u{00F7} 12`), which is the program's own rewriting (call 206 point 11).
pub fn typed(e: &Expr) -> String {
    fn rank(e: &Expr) -> u8 {
        match e {
            Expr::Bin(_, '+' | '-', _) => 1,
            Expr::Bin(..) => 2,
            Expr::Neg(_) => 3,
            _ => 4,
        }
    }
    match e {
        Expr::Num(x) => num_text(*x),
        Expr::Name(n) => n.clone(),
        Expr::Sized(x, unit) => format!("{} {}", num_text(*x), unit),
        Expr::Neg(a) => if rank(a) < 4 { format!("-({})", typed(a)) } else { format!("-{}", typed(a)) },
        Expr::Pick(pick, args) => format!("{}({})", pick.word(), args.iter().map(typed).collect::<Vec<_>>().join(", ")),
        Expr::Bin(a, op, b) => {
            let mine = rank(e);
            let left = if rank(a) < mine { format!("({})", typed(a)) } else { typed(a) };
            let right = if rank(b) < mine || (rank(b) == mine && matches!(op, '-' | '/')) { format!("({})", typed(b)) } else { typed(b) };
            format!("{left} {op} {right}")
        }
    }
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
                if let Some(at) = parts.iter().position(|p| p.dim != first.dim) {
                    let odd = &parts[at];
                    return Err(format!("takes the {} of what cannot be compared: {} is {} and {} is {}", if *pick == Pick::Min { "lesser" } else { "greater" }, typed(&args[0]), dim_word(first.dim), typed(&args[at]), dim_word(odd.dim)));
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
            Expr::Bin(ea, op, eb) => {
                let (a, b) = (go(ea, lookup)?, go(eb, lookup)?);
                let status = a.status.max(b.status);
                let both = a.value.zip(b.value);
                match op {
                    '+' | '-' => {
                        if a.dim != b.dim {
                            return Err(format!("{} what cannot be {}: {} is {} and {} is {}", if *op == '+' { "adds" } else { "takes away" }, if *op == '+' { "added" } else { "taken away" }, typed(ea), dim_word(a.dim), typed(eb), dim_word(b.dim)));
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
                            return Err(format!("divides by {}, which is nothing: a quantity cannot be divided by zero", typed(eb)));
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
