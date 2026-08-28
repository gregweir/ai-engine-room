//! View/IPC DTOs: serializable projections of the aer-core domain model.
//!
//! These types live in the application (Tauri) layer, not in `aer-core`, so the
//! domain core stays serde-free. Each DTO is a controlled projection that
//! preserves the four distinct state concepts — support, source availability,
//! metric availability, and the acquisition outcome — rather than collapsing
//! them into a single generic status. JSON convenience does not override the
//! approved semantic separation.
//!
//! Nothing unsafe crosses this boundary: raw acquisition errors, provider
//! implementation details, internal platform/runtime objects, filesystem paths,
//! environment data, and unrestricted session internals are not represented
//! here. A failed outcome carries only a controlled category code; the
//! controlled human message travels through the explanation fields. Provenance
//! is carried only by the `Ok` outcome, where a value exists.

use aer_core::{MetricAvailability, Outcome, Provenance, SourceAvailability, Support, Unit};
use serde::{Deserialize, Serialize};

use crate::runtime::inference::{
    ollama_inference_text, InferenceObservation, InferenceObservationState,
};
use crate::runtime::inventory::{
    InventoryState, ModelInventory, ModelLocality, RuntimeModelSummary,
};
use crate::runtime::loaded_models::{LoadedModelSet, LoadedModelState, LoadedModelSummary};
use crate::runtime::ollama::{
    ollama_inventory_text, ollama_loaded_models_text, ollama_status_text, resource_context_text,
};
use crate::runtime::status::{RuntimeStatus, RuntimeType};

// --- Bare-string enums (unit variants serialise as controlled strings) --------

/// The unit a metric is measured in. Serialised as a lowercase controlled
/// string (`bytes`, `count`, `percent`, `seconds`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitView {
    Bytes,
    Count,
    Percent,
    Seconds,
}

impl UnitView {
    pub fn from(unit: &Unit) -> Self {
        match unit {
            Unit::Bytes => Self::Bytes,
            Unit::Count => Self::Count,
            Unit::Percent => Self::Percent,
            Unit::Seconds => Self::Seconds,
        }
    }
}

/// Source availability, when it was evaluated (`None` when unevaluated, e.g.
/// for an unsupported capability). Serialised as a controlled snake_case
/// string.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAvailabilityView {
    Ready,
    NotDetected,
    NotRunning,
    Unreachable,
}

impl SourceAvailabilityView {
    pub fn from(s: &SourceAvailability) -> Self {
        match s {
            SourceAvailability::Ready => Self::Ready,
            SourceAvailability::NotDetected => Self::NotDetected,
            SourceAvailability::NotRunning => Self::NotRunning,
            SourceAvailability::Unreachable => Self::Unreachable,
        }
    }
}

/// Metric availability, when it was evaluated (`None` when the source is not
/// `Ready`). Serialised as a controlled snake_case string.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricAvailabilityView {
    Available,
    NotExposed,
    TransientlyUnavailable,
    NotApplicable,
}

impl MetricAvailabilityView {
    pub fn from(m: &MetricAvailability) -> Self {
        match m {
            MetricAvailability::Available => Self::Available,
            MetricAvailability::NotExposed => Self::NotExposed,
            MetricAvailability::TransientlyUnavailable => Self::TransientlyUnavailable,
            MetricAvailability::NotApplicable => Self::NotApplicable,
        }
    }
}

/// The six provenance categories as a payload-free report-safe projection. The
/// internal `Calculated`/`Estimated` inputs, formula, and assumptions do not
/// cross into the view. Serialised as a controlled snake_case string.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceView {
    OperatingSystemReported,
    DriverReported,
    RuntimeReported,
    ApplicationMeasured,
    Calculated,
    Estimated,
}

impl ProvenanceView {
    pub fn from(p: &Provenance) -> Self {
        match p {
            Provenance::OperatingSystemReported => Self::OperatingSystemReported,
            Provenance::DriverReported => Self::DriverReported,
            Provenance::RuntimeReported => Self::RuntimeReported,
            Provenance::ApplicationMeasured => Self::ApplicationMeasured,
            Provenance::Calculated { .. } => Self::Calculated,
            Provenance::Estimated { .. } => Self::Estimated,
        }
    }
}

// --- Internally-tagged enums (carry data) -------------------------------------

/// Support state. `Unsupported` carries a controlled limitation code (a stable
/// snake_case string); the controlled human message travels via the
/// explanation fields, not here.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SupportView {
    Supported,
    Unsupported { limitation_code: String },
}

/// The acquisition outcome category. `Ok` carries the value, its unit, and the
/// provenance (present only because a value exists). `Failed` carries only a
/// controlled category code — no raw error string. `NoValue` carries nothing;
/// the reason is conveyed by the separate availability fields and explanation.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutcomeView {
    Ok {
        value: f64,
        unit: UnitView,
        provenance: ProvenanceView,
    },
    NoValue,
    Failed {
        category: String,
    },
}

// --- Composite views ---------------------------------------------------------

/// One metric's view projection: identity (display name and declared unit),
/// the four state concepts kept distinct, the outcome, and the controlled
/// explanation text. No internal id, no raw error, no diagnostics.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SnapshotEntryView {
    pub display_name: String,
    pub unit: UnitView,
    pub support: SupportView,
    pub source_availability: Option<SourceAvailabilityView>,
    pub metric_availability: Option<MetricAvailabilityView>,
    pub outcome: OutcomeView,
    pub interpretation: String,
    pub why_it_matters: String,
    pub deeper: Option<String>,
}

/// A snapshot view: a timestamp (milliseconds since the Unix epoch) and the
/// projected entries.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SnapshotView {
    pub at: i64,
    pub entries: Vec<SnapshotEntryView>,
}

/// Map a domain snapshot to its view projection.
pub fn snapshot_view(snapshot: &aer_core::Snapshot) -> SnapshotView {
    SnapshotView {
        at: snapshot.at.millis_since_epoch(),
        entries: snapshot.entries.iter().map(entry_view).collect(),
    }
}

/// Map a single domain snapshot entry to its view projection.
pub fn entry_view(entry: &aer_core::SnapshotEntry) -> SnapshotEntryView {
    SnapshotEntryView {
        display_name: entry.descriptor.display_name.clone(),
        unit: UnitView::from(&entry.descriptor.unit),
        support: support_view(&entry.result.support),
        source_availability: entry
            .result
            .source_availability
            .as_ref()
            .map(SourceAvailabilityView::from),
        metric_availability: entry
            .result
            .metric_availability
            .as_ref()
            .map(MetricAvailabilityView::from),
        outcome: outcome_view(&entry.result.outcome),
        interpretation: entry.explanation.interpretation.as_str().to_string(),
        why_it_matters: entry.explanation.why_it_matters.as_str().to_string(),
        deeper: entry
            .explanation
            .deeper
            .as_ref()
            .map(|m| m.as_str().to_string()),
    }
}

// --- Runtime status view (Milestone 1C) ---------------------------------------

/// The controlled IPC projection of a runtime's detection status. This is
/// runtime *metadata* (the version a runtime reports), not a metric, so it
/// travels through its own DTO rather than `SnapshotEntryView`/the report
/// boundary. The four intended user-facing states are distinguished by
/// `(source_availability, version.is_some())`; the frontend renders the
/// pre-computed controlled text fields and does not infer detection semantics
/// from `source_availability`, status codes, or transport kinds.
///
/// No raw errors, URLs, hostnames, ports, status codes, or process information
/// cross this boundary. `version_reported_by` is present only when a version
/// exists, and in 1C is always `"runtime_reported"` (the version came from the
/// runtime's own API; Engine Room never infers one).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RuntimeStatusView {
    /// Controlled lowercase runtime identity (e.g. `"ollama"`). No runtime
    /// type enters `aer-core`; this string is authored in the app layer.
    pub runtime_type: String,
    pub source_availability: SourceAvailabilityView,
    /// The version string the runtime reported, verbatim, or `None`.
    pub version: Option<String>,
    /// How the version (when present) was obtained: `"runtime_reported"` in 1C,
    /// or `None` when there is no version.
    pub version_reported_by: Option<String>,
    pub status_label: String,
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map a runtime status to its controlled IPC projection. Controlled text is
/// produced by the application-layer Ollama text mapping (not `aer-core`, not
/// Svelte); the frontend renders these strings verbatim.
pub fn runtime_status_view(status: &RuntimeStatus) -> RuntimeStatusView {
    let text = ollama_status_text(status);
    RuntimeStatusView {
        runtime_type: runtime_type_view(status.runtime_type).to_string(),
        source_availability: SourceAvailabilityView::from(&status.source_availability),
        version: status.version.as_ref().map(|v| v.value.clone()),
        version_reported_by: status
            .version
            .as_ref()
            .map(|_| "runtime_reported".to_string()),
        status_label: text.status_label,
        interpretation: text.interpretation,
        why_it_matters: text.why_it_matters,
    }
}

/// Stable, controlled lowercase identity for a runtime type. Authored here so
/// no runtime identity lives in `aer-core`.
fn runtime_type_view(runtime_type: RuntimeType) -> &'static str {
    match runtime_type {
        RuntimeType::Ollama => "ollama",
        RuntimeType::LmStudio => "lm_studio",
    }
}

// --- Model inventory view (Milestone 1D) --------------------------------------

