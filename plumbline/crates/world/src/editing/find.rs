//! One search of the job by name (call 219 point 7, the model's side).
//!
//! What can be found by name, and where each name is: a level (its `Node` name; a `Storey`), a room (`Room`), a thing set
//! down (a fixture, a piece of equipment, a furnishing: named for what it is and numbered, "Lavatory 2"), what is built (a
//! wall, a footing, a floor, a door, a window, a run, a beam, a post, a roof and its planes), a type (a wall's, a floor's, a
//! door's or window's, a run's), an item of the tool chest (a `Type` that carries an `Item`), a system, a material, and
//! each line of the estimate by its description. Nothing is found by a path: a path is how the saved file tells things
//! apart and not how a builder does.
//!
//! The order of what is found: a name that is the whole of what was typed comes first, then a name that begins with it,
//! then a name that has every word of it in it (BIMCraft's Find is the last rule alone: `crates/ui-egui/src/find.rs`
//! `matches`, every word, in any capitals). Within each, a level, a room, a thing set down, what is built, an item, a
//! type, a system, a material, a line of the estimate; and within that, the job's own order. Capitals do not matter.
//! The window's commands and its keys are the window's to find; they are not in the job.

use crate::components::*;
use crate::job::Job;
use crate::view;
use crate::{FloorType, Item, OpeningType, SegmentType, WallType};

/// What was found, and where.
#[derive(Clone, Debug, PartialEq)]
pub struct NameFound {
    /// The thing, by path (a line of the estimate: its thing's).
    pub path: String,
    pub name: String,
    pub kind: Kind,
    /// The word for what it is ("level", "room", "item", "wall type", "estimate line").
    pub is: String,
    /// The level it is on, by name, where it is on one.
    pub level: Option<String>,
    /// The whole of what was typed is its name.
    pub exact: bool,
    /// An estimate line's amount; none where it has none, and none for a thing.
    pub amount: Option<f64>,
    /// The line's description, for a line of the estimate.
    pub line: Option<String>,
}

impl NameFound {
    /// "Lavatory 2: fixture, on Ground" / "Siding: estimate line on wall 1, $1,234.50".
    pub fn said(&self) -> String {
        let on = self.level.as_ref().map(|l| format!(", on {l}")).unwrap_or_default();
        match &self.line {
            Some(d) => format!("{d}: {} of {}{on}, {}", self.is, self.name, self.amount.map(view::money).unwrap_or_else(|| "no price yet".to_string())),
            None => format!("{}: {}{on}", self.name, self.is),
        }
    }
}

/// What is said where nothing was found.
pub fn nothing_said(text: &str) -> String {
    if text.trim().is_empty() { "type a name, or a word of one".to_string() } else { format!("nothing in the job is called {:?}, and no line of the estimate has it", text.trim()) }
}

fn words_in(label: &str, text: &str) -> bool {
    let label = label.to_lowercase();
    let mut words = text.split_whitespace().peekable();
    words.peek().is_some() && words.all(|w| label.contains(&w.to_lowercase()))
}

/// How well a name fits what was typed: 0 is the whole of it, 1 begins with it, 2 has every word, none is no match.
fn rank(label: &str, text: &str) -> Option<u8> {
    let (l, t) = (label.trim().to_lowercase(), text.trim().to_lowercase());
    if t.is_empty() {
        return None;
    }
    if l == t {
        Some(0)
    } else if l.starts_with(&t) {
        Some(1)
    } else if words_in(label, text) {
        Some(2)
    } else {
        None
    }
}

fn level_of(job: &mut Job, path: &str) -> Option<String> {
    let mut at = path.to_string();
    for _ in 0..64 {
        let e = job.find(&at)?;
        let n = job.world.get::<Node>(e)?.clone();
        if job.world.get::<Kind>(e) == Some(&Kind::Storey) && at != path {
            return Some(n.name);
        }
        at = n.parent.clone()?;
    }
    None
}

/// What a thing is called as a kind of thing, and where it sorts among the rest.
fn is_and_group(job: &mut Job, e: bevy_ecs::entity::Entity, kind: Kind) -> (String, u8) {
    let w = &job.world;
    match kind {
        Kind::Storey => ("level".into(), 0),
        Kind::Room => ("room".into(), 1),
        Kind::Fixture | Kind::Equipment | Kind::Furnishing => (kind.said().into(), 2),
        Kind::Type => {
            if w.get::<Item>(e).is_some() {
                ("item in the tool chest".into(), 4)
            } else if let Some(t) = w.get::<WallType>(e) {
                (if t.footing { "footing type" } else { "wall type" }.into(), 5)
            } else if w.get::<FloorType>(e).is_some() {
                ("floor type".into(), 5)
            } else if let Some(t) = w.get::<OpeningType>(e) {
                (format!("{} type", t.category.said()), 5)
            } else if let Some(t) = w.get::<SegmentType>(e) {
                (format!("{} type", t.category.said()), 5)
            } else {
                ("type".into(), 5)
            }
        }
        Kind::System => ("system".into(), 6),
        Kind::Material => ("material".into(), 7),
        other => (other.said().into(), 3),
    }
}

/// Everything in the job that has a name and that `text` fits, in the order above. Nothing is changed.
pub fn find_names(job: &mut Job, text: &str) -> Vec<NameFound> {
    let order = job.order.clone();
    let mut hits: Vec<((u8, u8, usize), NameFound)> = Vec::new();
    for (i, e) in order.into_iter().enumerate() {
        let (Some(node), Some(kind)) = (job.world.get::<Node>(e).cloned(), job.world.get::<Kind>(e).copied()) else { continue };
        if kind == Kind::Job {
            continue;
        }
        let Some(r) = rank(&node.name, text) else { continue };
        let (is, group) = is_and_group(job, e, kind);
        let level = level_of(job, &node.path);
        hits.push(((r, group, i), NameFound { path: node.path, name: node.name, kind, is, level, exact: r == 0, amount: None, line: None }));
    }
    hits.sort_by_key(|h| h.0);
    hits.into_iter().map(|h| h.1).collect()
}

/// The lines of the estimate that `text` fits, by their description, and what each comes to.
pub fn find_lines(job: &mut Job, text: &str) -> Vec<NameFound> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let sheet = view::estimate(job);
    let mut hits: Vec<((u8, usize), NameFound)> = Vec::new();
    for (i, l) in sheet.lines.iter().enumerate() {
        let Some(r) = rank(&l.description, text) else { continue };
        let level = level_of(job, &l.path);
        hits.push(((r, i), NameFound { path: l.path.clone(), name: l.element.clone(), kind: l.kind, is: "estimate line".into(), level, exact: r == 0, amount: l.amount.worked(), line: Some(l.description.clone()) }));
    }
    hits.sort_by_key(|h| h.0);
    hits.into_iter().map(|h| h.1).collect()
}

/// Names first, then lines of the estimate: one search.
pub fn find_in_job(job: &mut Job, text: &str) -> Vec<NameFound> {
    let mut out = find_names(job, text);
    out.extend(find_lines(job, text));
    out
}
