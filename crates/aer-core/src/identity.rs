//! Metric identity: the stable identity of a metric and the class of its
//! source. Semantics per design §4.1.

/// A stable, opaque identifier for a metric (e.g. `"os.ram.used"`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MetricId {
    value: String,
}

impl MetricId {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// The broad class of system a metric's source belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceClass {
    OperatingSystem,
    AiRuntime,
}

/// The physical dimension a metric is measured in. Distinct from
/// [`SampleValue`](crate::value::SampleValue), which is the observed reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    Bytes,
    Count,
    Percent,
    Seconds,
}

/// A metric's catalogue identity: what it is, how it is shown to people, the
/// unit it is measured in, and the class of its source. This is a definition,
/// not a runtime observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricDescriptor {
    pub id: MetricId,
    pub display_name: String,
    pub unit: Unit,
    pub source_class: SourceClass,
}
