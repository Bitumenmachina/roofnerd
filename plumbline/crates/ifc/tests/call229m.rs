//! Call 229 (contract 229M): the house written out as glTF. The job is the made-up whole house, priced in all sixteen divisions
//! (`sample::house_priced`): 40 ft by 28 ft on its centre lines, walls 6 in thick, the first floor at 0, the second at 10 ft.
//! Figures worked by hand beside each test and in STATUS.md ("The house written out as glTF: a demonstration").
//!
//! The reader here is written for the test and reads the file by its own table of contents: it checks every length, every bound and
//! every number the file says of itself, then hands the parts back. The BIM file is read by a small evaluator of the plain shapes the
//! writer writes (a rectangle, a circle or an outline pushed along an axis; a face), so each part is held to the BIM file's own box.

use plumbline_ifc::step::{self, V};
use plumbline_ifc::{gltf, write};
use plumbline_world::*;
use serde_json::Value;
use std::collections::HashMap;

const FT: f64 = 0.3048;

// ---- The glTF reader -------------------------------------------------------------------------------------------------------------

struct Read {
    json: Value,
    bin: Vec<u8>,
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

/// The file by its own table of contents. Every check is an assertion; a file that is cut short, mis-sized or mis-bounded fails here.
fn read(bytes: &[u8]) -> Read {
    assert!(bytes.len() >= 20, "shorter than a header and a chunk");
    assert_eq!(&bytes[0..4], b"glTF", "magic");
    assert_eq!(u32_at(bytes, 4), 2, "version");
    assert_eq!(u32_at(bytes, 8) as usize, bytes.len(), "the length the header gives is the file's length");
    let json_len = u32_at(bytes, 12) as usize;
    assert_eq!(json_len % 4, 0, "chunk length a multiple of four");
    assert_eq!(&bytes[16..20], b"JSON", "the first chunk is the JSON");
    let json: Value = serde_json::from_slice(&bytes[20..20 + json_len]).expect("the JSON chunk is JSON");
    let at = 20 + json_len;
    assert!(at + 8 <= bytes.len(), "a second chunk follows");
    let bin_len = u32_at(bytes, at) as usize;
    assert_eq!(bin_len % 4, 0, "chunk length a multiple of four");
    assert_eq!(&bytes[at + 4..at + 8], b"BIN\0", "the second chunk is the binary");
    assert_eq!(at + 8 + bin_len, bytes.len(), "no chunk after the binary and none cut short");
    let bin = bytes[at + 8..].to_vec();
    let r = Read { json, bin };
    assert_eq!(r.json["asset"]["version"], "2.0");
    // The buffer.
    let buffers = r.json["buffers"].as_array().unwrap();
    assert_eq!(buffers.len(), 1);
    assert!(buffers[0].get("uri").is_none(), "the buffer is the file's own binary chunk");
    assert!(buffers[0]["byteLength"].as_u64().unwrap() as usize <= r.bin.len(), "the buffer is no longer than the chunk");
    // Every buffer view lies inside the buffer, and a stride is a multiple of four.
    for v in r.json["bufferViews"].as_array().unwrap() {
        let (o, l) = (v["byteOffset"].as_u64().unwrap() as usize, v["byteLength"].as_u64().unwrap() as usize);
        assert!(l > 0 && o + l <= buffers[0]["byteLength"].as_u64().unwrap() as usize, "a view inside the buffer");
        if let Some(s) = v.get("byteStride") {
            assert_eq!(s.as_u64().unwrap() % 4, 0);
        }
    }
    // Every accessor lies inside its view, aligned, and states the bounds its data has.
    let materials = r.json["materials"].as_array().unwrap().len();
    let accessors = r.json["accessors"].as_array().unwrap();
    for (n, a) in accessors.iter().enumerate() {
        let view = &r.json["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
        let (size, comps) = (4usize, match a["type"].as_str().unwrap() { "SCALAR" => 1, "VEC3" => 3, t => panic!("type {t}") });
        assert!(matches!(a["componentType"].as_u64().unwrap(), 5126 | 5125), "a component type the file uses");
        let offset = a["byteOffset"].as_u64().unwrap_or(0) as usize;
        assert_eq!(offset % 4, 0, "accessor {n} aligned");
        let stride = view.get("byteStride").map_or(size * comps, |s| s.as_u64().unwrap() as usize);
        let count = a["count"].as_u64().unwrap() as usize;
        assert!(count > 0);
        assert!(offset + (count - 1) * stride + size * comps <= view["byteLength"].as_u64().unwrap() as usize, "accessor {n} inside its view");
        if a["componentType"] == 5126 && a["type"] == "VEC3" {
            let pts = r.points(n);
            for k in 0..3 {
                let lo = pts.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min) as f64;
                let hi = pts.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max) as f64;
                assert_eq!(a["min"][k].as_f64().unwrap(), lo, "accessor {n} min {k}");
                assert_eq!(a["max"][k].as_f64().unwrap(), hi, "accessor {n} max {k}");
            }
        }
    }
    // Every primitive: a position accessor with bounds, indices in range and in threes, a material that is there.
    for m in r.json["meshes"].as_array().unwrap() {
        for p in m["primitives"].as_array().unwrap() {
            let pos = p["attributes"]["POSITION"].as_u64().unwrap() as usize;
            assert!(accessors[pos].get("min").is_some() && accessors[pos].get("max").is_some());
            assert!((p["material"].as_u64().unwrap() as usize) < materials);
            let count = accessors[pos]["count"].as_u64().unwrap() as usize;
            match p["mode"].as_u64().unwrap_or(4) {
                4 => {
                    let idx = r.indices(p["indices"].as_u64().unwrap() as usize);
                    assert_eq!(idx.len() % 3, 0, "triangles in threes");
                    assert!(idx.iter().all(|i| (*i as usize) < count), "every index names a point");
                }
                1 => assert_eq!(count % 2, 0),
                m => panic!("mode {m}"),
            }
        }
    }
    // The nodes are strict trees: one parent at most, no cycles, the scene's roots are parentless.
    let nodes = r.json["nodes"].as_array().unwrap();
    let mut parent: Vec<Option<usize>> = vec![None; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        for c in n["children"].as_array().into_iter().flatten() {
            let c = c.as_u64().unwrap() as usize;
            assert!(parent[c].is_none(), "node {c} has two parents");
            parent[c] = Some(i);
        }
    }
    for i in 0..nodes.len() {
        let (mut at, mut steps) = (i, 0);
        while let Some(p) = parent[at] {
            at = p;
            steps += 1;
            assert!(steps <= nodes.len(), "a cycle");
        }
    }
    for root in r.json["scenes"][0]["nodes"].as_array().unwrap() {
        assert!(parent[root.as_u64().unwrap() as usize].is_none());
    }
    r
}

