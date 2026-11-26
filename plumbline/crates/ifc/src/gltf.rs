//! The house written out as glTF 2.0 in one binary file (.glb), to be looked at in a browser, on a phone or in any 3D program with
//! nothing installed (call 229).
//!
//! What is written is what the BIM file's writer writes a body for, from the one source of solids (`plumbline_world::solids`): one
//! named part for each wall (its openings cut), footing, door, window, floor, room, roof plane, run, fitting, fixture, piece of
//! equipment and thing set down; the parts stand under the level they are on, and the roof's planes under their roof.
//!
//! **Measure and axes are the format's** (Khronos glTF 2.0, `research/gltf.md`): metres; right-handed, +Y up, the asset's front
//! facing +Z. A place of the job, `(x east, y north, z up)` in feet, is written at `(x, z, -y)` metres: a quarter turn about the east
//! axis, never a mirror. North is -Z, so the house's south face is the asset's front.
//!
//! **Colour is plain and comes from what the estimate prices**: a wall's cladding, a roof's roofing, concrete, a kind of thing. A
//! pattern (lap, brick, shingle) is not carried. Prices, certainty and what a value was entered from are not carried: this is a
//! model to look at, and the BIM file and the workbook carry the rest.
//!
//! The layout (a 12-byte header, a JSON chunk, a binary chunk; positions as 32-bit floats with their bounds, faces as 32-bit
//! whole numbers; no normals, which the format gives a flat face) follows BIMCraft's `crates/core/src/gltf.rs` (MIT), written
//! again here for these parts.

use plumbline_world::solids::{self, Body, Finish, Part};
use plumbline_world::{finish::Cladding, finish::Roofing, Job, Kind};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

const FEET: f64 = 0.3048;
const FLOAT: u32 = 5126;
const UNSIGNED_INT: u32 = 5125;
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;

/// What writing the file did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Glb {
    /// The file.
    pub bytes: Vec<u8>,
    /// How many parts: one for each thing with a body.
    pub parts: usize,
    /// How many of each kind of thing, in the words the BIM file's reply uses.
    pub made: BTreeMap<String, usize>,
    /// What was not carried, in a builder's words: each a line.
    pub not_carried: Vec<String>,
}

/// What the plan's place `(east, north, up)` in feet is in the format's: metres, `(x, z, -y)`. A negative zero is a zero.
pub fn placed(p: [f64; 3]) -> [f32; 3] {
    [(p[0] * FEET + 0.0) as f32, (p[2] * FEET + 0.0) as f32, (-p[1] * FEET + 0.0) as f32]
}

/// The word the BIM file's reply counts a kind of thing by.
fn made_word(kind: Kind) -> &'static str {
    match kind {
        Kind::Wall => "walls",
        Kind::Footing => "footings",
        Kind::PadFooting => "pad footings",
        Kind::Door => "doors",
        Kind::Window => "windows",
        Kind::Floor => "floors",
        Kind::Room => "rooms",
        Kind::RoofPlane => "roof planes",
        Kind::Duct => "ducts",
        Kind::Pipe => "pipes",
        Kind::Conduit => "conduits",
        Kind::Cable => "cables",
        Kind::Beam => "beams",
        Kind::Post => "posts",
        Kind::Fitting => "fittings",
        Kind::Furnishing => "furnishings",
        Kind::Fixture => "fixtures",
        _ => "pieces of equipment",
    }
}

/// A plain colour (as trades read colour: 8 bits a channel), how much it hides (1 hides all), and what it is said to be.
struct Paint {
    word: &'static str,
    rgb: [u8; 3],
    alpha: f64,
}

