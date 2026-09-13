// ── Wind zones, and the uplift rating each one asks for ────────────────────
// Where the corners and the edges of a roof are, how hard the wind pulls on
// each, and what rating of assembly that makes the roof need. Everything a
// fastening pattern is chosen against, and nothing about the pattern itself —
// that is `fastening.rs`.
//
// THE ZONES (Part 1). NRCA *Membrane Roof Systems* 2019, Appendix 1, p.553:
// "Dimension 'a' is 10 percent of the least horizontal dimension of the building
// or 0.4 times the mean building height, whichever is smaller; but not less than
// either 4 percent of the least horizontal dimension or 3 feet." Figure 1-9 on
// the facing page draws what that dimension sets out: a corner region `a` by `a`
// at each outside corner, a perimeter band `a` wide along every edge, and the
// field of the roof inside that. The three tile the roof exactly, and
// `tests/the_zones.rs` says so to a hundredth of a square foot.
//
// The band is built by offsetting the traced outline inboard by `a` — the same
// `cavalier_contours` offset the parapet's own inboard face is built from, so a
// zone boundary at a concave corner is exactly as sound as the wall standing on
// it. The corner region is the quadrilateral `a` back along one edge and `a` on
// along the other, which on a square corner is the a-by-a square the figure
// draws and on any other corner is the shape those two edges actually make. Only
// corners that turn OUTWARD get one: a corner that turns into the roof is not a
// place the wind peels from, and the figure draws none there.
//
// ZONE 1'. The appendix names four zones, not three — Zone 1' at GCp 0.9 beside
// Zone 1 at 1.7 (Fig 1-2, p.546) — and then never draws the boundary. Figure 1-9
// is the only zone plan printed, for h ≤ 60 ft and for 60 < h ≤ 160 ft, and it
// draws 1, 2 and 3 and no 1'. Where Zone 1' begins comes from the ASCE figure
// itself, which is not in the box. So three zones are solved and the fourth is
// reported as missing rather than guessed at, and the solve says which document
// would close it.
//
// THE PRESSURE (Part 2). Never computed here. Design pressures are TYPED on the
// roof with their basis in the name of the field:
//
//   WIND_PSF_FIELD / _PERIMETER / _CORNER    strength design, as ASCE 7's own
//                                            maps are (p.546)
//   WIND_ASD_FIELD / _PERIMETER / _CORNER    already reduced to working load
//
// The basis rides in the name because every property in this program is a
// number: the Condition panel draws each one as a number field, and a word typed
// into a job would be wiped by the first edit of the field it sat in. Naming the
// two families apart says the same thing a typed "strength" would have said, in
// a place that cannot be lost, and it says it where the estimator reads it.
//
// Then, from the appendix:
//
//   working load = strength x 0.6        p.547: "applying a load reduction
//                                        factor of 0.6 to strength design values"
//   wanted       = working load x 2.0    p.554: "for low-slope membrane roof
//                                        systems, a minimum safety factor of 2.0"
//   rating       = the next rating up    p.565: ratings run "in increments of
//                                        15", and "an assembly rated FM 1-90 is
//                                        appropriate for installation in roof
//                                        areas with a maximum design uplift load
//                                        of 45 psf"
//
// so the rating number IS the wanted capacity in pounds a square foot, and the
// ladder is 60, 75, 90, 105, 120 and up in fifteens.

use crate::geom::{
    bounds, common_with, dedup_closing, offset_ring, signed_area, take_out, take_out_all, Point,
    Shape,
};
use crate::model::{Document, SolvedTaper};

/// The standard the zones are set out to, printed on every drawing that shows
/// them.
///
/// Georgia adopted the 2024 IBC/IRC on 1 January 2026, which references ASCE
/// 7-22. The zone plan and the `a` rule are Appendix 1's, which states the 7-16
/// method; both are unchanged for a roof under 60 ft, and the drawing prints the
/// edition it was drawn to so that a reader can tell which is which.
pub const WIND_STANDARD: &str = "ASCE 7-22 (Georgia 2024)";

/// What the zone geometry itself follows, said on the drawing beside the
/// standard.
pub const ZONE_RULE: &str = "NRCA Membrane Roof Systems 2019, Appendix 1 p.553";

/// Strength design to working load. Appendix 1 p.547.
pub const ASD_FACTOR: f64 = 0.6;

/// Working load to the capacity an assembly has to be rated for. Appendix 1
/// p.554, and ASTM D6630 behind it.
pub const SAFETY_FACTOR: f64 = 2.0;

