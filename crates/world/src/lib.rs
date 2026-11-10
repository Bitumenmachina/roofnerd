//! The job as one world. See SPEC.md at the repository root for the model in full.
//!
//! * `value`: Known, Assumed, Unknown, and where a value came from.
//! * `units`: feet, inches, feet-and-inches; what is typed, shown, and converted.
//! * `components`: what an element can carry.
//! * `check`: the rules an entered component has to meet.
//! * `geom`: plan geometry.
//! * `derive`: quantities worked out from geometry.
//! * `job`: the world and the edits made to it.
//! * `file`: the saved job.
//! * `sample`: a small made-up job for demonstrations and tests.

pub mod check;
pub mod components;
pub mod derive;
pub mod file;
pub mod geom;
pub mod job;
pub mod sample;
pub mod units;
pub mod value;
pub mod view;

pub use components::*;
pub use job::{Header, Job};
pub use units::Len;
pub use value::{Derived, Status, Val};
