//! Observed values, timestamps, and the building blocks provenance carries:
//! formulas, models, and controlled limitation codes.

use std::time::{SystemTime, UNIX_EPOCH};

/// A single observed reading. `Float` is kept distinct from `Integer`/`Count`
/// so consumers know whether a value is integral by construction.
#[derive(Clone, Debug, PartialEq)]
pub enum SampleValue {
    Integer(i64),
    Float(f64),
    Count(u64),
}

/// A point in time, stored as milliseconds since the Unix epoch. `now()` reads
/// the system clock; `from_millis()` exists for deterministic tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timestamp {
    millis_since_epoch: i64,
}

impl Timestamp {
    pub fn from_millis(millis_since_epoch: i64) -> Self {
        Self { millis_since_epoch }
    }

    pub fn now() -> Self {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            millis_since_epoch: since_epoch.as_millis() as i64,
        }
    }

    pub fn millis_since_epoch(&self) -> i64 {
        self.millis_since_epoch
    }
}

/// A deterministic calculation applied to identified inputs. Preserves the
/// human description and the expression so a value's derivation is recoverable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formula {
    description: String,
    expression: String,
}

impl Formula {
    pub fn new(description: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            expression: expression.into(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }
}

/// An assumptions/model used by an [`Estimated`](crate::provenance::Provenance::Estimated)
/// provenance. A short, developer-authored description of the model or
/// assumptions applied — never raw system text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    description: String,
}

impl Model {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// A controlled, developer-authored limitation reason. `message()` returns a
/// fixed human string per code — never raw system/provider text — so that
/// limitations remain report-safe and language-stable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LimitationCode {
    /// Not implemented for this platform/runtime in this build.
    NotImplementedOnThisPlatform,
    /// The runtime does not expose this metric.
    NotSupportedByRuntime,
    /// Requires a hardware driver that is not present.
    RequiresDriver,
    /// Requires elevated privileges that were not available.
    RequiresElevatedPrivileges,
    /// Value is an approximation rather than a direct reading.
    Approximation,
    /// Value is derived from a sample, not a full population.
    Sampling,
    /// Value is an estimate supplied by the operating system or kernel rather
    /// than a direct physical measurement.
    KernelEstimate,
}

impl LimitationCode {
    pub fn message(&self) -> &'static str {
        match self {
            LimitationCode::NotImplementedOnThisPlatform => {
                "This metric is not implemented for this platform in this build."
            }
            LimitationCode::NotSupportedByRuntime => "The runtime does not expose this metric.",
            LimitationCode::RequiresDriver => {
                "A required hardware driver is not present or not loaded."
            }
            LimitationCode::RequiresElevatedPrivileges => {
                "Acquiring this metric requires elevated privileges that are not available."
            }
            LimitationCode::Approximation => {
                "This value is an approximation, not a direct reading."
            }
            LimitationCode::Sampling => "This value is derived from a sample, not a full count.",
            LimitationCode::KernelEstimate => {
                "This value is an estimate supplied by the operating system, not a direct physical measurement."
            }
        }
    }
}
