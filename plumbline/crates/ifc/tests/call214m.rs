//! Call 214 (contract 214M): a maker's model brought in as items. Every file here is made up: the maker "Northgate Fixtures", the product
//! "NL-2018" and every figure. The files are written in the test (a small builder over STEP text); the five files of the demonstration are
//! kept in `demos/maker-model/` and are read here as well, to the figures worked by hand in STATUS.md.

use plumbline_ifc::{maker_model, MakerModel};
use plumbline_world::editing as ed;
use plumbline_world::*;

// ---- a small builder of STEP text ----

#[derive(Clone, Copy)]
enum U {
    Mm,
    M,
    Ft,
}

struct B {
    n: u64,
    lines: Vec<String>,
    ctx: u64,
    schema: &'static str,
}

fn r(x: f64) -> String {
    format!("{x:?}")
}

impl B {
    fn new(unit: U, schema: &'static str) -> B {
        let mut b = B { n: 0, lines: Vec::new(), ctx: 0, schema };
        let org = b.add("IFCCARTESIANPOINT((0.,0.,0.))");
        let z = b.add("IFCDIRECTION((0.,0.,1.))");
        let x = b.add("IFCDIRECTION((1.,0.,0.))");
        let wa = b.add(format!("IFCAXIS2PLACEMENT3D(#{org},#{z},#{x})"));
        b.ctx = b.add(format!("IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-05,#{wa},$)"));
        let u = match unit {
            U::Mm => b.add("IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.)"),
            U::M => b.add("IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)"),
            U::Ft => {
                let si = b.add("IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)");
                let dim = b.add("IFCDIMENSIONALEXPONENTS(1,0,0,0,0,0,0)");
                let m = b.add(format!("IFCMEASUREWITHUNIT(IFCLENGTHMEASURE(0.3048),#{si})"));
                b.add(format!("IFCCONVERSIONBASEDUNIT(#{dim},.LENGTHUNIT.,'foot',#{m})"))
            }
        };
        let ua = b.add(format!("IFCUNITASSIGNMENT((#{u}))"));
        let g = b.guid();
        b.add(format!("IFCPROJECT('{g}',$,'Northgate (made up)',$,$,$,$,(#{}),#{ua})", b.ctx));
        b
    }
    fn guid(&mut self) -> String {
        format!("{:022}", self.n + 1)
    }
    fn add(&mut self, s: impl AsRef<str>) -> u64 {
        self.n += 1;
        self.lines.push(format!("#{}={};", self.n, s.as_ref()));
        self.n
    }
    fn pt(&mut self, p: [f64; 3]) -> u64 {
        self.add(format!("IFCCARTESIANPOINT(({},{},{}))", r(p[0]), r(p[1]), r(p[2])))
    }
    fn dir(&mut self, p: [f64; 3]) -> u64 {
        self.add(format!("IFCDIRECTION(({},{},{}))", r(p[0]), r(p[1]), r(p[2])))
    }
    fn axis(&mut self, at: [f64; 3], z: Option<[f64; 3]>, x: Option<[f64; 3]>) -> u64 {
        let p = self.pt(at);
        let z = z.map(|d| format!("#{}", self.dir(d))).unwrap_or("$".into());
        let x = x.map(|d| format!("#{}", self.dir(d))).unwrap_or("$".into());
        self.add(format!("IFCAXIS2PLACEMENT3D(#{p},{z},{x})"))
    }
    fn place(&mut self, rel: Option<u64>, at: [f64; 3], z: Option<[f64; 3]>, x: Option<[f64; 3]>) -> u64 {
        let a = self.axis(at, z, x);
        self.add(format!("IFCLOCALPLACEMENT({},#{a})", rel.map(|p| format!("#{p}")).unwrap_or("$".into())))
    }
    /// A solid of six faces, corner to corner.
    fn brep(&mut self, lo: [f64; 3], hi: [f64; 3]) -> u64 {
        let c = |b: &mut B, x: usize, y: usize, z: usize| b.pt([[lo[0], hi[0]][x], [lo[1], hi[1]][y], [lo[2], hi[2]][z]]);
        let (a, bb, cc, d) = (c(self, 0, 0, 0), c(self, 1, 0, 0), c(self, 1, 1, 0), c(self, 0, 1, 0));
        let (e, f, g, h) = (c(self, 0, 0, 1), c(self, 1, 0, 1), c(self, 1, 1, 1), c(self, 0, 1, 1));
        let mut faces = Vec::new();
        for ring in [[a, d, cc, bb], [e, f, g, h], [a, bb, f, e], [bb, cc, g, f], [cc, d, h, g], [d, a, e, h]] {
            let l = self.add(format!("IFCPOLYLOOP((#{},#{},#{},#{}))", ring[0], ring[1], ring[2], ring[3]));
            let ob = self.add(format!("IFCFACEOUTERBOUND(#{l},.T.)"));
            faces.push(self.add(format!("IFCFACE((#{ob}))")));
        }
        let shell = self.add(format!("IFCCLOSEDSHELL(({}))", faces.iter().map(|f| format!("#{f}")).collect::<Vec<_>>().join(",")));
        self.add(format!("IFCFACETEDBREP(#{shell})"))
    }
    /// The shape a product has: its items in one representation of the given identifier.
    fn shape(&mut self, ident: &str, kind: &str, items: &[u64]) -> u64 {
        let rep = self.add(format!("IFCSHAPEREPRESENTATION(#{},'{ident}','{kind}',({}))", self.ctx, items.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
        self.add(format!("IFCPRODUCTDEFINITIONSHAPE($,$,(#{rep}))"))
    }
    /// A product. `tail` is what follows its tag: ",$" for IFC4's PredefinedType, "" for IFC2x3.
    fn product(&mut self, class: &str, name: &str, place: u64, pds: Option<u64>, tail: &str) -> u64 {
        let g = self.guid();
        self.add(format!("{class}('{g}',$,'{name}',$,$,#{place},{},${tail})", pds.map(|p| format!("#{p}")).unwrap_or("$".into())))
    }
    fn pset_maker(&mut self, maker: Option<&str>, label: Option<&str>, reference: Option<&str>) -> u64 {
        let mut props = Vec::new();
        for (k, v) in [("Manufacturer", maker), ("ModelLabel", label), ("ModelReference", reference)] {
            if let Some(v) = v {
                props.push(format!("#{}", self.add(format!("IFCPROPERTYSINGLEVALUE('{k}',$,IFCLABEL('{v}'),$)"))));
            }
        }
        let g = self.guid();
        self.add(format!("IFCPROPERTYSET('{g}',$,'Pset_ManufacturerTypeInformation',$,({}))", props.join(",")))
    }
    fn type_of(&mut self, class: &str, name: &str, psets: &[u64], maps: &[u64], tail: &str, products: &[u64]) -> u64 {
        let g = self.guid();
        let list = |v: &[u64]| if v.is_empty() { "$".to_string() } else { format!("({})", v.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")) };
        let t = self.add(format!("{class}('{g}',$,'{name}',$,$,{},{},$,${tail})", list(psets), list(maps)));
        let g = self.guid();
        self.add(format!("IFCRELDEFINESBYTYPE('{g}',$,$,$,({}),#{t})", products.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
        t
    }
    fn rel_props(&mut self, product: u64, pset: u64) {
        let g = self.guid();
        self.add(format!("IFCRELDEFINESBYPROPERTIES('{g}',$,$,$,(#{product}),#{pset})"));
    }
    /// A connection point on a product, placed relative to the product's own placement; it points along `z`.
    fn port(&mut self, product: u64, product_place: u64, name: &str, at: [f64; 3], z: [f64; 3], flow: &str, takes: &str, system: &str) -> u64 {
        let p = self.place(Some(product_place), at, Some(z), Some(if z[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] }));
        let g = self.guid();
        let port = self.add(format!("IFCDISTRIBUTIONPORT('{g}',$,'{name}',$,$,#{p},$,.{flow}.,.{takes}.,.{system}.)"));
        let g = self.guid();
        self.add(format!("IFCRELNESTS('{g}',$,$,$,#{product},(#{port}))"));
        port
    }
    fn text(&self) -> String {
        format!("ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('ViewDefinition [ReferenceView]'),'2;1');\nFILE_NAME('made-up.ifc','2026-11-25T00:00:00',('made up'),('made up'),'made up','made up','');\nFILE_SCHEMA(('{}'));\nENDSEC;\nDATA;\n{}\nENDSEC;\nEND-ISO-10303-21;\n", self.schema, self.lines.join("\n"))
    }
}

/// The sanitary terminal of the demonstration, in the unit given, its body a solid from the origin to (w, d, h) in that unit; a type of its
/// own with the maker's set on it.
fn basin(unit: U, size: [f64; 3]) -> (B, u64, u64) {
    let mut b = B::new(unit, "IFC4");
    let body = b.brep([0.0; 3], size);
    let pds = b.shape("Body", "Brep", &[body]);
    let place = b.place(None, [0.0; 3], None, None);
    let p = b.product("IFCSANITARYTERMINAL", "NL-2018", place, Some(pds), ",.WASHHANDBASIN.");
    let ps = b.pset_maker(Some("Northgate Fixtures"), Some("NL-2018"), None);
    b.type_of("IFCSANITARYTERMINALTYPE", "Northgate NL-2018", &[ps], &[], ",.WASHHANDBASIN.", &[p]);
    (b, p, place)
}

fn fresh() -> Job {
    Job::new("A job (made up)", "test", "2026-11-25")
}

fn bring(job: &mut Job, b: &B, as_: Option<Kind>) -> Result<MakerModel, String> {
    maker_model(job, b.text().as_bytes(), "made-up.ifc", as_)
}

fn item_at(job: &mut Job, path: &str) -> Item {
    let e = job.find(path).unwrap_or_else(|| panic!("{path} is not in the job"));
    job.world.get::<Item>(e).cloned().unwrap_or_else(|| panic!("{path} is not an item"))
}

fn inches(v: &Val<Len>) -> f64 {
    let ft = v.feet().expect("a size");
    (ft * 12.0 * 1.0e4).round() / 1.0e4
}

fn sizes(it: &Item) -> [f64; 3] {
    [inches(&it.width), inches(&it.depth), inches(&it.height)]
}

fn said(m: &MakerModel) -> String {
    m.made.iter().flat_map(|i| i.read.iter().map(|(a, b, c)| format!("{a} {b} {c}")).chain(i.unknown.iter().map(|(a, b)| format!("{a} {b}"))).chain(i.connections.clone()).chain(i.connections_not_written.clone()).chain(i.notes.clone())).chain(m.refused.iter().map(|(a, b)| format!("{a}: {b}"))).chain(m.left_out.iter().map(|(a, n, b)| format!("{n} {a}: {b}"))).chain(m.building.clone()).chain(m.notes.clone()).collect::<Vec<_>>().join("\n")
}

// ---- the demonstration's file 1, by hand ----

/// Step 1 and 2 of the demonstration: a basin in millimetres with no building: one item, 20 by 18 by 8 in, Assumed, naming the file; its
/// maker and model Assumed; nothing set down; the job has no level.
#[test]
fn a_basin_in_millimetres_is_one_item_of_its_sizes() {
    let (b, _, _) = basin(U::Mm, [508.0, 457.2, 203.2]);
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(m.made.len(), 1, "{m:?}");
    let made = &m.made[0];
    assert_eq!(made.name, "Northgate NL-2018");
    assert_eq!(made.kind, Some(Kind::Fixture));
    assert_eq!(made.count, 1);
    let it = item_at(&mut job, &made.path);
    assert_eq!(sizes(&it), [20.0, 18.0, 8.0]);
    for v in [&it.width, &it.depth, &it.height] {
        assert_eq!(v.status, Status::Assumed);
        assert!(v.source.contains("made-up.ifc") && v.source.contains("extent of its body"), "{}", v.source);
    }
    assert_eq!(it.maker.value.as_deref(), Some("Northgate Fixtures"));
    assert_eq!(it.model.value.as_deref(), Some("NL-2018"));
    assert_eq!((it.maker.status, it.model.status), (Status::Assumed, Status::Assumed));
    assert!(it.maker.source.contains("made-up.ifc") && it.maker.source.contains("Manufacturer") && it.maker.source.contains("Pset_ManufacturerTypeInformation"), "{}", it.maker.source);
    assert!(it.model.source.contains("ModelLabel"), "{}", it.model.source);
    // Drawn as a box of its sizes, and the body's own shape was not kept.
    assert_eq!(it.form.as_deref(), Some("box"));
    assert!(said(&m).contains("drawn as a box") || m.notes.iter().any(|n| n.contains("box")), "{}", said(&m));
    assert!(m.notes.iter().any(|n| n.contains("shape was not kept")), "{:?}", m.notes);
    // Nothing set down, and the job has no level and needs none.
    assert!(job.world.query::<&Level>().iter(&job.world).next().is_none());
    assert!(job.world.query::<&Placed>().iter(&job.world).next().is_none());
}

fn row(rows: &[ItemConnection], name: &str) -> ItemConnection {
    rows.iter().find(|r| r.name == name).unwrap_or_else(|| panic!("no {name} in {rows:?}")).clone()
}

/// The demonstration's file 1 with its three connection points: cold water in at (355.6, 457.2, 50.8), hot water in at (152.4, 457.2,
/// 50.8), waste out at (254, 457.2, 0) facing down, by hand: cold 4 in across (355.6 less 254 is 101.6 mm), at the back, 2 in up; hot -4 in; waste
/// on the middle, at the back, at the foot, down.
fn basin_with_ports() -> (B, u64) {
    let (mut b, p, place) = basin(U::Mm, [508.0, 457.2, 203.2]);
    b.port(p, place, "Cold water", [355.6, 457.2, 50.8], [0.0, 1.0, 0.0], "SINK", "PIPE", "DOMESTICCOLDWATER");
    b.port(p, place, "Hot water", [152.4, 457.2, 50.8], [0.0, 1.0, 0.0], "SINK", "PIPE", "DOMESTICHOTWATER");
    b.port(p, place, "Waste", [254.0, 457.2, 0.0], [0.0, 0.0, -1.0], "SOURCE", "PIPE", "DRAINAGE");
    (b, p)
}

#[test]
fn its_three_connections_are_three_rows_where_the_file_puts_them() {
    let (b, _) = basin_with_ports();
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    let rows = ed::item_rows(&mut job, &m.made[0].path).unwrap();
    assert_eq!(rows.len(), 3, "{rows:?}");
    let (cold, hot, waste) = (row(&rows, "Cold water"), row(&rows, "Hot water"), row(&rows, "Waste"));
    assert_eq!((cold.across.as_str(), cold.back.as_str(), cold.up.as_str()), ("4 in", "0", "2 in"));
    assert_eq!((hot.across.as_str(), hot.back.as_str(), hot.up.as_str()), ("-4 in", "0", "2 in"));
    assert_eq!((waste.across.as_str(), waste.back.as_str(), waste.up.as_str()), ("0", "0", "0"));
    assert_eq!((cold.faces, hot.faces, waste.faces), (Some(Facing::Back), Some(Facing::Back), Some(Facing::Down)));
    assert_eq!((cold.kind, cold.carries, cold.flow), (ConnectorKind::Pipe, Some(Domain::Cold), Flow::In));
    assert_eq!((hot.carries, hot.flow), (Some(Domain::Hot), Flow::In));
    assert_eq!((waste.carries, waste.flow), (Some(Domain::Waste), Flow::Out));
    for r in &rows {
        assert_eq!(r.place, Status::Assumed);
        assert!(r.place_from.contains("made-up.ifc"), "{}", r.place_from);
    }
    // As the item holds them, worked at its sizes, they say where in inches.
    let worked = ed::item_connections_worked(&mut job, &m.made[0].path).unwrap();
    assert!(worked.iter().any(|w| w.said.contains("across 4 in, back 0 in, up 2 in")), "{worked:?}");
    assert_eq!(m.made[0].connections.len(), 3);
}

/// Step 4: a level made and the item set down: `takeoff` names its three connections open, as for any item.
#[test]
fn set_down_its_connections_are_there_and_open() {
    let (b, _) = basin_with_ports();
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    let l = ed::storey(&mut job, "Level 1", "0 ft", "9 ft", ed::Entered { status: Status::Assumed, source: "a test" }).unwrap();
    let thing = ed::place_item(&mut job, &l, &m.made[0].path, [5.0, 5.0], "0 ft", ed::Entered { status: Status::Assumed, source: "a test" }).unwrap();
    let e = job.find(&thing).unwrap();
    let c = job.world.get::<Connectors>(e).expect("the thing has connections");
    assert_eq!(c.0.len(), 3);
}

/// Step 5: the same file again changes nothing: every item is refused by name and the job is byte for byte what it was.
#[test]
fn the_same_file_twice_changes_nothing_at_all() {
    let (b, _) = basin_with_ports();
    let mut job = fresh();
    bring(&mut job, &b, None).unwrap();
    let before = file::save(&mut job);
    let again = bring(&mut job, &b, None).unwrap();
    assert!(again.made.is_empty(), "{again:?}");
    assert_eq!(again.refused.len(), 1);
    assert!(again.refused[0].0.contains("Northgate NL-2018") && again.refused[0].1.contains("already"), "{:?}", again.refused);
    assert_eq!(file::save(&mut job), before, "the job changed");
}

/// A name the chest already has is refused for that item; the rest come in.
#[test]
fn a_name_the_chest_has_is_refused_and_the_rest_come_in() {
    let (mut b, _, _) = basin(U::Mm, [508.0, 457.2, 203.2]);
    let body = b.brep([0.0; 3], [600.0, 500.0, 900.0]);
    let pds = b.shape("Body", "Brep", &[body]);
    let place = b.place(None, [1000.0, 0.0, 0.0], None, None);
    let q = b.product("IFCSANITARYTERMINAL", "Other", place, Some(pds), ",.TOILETPAN.");
    b.type_of("IFCSANITARYTERMINALTYPE", "Northgate WC-9", &[], &[], ",.TOILETPAN.", &[q]);
    let mut job = fresh();
    let first = ed::item(&mut job, "northgate nl 2018", Item { category: Kind::Fixture, maker: Val::unknown("", "x"), model: Val::unknown("", "x"), width: Val::unknown("in", "x"), depth: Val::unknown("in", "x"), height: Val::unknown("in", "x"), weight: None, sheet: String::new(), division: None, form: None, priced_as: None }).unwrap();
    let before_items = first.clone();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(m.made.len(), 1);
    assert_eq!(m.made[0].name, "Northgate WC-9");
    assert_eq!(m.refused.len(), 1, "{m:?}");
    assert!(m.refused[0].0.contains("Northgate NL-2018"), "{:?}", m.refused);
    assert!(job.find(&before_items).is_some());
}

// ---- units ----

#[test]
fn metres_feet_and_millimetres_come_to_the_same_inches() {
    for (unit, size) in [(U::Mm, [508.0, 457.2, 203.2]), (U::M, [0.508, 0.4572, 0.2032]), (U::Ft, [20.0 / 12.0, 18.0 / 12.0, 8.0 / 12.0])] {
        let (b, _, _) = basin(unit, size);
        let mut job = fresh();
        let m = bring(&mut job, &b, None).unwrap();
        let it = item_at(&mut job, &m.made[0].path);
        assert_eq!(sizes(&it), [20.0, 18.0, 8.0], "unit {}", m.unit);
    }
    let (b, _, _) = basin(U::Ft, [20.0 / 12.0, 18.0 / 12.0, 8.0 / 12.0]);
    let m = bring(&mut fresh(), &b, None).unwrap();
    assert!(m.unit.contains("feet"), "{}", m.unit);
}

// ---- each kind of body ----

/// One thing whose body is built by `body`; its sizes in inches (the file is in millimetres).
fn body_of(body: impl FnOnce(&mut B) -> Vec<u64>) -> [f64; 3] {
    let mut b = B::new(U::Mm, "IFC4");
    let items = body(&mut b);
    let pds = b.shape("Body", "SweptSolid", &items);
    let place = b.place(None, [0.0; 3], None, None);
    let p = b.product("IFCFURNISHINGELEMENT", "Stool", place, Some(pds), "");
    let _ = p;
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(m.made.len(), 1, "{m:?}");
    sizes(&item_at(&mut job, &m.made[0].path))
}

fn extrusion(b: &mut B, profile: String, at: [f64; 3], dir: [f64; 3], depth: f64) -> u64 {
    let prof = b.add(profile);
    let pos = b.axis(at, None, None);
    let d = b.dir(dir);
    b.add(format!("IFCEXTRUDEDAREASOLID(#{prof},#{pos},#{d},{})", r(depth)))
}

/// A rectangle 254 by 127 mm (10 in by 5 in), pushed 762 mm (30 in) up from the floor: a pushed shape, as the program has always read.
#[test]
fn a_rectangle_pushed_up_is_its_own_sizes() {
    let s = body_of(|b| {
        let o = b.add("IFCAXIS2PLACEMENT2D(#1,$)");
        vec![extrusion(b, format!("IFCRECTANGLEPROFILEDEF(.AREA.,$,#{o},254.,127.)"), [0.0; 3], [0.0, 0.0, 1.0], 762.0)]
    });
    assert_eq!(s, [10.0, 5.0, 30.0]);
}

#[test]
fn a_round_body_is_the_square_it_fills() {
    let s = body_of(|b| {
        let o = b.add("IFCAXIS2PLACEMENT2D(#1,$)");
        vec![extrusion(b, format!("IFCCIRCLEPROFILEDEF(.AREA.,$,#{o},127.)"), [0.0; 3], [0.0, 0.0, 1.0], 457.2)]
    });
    assert_eq!(s, [10.0, 10.0, 18.0]);
}

/// An outline pushed up: its corners (0,0) (300,0) (300,100) (100,100) (100,200) (0,200), 25.4 mm tall times 10: an L, 300 by 200 mm, 254 mm high.
#[test]
fn an_outline_pushed_up_is_the_box_round_its_corners() {
    let s = body_of(|b| {
        let pts: Vec<u64> = [[0.0, 0.0], [304.8, 0.0], [304.8, 101.6], [101.6, 101.6], [101.6, 203.2], [0.0, 203.2], [0.0, 0.0]].iter().map(|p| b.add(format!("IFCCARTESIANPOINT(({},{}))", r(p[0]), r(p[1])))).collect();
        let pl = b.add(format!("IFCPOLYLINE(({}))", pts.iter().map(|p| format!("#{p}")).collect::<Vec<_>>().join(",")));
        vec![extrusion(b, format!("IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,#{pl})"), [0.0; 3], [0.0, 0.0, 1.0], 254.0)]
    });
    assert_eq!(s, [12.0, 8.0, 10.0]);
}

/// A body pushed along x and not up: a pushed shape lying down is read by its corners, not refused. The solid's own z is the world's x and its
/// own x the world's y, so a 127 by 254 mm profile pushed 508 mm is 508 along x, 127 across y and 254 up z.
#[test]
fn a_body_pushed_sideways_is_read_by_its_corners() {
    let s = body_of(|b| {
        let o = b.add("IFCAXIS2PLACEMENT2D(#1,$)");
        let prof = b.add(format!("IFCRECTANGLEPROFILEDEF(.AREA.,$,#{o},127.,254.)"));
        let pos = b.axis([0.0; 3], Some([1.0, 0.0, 0.0]), Some([0.0, 1.0, 0.0]));
        let d = b.dir([0.0, 0.0, 1.0]);
        vec![b.add(format!("IFCEXTRUDEDAREASOLID(#{prof},#{pos},#{d},508.)"))]
    });
    assert_eq!(s, [20.0, 5.0, 10.0]);
}

#[test]
fn a_solid_of_faces_is_its_corners() {
    let s = body_of(|b| vec![b.brep([25.4, 50.8, 0.0], [279.4, 355.6, 508.0])]);
    assert_eq!(s, [10.0, 12.0, 20.0]);
}

fn coords(b: &mut B, pts: &[[f64; 3]]) -> u64 {
    b.add(format!("IFCCARTESIANPOINTLIST3D(({}))", pts.iter().map(|p| format!("({},{},{})", r(p[0]), r(p[1]), r(p[2]))).collect::<Vec<_>>().join(",")))
}

/// Two triangles' worth of points: 0..254 across, 0..127 deep, 0..381 high.
const WEDGE: [[f64; 3]; 5] = [[0.0, 0.0, 0.0], [254.0, 0.0, 0.0], [254.0, 127.0, 0.0], [0.0, 127.0, 381.0], [100.0, 50.0, 200.0]];

#[test]
fn a_set_of_triangles_is_its_points() {
    let s = body_of(|b| {
        let c = coords(b, &WEDGE);
        vec![b.add(format!("IFCTRIANGULATEDFACESET(#{c},$,.T.,((1,2,3),(1,3,4),(2,3,5)),$)"))]
    });
    assert_eq!(s, [10.0, 5.0, 15.0]);
}

/// A point the faces do not use is no part of the body.
#[test]
fn a_point_no_face_uses_is_not_the_body() {
    let s = body_of(|b| {
        let mut pts = WEDGE.to_vec();
        pts.push([9999.0, 9999.0, 9999.0]);
        let c = coords(b, &pts);
        vec![b.add(format!("IFCTRIANGULATEDFACESET(#{c},$,.T.,((1,2,3),(1,3,4),(2,3,5)),$)"))]
    });
    assert_eq!(s, [10.0, 5.0, 15.0]);
}

#[test]
fn a_set_of_polygons_is_its_points() {
    let s = body_of(|b| {
        let c = coords(b, &WEDGE);
        let f1 = b.add("IFCINDEXEDPOLYGONALFACE((1,2,3,4))");
        let f2 = b.add("IFCINDEXEDPOLYGONALFACE((2,3,5))");
        vec![b.add(format!("IFCPOLYGONALFACESET(#{c},$,(#{f1},#{f2}),$)"))]
    });
    assert_eq!(s, [10.0, 5.0, 15.0]);
}

#[test]
fn a_bounding_box_is_its_sizes() {
    let s = body_of(|b| {
        let c = b.pt([0.0; 3]);
        vec![b.add(format!("IFCBOUNDINGBOX(#{c},304.8,152.4,609.6)"))]
    });
    assert_eq!(s, [12.0, 6.0, 24.0]);
}

/// A solid with a part cut off is the element it is: its first part, and the other too when they are joined (as BIMCraft reads a clipping).
#[test]
fn a_clipped_body_is_the_body_before_the_clip() {
    let s = body_of(|b| {
        let first = b.brep([0.0; 3], [254.0, 254.0, 254.0]);
        let second = b.brep([0.0; 3], [100.0, 100.0, 100.0]);
        vec![b.add(format!("IFCBOOLEANCLIPPINGRESULT(.DIFFERENCE.,#{first},#{second})"))]
    });
    assert_eq!(s, [10.0, 10.0, 10.0]);
}

#[test]
fn a_surface_model_of_faces_is_its_points() {
    let s = body_of(|b| {
        let a = b.pt([0.0; 3]);
        let c = b.pt([254.0, 0.0, 0.0]);
        let d = b.pt([254.0, 254.0, 508.0]);
        let l = b.add(format!("IFCPOLYLOOP((#{a},#{c},#{d}))"));
        let ob = b.add(format!("IFCFACEOUTERBOUND(#{l},.T.)"));
        let f = b.add(format!("IFCFACE((#{ob}))"));
        let set = b.add(format!("IFCCONNECTEDFACESET((#{f}))"));
        vec![b.add(format!("IFCFACEBASEDSURFACEMODEL((#{set}))"))]
    });
    assert_eq!(s, [10.0, 10.0, 20.0]);
}

/// A body turned about an axis is one this does not read: the item comes in with its three sizes Unknown, saying which kind it was, and says
/// it is not drawn until it has sizes.
#[test]
fn a_turned_body_comes_in_with_its_sizes_unknown_and_says_why() {
    let mut b = B::new(U::Mm, "IFC4");
    let o = b.add("IFCAXIS2PLACEMENT2D(#1,$)");
    let prof = b.add(format!("IFCRECTANGLEPROFILEDEF(.AREA.,$,#{o},50.,200.)"));
    let pos = b.axis([100.0, 0.0, 0.0], None, None);
    let at = b.pt([0.0; 3]);
    let d = b.dir([0.0, 1.0, 0.0]);
    let ax = b.add(format!("IFCAXIS1PLACEMENT(#{at},#{d})"));
    let rev = b.add(format!("IFCREVOLVEDAREASOLID(#{prof},#{pos},#{ax},6.283185307)"));
    let pds = b.shape("Body", "SweptSolid", &[rev]);
    let place = b.place(None, [0.0; 3], None, None);
    let p = b.product("IFCSANITARYTERMINAL", "NP-1", place, Some(pds), ",.BIDET.");
    b.type_of("IFCSANITARYTERMINALTYPE", "Northgate NP-1", &[], &[], ",.BIDET.", &[p]);
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(m.made.len(), 1);
    let it = item_at(&mut job, &m.made[0].path);
    for v in [&it.width, &it.depth, &it.height] {
        assert_eq!(v.status, Status::Unknown);
        assert!(v.value.is_none());
        assert!(v.source.contains("turned about an axis"), "{}", v.source);
    }
    assert!(!m.made[0].has_shape);
    assert!(said(&m).contains("not drawn"), "{}", said(&m));
}

// ---- a shape shared by its type ----

/// The usual way a maker ships: one shape on the type, and each of the type in the file uses it. Two of the type: one item, and the reply says two.
#[test]
fn a_shape_shared_by_its_type_is_one_item_and_the_reply_counts_the_two() {
    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [508.0, 457.2, 203.2]);
    let rep = b.add(format!("IFCSHAPEREPRESENTATION(#{},'Body','Brep',(#{body}))", b.ctx));
    let axis = b.axis([0.0; 3], None, None);
    let map = b.add(format!("IFCREPRESENTATIONMAP(#{axis},#{rep})"));
    let mut products = Vec::new();
    for at in [[0.0, 0.0, 0.0], [2000.0, 0.0, 0.0]] {
        let target = b.pt([0.0; 3]);
        let op = b.add(format!("IFCCARTESIANTRANSFORMATIONOPERATOR3D($,$,#{target},$,$)"));
        let mi = b.add(format!("IFCMAPPEDITEM(#{map},#{op})"));
        let pds = b.shape("Body", "MappedRepresentation", &[mi]);
        let place = b.place(None, at, None, None);
        products.push(b.product("IFCSANITARYTERMINAL", "NL-2018", place, Some(pds), ",.WASHHANDBASIN."));
    }
    let ps = b.pset_maker(Some("Northgate Fixtures"), None, Some("NL-2018"));
    b.type_of("IFCSANITARYTERMINALTYPE", "Northgate NL-2018", &[ps], &[map], ",.WASHHANDBASIN.", &products);
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(m.made.len(), 1, "{m:?}");
    assert_eq!(m.made[0].count, 2);
    assert!(said(&m).contains("2 of that type"), "{}", said(&m));
    assert_eq!(sizes(&item_at(&mut job, &m.made[0].path)), [20.0, 18.0, 8.0]);
    // Its model is the model reference, as the standard calls the number.
    let it = item_at(&mut job, &m.made[0].path);
    assert_eq!(it.model.value.as_deref(), Some("NL-2018"));
    assert!(it.model.source.contains("ModelReference"), "{}", it.model.source);
}

/// A mapped shape scaled by its operator is read at that scale, and one whose own origin is off the origin is not read (nothing is guessed).
#[test]
fn a_mapped_shape_is_read_at_its_scale() {
    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [254.0, 254.0, 254.0]);
    let rep = b.add(format!("IFCSHAPEREPRESENTATION(#{},'Body','Brep',(#{body}))", b.ctx));
    let axis = b.axis([0.0; 3], None, None);
    let map = b.add(format!("IFCREPRESENTATIONMAP(#{axis},#{rep})"));
    let target = b.pt([0.0; 3]);
    let op = b.add(format!("IFCCARTESIANTRANSFORMATIONOPERATOR3D($,$,#{target},2.,$)"));
    let mi = b.add(format!("IFCMAPPEDITEM(#{map},#{op})"));
    let pds = b.shape("Body", "MappedRepresentation", &[mi]);
    let place = b.place(None, [0.0; 3], None, None);
    b.product("IFCFURNISHINGELEMENT", "Cube", place, Some(pds), "");
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(sizes(&item_at(&mut job, &m.made[0].path)), [20.0, 20.0, 20.0]);

    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [254.0, 254.0, 254.0]);
    let rep = b.add(format!("IFCSHAPEREPRESENTATION(#{},'Body','Brep',(#{body}))", b.ctx));
    let axis = b.axis([500.0, 0.0, 0.0], None, None);
    let map = b.add(format!("IFCREPRESENTATIONMAP(#{axis},#{rep})"));
    let target = b.pt([0.0; 3]);
    let op = b.add(format!("IFCCARTESIANTRANSFORMATIONOPERATOR3D($,$,#{target},$,$)"));
    let mi = b.add(format!("IFCMAPPEDITEM(#{map},#{op})"));
    let pds = b.shape("Body", "MappedRepresentation", &[mi]);
    let place = b.place(None, [0.0; 3], None, None);
    b.product("IFCFURNISHINGELEMENT", "Cube", place, Some(pds), "");
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    let it = item_at(&mut job, &m.made[0].path);
    assert_eq!(it.width.status, Status::Unknown);
    assert!(it.width.source.contains("origin"), "{}", it.width.source);
}

/// A type that carries the shape and a product that carries none: the type's shape is the product's.
#[test]
fn a_type_with_the_shape_and_a_product_with_none() {
    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [508.0, 457.2, 203.2]);
    let rep = b.add(format!("IFCSHAPEREPRESENTATION(#{},'Body','Brep',(#{body}))", b.ctx));
    let axis = b.axis([0.0; 3], None, None);
    let map = b.add(format!("IFCREPRESENTATIONMAP(#{axis},#{rep})"));
    let place = b.place(None, [0.0; 3], None, None);
    let p = b.product("IFCSANITARYTERMINAL", "NL-2018", place, None, ",.WASHHANDBASIN.");
    b.type_of("IFCSANITARYTERMINALTYPE", "Northgate NL-2018", &[], &[map], ",.WASHHANDBASIN.", &[p]);
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(sizes(&item_at(&mut job, &m.made[0].path)), [20.0, 18.0, 8.0]);
}

// ---- what kind of thing it is ----

fn one_class(class: &str, tail: &str, as_: Option<Kind>) -> (Result<MakerModel, String>, Job) {
    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [508.0, 457.2, 203.2]);
    let pds = b.shape("Body", "Brep", &[body]);
    let place = b.place(None, [0.0; 3], None, None);
    b.product(class, "Thing A", place, Some(pds), tail);
    let mut job = fresh();
    (bring(&mut job, &b, as_), job)
}

