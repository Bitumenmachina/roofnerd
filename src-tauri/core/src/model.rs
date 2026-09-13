// ── The types on the wire ──────────────────────────────────────────────────
// Two halves: the job as it arrives, and the solved model as it goes out.
//
// The solved half is serialized `camelCase`, because the other end of the wire
// is TypeScript and `packages/engine/src/solved.ts` is its mirror. That file is
// the only copy of these names on the front end — no app-side clones (D108).
//
// The arriving half reads the document the shell holds, which is the job's own
// on-disk shape: conditions with traces in PAGE units, and pages carrying the
// scale that turns those into feet. Unknown fields are ignored on purpose — a
// condition carries a name, a colour, items and a waste, and none of them is
// geometry.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::geom::{Plane, Point, Shape};
use crate::profile::{LetterRow, ProfileSet};

// ── the job, arriving ──────────────────────────────────────────────────────

/// One traced shape, in the units of the page it was drawn on.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    pub page_id: String,
    #[serde(default)]
    pub points: Vec<Point>,
}

/// A sheet, and what a page unit is worth in feet on it. A sheet nobody has
/// scaled has no answer, and a trace on it has no length — never a zero.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    #[serde(default)]
    pub feet_per_unit: Option<f64>,
}

/// A condition, as far as geometry is concerned.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    /// The reserved property names and anything else the estimator named, each
    /// a number. Held as JSON so that a property which is not a number is
    /// simply absent rather than fatal to the whole solve.
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
    #[serde(default)]
    pub traces: Vec<Trace>,
    /// What a ridge serves: the drains, or the curb it stands behind.
    #[serde(default)]
    pub between: Vec<String>,
    /// What the estimator called it. A solver has no use for a name; a DRAWING
    /// does, because every label on a section is the estimator's own word for the
    /// thing it points at, and inventing one here would be the drawing naming the
    /// job back at them.
    #[serde(default)]
    pub name: String,
    /// The lines on it. Only the two fields a drawing reads: what each one is
    /// called and how thick it is where a thickness is stated. Money is the
    /// engine's and nothing here prices anything.
    #[serde(default)]
    pub items: Vec<Line>,
}

/// One line on a condition, as far as a drawing is concerned.
///
/// A line that carries a thickness is a LAYER: it is something that goes on the
/// roof and takes up room in a section. A line with no thickness is labour, a
/// dumpster or a warranty, and a section has nothing to draw for it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub thickness: Option<f64>,
}

impl Condition {
    /// The lines on it that take up room in a section, in the order they are
    /// listed — which is the order they go on, bottom to top. Nothing in the job
    /// format states a layer order, and the list is the only order there is; the
    /// drawing says so on itself rather than guessing at a stack.
    pub fn layers(&self) -> Vec<&Line> {
        self.items.iter().filter(|i| i.thickness.map(|t| t > 0.0).unwrap_or(false)).collect()
    }
}

impl Condition {
    /// A property as a number, or nothing. `T` absent and `T` set to a word are
    /// the same fact to a solver: nobody has said.
    pub fn property(&self, name: &str) -> Option<f64> {
        self.properties.get(name).and_then(Value::as_f64)
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.role.as_deref() == Some(role)
    }
}

/// The open job, as much of it as a solver reads.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub pages: Vec<Page>,
}

impl Document {
    /// Read the document the shell is holding. An error here means the job is
    /// not shaped like a job, and the window says so rather than drawing half
    /// of it.
    pub fn read(value: &Value) -> Result<Self, String> {
        Self::deserialize(value).map_err(|e| e.to_string())
    }

    /// Feet to the page unit, per page. A page with no scale is simply absent,
    /// which is what makes a trace on it have no feet.
    pub fn scales(&self) -> BTreeMap<String, f64> {
        let mut out = BTreeMap::new();
        for page in &self.pages {
            match page.feet_per_unit {
                Some(s) if s != 0.0 => {
                    out.insert(page.id.clone(), s);
                }
                _ => {}
            }
        }
        out
    }

    /// A trace's points in feet. Empty when its sheet has no scale — never
    /// zero, because an unscaled trace has no length.
    pub fn feet(&self, trace: &Trace, scales: &BTreeMap<String, f64>) -> Vec<Point> {
        match scales.get(&trace.page_id) {
            None => Vec::new(),
            Some(s) => trace.points.iter().map(|p| Point::new(p.x * s, p.y * s)).collect(),
        }
    }
}

// ── the solved model, going out ────────────────────────────────────────────

/// Why a tapered field stopped rising. The two fail differently: boards mean
/// buy another layer, flashing means the wall will not take one.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CapReason {
    Flashing,
    Boards,
}

