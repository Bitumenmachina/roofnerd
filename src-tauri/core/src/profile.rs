// ── The manufacturer's panel schedule ──────────────────────────────────────
// A tapered package is not a ramp. It is a stack of 4 ft boards off a printed
// table: a lettered tapered panel on top of however many flat fill boards it
// takes to reach the thickness the layout wants there. The letters, the edges
// they run between and the fill board they are drawn over are the manufacturer's
// and they are in `data/profiles/versico-versicore-mp-h.json`, transcribed off
// the bulletin's own profile drawings.
//
// The file is compiled in with `include_str!` rather than read at run time,
// because this crate touches no disk: the table is part of the program the way a
// printed chart is part of a roofer's folder. The file on disk stays the one copy
// — change it and the next build carries the change — and `data/README.md` says
// the manual on disk is what settles a dispute, not this.
//
// Which set of panels is in play is a typed choice (`PROFILE_SET`), because the
// bulletin prints two for every slope it draws and nothing in the box says which
// a job uses: a Standard set over a 2 in fill board and an Extended set over a
// 4 in one. Extended is the default — it is the one that reaches a 4 1/2 in
// tapered panel, and the one the four fixed scope names are drawn from.
//
// Nothing here computes an R value. The bulletin states no R per inch and
// neither does the manual on disk, so the solve reports the thickness at the
// drain and says that it is thickness and not resistance (NRCA p.163 refuses the
// averaged figure; it does not supply the per-inch number to replace it).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// The bulletin, as transcribed. One copy, on disk, compiled in.
const RAW: &str = include_str!("../../../data/profiles/versico-versicore-mp-h.json");

/// Which of the two printed sets of panels a job is drawn against.
///
/// Named by its fill board, because that is what actually differs: the Standard
/// set is drawn over a 2 in flat board and the Extended set over a 4 in one, and
/// every panel edge in the table follows from that. So the typed property is the
/// fill board's thickness in inches — 4 or 2 — which is a number an estimator can
/// put in a numeric field and read back without a legend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileSet {
    Standard,
    Extended,
}

impl ProfileSet {
    /// The set a typed `PROFILE_SET` names, or the default.
    ///
    /// `None` and anything that is not one of the two fill thicknesses reads as
    /// Extended, which is the default the plan sets. A number nobody recognises is
    /// not silently honoured: `profile_set_refused` says so.
    pub fn of(typed: Option<f64>) -> Self {
        match typed {
            Some(t) if (t - 2.0).abs() < 1e-9 => Self::Standard,
            _ => Self::Extended,
        }
    }

    /// A typed `PROFILE_SET` that names neither set, so the default was used.
    pub fn refused(typed: Option<f64>) -> Option<f64> {
        match typed {
            Some(t) if (t - 2.0).abs() < 1e-9 || (t - 4.0).abs() < 1e-9 => None,
            other => other,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Extended => "extended",
        }
    }
}

/// One lettered tapered panel: the designation printed on it, and the two edges
/// it runs between in inches.
///
/// The range is half open — `thin_edge_in <= t < thick_edge_in`. The table prints
/// shared edges (Y starts where X stops) and says nothing about which panel owns
/// one, so the rule is stated here: a thickness that lands exactly on a shared
/// edge takes the panel that edge is the THIN edge of. That is the panel whose
/// own low corner is the thickness asked for, so the board is cut at a printed
/// edge instead of at a dimension between two of them.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Panel {
    pub designation: String,
    pub thin_edge_in: f64,
    pub thick_edge_in: f64,
}

impl Panel {
    /// Does this panel cover a thickness? Thin edge in, thick edge out.
    pub fn covers(&self, inches: f64) -> bool {
        inches >= self.thin_edge_in - 1e-9 && inches < self.thick_edge_in - 1e-9
    }

    /// How far the panel itself rises across its own width, in inches.
    pub fn rise_in(&self) -> f64 {
        self.thick_edge_in - self.thin_edge_in
    }
}