#[test]
fn a_furnishing_a_fixture_a_piece_of_equipment_and_an_appliance_by_their_classes() {
    for (class, tail, kind) in [("IFCFURNITURE", ",$", Kind::Furnishing), ("IFCLIGHTFIXTURE", ",$", Kind::Fixture), ("IFCUNITARYEQUIPMENT", ",$", Kind::Equipment), ("IFCELECTRICAPPLIANCE", ",.FREESTANDINGRANGE.", Kind::Equipment)] {
        let (m, _) = one_class(class, tail, None);
        let m = m.unwrap_or_else(|why| panic!("{class}: {why}"));
        assert_eq!(m.made.len(), 1, "{class}: {m:?}");
        assert_eq!(m.made[0].kind, Some(kind), "{class}");
    }
    // An electric appliance says why it is equipment.
    let (m, _) = one_class("IFCELECTRICAPPLIANCE", ",.FREESTANDINGRANGE.", None);
    assert!(m.unwrap().made[0].kind_why.contains("electric appliance"), "why");
}

/// A proxy says nothing of what it is. Without --as it is left out and named, and the reply says how to say what such things are; with --as,
/// every proxy in the file is that.
#[test]
fn a_proxy_is_left_out_unless_the_person_says_what_it_is() {
    let (m, mut job) = one_class("IFCBUILDINGELEMENTPROXY", ",.NOTDEFINED.", None);
    let m = m.unwrap();
    assert!(m.made.is_empty());
    assert_eq!(m.left_out.len(), 1, "{m:?}");
    assert_eq!(m.left_out[0].1, 1);
    let said = said(&m);
    assert!(said.contains("--as fixture") && said.contains("equipment") && said.contains("furnishing"), "{said}");
    assert!(said.contains("of no stated kind"), "{said}");
    assert!(job.world.query::<&Item>().iter(&job.world).next().is_none());
    for (as_, kind) in [(Kind::Fixture, Kind::Fixture), (Kind::Equipment, Kind::Equipment), (Kind::Furnishing, Kind::Furnishing)] {
        let (m, _) = one_class("IFCBUILDINGELEMENTPROXY", ",.NOTDEFINED.", Some(as_));
        let m = m.unwrap();
        assert_eq!(m.made.len(), 1);
        assert_eq!(m.made[0].kind, Some(kind));
        assert!(m.made[0].kind_why.contains("you said"), "{}", m.made[0].kind_why);
    }
    // --as is for what says nothing: a fixture stays what its class says.
    let (m, _) = one_class("IFCLIGHTFIXTURE", ",$", Some(Kind::Equipment));
    assert_eq!(m.unwrap().made[0].kind, Some(Kind::Fixture));
}

