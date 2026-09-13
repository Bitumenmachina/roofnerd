// ── What holds the demo roof down ──────────────────────────────────────────
// The counts, on the demo's own layout. Nothing below is a figure copied out of
// a run: every number is the job's own board grid and the manufacturer's own
// table put together, which is the only way a count like this can be checked at
// all — a typed total would agree with itself for ever.

use std::fs;
use std::path::PathBuf;

use roofnerd_core::fastening::{self, FastenedField};
use roofnerd_core::model::Document;

fn demo() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../jobs/demo-job");
    let part = |rel: &str| -> serde_json::Value {
        let raw = fs::read_to_string(root.join(rel)).expect("the demo job is in the repository");
        serde_json::from_str(&raw).expect("the demo job is json")
    };
    serde_json::json!({ "conditions": part("conditions.json"), "pages": part("pages/pages.json") })
}

fn solved() -> roofnerd_core::Solved {
    roofnerd_core::solve(&demo(), 1).expect("the demo job solves")
}

fn roof(solved: &roofnerd_core::Solved) -> &FastenedField {
    solved.fastening.fields.first().expect("the demo's roof is fastened")
}

/// One fastened board per board position, and every one of them in a zone.
///
/// The base layer is the bottom course, and there is exactly one of those at
/// every position the layout lays a board at — which is the count C2's own bill
/// buys tapered panels against.
#[test]
fn every_board_position_carries_one_fastened_board_and_sits_in_one_zone() {
    let solved = solved();
    let roof = roof(&solved);
    let bill = solved.taper.fields[0].bill.as_ref().expect("the layout has a bill");

    assert_eq!(roof.base_boards, bill.boards, "one fastened board to a board position");
    let placed: usize = roof.zones.iter().map(|z| z.boards).sum();
    assert_eq!(placed, roof.base_boards, "no board is in two zones and none is in none");
}

/// A board lying across a boundary is counted in the higher zone, and said.
#[test]
fn a_board_across_a_boundary_takes_the_higher_zone_and_is_reported() {
    let solved = solved();
    let roof = roof(&solved);

    let straddling: usize = roof.zones.iter().map(|z| z.straddling).sum();
    assert_eq!(straddling, roof.straddling);
    assert!(roof.straddling > 0, "a 4 ft grid on an 8.8 ft band has to straddle somewhere");

    // The field is the lowest zone, so nothing can straddle INTO it: a board
    // half in the field and half at the perimeter is a perimeter board.
    let field = roof.zones.iter().find(|z| z.zone == "1").expect("the field is a zone");
    assert_eq!(field.straddling, 0, "there is no zone under the field to straddle from");

    // And every straddler is in a zone that was asked for a real pattern, so
    // taking the higher zone actually costs something.
    for zone in roof.zones.iter().filter(|z| z.straddling > 0) {
        assert!(zone.fasteners_per_board.is_some(), "{} has boards and no pattern", zone.label);
    }
}

/// The count in each zone is its own boards times its own printed pattern, and
/// the plates are one to a fastener.
#[test]
fn the_count_is_the_boards_times_the_pattern_and_the_plates_go_one_to_one() {
    let solved = solved();
    let roof = roof(&solved);
    let base = roof.base_board_inches.expect("the layout states a base board");
    let membrane = roof.membrane.as_deref();

    let mut all = 0usize;
    for zone in &roof.zones {
        let rating = zone.fm_class.expect("this roof types a pressure for every zone");
        let printed = fastening::pattern_for(
            &roof.board_size,
            rating,
            &roof.product_family,
            membrane,
            base,
        )
        .expect("the table prints a pattern for this zone");
        assert_eq!(zone.fasteners_per_board, Some(printed.fasteners));
        assert_eq!(zone.fasteners, Some(zone.boards * printed.fasteners));
        all += zone.boards * printed.fasteners;
    }
    assert_eq!(roof.plates, Some(all), "a plate to a fastener");

    // And the scope carries the same three numbers the zones do.
    let scope = &solved.scopes["c-field"];
    for (zone, name) in [("1", "FASTENERS_Z1_EA"), ("2", "FASTENERS_Z2_EA"), ("3", "FASTENERS_Z3_EA")] {
        let z = roof.zones.iter().find(|z| z.zone == zone).expect("solved");
        assert_eq!(scope[name], z.fasteners.map(|c| c as f64));
    }
    assert_eq!(scope["PLATES_EA"], Some(all as f64));
}