impl Read {
    fn view_bytes(&self, accessor: usize) -> (&[u8], usize, usize) {
        let a = &self.json["accessors"][accessor];
        let v = &self.json["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
        let start = v["byteOffset"].as_u64().unwrap() as usize + a["byteOffset"].as_u64().unwrap_or(0) as usize;
        let stride = v.get("byteStride").map_or(0, |s| s.as_u64().unwrap() as usize);
        (&self.bin[start..], stride, a["count"].as_u64().unwrap() as usize)
    }
    fn points(&self, accessor: usize) -> Vec<[f32; 3]> {
        let (b, stride, count) = self.view_bytes(accessor);
        let stride = if stride == 0 { 12 } else { stride };
        (0..count).map(|i| [0, 1, 2].map(|k| f32::from_le_bytes(b[i * stride + k * 4..i * stride + k * 4 + 4].try_into().unwrap()))).collect()
    }
    fn indices(&self, accessor: usize) -> Vec<u32> {
        let (b, _, count) = self.view_bytes(accessor);
        (0..count).map(|i| u32_at(b, i * 4)).collect()
    }
    /// Each part: its node's name, its path, its kind and its points and faces, in the format's metres and axes.
    fn parts(&self) -> Vec<Part> {
        let mut out = Vec::new();
        let nodes = self.json["nodes"].as_array().unwrap();
        let mut level_of: HashMap<usize, String> = HashMap::new();
        for g in nodes.iter().filter(|g| g["extras"]["kind"] == "levels") {
            for c in g["children"].as_array().unwrap() {
                level_of.insert(c.as_u64().unwrap() as usize, g["name"].as_str().unwrap().to_string());
            }
        }
        for (at, n) in nodes.iter().enumerate() {
            let Some(mesh) = n.get("mesh") else { continue };
            let prim = &self.json["meshes"][mesh.as_u64().unwrap() as usize]["primitives"][0];
            let points = self.points(prim["attributes"]["POSITION"].as_u64().unwrap() as usize);
            let faces = prim.get("indices").map(|i| self.indices(i.as_u64().unwrap() as usize)).unwrap_or_default();
            out.push(Part {
                name: n["name"].as_str().unwrap().to_string(),
                level: level_of.get(&at).cloned(),
                path: n["extras"]["path"].as_str().unwrap().to_string(),
                kind: n["extras"]["kind"].as_str().unwrap().to_string(),
                mode: prim["mode"].as_u64().unwrap_or(4),
                material: prim["material"].as_u64().unwrap() as usize,
                points,
                faces,
            });
        }
        out
    }
}

struct Part {
    name: String,
    level: Option<String>,
    path: String,
    kind: String,
    mode: u64,
    material: usize,
    points: Vec<[f32; 3]>,
    faces: Vec<u32>,
}

impl Part {
    fn bounds(&self) -> [[f64; 3]; 2] {
        let mut b = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
        for p in &self.points {
            for k in 0..3 {
                b[0][k] = b[0][k].min(p[k] as f64);
                b[1][k] = b[1][k].max(p[k] as f64);
            }
        }
        b
    }
    /// Cubic metres, by the signed volumes under each triangle.
    fn volume(&self) -> f64 {
        let p = |i: u32| self.points[i as usize].map(f64::from);
        self.faces.chunks(3).map(|f| {
            let (a, b, c) = (p(f[0]), p(f[1]), p(f[2]));
            (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0])) / 6.0
        }).sum()
    }
}

