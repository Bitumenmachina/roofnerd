// ── What holds the roof down, board by board ───────────────────────────────
// The zones and the rating each one asks for are `wind.rs`. This is the other
// half: how many fasteners and plates go in a board in each zone, how many
// boards are in each zone, and how much adhesive the layers above the fastened
// one take.
//
// THE BASE LAYER. One board per board position is mechanically fastened to the
// deck; everything over it is adhered. That is the assembly Patrick ruled for
// this coast — a fastened flat base, adhered tapered and cover board, adhered
// membrane — and it is what the manufacturer's own table is written for: DR-05's
// 1-1/2 in row says "base layer fastened only" in as many words. So the count is
// over BOARD POSITIONS, not over the courses at each one, and the fill boards
// above the first are adhesive and not fasteners.
//
// THE PATTERN. Read out of `data/fastening/dr-05-2024-07.json`, which is
// Carlisle's Design Reference DR-05 of July 2024 transcribed off its own page
// images; Versico publishes the same family under its own cover. A pattern is
// chosen by three things and nothing else: the board's size, the rating the zone
// asks for, and which printed row the base board on this roof actually matches.
// The document's own note — "all insulation and underlayment shown are the
// minimum thickness required for the established rating" — is what makes that
// last one a comparison and not a lookup: a 4 in flat board meets a row written
// for 2 in, and meets it with the FEWEST fasteners of the rows it meets, because
// a thinner board is the one that needs more of them.
//
// The board size is the layout's own, 4 ft square on this product, and the count
// goes with it: DR-05 prints 4 per 4x4 and 8 per 4x8 for the same product at the
// same rating, which is one pattern said twice.
//
// A BOARD THAT STRADDLES takes the higher zone. A fastener pattern is a minimum,
// and half a board in a corner is a board in a corner; the number that straddle
// is reported, because on a roof where it is large the layout wants moving
// rather than the count accepting.
//
// THE ADHESIVE. Versico Manual Table VIII (PDF p.14) gives the ribbon spacing for
// an adhered package by peak gust: 12 in in the field and 6 in at the perimeter
// to 72 mph, 6 and 6 at 90, and full spray or ribbons at 4 in on centre from 100
// through 120. The table STOPS at 120 and this coast is 130, so the 120 row is
// taken and the gap is said out loud — on the drawing, not only here. Coverage is
// the manual's own: 3,500 to 3,700 square feet to a set of tanks at 12 in on
// centre (PDF p.41), which scales with the spacing, and about 60 square feet to
// the gallon of bonding adhesive per finished surface — a rate the manual states
// already includes both faces (PDF p.38).

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::geom::{common_with, Shape};
use crate::model::{Condition, Document, SolvedTaper};
use crate::wind::{SolvedWind, WindPlan, Zone};

/// The fastening patterns, as transcribed. One copy, on disk, compiled in — the
/// same arrangement `profile.rs` uses and for the same reason: this crate reads
/// no disk, and a printed table is part of the folder.
const RAW: &str = include_str!("../../../data/fastening/dr-05-2024-07.json");

/// A kept piece smaller than this is not a board in a zone, it is a rounding
/// artefact off the boolean. `boards.rs` uses the same figure for the same reason.
const SLIVER_SQUARE_FEET: f64 = 0.01;

/// Square feet of insulation an adhesive set covers at 12 in on centre.
/// Versico Manual PDF p.41 prints 3,500 to 3,700; the lower is taken, because a
/// crew that runs out mid-roof cannot buy a third of a set.
pub const ADHESIVE_SQUARE_FEET_PER_SET: f64 = 3500.0;

/// What that rate is printed at, on centre.
pub const ADHESIVE_RATE_SPACING_IN: f64 = 12.0;

/// Square feet of finished roof a gallon of bonding adhesive covers. Versico
/// Manual PDF p.38: "approximately 60 square feet per gallon per finished
/// surface (includes coverage on both substrates)" — so the rate already carries
/// the second face and doubling it would buy the same adhesive twice.
pub const BONDING_SQUARE_FEET_PER_GALLON: f64 = 60.0;

/// The highest peak gust Table VIII prints a row for.
pub const TABLE_TOP_MPH: f64 = 120.0;