/// Uplift ratings step in fifteens, and the lowest one anybody publishes.
/// Appendix 1 p.565.
pub const RATING_STEP_PSF: f64 = 15.0;
pub const LEAST_RATING_PSF: f64 = 60.0;

/// The floor under the zone width, in feet. Appendix 1 p.553.
pub const LEAST_ZONE_FEET: f64 = 3.0;

/// Why the appendix names a Zone 1' and this does not draw one.
pub const NO_ZONE_ONE_PRIME: &str =
    "Appendix 1 names a Zone 1' at GCp 0.9 (Fig 1-2, p.546) and its only zone plan, \
     Fig 1-9, draws 1, 2 and 3 and dimensions no 1' boundary. Where it begins is in \
     the ASCE 7 figure itself, which is not on this machine, so the field is one zone.";

/// One wind zone on one roof: the ground it holds and what the wind does to it.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    /// `1`, `2` or `3`, as the appendix numbers them.
    pub zone: String,
    /// What a roofer calls it: the field, the perimeter, a corner.
    pub label: String,
    /// The ground, in pieces — a perimeter band round a roof with a bump in it
    /// is one piece, and a corner region clipped by the outline can be several.
    pub shapes: Vec<Shape>,
    pub square_feet: f64,
    /// What was typed for this zone, in pounds a square foot. Absent where
    /// nobody typed one — never nought, which would read as a roof the wind
    /// does not pull on.
    pub design_psf: Option<f64>,
    /// The same at working load: the typed figure, reduced where it was typed
    /// as strength design.
    pub asd_psf: Option<f64>,
    /// Working load times the safety factor — the capacity the assembly needs.
    pub required_psf: Option<f64>,
    /// The rating that covers it: the next step of fifteen at or above.
    pub fm_class: Option<f64>,
}

/// One roof's zones, and everything they were set out from.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindPlan {
    pub condition_id: String,
    pub trace: usize,
    /// The traced outline, in feet.
    pub ring: Vec<Point>,
    pub square_feet: f64,
    pub standard: String,
    pub zone_rule: String,
    /// `strength` or `working load`, off which family of names was typed.
    pub basis: String,
    /// False where nothing was typed at all, so nothing is being claimed.
    pub basis_stated: bool,
    pub height_feet: Option<f64>,
    pub height_stated: bool,
    pub least_dimension_feet: f64,
    /// The zone width. Absent where no height was typed: the rule cannot be
    /// worked without one, and a band of nothing is not an answer.
    pub a_feet: Option<f64>,
    /// Which of the three terms in the rule actually set it.
    pub a_bound: String,
    pub zones: Vec<Zone>,
    pub zone_one_prime: String,
    /// Why the zones could not be set out, where they could not.
    pub refused: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvedWind {
    pub solve_id: u64,
    pub plans: Vec<WindPlan>,
}

/// The zone width, and which term of the rule set it.
///
/// `min(0.1 x least side, 0.4 x mean height)`, never under `max(0.04 x least
/// side, 3 ft)`.
pub fn dimension_a(least_dimension_feet: f64, height_feet: f64) -> (f64, &'static str) {
    let tenth = least_dimension_feet * 0.1;
    let by_height = height_feet * 0.4;
    let floor = (least_dimension_feet * 0.04).max(LEAST_ZONE_FEET);
    let (smaller, word) = if by_height < tenth {
        (by_height, "0.4 of the mean roof height")
    } else {
        (tenth, "a tenth of the least side")
    };
    if smaller < floor {
        let word = if least_dimension_feet * 0.04 >= LEAST_ZONE_FEET {
            "the floor at 4 per cent of the least side"
        } else {
            "the floor at 3 ft"
        };
        (floor, word)
    } else {
        (smaller, word)
    }
}

/// The rating that covers a wanted capacity: the next step of fifteen at or
/// above it, never under the lowest one published.
pub fn rating_for(required_psf: f64) -> f64 {
    if required_psf <= 0.0 {
        return LEAST_RATING_PSF;
    }
    let steps = (required_psf / RATING_STEP_PSF - 1e-9).ceil();
    (steps * RATING_STEP_PSF).max(LEAST_RATING_PSF)
}

