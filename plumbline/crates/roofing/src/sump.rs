// ── The sump ───────────────────────────────────────────────────────────────
// The square the tapered field is cut down in to meet the drain bowl.
//
// The size is the trade's, not the drawing's: "tapered insulation layouts should
// be designed to form a sump that measures the size of the drain bowl's diameter
// plus approximately 24 inches at roof drains" (NRCA p.164). A 12 in bowl wants
// 36 in square. It is its own component — NRCA p.306 has them premanufactured at
// 4 or 8 ft a side and notes that shaving one in the field is no longer feasible
// at modern thicknesses — so it is counted in panels and not in square feet of
// ramp.
//
// WHAT SETS THE SIDE. Either the estimator states it (`SUMP`, in feet, because
// that is the unit the panels are sold in) or a bowl is stated and the rule above
// gives it. Where both are stated they are both reported, because a stated sump
// narrower than the bowl plus 24 in is a fact the estimator needs to see rather
// than a number to overrule.
//
// WHERE THE CUT IS. At the thickness the field has at the sump's own rim, read
// off the solved planes. On a four-way layout that is the same number the drawing
// used to work out for itself — `T + side/2 x TAPER` — and it is the same number
// all the way round the rim, because the thickness is the GREATER of two square
// runs and so a square standing square to the boards is a contour of it. The
// drawing now reads it instead of computing it, which is the rule in `CLAUDE.md`:
// two arithmetics for one number disagree eventually.

use crate::geom::{common_with, square_of, Plane, Point, Shape};
use crate::model::Sump;
use crate::taper::read_at;

/// How big a sump has to be, in inches on a side, for a drain bowl of a diameter.
///
/// `sumpSide` in `packages/engine/src/roof.ts` — NRCA p.164. The two copies are
/// pinned to each other by a test, the way every other convention in that file is.
pub fn sump_side_inches(bowl_inches: f64) -> f64 {
    bowl_inches + 24.0
}

/// How many panels a sump takes, at the board module the schedule is drawn on.
///
/// Whole panels each way and never a fraction: a sump four feet a side is one 4 ft
/// panel, and one eight feet a side is four of them. Rounding up is not an
/// allowance — a part of a panel is a panel you bought.
pub fn panels_for(side_feet: f64, panel_feet: f64) -> usize {
    if side_feet <= 0.0 || panel_feet <= 0.0 {
        return 0;
    }
    let each_way = (side_feet / panel_feet).ceil() as usize;
    each_way * each_way
}

/// The sump at one drain, cut to the ground the field holds.
///
/// `None` where the drain is not on this field's ground at all, which is a drain
/// that drains something else — C1 already refuses to lay a field out from one.
#[allow(clippy::too_many_arguments)]
pub fn sump_at(
    condition_id: &str,
    trace: usize,
    at: Point,
    side_feet: f64,
    from_bowl_feet: Option<f64>,
    ground: &[Shape],
    along: Point,
    across: Point,
    planes: &[Plane],
    start_inches: f64,
    cap_inches: Option<f64>,
    panel_feet: f64,
) -> Option<Sump> {
    if side_feet <= 0.0 {
        return None;
    }
    let half = side_feet / 2.0;
    let square = square_of(at, along, across, half);
    let shapes = common_with(&square, ground);
    if shapes.is_empty() {
        return None;
    }
    // The rim, read off the solved planes: half a sump out along an axis, which is
    // where the cut meets the field. Every point on the rim is the same depth,
    // because the square is a contour of a four-way layout.
    let rim = Point::new(at.x + along.x * half, at.y + along.y * half);
    let rim_inches = read_at(rim, planes, start_inches, cap_inches)
        .map(|r| r.thickness_inches)
        .unwrap_or(start_inches);
    Some(Sump {
        condition_id: condition_id.to_string(),
        trace,
        at,
        side_feet,
        from_bowl_feet,
        square_feet: shapes.iter().map(Shape::square_feet).sum(),
        clipped: shapes.len() != 1 || (shapes[0].square_feet() < side_feet * side_feet - 0.01),
        shapes,
        rim_inches,
        depth_inches: (rim_inches - start_inches).max(0.0),
        panels: panels_for(side_feet, panel_feet),
        panel_feet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rule, and the figure NRCA prints beside it.
    #[test]
    fn a_sump_is_the_bowl_plus_two_feet_all_round() {
        assert_eq!(sump_side_inches(12.0), 36.0);
        assert_eq!(sump_side_inches(4.0), 28.0);
        // Three feet of sump still wants a whole 4 ft panel.
        assert_eq!(panels_for(36.0 / 12.0, 4.0), 1);
        assert_eq!(panels_for(4.0, 4.0), 1);
        assert_eq!(panels_for(8.0, 4.0), 4);
        // Nothing stated is nothing counted, not one panel by default.
        assert_eq!(panels_for(0.0, 4.0), 0);
    }
}
