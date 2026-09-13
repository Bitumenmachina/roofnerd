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
// C0 is the seam and nothing more. `taper.rs` reproduces the cone the drawing
// draws TODAY, to the float, so that the change the estimator can see is none at
// all. The layout that replaces the cone is C1, and it lands behind this seam.

pub mod geom;
pub mod model;
pub mod scope;
pub mod taper;

pub use geom::{bounds, inside, Bounds, Plane, Point};
pub use model::{
    CapReason, Condition, Cricket, Document, FlowArrow, Grid, NotSolvedYet, Page, Solved,
    SolvedTaper, TaperField, Trace, Valley, ValleySample,
};
pub use scope::Scopes;

/// Solve the open job.
///
/// One entry point, called under the shell's one document lock whenever the
/// document changes. `solve_id` is the shell's counter: the window pulls
/// geometry when it moves, so a keystroke carries a number and not a roof.
pub fn solve(document: &serde_json::Value, solve_id: u64) -> Result<Solved, String> {
    let document = Document::read(document)?;
    Ok(Solved {
        solve_id,
        taper: taper::solve(&document, solve_id),
        scopes: scope::scopes_of(&document),
    })
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
    /// Three points on the demo roof, and the field's own aggregates, held
    /// against what `packages/app/src/editors/model.ts` produces today. Same
    /// inputs, same outputs, to the float — which is the whole claim C0 makes.
    ///
    /// How the right-hand sides were got, so the next seat can get them again
    /// rather than trust them. Every expression in `taperSurface`, `nearest`,
    /// `capInches` and `inside` was run on `jobs/demo-job` under node and
    /// printed at 21 significant figures. The job states: three drains at
    /// (36, 98), (74, 98) and (112, 98) feet; the field T = 1/2 in, TAPER =
    /// 1/4 in per foot, BOARDS = 3; a parapet 3 ft 6 in high and a curb 2 ft
    /// high, so the lowest thing the roof flashes against is 24 in. Then:
    ///
    ///   cap        min(24 − 8, 0.5 + 3 × 4.5) = min(16, 14) = 14 in, and the
    ///              boards are what ran out first, not the wall
    ///   (36, 98)   the first drain itself: run 0, so 0.5 in
    ///   (55, 98)   midway between drains one and two: run 19 ft exactly, so
    ///              0.5 + 19 × 0.25 = 5.25 in
    ///   (16, 132)  two feet inboard of the far south-west corner: run
    ///              sqrt(20² + 34²) = 39.4461658466320415073 ft, so
    ///              10.3615414616580103768 in — an irrational run, which is
    ///              the one of the three that can catch a difference in how
    ///              the distance itself is worked out
    ///   thin/thick 0.5 in at a drain; 14 in wherever the cone has run past
    ///              its ceiling — the 14 in the Northgate legend shows, and
    ///              the reason C1 exists
    ///   no fall    34 cells of 2 ft × 2 ft = 136 SF, up in the north bump
    ///              where the cone reaches 56 ft from the nearest drain
    #[test]
    fn the_cone_is_the_cone_the_drawing_draws_today() {
        let taper = solved();
        assert_eq!(taper.drains.len(), 3, "three drains");
        let field = &taper.fields[0];
        assert_eq!(field.condition_id, "c-field");
        assert_eq!(field.start_inches, 0.5);
        assert_eq!(field.slope_inches_per_foot, 0.25);
        assert_eq!(field.cap_inches, Some(14.0));
        assert_eq!(field.cap_reason, Some(CapReason::Boards));

        let read = |x: f64, y: f64| {
            taper::read_at(
                Point::new(x, y),
                &field.apexes,
                field.start_inches,
                field.slope_inches_per_foot,
                field.cap_inches,
            )
        };
        assert_eq!(read(36.0, 98.0).thickness_inches, 0.5);
        assert_eq!(read(55.0, 98.0).thickness_inches, 5.25);
        assert_eq!(read(16.0, 132.0).run_feet, 39.4461658466320415073);
        assert_eq!(read(16.0, 132.0).thickness_inches, 10.3615414616580103768);

        assert_eq!((field.grid.cols, field.grid.rows), (55, 47));
        assert_eq!((field.grid.step_x, field.grid.step_y), (2.0, 2.0));
        assert_eq!(field.thin_inches, Some(0.5));
        assert_eq!(field.thick_inches, Some(14.0));
        assert_eq!(field.no_fall_square_feet, 136.0);
    }

    /// The arrows are part of the picture, so they are part of the pin: 139 of
    /// them, and the first one where the TypeScript puts it.
    ///
    /// The angle is the one number in the whole port that is not bit-for-bit
    /// the TypeScript's. `atan2` is a library function, and this machine's and
    /// V8's disagree in the last bit: 1.9010936816241502 against
    /// 1.9010936816241504, which is 2 parts in 10^16 of a radian. Everything
    /// that is arithmetic — the runs, the thicknesses, the areas, the cricket's
    /// vertices — matches exactly, and that is where a difference would mean
    /// something. This one turns a cone that is a tenth of a foot long by two
    /// ten-thousandths of a nanometre, so it is held to 1e-15 rather than
    /// written down as equal when it is not.
    #[test]
    fn the_water_leaves_the_way_it_left_before() {
        let taper = solved();
        let flow = &taper.fields[0].flow;
        assert_eq!(flow.len(), 139);
        assert_eq!(flow[0].x, 52.57142857142857);
        assert_eq!(flow[0].y, 49.666666666666664);
        assert_eq!(flow[0].thickness_inches, 13.2738095238095237249);
        assert!(
            (flow[0].angle - 1.90109368162415037773).abs() < 1e-15,
            "the fall points at {}, the TypeScript says 1.90109368162415037773",
            flow[0].angle
        );
    }

    /// Three drains in a row read as two valleys and not as three, and each one
    /// is sampled every two feet onto the surface it lies in.
    #[test]
    fn three_drains_in_a_row_are_two_valleys() {
        let taper = solved();
        assert_eq!(taper.valleys.len(), 2);
        for valley in &taper.valleys {
            assert_eq!(valley.samples.len(), 20);
            assert_eq!(valley.elevation, 0.0);
        }
        // Halfway along the first valley the roof is at the ridge between two
        // drains: 19 ft of fall from either, capped by nothing, so 5 in.
        let mid = taper.valleys[0].samples[10];
        assert_eq!((mid.x, mid.y, mid.thickness_inches), (56.0, 98.0, 5.0));
    }

    /// The cricket is the one behind the curb, and it is the same wedge, vertex
    /// for vertex. Its ridge starts at the middle of the curb's south face and
    /// runs 5 ft upslope; it is 2 ft wide each side, which is half that face;
    /// and it rises 1 in — 1/2 in per foot over 2 ft — above the curb's own
    /// 9 in datum.
    #[test]
    fn the_cricket_behind_the_curb_is_the_same_wedge() {
        let taper = solved();
        assert_eq!(taper.crickets.len(), 1);
        let cricket = &taper.crickets[0];
        assert_eq!(cricket.condition_id, "c-cricket");
        assert_eq!(cricket.length_to_width, 2.5);
        assert_eq!(cricket.max_length_to_width, 3.0);
        assert_eq!(cricket.off_by, Some(0.0));
        assert!(!cricket.slope_disagrees);
        assert_eq!(
            cricket.positions,
            vec![
                55.0, 0.8333333333333334, 76.0, 55.0, 0.8333333333333334, 71.0, 57.0, 0.75, 76.0,
                55.0, 0.8333333333333334, 71.0, 57.0, 0.75, 71.0, 57.0, 0.75, 76.0,
                55.0, 0.8333333333333334, 76.0, 55.0, 0.8333333333333334, 71.0, 53.0, 0.75, 76.0,
                55.0, 0.8333333333333334, 71.0, 53.0, 0.75, 71.0, 53.0, 0.75, 76.0,
            ]
        );
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

    /// The solved model serializes in the words the front end reads, and the
    /// grid travels as three parallel lists the drawing can walk in one pass.
    #[test]
    fn the_payload_speaks_camel_case() {
        let out = serde_json::to_value(solved()).expect("it serializes");
        let field = &out["fields"][0];
        assert!(field["startInches"].is_number());
        assert!(field["slopeInchesPerFoot"].is_number());
        assert_eq!(field["capReason"], "boards");
        assert!(field["grid"]["thicknessInches"].is_array());
        assert!(field["noFallSquareFeet"].is_number());
        assert!(out["solveId"].is_number());
    }
}
