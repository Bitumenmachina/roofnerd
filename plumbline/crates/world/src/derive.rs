//! Working quantities out of geometry.
//!
//! A roof plan is flat; the roof is not. A plane's true area is its plan area times a
//! slope factor that comes from its pitch. A hip's true length is longer than the line
//! drawn on the plan, by however much the hip climbs. This module does that arithmetic
//! for every element, writes the result into the element's `pl::measure`, and reports
//! which elements' measures actually changed.
//!
//! Three rules hold throughout:
//! * a result is only as good as its worst input (see `value.rs`);
//! * every element of a geometric kind gets every one of its quantities, always. If an
//!   input is missing the quantity is still listed, as Unknown, with the reason. A
//!   quantity never just disappears;
//! * when the inputs do not fit together, no number is given. A hip sits on two planes,
//!   and it has to run along both outlines and be at the same height on both, at both
//!   ends. An eave, a ridge and a headwall are level; a rake, a hip and a valley climb.
//!   If not, the line is flagged with the figures instead of quietly taking one.
//!
//! All arithmetic here is in feet. An entered value keeps the unit it was typed in and is
//! converted on the way in (`points_ft`, `ends_ft`, `feet`); the formula written beside
//! each result says where a conversion happened.

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;

use crate::check::{self, FIT_TOLERANCE_FT};
use crate::components::*;
use crate::geom;
use crate::lines;
use crate::units;
use crate::value::{Derived, Status};

const NOT_CONVERTED: &str = "is in a unit this program does not convert";

/// The words added to a formula when an input was entered in inches.
fn entered_in(unit: &str, divisor: &str) -> String {
    if unit == units::IN {
        format!(" ÷ {divisor} (entered in inches)")
    } else {
        String::new()
    }
}

/// What a line needs to know about an element it names as one of its planes.
struct PlaneGeom {
    kind: Kind,
    points: Option<Vec<P2>>,
    /// The outline edge the eave lies along. Heights are measured from this edge's line.
    eave: Option<[P2; 2]>,
    pitch: Option<f64>,
    eave_height: Option<f64>,
    /// The worst status among the plane's entered values; Unknown if any is missing.
    status: Status,
    /// What the plane lacks, in words, if anything.
    lacks: Option<&'static str>,
    /// Was any of its geometry entered in something other than feet?
    converted: bool,
}

impl PlaneGeom {
    /// The height of the plane above a plan point, in feet: the eave height plus the climb
    /// from the eave line to that point.
    fn height_at(&self, p: P2) -> Option<f64> {
        let [a, b] = self.eave?;
        let off = geom::point_to_line(p, a, b)?;
        Some(self.eave_height? + off * self.pitch? / 12.0)
    }
}

