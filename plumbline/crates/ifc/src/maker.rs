//! A maker's model brought in as items (call 214, the bench's step E; contract 214M).
//!
//! A maker's IFC file has a product in it, or several: a basin, a range, a vanity. It has no building and no level, and nothing here sets
//! anything down: **each kind of product in the file becomes an item in the tool chest**, the way a maker's sheet does (`import spec`):
//!
//! * its sizes are the extent of its body in its own axes (across its x, front to back its y, up its z), each Assumed and saying the
//!   file; a body that cannot be read leaves the three Unknown and says which kind it was;
//! * its maker and model are read from the manufacturer's property set where the file has it (on the type, else on the thing),
//!   Assumed, saying the file and the property; where it has none they are Unknown, "not said in the model";
//! * each connection point the file puts on the product becomes a row of the item (call 211): what it takes and carries, which way it
//!   goes, its place as three lengths (across from the middle of the body, back from its back, up from its foot) and, where the
//!   file's direction lies along one of the six, which way it faces; each Assumed, saying the file. A point outside the body is not
//!   written, and is named;
//! * the item is drawn as a box of its sizes: the body's own shape is not kept.
//!
//! How the body is read follows BIMCraft's (`crates/core/src/import.rs`: `chosen`, `faces`, `face_set`, `looked`, `operator`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71): which items of a product are its body (the "Body" or "Facetation" representation, else any
//! but a line or a footprint), a pushed outline's prism, the faces of a solid or a shell, a face set's points, a shape shared through
//! its map and a transformation operator, a box, and a clipped solid as the solid it was cut from. It is written fresh over this
//! program's own reader and takes no code from BIMCraft; it reads only the extent of the body, which is all an item holds. What BIMCraft
//! reads and this does not (solids turned about an axis, swept along a curve, advanced B-reps: `import/shapes.rs`) needs a curved-surface
//! tessellator to find an extent, and a size found by sampling is a size nobody can vouch for: such a body leaves the sizes Unknown, with
//! the kind of body said.
//!
//! What the standard calls things is read from IfcOpenShell's own copy of the IFC4 and IFC2x3 schemas, run as an outside program and
//! never linked (PORTMAP.md, call 214).

use std::collections::{HashMap, HashSet};

use plumbline_world::building::connections::inches_of;
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::units::{self, Len};
use plumbline_world::value::{Status, Val};
use plumbline_world::{ConnectorKind, Domain, Facing, Flow, Item, Job, Kind, Node};

use super::step::{Entity, V};
use super::{cross, dot, length_unit, said as the_files_word, unit as unit_of, Frame, Reader, EQUIPMENT, FIXTURES, FURNISHINGS, P3, WORLD};

/// An item made from the file, and an account of the making.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MadeItem {
    /// Where the item is in the job.
    pub path: String,
    pub name: String,
    pub kind: Option<Kind>,
    /// Why it is that kind: what the file calls it, or "as you said".
    pub kind_why: String,
    /// What the file calls it, in words.
    pub class: String,
    /// How many of it the file has.
    pub count: usize,
    /// What was read: what it is, its value as it is written, where it is from. Each Assumed.
    pub read: Vec<(String, String, String)>,
    /// What was not in the file, or could not be read, and is Unknown: what, and why.
    pub unknown: Vec<(String, String)>,
    /// Each connection written, as the item holds it (a name, what it takes, its place in inches, which way it faces).
    pub connections: Vec<String>,
    /// Each connection point the file has that was not written, and why.
    pub connections_not_written: Vec<String>,
    /// Other things to know about it.
    pub notes: Vec<String>,
    /// Whether it has all three sizes: with them it is drawn, without them it is not.
    pub has_shape: bool,
}

/// What bringing a maker's model in did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MakerModel {
    pub schema: String,
    /// What the file's lengths are in, as the file says.
    pub unit: String,
    pub made: Vec<MadeItem>,
    /// Each item the file has that was not made, by its name, and why (the tool chest has an item of that name).
    pub refused: Vec<(String, String)>,
    /// What the file has that is not an item: by what it is called, how many, and why.
    pub left_out: Vec<(String, usize, String)>,
    /// What the file has of a building, said, and that none of it is brought in.
    pub building: Option<String>,
    pub notes: Vec<String>,
}

impl MakerModel {
    /// What was not made, a line each, as a person reads it.
    pub fn left_out_said(&self) -> Vec<String> {
        self.left_out
            .iter()
            .map(|(class, n, why)| {
                let plural = *n > 1 && !class.ends_with('s') && !class.contains('"') && !class.contains(" or ");
                format!("{n} {class}{}: {why}", if plural { "s" } else { "" })
            })
            .collect()
    }

    /// The whole reply, a line each. `file` is what the file is called; `job` is the job's file, for the command that enters a size.
    pub fn said(&self, file: &str, job: &str) -> Vec<String> {
        let mut out = Vec::new();
        let many = |n: usize, one: &str, more: &str| format!("{n} {}", if n == 1 { one } else { more });
        out.push(format!("{file}: a maker's model ({}, lengths in {}): {} made", self.schema, self.unit, many(self.made.len(), "item", "items")));
        for item in &self.made {
            let kind = item.kind.map(|k| k.a()).unwrap_or_default();
            out.push(format!("{}: {}, {kind} ({}), from {file}", item.path, item.name, item.kind_why));
            if !item.read.is_empty() {
                out.push("  read off the model, each Assumed until it is checked against the thing:".to_string());
                for (what, value, from) in &item.read {
                    out.push(format!("    ~ {what:<7} {value:<22} {from}"));
                }
            }
            if !item.unknown.is_empty() {
                out.push("  not in the model, or not read, and Unknown until entered:".to_string());
                for (what, why) in &item.unknown {
                    out.push(format!("    ? {what:<7} {why}"));
                }
            }
            if !item.has_shape {
                out.push(format!("  Without all three sizes it has no shape: set down, it is not drawn solid and not tested for clashes. Enter one with: plumbline measure {job} {} <width|depth|height> \"34 in\" --source \"<where from>\"", item.path));
            }
            if !item.connections.is_empty() {
                out.push(format!("  where it connects, {} (each Assumed, from the file):", many(item.connections.len(), "connection", "connections")));
                for c in &item.connections {
                    out.push(format!("    {c}"));
                }
            }
            for c in &item.connections_not_written {
                out.push(format!("  not written: {c}"));
            }
            for n in &item.notes {
                out.push(format!("  {n}"));
            }
        }
        for (name, why) in &self.refused {
            out.push(format!("  refused: {name}: {why}"));
        }
        for line in self.left_out_said() {
            out.push(format!("  left out: {line}"));
        }
        if let Some(b) = &self.building {
            out.push(format!("  {b}"));
        }
        for n in &self.notes {
            out.push(format!("  {n}"));
        }
        out
    }
}

