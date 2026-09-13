// ── C6's check: the verbs, on the roof the demo states ─────────────────────
// `verbs.rs` on its own is proved in the module's own tests, against a forty-foot
// square anybody can do in their head. These are the other half: the same three
// questions asked of the DEMO roof, with the arithmetic worked out here from what
// the job says — the drains, the axes, the thickness at the drain and the slope —
// rather than from anything this program printed.
//
// The demo, in feet: a field 14…122 by 42…134 with a bump in its north side; three
// drains in a row on y = 98 at x 36, 74 and 112; T = 1/2 in at the drain,
// TAPER = 1/4 in per foot, BOARDS = 3; a parapet 3 ft 6 in and a curb 2 ft, so the
// lowest thing the roof flashes against is 24 in and the cap is 14 in.

use roofnerd_core::geom::Point;
use roofnerd_core::{between, cricket_behind, high_and_low, taper, Between, Document, HighAndLow};
use std::fs;
use std::path::PathBuf;

/// The demo job, read off disk the way the shell reads it.
fn demo() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../jobs/demo-job");
    let part = |rel: &str| -> serde_json::Value {
        let raw = fs::read_to_string(root.join(rel)).expect("the demo job is in the repository");
        serde_json::from_str(&raw).expect("the demo job is json")
    };
    serde_json::json!({ "conditions": part("conditions.json"), "pages": part("pages/pages.json") })
}

fn open() -> (Document, roofnerd_core::SolvedTaper) {
    let document = Document::read(&demo()).expect("the demo job reads");
    let solved = taper::solve(&document, 1);
    (document, solved)
}

/// Every drainage point on the demo, in feet, off the job rather than the solve.
fn drains(document: &Document) -> Vec<Point> {
    let scales = document.scales();
    let mut out = Vec::new();
    for c in &document.conditions {
        if !c.has_role("drain") || c.hidden {
            continue;
        }
        for trace in &c.traces {
            out.extend(document.feet(trace, &scales));
        }
    }
    out
}

/// The field's ring, off the job.
fn field_ring(document: &Document) -> Vec<Point> {
    let scales = document.scales();
    let c = document
        .conditions
        .iter()
        .find(|c| c.kind == "area" && !c.hidden && c.property("TAPER").is_some())
        .expect("the demo has a tapered area");
    document.feet(&c.traces[0], &scales)
}

/// Two picks on the demo make the slope the run between them implies.
///
/// The pick is put square to the boards from the first drain — the boards run east
/// and west on this roof, because its longest side is the south wall — so the run
/// along the boards IS the distance, and the arithmetic has nothing in it but a
/// subtraction. Picking a thickness at each end then fixes the slope: the fall
/// divided by the run, which is the solve's own `T + TAPER × run` turned round.
#[test]
fn two_picks_on_the_demo_make_the_slope_the_run_between_them_implies() {
    let (document, solved) = open();
    let first = drains(&document)[0];
    let field = &solved.fields[0];

    // Twenty feet along the boards from the drain, staying on the same line
    // across, so the run is exactly twenty and the answer cannot be a diagonal.
    let high = Point::new(
        first.x + field.axis_along.x * 20.0,
        first.y + field.axis_along.y * 20.0,
    );

    let answer = high_and_low(
        &document,
        &solved,
        &HighAndLow {
            high,
            low: first,
            high_inches: Some(5.5),
            low_inches: Some(0.5),
        },
    );
    assert_eq!(answer.refused, None, "the pick is on the roof and at a drain");
    assert!((answer.run_feet - 20.0).abs() < 1e-6, "{} ft of run", answer.run_feet);
    // Five inches of fall over twenty feet is a quarter of an inch to the foot,
    // which happens to be what the demo already states — so this is the one case
    // where the answer can be checked against the job as well as against the sum.
    let slope = answer.taper_inches_per_foot.expect("a slope");
    assert!((slope - 0.25).abs() < 1e-9, "{slope} in 12");
    assert_eq!(answer.thickness_inches, Some(0.5));

    // And the strip's own readings are the solve's, not this file's: the roof at
    // the drain is what the job types, and the ceiling is the board count's.
    assert_eq!(answer.low_now_inches, Some(0.5));
    assert_eq!(answer.cap_inches, Some(14.0));
    assert_eq!(answer.drain_number, Some(1));
}

/// A pick past where the boards run out is a note, not a refusal — and the note
/// names the ceiling the roof actually stops at.
#[test]
fn a_high_pick_past_the_ceiling_says_the_roof_goes_flat() {
    let (document, solved) = open();
    let first = drains(&document)[0];
    let field = &solved.fields[0];
    let high = Point::new(
        first.x + field.axis_along.x * 20.0,
        first.y + field.axis_along.y * 20.0,
    );
    let answer = high_and_low(
        &document,
        &solved,
        &HighAndLow { high, low: first, high_inches: Some(18.0), low_inches: Some(0.5) },
    );
    assert_eq!(answer.refused, None);
    assert_eq!(answer.cap_inches, Some(14.0));
    assert!(answer.note.expect("a sentence").contains("flat"));
}

