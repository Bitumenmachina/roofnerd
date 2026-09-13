// ── The wind zones on the demo roof ────────────────────────────────────────
// The rule is Appendix 1's and the pressures are the job's. Nothing here types a
// zone width or an uplift rating: every figure below is worked out from what the
// job states, the way a designer works it out from the same two numbers.

use std::fs;
use std::path::PathBuf;

use roofnerd_core::geom::{bounds, Shape};
use roofnerd_core::model::Document;
use roofnerd_core::wind::{self, ASD_FACTOR, SAFETY_FACTOR};

fn demo() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../jobs/demo-job");
    let part = |rel: &str| -> serde_json::Value {
        let raw = fs::read_to_string(root.join(rel)).expect("the demo job is in the repository");
        serde_json::from_str(&raw).expect("the demo job is json")
    };
    serde_json::json!({ "conditions": part("conditions.json"), "pages": part("pages/pages.json") })
}

fn solved() -> (Document, wind::SolvedWind) {
    let value = demo();
    let document = Document::read(&value).expect("the demo job reads");
    let taper = roofnerd_core::taper::solve(&document, 1);
    let wind = wind::solve(&document, &taper, 1);
    (document, wind)
}

fn typed(document: &Document, name: &str) -> f64 {
    document
        .conditions
        .iter()
        .find(|c| c.id == "c-field")
        .and_then(|c| c.property(name))
        .unwrap_or_else(|| panic!("the demo types a {name} on its roof"))
}

/// The zone width comes off the two numbers the rule names, and says which won.
///
/// The demo's roof is 108 by 92 on the plan and the job types a mean roof height.
/// A tenth of the least side is 9.2 and 0.4 of the height is less, so the height
/// binds; the floor — 4 per cent of 92, or 3 ft, whichever is greater — is under
/// both and does not. Not one of those figures is typed here: the sides come off
/// the traced outline and the height off the job.
#[test]
fn the_zone_width_is_worked_out_from_the_roof_and_not_typed() {
    let (document, solved) = solved();
    let plan = solved.plans.first().expect("the demo's roof has a wind plan");
    let height = typed(&document, "ROOF_HEIGHT_FT");

    let box_of = bounds(&plan.ring).expect("the outline has a bounding box");
    let least = (box_of.max_x - box_of.min_x).min(box_of.max_y - box_of.min_y);
    assert_eq!(plan.least_dimension_feet, least);

    let a = plan.a_feet.expect("the zones are set out");
    assert_eq!(a, height * 0.4, "0.4 of the mean roof height is what binds here");
    assert!(a < least * 0.1, "a tenth of the least side is the looser of the two");
    assert!(a > (least * 0.04).max(3.0), "and the floor is looser still");
    assert_eq!(plan.a_bound, "0.4 of the mean roof height");
}

/// The three bands tile the roof: nothing is in two of them, nothing is in none.
#[test]
fn the_bands_tile_the_roof_to_a_hundredth_of_a_square_foot() {
    let (_, solved) = solved();
    let plan = solved.plans.first().expect("the demo's roof has a wind plan");
    assert_eq!(plan.zones.len(), 3, "three zones, and the fourth is reported missing");

    let summed: f64 = plan.zones.iter().map(|z| z.square_feet).sum();
    assert!(
        (summed - plan.square_feet).abs() < 0.01,
        "the zones come to {summed} against the roof's own {}",
        plan.square_feet
    );

    // And none of them overlaps any other, which the sum alone would not prove
    // if one band were short by exactly as much as another was long.
    for (i, mine) in plan.zones.iter().enumerate() {
        for theirs in plan.zones.iter().skip(i + 1) {
            let shared: f64 = mine
                .shapes
                .iter()
                .flat_map(|s| {
                    roofnerd_core::geom::common_with(s, &theirs.shapes)
                        .into_iter()
                        .map(|piece| piece.square_feet())
                })
                .sum();
            assert!(
                shared < 0.01,
                "{} and {} share {shared} SF",
                mine.label,
                theirs.label
            );
        }
    }
}

/// Every corner that turns outward gets a corner region, and none that turns in.
///
/// The demo's outline steps in twice at the bump on its south side, so six of its
/// eight corners face out. Six regions of `a` by `a` is the most the corner zone
/// can come to — and it comes to less, because two of those corners are closer
/// together than the band is wide: the one at the bump's far side and the one at
/// the east wall share a patch of roof, and shared ground is corner ground once,
/// not twice. That is the whole reason the regions are unioned rather than added.
#[test]
fn the_corner_zone_is_one_region_at_each_corner_that_turns_outward() {
    let (_, solved) = solved();
    let plan = solved.plans.first().expect("the demo's roof has a wind plan");
    let a = plan.a_feet.expect("the zones are set out");
    let corner = plan.zones.iter().find(|z| z.zone == "3").expect("there is a corner zone");

    // Count the turns the same way the solver does, off the traced ring.
    let ring = &plan.ring;
    let n = ring.len();
    let winding: f64 = (0..n)
        .map(|i| {
            let p = ring[i];
            let q = ring[(i + 1) % n];
            p.x * q.y - q.x * p.y
        })
        .sum();
    let outward = (0..n)
        .filter(|i| {
            let before = ring[(i + n - 1) % n];
            let here = ring[*i];
            let after = ring[(i + 1) % n];
            let ux = here.x - before.x;
            let uy = here.y - before.y;
            let wx = after.x - here.x;
            let wy = after.y - here.y;
            (ux * wy - uy * wx) * winding > 0.0
        })
        .count();
    assert_eq!(outward, 6, "the bump turns in twice");
    let most = outward as f64 * a * a;
    assert!(
        corner.square_feet < most,
        "two of the regions overlap on this outline, so the zone is under {most} SF"
    );
    assert!(
        most - corner.square_feet < a * a,
        "the corner zone is {} SF, which is short by more than a whole region",
        corner.square_feet
    );
}