fn round4(x: f64) -> f64 {
    (x * 1.0e4).round() / 1.0e4 + 0.0
}

fn ids(v: &V) -> Vec<u64> {
    Reader::list(v).iter().filter_map(|x| if let V::Ref(i) = x { Some(*i) } else { None }).collect()
}

/// A list of references, or one reference alone.
fn ids_or_one(v: &V) -> Vec<u64> {
    match v {
        V::Ref(i) => vec![*i],
        other => ids(other),
    }
}

/// What the file calls a class, in words a builder says.
fn class_word(class: &str) -> String {
    const WORDS: [(&str, &str); 17] = [
        ("IFCSANITARYTERMINAL", "plumbing fixture"),
        ("IFCFLOWTERMINAL", "fixture or outlet"),
        ("IFCOUTLET", "outlet"),
        ("IFCAIRTERMINAL", "grille or diffuser"),
        ("IFCLIGHTFIXTURE", "light fixture"),
        ("IFCSTACKTERMINAL", "stack terminal"),
        ("IFCWASTETERMINAL", "waste terminal"),
        ("IFCFURNITURE", "piece of furniture"),
        ("IFCFURNISHINGELEMENT", "furnishing"),
        ("IFCSYSTEMFURNITUREELEMENT", "system furniture"),
        ("IFCELECTRICAPPLIANCE", "electric appliance"),
        ("IFCUNITARYEQUIPMENT", "unit of equipment"),
        ("IFCBOILER", "boiler"),
        ("IFCPUMP", "pump"),
        ("IFCFAN", "fan"),
        ("IFCTANK", "tank"),
        ("IFCBUILDINGELEMENTPROXY", "thing of no stated kind"),
    ];
    match WORDS.iter().find(|(c, _)| *c == class) {
        Some((_, w)) => w.to_string(),
        None => the_files_word(class),
    }
}

/// What kind of item a class makes, and why.
fn kind_of(class: &str) -> Option<(Kind, String)> {
    let said = class_word(class);
    let a = if said.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    if FIXTURES.contains(&class) {
        Some((Kind::Fixture, format!("the file calls it {a} {said}")))
    } else if FURNISHINGS.contains(&class) {
        Some((Kind::Furnishing, format!("the file calls it {a} {said}")))
    } else if class == "IFCELECTRICAPPLIANCE" {
        Some((Kind::Equipment, "the file calls it an electric appliance, which is read as equipment".to_string()))
    } else if EQUIPMENT.contains(&class) {
        Some((Kind::Equipment, format!("the file calls it {a} {said}")))
    } else {
        None
    }
}

/// The body's extent, in the file's unit and in the thing's own axes: the least and the greatest of each coordinate its body reaches.
type Extent = ([f64; 3], [f64; 3]);

const DEPTH: usize = 16;
/// How many points a body may have before it is not read (a maker's body is thousands of faces, not tens of millions).
const MOST_POINTS: usize = 20_000_000;

struct Walk<'r, 'a> {
    r: &'r Reader<'a>,
    points: Vec<P3>,
    why: Option<String>,
}

impl<'r, 'a> Walk<'r, 'a> {
    fn stop(&mut self, why: impl Into<String>) {
        if self.why.is_none() {
            self.why = Some(why.into());
        }
    }

    fn add(&mut self, f: &Frame, p: P3) {
        if self.points.len() >= MOST_POINTS {
            self.stop("its body has more points than this reads");
            return;
        }
        self.points.push(f.place(p));
    }

