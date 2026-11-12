//! Working a cost line out: formula, then quantity, then waste, then order units.
//!
//! The same reading of a line serves the entry checks (what unit does this line come out
//! in, and does its formula make sense on this kind of element?) and the derivation (what
//! does it come to?). The checks pass no figures in; the derivation passes the element's.

use std::collections::BTreeMap;

use crate::components::*;
use crate::formula::{self, Expr, Operand};
use crate::units;
use crate::value::{Derived, Status, Val};

/// Names a formula can use that are not measures and not entered on the line.
pub const HEIGHT: &str = "height";
pub const STRETCH_OUT: &str = "stretch_out";

/// What the line is being read against: the kind of element, and its figures if there
/// are any to hand.
pub struct On<'a> {
    pub kind: Kind,
    pub measure: Option<&'a Measure>,
    pub height: Option<&'a Height>,
}

/// Is this a name an input can take? Lower-case letters, digits and underscores, starting
/// with a letter: something a formula can hold.
pub fn input_name_fault(kind: Kind, name: &str) -> Option<String> {
    let ok = name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !ok {
        return Some(format!("{name:?} cannot be used in a formula; a name entered on a line is plain words: small letters and digits, starting with a letter"));
    }
    if name == HEIGHT || name == STRETCH_OUT || name == "waste" || name == "quantity" || name == "unit_cost" || name == "order" || kind.measures().iter().any(|(m, _)| *m == name) {
        return Some(format!("{name:?} already means something on a line; give the input another name"));
    }
    None
}

fn operand(on: &On, line: &CostLine, name: &str) -> Result<Operand, String> {
    if name == STRETCH_OUT {
        let Some(so) = &line.stretch_out else {
            return Err("uses stretch_out, but the line has no stretch-out. Give the two together: the quantity, and with it the stretch-out it uses (at the command line, --with \"stretch_out=19 1/2 in\")".into());
        };
        return Ok(Operand { value: so.feet(), dim: 1, status: so.status, shown: units::to_feet_text(STRETCH_OUT, &so.unit) });
    }
    if let Some(v) = line.inputs.get(name) {
        return Ok(if v.unit.is_empty() {
            let value = match v.value {
                Some(units::Len::One(x)) => Some(x),
                _ => None,
            };
            Operand { value, dim: 0, status: v.status, shown: name.to_string() }
        } else {
            Operand { value: v.feet(), dim: 1, status: v.status, shown: units::to_feet_text(name, &v.unit) }
        });
    }
    if name == HEIGHT && on.kind == Kind::Parapet {
        return Ok(match on.height {
            Some(h) => Operand { value: h.0.feet(), dim: 1, status: h.0.status, shown: units::to_feet_text(HEIGHT, &h.0.unit) },
            None => Operand { value: None, dim: 1, status: Status::Unknown, shown: HEIGHT.to_string() },
        });
    }
    if let Some((_, unit)) = on.kind.measures().iter().find(|(m, _)| *m == name) {
        let dim = match *unit {
            "LF" => 1,
            "SF" => 2,
            "" => 0,
            other => return Err(format!("uses {name}, which is in {other}; inside a formula use the figure it comes from, or price the line by {name} alone", name = crate::view::words(name))),
        };
        let d = on.measure.and_then(|m| m.0.get(name));
        return Ok(Operand { value: d.and_then(|d| d.value), dim, status: d.map(|d| d.status).unwrap_or(Status::Unknown), shown: name.to_string() });
    }
    let mut have: Vec<&str> = on.kind.measures().iter().map(|(n, _)| *n).collect();
    if on.kind == Kind::Parapet {
        have.push(HEIGHT);
    }
    // (A thing with no figure of its own, the job, is not said to have "(it has: )".)
    let has = if have.is_empty() { format!("a {} has no figure of its own to take a quantity from", on.kind.said()) } else { format!("a {} has {}", on.kind.said(), have.iter().map(|n| crate::view::words(n)).collect::<Vec<_>>().join(", ")) };
    Err(format!("uses {:?}, which is not a figure of this thing ({has}) and is not a value entered on the line. A count is typed as a plain number", crate::view::words(name)))
}

/// A line read against an element: its formula, the unit it comes out in, and the
/// formula worked as far as the figures to hand allow.
pub struct Read {
    pub expr: Expr,
    pub unit: &'static str,
    pub worked: formula::Worked,
    /// The formula is one measure taken as it stands.
    pub plain_measure: bool,
}

