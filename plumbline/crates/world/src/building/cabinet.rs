//! A cabinet by rule (call 215): the letters of a cabinet's code (B, SB, W, T, F) and how its
//! number reads give the box, from a table the person owns.
//!
//! A rule is one family of boxes. Its **code** is the letters (`B`); **what it is** says what
//! the box is to a cabinet maker (a base, a wall box, a tall box, a filler); **number reads**
//! says how the figures after the letters are read (`B42` reads the width in inches: 42 in
//! wide; `W3342` reads four digits, the width then the height: 33 in wide, 42 in high); width,
//! depth and height are each a length or "from the number"; **stands** is on the floor or hung
//! so far up; **drawn as** is a shape; **stocked widths** are the widths the trade stocks.
//! Every rule is Assumed: "the trade's stock sizes by habit; the maker's catalog decides".
//!
//! The thirty-odd boxes of the tool chest are what the starter rules make
//! (`library/cabinet-rules.csv`, held to it by a test). A job carries only the rules it has
//! brought in or changed (`Job::cabinet_rules`); the program's own are never saved in it, so a job
//! that never touched them is saved as it always was.
//!
//! Ours. BIMCraft's families have types, each with a value for each named parameter
//! (`family.rs` `types`, `parameters`, `resolve`): a size comes from a parameter, and a type's
//! name (`1200 x 600`) is only a label that nothing reads. Here the code carries its size, and a
//! rule says how to read it.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::building::Item;
use crate::components::Kind;
use crate::units::{self, Len};
use crate::value::{Status, Val};

/// The headings of the table of rules as this program writes them.
pub const HEADINGS: [&str; 11] = ["Code", "What it is", "Number reads", "Width", "Depth", "Height", "Stands", "Drawn as", "Stocked widths", "How sure", "Where from"];

/// What a width or a height says when the code's number gives it.
pub const FROM_NUMBER: &str = "from the number";
/// What a rule says of a box that stands on the floor.
pub const ON_THE_FLOOR: &str = "on the floor";
/// The widest a box is made, in inches: a sheet's length (call 225).
pub const WIDEST: u32 = 96;
/// The most rules of its own a job carries.
pub const MOST_RULES: usize = 200;

/// What a box is to a cabinet maker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Is {
    Base,
    WallBox,
    TallBox,
    Filler,
}

impl Is {
    pub const ALL: [Is; 4] = [Is::Base, Is::WallBox, Is::TallBox, Is::Filler];

    /// The words the table writes it in.
    pub fn word(self) -> &'static str {
        match self {
            Is::Base => "base",
            Is::WallBox => "wall box",
            Is::TallBox => "tall box",
            Is::Filler => "filler",
        }
    }

    pub fn named(text: &str) -> Option<Is> {
        let t = text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
        Is::ALL.into_iter().find(|i| i.word() == t)
    }

    /// With "a" before it.
    pub fn a(self) -> &'static str {
        match self {
            Is::Base => "a base",
            Is::WallBox => "a wall box",
            Is::TallBox => "a tall box",
            Is::Filler => "a filler",
        }
    }
}

/// How the figures after a code's letters read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reads {
    /// The width in inches (`B42`: 42 in wide); one to three digits.
    Width,
    /// Four digits, two for the width then two for the height, in inches (`W3342`: 33 in wide, 42 in high).
    WidthThenHeight,
}

impl Reads {
    pub const ALL: [Reads; 2] = [Reads::Width, Reads::WidthThenHeight];

    pub fn word(self) -> &'static str {
        match self {
            Reads::Width => "the width",
            Reads::WidthThenHeight => "the width then the height",
        }
    }

    pub fn named(text: &str) -> Option<Reads> {
        let t = text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
        Reads::ALL.into_iter().find(|r| r.word() == t)
    }
}

