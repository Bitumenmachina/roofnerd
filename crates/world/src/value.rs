//! Every number on a job is one of three things, and an estimator needs to see which:
//!
//! * **Known**: read off a plan, measured in the field, or typed from a quote.
//! * **Assumed**: a working number someone chose because the real one is not in hand yet.
//! * **Unknown**: nobody has it. It carries no number at all.
//!
//! Each one also says where it came from. A value with no source is refused on load.
//! A value worked out from others (an area from an outline and a pitch) takes the worst
//! status among what it was worked out from, so an assumed pitch makes an assumed area,
//! and an unknown height makes an unknown wall area rather than a guessed one.

use serde::{Deserialize, Serialize};

/// Ordered from best to worst so `max` gives the status a derived value inherits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status {
    Known,
    Assumed,
    Unknown,
}

impl Status {
    /// The mark shown beside a value everywhere it is displayed.
    pub fn flag(self) -> &'static str {
        match self {
            Status::Known => " ",
            Status::Assumed => "~",
            Status::Unknown => "?",
        }
    }
}

/// An entered value: what it is, its unit, how sure we are, and where it came from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Val<T> {
    /// `null` exactly when the status is Unknown.
    pub value: Option<T>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    pub status: Status,
    pub source: String,
}

impl<T> Val<T> {
    pub fn known(value: T, unit: &str, source: &str) -> Self {
        Val { value: Some(value), unit: unit.into(), status: Status::Known, source: source.into() }
    }
    pub fn assumed(value: T, unit: &str, source: &str) -> Self {
        Val { value: Some(value), unit: unit.into(), status: Status::Assumed, source: source.into() }
    }
    pub fn unknown(unit: &str, source: &str) -> Self {
        Val { value: None, unit: unit.into(), status: Status::Unknown, source: source.into() }
    }

    /// The rules a stored value has to meet. `what` names it in the error.
    pub fn check(&self, what: &str) -> Result<(), String> {
        if self.source.trim().is_empty() {
            return Err(format!("{what}: no source given; every value says where it came from"));
        }
        match (self.status, self.value.is_some()) {
            (Status::Unknown, true) => Err(format!("{what}: marked Unknown but carries a value")),
            (Status::Known | Status::Assumed, false) => {
                Err(format!("{what}: marked {:?} but carries no value", self.status))
            }
            _ => Ok(()),
        }
    }
}

/// A value worked out from other values. It shows its formula and names its inputs,
/// so anyone reading the file can see how the number was reached and what feeds it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Derived {
    pub value: Option<f64>,
    pub unit: String,
    pub status: Status,
    pub formula: String,
    /// Each input as `entity-path#component`.
    pub inputs: Vec<String>,
    /// Why it is flagged, when it is.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

impl Derived {
    /// A derived number. The status is the worst among its inputs; an Unknown input
    /// leaves no number behind.
    pub fn new(value: Option<f64>, unit: &str, statuses: &[Status], formula: &str, inputs: &[String]) -> Self {
        let status = statuses.iter().copied().max().unwrap_or(Status::Unknown);
        let value = if status == Status::Unknown { None } else { value.map(round4) };
        let status = if value.is_none() { Status::Unknown } else { status };
        Derived { value, unit: unit.into(), status, formula: formula.into(), inputs: inputs.to_vec(), note: String::new() }
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.note = note.into();
        self
    }

    /// A derived value that cannot be given, with the reason.
    pub fn refused(unit: &str, formula: &str, inputs: &[String], why: &str) -> Self {
        Derived { value: None, unit: unit.into(), status: Status::Unknown, formula: formula.into(), inputs: inputs.to_vec(), note: why.into() }
    }
}

/// Derived numbers are written to four decimals: finer than a sixteenth of an inch on a
/// length in feet, and it keeps the saved file readable and its diffs free of float noise.
pub fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}
