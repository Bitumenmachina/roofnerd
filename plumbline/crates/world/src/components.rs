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

use crate::units::{self, Len};
use crate::value::{Derived, Status, Val};

/// A point on the plan: `[x, y]`, in the unit its value carries (feet or inches).
pub type P2 = [f64; 2];

/// The unit derived lengths are worked out in (LF, SF and SQ follow from it). Entered
/// values each carry their own unit and are converted to this only for arithmetic.
pub const U_LENGTH: &str = units::FT;
/// Pitch: inches of rise for every 12 inches of run.
pub const U_PITCH: &str = units::PITCH;

/// Plan points in feet, from points in `unit`. None if the unit is not one this program
/// converts.
fn points_in_feet<const N: usize>(pts: &[P2; N], unit: &str) -> Option<[P2; N]> {
    let d = units::coord_divisor(unit).ok()?;
    Some(pts.map(|p| [p[0] / d, p[1] / d]))
}

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
    /// Where two slopes of different pitch meet and both fall the same way: one falls
    /// to the line, the other on from it. A change of pitch, not a ridge, hip or valley.
    PitchBreak,
    Parapet,
    /// A condition as a recipe: the cost lines an element gets when the condition is
    /// applied to it. It is an element of the job so it can be read and edited there.
    Condition,
    // The rest of the building (see `building.rs`).
    /// One floor of the building and everything that stands on it.
    Storey,
    Wall,
    Door,
    Window,
    /// The floor itself: IFC's slab.
    Floor,
    /// A set of ducts, pipes, cables, equipment and fixtures that work as one.
    System,
    Duct,
    Pipe,
    Conduit,
    Cable,
    Equipment,
    /// What a system ends at in a room: a register, a lavatory, a receptacle, a light.
    Fixture,
    /// A type: what walls, floors, doors, windows or runs of one sort have in common (a
    /// thickness, a size), entered once. An element points at its type by the type's path.
    Type,
    /// What joins runs: an elbow, a tee, a cross, a change of size, a coupling.
    Fitting,
    /// A room: the space the walls round a point enclose.
    Room,
    /// A material: what something is made of, how it is bought and what it costs. An
    /// element says what it is made of by the material's name.
    Material,
    /// A sheet of the plans laid under the tracing of the storey it is under.
    Drawing,
    /// An estimate made before, brought in to be set against this one. Under the job.
    Prior,
}

impl Kind {
    /// The start of the path given to a new element of this kind.
    /// The kind as a builder says it: what a list or a refusal calls a thing of this kind.
    pub fn said(self) -> &'static str {
        match self {
            Kind::RoofPlane => "roof plane",
            Kind::LowSlopeArea => "low-slope area",
            Kind::PitchBreak => "pitch break",
            Kind::Condition => "assembly",
            other => other.stem(),
        }
    }
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
            Kind::PitchBreak => "break",
            Kind::Parapet => "parapet",
            Kind::Condition => "condition",
            Kind::Storey => "level",
            Kind::Wall => "wall",
            Kind::Door => "door",
            Kind::Window => "window",
            Kind::Floor => "floor",
            Kind::System => "system",
            Kind::Duct => "duct",
            Kind::Pipe => "pipe",
            Kind::Conduit => "conduit",
            Kind::Cable => "cable",
            Kind::Equipment => "equipment",
            Kind::Fixture => "fixture",
            Kind::Type => "type",
            Kind::Fitting => "fitting",
            Kind::Room => "room",
            Kind::Material => "material",
            Kind::Drawing => "drawing",
            Kind::Prior => "prior",
        }
    }
    pub fn is_area(self) -> bool {
        matches!(self, Kind::RoofPlane | Kind::LowSlopeArea)
    }
    pub fn is_run(self) -> bool {
        matches!(self, Kind::Eave | Kind::Rake | Kind::Ridge | Kind::Hip | Kind::Valley | Kind::Headwall | Kind::PitchBreak | Kind::Parapet)
    }
    /// How many planes a line of this kind lies on: (fewest, most). A hip or a valley is
    /// where two planes meet. An eave, a rake or a headwall is the edge of one. A ridge is
    /// two planes meeting, or the top edge of one shed plane. A parapet is on none.
    pub fn planes_allowed(self) -> (usize, usize) {
        match self {
            Kind::Hip | Kind::Valley | Kind::PitchBreak => (2, 2),
            Kind::Eave | Kind::Rake | Kind::Headwall => (1, 1),
            Kind::Ridge => (1, 2),
            _ => (0, 0),
        }
    }
    /// The one figure a thing of this kind is bought by, and its unit: what an allowance
    /// rate multiplies and what a detailed price is divided by to give a rate. None for a
    /// kind that has no such figure.
    pub fn priced_by(self) -> Option<(&'static str, &'static str)> {
        match self {
            Kind::RoofPlane | Kind::LowSlopeArea => Some(("surface_area", "SF")),
            Kind::Parapet => Some(("plan_length", "LF")),
            k if k.is_run() => Some(("true_length", "LF")),
            Kind::Wall => Some(("net_area", "SF")),
            Kind::Floor => Some(("plan_area", "SF")),
            Kind::Room => Some(("floor_area", "SF")),
            Kind::Duct | Kind::Pipe | Kind::Conduit | Kind::Cable => Some(("true_length", "LF")),
            _ => None,
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
        } else if self == Kind::Wall {
            &[("plan_length", "LF"), ("height", "ft"), ("face_area", "SF"), ("opening_area", "SF"), ("net_area", "SF")]
        } else if matches!(self, Kind::Door | Kind::Window) {
            &[("width", "ft"), ("height", "ft"), ("opening_area", "SF")]
        } else if self == Kind::Floor {
            &[("plan_area", "SF")]
        } else if self == Kind::Room {
            &[("floor_area", "SF"), ("gross_area", "SF"), ("perimeter", "LF")]
        } else if self == Kind::Duct {
            &[("true_length", "LF"), ("surface_area", "SF")]
        } else if matches!(self, Kind::Pipe | Kind::Conduit | Kind::Cable) {
            &[("true_length", "LF")]
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
    /// The unit derived lengths are worked out in. Only "ft" is accepted. Entered values
    /// carry their own units.
    pub length_unit: String,
    /// The number the next new element's path will take.
    pub next_id: u32,
}

/// `pl::outline`: the footprint of an area as traced on the plan, in feet or in inches,
/// traced either way round. `eave` is the low edge the pitch rises from, given as its own line so that
/// re-tracing or re-ordering the outline cannot quietly change which edge is the eave.
/// It has to lie along one edge of the outline. `eave_height` is that edge's height.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outline {
    pub points: Val<Vec<P2>>,
    pub eave: Val<[P2; 2]>,
    pub eave_height: Val<Len>,
}

