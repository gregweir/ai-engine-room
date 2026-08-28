//! Provenance — how a value that exists was obtained. Provenance applies only
//! when a value exists; it is not a confidence ranking. Semantics per design
//! §4.1.

use crate::identity::MetricId;
use crate::value::{Formula, LimitationCode, Model};

/// A reference to a metric used as an input to a derived provenance, paired
/// with the provenance under which that input was itself obtained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricRef {
    pub id: MetricId,
    pub provenance: Provenance,
}

/// The six provenance categories. `Calculated` and `Estimated` preserve their
/// inputs and (respectively) the formula or the assumptions and limitations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Read from the operating system (e.g. `/proc`, `sysfs`).
    OperatingSystemReported,
    /// From a hardware driver API (e.g. a GPU driver).
    DriverReported,
    /// From the AI runtime's own API.
    RuntimeReported,
    /// Engine Room itself performed the measurement (e.g. an elapsed duration).
    ApplicationMeasured,
    /// Deterministic arithmetic over identified inputs.
    Calculated {
        inputs: Vec<MetricRef>,
        formula: Formula,
    },
    /// An estimate involving assumptions, modelling, or approximation rather
    /// than pure arithmetic. Distinct from `Calculated`.
    Estimated {
        inputs: Vec<MetricRef>,
        assumptions: Model,
        limitations: Vec<LimitationCode>,
    },
}
