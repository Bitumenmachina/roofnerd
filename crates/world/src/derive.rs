//! Working quantities out of geometry.
//!
//! A roof plan is flat; the roof is not. A plane's true area is its plan area times a
//! slope factor that comes from its pitch. A hip's true length is longer than the line
//! drawn on the plan, by however much the hip climbs. This module does that arithmetic
//! for every element, writes the result into the element's `pl::measure`, and reports
//! which elements' measures actually changed.
//!
//! Two rules hold throughout:
//! * a result is only as good as its worst input (see `value.rs`);
//! * when the inputs disagree, no number is given. A hip sits on two planes, and a line
//!   that sits on two planes has to be at the same height on both, at both of its ends.
//!   If their pitches or eave heights put an end at two different heights, the line is
//!   flagged with both figures instead of quietly taking one.

use std::collections::{BTreeMap, HashMap};

use bevy_ecs::prelude::*;

use crate::components::*;
use crate::value::{Derived, Status};

/// Two planes agree on a height when they are within this many feet (about 1/8 inch).
const HEIGHT_TOLERANCE_FT: f64 = 0.01;

/// What a run needs to know about a plane it lies on.
struct PlaneGeom {
    points: Option<Vec<P2>>,
    points_status: Status,
    pitch: Option<f64>,
    pitch_status: Status,
    eave_height: Option<f64>,
    eave_height_status: Status,
}

impl PlaneGeom {
    /// The height of the plane above a plan point, in feet: the eave height plus the climb
    /// from the eave to that point.
    fn height_at(&self, p: P2) -> Option<f64> {
        let pts = self.points.as_ref()?;
        let pitch = self.pitch?;
        let eave_height = self.eave_height?;
        if pts.len() < 2 {
            return None;
        }
        let (a, b) = (pts[0], pts[1]);
        let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
        let len = (ex * ex + ey * ey).sqrt();
        if len == 0.0 {
            return None;
        }
        // Distance from the eave line, measured square to it on the plan.
        let off = ((ex * (p[1] - a[1])) - (ey * (p[0] - a[0]))).abs() / len;
        Some(eave_height + off * pitch / 12.0)
    }
}

fn plan_area(pts: &[P2]) -> f64 {
    let n = pts.len();
    let mut twice = 0.0;
    for i in 0..n {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        twice += p[0] * q[1] - q[0] * p[1];
    }
    twice.abs() / 2.0
}

fn dist(a: P2, b: P2) -> f64 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
}

