// ── The verbs ──────────────────────────────────────────────────────────────
// What a roofer does ON the roof, asked of the solve before anything is written.
//
// Everything up to here is solved from what the job already states: the
// estimator types a taper and a thickness and the package falls out. The acts in
// the handoff's tool rail are the other direction — set the high side and the
// low, box out a curb, measure between two places, cut a section — and each one
// is a question about the roof that has an answer before anything changes.
//
// So a verb is two steps and this module is the first of them:
//
//   1. the window picks two places and ASKS what they would mean. Nothing in the
//      document moves. The answer is the solve's own arithmetic run backwards —
//      the run along the boards, the fall over it — or a sentence saying why the
//      picks cannot be honoured
//   2. the window writes what the answer says, through `doc_set`, and the whole
//      model is solved again from the document the way it is after every other
//      edit
//
// The reason the first step is here and not in the window is the rule the whole
// of this crate exists for: two arithmetics for one number disagree eventually,
// and the one that disagrees silently is the drawing. A slope worked out in the
// view from a run the view measured would be a second layout beside the solved
// one. The run is measured here, on the field's own axes, by the same expression
// `read_at` reads a thickness with.

use serde::{Deserialize, Serialize};

use crate::geom::{inside, Point};
use crate::model::{Document, SolvedTaper, TaperField};
use crate::taper::{cap_inches, depth_inches_at, perimeter_inches, region_at, surface_feet};

/// How wide a thing on the roof has to be across the fall before NRCA asks for a
/// cricket behind it, in feet.
///
/// *Membrane Roof Systems* §8.1 p.252 and Figs. 10-13/10-14 pp.309-310: water
/// backs up behind a curb, and a unit more than two feet across the fall is wide
/// enough that it ponds there. Narrower than this the water goes round on its own
/// and a cricket is stock nobody buys.
pub const CRICKET_WANTED_FEET: f64 = 2.0;

// ── set high and low ───────────────────────────────────────────────────────

/// Two picks and the two thicknesses they stand for.
///
/// The thicknesses are typed in the strip, which is what the handoff's Frame 2
/// draws: "High 4½″ at parapet (pick 1) · Low ½″ at Drain 1 (pick 2) · Slope ◈".
/// The slope is the one thing on that strip carrying the derived mark, and this
/// is where it is derived.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighAndLow {
    pub high: Point,
    pub low: Point,
    /// Absent on the first ask, which is what the strip is filled FROM: the two
    /// places are picked before either thickness is typed, and the answer says
    /// what the roof is at each of them today so the estimator edits a real
    /// number rather than an empty field.
    #[serde(default)]
    pub high_inches: Option<f64>,
    #[serde(default)]
    pub low_inches: Option<f64>,
}

/// What two picks would make of the roof.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HighAndLowAnswer {
    /// The tapered area the high pick landed on.
    pub condition_id: Option<String>,
    /// The drain the low pick took, and whose count condition owns it.
    pub drain_condition_id: Option<String>,
    pub drain_at: Option<Point>,
    /// What the drain is called, so the strip can say which one it took.
    pub drain_named: Option<String>,
    /// Which drain on the ROOF it is, from one — the same number the drawing
    /// writes on it and the same one the Model's own label carries. Numbered
    /// across the job and not within one condition, because "Drain 2" has to
    /// mean the same drain wherever it is written.
    pub drain_number: Option<usize>,
    /// The run along the boards from the drain to the high pick, in feet. This is
    /// the run a tapered schedule states its thickness against — not the straight
    /// line, which on a four-way layout is the diagonal.
    pub run_feet: f64,
    /// What the field's `TAPER` would become, in inches per foot.
    pub taper_inches_per_foot: Option<f64>,
    /// What the field's `T` would become, in inches.
    pub thickness_inches: Option<f64>,
    /// Where the roof would stop rising, and how far short of the high pick that
    /// leaves it. Not a refusal — a roof that runs out of board is a roof with a
    /// flat patch on it, which this program already draws and says out loud.
    pub cap_inches: Option<f64>,
    /// What the roof is at each pick as it stands, in inches. The strip is seeded
    /// from these, so the estimator edits the roof they can see rather than an
    /// empty field, and so the seeding is a reading of the solve and not a second
    /// arithmetic in the window.
    pub high_now_inches: Option<f64>,
    pub low_now_inches: Option<f64>,
    /// Courses at the deepest position, beside the ceiling the job typed. Frame
    /// 2's strip carries both and the difference is a purchase somebody makes.
    pub boards_needed: Option<f64>,
    pub boards_allowed: Option<f64>,
    /// The drain's own sump as it stands: how wide it is typed and how far it
    /// falls. Frame 2's strip carries both, the second with the derived mark.
    pub sump_feet: Option<f64>,
    pub sump_depth_inches: Option<f64>,
    pub note: Option<String>,
    /// Why the picks cannot be honoured, in a sentence. Nothing is written when
    /// this is set.
    pub refused: Option<String>,
}

