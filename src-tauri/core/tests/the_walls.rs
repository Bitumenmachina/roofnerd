// ── C3's check: the walls, the curbs, the coping ────────────────────────────
// Rust tests against `walls.rs` and `coping.rs`, each with the arithmetic it
// proves worked out by hand in the doc comment beside it — never against a
// number this program printed. Two read the demo job on disk, the way
// `lib.rs`'s own pin does; the rest build the smallest fixture that puts the
// claim beyond doubt.

use roofnerd_core::geom::{offset_ring, signed_area, Point};
use roofnerd_core::{taper, walls, Document};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// The demo job, read off disk the way the shell reads it.
fn demo() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../jobs/demo-job");
    let part = |rel: &str| -> serde_json::Value {
        let raw = fs::read_to_string(root.join(rel)).expect("the demo job is in the repository");
        serde_json::from_str(&raw).expect("the demo job is json")
    };
    json!({ "conditions": part("conditions.json"), "pages": part("pages/pages.json") })
}

fn solved_walls(document: &serde_json::Value) -> roofnerd_core::SolvedWalls {
    roofnerd_core::solve(document, 1).expect("the job solves").walls
}

/// `FLASH_SF` is the exact sum of trapezoids, not the two-point guess a naive
/// reading would make.
///
/// A 40 by 40 field, one drain at (20, 0) on the south wall — a four-way
/// layout with no `LAYOUT` typed. The west wall (x = 0) stands `WALL` = 6 in
/// thick, so its inboard face is x = 0.5. There, along = |0.5 − 20| = 19.5 ft
/// exactly; across = y. Along governs while y < 19.5 (thickness pinned at
/// 0.5 + 0.25 x 19.5 = 5 3/8 in); across takes over past it
/// (thickness = 0.5 + 0.25y). `FLASHUP` is left absurdly large (1000 in) so
/// the wall's own remaining height is what binds, and `BOARDS` is left
/// untyped so nothing but that height caps the package either — the run's own
/// height (36 in) is the only ceiling in play, and it is the point of this
/// test that the ceiling still bends where the ROOF bends, at y = 19.5,
/// without a board ceiling or a `FLASHUP` ceiling anywhere near it.
///
///   clearance(y) = 36 − thickness(0.5, y)
///   y in [0, 19.5]     36 − 5.375 = 30.625, flat
///   y in [19.5, 40]    36 − (0.5 + 0.25y) = 35.5 − 0.25y, falling to 25.5
///
///   area, inch-feet  30.625 x 19.5 + (30.625 + 25.5)/2 x 20.5
///                   = 597.1875 + 575.28125 = 1172.46875
///   FLASH_SF         1172.46875 / 12 = 97.705729166...
///
/// A two-point trapezoid over the whole 40 ft run — ends only, no break at
/// 19.5 — would answer (30.625 + 25.5)/2 x 40 / 12 = 93.541666..., which is
/// what this test refuses.
#[test]
fn flash_sf_is_the_exact_sum_of_trapezoids_on_a_sloping_run() {
    let doc = json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area",
                "properties": { "T": 0.5, "TAPER": 0.25 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                    { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 },
                ] }],
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
                "traces": [{ "pageId": "page-1", "points": [{ "x": 20.0, "y": 0.0 }] }],
            },
            {
                "id": "c-west", "kind": "line",
                "properties": { "H": 3.0, "WALL": 6.0, "FLASHUP": 1000.0 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 0.0, "y": 40.0 },
                ] }],
            },
        ],
    });
    let walls = solved_walls(&doc);
    let run = walls.runs.iter().find(|r| r.condition_id == "c-west").expect("the west wall solved");
    assert!(run.inboard.is_some(), "a typed WALL gets an inboard face");

    let want_sf = 1172.46875 / 12.0;
    let got_sf = run.flash_square_feet.expect("the roof under the west wall is traced");
    assert!((got_sf - want_sf).abs() < 1e-6, "FLASH_SF came to {got_sf}, wanted {want_sf}");

    let naive = (30.625 + 25.5) / 2.0 * 40.0 / 12.0;
    assert!((got_sf - naive).abs() > 1.0, "the exact sum should not agree with the two-point guess");

    let want_min = 25.5;
    let got_min = run.flash_min_inches.expect("a least height is on the run");
    assert!((got_min - want_min).abs() < 1e-6, "FLASH_MIN_IN came to {got_min}, wanted {want_min}");
}

