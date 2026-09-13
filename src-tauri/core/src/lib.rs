// ── roofnerd-core: the solvers ─────────────────────────────────────────────
// What the roof actually has to be, in feet and inches.
//
// Patrick's ruling of 2026-09-12: the 3D model is the function, not a view. The
// tapered package laid out the trade's way, the walls and curbs with their
// flashing heights, the fastening pattern by wind zone — that is what has to get
// built, and the drawings, the section and the estimate are produced FROM it.
// This crate owns that model. Every window and every export reads the same
// result; the front end never recomputes it.
//
// What is here, and what is not:
//
//   here      the geometry and the numbers that fall out of it — thickness,
//             valleys, crickets, and (from C1 on) boards, walls, coping, zones,
//             fastener counts
//   not here  money. Formulas, lines, waste, prices and the recap stay in
//             `packages/engine`, which is plain arithmetic with no window near
//             it. A solve hands the engine NAMED NUMBERS; the engine decides
//             what they cost
//   not here  windows, the disk, the document. `src-tauri/src/lib.rs` holds the
//             one open job and calls in here
//
// No window, no file system, no network: `cargo test -p roofnerd-core` runs the
// whole thing with nothing on screen.
//
// C0 was the seam and nothing more: `taper.rs` reproduced the cone the drawing
// drew then, to the float, so that the change an estimator could see was none at
// all. C1 is the roof. The field is laid out the trade's way — drainage regions
// off the perpendicular bisectors between the drains, four planes to a region,
// valleys where two of them tie at 45 degrees, the plateau where the cap binds —
// and every window moved with it because none of them holds an opinion about the
// roof any more (D149).

pub mod boards;
pub mod coping;
pub mod cricket;
pub mod drawing;
pub mod fastening;
pub mod geom;
pub mod model;
pub mod profile;
pub mod scope;
pub mod sump;
pub mod taper;
pub mod verbs;
pub mod walls;
pub mod wind;

pub use coping::Coping;
pub use cricket::Layout;
pub use drawing::{Dimension, DrawnLine, DrawnShape, DrawnText, Drawing, Drawings};
pub use geom::{bounds, inside, Bounds, Edge, HalfPlane, Plane, Point, Shape};
pub use model::{
    Board, BoardBill, CapReason, Condition, Crease, Cricket, Document, Facet, FlowArrow, Line,
    NotSolvedYet, Page, Refused, RidgeSegment, Solved, SolvedTaper, Sump, TaperField, TaperRegion,
    Trace, LAYER_ORDER,
};
pub use profile::{LetterRow, Panel, ProfileSet, Schedule};
pub use scope::Scopes;
pub use fastening::{FastenedField, FastenedZone, Pattern, SolvedFastening};
pub use verbs::{
    between, cricket_behind, high_and_low, Between, BetweenAnswer, CricketBehindAnswer, HighAndLow,
    HighAndLowAnswer,
};
pub use walls::{SolvedWalls, WallRun};
pub use wind::{SolvedWind, WindPlan, Zone};