/// Recompute every element's measure. Returns the paths whose measure changed.
pub fn derive(world: &mut World) -> Vec<String> {
    let mut planes: HashMap<String, PlaneGeom> = HashMap::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Outline>, Option<&Pitch>)>();
    for (node, kind, outline, pitch) in q.iter(world) {
        let mut status = Status::Known;
        let mut lacks = None;
        match outline {
            Some(o) => status = status.max(o.points.status).max(o.eave.status).max(o.eave_height.status),
            None => lacks = Some("no outline entered"),
        }
        match pitch {
            Some(p) => status = status.max(p.0.status),
            None => lacks = lacks.or(Some("no pitch entered")),
        }
        // The eave as the outline edge it lies along. A plane built in code without the
        // entry checks may have an eave that fits no edge, or a unit this program does not
        // convert; it then gives no heights.
        let points = outline.and_then(|o| o.points_ft());
        let mut eave = None;
        let mut converted = false;
        if let Some(o) = outline {
            converted = [&o.points.unit, &o.eave.unit, &o.eave_height.unit].iter().any(|u| *u != units::FT);
            if o.points.value.is_some() && points.is_none() {
                lacks = lacks.or(Some("no outline in a unit this program converts"));
            }
            match (points.as_ref(), o.eave_ft()) {
                (Some(pts), Some(line)) => match check::eave_edge(pts, line) {
                    Ok(edge) => eave = Some(edge),
                    Err(_) => lacks = lacks.or(Some("no eave line that fits its outline")),
                },
                (Some(_), None) if o.eave.value.is_some() => lacks = lacks.or(Some("no eave line in a unit this program converts")),
                _ => {}
            }
            if o.eave_height.value.is_some() && o.eave_height.feet().is_none() {
                lacks = lacks.or(Some("no eave height in a unit this program converts"));
            }
        }
        if lacks.is_some() {
            status = Status::Unknown;
        }
        planes.insert(
            node.path.clone(),
            PlaneGeom {
                kind: *kind,
                points,
                eave,
                pitch: pitch.and_then(|p| p.0.value),
                eave_height: outline.and_then(|o| o.eave_height.feet()),
                status,
                lacks,
                converted,
            },
        );
    }

    let mut q_ids = world.query::<(Entity, &Node)>();
    let ids: HashMap<String, Entity> = q_ids.iter(world).map(|(e, n)| (n.path.clone(), e)).collect();

    // Lines first. Each line's length, and the plane (if any) that the line shows to have
    // its eave on the wrong edge: an eave that climbs on it, a rake that runs level. One
    // such line is evidence about the whole plane, so every other line on that plane
    // loses its length too. A rake can climb under both readings of the plane and would
    // otherwise keep a wrong length.
    let mut lengths: HashMap<String, Derived> = HashMap::new();
    let mut wrong_eave: BTreeMap<String, String> = BTreeMap::new();
    let mut q_lines = world.query::<(&Node, &Kind, &Run)>();
    for (node, kind, run) in q_lines.iter(world) {
        if !kind.is_run() || *kind == Kind::Parapet {
            continue;
        }
        let plan = run.ends_ft().map(|[a, b]| geom::dist(a, b));
        let mut shows = Vec::new();
        lengths.insert(node.path.clone(), true_length(*kind, Some(run), plan, &planes, &format!("{}#{K_RUN}", node.path), &mut shows));
        for plane in shows {
            let first = wrong_eave.entry(plane).or_insert_with(|| node.path.clone());
            if node.path < *first {
                *first = node.path.clone();
            }
        }
    }
    let mut q_lines = world.query::<(&Node, &Kind, &Run)>();
    for (node, kind, run) in q_lines.iter(world) {
        if !kind.is_run() || *kind == Kind::Parapet {
            continue;
        }
        let Some((plane, witness)) = run.planes.iter().find_map(|p| wrong_eave.get(p).filter(|w| **w != node.path).map(|w| (p, w))) else { continue };
        let Some(d) = lengths.get_mut(&node.path) else { continue };
        // A line refused for a reason of its own keeps that reason.
        if d.value.is_some() {
            *d = Derived::refused("LF", &d.formula.clone(), &d.inputs.clone(), &format!("plane {plane} has its eave entered on the wrong edge, by the evidence of {witness}; no length on that plane is given until its eave is settled"));
        }
    }

    // What each roof comes to, from its planes and its lines (call 134): its surface and
    // squares, and the length of its eaves, rakes, hips, ridges, valleys and headwalls,
    // so that a roof is priced as a roofer prices it, once and whole, and not plane by
    // plane. Each plane's surface by the same arithmetic as the plane's own
    // (`plane_measures`); each line's length is the one worked above.
    let mut roof_figures: HashMap<String, BTreeMap<String, Derived>> = HashMap::new();
    // Where a priced line names the rafters ("Rafters, 2x8 at 24 in"), on a roof or on one of
    // its planes: the spacing it names (call 165). Nothing is counted where nothing is named.
    let rafters_named: HashMap<String, f64> = {
        let mut nq = world.query::<(&Node, &Kind, Option<&Cost>, Option<&MadeOf>, Option<&Condition>)>();
        nq.iter(world)
            .filter(|(_, k, ..)| matches!(k, Kind::Roof | Kind::RoofPlane))
            .filter_map(|(n, _, cost, made_of, condition)| crate::finish::members_named(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str), &["rafter"]).map(|(_, spacing)| (n.path.clone(), spacing)))
            .collect()
    };
    let mut plane_rafters: HashMap<String, BTreeMap<String, Derived>> = HashMap::new();
    {
        let mut planes_of: BTreeMap<String, Vec<(String, Derived)>> = BTreeMap::new();
        let mut lines_of: BTreeMap<(String, &'static str), Vec<(String, Derived)>> = BTreeMap::new();
        let mut roofs: Vec<String> = Vec::new();
        let mut rq = world.query::<(&Node, &Kind, Option<&Outline>, Option<&Pitch>)>();
        for (node, kind, outline, pitch) in rq.iter(world) {
            if *kind == Kind::Roof {
                roofs.push(node.path.clone());
            } else if let Some(parent) = &node.parent {
                if kind.is_area() {
                    let surface = plane_measures(&node.path, outline, pitch).remove("surface_area").expect("a plane has a surface figure");
                    planes_of.entry(parent.clone()).or_default().push((node.path.clone(), surface));
                    // Its rafters, where its own lines or its roof's name them.
                    if *kind == Kind::RoofPlane {
                        if let Some(spacing) = rafters_named.get(&node.path).or_else(|| rafters_named.get(parent)) {
                            plane_rafters.insert(node.path.clone(), rafter_figures(&node.path, outline, pitch, *spacing));
                        }
                    }
                } else if let Some((_, name)) = ROOF_LINES.iter().find(|(k, _)| k == kind) {
                    if let Some(d) = lengths.get(&node.path) {
                        lines_of.entry((parent.clone(), *name)).or_default().push((node.path.clone(), d.clone()));
                    }
                }
            }
        }
        // In the order of their paths, not the order the world happens to hold them in:
        // a job read back from its file holds them in another (its figures have to read
        // back the same, inputs and all).
        for list in planes_of.values_mut().chain(lines_of.values_mut()) {
            list.sort_by(|a, b| a.0.cmp(&b.0));
        }
        for roof in roofs {
            let planes = planes_of.remove(&roof).unwrap_or_default();
            let mut figures = roof_measures(&planes, &mut lines_of, &roof);
            // The roof's own rafters, where its own lines name them: its planes' added up.
            if rafters_named.contains_key(&roof) {
                let mine: Vec<(String, &BTreeMap<String, Derived>)> = planes.iter().filter_map(|(p, _)| plane_rafters.get(p).map(|f| (p.clone(), f))).collect();
                figures.extend(roof_rafters(&mine, planes.len()));
            }
            roof_figures.insert(roof.clone(), figures);
        }
    }

    // The rest of the building: walls, floors, ducts, pipes. Worked out in `building.rs`
    // and added to each element's measure below.
    let mut building = crate::building::measures(world);

    // An eave's or a rake's overhang, where a line of it names it (call 185).
    let overhangs = {
        let wanted = crate::trim::naming(world, &[Kind::Eave, Kind::Rake], &[crate::trim::OVERHANG]);
        if wanted.is_empty() { HashMap::new() } else { crate::trim::overhangs(world, Some(&wanted)) }
    };

    let mut changed = Vec::new();
    let mut to_insert: Vec<(Entity, Option<Measure>, Option<LineQuantities>)> = Vec::new();
    let mut to_remove: Vec<Entity> = Vec::new();
    // What each element comes to in money, set on it after the loop.
    let mut priced_now: Vec<(Entity, String, Priced)> = Vec::new();
    // The job's materials, by name: what a line made of its element's material is ordered
    // and priced by.
    let mut materials: Vec<(String, String, Supply)> = Vec::new();
    let mut mq = world.query::<(&Node, &Kind, &Supply)>();
    for (node, kind, supply) in mq.iter(world) {
        // A material, or an item that has its own price (call 196).
        if matches!(*kind, Kind::Material | Kind::Type) {
            materials.push((node.name.clone(), node.path.clone(), supply.clone()));
        }
    }
    // What each item's price list entry is called, by the item's path: a line made of the element's material, on a
    // thing set down from an item, is made of its item (call 196).
    let mut item_material: HashMap<String, String> = HashMap::new();
    let mut iq = world.query::<(&Node, &crate::Item)>();
    for (node, _) in iq.iter(world) {
        item_material.insert(node.path.clone(), crate::editing::item_material_name(&node.name));
    }
    #[allow(clippy::type_complexity)]
    let mut q = world.query::<(&Node, &Kind, Option<&Outline>, Option<&Pitch>, Option<&Taper>, Option<&Run>, Option<&Height>, Option<&Cost>, Option<&Allowance>, Option<&MadeOf>, Option<&Condition>, Option<&crate::Placed>, Option<&mut Measure>, Option<&mut LineQuantities>)>();
    for (node, kind, outline, pitch, taper, run, height, cost, allowance, made_of, condition, placed, measure, line_quantities) in q.iter_mut(world) {
        let me = &node.path;
        let mut m = BTreeMap::new();

        if kind.is_area() {
            m.extend(plane_measures(me, outline, pitch));
        }
        if *kind == Kind::LowSlopeArea {
            m.extend(crate::taper::measures(me, outline, pitch, taper));
        }

        if kind.is_run() {
            let in_run = format!("{me}#{K_RUN}");
            let f_plan = format!("length of the line on the plan{}", run.map(|r| entered_in(&r.ends.unit, "12")).unwrap_or_default());
            let ends = run.and_then(|r| r.ends_ft());
            let plan = ends.map(|[a, b]| geom::dist(a, b));
            let unit_fault = run.is_some_and(|r| r.ends.value.is_some() && ends.is_none());
            m.insert(
                "plan_length".into(),
                match run {
                    Some(_) if unit_fault => Derived::refused("LF", &f_plan, std::slice::from_ref(&in_run), &format!("the line {NOT_CONVERTED}")),
                    Some(r) => Derived::new(plan, "LF", &[r.ends.status], &f_plan, std::slice::from_ref(&in_run)),
                    None => Derived::refused("LF", &f_plan, std::slice::from_ref(&in_run), "no line entered"),
                },
            );
            if *kind == Kind::Parapet {
                let in_h = format!("{me}#{K_HEIGHT}");
                // The formula shows the conversion: a height in inches is ÷ 12.
                let f_face = format!("plan_length × {}", units::to_feet_text("height", height.map(|h| h.0.unit.as_str()).unwrap_or(units::FT)));
                let inputs = [in_run.clone(), in_h];
                m.insert(
                    "face_area".into(),
                    match (run, height) {
                        (Some(_), Some(_)) if unit_fault => Derived::refused("SF", &f_face, &inputs, &format!("the line {NOT_CONVERTED}")),
                        (Some(_), Some(h)) if h.0.value.is_some() && h.0.feet().is_none() => Derived::refused("SF", &f_face, &inputs, &format!("the height {NOT_CONVERTED}")),
                        (Some(r), Some(h)) => Derived::new(plan.zip(h.0.feet()).map(|(l, h)| l * h), "SF", &[r.ends.status, h.0.status], &f_face, &inputs),
                        (None, _) => Derived::refused("SF", &f_face, &inputs, "no line entered"),
                        (_, None) => Derived::refused("SF", &f_face, &inputs, "no height entered"),
                    },
                );
            } else {
                let d = lengths.remove(me).unwrap_or_else(|| true_length(*kind, run, plan, &planes, &in_run, &mut Vec::new()));
                m.insert("true_length".into(), d);
                if let Some(o) = overhangs.get(me) {
                    m.insert(crate::trim::OVERHANG.into(), o.clone());
                }
            }
        }

        if let Some(more) = building.remove(me) {
            m.extend(more);
        }
        if let Some(more) = roof_figures.remove(me) {
            m.extend(more);
        }
        if let Some(more) = plane_rafters.remove(me) {
            m.extend(more);
        }

        // Every cost line, worked out against this element's figures: quantity, then
        // waste, then order units.
        let new = Measure(m);
        let mut lq = BTreeMap::new();
        if let Some(cost) = cost {
            let on = lines::On { kind: *kind, measure: Some(&new), height };
            // What the element is made of: what it says; failing that, if it is one of an item set down, its item
            // (a wall, a floor or a roof is none of them and is as it was).
            let of_its_item = placed.and_then(|p| p.item.as_ref().map(|i| (i, p.what.status)));
            let stock = match made_of.map(|m| &m.0) {
                None => match of_its_item.and_then(|(item, status)| item_material.get(item).map(|name| (name, status))) {
                    None => lines::Stock::NotSaid,
                    Some((name, status)) => {
                        let found: Vec<&(String, String, Supply)> = materials.iter().filter(|(n, _, _)| crate::check::same_name(n, name)).collect();
                        match found.as_slice() {
                            [(name, path, supply)] => lines::Stock::Found { path, name, status, supply },
                            [] => lines::Stock::Missing(format!("it is priced from the price list under {name:?}, and the price list has no price for that item yet")),
                            many => lines::Stock::Missing(format!("it is priced from the price list as {name:?}, and the job has {} materials of that name ({}); use the path", many.len(), many.iter().map(|f| f.1.clone()).collect::<Vec<_>>().join(", "))),
                        }
                    }
                },
                Some(m) => match &m.value {
                    None => lines::Stock::Missing(format!("what the element is made of is not known ({})", m.source)),
                    Some(name) => {
                        // By its path, or by its name as names are compared everywhere else (capitals,
                        // spacing and a full stop aside): a job file that named it "copper, 16 OZ"
                        // priced nothing while every other road took it (M21-12).
                        let found: Vec<&(String, String, Supply)> = materials.iter().filter(|(n, p, _)| p == name || crate::check::same_name(n, name)).collect();
                        match found.as_slice() {
                            [(name, path, supply)] => lines::Stock::Found { path, name, status: m.status, supply },
                            [] => lines::Stock::Missing(format!("the element is made of {name:?}, and the job has no material of that name")),
                            many => lines::Stock::Missing(format!("the element is made of {name:?}, and the job has {} materials of that name ({}); use the path", many.len(), many.iter().map(|f| f.1.clone()).collect::<Vec<_>>().join(", "))),
                        }
                    }
                },
            };
            for line in &cost.lines {
                lq.insert(line.description.clone(), lines::figures(me, &on, line, &stock, condition.map(|c| &c.0)));
            }
        }
        let new_lq = LineQuantities(lq);
        // What it comes to: from its allowance, from its lines, and the two compared.
        priced_now.push((ids[&node.path], node.path.clone(), Priced(lines::priced(me, *kind, &new, allowance, cost, &new_lq.0))));
        let mut moved = false;
        let mut insert_m = None;
        let mut insert_lq = None;
        match measure {
            Some(mut old) => {
                // What is written decides whether it moved; the figure kept for formulas
                // is always the one just worked (call 133: a figure read from a file has
                // none until it is worked again).
                if *old != new {
                    moved = true;
                }
                *old = new;
            }
            None if !new.0.is_empty() => {
                insert_m = Some(new);
                moved = true;
            }
            None => {}
        }
        match line_quantities {
            Some(mut old) => {
                if new_lq.0.is_empty() {
                    to_remove.push(ids[&node.path]);
                    moved = true;
                } else if *old != new_lq {
                    *old = new_lq;
                    moved = true;
                }
            }
            None if !new_lq.0.is_empty() => {
                insert_lq = Some(new_lq);
                moved = true;
            }
            None => {}
        }
        if insert_m.is_some() || insert_lq.is_some() {
            to_insert.push((ids[&node.path], insert_m, insert_lq));
        }
        if moved {
            changed.push(node.path.clone());
        }
    }
    for (e, m, lq) in to_insert {
        if let Some(m) = m {
            world.entity_mut(e).insert(m);
        }
        if let Some(lq) = lq {
            world.entity_mut(e).insert(lq);
        }
    }
    for e in to_remove {
        world.entity_mut(e).remove::<LineQuantities>();
    }
    for (e, path, new) in priced_now {
        let old = world.get::<Priced>(e);
        let moved = match (old, new.0.is_empty()) {
            (None, true) => false,
            (Some(old), false) if *old == new => false,
            (Some(_), true) => {
                world.entity_mut(e).remove::<Priced>();
                true
            }
            (_, false) => {
                world.entity_mut(e).insert(new);
                true
            }
        };
        if moved && !changed.contains(&path) {
            changed.push(path);
        }
    }
    changed
}

