//! The job: one world holding every element, plus the order they were added in.
//! Everything else in the program (the saved file, the sheet, the 3D view) is a way of
//! looking at this.

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::check;
use crate::components::*;
use crate::derive;
use crate::lines;
use crate::units::{self, Len};
use crate::value::{Status, Val};

/// The top of the saved file, in the IFC5 layout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub id: String,
    #[serde(rename = "ifcxVersion")]
    pub ifcx_version: String,
    #[serde(rename = "dataVersion")]
    pub data_version: String,
    pub author: String,
    /// When the job was started. Saving does not touch it, so a save changes nothing
    /// in the file except what was actually edited.
    pub timestamp: String,
    /// Anything else the header held when it was read. Kept and written back.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

pub struct Job {
    pub header: Header,
    pub world: World,
    /// Entities in the order they were added. The file is written in this order.
    pub order: Vec<Entity>,
    /// Parts of the file this program does not use, kept as they were read.
    pub imports: Vec<serde_json::Value>,
    pub schemas: BTreeMap<String, serde_json::Value>,
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Job {
    pub fn new(title: &str, author: &str, started: &str) -> Job {
        let mut job = Job {
            header: Header {
                id: format!("plumbline/{}", slug(title)),
                ifcx_version: "ifcx_alpha".into(),
                data_version: "1.0.0".into(),
                author: author.into(),
                timestamp: started.into(),
                extra: BTreeMap::new(),
            },
            world: World::new(),
            order: Vec::new(),
            imports: Vec::new(),
            schemas: BTreeMap::new(),
            extra: BTreeMap::new(),
        };
        let id = job.world.spawn((Node::new("job-0001", "job-0001", None), Kind::Job, JobInfo { title: title.into(), length_unit: U_LENGTH.into(), next_id: 2 })).id();
        job.order.push(id);
        job
    }

    pub fn root(&self) -> Entity {
        self.order[0]
    }

    pub fn index(&mut self) -> HashMap<String, Entity> {
        let mut q = self.world.query::<(Entity, &Node)>();
        q.iter(&self.world).map(|(e, n)| (n.path.clone(), e)).collect()
    }

    pub fn find(&mut self, path: &str) -> Option<Entity> {
        self.index().get(path).copied()
    }

    pub fn kind(&mut self, path: &str) -> Option<Kind> {
        let e = self.find(path)?;
        self.world.get::<Kind>(e).copied()
    }

    /// Add an element under `parent`. Its path is its kind and the job's next number,
    /// and it keeps that path for good. Two children of one parent cannot share a name.
    pub fn add(&mut self, parent: &str, kind: Kind, name: &str) -> Result<String, String> {
        if name.trim().is_empty() {
            return Err("an element needs a name".into());
        }
        if kind == Kind::Job {
            return Err("a job has one job element, and it is already there".into());
        }
        if self.find(parent).is_none() {
            return Err(format!("no element {parent} to add under"));
        }
        if self.children().get(parent).is_some_and(|c| c.contains_key(name)) {
            return Err(format!("{parent} already has a child named {name:?}"));
        }
        let root = self.root();
        // A path number is never given out twice, whatever the counter says: the next
        // number is past every number already in use.
        let highest = self.index().keys().filter_map(|p| p.rsplit('-').next()?.parse::<u32>().ok()).max().unwrap_or(0);
        let mut info = self.world.get_mut::<JobInfo>(root).ok_or("the job element carries no job information")?;
        let number = info.next_id.max(highest.checked_add(1).ok_or("the job has run out of path numbers")?);
        info.next_id = number.checked_add(1).ok_or("the job has run out of path numbers")?;
        let path = format!("{}-{:04}", kind.stem(), number);
        let id = self.world.spawn((Node::new(&path, name, Some(parent)), kind)).id();
        self.order.push(id);
        Ok(path)
    }

    /// Put a component on an element, unchecked. For building jobs in code; `file::load`
    /// and `set` are the checked routes.
    pub fn put<C: Component>(&mut self, path: &str, component: C) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        self.world.entity_mut(e).insert(component);
        Ok(())
    }