/// One family of boxes: how the code with these letters makes a box.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// The letters, in capitals: `B`, `SB`, `W`.
    pub code: String,
    pub is: Is,
    pub reads: Reads,
    /// A length (`24 in`), or None where the number gives it. The width is always the number's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    /// How far up its foot hangs (`54 in`). None: it stands on the floor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hung: Option<String>,
    /// A shape's name, or "box".
    pub drawn_as: String,
    /// The widths the trade stocks, in inches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stocked: Vec<u32>,
    pub status: Status,
    pub source: String,
}

/// A box made by a rule: its name, its item, and what was said of it.
#[derive(Clone, Debug)]
pub struct Made {
    pub name: String,
    pub item: Item,
    /// Its width is none the trade stocks: a made-to-order box.
    pub not_stocked: bool,
    /// What a person is told: which rule, its sizes, and a width not stocked.
    pub said: String,
}

/// What asking for a code the chest has not came to.
pub enum Asked {
    /// The words are not a code (letters then figures): the chest has no such item.
    NotACode,
    /// A code a rule reads, made.
    Made(Made),
    /// A code, but no rule reads it, or its number does not read: why, in a cabinet maker's words.
    Refused(String),
}

fn length_ok(what: &str, text: &str) -> Result<f64, String> {
    let (len, unit) = units::parse_length(text).map_err(|why| format!("its {what}: {why}"))?;
    let feet = len.feet(unit).map_err(|why| format!("its {what} {why}"))?;
    if !(feet.is_finite() && feet > 0.0) {
        return Err(format!("its {what} has to be more than nothing"));
    }
    Ok(feet)
}

/// A code as a person writes it: its letters in capitals, and its figures. None where it is not
/// letters then figures (a name like "Range, 30 in").
pub fn read_code(code: &str) -> Option<(String, String)> {
    let t = code.trim();
    let split = t.find(|c: char| !c.is_ascii_alphabetic())?;
    let (letters, digits) = t.split_at(split);
    (split > 0 && digits.chars().all(|c| c.is_ascii_digit())).then(|| (letters.to_uppercase(), digits.to_string()))
}

impl Rule {
    /// Its length as feet, from a cell like `24 in`.
    fn feet_of(text: &Option<String>) -> Option<f64> {
        text.as_deref().and_then(|t| units::parse_length(t).ok()).and_then(|(len, unit)| len.feet(unit).ok())
    }

    /// How far up it hangs, in feet; None where it stands on the floor.
    pub fn hung_ft(&self) -> Option<f64> {
        Rule::feet_of(&self.hung)
    }

    /// The row held: whether it can be used, in the words of the table.
    pub fn check(&self) -> Result<(), String> {
        let code = &self.code;
        if code.is_empty() || !code.chars().all(|c| c.is_ascii_uppercase()) || code.len() > 4 {
            return Err(format!("its code is {code:?}: a code is one to four capital letters, like B or SB"));
        }
        // What each of the three is, by how the number reads.
        let (width_number, height_number) = match self.reads {
            Reads::Width => (true, false),
            Reads::WidthThenHeight => (true, true),
        };
        for (what, cell, from_number) in [("width", &self.width, width_number), ("depth", &self.depth, false), ("height", &self.height, height_number)] {
            match (cell, from_number) {
                (None, true) => {}
                (None, false) => return Err(format!("its {what} has no length, and the number after the letters does not give it: the number reads {}", self.reads.word())),
                (Some(text), false) => {
                    length_ok(what, text)?;
                }
                (Some(text), true) => return Err(format!("its {what} is {text:?}, and the number after the letters gives it (it reads {}): write {FROM_NUMBER:?}", self.reads.word())),
            }
        }
        if let Some(text) = &self.hung {
            length_ok("hung height", text)?;
        }
        if (self.is == Is::WallBox) != self.hung.is_some() {
            return Err(if self.hung.is_some() { format!("it is {} and hangs: only a wall box hangs", self.is.a()) } else { "it is a wall box and stands on the floor: a wall box hangs, say how far up".to_string() });
        }
        if self.drawn_as.trim().is_empty() || self.drawn_as.trim() != self.drawn_as {
            return Err("what it is drawn as has to name a shape, or box".into());
        }
        if self.stocked.iter().any(|w| *w == 0) {
            return Err("a stocked width is more than nothing".into());
        }
        if self.status != Status::Assumed {
            return Err("how sure it is reads something other than Assumed: a cabinet rule is the trade's habit and the maker's catalog decides, so it is Assumed".into());
        }
        if self.source.trim().is_empty() {
            return Err("it does not say where it came from".into());
        }
        Ok(())
    }