// ---- the maker's property set ----

fn with_pset(on_type: Option<(Option<&str>, Option<&str>, Option<&str>)>, on_product: Option<(Option<&str>, Option<&str>, Option<&str>)>) -> Item {
    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [508.0, 457.2, 203.2]);
    let pds = b.shape("Body", "Brep", &[body]);
    let place = b.place(None, [0.0; 3], None, None);
    let p = b.product("IFCSANITARYTERMINAL", "NL-2018", place, Some(pds), ",.WASHHANDBASIN.");
    let ts: Vec<u64> = on_type.map(|(a, l, rf)| b.pset_maker(a, l, rf)).into_iter().collect();
    b.type_of("IFCSANITARYTERMINALTYPE", "Northgate NL-2018", &ts, &[], ",.WASHHANDBASIN.", &[p]);
    if let Some((a, l, rf)) = on_product {
        let ps = b.pset_maker(a, l, rf);
        b.rel_props(p, ps);
    }
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    item_at(&mut job, &m.made[0].path)
}

#[test]
fn the_maker_and_model_on_the_type_on_the_product_and_nowhere() {
    let it = with_pset(Some((Some("Northgate Fixtures"), Some("NL-2018"), None)), None);
    assert_eq!((it.maker.value.as_deref(), it.model.value.as_deref()), (Some("Northgate Fixtures"), Some("NL-2018")));
    assert!(it.maker.source.contains("on the type"), "{}", it.maker.source);
    let it = with_pset(None, Some((Some("Northgate Fixtures"), Some("NL-2018"), None)));
    assert_eq!((it.maker.value.as_deref(), it.model.value.as_deref()), (Some("Northgate Fixtures"), Some("NL-2018")));
    assert!(it.maker.source.contains("on the thing"), "{}", it.maker.source);
    assert_eq!(it.maker.status, Status::Assumed);
    // On both: the type's.
    let it = with_pset(Some((Some("Type Maker"), None, None)), Some((Some("Product Maker"), None, None)));
    assert_eq!(it.maker.value.as_deref(), Some("Type Maker"));
    // Neither: Unknown, "not said in the model".
    let it = with_pset(None, None);
    for v in [&it.maker, &it.model] {
        assert_eq!(v.status, Status::Unknown);
        assert!(v.value.is_none());
        assert_eq!(v.source, "not said in the model");
    }
    // Only the maker said: the model is Unknown and the maker is not.
    let it = with_pset(Some((Some("Northgate Fixtures"), None, None)), None);
    assert_eq!(it.maker.status, Status::Assumed);
    assert_eq!(it.model.status, Status::Unknown);
}