/// Evidence-based locality of a catalogue entry, serialised as a controlled
/// snake_case string. Always `"undetermined"` in 1D; `"local"` is reserved and
/// not assigned. Carried for forward use; not displayed per model in 1D's UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLocalityView {
    Local,
    Undetermined,
}

impl ModelLocalityView {
    pub fn from(locality: &ModelLocality) -> Self {
        match locality {
            ModelLocality::Local => Self::Local,
            ModelLocality::Undetermined => Self::Undetermined,
        }
    }
}

/// The inventory fetch state, serialised as a controlled snake_case string.
/// `available` with an empty `models` array is a normal state, not an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryStateView {
    Available,
    RespondedUnexpectedly,
    NotDetected,
    Unreachable,
}

impl InventoryStateView {
    pub fn from(state: InventoryState) -> Self {
        match state {
            InventoryState::Available => Self::Available,
            InventoryState::RespondedUnexpectedly => Self::RespondedUnexpectedly,
            InventoryState::NotDetected => Self::NotDetected,
            InventoryState::Unreachable => Self::Unreachable,
        }
    }
}

/// One model's inventory projection: display name, locality, reported size, and
/// the optional secondary labels. No `digest`, endpoint, host, port,
/// `remote_host`, or raw error crosses this boundary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RuntimeModelSummaryView {
    pub display_name: String,
    pub locality: ModelLocalityView,
    /// Reported model size in bytes; `None` when absent/invalid (not zero).
    pub size_bytes: Option<u64>,
    pub parameter_size: Option<String>,
    pub quantization: Option<String>,
    /// `details.families` preferred; `details.family` fallback. Held for
    /// forward use; rendered as a secondary label when present.
    pub families: Vec<String>,
}

/// The controlled IPC projection of a runtime's model inventory. This is
/// runtime metadata (the catalogue a runtime reports), not a metric, so it
/// travels through its own DTO rather than the snapshot/report boundary. The
/// frontend renders the pre-computed controlled text fields and does not infer
/// inventory state from availability, status codes, or transport kinds.
///
/// No raw errors, URLs, hostnames, ports, status codes, `digest`, or
/// `remote_host` cross this boundary. Locality is carried for forward use and
/// is `"undetermined"` for every entry in 1D; it is not displayed per model.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModelInventoryView {
    pub runtime_type: String,
    pub state: InventoryStateView,
    pub source_availability: SourceAvailabilityView,
    pub models: Vec<RuntimeModelSummaryView>,
    pub skipped: usize,
    pub state_label: String,
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map a model inventory to its controlled IPC projection. Controlled text is
/// produced by the application-layer Ollama inventory text mapping (not
/// `aer-core`, not Svelte); the frontend renders these strings verbatim.
pub fn model_inventory_view(inventory: &ModelInventory) -> ModelInventoryView {
    let text = ollama_inventory_text(inventory);
    ModelInventoryView {
        runtime_type: runtime_type_view(inventory.runtime_type).to_string(),
        state: InventoryStateView::from(inventory.state),
        source_availability: SourceAvailabilityView::from(&inventory.source_availability),
        models: inventory.models.iter().map(model_summary_view).collect(),
        skipped: inventory.skipped,
        state_label: text.state_label,
        interpretation: text.interpretation,
        why_it_matters: text.why_it_matters,
    }
}

fn model_summary_view(summary: &RuntimeModelSummary) -> RuntimeModelSummaryView {
    RuntimeModelSummaryView {
        display_name: summary.display_name.clone(),
        locality: ModelLocalityView::from(&summary.locality),
        size_bytes: summary.size_bytes,
        parameter_size: summary.parameter_size.clone(),
        quantization: summary.quantization.clone(),
        families: summary.families.clone(),
    }
}

// --- Loaded-models view (Milestone 1E) ----------------------------------------

/// The loaded-model fetch state, serialised as a controlled snake_case string.
/// `available` with an empty `models` array is a normal state, not an error
/// ("No models are currently running in Ollama").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadedModelStateView {
    Available,
    RespondedUnexpectedly,
    NotDetected,
    Unreachable,
}

impl LoadedModelStateView {
    pub fn from(state: LoadedModelState) -> Self {
        match state {
            LoadedModelState::Available => Self::Available,
            LoadedModelState::RespondedUnexpectedly => Self::RespondedUnexpectedly,
            LoadedModelState::NotDetected => Self::NotDetected,
            LoadedModelState::Unreachable => Self::Unreachable,
        }
    }
}

/// One loaded model's projection: display name and the optional
/// Ollama-reported residency fields. No `digest`, `details.*`, endpoint, host,
/// port, processor/offload, capabilities, or remote/cloud field crosses this
/// boundary. `expires_at` is carried verbatim for forward use and is **not**
/// displayed in 1E's initial UI; no countdown timer is implied.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct LoadedModelSummaryView {
    pub display_name: String,
    /// Ollama-reported loaded size in bytes; `None` when absent/invalid (not
    /// zero). "Ollama-reported loaded size" — not exact RAM/disk usage.
    pub reported_size_bytes: Option<u64>,
    /// Ollama-reported VRAM size in bytes; `None` when absent/invalid. Runtime-
    /// reported metadata, not independently-measured physical VRAM.
    pub reported_vram_bytes: Option<u64>,
    /// Ollama-reported context length as a count; `None` when absent/invalid.
    /// Runtime-reported, not a theoretical maximum or performance figure.
    pub reported_context_length: Option<u64>,
    /// Ollama-reported `expires_at` verbatim, carried for forward use; **not
    /// displayed** in 1E. A soft signal (activity can extend residency), not a
    /// guarantee; `None` when absent/non-string.
    pub expires_at: Option<String>,
}

/// The controlled IPC projection of a runtime's loaded-model set (Milestone 1E).
/// This is runtime metadata (what Ollama reports as currently loaded), not a
/// metric, so it travels through its own DTO rather than the snapshot/report
/// boundary. The frontend renders the pre-computed controlled text fields
/// verbatim and does not infer loaded-model state from availability, status
/// codes, or transport kinds.
///
/// No raw errors, URLs, hostnames, ports, status codes, `digest`, `details.*`,
/// processor/offload, or remote/cloud fields cross this boundary. 1D
/// (catalogue) and 1E (currently loaded) are distinct; no identity join to the
/// 1D catalogue and no locality claim for any loaded entry.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct LoadedModelSetView {
    pub runtime_type: String,
    pub state: LoadedModelStateView,
    pub source_availability: SourceAvailabilityView,
    pub models: Vec<LoadedModelSummaryView>,
    pub skipped: usize,
    pub state_label: String,
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map a loaded-model set to its controlled IPC projection. Controlled text is
/// produced by the application-layer Ollama loaded-models text mapping (not
/// `aer-core`, not Svelte); the frontend renders these strings verbatim.
pub fn loaded_models_view(set: &LoadedModelSet) -> LoadedModelSetView {
    let text = ollama_loaded_models_text(set);
    LoadedModelSetView {
        runtime_type: runtime_type_view(set.runtime_type).to_string(),
        state: LoadedModelStateView::from(set.state),
        source_availability: SourceAvailabilityView::from(&set.source_availability),
        models: set.models.iter().map(loaded_model_summary_view).collect(),
        skipped: set.skipped,
        state_label: text.state_label,
        interpretation: text.interpretation,
        why_it_matters: text.why_it_matters,
    }
}

fn loaded_model_summary_view(summary: &LoadedModelSummary) -> LoadedModelSummaryView {
    LoadedModelSummaryView {
        display_name: summary.display_name.clone(),
        reported_size_bytes: summary.reported_size_bytes,
        reported_vram_bytes: summary.reported_vram_bytes,
        reported_context_length: summary.reported_context_length,
        expires_at: summary.expires_at.clone(),
    }
}

// --- Resource context view (Milestone 1F) ------------------------------------

/// Controlled display name of the available-memory metric, used to locate that
/// entry within a `SnapshotView`. `SnapshotEntryView` carries no metric id (the
/// 1B view projection deliberately omits internal ids), so the available-memory
/// entry is identified by its controlled display name. This equals the 1B
/// `LinuxMetricsProvider` descriptor's `display_name` (`"Available memory"`); a
/// deterministic test pins the two together so the stringly match stays audited.
pub(crate) const AVAILABLE_MEMORY_DISPLAY_NAME: &str = "Available memory";