fn paint(part: &Part) -> Paint {
    let p = |word: &'static str, rgb: [u8; 3]| Paint { word, rgb, alpha: 1.0 };
    let mut out = match part.finish {
        Finish::Cladding(c) => p(
            c.said(),
            match c {
                Cladding::LapSiding => [214, 205, 190],
                Cladding::VerticalSiding => [200, 190, 170],
                Cladding::BoardAndBatten => [190, 180, 160],
                Cladding::Shingles => [150, 120, 95],
                Cladding::Panel => [205, 200, 190],
                Cladding::MetalPanel => [150, 155, 160],
                Cladding::Brick => [165, 85, 65],
                Cladding::Block => [160, 160, 158],
                Cladding::Stone => [140, 135, 125],
                Cladding::Stucco => [225, 215, 195],
            },
        ),
        Finish::Roofing(r) => p(
            r.said(),
            match r {
                Roofing::AsphaltShingles => [85, 85, 90],
                Roofing::StandingSeam => [110, 115, 120],
                Roofing::MetalPanel => [125, 130, 135],
                Roofing::MetalShingles => [115, 120, 125],
                Roofing::Slate => [70, 75, 85],
                Roofing::Tile => [170, 90, 60],
                Roofing::Shakes => [130, 100, 70],
                Roofing::Membrane => [60, 60, 62],
            },
        ),
        Finish::Concrete => p("concrete", [150, 150, 148]),
        Finish::Bare => match part.kind {
            Kind::Wall => p("no finish priced", [225, 222, 215]),
            Kind::RoofPlane => p("no roofing priced", [120, 120, 125]),
            Kind::Door => p("door", [130, 90, 55]),
            Kind::Window => Paint { word: "glass", rgb: [150, 190, 215], alpha: 0.45 },
            Kind::Floor => p("floor", [190, 170, 140]),
            Kind::Room => Paint { word: "room", rgb: [120, 170, 220], alpha: 0.15 },
            Kind::Duct => p("duct", [170, 175, 180]),
            Kind::Pipe => p("pipe", [60, 120, 180]),
            Kind::Conduit => p("conduit", [200, 160, 60]),
            Kind::Cable => p("cable", [200, 60, 60]),
            Kind::Beam | Kind::Post => p("lumber", [200, 165, 110]),
            Kind::Fitting => p("fitting", [150, 150, 150]),
            Kind::Fixture => p("fixture", [235, 235, 240]),
            Kind::Furnishing => p("furnishing", [170, 130, 90]),
            _ => p("equipment", [120, 130, 140]),
        },
    };
    // A colour a priced line names ("charcoal") is the colour.
    if let Some(rgb) = part.tone {
        out.rgb = rgb;
    }
    out
}

