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

/// A line both assemblies have, that read as the old assembly wrote it, and now reads as the new one writes it
/// (call 199): its quantity as a figure before and after, and as it is written before and after.
#[derive(Clone, Debug, PartialEq)]
pub struct Rewritten {
    pub line: String,
    pub was: String,
    pub now: String,
    pub was_written: String,
    pub now_written: String,
}

/// Why a shared line's quantity was not made the new assembly's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftAsIs {
    /// A person changed its quantity (its formula or an entry is no longer the old assembly's).
    Entered,
    /// The old assembly is no longer in the job, so the line cannot be compared with what it wrote.
    OldAssemblyGone,
    /// The new quantity comes out in another unit than the line is priced and ordered in.
    Unit,
}

/// A line both assemblies have, whose quantity was left as it was, and what the new assembly would have written.
#[derive(Clone, Debug, PartialEq)]
pub struct Left {
    pub line: String,
    pub has: String,
    pub has_written: String,
    pub would: String,
    pub would_written: String,
    pub why: LeftAsIs,
}

/// What changing a thing's assembly did (call 199): the lines added, the lines taken off, the shared lines whose
/// quantity is now the new assembly's, and the shared lines left as they were.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Changed {
    pub added: Vec<String>,
    pub taken: Vec<String>,
    /// Each line taken off, with what a person had entered on it (call 206 point 1); the amount is for the caller, who has the estimate.
    pub took: Vec<crate::entered::Took>,
    pub rewritten: Vec<Rewritten>,
    pub left: Vec<Left>,
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
    /// The shapes of its own the job carries (call 198): what an item can be drawn as, written
    /// as blocks between formulas. The thirteen the program draws are in the program, not here.
    pub shapes: Vec<crate::building::shape::Shape>,
    /// Where each path was last found, so that finding a thing is not a reading of the
    /// whole job (M21-9). Never trusted: `find` checks what it holds against the job
    /// and looks again where it is wrong, so nothing has to keep it up to date.
    pub(crate) found: HashMap<String, Entity>,
}