    /// How its number reads, with an example, as a cabinet maker says it: "W reads four digits,
    /// the width then the height: W3342 is 33 in wide and 42 in high".
    pub fn reading(&self) -> String {
        let l = &self.code;
        match self.reads {
            Reads::Width => format!("{l} reads the width in inches: {l}42 is 42 in wide"),
            Reads::WidthThenHeight => format!("{l} reads four digits, the width then the height: {l}3342 is 33 in wide and 42 in high"),
        }
    }

    /// The width and height a number gives, in inches.
    pub fn number(&self, code: &str, digits: &str) -> Result<(u32, Option<u32>), String> {
        let (width, height) = self.read_number(code, digits)?;
        if width > WIDEST {
            return Err(format!("{code} reads {width} in wide: no box is made wider than {WIDEST} in, a sheet's length"));
        }
        Ok((width, height))
    }

    fn read_number(&self, code: &str, digits: &str) -> Result<(u32, Option<u32>), String> {
        let bad = || format!("{code} cannot be read: {}", self.reading());
        match self.reads {
            Reads::Width => {
                if digits.len() > 3 || digits.starts_with('0') {
                    return Err(bad());
                }
                Ok((digits.parse().map_err(|_| bad())?, None))
            }
            Reads::WidthThenHeight => {
                if digits.len() != 4 {
                    return Err(bad());
                }
                let (w, h): (u32, u32) = (digits[..2].parse().map_err(|_| bad())?, digits[2..].parse().map_err(|_| bad())?);
                if w == 0 || h == 0 {
                    return Err(format!("{code} cannot be read: a width or a height of nothing. {}", self.reading()));
                }
                Ok((w, Some(h)))
            }
        }
    }

