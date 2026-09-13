// ── Crickets and saddles ───────────────────────────────────────────────────
// A cricket behind a curb and a saddle between two drains are two different
// things with two different rules, and the one thing that cannot tell them apart
// is the traced line, which is a ridge either way.
//
//   behind a curb    NRCA §8.1 p.252 Fig. 8-1, and Figs. 10-13/10-14 pp.309-310:
//                    a wedge behind the curb, its ridge square to the face and
//                    centred on it, as wide each side as half that face
//   between drains   NRCA pp.166-168 and Fig. 4-14: the ridge is the perpendicular
//                    bisector of the two drains and the fall runs half the span
//                    each way
//
// Both are cut at twice the slope of the field beside them (p.79 and p.165) and
// both are held to the length-to-width table (Fig. 4-13). Cricket stock is its own
// lettered series — Carlisle prints two charts side by side, headed Cricket and
// Primary Slope — and the bulletin in `data/profiles` carries only the primary
// one. So a cricket is billed by AREA and not by letter, and that gap is recorded
// rather than papered over with the field's own table.
//
// IT SITS ON THE SOLVED SURFACE. Every vertex is lifted by the roof's own depth at
// that point, read off the field's planes. Before C2 the whole wedge sat at a
// typed `ELEV` on the condition — the demo carried three quarters of a foot — so
// the cricket floated at one height over a roof that rises fourteen inches under
// it. That is the B1 pushback the plan names at S1 step 4, and this closes it: the
// only place `ELEV` is still used is a cricket on no tapered field at all, where
// there is no solved surface to sit on and the estimator's own datum is the honest
// answer.

use crate::geom::{inside, inside_ring, run_length, Point};
use crate::model::{Condition, Cricket, Document, Refused, TaperField};
use crate::taper::{
    max_length_to_width, ridge_between, ridge_disagreement, surface_feet, CRICKET_SLOPE_MULTIPLE,
};

/// Which way a tapered field falls: four ways off the drain, or two.
///
/// Typed as `LAYOUT`, and typed as a NUMBER because that is what a property is:
/// the number of directions the field falls in. Four is the default and the one
/// NRCA generally prefers at an interior drain (p.165, Fig. 4-11); two is the
/// layout that needs a saddle between every pair of drains, because the ground
/// along the line joining them comes out level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    FourWay,
    TwoWay,
}

impl Layout {
    pub fn of(typed: Option<f64>) -> Self {
        match typed {
            Some(t) if (t - 2.0).abs() < 1e-9 => Self::TwoWay,
            _ => Self::FourWay,
        }
    }

    /// A typed `LAYOUT` that names neither, so the default was used.
    pub fn refused(typed: Option<f64>) -> Option<f64> {
        match typed {
            Some(t) if (t - 2.0).abs() < 1e-9 || (t - 4.0).abs() < 1e-9 => None,
            other => other,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::FourWay => "four-way",
            Self::TwoWay => "two-way",
        }
    }
}

/// The directions two of a layout's planes tie in, out of the drain.
///
/// A tie is where the fall changes hands, which is what a valley is. On a four-way
/// layout the two runs are equal on the diagonals, so there are four of them at 45
/// degrees exactly — NRCA p.166, and it falls out of the planes rather than being
/// drawn. On a two-way layout the two planes tie along the line through the drain
/// square to the fall, which is the trough the water runs along to reach it.
pub fn tie_directions(layout: Layout, along: Point, across: Point) -> Vec<Point> {
    match layout {
        Layout::FourWay => [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
            .iter()
            .map(|(a, b)| {
                Point::new(
                    (along.x * a + across.x * b) / std::f64::consts::SQRT_2,
                    (along.y * a + across.y * b) / std::f64::consts::SQRT_2,
                )
            })
            .collect(),
        Layout::TwoWay => vec![along, Point::new(-along.x, -along.y)],
    }
}

/// Twice the field, unless the estimator said otherwise — in which case theirs is
/// drawn and the disagreement is reported. Cricket stock comes in the slopes it
/// comes in.
fn cricket_slope(stated: Option<f64>, field_slope: f64) -> (f64, bool) {
    let wanted = field_slope * CRICKET_SLOPE_MULTIPLE;
    let slope = stated.unwrap_or(wanted);
    let disagrees = match stated {
        Some(s) => field_slope > 0.0 && (s - wanted).abs() > 1e-6,
        None => false,
    };
    (slope, disagrees)
}

/// The ratio the length-to-width table is read at: the field's own slope, or —
/// where no field says — the cricket's slope halved back.
fn ratio_slope(field_slope: f64, slope: f64) -> f64 {
    if field_slope != 0.0 {
        field_slope
    } else {
        slope / CRICKET_SLOPE_MULTIPLE
    }
}

/// Inches to feet, because a slope is in inches and the world is in feet.
fn feet(inches: f64) -> f64 {
    inches / 12.0
}

/// Where the roof's own top is at a place, in feet, or a typed datum.
///
/// The pair is the whole point: the first is the solved surface and the second says
/// whether that is what it is. A cricket on no tapered field has no solved surface
/// under it, and the estimator's `ELEV` is then the only datum anybody has stated.
struct Ground<'a> {
    field: Option<&'a TaperField>,
    elevation: f64,
}

