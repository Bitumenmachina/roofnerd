//! What a person entered on a cost line (call 206 point 1). A line taken off a thing by a change (an assembly changed
//! for another, an item priced as another assembly or as nothing, the lines of a thing taken off, one line taken off) is
//! named with everything a person had entered on it, so that nothing typed is lost without a word. One function says it.

use crate::components::CostLine;
use crate::units;
use crate::view;

/// Every field of a cost line, and how `entered_on` treats it. A field added to `CostLine` has to be put here (a test
/// reads the line's fields from the file format and fails on one this list does not have) and in `entered_on`, whose
/// destructuring of the line does not compile until it is named.
pub const FIELDS: &[(&str, &str)] = &[
    ("description", "which line it is: the way lines are matched, never entered"),
    ("quantity", "named where it is not what the assembly writes: with its figure"),
    ("stretch_out", "named with the quantity, which it is part of"),
    ("inputs", "named with the quantity, which they are part of"),
    ("waste", "named where it is not the assembly's"),
    ("order", "named where it is not the assembly's"),
    ("unit_cost", "named where it is not the assembly's"),
    ("class", "named where it is not the assembly's"),
    ("division", "named where it is not the assembly's"),
    ("off", "named where it is not as the assembly has it"),
    ("of_material", "named where it is not as the assembly has it"),
];

/// A line taken off a thing: which, what was entered on it, and the figure and the amount it carried.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Took {
    pub line: String,
    /// Each thing entered on it, in a builder's words.
    pub entered: Vec<String>,
    /// What it came to as the estimate had it, where it had a figure.
    pub amount: Option<f64>,
}

impl Took {
    /// How a reply ends the line's name: ", with 7 EA ... entered on it (it came to $7,000.00)".
    pub fn tail(&self) -> String {
        let mut out = String::new();
        if !self.entered.is_empty() {
            out.push_str(&format!(", with {} entered on it", join(&self.entered)));
        }
        if let Some(a) = self.amount {
            out.push_str(&format!(" (it came to {})", view::money(a)));
        }
        out
    }
}

fn join(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

fn same_len(a: &Option<crate::value::Val<crate::units::Len>>, b: &Option<crate::value::Val<crate::units::Len>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x.value == y.value && x.unit == y.unit,
        _ => false,
    }
}

/// What a person entered on `line`, in the words a line is entered in. `assembly` is the line as its assembly writes it:
/// what is the same as that was not entered. With none (the assembly is not in the job to compare with), everything
/// the line holds is named. `figure` is the line's quantity as a figure with its unit ("7 EA"), where there is one.
pub fn entered_on(line: &CostLine, assembly: Option<&CostLine>, figure: Option<&str>) -> Vec<String> {
    // Every field is named here, and none is left to a `..`: a field added to the line does not compile until it is.
    let CostLine { description: _, quantity, stretch_out, inputs, waste, order, unit_cost, class, division, off, of_material } = line;
    let mut parts = Vec::new();
    let reads_so = match assembly {
        Some(a) => *quantity == a.quantity && same_len_map(inputs, &a.inputs) && same_len(stretch_out, &a.stretch_out),
        None => false,
    };
    if !reads_so && !(assembly.is_none() && quantity.is_empty()) {
        let w = crate::job::written(line);
        let said = match figure {
            Some(f) => format!("a quantity of {f}"),
            None => "a quantity".to_string(),
        };
        parts.push(match assembly {
            Some(a) => format!("{said} ({w:?}, where the assembly writes {:?})", crate::job::written(a)),
            None => format!("{said} ({w:?})"),
        });
    }
    if let Some(w) = waste {
        let theirs = assembly.and_then(|a| a.waste.as_ref());
        if !theirs.is_some_and(|t| t.value == w.value) {
            parts.push(match w.value {
                Some(x) => format!("{} % waste", view::num(x)),
                None => "a waste marked unknown".to_string(),
            });
        }
    }
    if let Some(o) = order {
        let theirs = assembly.and_then(|a| a.order.as_ref());
        if !theirs.is_some_and(|t| t.factor.value == o.factor.value && t.factor.unit == o.factor.unit && t.round_up == o.round_up && t.unit == o.unit) {
            parts.push(match o.factor.value {
                Some(x) => format!("{} {}", view::num(x), o.factor.unit),
                None => format!("how it is ordered ({}) marked unknown", o.unit),
            });
        }
    }
    if !CostLine::is_no_price_of_its_own(unit_cost) {
        let theirs = assembly.map(|a| &a.unit_cost).filter(|t| !CostLine::is_no_price_of_its_own(t));
        if !theirs.is_some_and(|t| t.value == unit_cost.value && t.unit == unit_cost.unit) {
            parts.push(match unit_cost.value {
                Some(x) => format!("${}/{}", view::rate(x), unit_cost.unit.trim_start_matches("$/")),
                None => "a price marked unknown".to_string(),
            });
        }
    }
    if let (Some(c), Some(a)) = (class, assembly) {
        if a.class != Some(*c) {
            parts.push(format!("its place in the recap, {}", c.title()));
        }
    } else if let (Some(c), None) = (class, assembly) {
        parts.push(format!("its place in the recap, {}", c.title()));
    } else if class.is_none() && assembly.is_some_and(|a| a.class.is_some()) {
        parts.push("its place in the recap taken away".to_string());
    }
    if let Some(d) = division {
        if assembly.map(|a| a.division) != Some(Some(*d)) {
            parts.push(format!("its division, {} {}", d.number(), d.title()));
        }
    } else if assembly.is_some_and(|a| a.division.is_some()) {
        parts.push("its division taken away".to_string());
    }
    match assembly {
        Some(a) if a.off != *off => parts.push(if *off { "switched off".to_string() } else { "switched on".to_string() }),
        None if *off => parts.push("switched off".to_string()),
        _ => {}
    }
    if let Some(a) = assembly {
        if a.of_material != *of_material {
            parts.push(if *of_material { "made of the thing's material".to_string() } else { "not made of the thing's material".to_string() });
        }
    }
    parts
}

/// As `entered_on`, for a line on a thing set down from an item: the item's trade (`trade`) is put on its lines by the
/// program, so a line in it is not said to have had its division entered (call 206 point 1).
pub fn entered_on_item(line: &CostLine, assembly: Option<&CostLine>, figure: Option<&str>, trade: Option<crate::components::Division>) -> Vec<String> {
    match (trade, assembly) {
        (Some(t), Some(a)) if line.division == Some(t) => entered_on(&CostLine { division: a.division, ..line.clone() }, assembly, figure),
        (Some(t), None) if line.division == Some(t) => entered_on(&CostLine { division: None, ..line.clone() }, assembly, figure),
        _ => entered_on(line, assembly, figure),
    }
}

fn same_len_map(a: &std::collections::BTreeMap<String, crate::value::Val<units::Len>>, b: &std::collections::BTreeMap<String, crate::value::Val<units::Len>>) -> bool {
    a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|w| w.value == v.value && w.unit == v.unit))
}