    /// Recompute what follows from the geometry. Returns the paths whose measures
    /// changed, in job order.
    pub fn derive(&mut self) -> Vec<String> {
        let changed = derive::derive(&mut self.world);
        let mut q = self.world.query::<&Node>();
        self.order.iter().filter_map(|e| q.get(&self.world, *e).ok()).map(|n| n.path.clone()).filter(|p| changed.contains(p)).collect()
    }

    /// Change one entered value by element path and field name. On the element: pitch,
    /// height, eave_height, condition, wbs. On one of its cost lines: stretch_out,
    /// quantity (the formula), waste, order, or any value the formula names. The value is
    /// read as typed, units and all (`30 in`, `25' 6 1/2"`, `6:12`), kept in the unit it
    /// was typed in, and has to pass the same checks a loaded file does. `line` picks the
    /// cost line when the element has more than one.
    pub fn set(&mut self, path: &str, field: &str, value: Option<&str>, status: Status, source: &str) -> Result<(), String> {
        self.set_on_line(path, field, None, value, status, source)
    }

    pub fn set_on_line(&mut self, path: &str, field: &str, line: Option<&str>, value: Option<&str>, status: Status, source: &str) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let on_element = matches!(field, "pitch" | "height" | "eave_height" | "condition" | "wbs");
        if line.is_some() && on_element {
            return Err(format!("{field} belongs to the element, not to one of its cost lines; leave --line off"));
        }
        if !on_element {
            return self.set_line_value(e, path, kind, field, line, value, &[], status, source).map(|_| ());
        }
        // A length keeps the unit it is typed in. Marked unknown, it keeps the unit the
        // value it replaces had, so nothing but the value changes in the file.
        let length = |was: Option<&Val<Len>>| typed_length(value, was, status, source);
        let text = || Val { value: value.map(str::to_string), unit: String::new(), status, source: source.into() };
        let missing = |what: &str| format!("{path} has no {what} to set");
        match field {
            "pitch" => {
                let rise = value.map(units::parse_pitch).transpose()?;
                let new = Pitch(Val { value: rise, unit: U_PITCH.into(), status, source: source.into() });
                check::pitch(path, kind, &new)?;
                *self.world.get_mut::<Pitch>(e).ok_or_else(|| missing("pitch"))? = new;
            }
            "height" => {
                let new = Height(length(self.world.get::<Height>(e).map(|h| &h.0))?);
                check::height(path, kind, &new)?;
                *self.world.get_mut::<Height>(e).ok_or_else(|| missing("height"))? = new;
            }
            "eave_height" => {
                let mut new = self.world.get::<Outline>(e).ok_or_else(|| missing("outline"))?.clone();
                new.eave_height = length(Some(&new.eave_height))?;
                check::outline(path, kind, &new)?;
                *self.world.get_mut::<Outline>(e).ok_or_else(|| missing("outline"))? = new;
            }
            "condition" => {
                let new = Condition(text());
                check::condition(path, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            _ => {
                let new = WorkCode { wbs: text() };
                check::work_code(path, &new)?;
                self.world.entity_mut(e).insert(new);
            }
        }
        Ok(())
    }

    /// Which cost line an edit is for: the one named, or the only one there is.
    fn line_at(cost: &Cost, path: &str, line: Option<&str>, what: &str) -> Result<usize, String> {
        let have = || cost.lines.iter().map(|l| format!("{:?}", l.description)).collect::<Vec<_>>().join(", ");
        match line {
            Some(description) => cost.lines.iter().position(|l| l.description == description).ok_or_else(|| format!("{path} has no cost line described as {description:?} (it has: {})", have())),
            None if cost.lines.len() == 1 => Ok(0),
            None if cost.lines.is_empty() => Err(format!("{path} has no cost line to set a {what} on")),
            None => Err(format!("{path} has {} cost lines; say which one with --line (it has: {})", cost.lines.len(), have())),
        }
    }

    /// Rewrite a cost line's formula. `with` gives a value for each name the new formula
    /// brings in (`("turn_up", "8 in")`), entered with this status and source. Values the
    /// new formula no longer names are taken off the line; their names are returned, so
    /// the caller can say so.
    pub fn set_formula(&mut self, path: &str, line: Option<&str>, formula: &str, with: &[(String, String)], status: Status, source: &str) -> Result<Vec<String>, String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        self.set_line_value(e, path, kind, "quantity", line, Some(formula), with, status, source)
    }