impl Outline {
    /// The outline's points in feet, whatever unit they were entered in.
    pub fn points_ft(&self) -> Option<Vec<P2>> {
        let d = units::coord_divisor(&self.points.unit).ok()?;
        self.points.value.as_ref().map(|pts| pts.iter().map(|p| [p[0] / d, p[1] / d]).collect())
    }
    /// The eave line in feet.
    pub fn eave_ft(&self) -> Option<[P2; 2]> {
        self.eave.value.as_ref().and_then(|line| points_in_feet(line, &self.eave.unit))
    }
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

impl Run {
    /// The line's two ends in feet, whatever unit they were entered in.
    pub fn ends_ft(&self) -> Option<[P2; 2]> {
        self.ends.value.as_ref().and_then(|ends| points_in_feet(ends, &self.ends.unit))
    }
}

/// `pl::height`: how tall a wall-like element stands (a parapet above the deck), in the
/// unit it was entered in.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Height(pub Val<Len>);

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

/// One priced line hanging off an element, worked in steps that each stay visible:
/// formula, then quantity, then waste, then order units.
///
/// `quantity` is the line's formula, readable and editable on the line: a measure of this
/// element taken as it stands (`surface_area`), or arithmetic over the element's measures,
/// a parapet's `height`, and the values entered on the line
/// (`plan_length * stretch_out`, `plan_length * sides / spacing`). So the line follows the
/// geometry.
///
/// `stretch_out` is the width of the flat sheet a profile is bent from, as the detail
/// gives it (usually in inches). It belongs to the line, not to the material: the same
/// copper is 12 in at an eave and 36 in in a valley. `inputs` are any other values the
/// formula names: a spacing, a number of sides.
///
/// `waste` is a percentage added to the quantity. `order` turns the result into what is
/// bought: sheets of so many SF, pounds at so many per SF.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostLine {
    pub description: String,
    pub quantity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch_out: Option<Val<Len>>,
    /// Each a length (`ft`, `in`, `ft-in`) or a plain number (no unit).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: BTreeMap<String, Val<Len>>,
    /// Percent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waste: Option<Val<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<Order>,
    /// Dollars for each of what the line is priced in. A line made of the element's
    /// material (`of_material`) carries none: its price is the material's.
    #[serde(default = "CostLine::no_price_of_its_own", skip_serializing_if = "CostLine::is_no_price_of_its_own")]
    pub unit_cost: Val<f64>,
    /// Which row of the recap the line's amount lands in. A line with none is named by
    /// the recap and left out of it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<Class>,
    /// Which of the sixteen divisions the line's work is: where its amount lands when
    /// the estimate is read division by division. A line with none is named by the
    /// recap and is in no division's total.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<Division>,
    /// Switched off: the line stays on the sheet with what it would come to, and is out
    /// of every total. Only a Labor line can be.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub off: bool,
    /// The line is made of the element's material (`pl::material`): how it is ordered and
    /// what it costs are the material's, so changing the element's material reprices it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub of_material: bool,
}