/// A curb 1 ft tall over a flat 6 in build-up: `FLASHUP` untyped defaults to
/// the wall's own height, 12 in; the roof leaves it 12 − 6 = 6 in of climb,
/// which is 2 in short of NRCA's 8 in minimum (p.285) — `CURB_RAISE_IN` = 2.
///
/// No taper anywhere in this fixture: the field is a flat, untapered area with
/// one item on it, which is the smallest thing `finished_roof_inches` can be
/// asked about and the reading everything else in this test hangs off.
#[test]
fn a_short_curb_over_a_flat_build_up_is_refused_by_two_inches() {
    let doc = json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area",
                "items": [{ "id": "l-1", "description": "Roof insulation, 6 in", "layer": "insulation", "thickness": 6.0 }],
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                    { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 },
                ] }],
            },
            {
                "id": "c-curb", "kind": "line", "role": "curb",
                "properties": { "H": 1.0 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 10.0, "y": 10.0 }, { "x": 14.0, "y": 10.0 },
                    { "x": 14.0, "y": 14.0 }, { "x": 10.0, "y": 14.0 }, { "x": 10.0, "y": 10.0 },
                ] }],
            },
        ],
    });
    let walls = solved_walls(&doc);
    let run = walls.runs.iter().find(|r| r.condition_id == "c-curb").expect("the curb solved");
    assert!(!run.flashup_stated, "nothing typed FLASHUP");
    assert_eq!(run.flashup_inches, 12.0, "it defaults to the wall's own height");
    assert_eq!(run.flash_min_inches, Some(6.0));
    assert_eq!(run.curb_raise_inches, Some(2.0));
}

/// A wall standing over nothing traced at all is refused, not zero.
#[test]
fn a_wall_over_no_traced_roof_is_refused_rather_than_zero() {
    let doc = json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [{
            "id": "c-wall", "kind": "line",
            "properties": { "H": 3.0 },
            "traces": [{ "pageId": "page-1", "points": [
                { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
            ] }],
        }],
    });
    let walls = solved_walls(&doc);
    let run = &walls.runs[0];
    assert_eq!(run.flash_min_inches, None);
    assert_eq!(run.flash_square_feet, None);
    assert!(run.refused.is_some());
}

