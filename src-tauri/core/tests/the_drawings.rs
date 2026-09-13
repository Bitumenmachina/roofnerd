//! ── The drawings, against the roof the demo job states ────────────────────
//!
//! Three drawings come off one solve: the tapered layout, the fastening plan
//! (empty, and saying so) and one section per drain. Every figure below is
//! worked out from what `jobs/demo-job` states rather than read off what this
//! program printed — the field is 14…122 by 42…134 feet with a bump in its north
//! side, three drains in a row on y = 98 at x 36, 74 and 112, a half inch at the
//! drain, a quarter in twelve, three boards allowed, a 4 ft sump, a 3 ft 6 in
//! parapet and a 2 ft curb. Then, by hand:
//!
//!   the cap      min(24 − 8, 0.5 + 3 × 4.5) = 14 in, and the boards ran out
//!   the axes     the south wall is the longest side, so the boards run east and
//!                west and every cut is taken along them
//!   Cut A–A      drain one, 36 − 14 = 22 ft west to the parapet, and the roof
//!                there is 0.5 + 22 × 0.25 = 6 in
//!   Cut B–B      drain two: the bisectors are x = 55 and x = 93, so 19 ft each
//!                way and neither reaches a wall. 0.5 + 19 × 0.25 = 5 1/4 in
//!   Cut C–C      drain three: 122 − 112 = 10 ft east to the parapet against 19
//!                ft west to a ridge — the wall wins, because a section detail
//!                exists to prove the assembly where the roof stops. 3 in
//!   the sump     4 ft square, so its rim is 2 ft out: 0.5 + 2 × 0.25 = 1 in,
//!                and the sump falls the half inch between that and the drain

use std::fs;
use std::path::PathBuf;

use roofnerd_core::drawing::{drawings, feet_written, inches_written, Drawing, Drawings};
use roofnerd_core::{taper, Document, Point};

fn demo() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../jobs/demo-job");
    let part = |rel: &str| -> serde_json::Value {
        let raw = fs::read_to_string(root.join(rel)).expect("the demo job is in the repository");
        serde_json::from_str(&raw).expect("the demo job is json")
    };
    serde_json::json!({ "conditions": part("conditions.json"), "pages": part("pages/pages.json") })
}

fn drawn() -> (Document, roofnerd_core::SolvedTaper, Drawings) {
    let raw = demo();
    let solved = roofnerd_core::solve(&raw, 1).expect("the demo job solves");
    let document = Document::read(&raw).expect("the document reads");
    let all = drawings(&document, &solved.taper);
    (document, solved.taper, all)
}

/// Every board the layout draws is a board the bill buys, and it carries its
/// letter.
///
/// The counts are C2's and they are not repeated here as figures: the bill is
/// asked what it bought and the drawing is held to that, so the day the roof is
/// re-traced the two move together or this goes red.
#[test]
fn the_layout_draws_the_bill_and_nothing_else() {
    let (_, taper, all) = drawn();
    let layout = &all.layout;
    assert!(layout.nothing_yet.is_none(), "{:?}", layout.nothing_yet);

    let field = &taper.fields[0];
    let bill = field.bill.as_ref().expect("the demo is billed by letter");

    // One letter on the drawing for every lettered board in the bill, and the
    // same split between the letters.
    let letters: Vec<&str> = layout
        .texts
        .iter()
        .filter(|t| t.role == "letter")
        .map(|t| t.text.as_str())
        .collect();
    let lettered: usize = bill.by_letter.values().map(|r| r.boards).sum();
    assert_eq!(letters.len(), lettered, "a board on the roof has no letter on the drawing");
    assert_eq!(bill.unlettered, 0, "the demo's chart covers every board");
    for (letter, row) in &bill.by_letter {
        let drawn = letters.iter().filter(|l| *l == letter).count();
        assert_eq!(drawn, row.boards, "{letter} is drawn {drawn} times and bought {} times", row.boards);
    }

    // And every board SHAPE on it is a piece of a board in the bill: the same
    // polygons the boards were cut to, not a grid drawn over them.
    let pieces: usize = field.regions.iter().flat_map(|r| &r.boards).map(|b| b.shapes.len()).sum();
    let drawn = layout.shapes.iter().filter(|s| s.role == "board").count();
    assert_eq!(drawn, pieces);

    // The ground the cap holds flat is on it, and it is the field's own figure.
    let flat: f64 = layout
        .shapes
        .iter()
        .filter(|s| s.role == "nofall")
        .map(|s| s.shape.square_feet())
        .sum();
    assert!(
        (flat - field.no_fall_square_feet).abs() < 0.01,
        "the drawing shades {flat} SF flat and the field has {} SF",
        field.no_fall_square_feet
    );

    // The creases are the solve's, drawn once each.
    let valleys = layout.lines.iter().filter(|l| l.role == "valley").count();
    let ridges = layout.lines.iter().filter(|l| l.role == "ridge").count();
    assert_eq!(valleys, field.regions.iter().map(|r| r.valleys.len()).sum::<usize>());
    assert_eq!(ridges, field.regions.iter().map(|r| r.ridges.len()).sum::<usize>());
}