impl Ground<'_> {
    fn at(&self, p: Point) -> f64 {
        match self.field {
            Some(field) => surface_feet(field, p),
            None => self.elevation,
        }
    }

    fn solved(&self) -> bool {
        self.field.is_some()
    }

    fn slope(&self) -> f64 {
        self.field.map(|f| f.slope_inches_per_foot).unwrap_or(0.0)
    }
}

/// The two falling planes of a cricket, as triangles on the roof it sits on.
///
/// A ridge from A to B, and a foot half a width out on each side. Every vertex is
/// lifted by the roof's own depth where it stands, so the wedge follows the field
/// instead of floating over it; the ridge is that plus its own rise.
fn tent(
    ridge_a: Point,
    ridge_b: Point,
    fall: Point,
    width: f64,
    rise: f64,
    ground: &Ground,
) -> Vec<f64> {
    let mut positions = Vec::with_capacity(36);
    let top = |p: Point| ground.at(p) + rise;
    for side in [1.0, -1.0] {
        let foot_a = Point::new(ridge_a.x + fall.x * width * side, ridge_a.y + fall.y * width * side);
        let foot_b = Point::new(ridge_b.x + fall.x * width * side, ridge_b.y + fall.y * width * side);
        positions.extend_from_slice(&[
            ridge_a.x, top(ridge_a), ridge_a.y,
            ridge_b.x, top(ridge_b), ridge_b.y,
            foot_a.x, ground.at(foot_a), foot_a.y,
        ]);
        positions.extend_from_slice(&[
            ridge_b.x, top(ridge_b), ridge_b.y,
            foot_b.x, ground.at(foot_b), foot_b.y,
            foot_a.x, ground.at(foot_a), foot_a.y,
        ]);
    }
    positions
}

/// The field a place on the roof is on, or none.
fn field_under<'a>(fields: &'a [TaperField], p: Point) -> Option<&'a TaperField> {
    fields.iter().find(|f| inside(p, &f.ring))
}

/// A cricket, from whatever the document says it serves.
///
/// `between` names what is served; a curb named there says this is the cricket
/// behind it. `model.ts:891-916` was where this decision used to be made.
pub fn cricket(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
    fields: &[TaperField],
    c: &Condition,
    trace: usize,
    ring: &[Point],
) -> Option<Cricket> {
    // The field this cricket sits in: the one its own traced line stands on. That
    // is what says how deep the roof is under it and what slope it should be cut
    // at, and it is read off the solve rather than off the first area condition
    // that happens to carry a TAPER.
    let field = ring.first().copied().and_then(|p| field_under(fields, p));
    let ground = Ground { field, elevation: c.property("ELEV").unwrap_or(0.0) };
    let stated = c.property("TAPER");

    match curb_named_by(document, scales, &c.between) {
        Some(curb) => cricket_behind(c, trace, ring, &curb, stated, &ground),
        None => {
            let pair = pair_for(ring, &drains_named_by(document, scales, &c.between))?;
            cricket_between(c, trace, ring, pair, stated, &ground)
        }
    }
}

/// The curb a cricket says it is behind, as a ring in feet.
fn curb_named_by(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
    names: &[String],
) -> Option<Vec<Point>> {
    if names.is_empty() {
        return None;
    }
    for c in &document.conditions {
        if !names.contains(&c.id) || c.hidden || !c.has_role("curb") {
            continue;
        }
        for trace in &c.traces {
            let ring = document.feet(trace, scales);
            if ring.len() >= 3 {
                return Some(ring);
            }
        }
    }
    None
}

/// The drains named by a cricket's `between`, and only those.
fn drains_named_by(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
    names: &[String],
) -> Vec<Point> {
    let mut out = Vec::new();
    if names.is_empty() {
        return out;
    }
    for c in &document.conditions {
        if !names.contains(&c.id) || c.hidden {
            continue;
        }
        for trace in &c.traces {
            out.extend(document.feet(trace, scales));
        }
    }
    out
}

/// The two drainage points a cricket sits between: the two nearest the ridge the
/// estimator drew.
fn pair_for(traced: &[Point], candidates: &[Point]) -> Option<(Point, Point)> {
    if candidates.len() < 2 || traced.is_empty() {
        return None;
    }
    let mid = traced[traced.len() / 2];
    let mut by_range: Vec<Point> = candidates.to_vec();
    by_range.sort_by(|a, b| {
        let ra = (a.x - mid.x).hypot(a.y - mid.y);
        let rb = (b.x - mid.x).hypot(b.y - mid.y);
        ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
    });
    Some((by_range[0], by_range[1]))
}