// ---- each connection kind ----

fn with_port(at: [f64; 3], z: [f64; 3], flow: &str, takes: &str, system: &str) -> (Vec<ItemConnection>, MakerModel) {
    let (mut b, p, place) = basin(U::Mm, [508.0, 457.2, 203.2]);
    b.port(p, place, "Point", at, z, flow, takes, system);
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    (ed::item_rows(&mut job, &m.made[0].path).unwrap(), m)
}

#[test]
fn each_kind_of_connection_is_read_as_the_model_reads_it_today() {
    let mid = [254.0, 228.6, 100.0];
    let up = [0.0, 0.0, 1.0];
    for (takes, system, flow, kind, carries, goes) in [
        ("PIPE", "DOMESTICCOLDWATER", "SINK", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In),
        ("PIPE", "DOMESTICHOTWATER", "SINK", ConnectorKind::Pipe, Some(Domain::Hot), Flow::In),
        ("PIPE", "SEWAGE", "SOURCE", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out),
        ("PIPE", "VENT", "SOURCEANDSINK", ConnectorKind::Pipe, Some(Domain::Vent), Flow::Both),
        ("PIPE", "RAINWATER", "SOURCE", ConnectorKind::Pipe, Some(Domain::Storm), Flow::Out),
        ("PIPE", "GAS", "SINK", ConnectorKind::Pipe, Some(Domain::Gas), Flow::In),
        ("DUCT", "EXHAUST", "SOURCE", ConnectorKind::Duct, None, Flow::Out),
        ("CABLECARRIER", "ELECTRICAL", "SINK", ConnectorKind::Conduit, None, Flow::In),
        ("CABLE", "ELECTRICAL", "SINK", ConnectorKind::Power, None, Flow::In),
    ] {
        let (rows, _) = with_port(mid, up, flow, takes, system);
        assert_eq!(rows.len(), 1, "{takes} {system}");
        let r = &rows[0];
        assert_eq!((r.kind, r.carries, r.flow), (kind, carries, goes), "{takes} {system}");
        assert_eq!(r.faces, Some(Facing::Up));
        assert_eq!((r.across.as_str(), r.back.as_str(), r.up.as_str()), ("0", "9 in", "3.937 in"), "{takes}");
    }
}