    /// Change something on one cost line: its `stretch_out`, its `quantity` (the formula),
    /// its `waste`, its `order` conversion, or one of the values its formula names.
    /// Returns the names of values a new formula dropped.
    #[allow(clippy::too_many_arguments)]
    fn set_line_value(&mut self, e: Entity, path: &str, kind: Kind, field: &str, line: Option<&str>, value: Option<&str>, with: &[(String, String)], status: Status, source: &str) -> Result<Vec<String>, String> {
        let Some(cost) = self.world.get::<Cost>(e) else {
            return Err(if matches!(field, "stretch_out" | "quantity" | "waste" | "order") {
                format!("{path} has no cost line to set a {field} on")
            } else {
                format!("{field:?} is not a field that can be set on {path}; an element takes pitch, height, eave_height, condition or wbs, and a cost line takes stretch_out, quantity, waste, order or a value its formula names")
            });
        };
        let mut new = cost.clone();
        let at = Self::line_at(&new, path, line, field)?;
        let l = &mut new.lines[at];
        let what = format!("{path} cost line {:?}", l.description);
        let mut dropped = Vec::new();
        // A value for a name the formula uses: a plain number where the line holds a plain
        // number or holds nothing yet, otherwise a length with its unit.
        let named = |name: &str, text: Option<&str>, was: Option<&Val<Len>>| -> Result<Val<Len>, String> {
            if let Some(fault) = lines::input_name_fault(kind, name) {
                return Err(format!("{what}: {fault}"));
            }
            match (text, was) {
                (None, Some(was)) => Ok(Val { value: None, unit: was.unit.clone(), status, source: source.into() }),
                (None, None) => Err(format!("{what} has no {name} to mark unknown")),
                (Some(text), was) if units::parse_number(text).is_ok() && was.is_none_or(|w| w.unit.is_empty()) => Ok(Val { value: Some(Len::One(units::parse_number(text)?)), unit: String::new(), status, source: source.into() }),
                (Some(text), Some(was)) if was.unit.is_empty() => Err(format!("{what}: {name} is a plain number on this line, and {text:?} is not one")),
                (Some(_), _) => typed_length(text, was, status, source),
            }
        };
        match field {
            "stretch_out" => l.stretch_out = Some(typed_length(value, l.stretch_out.as_ref(), status, source)?),
            "quantity" => {
                let Some(text) = value else {
                    return Err(format!("{what}: a quantity is a formula, not a value; it cannot be marked unknown. Mark what it uses unknown instead"));
                };
                l.quantity = text.trim().to_string();
                for (name, typed) in with {
                    if name == lines::STRETCH_OUT {
                        l.stretch_out = Some(typed_length(Some(typed), l.stretch_out.as_ref(), status, source)?);
                    } else {
                        let v = named(name, Some(typed), l.inputs.get(name))?;
                        l.inputs.insert(name.clone(), v);
                    }
                }
                // What the new formula no longer names comes off the line, and is reported.
                if let Ok(expr) = crate::formula::parse(&l.quantity) {
                    let used = crate::formula::names(&expr);
                    if let Some((unused, _)) = with.iter().find(|(name, _)| !used.contains(name)) {
                        return Err(format!("{what}: a value is given for {unused:?}, which the formula {:?} does not use", l.quantity));
                    }
                    let gone: Vec<String> = l.inputs.keys().filter(|k| !used.contains(k)).cloned().collect();
                    for name in gone {
                        l.inputs.remove(&name);
                        dropped.push(name);
                    }
                    if l.stretch_out.is_some() && !used.iter().any(|n| n == lines::STRETCH_OUT) {
                        l.stretch_out = None;
                        dropped.push(lines::STRETCH_OUT.to_string());
                    }
                }
            }
            "waste" => {
                let percent = value.map(|s| units::parse_number(s.trim().trim_end_matches('%').trim_end())).transpose()?;
                l.waste = Some(Val { value: percent, unit: "%".into(), status, source: source.into() });
            }
            "order" => {
                let on = lines::On { kind, measure: None, height: None };
                let unit = lines::read(&on, l).map_err(|why| format!("{what}: its quantity {:?} {why}", l.quantity))?.unit;
                match value {
                    None => {
                        let o = l.order.as_mut().ok_or_else(|| format!("{what} has no order conversion to mark unknown"))?;
                        o.factor = Val { value: None, unit: o.factor.unit.clone(), status, source: source.into() };
                    }
                    Some(text) => {
                        // "30 SF/sheet" or "1.25 lb/SF": a number, then what per what.
                        let how = format!("write it as a number and what per what, like 30 {unit}/sheet or 1.25 lb/{unit}");
                        let (number, per) = text.trim().split_once(' ').ok_or_else(|| format!("{text:?} is not an order conversion; {how}"))?;
                        let factor = units::parse_number(number)?;
                        let per = per.trim();
                        let (top, bottom) = per.split_once('/').ok_or_else(|| format!("{text:?} is not an order conversion; {how}"))?;
                        let (order_unit, round_up) = if top == unit && bottom != unit {
                            (bottom, true)
                        } else if bottom == unit && top != unit {
                            (top, false)
                        } else {
                            return Err(format!("{text:?} does not convert this line, which comes out in {unit}; {how}"));
                        };
                        // Buying whole ones is the rule for a package (a sheet, a box) and
                        // not for a weight; a line that already says keeps what it says.
                        let round_up = l.order.as_ref().filter(|o| o.unit == order_unit).map(|o| o.round_up).unwrap_or(round_up);
                        l.order = Some(Order { unit: order_unit.to_string(), factor: Val { value: Some(factor), unit: per.to_string(), status, source: source.into() }, round_up });
                    }
                }
            }
            name => {
                let v = named(name, value, l.inputs.get(name))?;
                l.inputs.insert(name.to_string(), v);
            }
        }
        // The line may now come out in another unit. A price entered for the old unit does
        // not carry over; a line with no price yet just changes what it is priced by.
        let on = lines::On { kind, measure: None, height: None };
        if let Ok(read) = lines::read(&on, &new.lines[at]) {
            let priced = format!("$/{}", lines::priced_in(&new.lines[at], read.unit));
            let l = &mut new.lines[at];
            if l.unit_cost.unit != priced {
                if l.unit_cost.value.is_some() {
                    return Err(format!("{what} is priced in {}; this change would make it a line in {priced}, and a price does not carry from one to the other. Re-enter the unit cost in the file", l.unit_cost.unit));
                }
                l.unit_cost.unit = priced;
            }
        }
        check::cost(path, kind, &new)?;
        *self.world.get_mut::<Cost>(e).ok_or_else(|| format!("{path} has no cost lines"))? = new;
        Ok(dropped)
    }

