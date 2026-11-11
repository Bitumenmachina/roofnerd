//! The formula on a cost line.
//!
//! A line's quantity is a formula the estimator can read and edit on the line:
//! `plan_length * stretch_out`, `plan_length * sides / spacing`. Its names are the
//! element's measures, the element's own entered dimensions, and the values entered on
//! the line. The four operations and brackets; nothing else.
//!
//! Three things a spreadsheet cell does not do, which is why this is ours:
//!
//! * **Units.** Every length comes in as feet, whatever it was entered in, and the worked
//!   formula says where (`stretch_out ÷ 12`). Lengths multiply to areas. A formula that
//!   adds a length to an area, or comes out as a volume, is refused.
//! * **Status.** The result is as good as its worst operand.
//! * **No number from nothing.** An Unknown operand leaves no result. It is never zero.

use crate::value::Status;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Name(String),
    Neg(Box<Expr>),
    Bin(Box<Expr>, char, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Op(char),
    Open,
    Close,
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
                out.push(Tok::Name(chars[start..i].iter().collect()));
            }
            other => return Err(format!("{other:?} is not something a formula can hold; it takes names, numbers, + - * / and brackets")),
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
                Ok(Expr::Num(x))
            }
            Some(Tok::Name(n)) => {
                self.at += 1;
                Ok(Expr::Name(n))
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
        Some(_) => Err("has two things side by side with nothing between them".into()),
    }
}

/// Every name the formula uses, once each, in the order they first appear.
pub fn names(expr: &Expr) -> Vec<String> {
    fn walk(e: &Expr, out: &mut Vec<String>) {
        match e {
            Expr::Num(_) => {}
            Expr::Name(n) => {
                if !out.contains(n) {
                    out.push(n.clone());
                }
            }
            Expr::Neg(a) => walk(a, out),
            Expr::Bin(a, _, b) => {
                walk(a, out);
                walk(b, out);
            }
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
                            return Err(format!("divides by {}, which is zero", b.text));
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
        _ => "neither a number, a length nor an area",
    }
}

/// The unit a result of this dimension is counted in, or why a line cannot be counted
/// in it.
pub fn unit_of(dim: i32) -> Result<&'static str, String> {
    match dim {
        0 => Ok("EA"),
        1 => Ok("LF"),
        2 => Ok("SF"),
        3 => Err("comes out as a volume (feet × feet × feet); a line is a count, a length or an area".into()),
        d if d < 0 => Err("comes out as one over a length; a line is a count, a length or an area".into()),
        _ => Err("comes out as more than a volume; a line is a count, a length or an area".into()),
    }
}
