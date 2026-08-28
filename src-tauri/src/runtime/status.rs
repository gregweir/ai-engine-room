//! Runtime status metadata — application layer, not `aer-core`.
//!
//! A detected runtime's version is *runtime metadata*, not a metric. These
//! types live here in `src-tauri` so `aer-core`, `SampleValue`, `MetricSample`,
//! metric `Provenance`, the report boundary, and the `AiRuntimeAdapter` trait
//! stay unchanged. `SourceAvailability` is reused from `aer-core` unchanged.
//!
//! The version provenance here is deliberately distinct from `aer-core`'s metric
//! `Provenance`: it records how a *runtime version string* (when present) was
//! obtained, and in Milestone 1C the only variant is `RuntimeReported` — Engine
//! Room never infers or constructs a version. A future inference path, if any,
//! would require a distinct marker; none is added now.

use aer_core::SourceAvailability;

/// Controlled identity of a supported runtime. Runtime-neutral; no Ollama type
/// enters `aer-core`. Future runtimes add a variant here (and an adapter).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeType {
    Ollama,
    LmStudio,
}

/// How a version string, when present, was obtained. Distinct from `aer-core`'s
/// metric `Provenance` (which applies to metric values). In 1C the only variant
/// is `RuntimeReported`: the version came from the runtime's own API. There is
/// no inference path in 1C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionProvenance {
    RuntimeReported,
}

/// A version string the runtime reported, preserved verbatim, plus how it was
/// obtained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeVersion {
    /// The version string verbatim from the runtime's API (no semver parsing,
    /// no normalisation).
    pub value: String,
    pub provenance: VersionProvenance,
}

/// The status of a supported runtime at one detection probe. The four intended
/// user-facing states are fully distinguished by `(source_availability,
/// version.is_some())`:
/// - `Ready` + `Some` → runtime available, version reported;
/// - `Ready` + `None` → reached, but responded unexpectedly;
/// - `NotDetected` + `None` → nothing responded at the local endpoint;
/// - `Unreachable` + `None` → could not be reached.
///
/// `NotRunning` is never produced for Ollama in 1C (an HTTP probe cannot
/// establish installed-but-stopped).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub runtime_type: RuntimeType,
    pub source_availability: SourceAvailability,
    pub version: Option<RuntimeVersion>,
}

impl RuntimeStatus {
    /// Construct the status for a runtime type from the source availability and
    /// optional verbatim version reported by the runtime.
    pub fn new(
        runtime_type: RuntimeType,
        source_availability: SourceAvailability,
        version: Option<RuntimeVersion>,
    ) -> Self {
        Self {
            runtime_type,
            source_availability,
            version,
        }
    }
}