/// One set of panels at one slope, with the flat board they stack on.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub manufacturer: String,
    pub product: String,
    pub document_id: String,
    pub document_date: String,
    pub set: ProfileSet,
    /// The slope as the bulletin prints it, in inches per foot.
    pub slope_inches_per_foot: f64,
    /// The flat fill board, in inches. `fill` is always a whole number of these.
    pub fill_step_inches: f64,
    /// How big a board is, in feet. Square, on this product.
    pub panel_feet: f64,
    pub panels: Vec<Panel>,
}

impl Schedule {
    /// How much flat board goes under a tapered panel to reach a thickness.
    ///
    /// `fill = step x floor((thickness - start) / step)`. The remainder is what
    /// the tapered panel itself has to be, and it lands in `[start, start + step)`
    /// by construction — which is exactly the span the four Extended panels at a
    /// quarter in twelve cover off a half inch drain.
    pub fn fill_inches(&self, thickness_inches: f64, start_inches: f64) -> f64 {
        if self.fill_step_inches <= 0.0 {
            return 0.0;
        }
        let over = thickness_inches - start_inches;
        if over <= 0.0 {
            return 0.0;
        }
        // A hair of slack, so a thickness that is a fill board to the last bit of
        // a float does not fall back a whole board.
        self.fill_step_inches * ((over + 1e-9) / self.fill_step_inches).floor()
    }

    /// The panel that covers a thickness, or nothing where the table has none.
    ///
    /// `None` is not a refusal to answer — it is the answer, and the bill counts
    /// it. A layout that starts thinner than the table's thinnest panel cannot be
    /// built out of this product, and saying so beats picking the nearest letter.
    pub fn panel_for(&self, panel_inches: f64) -> Option<&Panel> {
        self.panels.iter().find(|p| p.covers(panel_inches))
    }

    /// The panels a layout off this drain thickness can actually reach.
    ///
    /// The tapered panel is always `[start, start + step)` thick, so most of a
    /// long table is out of reach the moment the fill boards do the lifting. The
    /// bill reports these and only these, each at nought where the layout used
    /// none — a letter the layout did not need is a fact about the layout, and it
    /// is not the same fact as a name nobody solved.
    pub fn reachable(&self, start_inches: f64) -> Vec<&Panel> {
        let top = start_inches + self.fill_step_inches;
        self.panels
            .iter()
            .filter(|p| p.thick_edge_in > start_inches + 1e-9 && p.thin_edge_in < top - 1e-9)
            .collect()
    }
}

// ── the file, read once ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RawProfile {
    manufacturer: String,
    product: String,
    document_id: String,
    document_date: String,
    panel_size_ft: Vec<f64>,
    slopes: Vec<RawSlope>,
}

#[derive(Deserialize)]
struct RawSlope {
    slope_in_per_ft: f64,
    #[serde(default)]
    standard: Option<RawSet>,
    #[serde(default)]
    extended: Option<RawSet>,
}

#[derive(Deserialize)]
struct RawSet {
    /// As printed: "4.0 ISO". The number is the flat board's thickness in inches.
    fill_in: String,
    panels: Vec<RawPanel>,
}

#[derive(Deserialize)]
struct RawPanel {
    designation: String,
    thin_edge_in: f64,
    thick_edge_in: f64,
}

fn profile() -> &'static RawProfile {
    static ONCE: OnceLock<RawProfile> = OnceLock::new();
    ONCE.get_or_init(|| {
        serde_json::from_str(RAW).expect("the profile table in data/profiles is shaped like itself")
    })
}