    /// A transformation operator, as a frame with its scales in its axes.
    fn operator(&mut self, v: &V) -> Option<Frame> {
        let Some((_, e)) = self.r.get(v) else { return Some(WORLD) };
        let s = Reader::number(self.r.arg(e, 3)).unwrap_or(1.0);
        let k = match e.name.as_str() {
            "IFCCARTESIANTRANSFORMATIONOPERATOR3D" => [s, s, s],
            "IFCCARTESIANTRANSFORMATIONOPERATOR3DNONUNIFORM" => [s, Reader::number(self.r.arg(e, 5)).unwrap_or(s), Reader::number(self.r.arg(e, 6)).unwrap_or(s)],
            other => {
                self.stop(format!("its shape is placed by a {}, which this does not read", the_files_word(other)));
                return None;
            }
        };
        if k.iter().any(|x| !x.is_finite() || *x == 0.0) {
            self.stop("its shape is scaled by nothing");
            return None;
        }
        let o = self.r.triple(self.r.arg(e, 2)).unwrap_or([0.0; 3]);
        let z = self.r.triple(self.r.arg(e, 4)).and_then(unit_of).unwrap_or([0.0, 0.0, 1.0]);
        let toward = self.r.triple(self.r.arg(e, 0)).and_then(unit_of).unwrap_or(if z[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] });
        let along = dot(toward, z);
        let x = unit_of([toward[0] - z[0] * along, toward[1] - z[1] * along, toward[2] - z[2] * along]).unwrap_or([1.0, 0.0, 0.0]);
        let y = cross(z, x);
        let by = |v: P3, k: f64| [v[0] * k, v[1] * k, v[2] * k];
        Some(Frame { x: by(x, k[0]), y: by(y, k[1]), z: by(z, k[2]), o })
    }

    /// A profile's outline in the plane of the solid it is pushed from, with the profile's own placement applied.
    fn outline(&mut self, area: &Entity) -> Option<Vec<[f64; 2]>> {
        let n = |s: &Self, i: usize| Reader::number(s.r.arg(area, i));
        let rect = |w: f64, h: f64| vec![[-w / 2.0, -h / 2.0], [w / 2.0, -h / 2.0], [w / 2.0, h / 2.0], [-w / 2.0, h / 2.0]];
        // Round and oval outlines in 48 points, 7.5 degrees apart: they reach the axes and the diagonals exactly.
        let oval = |a: f64, b: f64| (0..48).map(|i| f64::from(i) * std::f64::consts::PI / 24.0).map(|t| [a * t.cos(), b * t.sin()]).collect::<Vec<_>>();
        let local = match area.name.as_str() {
            "IFCRECTANGLEPROFILEDEF" | "IFCROUNDEDRECTANGLEPROFILEDEF" | "IFCRECTANGLEHOLLOWPROFILEDEF" | "IFCISHAPEPROFILEDEF" | "IFCASYMMETRICISHAPEPROFILEDEF" => match (n(self, 3), n(self, 4)) {
                (Some(w), Some(h)) => rect(w, h),
                _ => {
                    self.stop("its profile has no sizes");
                    return None;
                }
            },
            "IFCLSHAPEPROFILEDEF" | "IFCUSHAPEPROFILEDEF" | "IFCCSHAPEPROFILEDEF" | "IFCTSHAPEPROFILEDEF" | "IFCZSHAPEPROFILEDEF" => match (n(self, 4), n(self, 3)) {
                (Some(w), Some(h)) => rect(w, h),
                _ => {
                    self.stop("its profile has no sizes");
                    return None;
                }
            },
            "IFCCIRCLEPROFILEDEF" | "IFCCIRCLEHOLLOWPROFILEDEF" => match n(self, 3) {
                Some(r) => oval(r, r),
                None => {
                    self.stop("its profile has no radius");
                    return None;
                }
            },
            "IFCELLIPSEPROFILEDEF" => match (n(self, 3), n(self, 4)) {
                (Some(a), Some(b)) => oval(a, b),
                _ => {
                    self.stop("its profile has no sizes");
                    return None;
                }
            },
            "IFCARBITRARYCLOSEDPROFILEDEF" | "IFCARBITRARYPROFILEDEFWITHVOIDS" => {
                let Some((_, curve)) = self.r.get(self.r.arg(area, 2)) else {
                    self.stop("its profile has no outline");
                    return None;
                };
                if curve.name != "IFCPOLYLINE" {
                    self.stop(format!("its outline is drawn as a {}, which this does not read; only a polyline is", the_files_word(&curve.name)));
                    return None;
                }
                let pts: Vec<[f64; 2]> = Reader::list(self.r.arg(curve, 0)).iter().filter_map(|p| self.r.triple(p)).map(|p| [p[0], p[1]]).collect();
                if pts.len() < 3 {
                    self.stop("its outline is not three or more corners");
                    return None;
                }
                pts
            }
            other => {
                self.stop(format!("its body is pushed from a {}, which this does not read", the_files_word(other)));
                return None;
            }
        };
        let place = self.r.axes(self.r.arg(area, 2));
        Some(
            local
                .into_iter()
                .map(|q| {
                    let w = place.place([q[0], q[1], 0.0]);
                    [w[0], w[1]]
                })
                .collect(),
        )
    }

    /// One item of a body, seen from `f`, the frame it is in.
    fn item(&mut self, id: u64, f: &Frame, depth: usize) {
        if self.why.is_some() {
            return;
        }
        let Some(e) = self.r.file.entities.get(&id) else {
            self.stop(format!("its body names a part (#{id}) that is not in the file"));
            return;
        };
        if depth > DEPTH {
            self.stop("its body's shapes are nested more than 16 deep");
            return;
        }
        let a = |i: usize| self.r.arg(e, i);
        match e.name.as_str() {
            "IFCEXTRUDEDAREASOLID" => {
                let Some((_, area)) = self.r.get(a(0)) else {
                    self.stop("its body is pushed from a profile that is not in the file");
                    return;
                };
                let Some(ring) = self.outline(area) else { return };
                let Some(length) = Reader::number(a(3)).filter(|l| l.is_finite()) else {
                    self.stop("its body is pushed no distance");
                    return;
                };
                let way = self.r.triple(a(2)).and_then(unit_of).unwrap_or([0.0, 0.0, 1.0]);
                let g = f.then(&self.r.axes(a(1)));
                for q in ring {
                    for t in [0.0, length] {
                        self.add(&g, [q[0] + way[0] * t, q[1] + way[1] * t, way[2] * t]);
                    }
                }
            }
            "IFCFACETEDBREP" | "IFCFACETEDBREPWITHVOIDS" | "IFCCSGSOLID" => {
                if let V::Ref(inner) = a(0) {
                    self.item(*inner, f, depth + 1);
                }
            }
            "IFCCLOSEDSHELL" | "IFCOPENSHELL" | "IFCCONNECTEDFACESET" | "IFCSHELLBASEDSURFACEMODEL" | "IFCFACEBASEDSURFACEMODEL" => {
                for part in ids(a(0)) {
                    self.item(part, f, depth + 1);
                }
            }
            "IFCFACE" | "IFCFACESURFACE" => {
                for bound in ids(a(0)) {
                    let Some(b) = self.r.file.entities.get(&bound) else { continue };
                    let Some((_, lp)) = self.r.get(self.r.arg(b, 0)) else { continue };
                    if lp.name != "IFCPOLYLOOP" {
                        self.stop(format!("a face of its body is bounded by a {}, which this does not read", the_files_word(&lp.name)));
                        return;
                    }
                    for p in Reader::list(self.r.arg(lp, 0)) {
                        if let Some(p) = self.r.triple(p) {
                            self.add(f, p);
                        }
                    }
                }
            }
            "IFCTRIANGULATEDFACESET" | "IFCPOLYGONALFACESET" => {
                let triangles = e.name == "IFCTRIANGULATEDFACESET";
                let Some((_, list)) = self.r.get(a(0)) else {
                    self.stop("its face set has no points");
                    return;
                };
                let coords: Vec<Option<P3>> = Reader::list(self.r.arg(list, 0))
                    .iter()
                    .map(|c| {
                        let n: Vec<f64> = Reader::list(c).iter().filter_map(Reader::number).collect();
                        (n.len() == 3).then(|| [n[0], n[1], n[2]])
                    })
                    .collect();
                let pn: Vec<usize> = Reader::list(a(if triangles { 4 } else { 3 })).iter().filter_map(Reader::number).map(|x| x as usize).collect();
                let used: Vec<usize> = if triangles {
                    Reader::list(a(3)).iter().flat_map(|t| Reader::list(t).iter().filter_map(Reader::number).map(|x| x as usize).collect::<Vec<_>>()).collect()
                } else {
                    ids(a(2)).into_iter().filter_map(|f| self.r.file.entities.get(&f)).flat_map(|face| Reader::list(self.r.arg(face, 0)).iter().filter_map(Reader::number).map(|x| x as usize).collect::<Vec<_>>()).collect()
                };
                for i in used {
                    let i = if pn.is_empty() { Some(i) } else { i.checked_sub(1).and_then(|k| pn.get(k)).copied() };
                    match i.and_then(|i| i.checked_sub(1)).and_then(|k| coords.get(k)).copied().flatten() {
                        Some(p) => self.add(f, p),
                        None => {
                            self.stop("a face of its body names a point that is not in its list");
                            return;
                        }
                    }
                }
            }
            "IFCMAPPEDITEM" => {
                let Some((_, map)) = self.r.get(a(0)) else {
                    self.stop("its shape is a map to a shape that is not in the file");
                    return;
                };
                if !self.at_the_origin(map) {
                    self.stop("its shape's own origin is not the origin of the shape, which this does not read");
                    return;
                }
                let Some(g) = self.operator(a(1)) else { return };
                let g = f.then(&g);
                let Some((_, rep)) = self.r.get(self.r.arg(map, 1)) else {
                    self.stop("its shape is a map to a shape that is not in the file");
                    return;
                };
                for item in ids(self.r.arg(rep, 3)) {
                    self.item(item, &g, depth + 1);
                }
            }
            "IFCBOUNDINGBOX" => {
                let corner = self.r.triple(a(0)).unwrap_or([0.0; 3]);
                let (Some(x), Some(y), Some(z)) = (Reader::number(a(1)), Reader::number(a(2)), Reader::number(a(3))) else {
                    self.stop("its bounding box has no sizes");
                    return;
                };
                for (i, j, k) in (0..8).map(|n| (n & 1, (n >> 1) & 1, (n >> 2) & 1)) {
                    self.add(f, [corner[0] + x * f64::from(i), corner[1] + y * f64::from(j), corner[2] + z * f64::from(k)]);
                }
            }
            "IFCBLOCK" => {
                let pos = f.then(&self.r.axes(a(0)));
                let (Some(x), Some(y), Some(z)) = (Reader::number(a(1)), Reader::number(a(2)), Reader::number(a(3))) else {
                    self.stop("its block has no sizes");
                    return;
                };
                for (i, j, k) in (0..8).map(|n| (n & 1, (n >> 1) & 1, (n >> 2) & 1)) {
                    self.add(&pos, [x * f64::from(i), y * f64::from(j), z * f64::from(k)]);
                }
            }
            // What a clipping cuts away does not change what the element is; a union is both.
            "IFCBOOLEANCLIPPINGRESULT" | "IFCBOOLEANRESULT" => {
                if let V::Ref(first) = a(1) {
                    self.item(*first, f, depth + 1);
                }
                if matches!(a(0), V::Enum(op) if op == "UNION") {
                    if let V::Ref(second) = a(2) {
                        self.item(*second, f, depth + 1);
                    }
                }
            }
            "IFCREVOLVEDAREASOLID" | "IFCREVOLVEDAREASOLIDTAPERED" => self.stop("its body is a solid turned about an axis, which this does not read"),
            "IFCSWEPTDISKSOLID" | "IFCSWEPTDISKSOLIDPOLYGONAL" | "IFCFIXEDREFERENCESWEPTAREASOLID" | "IFCSURFACECURVESWEPTAREASOLID" | "IFCEXTRUDEDAREASOLIDTAPERED" => self.stop("its body is swept along a curve or tapered, which this does not read"),
            "IFCADVANCEDBREP" | "IFCADVANCEDBREPWITHVOIDS" | "IFCADVANCEDFACE" => self.stop("its body is an advanced B-rep, with curved surfaces, which this does not read"),
            other => self.stop(format!("its body is made of a {}, which this does not read", the_files_word(other))),
        }
    }

    /// Whether a shape's own origin is the origin of its axes.
    fn at_the_origin(&self, map: &Entity) -> bool {
        let f = self.r.axes(self.r.arg(map, 0));
        let near = |a: P3, b: P3| (0..3).all(|i| (a[i] - b[i]).abs() < 1.0e-9);
        near(f.o, [0.0; 3]) && near(f.x, [1.0, 0.0, 0.0]) && near(f.z, [0.0, 0.0, 1.0])
    }
}

