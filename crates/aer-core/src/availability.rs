//! Availability — two distinct concepts, not one shared enum. Source
//! availability: whether the external source is currently usable. Metric
//! availability: whether a particular metric is obtainable once its source
//! context is considered. Semantics per design §4.3.

/// Whether the external source required to acquire information is currently
/// usable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceAvailability {
    /// Source present, reachable, responding.
    Ready,
    /// Source not found at all.
    NotDetected,
    /// Detected but not responding (installed, stopped).
    NotRunning,
    /// Detected as a target but cannot be reached.
    Unreachable,
}

/// Whether a particular metric is currently obtainable once its source
/// context is considered. Left unevaluated (`None`) when the source is not
/// `Ready`, rather than guessed. `NotApplicable` is used only when `Support`
/// is `Unsupported`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetricAvailability {
    /// Source ready and the metric is exposed.
    Available,
    /// Source ready but does not expose this metric.
    NotExposed,
    /// Source ready but cleanly indicates "not now".
    TransientlyUnavailable,
    /// Support is `Unsupported`, so the metric is moot.
    NotApplicable,
}
