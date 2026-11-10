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

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;

use crate::check::{self, FIT_TOLERANCE_FT};
use crate::components::*;
use crate::geom;
use crate::value::{Derived, Status};

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
        // entry checks may have an eave that fits no edge; it then gives no heights.
        let mut eave = None;
        if let Some(o) = outline {
            if let (Some(pts), Some(line)) = (o.points.value.as_ref(), o.eave.value) {
                match check::eave_edge(pts, line) {
                    Ok(edge) => eave = Some(edge),
                    Err(_) => lacks = lacks.or(Some("an eave line that fits its outline")),
                }
            }
        }
        if lacks.is_some() {
            status = Status::Unknown;
        }
        planes.insert(
            node.path.clone(),
            PlaneGeom {
                kind: *kind,
                points: outline.and_then(|o| o.points.value.clone()),
                eave,
                pitch: pitch.and_then(|p| p.0.value),
                eave_height: outline.and_then(|o| o.eave_height.value),
                status,
                lacks,
            },
        );
    }

    let mut q_ids = world.query::<(Entity, &Node)>();
    let ids: HashMap<String, Entity> = q_ids.iter(world).map(|(e, n)| (n.path.clone(), e)).collect();

    let mut changed = Vec::new();
    let mut to_insert: Vec<(Entity, Measure)> = Vec::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Outline>, Option<&Pitch>, Option<&Run>, Option<&Height>, Option<&mut Measure>)>();
    for (node, kind, outline, pitch, run, height, measure) in q.iter_mut(world) {
        let me = &node.path;
        let mut m = BTreeMap::new();

        if kind.is_area() {
            let in_outline = format!("{me}#{K_OUTLINE}");
            let in_pitch = format!("{me}#{K_PITCH}");
            let both = [in_outline.clone(), in_pitch.clone()];
            let f_area = "area of the outline on the plan";
            let f_factor = "√(1 + (pitch ÷ 12)²)";
            let f_surface = "plan_area × slope_factor";
            let f_squares = "surface_area ÷ 100";
            let area = outline.and_then(|o| o.points.value.as_ref().map(|p| geom::plan_area(p)));
            let factor = pitch.and_then(|p| p.0.value.map(|p| (1.0 + (p / 12.0).powi(2)).sqrt()));
            let surface = area.zip(factor).map(|(a, f)| a * f);
            m.insert(
                "plan_area".into(),
                match outline {
                    Some(o) => Derived::new(area, "SF", &[o.points.status], f_area, &[in_outline.clone()]),
                    None => Derived::refused("SF", f_area, &[in_outline.clone()], "no outline entered"),
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
            let f_plan = "length of the line on the plan";
            let plan = run.and_then(|r| r.ends.value.map(|[a, b]| geom::dist(a, b)));
            m.insert(
                "plan_length".into(),
                match run {
                    Some(r) => Derived::new(plan, "LF", &[r.ends.status], f_plan, &[in_run.clone()]),
                    None => Derived::refused("LF", f_plan, &[in_run.clone()], "no line entered"),
                },
            );
            if *kind == Kind::Parapet {
                let in_h = format!("{me}#{K_HEIGHT}");
                let f_face = "plan_length × height";
                let inputs = [in_run.clone(), in_h];
                m.insert(
                    "face_area".into(),
                    match (run, height) {
                        (Some(r), Some(h)) => Derived::new(plan.zip(h.0.value).map(|(l, h)| l * h), "SF", &[r.ends.status, h.0.status], f_face, &inputs),
                        (None, _) => Derived::refused("SF", f_face, &inputs, "no line entered"),
                        (_, None) => Derived::refused("SF", f_face, &inputs, "no height entered"),
                    },
                );
            } else {
                m.insert("true_length".into(), true_length(*kind, run, plan, &planes, &in_run));
            }
        }

        let new = Measure(m);
        match measure {
            Some(mut old) => {
                if *old != new {
                    *old = new;
                    changed.push(node.path.clone());
                }
            }
            None if !new.0.is_empty() => {
                to_insert.push((ids[&node.path], new));
                changed.push(node.path.clone());
            }
            None => {}
        }
    }
    for (e, m) in to_insert {
        world.entity_mut(e).insert(m);
    }
    changed
}

/// The length of a line along the slope. The line has to fit every plane it names: on the
/// plane's outline at both ends, and at the same height on every plane at both ends.
fn true_length(kind: Kind, run: Option<&Run>, plan: Option<f64>, planes: &HashMap<String, PlaneGeom>, in_run: &str) -> Derived {
    let formula = "√(plan_length² + rise²); rise = the difference in height between the two ends, each height = eave height + pitch ÷ 12 × distance from the plane's eave line";
    let mut inputs = vec![in_run.to_string()];
    let Some(run) = run else {
        return Derived::refused("LF", formula, &inputs, "no line entered");
    };
    for p in &run.planes {
        inputs.push(format!("{p}#{K_OUTLINE}"));
        inputs.push(format!("{p}#{K_PITCH}"));
    }
    let refuse = |why: String| Derived::refused("LF", formula, &inputs, &why);

    let (fewest, most) = kind.planes_allowed();
    let n = run.planes.len();
    if n < fewest || n > most {
        let want = if fewest == most { format!("{fewest}") } else { format!("{fewest} or {most}") };
        return refuse(format!("a {kind:?} lies on {want} plane(s); this one names {n}"));
    }
    let mut seen = Vec::new();
    for p in &run.planes {
        if seen.contains(&p) {
            return refuse(format!("names plane {p} twice"));
        }
        seen.push(p);
    }

    let mut statuses = vec![run.ends.status];
    // For each plane: its path and the height it gives each end of the line.
    let mut heights: Vec<(&String, [f64; 2])> = Vec::new();
    for p in &run.planes {
        let Some(g) = planes.get(p) else {
            return refuse(format!("names plane {p}, which is not in the job"));
        };
        if !g.kind.is_area() {
            return refuse(format!("names {p} as a plane, but {p} is a {:?}", g.kind));
        }
        if let Some(lacks) = g.lacks {
            return refuse(format!("plane {p} has {lacks}"));
        }
        statuses.push(g.status);
        let (Some([a, b]), Some(pts)) = (run.ends.value, g.points.as_ref()) else { continue };
        if let Err((t, off)) = geom::runs_along_outline(a, b, pts, FIT_TOLERANCE_FT) {
            let whereabouts = if t == 0.0 { "its start is".to_string() } else if t == 1.0 { "its end is".to_string() } else { format!("{:.0}% of the way along it is", t * 100.0) };
            return refuse(format!("the line does not run along the outline of plane {p}: {whereabouts} {off:.2} ft off it"));
        }
        if let (Some(za), Some(zb)) = (g.height_at(a), g.height_at(b)) {
            heights.push((p, [za, zb]));
        }
    }
    let all_given = heights.len() == n;
    if all_given {
        for (end, label) in [(0, "start"), (1, "end")] {
            let (lo, hi) = heights.iter().fold((f64::MAX, f64::MIN), |(lo, hi), (_, z)| (lo.min(z[end]), hi.max(z[end])));
            if hi - lo > FIT_TOLERANCE_FT {
                let each: Vec<String> = heights.iter().map(|(p, z)| format!("{p} puts it at {:.2} ft", z[end])).collect();
                return refuse(format!("the planes disagree about the height of the line's {label}: {}. The line as drawn does not fit these pitches and eave heights", each.join("; ")));
            }
        }
        // What the kind of line says about its slope. This is what catches a plane whose
        // eave was entered on the wrong edge: its rakes come out level and its eave climbs.
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
                _ => "a valley",
            };
            match kind {
                Kind::Eave | Kind::Ridge | Kind::Headwall if !level => {
                    return refuse(format!("{name} is level, but plane {p} puts this line's ends at {:.2} ft and {:.2} ft; check which edge of the plane is entered as its eave", z[0], z[1]));
                }
                Kind::Rake | Kind::Hip | Kind::Valley if level && could_climb => {
                    return refuse(format!("{name} climbs the slope, but plane {p} puts both ends of this line at {:.2} ft; check which edge of the plane is entered as its eave", z[0]));
                }
                _ => {}
            }
        }
    }
    let value = match (plan, heights.first()) {
        (Some(l), Some((_, z))) if all_given => Some((l * l + (z[1] - z[0]).powi(2)).sqrt()),
        _ => None,
    };
    Derived::new(value, "LF", &statuses, formula, &inputs)
}