/// The controlled IPC projection of a resource-context composition (Milestone
/// 1F): OS-reported available memory placed alongside Ollama-reported
/// loaded-model state, with a deterministic explanation that distinguishes the
/// two sources and states that they must not be combined to attribute exact RAM
/// or VRAM use.
///
/// This DTO carries ONLY the controlled explanation. The available-memory value
/// is rendered by the retained Available-memory card (from `current_snapshot`);
/// the loaded-model list is rendered by the retained Currently-running card
/// (from `current_loaded_models`). The frontend passes those already-acquired
/// views into `compose_resource_context` and renders the returned
/// `interpretation`/`why_it_matters` verbatim. No aggregate, no ratio, no "used
/// RAM", no `size - size_vram`, no `available - loaded`, and no fit/health claim
/// is ever produced.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResourceContextView {
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Compose the resource-context explanation from the views the frontend already
/// acquired (Milestone 1F). PURE: no I/O — does not read `/proc/meminfo` and does
/// not call `/api/ps`. It derives `ram_has_value` by locating the available-memory
/// entry in `snapshot` (by its controlled display name) and checking its outcome
/// is `Ok`, and derives `loaded_is_available` from `loaded.state`; then it
/// delegates to the pure `resource_context_text` mapping (the four controlled
/// variants). The function takes only the two already-built view DTOs and returns
/// the controlled explanation, so it physically cannot read `/proc/meminfo` or
/// call `/api/ps` — the composition adds zero acquisition of either source. The
/// inputs are borrowed and never mutated.
pub fn compose_resource_context(
    snapshot: &SnapshotView,
    loaded: &LoadedModelSetView,
) -> ResourceContextView {
    let ram_has_value = snapshot.entries.iter().any(|e| {
        e.display_name == AVAILABLE_MEMORY_DISPLAY_NAME
            && matches!(e.outcome, OutcomeView::Ok { .. })
    });
    let loaded_is_available = matches!(loaded.state, LoadedModelStateView::Available);
    let text = resource_context_text(ram_has_value, loaded_is_available);
    ResourceContextView {
        interpretation: text.interpretation,
        why_it_matters: text.why_it_matters,
    }
}

// --- Observed inference view (Milestone 1G) -----------------------------------

/// The per-run authorization request for one observed inference. Carries the
/// explicitly selected model identifier and the execution-boundary
/// acknowledgement. `execution_boundary_acknowledged == false` blocks the run
/// with no POST; `true` authorizes one synthetic observation and is **not
/// locality proof and not cryptographic**. Request-scoped: the backend keeps no
/// global/persistent auth state, no allow-list, no session token, and no
/// "Don't ask again".
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InferenceObservationRequest {
    /// Closed provider identity. Missing values from pre-1L callers default to
    /// Ollama for wire compatibility; the 1L UI always sends it explicitly.
    #[serde(default = "default_ollama_provider")]
    pub provider: String,
    /// The model to run the fixed diagnostic prompt against, selected from the
    /// 1D catalogue. The backend validates it defensively and against the
    /// current catalogue.
    pub model: String,
    /// Whether the user acknowledged the undetermined execution boundary for
    /// this one run. `false` → `blocked_by_execution_boundary`, no POST.
    pub execution_boundary_acknowledged: bool,
    /// Acknowledges provider-specific side effects, notably LM Studio JIT
    /// loading. Required for LM Studio only and scoped to this request.
    #[serde(default)]
    pub provider_behavior_acknowledged: bool,
}

fn default_ollama_provider() -> String {
    "ollama".to_string()
}

/// The controlled IPC projection of one observed-inference run (Milestone 1G).
/// This is runtime metadata (the counts/durations Ollama reported for one run,
/// plus an Engine-Room-calculated rate), not a metric, so it travels through its
/// own DTO rather than the snapshot/report boundary. The frontend renders the
/// pre-computed controlled text fields verbatim and does not infer locality,
/// benchmark, or quality from the counts.
///
/// No raw errors, URLs, hostnames, ports, status codes, **generated response
/// text**, transport kinds, or locality claims cross this boundary. Durations
/// are carried in nanoseconds (Ollama's unit); the frontend renders them as
/// seconds. `execution_boundary` is always `"undetermined"` in 1G — the only
/// value, no enum, no Local/Cloud variants. `generation_tokens_per_second` is
/// Engine-Room-calculated (not Ollama-reported); the other counts/durations are
/// Ollama-reported. `diagnostic_profile` is present only when a model was
/// selected.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InferenceObservationView {
    pub provider: String,
    /// The controlled result state (snake_case), e.g. `completed`,
    /// `blocked_by_execution_boundary`, `running`.
    pub state: InferenceObservationState,
    /// The model the run targeted, when one was selected. `None` only for
    /// `no_model_selected`.
    pub model: Option<String>,
    /// The fixed diagnostic profile identifier, present only when a model was
    /// selected.
    pub diagnostic_profile: Option<String>,
    /// The execution-boundary string. Always `"undetermined"` in 1G.
    pub execution_boundary: String,
    /// Ollama-reported prompt token count for the run. `None` when absent.
    pub prompt_eval_count: Option<u64>,
    /// Ollama-reported generated (output) token count for the run. `None` when
    /// absent.
    pub eval_count: Option<u64>,
    /// Ollama-reported model load duration, in nanoseconds. `None` when absent.
    pub load_duration_ns: Option<u64>,
    /// Ollama-reported prompt evaluation duration, in nanoseconds. `None` when
    /// absent.
    pub prompt_eval_duration_ns: Option<u64>,
    /// Ollama-reported generation (eval) duration, in nanoseconds. `None` when
    /// absent.
    pub eval_duration_ns: Option<u64>,
    /// Ollama-reported total duration, in nanoseconds. `None` when absent.
    pub total_duration_ns: Option<u64>,
    /// Engine-Room-calculated generation rate in tokens per second, from
    /// `eval_count` and `eval_duration_ns`. `None` unless the §8 guards pass.
    /// Not Ollama-reported.
    pub generation_tokens_per_second: Option<f64>,
    pub input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub provider_reported_tokens_per_second: Option<f64>,
    pub time_to_first_token_seconds: Option<f64>,
    pub model_load_time_seconds: Option<f64>,
    pub interpretation: String,
    pub why_it_matters: String,
    pub limitations: String,
}

/// Map an observed inference to its controlled IPC projection. Controlled text is
/// produced by the application-layer Ollama inference text mapping (not
/// `aer-core`, not Svelte); the frontend renders these strings verbatim.
/// `diagnostic_profile` is present only when a model was selected; the metrics
/// are copied through unchanged (durations in nanoseconds). PURE: no I/O.
pub fn inference_observation_view(observation: &InferenceObservation) -> InferenceObservationView {
    let text = ollama_inference_text(observation);
    InferenceObservationView {
        provider: "ollama".to_string(),
        state: observation.state,
        model: observation.model.clone(),
        diagnostic_profile: observation
            .model
            .as_ref()
            .map(|_| observation.diagnostic_profile.to_string()),
        execution_boundary: observation.execution_boundary.to_string(),
        prompt_eval_count: observation.prompt_eval_count,
        eval_count: observation.eval_count,
        load_duration_ns: observation.load_duration_ns,
        prompt_eval_duration_ns: observation.prompt_eval_duration_ns,
        eval_duration_ns: observation.eval_duration_ns,
        total_duration_ns: observation.total_duration_ns,
        generation_tokens_per_second: observation.generation_tokens_per_second,
        input_tokens: None,
        total_output_tokens: None,
        reasoning_output_tokens: None,
        provider_reported_tokens_per_second: None,
        time_to_first_token_seconds: None,
        model_load_time_seconds: None,
        interpretation: text.interpretation,
        why_it_matters: text.why_it_matters,
        limitations: text.limitations,
    }
}

fn support_view(support: &Support) -> SupportView {
    match support {
        Support::Supported => SupportView::Supported,
        Support::Unsupported { limitation } => SupportView::Unsupported {
            limitation_code: limitation_code(limitation).to_string(),
        },
    }
}

/// A stable, controlled snake_case code for a limitation. Kept explicit (not a
/// `Debug` format) so it is stable and language-independent.
fn limitation_code(limitation: &aer_core::LimitationCode) -> &'static str {
    match limitation {
        aer_core::LimitationCode::NotImplementedOnThisPlatform => {
            "not_implemented_on_this_platform"
        }
        aer_core::LimitationCode::NotSupportedByRuntime => "not_supported_by_runtime",
        aer_core::LimitationCode::RequiresDriver => "requires_driver",
        aer_core::LimitationCode::RequiresElevatedPrivileges => "requires_elevated_privileges",
        aer_core::LimitationCode::Approximation => "approximation",
        aer_core::LimitationCode::Sampling => "sampling",
        aer_core::LimitationCode::KernelEstimate => "kernel_estimate",
    }
}

fn outcome_view(outcome: &Outcome) -> OutcomeView {
    match outcome {
        Outcome::Ok(sample) => OutcomeView::Ok {
            value: sample_value_to_f64(&sample.value),
            unit: UnitView::from(&sample.unit),
            provenance: ProvenanceView::from(&sample.provenance),
        },
        Outcome::NoValue => OutcomeView::NoValue,
        Outcome::Failed(err) => OutcomeView::Failed {
            category: err.category().to_string(),
        },
    }
}

fn sample_value_to_f64(v: &aer_core::SampleValue) -> f64 {
    match v {
        aer_core::SampleValue::Integer(i) => *i as f64,
        aer_core::SampleValue::Float(f) => *f,
        aer_core::SampleValue::Count(c) => *c as f64,
    }
}

#[cfg(test)]
mod tests {
    //! Mapping tests construct domain values directly (no `mock` feature
    //! required) and assert the four state concepts stay distinct, provenance
    //! appears only with a value, and a failed outcome carries only a
    //! controlled category.

    use super::*;
    use aer_core::{
        explain, MetricAvailability, MetricDescriptor, MetricId, MetricResult, MetricSample,
        Provenance, SampleValue, Snapshot, SnapshotEntry, SourceAvailability, SourceClass,
        Timestamp, Unit,
    };