impl HighAndLowAnswer {
    fn refused(why: &str) -> Self {
        Self {
            condition_id: None,
            drain_condition_id: None,
            drain_at: None,
            drain_named: None,
            drain_number: None,
            run_feet: 0.0,
            taper_inches_per_foot: None,
            thickness_inches: None,
            cap_inches: None,
            high_now_inches: None,
            low_now_inches: None,
            boards_needed: None,
            boards_allowed: None,
            sump_feet: None,
            sump_depth_inches: None,
            note: None,
            refused: Some(why.to_string()),
        }
    }
}

/// One drainage point, with the condition that counted it.
struct Counted {
    at: Point,
    condition_id: String,
    named: String,
    number: usize,
}

/// Every drainage point in the job, in feet, with what it is called.
///
/// `taper.rs` gathers the same points for the layout in this same order; this
/// keeps the NAME and the NUMBER, because a strip that says "Low 1/2\" at Drain
/// 1" is the estimator's own confirmation that the pick took the drain they
/// meant — and it can only be that if the number is the one written on the same
/// drain everywhere else. So it runs across the whole job, the way
/// `drawing::which_drain` numbers them, and not within one condition.
fn counted_drains(document: &Document) -> Vec<Counted> {
    let scales = document.scales();
    let mut out = Vec::new();
    let mut number = 0usize;
    for c in &document.conditions {
        if !c.has_role("drain") || c.hidden {
            continue;
        }
        for trace in &c.traces {
            for at in document.feet(trace, &scales) {
                number += 1;
                out.push(Counted {
                    at,
                    condition_id: c.id.clone(),
                    named: c.name.clone(),
                    number,
                });
            }
        }
    }
    out
}

/// The tapered field a place stands on, or none.
fn field_under<'a>(taper: &'a SolvedTaper, p: Point) -> Option<&'a TaperField> {
    taper.fields.iter().find(|f| inside(p, &f.ring))
}

/// How far apart two places are along the field's own axes, in feet.
///
/// The greater of the two runs, which is exactly what `read_at` takes the maximum
/// of when it reads a four-way layout: the boards run one way, the courses step
/// the other, and the thickness at a place is set by whichever of the two has run
/// further. Running it backwards is what gives the slope two picks imply.
fn run_along_boards(field: &TaperField, from: Point, to: Point) -> f64 {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let along = dx * field.axis_along.x + dy * field.axis_along.y;
    let across = dx * field.axis_across.x + dy * field.axis_across.y;
    along.abs().max(across.abs())
}

