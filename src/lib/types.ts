// Frontend view/IPC types mirroring the Stage 8 Tauri view DTO wire shape.
//
// These describe only what crosses the data-source boundary. The four state
// concepts — support, source availability, metric availability, and the
// outcome — are kept as distinct fields, never collapsed into one status.
// Provenance appears only on the `ok` outcome, where a value exists. A failed
// outcome carries only a controlled category code, never a raw error string.
// Internal metric ids, filesystem paths, environment data, and raw diagnostics
// are not represented here. Milestone 1T adds a separate bounded controlled
// diagnosis input/output DTO without changing metric or Report DTOs.

/** Physical unit of a metric. Controlled lowercase string. */
export type Unit = "bytes" | "count" | "percent" | "seconds";

/** Source availability, when evaluated; `null` when unevaluated. */
export type SourceAvailability =
  "ready" | "not_detected" | "not_running" | "unreachable";

/** Metric availability, when evaluated; `null` when the source is not ready. */
export type MetricAvailability =
  "available" | "not_exposed" | "transiently_unavailable" | "not_applicable";

/** The six provenance categories, payload-free. */
export type Provenance =
  | "operating_system_reported"
  | "driver_reported"
  | "runtime_reported"
  | "application_measured"
  | "calculated"
  | "estimated";

/** Support state. `unsupported` carries a controlled limitation code. */
export type Support =
  { kind: "supported" } | { kind: "unsupported"; limitation_code: string };

/** Acquisition outcome. `ok` carries the value, unit, and provenance. `failed`
 * carries only a controlled category code. `no_value` carries nothing. */
export type Outcome =
  | { kind: "ok"; value: number; unit: Unit; provenance: Provenance }
  | { kind: "no_value" }
  | { kind: "failed"; category: string };

/** One metric's view projection: identity, the four distinct state concepts,
 * the outcome, and controlled explanation text. */
export interface SnapshotEntryView {
  display_name: string;
  unit: Unit;
  support: Support;
  source_availability: SourceAvailability | null;
  metric_availability: MetricAvailability | null;
  outcome: Outcome;
  interpretation: string;
  why_it_matters: string;
  deeper: string | null;
}

/** A snapshot view: a timestamp (ms since epoch) and the projected entries. */
export interface SnapshotView {
  at: number;
  entries: SnapshotEntryView[];
}

export interface MachineContextView {
  state: "available" | "not_exposed" | "failed";
  native_cpu_architecture: "x86_64" | "x86" | "arm64" | "arm" | "unknown" | null;
  interpretation: string;
  why_it_matters: string;
  qualification: string;
}

/** The controlled IPC projection of a runtime's detection status (Milestone 1C).
 * The runtime version is metadata, not a metric, so it travels through its own
 * DTO rather than the snapshot/report path. The four intended user-facing
 * states are distinguished by `(source_availability, version !== null)`. The
 * frontend renders the pre-computed controlled text fields verbatim and does
 * not infer detection semantics from availability, status codes, or transport
 * kinds. No raw errors, URLs, hostnames, ports, or status codes are present. */
export interface RuntimeStatusView {
  /** Controlled lowercase runtime identity (e.g. `"ollama"`). */
  runtime_type: string;
  source_availability: SourceAvailability;
  /** The version string the runtime reported, verbatim, or `null`. */
  version: string | null;
  /** How the version (when present) was obtained: `"runtime_reported"`, or
   * `null` when there is no version. */
  version_reported_by: string | null;
  status_label: string;
  interpretation: string;
  why_it_matters: string;
}

/** Evidence-based locality of a catalogue entry. Controlled snake_case string.
 * Always `"undetermined"` in 1D; `"local"` is reserved and not assigned. Carried
 * for forward use; not displayed per model in 1D's UI. */
export type ModelLocality = "local" | "undetermined";

/** The inventory fetch state, controlled snake_case. `available` with an empty
 * `models` array is a normal state, not an error. */
export type InventoryState =
  "available" | "responded_unexpectedly" | "not_detected" | "unreachable";

/** One model's inventory projection: display name, locality, reported size, and
 * the optional secondary labels. No `digest`, endpoint, host, port,
 * `remote_host`, or raw error crosses this boundary. */
export interface RuntimeModelSummaryView {
  display_name: string;
  locality: ModelLocality;
  /** Reported model size in bytes; `null` when absent/invalid (not zero). */
  size_bytes: number | null;
  parameter_size: string | null;
  quantization: string | null;
  /** `details.families` preferred; `details.family` fallback. Held for forward
   * use; rendered as a secondary label when present. */
  families: string[];
}

