//! A metric result combines support, source availability, metric
//! availability, and the acquisition outcome. The four concepts stay distinct
//! and are validated by `is_legal()` against design §4.5. `source_availability`
//! is `Option` (`None` when `Unsupported`/not evaluated); `metric_availability`
//! is `Option` (`None` when the source is not `Ready`).

use crate::availability::{MetricAvailability, SourceAvailability};
use crate::outcome::{AcquisitionError, Outcome};
use crate::sample::MetricSample;
use crate::support::Support;
use crate::value::LimitationCode;

/// The complete result of attempting a metric: capability, both availability
/// layers, and the outcome of the acquisition itself. Fields are public so the
/// view/IPC boundary can read them; `is_legal()` is the authoritative check
/// that a combination matches §4.5.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricResult {
    pub support: Support,
    pub source_availability: Option<SourceAvailability>,
    pub metric_availability: Option<MetricAvailability>,
    pub outcome: Outcome,
}

impl MetricResult {
    /// Row 1: not implemented here. Show the limitation, not an error.
    pub fn unsupported(limitation: LimitationCode) -> Self {
        Self {
            support: Support::Unsupported { limitation },
            source_availability: None,
            metric_availability: Some(MetricAvailability::NotApplicable),
            outcome: Outcome::NoValue,
        }
    }

    /// Row 2: normal — source state explains why there is no value.
    pub fn source_not_ready(source: SourceAvailability) -> Self {
        debug_assert!(
            source != SourceAvailability::Ready,
            "source_not_ready requires a non-Ready source"
        );
        Self {
            support: Support::Supported,
            source_availability: Some(source),
            metric_availability: None,
            outcome: Outcome::NoValue,
        }
    }

    /// Row 3: normal — metric state explains why there is no value.
    pub fn metric_not_available(metric: MetricAvailability) -> Self {
        debug_assert!(
            matches!(
                metric,
                MetricAvailability::NotExposed | MetricAvailability::TransientlyUnavailable
            ),
            "metric_not_available requires NotExposed or TransientlyUnavailable"
        );
        Self {
            support: Support::Supported,
            source_availability: Some(SourceAvailability::Ready),
            metric_availability: Some(metric),
            outcome: Outcome::NoValue,
        }
    }

    /// Row 4: a value was acquired; the sample carries provenance.
    pub fn ok(sample: MetricSample) -> Self {
        Self {
            support: Support::Supported,
            source_availability: Some(SourceAvailability::Ready),
            metric_availability: Some(MetricAvailability::Available),
            outcome: Outcome::Ok(sample),
        }
    }

    /// Row 5: an actual acquisition attempt failed.
    pub fn failed(err: AcquisitionError) -> Self {
        Self {
            support: Support::Supported,
            source_availability: Some(SourceAvailability::Ready),
            metric_availability: Some(MetricAvailability::Available),
            outcome: Outcome::Failed(err),
        }
    }

    /// True when the outcome is `Failed`. Unavailability (`NoValue`) is not a
    /// failure (§4.6).
    pub fn is_failure(&self) -> bool {
        matches!(self.outcome, Outcome::Failed(_))
    }

    /// Validates this result against the five legal rows of §4.5 exactly,
    /// including rejecting `NotApplicable` paired with `Supported`.
    pub fn is_legal(&self) -> bool {
        use MetricAvailability::*;
        use Outcome::*;
        use SourceAvailability::*;
        use Support::*;
        match (
            &self.support,
            &self.source_availability,
            &self.metric_availability,
            &self.outcome,
        ) {
            // Row 1: Unsupported | not evaluated | NotApplicable | NoValue
            (Unsupported { .. }, None, Some(NotApplicable), NoValue) => true,
            // Row 2: Supported | not Ready | None | NoValue
            (Supported, Some(s), None, NoValue) if *s != Ready => true,
            // Row 3: Supported | Ready | not Available | NoValue
            (Supported, Some(Ready), Some(NotExposed), NoValue) => true,
            (Supported, Some(Ready), Some(TransientlyUnavailable), NoValue) => true,
            // Row 4: Supported | Ready | Available | Ok
            (Supported, Some(Ready), Some(Available), Ok(_)) => true,
            // Row 5: Supported | Ready | Available | Failed
            (Supported, Some(Ready), Some(Available), Failed(_)) => true,
            _ => false,
        }
    }
}