/// Recompute every element's measure. Returns the paths whose measure changed, in job order.
pub fn derive(world: &mut World) -> Vec<String> {
    let mut planes: HashMap<String, PlaneGeom> = HashMap::new();
    let mut q = world.query::<(&Node, &Outline, &Pitch)>();
    for (node, outline, pitch) in q.iter(world) {
        planes.insert(
            node.path.clone(),
            PlaneGeom {
                points: outline.points.value.clone(),
                points_status: outline.points.status,
                pitch: pitch.0.value,
                pitch_status: pitch.0.status,
                eave_height: outline.eave_height.value,
                eave_height_status: outline.eave_height.status,
            },
        );
    }

    let mut changed = Vec::new();
    let mut q = world.query::<(&Node, &Kind, Option<&Outline>, Option<&Pitch>, Option<&Run>, Option<&Height>, Option<&mut Measure>)>();
    let mut to_insert: Vec<(Entity, Measure)> = Vec::new();
    let mut q_ids = world.query::<(Entity, &Node)>();
    let ids: HashMap<String, Entity> = q_ids.iter(world).map(|(e, n)| (n.path.clone(), e)).collect();

    for (node, kind, outline, pitch, run, height, measure) in q.iter_mut(world) {
        let mut m = BTreeMap::new();
        let me = &node.path;

        if kind.is_area() {
            if let (Some(outline), Some(pitch)) = (outline, pitch) {
                let in_outline = format!("{me}#{K_OUTLINE}");
                let in_pitch = format!("{me}#{K_PITCH}");
                let area = outline.points.value.as_ref().map(|p| plan_area(p));
                let factor = pitch.0.value.map(|p| (1.0 + (p / 12.0).powi(2)).sqrt());
                let surface = area.zip(factor).map(|(a, f)| a * f);
                let both = [outline.points.status, pitch.0.status];
                m.insert("plan_area".into(), Derived::new(area, "SF", &[outline.points.status], "area of the outline on the plan", &[in_outline.clone()]));
                m.insert("slope_factor".into(), Derived::new(factor, "", &[pitch.0.status], "√(1 + (pitch ÷ 12)²)", &[in_pitch.clone()]));
                m.insert("surface_area".into(), Derived::new(surface, "SF", &both, "plan_area × slope_factor", &[in_outline.clone(), in_pitch.clone()]));
                m.insert("squares".into(), Derived::new(surface.map(|s| s / 100.0), "SQ", &both, "surface_area ÷ 100", &[in_outline, in_pitch]));
            }
        }

        if kind.is_run() {
            if let Some(run) = run {
                let in_run = format!("{me}#{K_RUN}");
                let plan = run.ends.value.map(|[a, b]| dist(a, b));
                m.insert("plan_length".into(), Derived::new(plan, "LF", &[run.ends.status], "length of the line on the plan", &[in_run.clone()]));

                if *kind == Kind::Parapet {
                    if let Some(height) = height {
                        let in_h = format!("{me}#{K_HEIGHT}");
                        let face = plan.zip(height.0.value).map(|(l, h)| l * h);
                        m.insert("face_area".into(), Derived::new(face, "SF", &[run.ends.status, height.0.status], "plan_length × height", &[in_run.clone(), in_h]));
                    }
                } else {
                    m.insert("true_length".into(), true_length(run, plan, &planes, &in_run));
                }
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

/// The length of a run along the slope. Every plane the run lies on gives a height at
/// each of the run's two ends; they have to agree.
fn true_length(run: &Run, plan: Option<f64>, planes: &HashMap<String, PlaneGeom>, in_run: &str) -> Derived {
    let formula = "√(plan_length² + rise²); rise = the difference in height between the two ends, each height = eave height + pitch ÷ 12 × distance from the plane's eave";
    let mut inputs = vec![in_run.to_string()];
    for p in &run.planes {
        inputs.push(format!("{p}#{K_OUTLINE}"));
        inputs.push(format!("{p}#{K_PITCH}"));
    }
    if run.planes.is_empty() {
        return Derived::refused("LF", formula, &inputs, "no plane is named for this line, so its slope length cannot be worked out");
    }
    let mut statuses = vec![run.ends.status];
    // For each plane: its name and the height it gives each end.
    let mut heights: Vec<(String, [f64; 2])> = Vec::new();
    for p in &run.planes {
        let Some(g) = planes.get(p) else {
            return Derived::refused("LF", formula, &inputs, &format!("names plane {p}, which is not in the job"));
        };
        statuses.extend([g.points_status, g.pitch_status, g.eave_height_status]);
        if let Some([a, b]) = run.ends.value {
            if let (Some(za), Some(zb)) = (g.height_at(a), g.height_at(b)) {
                heights.push((p.clone(), [za, zb]));
            }
        }
    }
    let all_given = heights.len() == run.planes.len();
    if all_given {
        for (end, label) in [(0, "start"), (1, "end")] {
            let (lo, hi) = heights.iter().fold((f64::MAX, f64::MIN), |(lo, hi), (_, z)| (lo.min(z[end]), hi.max(z[end])));
            if hi - lo > HEIGHT_TOLERANCE_FT {
                let each: Vec<String> = heights.iter().map(|(p, z)| format!("{p} puts it at {:.2} ft", z[end])).collect();
                return Derived::refused("LF", formula, &inputs, &format!("the planes disagree about the height of the line's {label}: {}. The line as drawn does not fit these pitches and eave heights", each.join("; ")));
            }
        }
    }
    let value = match (plan, heights.first()) {
        (Some(l), Some((_, z))) if all_given => Some((l * l + (z[1] - z[0]).powi(2)).sqrt()),
        _ => None,
    };
    Derived::new(value, "LF", &statuses, formula, &inputs)
}