/** The controlled IPC projection of a runtime's model inventory (Milestone 1D).
 * The inventory is runtime metadata (the catalogue a runtime reports), not a
 * metric, so it travels through its own DTO rather than the snapshot/report
 * boundary. The frontend renders the pre-computed controlled text fields
 * verbatim and does not infer inventory state from availability, status codes,
 * or transport kinds. No raw errors, URLs, hostnames, ports, status codes,
 * `digest`, or `remote_host` are present. Locality is `"undetermined"` for
 * every entry in 1D and is not displayed per model. */
export interface ModelInventoryView {
  /** Controlled lowercase runtime identity (e.g. `"ollama"`). */
  runtime_type: string;
  state: InventoryState;
  source_availability: SourceAvailability;
  models: RuntimeModelSummaryView[];
  /** Entries dropped due to malformed individual model JSON. */
  skipped: number;
  state_label: string;
  interpretation: string;
  why_it_matters: string;
}

/** The loaded-model fetch state, controlled snake_case. Distinct from
 * `InventoryState` because 1E (currently loaded) and 1D (catalogue) answer
 * different questions. `available` with an empty `models` array is a normal
 * state ("no models are currently running"), not an error. */
export type LoadedModelState =
  "available" | "responded_unexpectedly" | "not_detected" | "unreachable";

/** One loaded (runtime-resident) model's projection: display name and the
 * sizes/counts Ollama reports for the loaded instance. `reported_*` fields are
 * what Ollama reports for the loaded model, not independently-measured physical
 * quantities. `expires_at` is carried internally but is NOT displayed (no
 * countdown, no date/time dependency). No `digest`, endpoint, host, port,
 * `remote_host`, or raw error crosses this boundary. */
export interface LoadedModelSummaryView {
  display_name: string;
  /** Loaded size Ollama reports (not exact RAM/disk); `null` when absent. */
  reported_size_bytes: number | null;
  /** VRAM size Ollama reports (not independently-measured physical VRAM);
   * `null` when absent. */
  reported_vram_bytes: number | null;
  /** Context length Ollama reports (not a theoretical max/performance figure);
   * `null` when absent. */
  reported_context_length: number | null;
  /**
   * Carried internally, NOT displayed. No countdown; no date/time dependency.
   */
  expires_at: string | null;
}

/** The controlled IPC projection of a runtime's currently loaded
 * (runtime-resident) models (Milestone 1E). Distinct from `ModelInventoryView`
 * (1D): this answers "what is running now?", not "what is in the catalogue?".
 * The loaded-model set is runtime metadata, not a metric, so it travels through
 * its own DTO rather than the snapshot/report boundary. The frontend renders the
 * pre-computed controlled text fields verbatim and does not infer loaded-model
 * state from availability, status codes, or transport kinds. No raw errors,
 * URLs, hostnames, ports, status codes, `digest`, or `remote_host` are present.
 * `expires_at` is carried but not displayed. No aggregate total is computed. */
export interface LoadedModelSetView {
  /** Controlled lowercase runtime identity (e.g. `"ollama"`). */
  runtime_type: string;
  state: LoadedModelState;
  source_availability: SourceAvailability;
  models: LoadedModelSummaryView[];
  /** Entries dropped due to malformed individual model JSON. */
  skipped: number;
  state_label: string;
  interpretation: string;
  why_it_matters: string;
  resource_interpretation: string;
  resource_qualification: string;
}

/** Controlled IPC projection of a resource-context composition (Milestones 1F
 * and 1V). Carries controlled explanation and nonnumeric concept states. The
 * frontend passes the
 * already-acquired `SnapshotView` and `LoadedModelSetView` to
 * `composeResourceContext` and renders the returned `interpretation`/
 * `why_it_matters` and concept text verbatim. It does not combine values, compute ratios, or pick
 * the variant. No aggregate, ratio, fit, or health claim. No raw endpoint/host/
 * digest. The command performs no I/O — it composes from the two views the
 * frontend already holds, so it reads no `/proc/meminfo` and calls no `/api/ps`.
 * The OS-reported Available memory and the Ollama-reported loaded-model state
 * are juxtaposed as context only; Engine Room does not attribute exact RAM/VRAM
 * consumption to a model. */
export interface ResourceContextView {
  interpretation: string;
  why_it_matters: string;
  concepts: ResourceConceptView[];
}