/// Two planes falling from the ridge the drains put there.
fn cricket_between(
    c: &Condition,
    trace: usize,
    traced: &[Point],
    pair: (Point, Point),
    stated: Option<f64>,
    ground: &Ground,
) -> Option<Cricket> {
    let derived = ridge_between(pair.0, pair.1)?;
    let (slope, slope_disagrees) = cricket_slope(stated, ground.slope());

    let length = run_length(traced);
    if length < 1e-6 {
        return None;
    }

    let width = derived.width;
    let rise = feet(slope * width);
    let half = length / 2.0;
    // Along the ridge, and square to it — square to the ridge is the direction the
    // water falls, which is the line joining the two drains.
    let along = derived.along;
    let fall = Point::new(-along.y, along.x);

    let ridge_a = Point::new(derived.at.x - along.x * half, derived.at.y - along.y * half);
    let ridge_b = Point::new(derived.at.x + along.x * half, derived.at.y + along.y * half);

    Some(Cricket {
        condition_id: c.id.clone(),
        trace,
        positions: tent(ridge_a, ridge_b, fall, width, rise, ground),
        plan_square_feet: length * width * 2.0,
        slope_inches_per_foot: slope,
        length_to_width: length / width,
        max_length_to_width: max_length_to_width(ratio_slope(ground.slope(), slope)),
        off_by: ridge_disagreement(traced, derived.along),
        slope_disagrees,
        derived: false,
        from_solved_surface: ground.solved(),
    })
}

/// A cricket behind a curb, sized off the curb rather than off the drains.
fn cricket_behind(
    c: &Condition,
    trace: usize,
    traced: &[Point],
    curb: &[Point],
    stated: Option<f64>,
    ground: &Ground,
) -> Option<Cricket> {
    if traced.len() < 2 || curb.len() < 3 {
        return None;
    }

    let middle = Point::new(
        curb.iter().map(|p| p.x).sum::<f64>() / curb.len() as f64,
        curb.iter().map(|p| p.y).sum::<f64>() / curb.len() as f64,
    );

    // Which face of the curb this cricket sits against: the one the estimator
    // started their ridge at. A slope direction could not break the tie on a curb
    // standing between two drains, and this curb does.
    let start = traced[0];
    let mut face: Option<(Point, f64, Point, Point)> = None;
    let mut nearest = f64::INFINITY;
    for pair in curb.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let run = dx.hypot(dy);
        if run < 1e-6 {
            continue;
        }
        let mid = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        let range = (mid.x - start.x).hypot(mid.y - start.y);
        if range >= nearest {
            continue;
        }
        nearest = range;
        // Square to the face and pointing away from the curb, so the cricket is
        // always outside the thing it sheds water around.
        let out = if (middle.x - mid.x) * (-dy / run) + (middle.y - mid.y) * (dx / run) >= 0.0 {
            -1.0
        } else {
            1.0
        };
        face = Some((
            mid,
            run / 2.0,
            Point::new(dx / run, dy / run),
            Point::new((-dy / run) * out, (dx / run) * out),
        ));
    }
    let (mid, half, across, along) = face?;
    if half < 1e-6 {
        return None;
    }

    let (slope, slope_disagrees) = cricket_slope(stated, ground.slope());
    let length = run_length(traced);
    if length < 1e-6 {
        return None;
    }

    let rise = feet(slope * half);
    let ridge_b = Point::new(mid.x + along.x * length, mid.y + along.y * length);

    Some(Cricket {
        condition_id: c.id.clone(),
        trace,
        positions: tent(mid, ridge_b, across, half, rise, ground),
        plan_square_feet: length * half * 2.0,
        slope_inches_per_foot: slope,
        length_to_width: length / half,
        max_length_to_width: max_length_to_width(ratio_slope(ground.slope(), slope)),
        // Square to the curb's face is what the figures draw, so that is what the
        // traced line is held against here.
        off_by: ridge_disagreement(traced, along),
        slope_disagrees,
        derived: false,
        from_solved_surface: ground.solved(),
    })
}