/// The fastening plan is empty, and the emptiness is a sentence.
///
/// C4 solves the zones. Until it does, a plan with bands drawn on it would be a
/// pattern a crew could work to, which is the worst thing this program could put
/// on a screen.
#[test]
fn the_fastening_plan_says_it_is_not_solved_rather_than_drawing_one() {
    let (_, _, all) = drawn();
    let plan = &all.fastening;
    assert!(plan.shapes.is_empty());
    assert!(plan.lines.is_empty());
    let why = plan.nothing_yet.as_ref().expect("an empty drawing with nothing to say");
    assert!(why.contains("wind"), "{why}");
}

/// One cut per drain, each along the boards, each ending where it should.
#[test]
fn every_drain_gets_a_cut_and_it_runs_along_the_boards() {
    let (_, taper, all) = drawn();
    assert_eq!(all.sections.len(), taper.drains.len());

    let want = [
        ("Cut A\u{2013}A", 22.0, 6.0, true, "Parapet Wall Flashing"),
        ("Cut B\u{2013}B", 19.0, 5.25, false, "ridge"),
        ("Cut C\u{2013}C", 10.0, 3.0, true, "Parapet Wall Flashing"),
    ];
    for (section, (named, run, high, at_wall, ends_at)) in all.sections.iter().zip(want) {
        let subtitle = section.subtitle.as_deref().unwrap_or("");
        assert!(subtitle.starts_with(named), "{subtitle}");
        assert!(subtitle.contains(ends_at), "{subtitle}");

        let run_text = dimension(section, "high point to drain");
        assert_eq!(run_text, format!("{} high point to drain", feet_written(run)));

        let end = if at_wall { "at the wall" } else { "at the ridge" };
        assert_eq!(dimension(section, end), format!("{} {end}", inches_written(high)));
        assert_eq!(dimension(section, "at the drain"), "1/2\" at the drain");
    }

    // The two that reach the parapet carry it, and the one that stops at a ridge
    // does not draw a wall it never met.
    assert_eq!(all.sections[0].shapes.iter().filter(|s| s.role == "wall").count(), 1);
    assert_eq!(all.sections[1].shapes.iter().filter(|s| s.role == "wall").count(), 0);
    assert_eq!(all.sections[2].shapes.iter().filter(|s| s.role == "wall").count(), 1);
}

/// The section stacks the roof the way a crew builds it, bottom to top.
///
/// The deck it lands on, the flat board by the course, the lettered panel, the
/// sump at the drain, and then whatever the condition's own lines say goes over
/// it — in the order the condition lists them, because nothing in the job format
/// states a layer order and the list is the only order there is.
#[test]
fn the_section_stacks_the_roof_the_way_it_gets_built() {
    let (document, taper, all) = drawn();
    let cut = &all.sections[0];

    let mut order: Vec<String> = Vec::new();
    for layer in cut.lines.iter().filter_map(|l| l.layer.clone()) {
        if !order.contains(&layer) {
            order.push(layer);
        }
    }
    for layer in cut.shapes.iter().filter_map(|s| s.layer.clone()) {
        if !order.contains(&layer) {
            order.push(layer);
        }
    }
    let field = document
        .conditions
        .iter()
        .find(|c| c.id == taper.fields[0].condition_id)
        .expect("the field is in the job");
    let stated: Vec<String> = field.layers().iter().map(|l| l.description.clone()).collect();
    assert_eq!(stated.len(), 2, "the demo states a cover board and a membrane");

    let mut want = vec!["deck".to_string(), "fill".to_string(), "panel".to_string(), "sump".to_string()];
    want.extend(stated);
    assert_eq!(order, want);

    // The flat board is drawn course by course, not as one band: a crew lays
    // three of them and the joints are what a section is read for.
    let courses = cut.shapes.iter().filter(|s| s.layer.as_deref() == Some("fill")).count();
    assert!(courses >= 2, "{courses} courses of flat board on a 22 ft cut");
}

