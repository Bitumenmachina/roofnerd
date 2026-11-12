//! The job's building written as an IFC4 model, for another trade or a designer to read.
//!
//! What is written is what the building model places in space: each level; each wall as
//! a rectangle on plan pushed up, with a hole for each door and window and the door or
//! window in it; each floor as its outline pushed up; each room as the outline its
//! floor area is the area of; the roof's planes as they lie; each duct, pipe, conduit
//! and cable as its section pushed along its run; each fitting, fixture and piece of
//! equipment as its box; each system with what is on it. Lengths are in feet.
//!
//! Every value entered on a thing goes with it, with how sure it is and where it came
//! from, in a property set called `Plumbline_Values`: what is Assumed here is still said
//! to be Assumed in the model another program reads.
//!
//! **What an IFC model has no place for is counted and said, not dropped in silence**:
//! the estimate's lines and prices, materials, conditions, drawings, estimates made
//! before, and what has no shape (each with why).
//!
//! Ours. The shapes are the plain ones this crate's own reader reads back (`lib.rs`),
//! which is how the two are checked against each other.

use plumbline_world::building::forms::Form;
use std::collections::{BTreeMap, HashMap};

use plumbline_world::building::{self, FittingKind, Shape, SystemInfo};
use plumbline_world::{roof, view, Cost, Discipline, Fitting, Job, JobInfo, Kind, Level, Node};

type P3 = [f64; 3];

/// What writing the model did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Written {
    /// The file, as text.
    pub text: String,
    /// How many of each kind of thing were written, by what this program calls it.
    pub made: BTreeMap<String, usize>,
    /// What was not written: what, how many, and why.
    pub not_written: Vec<(String, usize, String)>,
}

/// A number as a STEP file writes one: a point always, no exponent, no negative zero.
fn real(x: f64) -> String {
    let rounded = (x * 1.0e6).round() / 1.0e6;
    let rounded = if rounded == 0.0 { 0.0 } else { rounded };
    let s = format!("{rounded:.6}");
    let s = s.trim_end_matches('0');
    s.to_string()
}

/// Text as a STEP file writes it: in quotation marks, a quotation mark doubled, a
/// backslash doubled, and anything outside the plain characters by its number.
fn text(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\'' => out.push_str("''"),
            '\\' => out.push_str("\\\\"),
            c if (' '..='~').contains(&c) => out.push(c),
            c => {
                let mut units = [0u16; 2];
                out.push_str("\\X2\\");
                for u in c.encode_utf16(&mut units) {
                    out.push_str(&format!("{u:04X}"));
                }
                out.push_str("\\X0\\");
            }
        }
    }
    out.push('\'');
    out
}

fn name_or_none(s: &str) -> String {
    if s.is_empty() {
        "$".to_string()
    } else {
        text(s)
    }
}

/// A thing's identifier in the file: 22 characters, the same for the same thing of the
/// same job every time it is written, so a model written again matches the one before.
fn guid(job: &str, key: &str) -> String {
    const LETTERS: &[u8; 64] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$";
    let hash = |seed: u64| -> u64 {
        let mut h = seed;
        for b in job.bytes().chain([0u8]).chain(key.bytes()) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        h ^ (h >> 29)
    };
    let bits = (u128::from(hash(0xCBF2_9CE4_8422_2325)) << 64) | u128::from(hash(0x8422_2325_CBF2_9CE4));
    let mut out = String::with_capacity(22);
    out.push(LETTERS[((bits >> 126) & 0x3) as usize] as char);
    for i in (0..21).rev() {
        out.push(LETTERS[((bits >> (i * 6)) & 0x3F) as usize] as char);
    }
    out
}

struct Out {
    lines: Vec<String>,
    next: u64,
    job: String,
    body: u64,
}