    fn descriptor(id: &str, name: &str) -> MetricDescriptor {
        MetricDescriptor {
            id: MetricId::new(id),
            display_name: String::from(name),
            unit: Unit::Bytes,
            source_class: SourceClass::OperatingSystem,
        }
    }

    fn ok_sample(value: u64, provenance: Provenance) -> MetricSample {
        MetricSample {
            value: SampleValue::Count(value),
            unit: Unit::Bytes,
            provenance,
            timestamp: Timestamp::from_millis(0),
            limitations: Vec::new(),
        }
    }

    fn entry(descriptor: MetricDescriptor, result: MetricResult) -> SnapshotEntry {
        let explanation = explain(&descriptor, &result);
        SnapshotEntry {
            descriptor,
            result,
            explanation,
        }
    }

    fn snapshot(entries: Vec<SnapshotEntry>) -> Snapshot {
        Snapshot {
            at: Timestamp::from_millis(1),
            entries,
        }
    }

    #[test]
    fn ok_outcome_carries_value_unit_and_provenance() {
        let e = entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::ok(ok_sample(1024, Provenance::OperatingSystemReported)),
        );
        let v = entry_view(&e);
        assert_eq!(v.display_name, "Memory in use");
        assert_eq!(v.unit, UnitView::Bytes);
        assert_eq!(v.support, SupportView::Supported);
        match &v.outcome {
            OutcomeView::Ok {
                value,
                unit,
                provenance,
            } => {
                assert!((value - 1024.0).abs() < f64::EPSILON);
                assert_eq!(*unit, UnitView::Bytes);
                assert_eq!(*provenance, ProvenanceView::OperatingSystemReported);
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn failed_outcome_carries_controlled_category_only() {
        let e = entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::failed(aer_core::AcquisitionError::Timeout),
        );
        let v = entry_view(&e);
        // The outcome carries only a controlled category code — never raw text.
        match &v.outcome {
            OutcomeView::Failed { category } => assert_eq!(category, "timeout"),
            other => panic!("expected Failed, got {other:?}"),
        }
        // The controlled human message travels through the explanation, not the
        // outcome, and is a developer-authored string (not a raw system string).
        assert_eq!(
            v.why_it_matters,
            aer_core::AcquisitionError::Timeout.message()
        );
    }

    #[test]
    fn unsupported_maps_to_unsupported_with_limitation_code() {
        let e = entry(
            descriptor("gpu.fan.rpm", "GPU fan speed"),
            MetricResult::unsupported(aer_core::LimitationCode::RequiresDriver),
        );
        let v = entry_view(&e);
        match &v.support {
            SupportView::Unsupported { limitation_code } => {
                assert_eq!(limitation_code, "requires_driver");
            }
            other => panic!("expected Unsupported, got {other:?}"),
        }
        assert!(matches!(v.outcome, OutcomeView::NoValue));
        // Unsupported leaves source availability unevaluated.
        assert!(v.source_availability.is_none());
    }

    #[test]
    fn absence_states_remain_distinguishable() {
        // Source not ready (row 2): source carries the reason, metric unevaluated.
        let not_detected = entry(
            descriptor("rt.context.tokens", "AI runtime context tokens"),
            MetricResult::source_not_ready(SourceAvailability::NotDetected),
        );
        let v1 = entry_view(&not_detected);
        assert_eq!(
            v1.source_availability,
            Some(SourceAvailabilityView::NotDetected)
        );
        assert!(v1.metric_availability.is_none());
        assert!(matches!(v1.outcome, OutcomeView::NoValue));

        // Source ready, metric not exposed (row 3): metric carries the reason.
        let not_exposed = entry(
            descriptor("os.temp", "Thermal sensor"),
            MetricResult::metric_not_available(MetricAvailability::NotExposed),
        );
        let v2 = entry_view(&not_exposed);
        assert_eq!(v2.source_availability, Some(SourceAvailabilityView::Ready));
        assert_eq!(
            v2.metric_availability,
            Some(MetricAvailabilityView::NotExposed)
        );
        assert!(matches!(v2.outcome, OutcomeView::NoValue));

        // The two NoValue entries are distinguishable by their availability
        // fields, not collapsed into a shared status.
        assert_ne!(v1.source_availability, v2.source_availability);
        assert_ne!(v1.metric_availability, v2.metric_availability);
    }

    #[test]
    fn provenance_appears_only_with_values() {
        let ok = entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::ok(ok_sample(1, Provenance::OperatingSystemReported)),
        );
        let no_value = entry(
            descriptor("os.temp", "Thermal sensor"),
            MetricResult::metric_not_available(MetricAvailability::NotExposed),
        );
        let failed = entry(
            descriptor("os.net.errors", "Network errors"),
            MetricResult::failed(aer_core::AcquisitionError::Timeout),
        );
        assert!(matches!(entry_view(&ok).outcome, OutcomeView::Ok { .. }));
        assert!(matches!(
            entry_view(&no_value).outcome,
            OutcomeView::NoValue
        ));
        assert!(matches!(
            entry_view(&failed).outcome,
            OutcomeView::Failed { .. }
        ));
        // Provenance is structurally reachable only from the Ok variant.
        if let OutcomeView::Ok { provenance, .. } = &entry_view(&ok).outcome {
            assert_eq!(*provenance, ProvenanceView::OperatingSystemReported);
        }
    }