/// Ribbon spacing at that row, field and perimeter alike: "FS = Full Spray or
/// Ribbons @ 4" O.C."
pub const TABLE_TOP_SPACING_IN: f64 = 4.0;

/// What the adhesive rate rests on, and what it does not.
pub const ADHESIVE_BASIS: &str =
    "Versico Manual Table VIII (p.14) stops at a 120 mph peak gust and this roof is \
     designed for more, so the 120 row is taken: full spray, or ribbons at 4 in on \
     centre, in the field and at the perimeter alike. The Florida HVHZ TPO sheet in \
     the box rates the adhered families well past this pressure and prints no \
     coverage rate of its own — it sends the reader to the evaluation report for \
     the adhesive coverage, and that report is not on this machine.";

// ── the table ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RawTable {
    publisher: String,
    document_id: String,
    document_date: String,
    minimum_thickness_note: String,
    patterns: Vec<RawPattern>,
}

#[derive(Deserialize)]
struct RawPattern {
    board: String,
    fasteners: usize,
    #[serde(default)]
    ratings: Vec<RawRating>,
}

#[derive(Deserialize)]
struct RawRating {
    fm_class: String,
    products: Vec<String>,
}

fn table() -> &'static RawTable {
    static TABLE: std::sync::OnceLock<RawTable> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| serde_json::from_str(RAW).expect("the fastening table is json"))
}

/// One printed pattern, chosen for a board on a roof.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pattern {
    pub board: String,
    pub fasteners: usize,
    /// The rating the row carries — at or above the one the zone asked for.
    pub fm_class: f64,
    /// The product line it was printed against, as the document prints it.
    pub product: String,
    /// The least thickness that row is written for, in inches.
    pub minimum_inches: f64,
    pub publisher: String,
    pub document_id: String,
    pub document_date: String,
}

/// The rating a class string carries: `1-90` is ninety pounds a square foot.
fn class_psf(text: &str) -> Option<f64> {
    text.rsplit('-').next().and_then(|n| n.parse::<f64>().ok())
}

/// The thickness a product name opens with, in inches: `2"`, `1-1/2"`, `5/8"`.
fn stated_inches(product: &str) -> Option<f64> {
    let text = product.trim();
    let quote = text.find('"')?;
    let head = text[..quote].trim();
    let (whole, fraction) = match head.split_once('-') {
        Some((w, f)) if f.contains('/') => (w.trim().parse::<f64>().ok()?, Some(f.trim())),
        _ => {
            if head.contains('/') {
                (0.0, Some(head))
            } else {
                (head.parse::<f64>().ok()?, None)
            }
        }
    };
    let part = match fraction {
        None => 0.0,
        Some(f) => {
            let (top, bottom) = f.split_once('/')?;
            top.trim().parse::<f64>().ok()? / bottom.trim().parse::<f64>().ok()?
        }
    };
    Some(whole + part)
}

/// Whether a printed product line is the board on this roof.
///
/// Three tests, all of them the document's own words. The product family has to
/// match; a row written for a recover — a board laid over an old roof rather than
/// on a deck — is not this job; and where the row names which membranes it was
/// rated with, the membrane on this roof has to be one of them.
fn product_matches(product: &str, family: &str, membrane: Option<&str>) -> bool {
    let name = product.to_lowercase();
    if !name.contains(&family.to_lowercase()) {
        return false;
    }
    if name.contains("recover") {
        return false;
    }
    let Some(membrane) = membrane else { return true };
    match name.split_once('(') {
        None => true,
        Some((_, rest)) => {
            let inside = rest.split(')').next().unwrap_or("");
            let names_membranes = ["epdm", "tpo", "pvc", "fleeceback"]
                .iter()
                .any(|m| inside.contains(m));
            !names_membranes || inside.contains(&membrane.to_lowercase())
        }
    }
}

