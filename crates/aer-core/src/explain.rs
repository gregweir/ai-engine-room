//! Deterministic explanation boundary.
//!
//! `explain()` is a pure function selecting developer-authored constant
//! strings from domain state. Explanations derive from the metric result, not
//! from value magnitude, so a high value never triggers alarmist language.
//! Hardware suitability, model compatibility, safety, and performance claims
//! are not inferred here. No embedded LLM, rules-engine framework, or
//! external content dependency is used. Semantics per design §6.

use crate::availability::{MetricAvailability, SourceAvailability};
use crate::identity::MetricDescriptor;
use crate::outcome::Outcome;
use crate::provenance::Provenance;
use crate::result::MetricResult;
use crate::sample::MetricSample;
use crate::support::Support;
use crate::value::LimitationCode;

/// A developer-authored, user-facing string. Wraps a static constant so the
/// explanation surface stays controlled and deterministic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlledMessage(&'static str);

impl ControlledMessage {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

/// An explanation of a metric result: a concise interpretation, why it
/// matters, and an optional deeper note. All three are controlled messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explanation {
    pub interpretation: ControlledMessage,
    pub why_it_matters: ControlledMessage,
    pub deeper: Option<ControlledMessage>,
}

// Controlled message constants. Kept small and state-driven; no value
// thresholds and no hardware/safety/performance claims.
const UNSUPPORTED_INTERP: ControlledMessage =
    ControlledMessage("This metric is not supported in this build.");
const UNSUPPORTED_WHY: ControlledMessage = ControlledMessage(
    "Engine Room does not yet know how to acquire it here, so no value is shown.",
);
const NO_VALUE_SOURCE_INTERP: ControlledMessage =
    ControlledMessage("The source needed for this metric is not available right now.");
const NO_VALUE_METRIC_INTERP: ControlledMessage =
    ControlledMessage("The source is available but is not providing this metric right now.");
const NO_VALUE_WHY: ControlledMessage = ControlledMessage(
    "No value is shown because the metric is unavailable, not because of an error.",
);
const OK_WHY: ControlledMessage =
    ControlledMessage("This is the most recent reading for this metric.");
const RAM_AVAILABLE_INTERP: ControlledMessage = ControlledMessage(
    "This is Linux's estimate of how much memory is available for starting new applications without swapping.",
);
const RAM_AVAILABLE_WHY: ControlledMessage = ControlledMessage(
    "Local AI models need memory to load and run. A low or shrinking value reduces headroom and can slow inference if the system starts using swap.",
);
const RAM_AVAILABLE_DEEPER: ControlledMessage = ControlledMessage(
    "The value comes from /proc/meminfo MemAvailable. It is an estimate, not a guarantee that a single process can allocate the full amount.",
);
const PLATFORM_RAM_AVAILABLE_INTERP: ControlledMessage = ControlledMessage(
    "This is the operating system's observation of how much physical memory is currently available.",
);
const PLATFORM_RAM_AVAILABLE_DEEPER: ControlledMessage = ControlledMessage(
    "Operating systems define available memory differently. Values from different platforms are not claimed to be numerically equivalent.",
);
const RAM_TOTAL_INTERP: ControlledMessage =
    ControlledMessage("This is the operating system's observation of total system memory.");
const RAM_TOTAL_WHY: ControlledMessage = ControlledMessage(
    "Total memory provides machine context, but it does not establish memory pressure, model fit, or available headroom.",
);
const RAM_TOTAL_DEEPER: ControlledMessage = ControlledMessage(
    "Linux reports usable RAM through /proc/meminfo MemTotal; Windows reports physical RAM through MEMORYSTATUSEX. Values keep their platform-specific definitions.",
);
const FAILED_INTERP: ControlledMessage = ControlledMessage("Acquiring this metric failed.");
const FAILED_DEEPER: ControlledMessage =
    ControlledMessage("This is an acquisition failure, not a normal unavailable state.");

/// Produce an explanation from a metric result. Pure: identical inputs yield
/// identical outputs. The descriptor is accepted for per-metric explanation
/// branches; explanation text does not echo the descriptor's display name.
pub fn explain(descriptor: &MetricDescriptor, result: &MetricResult) -> Explanation {
    match &result.support {
        Support::Unsupported { limitation } => Explanation {
            interpretation: UNSUPPORTED_INTERP,
            why_it_matters: UNSUPPORTED_WHY,
            deeper: Some(ControlledMessage(limitation.message())),
        },
        Support::Supported => match &result.outcome {
            Outcome::Ok(sample) => ok_explanation(descriptor, sample),
            Outcome::NoValue => no_value_explanation(result),
            Outcome::Failed(err) => Explanation {
                interpretation: FAILED_INTERP,
                why_it_matters: ControlledMessage(err.message()),
                deeper: Some(FAILED_DEEPER),
            },
        },
    }
}

fn ok_explanation(descriptor: &MetricDescriptor, sample: &MetricSample) -> Explanation {
    if descriptor.id.as_str() == "os.ram.total" {
        return Explanation {
            interpretation: RAM_TOTAL_INTERP,
            why_it_matters: RAM_TOTAL_WHY,
            deeper: Some(RAM_TOTAL_DEEPER),
        };
    }
    if descriptor.id.as_str() == "os.ram.available" {
        if !sample.limitations.contains(&LimitationCode::KernelEstimate) {
            return Explanation {
                interpretation: PLATFORM_RAM_AVAILABLE_INTERP,
                why_it_matters: RAM_AVAILABLE_WHY,
                deeper: Some(PLATFORM_RAM_AVAILABLE_DEEPER),
            };
        }
        return Explanation {
            interpretation: RAM_AVAILABLE_INTERP,
            why_it_matters: RAM_AVAILABLE_WHY,
            deeper: Some(RAM_AVAILABLE_DEEPER),
        };
    }

    let interpretation = match &sample.provenance {
        Provenance::OperatingSystemReported => {
            ControlledMessage("This value was reported by the operating system.")
        }
        Provenance::DriverReported => {
            ControlledMessage("This value was reported by a hardware driver.")
        }
        Provenance::RuntimeReported => {
            ControlledMessage("This value was reported by the AI runtime.")
        }
        Provenance::ApplicationMeasured => {
            ControlledMessage("Engine Room measured this value directly.")
        }
        Provenance::Calculated { .. } => {
            ControlledMessage("This value was calculated from other metrics.")
        }
        Provenance::Estimated { .. } => {
            ControlledMessage("This value is an estimate based on other metrics.")
        }
    };
    Explanation {
        interpretation,
        why_it_matters: OK_WHY,
        deeper: None,
    }
}

fn no_value_explanation(result: &MetricResult) -> Explanation {
    let interpretation = match (&result.source_availability, &result.metric_availability) {
        // Source not ready (row 2): the source state explains the absence.
        (Some(SourceAvailability::NotDetected), _)
        | (Some(SourceAvailability::NotRunning), _)
        | (Some(SourceAvailability::Unreachable), _) => NO_VALUE_SOURCE_INTERP,
        // Source ready, metric not exposed or transient (row 3).
        (_, Some(MetricAvailability::NotExposed))
        | (_, Some(MetricAvailability::TransientlyUnavailable)) => NO_VALUE_METRIC_INTERP,
        // Any other legal NoValue combination: the source-state framing is
        // the safe default and still does not claim failure.
        _ => NO_VALUE_SOURCE_INTERP,
    };
    Explanation {
        interpretation,
        why_it_matters: NO_VALUE_WHY,
        deeper: None,
    }
}