/// The number at the front of a printed fill column: "4.0 ISO" is 4 inches.
fn fill_step_of(printed: &str) -> f64 {
    printed
        .split_whitespace()
        .next()
        .and_then(|n| n.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// The schedule for one slope and one set, or nothing.
///
/// `None` where the bulletin prints no panel table for that slope — three of its
/// six listed slopes are named in the text with no chart beside them, and a
/// schedule invented for one of those would be a table nobody published.
pub fn schedule(slope_inches_per_foot: f64, set: ProfileSet) -> Option<Schedule> {
    let raw = profile();
    let slope = raw
        .slopes
        .iter()
        .find(|s| (s.slope_in_per_ft - slope_inches_per_foot).abs() < 1e-9)?;
    let chosen = match set {
        ProfileSet::Standard => slope.standard.as_ref(),
        ProfileSet::Extended => slope.extended.as_ref(),
    }?;
    Some(Schedule {
        manufacturer: raw.manufacturer.clone(),
        product: raw.product.clone(),
        document_id: raw.document_id.clone(),
        document_date: raw.document_date.clone(),
        set,
        slope_inches_per_foot: slope.slope_in_per_ft,
        fill_step_inches: fill_step_of(&chosen.fill_in),
        // Square boards on this product, and the short side is what a grid cell is.
        panel_feet: raw.panel_size_ft.iter().copied().fold(f64::INFINITY, f64::min),
        panels: chosen
            .panels
            .iter()
            .map(|p| Panel {
                designation: p.designation.clone(),
                thin_edge_in: p.thin_edge_in,
                thick_edge_in: p.thick_edge_in,
            })
            .collect(),
    })
}

/// Every slope the bulletin prints a chart for, in inches per foot.
pub fn slopes_charted(set: ProfileSet) -> Vec<f64> {
    profile()
        .slopes
        .iter()
        .filter(|s| match set {
            ProfileSet::Standard => s.standard.is_some(),
            ProfileSet::Extended => s.extended.is_some(),
        })
        .map(|s| s.slope_in_per_ft)
        .collect()
}

/// What one board position comes to: a lettered panel over so much flat board.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    /// The letter printed on the tapered panel, or nothing where the table has none.
    pub letter: Option<String>,
    /// How thick the tapered panel is at the point it was read, in inches.
    pub panel_inches: f64,
    pub fill_inches: f64,
    pub fill_boards: usize,
    /// Layers of board at this position: the fill boards and the tapered one.
    pub courses: usize,
}

/// The stack a thickness comes to on this schedule.
pub fn stack_at(schedule: &Schedule, thickness_inches: f64, start_inches: f64) -> Stack {
    let fill_inches = schedule.fill_inches(thickness_inches, start_inches);
    let panel_inches = thickness_inches - fill_inches;
    let fill_boards = if schedule.fill_step_inches > 0.0 {
        (fill_inches / schedule.fill_step_inches).round() as usize
    } else {
        0
    };
    Stack {
        letter: schedule.panel_for(panel_inches).map(|p| p.designation.clone()),
        panel_inches,
        fill_inches,
        fill_boards,
        courses: fill_boards + 1,
    }
}

/// How many square feet of each letter, and how many boards — the bill's core row.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LetterRow {
    pub boards: usize,
    /// Square feet of that letter BOUGHT: a whole board apiece, cut or not.
    pub square_feet: f64,
}