/// Gauge steps at exactly 8 and 10 in of face height, read through the whole
/// solve rather than off `coping::gauge_of` directly — the same claim
/// `coping.rs`'s own unit test makes, proved this time end to end from a
/// job's `W` and `STRETCHOUT`.
#[test]
fn gauge_steps_at_eight_and_ten_inches_through_the_whole_solve() {
    let wall = |stretchout: f64| {
        json!({
            "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
            "conditions": [
                {
                    "id": "c-field", "kind": "area",
                    "traces": [{ "pageId": "page-1", "points": [
                        { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                        { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 },
                    ] }],
                },
                {
                    "id": "c-wall", "kind": "line",
                    "properties": { "H": 3.0, "W": 1.0, "STRETCHOUT": stretchout },
                    "traces": [{ "pageId": "page-1", "points": [
                        { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                        { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 }, { "x": 0.0, "y": 0.0 },
                    ] }],
                },
            ],
        })
    };
    // A 12 in cap: a leg of exactly 8 in is `12 + 2 x 8 = 28` in of girth.
    let at_eight = solved_walls(&wall(28.0));
    let coping = &at_eight.runs[0].coping;
    assert_eq!(coping.face_height_inches, Some(8.0));
    assert_eq!(coping.gauge_ga, Some(24));

    // A leg of exactly 10 in is `12 + 2 x 10 = 32` in of girth.
    let at_ten = solved_walls(&wall(32.0));
    let coping = &at_ten.runs[0].coping;
    assert_eq!(coping.face_height_inches, Some(10.0));
    assert_eq!(coping.gauge_ga, Some(22));

    // And past it, reported rather than invented.
    let past_ten = solved_walls(&wall(32.01));
    let coping = &past_ten.runs[0].coping;
    assert!(coping.face_height_inches.unwrap() > 10.0);
    assert_eq!(coping.gauge_ga, None);
}

/// `COPING_PCS_EA` and the outside/inside split on the demo's own 9-vertex
/// parapet: 8 distinct corners, 6 outside and 2 inside (`coping.rs`'s own unit
/// test works this out by hand off the same ring), 400 ft of run at a 10 ft
/// section — `ceil(400 / 10) + 8 = 48`.
#[test]
fn the_demo_s_parapet_pieces_come_to_sections_plus_its_own_corners() {
    let walls = solved_walls(&demo());
    let run = walls.runs.iter().find(|r| r.condition_id == "c-parapet").expect("the parapet solved");
    let coping = &run.coping;
    assert_eq!((coping.corners_outside, coping.corners_inside), (6, 2));
    assert_eq!(coping.run_feet, 400.0);
    assert_eq!(coping.section_feet, 10.0);
    assert_eq!(coping.pieces_ea, 48);
    assert_eq!(coping.cleat_feet, 800.0);
}

/// `cavalier_contours`' offset, proved on a concave ring this crate did not
/// already have — an L, not the demo's rectangle-with-a-bump.
///
/// (0,0) (10,0) (10,4) (4,4) (4,10) (0,10), CCW, so the interior is on the
/// left of each edge and `sign` is `1.0`. Every edge is axis-aligned, so each
/// of the five CONVEX corners insets by exactly (±1, ±1) — (0,0) to (1,1),
/// (10,0) to (9,1), (10,4) to (9,3), (4,10) to (3,9), (0,10) to (1,9), by the
/// same construction the demo's own parapet corners use.
///
/// The REFLEX corner at (4, 4) is the point of this test, and it is where a
/// per-segment "turn toward the middle" heuristic and a real offset diverge
/// hardest: each of the two edges meeting there offsets on its own line first
/// — the (10,4)-(4,4) edge to the line y = 3, the (4,4)-(4,10) edge to x = 3 —
/// and those two RAW offset segments do not reach each other at all: one
/// covers x in [4, 10] at y = 3, the other y in [4, 10] at x = 3, and their
/// lines only cross at (3, 3) if extended past where either segment actually
/// runs. A hand-rolled miter would have to invent a rule for stretching a
/// segment past its own end on a reflex corner (and could not tell how far);
/// `cavalier_contours` instead keeps each raw offset's own true end — (4, 3)
/// and (3, 4) — and joins them directly, which is what an offset polyline
/// actually is at a corner an extension cannot reach without assuming a rule
/// nothing states.
///
/// So the offset L has SEVEN corners, not six: (1,1) (9,1) (9,3) (4,3) (3,4)
/// (3,9) (1,9) — a shoelace sum of its own, worked out by hand and not read
/// off what the library returned, comes to 57, so 28.5 SF.
#[test]
fn the_offset_is_proved_on_an_l_shaped_ring() {
    let ring = vec![
        Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(10.0, 4.0),
        Point::new(4.0, 4.0), Point::new(4.0, 10.0), Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    assert!((signed_area(&ring[..6]).abs() / 2.0 - 64.0).abs() < 1e-9, "the outer L is 64 SF");
    assert!(signed_area(&ring[..6]) > 0.0, "traced CCW, so the interior is on the left");

    let inboard = offset_ring(&ring, true, 1.0, 1.0).expect("a concave ring still offsets");
    assert_eq!(inboard.ring.len(), 7, "a direct join at the reflex corner, not a mitre");

    let has = |x: f64, y: f64| {
        inboard.ring.iter().any(|p| (p.x - x).abs() < 1e-6 && (p.y - y).abs() < 1e-6)
    };
    for (x, y) in [(1.0, 1.0), (9.0, 1.0), (9.0, 3.0), (3.0, 9.0), (1.0, 9.0)] {
        assert!(has(x, y), "convex corner ({x}, {y}) missing from {:?}", inboard.ring);
    }
    assert!(has(4.0, 3.0) && has(3.0, 4.0), "the reflex corner's own raw offset ends, joined directly");
    assert!(!has(3.0, 3.0), "not mitred to a point neither raw offset segment actually reaches");

    let want = 28.5;
    let area = signed_area(&inboard.ring).abs() / 2.0;
    assert!((area - want).abs() < 1e-9, "the offset L came to {area} SF, wanted {want}");
}

/// `taper` is threaded through even where a run has no drain within reach of
/// it, so that a wall over a tapered field still reads the field's own
/// thickness rather than treating it as untapered ground.
#[test]
fn a_wall_over_a_tapered_field_reads_the_layout_not_a_flat_guess() {
    let doc = demo();
    let document = Document::read(&doc).expect("the demo document reads");
    let solved = taper::solve(&document, 1);
    let sf = walls::finished_roof_inches(&document, &solved, Point::new(36.0, 98.0));
    // The first drain itself: 0.5 in of taper, plus the demo's own cover board
    // (0.5 in) and membrane (0.06 in) — 1.06 in, off the pinned figures in
    // `lib.rs`'s own test and `jobs/demo-job/conditions.json`.
    assert!((sf.unwrap() - 1.06).abs() < 1e-6, "{:?}", sf);
}