    #[test]
    fn snapshot_view_carries_timestamp_and_entries() {
        let s = snapshot(vec![entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::ok(ok_sample(1, Provenance::OperatingSystemReported)),
        )]);
        let v = snapshot_view(&s);
        assert_eq!(v.at, 1);
        assert_eq!(v.entries.len(), 1);
    }

    // --- Runtime status view (1C) ---------------------------------------------

    use crate::runtime::status::{RuntimeStatus, RuntimeType, RuntimeVersion, VersionProvenance};

    fn runtime_status(source: SourceAvailability, version: Option<&str>) -> RuntimeStatus {
        RuntimeStatus::new(
            RuntimeType::Ollama,
            source,
            version.map(|value| RuntimeVersion {
                value: value.to_string(),
                provenance: VersionProvenance::RuntimeReported,
            }),
        )
    }

    #[test]
    fn runtime_status_view_ready_with_version_carries_version_and_reporter() {
        let v = runtime_status_view(&runtime_status(SourceAvailability::Ready, Some("0.1.32")));
        assert_eq!(v.runtime_type, "ollama");
        assert_eq!(v.source_availability, SourceAvailabilityView::Ready);
        assert_eq!(v.version.as_deref(), Some("0.1.32"));
        assert_eq!(v.version_reported_by.as_deref(), Some("runtime_reported"));
        assert_eq!(v.status_label, "Ollama available");
        assert!(!v.interpretation.is_empty());
        assert!(!v.why_it_matters.is_empty());
    }

    #[test]
    fn runtime_status_view_ready_without_version_has_no_version_fields() {
        let v = runtime_status_view(&runtime_status(SourceAvailability::Ready, None));
        assert_eq!(v.source_availability, SourceAvailabilityView::Ready);
        assert!(v.version.is_none());
        assert!(v.version_reported_by.is_none());
        assert_eq!(v.status_label, "Ollama responded unexpectedly");
    }

    #[test]
    fn runtime_status_view_not_detected_has_no_version_fields() {
        let v = runtime_status_view(&runtime_status(SourceAvailability::NotDetected, None));
        assert_eq!(v.source_availability, SourceAvailabilityView::NotDetected);
        assert!(v.version.is_none());
        assert!(v.version_reported_by.is_none());
        assert_eq!(
            v.status_label,
            "Ollama was not detected at the local endpoint"
        );
    }

    #[test]
    fn runtime_status_view_unreachable_has_no_version_fields() {
        let v = runtime_status_view(&runtime_status(SourceAvailability::Unreachable, None));
        assert_eq!(v.source_availability, SourceAvailabilityView::Unreachable);
        assert!(v.version.is_none());
        assert!(v.version_reported_by.is_none());
        assert_eq!(v.status_label, "Ollama could not be reached");
    }

    #[test]
    fn runtime_status_view_serialises_to_controlled_wire_shape() {
        let v = runtime_status_view(&runtime_status(SourceAvailability::Ready, Some("0.1.32")));
        let json = serde_json::to_value(&v).expect("serialisable");
        assert_eq!(json["runtime_type"], "ollama");
        assert_eq!(json["source_availability"], "ready");
        assert_eq!(json["version"], "0.1.32");
        assert_eq!(json["version_reported_by"], "runtime_reported");
        assert_eq!(json["status_label"], "Ollama available");
        // No raw transport detail leaks into any text field.
        for field in ["status_label", "interpretation", "why_it_matters"] {
            let lower = json[field].as_str().unwrap().to_lowercase();
            assert!(!lower.contains("127.0.0.1"));
            assert!(!lower.contains("11434"));
            assert!(!lower.contains("http"));
        }
    }

    #[test]
    fn runtime_status_view_serialises_absent_version_as_null() {
        let v = runtime_status_view(&runtime_status(SourceAvailability::NotDetected, None));
        let json = serde_json::to_value(&v).expect("serialisable");
        assert!(json["version"].is_null());
        assert!(json["version_reported_by"].is_null());
        assert_eq!(json["source_availability"], "not_detected");
    }

    #[test]
    fn view_serialises_to_controlled_wire_shape() {
        // Pin the IPC wire shape the frontend will consume: the four state
        // concepts stay distinct fields, enums are tagged/lowercased, and a
        // failed outcome carries only a controlled category — no raw error.
        let ok = entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::ok(ok_sample(1024, Provenance::OperatingSystemReported)),
        );
        let unsupported = entry(
            descriptor("gpu.fan.rpm", "GPU fan speed"),
            MetricResult::unsupported(aer_core::LimitationCode::RequiresDriver),
        );
        let not_detected = entry(
            descriptor("rt.context.tokens", "AI runtime context tokens"),
            MetricResult::source_not_ready(SourceAvailability::NotDetected),
        );
        let failed = entry(
            descriptor("os.net.errors", "Network errors"),
            MetricResult::failed(aer_core::AcquisitionError::Timeout),
        );
        let v = snapshot_view(&snapshot(vec![ok, unsupported, not_detected, failed]));
        let json = serde_json::to_value(&v).expect("serialisable");

        let entries = json["entries"].as_array().expect("entries array");
        assert_eq!(entries.len(), 4);

        // Ok: provenance present, value present.
        let ok = &entries[0];
        assert_eq!(ok["display_name"], "Memory in use");
        assert_eq!(ok["unit"], "bytes");
        assert_eq!(ok["support"]["kind"], "supported");
        assert_eq!(ok["outcome"]["kind"], "ok");
        assert_eq!(ok["outcome"]["provenance"], "operating_system_reported");
        assert_eq!(ok["outcome"]["value"], 1024.0);

        // Unsupported: limitation code, no source availability, NoValue.
        let unsup = &entries[1];
        assert_eq!(unsup["support"]["kind"], "unsupported");
        assert_eq!(unsup["support"]["limitation_code"], "requires_driver");
        assert!(unsup["source_availability"].is_null());
        assert_eq!(unsup["outcome"]["kind"], "no_value");

        // Source not detected: source carries the reason, metric unevaluated.
        let nd = &entries[2];
        assert_eq!(nd["source_availability"], "not_detected");
        assert!(nd["metric_availability"].is_null());
        assert_eq!(nd["outcome"]["kind"], "no_value");

        // Failed: controlled category only, no message field, no provenance.
        let f = &entries[3];
        assert_eq!(f["outcome"]["kind"], "failed");
        assert_eq!(f["outcome"]["category"], "timeout");
        assert!(f["outcome"].get("message").is_none());
        assert!(f["outcome"].get("provenance").is_none());
    }

    // --- Model inventory view (1D) -------------------------------------------

    use crate::runtime::inventory::{
        InventoryState, ModelInventory, ModelLocality, RuntimeModelSummary,
    };

    fn summary(name: &str, size: Option<u64>) -> RuntimeModelSummary {
        RuntimeModelSummary {
            display_name: name.to_string(),
            locality: ModelLocality::Undetermined,
            size_bytes: size,
            parameter_size: Some("8.0B".to_string()),
            quantization: Some("Q4_0".to_string()),
            families: vec!["llama".to_string()],
        }
    }

    fn inventory(state: InventoryState, models: Vec<RuntimeModelSummary>) -> ModelInventory {
        let source_availability = match state {
            InventoryState::Available | InventoryState::RespondedUnexpectedly => {
                SourceAvailability::Ready
            }
            InventoryState::NotDetected => SourceAvailability::NotDetected,
            InventoryState::Unreachable => SourceAvailability::Unreachable,
        };
        ModelInventory {
            runtime_type: RuntimeType::Ollama,
            state,
            source_availability,
            models,
            skipped: 0,
        }
    }

    #[test]
    fn model_inventory_view_available_with_models_carries_entries_and_text() {
        let inv = inventory(
            InventoryState::Available,
            vec![summary("example-chat:8b", Some(4_975_683_088))],
        );
        let v = model_inventory_view(&inv);
        assert_eq!(v.runtime_type, "ollama");
        assert_eq!(v.state, InventoryStateView::Available);
        assert_eq!(v.source_availability, SourceAvailabilityView::Ready);
        assert_eq!(v.models.len(), 1);
        assert_eq!(v.skipped, 0);
        assert_eq!(v.models[0].display_name, "example-chat:8b");
        assert_eq!(v.models[0].size_bytes, Some(4_975_683_088));
        assert_eq!(v.state_label, "Ollama model catalogue");
        assert!(!v.interpretation.is_empty());
        assert!(!v.why_it_matters.is_empty());
    }

    #[test]
    fn model_inventory_view_available_empty_is_not_an_error() {
        let v = model_inventory_view(&inventory(InventoryState::Available, vec![]));
        assert_eq!(v.state, InventoryStateView::Available);
        assert!(v.models.is_empty());
        assert_eq!(v.state_label, "No models in the catalogue");
    }

    #[test]
    fn model_inventory_view_maps_each_state_and_mirrors_1c_wording() {
        let ru = model_inventory_view(&inventory(InventoryState::RespondedUnexpectedly, vec![]));
        assert_eq!(ru.state, InventoryStateView::RespondedUnexpectedly);
        assert_eq!(ru.state_label, "Ollama responded unexpectedly");

        let nd = model_inventory_view(&inventory(InventoryState::NotDetected, vec![]));
        assert_eq!(nd.state, InventoryStateView::NotDetected);
        assert_eq!(
            nd.state_label,
            "Ollama was not detected at the local endpoint"
        );

        let un = model_inventory_view(&inventory(InventoryState::Unreachable, vec![]));
        assert_eq!(un.state, InventoryStateView::Unreachable);
        assert_eq!(un.state_label, "Ollama could not be reached");
    }

    #[test]
    fn model_inventory_view_serialises_to_controlled_wire_shape() {
        let mut inv = inventory(
            InventoryState::Available,
            vec![summary("example-chat:8b", Some(4_975_683_088))],
        );
        // A second entry with no size to assert absence is null, not zero.
        inv.models.push(RuntimeModelSummary {
            display_name: "bare:1b".to_string(),
            locality: ModelLocality::Undetermined,
            size_bytes: None,
            parameter_size: None,
            quantization: None,
            families: Vec::new(),
        });
        let v = model_inventory_view(&inv);
        let json = serde_json::to_value(&v).expect("serialisable");

        assert_eq!(json["runtime_type"], "ollama");
        assert_eq!(json["state"], "available");
        assert_eq!(json["source_availability"], "ready");
        assert_eq!(json["skipped"], 0);
        assert_eq!(json["state_label"], "Ollama model catalogue");

        let models = json["models"].as_array().expect("models array");
        assert_eq!(models.len(), 2);
        assert_eq!(models[0]["display_name"], "example-chat:8b");
        assert_eq!(models[0]["size_bytes"], 4_975_683_088_u64);
        assert_eq!(models[0]["locality"], "undetermined");
        assert_eq!(models[0]["parameter_size"], "8.0B");
        assert_eq!(models[0]["quantization"], "Q4_0");
        // Missing size is null, not zero.
        assert!(models[1]["size_bytes"].is_null());
        assert_eq!(models[1]["locality"], "undetermined");

        // No private/transport detail leaks into any field.
        let flat = json.to_string();
        assert!(!flat.contains("digest"));
        assert!(!flat.contains("remote_host"));
        assert!(!flat.contains("remote_model"));
        assert!(!flat.contains("127.0.0.1"));
        assert!(!flat.contains("11434"));
        assert!(!flat.contains("http"));
        assert!(!flat.contains("endpoint"));
    }

    #[test]
    fn model_inventory_view_locality_is_undetermined_for_every_entry() {
        let inv = inventory(
            InventoryState::Available,
            vec![
                summary("thing:cloud", Some(0)),
                summary("thing:8b", Some(1)),
            ],
        );
        let v = model_inventory_view(&inv);
        for m in &v.models {
            assert_eq!(m.locality, ModelLocalityView::Undetermined);
        }
        let json = serde_json::to_value(&v).expect("serialisable");
        for m in json["models"].as_array().unwrap() {
            assert_eq!(m["locality"], "undetermined");
        }
    }

    // --- Loaded-models view (1E) ---------------------------------------------

    use crate::runtime::loaded_models::{LoadedModelSet, LoadedModelState, LoadedModelSummary};

    fn loaded_summary(
        name: &str,
        size: Option<u64>,
        vram: Option<u64>,
        ctx: Option<u64>,
        expires_at: Option<&str>,
    ) -> LoadedModelSummary {
        LoadedModelSummary {
            display_name: name.to_string(),
            reported_size_bytes: size,
            reported_vram_bytes: vram,
            reported_context_length: ctx,
            expires_at: expires_at.map(String::from),
        }
    }

    fn loaded_set(state: LoadedModelState, models: Vec<LoadedModelSummary>) -> LoadedModelSet {
        let source_availability = match state {
            LoadedModelState::Available | LoadedModelState::RespondedUnexpectedly => {
                SourceAvailability::Ready
            }
            LoadedModelState::NotDetected => SourceAvailability::NotDetected,
            LoadedModelState::Unreachable => SourceAvailability::Unreachable,
        };
        LoadedModelSet {
            runtime_type: RuntimeType::Ollama,
            state,
            source_availability,
            models,
            skipped: 0,
        }
    }

    #[test]
    fn loaded_models_view_available_populated_carries_entries_and_text() {
        let set = loaded_set(
            LoadedModelState::Available,
            vec![loaded_summary(
                "example-chat:8b",
                Some(4_000_000_000),
                Some(2_000_000_000),
                Some(8192),
                Some("2024-01-01T00:00:00.000Z"),
            )],
        );
        let v = loaded_models_view(&set);
        assert_eq!(v.runtime_type, "ollama");
        assert_eq!(v.state, LoadedModelStateView::Available);
        assert_eq!(v.source_availability, SourceAvailabilityView::Ready);
        assert_eq!(v.models.len(), 1);
        assert_eq!(v.skipped, 0);
        let m = &v.models[0];
        assert_eq!(m.display_name, "example-chat:8b");
        assert_eq!(m.reported_size_bytes, Some(4_000_000_000));
        assert_eq!(m.reported_vram_bytes, Some(2_000_000_000));
        assert_eq!(m.reported_context_length, Some(8192));
        assert_eq!(m.expires_at.as_deref(), Some("2024-01-01T00:00:00.000Z"));
        assert_eq!(v.state_label, "Currently loaded in Ollama");
        assert!(!v.interpretation.is_empty());
        assert!(!v.why_it_matters.is_empty());
    }

    #[test]
    fn loaded_models_view_available_empty_is_not_an_error() {
        let v = loaded_models_view(&loaded_set(LoadedModelState::Available, vec![]));
        assert_eq!(v.state, LoadedModelStateView::Available);
        assert!(v.models.is_empty());
        assert_eq!(v.state_label, "No models currently running in Ollama");
    }

    #[test]
    fn loaded_models_view_maps_each_state_and_mirrors_1c_1d_wording() {
        let ru = loaded_models_view(&loaded_set(LoadedModelState::RespondedUnexpectedly, vec![]));
        assert_eq!(ru.state, LoadedModelStateView::RespondedUnexpectedly);
        assert_eq!(ru.state_label, "Ollama responded unexpectedly");

        let nd = loaded_models_view(&loaded_set(LoadedModelState::NotDetected, vec![]));
        assert_eq!(nd.state, LoadedModelStateView::NotDetected);
        assert_eq!(
            nd.state_label,
            "Ollama was not detected at the local endpoint"
        );

        let un = loaded_models_view(&loaded_set(LoadedModelState::Unreachable, vec![]));
        assert_eq!(un.state, LoadedModelStateView::Unreachable);
        assert_eq!(un.state_label, "Ollama could not be reached");
    }

    #[test]
    fn loaded_models_view_serialises_to_controlled_wire_shape() {
        let mut set = loaded_set(
            LoadedModelState::Available,
            vec![loaded_summary(
                "example-chat:8b",
                Some(4_000_000_000),
                Some(2_000_000_000),
                Some(8192),
                Some("2024-01-01T00:00:00.000Z"),
            )],
        );
        // A second entry with no optional fields to assert absence is null.
        set.models
            .push(loaded_summary("bare:1b", None, None, None, None));
        let v = loaded_models_view(&set);
        let json = serde_json::to_value(&v).expect("serialisable");

        assert_eq!(json["runtime_type"], "ollama");
        assert_eq!(json["state"], "available");
        assert_eq!(json["source_availability"], "ready");
        assert_eq!(json["skipped"], 0);
        assert_eq!(json["state_label"], "Currently loaded in Ollama");

        let models = json["models"].as_array().expect("models array");
        assert_eq!(models.len(), 2);
        assert_eq!(models[0]["display_name"], "example-chat:8b");
        assert_eq!(models[0]["reported_size_bytes"], 4_000_000_000_u64);
        assert_eq!(models[0]["reported_vram_bytes"], 2_000_000_000_u64);
        assert_eq!(models[0]["reported_context_length"], 8192);
        assert_eq!(models[0]["expires_at"], "2024-01-01T00:00:00.000Z");
        // Missing optional fields are null, not zero.
        assert!(models[1]["reported_size_bytes"].is_null());
        assert!(models[1]["reported_vram_bytes"].is_null());
        assert!(models[1]["reported_context_length"].is_null());
        assert!(models[1]["expires_at"].is_null());

        // No private/transport/detail leaks into any field. `expires_at` is a
        // documented, carried string (not a digest/host/endpoint).
        let flat = json.to_string();
        assert!(!flat.contains("digest"));
        assert!(!flat.contains("details"));
        assert!(!flat.contains("remote_host"));
        assert!(!flat.contains("processor"));
        assert!(!flat.contains("capabilities"));
        assert!(!flat.contains("127.0.0.1"));
        assert!(!flat.contains("11434"));
        assert!(!flat.contains("http"));
        assert!(!flat.contains("endpoint"));
    }

    #[test]
    fn loaded_models_view_does_not_claim_install_state_or_overclaim() {
        for state in [
            LoadedModelState::Available,
            LoadedModelState::RespondedUnexpectedly,
            LoadedModelState::NotDetected,
            LoadedModelState::Unreachable,
        ] {
            let set = loaded_set(
                state,
                if matches!(state, LoadedModelState::Available) {
                    vec![loaded_summary("example:1b", Some(1), None, None, None)]
                } else {
                    vec![]
                },
            );
            let v = loaded_models_view(&set);
            for field in [
                v.state_label.as_str(),
                v.interpretation.as_str(),
                v.why_it_matters.as_str(),
            ] {
                let lower = field.to_lowercase();
                assert!(
                    !lower.contains("not installed"),
                    "no install claim: {field:?}"
                );
                assert!(!lower.contains("health"), "no health claim: {field:?}");
                assert!(!lower.contains("compat"), "no compat claim: {field:?}");
                assert!(!lower.contains("secure"), "no security claim: {field:?}");
                assert!(
                    !lower.contains("perform"),
                    "no performance claim: {field:?}"
                );
                // No raw transport/endpoint detail in the controlled text.
                assert!(!lower.contains("127.0.0.1"), "no endpoint IP: {field:?}");
                assert!(!lower.contains("11434"), "no endpoint port: {field:?}");
                assert!(!lower.contains("http"), "no scheme: {field:?}");
                assert!(!lower.contains("refus"), "no transport word: {field:?}");
            }
        }
    }

    #[test]
    fn loaded_models_view_states_have_distinct_labels() {
        let labels = [
            loaded_models_view(&loaded_set(
                LoadedModelState::Available,
                vec![loaded_summary("x:1b", Some(1), None, None, None)],
            ))
            .state_label,
            loaded_models_view(&loaded_set(LoadedModelState::Available, vec![])).state_label,
            loaded_models_view(&loaded_set(LoadedModelState::RespondedUnexpectedly, vec![]))
                .state_label,
            loaded_models_view(&loaded_set(LoadedModelState::NotDetected, vec![])).state_label,
            loaded_models_view(&loaded_set(LoadedModelState::Unreachable, vec![])).state_label,
        ];
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j], "labels {i} and {j} collide");
            }
        }
    }

    // --- Resource context composition (Milestone 1F) -----------------------
    //
    // `compose_resource_context` is PURE: it derives `ram_has_value` from the
    // available-memory entry (located by its controlled display name and its
    // `Ok` outcome) and `loaded_is_available` from `loaded.state`, then delegates
    // to the pure `resource_context_text`. The exact §9 strings are owned by the
    // `ollama.rs` tests; these tests own selection/delegation, the `ram_has_value`
    // derivation rules, purity, no-aggregate/no-model-names, and the wire-shape
    // round-trip that proves the additive `Deserialize` derives are non-behavioral.

    use crate::runtime::ollama::resource_context_text as resource_text;

    /// The expected `ResourceContextView` for the two derived booleans, mirroring
    /// `compose_resource_context`'s delegation to `resource_context_text`.
    fn expected_rc(ram_has_value: bool, loaded_is_available: bool) -> ResourceContextView {
        let t = resource_text(ram_has_value, loaded_is_available);
        ResourceContextView {
            interpretation: t.interpretation,
            why_it_matters: t.why_it_matters,
        }
    }

    /// A snapshot view whose only entry is the available-memory metric with the
    /// given outcome. The descriptor display name is the controlled
    /// "Available memory".
    fn ram_snapshot(result: MetricResult) -> SnapshotView {
        snapshot_view(&snapshot(vec![entry(
            descriptor("os.ram.available", AVAILABLE_MEMORY_DISPLAY_NAME),
            result,
        )]))
    }

    fn ram_ok() -> SnapshotView {
        ram_snapshot(MetricResult::ok(ok_sample(
            1,
            Provenance::OperatingSystemReported,
        )))
    }

    fn loaded_view(state: LoadedModelState, models: Vec<LoadedModelSummary>) -> LoadedModelSetView {
        loaded_models_view(&loaded_set(state, models))
    }

    fn loaded_available_populated() -> LoadedModelSetView {
        loaded_view(
            LoadedModelState::Available,
            vec![loaded_summary(
                "example-runner:8b",
                Some(5_000_000_000),
                None,
                None,
                None,
            )],
        )
    }

    #[test]
    fn compose_variant_1_ram_available_loaded_available_populated() {
        let rc = compose_resource_context(&ram_ok(), &loaded_available_populated());
        assert_eq!(rc, expected_rc(true, true));
        // Source distinction and non-attribution are present in variant 1.
        let combined = format!("{} {}", rc.interpretation, rc.why_it_matters).to_lowercase();
        assert!(combined.contains("operating system"));
        assert!(combined.contains("define it differently"));
        assert!(combined.contains("ollama"));
        assert!(combined.contains("not be added to or subtracted from"));
        // The approved context-length line is present in variant 1 only.
        assert!(rc
            .why_it_matters
            .contains("A larger configured context can require more memory."));
    }

    #[test]
    fn compose_variant_1_ram_available_loaded_available_empty_is_same_as_populated() {
        // A valid empty loaded list is still Available, so it must use the SAME
        // resource-context variant as a populated Available list (not an error,
        // not a separate variant).
        let empty = loaded_view(LoadedModelState::Available, Vec::new());
        let populated_rc = compose_resource_context(&ram_ok(), &loaded_available_populated());
        let empty_rc = compose_resource_context(&ram_ok(), &empty);
        assert_eq!(empty_rc, expected_rc(true, true));
        assert_eq!(empty_rc, populated_rc);
    }

    #[test]
    fn compose_variant_2_ram_available_loaded_unavailable() {
        for state in [
            LoadedModelState::NotDetected,
            LoadedModelState::Unreachable,
            LoadedModelState::RespondedUnexpectedly,
        ] {
            let rc = compose_resource_context(&ram_ok(), &loaded_view(state, Vec::new()));
            assert_eq!(
                rc,
                expected_rc(true, false),
                "variant 2 for loaded state {state:?}"
            );
            // No context-length line outside variant 1.
            assert!(!rc.why_it_matters.contains("A larger configured context"));
        }
    }

    #[test]
    fn compose_variant_3_ram_unavailable_loaded_available() {
        // NoValue / Failed / Unsupported outcomes all yield ram_has_value == false.
        let cases = [
            ram_snapshot(MetricResult::metric_not_available(
                MetricAvailability::NotExposed,
            )),
            ram_snapshot(MetricResult::failed(aer_core::AcquisitionError::Timeout)),
            ram_snapshot(MetricResult::unsupported(
                aer_core::LimitationCode::KernelEstimate,
            )),
        ];
        for snap in cases {
            let rc = compose_resource_context(&snap, &loaded_available_populated());
            assert_eq!(rc, expected_rc(false, true));
            assert!(!rc.why_it_matters.contains("A larger configured context"));
        }
    }

    #[test]
    fn compose_variant_3_when_available_memory_entry_is_absent() {
        // A snapshot with no available-memory entry (e.g. non-Linux empty snapshot)
        // yields ram_has_value == false.
        let empty_snapshot = snapshot_view(&snapshot(Vec::new()));
        let rc = compose_resource_context(&empty_snapshot, &loaded_available_populated());
        assert_eq!(rc, expected_rc(false, true));
    }

    #[test]
    fn compose_variant_3_when_a_different_metric_has_an_ok_outcome() {
        // The match is name-specific: an Ok entry whose display name is NOT
        // "Available memory" must NOT satisfy ram_has_value.
        let snap = snapshot_view(&snapshot(vec![entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::ok(ok_sample(1, Provenance::OperatingSystemReported)),
        )]));
        let rc = compose_resource_context(&snap, &loaded_available_populated());
        assert_eq!(rc, expected_rc(false, true));
    }

    #[test]
    fn compose_variant_4_both_unavailable() {
        let rc = compose_resource_context(
            &ram_snapshot(MetricResult::failed(aer_core::AcquisitionError::Timeout)),
            &loaded_view(LoadedModelState::NotDetected, Vec::new()),
        );
        assert_eq!(rc, expected_rc(false, false));
        assert!(!rc.why_it_matters.contains("A larger configured context"));
    }

    #[test]
    fn compose_ok_outcome_yields_ram_has_value_true() {
        // The Ok outcome (the only one that counts) maps to variant 1 with loaded
        // available; a NoValue with the same loaded would map to variant 3.
        assert_eq!(
            compose_resource_context(&ram_ok(), &loaded_available_populated()),
            expected_rc(true, true)
        );
        assert_eq!(
            compose_resource_context(
                &ram_snapshot(MetricResult::metric_not_available(
                    MetricAvailability::NotExposed
                )),
                &loaded_available_populated()
            ),
            expected_rc(false, true)
        );
    }

    #[test]
    fn compose_does_not_mutate_its_inputs() {
        let snap = ram_ok();
        let loaded = loaded_available_populated();
        let snap_before = snap.clone();
        let loaded_before = loaded.clone();
        let _ = compose_resource_context(&snap, &loaded);
        assert_eq!(snap, snap_before, "snapshot view unchanged after compose");
        assert_eq!(loaded, loaded_before, "loaded view unchanged after compose");
    }

    #[test]
    fn compose_is_pure_same_inputs_yield_same_output() {
        let snap = ram_ok();
        let loaded = loaded_available_populated();
        let a = compose_resource_context(&snap, &loaded);
        let b = compose_resource_context(&snap, &loaded);
        assert_eq!(a, b);
    }

    #[test]
    fn resource_context_view_has_no_aggregate_field_and_no_model_names() {
        // The DTO carries ONLY interpretation/why_it_matters. No aggregate, no
        // model name, no reported size leaks into the composed explanation.
        let rc = compose_resource_context(&ram_ok(), &loaded_available_populated());
        let json = serde_json::to_value(&rc).expect("serialisable");
        let obj = json.as_object().expect("object");
        assert_eq!(obj.len(), 2, "exactly two fields");
        assert!(obj.contains_key("interpretation"));
        assert!(obj.contains_key("why_it_matters"));
        let flat = json.to_string();
        assert!(!flat.contains("example-runner"), "no model name");
        assert!(!flat.contains("reported_size_bytes"), "no size field");
        assert!(!flat.contains("models"), "no models field");
        assert!(!flat.contains("aggregate"), "no aggregate");
    }

    #[test]
    fn resource_context_view_makes_no_forbidden_claim_and_leaks_no_raw_detail() {
        // End-to-end check on the composed DTO: whole-word forbidden claims are
        // tokenised (so "Loaded" is not flagged as the verb "load"); forbidden
        // phrases/symbols are checked as substrings; raw transport detail must
        // not leak.
        fn words(text: &str) -> Vec<String> {
            text.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        }
        for (ram, loaded_state) in [
            (true, LoadedModelState::Available),
            (true, LoadedModelState::NotDetected),
            (false, LoadedModelState::Available),
            (false, LoadedModelState::NotDetected),
        ] {
            let snap = if ram {
                ram_ok()
            } else {
                ram_snapshot(MetricResult::failed(aer_core::AcquisitionError::Timeout))
            };
            let rc = compose_resource_context(&snap, &loaded_view(loaded_state, Vec::new()));
            let combined = format!("{} {}", rc.interpretation, rc.why_it_matters);
            let lower = combined.to_lowercase();
            let tokens = words(&combined);
            for forbidden in [
                "fit",
                "headroom",
                "health",
                "low",
                "dangerous",
                "sufficient",
                "insufficient",
                "good",
                "bad",
                "safe",
                "performance",
                "fast",
                "slow",
                "total",
                "recommend",
                "unload",
                "load",
            ] {
                assert!(
                    !tokens.iter().any(|t| t.as_str() == forbidden),
                    "forbidden word {forbidden:?} in variant ({ram},{loaded_state:?}): {combined:?}"
                );
            }
            for forbidden in ["%", "used by ollama", "percent of ram"] {
                assert!(
                    !lower.contains(forbidden),
                    "forbidden phrase {forbidden:?} in variant ({ram},{loaded_state:?}): {combined:?}"
                );
            }
            for leak in ["digest", "127.0.0.1", "11434", "http", "endpoint", "host"] {
                assert!(
                    !lower.contains(leak),
                    "raw leakage {leak:?} in variant ({ram},{loaded_state:?})"
                );
            }
        }
    }

    #[test]
    fn loaded_model_state_view_maps_unchanged_through_compose() {
        // LoadedModelStateView::Available (populated or empty) drives the variant;
        // the state itself is not mutated or reinterpreted by compose.
        let loaded = loaded_view(LoadedModelState::Available, Vec::new());
        assert_eq!(loaded.state, LoadedModelStateView::Available);
        let rc = compose_resource_context(&ram_ok(), &loaded);
        assert_eq!(rc, expected_rc(true, true));
        assert_eq!(
            loaded.state,
            LoadedModelStateView::Available,
            "state unchanged"
        );
    }

    #[test]
    fn available_memory_display_name_constant_is_the_controlled_value() {
        assert_eq!(AVAILABLE_MEMORY_DISPLAY_NAME, "Available memory");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn available_memory_display_name_matches_linux_descriptor() {
        // Pin the stringly match against the existing 1B descriptor so the
        // constant stays in sync with the provider's actual display name.
        use crate::platform::linux::LinuxMetricsProvider;
        use aer_core::OsMetricsProvider;
        let provider = LinuxMetricsProvider::new();
        let descriptors = provider.list_metrics();
        assert!(
            !descriptors.is_empty(),
            "Linux provider lists os.ram.available"
        );
        assert_eq!(
            descriptors[0].display_name, AVAILABLE_MEMORY_DISPLAY_NAME,
            "constant matches the 1B descriptor display name"
        );
    }

    #[test]
    fn snapshot_view_round_trips_through_deserialize_unchanged() {
        // The additive `Deserialize` derive is non-behavioral: the wire shape
        // `Serialize` produces is exactly the shape `Deserialize` reads back.
        let snap = snapshot_view(&snapshot(vec![
            entry(
                descriptor("os.ram.available", AVAILABLE_MEMORY_DISPLAY_NAME),
                MetricResult::ok(ok_sample(1024, Provenance::OperatingSystemReported)),
            ),
            entry(
                descriptor("os.net.errors", "Network errors"),
                MetricResult::failed(aer_core::AcquisitionError::Timeout),
            ),
            entry(
                descriptor("gpu.fan.rpm", "GPU fan speed"),
                MetricResult::unsupported(aer_core::LimitationCode::RequiresDriver),
            ),
        ]));
        let json = serde_json::to_value(&snap).expect("serialisable");
        let back: SnapshotView = serde_json::from_value(json).expect("deserialisable");
        assert_eq!(snap, back, "wire shape round-trips unchanged");
    }

    #[test]
    fn loaded_model_set_view_round_trips_through_deserialize_unchanged() {
        let set = loaded_models_view(&loaded_set(
            LoadedModelState::Available,
            vec![loaded_summary(
                "example-runner:8b",
                Some(5_000_000_000),
                Some(1),
                Some(8192),
                None,
            )],
        ));
        let json = serde_json::to_value(&set).expect("serialisable");
        let back: LoadedModelSetView = serde_json::from_value(json).expect("deserialisable");
        assert_eq!(set, back, "wire shape round-trips unchanged");
    }

    // --- Observed inference view (1G) ---------------------------------------

    use crate::runtime::inference::{
        InferenceObservation, InferenceObservationState, DIAGNOSTIC_PROFILE,
        EXECUTION_BOUNDARY_UNDETERMINED,
    };

    fn observation(state: InferenceObservationState, model: Option<&str>) -> InferenceObservation {
        InferenceObservation {
            state,
            model: model.map(|s| s.to_string()),
            diagnostic_profile: DIAGNOSTIC_PROFILE,
            execution_boundary: EXECUTION_BOUNDARY_UNDETERMINED,
            prompt_eval_count: None,
            eval_count: None,
            load_duration_ns: None,
            prompt_eval_duration_ns: None,
            eval_duration_ns: None,
            total_duration_ns: None,
            generation_tokens_per_second: None,
        }
    }

    fn completed_observation() -> InferenceObservation {
        InferenceObservation {
            state: InferenceObservationState::Completed,
            model: Some("example:1b".to_string()),
            diagnostic_profile: DIAGNOSTIC_PROFILE,
            execution_boundary: EXECUTION_BOUNDARY_UNDETERMINED,
            prompt_eval_count: Some(12),
            eval_count: Some(8),
            load_duration_ns: Some(500_000_000),
            prompt_eval_duration_ns: Some(300_000_000),
            eval_duration_ns: Some(1_000_000_000),
            total_duration_ns: Some(1_800_000_000),
            generation_tokens_per_second: Some(8.0),
        }
    }

    #[test]
    fn inference_observation_view_completed_carries_metrics_text_and_undetermined() {
        let v = inference_observation_view(&completed_observation());
        assert_eq!(v.state, InferenceObservationState::Completed);
        assert_eq!(v.model.as_deref(), Some("example:1b"));
        assert_eq!(v.diagnostic_profile.as_deref(), Some("diagnostic-basic-v1"));
        assert_eq!(v.execution_boundary, "undetermined");
        assert_eq!(v.prompt_eval_count, Some(12));
        assert_eq!(v.eval_count, Some(8));
        assert_eq!(v.load_duration_ns, Some(500_000_000));
        assert_eq!(v.prompt_eval_duration_ns, Some(300_000_000));
        assert_eq!(v.eval_duration_ns, Some(1_000_000_000));
        assert_eq!(v.total_duration_ns, Some(1_800_000_000));
        assert!((v.generation_tokens_per_second.unwrap() - 8.0).abs() < f64::EPSILON);
        assert!(!v.interpretation.is_empty());
        assert!(!v.why_it_matters.is_empty());
        assert_eq!(v.limitations, "Execution location not determined.");
    }

    #[test]
    fn inference_observation_view_no_model_has_no_model_and_no_profile() {
        let v = inference_observation_view(&observation(
            InferenceObservationState::NoModelSelected,
            None,
        ));
        assert!(v.model.is_none());
        assert!(
            v.diagnostic_profile.is_none(),
            "no diagnostic profile when no model selected"
        );
        assert_eq!(v.execution_boundary, "undetermined");
        assert!(v.prompt_eval_count.is_none());
        assert!(v.generation_tokens_per_second.is_none());
    }

    #[test]
    fn inference_observation_view_blocked_carries_model_profile_and_undetermined() {
        let v = inference_observation_view(&observation(
            InferenceObservationState::BlockedByExecutionBoundary,
            Some("example:1b"),
        ));
        assert_eq!(
            v.state,
            InferenceObservationState::BlockedByExecutionBoundary
        );
        assert_eq!(v.model.as_deref(), Some("example:1b"));
        assert_eq!(v.diagnostic_profile.as_deref(), Some("diagnostic-basic-v1"));
        assert_eq!(v.execution_boundary, "undetermined");
        assert!(v.eval_count.is_none());
    }

    #[test]
    fn inference_observation_view_serialises_to_controlled_wire_shape() {
        let v = inference_observation_view(&completed_observation());
        let json = serde_json::to_value(&v).expect("serialisable");
        assert_eq!(json["state"], "completed");
        assert_eq!(json["model"], "example:1b");
        assert_eq!(json["diagnostic_profile"], "diagnostic-basic-v1");
        assert_eq!(json["execution_boundary"], "undetermined");
        assert_eq!(json["eval_count"], 8);
        assert_eq!(json["eval_duration_ns"], 1_000_000_000_u64);
        assert_eq!(json["total_duration_ns"], 1_800_000_000_u64);
        assert_eq!(json["generation_tokens_per_second"], 8.0);
        assert!(json["interpretation"].is_string());
        assert!(json["why_it_matters"].is_string());
        assert_eq!(json["limitations"], "Execution location not determined.");

        // No generated-response field and no raw transport detail leak into any
        // field. (Text-wording claims — no "local"/positive-benchmark language —
        // are covered by `inference_observation_view_makes_no_forbidden_claim_in_text`;
        // the Completed text legitimately says "not a general benchmark" as a
        // disclaimer, so "benchmark" is not a forbidden substring here.)
        let flat = json.to_string();
        assert!(!flat.contains("response"), "no generated-response field");
        assert!(!flat.contains("generated"), "no generated-output field");
        assert!(!flat.contains("127.0.0.1"));
        assert!(!flat.contains("11434"));
        assert!(!flat.contains("http"));
        assert!(!flat.contains("endpoint"));
    }

    #[test]
    fn inference_observation_view_makes_no_forbidden_claim_in_text() {
        for state in [
            InferenceObservationState::NoModelSelected,
            InferenceObservationState::BlockedByExecutionBoundary,
            InferenceObservationState::Running,
            InferenceObservationState::Completed,
            InferenceObservationState::NotDetected,
            InferenceObservationState::Unreachable,
            InferenceObservationState::TimedOut,
            InferenceObservationState::RespondedUnexpectedly,
            InferenceObservationState::ModelUnavailable,
        ] {
            let obs = observation(state, Some("example:1b"));
            let v = inference_observation_view(&obs);
            for field in [
                v.interpretation.as_str(),
                v.why_it_matters.as_str(),
                v.limitations.as_str(),
            ] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("local"), "no locality claim: {field:?}");
                assert!(!lower.contains("verified"), "no verified claim: {field:?}");
                assert!(!lower.contains("install"), "no install claim: {field:?}");
                assert!(!lower.contains("health"), "no health claim: {field:?}");
                assert!(!lower.contains("secure"), "no security claim: {field:?}");
                assert!(
                    !lower.contains("perform"),
                    "no performance claim: {field:?}"
                );
                assert!(!lower.contains("127.0.0.1"), "no endpoint IP: {field:?}");
                assert!(!lower.contains("11434"), "no endpoint port: {field:?}");
                assert!(!lower.contains("http"), "no scheme: {field:?}");
            }
        }
    }

    #[test]
    fn inference_observation_request_round_trips_through_deserialize_unchanged() {
        let request = InferenceObservationRequest {
            provider: "ollama".to_string(),
            model: "example:1b".to_string(),
            execution_boundary_acknowledged: true,
            provider_behavior_acknowledged: false,
        };
        let json = serde_json::to_value(&request).expect("serialisable");
        let back: InferenceObservationRequest =
            serde_json::from_value(json).expect("deserialisable");
        assert_eq!(request, back, "request wire shape round-trips unchanged");
        assert_eq!(back.model, "example:1b");
        assert!(back.execution_boundary_acknowledged);
    }

    #[test]
    fn inference_observation_request_false_acknowledgement_round_trips() {
        let request = InferenceObservationRequest {
            provider: "ollama".to_string(),
            model: "example:1b".to_string(),
            execution_boundary_acknowledged: false,
            provider_behavior_acknowledged: false,
        };
        let json = serde_json::to_value(&request).expect("serialisable");
        let back: InferenceObservationRequest =
            serde_json::from_value(json).expect("deserialisable");
        assert_eq!(request, back);
        assert!(!back.execution_boundary_acknowledged);
    }
}