/// A point whose direction lies along each of the six, and one that lies along none.
#[test]
fn the_way_it_faces_only_where_the_file_says_one_of_the_six() {
    let at = [254.0, 457.2, 100.0];
    for (z, face) in [([0.0, 1.0, 0.0], Facing::Back), ([0.0, -1.0, 0.0], Facing::Front), ([-1.0, 0.0, 0.0], Facing::Left), ([1.0, 0.0, 0.0], Facing::Right), ([0.0, 0.0, 1.0], Facing::Up), ([0.0, 0.0, -1.0], Facing::Down)] {
        let (rows, _) = with_port(at, z, "SINK", "PIPE", "GAS");
        assert_eq!(rows[0].faces, Some(face), "{z:?}");
    }
    let (rows, m) = with_port(at, [0.6, 0.8, 0.0], "SINK", "PIPE", "GAS");
    // Round 3: where it connects is worth more than which way it faces: written, with no facing, and said.
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].faces, None);
    assert!(m.made[0].notes.iter().any(|n| n.contains("1 connection point says nothing of which way it faces") && n.contains("none of the six")), "{:?}", m.made[0].notes);
}

/// A point 100 mm outside the body (past its back, 4 in) is not written and is named; the three inside are.
#[test]
fn a_point_outside_the_body_is_not_written_and_is_named() {
    let (mut b, p, place) = basin(U::Mm, [508.0, 457.2, 203.2]);
    b.port(p, place, "Cold water", [355.6, 457.2, 50.8], [0.0, 1.0, 0.0], "SINK", "PIPE", "DOMESTICCOLDWATER");
    b.port(p, place, "Hot water", [152.4, 457.2, 50.8], [0.0, 1.0, 0.0], "SINK", "PIPE", "DOMESTICHOTWATER");
    b.port(p, place, "Waste", [254.0, 457.2, 0.0], [0.0, 0.0, -1.0], "SOURCE", "PIPE", "DRAINAGE");
    b.port(p, place, "Overflow", [254.0, 557.2, 100.0], [0.0, 1.0, 0.0], "SOURCE", "PIPE", "DRAINAGE");
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(ed::item_rows(&mut job, &m.made[0].path).unwrap().len(), 3);
    let not = &m.made[0].connections_not_written;
    assert_eq!(not.len(), 1, "{not:?}");
    assert!(not[0].contains("Overflow") && not[0].contains("outside") && not[0].contains("3 15/16 in") && not[0].contains("behind"), "{not:?}");
}

