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

    // The rest of the building: walls, floors, ducts, pipes. Worked out in `building.rs`
    // and added to each element's measure below.
    let mut building = crate::building::measures(world);

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
        if *kind == Kind::Material {
            materials.push((node.name.clone(), node.path.clone(), supply.clone()));
        }
    }
    #[allow(clippy::type_complexity)]
    let mut q = world.query::<(&Node, &Kind, Option<&Outline>, Option<&Pitch>, Option<&Run>, Option<&Height>, Option<&Cost>, Option<&Allowance>, Option<&MadeOf>, Option<&Condition>, Option<&mut Measure>, Option<&mut LineQuantities>)>();
    for (node, kind, outline, pitch, run, height, cost, allowance, made_of, condition, measure, line_quantities) in q.iter_mut(world) {
        let me = &node.path;
        let mut m = BTreeMap::new();

        if kind.is_area() {
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
                    Some(_) if unit_fault => Derived::refused("SF", &f_area, &[in_outline.clone()], &format!("the outline {NOT_CONVERTED}")),
                    Some(o) => Derived::new(area, "SF", &[o.points.status], &f_area, &[in_outline.clone()]),
                    None => Derived::refused("SF", &f_area, &[in_outline.clone()], "no outline entered"),
                },
            );
            m.insert(
                "slope_factor".into(),
                match pitch {
                    Some(p) => Derived::new(factor, "", &[p.0.status], f_factor, &[in_pitch.clone()]),
                    None => Derived::refused("", f_factor, &[in_pitch.clone()], "no pitch entered"),
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
                    Some(_) if unit_fault => Derived::refused("LF", &f_plan, &[in_run.clone()], &format!("the line {NOT_CONVERTED}")),
                    Some(r) => Derived::new(plan, "LF", &[r.ends.status], &f_plan, &[in_run.clone()]),
                    None => Derived::refused("LF", &f_plan, &[in_run.clone()], "no line entered"),
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
            }
        }

        if let Some(more) = building.remove(me) {
            m.extend(more);
        }

        // Every cost line, worked out against this element's figures: quantity, then
        // waste, then order units.
        let new = Measure(m);
        let mut lq = BTreeMap::new();
        if let Some(cost) = cost {
            let on = lines::On { kind: *kind, measure: Some(&new), height };
            let stock = match made_of.map(|m| &m.0) {
                None => lines::Stock::NotSaid,
                Some(m) => match &m.value {
                    None => lines::Stock::Missing(format!("what the element is made of is not known ({})", m.source)),
                    Some(name) => {
                        let found: Vec<&(String, String, Supply)> = materials.iter().filter(|(n, p, _)| n == name || p == name).collect();
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
                if *old != new {
                    *old = new;
                    moved = true;
                }
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
            Kind::Eave | Kind::Ridge | Kind::Headwall if !level => Some(format!("{name} is level, but plane {p} puts this line's ends at {:.2} ft and {:.2} ft; check which edge of the plane is entered as its eave", z[0], z[1])),
            Kind::Rake | Kind::Hip | Kind::Valley if level && could_climb => Some(format!("{name} climbs the slope, but plane {p} puts both ends of this line at {:.2} ft; check which edge of the plane is entered as its eave", z[0])),
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
                let each: Vec<String> = heights.iter().map(|(p, z)| format!("{p} puts it at {:.2} ft", z[end])).collect();
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