/// The saddles a two-way layout needs, and the ones it cannot have.
///
/// On a two-way field the ground along the line joining two drains comes out level,
/// so water stands there unless a saddle is built: ridge on the perpendicular
/// bisector, fall half the span each way, cut at twice the field (NRCA Fig. 4-14).
/// The saddle's length is the run of that bisector across the field — it has to
/// cover the trough, and how long the trough is is a fact about the roof.
///
/// Fig. 4-13 caps how long a saddle may be against how wide it is. Past that it
/// ponds at the valley however right it looks in plan, so an over-long one is
/// REFUSED with the ratio in words rather than drawn: the answer is another drain
/// or an area divider, and that is the estimator's call to make.
pub fn saddles_between(field: &TaperField, field_index: usize) -> (Vec<Cricket>, Vec<Refused>) {
    let mut out = Vec::new();
    let mut refused = Vec::new();
    let slope = field.slope_inches_per_foot * CRICKET_SLOPE_MULTIPLE;
    let most = max_length_to_width(field.slope_inches_per_foot);
    let ground = Ground { field: Some(field), elevation: field.elevation };
    let _ = field_index;

    for (i, region) in field.regions.iter().enumerate() {
        for j in region.ridges.iter().map(|r| r.meets).collect::<std::collections::BTreeSet<_>>() {
            if j <= i {
                continue;
            }
            let Some(other) = field.regions.get(j) else { continue };
            let Some(bisector) = ridge_between(region.drain, other.drain) else { continue };
            let reach = (field.square_feet.sqrt() + 1.0) * 4.0;
            let from = Point::new(
                bisector.at.x - bisector.along.x * reach,
                bisector.at.y - bisector.along.y * reach,
            );
            let to = Point::new(
                bisector.at.x + bisector.along.x * reach,
                bisector.at.y + bisector.along.y * reach,
            );
            // The longest piece of the bisector that is actually on the roof: a
            // concave field can hand it two, and the saddle spans the trough.
            let Some((a, b)) = inside_ring(from, to, &field.ring)
                .into_iter()
                .max_by(|x, y| {
                    let lx = (x.1.x - x.0.x).hypot(x.1.y - x.0.y);
                    let ly = (y.1.x - y.0.x).hypot(y.1.y - y.0.y);
                    lx.partial_cmp(&ly).unwrap_or(std::cmp::Ordering::Equal)
                })
            else {
                continue;
            };
            let length = (b.x - a.x).hypot(b.y - a.y);
            let width = bisector.width;
            if length < 1e-6 || width < 1e-6 {
                continue;
            }
            let ratio = length / width;
            if ratio > most {
                refused.push(Refused {
                    condition_id: field.condition_id.clone(),
                    trace: field.trace,
                    why: format!(
                        "the saddle the two-way layout needs between two drains would run {ratio:.1} long against its width and the table allows {most:.0} — it wants another drain or an area divider"
                    ),
                });
                continue;
            }
            let fall = Point::new(-bisector.along.y, bisector.along.x);
            out.push(Cricket {
                condition_id: field.condition_id.clone(),
                trace: field.trace,
                positions: tent(a, b, fall, width, feet(slope * width), &ground),
                plan_square_feet: length * width * 2.0,
                slope_inches_per_foot: slope,
                length_to_width: ratio,
                max_length_to_width: most,
                // Nobody traced it, so there is no traced line to be off.
                off_by: None,
                slope_disagrees: false,
                derived: true,
                from_solved_surface: true,
            });
        }
    }
    (out, refused)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_layout_is_four_ways_unless_somebody_says_two() {
        assert_eq!(Layout::of(None), Layout::FourWay);
        assert_eq!(Layout::of(Some(4.0)), Layout::FourWay);
        assert_eq!(Layout::of(Some(2.0)), Layout::TwoWay);
        // A number that names neither is not honoured in silence.
        assert_eq!(Layout::refused(Some(3.0)), Some(3.0));
        assert_eq!(Layout::refused(Some(2.0)), None);
    }

    /// Four ties at 45 degrees, or two along the trough. Nothing else.
    #[test]
    fn the_ties_are_the_diagonals_or_the_trough() {
        let along = Point::new(1.0, 0.0);
        let across = Point::new(0.0, 1.0);
        let four = tie_directions(Layout::FourWay, along, across);
        assert_eq!(four.len(), 4);
        for d in four {
            assert!((d.x.abs() - d.y.abs()).abs() < 1e-12, "{d:?} is not a diagonal");
        }
        let two = tie_directions(Layout::TwoWay, along, across);
        assert_eq!(two, vec![Point::new(1.0, 0.0), Point::new(-1.0, -0.0)]);
    }

    /// The cricket is cut at twice the field, and a stated slope that is not says so.
    #[test]
    fn twice_the_field_unless_the_estimator_said_otherwise() {
        assert_eq!(cricket_slope(None, 0.25), (0.5, false));
        assert_eq!(cricket_slope(Some(0.5), 0.25), (0.5, false));
        let (slope, disagrees) = cricket_slope(Some(0.25), 0.25);
        assert_eq!(slope, 0.25);
        assert!(disagrees, "a cricket at the field's own slope is not twice it");
    }
}