/// A point with no sort in the file (not a pipe, a duct or a cable) is named and not written.
#[test]
fn a_point_that_says_no_sort_is_named() {
    let (rows, m) = with_port([254.0, 228.6, 100.0], [0.0, 0.0, 1.0], "SINK", "NOTDEFINED", "NOTDEFINED");
    assert!(rows.is_empty());
    assert!(m.made[0].connections_not_written[0].contains("no sort"), "{:?}", m.made[0].connections_not_written);
}

/// A product that is turned and put away from the origin: the points are worked in its own axes, not the world's. Turned a quarter turn
/// about z, 5 m along x and 2 m along y, the basin's back (its own y) lies along the world's -x.
#[test]
fn the_points_are_worked_in_the_things_own_axes_when_it_is_turned() {
    let mut b = B::new(U::Mm, "IFC4");
    let body = b.brep([0.0; 3], [508.0, 457.2, 203.2]);
    let pds = b.shape("Body", "Brep", &[body]);
    let place = b.place(None, [5000.0, 2000.0, 0.0], None, Some([0.0, 1.0, 0.0]));
    let p = b.product("IFCSANITARYTERMINAL", "NL-2018", place, Some(pds), ",.WASHHANDBASIN.");
    // Its own (355.6, 457.2, 50.8) is, in the world, x = 5000 - 457.2, y = 2000 + 355.6; the port is placed in the world's axes by its own
    // placement (relative to nothing) and its z is the thing's own y (back), which is the world's -x.
    let wp = b.place(None, [5000.0 - 457.2, 2000.0 + 355.6, 50.8], Some([-1.0, 0.0, 0.0]), Some([0.0, 1.0, 0.0]));
    let g = b.guid();
    let port = b.add(format!("IFCDISTRIBUTIONPORT('{g}',$,'Cold water',$,$,#{wp},$,.SINK.,.PIPE.,.DOMESTICCOLDWATER.)"));
    let g = b.guid();
    b.add(format!("IFCRELNESTS('{g}',$,$,$,#{p},(#{port}))"));
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    let rows = ed::item_rows(&mut job, &m.made[0].path).unwrap();
    assert_eq!(rows.len(), 1, "{:?}", m.made[0].connections_not_written);
    assert_eq!((rows[0].across.as_str(), rows[0].back.as_str(), rows[0].up.as_str(), rows[0].faces), ("4 in", "0", "2 in", Some(Facing::Back)));
}

