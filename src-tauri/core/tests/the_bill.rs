// ── The bill, on roofs worked out by hand ──────────────────────────────────
// C1 laid the field out. This holds what a crew actually buys off that layout:
// 4 ft boards by letter, the flat fill under them, the sumps and the crickets.
//
// Every figure in here is arithmetic on a roof invented here in whole feet, and
// the arithmetic is written out in the test's own doc comment. The 64 by 128 roof
// is the plan's own hand-checked case — "corner 32 ft → 8.5 in, fill 8.0 in under
// X" — and everything else on it follows from two facts: at a quarter inch to the
// foot a 4 ft board rises exactly one inch, and the grid is anchored on the drain.
// So the thickness at the middle of the cell m cells out from the drain is
// `T + m` inches, and the whole bill is a count of cells by `m`.
//
//   the letters     NRCA p.166 for the layout; the designations, the edges they
//                   run between and the flat board they stack on are Versico's,
//                   transcribed in `data/profiles`
//   the joints      p.208 Fig 5-9, straight both ways on the top layer
//   the sump        p.164, the bowl plus about 24 in; p.306, bought as panels
//   the saddle      Fig 4-13 p.166, the length-to-width table, and Fig 4-14 for
//                   the drain-to-drain saddle a two-way layout needs

use std::collections::BTreeMap;

use roofnerd_core::{solve, BoardBill, Point, SolvedTaper, TaperField};

/// A job with one tapered area and some drains on it, in feet.
///
/// The page is scaled at a foot to the unit, so the numbers in each test are the
/// feet on the roof with no scale arithmetic in between. `field` and `drains` carry
/// whatever properties the case needs — `LAYOUT`, `PROFILE_SET`, `SUMP`, `BOWL`.
fn job(
    ring: &[(f64, f64)],
    drains: &[(f64, f64)],
    field: serde_json::Value,
    on_drains: serde_json::Value,
) -> SolvedTaper {
    let points = |pairs: &[(f64, f64)]| -> Vec<serde_json::Value> {
        pairs.iter().map(|(x, y)| serde_json::json!({ "x": x, "y": y })).collect()
    };
    let document = serde_json::json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area", "properties": field,
                "traces": [{ "pageId": "page-1", "points": points(ring) }],
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
                "properties": on_drains,
                "traces": [{ "pageId": "page-1", "points": points(drains) }],
            },
        ],
    });
    solve(&document, 1).expect("the job solves").taper
}

fn only_field(taper: &SolvedTaper) -> &TaperField {
    assert!(taper.refused.is_empty(), "the layout refused: {:?}", taper.refused);
    assert_eq!(taper.fields.len(), 1, "one tapered area");
    &taper.fields[0]
}

fn bill_of(field: &TaperField) -> &BoardBill {
    field.bill.as_ref().unwrap_or_else(|| {
        panic!("no bill: {}", field.bill_refused.clone().unwrap_or_default())
    })
}

/// How many boards of each letter, as a plain map for comparing whole.
fn letters(bill: &BoardBill) -> BTreeMap<&str, usize> {
    bill.by_letter.iter().map(|(k, v)| (k.as_str(), v.boards)).collect()
}