impl Out {
    fn add(&mut self, entity: String) -> u64 {
        self.next += 1;
        self.lines.push(format!("#{}={};", self.next, entity));
        self.next
    }
    fn id(&self, key: &str) -> String {
        text(&guid(&self.job, key))
    }
    fn point(&mut self, p: P3) -> u64 {
        self.add(format!("IFCCARTESIANPOINT(({},{},{}))", real(p[0]), real(p[1]), real(p[2])))
    }
    fn point2(&mut self, p: [f64; 2]) -> u64 {
        self.add(format!("IFCCARTESIANPOINT(({},{}))", real(p[0]), real(p[1])))
    }
    fn dir(&mut self, d: P3) -> u64 {
        self.add(format!("IFCDIRECTION(({},{},{}))", real(d[0]), real(d[1]), real(d[2])))
    }
    /// A place and a turn: where, which way is up for it, and which way is along.
    fn axes(&mut self, o: P3, z: P3, x: P3) -> u64 {
        let (o, z, x) = (self.point(o), self.dir(z), self.dir(x));
        self.add(format!("IFCAXIS2PLACEMENT3D(#{o},#{z},#{x})"))
    }
    fn placed(&mut self, within: Option<u64>, o: P3, z: P3, x: P3) -> u64 {
        let axes = self.axes(o, z, x);
        let within = within.map(|w| format!("#{w}")).unwrap_or_else(|| "$".to_string());
        self.add(format!("IFCLOCALPLACEMENT({within},#{axes})"))
    }
    /// A rectangle `x` by `y` with its middle at `at`, pushed `depth` along its own up.
    fn pushed_rect(&mut self, at: [f64; 2], x: f64, y: f64, depth: f64) -> u64 {
        let middle = self.point2(at);
        let on = self.add(format!("IFCAXIS2PLACEMENT2D(#{middle},$)"));
        let profile = self.add(format!("IFCRECTANGLEPROFILEDEF(.AREA.,$,#{on},{},{})", real(x), real(y)));
        self.pushed(profile, depth)
    }
    fn pushed_circle(&mut self, radius: f64, depth: f64) -> u64 {
        let middle = self.point2([0.0, 0.0]);
        let on = self.add(format!("IFCAXIS2PLACEMENT2D(#{middle},$)"));
        let profile = self.add(format!("IFCCIRCLEPROFILEDEF(.AREA.,$,#{on},{})", real(radius)));
        self.pushed(profile, depth)
    }
    fn pushed_outline(&mut self, ring: &[[f64; 2]], depth: f64) -> u64 {
        let mut points: Vec<String> = Vec::with_capacity(ring.len() + 1);
        for p in ring.iter().chain(ring.first()) {
            points.push(format!("#{}", self.point2(*p)));
        }
        let line = self.add(format!("IFCPOLYLINE(({}))", points.join(",")));
        let profile = self.add(format!("IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,#{line})"));
        self.pushed(profile, depth)
    }
    fn pushed(&mut self, profile: u64, depth: f64) -> u64 {
        let at = self.axes([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let up = self.dir([0.0, 0.0, 1.0]);
        let solid = self.add(format!("IFCEXTRUDEDAREASOLID(#{profile},#{at},#{up},{})", real(depth)));
        self.shape("SweptSolid", solid)
    }
    /// One flat face in space, by its corners in turn.
    fn face(&mut self, corners: &[P3]) -> u64 {
        let list = corners.iter().map(|p| format!("({},{},{})", real(p[0]), real(p[1]), real(p[2]))).collect::<Vec<_>>().join(",");
        let points = self.add(format!("IFCCARTESIANPOINTLIST3D(({list}))"));
        let index = (1..=corners.len()).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
        let face = self.add(format!("IFCINDEXEDPOLYGONALFACE(({index}))"));
        let set = self.add(format!("IFCPOLYGONALFACESET(#{points},.F.,(#{face}),$)"));
        self.shape("Tessellation", set)
    }
    fn shape(&mut self, how: &str, item: u64) -> u64 {
        let body = self.body;
        let rep = self.add(format!("IFCSHAPEREPRESENTATION(#{body},'Body','{how}',(#{item}))"));
        self.add(format!("IFCPRODUCTDEFINITIONSHAPE($,$,(#{rep}))"))
    }
}

/// Along a run: the unit way from `a` to `b`, and a level way across it.
fn along(a: P3, b: P3) -> Option<(P3, P3, f64)> {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if length < 1.0e-9 {
        return None;
    }
    let way = [d[0] / length, d[1] / length, d[2] / length];
    let flat = (way[0] * way[0] + way[1] * way[1]).sqrt();
    // Across, and level: square to the run on plan; for a riser, east.
    let across = if flat < 1.0e-9 { [1.0, 0.0, 0.0] } else { [-way[1] / flat, way[0] / flat, 0.0] };
    Some((way, across, length))
}

/// The job's building as an IFC4 file. `when` is the moment it is written, as the file's
/// own heading says it (`2026-01-31T09:30:00`), and `file` what the file is called.
pub fn export(job: &mut Job, file: &str, when: &str) -> Result<Written, String> {
    let title = job.world.get::<JobInfo>(job.root()).map(|j| j.title.clone()).unwrap_or_default();
    let elements = view::elements(job);
    let names: HashMap<String, String> = elements.iter().map(|e| (e.path.clone(), e.name.clone())).collect();
    let storeys = building::storey_totals(job);
    if storeys.is_empty() {
        return Err("the job has no level: there is no building to write a model of".into());
    }
    let (placed, not_drawn) = building::placements(job);
    let outlines = building::room_outlines(job);
    let (planes, planes_out) = roof::solid_planes(job);

    let mut o = Out { lines: Vec::new(), next: 0, job: title.clone(), body: 0 };
    let mut made: BTreeMap<String, usize> = BTreeMap::new();
    let mut count = |what: &str| *made.entry(what.to_string()).or_insert(0) += 1;
    let mut not_written: Vec<(String, usize, String)> = Vec::new();

    // ---- Units: feet, square feet, cubic feet, radians.
    let metre = o.add("IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)".into());
    let length = o.add("IFCDIMENSIONALEXPONENTS(1,0,0,0,0,0,0)".into());
    let by = o.add(format!("IFCMEASUREWITHUNIT(IFCLENGTHMEASURE(0.3048),#{metre})"));
    let foot = o.add(format!("IFCCONVERSIONBASEDUNIT(#{length},.LENGTHUNIT.,'foot',#{by})"));
    let square_metre = o.add("IFCSIUNIT(*,.AREAUNIT.,$,.SQUARE_METRE.)".into());
    let area = o.add("IFCDIMENSIONALEXPONENTS(2,0,0,0,0,0,0)".into());
    let by = o.add(format!("IFCMEASUREWITHUNIT(IFCAREAMEASURE(0.09290304),#{square_metre})"));
    let square_foot = o.add(format!("IFCCONVERSIONBASEDUNIT(#{area},.AREAUNIT.,'square foot',#{by})"));
    let cubic_metre = o.add("IFCSIUNIT(*,.VOLUMEUNIT.,$,.CUBIC_METRE.)".into());
    let volume = o.add("IFCDIMENSIONALEXPONENTS(3,0,0,0,0,0,0)".into());
    let by = o.add(format!("IFCMEASUREWITHUNIT(IFCVOLUMEMEASURE(0.028316846592),#{cubic_metre})"));
    let cubic_foot = o.add(format!("IFCCONVERSIONBASEDUNIT(#{volume},.VOLUMEUNIT.,'cubic foot',#{by})"));
    let radian = o.add("IFCSIUNIT(*,.PLANEANGLEUNIT.,$,.RADIAN.)".into());
    let units = o.add(format!("IFCUNITASSIGNMENT((#{foot},#{square_foot},#{cubic_foot},#{radian}))"));

    // ---- The project, and where shapes are.
    let world = o.axes([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let context = o.add(format!("IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-5,#{world},$)"));
    o.body = o.add(format!("IFCGEOMETRICREPRESENTATIONSUBCONTEXT('Body','Model',*,*,*,*,#{context},$,.MODEL_VIEW.,$)"));
    let project = o.add(format!("IFCPROJECT({},$,{},$,$,$,$,(#{context}),#{units})", o.id("project"), name_or_none(&title)));
    let site_at = o.placed(None, [0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let site = o.add(format!("IFCSITE({},$,'Site',$,$,#{site_at},$,$,.ELEMENT.,$,$,$,$,$)", o.id("site")));
    let building_at = o.placed(Some(site_at), [0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let whole = o.add(format!("IFCBUILDING({},$,{},$,$,#{building_at},$,$,.ELEMENT.,$,$,$)", o.id("building"), name_or_none(&title)));
    o.add(format!("IFCRELAGGREGATES({},$,$,$,#{project},(#{site}))", o.id("project has site")));
    o.add(format!("IFCRELAGGREGATES({},$,$,$,#{site},(#{whole}))", o.id("site has building")));

    // ---- Levels, lowest first.
    let mut levels: Vec<&building::StoreyTotals> = storeys.iter().collect();
    levels.sort_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0)));
    // Each level: its entity, where it is placed, and its floor's height.
    let mut level_of: HashMap<String, (u64, u64, f64)> = HashMap::new();
    let mut level_ids = Vec::new();
    let mut no_elevation = 0usize;
    for s in &levels {
        let Some(elevation) = s.elevation else {
            no_elevation += 1;
            continue;
        };
        let at = o.placed(Some(building_at), [0.0, 0.0, elevation], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let id = o.add(format!("IFCBUILDINGSTOREY({},$,{},$,$,#{at},$,$,.ELEMENT.,{})", o.id(&s.path), name_or_none(&s.name), real(elevation)));
        level_of.insert(s.path.clone(), (id, at, elevation));
        level_ids.push(id);
        count("levels");
    }
    if no_elevation > 0 {
        not_written.push(("level".into(), no_elevation, "its floor's elevation is not known, so nothing on it can be placed".into()));
    }
    if level_ids.is_empty() {
        return Err("no level of the job has an elevation: nothing can be placed".into());
    }
    o.add(format!("IFCRELAGGREGATES({},$,$,$,#{whole},({}))", o.id("building has levels"), level_ids.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));

    // What stands on each level, the parts of each level (its rooms), and what is on
    // each system, gathered as the things are written.
    let mut on_level: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut on_building: Vec<u64> = Vec::new();
    let mut rooms_of: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut on_system: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut written: Vec<(String, u64)> = Vec::new();
    let up = [0.0, 0.0, 1.0];

    for p in &placed {
        // Where it is placed from: its level's floor, or the building's zero.
        let (level, within, floor) = match p.storey.as_ref().and_then(|s| level_of.get(s)) {
            Some((id, at, elevation)) => (Some(*id), *at, *elevation),
            None => (None, building_at, 0.0),
        };
        let name = name_or_none(&p.name);
        let id = match &p.shape {
            Shape::Wall { a, b, base, height, thickness, holes } => {
                let Some((way, _, length)) = along([a[0], a[1], 0.0], [b[0], b[1], 0.0]) else {
                    not_written.push((format!("wall {}", p.name), 1, "it has no length".into()));
                    continue;
                };
                let at = o.placed(Some(within), [a[0], a[1], base - floor], up, way);
                let body = o.pushed_rect([length / 2.0, 0.0], length, *thickness, *height);
                let wall = o.add(format!("IFCWALL({},$,{name},$,$,#{at},#{body},$,.STANDARD.)", o.id(&p.path)));
                count("walls");
                // Each door and window: a hole through the wall, and the door or the
                // window as the space it takes up in it.
                for (path, kind, [from, to, sill, head]) in holes {
                    let (wide, high) = (to - from, head - sill);
                    if wide <= 0.0 || high <= 0.0 {
                        continue;
                    }
                    let hole_at = o.placed(Some(at), [*from, 0.0, *sill], up, [1.0, 0.0, 0.0]);
                    let hole_body = o.pushed_rect([wide / 2.0, 0.0], wide, *thickness, high);
                    let hole = o.add(format!("IFCOPENINGELEMENT({},$,$,$,$,#{hole_at},#{hole_body},$,.OPENING.)", o.id(&format!("{path} hole"))));
                    o.add(format!("IFCRELVOIDSELEMENT({},$,$,$,#{wall},#{hole})", o.id(&format!("{path} cuts"))));
                    let in_at = o.placed(Some(at), [*from, 0.0, *sill], up, [1.0, 0.0, 0.0]);
                    let in_body = o.pushed_rect([wide / 2.0, 0.0], wide, *thickness, high);
                    let called = name_or_none(names.get(path).map(String::as_str).unwrap_or(""));
                    let filling = match kind {
                        Kind::Door => {
                            count("doors");
                            o.add(format!("IFCDOOR({},$,{called},$,$,#{in_at},#{in_body},$,{},{},.DOOR.,.NOTDEFINED.,$)", o.id(path), real(high), real(wide)))
                        }
                        _ => {
                            count("windows");
                            o.add(format!("IFCWINDOW({},$,{called},$,$,#{in_at},#{in_body},$,{},{},.WINDOW.,.NOTDEFINED.,$)", o.id(path), real(high), real(wide)))
                        }
                    };
                    o.add(format!("IFCRELFILLSELEMENT({},$,$,$,#{hole},#{filling})", o.id(&format!("{path} fills"))));
                    match level {
                        Some(l) => on_level.entry(l).or_default().push(filling),
                        None => on_building.push(filling),
                    }
                    written.push((path.clone(), filling));
                }
                wall
            }
            Shape::Floor { boundary, top, thickness } => {
                let at = o.placed(Some(within), [0.0, 0.0, top - thickness - floor], up, [1.0, 0.0, 0.0]);
                let body = o.pushed_outline(boundary, *thickness);
                count("floors");
                o.add(format!("IFCSLAB({},$,{name},$,$,#{at},#{body},$,.FLOOR.)", o.id(&p.path)))
            }
            Shape::Run { a, b, round, rect } => {
                let Some((way, across, length)) = along(*a, *b) else {
                    not_written.push((format!("{} {}", p.kind.stem(), p.name), 1, "it has no length".into()));
                    continue;
                };
                let at = o.placed(Some(within), [a[0], a[1], a[2] - floor], way, across);
                let body = match (round, rect) {
                    (Some(d), _) => o.pushed_circle(d / 2.0, length),
                    (None, Some((width, depth))) => o.pushed_rect([0.0, 0.0], *width, *depth, length),
                    (None, None) => {
                        not_written.push((format!("{} {}", p.kind.stem(), p.name), 1, "its size is not known, so it has no shape to write".into()));
                        continue;
                    }
                };
                let (class, sort, what) = match p.kind {
                    Kind::Duct => ("IFCDUCTSEGMENT", "RIGIDSEGMENT", "ducts"),
                    Kind::Pipe => ("IFCPIPESEGMENT", "RIGIDSEGMENT", "pipes"),
                    Kind::Conduit => ("IFCCABLECARRIERSEGMENT", "CONDUITSEGMENT", "conduits"),
                    _ => ("IFCCABLESEGMENT", "CABLESEGMENT", "cables"),
                };
                count(what);
                o.add(format!("{class}({},$,{name},$,$,#{at},#{body},$,.{sort}.)", o.id(&p.path)))
            }
            Shape::Box { at, size, angle } => {
                let turn = angle.to_radians();
                let placed_at = o.placed(Some(within), [at[0], at[1], at[2] - floor], up, [turn.cos(), turn.sin(), 0.0]);
                let body = o.pushed_rect([0.0, 0.0], size[0], size[1], size[2]);
                match p.kind {
                    Kind::Fitting => {
                        let kind = job.find(&p.path).and_then(|e| job.world.get::<Fitting>(e)).map(|f| f.kind);
                        // What a fitting is called depends on what it joins.
                        let (class, sort) = match (p.discipline, kind) {
                            (Discipline::Electrical, Some(FittingKind::Bend)) => ("IFCCABLECARRIERFITTING", "BEND"),
                            (Discipline::Electrical, Some(FittingKind::Tee)) => ("IFCCABLECARRIERFITTING", "TEE"),
                            (Discipline::Electrical, Some(FittingKind::Cross)) => ("IFCCABLECARRIERFITTING", "CROSS"),
                            (Discipline::Electrical, Some(FittingKind::Transition)) => ("IFCCABLECARRIERFITTING", "REDUCER"),
                            (Discipline::Electrical, _) => ("IFCCABLECARRIERFITTING", "NOTDEFINED"),
                            (d, kind) => (
                                if d == Discipline::Plumbing { "IFCPIPEFITTING" } else { "IFCDUCTFITTING" },
                                match kind {
                                    Some(FittingKind::Bend) => "BEND",
                                    Some(FittingKind::Tee | FittingKind::Cross) => "JUNCTION",
                                    Some(FittingKind::Transition) => "TRANSITION",
                                    Some(FittingKind::Coupling) => "CONNECTOR",
                                    None => "NOTDEFINED",
                                },
                            ),
                        };
                        count("fittings");
                        o.add(format!("{class}({},$,{name},$,$,#{placed_at},#{body},$,.{sort}.)", o.id(&p.path)))
                    }
                    // Furniture and casework: IFC4's IfcFurniture.
                    Kind::Furnishing => {
                        count("furnishings");
                        // The sort of furniture, where its name says and IFC has a word for it.
                        let sort = match plumbline_world::building::forms::form_of(&p.name) {
                            Some(Form::Table) => "TABLE",
                            Some(Form::Chair) => "CHAIR",
                            Some(Form::Bed) => "BED",
                            Some(Form::Sofa) => "SOFA",
                            Some(Form::Desk) => "DESK",
                            _ => "NOTDEFINED",
                        };
                        o.add(format!("IFCFURNITURE({},$,{name},$,'furnishing',#{placed_at},#{body},$,.{sort}.)", o.id(&p.path)))
                    }
                    Kind::Fixture => {
                        count("fixtures");
                        // A plumbing fixture that IFC has a word for is written as one (J19-39):
                        // a designer's program then knows a water closet from a register.
                        match plumbline_world::building::forms::form_of(&p.name) {
                            Some(Form::Wc) => o.add(format!("IFCSANITARYTERMINAL({},$,{name},$,'fixture',#{placed_at},#{body},$,.TOILETPAN.)", o.id(&p.path))),
                            Some(Form::Washbasin) => o.add(format!("IFCSANITARYTERMINAL({},$,{name},$,'fixture',#{placed_at},#{body},$,.WASHHANDBASIN.)", o.id(&p.path))),
                            Some(Form::Bathtub) => o.add(format!("IFCSANITARYTERMINAL({},$,{name},$,'fixture',#{placed_at},#{body},$,.BATH.)", o.id(&p.path))),
                            _ => o.add(format!("IFCFLOWTERMINAL({},$,{name},$,'fixture',#{placed_at},#{body},$)", o.id(&p.path))),
                        }
                    }
                    _ => {
                        count("pieces of equipment");
                        o.add(format!("IFCDISTRIBUTIONFLOWELEMENT({},$,{name},$,'equipment',#{placed_at},#{body},$)", o.id(&p.path)))
                    }
                }
            }
        };
        match level {
            Some(l) => on_level.entry(l).or_default().push(id),
            None => on_building.push(id),
        }
        if let Some(system) = &p.system {
            on_system.entry(system.clone()).or_default().push(id);
        }
        written.push((p.path.clone(), id));
    }

    // ---- Rooms: each as the outline its floor area is the area of, as high as its
    // level is to the next. With no such height there is no body to give it.
    let mut rooms_no_walls = 0usize;
    let mut rooms_no_height = 0usize;
    for el in elements.iter().filter(|e| e.kind == Kind::Room) {
        let Some((level, within, _)) = job.find(&el.path).and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone()).and_then(|s| level_of.get(&s).copied()) else {
            continue;
        };
        let Some(ring) = outlines.get(&el.path) else {
            rooms_no_walls += 1;
            continue;
        };
        let high = storeys.iter().find(|s| level_of.get(&s.path).is_some_and(|l| l.0 == level)).and_then(|s| job.find(&s.path)).and_then(|e| job.world.get::<Level>(e)).and_then(|l| l.height.feet()).filter(|h| *h > 0.0);
        let Some(high) = high else {
            rooms_no_height += 1;
            continue;
        };
        let at = o.placed(Some(within), [0.0; 3], up, [1.0, 0.0, 0.0]);
        let body = o.pushed_outline(ring, high);
        let id = o.add(format!("IFCSPACE({},$,$,$,$,#{at},#{body},{},.ELEMENT.,.INTERNAL.,$)", o.id(&el.path), name_or_none(&el.name)));
        rooms_of.entry(level).or_default().push(id);
        written.push((el.path.clone(), id));
        count("rooms");
    }
    if rooms_no_walls > 0 {
        not_written.push(("room".into(), rooms_no_walls, "no walls close round its point, so it has no outline".into()));
    }
    if rooms_no_height > 0 {
        not_written.push(("room".into(), rooms_no_height, "its level has no floor-to-floor height, so it has no body to write".into()));
    }

    // ---- The roof: each plane as it lies, under its roof.
    let mut by_roof: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for plane in &planes {
        if plane.points.len() < 3 {
            continue;
        }
        let corners: Vec<P3> = plane.points.iter().map(|q| [q[0], q[1], plane.height_at(*q)]).collect();
        let at = o.placed(Some(building_at), [0.0; 3], up, [1.0, 0.0, 0.0]);
        let body = o.face(&corners);
        let id = o.add(format!("IFCSLAB({},$,{},$,$,#{at},#{body},$,.ROOF.)", o.id(&plane.path), name_or_none(names.get(&plane.path).map(String::as_str).unwrap_or(""))));
        let roof = job.find(&plane.path).and_then(|e| job.world.get::<Node>(e)).and_then(|n| n.parent.clone()).unwrap_or_default();
        by_roof.entry(roof).or_default().push(id);
        written.push((plane.path.clone(), id));
        count("roof planes");
    }
    for (roof, parts) in &by_roof {
        let at = o.placed(Some(building_at), [0.0; 3], up, [1.0, 0.0, 0.0]);
        let id = o.add(format!("IFCROOF({},$,{},$,$,#{at},$,$,.NOTDEFINED.)", o.id(roof), name_or_none(names.get(roof).map(String::as_str).unwrap_or("roof"))));
        o.add(format!("IFCRELAGGREGATES({},$,$,$,#{id},({}))", o.id(&format!("{roof} has planes")), parts.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
        on_building.push(id);
        count("roofs");
    }
    if !planes_out.is_empty() {
        not_written.push(("roof plane".into(), planes_out.len(), "it cannot be placed in space: a value its shape needs is not known".into()));
    }

    // ---- What stands on which level; each level's rooms.
    for (level, things) in &on_level {
        o.add(format!("IFCRELCONTAINEDINSPATIALSTRUCTURE({},$,$,$,({}),#{level})", o.id(&format!("on level {level}")), things.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
    }
    if !on_building.is_empty() {
        o.add(format!("IFCRELCONTAINEDINSPATIALSTRUCTURE({},$,$,$,({}),#{whole})", o.id("on the building"), on_building.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
    }
    for (level, rooms) in &rooms_of {
        o.add(format!("IFCRELAGGREGATES({},$,$,$,#{level},({}))", o.id(&format!("rooms of level {level}")), rooms.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
    }

    // ---- Systems, each with what is on it.
    for (path, things) in &on_system {
        let Some(info) = job.find(path).and_then(|e| job.world.get::<SystemInfo>(e)) else { continue };
        let sort = match info.classification.as_str() {
            "Supply Air" => ".VENTILATION.",
            "Exhaust Air" => ".EXHAUST.",
            "Domestic Cold Water" => ".DOMESTICCOLDWATER.",
            "Domestic Hot Water" => ".DOMESTICHOTWATER.",
            "Sanitary" => ".SEWAGE.",
            "Vent" => ".VENT.",
            "Refrigerant" => ".REFRIGERATION.",
            "Power" => ".ELECTRICAL.",
            "Lighting" => ".LIGHTING.",
            // IFC has no word of its own for it: it is written by its name.
            _ => ".USERDEFINED.",
        };
        let kind = if sort == ".USERDEFINED." { text(&info.classification) } else { "$".to_string() };
        let id = o.add(format!("IFCDISTRIBUTIONSYSTEM({},$,{},$,{kind},{},{sort})", o.id(path), name_or_none(names.get(path).map(String::as_str).unwrap_or("")), text(&info.classification)));
        o.add(format!("IFCRELASSIGNSTOGROUP({},$,$,$,({}),$,#{id})", o.id(&format!("{path} holds")), things.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")));
        count("systems");
    }

    // ---- Every value entered on a thing, with how sure it is and where it came from.
    for (path, id) in &written {
        let Some(el) = elements.iter().find(|e| e.path == *path) else { continue };
        let mut props = Vec::new();
        for row in el.rows.iter().filter(|r| !r.derived && !r.label.contains(": ")) {
            let said = format!("{} ({:?}; {})", row.text, row.status, row.basis);
            props.push(format!("#{}", o.add(format!("IFCPROPERTYSINGLEVALUE({},$,IFCTEXT({}),$)", text(&row.label), text(&said)))));
        }
        // One of an item (a thing made from a maker's sheet) carries what the sheet said
        // of it: who makes it, its model, what it weighs, each as sure as the item has
        // it and from where. The item itself is a type and is not written; without
        // this a designer opening the model gets a box with a name (J19-39).
        let item = job.find(path).and_then(|e| job.world.get::<plumbline_world::Placed>(e)).and_then(|p| p.item.clone());
        if let Some(of) = item.and_then(|item| elements.iter().find(|e| e.path == item)) {
            for row in of.rows.iter().filter(|r| !r.derived && matches!(r.label.as_str(), "maker" | "model" | "weight" | "made from")) {
                let said = format!("{} ({:?}; {})", row.text, row.status, row.basis);
                props.push(format!("#{}", o.add(format!("IFCPROPERTYSINGLEVALUE({},$,IFCTEXT({}),$)", text(&row.label), text(&said)))));
            }
        }
        if props.is_empty() {
            continue;
        }
        let set = o.add(format!("IFCPROPERTYSET({},$,'Plumbline_Values',$,({}))", o.id(&format!("{path} values")), props.join(",")));
        o.add(format!("IFCRELDEFINESBYPROPERTIES({},$,$,$,(#{id}),#{set})", o.id(&format!("{path} has values"))));
    }

    // ---- What a model has no place for, counted.
    for (path, why) in &not_drawn {
        let called = names.get(path).cloned().unwrap_or_else(|| path.clone());
        not_written.push((called, 1, format!("it has no shape: {why}")));
    }
    let lines: usize = job.order.iter().filter_map(|e| job.world.get::<Cost>(*e)).map(|c| c.lines.len()).sum();
    if lines > 0 {
        not_written.push(("cost line".into(), lines, "the estimate is not part of a building model: write it out with `export estimate`".into()));
    }
    for (kind, what, why) in [
        (Kind::Material, "material", "its price and how it is ordered are the estimate's"),
        (Kind::Condition, "condition", "its recipe of cost lines is the estimate's"),
        (Kind::Drawing, "drawing", "a plan laid under a level is kept as where its file is, which another program cannot follow"),
        (Kind::Prior, "estimate made before", "it is kept beside the job, and is no part of the building"),
        (Kind::Type, "type", "each thing is written with its own size; the type it took it from is named among its values"),
    ] {
        let n = elements.iter().filter(|e| e.kind == kind).count();
        if n > 0 {
            not_written.push((what.to_string(), n, why.to_string()));
        }
    }

    let mut out = String::new();
    out.push_str("ISO-10303-21;\nHEADER;\n");
    out.push_str("FILE_DESCRIPTION(('ViewDefinition [ReferenceView_V1.2]'),'2;1');\n");
    out.push_str(&format!("FILE_NAME({},{},(''),(''),'plumbline','plumbline','');\n", text(file), text(when)));
    out.push_str("FILE_SCHEMA(('IFC4'));\nENDSEC;\nDATA;\n");
    for line in &o.lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
    Ok(Written { text: out, made, not_written })
}
