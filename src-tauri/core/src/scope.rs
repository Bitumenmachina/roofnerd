// ── The named numbers a condition carries ──────────────────────────────────
// A solve produces geometry AND numbers a formula can use by name: the square
// feet of X panel, the sump count, the cricket's footprint. Those names ride the
// document event, because they are hundreds of bytes; the geometry is pulled on
// demand, because it is hundreds of kilobytes.
//
// C0 shipped the map and filled nothing into it, so that the wire, the merge into
// a formula's scope and the collision rule against the reserved property names
// were all wired and tested before there was a bill to put on them. C2 is the
// bill.
//
// THREE KINDS OF ANSWER, AND THEY ARE DIFFERENT. A name with a number is solved.
// A name with `None` is one the solve knows about and cannot answer — that reads
// as pending on the line and says which name (`packages/engine/src/line.ts`),
// never as zero. A name that is ABSENT is one this roof has no such thing of, and
// a formula naming it says its own name back. The three are not
// interchangeable: nought square feet of ZZ panel means the layout uses none,
// which is a fact; `None` against `BOARDS_MAX` means nobody can say how many
// courses it takes, which is a gap; and no `TAPER_ZZ_SF` at all means the chart
// this field is drawn against has no such letter.
//
// The names are the plan's, unit-suffixed, and two are C2's own:
//
//   TAPER_<letter>_SF   square feet of that lettered panel BOUGHT — a whole board
//                       apiece, cut or not, because one board is what a cut board
//                       costs. One name per letter the chart can reach
//   FILL_SF             square feet of flat board bought, counting each course
//   FILL_BOARDS_EA      how many flat boards that is
//   BOARD_WASTE_SF      of the tapered course, what the cuts take off. Not the
//                       item's waste allowance — that is for damage and trim, and
//                       adding the two charges the cut twice. Each fill course
//                       repeats the same cut
//   BOARDS_MAX          courses at the deepest position, the tapered panel
//                       included. What the layout NEEDS, beside the `BOARDS` the
//                       estimator typed, which is a ceiling and not a count
//   NOFALL_SF           ground the cap holds flat
//   SUMP_PANELS_EA      whole sump panels, on the drains
//   CRICKET_SF          a cricket's footprint. Cricket stock is its own lettered
//                       series and the chart on disk carries only the primary
//                       slope, so a cricket is billed by area and not by letter
//
// C3 adds the wall and curb names, and the same three-kinds-of-answer rule
// applies to them: `FLASH_SF` at nought is a wall the membrane never has to
// climb, `None` is a wall whose ceiling could not be worked out (no roof traced
// under part of it), and absent is a condition that is not a wall or curb run
// at all.
//
//   FLASH_SF            square feet under the flashing ceiling: the exact sum
//                       of trapezoids along the run's own breakpoints
//   FLASH_MIN_IN        the least the ceiling comes to anywhere on the run
//   CURB_RAISE_IN       how far short of NRCA's 8 in minimum the run's own
//                       least flashing height falls, or nought
//   COPING_LF           the condition's own `LF`, named so a formula never
//                       spells `LF * (STRETCHOUT / 12)` again
//   COPING_PCS_EA       sections plus corners
//   CLEAT_LF            both faces: `LF * 2`

use std::collections::BTreeMap;

use crate::fastening::SolvedFastening;
use crate::model::Document;
use crate::model::SolvedTaper;
use crate::walls::SolvedWalls;

/// Per condition, the numbers it carries, by name.
pub type Scopes = BTreeMap<String, BTreeMap<String, Option<f64>>>;