/// The corners of a ring that turn outward, as (before, here, after).
///
/// The sign of the turn against the ring's own winding, which is the same test
/// `coping.rs` classes a mitre by. A corner turning into the roof is not a place
/// the wind peels from and Fig 1-9 draws no region at one.
fn outside_corners(ring: &[Point]) -> Vec<(Point, Point, Point)> {
    let winding = signed_area(ring);
    let n = ring.len();
    let mut out = Vec::new();
    for i in 0..n {
        let before = ring[(i + n - 1) % n];
        let here = ring[i];
        let after = ring[(i + 1) % n];
        let u = Point::new(here.x - before.x, here.y - before.y);
        let w = Point::new(after.x - here.x, after.y - here.y);
        if (u.x * w.y - u.y * w.x) * winding > 0.0 {
            out.push((before, here, after));
        }
    }
    out
}

/// The corner region at one corner: `a` back along the edge coming in, `a` on
/// along the edge going out. A square where the corner is square.
fn corner_region(before: Point, here: Point, after: Point, a: f64) -> Option<Shape> {
    let u = Point::new(here.x - before.x, here.y - before.y);
    let w = Point::new(after.x - here.x, after.y - here.y);
    let lu = u.x.hypot(u.y);
    let lw = w.x.hypot(w.y);
    if lu < 1e-9 || lw < 1e-9 {
        return None;
    }
    let u = Point::new(u.x / lu * a, u.y / lu * a);
    let w = Point::new(w.x / lw * a, w.y / lw * a);
    Some(Shape::of(vec![
        here,
        Point::new(here.x - u.x, here.y - u.y),
        Point::new(here.x - u.x + w.x, here.y - u.y + w.y),
        Point::new(here.x + w.x, here.y + w.y),
    ]))
}

/// Which way `offset_ring` has to be told to go to move a ring inboard.
///
/// Read off the answer rather than off a winding convention: the offset that
/// SHRINKS the ring is the one going in. A ring that offsets to nothing either
/// way is narrower than twice the band, and has no field left in it.
fn inboard_sign(ring: &[Point], distance: f64) -> Option<(f64, Shape)> {
    let outer = Shape::of(dedup_closing(ring)).square_feet();
    let mut best: Option<(f64, Shape)> = None;
    for sign in [1.0, -1.0] {
        let Some(shape) = offset_ring(ring, true, sign, distance) else { continue };
        let area = shape.square_feet();
        if area >= outer {
            continue;
        }
        if best.as_ref().map(|(_, had)| area < had.square_feet()).unwrap_or(true) {
            best = Some((sign, shape));
        }
    }
    best
}

/// The three zones of one outline, in order: field, perimeter, corners.
///
/// They tile the outline. The corner regions are cut out of the band and out of
/// the field both, so nothing is counted twice and nothing falls between: a
/// corner region at a square corner lies wholly inside the band anyway, and at
/// an acute one it reaches past it, and this way neither case needs its own rule.
pub fn zone_shapes(ring: &[Point], a: f64) -> [Vec<Shape>; 3] {
    let outline = Shape::of(dedup_closing(ring));
    let inner = inboard_sign(ring, a).map(|(_, shape)| shape);

    let mut corners: Vec<Shape> = Vec::new();
    for (before, here, after) in outside_corners(&dedup_closing(ring)) {
        let Some(quad) = corner_region(before, here, after, a) else { continue };
        let mine = common_with(&quad, std::slice::from_ref(&outline));
        let fresh = if corners.is_empty() { mine } else { take_out_all(&mine, &corners) };
        corners.extend(fresh);
    }

    let band = match &inner {
        Some(shape) => take_out(&outline, shape),
        None => vec![outline.clone()],
    };
    let perimeter = take_out_all(&band, &corners);
    let field = match &inner {
        Some(shape) => take_out_all(&common_with(shape, std::slice::from_ref(&outline)), &corners),
        None => Vec::new(),
    };
    [field, perimeter, corners]
}

/// What was typed for one zone, and on what basis.
struct Typed {
    psf: Option<f64>,
    strength: bool,
}

fn typed_for(c: &crate::model::Condition, zone: &str) -> Typed {
    match c.property(&format!("WIND_ASD_{zone}")) {
        Some(psf) => Typed { psf: Some(psf), strength: false },
        None => Typed { psf: c.property(&format!("WIND_PSF_{zone}")), strength: true },
    }
}

/// Solve the wind zones on every tapered field of the job.
///
/// One plan per laid-out field, because the zones belong to the roof the boards
/// are laid on: a second roof at a second elevation is a second building as far
/// as `a` is concerned, and it carries its own typed pressures.
pub fn solve(document: &Document, taper: &SolvedTaper, solve_id: u64) -> SolvedWind {
    let mut plans = Vec::new();
    for field in &taper.fields {
        let Some(c) = document.conditions.iter().find(|c| c.id == field.condition_id) else {
            continue;
        };
        plans.push(plan_of(c, &field.ring, field.condition_id.clone(), field.trace));
    }
    SolvedWind { solve_id, plans }
}