/// A piece of ground the layout holds, with the depths that go with it.
///
/// One plane governs a facet, or the cap does and the roof is flat on it. Either
/// way the surface over it is exactly planar, which is what lets the drawing
/// triangulate it and lift each corner without inventing a fold in between.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Facet {
    pub shape: Shape,
    /// Which of the region's planes governs here.
    pub plane: usize,
    /// True where the cap binds: the roof is flat, and the water stops moving.
    pub flat: bool,
}

/// A line the boards are cut along, and how deep the roof is at each end.
///
/// Two ends and two depths say all of it, exactly: a crease lies where one plane
/// governs on each side, so the depth along it is a straight line between the
/// ends. C0 shipped twenty samples of a valley because the surface underneath was
/// sampled too; a laid-out field needs none.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Crease {
    pub a: Point,
    pub b: Point,
    pub a_inches: f64,
    pub b_inches: f64,
}

/// The boundary a region shares with a neighbour, and whether the two agree on it.
///
/// NRCA p.166 wants a common, uniform thickness at a ridge and offers an area
/// divider where the two patterns cannot give one. `disagreement_inches` is the
/// worst gap between what this region's planes make the roof here and what the
/// neighbour's make it — zero wherever the line joining the two drains is square
/// to the field's axes, which is the layout an estimator would draw, and a real
/// number where it is slanted. It is reported, never averaged away.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RidgeSegment {
    pub a: Point,
    pub b: Point,
    pub a_inches: f64,
    pub b_inches: f64,
    /// Which region is on the other side, by its place in the field's own list.
    pub meets: usize,
    pub disagreement_inches: f64,
}

/// One board position: a 4 ft panel, where it lies, how thick and which letter.
///
/// A cut board is still one board, because one is what gets bought — `shapes` is
/// what is left of it after the region's edges and the roof's outline have had
/// their say, and `waste_square_feet` is what the cut took off. `centre` is the
/// middle of the WHOLE cell whether or not the cut left that middle on the roof,
/// because that is where the thickness is read and where the board is set out from.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub centre: Point,
    /// What is left of the board after the cut. In pieces, because a concave
    /// outline can cut one cell twice.
    pub shapes: Vec<Shape>,
    /// Square feet of it that land on the roof.
    pub square_feet: f64,
    /// Square feet of it that got bought: a whole board, cut or not.
    pub board_square_feet: f64,
    pub cut: bool,
    pub waste_square_feet: f64,
    /// The roof's depth at the board's middle, in inches, off the solved plane.
    pub thickness_inches: f64,
    /// True where the cap binds here and the board sits on flat ground.
    pub flat: bool,
    /// The letter printed on the tapered panel, or nothing where the table has none.
    pub letter: Option<String>,
    /// How thick the tapered panel itself is here, in inches.
    pub panel_inches: f64,
    pub fill_inches: f64,
    pub fill_boards: usize,
    /// Layers of board at this position: the fill boards and the tapered one.
    pub courses: usize,
}

/// What a run of boards comes to, in the words a supply house quotes.
///
/// Nought against a letter is not a collapsed unknown: it means the layout uses
/// none of that panel, which is a fact about the layout. A name nobody solved is
/// absent, and reads as pending on a line. The two are different and they stay
/// different.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardBill {
    pub manufacturer: String,
    pub product: String,
    pub document_id: String,
    pub profile_set: ProfileSet,
    pub slope_inches_per_foot: f64,
    /// The flat board the lettered panels stack on, in inches.
    pub fill_step_inches: f64,
    pub panel_feet: f64,
    /// How far each course under the top one is offset, in feet.
    pub stagger_feet: f64,
    pub by_letter: std::collections::BTreeMap<String, LetterRow>,
    pub boards: usize,
    pub cut_boards: usize,
    /// Square feet of tapered panel bought: a whole board apiece.
    pub square_feet: f64,
    /// Of that, what the cuts take off. Not the item's waste allowance — that is
    /// for damage and trim, and adding the two would charge the cut twice.
    pub waste_square_feet: f64,
    /// What lands on the roof.
    pub laid_square_feet: f64,
    pub fill_boards: usize,
    pub fill_square_feet: f64,
    /// The most courses at any one position, the tapered panel included. This is
    /// what the layout actually needs, and it is not what `BOARDS` was typed as
    /// unless the board ceiling is what bound.
    pub courses_max: usize,
    /// Positions the table has no letter for, counted rather than hidden.
    pub unlettered: usize,
}