    /// The box this rule makes for a code, as an item of the tool chest. Every size Assumed and
    /// saying the rule and how the number was read; a width the trade does not stock is made
    /// and said so.
    pub fn make(&self, code: &str) -> Result<Made, String> {
        let (letters, digits) = read_code(code).ok_or_else(|| format!("{code} is not a code: letters, then figures"))?;
        if letters != self.code {
            return Err(format!("{code} is not for the rule for {}", self.code));
        }
        let (width, height) = self.number(code.trim(), &digits)?;
        let name = format!("{}{digits}", self.code);
        let not_stocked = !self.stocked.contains(&width);
        let rule = format!("the cabinet rule for {}", self.code);
        let stock = if self.stocked.is_empty() { "none is said to be stocked".to_string() } else { format!("stocked: {}", self.stocked.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")) };
        let length = |text: &str, source: String| -> Result<Val<Len>, String> {
            let (len, unit) = units::parse_length(text).map_err(|why| format!("{name}: {why}"))?;
            Ok(Val { value: Some(len), unit: unit.to_string(), status: self.status, source })
        };
        let unstocked = if not_stocked { format!("; {width} in is not a stocked width ({stock})") } else { String::new() };
        let w_source = if height.is_some() { format!("{rule}: the first two digits of {digits} are the width in inches; {}{unstocked}", self.source) } else { format!("{rule}: the number {digits} is the width in inches; {}{unstocked}", self.source) };
        let width_val = length(&format!("{width} in"), w_source)?;
        let depth_val = length(self.depth.as_deref().unwrap_or(""), format!("{rule}: {} deep; {}", self.depth.as_deref().unwrap_or(""), self.source))?;
        let height_val = match height {
            Some(h) => length(&format!("{h} in"), format!("{rule}: the last two digits of {digits} are the height in inches; {}", self.source))?,
            None => length(self.height.as_deref().unwrap_or(""), format!("{rule}: {} high; {}", self.height.as_deref().unwrap_or(""), self.source))?,
        };
        let sizes = [&width_val, &depth_val, &height_val].map(|v| v.value.and_then(|l| l.feet(&v.unit).ok()).map(|f| units::inches_said(f)).unwrap_or_default());
        let said = format!(
            "{name} was made by the rule for {}: {} wide, {} deep and {} high; Assumed{}",
            self.code,
            sizes[0],
            sizes[1],
            sizes[2],
            if not_stocked { format!("; {width} in is not a stocked width of {}, so it is a made-to-order box", self.code) } else { String::new() }
        );
        let item = Item {
            category: Kind::Furnishing,
            maker: Val { value: None, unit: String::new(), status: Status::Unknown, source: "not chosen".into() },
            model: Val { value: None, unit: String::new(), status: Status::Unknown, source: "not chosen".into() },
            width: width_val,
            depth: depth_val,
            height: height_val,
            weight: None,
            sheet: String::new(),
            division: crate::components::Division::of_number(12),
            form: Some(self.drawn_as.to_lowercase()),
            priced_as: None,
        };
        Ok(Made { name, item, not_stocked, said })
    }

    /// The row as the table writes it.
    pub fn cells(&self) -> Vec<String> {
        let stocked = if self.stocked.is_empty() { String::new() } else { format!("{} in", self.stocked.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")) };
        let cell = |c: &Option<String>| c.clone().unwrap_or_else(|| FROM_NUMBER.to_string());
        vec![
            self.code.clone(),
            self.is.word().to_string(),
            self.reads.word().to_string(),
            cell(&self.width),
            cell(&self.depth),
            cell(&self.height),
            self.hung.as_ref().map(|h| format!("hung {h} up")).unwrap_or_else(|| ON_THE_FLOOR.to_string()),
            self.drawn_as.clone(),
            stocked,
            match self.status {
                Status::Known => "Known",
                Status::Assumed => "Assumed",
                Status::Unknown => "Unknown",
            }
            .to_string(),
            self.source.clone(),
        ]
    }

    /// A row of the table, read: each cell as the table writes it. The refusal is in the table's words.
    #[allow(clippy::too_many_arguments)]
    pub fn from_cells(code: &str, is: &str, reads: &str, width: &str, depth: &str, height: &str, stands: &str, drawn_as: &str, stocked: &str, status: &str, source: &str) -> Result<Rule, String> {
        let letters = code.trim().to_uppercase();
        let is = Is::named(is).ok_or_else(|| format!("what it is reads {is:?}: a base, a wall box, a tall box or a filler"))?;
        let reads = Reads::named(reads).ok_or_else(|| format!("its number reads {reads:?}: the width, or the width then the height"))?;
        let cell = |what: &str, text: &str| -> Result<Option<String>, String> {
            let t = text.trim();
            if t.eq_ignore_ascii_case(FROM_NUMBER) {
                return Ok(None);
            }
            if t.is_empty() {
                return Err(format!("its {what} has nothing in it: a length, or {FROM_NUMBER:?}"));
            }
            Ok(Some(t.to_string()))
        };
        let hung = {
            let t = stands.trim();
            let lower = t.to_lowercase();
            if t.is_empty() || lower == ON_THE_FLOOR {
                None
            } else {
                let inner = lower.strip_prefix("hung").and_then(|r| r.trim().strip_suffix("up")).map(str::trim).filter(|r| !r.is_empty());
                match inner {
                    Some(i) => Some(i.to_string()),
                    None => return Err(format!("where it stands reads {t:?}: \"on the floor\", or \"hung 54 in up\"")),
                }
            }
        };
        let stocked: Vec<u32> = {
            let t = stocked.trim();
            let t = t.strip_suffix("in").or_else(|| t.strip_suffix("\"")).unwrap_or(t).trim();
            if t.is_empty() {
                Vec::new()
            } else {
                t.split(|c: char| c == ',' || c.is_whitespace()).filter(|p| !p.is_empty()).map(|p| p.parse::<u32>().map_err(|_| format!("the stocked widths read {p:?}: whole inches, like 12, 15, 18 in"))).collect::<Result<_, _>>()?
            }
        };
        let status = match status.trim().to_lowercase().as_str() {
            "known" => Status::Known,
            "assumed" => Status::Assumed,
            "unknown" => Status::Unknown,
            other => return Err(format!("how sure it is reads {other:?}, which is not Known, Assumed or Unknown")),
        };
        let rule = Rule { code: letters, is, reads, width: cell("width", width)?, depth: cell("depth", depth)?, height: cell("height", height)?, hung, drawn_as: drawn_as.trim().to_string(), stocked, status, source: source.trim().to_string() };
        rule.check()?;
        Ok(rule)
    }
}

/// The starter rules, as `library/cabinet-rules.csv` gives them.
pub fn starter() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let rows = crate::building::shape::csv_rows(crate::tables::STARTER_CABINET_RULES);
        crate::tables::cabinet_rules_from_rows(&rows, None).map(|(rules, unused)| {
            assert!(unused.is_empty(), "the starter cabinet rules have rows that cannot be read: {unused:?}");
            rules
        }).unwrap_or_else(|why| panic!("the starter cabinet rules cannot be read: {why}"))
    })
}

/// The rules in force: the starter's, with the job's own in place of any it changed, then the
/// job's own others.
pub fn effective(job: &[Rule]) -> Vec<Rule> {
    let mut out: Vec<Rule> = starter().iter().map(|s| job.iter().find(|j| j.code == s.code).unwrap_or(s).clone()).collect();
    out.extend(job.iter().filter(|j| !starter().iter().any(|s| s.code == j.code)).cloned());
    out
}

/// The rule for these letters (capitals or not).
pub fn rule_for<'a>(rules: &'a [Rule], letters: &str) -> Option<&'a Rule> {
    rules.iter().find(|r| r.code.eq_ignore_ascii_case(letters.trim()))
}

