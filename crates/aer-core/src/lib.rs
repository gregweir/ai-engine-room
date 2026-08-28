// AI Engine Room domain core.
//
// Depends only on its own types and the standard library. No platform,
// runtime, serialization, or third-party crates appear here. See
// docs/design/architecture.md (§4.7) for the boundary this enforces.

pub mod availability;
pub mod explain;
pub mod identity;
pub mod outcome;
pub mod provenance;
pub mod providers;
pub mod report;
pub mod result;
pub mod sample;
pub mod session;
pub mod support;
pub mod value;

#[cfg(feature = "mock")]
pub mod mock;

pub use availability::{MetricAvailability, SourceAvailability};
pub use explain::{explain, ControlledMessage, Explanation};
pub use identity::{MetricDescriptor, MetricId, SourceClass, Unit};
pub use outcome::{AcquisitionError, Outcome};
pub use provenance::{MetricRef, Provenance};
pub use providers::{AiRuntimeAdapter, OsMetricsProvider, RuntimeModel};
pub use report::{
    project, render_preview, sanitize, ReportEntry, ReportProvenance, ReportSnapshot, ReportStatus,
    ReportValue, REPORT_ALLOWED_METRICS,
};
pub use result::MetricResult;
pub use sample::MetricSample;
pub use session::{Event, Session, Snapshot, SnapshotEntry};
pub use support::Support;
pub use value::{Formula, LimitationCode, Model, SampleValue, Timestamp};