/// The square the field is cut down in to meet a drain bowl.
///
/// `rim_inches` is read off the solved planes at the sump's own edge, which on a
/// four-way layout is the same number all the way round it — the thickness is the
/// greater of two square runs, so a square standing square to the boards is a
/// contour. The drawing reads this instead of working it out again.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sump {
    pub condition_id: String,
    pub trace: usize,
    pub at: Point,
    pub side_feet: f64,
    /// The side the bowl-plus-24-inches rule wants, where a bowl was stated.
    pub from_bowl_feet: Option<f64>,
    /// The cut, in pieces where the roof's edge takes some of it.
    pub shapes: Vec<Shape>,
    pub square_feet: f64,
    /// True where the roof's own edge cut the square short.
    pub clipped: bool,
    /// The roof's depth at the sump's rim, in inches: where the cut is.
    pub rim_inches: f64,
    /// How far the sump falls from that rim to the drain, in inches.
    pub depth_inches: f64,
    pub panels: usize,
    pub panel_feet: f64,
}

/// One drainage region: the ground one drain takes, and the fall over it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaperRegion {
    /// The drainage point this ground falls to.
    pub drain: Point,
    /// The ground itself. In pieces, because a concave field can hand one drain
    /// two separate pieces of roof, and dropping one would drop roof.
    pub shapes: Vec<Shape>,
    pub square_feet: f64,
    /// The field's axes, carried here too so a region is complete on its own.
    pub axis_along: Point,
    pub axis_across: Point,
    /// The four planes of the four-way layout, all rising from the drain.
    pub planes: Vec<Plane>,
    pub cap_inches: Option<f64>,
    pub facets: Vec<Facet>,
    /// The 45 degree ties, cut to this region.
    pub valleys: Vec<Crease>,
    pub ridges: Vec<RidgeSegment>,
    pub no_fall_square_feet: f64,
    pub thin_inches: f64,
    pub thick_inches: f64,
    /// The board grid over this region's own ground, one entry per position.
    pub boards: Vec<Board>,
    /// What those boards come to. `None` where the table prints no chart for this
    /// slope, and `TaperField::bill_refused` says so in words.
    pub bill: Option<BoardBill>,
}

/// Which way the water leaves a place on the roof, for one arrow.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowArrow {
    pub x: f64,
    pub y: f64,
    pub thickness_inches: f64,
    /// The angle about the vertical that turns plan +x onto the fall direction.
    pub angle: f64,
    /// How far the fall runs across the ground this arrow stands on, in feet.
    ///
    /// One arrow to a facet needs a length, because a mark that says "the water
    /// goes this way over all of this" has to reach across the ground it is
    /// speaking for. A field of identical darts did not: every one of them was
    /// the same size wherever it stood.
    pub length_feet: f64,
}

/// One tapered field: one trace on one condition, laid out.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaperField {
    pub condition_id: String,
    /// Which trace on that condition. A condition can carry several.
    pub trace: usize,
    pub ring: Vec<Point>,
    /// Top of structural deck, in feet. The datum the build-up is dimensioned
    /// from.
    pub elevation: f64,
    /// Thickness at the drain, in inches — the low point the code sets.
    pub start_inches: f64,
    pub slope_inches_per_foot: f64,
    /// Where the field stops rising, in inches, and which ceiling stopped it.
    /// `None` is not a zero: it is a field with nothing capping it, which is
    /// the honest answer when the job states neither a board count nor a
    /// height to flash against.
    pub cap_inches: Option<f64>,
    pub cap_reason: Option<CapReason>,
    /// The axes the whole layout is laid out on: along the field's longest side,
    /// and square to it.
    pub axis_along: Point,
    pub axis_across: Point,
    /// The side itself, so a drawing can say which wall the boards were laid to.
    pub axis_edge: [Point; 2],
    /// One region per drainage point on this field.
    pub regions: Vec<TaperRegion>,
    pub flow: Vec<FlowArrow>,
    pub square_feet: f64,
    pub no_fall_square_feet: f64,
    pub thin_inches: Option<f64>,
    pub thick_inches: Option<f64>,
    /// Four ways off each drain, or two. Typed as `LAYOUT`.
    pub layout: String,
    /// A typed `LAYOUT` that named neither, so the default was used.
    pub layout_refused: Option<f64>,
    /// A typed `PROFILE_SET` that named neither set, so the default was used.
    pub profile_set_refused: Option<f64>,
    /// The whole field's bill: its regions' added up.
    pub bill: Option<BoardBill>,
    /// Why there is no bill, where there is none.
    pub bill_refused: Option<String>,
    /// The sumps cut into it, which the boards are laid around.
    pub sumps: Vec<Sump>,
}