impl CostLine {
    /// A line priced by one measure and nothing else.
    pub fn of(description: &str, quantity: &str, unit_cost: Val<f64>) -> CostLine {
        CostLine { description: description.into(), quantity: quantity.into(), stretch_out: None, inputs: BTreeMap::new(), waste: None, order: None, unit_cost, class: None, division: None, off: false, of_material: false }
    }
    /// A line made of the element's material: ordered and priced as the material is.
    pub fn of_material(description: &str, quantity: &str) -> CostLine {
        CostLine { of_material: true, class: Some(Class::Material), ..CostLine::of(description, quantity, CostLine::no_price_of_its_own()) }
    }
    /// What stands where a unit cost would on a line that has no price of its own. It is
    /// not a value anyone can enter: an entered value says where it came from.
    pub fn no_price_of_its_own() -> Val<f64> {
        Val { value: None, unit: String::new(), status: Status::Unknown, source: String::new() }
    }
    pub fn is_no_price_of_its_own(v: &Val<f64>) -> bool {
        *v == CostLine::no_price_of_its_own()
    }
    /// The line is in-house work: it is priced in hours.
    pub fn in_hours(&self) -> bool {
        self.order.as_ref().is_some_and(|o| o.unit == HOURS)
    }
}

/// The sixteen divisions a building's work is estimated under: those of CSI MasterFormat
/// as published through its 1995 edition (DECISIONS.md call 57 has the list's source).
/// In the job file a division is its number, 1 to 16.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Division {
    GeneralRequirements = 1,
    SiteConstruction,
    Concrete,
    Masonry,
    Metals,
    WoodAndPlastics,
    ThermalAndMoistureProtection,
    DoorsAndWindows,
    Finishes,
    Specialties,
    Equipment,
    Furnishings,
    SpecialConstruction,
    ConveyingSystems,
    Mechanical,
    Electrical,
}

impl Division {
    pub const ALL: [Division; 16] = [
        Division::GeneralRequirements,
        Division::SiteConstruction,
        Division::Concrete,
        Division::Masonry,
        Division::Metals,
        Division::WoodAndPlastics,
        Division::ThermalAndMoistureProtection,
        Division::DoorsAndWindows,
        Division::Finishes,
        Division::Specialties,
        Division::Equipment,
        Division::Furnishings,
        Division::SpecialConstruction,
        Division::ConveyingSystems,
        Division::Mechanical,
        Division::Electrical,
    ];

    pub fn number(self) -> u8 {
        self as u8
    }

    pub fn title(self) -> &'static str {
        match self {
            Division::GeneralRequirements => "General Requirements",
            Division::SiteConstruction => "Site Construction",
            Division::Concrete => "Concrete",
            Division::Masonry => "Masonry",
            Division::Metals => "Metals",
            Division::WoodAndPlastics => "Wood and Plastics",
            Division::ThermalAndMoistureProtection => "Thermal and Moisture Protection",
            Division::DoorsAndWindows => "Doors and Windows",
            Division::Finishes => "Finishes",
            Division::Specialties => "Specialties",
            Division::Equipment => "Equipment",
            Division::Furnishings => "Furnishings",
            Division::SpecialConstruction => "Special Construction",
            Division::ConveyingSystems => "Conveying Systems",
            Division::Mechanical => "Mechanical",
            Division::Electrical => "Electrical",
        }
    }

    /// "Division 7, Thermal and Moisture Protection".
    pub fn said(self) -> String {
        format!("Division {}, {}", self.number(), self.title())
    }

    pub fn of_number(n: u8) -> Option<Division> {
        Division::ALL.get(usize::from(n).wrapping_sub(1)).copied()
    }

    /// A division as a person writes it: its number (`7`, `07`, `Division 7`, `Div. 7`)
    /// or its title, capitals aside.
    pub fn parse(text: &str) -> Result<Division, String> {
        let t = text.trim();
        let lower = t.to_ascii_lowercase();
        let number = lower.strip_prefix("division").or_else(|| lower.strip_prefix("div.")).or_else(|| lower.strip_prefix("div")).unwrap_or(&lower).trim();
        if let Ok(n) = number.parse::<u8>() {
            return Division::of_number(n).ok_or_else(|| format!("{t:?} is not a division: there are sixteen, 1 to 16"));
        }
        Division::ALL
            .iter()
            .copied()
            .find(|d| d.title().eq_ignore_ascii_case(t))
            .ok_or_else(|| format!("{t:?} is not a division: give its number, 1 to 16, or its title ({})", Division::ALL.iter().map(|d| format!("{} {}", d.number(), d.title())).collect::<Vec<_>>().join(", ")))
    }
}