/// What the two picks would make of the roof, or why they cannot be honoured.
pub fn high_and_low(
    document: &Document,
    taper: &SolvedTaper,
    ask: &HighAndLow,
) -> HighAndLowAnswer {
    let Some(field) = field_under(taper, ask.high) else {
        return HighAndLowAnswer::refused(
            "the high pick is not on a tapered area — pick the high side of the roof itself",
        );
    };

    // The low pick takes the nearest drain ON THIS FIELD, and the strip says
    // which one it took. There is no distance to be inside of: a drain the
    // estimator did not mean is a drain they can see named in front of them and
    // pick again, and a made-up reach would refuse picks that are perfectly
    // clear on a roof with its drains far apart.
    let mine: Vec<Counted> = counted_drains(document)
        .into_iter()
        .filter(|d| inside(d.at, &field.ring))
        .collect();
    let Some(drain) = mine.into_iter().min_by(|a, b| {
        let ra = a.at.distance(ask.low);
        let rb = b.at.distance(ask.low);
        ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
    }) else {
        return HighAndLowAnswer::refused(
            "there is no drain on this roof area, so there is nowhere for the low side to be",
        );
    };

    let run = run_along_boards(field, drain.at, ask.high);
    let sump = taper
        .sumps
        .iter()
        .find(|s| s.at.distance(drain.at) < 1e-9 && s.condition_id == drain.condition_id);
    let mut answer = HighAndLowAnswer {
        condition_id: Some(field.condition_id.clone()),
        drain_condition_id: Some(drain.condition_id.clone()),
        drain_at: Some(drain.at),
        drain_named: Some(drain.named.clone()),
        drain_number: Some(drain.number),
        run_feet: run,
        taper_inches_per_foot: None,
        thickness_inches: None,
        cap_inches: None,
        high_now_inches: depth_inches_at(field, ask.high),
        low_now_inches: Some(field.start_inches),
        boards_needed: field.bill.as_ref().map(|b| b.courses_max as f64),
        boards_allowed: document
            .conditions
            .iter()
            .find(|c| c.id == field.condition_id)
            .and_then(|c| c.property("BOARDS")),
        sump_feet: sump.map(|s| s.side_feet),
        sump_depth_inches: sump.map(|s| s.depth_inches),
        note: None,
        refused: None,
    };

    if run <= 1e-6 {
        answer.refused = Some(
            "the high pick is on the drain itself, so there is no run for the roof to fall over"
                .to_string(),
        );
        return answer;
    }
    // Neither thickness typed yet: the two places are picked and the strip is
    // about to be filled from what the roof already is at them. That is not a
    // refusal, it is the first half of the act.
    let (Some(high_inches), Some(low_inches)) = (ask.high_inches, ask.low_inches) else {
        return answer;
    };

    if !(high_inches > low_inches) {
        answer.refused = Some(format!(
            "the high side has to be thicker than the low one, and {} is not more than {}",
            crate::drawing::inches_written(high_inches),
            crate::drawing::inches_written(low_inches),
        ));
        return answer;
    }
    if low_inches <= 0.0 {
        answer.refused =
            Some("the roof has a thickness at the drain — it cannot be nothing".to_string());
        return answer;
    }

    // Rounded to the nearest sixteenth of an inch to the foot: a roofer states a
    // slope that way, the strip already prints "1/8 in 12" and not a fifth
    // decimal place, and a pick never lands on the exact point it was aimed at
    // — the screen-to-roof round trip the C6B notes flag has its own slop — so
    // the raw quotient two real picks make is not a round number even when
    // they were aimed at one. The shipped probe's own picks, aimed at an
    // eighth in twelve, wrote a slope several thousandths short of it, which
    // is not a printed slope at all: `profile::schedule` matches a chart entry
    // exactly, so it found none, the whole lettered bill went unbought, and
    // every name below it in `scope.rs` — the fill, the fasteners, the
    // adhesive, the plates — went pending with it. A tapered package is
    // bought at the slopes the manufacturer prints, so the fix is not a
    // tighter tolerance on the lookup, it is not carrying a slope nobody can
    // order in the first place. D158 addendum records this.
    let taper_now = ((high_inches - low_inches) / run * 16.0).round() / 16.0;
    answer.taper_inches_per_foot = Some(taper_now);
    answer.thickness_inches = Some(low_inches);

    // Where the roof would stop rising under the new thickness. The board count
    // the job types and the lowest thing the membrane flashes against are both
    // the estimator's, and neither moves because a slope did.
    let boards = document
        .conditions
        .iter()
        .find(|c| c.id == field.condition_id)
        .and_then(|c| c.property("BOARDS"));
    let (cap, _) = cap_inches(low_inches, boards, perimeter_inches(&document.conditions));
    answer.cap_inches = cap;
    if let Some(cap) = cap {
        if high_inches > cap + 1e-9 {
            answer.note = Some(format!(
                "the roof stops at {} — it goes flat before it reaches the high pick",
                crate::drawing::inches_written(cap),
            ));
        }
    }
    answer
}

// ── measure ────────────────────────────────────────────────────────────────

/// Two clicks, and what the roof does between them.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Between {
    pub a: Point,
    pub b: Point,
}