/// The pattern for a board of this size, at or above this rating, that the board
/// on the roof actually matches.
///
/// The lowest rating that reaches what is wanted, and within it the row written
/// for the THICKEST board this one still meets — which is the row with the
/// fewest fasteners, because that is what a minimum thickness buys.
pub fn pattern_for(
    board: &str,
    rating_psf: f64,
    family: &str,
    membrane: Option<&str>,
    base_inches: f64,
) -> Option<Pattern> {
    let table = table();
    let mut best: Option<Pattern> = None;
    for printed in &table.patterns {
        if printed.board != board {
            continue;
        }
        for rated in &printed.ratings {
            let Some(psf) = class_psf(&rated.fm_class) else { continue };
            if psf < rating_psf {
                continue;
            }
            for product in &rated.products {
                if !product_matches(product, family, membrane) {
                    continue;
                }
                let Some(minimum) = stated_inches(product) else { continue };
                if minimum > base_inches {
                    continue;
                }
                let mine = Pattern {
                    board: printed.board.clone(),
                    fasteners: printed.fasteners,
                    fm_class: psf,
                    product: product.clone(),
                    minimum_inches: minimum,
                    publisher: table.publisher.clone(),
                    document_id: table.document_id.clone(),
                    document_date: table.document_date.clone(),
                };
                let better = match &best {
                    None => true,
                    Some(had) => {
                        mine.fm_class < had.fm_class
                            || (mine.fm_class == had.fm_class
                                && mine.minimum_inches > had.minimum_inches)
                    }
                };
                if better {
                    best = Some(mine);
                }
            }
        }
    }
    best
}

/// What the document says about the thickness column, for the drawing to repeat.
pub fn minimum_thickness_note() -> &'static str {
    &table().minimum_thickness_note
}

// ── the solved plan ────────────────────────────────────────────────────────

/// One zone, fastened: the wind's answer for it and the boards that sit in it.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FastenedZone {
    pub zone: String,
    pub label: String,
    pub shapes: Vec<Shape>,
    pub square_feet: f64,
    pub design_psf: Option<f64>,
    pub asd_psf: Option<f64>,
    pub required_psf: Option<f64>,
    /// The rating this zone ASKS for. Absent where nobody typed a pressure.
    pub fm_class: Option<f64>,
    /// Base boards in it, a straddler counted here and not in the lower zone.
    pub boards: usize,
    /// Of those, how many lie across this zone's own boundary.
    pub straddling: usize,
    /// Of those, how many have less under the fasteners than the printed row is
    /// written for. Reported, never quietly fastened as if they did.
    pub thin_boards: usize,
    /// What goes in a board here. Absent where the table prints no row this roof
    /// matches at the rating this zone wants.
    pub fasteners_per_board: Option<usize>,
    pub pattern: Option<Pattern>,
    pub fasteners: Option<usize>,
    /// Square feet of adhered layer over this zone: every board course above the
    /// fastened one, and the cover board over all of it.
    pub adhered_square_feet: f64,
    pub adhesive_sets: Option<usize>,
    /// Why there is no pattern, where there is none.
    pub refused: Option<String>,
}

/// One roof, fastened.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FastenedField {
    pub condition_id: String,
    pub trace: usize,
    pub ring: Vec<crate::geom::Point>,
    pub square_feet: f64,
    pub standard: String,
    pub zone_rule: String,
    pub basis: String,
    pub basis_stated: bool,
    pub height_feet: Option<f64>,
    pub least_dimension_feet: f64,
    pub a_feet: Option<f64>,
    pub a_bound: String,
    pub zone_one_prime: String,
    /// Board positions on the roof: one fastened board apiece.
    pub base_boards: usize,
    /// How many of those lie across a zone boundary and took the higher zone.
    pub straddling: usize,
    /// The flat board under the tapered package, in inches — what the fasteners
    /// go through.
    pub base_board_inches: Option<f64>,
    /// Positions where the layout puts no flat board at all and the tapered panel
    /// itself lands on the deck, so there is less under the fastener than the
    /// printed row is written for.
    pub thin_boards: usize,
    pub board_size: String,
    pub product_family: String,
    pub membrane: Option<String>,
    pub minimum_thickness_note: String,
    pub ribbon_spacing_inches: f64,
    pub adhesive_basis: String,
    pub bonding_gallons: Option<f64>,
    pub plates: Option<usize>,
    pub zones: Vec<FastenedZone>,
    pub refused: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvedFastening {
    pub solve_id: u64,
    pub fields: Vec<FastenedField>,
}