/// The 64 by 128 roof, two drains, and the whole bill by hand.
///
/// The roof is 128 ft the long way, so the boards are laid to that side and the
/// two drains sit at the middle of a half each: (32, 32) and (96, 32). The
/// bisector is x = 64, so each drain takes a 64 by 64 square. T is 1/2 in and the
/// slope a quarter in the foot, and nothing caps it — no board count, no wall.
///
/// THE PLAN'S OWN FIGURE. The far corner (0, 0) is 32 ft along and 32 across from
/// the drain, so the greater run is 32 and the roof is 0.5 + 32 x 0.25 = 8.5 in.
/// Two flat 4 in boards is 8 in of that, leaving a half inch of tapered panel,
/// which is X — the bottom of the Extended table at this slope.
///
/// THE GRID. Anchored on the drain, 4 ft cells. So cell centres sit at 4 ft steps
/// from the drain and the cells run -8 to 8 each way: 17 columns by 17 rows, 289
/// positions a region, 578 on the roof. Of those, 15 by 15 = 225 a region are whole
/// boards; the outer ring of 64 is cut by the roof's edge. 225 x 16 + the edge
/// strips comes back to 64 x 64 = 4,096 SF a region, which is the whole roof.
///
/// THE LETTERS. A 4 ft board at a quarter in the foot rises exactly 1 in, so the
/// thickness at the middle of a cell `m` cells out from the drain — `m` being the
/// greater of the two counts, because the run is the greater of the two runs — is
/// 0.5 + m inches. Then the fill is 4 x floor(m / 4) and the panel is
/// 0.5 + (m mod 4), which is X, Y, Z, ZZ as m mod 4 runs 0, 1, 2, 3. Cells at
/// `m`: one at 0 and 8m after that, so
///
///   m        0   1   2   3   4   5   6   7   8
///   cells    1   8  16  24  32  40  48  56  64      = 289
///   letter   X   Y   Z  ZZ   X   Y   Z  ZZ   X
///   fill     0   0   0   0   1   1   1   1   2      boards
///
/// X is 1 + 32 + 64 = 97 a region, Y is 8 + 40 = 48, Z is 16 + 48 = 64 and ZZ is
/// 24 + 56 = 80. Double it for the two regions. The fill is 176 x 1 + 64 x 2 = 304
/// a region, 608 on the roof, and the deepest stack is two fill boards and a
/// tapered one — three courses.
#[test]
fn the_hand_checked_roof_comes_to_the_bill_the_table_gives() {
    let taper = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0), (96.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&taper);
    assert_eq!(field.layout, "four-way");
    assert_eq!(field.regions.len(), 2);
    assert_eq!(field.cap_inches, None, "nothing caps this roof");

    // The plan's own figure, read off the layout. A ring's own corner is neither in
    // nor out of it as far as ray casting goes, so this asks the region that holds
    // the corner rather than searching for one.
    let corner = roofnerd_core::taper::read_at(
        Point::new(0.0, 0.0),
        &field.regions[0].planes,
        field.start_inches,
        field.regions[0].cap_inches,
    )
    .expect("the region has planes");
    assert_eq!(corner.thickness_inches, 8.5);
    assert_eq!(corner.run_feet, 32.0);

    let bill = bill_of(field);
    assert_eq!(bill.panel_feet, 4.0);
    assert_eq!(bill.fill_step_inches, 4.0);
    assert_eq!(bill.stagger_feet, 2.0, "half a board");
    assert_eq!(bill.boards, 578);
    assert_eq!(bill.cut_boards, 128, "the outer ring of each region's grid");
    assert_eq!(bill.square_feet, 578.0 * 16.0);
    assert!(
        (bill.laid_square_feet - 8192.0).abs() < 0.01,
        "the boards land on {} SF and the roof is 8192",
        bill.laid_square_feet
    );
    assert!((bill.waste_square_feet - (578.0 * 16.0 - 8192.0)).abs() < 0.01);
    assert_eq!(
        letters(bill),
        BTreeMap::from([("X", 194), ("Y", 96), ("Z", 128), ("ZZ", 160)])
    );
    for (letter, row) in &bill.by_letter {
        assert_eq!(row.square_feet, row.boards as f64 * 16.0, "{letter} is not whole boards");
    }
    assert_eq!(bill.fill_boards, 608);
    assert_eq!(bill.fill_square_feet, 608.0 * 16.0);
    assert_eq!(bill.courses_max, 3, "two fill boards and a tapered one");
    assert_eq!(bill.unlettered, 0, "every thickness on this roof is in the table");
    assert_eq!(bill.document_id, "VE-7830");
}

