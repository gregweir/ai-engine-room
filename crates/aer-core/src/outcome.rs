//! Acquisition outcome and the controlled error categories. A missing
//! runtime or unavailable metric is a normal `NoValue`; an actual connection,
//! timeout, parsing, measurement, or permission failure is `Failed` and stays
//! distinguishable from absence. Semantics per design §4.4, §4.6.

use crate::sample::MetricSample;

/// The result of an acquisition attempt.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// A value exists; carries provenance.
    Ok(MetricSample),
    /// No value; explained by support and availability (normal).
    NoValue,
    /// An actual acquisition attempt failed.
    Failed(AcquisitionError),
}

/// Controlled acquisition-failure categories. `message()` returns a fixed,
/// developer-authored string per category — never raw system/provider text.
/// `category()` returns a stable lowercase code for the report/IPC boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcquisitionError {
    Connection,
    Timeout,
    Parsing,
    Measurement,
    Permission,
    Other,
}

impl AcquisitionError {
    pub fn message(&self) -> &'static str {
        match self {
            AcquisitionError::Connection => "Could not reach the source to acquire this metric.",
            AcquisitionError::Timeout => "Acquiring this metric timed out.",
            AcquisitionError::Parsing => "The source returned a response that could not be read.",
            AcquisitionError::Measurement => "The measurement could not be completed.",
            AcquisitionError::Permission => "Permission was denied while acquiring this metric.",
            AcquisitionError::Other => "Acquiring this metric failed.",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            AcquisitionError::Connection => "connection",
            AcquisitionError::Timeout => "timeout",
            AcquisitionError::Parsing => "parsing",
            AcquisitionError::Measurement => "measurement",
            AcquisitionError::Permission => "permission",
            AcquisitionError::Other => "other",
        }
    }
}