// ---- a building in the file; a file that is not a model ----

/// A file with a building in it still makes its items and sets nothing down; the reply says what it left of the building.
#[test]
fn a_file_with_a_building_in_it_makes_items_and_sets_nothing_down() {
    let (mut b, _, _) = basin(U::Mm, [508.0, 457.2, 203.2]);
    let g = b.guid();
    let storey = b.add(format!("IFCBUILDINGSTOREY('{g}',$,'Level 1',$,$,$,$,$,.ELEMENT.,0.)"));
    let place = b.place(None, [0.0; 3], None, None);
    let g = b.guid();
    b.add(format!("IFCWALL('{g}',$,'Wall',$,$,#{place},$,$,.NOTDEFINED.)"));
    let _ = storey;
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert_eq!(m.made.len(), 1);
    let building = m.building.clone().expect("it says what the building is");
    assert!(building.contains("1 level") && building.contains("1 wall") && building.contains("not brought in") && building.contains("nothing is set down"), "{building}");
    assert!(building.contains("designer's model"), "{building}");
    assert!(job.world.query::<&Level>().iter(&job.world).next().is_none());
    assert!(job.world.query::<&Wall>().iter(&job.world).next().is_none());
}

#[test]
fn something_that_is_no_model_is_refused_and_the_job_is_as_it_was() {
    let mut job = fresh();
    let before = file::save(&mut job);
    assert!(maker_model(&mut job, b"this is not a model", "x.ifc", None).is_err());
    assert_eq!(file::save(&mut job), before);
}