/// The lines a roof's lengths are added up from, and the figure each kind adds up to.
const ROOF_LINES: [(Kind, &str); 6] = [(Kind::Eave, "eave_length"), (Kind::Rake, "rake_length"), (Kind::Hip, "hip_length"), (Kind::Ridge, "ridge_length"), (Kind::Valley, "valley_length"), (Kind::Headwall, "headwall_length")];

/// A roof's figures from its planes' surfaces and its lines' lengths (call 134). A sum
/// with a member that has no figure has no figure, and says which; a roof with no
/// planes has no surface. A roof with no line of a kind (no valleys on a hip roof) has
/// none of that length: 0, as sure as its planes, and its formula says why.
fn roof_measures(planes: &[(String, Derived)], lines_of: &mut BTreeMap<(String, &'static str), Vec<(String, Derived)>>, roof: &str) -> BTreeMap<String, Derived> {
    let mut m = BTreeMap::new();
    let inputs: Vec<String> = planes.iter().map(|(p, _)| format!("{p}#{K_MEASURE}")).collect();
    let f_surface = format!("the surface of its {} on the slope, added up", if planes.len() == 1 { "one plane".to_string() } else { format!("{} planes", planes.len()) });
    let planes_status = planes.iter().map(|(_, d)| d.status).max().unwrap_or(Status::Unknown);
    let surface = if planes.is_empty() {
        Derived::refused("SF", &f_surface, &inputs, "the roof has no planes")
    } else if let Some((p, d)) = planes.iter().find(|(_, d)| d.value.is_none()) {
        Derived::refused("SF", &f_surface, &inputs, &format!("{p} has no surface{}", if d.note.is_empty() { String::new() } else { format!(": {}", d.note) }))
    } else {
        Derived::new(Some(planes.iter().filter_map(|(_, d)| d.worked()).sum()), "SF", &planes.iter().map(|(_, d)| d.status).collect::<Vec<_>>(), &f_surface, &inputs)
    };
    let squares = match surface.worked() {
        Some(v) => Derived::new(Some(v / 100.0), "SQ", &[surface.status], "surface_area ÷ 100", &inputs),
        None => Derived::refused("SQ", "surface_area ÷ 100", &inputs, if surface.note.is_empty() { "the roof has no surface" } else { &surface.note }),
    };
    m.insert("surface_area".into(), surface);
    m.insert("squares".into(), squares);
    for (kind, name) in ROOF_LINES {
        let lines = lines_of.remove(&(roof.to_string(), name)).unwrap_or_default();
        let said = kind.stem();
        let inputs: Vec<String> = lines.iter().map(|(p, _)| format!("{p}#{K_MEASURE}")).collect();
        let d = if lines.is_empty() {
            if planes.is_empty() {
                Derived::refused("LF", &format!("the true length of its {said} lines, added up"), &inputs, "the roof has no planes")
            } else {
                Derived::new(Some(0.0), "LF", &[planes_status], &format!("the roof has no {said} line"), &inputs)
            }
        } else if let Some((p, d)) = lines.iter().find(|(_, d)| d.value.is_none()) {
            Derived::refused("LF", &format!("the true length of its {said} lines, added up"), &inputs, &format!("{p} has no length{}", if d.note.is_empty() { String::new() } else { format!(": {}", d.note) }))
        } else {
            let f = if lines.len() == 1 { format!("the true length of its one {said} line") } else { format!("the true length of its {} {said} lines, added up", lines.len()) };
            Derived::new(Some(lines.iter().filter_map(|(_, d)| d.worked()).sum()), "LF", &lines.iter().map(|(_, d)| d.status).collect::<Vec<_>>(), &f, &inputs)
        };
        m.insert(name.to_string(), d);
    }
    m
}

/// Which way a plane's rafters run: its outline on the plan, the way up the plane (square to
/// its eave, toward where the plane lies) and its rise in 12. One rule, for what is counted
/// and what is drawn (call 177).
pub(crate) fn rafter_layout(outline: Option<&Outline>, pitch: Option<&Pitch>) -> Result<(Vec<P2>, P2, f64), String> {
    let (Some(o), Some(p)) = (outline, pitch) else { return Err((if outline.is_none() { "no outline entered" } else { "no pitch entered" }).into()) };
    let Some(ring) = o.points_ft() else { return Err(format!("the outline {NOT_CONVERTED}")) };
    let Some(rise) = p.0.value else { return Err("no pitch entered: the rafters' length is along the slope".into()) };
    let Some(eave) = o.eave_ft().and_then(|line| check::eave_edge(&ring, line).ok()) else { return Err("it has no eave: the rafters run up the plane from its eave".into()) };
    // Up the plane: square to the eave, toward where the plane lies.
    let (ex, ey) = (eave[1][0] - eave[0][0], eave[1][1] - eave[0][1]);
    let middle = [ring.iter().map(|q| q[0]).sum::<f64>() / ring.len() as f64, ring.iter().map(|q| q[1]).sum::<f64>() / ring.len() as f64];
    let toward = (middle[0] - eave[0][0]) * -ey + (middle[1] - eave[0][1]) * ex;
    let up = if toward >= 0.0 { [-ey, ex] } else { [ey, -ex] };
    Ok((ring, up, rise))
}

/// A plane's rafters counted (call 165): `rafters`, `rafter_length` (along the slope: each
/// piece's length on plan times the plane's slope factor, its overhang being in the outline as
/// drawn, so a rafter's tail is in its length) and `rafter_longest`. They run up the plane
/// square to its eave, on marks along the eave (`building::members`).
fn rafter_figures(me: &str, outline: Option<&Outline>, pitch: Option<&Pitch>, spacing: f64) -> BTreeMap<String, Derived> {
    let names = [
        ("rafters", "EA", "one rafter at each multiple of the spacing along the eave from its first end, and one inside each rake, each cut to the outline"),
        ("rafter_length", "LF", "the lengths of its rafters on the slope, added: each one's length on the plan × slope_factor"),
        ("rafter_longest", "ft", "the longest of its rafters on the slope: the stock they are bought in is that length or the next up"),
    ];
    let mut inputs = vec![format!("{me}#{K_OUTLINE}"), format!("{me}#{K_PITCH}")];
    inputs.push(format!("{me}#{K_COST}"));
    let refused = |why: &str| names.iter().map(|(n, u, f)| (n.to_string(), Derived::refused(u, f, &inputs, why))).collect::<BTreeMap<_, _>>();
    let (ring, up, rise) = match rafter_layout(outline, pitch) {
        Ok(laid) => laid,
        Err(why) => return refused(&why),
    };
    let (Some(o), Some(p)) = (outline, pitch) else { return refused("no outline entered") };
    let factor = (1.0 + (rise / 12.0).powi(2)).sqrt();
    match crate::building::members::members_across(&ring, up, spacing) {
        Err(why) => refused(&why),
        Ok(across) => {
            let mut note = Vec::new();
            if across.left_out > 0 {
                note.push(format!("{} piece{} under 6 in left out", across.left_out, if across.left_out == 1 { "" } else { "s" }));
            }
            note.push(crate::building::NOT_COUNTED.to_string());
            let status = [o.points.status, o.eave.status, p.0.status];
            let figures = [across.count() as f64, across.total() * factor, across.longest() * factor];
            names
                .iter()
                .zip(figures)
                .map(|((n, u, f), v)| {
                    let mut d = Derived::new(Some(v), u, &status, f, &inputs);
                    d.note = note.join("; ");
                    (n.to_string(), d)
                })
                .collect()
        }
    }
}

/// A roof's rafters: its planes' added up, the longest of them. A roof with a plane that has
/// no rafters has none, and says which plane and why.
fn roof_rafters(planes: &[(String, &BTreeMap<String, Derived>)], of: usize) -> BTreeMap<String, Derived> {
    let inputs: Vec<String> = planes.iter().map(|(p, _)| format!("{p}#{K_MEASURE}")).collect();
    let mut m = BTreeMap::new();
    for (name, unit, longest) in [("rafters", "EA", false), ("rafter_length", "LF", false), ("rafter_longest", "ft", true)] {
        let f = if longest { "the longest of its planes' rafters" } else { "its planes' rafters, added up" };
        let d = if planes.is_empty() || planes.len() < of {
            Derived::refused(unit, f, &inputs, if planes.is_empty() { "the roof has no planes" } else { "a plane of the roof has no rafters counted" })
        } else if let Some((p, d)) = planes.iter().filter_map(|(p, f)| f.get(name).map(|d| (p, d))).find(|(_, d)| d.value.is_none()) {
            Derived::refused(unit, f, &inputs, &format!("{p} has no {}{}", name.replace('_', " "), if d.note.is_empty() { String::new() } else { format!(": {}", d.note) }))
        } else {
            let values = planes.iter().filter_map(|(_, f)| f.get(name)).filter_map(|d| d.worked());
            let value = if longest { values.fold(0.0, f64::max) } else { values.sum() };
            let status: Vec<Status> = planes.iter().filter_map(|(_, f)| f.get(name)).map(|d| d.status).collect();
            let mut d = Derived::new(Some(value), unit, &status, f, &inputs);
            d.note = crate::building::NOT_COUNTED.to_string();
            d
        };
        m.insert(name.to_string(), d);
    }
    m
}

/// A roof plane's figures: its area on the plan, its slope factor, its surface on the
/// slope and that surface in squares. One arithmetic, used for the plane itself and for
/// the roof its surface adds up to (call 134).
fn plane_measures(me: &str, outline: Option<&Outline>, pitch: Option<&Pitch>) -> BTreeMap<String, Derived> {
    let mut m = BTreeMap::new();
    let in_outline = format!("{me}#{K_OUTLINE}");
    let in_pitch = format!("{me}#{K_PITCH}");
    let both = [in_outline.clone(), in_pitch.clone()];
    let f_area = format!("area of the outline on the plan{}", outline.map(|o| entered_in(&o.points.unit, "144")).unwrap_or_default());
    let f_factor = "√(1 + (pitch ÷ 12)²)";
    let f_surface = "plan_area × slope_factor";
    let f_squares = "surface_area ÷ 100";
    let pts = outline.and_then(|o| o.points_ft());
    let area = pts.as_ref().map(|p| geom::plan_area(p));
    let factor = pitch.and_then(|p| p.0.value.map(|p| (1.0 + (p / 12.0).powi(2)).sqrt()));
    let surface = area.zip(factor).map(|(a, f)| a * f);
    let unit_fault = outline.is_some_and(|o| o.points.value.is_some() && pts.is_none());
    m.insert(
        "plan_area".into(),
        match outline {
            Some(_) if unit_fault => Derived::refused("SF", &f_area, std::slice::from_ref(&in_outline), &format!("the outline {NOT_CONVERTED}")),
            Some(o) => Derived::new(area, "SF", &[o.points.status], &f_area, std::slice::from_ref(&in_outline)),
            None => Derived::refused("SF", &f_area, std::slice::from_ref(&in_outline), "no outline entered"),
        },
    );
    m.insert(
        "slope_factor".into(),
        match pitch {
            Some(p) => Derived::new(factor, "", &[p.0.status], f_factor, std::slice::from_ref(&in_pitch)),
            None => Derived::refused("", f_factor, std::slice::from_ref(&in_pitch), "no pitch entered"),
        },
    );
    let (surface_d, squares_d) = match (outline, pitch) {
        (Some(_), Some(_)) if unit_fault => {
            let why = format!("the outline {NOT_CONVERTED}");
            (Derived::refused("SF", f_surface, &both, &why), Derived::refused("SQ", f_squares, &both, &why))
        }
        (Some(o), Some(p)) => {
            let s = [o.points.status, p.0.status];
            (Derived::new(surface, "SF", &s, f_surface, &both), Derived::new(surface.map(|s| s / 100.0), "SQ", &s, f_squares, &both))
        }
        _ => {
            let why = if outline.is_none() { "no outline entered" } else { "no pitch entered" };
            (Derived::refused("SF", f_surface, &both, why), Derived::refused("SQ", f_squares, &both, why))
        }
    };
    m.insert("surface_area".into(), surface_d);
    m.insert("squares".into(), squares_d);
    m
}

/// The length of a line along the slope. The line has to fit every plane it names: on the
/// plane's outline at both ends, and at the same height on every plane at both ends.
/// `shows_wrong_eave` gains the path of every plane on which this line's kind and its
/// slope disagree, which is evidence that plane's eave is on the wrong edge.
fn true_length(kind: Kind, run: Option<&Run>, plan: Option<f64>, planes: &HashMap<String, PlaneGeom>, in_run: &str, shows_wrong_eave: &mut Vec<String>) -> Derived {
    let base = "√(plan_length² + rise²); rise = the difference in height between the two ends, each height = eave height + pitch ÷ 12 × distance from the plane's eave line";
    let mut inputs = vec![in_run.to_string()];
    let Some(run) = run else {
        return Derived::refused("LF", base, &inputs, "no line entered");
    };
    // Say so when anything feeding this was entered in inches rather than feet.
    let converted = run.ends.unit != units::FT || run.planes.iter().any(|p| planes.get(p).is_some_and(|g| g.converted));
    let formula = &if converted { format!("{base}; lengths not entered in feet are converted to feet first (inches ÷ 12)") } else { base.to_string() };
    if run.ends.value.is_some() && run.ends_ft().is_none() {
        return Derived::refused("LF", formula, &inputs, &format!("the line {NOT_CONVERTED}"));
    }
    for p in &run.planes {
        inputs.push(format!("{p}#{K_OUTLINE}"));
        inputs.push(format!("{p}#{K_PITCH}"));
    }
    let refuse = |why: String| Derived::refused("LF", formula, &inputs, &why);

    // Nothing below returns until every plane the line names has been looked at. The
    // first thing wrong with the line is what it will say, but a line that is refused for
    // one plane is still evidence about another, and that evidence must not be lost.
    let mut first_fault: Option<String> = None;
    let (fewest, most) = kind.planes_allowed();
    let n = run.planes.len();
    if n < fewest || n > most {
        let want = if fewest == most { format!("{fewest} plane{}", if fewest == 1 { "" } else { "s" }) } else { format!("{fewest} or {most} planes") };
        first_fault.get_or_insert(format!("{kind:#} lies on {want}; this one names {n}"));
    }
    let mut statuses = vec![run.ends.status];
    // For each plane: its path and the height it gives each end of the line.
    let mut heights: Vec<(&String, [f64; 2])> = Vec::new();
    let mut seen: Vec<&String> = Vec::new();
    for p in &run.planes {
        if seen.contains(&p) {
            first_fault.get_or_insert(format!("names plane {p} twice"));
            continue;
        }
        seen.push(p);
        let Some(g) = planes.get(p) else {
            first_fault.get_or_insert(format!("names plane {p}, which is not in the job"));
            continue;
        };
        if !g.kind.is_area() {
            first_fault.get_or_insert(format!("names {p} as a plane, but {p} is a {:?}", g.kind));
            continue;
        }
        if let Some(lacks) = g.lacks {
            first_fault.get_or_insert(format!("plane {p} has {lacks}"));
            continue;
        }
        statuses.push(g.status);
        let (Some([a, b]), Some(pts)) = (run.ends_ft(), g.points.as_ref()) else { continue };
        if let Err((t, off)) = geom::runs_along_outline(a, b, pts, FIT_TOLERANCE_FT) {
            // A line that is not on this plane's outline says nothing about this plane.
            let whereabouts = if t == 0.0 { "its start is".to_string() } else if t == 1.0 { "its end is".to_string() } else { format!("{:.0}% of the way along it is", t * 100.0) };
            first_fault.get_or_insert(format!("the line does not run along the outline of plane {p}: {whereabouts} {off:.4} ft off it"));
            continue;
        }
        if let (Some(za), Some(zb)) = (g.height_at(a), g.height_at(b)) {
            heights.push((p, [za, zb]));
        }
    }
    let all_given = first_fault.is_none() && heights.len() == n;
    // What the kind of line says about its slope, on every plane it names. This is what
    // catches a plane whose eave was entered on the wrong edge: its rakes come out level
    // and its eave climbs. It is asked of each plane before the planes are compared with
    // each other, and every plane that fails is recorded, not only the first: the line is
    // evidence about each of them whatever else is wrong with it.
    let mut kind_fault: Option<String> = None;
    for (p, z) in &heights {
        let rise = (z[1] - z[0]).abs();
        let level = rise <= FIT_TOLERANCE_FT;
        let pitch = planes.get(*p).and_then(|g| g.pitch).unwrap_or(0.0);
        // Would this line climb noticeably if it ran straight up the slope?
        let could_climb = plan.is_some_and(|l| l * pitch / 12.0 > 10.0 * FIT_TOLERANCE_FT);
        let name = match kind {
            Kind::Eave => "an eave",
            Kind::Ridge => "a ridge",
            Kind::Headwall => "a headwall",
            Kind::Rake => "a rake",
            Kind::Hip => "a hip",
            Kind::Valley => "a valley",
            // A change of pitch runs level or climbs; its kind says nothing about which.
            _ => continue,
        };
        let fault = match kind {
            Kind::Eave | Kind::Ridge | Kind::Headwall if !level => Some(format!("{name} is level, but plane {p} puts this line's ends at {} and {}; check which edge of the plane is entered as its eave", crate::units::length_said(z[0]), crate::units::length_said(z[1]))),
            Kind::Rake | Kind::Hip | Kind::Valley if level && could_climb => Some(format!("{name} climbs the slope, but plane {p} puts both ends of this line at {}; check which edge of the plane is entered as its eave", crate::units::length_said(z[0]))),
            _ => None,
        };
        if let Some(fault) = fault {
            shows_wrong_eave.push((*p).clone());
            kind_fault.get_or_insert(fault);
        }
    }
    if let Some(fault) = first_fault {
        return refuse(fault);
    }
    if all_given {
        for (end, label) in [(0, "start"), (1, "end")] {
            let (lo, hi) = heights.iter().fold((f64::MAX, f64::MIN), |(lo, hi), (_, z)| (lo.min(z[end]), hi.max(z[end])));
            if hi - lo > FIT_TOLERANCE_FT {
                let each: Vec<String> = heights.iter().map(|(p, z)| format!("{p} puts it at {}", crate::units::length_said(z[end]))).collect();
                return refuse(format!("the planes disagree about the height of the line's {label}: {}. The line as drawn does not fit these pitches and eave heights", each.join("; ")));
            }
        }
    }
    if let Some(fault) = kind_fault {
        return refuse(fault);
    }
    let value = match (plan, heights.first()) {
        (Some(l), Some((_, z))) if all_given => Some((l * l + (z[1] - z[0]).powi(2)).sqrt()),
        _ => None,
    };
    Derived::new(value, "LF", &statuses, formula, &inputs)
}
