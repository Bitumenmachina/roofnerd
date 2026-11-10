//! The saved job. It is the world written out: one entry per element, each entry holding
//! that element's components under their `pl::` keys. The layout follows IFC5's
//! entity-and-component pattern (`header`, `imports`, `schemas`, `data`; each data entry a
//! `path` with `children` and `attributes`). It is plain JSON a person can read and diff.
//!
//! Saving is repeatable: the same world always writes the same bytes, so a diff between
//! two saves shows exactly what was edited and what followed from it, and nothing else.
//!
//! Loading checks the file. A value with no source, an Unknown carrying a number, or a
//! child that points nowhere is refused. Measures are recomputed from the geometry, and
//! any measure in the file that does not match its recomputed figure is reported.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::components::*;
use crate::job::{Header, Job};

#[derive(Serialize, Deserialize)]
struct FileRoot {
    header: Header,
    #[serde(default)]
    imports: Vec<Value>,
    #[serde(default)]
    schemas: BTreeMap<String, Value>,
    data: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    path: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    children: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    inherits: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    attributes: BTreeMap<String, Value>,
}

const KNOWN_KEYS: [&str; 10] = [K_CLASS, K_JOB, K_OUTLINE, K_PITCH, K_RUN, K_HEIGHT, K_CONDITION, K_COST, K_SCHEDULE, K_MEASURE];

/// What loading found that the reader should know about. Not errors: the job loaded.
#[derive(Debug, Default)]
pub struct LoadReport {
    /// Measures in the file that did not match what the geometry gives.
    pub stale: Vec<String>,
}

pub fn save(job: &mut Job) -> String {
    let children = job.children();
    let mut data = Vec::new();
    for &e in &job.order.clone() {
        let w = &job.world;
        let node = w.get::<Node>(e).expect("every element has a node");
        let mut a: BTreeMap<String, Value> = BTreeMap::new();
        macro_rules! put {
            ($t:ty, $k:expr) => {
                if let Some(c) = w.get::<$t>(e) {
                    a.insert($k.to_string(), serde_json::to_value(c).expect("components serialise"));
                }
            };
        }
        put!(Kind, K_CLASS);
        put!(JobInfo, K_JOB);
        put!(Outline, K_OUTLINE);
        put!(Pitch, K_PITCH);
        put!(Run, K_RUN);
        put!(Height, K_HEIGHT);
        put!(Condition, K_CONDITION);
        put!(Cost, K_COST);
        put!(WorkCode, K_SCHEDULE);
        if let Some(m) = w.get::<Measure>(e) {
            if !m.0.is_empty() {
                a.insert(K_MEASURE.to_string(), serde_json::to_value(m).expect("measures serialise"));
            }
        }
        if let Some(f) = w.get::<Foreign>(e) {
            for (k, v) in &f.0 {
                a.insert(k.clone(), v.clone());
            }
        }
        data.push(Entry { path: node.path.clone(), children: children.get(&node.path).cloned().unwrap_or_default(), inherits: BTreeMap::new(), attributes: a });
    }
    let schemas: BTreeMap<String, Value> = KNOWN_KEYS.iter().map(|k| (k.to_string(), json!({ "value": { "dataType": if *k == K_CLASS { "String" } else { "Object" } } }))).collect();
    let root = FileRoot { header: job.header.clone(), imports: job.imports.clone(), schemas, data };
    let mut out = String::new();
    write_pretty(&serde_json::to_value(&root).expect("the job serialises"), 0, &mut out);
    out.push('\n');
    out
}