impl Serialize for Division {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(self.number())
    }
}

impl<'de> Deserialize<'de> for Division {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Division, D::Error> {
        let n = u8::deserialize(d)?;
        Division::of_number(n).ok_or_else(|| serde::de::Error::custom(format!("{n} is not a division: there are sixteen, 1 to 16")))
    }
}

/// The order unit of in-house work. A line in hours carries a production rate as its
/// order factor (`SF/hr`: so many to the man-hour; or `hr/SF`) and dollars an hour as its
/// unit cost.
pub const HOURS: &str = "hr";

/// The rows of the recap. A line is one of them and its amount lands in that row and no
/// other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Class {
    Material,
    /// In-house work on site, by a production rate.
    Labor,
    /// In-house work in the shop, by a production rate.
    Fabrication,
    /// In-house supervision, in hours.
    Supervision,
    /// Somebody else's work: a quantity and a price.
    Subcontract,
    /// General conditions: what the job costs that is not built into it.
    General,
}

impl Class {
    pub const ALL: [Class; 6] = [Class::Material, Class::Labor, Class::Fabrication, Class::Supervision, Class::Subcontract, Class::General];
    pub fn title(self) -> &'static str {
        match self {
            Class::Material => "Material",
            Class::Labor => "Labor",
            Class::Fabrication => "Fabrication",
            Class::Supervision => "Supervision",
            Class::Subcontract => "Subcontract",
            Class::General => "General conditions",
        }
    }
    /// In-house work, counted in hours.
    pub fn in_hours(self) -> bool {
        matches!(self, Class::Labor | Class::Fabrication | Class::Supervision)
    }
    /// The column of the estimate sheet the class's amounts stand in.
    pub fn column(self) -> Column {
        match self {
            Class::Material => Column::Material,
            Class::Labor | Class::Fabrication | Class::Supervision => Column::Labor,
            Class::Subcontract | Class::General => Column::General,
        }
    }
    /// A class as it is typed: its name, in any capitals; `sub` and `general` are enough.
    pub fn parse(text: &str) -> Result<Class, String> {
        match text.trim().to_lowercase().as_str() {
            "material" => Ok(Class::Material),
            "labor" => Ok(Class::Labor),
            "fabrication" => Ok(Class::Fabrication),
            "supervision" => Ok(Class::Supervision),
            "subcontract" | "sub" => Ok(Class::Subcontract),
            "general" | "general conditions" => Ok(Class::General),
            _ => Err(format!("{text:?} is not a cost type; a line is material, labor, fabrication, supervision, subcontract or general")),
        }
    }
}

/// The three money columns of the estimate sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Column {
    Material,
    Labor,
    General,
}

/// How a line's quantity becomes what is bought. `factor` carries the conversion in its
/// unit: `SF/sheet` (the quantity is divided: 30 SF to a sheet) or `lb/SF` (multiplied:
/// 1.25 lb to the SF). `round_up` buys whole ones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Order {
    pub unit: String,
    pub factor: Val<f64>,
    #[serde(default)]
    pub round_up: bool,
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

/// What one cost line works out to, step by step. Each step is a figure with its own
/// formula, unit and status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineFigures {
    /// The line's formula worked out.
    pub quantity: Derived,
    /// With the line's waste added, when it has waste.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub with_waste: Option<Derived>,
    /// In order units, when the line has them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<Derived>,
    /// What the line comes to in dollars: what it is priced in, times its unit cost. A
    /// line with no price has no extension, and its note says so. (Absent only in a file
    /// saved before lines were extended; it is worked out again on load.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension: Option<Derived>,
    /// For a line made of the element's material: what it comes to at each supplier's
    /// price for that material, by the supplier's name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub quotes: BTreeMap<String, Derived>,
}

/// `pl::line_quantities`: what each cost line on the element works out to, by the line's
/// description. Nobody types these; like measures they are recomputed on every change and
/// every load.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LineQuantities(pub BTreeMap<String, LineFigures>);