/// Other things in the file that are not fixtures, equipment or furnishings are counted and left out, by what the file calls them.
#[test]
fn things_that_are_not_items_are_counted_and_left_out() {
    let (mut b, _, _) = basin(U::Mm, [508.0, 457.2, 203.2]);
    let body = b.brep([0.0; 3], [100.0, 100.0, 3000.0]);
    let pds = b.shape("Body", "Brep", &[body]);
    let place = b.place(None, [0.0; 3], None, None);
    b.product("IFCCOLUMN", "C1", place, Some(pds), ",$");
    b.product("IFCCOLUMN", "C2", place, Some(pds), ",$");
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    assert!(m.left_out.iter().any(|(what, n, _)| what.contains("column") && *n == 2), "{:?}", m.left_out);
}

// ---- the demonstration's five files, as kept ----

fn demo(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/../../demos/maker-model/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn the_demonstrations_file_1_is_what_the_hand_figures_say() {
    let mut job = fresh();
    let m = maker_model(&mut job, &demo("nl-2018.ifc"), "nl-2018.ifc", None).unwrap();
    assert_eq!(m.made.len(), 1);
    let it = item_at(&mut job, &m.made[0].path);
    assert_eq!(sizes(&it), [20.0, 18.0, 8.0]);
    assert_eq!(it.maker.value.as_deref(), Some("Northgate Fixtures"));
    assert_eq!(it.model.value.as_deref(), Some("NL-2018"));
    let rows = ed::item_rows(&mut job, &m.made[0].path).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(row(&rows, "Cold water").across, "4 in");
    assert_eq!(row(&rows, "Hot water").across, "-4 in");
    assert_eq!(row(&rows, "Waste").faces, Some(Facing::Down));
}

#[test]
fn the_demonstrations_other_files() {
    // File 2: the same product as a proxy.
    let mut job = fresh();
    let m = maker_model(&mut job, &demo("nl-2018-proxy.ifc"), "nl-2018-proxy.ifc", None).unwrap();
    assert!(m.made.is_empty() && m.left_out.len() == 1, "{m:?}");
    let m = maker_model(&mut job, &demo("nl-2018-proxy.ifc"), "nl-2018-proxy.ifc", Some(Kind::Fixture)).unwrap();
    assert_eq!(m.made.len(), 1);
    // File 3: the shape shared by its type, two of them.
    let mut job = fresh();
    let m = maker_model(&mut job, &demo("nl-2018-shared.ifc"), "nl-2018-shared.ifc", None).unwrap();
    assert_eq!((m.made.len(), m.made[0].count), (1, 2));
    assert_eq!(sizes(&item_at(&mut job, &m.made[0].path)), [20.0, 18.0, 8.0]);
    // File 4: a body that cannot be read.
    let mut job = fresh();
    let m = maker_model(&mut job, &demo("np-1-turned.ifc"), "np-1-turned.ifc", None).unwrap();
    assert!(!m.made[0].has_shape);
    // File 5: no maker's set.
    let mut job = fresh();
    let m = maker_model(&mut job, &demo("nl-2018-no-maker.ifc"), "nl-2018-no-maker.ifc", None).unwrap();
    let it = item_at(&mut job, &m.made[0].path);
    assert_eq!((it.maker.status, it.model.status), (Status::Unknown, Status::Unknown));
    // File 1 with a fourth point 100 mm outside the body.
    let mut job = fresh();
    let m = maker_model(&mut job, &demo("nl-2018-outside.ifc"), "nl-2018-outside.ifc", None).unwrap();
    assert_eq!(ed::item_rows(&mut job, &m.made[0].path).unwrap().len(), 3);
    assert_eq!(m.made[0].connections_not_written.len(), 1);
}

/// Round 2: a connection point whose placement gives no axis (the standard's default) says nothing of which way it faces, and a default is
/// not a statement: it is not written, and the reply counts them. One whose placement gives an axis along one of the six is written.
#[test]
fn a_placement_with_no_axis_says_nothing_of_which_way_it_faces() {
    let (mut b, p, place) = basin(U::Mm, [508.0, 457.2, 203.2]);
    for (name, x) in [("Plain 1", 100.0), ("Plain 2", 200.0), ("Plain 3", 300.0)] {
        let at = b.place(Some(place), [x, 228.6, 100.0], None, None);
        let g = b.guid();
        let port = b.add(format!("IFCDISTRIBUTIONPORT('{g}',$,'{name}',$,$,#{at},$,.SINK.,.PIPE.,.GAS.)"));
        let g = b.guid();
        b.add(format!("IFCRELNESTS('{g}',$,$,$,#{p},(#{port}))"));
    }
    b.port(p, place, "Stated", [254.0, 457.2, 100.0], [0.0, 1.0, 0.0], "SINK", "PIPE", "GAS");
    let mut job = fresh();
    let m = bring(&mut job, &b, None).unwrap();
    let rows = ed::item_rows(&mut job, &m.made[0].path).unwrap();
    // Round 3: the three with no axis are written too, with no facing; the one with an axis keeps its facing.
    assert_eq!(rows.len(), 4, "{rows:?}");
    assert_eq!(row(&rows, "Stated").faces, Some(Facing::Back));
    for n in ["Plain 1", "Plain 2", "Plain 3"] {
        assert_eq!(row(&rows, n).faces, None, "{n}");
    }
    assert!(m.made[0].notes.iter().any(|c| c.contains("3 connection points say nothing of which way they face")), "{:?}", m.made[0].notes);
    assert!(m.made[0].connections.iter().any(|c| c.contains("which way it faces is not said")), "{:?}", m.made[0].connections);
}