export interface ResourceConceptView {
  concept: string;
  state_label: string;
  interpretation: string;
}

/** The per-run authorization request for one observed inference (Milestone 1G).
 * Carries the explicitly selected model identifier and the execution-boundary
 * acknowledgement. `execution_boundary_acknowledged === false` blocks the run
 * with no POST; `true` authorizes one synthetic observation and is NOT locality
 * proof and NOT cryptographic. Request-scoped: the backend keeps no
 * global/persistent auth state, no allow-list, no session token, and no
 * "Don't ask again". */
export interface InferenceObservationRequest {
  provider?: RuntimeProvider;
  /** The model to run the fixed diagnostic prompt against, selected from the 1D
   * catalogue. The backend validates it defensively and against the current
   * catalogue. */
  model: string;
  /** Whether the user acknowledged the undetermined execution boundary for this
   * one run. `false` → `blocked_by_execution_boundary`, no POST. */
  execution_boundary_acknowledged: boolean;
  provider_behavior_acknowledged?: boolean;
}

/** The controlled result state of an observed-inference run, serialised as a
 * controlled snake_case string. The nine values are the complete 1G result
 * model. `no_model_selected`, `blocked_by_execution_boundary`, `running`, and
 * `model_unavailable` are produced by the command-layer orchestration (no model
 * / no authorization / already running / model off the current catalogue).
 * `completed`, `unreachable`, `timed_out`, and `responded_unexpectedly` are
 * produced from a generate outcome. The UI renders the controlled text
 * verbatim and never infers locality, benchmark, or quality from the state. */
export type InferenceObservationState =
  | "no_model_selected"
  | "blocked_by_execution_boundary"
  | "running"
  | "completed"
  | "not_detected"
  | "unreachable"
  | "timed_out"
  | "responded_unexpectedly"
  | "model_unavailable";

/** The controlled IPC projection of one observed-inference run (Milestone 1G).
 * This is runtime metadata (the counts/durations Ollama reported for one run,
 * plus an Engine-Room-calculated rate), not a metric, so it travels through its
 * own DTO rather than the snapshot/report boundary. The frontend renders the
 * pre-computed controlled text fields verbatim and does not infer locality,
 * benchmark, or quality from the counts.
 *
 * No raw errors, URLs, hostnames, ports, status codes, **generated response
 * text**, transport kinds, or locality claims are present. Durations are carried
 * in nanoseconds (Ollama's unit); the frontend renders them as seconds.
 * `execution_boundary` is always `"undetermined"` in 1G — the only value, no
 * enum, no Local/Cloud variants. `generation_tokens_per_second` is
 * Engine-Room-calculated (not Ollama-reported); the other counts/durations are
 * Ollama-reported. `diagnostic_profile` is present only when a model was
 * selected. The fixed prompt is never echoed and the generated response is
 * never carried. */
export interface InferenceObservationView {
  /** Collision-safe runtime provider identity. Pre-1L fixtures default to Ollama. */
  provider?: RuntimeProvider;
  /** The controlled result state (snake_case). */
  state: InferenceObservationState;
  /** The model the run targeted, when one was selected. `null` only for
   * `no_model_selected`. */
  model: string | null;
  /** The fixed diagnostic profile identifier, present only when a model was
   * selected. */
  diagnostic_profile: string | null;
  /** The execution-boundary string. Always `"undetermined"` in 1G. */
  execution_boundary: string;
  /** Ollama-reported prompt token count for the run. `null` when absent. */
  prompt_eval_count: number | null;
  /** Ollama-reported generated (output) token count for the run. `null` when
   * absent. */
  eval_count: number | null;
  /** Ollama-reported model load duration, in nanoseconds. `null` when absent. */
  load_duration_ns: number | null;
  /** Ollama-reported prompt evaluation duration, in nanoseconds. `null` when
   * absent. */
  prompt_eval_duration_ns: number | null;
  /** Ollama-reported generation (eval) duration, in nanoseconds. `null` when
   * absent. */
  eval_duration_ns: number | null;
  /** Ollama-reported total duration, in nanoseconds. `null` when absent. */
  total_duration_ns: number | null;
  /** Engine-Room-calculated generation rate in tokens per second, from
   * `eval_count` and `eval_duration_ns`. `null` unless the guards pass. Not
   * Ollama-reported. */
  generation_tokens_per_second: number | null;
  input_tokens?: number | null;
  total_output_tokens?: number | null;
  reasoning_output_tokens?: number | null;
  provider_reported_tokens_per_second?: number | null;
  time_to_first_token_seconds?: number | null;
  model_load_time_seconds?: number | null;
  interpretation: string;
  why_it_matters: string;
  limitations: string;
}