/// `pl::recipe`: on a Condition element, the kinds of element the condition can be
/// applied to and the cost code and lines an element gets when it is.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub applies_to: Vec<Kind>,
    pub cost: Cost,
}

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
/// How one edge of a footprint rises: at a pitch, or not at all (a gable end, which stands
/// upright and carries no roof plane of its own). In the file a pitch is its number of
/// inches in 12 and a gable is the word `"gable"`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rise {
    Pitch(f64),
    Gable,
}

impl Serialize for Rise {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Rise::Pitch(x) => x.serialize(s),
            Rise::Gable => "gable".serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for Rise {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match serde_json::Value::deserialize(d)? {
            serde_json::Value::Number(n) => n.as_f64().map(Rise::Pitch).ok_or_else(|| serde::de::Error::custom("a pitch is a number")),
            serde_json::Value::String(s) if s == "gable" => Ok(Rise::Gable),
            other => Err(serde::de::Error::custom(format!("an edge rises at a pitch (a number of inches in 12) or is \"gable\"; {other} is neither"))),
        }
    }
}

/// `pl::footprint`: on a Roof, the outline the roof stands on, the height of its eaves, and
/// how each edge rises. Edge i runs from point i to the next. From these the roof is
/// solved: where the slopes meet gives the planes, and the planes give the hips, valleys
/// and ridges. Nobody enters those; change a pitch or move a corner and they follow.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Footprint {
    pub points: Val<Vec<P2>>,
    pub eave_height: Val<Len>,
    pub edges: Vec<Val<Rise>>,
}

impl Footprint {
    /// The footprint's points in feet, whatever unit they were entered in.
    pub fn points_ft(&self) -> Option<Vec<P2>> {
        let d = units::coord_divisor(&self.points.unit).ok()?;
        self.points.value.as_ref().map(|pts| pts.iter().map(|p| [p[0] / d, p[1] / d]).collect())
    }
}

/// `pl::solved`: on a plane or a line the solver made, the roof it was solved from and,
/// for a plane, which edge of the footprint it rises from (counted from 1). The solver
/// replaces what carries this and leaves alone what does not.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Solved {
    pub from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge: Option<usize>,
}

pub const K_LINE_QUANTITIES: &str = "pl::line_quantities";
pub const K_ALLOWANCE: &str = "pl::allowance";
pub const K_PRICED: &str = "pl::priced";

/// `pl::allowance`: a design-phase price for the element as a whole: so many dollars for
/// each SF or LF of the one figure a thing of its kind is bought by (`Kind::priced_by`).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Allowance {
    pub rate: Val<f64>,
}

/// `pl::priced`: what the element comes to, both ways, by name: `allowance` (its figure
/// times its allowance rate), `detail` (its cost lines' extensions added up),
/// `detail_rate` (the detail over the same figure), `difference` (detail less allowance)
/// and `difference_percent`. Nobody types these; like measures they are worked out again
/// on every change and every load.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Priced(pub BTreeMap<String, Derived>);
/// `pl::solve`: on a Roof with a footprint, what the last solve came to: whether the roof
/// could be solved, and what there is to know about how its planes meet. Worked out, never
/// typed; it is worked out again on every load.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolveState {
    /// Empty when the roof is solved. Otherwise why it could not be.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub not_solved: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

pub const K_MATERIAL: &str = "pl::material";
pub const K_SUPPLY: &str = "pl::supply";
pub const K_MARKUP: &str = "pl::markup";

/// `pl::material`: what the element is made of, by the name of a Material element of the
/// job. Its cost lines marked `of_material` are ordered and priced as that material is.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MadeOf(pub Val<String>);

/// `pl::supply`: on a Material element, how it is bought and what it costs. `order` is
/// how what is measured becomes what is bought (pounds at so many to the SF, sheets of so
/// many SF); without one it is bought as it is measured. `price` is the generic price for
/// each of what is bought (`$/lb`); `quotes` are the same from suppliers, by name.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Supply {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<Order>,
    pub price: Val<f64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub quotes: BTreeMap<String, Val<f64>>,
}

/// `pl::markup`: on the job, overhead and profit, each a percentage. Overhead is taken on
/// the job cost; profit on the job cost and the overhead together.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Markup {
    pub overhead: Val<f64>,
    pub profit: Val<f64>,
}

pub const K_SOLVE: &str = "pl::solve";
pub const K_FOOTPRINT: &str = "pl::footprint";
pub const K_SOLVED: &str = "pl::solved";
pub const K_RECIPE: &str = "pl::recipe";
