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

use serde::de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
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

const KNOWN_KEYS: [&str; 15] = [K_CLASS, K_JOB, K_FOOTPRINT, K_SOLVE, K_SOLVED, K_OUTLINE, K_PITCH, K_RUN, K_HEIGHT, K_CONDITION, K_COST, K_SCHEDULE, K_RECIPE, K_MEASURE, K_LINE_QUANTITIES];

/// What loading found that the reader should know about. Not errors: the job loaded.
#[derive(Debug, Default)]
pub struct LoadReport {
    /// Elements whose measures or line quantities in the file did not match what the
    /// geometry gives.
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
        put!(Footprint, K_FOOTPRINT);
        put!(SolveState, K_SOLVE);
        put!(Solved, K_SOLVED);
        put!(Outline, K_OUTLINE);
        put!(Pitch, K_PITCH);
        put!(Run, K_RUN);
        put!(Height, K_HEIGHT);
        put!(Condition, K_CONDITION);
        put!(Cost, K_COST);
        put!(WorkCode, K_SCHEDULE);
        put!(Recipe, K_RECIPE);
        if let Some(m) = w.get::<Measure>(e) {
            if !m.0.is_empty() {
                a.insert(K_MEASURE.to_string(), serde_json::to_value(m).expect("measures serialise"));
            }
        }
        if let Some(q) = w.get::<LineQuantities>(e) {
            if !q.0.is_empty() {
                a.insert(K_LINE_QUANTITIES.to_string(), serde_json::to_value(q).expect("line quantities serialise"));
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
    let strict: Strict = serde_json::from_str(text).map_err(|e| format!("not a job file: {e}"))?;
    let root: FileRoot = serde_json::from_value(strict.0).map_err(|e| format!("not a job file: {e}"))?;
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
    let mut file_line_quantities: BTreeMap<String, LineQuantities> = BTreeMap::new();
    let mut file_solve: BTreeMap<String, SolveState> = BTreeMap::new();

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
                K_FOOTPRINT => {
                    let c: Footprint = parse(&path, &k, v)?;
                    check::footprint(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_SOLVE => {
                    // Worked out, like a measure: read only to see whether it still holds.
                    let c: SolveState = parse(&path, &k, v)?;
                    file_solve.insert(path.clone(), c);
                }
                K_SOLVED => {
                    let c: Solved = parse(&path, &k, v)?;
                    check::solved(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_RECIPE => {
                    let c: Recipe = parse(&path, &k, v)?;
                    check::recipe(&path, kind, &c)?;
                    ent.insert(c);
                }
                K_MEASURE => {
                    file_measures.insert(path.clone(), parse(&path, &k, v)?);
                }
                K_LINE_QUANTITIES => {
                    file_line_quantities.insert(path.clone(), parse(&path, &k, v)?);
                }
                _ if near_miss(&k).is_some() => {
                    let near = near_miss(&k).unwrap_or(K_CLASS);
                    return Err(format!(
                        "{path} carries {k:?}, which is not one of this program's components but is spelled like one (did you mean {near:?}?); a value under a key the program does not read would sit in the file unused"
                    ));
                }
                _ => {
                    foreign.insert(k, v);
                }
            }
        }
        if !foreign.is_empty() {
            job.world.entity_mut(id).insert(Foreign(foreign));
        }
        if kind == Kind::Condition && job.world.get::<Recipe>(id).is_none() {
            return Err(format!("{path} is a Condition with no {K_RECIPE}; a condition is its recipe"));
        }
    }

    // The path counter has to be past every number already in use.
    let root_id = job.root();
    let next_id = job.world.get::<JobInfo>(root_id).map(|j| j.next_id).unwrap_or(0);
    for path in &seen {
        let digits = path.rsplit('-').next().unwrap_or("");
        if let Ok(n) = digits.parse::<u32>() {
            if n >= next_id {
                return Err(format!("the job's next_id is {next_id}, but {path} already uses {n}; next_id has to be higher than every path number in use"));
            }
        }
    }

    // What was solved from a footprint is never trusted from the file either. Every such
    // roof is solved again; what the file held for its planes and lines is compared with
    // what the footprint gives now, and a roof whose footprint can no longer be solved
    // does not keep the planes of an earlier solve as if they still stood.
    let solved_now = |job: &mut Job| -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let mut q = job.world.query::<(&Node, Option<&Solved>, Option<&Outline>, Option<&Pitch>, Option<&Run>)>();
        for (node, solved, outline, pitch, run) in q.iter(&job.world) {
            if solved.is_some() {
                out.insert(node.path.clone(), format!("{:?}", (&node.name, solved, outline, pitch, run)));
            }
        }
        out
    };
    let solved_before = solved_now(&mut job);
    crate::solve::solve_all(&mut job);
    let solved_after = solved_now(&mut job);
    let mut report = LoadReport::default();
    for path in solved_before.keys().chain(solved_after.keys()) {
        if solved_before.get(path) != solved_after.get(path) && !report.stale.contains(path) {
            report.stale.push(path.clone());
        }
    }
    {
        let mut q = job.world.query::<(&Node, Option<&SolveState>)>();
        for (node, state) in q.iter(&job.world) {
            let then = file_solve.remove(&node.path);
            if then.as_ref() != state && !report.stale.contains(&node.path) {
                report.stale.push(node.path.clone());
            }
        }
    }

    // Measures are never trusted from the file. Work them out again and compare.
    job.derive();
    let mut q = job.world.query::<(&Node, Option<&Measure>, Option<&LineQuantities>)>();
    for (node, measure, line_quantities) in q.iter(&job.world) {
        let now = measure.cloned().unwrap_or_default();
        let then = file_measures.remove(&node.path).unwrap_or_default();
        let now_lq = line_quantities.cloned().unwrap_or_default();
        let then_lq = file_line_quantities.remove(&node.path).unwrap_or_default();
        if (now != then || now_lq != then_lq) && !report.stale.contains(&node.path) {
            report.stale.push(node.path.clone());
        }
    }
    report.stale.sort();
    Ok((job, report))
}

/// One of this program's keys that `key` is a mistyping of, if it is one. Two ways:
/// the key is under our prefix, however the prefix and its colons are spelled (`PL::`,
/// `pl:`, ` pl ::`, `pl;;`), and is not one of our components; or its prefix is one
/// letter off ours (`pI::`, `p1::`) and the rest is exactly one of our names. Another
/// program's key (`ex::cost`, `ifc::class`) is neither, and is kept.
fn near_miss(key: &str) -> Option<&'static str> {
    let squeezed: String = key.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).map(|c| if matches!(c, ';' | '\u{FF1A}' | '\u{FF1B}') { ':' } else { c }).collect();
    if KNOWN_KEYS.contains(&key) {
        return None;
    }
    let (prefix, rest) = squeezed.split_once(':')?;
    let name = rest.trim_start_matches(':');
    let nearest = |name: &str| {
        // The known key whose name is fewest edits from this one.
        KNOWN_KEYS.iter().copied().min_by_key(|known| {
            let k: Vec<char> = known.trim_start_matches("pl::").chars().collect();
            let mut row: Vec<usize> = (0..=k.len()).collect();
            for (i, a) in name.chars().enumerate() {
                let mut prev = row[0];
                row[0] = i + 1;
                for (j, b) in k.iter().enumerate() {
                    let cost = if a == *b { prev } else { prev + 1 };
                    prev = row[j + 1];
                    row[j + 1] = cost.min(row[j] + 1).min(prev + 1);
                }
            }
            row[k.len()]
        })
    };
    if prefix == "pl" {
        return nearest(name);
    }
    let one_off = prefix.chars().count() == 2 && prefix.chars().zip("pl".chars()).filter(|(a, b)| a != b).count() == 1 && prefix.starts_with('p');
    if one_off && KNOWN_KEYS.iter().any(|known| known.trim_start_matches("pl::") == name) {
        return nearest(name);
    }
    None
}

/// JSON read strictly: the same key twice in one object is an error, anywhere in the
/// file. Left alone, the parser would keep the last one and drop the other without a word.
struct Strict(Value);

impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(StrictVisitor).map(Strict)
    }
}

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("JSON")
    }
    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v).map(Value::Number).ok_or_else(|| E::custom("a number is not finite"))
    }
    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_string()))
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut out = Vec::new();
        while let Some(item) = seq.next_element::<Strict>()? {
            out.push(item.0);
        }
        Ok(Value::Array(out))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut out = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if out.contains_key(&key) {
                return Err(de::Error::custom(format!("the key {key:?} appears twice in one object; one of them would be lost")));
            }
            let value = map.next_value::<Strict>()?;
            out.insert(key, value.0);
        }
        Ok(Value::Object(out))
    }
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
