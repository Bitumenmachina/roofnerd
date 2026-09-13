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