    /// Apply a condition to an element: the element takes the condition's name and its
    /// cost lines. The condition is found by its path or its name. The lines are copied
    /// onto the element, where each can then be edited; what the condition leaves blank
    /// stays Unknown on the element until someone enters it.
    pub fn apply(&mut self, path: &str, condition: &str, source: &str) -> Result<Vec<String>, String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let mut found: Vec<(String, String, Recipe)> = Vec::new();
        let mut q = self.world.query::<(&Node, &Recipe)>();
        for (node, t) in q.iter(&self.world) {
            if node.path == condition || node.name == condition {
                found.push((node.path.clone(), node.name.clone(), t.clone()));
            }
        }
        let (_, name, recipe) = match found.len() {
            1 => found.remove(0),
            0 => return Err(format!("no condition {condition:?} in this job")),
            _ => return Err(format!("{condition:?} names {} conditions ({}); use the path", found.len(), found.iter().map(|f| f.0.clone()).collect::<Vec<_>>().join(", "))),
        };
        if !recipe.applies_to.contains(&kind) {
            return Err(format!("{name:?} applies to {}, and {path} is a {kind:?}", recipe.applies_to.iter().map(|k| format!("a {k:?}")).collect::<Vec<_>>().join(" or ")));
        }
        let new = match self.world.get::<Cost>(e) {
            None => recipe.cost.clone(),
            Some(have) => {
                if have.code.value != recipe.cost.code.value {
                    return Err(format!("{path} already carries a cost code of its own, and {name:?} brings another; an element has one cost code"));
                }
                let mut joined = have.clone();
                joined.lines.extend(recipe.cost.lines.iter().cloned());
                joined
            }
        };
        check::cost(path, kind, &new)?;
        let assigned = Condition(Val { value: Some(name.clone()), unit: String::new(), status: Status::Known, source: source.into() });
        check::condition(path, &assigned)?;
        let added = recipe.cost.lines.iter().map(|l| l.description.clone()).collect();
        self.world.entity_mut(e).insert((new, assigned));
        Ok(added)
    }

    /// One entered value as it would be shown, for reporting an edit back: the text in
    /// the unit it was entered in, or "unknown".
    pub fn shown(&mut self, path: &str, field: &str, line: Option<&str>) -> Option<String> {
        let e = self.find(path)?;
        let length = |v: &Val<Len>| v.value.as_ref().map(|len| units::show_length(len, &v.unit)).unwrap_or_else(|| "unknown".into());
        let number = |v: &Val<f64>| v.value.map(|x| format!("{x} {}", v.unit)).unwrap_or_else(|| "unknown".into());
        match field {
            "pitch" => self.world.get::<Pitch>(e).map(|p| number(&p.0)),
            "height" => self.world.get::<Height>(e).map(|h| length(&h.0)),
            "eave_height" => self.world.get::<Outline>(e).map(|o| length(&o.eave_height)),
            "condition" => self.world.get::<Condition>(e).map(|c| c.0.value.clone().unwrap_or_else(|| "unknown".into())),
            "wbs" => self.world.get::<WorkCode>(e).map(|c| c.wbs.value.clone().unwrap_or_else(|| "unknown".into())),
            _ => {
                let cost = self.world.get::<Cost>(e)?;
                let l = match line {
                    Some(d) => cost.lines.iter().find(|l| l.description == d)?,
                    None => cost.lines.first()?,
                };
                match field {
                    "stretch_out" => l.stretch_out.as_ref().map(length),
                    "quantity" => Some(l.quantity.clone()),
                    "waste" => l.waste.as_ref().map(number),
                    "order" => l.order.as_ref().map(|o| format!("{}{}", number(&o.factor), if o.round_up { ", whole ones" } else { "" })),
                    name => l.inputs.get(name).map(length),
                }
            }
        }
    }

    /// Children of each element, by name, as the file lists them.
    pub fn children(&mut self) -> BTreeMap<String, BTreeMap<String, String>> {
        let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let mut q = self.world.query::<&Node>();
        for n in q.iter(&self.world) {
            if let Some(parent) = &n.parent {
                out.entry(parent.clone()).or_default().insert(n.name.clone(), n.path.clone());
            }
        }
        out
    }
}

pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// A length as typed, kept in the unit it was typed in. Marked unknown, it keeps the unit
/// of the value it replaces, so nothing but the value changes in the file.
fn typed_length(value: Option<&str>, was: Option<&Val<Len>>, status: Status, source: &str) -> Result<Val<Len>, String> {
    match value {
        Some(s) => {
            let (len, unit) = units::parse_length(s)?;
            Ok(Val { value: Some(len), unit: unit.into(), status, source: source.into() })
        }
        None => Ok(Val { value: None, unit: was.map(|v| v.unit.clone()).unwrap_or_else(|| units::FT.into()), status, source: source.into() }),
    }
}