/// Solve the open job.
///
/// One entry point, called under the shell's one document lock whenever the
/// document changes. `solve_id` is the shell's counter: the window pulls
/// geometry when it moves, so a keystroke carries a number and not a roof.
pub fn solve(document: &serde_json::Value, solve_id: u64) -> Result<Solved, String> {
    let document = Document::read(document)?;
    let taper = taper::solve(&document, solve_id);
    let walls = walls::solve(&document, &taper, solve_id);
    let wind = wind::solve(&document, &taper, solve_id);
    let fastening = fastening::solve(&document, &taper, &wind, solve_id);
    let scopes = scope::scopes_of(&document, &taper, &walls, &fastening);
    Ok(Solved { solve_id, taper, walls, fastening, scopes })
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn solved() -> SolvedTaper {
        solve(&demo(), 1).expect("the demo job solves").taper
    }

    /// ── THE PIN ──────────────────────────────────────────────────────────
    ///
    /// The demo roof, laid out. Four places on it and the field's own aggregates,
    /// each one worked out from what the job says rather than from what this
    /// program printed. `packages/engine/test/takeoff.test.mjs` holds the same
    /// four places from the other side, so the two copies of the one expression
    /// that reads a plane cannot drift apart.
    ///
    /// What the job states, in feet: a field 14…122 by 42…134 with a bump in its
    /// north side; three drains in a row on y = 98, at x 36, 74 and 112; T = 1/2
    /// in at the drain, TAPER = 1/4 in per foot, BOARDS = 3; a parapet 3 ft 6 in
    /// and a curb 2 ft, so the lowest thing the roof flashes against is 24 in.
    /// Then, by hand:
    ///
    ///   cap        min(24 − 8, 0.5 + 3 × 4.5) = min(16, 14) = 14 in, and the
    ///              boards are what ran out first, not the wall
    ///   the axes   the longest side is the south wall, 108 ft of it, so the
    ///              boards run east and west and the layout is square to it
    ///   the ridges the bisectors of the three drains: x = 55 between the first
    ///              two and x = 93 between the last two. The pair either end of
    ///              the row do not touch, so there is no third ridge
    ///   (36, 98)   the first drain itself: no run, so 1/2 in exactly
    ///   (46, 108)  ten feet along and ten across from that drain — a valley, by
    ///              construction, because the two runs are equal. 0.5 + 10 ×
    ///              0.25 = 3 in from either of the two planes that tie there
    ///   (16, 132)  two feet inboard of the far south-west corner: 20 ft along
    ///              and 34 across, so the greater run is 34 and the roof is
    ///              0.5 + 34 × 0.25 = 9 in. THE CONE MADE IT 10.3615… in, off
    ///              the diagonal — this one number is the whole of C1 on the
    ///              screen
    ///   (52, 43)   up in the bump, 55 ft across from the drain: 0.5 + 55 ×
    ///              0.25 = 14.25 in, which is past the cap, so the roof is 14 in
    ///              and flat, and the water stops there
    ///   the high   56 ft across from a drain at the bump's own north wall, which
    ///              wants 14 1/2 in and gets the cap's 14 in. Every region
    ///              reaches it, because the three drains stand in a row
    #[test]
    fn the_field_is_laid_out_the_way_the_book_lays_it_out() {
        let taper = solved();
        assert!(taper.refused.is_empty(), "{:?}", taper.refused);
        assert_eq!(taper.drains.len(), 3, "three drains");
        let field = &taper.fields[0];
        assert_eq!(field.condition_id, "c-field");
        assert_eq!(field.start_inches, 0.5);
        assert_eq!(field.slope_inches_per_foot, 0.25);
        assert_eq!(field.cap_inches, Some(14.0));
        assert_eq!(field.cap_reason, Some(CapReason::Boards));

        // One region per drain, each round its own drain, in the order the job
        // lists them.
        assert_eq!(field.regions.len(), taper.drains.len());
        for (region, drain) in field.regions.iter().zip(&taper.drains) {
            assert_eq!(region.drain, *drain);
        }

        // The axes are the field's longest side: the south wall.
        assert_eq!(field.axis_along, Point::new(1.0, -0.0));
        assert_eq!(field.axis_across, Point::new(0.0, 1.0));
        assert_eq!(field.axis_edge, [Point::new(122.0, 134.0), Point::new(14.0, 134.0)]);

        let read = |x: f64, y: f64| {
            taper::read_field_at(field, Point::new(x, y)).expect("the place is on the field")
        };
        // At the drain, on a plane, on a valley, and out on the plateau.
        assert_eq!(read(36.0, 98.0).thickness_inches, 0.5);
        assert_eq!(read(36.0, 98.0).run_feet, 0.0);
        assert_eq!(read(46.0, 108.0).thickness_inches, 3.0);
        assert_eq!(read(16.0, 132.0).run_feet, 34.0);
        assert_eq!(read(16.0, 132.0).thickness_inches, 9.0);
        assert_eq!(read(52.0, 43.0).thickness_inches, 14.0);
        assert!(read(52.0, 43.0).flat, "the bump is past the cap and the water stays");
        assert!(!read(16.0, 132.0).flat);

        assert_eq!(field.thin_inches, Some(0.5));
        assert_eq!(field.thick_inches, Some(14.0));
        assert_eq!(field.square_feet, 9600.0);
    }

    /// Every valley is at 45 degrees to the boards, and nothing else can happen.
    ///
    /// NRCA p.166: "all valley centrelines should be 45 degrees from the direction
    /// of slope; that is, the valley centrelines are 90 degrees apart." Here that
    /// is not a rule applied but a consequence: a valley IS where two planes at
    /// right angles have risen equally, and equal runs in two square directions is
    /// the diagonal between them. So this reads the solved creases back and holds
    /// each one to equal runs along both axes — four to a region, out of the drain.
    #[test]
    fn every_valley_is_at_45_degrees_because_that_is_what_a_tie_is() {
        let taper = solved();
        let field = &taper.fields[0];
        let mut counted = 0;
        for region in &field.regions {
            assert_eq!(region.valleys.len(), 4, "four ties to a drain");
            for valley in &region.valleys {
                assert_eq!(valley.a, region.drain, "a valley starts at the drain");
                let dx = valley.b.x - valley.a.x;
                let dy = valley.b.y - valley.a.y;
                let along = dx * field.axis_along.x + dy * field.axis_along.y;
                let across = dx * field.axis_across.x + dy * field.axis_across.y;
                assert!(
                    (along.abs() - across.abs()).abs() < 1e-6,
                    "the crease runs {along} along and {across} across, which is not 45 degrees"
                );
                // It lies IN the surface: the depth at each end is the depth the
                // planes make there, so the drawn line cannot float over the roof.
                assert_eq!(valley.a_inches, 0.5);
                assert!(valley.b_inches > valley.a_inches);
                counted += 1;
            }
        }
        assert_eq!(counted, 12);
    }

    /// The ridge between two drains is the same depth from both sides.
    ///
    /// Which is what NRCA p.166 asks for — "a common, uniform thickness occurs at
    /// the ridge line" — and it is not automatic: one slope and one set of axes
    /// give it where the line joining the two drains is square to the axes, and
    /// not where that line is slanted, which is why the disagreement is measured
    /// and carried rather than assumed. On this roof the three drains stand in a
    /// row along the boards, so it is zero to the float.
    ///
    /// The bisectors are x = 55 and x = 93. Each is cut into the pieces on which
    /// both sides are straight — four of them, folded where the governing plane
    /// changes hands at the two 45 degree ties and where the cap takes over — so
    /// the depth at the ends says all of each piece. The outer two drains do not
    /// share a boundary, so neither carries a ridge to the other.
    #[test]
    fn the_ridge_between_two_drains_is_one_thickness_from_both_sides() {
        let taper = solved();
        let field = &taper.fields[0];
        for (index, region) in field.regions.iter().enumerate() {
            // Four pieces to a boundary — folded at the two 45 degree ties and
            // where the cap takes over — and the middle region has two boundaries.
            assert_eq!(region.ridges.len(), if index == 1 { 8 } else { 4 });
            for ridge in &region.ridges {
                assert_eq!(
                    ridge.disagreement_inches, 0.0,
                    "the two patterns disagree by {} in at {:?}",
                    ridge.disagreement_inches, ridge.a
                );
                assert_ne!(ridge.a, ridge.b);
            }
        }
        // Which regions touch: the middle one touches both, the ends touch only it.
        let meets: Vec<Vec<usize>> = field
            .regions
            .iter()
            .map(|r| {
                let mut ids: Vec<usize> = r.ridges.iter().map(|x| x.meets).collect();
                ids.dedup();
                ids
            })
            .collect();
        assert_eq!(meets, vec![vec![1], vec![0, 2], vec![1]]);

        // The piece of the first bisector that spans the drains' own line: 19 ft
        // of run either side, and LEVEL along it — which is the four-way pattern
        // and not a cone. Between two drains the ground between the two 45 degree
        // ties is a flat-bottomed trough, so there is no valley along the line
        // joining them at all; C0 drew one there and called it the valley between
        // drains, and this is why that line is gone.
        let ridge = field.regions[0]
            .ridges
            .iter()
            .find(|r| (r.a.y - 98.0) * (r.b.y - 98.0) <= 0.0)
            .expect("the ridge spans the drains' own line");
        assert_eq!(ridge.a.x, 55.0);
        assert_eq!(ridge.b.x, 55.0);
        assert_eq!(ridge.a_inches, 5.25);
        assert_eq!(ridge.b_inches, 5.25);
    }

    /// The ground with no fall is the square the cap draws, and it is reported.
    ///
    /// 54 ft of run is where 1/4 in to the foot off a 1/2 in drain reaches the
    /// 14 in the boards allow, so the sloping ground is the 108 ft square about
    /// each drain and what is left over is flat. On this roof that is a 2 ft strip
    /// inside the bump's north wall — 54 ft of the 56 the wall stands off the
    /// drains — which comes to 2 ft × (5 + 38 + 23) ft = 132 SF.
    ///
    /// It is held to a hundredth rather than to the float: `geo`'s boolean cuts go
    /// through a fixed-point grid (i_overlay), so a cut corner lands within about
    /// three ten-millionths of a foot of where the arithmetic puts it — a fraction
    /// of a thousandth of an inch on the roof, and a few hundred-thousandths of a
    /// square foot in an area. The figure an estimator reads is whole square feet.
    #[test]
    fn the_ground_with_no_fall_is_the_plateau_the_cap_makes() {
        let taper = solved();
        let field = &taper.fields[0];
        assert!(
            (field.no_fall_square_feet - 132.0).abs() < 0.01,
            "the plateau came to {} SF",
            field.no_fall_square_feet
        );
        // Each region's own share of it, and every flat facet accounted for.
        let mut flat = 0.0;
        for region in &field.regions {
            let mine: f64 =
                region.facets.iter().filter(|f| f.flat).map(|f| f.shape.square_feet()).sum();
            assert!((mine - region.no_fall_square_feet).abs() < 1e-9);
            flat += mine;
        }
        assert!((flat - field.no_fall_square_feet).abs() < 1e-9);

        // The facets tile the region: every piece of ground is on exactly one
        // plane or on the flat, and none of it is left out.
        for region in &field.regions {
            let facets: f64 = region.facets.iter().map(|f| f.shape.square_feet()).sum();
            assert!(
                (facets - region.square_feet).abs() < 0.01,
                "the facets come to {facets} SF and the region is {} SF",
                region.square_feet
            );
        }
        let regions: f64 = field.regions.iter().map(|r| r.square_feet).sum();
        assert!((regions - field.square_feet).abs() < 0.01, "the regions come to {regions} SF");
    }

    /// The arrows run down the plane a place is on, and not at the drain.
    ///
    /// ONE TO A FACET (C5, D156). A facet is the ground a single plane governs, so
    /// the water does exactly one thing over the whole of it: one long mark down
    /// the middle says that, and the hundred and thirty-nine identical eight-foot
    /// darts this used to place said it a hundred and thirty-nine times without
    /// saying how far any of them reached. Four planes off each of three drains,
    /// and every one of those wedges reaches ground that is still falling, so the
    /// demo comes to twelve.
    ///
    /// Every angle is a quarter turn or a half of one, because the fall on a
    /// four-way layout is square to the boards — the axes here are the south wall
    /// and square to it — and never a slant at the nearest drain, which is what
    /// the cone drew.
    #[test]
    fn the_water_runs_down_the_plane_it_is_on() {
        let taper = solved();
        let field = &taper.fields[0];
        let flow = &field.flow;
        let falling: usize =
            field.regions.iter().map(|r| r.facets.iter().filter(|f| !f.flat).count()).sum();
        assert_eq!(flow.len(), falling, "one arrow to each piece of ground a plane governs");
        assert_eq!(flow.len(), 12, "four ways off each of three drains");

        for arrow in flow {
            // No arrow stands on flat ground: every one of them is under the cap.
            assert!(arrow.thickness_inches < 14.0, "an arrow stands on the plateau");
            // Square to the boards, to the float: a quarter turn from the axis.
            let quarters = arrow.angle / std::f64::consts::FRAC_PI_2;
            assert!(
                (quarters - quarters.round()).abs() < 1e-12,
                "an arrow runs at {} radians, which is not square to the boards",
                arrow.angle
            );
            // And it is long enough to speak for the ground it stands on: a fall
            // of 54 ft of run is the widest a wedge gets here, so every mark is
            // several feet and none of them is the whole roof.
            assert!(
                arrow.length_feet > 1.0 && arrow.length_feet < 60.0,
                "an arrow is drawn {} ft long",
                arrow.length_feet
            );
        }

        // The four off the first drain run four different ways, which is what
        // makes them one to a plane rather than four copies of one mark.
        let first: Vec<f64> = flow[..4].iter().map(|a| a.angle).collect();
        for (i, a) in first.iter().enumerate() {
            for b in &first[i + 1..] {
                assert!((a - b).abs() > 1e-9, "two of a drain's four arrows run the same way");
            }
        }
    }

    /// A trace the layout cannot lay out is refused, with the reason, not hidden.
    ///
    /// C0 solved a crossing ring and the drawing quietly declined to triangulate
    /// it, which is wasted work in here and a fact about the job that nothing said
    /// out loud (C0's pushback 3). A tapered area with no drain on it is the same
    /// kind of fact: there is nowhere for the water to go, and that is the
    /// estimator's to fix rather than the solve's to average away.
    #[test]
    fn a_field_the_layout_cannot_lay_out_says_so_and_says_why() {
        let crossed = serde_json::json!({
            "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
            "conditions": [
                {
                    "id": "c-bowtie", "kind": "area", "properties": { "T": 0.5, "TAPER": 0.25 },
                    "traces": [{ "pageId": "page-1", "points": [
                        { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 40.0 },
                        { "x": 40.0, "y": 0.0 }, { "x": 0.0, "y": 40.0 }
                    ] }]
                },
                {
                    "id": "c-drains", "kind": "count", "role": "drain",
                    "traces": [{ "pageId": "page-1", "points": [{ "x": 20.0, "y": 10.0 }] }]
                }
            ],
        });
        let taper = solve(&crossed, 1).expect("the job solves").taper;
        assert!(taper.fields.is_empty(), "a crossing ring was laid out anyway");
        assert_eq!(taper.refused.len(), 1);
        assert_eq!(taper.refused[0].condition_id, "c-bowtie");
        assert!(taper.refused[0].why.contains("crosses itself"), "{}", taper.refused[0].why);

        let no_drain = serde_json::json!({
            "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
            "conditions": [{
                "id": "c-field", "kind": "area", "properties": { "T": 0.5, "TAPER": 0.25 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 0.0 },
                    { "x": 40.0, "y": 40.0 }, { "x": 0.0, "y": 40.0 }
                ] }]
            }],
        });
        let taper = solve(&no_drain, 1).expect("the job solves").taper;
        assert!(taper.fields.is_empty());
        assert!(taper.refused[0].why.contains("no drainage point"), "{}", taper.refused[0].why);
    }

    /// ── THE CRICKET IS ON THE ROOF NOW, NOT OVER IT ─────────────────────
    ///
    /// Same wedge, same place, same proportions — and every vertex lifted by the
    /// depth the tapered package makes under it instead of by a typed `ELEV`.
    /// That closes the B1 pushback the plan names at S1 step 4: the demo used to
    /// carry `ELEV: 0.75` on this condition, so the whole cricket sat at nine
    /// inches over a roof that runs from a half inch to fourteen.
    ///
    /// By hand, from what the job states. The ridge starts at the middle of the
    /// curb's south face at (55, 76) and runs 5 ft north to (55, 71); it is 2 ft
    /// wide each side, which is half that 4 ft face; and it is cut at 1/2 in per
    /// foot, so it rises 1 in over those 2 ft. The roof under it, off the layout:
    ///
    ///   (55, 76)  19 ft along and 22 across from a drain, so 22 governs:
    ///             0.5 + 22 x 0.25 = 6 in = 1/2 ft
    ///   (55, 71)  19 along, 27 across: 0.5 + 27 x 0.25 = 7 1/4 in
    ///   (53, .)   17 along, same across as above — 22 and 27 still govern, so the
    ///   (57, .)   feet either side sit at the same two depths as the ridge ends
    ///
    /// So the ridge runs from 6 + 1 = 7 in up to 7 1/4 + 1 = 8 1/4 in, and the feet
    /// from 6 in to 7 1/4 in. The wedge FOLLOWS THE FALL, which is what a cricket
    /// does and what a wedge at one elevation cannot.
    #[test]
    fn the_cricket_behind_the_curb_stands_on_the_roof_it_sheds() {
        let taper = solved();
        assert_eq!(taper.crickets.len(), 1);
        let cricket = &taper.crickets[0];
        assert_eq!(cricket.condition_id, "c-cricket");
        assert_eq!(cricket.length_to_width, 2.5);
        assert_eq!(cricket.max_length_to_width, 3.0);
        assert_eq!(cricket.off_by, Some(0.0));
        assert!(!cricket.slope_disagrees);
        assert!(!cricket.derived, "nobody traced a saddle here");
        assert!(cricket.from_solved_surface, "it is sitting on a typed elevation");
        // 5 ft of ridge, 2 ft of fall each side.
        assert_eq!(cricket.plan_square_feet, 20.0);
        assert_eq!(cricket.slope_inches_per_foot, 0.5);

        let inches = |v: f64| v * 12.0;
        let field = &taper.fields[0];
        let at = |x: f64, y: f64| taper::surface_feet(field, Point::new(x, y));
        assert_eq!(inches(at(55.0, 76.0)), 6.0);
        assert_eq!(inches(at(55.0, 71.0)), 7.25);
        assert_eq!(
            cricket.positions,
            vec![
                55.0, 0.5833333333333334, 76.0, 55.0, 0.6875, 71.0, 57.0, 0.5, 76.0,
                55.0, 0.6875, 71.0, 57.0, 0.6041666666666666, 71.0, 57.0, 0.5, 76.0,
                55.0, 0.5833333333333334, 76.0, 55.0, 0.6875, 71.0, 53.0, 0.5, 76.0,
                55.0, 0.6875, 71.0, 53.0, 0.6041666666666666, 71.0, 53.0, 0.5, 76.0,
            ]
        );
        // The ridge is 1 in above the roof at each of its own ends — the cricket's
        // own rise, and nothing else. A thousandth of an inch of slack, because a
        // foot converted to inches and back does not land on the same last bit.
        assert!((inches(0.5833333333333334 - at(55.0, 76.0)) - 1.0).abs() < 1e-3);
        assert!((inches(0.6875 - at(55.0, 71.0)) - 1.0).abs() < 1e-3);
    }

    /// Nothing open is not an error and not an empty roof: it is a solve with
    /// nothing in it, and the window draws nothing.
    #[test]
    fn no_job_is_no_geometry_and_no_complaint() {
        let solved = solve(&serde_json::json!({}), 0).expect("nothing solves");
        assert_eq!(solved.solve_id, 0);
        assert!(solved.taper.fields.is_empty());
        assert!(solved.taper.drains.is_empty());
        assert!(solved.scopes.is_empty());
    }

    /// A field with no cap rises unbounded, and says so with `None` rather than
    /// with a number nobody stated.
    #[test]
    fn a_field_with_nothing_to_stop_it_has_no_cap() {
        let (cap, reason) = taper::cap_inches(0.5, None, None);
        assert_eq!(cap, None);
        assert_eq!(reason, None);
        let (cap, reason) = taper::cap_inches(0.5, None, Some(24.0));
        assert_eq!(cap, Some(16.0));
        assert_eq!(reason, Some(CapReason::Flashing));
    }

    /// The solved model serializes in the words the front end reads, and a region
    /// travels whole: its ground, its planes, its facets and its creases.
    #[test]
    fn the_payload_speaks_camel_case() {
        let out = serde_json::to_value(solved()).expect("it serializes");
        let field = &out["fields"][0];
        assert!(field["startInches"].is_number());
        assert!(field["slopeInchesPerFoot"].is_number());
        assert_eq!(field["capReason"], "boards");
        assert!(field["axisEdge"].is_array());
        assert!(field["noFallSquareFeet"].is_number());
        assert!(out["solveId"].is_number());
        assert!(out["refused"].is_array());

        let region = &field["regions"][0];
        assert!(region["drain"]["x"].is_number());
        assert!(region["shapes"][0]["ring"].is_array());
        assert!(region["planes"][0]["direction"]["y"].is_number());
        assert!(region["planes"][0]["slope"].is_number());
        assert!(region["facets"][0]["flat"].is_boolean());
        assert!(region["facets"][0]["plane"].is_number());
        assert!(region["valleys"][0]["aInches"].is_number());
        assert!(region["ridges"][0]["disagreementInches"].is_number());
        assert!(region["noFallSquareFeet"].is_number());
    }
}