/// Read a line's formula against an element. The error is a clause that follows
/// "its quantity <formula>".
pub fn read(on: &On, line: &CostLine) -> Result<Read, String> {
    let expr = formula::parse(&line.quantity)?;
    // One measure taken as it stands keeps that measure's own unit (SQ included).
    if let Expr::Name(name) = &expr {
        if let Some((_, unit)) = on.kind.measures().iter().find(|(m, _)| m == name) {
            if unit.is_empty() {
                return Err(format!("is {name:?}, which is a factor, not a quantity"));
            }
            let d = on.measure.and_then(|m| m.0.get(name.as_str()));
            let worked = formula::Worked { value: d.and_then(|d| d.value), dim: 0, status: d.map(|d| d.status).unwrap_or(Status::Unknown), text: name.clone() };
            return Ok(Read { expr, unit, worked, plain_measure: true });
        }
    }
    let worked = formula::work(&expr, &|name| operand(on, line, name))?;
    let unit = formula::unit_of(worked.dim)?;
    // A factor dressed as a formula (`slope_factor * 1`) is still a factor, not a count.
    let names = formula::names(&expr);
    let is_factor = |n: &String| on.kind.measures().iter().any(|(m, u)| m == n && u.is_empty());
    let has_size = |n: &String| operand(on, line, n).is_ok_and(|o| o.dim > 0);
    if worked.dim == 0 && names.iter().any(is_factor) && !names.iter().any(has_size) {
        return Err("is made of a factor and plain numbers, which is a factor, not a quantity".into());
    }
    Ok(Read { expr, unit, worked, plain_measure: false })
}

/// The unit a line is priced in: its order unit if it has one, else the unit its
/// quantity comes out in.
pub fn priced_in<'a>(line: &'a CostLine, quantity_unit: &'a str) -> &'a str {
    line.order.as_ref().map(|o| o.unit.as_str()).unwrap_or(quantity_unit)
}

/// Does the order factor multiply the quantity (`lb/SF`) or divide it (`SF/sheet`)?
pub fn order_multiplies(order: &Order, quantity_unit: &str) -> Result<bool, String> {
    let (times, over) = (format!("{}/{quantity_unit}", order.unit), format!("{quantity_unit}/{}", order.unit));
    if order.unit == quantity_unit {
        return Err(format!("orders in {}, which is what the line already comes out in", order.unit));
    }
    if order.factor.unit == times {
        Ok(true)
    } else if order.factor.unit == over {
        Ok(false)
    } else {
        Err(format!("has an order factor in {:?}; for a line in {quantity_unit} ordered by the {} it is {times:?} (so many {} to the {quantity_unit}) or {over:?} (so many {quantity_unit} to the {})", order.factor.unit, order.unit, order.unit, order.unit))
    }
}

/// Up to the next whole number, without being pushed over by the crumbs binary
/// arithmetic leaves (30.000000000000004 is 30). Only for a figure that is zero or more:
/// the caller refuses what is below zero first. A figure above zero but too small to tell
/// from a crumb is neither zero nor one, and is given back as it is for the caller to
/// refuse.
fn whole_up(x: f64) -> f64 {
    if x > 0.0 && x < 1e-9 {
        return x;
    }
    (x - 1e-9 * x.abs().max(1.0)).ceil().max(0.0)
}

/// A count rounded up, or the reason it cannot be.
fn count(value: Option<f64>) -> Result<Option<f64>, &'static str> {
    match value {
        Some(v) if v < 0.0 => Err("the formula comes out below zero"),
        Some(v) if v > 0.0 && v < 1e-9 => Err("the formula comes out too small to tell from zero"),
        Some(v) => Ok(Some(whole_up(v))),
        None => Ok(None),
    }
}

/// To the cent.
pub fn cents(x: f64) -> f64 {
    // 1.015 is held as 1.01499999999999990230; the half cent that was meant is rounded
    // up, as it would be on paper, and not down by what binary arithmetic lost.
    // The push is what the arithmetic that made the figure can have lost (a quantity, a
    // price, their product and the hundred, each good to half a unit in the last place:
    // under three units in all), and never as much as a thousandth of a cent: enough to
    // restore a half, too little to make one out of 0.4999 of a cent.
    let y = x * 100.0;
    let push = (y.abs() * 3.0 * f64::EPSILON).min(1e-3);
    (y + y.signum() * push).round() / 100.0 + 0.0
}

/// The most money that is still exact to the cent in the arithmetic used here.
pub const MAX_DOLLARS: f64 = 1e13;

