//! What an element of the building can carry. An element is an entity; each of these is a
//! component it may or may not have. A roof plane has an outline and a pitch. A hip has a
//! run and names the planes it sits between. A parapet has a run and a height. Any of them
//! can carry a condition, a cost code with lines, and a schedule code.
//!
//! The name in quotes on each component is its key in the saved job file. A field this
//! program does not know inside one of its own components is refused on load, so nothing
//! (a price with no status, say) can ride in unseen and nothing is dropped unseen.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::value::{Derived, Val};

/// A point on the plan, in feet: `[x, y]`.
pub type P2 = [f64; 2];

/// The one length unit plan geometry is held in. The program converts nothing.
pub const U_LENGTH: &str = "ft";
/// Pitch: inches of rise for every 12 inches of run.
pub const U_PITCH: &str = "in/12";

/// Where the element sits in the job: its permanent path, its name under its parent, and
/// the parent's path. The path never changes once given; the name can.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Node {
    pub path: String,
    pub name: String,
    pub parent: Option<String>,
    /// IFC5's `inherits` and any other entry-level keys in the file, kept as read.
    pub inherits: BTreeMap<String, String>,
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Node {
    pub fn new(path: &str, name: &str, parent: Option<&str>) -> Node {
        Node { path: path.into(), name: name.into(), parent: parent.map(str::to_string), inherits: BTreeMap::new(), extra: BTreeMap::new() }
    }
}

/// `pl::class`: what kind of element this is.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Job,
    Roof,
    /// One pitched face of a roof.
    RoofPlane,
    /// A low-slope (flat) roof area.
    LowSlopeArea,
    Eave,
    Rake,
    Ridge,
    Hip,
    Valley,
    /// Where a roof plane dies into a wall above it.
    Headwall,
    Parapet,
}

impl Kind {
    /// The start of the path given to a new element of this kind.
    pub fn stem(self) -> &'static str {
        match self {
            Kind::Job => "job",
            Kind::Roof => "roof",
            Kind::RoofPlane => "plane",
            Kind::LowSlopeArea => "lowslope",
            Kind::Eave => "eave",
            Kind::Rake => "rake",
            Kind::Ridge => "ridge",
            Kind::Hip => "hip",
            Kind::Valley => "valley",
            Kind::Headwall => "headwall",
            Kind::Parapet => "parapet",
        }
    }
    pub fn is_area(self) -> bool {
        matches!(self, Kind::RoofPlane | Kind::LowSlopeArea)
    }
    pub fn is_run(self) -> bool {
        matches!(self, Kind::Eave | Kind::Rake | Kind::Ridge | Kind::Hip | Kind::Valley | Kind::Headwall | Kind::Parapet)
    }
    /// How many planes a line of this kind lies on: (fewest, most). A hip or a valley is
    /// where two planes meet. An eave, a rake or a headwall is the edge of one. A ridge is
    /// two planes meeting, or the top edge of one shed plane. A parapet is on none.
    pub fn planes_allowed(self) -> (usize, usize) {
        match self {
            Kind::Hip | Kind::Valley => (2, 2),
            Kind::Eave | Kind::Rake | Kind::Headwall => (1, 1),
            Kind::Ridge => (1, 2),
            _ => (0, 0),
        }
    }
    /// The quantities worked out for an element of this kind, each with its unit.
    pub fn measures(self) -> &'static [(&'static str, &'static str)] {
        if self.is_area() {
            &[("plan_area", "SF"), ("slope_factor", ""), ("surface_area", "SF"), ("squares", "SQ")]
        } else if self == Kind::Parapet {
            &[("plan_length", "LF"), ("face_area", "SF")]
        } else if self.is_run() {
            &[("plan_length", "LF"), ("true_length", "LF")]
        } else {
            &[]
        }
    }
}

/// `pl::job`: what the root element carries.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobInfo {
    pub title: String,
    /// Plan coordinates and lengths are in this unit. Only "ft" is accepted.
    pub length_unit: String,
    /// The number the next new element's path will take.
    pub next_id: u32,
}

/// `pl::outline`: the footprint of an area as traced on the plan, in feet, traced either
/// way round. `eave` is the low edge the pitch rises from, given as its own line so that
/// re-tracing or re-ordering the outline cannot quietly change which edge is the eave.
/// It has to lie along one edge of the outline. `eave_height` is that edge's height.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outline {
    pub points: Val<Vec<P2>>,
    pub eave: Val<[P2; 2]>,
    pub eave_height: Val<f64>,
}

/// `pl::pitch`: rise in inches for every 12 inches of run.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pitch(pub Val<f64>);

/// `pl::run`: a line on the plan (an eave, a hip, a parapet) by its two ends, and the
/// paths of the planes it lies on.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub ends: Val<[P2; 2]>,
    #[serde(default)]
    pub planes: Vec<String>,
}

/// `pl::height`: how tall a wall-like element stands, in feet (a parapet above the deck).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Height(pub Val<f64>);

/// `pl::condition`: the condition assigned to the element, by name.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Condition(pub Val<String>);

/// A cost code and the scheme it belongs to. The scheme is written out on every code
/// because which scheme is primary has not been ruled on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostCode {
    pub scheme: String,
    pub code: String,
    pub title: String,
}

/// One priced line hanging off an element. `quantity` names the measure on this element
/// the line is priced by, so the line follows the geometry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostLine {
    pub description: String,
    pub quantity: String,
    pub unit_cost: Val<f64>,
}

/// `pl::cost`: the element's cost code and its lines.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cost {
    pub code: Val<CostCode>,
    #[serde(default)]
    pub lines: Vec<CostLine>,
}

/// `pl::schedule`: the work breakdown code the element belongs to.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkCode {
    pub wbs: Val<String>,
}

/// `pl::measure`: every quantity worked out from this element's geometry, by name.
/// Nobody types these. They are recomputed whenever the job changes and on every load.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Measure(pub BTreeMap<String, Derived>);

/// Attributes in the file this program does not know. They are kept and written back
/// untouched, so a file that passes through loses nothing.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct Foreign(pub BTreeMap<String, serde_json::Value>);

pub const K_CLASS: &str = "pl::class";
pub const K_JOB: &str = "pl::job";
pub const K_OUTLINE: &str = "pl::outline";
pub const K_PITCH: &str = "pl::pitch";
pub const K_RUN: &str = "pl::run";
pub const K_HEIGHT: &str = "pl::height";
pub const K_CONDITION: &str = "pl::condition";
pub const K_COST: &str = "pl::cost";
pub const K_SCHEDULE: &str = "pl::schedule";
pub const K_MEASURE: &str = "pl::measure";
