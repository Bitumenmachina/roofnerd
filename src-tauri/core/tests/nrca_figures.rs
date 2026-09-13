// ── The book's own figures ─────────────────────────────────────────────────
// NRCA, *The NRCA Roofing Manual: Membrane Roof Systems — 2019*, ch.4 pp.163–168,
// is where the tapered layout in `taper.rs` comes from. This holds the solver to
// the figures in it, one test per figure, on roofs invented here — plain
// dimensions in whole feet, so that what each test proves is the layout and not
// a fixture.
//
//   Fig 4-9   p.164   one-way slope layout
//   Fig 4-10  p.165   two-way slope layout
//   Fig 4-11  p.165   four-way slope to an interior drain
//   Fig 4-15  p.167   layout of valley lines for four-way slope to one drain
//   Fig 4-17  p.168   common ridge lines for adjacent slope patterns
//
// And two things the figures do not draw but the text does: the plateau where the
// build-up hits its ceiling, and NRCA p.166's warning that two adjoining patterns
// do not always give a common thickness at their ridge — "if the patterns do not
// permit a common, uniform thickness at the ridge, an area divider may be
// required." That case is proven to be REPORTED here rather than smoothed over.
//
// The last test is not a figure. It holds the conventions this crate copies from
// `packages/engine/src/roof.ts` to the numbers in that file, read out of the
// TypeScript source at test time, so the two copies cannot drift.

use roofnerd_core::{solve, Point, SolvedTaper, TaperField};