/// Which membrane this roof is getting, off its own lines.
///
/// The table rates some patterns against the membrane over them, so the answer
/// comes off the job rather than a default: a line that says what it IS in the
/// roof (`layer`) and what it is called is the same pair the section reads.
fn membrane_of(c: &Condition) -> Option<String> {
    for line in &c.items {
        if line.layer.as_deref() != Some("membrane") {
            continue;
        }
        let said = line.description.to_uppercase();
        for name in ["TPO", "PVC", "EPDM", "KEE"] {
            if said.contains(name) {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Solve the fastening on every roof the zones were set out on.
pub fn solve(
    document: &Document,
    taper: &SolvedTaper,
    wind: &SolvedWind,
    solve_id: u64,
) -> SolvedFastening {
    let mut fields = Vec::new();
    for plan in &wind.plans {
        let Some(field) = taper
            .fields
            .iter()
            .find(|f| f.condition_id == plan.condition_id && f.trace == plan.trace)
        else {
            continue;
        };
        let Some(c) = document.conditions.iter().find(|c| c.id == plan.condition_id) else {
            continue;
        };
        fields.push(field_of(c, plan, field));
    }
    SolvedFastening { solve_id, fields }
}

/// One board, placed: which zone it is in and how deep the course under it is.
struct Placed {
    zone: usize,
    straddles: bool,
    base_inches: f64,
    /// Square feet of adhered layer this position carries above the fastened one.
    adhered_square_feet: f64,
}

fn place(
    board: &crate::model::Board,
    zones: &[Zone],
    fill_step_inches: f64,
) -> Option<Placed> {
    let mut held = Vec::new();
    for (index, zone) in zones.iter().enumerate() {
        let shared: f64 = board
            .shapes
            .iter()
            .flat_map(|piece| common_with(piece, &zone.shapes))
            .map(|piece| piece.square_feet())
            .sum();
        if shared > SLIVER_SQUARE_FEET {
            held.push(index);
        }
    }
    let highest = *held.last()?;
    let base_inches = if board.fill_boards > 0 { fill_step_inches } else { board.panel_inches };
    Some(Placed {
        zone: highest,
        straddles: held.len() > 1,
        // Every course above the fastened one is adhered, and the cover board
        // over the lot is counted once per zone rather than per board.
        adhered_square_feet: board.square_feet * (board.courses.saturating_sub(1)) as f64,
        base_inches,
    })
}

fn field_of(c: &Condition, plan: &WindPlan, field: &crate::model::TaperField) -> FastenedField {
    let membrane = membrane_of(c);
    let bill = field.bill.as_ref();
    let base_board_inches = bill.map(|b| b.fill_step_inches);
    let board_size = bill
        .map(|b| format!("{}x{}", b.panel_feet as i64, b.panel_feet as i64))
        .unwrap_or_default();
    let product_family = "Polyiso HP-H/InsulBase".to_string();

    let mut out = FastenedField {
        condition_id: plan.condition_id.clone(),
        trace: plan.trace,
        ring: plan.ring.clone(),
        square_feet: plan.square_feet,
        standard: plan.standard.clone(),
        zone_rule: plan.zone_rule.clone(),
        basis: plan.basis.clone(),
        basis_stated: plan.basis_stated,
        height_feet: plan.height_feet,
        least_dimension_feet: plan.least_dimension_feet,
        a_feet: plan.a_feet,
        a_bound: plan.a_bound.clone(),
        zone_one_prime: plan.zone_one_prime.clone(),
        base_boards: 0,
        straddling: 0,
        base_board_inches,
        thin_boards: 0,
        board_size: board_size.clone(),
        product_family: product_family.clone(),
        membrane: membrane.clone(),
        minimum_thickness_note: minimum_thickness_note().to_string(),
        ribbon_spacing_inches: TABLE_TOP_SPACING_IN,
        adhesive_basis: ADHESIVE_BASIS.to_string(),
        // The membrane's own bonding adhesive is not a wind question: one rate,
        // over the finished surface, whatever the zones come to. So it is
        // answered before anything about the zones is, and a roof nobody has
        // typed a height or a pressure for still knows what it takes to stick
        // its membrane down.
        bonding_gallons: Some(plan.square_feet / BONDING_SQUARE_FEET_PER_GALLON),
        plates: None,
        zones: Vec::new(),
        refused: plan.refused.clone(),
    };

    if plan.zones.is_empty() {
        return out;
    }
    let Some(base_board_inches) = base_board_inches else {
        out.refused = Some(
            "the layout has no board bill, so there is no base board for a pattern to \
             be read against"
                .to_string(),
        );
        return out;
    };

    // The pattern per zone, off the rating the zone asked for.
    // A zone nobody typed a pressure for asks for no rating, so it is handed no
    // pattern: a pattern chosen against a pressure that was never stated would be
    // the worst kind of answer, one that looks solved.
    let patterns: Vec<Option<Pattern>> = plan
        .zones
        .iter()
        .map(|zone| {
            zone.fm_class.and_then(|rating| {
                pattern_for(
                    &board_size,
                    rating,
                    &product_family,
                    membrane.as_deref(),
                    base_board_inches,
                )
            })
        })
        .collect();

    // Every base board, placed.
    let mut boards = vec![0usize; plan.zones.len()];
    let mut straddling = vec![0usize; plan.zones.len()];
    let mut thin = vec![0usize; plan.zones.len()];
    let mut adhered = vec![0.0f64; plan.zones.len()];
    for region in &field.regions {
        for board in &region.boards {
            let Some(placed) = place(board, &plan.zones, base_board_inches) else { continue };
            boards[placed.zone] += 1;
            if placed.straddles {
                straddling[placed.zone] += 1;
                out.straddling += 1;
            }
            let wanted = patterns[placed.zone].as_ref().map(|p| p.minimum_inches).unwrap_or(0.0);
            if placed.base_inches + 1e-9 < wanted {
                thin[placed.zone] += 1;
                out.thin_boards += 1;
            }
            adhered[placed.zone] += placed.adhered_square_feet;
            out.base_boards += 1;
        }
    }

    let mut all_fasteners = 0usize;
    let mut every_zone_answered = true;
    for (index, zone) in plan.zones.iter().enumerate() {
        let pattern = patterns[index].clone();
        let fasteners = pattern.as_ref().map(|p| p.fasteners * boards[index]);
        if let Some(count) = fasteners {
            all_fasteners += count;
        } else {
            every_zone_answered = false;
        }
        // The cover board goes over the whole zone; the courses above the
        // fastened one are already counted board by board.
        let adhered_square_feet = adhered[index] + zone.square_feet;
        let per_set =
            ADHESIVE_SQUARE_FEET_PER_SET * (TABLE_TOP_SPACING_IN / ADHESIVE_RATE_SPACING_IN);
        let sets = if per_set > 0.0 {
            Some((adhered_square_feet / per_set).ceil() as usize)
        } else {
            None
        };
        out.zones.push(FastenedZone {
            zone: zone.zone.clone(),
            label: zone.label.clone(),
            shapes: zone.shapes.clone(),
            square_feet: zone.square_feet,
            design_psf: zone.design_psf,
            asd_psf: zone.asd_psf,
            required_psf: zone.required_psf,
            fm_class: zone.fm_class,
            boards: boards[index],
            straddling: straddling[index],
            thin_boards: thin[index],
            fasteners_per_board: pattern.as_ref().map(|p| p.fasteners),
            refused: match (&pattern, zone.fm_class) {
                (Some(_), _) => None,
                (None, None) => Some(
                    "nobody has typed a design pressure for this zone, so nothing rates it"
                        .to_string(),
                ),
                (None, Some(rating)) => Some(format!(
                    "the table prints no {board_size} pattern in {product_family} at FM 1-{} \
                     or above for this roof",
                    rating as i64
                )),
            },
            pattern,
            fasteners,
            adhered_square_feet,
            adhesive_sets: sets,
        });
    }

    out.plates = if every_zone_answered { Some(all_fasteners) } else { None };
    out
}

/// The named numbers one roof's fastening carries, for `scope.rs` to merge in.
pub fn named(field: &FastenedField) -> BTreeMap<String, Option<f64>> {
    let mut out: BTreeMap<String, Option<f64>> = BTreeMap::new();
    out.insert("ZONE_A_FT".to_string(), field.a_feet);
    out.insert("BASE_BOARD_IN".to_string(), field.base_board_inches);
    out.insert("BASE_THIN_EA".to_string(), Some(field.thin_boards as f64));
    out.insert("PLATES_EA".to_string(), field.plates.map(|n| n as f64));
    out.insert("BONDING_GAL".to_string(), field.bonding_gallons);
    // The three zones are named whether or not they could be set out. A roof
    // whose height nobody typed HAS corners; what it has not got is a width to
    // set them out at, and that is an unanswered name (which reads as pending
    // and says which name), not a missing one.
    for n in ["Z1", "Z2", "Z3"] {
        out.insert(format!("FASTENERS_{n}_EA"), None);
        out.insert(format!("FM_CLASS_{n}"), None);
        out.insert(format!("ADHESIVE_SETS_{n}_EA"), None);
    }
    out.insert("WIND_ASD_Z3_PSF".to_string(), None);
    for zone in &field.zones {
        let n = match zone.zone.as_str() {
            "1" => "Z1",
            "2" => "Z2",
            "3" => "Z3",
            other => other,
        };
        out.insert(format!("FASTENERS_{n}_EA"), zone.fasteners.map(|c| c as f64));
        out.insert(format!("FM_CLASS_{n}"), zone.fm_class);
        out.insert(format!("ADHESIVE_SETS_{n}_EA"), zone.adhesive_sets.map(|c| c as f64));
        if zone.zone == "3" {
            out.insert("WIND_ASD_Z3_PSF".to_string(), zone.asd_psf);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_product_line_states_its_own_least_thickness() {
        assert_eq!(stated_inches("2\" Polyiso HP-H/Insulbase or SecurShield"), Some(2.0));
        assert_eq!(stated_inches("1-1/2\" Polyiso HP-H/InsulBase"), Some(1.5));
        assert_eq!(stated_inches("1/2\" Dens Deck Prime"), Some(0.5));
        assert_eq!(stated_inches("5/8\" Securock"), Some(0.625));
        assert_eq!(stated_inches("7/16\" OSB (EPDM)"), Some(0.4375));
        assert_eq!(stated_inches("a board nobody dimensioned"), None);
    }

    #[test]
    fn a_class_string_says_how_many_pounds_a_square_foot_it_covers() {
        assert_eq!(class_psf("1-90"), Some(90.0));
        assert_eq!(class_psf("1-285"), Some(285.0));
        assert_eq!(class_psf("nothing"), None);
    }

    /// The row for a recover is not a row for a deck, and a row rated against
    /// one membrane is not a row for another.
    #[test]
    fn a_row_is_only_this_roofs_row_where_the_document_says_it_is() {
        let family = "Polyiso HP-H/InsulBase";
        assert!(product_matches("2\" Polyiso HP-H/InsulBase", family, Some("TPO")));
        assert!(!product_matches("1\" Polyiso HP-H/InsulBase (Recover Only)", family, Some("TPO")));
        assert!(product_matches(
            "2\" Polyiso HP-H/InsulBase (EPDM, TPO and FleeceBACK)",
            family,
            Some("TPO")
        ));
        assert!(!product_matches("2\" Polyiso HP-H/InsulBase (PVC)", family, Some("TPO")));
        // A parenthesis that names no membrane at all is a note, not a rating.
        assert!(product_matches(
            "1-1/2\" Polyiso HP-H/InsulBase (base layer fastened only)",
            family,
            Some("TPO")
        ));
        assert!(!product_matches("1/2\" Dens Deck Prime", family, Some("TPO")));
    }

    /// The whole point of the lookup, on the figures the demo actually uses.
    ///
    /// A 4 in flat board meets the 2 in row and the 1-1/2 in row both; the 2 in
    /// row is the one taken, because the thicker minimum is the one written with
    /// fewer fasteners in it.
    #[test]
    fn a_thicker_board_than_the_row_asks_for_takes_the_rows_own_count() {
        let family = "Polyiso HP-H/InsulBase";
        let at_ninety = pattern_for("4x4", 90.0, family, Some("TPO"), 4.0).expect("printed");
        assert_eq!(at_ninety.fasteners, 4);
        assert_eq!(at_ninety.minimum_inches, 2.0);
        assert_eq!(at_ninety.fm_class, 90.0);

        // A board that only just meets the thinner row takes the thinner row,
        // which is the denser pattern.
        let thin = pattern_for("4x4", 90.0, family, Some("TPO"), 1.5).expect("printed");
        assert_eq!(thin.fasteners, 5);
        assert_eq!(thin.minimum_inches, 1.5);

        // And a board thinner than every printed row has no pattern at all.
        assert!(pattern_for("4x4", 90.0, family, Some("TPO"), 0.75).is_none());
    }

    /// A rating the table prints no row for is answered by the next row up, not
    /// by the nearest one below it.
    #[test]
    fn a_rating_with_no_row_of_its_own_takes_the_next_row_above() {
        let family = "Polyiso HP-H/InsulBase";
        // DR-05 prints 1-90 and 1-150 for this product on a 4x4 and nothing
        // between, so a zone wanting 1-120 is fastened to 1-150.
        let wanted = pattern_for("4x4", 120.0, family, Some("TPO"), 4.0).expect("printed");
        assert_eq!(wanted.fm_class, 150.0);
        assert_eq!(wanted.fasteners, 10);
        let at_hundred_and_fifty =
            pattern_for("4x4", 150.0, family, Some("TPO"), 4.0).expect("printed");
        assert_eq!(at_hundred_and_fifty.fasteners, wanted.fasteners);
    }

    /// One pattern said twice: a 4x8 is two 4x4s, and the table agrees.
    #[test]
    fn the_same_pattern_on_a_board_twice_the_size_is_twice_the_fasteners() {
        let family = "Polyiso HP-H/InsulBase";
        for rating in [90.0, 150.0, 195.0] {
            let small = pattern_for("4x4", rating, family, Some("TPO"), 4.0).expect("printed");
            let big = pattern_for("4x8", rating, family, Some("TPO"), 4.0).expect("printed");
            assert_eq!(big.fasteners, small.fasteners * 2, "at FM 1-{rating}");
        }
    }

    /// Versico's own warranty table, against Carlisle's pattern sheet.
    ///
    /// Table VIII (Manual PDF p.14) prints 8 fasteners per 4 by 8 board for 2 in
    /// polyiso at its 55 and 72 mph rows — the rows an FM 1-90 assembly covers.
    /// Two documents, two covers, one answer.
    #[test]
    fn the_manuals_own_table_and_the_pattern_sheet_give_the_same_count() {
        let printed =
            pattern_for("4x8", 90.0, "Polyiso HP-H/InsulBase", Some("TPO"), 2.0).expect("printed");
        assert_eq!(printed.fasteners, 8);
    }

    /// NRCA's worked example is in the table this reads.
    ///
    /// p.207: "an FM 1-90-rated roof assembly may have six fasteners per 4- by
    /// 8-foot insulation board in the field zones, 10 fasteners in the perimeter
    /// zones and 16 fasteners in the corner zones." All three are printed rows at
    /// FM 1-90 on a 4x8 board, for the products each is written for. The example
    /// is one assembly enhanced toward its corners; the chain here rates each
    /// zone on its own and reads the row for it. Both arrive at the same sheet.
    #[test]
    fn the_worked_example_in_the_book_is_in_the_table_on_disk() {
        for count in [6usize, 10, 16] {
            let there = table().patterns.iter().any(|p| {
                p.board == "4x8"
                    && p.fasteners == count
                    && p.ratings.iter().any(|r| r.fm_class == "1-90")
            });
            assert!(there, "no 4x8 pattern of {count} at FM 1-90");
        }
    }

    /// The adhesive rate scales with the spacing, and the coverage is the
    /// manual's own.
    #[test]
    fn tightening_the_ribbons_to_a_third_of_the_spacing_thirds_the_coverage() {
        let at_twelve = ADHESIVE_SQUARE_FEET_PER_SET;
        let at_four = ADHESIVE_SQUARE_FEET_PER_SET * (TABLE_TOP_SPACING_IN / ADHESIVE_RATE_SPACING_IN);
        assert!((at_four * 3.0 - at_twelve).abs() < 1e-9);
        // A square of roof at 4 in on centre takes three times the adhesive it
        // takes at 12, which is what "full spray or ribbons at 4 in" costs.
        assert!(at_four < at_twelve);
    }
}