/// A corner costs more than the field, board for board.
///
/// Which is the whole reason for any of this: if the pattern came out the same
/// everywhere, the zones would be decoration.
#[test]
fn a_board_in_a_corner_takes_more_than_a_board_in_the_field() {
    let solved = solved();
    let roof = roof(&solved);
    let per = |zone: &str| {
        roof.zones
            .iter()
            .find(|z| z.zone == zone)
            .and_then(|z| z.fasteners_per_board)
            .expect("every zone on this roof has a pattern")
    };
    assert!(per("3") > per("1"), "a corner is denser than the field");
    assert!(per("2") >= per("1"), "and the perimeter is no lighter than the field");
}

/// The pattern is chosen against the base board the layout actually lays, and
/// where the layout lays less than the printed row is written for, it is said.
///
/// The demo runs a quarter in the foot off a half inch at the drain, so within
/// sixteen feet of a drain the layout puts no flat board under the tapered panel
/// at all — the panel lands on the deck, and there is less under the fastener
/// than the row it is being fastened to is drawn for. That is a fact about this
/// layout, not about the arithmetic, and it is reported rather than rounded away.
#[test]
fn a_position_with_less_under_the_fastener_than_the_row_wants_is_counted() {
    let solved = solved();
    let roof = roof(&solved);
    let base = roof.base_board_inches.expect("the layout states a base board");

    let counted: usize = roof.zones.iter().map(|z| z.thin_boards).sum();
    assert_eq!(counted, roof.thin_boards);
    assert!(roof.thin_boards > 0, "this layout does put panels straight on the deck");
    assert!(roof.thin_boards < roof.base_boards, "and it does not do it everywhere");

    // The row every zone was fastened to wants less than the flat board gives.
    for zone in &roof.zones {
        let printed = zone.pattern.as_ref().expect("a pattern");
        assert!(printed.minimum_inches <= base, "{} was read against a thicker board", zone.label);
    }
    assert!(
        roof.minimum_thickness_note.to_lowercase().contains("minimum thickness"),
        "the document's own note on the thickness column travels with the answer"
    );
}

/// The adhesive is bought by the zone, at the spacing the table's top row gives,
/// and the membrane by the surface.
#[test]
fn the_adhesive_follows_the_zone_and_the_membrane_follows_the_roof() {
    let solved = solved();
    let roof = roof(&solved);
    assert_eq!(roof.ribbon_spacing_inches, fastening::TABLE_TOP_SPACING_IN);

    let per_set = fastening::ADHESIVE_SQUARE_FEET_PER_SET
        * (fastening::TABLE_TOP_SPACING_IN / fastening::ADHESIVE_RATE_SPACING_IN);
    for zone in &roof.zones {
        // Every course above the fastened one, plus the cover board over the lot.
        assert!(
            zone.adhered_square_feet >= zone.square_feet,
            "{} adheres less than one layer over itself",
            zone.label
        );
        let sets = zone.adhesive_sets.expect("a zone with area takes adhesive");
        assert_eq!(sets, (zone.adhered_square_feet / per_set).ceil() as usize);
        assert!(sets > 0, "{} takes no adhesive at all", zone.label);
    }

    let gallons = roof.bonding_gallons.expect("the membrane is bonded");
    assert!(
        (gallons - roof.square_feet / fastening::BONDING_SQUARE_FEET_PER_GALLON).abs() < 1e-9,
        "the rate the manual prints already carries both faces"
    );

    let scope = &solved.scopes["c-field"];
    assert_eq!(scope["BONDING_GAL"], Some(gallons));
    for (zone, name) in [("1", "ADHESIVE_SETS_Z1_EA"), ("2", "ADHESIVE_SETS_Z2_EA"), ("3", "ADHESIVE_SETS_Z3_EA")] {
        let z = roof.zones.iter().find(|z| z.zone == zone).expect("solved");
        assert_eq!(scope[name], z.adhesive_sets.map(|c| c as f64));
    }
}

/// The adhesive gap is carried with the answer, not left in a note somewhere.
#[test]
fn the_roof_says_the_table_it_was_priced_from_stops_short_of_this_coast() {
    let solved = solved();
    let roof = roof(&solved);
    assert!(roof.adhesive_basis.contains("Table VIII"));
    assert!(roof.adhesive_basis.contains("120"), "the row taken is named");
    assert!(
        roof.adhesive_basis.contains("not on this machine"),
        "and what would close it is named too"
    );
}