/// A cut board counts one and says what the cut took off it.
///
/// The board at the far corner of the hand-checked roof is the case, and it is the
/// same board the plan checked by hand: its middle is at the corner (0, 0), so it
/// reads 8.5 in and comes out an X over 8 in of fill — and only a 2 ft by 2 ft
/// quarter of it is on the roof. So it is one board, 4 SF laid, 12 SF cut away.
#[test]
fn a_cut_board_counts_one_and_reports_its_own_waste() {
    let taper = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0), (96.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&taper);
    let corner = field.regions[0]
        .boards
        .iter()
        .find(|b| b.centre == Point::new(0.0, 0.0))
        .expect("there is a board whose middle is the corner of the roof");
    assert!(corner.cut);
    assert!((corner.square_feet - 4.0).abs() < 0.01, "{} SF of it landed", corner.square_feet);
    assert_eq!(corner.board_square_feet, 16.0);
    assert!((corner.waste_square_feet - 12.0).abs() < 0.01);
    assert_eq!(corner.thickness_inches, 8.5);
    assert_eq!(corner.fill_inches, 8.0);
    assert_eq!(corner.fill_boards, 2);
    assert_eq!(corner.letter.as_deref(), Some("X"));
    assert_eq!(corner.courses, 3);
    // And it is still one board in the count: the bill's board total is the number
    // of positions, whole or cut.
    let bill = bill_of(field);
    assert_eq!(bill.boards, field.regions.iter().map(|r| r.boards.len()).sum::<usize>());

    // A whole board next to it, for the contrast.
    let inside = field.regions[0]
        .boards
        .iter()
        .find(|b| b.centre == Point::new(32.0, 32.0))
        .expect("there is a board over the drain");
    assert!(!inside.cut);
    assert_eq!(inside.waste_square_feet, 0.0);
    assert_eq!(inside.thickness_inches, 0.5, "the board over the drain is the thin one");
    assert_eq!(inside.letter.as_deref(), Some("X"));
    assert_eq!(inside.courses, 1, "no fill under the thinnest board");
}