/// A trace the layout would not lay out, and why.
///
/// Not silence and not an empty field: a crossing ring, or a tapered area with no
/// drain on it, is a fact about the job that somebody has to fix, and the solve
/// is the one place that knows it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Refused {
    pub condition_id: String,
    pub trace: usize,
    pub why: String,
}

/// A cricket, as triangles and as the two facts the legend has to say about it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cricket {
    pub condition_id: String,
    pub trace: usize,
    /// Triangles, as x, y, z in feet — y is up. World coordinates, so the
    /// drawing places them and computes nothing.
    pub positions: Vec<f64>,
    pub length_to_width: f64,
    pub max_length_to_width: f64,
    /// How far the traced ridge is off the one the drains (or the curb's face)
    /// imply, in degrees.
    pub off_by: Option<f64>,
    /// True when the ridge slope is not twice the field it sits in.
    pub slope_disagrees: bool,
    /// Its footprint on the plan, in square feet. Cricket stock is its own
    /// lettered series and the bulletin in `data/profiles` carries only the
    /// primary-slope chart, so a cricket is billed by area and not by letter.
    pub plan_square_feet: f64,
    pub slope_inches_per_foot: f64,
    /// True where the solve put it there rather than an estimator tracing it: the
    /// saddle a two-way layout needs between two drains.
    pub derived: bool,
    /// True where every vertex was lifted by the roof's own depth under it. False
    /// is a cricket on no tapered field, sitting on the estimator's typed datum
    /// because there is no solved surface to sit on.
    pub from_solved_surface: bool,
}

/// Everything the tapered package comes to.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvedTaper {
    /// Which solve this is. The window pulls geometry when this moves and not
    /// on every keystroke.
    pub solve_id: u64,
    /// Every drainage point on the roof, in feet.
    pub drains: Vec<Point>,
    pub fields: Vec<TaperField>,
    /// Every sump on the roof, in the order the drains are listed.
    pub sumps: Vec<Sump>,
    pub crickets: Vec<Cricket>,
    /// What the layout would not lay out, and why.
    pub refused: Vec<Refused>,
}

/// A part of the model that is not solved yet, said out loud.
///
/// Walls, fastening and the drawing land in C3, C4 and C5. Until they do, the
/// command answers with this rather than with an empty geometry that would read
/// as a roof with no walls on it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotSolvedYet {
    pub part: String,
    pub solve_id: u64,
    pub not_solved_yet: bool,
}

impl NotSolvedYet {
    pub fn new(part: &str, solve_id: u64) -> Self {
        Self { part: part.to_string(), solve_id, not_solved_yet: true }
    }
}

/// The whole solve: the geometry, and the named numbers each condition carries.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Solved {
    pub solve_id: u64,
    pub taper: SolvedTaper,
    pub scopes: crate::scope::Scopes,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_property_that_is_not_a_number_is_absent_rather_than_fatal() {
        let c: Condition = serde_json::from_value(json!({
            "id": "c-1", "kind": "area", "properties": { "T": 0.5, "NOTE": "half an inch" }
        }))
        .expect("a condition reads");
        assert_eq!(c.property("T"), Some(0.5));
        assert_eq!(c.property("NOTE"), None);
        assert_eq!(c.property("TAPER"), None);
    }

    #[test]
    fn a_trace_on_an_unscaled_sheet_has_no_feet() {
        let doc = Document::read(&json!({
            "pages": [{ "id": "page-1" }],
            "conditions": [],
        }))
        .expect("the document reads");
        let trace = Trace { page_id: "page-1".into(), points: vec![Point::new(9.0, 9.0)] };
        assert!(doc.feet(&trace, &doc.scales()).is_empty());
    }

    #[test]
    fn a_trace_on_a_scaled_sheet_is_in_feet() {
        let doc = Document::read(&json!({
            "pages": [{ "id": "page-1", "feetPerUnit": 2.0 }],
            "conditions": [],
        }))
        .expect("the document reads");
        let trace = Trace { page_id: "page-1".into(), points: vec![Point::new(9.0, 4.0)] };
        assert_eq!(doc.feet(&trace, &doc.scales()), vec![Point::new(18.0, 8.0)]);
    }

    /// The solved names on the wire are the names the front end reads.
    #[test]
    fn the_solved_model_goes_out_in_the_words_the_window_reads() {
        let out = serde_json::to_string(&NotSolvedYet::new("walls", 3)).expect("it serializes");
        assert_eq!(out, r#"{"part":"walls","solveId":3,"notSolvedYet":true}"#);
    }
}