/// The letters a schedule can reach, each starting at nothing.
pub fn empty_rows(schedule: &Schedule, start_inches: f64) -> BTreeMap<String, LetterRow> {
    schedule
        .reachable(start_inches)
        .into_iter()
        .map(|p| (p.designation.clone(), LetterRow::default()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bulletin's quarter in twelve, Extended: four letters over a 4 in board.
    #[test]
    fn the_extended_quarter_table_is_the_one_the_bill_is_drawn_from() {
        let s = schedule(0.25, ProfileSet::Extended).expect("the bulletin charts a quarter");
        assert_eq!(s.fill_step_inches, 4.0);
        assert_eq!(s.panel_feet, 4.0);
        assert_eq!(s.manufacturer, "Versico");
        // Every panel rises exactly its own width at the slope it is cut to: 4 ft
        // of board at a quarter in the foot is one inch, which is why reading the
        // thickness at a board's middle and picking the panel that covers it is one
        // rule and not two.
        for panel in &s.panels {
            assert!(
                (panel.rise_in() - s.slope_inches_per_foot * s.panel_feet).abs() < 1e-9,
                "{} rises {} in across a {} ft board",
                panel.designation,
                panel.rise_in(),
                s.panel_feet
            );
        }
        // Off a half inch drain the fill boards leave the tapered panel between a
        // half and four and a half, which is the four letters and no more.
        let reachable: Vec<&str> = s.reachable(0.5).iter().map(|p| p.designation.as_str()).collect();
        assert_eq!(reachable, vec!["X", "Y", "Z", "ZZ"]);
    }

    /// The plan's own hand-checked figure: 32 ft of run comes to 8 1/2 in, which
    /// is 8 in of flat board under an X.
    #[test]
    fn thirty_two_feet_of_run_is_eight_inches_of_fill_under_an_x() {
        let s = schedule(0.25, ProfileSet::Extended).expect("the bulletin charts a quarter");
        let thickness = 0.5 + 32.0 * 0.25;
        assert_eq!(thickness, 8.5);
        let stack = stack_at(&s, thickness, 0.5);
        assert_eq!(stack.fill_inches, 8.0);
        assert_eq!(stack.fill_boards, 2);
        assert_eq!(stack.panel_inches, 0.5);
        assert_eq!(stack.letter.as_deref(), Some("X"));
        assert_eq!(stack.courses, 3);
    }

    /// A thickness exactly on a shared edge takes the panel above it — the one
    /// that edge is the thin edge of. The table prints no rule, so this is ours.
    #[test]
    fn a_thickness_on_a_shared_edge_takes_the_panel_above_it() {
        let s = schedule(0.25, ProfileSet::Extended).expect("the bulletin charts a quarter");
        // 1 1/2 in is X's thick edge and Y's thin edge.
        assert_eq!(s.panel_for(1.5).map(|p| p.designation.as_str()), Some("Y"));
        assert_eq!(s.panel_for(1.4999).map(|p| p.designation.as_str()), Some("X"));
        // And mid range is not a boundary case at all.
        assert_eq!(s.panel_for(3.0).map(|p| p.designation.as_str()), Some("Z"));
    }

    /// Fill steps a whole board at a time, and it steps where the board does.
    #[test]
    fn fill_steps_a_whole_board_at_a_time() {
        let s = schedule(0.25, ProfileSet::Extended).expect("the bulletin charts a quarter");
        // Just under a board's worth of rise: no fill yet, and the thickest panel.
        let under = stack_at(&s, 4.4, 0.5);
        assert_eq!(under.fill_inches, 0.0);
        assert_eq!(under.letter.as_deref(), Some("ZZ"));
        assert_eq!(under.courses, 1);
        // Exactly a board's worth: one fill board, and back to the thinnest panel.
        let over = stack_at(&s, 4.5, 0.5);
        assert_eq!(over.fill_inches, 4.0);
        assert_eq!(over.fill_boards, 1);
        assert_eq!(over.letter.as_deref(), Some("X"));
        assert_eq!(over.courses, 2);
    }

    /// A layout thinner than the table's thinnest panel has no letter, and the
    /// answer says so rather than picking the nearest one.
    #[test]
    fn a_thickness_the_table_does_not_cover_has_no_letter() {
        let s = schedule(0.25, ProfileSet::Extended).expect("the bulletin charts a quarter");
        let stack = stack_at(&s, 0.25, 0.25);
        assert_eq!(stack.fill_inches, 0.0);
        assert_eq!(stack.letter, None);
    }

    /// The Standard set is the other chart on the same page, over a 2 in board.
    #[test]
    fn the_standard_set_is_two_letters_over_a_two_inch_board() {
        let s = schedule(0.25, ProfileSet::Standard).expect("the bulletin charts a quarter");
        assert_eq!(s.fill_step_inches, 2.0);
        let reachable: Vec<&str> = s.reachable(0.5).iter().map(|p| p.designation.as_str()).collect();
        assert_eq!(reachable, vec!["X", "Y"]);
        assert_eq!(ProfileSet::of(Some(2.0)), ProfileSet::Standard);
        assert_eq!(ProfileSet::of(Some(4.0)), ProfileSet::Extended);
        assert_eq!(ProfileSet::of(None), ProfileSet::Extended);
        // A number that names neither set is not honoured in silence.
        assert_eq!(ProfileSet::refused(Some(3.0)), Some(3.0));
        assert_eq!(ProfileSet::refused(Some(4.0)), None);
    }

    /// A slope the bulletin lists but charts no panels for has no schedule, and
    /// the solve says so instead of drawing one nobody published.
    #[test]
    fn a_slope_with_no_printed_chart_has_no_schedule() {
        assert!(schedule(0.1875, ProfileSet::Extended).is_none());
        assert!(schedule(0.0625, ProfileSet::Extended).is_none());
        let charted = slopes_charted(ProfileSet::Extended);
        assert_eq!(charted, vec![0.125, 0.25, 0.5]);
    }
}