/// The moment now by the machine's clock, as a file's heading writes it:
/// `2026-01-31T09:30:00`, by the clock at Greenwich. Its first ten letters are the day.
/// (Days since 1970 to a date by the civil-calendar arithmetic everyone uses.)
pub fn now() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let (day, month) = (doy - (153 * mp + 2) / 5 + 1, if mp < 10 { mp + 3 } else { mp - 9 });
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}", rest / 3_600, rest % 3_600 / 60, rest % 60)
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
            shapes: Vec::new(),
            found: HashMap::new(),
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
        // Where it was last found, if it is still there and still that.
        if let Some(&e) = self.found.get(path) {
            if self.world.get::<Node>(e).is_some_and(|n| n.path == path) {
                return Some(e);
            }
            self.found.remove(path);
        }
        let mut q = self.world.query::<(Entity, &Node)>();
        let e = q.iter(&self.world).find(|(_, n)| n.path == path).map(|(e, _)| e)?;
        self.found.insert(path.to_string(), e);
        Some(e)
    }

    pub fn kind(&mut self, path: &str) -> Option<Kind> {
        let e = self.find(path)?;
        self.world.get::<Kind>(e).copied()
    }

    /// The other planes of the roof a plane was solved from, in the order they were
    /// made: an estimator prices "the roof", and what is given to one plane of it is
    /// offered to the rest (J24A-6, -8). None for anything that is not a solved plane.
    pub fn sister_planes(&mut self, path: &str) -> Vec<String> {
        let Some(e) = self.find(path) else { return Vec::new() };
        let (Some(kind), Some(from)) = (self.world.get::<Kind>(e).copied(), self.world.get::<Solved>(e).map(|s| s.from.clone())) else { return Vec::new() };
        if kind != Kind::RoofPlane {
            return Vec::new();
        }
        self.order.iter().filter(|x| **x != e).filter(|x| self.world.get::<Kind>(**x) == Some(&kind) && self.world.get::<Solved>(**x).is_some_and(|s| s.from == from)).filter_map(|x| self.world.get::<Node>(*x).map(|n| n.path.clone())).collect()
    }

    /// Add an element under `parent`. Its path is its kind and the job's next number,
    /// and it keeps that path for good. Two children of one parent cannot share a name.
    pub fn add(&mut self, parent: &str, kind: Kind, name: &str) -> Result<String, String> {
        // A name has no space before it or after it: two names that differ only there
        // would read as one, and a table written out and brought back would lose them.
        let name = name.trim();
        if name.is_empty() {
            return Err("an element needs a name".into());
        }
        if kind == Kind::Job {
            return Err("a job has one job element, and it is already there".into());
        }
        if self.find(parent).is_none() {
            return Err(format!("no element {parent} to add under"));
        }
        let taken = {
            let mut q = self.world.query::<&Node>();
            q.iter(&self.world).any(|n| n.parent.as_deref() == Some(parent) && n.name == name)
        };
        if taken {
            let root = self.root();
            let at = self.find(parent).filter(|e| *e != root).and_then(|e| self.world.get::<Node>(e)).map(|n| format!("under {}", n.name)).unwrap_or_else(|| "in the job".to_string());
            return Err(format!("there is already something called \"{name}\" {at}: two things in one place cannot go by one name"));
        }
        let root = self.root();
        // A path number is never given out twice, whatever the counter says: the next
        // number is past every number already in use.
        let highest = {
            let mut q = self.world.query::<&Node>();
            q.iter(&self.world).filter_map(|n| n.path.rsplit('-').next()?.parse::<u32>().ok()).max().unwrap_or(0)
        };
        let mut info = self.world.get_mut::<JobInfo>(root).ok_or("the job element carries no job information")?;
        let number = info.next_id.max(highest.checked_add(1).ok_or("the job has run out of path numbers")?);
        info.next_id = number.checked_add(1).ok_or("the job has run out of path numbers")?;
        let path = format!("{}-{:04}", kind.stem(), number);
        let id = self.world.spawn((Node::new(&path, name, Some(parent)), kind)).id();
        self.order.push(id);
        self.found.insert(path.clone(), id);
        Ok(path)
    }

    /// Take an element out of the job. It must have no children.
    pub fn remove(&mut self, path: &str) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        if e == self.root() {
            return Err("the job element cannot be removed".into());
        }
        let under = self.children().get(path).map(|c| c.len()).unwrap_or(0);
        if under > 0 {
            return Err(format!("{path} has {} under it: delete {} first", crate::view::so_many(under, "thing"), if under == 1 { "that" } else { "those" }));
        }
        self.order.retain(|x| *x != e);
        self.world.despawn(e);
        Ok(())
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
        // What a thing's assembly is was once called its condition, and is taken by either word.
        let field = if field == "assembly" { "condition" } else { field };
        // What is said of a wall is named by its words, in any capitals ("In The Wall").
        let lower = field.to_ascii_lowercase();
        let field = if matches!(lower.as_str(), "bearing" | "in_the_wall" | "inthewall") { if lower == "bearing" { "bearing" } else { "in_the_wall" } } else { field };
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let on_element = matches!(field, "pitch" | "height" | "eave_height" | "condition" | "wbs" | "allowance" | "material" | "overhead" | "profit" | "price" | "bearing" | "in_the_wall") || (field == "order" && kind == Kind::Material) || (field == "type" && kind.is_framing()) || (field == "joists_span" && kind == Kind::Floor);
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
        // A plane solved from its roof's footprint rises as its edge of the footprint
        // does and has the roof's eave height. Set on the plane, either was taken, said
        // to be Known, and thrown away the next time the roof was solved, leaving a
        // file that failed its own check (J23-1). It is the roof's: it is put on the
        // roof, and the roof is solved again (call 91 did this for a measurement).
        if matches!(field, "pitch" | "eave_height") {
            if let Some(solved) = self.world.get::<Solved>(e).cloned() {
                let typed = value.ok_or_else(|| format!("{path} is solved from {}'s footprint, and a roof with no {} cannot be solved: give it the value it has", solved.from, field.replace('_', " ")))?;
                let change = match (field, solved.edge) {
                    ("pitch", Some(edge)) => crate::solve::Change::Rise { edge, rise: Val { value: Some(Rise::Pitch(units::parse_pitch(typed)?)), unit: U_PITCH.into(), status, source: source.into() } },
                    ("pitch", None) => return Err(format!("{path} is an eave, a hip or a ridge of {}, and has no pitch of its own", solved.from)),
                    _ => {
                        let (len, unit) = units::parse_length(typed)?;
                        crate::solve::Change::EaveHeight(Val { value: Some(len), unit: unit.into(), status, source: source.into() })
                    }
                };
                crate::solve::edit(self, &solved.from, change)?;
                return Ok(());
            }
        }
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
            "material" => {
                // A material is known by its name: one typed with other capitals or
                // spacing is the material already in the job, written as the job has it.
                let mut new = MadeOf(text());
                if let Some(typed) = new.0.value.clone() {
                    let mut q = self.world.query::<(&Node, &Kind)>();
                    let known = q.iter(&self.world).filter(|(_, k)| **k == Kind::Material).map(|(n, _)| n.name.clone()).find(|n| check::same_name(n, &typed));
                    if let Some(known) = known {
                        new.0.value = Some(known);
                    }
                }
                check::made_of(path, kind, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            // Which way a floor's joists run (call 165): "the short way" or "the long way".
            "joists_span" => {
                let word = value.map(crate::JoistsSpan::read).transpose()?;
                let new = crate::JoistsSpan(Val { value: word.map(str::to_string), unit: String::new(), status, source: source.into() });
                crate::building::check_joists_span(path, kind, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            // A beam, a post or a pad footing changed to another type of its kind (call 164).
            "type" => {
                let typed = value.ok_or_else(|| format!("{path} has to have a type: give the type's path"))?;
                let made_for = self.find(typed).and_then(|t| self.world.get::<crate::SegmentType>(t).map(|t| t.category));
                if made_for != Some(kind) {
                    return Err(format!("{typed} is not a type of {}: {}", kind.said(), match made_for { Some(other) => format!("it is a type of {}", other.said()), None => format!("make one with `type {}`", kind.said().replace("pad footing", "pad")) }));
                }
                let new = Val { value: Some(typed.to_string()), unit: String::new(), status, source: source.into() };
                if let Some(mut c) = self.world.get_mut::<crate::Stood>(e) {
                    c.type_name = new;
                } else if let Some(mut c) = self.world.get_mut::<crate::Segment>(e) {
                    c.type_name = new;
                } else {
                    return Err(missing("place"));
                }
            }
            "bearing" | "in_the_wall" => {
                // What the wall is and holds, said of a wall (call 160): bearing or not
                // bearing; and in words, what runs inside it.
                if kind != Kind::Wall {
                    return Err(format!("{path} is {kind:#}, not a wall: bearing and what is in the wall are said of a wall"));
                }
                let mut new = self.world.get::<Hidden>(e).cloned().unwrap_or(Hidden { bearing: None, in_the_wall: None });
                if field == "bearing" {
                    new.bearing = Some(Val { value: value.map(Bearing::read).transpose()?, unit: String::new(), status, source: source.into() });
                } else {
                    new.in_the_wall = Some(Val { value: value.map(|t| t.split_whitespace().collect::<Vec<_>>().join(" ")), unit: String::new(), status, source: source.into() });
                }
                check::hidden(path, kind, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            "overhead" | "profit" => {
                let percent = value.map(|s| units::parse_number(s.trim().trim_end_matches('%').trim_end())).transpose()?;
                let entered = Val { value: percent, unit: "%".into(), status, source: source.into() };
                let blank = || Val::unknown("%", "not entered");
                let mut new = self.world.get::<Markup>(e).cloned().unwrap_or_else(|| Markup { overhead: blank(), profit: blank() });
                if field == "overhead" {
                    new.overhead = entered;
                } else {
                    new.profit = entered;
                }
                check::markup(path, kind, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            "price" => {
                let mut new = self.world.get::<Supply>(e).cloned().ok_or_else(|| format!("{path} is {kind:#}, not a material; a price is set on a material, and a cost line's is its unit_cost"))?;
                let per = new.price.unit.clone();
                new.price = Val { value: value.map(|t| units::parse_price(t, &per)).transpose()?, unit: per, status, source: source.into() };
                check::supply(path, kind, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            "order" => {
                // On a material: how what is measured becomes what is bought.
                let mut new = self.world.get::<Supply>(e).cloned().ok_or_else(|| missing("supply"))?;
                let bought = new.price.unit.trim_start_matches("$/").to_string();
                match value {
                    None => {
                        let o = new.order.as_mut().ok_or_else(|| format!("{path} has no order factor to mark unknown"))?;
                        o.factor = Val { value: None, unit: o.factor.unit.clone(), status, source: source.into() };
                    }
                    Some(text) => {
                        let how = format!("write it as a number and what per what, like 1.25 {bought}/SF or 30 SF/{bought}");
                        let (number, per) = text.trim().split_once(' ').ok_or_else(|| format!("{text:?} is not an order factor; {how}"))?;
                        let factor = units::parse_number(number)?;
                        let (per, said) = match per.trim().rsplit_once(' ') {
                            Some((rest, "whole")) => (rest.trim(), Some(true)),
                            Some((rest, "exact")) => (rest.trim(), Some(false)),
                            _ => (per.trim(), None),
                        };
                        let (top, bottom) = per.split_once('/').ok_or_else(|| format!("{text:?} is not an order factor; {how}"))?;
                        if (top == bought) == (bottom == bought) {
                            return Err(format!("{text:?} does not say how many {bought} it takes: this material is priced by the {bought}; {how}"));
                        }
                        // Whole ones for a package (so many SF to the sheet), not for a weight.
                        let round_up = said.or(new.order.as_ref().map(|o| o.round_up)).unwrap_or(bottom == bought);
                        new.order = Some(Order { unit: bought, factor: Val { value: Some(factor), unit: per.to_string(), status, source: source.into() }, round_up });
                    }
                }
                check::supply(path, kind, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            "allowance" => {
                // A rate on the one figure a thing of this kind is bought by.
                let Some((_, unit)) = kind.priced_by() else {
                    return Err(format!("{path}: {kind:#} cannot carry an allowance; it has no area or length a rate could be on"));
                };
                let per = format!("$/{unit}");
                let rate = value.map(|text| units::parse_price(text, &per)).transpose()?;
                let new = Allowance { rate: Val { value: rate, unit: per, status, source: source.into() } };
                check::allowance(path, kind, &new)?;
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

    /// A supplier's price for a material. `None` marks it unknown; the quote stays, as a
    /// price that was asked for and has not come.
    pub fn set_quote(&mut self, path: &str, supplier: &str, value: Option<&str>, status: Status, source: &str) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let mut new = self.world.get::<Supply>(e).cloned().ok_or_else(|| format!("{path} is {kind:#}, not a material; a supplier quotes a material"))?;
        // A supplier already in the job is named the way it is named there.
        let mut q = self.world.query::<&Supply>();
        if let Some(known) = q.iter(&self.world).flat_map(|s| s.quotes.keys()).find(|k| *k != supplier && check::same_name(k, supplier)) {
            return Err(format!("{supplier:?} is {known:?} written another way; a supplier has one name in a job, so use {known:?}"));
        }
        let per = new.price.unit.clone();
        new.quotes.insert(supplier.to_string(), Val { value: value.map(|t| units::parse_price(t, &per)).transpose()?, unit: per, status, source: source.into() });
        check::supply(path, kind, &new)?;
        self.world.entity_mut(e).insert(new);
        Ok(())
    }

    /// A new material, bought and priced by `unit` (lb, sheet, SF). Its price is Unknown
    /// until it is entered. Returns its path.
    pub fn add_material(&mut self, name: &str, unit: &str) -> Result<String, String> {
        let plain = check::plain_name(name);
        let mut q = self.world.query::<(&Node, &Kind)>();
        if let Some((known, _)) = q.iter(&self.world).find(|(n, k)| **k == Kind::Material && check::plain_name(&n.name) == plain) {
            return Err(format!("the job already has a material called {:?}", known.name));
        }
        self.add_material_not_there(name, unit)
    }

    /// The same, for a caller that has already found that the job has no material of
    /// this name (a price list keeps every name it has seen, and asks once a row).
    pub(crate) fn add_material_not_there(&mut self, name: &str, unit: &str) -> Result<String, String> {
        let root = self.world.get::<Node>(self.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let supply = Supply { order: None, price: Val::unknown(&format!("$/{unit}"), "no price entered"), quotes: BTreeMap::new() };
        check::supply("the new material", Kind::Material, &supply)?;
        let path = self.add(&root, Kind::Material, name)?;
        self.put(&path, supply)?;
        Ok(path)
    }

    /// A new cost line on an element, typed by hand: what it is, its quantity (a number,
    /// or a formula over the element's figures) and, if it is said, its class. Its unit
    /// cost is Unknown until it is entered. A line made of the element's material
    /// (`of_material`) carries no price of its own.
    pub fn add_line(&mut self, path: &str, description: &str, quantity: &str, class: Option<Class>, of_material: bool) -> Result<(), String> {
        self.add_line_with(path, description, quantity, class, of_material, BTreeMap::new())
    }

    fn add_line_with(&mut self, path: &str, description: &str, quantity: &str, class: Option<Class>, of_material: bool, inputs: BTreeMap<String, Val<Len>>) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let mut line = if of_material { CostLine::of_material(description, quantity) } else { CostLine::of(description, quantity, Val::unknown("", "no price entered")) };
        line.inputs = inputs;
        if !of_material {
            line.class = class;
            let on = lines::On { kind, measure: None, height: None };
            let unit = lines::read(&on, &line).map_err(|why| format!("{path} cost line {description:?}: its quantity {quantity:?} {why}"))?.unit;
            line.unit_cost.unit = format!("$/{unit}");
        } else if class.is_some_and(|c| c != Class::Material) {
            return Err(format!("{path} cost line {description:?}: is made of the element's material, so its cost type is Material"));
        }
        let mut new = self.world.get::<Cost>(e).cloned().unwrap_or_else(|| Cost { code: Val::unknown("", "not coded"), lines: Vec::new() });
        new.lines.push(line);
        check::cost(path, kind, &new)?;
        self.world.entity_mut(e).insert(new);
        Ok(())
    }

    /// Take one cost line off a thing, with whatever was entered on it (call 166). The
    /// thing's other lines, its assembly and its code stay as they are. Gives the line as
    /// it stood. A thing with no line of that name is refused, with the lines it has.
    pub fn remove_line(&mut self, path: &str, description: &str) -> Result<CostLine, String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let mut cost = self.world.get::<Cost>(e).cloned().ok_or_else(|| format!("{path} has no cost lines to take one off"))?;
        let Some(at) = cost.lines.iter().position(|l| l.description == description) else {
            let has: Vec<String> = cost.lines.iter().map(|l| format!("{:?}", l.description)).collect();
            return Err(if has.is_empty() { format!("{path} has no cost lines to take one off") } else { format!("{path} has no cost line {description:?}; it has {}", has.join(", ")) });
        };
        let line = cost.lines.remove(at);
        check::cost(path, kind, &cost)?;
        self.world.entity_mut(e).insert(cost);
        Ok(line)
    }

    /// A new cost line whose quantity is a measure typed outright ("60 SF": call 159): a
    /// number and one of LF, SF, SQ, CY or EA. It is held as a value entered on the line,
    /// named `measured`, with how sure it is and where it came from, as every entered
    /// value is; the line's quantity is that value.
    #[allow(clippy::too_many_arguments)]
    pub fn add_line_measured(&mut self, path: &str, description: &str, measure: &str, class: Option<Class>, of_material: bool, status: Status, source: &str) -> Result<(), String> {
        let (x, unit) = units::parse_measure_on_line(measure)?;
        if status == Status::Unknown {
            return Err(format!("{measure:?} is a figure that was typed, so it is Known or Assumed; to leave a quantity not known, give the line a figure to take it from"));
        }
        if source.trim().is_empty() {
            return Err(format!("{measure:?} says where it came from: a measure typed on a line gives its source"));
        }
        let measured = Val { value: Some(Len::One(x)), unit: unit.into(), status, source: source.into() };
        self.add_line_with(path, description, "measured", class, of_material, BTreeMap::from([("measured".to_string(), measured)]))
    }

    /// Add a line of in-house work (Labor, Fabrication, Supervision): its hours come from
    /// a production rate over its quantity ("40 SF/hr" or "0.025 hr/SF"), and it is
    /// priced by the hour. All of it or none: a rate that cannot be read leaves the
    /// element as it was. (A line added with `add_line` and one of these cost types was
    /// refused for having no rate, and nothing but the sample's own code could give it
    /// one in the same breath.)
    pub fn add_line_in_hours(&mut self, path: &str, description: &str, quantity: &str, class: Class, rate: &str, status: Status, source: &str) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        // A rate that names no hour is refused here, as what it is: further on it is
        // read before the line is known to be in-house work.
        if !(rate.contains("/hr") || rate.contains("hr/")) {
            return Err(format!("{rate:?} is not a production rate; write how fast the work goes, like 40 SF/hr or 0.025 hr/SF"));
        }
        let before = self.world.get::<Cost>(e).cloned();
        let done = self
            .add_line(path, description, quantity, None, false)
            .and_then(|()| self.set_on_line(path, "order", Some(description), Some(rate), status, source))
            .and_then(|()| self.set_class(path, Some(description), class));
        if done.is_err() {
            match before {
                Some(cost) => {
                    self.world.entity_mut(e).insert(cost);
                }
                None => {
                    self.world.entity_mut(e).remove::<Cost>();
                }
            }
        }
        done
    }

    /// Say which row of the recap a cost line lands in.
    pub fn set_class(&mut self, path: &str, line: Option<&str>, class: Class) -> Result<(), String> {
        self.edit_line(path, line, "class", |l| l.class = Some(class))
    }

    /// Say which of the sixteen divisions a cost line's work is.
    pub fn set_division(&mut self, path: &str, line: Option<&str>, division: crate::components::Division) -> Result<(), String> {
        self.edit_line(path, line, "division", |l| l.division = Some(division))
    }

    /// Switch a Labor line off or on. Off, it stays on the sheet and is out of every total.
    pub fn set_off(&mut self, path: &str, line: Option<&str>, off: bool) -> Result<(), String> {
        self.edit_line(path, line, "switch", |l| l.off = off)
    }

    fn edit_line(&mut self, path: &str, line: Option<&str>, what: &str, edit: impl FnOnce(&mut CostLine)) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let mut new = self.world.get::<Cost>(e).cloned().ok_or_else(|| format!("{path} has no cost line to set a {what} on"))?;
        let at = Self::line_at(&new, path, line, what)?;
        edit(&mut new.lines[at]);
        check::cost(path, kind, &new)?;
        self.world.entity_mut(e).insert(new);
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
            return Err(if matches!(field, "stretch_out" | "quantity" | "waste" | "order" | "unit_cost") {
                format!("{path} has no cost line to set a {field} on")
            } else {
                format!("{field:?} is not a field that can be set on {path}; an element takes pitch, height, eave_height, assembly, wbs or allowance, a wall also takes bearing (bearing or not bearing) and in the wall (what runs in it), a roof drawn from its footprint takes edge:<n> (a pitch, or gable) and move:<n>, and a cost line takes stretch_out, quantity, waste, order, unit_cost or a value its formula names")
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
                // 60 SF, 24 LF, 2 CY, 3 SQ, 4 EA: a measure typed outright, as estimators
                // write it (1,200 SF, 60 sq ft). Where the value is a measure already (or is
                // the one a line is typed with), a length in feet is LF, and what is typed
                // that is not a measure is refused with the units a measure takes.
                (Some(text), was) => {
                    let measure_held = name == "measured" || was.is_some_and(|w| units::is_measure_unit(&w.unit));
                    match units::read_measure(text, measure_held) {
                        Some(read) => {
                            let (x, unit) = read?;
                            Ok(Val { value: Some(Len::One(x)), unit: unit.into(), status, source: source.into() })
                        }
                        None if measure_held => Err(format!("{what}: {}", units::parse_measure(text).unwrap_err())),
                        None => typed_length(Some(text), was, status, source),
                    }
                }
            }
        };
        // A height a line enters is never below nothing: a wainscot of -1 ft reads as 0 and says nothing (call 206 point 13). A
        // height is what a formula takes by `net_area_below_<name>` or `corner_board_length_above_<name>`, or a name that says it is one.
        let below_nothing = |name: &str, v: &Val<Len>, quantity: &str| -> Result<(), String> {
            let heights: Vec<String> = crate::formula::parse(quantity).map(|e| crate::formula::names(&e).iter().filter_map(|n| crate::formula::height_entry(n).map(str::to_string)).collect()).unwrap_or_default();
            let is_height = name.contains("height") || name == "wainscot" || heights.iter().any(|h| h == name);
            let feet = v.value.as_ref().and_then(|len| len.feet(&v.unit).ok());
            if is_height && !v.unit.is_empty() && !units::is_measure_unit(&v.unit) && feet.is_some_and(|f| f < 0.0) {
                return Err(format!("{what}: {} is below nothing ({}): a height a line enters is nothing or more", name.replace('_', " "), units::show_length(v.value.as_ref().unwrap_or(&Len::One(0.0)), &v.unit)));
            }
            Ok(())
        };
        match field {
            "stretch_out" => {
                let v = typed_length(value, l.stretch_out.as_ref(), status, source)?;
                below_nothing("stretch_out", &v, &l.quantity.clone())?;
                l.stretch_out = Some(v);
            }
            "quantity" => {
                let Some(text) = value else {
                    return Err(format!("{what}: a quantity is a formula, not a value; it cannot be marked unknown. Mark what it uses unknown instead"));
                };
                l.quantity = text.trim().to_string();
                for (name, typed) in with {
                    if name == lines::STRETCH_OUT {
                        let v = typed_length(Some(typed), l.stretch_out.as_ref(), status, source)?;
                        below_nothing(name, &v, &l.quantity.clone())?;
                        l.stretch_out = Some(v);
                    } else {
                        let v = named(name, Some(typed), l.inputs.get(name))?;
                        below_nothing(name, &v, &l.quantity.clone())?;
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
                        // In-house work says how fast it goes; anything else, how what
                        // is measured becomes what is bought (J23-5: a labor line's
                        // refusal offered "30 SF/sheet").
                        let in_hours = l.class.is_some_and(|c| c.in_hours()) || l.order.as_ref().is_some_and(|o| o.unit == HOURS) || text.contains("/hr") || text.contains("hr/");
                        let (is_not, how) = if in_hours { ("is not a production rate", format!("write how fast the work goes, like 40 {unit}/hr or 0.025 hr/{unit}")) } else { ("is not an order conversion", format!("write it as a number and what per what, like 30 {unit}/sheet or 1.25 lb/{unit}")) };
                        let (number, per) = text.trim().split_once(' ').ok_or_else(|| format!("{text:?} {is_not}; {how}"))?;
                        let factor = units::parse_number(number)?;
                        // An optional last word says whether whole ones are bought.
                        let (per, said) = match per.trim().rsplit_once(' ') {
                            Some((rest, "whole")) => (rest.trim(), Some(true)),
                            Some((rest, "exact")) => (rest.trim(), Some(false)),
                            _ => (per.trim(), None),
                        };
                        if per.chars().any(char::is_whitespace) {
                            return Err(format!("{text:?} {is_not}; {how}{}", if in_hours { "" } else { ", with whole or exact after it if it needs saying" }));
                        }
                        let (top, bottom) = per.split_once('/').ok_or_else(|| format!("{text:?} {is_not}; {how}"))?;
                        // A roofer's rate is in squares: a square is 100 SF, and a
                        // rate said in one is taken for a line that comes out in the
                        // other. ("1 SQ/hr" was refused on a line in SF, and the roofer
                        // left to do the arithmetic: J24A-7.) It is kept in the line's
                        // own unit, so the hours are one division.
                        let other = match unit { "SF" => Some(("SQ", 100.0)), "SQ" => Some(("SF", 0.01)), _ => None };
                        let (factor, top, bottom, per) = match other {
                            Some((sq, k)) if in_hours && top == sq && bottom == HOURS => (factor * k, unit, bottom, format!("{unit}/{HOURS}")),
                            Some((sq, k)) if in_hours && bottom == sq && top == HOURS => (factor / k, top, unit, format!("{HOURS}/{unit}")),
                            _ => (factor, top, bottom, per.to_string()),
                        };
                        let per = per.as_str();
                        let (order_unit, round_up) = if top == unit && bottom != unit {
                            // Whole ones for a package; hours are not bought whole.
                            (bottom, bottom != HOURS)
                        } else if bottom == unit && top != unit {
                            (top, false)
                        } else {
                            return Err(format!("{text:?} does not convert this line, which comes out in {unit}; {how}"));
                        };
                        // Buying whole ones is the rule for a package (a sheet, a box) and
                        // not for a weight; a line that already says keeps what it says.
                        let round_up = said.or(l.order.as_ref().filter(|o| o.unit == order_unit).map(|o| o.round_up)).unwrap_or(round_up);
                        l.order = Some(Order { unit: order_unit.to_string(), factor: Val { value: Some(factor), unit: per.to_string(), status, source: source.into() }, round_up });
                    }
                }
            }
            "unit_cost" => {
                // A price for what the line is priced in: its order unit, else its quantity's.
                let per = l.unit_cost.unit.clone();
                let price = value.map(|text| units::parse_price(text, &per)).transpose()?;
                l.unit_cost = Val { value: price, unit: per, status, source: source.into() };
            }
            name => {
                let v = named(name, value, l.inputs.get(name))?;
                below_nothing(name, &v, &l.quantity.clone())?;
                l.inputs.insert(name.to_string(), v);
            }
        }
        // The line may now come out in another unit. A price entered for the old unit does
        // not carry over; a line with no price yet just changes what it is priced by.
        let on = lines::On { kind, measure: None, height: None };
        if let Ok(read) = lines::read(&on, &new.lines[at]) {
            let priced = format!("$/{}", lines::priced_in(&new.lines[at], read.unit));
            let l = &mut new.lines[at];
            if !l.of_material && l.unit_cost.unit != priced {
                if l.unit_cost.value.is_some() {
                    return Err(format!("{what} is priced in {}; this change would make it a line in {priced}, and a price does not carry from one to the other. Mark its unit cost unknown first (set unit_cost unknown), make the change, then enter the price for the new unit", l.unit_cost.unit));
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
        self.apply_as(path, condition, Status::Known, source)
    }

    /// As `apply`, saying how sure the assignment is.
    pub fn apply_as(&mut self, path: &str, condition: &str, status: Status, source: &str) -> Result<Vec<String>, String> {
        if status == Status::Unknown {
            return Err("an assembly is applied as Known or Assumed; to say nobody knows it, mark the element's assembly unknown".into());
        }
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
            0 => return Err(format!("no assembly {condition:?} in this job")),
            _ => return Err(format!("{condition:?} names {} assemblies ({}); use the path", found.len(), found.iter().map(|f| f.0.clone()).collect::<Vec<_>>().join(", "))),
        };
        if !recipe.applies_to.contains(&kind) {
            return Err(format!("{name:?} applies to {}, and {path} is {kind:#}", recipe.applies_to.iter().map(|k| format!("{k:#}")).collect::<Vec<_>>().join(" or ")));
        }
        // An element is one condition. A second recipe's lines would sit under the first
        // one's name, or the first one's name would be written over.
        if let Some(have) = self.world.get::<Condition>(e).and_then(|c| c.0.value.clone()) {
            if have != name {
                return Err(format!("{path} is already {have:?}; applying {name:?} would put its lines under another assembly's name. To change it to this one, say so (at the command line, --change; in the window, pick it from the list)"));
            }
        }
        // The assembly it has already, put on again: only what it lacks of it goes on (a
        // line taken off with remove-line comes back), and no other line is touched.
        let has_it = self.world.get::<Condition>(e).and_then(|c| c.0.value.as_deref()) == Some(name.as_str());
        let lacking: Vec<CostLine> = match self.world.get::<Cost>(e) {
            Some(have) if has_it => recipe.cost.lines.iter().filter(|l| !have.lines.iter().any(|h| h.description == l.description)).cloned().collect(),
            _ => recipe.cost.lines.clone(),
        };
        if has_it && lacking.is_empty() && self.world.get::<Cost>(e).is_some() {
            return Err(format!("{path} is already {name:?} and has every line of it"));
        }
        let new = match self.world.get::<Cost>(e) {
            None => recipe.cost.clone(),
            Some(have) => {
                if have.code.value != recipe.cost.code.value {
                    return Err(format!("{path} already carries a cost code of its own, and {name:?} brings another; an element has one cost code"));
                }
                let mut joined = have.clone();
                joined.lines.extend(lacking.iter().cloned());
                joined
            }
        };
        check::cost(path, kind, &new)?;
        let assigned = Condition(Val { value: Some(name.clone()), unit: String::new(), status, source: source.into() });
        check::condition(path, &assigned)?;
        let added = lacking.iter().map(|l| l.description.clone()).collect();
        self.world.entity_mut(e).insert((new, assigned));
        Ok(added)
    }

    /// Give a thing an assembly in place of the one it has, if it has one. The lines the
    /// old assembly brought and the new one does not have are taken off (with whatever
    /// was priced on them: the reply names each); a line both have stays as it is, price
    /// and all; the rest of the new one's lines are added. Lines put on the thing by hand
    /// are not touched. Gives the lines added and the lines taken off.
    pub fn change_assembly(&mut self, path: &str, assembly: &str, status: Status, source: &str) -> Result<(Vec<String>, Vec<String>), String> {
        self.change_assembly_said(path, assembly, status, source).map(|c| (c.added, c.taken))
    }

    /// As `change_assembly`, and saying what became of the lines the two assemblies share (call 199). Such a line keeps
    /// what a person can have entered on it: its price, its waste, how it is ordered, whether it is off. Its quantity
    /// (the formula, and the entries and stretch-out the formula takes) becomes the new assembly's where the line still
    /// reads as the old assembly wrote it; where a person changed it, or the old assembly is no longer in the job to be
    /// compared with, or the new quantity comes out in another unit than the line is priced in, it stays as it is. Each
    /// is named, with the figure before and after.
    pub fn change_assembly_said(&mut self, path: &str, assembly: &str, status: Status, source: &str) -> Result<Changed, String> {
        self.change_assembly_in(path, assembly, status, source, None)
    }

    /// As `change_assembly_said`, for a thing set down from an item: the lines the program puts in the item's trade
    /// (`trade`) were not entered by a person, and are not named as if they were (call 206 point 1).
    pub fn change_assembly_in(&mut self, path: &str, assembly: &str, status: Status, source: &str, trade: Option<Division>) -> Result<Changed, String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let mut all: Vec<(String, String, Recipe)> = Vec::new();
        let mut q = self.world.query::<(&Node, &Recipe)>();
        for (node, t) in q.iter(&self.world) {
            all.push((node.path.clone(), node.name.clone(), t.clone()));
        }
        let have = self.world.get::<Condition>(e).and_then(|c| c.0.value.clone());
        let new: Vec<&(String, String, Recipe)> = all.iter().filter(|r| r.0 == assembly || r.1 == assembly).collect();
        let (Some(have), [new]) = (have, new.as_slice()) else {
            // Nothing to take the place of, or no one assembly of that name: as a first one is given.
            return self.apply_as(path, assembly, status, source).map(|added| Changed { added, ..Changed::default() });
        };
        // The same assembly again: what it lacks of it is put back, or it is said to be whole.
        if have == new.1 {
            return self.apply_as(path, assembly, status, source).map(|added| Changed { added, ..Changed::default() });
        }
        if !new.2.applies_to.contains(&kind) {
            return Err(format!("{:?} goes on {}, and {path} is a {}", new.1, new.2.applies_to.iter().map(|k| format!("a {}", k.said())).collect::<Vec<_>>().join(" or "), kind.said()));
        }
        let old = all.iter().find(|r| r.1 == have || r.0 == have);
        let was: Vec<String> = old.map(|r| r.2.cost.lines.iter().map(|l| l.description.clone()).collect()).unwrap_or_default();
        let will: Vec<String> = new.2.cost.lines.iter().map(|l| l.description.clone()).collect();
        let mut cost = self.world.get::<Cost>(e).cloned().unwrap_or_else(|| Cost { code: new.2.cost.code.clone(), lines: Vec::new() });
        let taken: Vec<String> = cost.lines.iter().filter(|l| was.contains(&l.description) && !will.contains(&l.description)).map(|l| l.description.clone()).collect();
        // What a person had entered on each line that comes off, against the line as the old assembly wrote it (call 206 point 1).
        let measure = self.world.get::<Measure>(e).cloned();
        let height = self.world.get::<Height>(e).cloned();
        let on = lines::On { kind, measure: measure.as_ref(), height: height.as_ref() };
        let took: Vec<crate::entered::Took> = cost
            .lines
            .iter()
            .filter(|l| taken.contains(&l.description))
            .map(|l| {
                let theirs = old.and_then(|r| r.2.cost.lines.iter().find(|o| o.description == l.description));
                let fig = lines::read(&on, l).ok().and_then(|r| r.worked.value.map(|v| format!("{} {}", crate::view::figure2(v), r.unit)));
                crate::entered::Took { line: l.description.clone(), entered: crate::entered::entered_on_item(l, theirs, fig.as_deref(), trade), amount: None }
            })
            .collect();
        cost.lines.retain(|l| !taken.contains(&l.description));
        // The code the old assembly brought gives way to the new one's.
        if old.is_some_and(|r| r.2.cost.code.value == cost.code.value) {
            cost.code = new.2.cost.code.clone();
        }
        // The lines both have: each keeps what was entered on it; its quantity is the new assembly's where it still reads as
        // the old assembly wrote it (call 199).
        let figure = |l: &CostLine| -> (Option<f64>, Option<&'static str>) {
            match lines::read(&on, l) {
                Ok(read) => (read.worked.value, Some(read.unit)),
                Err(_) => (None, None),
            }
        };
        let said = |l: &CostLine| -> String {
            match figure(l) {
                (Some(v), Some(unit)) => format!("{} {unit}", crate::view::figure2(v)),
                _ => "no figure".to_string(),
            }
        };
        let (mut rewritten, mut left) = (Vec::new(), Vec::new());
        let mut probes: Vec<(bool, usize, CostLine)> = Vec::new();
        for line in cost.lines.iter_mut() {
            let Some(nl) = new.2.cost.lines.iter().find(|n| n.description == line.description) else { continue };
            let reads_the_same = |a: &CostLine, b: &CostLine| a.quantity == b.quantity && same_entries(&a.inputs, &b.inputs) && same_stretch(&a.stretch_out, &b.stretch_out);
            // Nothing to say where the new assembly writes it as the line is.
            let would_change = line.quantity != nl.quantity || nl.inputs.iter().any(|(k, v)| !line.inputs.get(k).is_some_and(|h| h.value == v.value && h.unit == v.unit)) || !same_stretch(&line.stretch_out, &nl.stretch_out);
            if !would_change {
                continue;
            }
            let old_line = old.and_then(|r| r.2.cost.lines.iter().find(|o| o.description == line.description));
            let (has, has_written, would_written) = (said(line), written(line), written(nl));
            let why = match old_line {
                None if old.is_none() => Some(LeftAsIs::OldAssemblyGone),
                Some(o) if reads_the_same(line, o) => None,
                _ => Some(LeftAsIs::Entered),
            };
            if let Some(why) = why {
                probes.push((true, left.len(), nl.clone()));
                left.push(Left { line: line.description.clone(), has, has_written, would: String::new(), would_written, why });
                continue;
            }
            let mut candidate = line.clone();
            candidate.quantity = nl.quantity.clone();
            candidate.inputs = nl.inputs.clone();
            candidate.stretch_out = nl.stretch_out.clone();
            // The unit it comes out in has to be the one it is priced and ordered in: a price does not carry from one to another.
            let (units_now, units_new) = (figure(line).1, figure(&candidate).1);
            // (The new assembly's figure for it is worked out below, where the thing's figures include what the new
            // formula names.)
            if units_now.is_none() || units_now != units_new {
                probes.push((true, left.len(), nl.clone()));
                left.push(Left { line: line.description.clone(), has, has_written, would: String::new(), would_written, why: LeftAsIs::Unit });
                continue;
            }
            probes.push((false, rewritten.len(), candidate.clone()));
            rewritten.push(Rewritten { line: line.description.clone(), was: has, now: String::new(), was_written: has_written, now_written: would_written });
            *line = candidate;
        }
        let mut added = Vec::new();
        for l in &new.2.cost.lines {
            if !cost.lines.iter().any(|h| h.description == l.description) {
                cost.lines.push(l.clone());
                added.push(l.description.clone());
            }
        }
        check::cost(path, kind, &cost)?;
        let assigned = Condition(Val { value: Some(new.1.clone()), unit: String::new(), status, source: source.into() });
        check::condition(path, &assigned)?;
        // What each comes to as the new assembly writes it. A figure such as the wall's area below a wainscot is worked
        // out only where a line names it, so the new quantities are put on the thing beside its lines, its figures are
        // worked out again, and they are read; the thing then takes its lines and nothing else.
        if !probes.is_empty() {
            let mut probed = cost.clone();
            for (i, (_, _, l)) in probes.iter().enumerate() {
                let mut l = l.clone();
                l.description = format!("probe {i}");
                probed.lines.push(l);
            }
            self.world.entity_mut(e).insert(probed);
            self.derive();
            let measure = self.world.get::<Measure>(e).cloned();
            let on = lines::On { kind, measure: measure.as_ref(), height: height.as_ref() };
            let work = |l: &CostLine| match lines::read(&on, l) {
                Ok(read) => match read.worked.value {
                    Some(v) => format!("{} {}", crate::view::figure2(v), read.unit),
                    None => "no figure".to_string(),
                },
                Err(why) => format!("no figure ({why})"),
            };
            for (is_left, at, l) in &probes {
                let text = work(l);
                if *is_left {
                    left[*at].would = text;
                } else {
                    rewritten[*at].now = text;
                }
            }
        }
        self.world.entity_mut(e).insert((cost, assigned));
        self.derive();
        Ok(Changed { added, taken, took, rewritten, left })
    }

    /// A line that is on, or has just come off, a thing: what a person entered on it, against the line as the thing's assembly
    /// writes it, and its figure; the amount is for the caller, who has the estimate (call 206 point 1). The thing's item's
    /// trade, if it is set down from one, is the program's and not entered.
    pub fn took_of(&mut self, path: &str, l: &CostLine) -> crate::entered::Took {
        let theirs: Option<CostLine> = (|| {
            let e = self.find(path)?;
            let have = self.world.get::<Condition>(e)?.0.value.clone()?;
            let mut q = self.world.query::<(&Node, &Recipe)>();
            let recipe = q.iter(&self.world).find(|(n, _)| n.name == have || n.path == have).map(|(_, r)| r.clone())?;
            // The item line of a thing set down from an item carries the item's name, not the assembly's word.
            let made: Vec<&CostLine> = recipe.cost.lines.iter().filter(|o| o.of_material).collect();
            recipe.cost.lines.iter().find(|o| o.description == l.description).or(if l.of_material && made.len() == 1 { Some(made[0]) } else { None }).cloned()
        })();
        let from_item: Option<String> = self.find(path).and_then(|e| self.world.get::<crate::Placed>(e)).and_then(|p| p.item.clone());
        let trade = from_item.and_then(|i| self.find(&i)).and_then(|ie| self.world.get::<crate::Item>(ie).and_then(|i| i.division));
        let fig = self.line_figure(path, l);
        crate::entered::Took { line: l.description.clone(), entered: crate::entered::entered_on_item(l, theirs.as_ref(), fig.as_deref(), trade), amount: None }
    }

    /// A line as a figure with its unit ("7 EA"), as it comes out on the thing it is on: None where it comes to none.
    pub fn line_figure(&mut self, path: &str, l: &CostLine) -> Option<String> {
        let e = self.find(path)?;
        let kind = *self.world.get::<Kind>(e)?;
        let measure = self.world.get::<Measure>(e).cloned();
        let height = self.world.get::<Height>(e).cloned();
        let on = lines::On { kind, measure: measure.as_ref(), height: height.as_ref() };
        lines::read(&on, l).ok().and_then(|r| r.worked.value.map(|v| format!("{} {}", crate::view::figure2(v), r.unit)))
    }

    /// The unit a cost line comes out in, and the unit it is priced in.
    pub fn line_units(&mut self, path: &str, line: Option<&str>) -> Option<(String, String)> {
        let e = self.find(path)?;
        let kind = *self.world.get::<Kind>(e)?;
        let cost = self.world.get::<Cost>(e)?;
        let l = match line {
            Some(d) => cost.lines.iter().find(|l| l.description == d)?,
            None => cost.lines.first()?,
        };
        let read = lines::read(&lines::On { kind, measure: None, height: None }, l).ok()?;
        Some((read.unit.to_string(), l.unit_cost.unit.clone()))
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
            "bearing" => self.world.get::<Hidden>(e).and_then(|h| h.bearing.as_ref()).map(|b| b.value.map(|v| v.said().to_string()).unwrap_or_else(|| "unknown".into())),
            "in_the_wall" => self.world.get::<Hidden>(e).and_then(|h| h.in_the_wall.as_ref()).map(|b| b.value.clone().unwrap_or_else(|| "unknown".into())),
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

/// A line's entries are the same where each has the same names with the same values (how sure each is does not make them another).
fn same_entries(a: &BTreeMap<String, Val<Len>>, b: &BTreeMap<String, Val<Len>>) -> bool {
    a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|w| w.value == v.value && w.unit == v.unit))
}

fn same_stretch(a: &Option<Val<Len>>, b: &Option<Val<Len>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x.value == y.value && x.unit == y.unit,
        _ => false,
    }
}

/// A line's quantity as it is written: its formula, and the entries and stretch-out it takes.
pub(crate) fn written(l: &CostLine) -> String {
    let mut out = l.quantity.clone();
    if let Some(so) = &l.stretch_out {
        if let Some(len) = &so.value {
            out.push_str(&format!(", stretch out = {}", units::show_length(len, &so.unit)));
        }
    }
    for (name, v) in &l.inputs {
        match &v.value {
            Some(len) => out.push_str(&format!(", {} = {}", name.replace('_', " "), units::show_length(len, &v.unit))),
            None => out.push_str(&format!(", {} = not known", name.replace('_', " "))),
        }
    }
    out
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