/// An amount of money worked out: to the cent, with the worst status of what it came
/// from. An amount that is more than nothing and less than half a cent is refused: it
/// would be written as $0.00, and nothing is silently zero.
pub fn dollars(value: Option<f64>, statuses: &[Status], formula: &str, inputs: &[String]) -> Derived {
    match value {
        Some(v) if v != 0.0 && cents(v) == 0.0 => Derived::refused("$", formula, inputs, "it comes to less than half a cent and would be written as $0.00"),
        Some(v) if v.abs() > MAX_DOLLARS => Derived::refused("$", formula, inputs, "it comes to more than can be held to the cent"),
        Some(v) => Derived::new(Some(cents(v)), "$", statuses, formula, inputs),
        None => Derived::new(None, "$", statuses, formula, inputs),
    }
}

/// The note on an extension of $0.00 that is $0.00 because there is nothing to price.
pub const ZERO_QUANTITY: &str = "the line's quantity is 0, so it comes to nothing";

/// What the element is made of, as a line made of it needs to know it.
pub enum Stock<'a> {
    /// The element does not say what it is made of.
    NotSaid,
    /// It names a material, and the job has it: the material's path and name, how sure
    /// the element is that it is made of it, and how the material is bought and priced.
    Found { path: &'a str, name: &'a str, status: Status, supply: &'a Supply },
    /// It names one that cannot be used, and why.
    Missing(String),
}

/// Whose price a line is extended at, and what else the extension rests on.
struct Price<'a> {
    /// The price, or why there is none to use.
    at: Result<&'a Val<f64>, String>,
    /// "the line" or the material, for the note of a line with no price.
    whose: String,
    /// Statuses the extension takes besides its own: how sure the element is of its
    /// condition and of its material.
    statuses: Vec<Status>,
    /// Statuses the order quantity takes besides its own: how sure the element is of its
    /// material. (Not its condition: how much there is of a thing does not rest on that.)
    order_statuses: Vec<Status>,
    inputs: Vec<String>,
}

/// What a line comes to in dollars: what it is priced in (its order units, else its
/// quantity with waste, else its quantity) times its unit cost.
fn extension(price: &Price, quantity: &Derived, with_waste: Option<&Derived>, order: Option<&Derived>) -> Derived {
    let (base, name) = match (order, with_waste) {
        (Some(o), _) => (o, "order"),
        (None, Some(w)) => (w, "with_waste"),
        (None, None) => (quantity, "quantity"),
    };
    let formula = format!("{name} × unit_cost");
    let inputs = &price.inputs;
    let at = match &price.at {
        Err(why) => return Derived::refused("$", &formula, inputs, why),
        Ok(v) => *v,
    };
    match (at.value, base.value) {
        (None, _) => Derived::refused("$", &formula, inputs, &format!("{} has no price ({})", price.whose, at.source)),
        (Some(_), None) if base.note.is_empty() => Derived::refused("$", &formula, inputs, "the line has no quantity to price: a figure it is worked out from is not known"),
        (Some(_), None) => Derived::refused("$", &formula, inputs, &format!("the line has no quantity to price: {}", base.note)),
        (Some(p), Some(q)) => {
            let mut statuses = vec![base.status, at.status];
            statuses.extend(&price.statuses);
            let mut d = dollars(Some(q * p), &statuses, &formula, inputs);
            // Nothing is silently zero: a line that prices nothing says so. A price of
            // $0 that was entered is an answer and needs no note.
            if q == 0.0 && d.value.is_some() {
                d.note = ZERO_QUANTITY.into();
            }
            d
        }
    }
}