// ---- The BIM file's boxes ---------------------------------------------------------------------------------------------------------

type V3 = [f64; 3];

#[derive(Clone, Copy)]
struct Frame {
    o: V3,
    x: V3,
    y: V3,
    z: V3,
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

impl Frame {
    fn apply(&self, p: V3) -> V3 {
        [0, 1, 2].map(|k| self.o[k] + self.x[k] * p[0] + self.y[k] * p[1] + self.z[k] * p[2])
    }
    fn then(&self, child: &Frame) -> Frame {
        let turn = |d: V3| [0, 1, 2].map(|k| self.x[k] * d[0] + self.y[k] * d[1] + self.z[k] * d[2]);
        Frame { o: self.apply(child.o), x: turn(child.x), y: turn(child.y), z: turn(child.z) }
    }
}

struct Ifc {
    f: step::File,
}

impl Ifc {
    fn e(&self, id: u64) -> &step::Entity {
        &self.f.entities[&id]
    }
    fn r(v: &V) -> u64 {
        match v { V::Ref(i) => *i, v => panic!("not a reference: {v:?}") }
    }
    fn num(v: &V) -> f64 {
        match v { V::Real(r) => *r, V::Int(i) => *i as f64, V::Typed(_, b) => Self::num(b), v => panic!("not a number: {v:?}") }
    }
    fn list(v: &V) -> &Vec<V> {
        match v { V::List(l) => l, v => panic!("not a list: {v:?}") }
    }
    fn vec(&self, id: u64) -> Vec<f64> {
        Self::list(&self.e(id).args[0]).iter().map(Self::num).collect()
    }
    fn axes3(&self, id: u64) -> Frame {
        let e = self.e(id);
        let o = self.vec(Self::r(&e.args[0]));
        let z = if let V::Ref(i) = &e.args[1] { self.vec(*i) } else { vec![0.0, 0.0, 1.0] };
        let x = if let V::Ref(i) = &e.args[2] { self.vec(*i) } else { vec![1.0, 0.0, 0.0] };
        let (z, x): (V3, V3) = (z.try_into().unwrap(), x.try_into().unwrap());
        Frame { o: o.try_into().unwrap(), x, y: cross(z, x), z }
    }
    fn placement(&self, id: u64) -> Frame {
        let e = self.e(id);
        assert_eq!(e.name, "IFCLOCALPLACEMENT");
        let own = self.axes3(Self::r(&e.args[1]));
        match &e.args[0] {
            V::Ref(parent) => self.placement(*parent).then(&own),
            _ => own,
        }
    }
    /// A 2-D profile's outline in its own plane, and whether it is round (its radius).
    fn profile(&self, id: u64) -> (Vec<[f64; 2]>, Option<f64>) {
        let e = self.e(id);
        let centre = |v: &V| -> [f64; 2] { let p = self.vec(Self::r(&self.e(Self::r(v)).args[0])); [p[0], p[1]] };
        match e.name.as_str() {
            "IFCRECTANGLEPROFILEDEF" => {
                let c = centre(&e.args[2]);
                let (x, y) = (Self::num(&e.args[3]) / 2.0, Self::num(&e.args[4]) / 2.0);
                (vec![[c[0] - x, c[1] - y], [c[0] + x, c[1] - y], [c[0] + x, c[1] + y], [c[0] - x, c[1] + y]], None)
            }
            "IFCCIRCLEPROFILEDEF" => {
                let c = centre(&e.args[2]);
                let r = Self::num(&e.args[3]);
                (vec![[c[0] - r, c[1] - r], [c[0] + r, c[1] + r]], Some(r))
            }
            "IFCARBITRARYCLOSEDPROFILEDEF" => {
                let line = self.e(Self::r(&e.args[2]));
                (Self::list(&line.args[0]).iter().map(|p| { let v = self.vec(Self::r(p)); [v[0], v[1]] }).collect(), None)
            }
            n => panic!("profile {n}"),
        }
    }
    /// The box of one thing in feet, in the job's axes: lowest and highest of each, and whether it has a round section.
    /// None where it has no shape.
    fn box_of(&self, id: u64) -> Option<([V3; 2], bool)> {
        let e = self.e(id);
        let V::Ref(rep) = &e.args[6] else { return None };
        let at = self.placement(Self::r(&e.args[5]));
        let mut pts: Vec<V3> = Vec::new();
        let mut round = false;
        let reps = Self::list(&self.e(*rep).args[2]);
        let chosen = reps.iter().find(|r| self.e(Self::r(r)).args[1] == V::Str("Body".into())).or_else(|| reps.first()).unwrap();
        for item in Self::list(&self.e(Self::r(chosen)).args[3]) {
            let it = self.e(Self::r(item));
            match it.name.as_str() {
                "IFCEXTRUDEDAREASOLID" => {
                    let (outline, circle) = self.profile(Self::r(&it.args[0]));
                    let depth = Self::num(&it.args[3]);
                    let place = self.axes3(Self::r(&it.args[1]));
                    if let Some(r) = circle {
                        round = true;
                        // A circle: each end's centre, and as far each way as the circle reaches on that axis.
                        let c = [(outline[0][0] + outline[1][0]) / 2.0, (outline[0][1] + outline[1][1]) / 2.0];
                        let frame = at.then(&place);
                        for w in [0.0, depth] {
                            let centre = frame.apply([c[0], c[1], w]);
                            for k in 0..3 {
                                let reach = r * (frame.x[k] * frame.x[k] + frame.y[k] * frame.y[k]).sqrt();
                                let (mut lo, mut hi) = (centre, centre);
                                lo[k] -= reach;
                                hi[k] += reach;
                                pts.push(lo);
                                pts.push(hi);
                            }
                        }
                    } else {
                        for w in [0.0, depth] {
                            for p in &outline {
                                pts.push(at.then(&place).apply([p[0], p[1], w]));
                            }
                        }
                    }
                }
                "IFCPOLYGONALFACESET" => {
                    let list = self.e(Self::r(&it.args[0]));
                    for p in Self::list(&list.args[0]) {
                        let v: Vec<f64> = Self::list(p).iter().map(Self::num).collect();
                        pts.push(at.apply([v[0], v[1], v[2]]));
                    }
                }
                "IFCPOLYLINE" => {
                    for p in Self::list(&it.args[0]) {
                        let v = self.vec(Self::r(p));
                        pts.push(at.apply([v[0], v[1], *v.get(2).unwrap_or(&0.0)]));
                    }
                }
                n => panic!("a shape {n}"),
            }
        }
        let mut b = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
        for p in &pts {
            for k in 0..3 {
                b[0][k] = b[0][k].min(p[k]);
                b[1][k] = b[1][k].max(p[k]);
            }
        }
        Some((b, round))
    }
    /// The things that have a body (or a line) in the file: not the holes, which are only cut.
    fn things(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.f.entities.iter().filter(|(_, e)| e.name != "IFCOPENINGELEMENT" && e.args.len() >= 7 && matches!((&e.args[5], &e.args[6]), (V::Ref(_), V::Ref(_)))).map(|(i, _)| *i).collect();
        ids.sort_unstable();
        ids
    }
}

/// The job's place as the format has it: `(x, z, -y)` metres, from feet.
fn as_gltf(b: [V3; 2]) -> [V3; 2] {
    let m = |p: V3| [p[0] * FT, p[2] * FT, -p[1] * FT];
    let (a, c) = (m(b[0]), m(b[1]));
    [[a[0].min(c[0]), a[1].min(c[1]), a[2].min(c[2])], [a[0].max(c[0]), a[1].max(c[1]), a[2].max(c[2])]]
}

struct Both {
    title: String,
    glb: gltf::Glb,
    read: Read,
    ifc: Ifc,
    ifc_text: String,
}

fn both() -> Both {
    let mut job = sample::house_priced();
    let title = write::job_title(&mut job);
    let ifc_text = write::export(&mut job, "house.ifc", "2026-11-26T10:00:00").unwrap().text;
    let glb = gltf::export(&mut job).unwrap();
    let read = read(&glb.bytes);
    let ifc = Ifc { f: step::parse(ifc_text.as_bytes()).unwrap() };
    Both { title, glb, read, ifc, ifc_text }
}

// ---- The tests ---------------------------------------------------------------------------------------------------------------------

#[test]
fn the_file_is_whole_by_its_own_table_of_contents() {
    let b = both();
    let parts = b.read.parts();
    assert_eq!(parts.len(), b.glb.parts);
    // Cut short anywhere, it no longer reads.
    for cut in [10usize, 19, 40, b.glb.bytes.len() / 2, b.glb.bytes.len() - 1] {
        assert!(std::panic::catch_unwind(|| read(&b.glb.bytes[..cut])).is_err(), "cut at {cut} read");
    }
}

#[test]
fn one_part_for_each_thing_the_bim_file_writes_a_shape_for() {
    let b = both();
    let parts = b.read.parts();
    let things = b.ifc.things();
    // 238 things in the BIM file with a shape or a line: 20 walls, 4 footings, 9 doors, 17 windows, 2 floors, 10 rooms, 4 roof planes,
    // 16 cables, 2 conduits, 23 ducts, 63 pipes, 39 fittings (41 written, 2 with no size and so no body), 22 fixtures, 5 pieces of
    // equipment: 20+4+9+17+2+10+4+16+2+23+63+39+22+5 = 236.
    assert_eq!(things.len(), 236, "the BIM file's things with a shape");
    assert_eq!(parts.len(), 236);
    // The same things: each part's path made an id by the BIM file's own rule is a thing of the BIM file.
    let ids: HashMap<String, u64> = things.iter().map(|i| match &b.ifc.e(*i).args[0] { V::Str(s) => (s.clone(), *i), v => panic!("{v:?}") }).collect();
    let mut seen = std::collections::HashSet::new();
    for p in &parts {
        let id = write::guid(&b.title, &p.path);
        assert!(ids.contains_key(&id), "{} ({}) is not a thing of the BIM file", p.name, p.path);
        assert!(seen.insert(id), "{} twice", p.path);
    }
    // And the kinds come out as the BIM file counts them.
    let count = |k: &str| parts.iter().filter(|p| p.kind == k).count();
    for (k, n) in [("walls", 20), ("footings", 4), ("doors", 9), ("windows", 17), ("floors", 2), ("rooms", 10), ("roof planes", 4), ("fixtures", 22), ("fittings", 39), ("pipes", 63), ("ducts", 23), ("cables", 16), ("conduits", 2), ("pieces of equipment", 5)] {
        assert_eq!(count(k), n, "{k}");
        assert_eq!(b.glb.made[k], n);
    }
}

#[test]
fn each_part_lies_in_the_bim_files_box_in_the_formats_metres_and_axes() {
    let b = both();
    let by_id: HashMap<String, u64> = b.ifc.things().iter().map(|i| match &b.ifc.e(*i).args[0] { V::Str(s) => (s.clone(), *i), v => panic!("{v:?}") }).collect();
    let mut round = 0;
    for p in b.read.parts() {
        let (theirs, was_round) = b.ifc.box_of(by_id[&write::guid(&b.title, &p.path)]).unwrap();
        let want = as_gltf(theirs);
        let got = p.bounds();
        // A round section is a polygon of 24 sides, inside its circle by under 1 % of its radius; nothing else is off by more than a
        // thirty-second of a millimetre (the file keeps lengths as 32-bit numbers).
        let tolerance = if was_round { 0.001 } else { 0.00003 };
        round += was_round as usize;
        for k in 0..3 {
            assert!((got[0][k] - want[0][k]).abs() <= tolerance && (got[1][k] - want[1][k]).abs() <= tolerance, "{} ({}): axis {k}: the file has {:?}..{:?}, the BIM file's box is {:?}..{:?}", p.name, p.kind, got[0][k], got[1][k], want[0][k], want[1][k]);
        }
    }
    assert!(round > 50, "the round runs were held to a box too ({round})");
}

#[test]
fn a_corner_of_the_house_by_hand_is_where_the_axes_put_it_and_the_house_is_not_mirrored() {
    let b = both();
    let parts = b.read.parts();
    let first = parts.iter().find(|p| p.name == "south wall" && p.level.as_deref() == Some("first floor")).unwrap();
    // The first floor's south wall is drawn from (0, 0) to (40, 0), 6 in thick, and runs past both ends by half the thickness of the walls
    // it corners with (J20-6): from x = -0.25 ft to 40.25 ft, y from -0.25 to 0.25, standing from z = 0. Its low outside corner on the
    // south-west is (-0.25, -0.25, 0) ft = (-0.0762, 0, 0.0762) in the format: x as it is, up from z, north as -Z so south is +Z.
    let lo = first.bounds()[0];
    assert!((lo[0] - -0.0762).abs() < 1e-5 && lo[1].abs() < 1e-6 && (first.bounds()[1][2] - 0.0762).abs() < 1e-5, "{:?}", first.bounds());
    // Its far end: x = 40.25 ft = 12.2682 m.
    assert!((first.bounds()[1][0] - 12.2682).abs() < 1e-4);
    // The whole house stands at least 40 ft by 28 ft (12.192 m by 8.534 m) and no more than that by the eaves' overhang and the wall's
    // own thickness; its lowest point is at or below the foundation's floor, -4 ft = -1.2192 m.
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    // (The first floor's walls: 40 ft by 28 ft on their centre lines and 40.5 ft by 28.5 ft with their thickness, which is 12.3444 m by
    // 8.6868 m. The foundation's walls under them are 8 in thick: 40 2/3 ft by 28 2/3 ft, 12.3952 m by 8.7376 m.)
    for p in parts.iter().filter(|p| p.kind == "walls" && p.level.as_deref() == Some("first floor")) {
        for k in 0..3 {
            lo[k] = lo[k].min(p.bounds()[0][k]);
            hi[k] = hi[k].max(p.bounds()[1][k]);
        }
    }
    assert!(hi[0] - lo[0] >= 12.192 && hi[0] - lo[0] <= 12.192 + 0.4, "east to west {}", hi[0] - lo[0]);
    assert!(hi[2] - lo[2] >= 8.534 && hi[2] - lo[2] <= 8.534 + 0.4, "north to south {}", hi[2] - lo[2]);
    assert!((hi[0] - lo[0] - 12.3444).abs() < 1e-4 && (hi[2] - lo[2] - 8.6868).abs() < 1e-4, "{:?} {:?}", lo, hi);
    // The lowest point of anything is at or below the foundation's floor, -4 ft = -1.2192 m.
    let lowest = parts.iter().map(|p| p.bounds()[0][1]).fold(f64::INFINITY, f64::min);
    assert!(lowest <= -1.2192 + 1e-4, "lowest {lowest}");
    // Not mirrored: the front door is on the south side of the house's middle (y = 14 ft is Z = -4.2672 m), and the kitchen sink on the north.
    let middle = -14.0 * FT;
    let door = parts.iter().find(|p| p.name == "front door").unwrap();
    let door_z = (door.bounds()[0][2] + door.bounds()[1][2]) / 2.0;
    assert!(door_z > middle, "the front door is at Z {door_z}, the middle is {middle}");
    let sink = parts.iter().find(|p| p.name.to_lowercase().contains("kitchen sink") && p.kind == "fixtures").unwrap();
    let sink_z = (sink.bounds()[0][2] + sink.bounds()[1][2]) / 2.0;
    assert!(sink_z < middle, "the kitchen sink is at Z {sink_z}, the middle is {middle}");
    // The handedness holds in the data too: a wall's faces face out (a positive volume), none inside out.
    for p in parts.iter().filter(|p| p.mode == 4 && p.kind != "roof planes") {
        assert!(p.volume() > 0.0, "{} is inside out or empty: {}", p.name, p.volume());
    }
}

#[test]
fn each_part_has_the_name_the_clash_list_uses() {
    let mut job = sample::house_priced();
    let found = building::clash::clashes(&mut job, None).unwrap();
    let list = found.in_the_same_space();
    let glb = gltf::export(&mut job).unwrap();
    let parts = read(&glb.bytes).parts();
    // Every thing the clash list names is a part, by the path the list gives it, under the very name the list gives it: 11 clashes, 22
    // names, among them a return grille the list calls "Return grille, 20 × 20 (return grille, first floor)".
    let mut held = 0;
    for c in &list {
        for (path, name) in [(&c.a, &c.a_name), (&c.b, &c.b_name)] {
            let part = parts.iter().find(|p| &p.path == path).unwrap_or_else(|| panic!("{path} is in the clash list and not in the file"));
            assert_eq!(&part.name, name, "{path}");
            held += 1;
        }
    }
    assert_eq!(held, 22);
    assert!(parts.iter().any(|p| p.name == "Return grille, 20 × 20 (return grille, first floor)"));
    // A door and a window go by the name the model gives them, as the BIM file names them: no part is nameless but a door or a window the
    // model gives no name.
    let nameless = parts.iter().filter(|p| p.name.is_empty()).count();
    assert!(nameless <= 2, "{nameless}");
}

#[test]
fn a_wall_with_openings_has_the_holes_cut_and_its_volume_is_its_boxes_less_theirs() {
    let b = both();
    let parts = b.read.parts();
    // The first floor's south wall has the front door and two windows in it. Its volume is the BIM file's wall box less the three
    // BIM file openings (their own boxes), in cubic metres.
    let ids: HashMap<String, u64> = b.ifc.things().iter().map(|i| match &b.ifc.e(*i).args[0] { V::Str(s) => (s.clone(), *i), v => panic!("{v:?}") }).collect();
    let volume = |bx: [V3; 2]| (0..3).map(|k| bx[1][k] - bx[0][k]).product::<f64>() * FT * FT * FT;
    let mut tried = 0;
    for p in parts.iter().filter(|p| p.kind == "walls") {
        let wall = ids[&write::guid(&b.title, &p.path)];
        let whole = volume(b.ifc.box_of(wall).unwrap().0);
        // The openings that cut it: each IfcRelVoidsElement from it.
        let mut cut = 0.0;
        let mut holes = 0;
        for e in b.ifc.f.entities.values().filter(|e| e.name == "IFCRELVOIDSELEMENT") {
            if Ifc::r(&e.args[4]) == wall {
                let hole = b.ifc.e(Ifc::r(&e.args[5]));
                assert_eq!(hole.name, "IFCOPENINGELEMENT");
                let id = b.ifc.f.entities.iter().find(|(_, x)| std::ptr::eq(*x, hole)).map(|(i, _)| *i).unwrap();
                cut += volume(b.ifc.box_of(id).unwrap().0);
                holes += 1;
            }
        }
        assert!((p.volume() - (whole - cut)).abs() < 1e-4, "{} ({holes} openings): {} against {} - {}", p.name, p.volume(), whole, cut);
        if holes > 0 {
            tried += 1;
            assert!(p.volume() < whole - 1e-3, "{} has its openings cut", p.name);
        }
    }
    assert!(tried >= 4, "walls with openings were held to it ({tried})");
}

#[test]
fn colour_is_plain_and_comes_from_what_is_finished() {
    let b = both();
    let parts = b.read.parts();
    let colour = |p: &Part| b.read.json["materials"][p.material]["pbrMetallicRoughness"]["baseColorFactor"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>();
    let finish = |p: &Part| b.read.json["materials"][p.material]["name"].as_str().unwrap().to_string();
    // A pipe is a pipe's colour, a door a door's, and glass is seen through.
    let pipe = parts.iter().find(|p| p.kind == "pipes").unwrap();
    assert_eq!(finish(pipe), "pipe");
    let glass = parts.iter().find(|p| p.kind == "windows").unwrap();
    assert!(colour(glass)[3] < 1.0 && b.read.json["materials"][glass.material]["alphaMode"] == "BLEND");
    assert!(parts.iter().filter(|p| p.kind == "doors").all(|p| colour(p)[3] == 1.0));
    // A colour is a factor in linear light between 0 and 1.
    for m in b.read.json["materials"].as_array().unwrap() {
        for c in m["pbrMetallicRoughness"]["baseColorFactor"].as_array().unwrap() {
            let c = c.as_f64().unwrap();
            assert!((0.0..=1.0).contains(&c));
        }
    }
    // A wall is coloured by the cladding the estimate prices on it, where one is (the made-up house prices none a pattern needs but the
    // finish it names is the material's name): the material's name is the finish said, or says nothing is priced.
    for w in parts.iter().filter(|p| p.kind == "walls") {
        let f = finish(w);
        assert!(f == "no finish priced" || f == "concrete" || f.contains("siding") || f.contains("brick") || f.contains("stone") || f.contains("stucco") || f.contains("shingles") || f.contains("block") || f.contains("panel") || f.contains("batten"), "{f}");
    }
    // Rooms and roof planes are not solid walls: rooms are see-through, a roof plane is seen from both sides.
    let room = parts.iter().find(|p| p.kind == "rooms").unwrap();
    assert!(colour(room)[3] < 0.5);
    assert_eq!(b.read.json["materials"][parts.iter().find(|p| p.kind == "roof planes").unwrap().material]["doubleSided"], true);
}

#[test]
fn a_run_with_no_size_is_its_line_and_a_fitting_with_no_size_is_said_to_be_left_out() {
    let b = both();
    let parts = b.read.parts();
    // The three hot-water runs with no size are lines (the BIM file writes only the line); the two fittings with no size have no body in
    // the BIM file and are named as not carried.
    assert_eq!(parts.iter().filter(|p| p.mode == 1).count(), 3);
    for line in parts.iter().filter(|p| p.mode == 1) {
        assert_eq!(line.points.len(), 2);
    }
    let said = b.glb.not_carried.join("\n");
    assert!(said.contains("hot to the kitchen sink, elbow 1: it has no shape"), "{said}");
    assert!(said.contains("hot to the kitchen sink, elbow 2: it has no shape"), "{said}");
}

#[test]
fn what_is_not_carried_is_said_and_in_the_file_too() {
    let b = both();
    let said = b.glb.not_carried.join("\n");
    for needed in ["prices", "Known, Assumed or Unknown", "patterns", "lap siding, brick, shingles", "drawn as in the window", "no thickness"] {
        assert!(said.contains(needed), "{needed} is not said: {said}");
    }
    // The file carries the same list, so whoever opens it is told.
    let in_file: Vec<String> = b.read.json["asset"]["extras"]["not carried"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    assert_eq!(in_file, b.glb.not_carried);
    // No software word in what a person reads.
    for line in &b.glb.not_carried {
        for word in ["IFC file", "identifier", "mesh", "vertex", "vertices", "triangle", "glTF"] {
            assert!(!line.contains(word), "{word} in: {line}");
        }
    }
}

#[test]
fn no_name_machine_or_path_is_written_into_the_file() {
    let b = both();
    let json = String::from_utf8_lossy(&b.glb.bytes[20..20 + u32_at(&b.glb.bytes, 12) as usize]).to_string();
    for bad in ["/tmp", "/home", "/root", "C:\\", "claude", "ipedrax", "scratchpad"] {
        assert!(!json.contains(bad), "{bad} is in the file");
    }
    // The job's own title is the building's name, as in the BIM file.
    assert_eq!(b.read.json["nodes"].as_array().unwrap().iter().filter(|n| n["name"] == b.title.as_str()).count(), 1);
    assert_eq!(b.read.json["asset"]["generator"], "BuildOS");
    assert!(b.ifc_text.contains("IFCBUILDING"));
}

#[test]
fn the_same_job_writes_the_same_file_each_time() {
    let mut job = sample::house_priced();
    let a = gltf::export(&mut job).unwrap();
    let mut other = sample::house_priced();
    let c = gltf::export(&mut other).unwrap();
    assert_eq!(a.bytes, c.bytes);
    // And the BIM file, written after the glTF, is the BIM file.
    let first = write::export(&mut job, "house.ifc", "2026-11-26T10:00:00").unwrap();
    let again = write::export(&mut other, "house.ifc", "2026-11-26T10:00:00").unwrap();
    assert_eq!(first.text, again.text);
}

#[test]
fn the_levels_stand_in_the_file_lowest_first_with_their_parts_under_them() {
    let b = both();
    let nodes = b.read.json["nodes"].as_array().unwrap();
    let root = b.read.json["scenes"][0]["nodes"][0].as_u64().unwrap() as usize;
    assert_eq!(nodes[root]["name"], b.title.as_str());
    let kids: Vec<&Value> = nodes[root]["children"].as_array().unwrap().iter().map(|c| &nodes[c.as_u64().unwrap() as usize]).collect();
    let levels: Vec<&str> = kids.iter().filter(|k| k["extras"]["kind"] == "levels").map(|k| k["name"].as_str().unwrap()).collect();
    assert_eq!(levels.len(), 3, "{levels:?}");
    let elevation: Vec<f64> = kids.iter().filter(|k| k["extras"]["kind"] == "levels").map(|k| k["extras"]["elevation_m"].as_f64().unwrap()).collect();
    assert!(elevation.windows(2).all(|w| w[0] < w[1]), "lowest first: {elevation:?}");
    assert!((elevation[2] - 3.048).abs() < 1e-9, "the second floor at 10 ft is 3.048 m");
    assert_eq!(kids.iter().filter(|k| k["extras"]["kind"] == "roofs").count(), 1);
}