/// The chain, on the demo's own typed pressures: reduce, double, step up.
///
/// The job types strength-design pressures, so each is reduced by 0.6 to working
/// load, doubled for the safety factor, and covered by the next published rating
/// of fifteen at or above. The corner is the one worth reading: a typed strength
/// pressure at working load and doubled lands between two steps, and the rating
/// taken is the one ABOVE.
#[test]
fn each_zone_asks_for_the_rating_its_own_pressure_makes_it_need() {
    let (document, solved) = solved();
    let plan = solved.plans.first().expect("the demo's roof has a wind plan");
    assert_eq!(plan.basis, "strength");
    assert!(plan.basis_stated);

    for (zone, name) in [("1", "WIND_PSF_FIELD"), ("2", "WIND_PSF_PERIMETER"), ("3", "WIND_PSF_CORNER")] {
        let z = plan.zones.iter().find(|z| z.zone == zone).expect("the zone is solved");
        let design = typed(&document, name);
        let asd = z.asd_psf.expect("a zone with a typed pressure works at a load");
        let required = z.required_psf.expect("and asks for a capacity");
        let rating = z.fm_class.expect("and a rating that covers it");
        assert_eq!(z.design_psf, Some(design));
        assert!((asd - design * ASD_FACTOR).abs() < 1e-9);
        assert!((required - design * ASD_FACTOR * SAFETY_FACTOR).abs() < 1e-9);
        assert!(rating >= required, "the rating has to cover what is wanted");
        assert!(rating - required < 15.0, "and it is the next step up, not two steps");
    }

    // The three are in order and none of them ties: a corner is worse than an
    // edge and an edge is worse than the field, which is the premise the whole
    // appendix rests on (p.553).
    let by = |zone: &str| {
        plan.zones
            .iter()
            .find(|z| z.zone == zone)
            .and_then(|z| z.required_psf)
            .expect("solved")
    };
    assert!(by("1") < by("2"), "the perimeter is worse than the field");
    assert!(by("2") < by("3"), "and a corner is worse than the perimeter");
}

/// The zones say the standard they were set out to, and why there is no fourth.
#[test]
fn the_plan_says_what_it_was_drawn_to_and_what_it_could_not_draw() {
    let (_, solved) = solved();
    let plan = solved.plans.first().expect("the demo's roof has a wind plan");
    assert!(plan.standard.contains("ASCE 7-22"));
    assert!(plan.zone_rule.contains("Appendix 1"));
    assert!(plan.zone_one_prime.contains("Zone 1'"));
    assert_eq!(plan.refused, None, "this roof states everything the zones need");
}

/// A roof with no typed height is not given one, and says what is missing.
#[test]
fn a_roof_with_no_typed_height_has_no_zones_and_says_why() {
    let value = serde_json::json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area",
                "properties": { "T": 0.5, "TAPER": 0.25, "WIND_PSF_CORNER": 100.0 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                    { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 }
                ] }]
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
                "traces": [{ "pageId": "page-1", "points": [{ "x": 20.0, "y": 20.0 }] }]
            }
        ],
    });
    let document = Document::read(&value).expect("it reads");
    let taper = roofnerd_core::taper::solve(&document, 1);
    let solved = wind::solve(&document, &taper, 1);
    let plan = solved.plans.first().expect("there is still a plan, saying nothing was solved");
    assert_eq!(plan.a_feet, None, "a band of nothing is not an answer");
    assert!(plan.zones.is_empty());
    let why = plan.refused.as_deref().unwrap_or("");
    assert!(why.contains("mean roof height"), "it says what is missing: {why}");
}

/// A pressure typed as already at working load is not reduced a second time.
#[test]
fn a_pressure_typed_at_working_load_is_taken_as_it_stands() {
    let value = serde_json::json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area",
                "properties": {
                    "T": 0.5, "TAPER": 0.25, "ROOF_HEIGHT_FT": 20.0,
                    "WIND_ASD_FIELD": 30.0, "WIND_ASD_PERIMETER": 40.0, "WIND_ASD_CORNER": 45.0
                },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                    { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 }
                ] }]
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
                "traces": [{ "pageId": "page-1", "points": [{ "x": 20.0, "y": 20.0 }] }]
            }
        ],
    });
    let document = Document::read(&value).expect("it reads");
    let taper = roofnerd_core::taper::solve(&document, 1);
    let solved = wind::solve(&document, &taper, 1);
    let plan = solved.plans.first().expect("the plan is solved");
    assert_eq!(plan.basis, "working load");
    let corner = plan.zones.iter().find(|z| z.zone == "3").expect("solved");
    assert_eq!(corner.asd_psf, Some(45.0), "it was typed at working load and stays there");
    // p.565: an assembly rated FM 1-90 covers a design uplift load of 45 psf.
    assert_eq!(corner.fm_class, Some(90.0));
}

/// A band cannot be wider than the roof it is on.
#[test]
fn the_pieces_of_every_zone_are_real_ground_and_not_slivers() {
    let (_, solved) = solved();
    let plan = solved.plans.first().expect("the demo's roof has a wind plan");
    for zone in &plan.zones {
        let summed: f64 = zone.shapes.iter().map(Shape::square_feet).sum();
        assert!((summed - zone.square_feet).abs() < 1e-6);
        assert!(zone.square_feet > 0.0, "{} holds no ground at all", zone.label);
    }
}