fn plan_of(
    c: &crate::model::Condition,
    ring: &[Point],
    condition_id: String,
    trace: usize,
) -> WindPlan {
    let outline = Shape::of(dedup_closing(ring));
    let square_feet = outline.square_feet();
    let least_dimension_feet = bounds(ring)
        .map(|b| (b.max_x - b.min_x).min(b.max_y - b.min_y))
        .unwrap_or(0.0);

    let height = c.property("ROOF_HEIGHT_FT");
    let typed = [("FIELD", "1", "the field"), ("PERIMETER", "2", "the perimeter"), ("CORNER", "3", "a corner")]
        .map(|(name, zone, label)| (typed_for(c, name), zone, label));
    let any_asd = typed.iter().any(|(t, _, _)| t.psf.is_some() && !t.strength);
    let any_strength = typed.iter().any(|(t, _, _)| t.psf.is_some() && t.strength);
    let basis_stated = any_asd || any_strength;
    let basis = if any_asd && any_strength {
        "mixed".to_string()
    } else if any_asd {
        "working load".to_string()
    } else {
        "strength".to_string()
    };

    let mut plan = WindPlan {
        condition_id,
        trace,
        ring: dedup_closing(ring),
        square_feet,
        standard: WIND_STANDARD.to_string(),
        zone_rule: ZONE_RULE.to_string(),
        basis,
        basis_stated,
        height_feet: height,
        height_stated: height.is_some(),
        least_dimension_feet,
        a_feet: None,
        a_bound: String::new(),
        zones: Vec::new(),
        zone_one_prime: NO_ZONE_ONE_PRIME.to_string(),
        refused: None,
    };

    let Some(height) = height else {
        plan.refused = Some(
            "nobody has typed a mean roof height, and the corner and edge zones are \
             set out from it"
                .to_string(),
        );
        return plan;
    };
    if height <= 0.0 || least_dimension_feet <= 0.0 {
        plan.refused = Some("this roof has no height or no width to set the zones out from".to_string());
        return plan;
    }

    let (a, bound) = dimension_a(least_dimension_feet, height);
    plan.a_feet = Some(a);
    plan.a_bound = bound.to_string();

    let [one, two, three] = zone_shapes(ring, a);
    for (shapes, (t, zone, label)) in [one, two, three].into_iter().zip(typed) {
        let square_feet: f64 = shapes.iter().map(Shape::square_feet).sum();
        // An uplift is a pull, so a figure typed either way means the same
        // thing; what is NOT the same is a figure nobody typed, and that stays
        // absent all the way down the chain rather than becoming a nought that
        // would rate at the lowest published class.
        let design_psf = t.psf.map(f64::abs);
        let asd_psf =
            design_psf.map(|psf| if t.strength { psf * ASD_FACTOR } else { psf });
        let required_psf = asd_psf.map(|psf| psf * SAFETY_FACTOR);
        plan.zones.push(Zone {
            zone: zone.to_string(),
            label: label.to_string(),
            shapes,
            square_feet,
            design_psf,
            asd_psf,
            required_psf,
            fm_class: required_psf.map(rating_for),
        });
    }
    if !basis_stated {
        plan.refused = Some(
            "nobody has typed a design pressure for this roof, so the zones are set out \
             and nothing is rated"
                .to_string(),
        );
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 60 by 40 roof at 30 ft: a tenth of the least side is 4 and 0.4 of the
    /// height is 12, so the tenth binds. The floor is 1.6, which does not.
    #[test]
    fn the_zone_width_takes_the_smaller_of_the_two_and_never_less_than_the_floor() {
        let (a, bound) = dimension_a(40.0, 30.0);
        assert_eq!(a, 4.0);
        assert_eq!(bound, "a tenth of the least side");

        // A tall building: 0.4 of the height is 40 against a tenth at 4, so the
        // tenth still binds. Height only binds on a squat wide roof.
        let (a, _) = dimension_a(40.0, 100.0);
        assert_eq!(a, 4.0);

        // A low one: 0.4 of 8 ft is 3.2 against a tenth at 4, so the height binds.
        let (a, bound) = dimension_a(40.0, 8.0);
        assert!((a - 3.2).abs() < 1e-9);
        assert_eq!(bound, "0.4 of the mean roof height");

        // Lower still, and the floor takes over: 0.4 of 5 ft is 2, under both
        // 4 per cent of 40 (1.6) and the 3 ft floor.
        let (a, bound) = dimension_a(40.0, 5.0);
        assert_eq!(a, 3.0);
        assert_eq!(bound, "the floor at 3 ft");

        // On a big roof 4 per cent of the least side is the higher floor.
        let (a, bound) = dimension_a(200.0, 10.0);
        assert_eq!(a, 8.0);
        assert_eq!(bound, "the floor at 4 per cent of the least side");
    }

    /// p.565: an assembly rated FM 1-90 covers a design uplift load of 45 psf.
    /// So 45 at working load, doubled, is exactly 90 and the ladder steps in 15s.
    #[test]
    fn the_rating_is_the_next_fifteen_at_or_above_what_the_zone_wants() {
        assert_eq!(rating_for(45.0 * SAFETY_FACTOR), 90.0);
        assert_eq!(rating_for(90.0), 90.0);
        assert_eq!(rating_for(90.1), 105.0);
        assert_eq!(rating_for(1.0), LEAST_RATING_PSF);
        assert_eq!(rating_for(0.0), LEAST_RATING_PSF);
        assert_eq!(rating_for(146.4), 150.0);
    }

    /// A strength-design pressure is reduced and then doubled; one already at
    /// working load is only doubled.
    #[test]
    fn the_basis_decides_whether_the_reduction_is_applied_at_all() {
        let strength = 122.0;
        let working = strength * ASD_FACTOR;
        assert!((working - 73.2).abs() < 1e-9);
        assert_eq!(rating_for(working * SAFETY_FACTOR), 150.0);
        // The same figure typed as working load already: no reduction, and it
        // wants a much higher rating. Which is why the basis is not a guess.
        assert_eq!(rating_for(strength * SAFETY_FACTOR), 255.0);
    }

    fn square(side: f64) -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(side, 0.0),
            Point::new(side, side),
            Point::new(0.0, side),
        ]
    }

    /// The three zones tile the roof, and each is the size the rule makes it.
    ///
    /// On a 40 ft square at a = 4: four corner regions of 16 SF, a band 4 ft wide
    /// round a 40 ft square less those corners, and a 32 by 32 field.
    #[test]
    fn the_three_zones_tile_the_roof_and_are_the_size_the_rule_makes_them() {
        let ring = square(40.0);
        let [one, two, three] = zone_shapes(&ring, 4.0);
        let sf = |shapes: &[Shape]| shapes.iter().map(Shape::square_feet).sum::<f64>();
        assert!((sf(&three) - 4.0 * 16.0).abs() < 0.01, "corners: {}", sf(&three));
        assert!((sf(&one) - 32.0 * 32.0).abs() < 0.01, "field: {}", sf(&one));
        assert!((sf(&two) - (1600.0 - 1024.0 - 64.0)).abs() < 0.01, "perimeter: {}", sf(&two));
        assert!((sf(&one) + sf(&two) + sf(&three) - 1600.0).abs() < 0.01);
    }

    /// A roof narrower than two bands has no field left, and says so by having
    /// none rather than by reporting a negative one.
    #[test]
    fn a_roof_narrower_than_two_bands_is_all_perimeter_and_corner() {
        let ring = square(6.0);
        let [one, two, three] = zone_shapes(&ring, 4.0);
        let sf = |shapes: &[Shape]| shapes.iter().map(Shape::square_feet).sum::<f64>();
        assert_eq!(sf(&one), 0.0);
        assert!((sf(&two) + sf(&three) - 36.0).abs() < 0.01);
    }

    /// A corner that turns INTO the roof gets no corner region.
    ///
    /// An L, traced counter-clockwise: five corners turn out and one turns in.
    #[test]
    fn a_corner_that_turns_into_the_roof_is_not_a_corner_zone() {
        let l = vec![
            Point::new(0.0, 0.0),
            Point::new(40.0, 0.0),
            Point::new(40.0, 20.0),
            Point::new(20.0, 20.0),
            Point::new(20.0, 40.0),
            Point::new(0.0, 40.0),
        ];
        assert_eq!(outside_corners(&l).len(), 5);
        let [one, two, three] = zone_shapes(&l, 4.0);
        let sf = |shapes: &[Shape]| shapes.iter().map(Shape::square_feet).sum::<f64>();
        // Five corner regions of 16 SF, none of them overlapping on this shape.
        assert!((sf(&three) - 5.0 * 16.0).abs() < 0.01, "corners: {}", sf(&three));
        assert!((sf(&one) + sf(&two) + sf(&three) - 1200.0).abs() < 0.01);
    }
}