/// A ruler between two drains reads the distance between them, and both ends are
/// at the thickness the job states at a drain.
///
/// The demo's drains stand in a row along the boards, so this is a straight
/// subtraction of two x figures, and the roof is half an inch at each end because
/// that is what a drain IS in a tapered package.
#[test]
fn a_ruler_between_two_drains_reads_the_distance_and_half_an_inch_at_each_end() {
    let (document, solved) = open();
    let all = drains(&document);
    let (a, b) = (all[0], all[1]);
    let answer = between(&solved, &Between { a, b });

    let by_hand = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    assert!((answer.feet - by_hand).abs() < 1e-9, "{} ft", answer.feet);
    assert!((answer.a_inches.expect("a depth") - 0.5).abs() < 1e-9);
    assert!((answer.b_inches.expect("a depth") - 0.5).abs() < 1e-9);
    // Two drains on one field at one stated thickness: no rise between them.
    assert!(answer.rise_inches.expect("a rise").abs() < 1e-9);
}

/// A ruler with one end off the roof still has a length, and says nothing it
/// cannot say about the end that is off.
#[test]
fn a_ruler_off_the_roof_is_a_length_and_two_honest_blanks() {
    let (document, solved) = open();
    let ring = field_ring(&document);
    let a = drains(&document)[0];
    // Well outside the traced outline, on the same line.
    let off = Point::new(
        ring.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + 50.0,
        a.y,
    );
    let answer = between(&solved, &Between { a, b: off });
    assert!(answer.feet > 50.0);
    assert_eq!(answer.b_inches, None);
    assert_eq!(answer.rise_inches, None);
}

/// The cricket behind a curb boxed out on the demo stands on the face the water
/// arrives at, and it is as wide each side as half that face.
///
/// The curb is put on the roof between two drains and square to the boards, so
/// which face is upslope is not a matter of taste: the roof climbs away from the
/// drain it is nearest, and the cricket's ridge starts on the face that looks back
/// up the hill and runs on from it.
#[test]
fn a_curb_boxed_out_on_the_demo_gets_its_cricket_on_the_upslope_face() {
    let (document, solved) = open();
    let first = drains(&document)[0];
    let field = &solved.fields[0];
    let along = field.axis_along;
    let across = field.axis_across;

    // A curb eight feet across the fall and four along it, ten feet up the boards
    // from the first drain.
    let at = Point::new(first.x + along.x * 10.0, first.y + along.y * 10.0);
    let corner = |a: f64, b: f64| {
        Point::new(at.x + along.x * a + across.x * b, at.y + along.y * a + across.y * b)
    };
    let ring = [
        corner(-2.0, -4.0),
        corner(2.0, -4.0),
        corner(2.0, 4.0),
        corner(-2.0, 4.0),
        corner(-2.0, -4.0),
    ];

    let answer = cricket_behind(&solved, &ring);
    assert_eq!(answer.refused, None);
    assert!((answer.face_feet - 8.0).abs() < 1e-6, "{} ft of face", answer.face_feet);
    let ridge = answer.ridge.expect("a ridge");

    // The upslope face is the far one from the drain, so its middle is two feet
    // further along the boards than the curb's own middle, and the ridge runs on
    // from there by half the face — four feet.
    let face_mid = Point::new(at.x + along.x * 2.0, at.y + along.y * 2.0);
    assert!(ridge[0].distance(face_mid) < 1e-6, "the ridge starts on the upslope face");
    let out = Point::new(ridge[1].x - ridge[0].x, ridge[1].y - ridge[0].y);
    assert!((out.x.hypot(out.y) - 4.0).abs() < 1e-6, "the ridge runs half the face back");
    assert!(out.x * along.x + out.y * along.y > 0.0, "and it runs UP the boards");
}

/// A curb no wider than two feet across the fall gets no cricket and is told why.
#[test]
fn a_curb_under_the_two_foot_rule_gets_no_cricket() {
    let (document, solved) = open();
    let first = drains(&document)[0];
    let field = &solved.fields[0];
    let along = field.axis_along;
    let across = field.axis_across;
    let at = Point::new(first.x + along.x * 10.0, first.y + along.y * 10.0);
    let corner = |a: f64, b: f64| {
        Point::new(at.x + along.x * a + across.x * b, at.y + along.y * a + across.y * b)
    };
    let ring = [
        corner(-3.0, -0.75),
        corner(3.0, -0.75),
        corner(3.0, 0.75),
        corner(-3.0, 0.75),
        corner(-3.0, -0.75),
    ];
    let answer = cricket_behind(&solved, &ring);
    assert!(answer.ridge.is_none());
    assert!(answer.refused.expect("a sentence").contains("goes round"));
}

/// Every drain on the demo has a section through it, and pointing anywhere on the
/// ground one drain takes finds that drain's own cut.
#[test]
fn pointing_at_the_roof_finds_the_section_through_the_drain_that_serves_it() {
    let (document, solved) = open();
    let all = drains(&document);
    let mut found = Vec::new();
    for drain in &all {
        let cut = roofnerd_core::drawing::section_at(&solved, *drain)
            .expect("a drain stands on the ground it drains");
        // The cut runs out of the drain it is taken at, and it runs somewhere.
        assert!(cut.at.distance(*drain) < 1e-6);
        assert!(cut.run_feet > 1.0);
        found.push(cut.index);
    }
    found.sort_unstable();
    found.dedup();
    assert_eq!(found.len(), all.len(), "one cut apiece, and no two drains share one");

    // A place on no tapered area has no section, and says so by answering nothing.
    let ring = field_ring(&document);
    let off = Point::new(
        ring.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + 50.0,
        all[0].y,
    );
    assert!(roofnerd_core::drawing::section_at(&solved, off).is_none());
}