/// Nothing on the section is a number this file made up: every panel's top is
/// the solved surface, read at its own corners.
#[test]
fn the_top_of_every_panel_is_the_roof_the_layout_solved() {
    let (_, taper, all) = drawn();
    let field = &taper.fields[0];
    let region = &field.regions[0];
    let cut = &all.sections[0];
    // Cut A–A runs west out of the first drain, so a point x feet from the far
    // end is (14 + x, 98) on the plan. That is this file's own arithmetic, off
    // the job's own outline and drains.
    let plan = |x: f64| Point::new(14.0 + x, 98.0);

    let mut looked = 0;
    for panel in cut.shapes.iter().filter(|s| s.layer.as_deref() == Some("panel")) {
        // The ring is laid down bottom-left, bottom-right, top-right, top-left.
        let ring = &panel.shape.ring;
        assert_eq!(ring.len(), 4);
        for corner in [ring[2], ring[3]] {
            let solved = taper::read_at(
                plan(corner.x),
                &region.planes,
                field.start_inches,
                region.cap_inches,
            )
            .expect("the cut is on the region")
            .thickness_inches;
            // A ten-thousandth of an inch, and the slack is `geo`'s rather than
            // this drawing's: where the cut meets the traced outline comes back
            // off the boolean's fixed-point grid, about three ten-millionths of
            // a foot out, so the far end of the run is 22.0000002 ft and the two
            // sides read the plane a quarter of a millionth of an inch apart.
            assert!(
                (corner.y * -12.0 - solved).abs() < 1e-4,
                "the drawing puts the roof at {} in and the layout makes it {solved} in",
                corner.y * -12.0
            );
            looked += 1;
        }
    }
    assert!(looked >= 8, "only {looked} corners were looked at");
}

/// The sump and the parapet on the drawing are the numbers the job states.
#[test]
fn the_sump_and_the_wall_are_the_job_own_numbers() {
    let (_, taper, all) = drawn();
    let cut = &all.sections[0];
    let sump = taper.sumps.first().expect("the demo sumps its drains");

    assert_eq!(
        dimension(cut, "sump"),
        format!(
            "sump {} \u{d7} {} deep",
            feet_written(sump.side_feet),
            inches_written(sump.depth_inches)
        )
    );
    // A 4 ft sump on a quarter in twelve falls half an inch from its rim.
    assert_eq!(sump.side_feet, 4.0);
    assert_eq!(sump.depth_inches, 0.5);
    assert_eq!(dimension(cut, "parapet"), "42\" parapet");

    // The flashing is drawn to the height somebody typed, and says so.
    let flashing = cut
        .texts
        .iter()
        .find(|t| t.text.contains("wall flashing"))
        .expect("the membrane turns up the wall");
    assert!(flashing.text.contains("24\" up the wall"), "{}", flashing.text);

    // And the conditions the plane passed through are named, so the Estimate can
    // say which lines the cut is of.
    assert_eq!(cut.through, vec!["c-field", "c-drains", "c-parapet"]);
}

/// A dimension on a drawing, by the words on it.
fn dimension(drawing: &Drawing, what: &str) -> String {
    drawing
        .dimensions
        .iter()
        .find(|d| d.text.contains(what))
        .unwrap_or_else(|| {
            panic!(
                "nothing on {} is dimensioned \"{what}\": {:?}",
                drawing.id,
                drawing.dimensions.iter().map(|d| &d.text).collect::<Vec<_>>()
            )
        })
        .text
        .clone()
}