/// Every board on the hand-checked roof lands exactly on a shared edge, and the
/// rule this crate states is the one that answers.
///
/// A 4 ft board rises exactly 1 in at a quarter in the foot, and the panels in the
/// table are 1 in ranges, so a grid anchored on a 1/2 in drain reads 0.5, 1.5, 2.5
/// or 3.5 at every cell middle — the table's own printed edges, three of which two
/// panels share. The table prints no rule for that; this crate's is that the panel
/// whose THIN edge it is takes it, so 1.5 in is a Y and not an X.
///
/// Move the drain thickness to 1 in and every reading lands mid range instead, so
/// the same roof proves both: 1.0 is inside X, 2.0 inside Y, and nothing is on a
/// boundary at all.
#[test]
fn the_letter_is_picked_on_a_shared_edge_and_mid_range_alike() {
    let ring = [(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)];
    let drains = [(32.0, 32.0), (96.0, 32.0)];

    let on_edges = job(
        &ring,
        &drains,
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&on_edges);
    for board in &field.regions[0].boards {
        let panel = board.thickness_inches - board.fill_inches;
        assert_eq!(
            panel.fract(),
            0.5,
            "{panel} in of panel is not on one of the table's printed edges"
        );
    }
    // One cell along from the drain is 1 1/2 in, the edge X and Y share, and it
    // reads as a Y.
    let shared = field.regions[0]
        .boards
        .iter()
        .find(|b| b.centre == Point::new(36.0, 32.0))
        .expect("one cell along from the drain");
    assert_eq!(shared.thickness_inches, 1.5);
    assert_eq!(shared.letter.as_deref(), Some("Y"));

    let mid_range = job(
        &ring,
        &drains,
        serde_json::json!({ "T": 1.0, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&mid_range);
    for board in &field.regions[0].boards {
        let panel = board.thickness_inches - board.fill_inches;
        assert_eq!(panel.fract(), 0.0, "{panel} in of panel is not mid range");
    }
    let one_along = field.regions[0]
        .boards
        .iter()
        .find(|b| b.centre == Point::new(36.0, 32.0))
        .expect("one cell along from the drain");
    assert_eq!(one_along.thickness_inches, 2.0);
    assert_eq!(one_along.letter.as_deref(), Some("Y"), "2 in is the middle of Y");
    assert_eq!(one_along.fill_inches, 0.0);
}

/// Fill steps a whole flat board at a time, and it steps where the board does.
///
/// Out from the drain on the hand-checked roof the thickness climbs an inch a
/// board: 0.5, 1.5, 2.5, 3.5, then 4.5 — and at 4.5 the tapered panel has run out
/// of table, so a flat 4 in board goes under it and the panel drops back to a half
/// inch. Every four cells, one more fill board and back to X.
#[test]
fn fill_steps_a_whole_board_at_a_time_across_the_boundary() {
    let taper = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0), (96.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&taper);
    let at = |cells: usize| {
        field.regions[0]
            .boards
            .iter()
            .find(|b| b.centre == Point::new(32.0 + 4.0 * cells as f64, 32.0))
            .unwrap_or_else(|| panic!("{cells} cells along from the drain"))
            .clone()
    };
    let walk: Vec<(f64, f64, usize, Option<String>)> = (0..9)
        .map(|m| {
            let b = at(m);
            (b.thickness_inches, b.fill_inches, b.fill_boards, b.letter.clone())
        })
        .collect();
    assert_eq!(
        walk,
        vec![
            (0.5, 0.0, 0, Some("X".into())),
            (1.5, 0.0, 0, Some("Y".into())),
            (2.5, 0.0, 0, Some("Z".into())),
            (3.5, 0.0, 0, Some("ZZ".into())),
            // The step: a flat board goes in and the panel starts again at X.
            (4.5, 4.0, 1, Some("X".into())),
            (5.5, 4.0, 1, Some("Y".into())),
            (6.5, 4.0, 1, Some("Z".into())),
            (7.5, 4.0, 1, Some("ZZ".into())),
            (8.5, 8.0, 2, Some("X".into())),
        ]
    );
}

/// A sump is the bowl plus about two feet, and it is bought in whole panels.
///
/// NRCA p.164. A 12 in bowl wants 36 in square — three feet — and three feet of
/// sump still takes a whole 4 ft panel, because part of a panel is a panel you
/// bought. A 60 in bowl wants 84 in, which is seven feet, and seven feet needs two
/// panels each way: four.
///
/// The cut is at the thickness the field has at the sump's own rim, read off the
/// solved planes. On this roof that is half a sump out at a quarter in the foot off
/// a half inch drain, and it is the same all the way round the rim — the thickness
/// is the greater of two square runs, so the square IS a contour of it.
#[test]
fn the_sump_is_the_bowl_plus_two_feet_and_is_bought_in_panels() {
    let small = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({ "BOWL": 12.0 }),
    );
    assert_eq!(small.sumps.len(), 1);
    let sump = &small.sumps[0];
    assert_eq!(sump.side_feet, 3.0, "12 in of bowl and 24 in more, in feet");
    assert_eq!(sump.from_bowl_feet, Some(3.0));
    assert_eq!(sump.panels, 1);
    assert_eq!(sump.panel_feet, 4.0);
    assert!(!sump.clipped);
    assert!((sump.square_feet - 9.0).abs() < 0.01);
    // The rim: 1.5 ft out at 1/4 in per foot off 1/2 in.
    assert_eq!(sump.rim_inches, 0.5 + 1.5 * 0.25);
    assert_eq!(sump.depth_inches, 1.5 * 0.25);
    assert_eq!(sump.condition_id, "c-drains");

    let big = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({ "BOWL": 60.0 }),
    );
    assert_eq!(big.sumps[0].side_feet, 7.0);
    assert_eq!(big.sumps[0].panels, 4, "seven feet is two panels each way");

    // A typed SUMP is the estimator's own and wins over the rule, with what the
    // rule wanted reported beside it rather than overruled in silence.
    let typed = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0), (96.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0 }),
        serde_json::json!({ "SUMP": 4.0, "BOWL": 60.0 }),
    );
    assert_eq!(typed.sumps.len(), 2);
    assert_eq!(typed.sumps[0].side_feet, 4.0);
    assert_eq!(typed.sumps[0].from_bowl_feet, Some(7.0));

    // And the boards do not run through it: a 4 ft sump on a drain at the middle of
    // a cell takes exactly one board position out of each region's 289, and it is
    // the X over the drain.
    let field = only_field(&typed);
    let bill = bill_of(field);
    assert_eq!(bill.boards, 576, "two fewer than the 578 cells the grid lays out");
    assert!(
        !field.regions[0].boards.iter().any(|b| b.centre == Point::new(32.0, 32.0)),
        "a board was laid over the sump"
    );
    assert!(
        (bill.laid_square_feet - (8192.0 - 32.0)).abs() < 0.01,
        "the boards cover {} SF and the roof less the two sumps is 8160",
        bill.laid_square_feet
    );
}

