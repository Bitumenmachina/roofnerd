//! Working a cost line out: formula, then quantity, then waste, then order units.
//!
//! The same reading of a line serves the entry checks (what unit does this line come out
//! in, and does its formula make sense on this kind of element?) and the derivation (what
//! does it come to?). The checks pass no figures in; the derivation passes the element's.

use crate::components::*;
use crate::formula::{self, Expr, Operand};
use crate::units;
use crate::value::{Derived, Status};

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
        return Some(format!("{name:?} cannot be used in a formula; an input's name is lower-case letters, digits and underscores, starting with a letter"));
    }
    if name == HEIGHT || name == STRETCH_OUT || name == "waste" || name == "quantity" || kind.measures().iter().any(|(m, _)| *m == name) {
        return Some(format!("{name:?} already means something on a line; give the input another name"));
    }
    None
}

fn operand(on: &On, line: &CostLine, name: &str) -> Result<Operand, String> {
    if name == STRETCH_OUT {
        let Some(so) = &line.stretch_out else {
            return Err("uses stretch_out, but the line has no stretch-out; enter it on the line".into());
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
            other => return Err(format!("uses {name}, which is in {other}; inside a formula use the measure it comes from, or price the line by {name} alone")),
        };
        let d = on.measure.and_then(|m| m.0.get(name));
        return Ok(Operand { value: d.and_then(|d| d.value), dim, status: d.map(|d| d.status).unwrap_or(Status::Unknown), shown: name.to_string() });
    }
    let mut have: Vec<&str> = on.kind.measures().iter().map(|(n, _)| *n).collect();
    if on.kind == Kind::Parapet {
        have.push(HEIGHT);
    }
    Err(format!("uses {name:?}, which a {:?} does not have (it has: {}) and which is not a value entered on the line", on.kind, have.join(", ")))
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

/// Everything one line works out to on one element.
pub fn figures(me: &str, on: &On, line: &CostLine) -> LineFigures {
    let cost_in = format!("{me}#{K_COST}");
    let read = match read(on, line) {
        Ok(r) => r,
        Err(why) => {
            let refused = Derived::refused("", &line.quantity, std::slice::from_ref(&cost_in), &format!("the line's quantity {why}"));
            return LineFigures { quantity: refused, with_waste: None, order: None };
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
    let is_count = read.unit == "EA" && !read.plain_measure;
    let value = if is_count {
        text.push_str(", rounded up to a whole number");
        count(read.worked.value)
    } else {
        Ok(read.worked.value)
    };
    let quantity = match value {
        Err(why) => Derived::refused(read.unit, &text, &inputs, why),
        Ok(Some(v)) if v < 0.0 => Derived::refused(read.unit, &text, &inputs, "the formula comes out below zero"),
        Ok(v) => Derived::new(v, read.unit, &[read.worked.status], &text, &inputs),
    };

    let with_waste = line.waste.as_ref().map(|w| {
        let v = quantity.value.zip(w.value).map(|(q, w)| q * (1.0 + w / 100.0));
        // A count with waste on it is still a count.
        let (v, formula) = if is_count { (v.map(whole_up), "quantity × (1 + waste ÷ 100), rounded up to a whole number") } else { (v, "quantity × (1 + waste ÷ 100)") };
        Derived::new(v, read.unit, &[quantity.status, w.status], formula, std::slice::from_ref(&cost_in))
    });

    let order = line.order.as_ref().map(|o| {
        let (base, base_name) = match &with_waste {
            Some(w) => (w, "with_waste"),
            None => (&quantity, "quantity"),
        };
        match order_multiplies(o, read.unit) {
            Err(why) => Derived::refused(&o.unit, "", std::slice::from_ref(&cost_in), &format!("the line {why}")),
            Ok(multiplies) => {
                let mut formula = format!("{base_name} {} factor", if multiplies { '\u{00D7}' } else { '\u{00F7}' });
                let mut v = base.value.zip(o.factor.value).map(|(q, f)| if multiplies { q * f } else { q / f });
                if o.round_up {
                    v = v.map(whole_up);
                    formula.push_str(&format!(", rounded up to a whole {}", o.unit));
                }
                Derived::new(v, &o.unit, &[base.status, o.factor.status], &formula, std::slice::from_ref(&cost_in))
            }
        }
    });
    LineFigures { quantity, with_waste, order }
}