pub fn load(text: &str) -> Result<(Job, LoadReport), String> {
    let root: FileRoot = serde_json::from_str(text).map_err(|e| format!("not a job file: {e}"))?;
    if root.data.is_empty() {
        return Err("the file has no elements".into());
    }
    let mut seen = BTreeSet::new();
    for entry in &root.data {
        if !seen.insert(entry.path.clone()) {
            return Err(format!("{} appears twice; a job has one entry per element", entry.path));
        }
    }
    // Who is whose child, and under what name.
    let mut parent_of: BTreeMap<String, (String, String)> = BTreeMap::new();
    for entry in &root.data {
        for (name, child) in &entry.children {
            if !seen.contains(child) {
                return Err(format!("{} lists child {name:?} at {child}, which is not in the file", entry.path));
            }
            if let Some((other, _)) = parent_of.insert(child.clone(), (entry.path.clone(), name.clone())) {
                return Err(format!("{child} is listed as a child of both {other} and {}", entry.path));
            }
        }
    }

    let mut job = Job { header: root.header, world: bevy_ecs::world::World::new(), order: Vec::new(), imports: root.imports };
    let mut file_measures: BTreeMap<String, Measure> = BTreeMap::new();
    let mut roots = 0;

    for entry in root.data {
        let path = entry.path.clone();
        let bad = |k: &str, e: serde_json::Error| format!("{path} {k}: {e}");
        let (parent, name) = match parent_of.get(&path) {
            Some((p, n)) => (Some(p.clone()), n.clone()),
            None => (None, path.clone()),
        };
        if parent.is_none() {
            roots += 1;
        }
        let id = job.world.spawn(Node { path: path.clone(), name, parent }).id();
        job.order.push(id);
        let mut foreign = BTreeMap::new();
        let mut has_class = false;
        for (k, v) in entry.attributes {
            let mut ent = job.world.entity_mut(id);
            match k.as_str() {
                K_CLASS => {
                    ent.insert(serde_json::from_value::<Kind>(v).map_err(|e| bad(&k, e))?);
                    has_class = true;
                }
                K_JOB => {
                    ent.insert(serde_json::from_value::<JobInfo>(v).map_err(|e| bad(&k, e))?);
                }
                K_OUTLINE => {
                    let c: Outline = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.points.check(&format!("{path} outline points"))?;
                    c.eave_height.check(&format!("{path} eave height"))?;
                    if let Some(pts) = &c.points.value {
                        if pts.len() < 3 {
                            return Err(format!("{path} outline: an area needs at least three points"));
                        }
                        if pts.iter().flatten().any(|x| !x.is_finite()) {
                            return Err(format!("{path} outline: a coordinate is not a usable number"));
                        }
                    }
                    ent.insert(c);
                }
                K_PITCH => {
                    let c: Pitch = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.0.check(&format!("{path} pitch"))?;
                    if c.0.value.is_some_and(|p| !p.is_finite() || p < 0.0) {
                        return Err(format!("{path} pitch: must be zero or more"));
                    }
                    ent.insert(c);
                }
                K_RUN => {
                    let c: Run = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.ends.check(&format!("{path} run ends"))?;
                    ent.insert(c);
                }
                K_HEIGHT => {
                    let c: Height = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.0.check(&format!("{path} height"))?;
                    if c.0.value.is_some_and(|h| !h.is_finite() || h < 0.0) {
                        return Err(format!("{path} height: must be zero or more"));
                    }
                    ent.insert(c);
                }
                K_CONDITION => {
                    let c: Condition = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.0.check(&format!("{path} condition"))?;
                    ent.insert(c);
                }
                K_COST => {
                    let c: Cost = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.code.check(&format!("{path} cost code"))?;
                    for line in &c.lines {
                        line.unit_cost.check(&format!("{path} cost line {:?} unit cost", line.description))?;
                    }
                    ent.insert(c);
                }
                K_SCHEDULE => {
                    let c: WorkCode = serde_json::from_value(v).map_err(|e| bad(&k, e))?;
                    c.wbs.check(&format!("{path} schedule code"))?;
                    ent.insert(c);
                }
                K_MEASURE => {
                    file_measures.insert(path.clone(), serde_json::from_value(v).map_err(|e| bad(&k, e))?);
                }
                _ => {
                    foreign.insert(k, v);
                }
            }
        }
        if !has_class {
            return Err(format!("{path} has no {K_CLASS}; every element says what it is"));
        }
        if !foreign.is_empty() {
            job.world.entity_mut(id).insert(Foreign(foreign));
        }
    }
    if roots != 1 {
        return Err(format!("the file has {roots} elements with no parent; a job has exactly one, the job itself"));
    }
    let root = job.root();
    if job.world.get::<JobInfo>(root).is_none() {
        return Err(format!("the first element must be the job and carry {K_JOB}"));
    }

    // Measures are never trusted from the file. Work them out again and compare.
    job.derive();
    let mut report = LoadReport::default();
    let mut q = job.world.query::<(&Node, Option<&Measure>)>();
    for (node, measure) in q.iter(&job.world) {
        let now = measure.cloned().unwrap_or_default();
        let then = file_measures.remove(&node.path).unwrap_or_default();
        if now != then {
            report.stale.push(node.path.clone());
        }
    }
    report.stale.sort();
    Ok((job, report))
}

/// JSON laid out for reading: one key per line, but a list of plain numbers (a point, a
/// row of points) stays on one line, so an outline reads as a row of coordinates.
fn write_pretty(v: &Value, depth: usize, out: &mut String) {
    match v {
        Value::Object(map) => write_object(map, depth, out),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
            } else if items.iter().all(is_flat) {
                out.push_str(&serde_json::to_string(v).expect("values serialise").replace(',', ", "));
            } else {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    indent(depth + 1, out);
                    write_pretty(item, depth + 1, out);
                    out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
                }
                indent(depth, out);
                out.push(']');
            }
        }
        other => out.push_str(&serde_json::to_string(other).expect("values serialise")),
    }
}

fn write_object(map: &Map<String, Value>, depth: usize, out: &mut String) {
    if map.is_empty() {
        out.push_str("{}");
        return;
    }
    out.push_str("{\n");
    let n = map.len();
    for (i, (k, v)) in map.iter().enumerate() {
        indent(depth + 1, out);
        out.push_str(&serde_json::to_string(k).expect("keys serialise"));
        out.push_str(": ");
        write_pretty(v, depth + 1, out);
        out.push_str(if i + 1 < n { ",\n" } else { "\n" });
    }
    indent(depth, out);
    out.push('}');
}

/// A number, or a list of numbers, or a list of those: something that reads fine on one line.
fn is_flat(v: &Value) -> bool {
    match v {
        Value::Number(_) => true,
        Value::Array(a) => a.iter().all(|x| x.is_number()),
        _ => false,
    }
}

fn indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}
