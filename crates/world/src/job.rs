//! The job: one world holding every element, plus the order they were added in.
//! Everything else in the program (the saved file, the sheet, the 3D view) is a way of
//! looking at this.

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::check;
use crate::components::*;
use crate::derive;
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

    /// Change one entered value by element path and field name. The fields are the ones an
    /// estimator would edit: pitch, height, eave_height, condition, wbs. The new value has
    /// to pass the same checks a loaded file does.
    pub fn set(&mut self, path: &str, field: &str, value: Option<&str>, status: Status, source: &str) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let kind = *self.world.get::<Kind>(e).ok_or_else(|| format!("{path} has no kind"))?;
        let number = |unit: &str| -> Result<Val<f64>, String> {
            let v = match value {
                // Adding zero turns a typed "-0" into plain 0.
                Some(s) => Some(s.parse::<f64>().map_err(|_| format!("{s:?} is not a number"))? + 0.0),
                None => None,
            };
            if v.is_some_and(|x| !x.is_finite()) {
                return Err(format!("{} is not a usable number", value.unwrap_or_default()));
            }
            Ok(Val { value: v, unit: unit.into(), status, source: source.into() })
        };
        let text = || Val { value: value.map(str::to_string), unit: String::new(), status, source: source.into() };
        let missing = |what: &str| format!("{path} has no {what} to set");
        match field {
            "pitch" => {
                let new = Pitch(number(U_PITCH)?);
                check::pitch(path, kind, &new)?;
                *self.world.get_mut::<Pitch>(e).ok_or_else(|| missing("pitch"))? = new;
            }
            "height" => {
                let new = Height(number(U_LENGTH)?);
                check::height(path, kind, &new)?;
                *self.world.get_mut::<Height>(e).ok_or_else(|| missing("height"))? = new;
            }
            "eave_height" => {
                let mut new = self.world.get::<Outline>(e).ok_or_else(|| missing("outline"))?.clone();
                new.eave_height = number(U_LENGTH)?;
                check::outline(path, kind, &new)?;
                *self.world.get_mut::<Outline>(e).ok_or_else(|| missing("outline"))? = new;
            }
            "condition" => {
                let new = Condition(text());
                check::condition(path, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            "wbs" => {
                let new = WorkCode { wbs: text() };
                check::work_code(path, &new)?;
                self.world.entity_mut(e).insert(new);
            }
            other => return Err(format!("{other:?} is not a field that can be set; use pitch, height, eave_height, condition or wbs")),
        }
        Ok(())
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