/// One entry per condition in the job, filled from the solve.
///
/// Every condition appears, including the ones that will never carry a derived
/// number. A window that asks for a condition it can see and gets nothing back
/// cannot tell "this condition has no derived numbers" from "the solve did not
/// reach it", and those are different facts.
pub fn scopes_of(
    document: &Document,
    taper: &SolvedTaper,
    walls: &SolvedWalls,
    fastening: &SolvedFastening,
) -> Scopes {
    let mut out = Scopes::new();
    for condition in &document.conditions {
        out.insert(condition.id.clone(), BTreeMap::new());
    }

    // ── the fields ──────────────────────────────────────────────────────────
    for field in &taper.fields {
        let Some(mine) = out.get_mut(&field.condition_id) else { continue };
        add(mine, "NOFALL_SF", Some(field.no_fall_square_feet));
        match &field.bill {
            None => {
                // The layout is laid out and there is no lettered board to buy it
                // out of — a slope the chart does not draw. The names the bill
                // would have carried are present and unanswered, which reads as
                // pending and names itself; the letters are absent, because
                // without a chart there is no telling which letters there would be.
                for name in ["FILL_SF", "FILL_BOARDS_EA", "BOARD_WASTE_SF", "BOARDS_MAX"] {
                    mine.insert(name.to_string(), None);
                }
            }
            Some(bill) => {
                for (letter, row) in &bill.by_letter {
                    add(mine, &format!("TAPER_{letter}_SF"), Some(row.square_feet));
                }
                add(mine, "FILL_SF", Some(bill.fill_square_feet));
                add(mine, "FILL_BOARDS_EA", Some(bill.fill_boards as f64));
                add(mine, "BOARD_WASTE_SF", Some(bill.waste_square_feet));
                // The deepest stack on the roof, not the sum of the deepest on each
                // trace: two tapered areas on one condition are two roofs, and how
                // many courses the deeper one takes is the answer for both.
                let most = bill.courses_max as f64;
                let now = mine.get("BOARDS_MAX").copied().flatten().unwrap_or(0.0);
                mine.insert("BOARDS_MAX".to_string(), Some(now.max(most)));
            }
        }
    }

    // ── the sumps, on whichever condition counted the drains ────────────────
    for sump in &taper.sumps {
        if let Some(mine) = out.get_mut(&sump.condition_id) {
            add(mine, "SUMP_PANELS_EA", Some(sump.panels as f64));
        }
    }

    // ── the crickets, traced and derived alike ──────────────────────────────
    for cricket in &taper.crickets {
        if let Some(mine) = out.get_mut(&cricket.condition_id) {
            add(mine, "CRICKET_SF", Some(cricket.plan_square_feet));
        }
    }

    // ── the walls and curbs ──────────────────────────────────────────────────
    for run in &walls.runs {
        let Some(mine) = out.get_mut(&run.condition_id) else { continue };
        add(mine, "FLASH_SF", run.flash_square_feet);
        add(mine, "FLASH_MIN_IN", run.flash_min_inches);
        add(mine, "CURB_RAISE_IN", run.curb_raise_inches);
        // Every run carries a `Coping`, but a run with no wall thickness and no
        // typed width has nothing to centre a cap on — no cap width means no
        // cap, and a run with no cap has nothing to buy sections, pieces or a
        // cleat for. Absent, not nought: naming one of these on a run with no
        // cap gets the name back off the formula parser, not a zero.
        if run.coping.cap_width_feet.is_some() {
            add(mine, "COPING_LF", Some(run.coping.run_feet));
            add(mine, "COPING_PCS_EA", Some(run.coping.pieces_ea as f64));
            add(mine, "CLEAT_LF", Some(run.coping.cleat_feet));
        }
    }

    // ── the wind zones, the fasteners and the adhesive ──────────────────────
    // `ZONE_A_FT` and the three `FM_CLASS_` are not quantities anybody buys and
    // they are named all the same: they are what the counts beside them were
    // worked out from, and a bill whose basis is only on a drawing is a bill
    // nobody can check. The same three kinds of answer apply — a count of
    // nought is a zone with no boards in it, `None` is a zone the table prints
    // no pattern for, and absent is a condition with no roof on it.
    for field in &fastening.fields {
        let Some(mine) = out.get_mut(&field.condition_id) else { continue };
        for (name, value) in crate::fastening::named(field) {
            add(mine, &name, value);
        }
    }

    out
}