/// The basis the whole chain rests on travels with it.
#[test]
fn the_standard_the_basis_and_the_document_all_travel_with_the_count() {
    let solved = solved();
    let roof = roof(&solved);
    assert!(roof.standard.contains("ASCE 7-22"));
    assert_eq!(roof.basis, "strength");
    assert!(roof.basis_stated);
    assert_eq!(roof.membrane.as_deref(), Some("TPO"), "read off the roof's own membrane line");
    for zone in &roof.zones {
        let printed = zone.pattern.as_ref().expect("a pattern");
        assert_eq!(printed.document_id, "DR-05");
        let asked = zone.fm_class.expect("this roof types a pressure for every zone");
        assert!(printed.fm_class >= asked, "the row covers what the zone asked for");
    }
}

/// A roof with no typed pressure is not fastened to a guess.
#[test]
fn a_roof_with_no_typed_pressure_gets_zones_and_no_pattern() {
    let value = serde_json::json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area",
                "properties": { "T": 0.5, "TAPER": 0.25, "ROOF_HEIGHT_FT": 20.0 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 60.0, "y": 0.0 },
                    { "x": 60.0, "y": 60.0 }, { "x": 0.0, "y": 60.0 }
                ] }]
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
                "traces": [{ "pageId": "page-1", "points": [{ "x": 30.0, "y": 30.0 }] }]
            }
        ],
    });
    let solved = roofnerd_core::solve(&value, 1).expect("it solves");
    let roof = solved.fastening.fields.first().expect("the zones are still set out");
    assert!(roof.a_feet.is_some(), "the zones come off the height and the roof, not the pressure");
    assert!(roof.refused.is_some(), "and it says nothing was rated");

    // Nought pressure rates at the lowest published class, which the table
    // prints no polyiso row for on a 4 ft board — so the count is unanswered
    // rather than nought, and the line will read pending.
    let scope = &solved.scopes["c-field"];
    assert_eq!(scope["FASTENERS_Z1_EA"], None);
    assert_eq!(scope["PLATES_EA"], None);
    assert_eq!(scope["ZONE_A_FT"], Some(6.0), "a tenth of the 60 ft side, which is the smaller");

    let document = Document::read(&value).expect("it reads");
    assert!(document.conditions.iter().any(|c| c.id == "c-field"));
}

/// A roof with no typed HEIGHT has no zones set out, and still names all three.
///
/// A roof has corners whether or not anybody has said how tall it is. What is
/// missing is the width to set them out at, so the names are there and
/// unanswered — which reads as pending on a line and says which name — rather
/// than absent, which would read as a roof that has no corners.
#[test]
fn a_roof_with_no_typed_height_names_its_zones_and_answers_none_of_them() {
    let value = serde_json::json!({
        "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
        "conditions": [
            {
                "id": "c-field", "kind": "area",
                "properties": { "T": 0.5, "TAPER": 0.25, "WIND_PSF_CORNER": 100.0 },
                "traces": [{ "pageId": "page-1", "points": [
                    { "x": 0.0, "y": 0.0 }, { "x": 60.0, "y": 0.0 },
                    { "x": 60.0, "y": 60.0 }, { "x": 0.0, "y": 60.0 }
                ] }]
            },
            {
                "id": "c-drains", "kind": "count", "role": "drain",
                "traces": [{ "pageId": "page-1", "points": [{ "x": 30.0, "y": 30.0 }] }]
            }
        ],
    });
    let solved = roofnerd_core::solve(&value, 1).expect("it solves");
    let roof = solved.fastening.fields.first().expect("there is still a roof");
    assert!(roof.zones.is_empty());
    let scope = &solved.scopes["c-field"];
    for name in [
        "FASTENERS_Z1_EA", "FASTENERS_Z2_EA", "FASTENERS_Z3_EA",
        "FM_CLASS_Z1", "FM_CLASS_Z2", "FM_CLASS_Z3",
        "ADHESIVE_SETS_Z1_EA", "ADHESIVE_SETS_Z2_EA", "ADHESIVE_SETS_Z3_EA",
        "WIND_ASD_Z3_PSF", "ZONE_A_FT", "PLATES_EA",
    ] {
        assert!(scope.contains_key(name), "{name} is not even named");
        assert_eq!(scope[name], None, "{name} is answered on a roof with no height");
    }
    // And the membrane's own bonding adhesive is not a wind question at all, so
    // it is answered.
    assert!(scope["BONDING_GAL"].is_some());
}