/// The items of a product's shapes that are its body: its "Body" or "Facetation" representation's, else any but a line or a footprint's.
fn body_items(r: &Reader, shape: &V) -> Vec<u64> {
    let Some((_, pds)) = r.get(shape) else { return Vec::new() };
    let reps: Vec<(String, Vec<u64>)> = ids(r.arg(pds, 2)).into_iter().filter_map(|id| r.file.entities.get(&id)).map(|rep| (Reader::text(r.arg(rep, 1)).unwrap_or("").to_ascii_lowercase(), ids(r.arg(rep, 3)))).collect();
    let pick = |keep: &dyn Fn(&str) -> bool| reps.iter().filter(|(i, _)| keep(i)).flat_map(|(_, it)| it.clone()).collect::<Vec<u64>>();
    let chosen = pick(&|i| i == "body" || i == "facetation");
    if chosen.is_empty() {
        pick(&|i| !matches!(i, "axis" | "footprint" | "annotation" | "plan" | "profile" | "reference" | "surface"))
    } else {
        chosen
    }
}

/// A product's body: the extent of it in the product's own axes, in the file's unit; or why it cannot be read. The product's own shapes, else
/// the first shape its type carries.
fn body_of(r: &Reader, product: &Entity, its_type: Option<&Entity>) -> Result<Extent, String> {
    let mut walk = Walk { r, points: Vec::new(), why: None };
    let items = body_items(r, r.arg(product, 6));
    if !items.is_empty() {
        for id in items {
            walk.item(id, &WORLD, 0);
        }
    } else if r.get(r.arg(product, 6)).is_some() {
        return Err("it has no body in the file, only a line or a drawing".to_string());
    } else if let Some(map) = its_type.and_then(|t| ids(r.arg(t, 6)).into_iter().next()).and_then(|m| r.file.entities.get(&m)) {
        if !walk.at_the_origin(map) {
            return Err("its shape's own origin is not the origin of the shape, which this does not read".to_string());
        }
        if let Some((_, rep)) = r.get(r.arg(map, 1)) {
            for id in ids(r.arg(rep, 3)) {
                walk.item(id, &WORLD, 0);
            }
        }
    } else {
        return Err("it has no body in the file".to_string());
    }
    if let Some(why) = walk.why {
        return Err(why);
    }
    if walk.points.is_empty() {
        return Err("its body has no points in it".to_string());
    }
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for p in &walk.points {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    if (0..3).any(|i| !lo[i].is_finite() || !hi[i].is_finite()) {
        return Err("its body has a point that is no figure".to_string());
    }
    Ok((lo, hi))
}

/// The maker's set: what it says of the maker and the model, each with the set's number in the file.
#[derive(Default)]
struct Said {
    maker: Option<(String, u64)>,
    label: Option<(String, u64)>,
    reference: Option<(String, u64)>,
}

fn manufacturer(r: &Reader, sets: &[u64]) -> Said {
    let mut out = Said::default();
    for id in sets {
        let Some(set) = r.file.entities.get(id) else { continue };
        if set.name != "IFCPROPERTYSET" || !Reader::text(r.arg(set, 2)).is_some_and(|n| n.eq_ignore_ascii_case("Pset_ManufacturerTypeInformation")) {
            continue;
        }
        for prop in ids(r.arg(set, 4)) {
            let Some(p) = r.file.entities.get(&prop) else { continue };
            if p.name != "IFCPROPERTYSINGLEVALUE" {
                continue;
            }
            let Some(value) = Reader::text(r.arg(p, 2)).map(str::trim).filter(|v| !v.is_empty()) else { continue };
            let slot = match Reader::text(r.arg(p, 0)) {
                Some("Manufacturer") => &mut out.maker,
                Some("ModelLabel") => &mut out.label,
                Some("ModelReference") => &mut out.reference,
                _ => continue,
            };
            slot.get_or_insert((value.to_string(), *id));
        }
    }
    out
}

/// What a connection point's placement says of which way it faces.
pub(super) enum Way {
    /// The placement gives no axis: the standard's default is not a statement.
    NoAxis,
    /// It gives an axis that lies along none of the six ways a connection faces.
    Odd,
    Is(Facing),
}

/// Which way a connection point faces in the thing's own axes (`thing` is the thing's frame in the file): the placement's own axis, where the
/// file gives one.
pub(super) fn way_of(r: &Reader, port: &Entity, thing: &Frame) -> Way {
    let given = r.get(r.arg(port, 5)).and_then(|(_, pl)| r.get(r.arg(pl, 1))).is_some_and(|(_, ax)| ax.name == "IFCAXIS2PLACEMENT3D" && matches!(r.arg(ax, 1), V::Ref(_)));
    if !given {
        return Way::NoAxis;
    }
    let z = r.placed(r.arg(port, 5)).z;
    let way = [dot(z, thing.x), dot(z, thing.y), dot(z, thing.z)];
    [(Facing::Back, 1, 1.0), (Facing::Front, 1, -1.0), (Facing::Right, 0, 1.0), (Facing::Left, 0, -1.0), (Facing::Up, 2, 1.0), (Facing::Down, 2, -1.0)]
        .into_iter()
        .find(|(_, axis, sign)| way[*axis] * sign >= 0.9999)
        .map(|(f, ..)| Way::Is(f))
        .unwrap_or(Way::Odd)
}

/// How far, in inches, and which way, a point is outside a span: `lo` to `hi`.
fn beyond(x: f64, lo: f64, hi: f64) -> Option<(f64, bool)> {
    const TOLERANCE_IN: f64 = 0.001;
    if x < lo - TOLERANCE_IN {
        Some((lo - x, true))
    } else if x > hi + TOLERANCE_IN {
        Some((x - hi, false))
    } else {
        None
    }
}

/// A number of inches as a figure a row is written with: up to five decimals, none trailing, `0` alone for nothing.
fn figure(x: f64) -> String {
    let x = ((x * 1.0e5).round() / 1.0e5) + 0.0;
    if x == 0.0 {
        return "0".to_string();
    }
    format!("{x:.5}").trim_end_matches('0').trim_end_matches('.').to_string() + " in"
}

/// What a connection point says it carries, by the system the file puts it on (IfcDistributionSystemEnum, as `import ifc` reads it).
fn carries(system: &V) -> Option<Domain> {
    match system {
        V::Enum(w) => match w.as_str() {
            "DOMESTICCOLDWATER" | "WATERSUPPLY" => Some(Domain::Cold),
            "DOMESTICHOTWATER" => Some(Domain::Hot),
            "SEWAGE" | "DRAINAGE" | "WASTEWATER" => Some(Domain::Waste),
            "VENT" => Some(Domain::Vent),
            "STORMWATER" | "RAINWATER" => Some(Domain::Storm),
            "GAS" => Some(Domain::Gas),
            _ => None,
        },
        _ => None,
    }
}

/// The classes a building is made of, by the word for each.
const BUILDING: [(&str, &str, &str); 8] = [
    ("IFCBUILDINGSTOREY", "level", "levels"),
    ("IFCWALL", "wall", "walls"),
    ("IFCWALLSTANDARDCASE", "wall", "walls"),
    ("IFCSLAB", "floor slab", "floor slabs"),
    ("IFCROOF", "roof", "roofs"),
    ("IFCDOOR", "door", "doors"),
    ("IFCWINDOW", "window", "windows"),
    ("IFCSPACE", "room", "rooms"),
];

/// Bring a maker's model in as items. `name` is what the file is called, for each value's source. `proxies_as` says what a thing of no
/// stated kind (a building element proxy) is: without it such things are left out and named, and with it every one in the file is that.
///
/// Nothing is set down, and no level or building is needed. An item the tool chest already has by that name is refused for that item and
/// the rest come in. If the job would not check afterwards, it is as it was.
pub fn maker_model(job: &mut Job, bytes: &[u8], name: &str, proxies_as: Option<Kind>) -> Result<MakerModel, String> {
    let file = super::step::parse(bytes).map_err(|why| format!("it is not an IFC file this can read ({why})"))?;
    if !file.schema.starts_with("IFC") {
        return Err(format!("it is a file written the way a model is, but of another kind (\"{}\"): not an IFC model", file.schema));
    }
    let r = Reader { file: &file };
    let (unit_name, k) = length_unit(&r);
    // Inches to one of the file's unit.
    let inch = k * 12.0;
    let mut ids_sorted: Vec<u64> = file.entities.keys().copied().collect();
    ids_sorted.sort_unstable();
    let of = |class: &[&str]| -> Vec<(u64, &Entity)> { ids_sorted.iter().filter_map(|id| file.entities.get(id).map(|e| (*id, e))).filter(|(_, e)| class.contains(&e.name.as_str())).collect() };

    // The type of each thing, and the sets each carries.
    let mut type_of: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELDEFINESBYTYPE"]) {
        if let V::Ref(t) = r.arg(rel, 5) {
            for o in ids(r.arg(rel, 4)) {
                type_of.entry(o).or_insert(*t);
            }
        }
    }
    let mut sets_of: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, rel) in of(&["IFCRELDEFINESBYPROPERTIES"]) {
        let sets = ids_or_one(r.arg(rel, 5));
        for o in ids(r.arg(rel, 4)) {
            sets_of.entry(o).or_default().extend(sets.iter().copied());
        }
    }
    // The connection points of each thing: nested in it (IFC4), or tied to it (both schemas).
    let port_ids: HashSet<u64> = of(&["IFCDISTRIBUTIONPORT"]).into_iter().map(|(id, _)| id).collect();
    let mut ports_of: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, rel) in of(&["IFCRELNESTS"]) {
        if let V::Ref(whole) = r.arg(rel, 4) {
            for p in ids(r.arg(rel, 5)) {
                if port_ids.contains(&p) {
                    ports_of.entry(*whole).or_default().push(p);
                }
            }
        }
    }
    for (_, rel) in of(&["IFCRELCONNECTSPORTTOELEMENT"]) {
        if let (V::Ref(p), V::Ref(whole)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            if port_ids.contains(p) && !ports_of.get(whole).is_some_and(|l| l.contains(p)) {
                ports_of.entry(*whole).or_default().push(*p);
            }
        }
    }

    let mut out = MakerModel { schema: file.schema.clone(), unit: unit_name, ..MakerModel::default() };

    // ---- What the file has: the products that make items, grouped by type (else by name); those that do not; the building.
    struct Group {
        first: u64,
        members: Vec<u64>,
        kind: Kind,
        why: String,
    }
    let mut groups: Vec<Group> = Vec::new();
    let mut at: HashMap<String, usize> = HashMap::new();
    let mut proxies = 0usize;
    let mut left: Vec<(String, usize, String)> = Vec::new();
    let mut building: Vec<(&str, usize)> = Vec::new();
    let mut leave = |class: &str, why: &str| match left.iter_mut().find(|(c, _, w)| c == class && w == why) {
        Some(row) => row.1 += 1,
        None => left.push((class.to_string(), 1, why.to_string())),
    };
    for (id, e) in ids_sorted.iter().filter_map(|id| file.entities.get(id).map(|e| (*id, e))) {
        let class = e.name.as_str();
        let (kind, why) = match kind_of(class) {
            Some(found) => found,
            None if class == "IFCBUILDINGELEMENTPROXY" => match proxies_as {
                Some(kind) => (kind, "as you said".to_string()),
                None => {
                    proxies += 1;
                    continue;
                }
            },
            None => {
                if let Some((_, one, more)) = BUILDING.iter().find(|(c, ..)| *c == class) {
                    let word = if building.iter().any(|(w, _)| w == one) { *one } else { *one };
                    let _ = more;
                    match building.iter_mut().find(|(w, _)| *w == word) {
                        Some(row) => row.1 += 1,
                        None => building.push((word, 1)),
                    }
                } else if matches!(r.get(r.arg(e, 6)), Some((_, s)) if s.name == "IFCPRODUCTDEFINITIONSHAPE") && !matches!(class, "IFCOPENINGELEMENT" | "IFCDISTRIBUTIONPORT" | "IFCANNOTATION" | "IFCGRID") {
                    leave(&the_files_word(class), "a maker's model makes items of fixtures, equipment and furnishings only");
                }
                continue;
            }
        };
        let called = r.name(e);
        let key = match type_of.get(&id) {
            Some(t) => format!("type {t}"),
            None if !called.is_empty() => format!("name {}", plumbline_world::check::plain_name(&called)),
            None => format!("thing {id}"),
        };
        match at.get(&key) {
            Some(g) => groups[*g].members.push(id),
            None => {
                at.insert(key, groups.len());
                groups.push(Group { first: id, members: vec![id], kind, why });
            }
        }
    }
    if proxies > 0 {
        out.left_out.push((class_word("IFCBUILDINGELEMENTPROXY"), proxies, "it says nothing of what it is: say what such things are with --as fixture, --as equipment or --as furnishing, and every one in the file is that".to_string()));
    }
    out.left_out.extend(left);
    if !building.is_empty() {
        let each: Vec<String> = building
            .iter()
            .map(|(w, n)| {
                let more = BUILDING.iter().find(|(_, one, _)| one == w).map(|(_, _, more)| *more).unwrap_or(w);
                format!("{n} {}", if *n == 1 { w } else { more })
            })
            .collect();
        out.building = Some(format!("the file has a building in it ({}): it is not brought in, and nothing is set down; bring it in as a designer's model", each.join(", ")));
    }

    let done = ed::all_or_nothing(job, |job| {
        let mut taken: HashSet<String> = {
            let mut q = job.world.query::<(&Node, &Item)>();
            q.iter(&job.world).map(|(n, _)| plumbline_world::check::plain_name(&n.name)).collect()
        };
        for g in &groups {
            let Some(product) = file.entities.get(&g.first) else { continue };
            let its_type = type_of.get(&g.first).and_then(|t| file.entities.get(t));
            let called = r.name(product);
            let item_name = its_type.map(|t| r.name(t)).filter(|n| !n.is_empty()).or_else(|| Some(called.clone()).filter(|n| !n.is_empty())).unwrap_or_else(|| format!("{} {}", class_word(&product.name), g.first));
            if !taken.insert(plumbline_world::check::plain_name(&item_name)) {
                out.refused.push((item_name.clone(), "the tool chest has an item of that name already; it is left as it is".to_string()));
                continue;
            }
            let word = class_word(&product.name);
            let from = |what: &str| format!("{name}, #{} {word} \"{}\": {what}", g.first, if called.is_empty() { item_name.as_str() } else { called.as_str() });
            let mut made = MadeItem { name: item_name.clone(), kind: Some(g.kind), kind_why: g.why.clone(), class: word.clone(), count: g.members.len(), ..MadeItem::default() };

            // The body.
            let body = body_of(&r, product, its_type);
            let (width, depth, height) = match &body {
                Ok((lo, hi)) => {
                    let mut one = |what: &str, how: &str, i: usize| {
                        let v = round4((hi[i] - lo[i]) * inch);
                        let source = from(&format!("the extent of its body, {how}"));
                        made.read.push((what.to_string(), units::show_length(&Len::One(v), "in"), source.clone()));
                        Val { value: Some(Len::One(v)), unit: "in".to_string(), status: Status::Assumed, source }
                    };
                    (one("width", "across", 0), one("depth", "front to back", 1), one("height", "up", 2))
                }
                Err(why) => {
                    let unknown = |what: &str, made: &mut MadeItem| {
                        made.unknown.push((what.to_string(), why.clone()));
                        Val::unknown("in", why)
                    };
                    (unknown("width", &mut made), unknown("depth", &mut made), unknown("height", &mut made))
                }
            };
            if let Err(why) = &body {
                made.notes.push(format!("its body was not read: {why}"));
            }
            // A body that comes to no length at all in a direction is flat or a line: it has no such size.
            let flat = [("width", &width), ("depth", &depth), ("height", &height)].iter().find(|(_, v)| v.value.as_ref().is_some_and(|l| matches!(l, Len::One(x) if *x <= 0.0))).map(|(w, _)| *w);
            let (width, depth, height) = if let Some(w) = flat {
                let why = format!("its body is flat in one direction (its {w} comes to nothing)");
                made.read.clear();
                made.notes.push(why.clone());
                for what in ["width", "depth", "height"] {
                    made.unknown.push((what.to_string(), why.clone()));
                }
                (Val::unknown("in", &why), Val::unknown("in", &why), Val::unknown("in", &why))
            } else {
                (width, depth, height)
            };
            made.has_shape = width.value.is_some() && depth.value.is_some() && height.value.is_some();

            // The maker and the model, from the manufacturer's set on the type, else on the thing.
            let on_type: Vec<u64> = its_type.map(|t| ids(r.arg(t, 5))).unwrap_or_default();
            let on_thing: Vec<u64> = sets_of.get(&g.first).cloned().unwrap_or_default();
            let (said_type, said_thing) = (manufacturer(&r, &on_type), manufacturer(&r, &on_thing));
            let pick = |from_type: Option<(String, u64)>, from_thing: Option<(String, u64)>, property: &str| -> Option<(String, String)> {
                from_type.map(|(v, id)| (v, format!("{name}, #{id}: Pset_ManufacturerTypeInformation, {property} (on the type)"))).or_else(|| from_thing.map(|(v, id)| (v, format!("{name}, #{id}: Pset_ManufacturerTypeInformation, {property} (on the thing)"))))
            };
            let maker = pick(said_type.maker.clone(), said_thing.maker.clone(), "Manufacturer");
            // The model: the number the maker gives it, else its name.
            let model = pick(said_type.reference.clone(), said_thing.reference.clone(), "ModelReference").or_else(|| pick(said_type.label.clone(), said_thing.label.clone(), "ModelLabel"));
            let text = |what: &str, found: Option<(String, String)>, made: &mut MadeItem| match found {
                Some((v, source)) => {
                    made.read.push((what.to_string(), v.clone(), source.clone()));
                    Val { value: Some(v), unit: String::new(), status: Status::Assumed, source }
                }
                None => {
                    made.unknown.push((what.to_string(), "not said in the model".to_string()));
                    Val::unknown("", "not said in the model")
                }
            };
            let maker = text("maker", maker, &mut made);
            let model = text("model", model, &mut made);
            // Which things the type's shape is shared by, and what differs among them.
            if g.members.len() > 1 {
                let sizes: Vec<Option<Extent>> = g.members.iter().filter_map(|id| file.entities.get(id)).map(|p| body_of(&r, p, its_type).ok()).collect();
                let first = sizes.first().copied().flatten();
                if let Some((lo, hi)) = first {
                    let ext = |e: Extent| [round4((e.1[0] - e.0[0]) * inch), round4((e.1[1] - e.0[1]) * inch), round4((e.1[2] - e.0[2]) * inch)];
                    if sizes.iter().any(|s| s.map(|s| ext(s).iter().zip(ext((lo, hi))).any(|(a, b)| (a - b).abs() > 0.0625)).unwrap_or(true)) {
                        made.notes.push("the things of that type do not all come to the same sizes: the first one's are used".to_string());
                    }
                }
            }

            let weight = None;
            let item = Item { category: g.kind, maker, model, width, depth, height, weight, sheet: name.to_string(), division: None, form: Some("box".to_string()), priced_as: None };
            let path = match ed::item(job, &item_name, item) {
                Ok(p) => p,
                Err(why) => {
                    out.refused.push((item_name.clone(), why));
                    continue;
                }
            };
            made.path = path.clone();

            // Where it connects.
            let ports = ports_of.get(&g.first).cloned().unwrap_or_default();
            let mut used: Vec<String> = Vec::new();
            let (mut no_way, mut odd_way) = (0usize, 0usize);
            if let (Ok((lo, hi)), true) = (&body, made.has_shape) {
                let fp = r.placed(r.arg(product, 5));
                let size_in = [round4((hi[0] - lo[0]) * inch), round4((hi[1] - lo[1]) * inch), round4((hi[2] - lo[2]) * inch)];
                for (n, pid) in ports.iter().enumerate() {
                    let Some(port) = file.entities.get(pid) else { continue };
                    let pname = {
                        let t = r.name(port);
                        if t.is_empty() { format!("Connection {}", n + 1) } else { t }
                    };
                    let mut said_name = pname.clone();
                    let mut again = 2;
                    while used.iter().any(|u| u.eq_ignore_ascii_case(&said_name)) {
                        said_name = format!("{pname} {again}");
                        again += 1;
                    }
                    if r.get(r.arg(port, 5)).is_none() {
                        made.connections_not_written.push(format!("{pname}: it has no place in the file"));
                        continue;
                    }
                    let takes = match r.arg(port, 8) {
                        V::Enum(w) => match w.as_str() {
                            "DUCT" => Some(ConnectorKind::Duct),
                            "PIPE" => Some(ConnectorKind::Pipe),
                            "CABLECARRIER" => Some(ConnectorKind::Conduit),
                            "CABLE" => Some(ConnectorKind::Power),
                            _ => None,
                        },
                        _ => None,
                    };
                    let Some(takes) = takes else {
                        made.connections_not_written.push(format!("{pname}: the file says no sort for it (a pipe, a duct, a conduit or a cable), so it is not written"));
                        continue;
                    };
                    let flow = match r.arg(port, 7) {
                        V::Enum(w) if w == "SOURCE" => Flow::Out,
                        V::Enum(w) if w == "SINK" => Flow::In,
                        _ => Flow::Both,
                    };
                    let carried = if takes == ConnectorKind::Pipe { carries(r.arg(port, 9)) } else { None };
                    // Where it is and which way it points, in the thing's own axes.
                    let world = r.placed(r.arg(port, 5));
                    let d = [world.o[0] - fp.o[0], world.o[1] - fp.o[1], world.o[2] - fp.o[2]];
                    let local = [dot(d, fp.x), dot(d, fp.y), dot(d, fp.z)];
                    let way = [dot(world.z, fp.x), dot(world.z, fp.y), dot(world.z, fp.z)];
                    let (across, back, up) = ((local[0] - (lo[0] + hi[0]) / 2.0) * inch, (hi[1] - local[1]) * inch, (local[2] - lo[2]) * inch);
                    let outside = [
                        beyond(across, -size_in[0] / 2.0, size_in[0] / 2.0).map(|(far, low)| (far, if low { "to the left of" } else { "to the right of" })),
                        beyond(back, 0.0, size_in[1]).map(|(far, low)| (far, if low { "behind" } else { "in front of" })),
                        beyond(up, 0.0, size_in[2]).map(|(far, low)| (far, if low { "below" } else { "above" })),
                    ];
                    if let Some((far, how)) = outside.iter().flatten().copied().max_by(|a, b| a.0.total_cmp(&b.0)) {
                        made.connections_not_written.push(format!("{pname}: it stands outside the body, {} {how} it, so it is not written (a connection is on the body or inside it)", inches_of(far / 12.0)));
                        continue;
                    }
                    let faces = match way_of(&r, port, &fp) {
                        Way::Is(f) => Some(f),
                        Way::NoAxis => {
                            no_way += 1;
                            None
                        }
                        Way::Odd => {
                            odd_way += 1;
                            None
                        }
                    };
                    let clamp = |x: f64, lo: f64, hi: f64| round4(x).clamp(lo, hi);
                    let figures = [
                        figure(clamp(across, -size_in[0] / 2.0, size_in[0] / 2.0)),
                        figure(clamp(back, 0.0, size_in[1])),
                        figure(clamp(up, 0.0, size_in[2])),
                    ];
                    let source = format!("{name}, #{pid}: connection point \"{pname}\" on {word} \"{}\", where it is in the thing's own axes and the way its own axis points", if called.is_empty() { item_name.as_str() } else { called.as_str() });
                    let row = ed::connection_row_facing(&said_name, takes, carried, flow, &figures[0], &figures[1], &figures[2], faces, None, Entered { status: Status::Assumed, source: &source });
                    match row.and_then(|row| ed::connection_add(job, &path, row)) {
                        Ok(_) => used.push(said_name),
                        Err(why) => made.connections_not_written.push(format!("{pname}: {why}")),
                    }
                }
                if no_way + odd_way > 0 {
                    let n = no_way + odd_way;
                    let why = match (no_way, odd_way) {
                        (_, 0) => "their placement gives no axis".to_string(),
                        (0, _) => "their axis lies along none of the six ways a connection faces (back, front, left, right, up, down)".to_string(),
                        _ => format!("{no_way} give no axis and {odd_way} have one along none of the six ways a connection faces"),
                    };
                    made.notes.push(format!("{n} connection point{} say{} nothing of which way {} face{} ({why}): written with where {} {}, and no facing; say one with plumbline connection <job> <item> set <connection> faces <back|front|left|right|up|down>", if n == 1 { "" } else { "s" }, if n == 1 { "s" } else { "" }, if n == 1 { "it" } else { "they" }, if n == 1 { "s" } else { "" }, if n == 1 { "it" } else { "they" }, if n == 1 { "is" } else { "are" }));
                }
                if let Ok(worked) = ed::item_connections_worked(job, &path) {
                    made.connections = worked.into_iter().map(|w| w.said).collect();
                }
            } else if !ports.is_empty() {
                made.connections_not_written.push(format!("its {} in the file {} not written: with no sizes there is no body to say where on it they are", if ports.len() == 1 { "connection point".to_string() } else { format!("{} connection points", ports.len()) }, if ports.len() == 1 { "is" } else { "are" }));
            }
            if made.count > 1 {
                made.notes.push(format!("{} of that type are in the file, and one item is made of them", made.count));
            }
            if !made.has_shape {
                made.notes.push("it is not drawn until it has sizes".to_string());
            }
            out.made.push(made);
        }
        Ok(())
    });
    done?;
    if out.made.iter().any(|m| m.has_shape) {
        out.notes.push("each item with sizes is drawn as a box of its sizes, and the body's own shape was not kept: a maker's body is thousands of faces at one size, and a shape here is blocks written from the item's sizes".to_string());
    }
    if !out.made.is_empty() {
        out.notes.push("its weight, its material, its price and what it is classified as are not read".to_string());
        out.notes.push("the way a connection faces is read from the connection point's own axis; the standard's own text on that could not be read on the machine this was built on, so check one against the maker's sheet".to_string());
    }
    Ok(out)
}