/// A two-way layout falls two ways and needs a saddle between every pair of drains.
///
/// NRCA Fig 4-10 p.165 for the pattern and Fig 4-14 for the saddle. The field falls
/// square to the side the boards are laid to, so on a roof 120 ft the long way with
/// its drains in a row down the middle the thickness depends only on how far off
/// that row a place is — and the ground along the row comes out level, which is
/// what the saddle is for. Its ridge is the perpendicular bisector of the two
/// drains, it falls half the span each way, and it is cut at twice the field.
#[test]
fn a_two_way_layout_gets_the_saddle_its_drains_need() {
    let taper = job(
        &[(0.0, 0.0), (120.0, 0.0), (120.0, 30.0), (0.0, 30.0)],
        &[(20.0, 15.0), (100.0, 15.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0, "LAYOUT": 2.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&taper);
    assert_eq!(field.layout, "two-way");
    assert_eq!(field.regions[0].planes.len(), 2, "two planes, not four");

    // The thickness is the run off the drains' own row and nothing else: 15 ft of
    // it at the roof's edge, whatever the x.
    let at = |x: f64, y: f64| {
        roofnerd_core::taper::read_field_at(field, Point::new(x, y))
            .unwrap_or_else(|| panic!("{x}, {y} is off the field"))
            .thickness_inches
    };
    // A foot inside the edge, because a point exactly on a ring's own edge is
    // neither in nor out of it — 14 ft off the drains' row either way.
    assert_eq!(at(5.0, 1.0), 0.5 + 14.0 * 0.25);
    assert_eq!(at(50.0, 1.0), 0.5 + 14.0 * 0.25);
    assert_eq!(at(20.0, 15.0), 0.5, "at the drain");
    assert_eq!(at(50.0, 15.0), 0.5, "and level all the way along their row");

    // The facets still tile each region, which is what says the two half planes
    // between them leave nothing out.
    for region in &field.regions {
        let facets: f64 = region.facets.iter().map(|f| f.shape.square_feet()).sum();
        assert!(
            (facets - region.square_feet).abs() < 0.01,
            "the facets come to {facets} SF and the region is {} SF",
            region.square_feet
        );
    }

    // One saddle, on the bisector, 40 ft of fall each way, cut at a half inch to
    // the foot — twice the field — and 30 ft long, which is the run of the bisector
    // across the roof.
    assert_eq!(taper.crickets.len(), 1);
    let saddle = &taper.crickets[0];
    assert!(saddle.derived, "nobody traced this one");
    assert!(saddle.from_solved_surface);
    assert_eq!(saddle.condition_id, "c-field");
    assert_eq!(saddle.slope_inches_per_foot, 0.5);
    assert_eq!(saddle.length_to_width, 30.0 / 40.0);
    assert_eq!(saddle.max_length_to_width, 3.0);
    assert_eq!(saddle.plan_square_feet, 30.0 * 40.0 * 2.0);
    assert_eq!(saddle.off_by, None, "there is no traced line for it to be off");
}

/// A saddle too long for its width is refused, with the ratio in words.
///
/// Fig 4-13 p.166 caps a cricket at 3:1 under a half inch to the foot. Put the two
/// drains 8 ft apart on a roof 30 ft across and the saddle between them would be 4
/// ft wide and 30 ft long — seven and a half to one. It would pond at the valley
/// however right it looks in plan, so the solve says so and draws nothing: the
/// answer is another drain or an area divider, and that is the estimator's call.
#[test]
fn a_saddle_too_long_for_its_width_is_refused_and_says_the_ratio() {
    let taper = job(
        &[(0.0, 0.0), (120.0, 0.0), (120.0, 30.0), (0.0, 30.0)],
        &[(36.0, 15.0), (44.0, 15.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0, "LAYOUT": 2.0 }),
        serde_json::json!({}),
    );
    assert!(taper.crickets.is_empty(), "an over-long saddle was drawn anyway");
    assert_eq!(taper.refused.len(), 1);
    let why = &taper.refused[0].why;
    assert!(why.contains("7.5"), "{why}");
    assert!(why.contains("area divider"), "{why}");
    assert_eq!(taper.refused[0].condition_id, "c-field");
    // And the field itself still solved: a refusal is about the saddle, not the roof.
    assert_eq!(taper.fields.len(), 1);
}

/// A slope the table prints no chart for has no bill, and says why.
///
/// The bulletin lists 3/16 in per foot among its slopes and prints no panel
/// diagram for it. A bill of nought boards would read as a roof that needs no
/// insulation, so there is none, and the sentence names the missing chart.
#[test]
fn a_slope_the_table_does_not_chart_has_no_bill_and_says_so() {
    let taper = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.1875, "ELEV": 0.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&taper);
    assert!(field.bill.is_none());
    let why = field.bill_refused.as_deref().expect("no reason given");
    assert!(why.contains("no extended panel chart"), "{why}");
    assert!(field.regions[0].boards.is_empty(), "boards were laid off no table");
    // The layout is still the layout: the fall, the valleys and the ridges are all
    // there, and only the lettered board is missing.
    assert_eq!(field.regions.len(), 1);
    assert_eq!(field.regions[0].valleys.len(), 4);
}

/// The Standard table is the other chart on the same page, and a typed
/// `PROFILE_SET` picks it. A number that names neither is not honoured in silence.
#[test]
fn the_typed_profile_set_picks_the_other_chart_or_says_it_did_not() {
    let standard = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0), (96.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0, "PROFILE_SET": 2.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&standard);
    let bill = bill_of(field);
    assert_eq!(bill.fill_step_inches, 2.0, "the Standard chart stacks on a 2 in board");
    assert_eq!(letters(bill).keys().copied().collect::<Vec<_>>(), vec!["X", "Y"]);
    assert_eq!(field.profile_set_refused, None);
    // Far more fill boards than the 4 in chart needs, for the same roof: the
    // stack at m cells out is floor(m / 2) of them instead of floor(m / 4), which
    // over the 289 cells of each region is 752 apiece.
    assert_eq!(bill.fill_boards, 1504);

    let nonsense = job(
        &[(0.0, 0.0), (128.0, 0.0), (128.0, 64.0), (0.0, 64.0)],
        &[(32.0, 32.0), (96.0, 32.0)],
        serde_json::json!({ "T": 0.5, "TAPER": 0.25, "ELEV": 0.0, "PROFILE_SET": 3.0, "LAYOUT": 3.0 }),
        serde_json::json!({}),
    );
    let field = only_field(&nonsense);
    assert_eq!(field.profile_set_refused, Some(3.0));
    assert_eq!(field.layout_refused, Some(3.0));
    assert_eq!(field.layout, "four-way", "and the default was used");
    assert_eq!(bill_of(field).fill_step_inches, 4.0);
}

/// The core and the engine state the same sump rule.
///
/// `sumpSide` in `packages/engine/src/roof.ts` is the bowl plus 24 in, and this
/// crate has its own copy because the core may not read the engine. The two are
/// held together the way every other convention in that file is —
/// `nrca_figures.rs` does the same for the three it already had.
#[test]
fn the_core_and_the_engine_state_the_same_sump_rule() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/engine/src/roof.ts"),
    )
    .expect("the engine's conventions are in the repository");
    let line = source
        .lines()
        .find(|line| line.contains("export const sumpSide"))
        .expect("`sumpSide` is not in roof.ts");
    let added: f64 = line
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter(|piece| !piece.is_empty())
        .last()
        .expect("no number on the line")
        .parse()
        .expect("not a number");
    assert_eq!(roofnerd_core::sump::sump_side_inches(0.0), added);
    assert_eq!(roofnerd_core::sump::sump_side_inches(12.0), 12.0 + added);
}
