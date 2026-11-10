//! The saved job. It is the world written out: one entry per element, each entry holding
//! that element's components under their `pl::` keys. The layout follows IFC5's
//! entity-and-component pattern (`header`, `imports`, `schemas`, `data`; each data entry a
//! `path` with `children` and `attributes`). It is plain JSON a person can read and diff.
//!
//! Saving is repeatable: the same world always writes the same bytes, so a diff between
//! two saves shows exactly what was edited and what followed from it, and nothing else.
//!
//! Loading checks the file against the rules in `check.rs` and refuses what breaks them.
//! Nothing is dropped quietly: a key this program does not know is either kept and
//! written back (at the top of the file, in the header, on an entry, or as an attribute)
//! or, inside one of this program's own components, refused with its name.
//! Measures are recomputed from the geometry, and any measure in the file that does not
//! match its recomputed figure is reported.

use std::collections::{BTreeMap, BTreeSet};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::check;
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
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
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
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

const KNOWN_KEYS: [&str; 10] = [K_CLASS, K_JOB, K_OUTLINE, K_PITCH, K_RUN, K_HEIGHT, K_CONDITION, K_COST, K_SCHEDULE, K_MEASURE];

/// What loading found that the reader should know about. Not errors: the job loaded.
#[derive(Debug, Default)]
pub struct LoadReport {
    /// Elements whose measures in the file did not match what the geometry gives.
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
        data.push(Entry {
            path: node.path.clone(),
            children: children.get(&node.path).cloned().unwrap_or_default(),
            inherits: node.inherits.clone(),
            attributes: a,
            extra: node.extra.clone(),
        });
    }
    // The file's own schema entries are kept; ours are added beside them.
    let mut schemas = job.schemas.clone();
    for k in KNOWN_KEYS {
        schemas.entry(k.to_string()).or_insert_with(|| json!({ "value": { "dataType": if k == K_CLASS { "String" } else { "Object" } } }));
    }
    let root = FileRoot { header: job.header.clone(), imports: job.imports.clone(), schemas, data, extra: job.extra.clone() };
    let mut out = String::new();
    write_pretty(&serde_json::to_value(&root).expect("the job serialises"), 0, &mut out);
    out.push('\n');
    out
}

fn parse<T: DeserializeOwned>(path: &str, key: &str, v: Value) -> Result<T, String> {
    serde_json::from_value(v).map_err(|e| format!("{path} {key}: {e}"))
}

pub fn load(text: &str) -> Result<(Job, LoadReport), String> {
    let root: FileRoot = serde_json::from_str(text).map_err(|e| format!("not a job file: {e}"))?;
    if root.data.is_empty() {
        return Err("the file has no elements".into());
    }
    let mut seen = BTreeSet::new();
    for entry in &root.data {
        if entry.path.trim().is_empty() {
            return Err("an element has a blank path".into());
        }
        if !seen.insert(entry.path.clone()) {
            return Err(format!("{} appears twice; a job has one entry per element", entry.path));
        }
    }
    // Who is whose child, and under what name.
    let mut parent_of: BTreeMap<String, (String, String)> = BTreeMap::new();
    for entry in &root.data {
        for (name, child) in &entry.children {
            if name.trim().is_empty() {
                return Err(format!("{} lists a child with a blank name", entry.path));
            }
            if !seen.contains(child) {
                return Err(format!("{} lists child {name:?} at {child}, which is not in the file", entry.path));
            }
            if let Some((other, _)) = parent_of.insert(child.clone(), (entry.path.clone(), name.clone())) {
                return Err(format!("{child} is listed as a child of both {other} and {}", entry.path));
            }
        }
    }
    // The tree: the first entry is the job, it alone has no parent, and every other
    // element leads back up to it. A loop of children cut off from the job is refused.
    let top = root.data[0].path.clone();
    if parent_of.contains_key(&top) {
        return Err(format!("the first element, {top}, has to be the job and cannot be anyone's child"));
    }
    for entry in &root.data[1..] {
        let mut at = &entry.path;
        let mut steps = 0;
        loop {
            match parent_of.get(at) {
                Some((parent, _)) => at = parent,
                None if *at == top => break,
                None => return Err(format!("{} is not under the job: {at} has no parent", entry.path)),
            }
            steps += 1;
            if steps > root.data.len() {
                return Err(format!("{} is in a loop of children that never reaches the job", entry.path));
            }
        }
    }

    let mut job = Job { header: root.header, world: bevy_ecs::world::World::new(), order: Vec::new(), imports: root.imports, schemas: root.schemas, extra: root.extra };
    for k in KNOWN_KEYS {
        // Ours are rewritten on save; do not keep a stale copy.
        if job.schemas.get(k) == Some(&json!({ "value": { "dataType": if k == K_CLASS { "String" } else { "Object" } } })) {
            job.schemas.remove(k);
        }
    }
    let mut file_measures: BTreeMap<String, Measure> = BTreeMap::new();

    for (i, entry) in root.data.into_iter().enumerate() {
        let path = entry.path.clone();
        let (parent, name) = match parent_of.get(&path) {
            Some((p, n)) => (Some(p.clone()), n.clone()),
            None => (None, path.clone()),
        };
        let mut attributes = entry.attributes;
        let kind: Kind = match attributes.remove(K_CLASS) {
            Some(v) => parse(&path, K_CLASS, v)?,
            None => return Err(format!("{path} has no {K_CLASS}; every element says what it is")),
        };
        if (kind == Kind::Job) != (i == 0) {
            return Err(if i == 0 { format!("the first element, {path}, has to be the job; it is a {kind:?}") } else { format!("{path} is a second job element; a file holds one job") });
        }
        if (i == 0) != attributes.contains_key(K_JOB) {
            return Err(if i == 0 { format!("the job element {path} must carry {K_JOB}") } else { format!("{path} carries {K_JOB}; only the job element does") });
        }
        let node = Node { path: path.clone(), name, parent, inherits: entry.inherits, extra: entry.extra };
        let id = job.world.spawn((node, kind)).id();
        job.order.push(id);
        let mut foreign = BTreeMap::new();
        for (k, v) in attributes {
            let mut ent = job.world.entity_mut(id);
            match k.as_str() {
                K_JOB => {
                    let c: JobInfo = parse(&path, &k, v)?;
                    check::job_info(&path, &c)?;
                    ent.insert(c);
                }
                K_OUTLINE => {
                    let c: Outline = parse(&path, &k, v)?;
                    check::outline(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_PITCH => {
                    let c: Pitch = parse(&path, &k, v)?;
                    check::pitch(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_RUN => {
                    let c: Run = parse(&path, &k, v)?;
                    check::run(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_HEIGHT => {
                    let c: Height = parse(&path, &k, v)?;
                    check::height(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_CONDITION => {
                    let c: Condition = parse(&path, &k, v)?;
                    check::condition(&path, &c)?;
                    ent.insert(c);
                }
                K_COST => {
                    let c: Cost = parse(&path, &k, v)?;
                    check::cost(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_SCHEDULE => {
                    let c: WorkCode = parse(&path, &k, v)?;
                    check::work_code(&path, &c)?;
                    ent.insert(c);
                }
                K_MEASURE => {
                    file_measures.insert(path.clone(), parse(&path, &k, v)?);
                }
                _ => {
                    foreign.insert(k, v);
                }
            }
        }
        if !foreign.is_empty() {
            job.world.entity_mut(id).insert(Foreign(foreign));
        }
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