/// Everything one line works out to on one element. `stock` is what the element is made
/// of: a line made of it is ordered and priced as that material is.
///
/// `condition` is the element's condition, when it has one. The lines are what that
/// condition is made of, so what they come to is no surer than that the element is that
/// condition: an assumed condition makes assumed money, and one that is not known leaves
/// none.
pub fn figures(me: &str, on: &On, line: &CostLine, stock: &Stock, condition: Option<&Val<String>>) -> LineFigures {
    let cost_in = format!("{me}#{K_COST}");
    let mut held: Vec<Status> = Vec::new();
    let mut inputs = vec![cost_in];
    // With a condition that is not known, the line is still measured and ordered; it is
    // what it comes to that has no figure.
    let mut refusal: Option<String> = None;
    if let Some(c) = condition {
        inputs.push(format!("{me}#{K_CONDITION}"));
        if c.value.is_none() {
            refusal = Some(format!("the element's assembly is not known ({}), so what its lines come to is not", c.source));
        } else {
            held.push(c.status);
        }
    }
    // The price to extend at, or why there is none to use.
    fn at<'a>(refusal: &Option<String>, price: &'a Val<f64>) -> Result<&'a Val<f64>, String> {
        match refusal {
            Some(why) => Err(why.clone()),
            None => Ok(price),
        }
    }
    if !line.of_material {
        let price = Price { at: at(&refusal, &line.unit_cost), whose: "the line".into(), statuses: held, order_statuses: Vec::new(), inputs };
        return worked(me, on, line, line.order.as_ref(), &price);
    }
    let made_of = format!("{me}#{K_MATERIAL}");
    match stock {
        Stock::NotSaid => {
            let why = refusal.clone().unwrap_or_else(|| "the line is made of the element's material, and the element does not say what it is made of".into());
            let price = Price { at: Err(why), whose: String::new(), statuses: Vec::new(), order_statuses: Vec::new(), inputs };
            worked(me, on, line, None, &price)
        }
        Stock::Missing(why) => {
            inputs.push(made_of);
            let price = Price { at: Err(refusal.clone().unwrap_or_else(|| why.clone())), whose: String::new(), statuses: Vec::new(), order_statuses: Vec::new(), inputs };
            worked(me, on, line, None, &price)
        }
        Stock::Found { path, name, status, supply } => {
            inputs.extend([made_of, format!("{path}#{K_SUPPLY}")]);
            let whose = format!("its material, {name:?},");
            held.push(*status);
            let price = Price { at: at(&refusal, &supply.price), whose: whose.clone(), statuses: held.clone(), order_statuses: vec![*status], inputs: inputs.clone() };
            let mut out = worked(me, on, line, supply.order.as_ref(), &price);
            // A price is for a unit. One for another unit is not used.
            let priced = out.order.as_ref().map(|o| o.unit.clone()).unwrap_or_else(|| out.quantity.unit.clone());
            let wrong = |v: &Val<f64>| (v.unit != format!("$/{priced}")).then(|| format!("{whose} is priced in {}, and this line comes out in {priced}", v.unit));
            if let (None, Some(why)) = (&refusal, wrong(&supply.price)) {
                out.extension = Some(Derived::refused("$", "", &inputs, &why));
            }
            for (supplier, quote) in &supply.quotes {
                let quoted = Price { at: at(&refusal, quote), whose: format!("{supplier}'s quote for {name:?}"), statuses: held.clone(), order_statuses: vec![*status], inputs: inputs.clone() };
                let d = match (&refusal, wrong(quote)) {
                    (None, Some(why)) => Derived::refused("$", "", &inputs, &why),
                    _ => extension(&quoted, &out.quantity, out.with_waste.as_ref(), out.order.as_ref()),
                };
                out.quotes.insert(supplier.clone(), d);
            }
            out
        }
    }
}