/// How far up a box named by this code hangs, in feet, by its rule; None where it stands on
/// the floor, or where no rule reads its letters.
pub fn hung_ft(rules: &[Rule], code: &str) -> Option<f64> {
    let (letters, _) = read_code(code)?;
    rule_for(rules, &letters)?.hung_ft()
}

/// A code the chest has not, asked for: made by its rule, or refused in a cabinet maker's words
/// (what the rules read and how the nearest one reads its number).
pub fn ask(rules: &[Rule], typed: &str) -> Asked {
    let typed = typed.trim();
    let split = typed.find(|c: char| !c.is_ascii_alphabetic());
    // Letters and nothing else is a code with no number ("B"); letters then figures is a code.
    let letters_only = split.is_none() && !typed.is_empty();
    let Some((letters, digits)) = read_code(typed).or_else(|| letters_only.then(|| (typed.to_uppercase(), String::new()))) else {
        return Asked::NotACode;
    };
    let known = || rules.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join(", ");
    match rule_for(rules, &letters) {
        Some(rule) if digits.is_empty() => Asked::Refused(format!("{typed} has no number after its letters: {}", rule.reading())),
        Some(rule) => match rule.make(typed) {
            Ok(made) => Asked::Made(made),
            Err(why) => Asked::Refused(why),
        },
        // Letters that no rule reads: a name with no figures in it is a name, not a code.
        None if digits.is_empty() => Asked::NotACode,
        None => {
            // The nearest rule: the one whose letters share the most with these, first.
            let nearest = rules.iter().max_by_key(|r| (r.code.chars().zip(letters.chars()).take_while(|(a, b)| a == b).count(), std::cmp::Reverse(r.code.len()))).or(rules.first());
            let how = nearest.map(|r| format!(" {}", r.reading())).unwrap_or_default();
            Asked::Refused(format!("{typed} is not in the tool chest, and no cabinet rule reads {letters}. The letters the rules read are {}.{how}", known()))
        }
    }
}