/// A colour's channel from the 8 bits trades read, as the format's factors are, in linear light.
fn linear(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

#[derive(Default)]
struct File {
    positions: Vec<u8>,
    indices: Vec<u8>,
    accessors: Vec<Value>,
    materials: Vec<Value>,
    material_of: HashMap<(String, [u8; 3], u32), usize>,
    meshes: Vec<Value>,
    nodes: Vec<Value>,
}

impl File {
    fn material(&mut self, paint: &Paint, double: bool) -> usize {
        let key = (paint.word.to_string(), paint.rgb, (paint.alpha * 100.0) as u32 + if double { 1000 } else { 0 });
        if let Some(i) = self.material_of.get(&key) {
            return *i;
        }
        let [r, g, b] = paint.rgb;
        let mut m = json!({
            "name": paint.word,
            "pbrMetallicRoughness": {"baseColorFactor": [linear(r), linear(g), linear(b), paint.alpha], "metallicFactor": 0.0, "roughnessFactor": 0.9},
            "doubleSided": double,
        });
        if paint.alpha < 1.0 {
            m["alphaMode"] = json!("BLEND");
        }
        self.materials.push(m);
        self.material_of.insert(key, self.materials.len() - 1);
        self.materials.len() - 1
    }

    /// A part as a mesh and a node; the node's index.
    fn part(&mut self, part: &Part) -> usize {
        let (points, faces): (Vec<[f32; 3]>, Option<&Vec<[u32; 3]>>) = match &part.body {
            Body::Solid(m) | Body::Surface(m) => (m.points.iter().map(|p| placed(*p)).collect(), Some(&m.faces)),
            Body::Line([a, b]) => (vec![placed(*a), placed(*b)], None),
        };
        let at = self.positions.len();
        self.positions.extend(points.iter().flatten().flat_map(|v| v.to_le_bytes()));
        let bound = |pick: fn(f32, f32) -> f32, start: f32| -> Vec<f64> { (0..3).map(|k| f64::from(points.iter().map(|p| p[k]).fold(start, pick))).collect() };
        self.accessors.push(json!({"bufferView": 0, "byteOffset": at, "componentType": FLOAT, "count": points.len(), "type": "VEC3", "min": bound(f32::min, f32::INFINITY), "max": bound(f32::max, f32::NEG_INFINITY)}));
        let position = self.accessors.len() - 1;
        let double = !matches!(part.body, Body::Solid(_)) || part.kind == Kind::Room;
        let material = self.material(&paint(part), double);
        let mut primitive = json!({"attributes": {"POSITION": position}, "material": material});
        match faces {
            Some(faces) => {
                let from = self.indices.len();
                self.indices.extend(faces.iter().flatten().flat_map(|i| i.to_le_bytes()));
                self.accessors.push(json!({"bufferView": 1, "byteOffset": from, "componentType": UNSIGNED_INT, "count": faces.len() * 3, "type": "SCALAR"}));
                primitive["indices"] = json!(self.accessors.len() - 1);
                primitive["mode"] = json!(4);
            }
            None => primitive["mode"] = json!(1),
        }
        self.meshes.push(json!({"name": part.name, "primitives": [primitive]}));
        let paint = paint(part);
        self.nodes.push(json!({"name": part.name, "mesh": self.meshes.len() - 1, "extras": {"path": part.path, "kind": made_word(part.kind), "finish": paint.word}}));
        self.nodes.len() - 1
    }

    fn group(&mut self, name: &str, children: Vec<usize>, extras: Value) -> usize {
        self.nodes.push(json!({"name": name, "children": children, "extras": extras}));
        self.nodes.len() - 1
    }
}

/// What the file does not carry, in a builder's words: what is always so, then what this job's parts left out.
fn left_out(not_carried: &[(String, usize, String)]) -> Vec<String> {
    let mut out = vec![
        "prices, and how sure each value is (Known, Assumed or Unknown): it is a model to look at, and the estimate workbook and the BIM file carry them".to_string(),
        "the patterns the window draws (lap siding, brick, shingles): each part is one plain colour from what the estimate prices on it".to_string(),
        "what a thing set down is drawn as in the window (a tub, a sink): each is its box, as the BIM file has it".to_string(),
        "the inside face of a wall, its frame and its layers, and the frames, sashes and glass of a window: a window is one pane of glass the size of its opening".to_string(),
        "systems and what is connected to what".to_string(),
        "a roof plane has no thickness: it is the face the BIM file writes".to_string(),
    ];
    for (what, n, why) in not_carried {
        out.push(if *n == 1 { format!("{what}: {why}") } else { format!("{n} {what}s: {why}") });
    }
    out
}

/// The house as a .glb.
pub fn export(job: &mut Job) -> Result<Glb, String> {
    let all = solids::solids(job)?;
    if all.parts.is_empty() {
        return Err("the job has no wall, floor, roof or other thing with a shape to write".into());
    }
    let mut file = File::default();
    let mut made: BTreeMap<String, usize> = BTreeMap::new();
    let mut on_level: HashMap<Option<String>, Vec<usize>> = HashMap::new();
    let mut on_roof: HashMap<String, Vec<usize>> = HashMap::new();
    for part in &all.parts {
        *made.entry(made_word(part.kind).to_string()).or_insert(0) += 1;
        let node = file.part(part);
        match &part.roof {
            Some(roof) => on_roof.entry(roof.clone()).or_default().push(node),
            None => on_level.entry(part.level.clone()).or_default().push(node),
        }
    }
    let mut under_root: Vec<usize> = Vec::new();
    for level in &all.levels {
        if let Some(children) = on_level.remove(&Some(level.path.clone())) {
            under_root.push(file.group(&level.name, children, json!({"path": level.path, "kind": "levels", "elevation_m": level.elevation * FEET})));
        }
    }
    for roof in &all.roofs {
        if let Some(children) = on_roof.remove(&roof.path) {
            *made.entry("roofs".to_string()).or_insert(0) += 1;
            under_root.push(file.group(&roof.name, children, json!({"path": roof.path, "kind": "roofs"})));
        }
    }
    under_root.extend(on_level.remove(&None).unwrap_or_default());
    let root = file.group(&all.title, under_root, json!({"kind": "building"}));
    let not_carried = left_out(&all.not_carried);

    let mut bin = std::mem::take(&mut file.positions);
    let indices_at = bin.len();
    bin.extend(&file.indices);
    // The faces' view is only there when some part has faces (a view of no bytes is not allowed).
    let mut views = vec![json!({"buffer": 0, "byteOffset": 0, "byteLength": indices_at, "byteStride": 12, "target": ARRAY_BUFFER})];
    if bin.len() > indices_at {
        views.push(json!({"buffer": 0, "byteOffset": indices_at, "byteLength": bin.len() - indices_at, "target": ELEMENT_ARRAY_BUFFER}));
    }
    let document = json!({
        "asset": {
            "version": "2.0",
            "generator": "BuildOS",
            "extras": {"units": "metres", "up": "+Y", "north": "-Z: the plan's north is behind the asset, and its south face is the front (+Z)", "not carried": not_carried},
        },
        "scene": 0,
        "scenes": [{"nodes": [root]}],
        "nodes": file.nodes,
        "meshes": file.meshes,
        "materials": file.materials,
        "accessors": file.accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": bin.len()}],
    });
    let mut json = serde_json::to_vec(&document).map_err(|e| format!("glTF: {e}"))?;
    json.resize(json.len().next_multiple_of(4), b' ');
    bin.resize(bin.len().next_multiple_of(4), 0);
    let total = 12 + 8 + json.len() + 8 + bin.len();
    let too_big = || format!("glTF: {total} bytes, more than one file of this kind holds (4 GB)");
    let (total32, json32, bin32) = (u32::try_from(total).map_err(|_| too_big())?, u32::try_from(json.len()).map_err(|_| too_big())?, u32::try_from(bin.len()).map_err(|_| too_big())?);
    let mut bytes = Vec::with_capacity(total);
    bytes.extend(b"glTF");
    bytes.extend(2u32.to_le_bytes());
    bytes.extend(total32.to_le_bytes());
    bytes.extend(json32.to_le_bytes());
    bytes.extend(b"JSON");
    bytes.extend(json);
    bytes.extend(bin32.to_le_bytes());
    bytes.extend(b"BIN\0");
    bytes.extend(bin);
    Ok(Glb { bytes, parts: all.parts.len(), made, not_carried })
}
