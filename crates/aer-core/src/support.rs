//! Support — whether Engine Room knows how to acquire the metric or interact
//! with the provider/runtime on this platform and in this implementation.
//! `Unsupported` is a stable capability statement, not a runtime failure.
//! Semantics per design §4.2.

use crate::value::LimitationCode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Support {
    Supported,
    Unsupported { limitation: LimitationCode },
}

impl Support {
    /// A stable limitation for a metric id that this provider does not know how
    /// to acquire. This is a capability statement, not a runtime failure.
    pub fn limitation_unsupported_metric() -> LimitationCode {
        LimitationCode::NotImplementedOnThisPlatform
    }

    pub fn limitation_unsupported_metric_message() -> &'static str {
        "This metric is not implemented in this provider."
    }
}