export type RuntimeProvider = "ollama" | "lm_studio";

export type LmStudioState =
  | "available"
  | "not_detected"
  | "unreachable"
  | "authentication_required"
  | "unsupported_api"
  | "responded_unexpectedly";

export interface LmStudioLoadedInstanceView {
  instance_id: string;
  context_length: number | null;
}

export interface LmStudioModelView {
  model_id: string;
  display_name: string;
  model_type: string;
  inference_eligible: boolean;
  publisher: string | null;
  architecture: string | null;
  quantization: string | null;
  bits_per_weight: number | null;
  size_bytes: number | null;
  params_string: string | null;
  max_context_length: number | null;
  format: string | null;
  loaded_instances: LmStudioLoadedInstanceView[];
}

export interface LmStudioSnapshotView {
  provider: "lm_studio";
  state: LmStudioState;
  models: LmStudioModelView[];
  skipped: number;
  state_label: string;
  interpretation: string;
  why_it_matters: string;
  resource_interpretation: string;
  resource_qualification: string;
}

export type LlamaCppState =
  | "unreachable"
  | "loading"
  | "ready"
  | "authentication_required"
  | "incompatible";

export interface LlamaCppServedModelView {
  model_id: string;
}

/** Passive traditional single-model llama-server state. Deliberately separate
 * from RuntimeProvider, which is the observed-inference provider union. */
export interface LlamaCppSnapshotView {
  provider: "llama_cpp";
  state: LlamaCppState;
  served_model: LlamaCppServedModelView | null;
  api_scope: "same_machine_loopback";
  compute_location: "not_independently_verified";
  state_label: string;
  interpretation: string;
  why_it_matters: string;
}

// --- Session-only deterministic diagnosis -----------------------------------

/** A bounded source supplied to the pure diagnosis command. `gap` means the
 * acquisition invocation did not yield a controlled view; it never carries a
 * raw error. */
export type DiagnosticAvailableMemoryInput =
  { kind: "value" } | { kind: "controlled_gap" } | { kind: "source_gap" };

export type DiagnosticOllamaRuntimeInput =
  | { kind: "gap" }
  | {
      kind: "observed";
      state: SourceAvailability;
      version: string | null;
    };

export type DiagnosticOllamaModelSetInput =
  | { kind: "gap" }
  | {
      kind: "observed";
      state: InventoryState | LoadedModelState;
      model_ids: string[];
    };

export interface DiagnosticLmStudioModelInput {
  model_id: string;
  inference_eligible: boolean;
  loaded_instance_ids: string[];
}

export type DiagnosticLmStudioInput =
  | { kind: "gap" }
  | {
      kind: "observed";
      state: LmStudioState;
      models: DiagnosticLmStudioModelInput[];
    };

export type DiagnosticLlamaCppInput =
  | { kind: "gap" }
  | {
      kind: "observed";
      state: LlamaCppState;
      served_model_id: string | null;
    };

/** Minimum controlled input for the pure, no-I/O diagnosis command. Model
 * identities remain provider-qualified by their enclosing source. */
export interface DiagnosticInput {
  available_memory: DiagnosticAvailableMemoryInput;
  ollama_runtime: DiagnosticOllamaRuntimeInput;
  ollama_catalogue: DiagnosticOllamaModelSetInput;
  ollama_loaded: DiagnosticOllamaModelSetInput;
  lm_studio: DiagnosticLmStudioInput;
  llama_cpp: DiagnosticLlamaCppInput;
  supported_change_count: number;
}

export type DiagnosticFindingSource =
  "ollama" | "lm_studio" | "llama_cpp" | "observation" | "cross_provider";

/** Controlled diagnosis output. These fields are authored by the native app
 * layer and contain no raw error/provider response. */
export interface DiagnosticFindingView {
  code: string;
  source: DiagnosticFindingSource;
  observation: string;
  meaning: string;
  safe_next_check: string;
}

export interface DiagnosisResultView {
  state: "available" | "invalid_input";
  findings: DiagnosticFindingView[];
}
export interface ReportPreviewView {
  text: string;
  generation: string;
}

export type ReportSaveResult =
  | "saved"
  | "cancelled"
  | "busy"
  | "preview_changed"
  | "invalid_destination"
  | "destination_exists"
  | "unavailable"
  | "failed"
  | "cleanup_incomplete"
  | "completion_uncertain";