/// A ruler on the roof: how far, how much rise, and how thick at each end.
///
/// Nothing is written. A tape reads what is there, and the two thicknesses come
/// off the solved planes rather than off the grid the drawing happens to be on.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetweenAnswer {
    pub feet: f64,
    /// The package's depth at each end, in inches. `None` at an end that is on no
    /// tapered area, which is a real answer and not a nought.
    pub a_inches: Option<f64>,
    pub b_inches: Option<f64>,
    /// How far the finished roof climbs from the first pick to the second, in
    /// inches. Signed: a ruler dragged downhill answers a fall.
    pub rise_inches: Option<f64>,
    /// What the ends are on, for the strip.
    pub a_on: Option<String>,
    pub b_on: Option<String>,
}

pub fn between(taper: &SolvedTaper, ask: &Between) -> BetweenAnswer {
    let a_field = field_under(taper, ask.a);
    let b_field = field_under(taper, ask.b);
    let depth = |field: Option<&TaperField>, p: Point| field.and_then(|f| depth_inches_at(f, p));
    let top = |field: Option<&TaperField>, p: Point| field.map(|f| surface_feet(f, p));

    let rise = match (top(a_field, ask.a), top(b_field, ask.b)) {
        (Some(a), Some(b)) => Some((b - a) * 12.0),
        _ => None,
    };

    BetweenAnswer {
        feet: ask.a.distance(ask.b),
        a_inches: depth(a_field, ask.a),
        b_inches: depth(b_field, ask.b),
        rise_inches: rise,
        a_on: a_field.map(|f| f.condition_id.clone()),
        b_on: b_field.map(|f| f.condition_id.clone()),
    }
}

// ── box out a curb ─────────────────────────────────────────────────────────

/// Where the cricket behind a curb goes, asked of the roof the curb stands on.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CricketBehindAnswer {
    /// The ridge line, as the two ends a `ridge` condition is traced with. It
    /// starts on the curb's own upslope face and runs away from it.
    pub ridge: Option<[Point; 2]>,
    /// How wide that face is, in feet.
    pub face_feet: f64,
    /// Which way the water comes at the curb, as a unit vector on the plan.
    pub from: Option<Point>,
    /// Why there is no cricket, where there is none. Not every curb wants one.
    pub refused: Option<String>,
}

/// The upslope direction at a place on a field: the governing plane's own.
///
/// A plane rises along its `direction` by its slope, and whichever of a region's
/// planes has risen furthest is the one that governs there — so its direction is
/// which way the roof climbs, and the water comes down it. That is a reading of
/// the solved layout and not a second one.
fn upslope_at(field: &TaperField, p: Point) -> Option<Point> {
    let region = region_at(field, p)?;
    let mut best: Option<(f64, Point)> = None;
    for plane in &region.planes {
        let rise = plane.rise_at(p);
        if best.map(|(had, _)| rise > had).unwrap_or(true) {
            best = Some((rise, plane.direction));
        }
    }
    best.map(|(_, d)| d)
}