/// A line worked out with the order and the price it is to use: its own, or its
/// material's.
fn worked(me: &str, on: &On, line: &CostLine, order_by: Option<&Order>, price: &Price) -> LineFigures {
    let cost_in = format!("{me}#{K_COST}");
    let read = match read(on, line) {
        Ok(r) => r,
        Err(why) => {
            let refused = Derived::refused("", &line.quantity, std::slice::from_ref(&cost_in), &format!("the line's quantity {why}"));
            let extension = Some(extension(price, &refused, None, None));
            return LineFigures { quantity: refused, with_waste: None, order: None, extension, quotes: BTreeMap::new() };
        }
    };
    // What the figure rests on, for a reader following it back.
    let used = formula::names(&read.expr);
    let mut inputs = Vec::new();
    if used.iter().any(|n| on.kind.measures().iter().any(|(m, _)| m == n)) {
        inputs.push(format!("{me}#{K_MEASURE}"));
    }
    if used.iter().any(|n| n == HEIGHT) && on.kind == Kind::Parapet && !line.inputs.contains_key(HEIGHT) {
        inputs.push(format!("{me}#{K_HEIGHT}"));
    }
    if used.iter().any(|n| n == STRETCH_OUT || line.inputs.contains_key(n)) {
        inputs.push(cost_in.clone());
    }

    let mut text = read.worked.text.clone();
    // A count is whole: 26.67 fasteners is 27. What is below zero is refused before it is
    // rounded, so it is never rounded up to nothing.
    // A number typed as the whole quantity (2.5 months of a dumpster) is what was typed.
    let typed = matches!(read.expr, Expr::Num(_));
    let is_count = read.unit == "EA" && !read.plain_measure && !typed;
    let value = if is_count {
        text.push_str(", rounded up to a whole number");
        count(read.worked.value)
    } else {
        Ok(read.worked.value)
    };
    let mut quantity = match value {
        Err(why) => Derived::refused(read.unit, &text, &inputs, why),
        Ok(Some(v)) if v < 0.0 => Derived::refused(read.unit, &text, &inputs, "the formula comes out below zero"),
        Ok(v) => Derived::new(v, read.unit, &[read.worked.status], &text, &inputs),
    };
    // A count that was rounded up says what it was rounded from.
    if let (true, Some(raw), Some(whole)) = (is_count, read.worked.value, quantity.value) {
        if (raw - whole).abs() > 1e-9 {
            quantity.note = format!("{} rounded up to {}: a count is whole", crate::view::num(raw), crate::view::num(whole));
        }
    }

    let with_waste = line.waste.as_ref().map(|w| {
        let v = quantity.value.zip(w.value).map(|(q, w)| q * (1.0 + w / 100.0));
        // A count with waste on it is still a count.
        let (v, formula) = if is_count { (v.map(whole_up), "quantity × (1 + waste ÷ 100), rounded up to a whole number") } else { (v, "quantity × (1 + waste ÷ 100)") };
        Derived::new(v, read.unit, &[quantity.status, w.status], formula, std::slice::from_ref(&cost_in))
    });

    let order = order_by.map(|o| {
        let (base, base_name) = match &with_waste {
            Some(w) => (w, "with_waste"),
            None => (&quantity, "quantity"),
        };
        match order_multiplies(o, read.unit) {
            Err(why) => Derived::refused(&o.unit, "", &price.inputs, &format!("the line {why}")),
            Ok(multiplies) => {
                let mut formula = format!("{base_name} {} factor", if multiplies { '\u{00D7}' } else { '\u{00F7}' });
                let mut v = base.value.zip(o.factor.value).map(|(q, f)| if multiplies { q * f } else { q / f });
                if o.round_up {
                    v = v.map(whole_up);
                    formula.push_str(&format!(", rounded up to a whole {}", o.unit));
                }
                let mut statuses = vec![base.status, o.factor.status];
                statuses.extend(&price.order_statuses);
                Derived::new(v, &o.unit, &statuses, &formula, &price.inputs)
            }
        }
    });
    let extension = Some(extension(price, &quantity, with_waste.as_ref(), order.as_ref()));
    LineFigures { quantity, with_waste, order, extension, quotes: BTreeMap::new() }
}

