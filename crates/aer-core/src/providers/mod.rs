//! Dependency-neutral provider and runtime-adapter interfaces.
//!
//! These traits are the boundary across which the application acquires metric
//! data. They reference only core domain types — no platform, runtime,
//! serialisation, or networking types, and no Linux/Ollama-specific shapes. A
//! future real OS provider, an Ollama adapter, and an LM Studio adapter all
//! implement the same traits; the traits do not assume any one runtime exposes
//! identical capabilities. Semantics per design §5.

use crate::availability::SourceAvailability;
use crate::identity::{MetricDescriptor, MetricId};
use crate::result::MetricResult;

/// A runtime-neutral identity for a model an AI runtime knows about. Holds an
/// opaque id and a display name only — it does not encode Ollama tags,
/// parameter counts, or any other runtime-specific shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeModel {
    pub id: String,
    pub display_name: String,
}

/// Source of operating-system-level metrics. Implementations read from the
/// platform (e.g. /proc, sysfs) in a real build; the 1A mock is deterministic.
/// The trait is `Send + Sync` so a provider can be shared across the Tauri
/// async boundary.
pub trait OsMetricsProvider: Send + Sync {
    /// Whether the OS metric source is currently usable.
    fn source_availability(&self) -> SourceAvailability;
    /// The metrics this provider can attempt to acquire.
    fn list_metrics(&self) -> Vec<MetricDescriptor>;
    /// Attempt to acquire one metric. Unsupported or unavailable metrics return
    /// a legal `MetricResult` (e.g. `NoValue`/`Unsupported`), never a panic.
    fn acquire(&self, id: &MetricId) -> MetricResult;
}

/// Adapter to an AI runtime (e.g. Ollama or LM Studio). Runtime-neutral: the
/// trait exposes model listing, loaded-model listing, and runtime metric
/// acquisition without assuming any one runtime's data shapes. `Send + Sync`.
pub trait AiRuntimeAdapter: Send + Sync {
    /// Whether the runtime is currently detected and usable.
    fn detect(&self) -> SourceAvailability;
    /// Models the runtime knows about (installed, not necessarily loaded).
    fn list_models(&self) -> Vec<RuntimeModel>;
    /// Models currently loaded into the runtime.
    fn loaded_models(&self) -> Vec<RuntimeModel>;
    /// Attempt to acquire one runtime metric. Unavailable metrics return a
    /// legal `MetricResult`, never a panic.
    fn runtime_metrics(&self, id: &MetricId) -> MetricResult;
}