/// Where the cricket behind a curb runs, from the curb's own ring.
///
/// NRCA §8.1 p.252, Fig. 8-1 and Figs. 10-13/10-14 pp.309-310: the wedge sits on
/// the UPSLOPE side, its ridge square to that face and centred on it, and it is
/// as wide each side as half the face. How far it runs back the figures do not
/// dimension, so the line this answers with is square: it runs back exactly as
/// far as it reaches each way, which is the shortest tent that covers the face
/// and is well inside the length-to-width table (Fig. 4-13, which allows three
/// to one at this slope). It is an ordinary traced ridge from there on — the
/// estimator's line, on the estimator's condition — and `cricket.rs` cuts the
/// wedge from it the same way it cuts one from a line somebody drew by hand.
///
/// The edges are read as `windows(2)`, which is what `cricket::cricket_behind`
/// reads: a ring whose first corner is repeated at the end offers all four of a
/// rectangle's faces, and one that is not offers three. The verb writes a closed
/// ring for exactly that reason.
pub fn cricket_behind(taper: &SolvedTaper, ring: &[Point]) -> CricketBehindAnswer {
    let nothing = |why: &str, face: f64| CricketBehindAnswer {
        ridge: None,
        face_feet: face,
        from: None,
        refused: Some(why.to_string()),
    };
    if ring.len() < 3 {
        return nothing("a curb needs three corners before anything can stand behind it", 0.0);
    }
    let middle = Point::new(
        ring.iter().map(|p| p.x).sum::<f64>() / ring.len() as f64,
        ring.iter().map(|p| p.y).sum::<f64>() / ring.len() as f64,
    );
    let Some(field) = field_under(taper, middle) else {
        return nothing(
            "this curb is not on a tapered area, so nothing says which way the water comes at it",
            0.0,
        );
    };
    let Some(up) = upslope_at(field, middle) else {
        return nothing("no drainage area covers this curb, so the fall at it is unknown", 0.0);
    };

    // The face the water arrives at: the one whose outward side looks up the
    // slope. Outward is away from the middle of the curb, which is the same test
    // `cricket.rs` makes when it takes a traced ridge.
    let mut best: Option<(f64, Point, f64, Point)> = None;
    for pair in ring.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let run = dx.hypot(dy);
        if run < 1e-6 {
            continue;
        }
        let mid = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        let sign = if (middle.x - mid.x) * (-dy / run) + (middle.y - mid.y) * (dx / run) >= 0.0 {
            -1.0
        } else {
            1.0
        };
        let out = Point::new((-dy / run) * sign, (dx / run) * sign);
        let facing = out.x * up.x + out.y * up.y;
        if best.map(|(had, _, _, _)| facing > had).unwrap_or(true) {
            best = Some((facing, mid, run, out));
        }
    }
    let Some((_, mid, run, out)) = best else {
        return nothing("this curb has no face with any length to it", 0.0);
    };

    if run <= CRICKET_WANTED_FEET + 1e-9 {
        return nothing(
            "this curb is not more than two feet across the fall, so the water goes round it",
            run,
        );
    }

    let half = run / 2.0;
    CricketBehindAnswer {
        ridge: Some([mid, Point::new(mid.x + out.x * half, mid.y + out.y * half)]),
        face_feet: run,
        from: Some(up),
        refused: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Document;
    use serde_json::json;

    /// A square roof with one drain in the middle of it, and nothing else.
    ///
    /// Made up here rather than read off the demo: what these answers are held to
    /// is arithmetic, and arithmetic on a forty-foot square is arithmetic anybody
    /// can do in their head and check.
    fn square() -> Document {
        Document::read(&json!({
            "pages": [{ "id": "p", "feetPerUnit": 1 }],
            "conditions": [
                {
                    "id": "c-roof", "name": "Roof", "kind": "area",
                    "properties": { "T": 0.5, "TAPER": 0.25, "BOARDS": 3 },
                    "traces": [{ "pageId": "p", "points": [
                        { "x": 0, "y": 0 }, { "x": 40, "y": 0 },
                        { "x": 40, "y": 40 }, { "x": 0, "y": 40 }
                    ] }]
                },
                {
                    "id": "c-drain", "name": "Drain", "kind": "count", "role": "drain",
                    "properties": { "SUMP": 4 },
                    "traces": [{ "pageId": "p", "points": [{ "x": 20, "y": 20 }] }]
                }
            ]
        }))
        .expect("the square reads")
    }

    fn solved(document: &Document) -> SolvedTaper {
        crate::taper::solve(document, 1)
    }

    /// Two picks, and the slope is the fall over the run the boards make —
    /// rounded to the nearest sixteenth, because that is the finest a roofer
    /// ever states one and it is what a tapered chart is printed at.
    ///
    /// Ten feet along and ten across from the drain is ten feet of run — the
    /// greater of the two, which is what the four-way layout reads — so an inch
    /// and a half at the far end over half an inch at the drain is an inch over
    /// ten feet: a tenth of an inch to the foot, raw. Nothing about this is the
    /// straight-line distance, which is fourteen feet and a bit. A tenth of an
    /// inch is not a printed slope — the chart runs in sixteenths — and 0.1 is
    /// closer to 2/16 than to 1/16, so the answer the strip carries is an
    /// eighth in twelve.
    #[test]
    fn the_slope_is_the_fall_over_the_run_the_boards_make() {
        let document = square();
        let taper = solved(&document);
        let answer = high_and_low(
            &document,
            &taper,
            &HighAndLow {
                high: Point::new(30.0, 30.0),
                low: Point::new(20.0, 20.0),
                high_inches: Some(1.5),
                low_inches: Some(0.5),
            },
        );
        assert_eq!(answer.refused, None);
        assert!((answer.run_feet - 10.0).abs() < 1e-9, "{} ft of run", answer.run_feet);
        let taper_now = answer.taper_inches_per_foot.expect("a slope");
        assert!((taper_now - 0.125).abs() < 1e-9, "{taper_now} in 12, rounded from a raw 0.1");
        assert_eq!(answer.thickness_inches, Some(0.5));
        assert_eq!(answer.drain_condition_id.as_deref(), Some("c-drain"));
        assert_eq!(answer.drain_number, Some(1));
    }

    /// A run that already lands on a sixteenth is not moved by rounding it.
    ///
    /// Two and a half inches of fall over ten feet of run is a quarter in
    /// twelve exactly — 4/16 — so the roundtrip through the nearest sixteenth
    /// is a no-op, and this is the case a chart lookup on the raw quotient
    /// would also have passed. The point of pairing it with the test above is
    /// that ONE of the two carries a raw value the chart has no row for at
    /// all: `0.1` in per foot is not a printed slope, `profile::schedule`
    /// answers `None` for it, and every name `scope.rs` builds off the bill —
    /// the lettered panels, the fill, the fastener and adhesive counts — goes
    /// pending with it. The rounding here is not cosmetic; it is what keeps
    /// the verb from writing a slope nobody can buy.
    #[test]
    fn a_run_already_at_a_sixteenth_is_unchanged() {
        let document = square();
        let taper = solved(&document);
        let answer = high_and_low(
            &document,
            &taper,
            &HighAndLow {
                high: Point::new(30.0, 30.0),
                low: Point::new(20.0, 20.0),
                high_inches: Some(3.0),
                low_inches: Some(0.5),
            },
        );
        assert_eq!(answer.refused, None);
        let taper_now = answer.taper_inches_per_foot.expect("a slope");
        assert!((taper_now - 0.25).abs() < 1e-9, "{taper_now} in 12");
    }

    /// The high pick is where the roof is deep, and a low pick above it is refused.
    #[test]
    fn a_low_side_that_is_not_below_the_high_one_is_refused() {
        let document = square();
        let taper = solved(&document);
        let answer = high_and_low(
            &document,
            &taper,
            &HighAndLow {
                high: Point::new(30.0, 30.0),
                low: Point::new(20.0, 20.0),
                high_inches: Some(0.5),
                low_inches: Some(2.0),
            },
        );
        assert!(answer.refused.is_some(), "a roof cannot fall upwards");
        assert_eq!(answer.taper_inches_per_foot, None);
    }

    /// A pick off the roof is refused, and a roof with no drain on it says so.
    #[test]
    fn a_pick_with_no_roof_or_no_drain_under_it_is_refused() {
        let document = square();
        let taper = solved(&document);
        let off = high_and_low(
            &document,
            &taper,
            &HighAndLow {
                high: Point::new(200.0, 200.0),
                low: Point::new(20.0, 20.0),
                high_inches: Some(4.0),
                low_inches: Some(0.5),
            },
        );
        assert!(off.refused.expect("a sentence").contains("tapered area"));

        let drainless = Document::read(&json!({
            "pages": [{ "id": "p", "feetPerUnit": 1 }],
            "conditions": [{
                "id": "c-roof", "name": "Roof", "kind": "area",
                "properties": { "T": 0.5, "TAPER": 0.25 },
                "traces": [{ "pageId": "p", "points": [
                    { "x": 0, "y": 0 }, { "x": 40, "y": 0 },
                    { "x": 40, "y": 40 }, { "x": 0, "y": 40 }
                ] }]
            }]
        }))
        .expect("the drainless square reads");
        let taper = solved(&drainless);
        let answer = high_and_low(
            &drainless,
            &taper,
            &HighAndLow {
                high: Point::new(30.0, 30.0),
                low: Point::new(20.0, 20.0),
                high_inches: Some(4.0),
                low_inches: Some(0.5),
            },
        );
        assert!(answer.refused.is_some(), "a field with no drain has no low side");
    }

    /// The roof runs out of board before the high pick, and it says so rather
    /// than refusing: a flat patch is a roof this program already draws.
    #[test]
    fn a_high_side_past_the_boards_is_a_note_and_not_a_refusal() {
        let document = square();
        let taper = solved(&document);
        let answer = high_and_low(
            &document,
            &taper,
            &HighAndLow {
                high: Point::new(39.0, 20.0),
                low: Point::new(20.0, 20.0),
                high_inches: Some(20.0),
                low_inches: Some(0.5),
            },
        );
        assert_eq!(answer.refused, None);
        // Half an inch plus three boards of four and a half is fourteen.
        assert_eq!(answer.cap_inches, Some(14.0));
        assert!(answer.note.expect("a sentence").contains("flat"));
    }

    /// A tape reads the run, the rise and the thickness at each end.
    ///
    /// Ten feet each way from the drain at a quarter in twelve is half an inch
    /// plus two and a half: three inches. The drain itself is half an inch. So a
    /// ruler between them reads two and a half inches of rise, and its length is
    /// the straight line, which is what a tape measures.
    #[test]
    fn a_ruler_reads_the_run_the_rise_and_both_ends() {
        let document = square();
        let taper = solved(&document);
        let answer = between(
            &taper,
            &Between { a: Point::new(20.0, 20.0), b: Point::new(30.0, 20.0) },
        );
        assert!((answer.feet - 10.0).abs() < 1e-9);
        assert!((answer.a_inches.expect("a depth") - 0.5).abs() < 1e-9);
        assert!((answer.b_inches.expect("a depth") - 3.0).abs() < 1e-9);
        assert!((answer.rise_inches.expect("a rise") - 2.5).abs() < 1e-9);
    }

    /// A tape off the roof still measures; the thickness at that end does not.
    #[test]
    fn a_ruler_off_the_roof_still_has_a_length() {
        let document = square();
        let taper = solved(&document);
        let answer = between(
            &taper,
            &Between { a: Point::new(20.0, 20.0), b: Point::new(200.0, 20.0) },
        );
        assert!(answer.feet > 100.0);
        assert_eq!(answer.b_inches, None);
        assert_eq!(answer.rise_inches, None);
    }

    /// The cricket goes on the face the water comes at, and it is square.
    ///
    /// The roof climbs east away from the drain, so the water on it runs west and
    /// arrives at the curb's EAST face. That is the upslope side, the cricket
    /// stands against it, and the ridge runs on east from it half that face's
    /// width. A cricket on the downhill side would shed nothing.
    #[test]
    fn the_cricket_stands_against_the_face_the_water_comes_at() {
        let document = square();
        let taper = solved(&document);
        let ring = [
            Point::new(30.0, 16.0),
            Point::new(34.0, 16.0),
            Point::new(34.0, 24.0),
            Point::new(30.0, 24.0),
            Point::new(30.0, 16.0),
        ];
        let answer = cricket_behind(&taper, &ring);
        assert_eq!(answer.refused, None);
        let ridge = answer.ridge.expect("a ridge");
        // The east face runs eight feet, from y 16 to y 24 at x 34.
        assert!((answer.face_feet - 8.0).abs() < 1e-9, "{} ft of face", answer.face_feet);
        assert!((ridge[0].x - 34.0).abs() < 1e-9 && (ridge[0].y - 20.0).abs() < 1e-9);
        // Square: it runs back as far as it reaches each way, which is four feet.
        assert!((ridge[1].x - 38.0).abs() < 1e-9 && (ridge[1].y - 20.0).abs() < 1e-9);
    }

    /// A curb narrower than two feet across the fall gets no cricket, and says why.
    #[test]
    fn a_narrow_curb_is_told_the_water_goes_round_it() {
        let document = square();
        let taper = solved(&document);
        let ring = [
            Point::new(30.0, 19.0),
            Point::new(32.0, 19.0),
            Point::new(32.0, 20.5),
            Point::new(30.0, 20.5),
            Point::new(30.0, 19.0),
        ];
        let answer = cricket_behind(&taper, &ring);
        assert!(answer.ridge.is_none());
        assert!(answer.refused.expect("a sentence").contains("goes round"));
    }
}