/// What an element comes to, both ways (`pl::priced`): from its allowance, from its cost
/// lines, and the two set against each other. `figure` is the one figure a thing of its
/// kind is bought by. Empty when the element has neither an allowance nor a cost line.
pub fn priced(me: &str, kind: Kind, measure: &Measure, allowance: Option<&Allowance>, cost: Option<&Cost>, line_quantities: &BTreeMap<String, LineFigures>) -> BTreeMap<String, Derived> {
    use std::collections::BTreeMap as Map;
    let mut out: Map<String, Derived> = Map::new();
    // A line switched off is on the sheet and out of the detail.
    let lines: Vec<&CostLine> = cost.map(|c| c.lines.iter().filter(|l| !l.off).collect()).unwrap_or_default();
    if allowance.is_none() && lines.is_empty() {
        return out;
    }
    let basis = kind.priced_by().map(|(name, unit)| (name, unit, measure.0.get(name)));
    let in_measure = format!("{me}#{K_MEASURE}");
    // From the allowance.
    if let Some(a) = allowance {
        let inputs = [in_measure.clone(), format!("{me}#{K_ALLOWANCE}")];
        let d = match &basis {
            None => Derived::refused("$", "", &inputs, &format!("a {kind:?} has no figure an allowance is a rate on")),
            Some((name, _, figure)) => {
                let f = format!("{name} × allowance rate");
                match (figure.and_then(|d| d.value), a.rate.value) {
                    (_, None) => Derived::refused("$", &f, &inputs, &format!("the allowance has no rate ({})", a.rate.source)),
                    (None, Some(_)) => Derived::refused("$", &f, &inputs, &format!("its {name} has no figure{}", figure.filter(|d| !d.note.is_empty()).map(|d| format!(": {}", d.note)).unwrap_or_default())),
                    (Some(q), Some(rate)) => dollars(Some(q * rate), &[figure.map(|d| d.status).unwrap_or(Status::Unknown), a.rate.status], &f, &inputs),
                }
            }
        };
        out.insert("allowance".into(), d);
    }
    // From the lines: whole, or not at all.
    if !lines.is_empty() {
        let inputs = [format!("{me}#{K_LINE_QUANTITIES}")];
        let f = if lines.len() == 1 { "its cost line's extension".to_string() } else { format!("its {} cost lines' extensions, added up", lines.len()) };
        let each: Vec<(&str, Option<&Derived>)> = lines.iter().map(|l| (l.description.as_str(), line_quantities.get(&l.description).and_then(|q| q.extension.as_ref()))).collect();
        let missing: Vec<&(&str, Option<&Derived>)> = each.iter().filter(|(_, d)| d.and_then(|d| d.value).is_none()).collect();
        let detail = match missing.first() {
            Some((description, d)) => {
                let why = d.map(|d| d.note.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| "it has no extension".into());
                let more = if missing.len() > 1 { format!(", and {} more", missing.len() - 1) } else { String::new() };
                Derived::refused("$", &f, &inputs, &format!("the detail is not whole: {} of its {} lines cannot be extended. {description:?}: {why}{more}", missing.len(), lines.len()))
            }
            None => {
                let statuses: Vec<Status> = each.iter().filter_map(|(_, d)| d.map(|d| d.status)).collect();
                let sum: f64 = each.iter().filter_map(|(_, d)| d.and_then(|d| d.value)).sum();
                let mut d = dollars(Some(sum), &statuses, &f, &inputs);
                // A line that comes to nothing because there is nothing to price is named.
                let nothing: Vec<String> = each.iter().filter(|(_, d)| d.is_some_and(|d| d.note == ZERO_QUANTITY)).map(|(description, _)| format!("{description:?}")).collect();
                if !nothing.is_empty() && d.value.is_some() {
                    d.note = format!("{} of its {} lines come{} to nothing because the quantity is 0: {}", nothing.len(), lines.len(), if nothing.len() == 1 { "s" } else { "" }, nothing.join(", "));
                }
                d
            }
        };
        // Rolled back up: over the same figure the allowance is priced on.
        if let Some((name, unit, figure)) = &basis {
            let inputs = [in_measure.clone(), inputs[0].clone()];
            let f = format!("detail ÷ {name}");
            let rate_unit = format!("$/{unit}");
            let rate = match (detail.value, figure.and_then(|d| d.value)) {
                (None, _) => Derived::refused(&rate_unit, &f, &inputs, &detail.note),
                (Some(_), None) => Derived::refused(&rate_unit, &f, &inputs, &format!("its {name} has no figure")),
                (Some(_), Some(q)) if q <= 0.0 => Derived::refused(&rate_unit, &f, &inputs, &format!("its {name} is nothing, and a price cannot be spread over nothing")),
                (Some(amount), Some(q)) => Derived::new(Some(amount / q), &rate_unit, &[detail.status, figure.map(|d| d.status).unwrap_or(Status::Unknown)], &f, &inputs),
            };
            out.insert("detail_rate".into(), rate);
        }
        out.insert("detail".into(), detail);
    }
    // Set against each other, when both are there.
    if let (Some(a), Some(d)) = (out.get("allowance").cloned(), out.get("detail").cloned()) {
        let inputs = [format!("{me}#{K_PRICED}")];
        let (difference, percent) = match (a.value, d.value) {
            (Some(a_v), Some(d_v)) => {
                let statuses = [a.status, d.status];
                let diff = Derived::new(Some(cents(d_v - a_v)), "$", &statuses, "detail − allowance", &inputs);
                let pct = if a_v == 0.0 { Derived::refused("%", "difference ÷ allowance × 100", &inputs, "the allowance is $0.00, and a difference is not a share of nothing") } else { Derived::new(Some((d_v - a_v) / a_v * 100.0), "%", &statuses, "difference ÷ allowance × 100", &inputs) };
                (diff, pct)
            }
            _ => {
                let why = if a.value.is_none() { format!("the allowance has no figure: {}", a.note) } else { d.note.clone() };
                (Derived::refused("$", "detail − allowance", &inputs, &why), Derived::refused("%", "difference ÷ allowance × 100", &inputs, &why))
            }
        };
        out.insert("difference".into(), difference);
        out.insert("difference_percent".into(), percent);
    }
    out
}
