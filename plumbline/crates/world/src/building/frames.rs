//! A floor's joists and a roof plane's rafters, placed (call 177).
//!
//! Ours; the layout is `members::members_across`'s (call 165), here handed to a window as
//! lines on the plan instead of lengths. Nothing is laid that a priced line does not name:
//! a floor has joists where a line on it names the lumber and the spacing ("Floor joists,
//! 2x10 at 16 in"), a roof plane has rafters where a line on it or on its roof does ("Rafters,
//! 2x8 at 24 in"), by the rule the counts use (`finish::members_named`). A floor whose
//! joists' direction is not said has none laid and says so (`said`): the count takes the
//! short way, Assumed, and a drawing of members on that guess would be a guess in lumber.
//! The way they run is the count's (`joist_run`, `derive::rafter_layout`), so what is drawn is
//! what is counted.

use crate::building::{joist_run, members, Floor, JoistsSpan, Kind, Tree};
use crate::components::{Condition, Cost, MadeOf, Node, Outline, Pitch, P2};
use crate::Job;

/// The members of one floor or one roof plane.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// The floor's or the plane's path.
    pub path: String,
    pub kind: Kind,
    /// The storey a floor is on, by path.
    pub storey: Option<String>,
    /// How deep the lumber is, in feet: the line's own words.
    pub depth: f64,
    /// How far apart the members are, in feet.
    pub spacing: f64,
    /// Each member's two ends on the plan, in feet.
    pub members: Vec<[P2; 2]>,
    /// A floor's rim: the edges the joists' ends meet, on the plan.
    pub rim: Vec<[P2; 2]>,
    /// False: which way a floor's joists run is not said, and none are laid.
    pub said: bool,
}

/// Every floor with joists named and every roof plane with rafters named, laid.
pub fn frames(job: &mut Job) -> Vec<Frame> {
    let tree = Tree::of(&mut job.world);
    let world = &mut job.world;
    let mut out = Vec::new();
    let mut nq = world.query::<(&Node, &Kind, Option<&Cost>, Option<&MadeOf>, Option<&Condition>, Option<&Floor>, Option<&JoistsSpan>, Option<&Outline>, Option<&Pitch>)>();
    let rafters_named: std::collections::HashMap<String, (f64, f64)> = nq
        .iter(world)
        .filter(|(_, k, ..)| matches!(k, Kind::Roof | Kind::RoofPlane))
        .filter_map(|(n, _, cost, made_of, condition, ..)| crate::finish::members_named(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str), &["rafter"]).map(|named| (n.path.clone(), named)))
        .collect();
    let mut rows: Vec<_> = nq.iter(world).collect();
    rows.sort_by(|a, b| a.0.path.cmp(&b.0.path));
    for (node, kind, cost, made_of, condition, floor, span, outline, pitch) in rows {
        match kind {
            Kind::Floor => {
                let Some((depth, spacing)) = crate::finish::members_named(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str), &["joist"]) else { continue };
                let Some(ring) = floor.and_then(Floor::boundary_ft) else { continue };
                let storey = tree.storey_of(&node.path).map(str::to_string);
                let Some((run, said, _)) = joist_run(&ring, span) else { continue };
                if said.is_none() {
                    out.push(Frame { path: node.path.clone(), kind: *kind, storey, depth, spacing, members: Vec::new(), rim: Vec::new(), said: false });
                    continue;
                }
                let Ok((members, _)) = members::members_placed(&ring, run, spacing) else { continue };
                // The edges square to the joists are where their ends meet.
                let long = run[0].hypot(run[1]);
                let u = [run[0] / long, run[1] / long];
                let rim = (0..ring.len())
                    .map(|i| [ring[i], ring[(i + 1) % ring.len()]])
                    .filter(|[p, q]| {
                        let edge = (q[0] - p[0]).hypot(q[1] - p[1]);
                        edge > 1.0e-9 && ((q[0] - p[0]) * u[0] + (q[1] - p[1]) * u[1]).abs() / edge < 0.7
                    })
                    .collect();
                out.push(Frame { path: node.path.clone(), kind: *kind, storey, depth, spacing, members, rim, said: true });
            }
            Kind::RoofPlane => {
                let named = rafters_named.get(&node.path).or_else(|| node.parent.as_ref().and_then(|p| rafters_named.get(p)));
                let Some((depth, spacing)) = named.copied() else { continue };
                let Ok((ring, up, _)) = crate::derive::rafter_layout(outline, pitch) else { continue };
                let Ok((members, _)) = members::members_placed(&ring, up, spacing) else { continue };
                out.push(Frame { path: node.path.clone(), kind: *kind, storey: None, depth, spacing, members, rim: Vec::new(), said: true });
            }
            _ => {}
        }
    }
    out
}
