//! The job as one world. See SPEC.md at the repository root for the model in full.
//!
//! * `value`: Known, Assumed, Unknown, and where a value came from.
//! * `units`: feet, inches, feet-and-inches; what is typed, shown, and converted.
//! * `components`: what an element can carry.
//! * `check`: the rules an entered component has to meet.
//! * `formula`: the formula on a cost line, with units and statuses carried through it.
//! * `lines`: a cost line worked out: quantity, waste, order units.
//! * `geom`: plan geometry.
//! * `derive`: quantities worked out from geometry.
//! * `roof`: the lines a set of planes implies (ridge, hip, valley, eave, rake).
//! * `skeleton`: where a footprint's slopes meet (ported from BIMCraft; see the file).
//! * `solve`: a roof's planes and lines made from its footprint and kept in step with it.
//! * `building`: storeys, walls, floors, and the ducts, pipes, cables and equipment in them.
//! * `paper`: a page of line work and the PDF it is written as (ported from BIMCraft; see the files).
//! * `plan_set`: the plan of a level as it stands, the demolition plan and the proposed plan, on paper.
//! * `plans`: a sheet of the plans laid under the tracing, and its scale.
//! * `job`: the world and the edits made to it.
//! * `file`: the saved job.
//! * `sample`: a small made-up job for demonstrations and tests.

pub mod building;
pub mod carries;
pub mod check;
pub mod components;
pub mod derive;
pub mod editing;
pub mod export;
pub mod file;
pub mod finish;
pub mod formula;
pub mod gable;
pub mod geom;
pub mod hidden;
pub mod job;
pub mod lines;
pub mod paper;
pub mod plan_set;
pub mod plans;
pub mod tables;
pub mod taper;
pub mod trim;
pub mod roof;
pub mod sample;
pub mod skeleton;
pub mod solve;
pub mod spec;
pub mod structure;
pub mod units;
pub mod value;
pub mod view;
pub mod vif;

pub use building::{JoistsSpan, Stood, Item, Connector, ConnectorKind, Connectors, Discipline, Domain, Flow, Fitting, FittingKind, Floor, FloorType, Function, InSystem, Justify, Layer, Level, Location, Opening, OpeningType, Placed, Room, RoomLine, Segment, SegmentType, SystemInfo, Wall, WallType, P3};
pub use components::*;
pub use job::{Changed, Header, Job, Left, LeftAsIs, Rewritten};
pub use units::Len;
pub use value::{Derived, Status, Val};
