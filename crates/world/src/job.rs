//! The job: one world holding every element, plus the order they were added in.
//! Everything else in the program (the saved file, the sheet, the 3D view) is a way of
//! looking at this.

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

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
}

pub struct Job {
    pub header: Header,
    pub world: World,
    /// Entities in the order they were added. The file is written in this order.
    pub order: Vec<Entity>,
    /// Parts of the file kept as they were read.
    pub imports: Vec<serde_json::Value>,
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
            },
            world: World::new(),
            order: Vec::new(),
            imports: Vec::new(),
        };
        let root = job.world.spawn((
            Node { path: "job-0001".into(), name: slug(title), parent: None },
            Kind::Job,
            JobInfo { title: title.into(), length_unit: "ft".into(), next_id: 2 },
        ));
        let id = root.id();
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

    /// Add an element under `parent`. Its path is its kind and the job's next number,
    /// and it keeps that path for good.
    pub fn add(&mut self, parent: &str, kind: Kind, name: &str) -> String {
        let root = self.root();
        let mut info = self.world.get_mut::<JobInfo>(root).expect("the root carries the job info");
        let path = format!("{}-{:04}", kind.stem(), info.next_id);
        info.next_id += 1;
        let id = self.world.spawn((Node { path: path.clone(), name: name.into(), parent: Some(parent.into()) }, kind)).id();
        self.order.push(id);
        path
    }

    pub fn put<C: Component>(&mut self, path: &str, component: C) {
        let e = self.find(path).unwrap_or_else(|| panic!("no element {path}"));
        self.world.entity_mut(e).insert(component);
    }

    /// Recompute what follows from the geometry. Returns the paths whose measures changed.
    pub fn derive(&mut self) -> Vec<String> {
        let changed = derive::derive(&mut self.world);
        // Report in job order.
        let mut q = self.world.query::<&Node>();
        self.order.iter().filter_map(|e| q.get(&self.world, *e).ok()).map(|n| n.path.clone()).filter(|p| changed.contains(p)).collect()
    }

    /// Change one entered value by element path and field name. The fields are the ones an
    /// estimator would edit: pitch, height, eave_height, condition, wbs.
    pub fn set(&mut self, path: &str, field: &str, value: Option<&str>, status: Status, source: &str) -> Result<(), String> {
        let e = self.find(path).ok_or_else(|| format!("no element {path}"))?;
        let number = |unit: &str| -> Result<Val<f64>, String> {
            let v = match value {
                Some(s) => Some(s.parse::<f64>().map_err(|_| format!("{s:?} is not a number"))?),
                None => None,
            };
            if let Some(x) = v {
                if !x.is_finite() {
                    return Err(format!("{x} is not a usable number"));
                }
            }
            let val = Val { value: v, unit: unit.into(), status, source: source.into() };
            val.check(&format!("{path} {field}"))?;
            Ok(val)
        };
        let text = || -> Result<Val<String>, String> {
            let val = Val { value: value.map(str::to_string), unit: String::new(), status, source: source.into() };
            val.check(&format!("{path} {field}"))?;
            Ok(val)
        };
        let missing = |what: &str| format!("{path} has no {what} to set");
        match field {
            "pitch" => {
                let v = number("in/12")?;
                if v.value.is_some_and(|p| p < 0.0) {
                    return Err("a pitch cannot be negative".into());
                }
                let mut c = self.world.get_mut::<Pitch>(e).ok_or_else(|| missing("pitch"))?;
                c.0 = v;
            }
            "height" => {
                let v = number("ft")?;
                if v.value.is_some_and(|h| h < 0.0) {
                    return Err("a height cannot be negative".into());
                }
                let mut c = self.world.get_mut::<Height>(e).ok_or_else(|| missing("height"))?;
                c.0 = v;
            }
            "eave_height" => {
                let v = number("ft")?;
                let mut c = self.world.get_mut::<Outline>(e).ok_or_else(|| missing("outline"))?;
                c.eave_height = v;
            }
            "condition" => {
                let v = text()?;
                match self.world.get_mut::<Condition>(e) {
                    Some(mut c) => c.0 = v,
                    None => {
                        self.world.entity_mut(e).insert(Condition(v));
                    }
                }
            }
            "wbs" => {
                let v = text()?;
                match self.world.get_mut::<WorkCode>(e) {
                    Some(mut c) => c.wbs = v,
                    None => {
                        self.world.entity_mut(e).insert(WorkCode { wbs: v });
                    }
                }
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