/// Add a figure to a name, or start it. A condition with two traces on it carries
/// the sum, the way its own `SF` does.
fn add(into: &mut BTreeMap<String, Option<f64>>, name: &str, value: Option<f64>) {
    let now = into.get(name).copied().flatten();
    let sum = match (now, value) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, v) => v,
    };
    into.insert(name.to_string(), sum);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scopes(document: &Document) -> Scopes {
        let taper = crate::taper::solve(document, 1);
        let walls = crate::walls::solve(document, &taper, 1);
        let wind = crate::wind::solve(document, &taper, 1);
        let fastening = crate::fastening::solve(document, &taper, &wind, 1);
        scopes_of(document, &taper, &walls, &fastening)
    }

    #[test]
    fn every_condition_is_in_the_map_whether_it_carries_anything_or_not() {
        let doc = Document::read(&json!({
            "conditions": [{ "id": "c-field", "kind": "area" }, { "id": "c-curb", "kind": "line" }],
            "pages": [],
        }))
        .expect("the document reads");
        let scopes = scopes(&doc);
        assert_eq!(scopes.len(), 2);
        assert!(scopes["c-field"].is_empty(), "a condition with no taper on it carries nothing");
        assert!(scopes.contains_key("c-curb"));
    }

    /// A 32 by 32 roof, one drain in the middle, and the bill by hand.
    ///
    /// The drain is at (16, 16), so the grid runs -4 to 4 each way: 9 by 9 = 81
    /// board positions, of which 7 by 7 = 49 are whole. At a quarter in the foot a
    /// 4 ft board rises 1 in, so the thickness at the cell `m` out is 0.5 + m and
    /// the letters run X, Y, Z, ZZ, X as m goes 0..4. Cells at m: 1, 8, 16, 24, 32.
    ///
    ///   X   1 + 32 = 33 boards       Z   16
    ///   Y   8                        ZZ  24
    ///
    /// and the fill is one board under each of the 32 at m = 4, so 32 of them.
    #[test]
    fn the_field_carries_the_bill_by_letter_and_the_fill_under_it() {
        let doc = Document::read(&json!({
            "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
            "conditions": [
                {
                    "id": "c-field", "kind": "area",
                    "properties": { "T": 0.5, "TAPER": 0.25 },
                    "traces": [{ "pageId": "page-1", "points": [
                        { "x": 0.0, "y": 0.0 }, { "x": 32.0, "y": 0.0 },
                        { "x": 32.0, "y": 32.0 }, { "x": 0.0, "y": 32.0 }
                    ] }]
                },
                {
                    "id": "c-drains", "kind": "count", "role": "drain",
                    "properties": { "SUMP": 4.0 },
                    "traces": [{ "pageId": "page-1", "points": [{ "x": 16.0, "y": 16.0 }] }]
                }
            ],
        }))
        .expect("the document reads");
        let scopes = scopes(&doc);
        let field = &scopes["c-field"];
        // The sump takes the board over the drain out, so X is one short of 33.
        assert_eq!(field["TAPER_X_SF"], Some(32.0 * 16.0));
        assert_eq!(field["TAPER_Y_SF"], Some(8.0 * 16.0));
        assert_eq!(field["TAPER_Z_SF"], Some(16.0 * 16.0));
        assert_eq!(field["TAPER_ZZ_SF"], Some(24.0 * 16.0));
        assert_eq!(field["FILL_BOARDS_EA"], Some(32.0));
        assert_eq!(field["FILL_SF"], Some(32.0 * 16.0));
        assert_eq!(field["BOARDS_MAX"], Some(2.0), "one fill board and a tapered one");
        // Nothing caps this roof, so nothing on it is flat.
        assert_eq!(field["NOFALL_SF"], Some(0.0));
        // A letter the layout cannot reach is not in the map at all: naming it in a
        // formula gets the name back, which is a different fact from nought SF.
        assert!(!field.contains_key("TAPER_X1_SF"));
        // The drains carry their own panels, on their own condition.
        assert_eq!(scopes["c-drains"]["SUMP_PANELS_EA"], Some(1.0));
    }

    /// A slope the chart does not draw: the names are there and unanswered.
    ///
    /// Which is the case `line.ts` owes a sentence to. Nought would read as a roof
    /// that needs no insulation; absent would read as a name nobody has heard of;
    /// `None` reads as "nobody has solved this", and says which.
    #[test]
    fn a_field_with_no_chart_carries_the_names_unanswered_and_not_at_nought() {
        let doc = Document::read(&json!({
            "pages": [{ "id": "page-1", "feetPerUnit": 1.0 }],
            "conditions": [
                {
                    "id": "c-field", "kind": "area",
                    "properties": { "T": 0.5, "TAPER": 0.1875 },
                    "traces": [{ "pageId": "page-1", "points": [
                        { "x": 0.0, "y": 0.0 }, { "x": 32.0, "y": 0.0 },
                        { "x": 32.0, "y": 32.0 }, { "x": 0.0, "y": 32.0 }
                    ] }]
                },
                {
                    "id": "c-drains", "kind": "count", "role": "drain",
                    "traces": [{ "pageId": "page-1", "points": [{ "x": 16.0, "y": 16.0 }] }]
                }
            ],
        }))
        .expect("the document reads");
        let scopes = scopes(&doc);
        let field = &scopes["c-field"];
        assert_eq!(field["BOARDS_MAX"], None);
        assert_eq!(field["FILL_SF"], None);
        assert_eq!(field["BOARD_WASTE_SF"], None);
        assert!(field.keys().all(|k| !k.starts_with("TAPER_")));
        // The geometry is still solved: only the lettered board is missing.
        assert_eq!(field["NOFALL_SF"], Some(0.0));
        // And no sump, because nothing states a width or a bowl.
        assert!(!scopes["c-drains"].contains_key("SUMP_PANELS_EA"));
    }
}