/// A job with one tapered area and some drains on it, in feet.
///
/// The page is scaled at a foot to the unit so that the numbers in each test are
/// the feet on the roof, with no scale arithmetic standing between the two.
fn job(ring: &[(f64, f64)], drains: &[(f64, f64)], start: f64, slope: f64, boards: Option<f64>) -> SolvedTaper {
    let points = |pairs: &[(f64, f64)]| -> Vec<serde_json::Value> {
        pairs.iter().map(|(x, y)| serde_json::json!({ "x": x, "y": y })).collect()
    };
    let mut properties = serde_json::json!({ "T": start, "TAPER": slope, "ELEV": 0.0 });
    if let Some(boards) = boards {
        properties["BOARDS"] = serde_json::json!(boards);
    }
    let document = serde_json::json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area", "properties": properties,
                "traces": [{ "pageId": "page-1", "points": points(ring) }],
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
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

/// How thick the roof is at a place, asked of the solved field.
fn inches(field: &TaperField, x: f64, y: f64) -> f64 {
    roofnerd_core::taper::read_field_at(field, Point::new(x, y))
        .unwrap_or_else(|| panic!("{x}, {y} is not on the field"))
        .thickness_inches
}

/// Fig 4-9, p.164 — one-way slope layout: everything falls to one edge.
///
/// "For a square or rectangular roof area with perimeter drainage, such as
/// gutters, a simple one- or two-way slope pattern may work well." A one-way
/// pattern is not a layout this crate builds — four-way is (see the notes; the
/// two-way with a cricket-stock saddle is recorded as the other trade layout for
/// a later batch). What a four-way pattern does on such a roof is what this
/// pins: the drain sits two feet off one wall, the plane facing away from it takes
/// half the roof, and the far wall comes out at ONE depth the whole way along,
/// because the run across beats the run along everywhere on it. NRCA p.164 names
/// that property as the reason the pattern gets chosen at all: "uniform perimeter
/// thickness and simplify edge detailing."
#[test]
fn fig_4_9_a_drain_at_one_wall_falls_one_way_and_the_far_wall_is_one_depth() {
    let taper = job(
        &[(0.0, 0.0), (40.0, 0.0), (40.0, 24.0), (0.0, 24.0)],
        &[(20.0, 2.0)],
        0.5,
        0.25,
        None,
    );
    let field = only_field(&taper);
    assert_eq!(field.regions.len(), 1, "one drain, one region");
    let region = &field.regions[0];

    // A foot inboard of the far wall, at three places along it, the roof is one
    // depth: 21 ft of run at a quarter inch to the foot on top of the half inch at
    // the drain. A foot inboard because the wall itself is the outline, and a point
    // on a polygon's own edge is neither in nor out of it.
    for x in [1.0, 20.0, 39.0] {
        assert_eq!(inches(field, x, 23.0), 0.5 + 21.0 * 0.25, "at {x} ft along the far wall");
    }
    assert_eq!(inches(field, 20.0, 2.0), 0.5, "the drain is the low point");

    // And this is where the four-way pattern is NOT the one-way pattern the figure
    // draws, which is worth pinning rather than glossing. The plane facing away
    // from the drain takes exactly half the roof — 480 of 960 SF — the two side
    // planes take 238 SF each, and the plane facing the near wall takes the 4 SF
    // triangle between the drain and that wall. Half this roof falls sideways to a
    // valley and then along it, where a one-way layout would send all of it
    // straight at the wall. One-way is a layout this crate does not build; it is
    // recorded as an option beside the two-way saddle, and the four-way pattern
    // still drains the roof.
    let area = |plane: usize| -> f64 {
        region
            .facets
            .iter()
            .filter(|f| f.plane == plane)
            .map(|f| f.shape.square_feet())
            .sum()
    };
    // The planes are [along, back along, across, back across], and the field's
    // axis is the 40 ft wall, so "across" is the one pointing into the roof.
    assert!((area(2) - 480.0).abs() < 0.01, "the plane facing the roof holds {} SF", area(2));
    assert!((area(3) - 4.0).abs() < 0.01, "the plane facing the near wall holds {} SF", area(3));
    assert!((area(0) - area(1)).abs() < 0.01, "the two sides are not even");
    let total: f64 = (0..4).map(area).sum();
    assert!((total - region.square_feet).abs() < 0.01, "the facets come to {total} SF");
}

/// Fig 4-10, p.165 — two-way slope layout: two drains, and the ridge at mid-span.
///
/// NRCA p.166: "the ridge line should be spaced equidistant between drainage
/// points. The ridge line will be perpendicular to a line that connects the two
/// drainage points." Two drains 40 ft apart on the axis put it at mid-span, square
/// to the row, and split the roof in half; with one slope and one set of axes the
/// two patterns meet at one depth all along it — 20 ft of run either side.
#[test]
fn fig_4_10_two_drains_put_the_ridge_at_mid_span_and_split_the_roof() {
    let taper = job(
        &[(0.0, 0.0), (80.0, 0.0), (80.0, 40.0), (0.0, 40.0)],
        &[(20.0, 20.0), (60.0, 20.0)],
        0.5,
        0.25,
        None,
    );
    let field = only_field(&taper);
    assert_eq!(field.regions.len(), 2);
    for region in &field.regions {
        assert!((region.square_feet - 1600.0).abs() < 0.01, "{} SF", region.square_feet);
        assert_eq!(region.ridges.len(), 1, "one boundary, and it does not fold");
        let ridge = region.ridges[0];
        assert_eq!(ridge.a.x, 40.0, "the ridge is not at mid-span");
        assert_eq!(ridge.b.x, 40.0);
        // Level along its whole length: 20 ft of run either side of it, wherever
        // you stand on it. Held to a ten-millionth of an inch and not to the float
        // because the ends of the line are where it leaves the traced ring, and
        // that crossing is a division.
        assert!((ridge.a_inches - (0.5 + 20.0 * 0.25)).abs() < 1e-7, "{}", ridge.a_inches);
        assert!((ridge.b_inches - (0.5 + 20.0 * 0.25)).abs() < 1e-7, "{}", ridge.b_inches);
        assert_eq!(ridge.disagreement_inches, 0.0);
        // It runs the full depth of the roof, wall to wall.
        assert!(((ridge.a.y - ridge.b.y).abs() - 40.0).abs() < 1e-7);
    }
}

/// Fig 4-11, p.165, with Fig 4-15, p.167 — four-way slope to an interior drain.
///
/// "For a square roof area with an interior drain, a four-way slope pattern
/// directed toward the center can provide uniform perimeter thickness and simplify
/// edge detailing. With a four-way slope pattern, uniform slope is provided over
/// all four intersecting insulation areas." And p.166: the valley centrelines are
/// at 45 degrees, corner to corner.
///
/// So on a 48 ft square with the drain in the middle: four facets of 576 SF each,
/// four valleys running exactly to the four corners, and the depth at a corner is
/// 24 ft of run — NOT the 33.94 ft diagonal, which is what a cone would have
/// charged for. The whole perimeter is at that one depth, which is the property
/// the book names.
#[test]
fn fig_4_11_a_centred_drain_gives_four_quarters_and_one_perimeter_depth() {
    let taper = job(
        &[(0.0, 0.0), (48.0, 0.0), (48.0, 48.0), (0.0, 48.0)],
        &[(24.0, 24.0)],
        0.5,
        0.25,
        None,
    );
    let field = only_field(&taper);
    assert_eq!(field.regions.len(), 1);
    let region = &field.regions[0];
    assert!((region.square_feet - 2304.0).abs() < 0.01);

    // Four facets, a quarter of the roof each.
    assert_eq!(region.facets.len(), 4);
    for facet in &region.facets {
        assert!(!facet.flat, "nothing caps this field");
        assert!((facet.shape.square_feet() - 576.0).abs() < 0.01, "{} SF", facet.shape.square_feet());
    }

    // Four valleys, out of the drain to the four corners: the 45 degree miter cut.
    assert_eq!(region.valleys.len(), 4);
    let mut corners: Vec<(f64, f64)> =
        region.valleys.iter().map(|v| ((v.b.x * 1e6).round() / 1e6, (v.b.y * 1e6).round() / 1e6)).collect();
    corners.sort_by(|a, b| a.partial_cmp(b).expect("real corners"));
    assert_eq!(corners, vec![(0.0, 0.0), (0.0, 48.0), (48.0, 0.0), (48.0, 48.0)]);

    // The corner depth is the run along an axis, not the diagonal to the drain.
    let at_corner = 0.5 + 24.0 * 0.25;
    assert_eq!(at_corner, 6.5);
    for valley in &region.valleys {
        assert!((valley.b_inches - at_corner).abs() < 1e-7, "{}", valley.b_inches);
    }
    // And the perimeter is one depth the whole way round: a wall at a corner and
    // a wall at its middle are the same, which is what the book promises.
    for (x, y) in [(1.0, 1.0), (24.0, 1.0), (47.0, 24.0), (24.0, 47.0), (1.0, 47.0)] {
        assert_eq!(inches(field, x, y), 0.5 + 23.0 * 0.25, "at {x}, {y}");
    }
}

/// A field that is not convex: the regions follow the trace, notch and all.
///
/// The demo roof has a bump in one side, and this is that shape with plain
/// dimensions: a 60 by 40 field with a 20 by 10 bump standing out of its north
/// wall. A convex clip would square the notch off and hand the estimator roof
/// that is not there, so the cut is `geo`'s boolean intersection (D147). Two
/// drains, so the bump is split between them as well.
#[test]
fn a_concave_field_keeps_its_notch_and_the_regions_add_up_to_it() {
    let taper = job(
        &[
            (0.0, 10.0), (20.0, 10.0), (20.0, 0.0), (40.0, 0.0), (40.0, 10.0),
            (60.0, 10.0), (60.0, 50.0), (0.0, 50.0),
        ],
        &[(15.0, 30.0), (45.0, 30.0)],
        0.5,
        0.25,
        None,
    );
    let field = only_field(&taper);
    assert!((field.square_feet - 2600.0).abs() < 1e-9, "{} SF traced", field.square_feet);
    assert_eq!(field.regions.len(), 2);

    let total: f64 = field.regions.iter().map(|r| r.square_feet).sum();
    assert!((total - 2600.0).abs() < 0.01, "the regions come to {total} SF");
    for region in &field.regions {
        assert!((region.square_feet - 1300.0).abs() < 0.01, "{} SF", region.square_feet);
    }

    // Beside the bump, where the box says roof and the trace says nothing, no
    // region claims the ground and no reading is invented for it.
    assert!(roofnerd_core::taper::read_field_at(field, Point::new(10.0, 5.0)).is_none());
    assert!(roofnerd_core::taper::read_field_at(field, Point::new(30.0, 5.0)).is_some());
}

/// The cap: where the build-up hits its ceiling the roof goes flat, and it is a
/// square, not a circle.
///
/// One layer of board off a half inch drain reaches 5 in (`capInches`, and NRCA's
/// 8 in of flashing is not what binds here), and a quarter inch to the foot gets
/// there in 18 ft of run. The run that governs is the greater of the two runs
/// along the axes, so the ground that is still falling is the 36 ft square about
/// the drain — 1,296 SF of a 3,600 SF roof — and the 2,304 SF left over has no
/// fall on it at all. That is the figure an estimator walks, and it is reported in
/// square feet rather than shaded and left to be guessed at.
#[test]
fn the_cap_makes_a_square_plateau_and_it_is_reported_in_square_feet() {
    let taper = job(
        &[(0.0, 0.0), (60.0, 0.0), (60.0, 60.0), (0.0, 60.0)],
        &[(30.0, 30.0)],
        0.5,
        0.25,
        Some(1.0),
    );
    let field = only_field(&taper);
    assert_eq!(field.cap_inches, Some(5.0));
    assert_eq!(field.thick_inches, Some(5.0));
    assert!(
        (field.no_fall_square_feet - 2304.0).abs() < 0.01,
        "the plateau came to {} SF",
        field.no_fall_square_feet
    );

    // Inside the square the roof is still falling; outside it, it is not. The
    // square reaches 18 ft either way from the drain at (30, 30), so 20 ft along
    // is out on the flat and 10 ft along is not.
    let out_on_the_flat = roofnerd_core::taper::read_field_at(field, Point::new(50.0, 30.0))
        .expect("on the field");
    assert!(out_on_the_flat.flat, "20 ft of run past an 18 ft cap and the roof is still rising");
    assert_eq!(out_on_the_flat.thickness_inches, 5.0);
    let inside_it =
        roofnerd_core::taper::read_field_at(field, Point::new(40.0, 30.0)).expect("on the field");
    assert!(!inside_it.flat);
    assert_eq!(inside_it.thickness_inches, 0.5 + 10.0 * 0.25);

    // The plateau is a square with a square hole in it, which is why a region's
    // ground carries holes at all: answering with the outline would report the
    // whole roof as having no fall.
    let region = &field.regions[0];
    let flat: f64 = region.facets.iter().filter(|f| f.flat).map(|f| f.shape.square_feet()).sum();
    assert!((flat - 2304.0).abs() < 0.01);
}

/// NRCA p.166, the case the figures do not draw: two patterns that will not meet.
///
/// "When a ridge occurs, especially between adjoining tapered patterns, thought
/// must be given to the resulting thicknesses of the two adjoining patterns so a
/// common, uniform thickness occurs at the ridge line… If the patterns do not
/// permit a common, uniform thickness at the ridge, an area divider may be
/// required to provide an adequate transition."
///
/// With one slope and one set of axes over the whole field, the two patterns meet
/// at one depth wherever the line joining their drains is square to the axes — and
/// they do not where that line is slanted, because the bisector is square to the
/// slant while the runs are square to the boards. Two drains 40 ft apart along the
/// boards and 10 ft apart across them is that case, and the ridge carries the
/// disagreement in inches so the drawing can say an area divider is wanted rather
/// than quietly averaging the two.
#[test]
fn a_slanted_pair_of_drains_does_not_give_one_ridge_thickness_and_says_so() {
    let taper = job(
        &[(0.0, 0.0), (60.0, 0.0), (60.0, 50.0), (0.0, 50.0)],
        &[(10.0, 10.0), (50.0, 20.0)],
        0.5,
        0.25,
        None,
    );
    let field = only_field(&taper);
    assert_eq!(field.regions.len(), 2);
    let worst = field
        .regions
        .iter()
        .flat_map(|r| r.ridges.iter())
        .fold(0.0f64, |most, r| most.max(r.disagreement_inches));
    assert!(worst > 0.5, "the two patterns are only {worst} in apart at the ridge");

    // And it is still zero where the bisector crosses the line joining the drains:
    // the disagreement is a fact about the ends of the ridge, not about all of it.
    let middle = Point::new(30.0, 15.0);
    let both: Vec<f64> = field
        .regions
        .iter()
        .map(|r| {
            roofnerd_core::taper::read_at(middle, &r.planes, field.start_inches, r.cap_inches)
                .expect("four planes")
                .thickness_inches
        })
        .collect();
    assert_eq!(both[0], both[1]);
}

/// The conventions this crate copies are the numbers `roof.ts` states.
///
/// They are written twice on purpose — once in `packages/engine/src/roof.ts` for
/// the money, with the manual on each one, and once in `src-tauri/core/src/taper.rs`
/// for the model — and two copies of a number agree on the day they are written.
/// So this reads the TypeScript source itself at test time rather than restating
/// its figures here: change one and this test says which.
#[test]
fn the_core_and_the_engine_state_the_same_conventions() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/roof.ts"),
    )
    .expect("the engine's conventions are in the repository");

    /// The number that follows a piece of the source, whatever it is.
    fn after(source: &str, lead: &str) -> f64 {
        let at = source.find(lead).unwrap_or_else(|| panic!("`{lead}` is not in roof.ts"));
        let rest = &source[at + lead.len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        rest[..end].parse().unwrap_or_else(|_| panic!("`{lead}` is not followed by a number"))
    }

    assert_eq!(
        roofnerd_core::taper::SINGLE_LAYER_MAX_INCHES,
        after(&source, "export const SINGLE_LAYER_MAX_INCHES = ")
    );
    assert_eq!(
        roofnerd_core::taper::MIN_FLASHING_INCHES,
        after(&source, "export const MIN_FLASHING_INCHES = ")
    );
    assert_eq!(
        roofnerd_core::taper::CRICKET_SLOPE_MULTIPLE,
        after(&source, "export const CRICKET_SLOPE_MULTIPLE = ")
    );

    // The length-to-width table, Fig 4-13 p.166: a half inch to the foot in the
    // field allows 4:1, anything shallower 3:1. Read off the one line that states
    // it, because `? ` and ` : ` appear all over a TypeScript file and a search of
    // the whole source would answer with whichever came first.
    let table = source
        .lines()
        .find(|line| line.contains("fieldSlope >="))
        .expect("`maxLengthToWidth` is not in roof.ts");
    let figures: Vec<f64> = table
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter(|piece| !piece.is_empty())
        .filter_map(|piece| piece.parse().ok())
        .collect();
    assert_eq!(figures.len(), 3, "the table reads `{table}`");
    let (steeper, wider, narrower) = (figures[0], figures[1], figures[2]);
    assert_eq!(roofnerd_core::taper::max_length_to_width(steeper), wider);
    assert_eq!(roofnerd_core::taper::max_length_to_width(steeper / 2.0), narrower);

    // And the cap is the lower of the two ceilings, with the reason on it — the
    // one place the two files have to agree about an order of operations and not
    // only about a figure.
    let (cap, reason) = roofnerd_core::taper::cap_inches(0.5, Some(3.0), Some(24.0));
    assert_eq!(cap, Some(14.0));
    assert_eq!(reason, Some(roofnerd_core::CapReason::Boards));

    // C3's own conventions, pinned the same way: read off the TypeScript
    // rather than restated here.
    assert_eq!(
        roofnerd_core::coping::CAP_OVERHANG_DEFAULT_IN,
        after(&source, "export const CAP_OVERHANG_IN = ")
    );
    assert_eq!(
        roofnerd_core::coping::COPING_SECTION_DEFAULT_FT,
        after(&source, "export const COPING_SECTION_FT = ")
    );
    assert_eq!(
        roofnerd_core::coping::COPING_SECTION_MIN_FT,
        after(&source, "export const COPING_SECTION_MIN_FT = ")
    );
    assert_eq!(
        roofnerd_core::coping::COPING_SECTION_MAX_FT,
        after(&source, "export const COPING_SECTION_MAX_FT = ")
    );
    assert_eq!(
        roofnerd_core::coping::FASTEN_OC_DEFAULT_IN,
        after(&source, "export const FASTEN_OC_IN = ")
    );

    // Gauge steps at the same two places on both sides.
    for face in [7.99, 8.0, 8.01, 10.0, (10.0 + 0.01)] {
        assert_eq!(
            roofnerd_core::coping::gauge_of(face),
            gauge_ga_from_ts(&source, face),
            "the two files disagree about {face} in of face"
        );
    }
}

/// `copingGaugeGa` in `roof.ts`, read rather than restated: two `if` lines,
/// each `<= number) return number;`, and `null` past both.
fn gauge_ga_from_ts(source: &str, face: f64) -> Option<u32> {
    let start = source.find("export function copingGaugeGa").expect("copingGaugeGa is not in roof.ts");
    let body = &source[start..];
    for line in body.lines().skip(1) {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("if (faceHeightInches <= ") {
            let end = rest.find(')').unwrap();
            let threshold: f64 = rest[..end].parse().unwrap();
            let value: u32 = rest[rest.find("return ").unwrap() + "return ".len()..]
                .trim_end_matches(';')
                .parse()
                .unwrap();
            if face <= threshold {
                return Some(value);
            }
            continue;
        }
        if line.starts_with("return null") {
            return None;
        }
    }
    panic!("copingGaugeGa's body did not end the way this reader expects");
}
