//! A runtime observation: a value and its provenance. `MetricSample` carries
//! only what is required to represent the observation and its provenance — no
//! per-sample evidence field (design §4.1).

use crate::identity::Unit;
use crate::provenance::Provenance;
use crate::value::{LimitationCode, SampleValue, Timestamp};

/// A single observed metric reading and the provenance under which it was
/// obtained. Exactly five fields; validation evidence is a separate concern
/// that lives in tests and provider documentation, not on the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricSample {
    pub value: SampleValue,
    pub unit: Unit,
    pub provenance: Provenance,
    pub timestamp: Timestamp,
    pub limitations: Vec<LimitationCode>,
}
