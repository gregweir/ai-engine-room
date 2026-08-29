// Browser-development fixtures. Clearly artificial: round-number values,
// generic display names, no real machine/model/path/address/username data. The
// explanation strings are copied verbatim from the Rust `explain()` controlled
// constants so the browser preview shows the same text the backend would, and
// the UI never invents its own technical interpretation.
//
// These mirror the Stage 8 view DTO shapes. The fixture snapshot covers every
// provenance category, every source/metric availability state, and the
// Ok/NoValue/Failed outcomes. The report preview includes only the allow-listed
// report-safe metrics (no internal ids, paths, or raw errors).

import type {
  InferenceObservationView,
  LoadedModelSetView,
  LoadedModelSummaryView,
  ModelInventoryView,
  ResourceContextView,
  RuntimeModelSummaryView,
  RuntimeStatusView,
  SnapshotEntryView,
  SnapshotView,
} from "../types";

// Controlled explanation strings (mirrors of crates/aer-core/src/explain.rs).
const OK_DRIVER = "This value was reported by a hardware driver.";
const OK_RUNTIME = "This value was reported by the AI runtime.";
const OK_MEASURED = "Engine Room measured this value directly.";
const OK_CALCULATED = "This value was calculated from other metrics.";
const OK_ESTIMATED = "This value is an estimate based on other metrics.";
const OK_WHY = "This is the most recent reading for this metric.";

const RAM_AVAILABLE_INTERP =
  "This is Linux's estimate of how much memory is available for starting new applications without swapping.";
const RAM_AVAILABLE_WHY =
  "Local AI models need memory to load and run. A low or shrinking value reduces headroom and can slow inference if the system starts using swap.";
const RAM_AVAILABLE_DEEPER =
  "The value comes from /proc/meminfo MemAvailable. It is an estimate, not a guarantee that a single process can allocate the full amount.";

const NO_VALUE_SOURCE =
  "The source needed for this metric is not available right now.";
const NO_VALUE_METRIC =
  "The source is available but is not providing this metric right now.";
const NO_VALUE_WHY =
  "No value is shown because the metric is unavailable, not because of an error.";

const UNSUPPORTED_INTERP = "This metric is not supported in this build.";
const UNSUPPORTED_WHY =
  "Engine Room does not yet know how to acquire it here, so no value is shown.";
const REQUIRES_DRIVER_DEEPER =
  "A required hardware driver is not present or not loaded.";

const FAILED_INTERP = "Acquiring this metric failed.";
const TIMEOUT_WHY = "Acquiring this metric timed out.";
const FAILED_DEEPER =
  "This is an acquisition failure, not a normal unavailable state.";

const entries: SnapshotEntryView[] = [
  // --- Allow-listed metrics (appear in the report preview) ---
  {
    display_name: "Available memory",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 8_589_934_592,
      unit: "bytes",
      provenance: "operating_system_reported",
    },
    interpretation: RAM_AVAILABLE_INTERP,
    why_it_matters: RAM_AVAILABLE_WHY,
    deeper: RAM_AVAILABLE_DEEPER,
  },
  {
    display_name: "Total memory",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 17_179_869_184,
      unit: "bytes",
      provenance: "operating_system_reported",
    },
    interpretation: "This artificial fixture represents total system memory.",
    why_it_matters:
      "Total memory provides context but does not establish model fit or headroom.",
    deeper:
      "This browser value is artificial and is not native machine telemetry.",
  },
  {
    display_name: "Memory in use",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 8_589_934_592,
      unit: "bytes",
      provenance: "calculated",
    },
    interpretation: OK_CALCULATED,
    why_it_matters: OK_WHY,
    deeper: null,
  },
  {
    display_name: "Swap in use",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "transiently_unavailable",
    outcome: { kind: "no_value" },
    interpretation: NO_VALUE_METRIC,
    why_it_matters: NO_VALUE_WHY,
    deeper: null,
  },
  {
    display_name: "CPU load",
    unit: "percent",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 0.42,
      unit: "percent",
      provenance: "application_measured",
    },
    interpretation: OK_MEASURED,
    why_it_matters: OK_WHY,
    deeper: null,
  },
  {
    display_name: "Disk in use",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 500_000_000_000,
      unit: "bytes",
      provenance: "estimated",
    },
    interpretation: OK_ESTIMATED,
    why_it_matters: OK_WHY,
    deeper: null,
  },

  // --- Representative states (full view only, not report-safe) ---
  {
    display_name: "GPU memory in use",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 2_147_483_648,
      unit: "bytes",
      provenance: "driver_reported",
    },
    interpretation: OK_DRIVER,
    why_it_matters: OK_WHY,
    deeper: null,
  },
  {
    display_name: "AI runtime context tokens",
    unit: "count",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 4096,
      unit: "count",
      provenance: "runtime_reported",
    },
    interpretation: OK_RUNTIME,
    why_it_matters: OK_WHY,
    deeper: null,
  },
  {
    display_name: "Cached memory",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: {
      kind: "ok",
      value: 2_147_483_648,
      unit: "bytes",
      provenance: "calculated",
    },
    interpretation: OK_CALCULATED,
    why_it_matters: OK_WHY,
    deeper: null,
  },
  {
    display_name: "GPU fan speed",
    unit: "count",
    support: { kind: "unsupported", limitation_code: "requires_driver" },
    source_availability: null,
    metric_availability: "not_applicable",
    outcome: { kind: "no_value" },
    interpretation: UNSUPPORTED_INTERP,
    why_it_matters: UNSUPPORTED_WHY,
    deeper: REQUIRES_DRIVER_DEEPER,
  },
  {
    display_name: "AI runtime GPU layers",
    unit: "count",
    support: { kind: "supported" },
    source_availability: "not_detected",
    metric_availability: null,
    outcome: { kind: "no_value" },
    interpretation: NO_VALUE_SOURCE,
    why_it_matters: NO_VALUE_WHY,
    deeper: null,
  },
  {
    display_name: "AI runtime loaded models",
    unit: "count",
    support: { kind: "supported" },
    source_availability: "not_running",
    metric_availability: null,
    outcome: { kind: "no_value" },
    interpretation: NO_VALUE_SOURCE,
    why_it_matters: NO_VALUE_WHY,
    deeper: null,
  },
  {
    display_name: "AI runtime remote health",
    unit: "count",
    support: { kind: "supported" },
    source_availability: "unreachable",
    metric_availability: null,
    outcome: { kind: "no_value" },
    interpretation: NO_VALUE_SOURCE,
    why_it_matters: NO_VALUE_WHY,
    deeper: null,
  },
  {
    display_name: "Thermal sensor",
    unit: "count",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "not_exposed",
    outcome: { kind: "no_value" },
    interpretation: NO_VALUE_METRIC,
    why_it_matters: NO_VALUE_WHY,
    deeper: null,
  },
  {
    display_name: "Network errors",
    unit: "count",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: { kind: "failed", category: "timeout" },
    interpretation: FAILED_INTERP,
    why_it_matters: TIMEOUT_WHY,
    deeper: FAILED_DEEPER,
  },
];

export function fixtureSnapshot(): SnapshotView {
  // Deterministic fixture: a fixed timestamp and the representative entries.
  return {
    at: 1_700_000_000_000,
    entries: entries.map((entry) => ({ ...entry })),
  };
}

/** Report-safe human-readable preview: only the allow-listed metrics, no
 * internal ids, paths, or raw error text. Mirrors the backend `render_preview`. */
export function fixtureReportPreview(): string {
  return [
    "AI Engine Room — Observation Report",
    "",
    "Observation",
    "Available memory",
    "Status: Available",
    "Value: 8.59 GB",
    "Exact value: 8,589,934,592 bytes",
    "Source: Operating system",
    "",
    "Qualifications",
    "- Available memory is an operating-system observation.",
    "- Operating systems define available memory differently; values from different platforms are not claimed to be numerically equivalent.",
    "",
    "Observation",
    "Memory in use",
    "Status: Available",
    "Value: 8.59 GB",
    "Exact value: 8,589,934,592 bytes",
    "Source: Calculated from other metrics",
    "",
    "Observation",
    "Swap in use",
    "Status: Not available",
    "Value: No value reported",
    "",
    "Observation",
    "CPU load",
    "Status: Available",
    "Value: 0.42 percent",
    "Source: AI Engine Room",
    "",
    "Observation",
    "Disk in use",
    "Status: Available",
    "Value: 500 GB",
    "Exact value: 500,000,000,000 bytes",
    "Source: Estimate",
    "",
    "Scope",
    "- This report contains only allow-listed report-safe observations.",
    "- Available memory does not establish memory pressure, model fit, headroom, or compute placement.",
    "",
  ].join("\n");
}

// --- Milestone 1C runtime status fixtures ------------------------------------
//
// Clearly artificial runtime detection statuses that mirror the Rust
// `RuntimeStatusView` wire shape and the controlled text the backend authors.
// The status_label/interpretation/why_it_matters strings are copied verbatim
// from the Rust `ollama_status_text` mapping so the browser preview shows the
// same text the backend would, and the UI never invents its own technical
// interpretation or infers detection semantics from transport kinds. No raw
// errors, URLs, hostnames, ports, or status codes appear here; nothing claims
// install state or health/compatibility/security/performance.
//
// The four fixtures cover the four intended user-facing states distinguished
// by `(source_availability, version !== null)`: available+version, responded
// unexpectedly, not detected, unreachable.

const RUNTIME_AVAILABLE: RuntimeStatusView = {
  runtime_type: "ollama",
  source_availability: "ready",
  version: "0.1.32",
  version_reported_by: "runtime_reported",
  status_label: "Ollama available",
  interpretation:
    "Ollama is responding at the local endpoint and reported its version.",
  why_it_matters:
    "Engine Room can read the version Ollama reports for this local runtime.",
};

const RUNTIME_RESPONDED_UNEXPECTEDLY: RuntimeStatusView = {
  runtime_type: "ollama",
  source_availability: "ready",
  version: null,
  version_reported_by: null,
  status_label: "Ollama responded unexpectedly",
  interpretation:
    "Ollama answered at the local endpoint, but the response did not include usable version information.",
  why_it_matters:
    "The endpoint was reached, so Ollama is present, but its version could not be read from the response.",
};

const RUNTIME_NOT_DETECTED: RuntimeStatusView = {
  runtime_type: "ollama",
  source_availability: "not_detected",
  version: null,
  version_reported_by: null,
  status_label: "Ollama was not detected at the local endpoint",
  interpretation: "Nothing responded at the local Ollama endpoint.",
  why_it_matters:
    "No service answered the local endpoint. This only shows that nothing answered there right now — not whether Ollama is running on this machine.",
};

const RUNTIME_UNREACHABLE: RuntimeStatusView = {
  runtime_type: "ollama",
  source_availability: "unreachable",
  version: null,
  version_reported_by: null,
  status_label: "Ollama could not be reached",
  interpretation: "Ollama could not be reached at the local endpoint.",
  why_it_matters:
    "The local endpoint did not respond in time. Ollama may be unreachable or too slow to answer right now.",
};

/** All four representative runtime status fixtures, one per user-facing state.
 * The browser/preview path uses these; the native path fetches the real status
 * over Tauri IPC. Static and artificial — no real Ollama is contacted. */
export function fixtureRuntimeStatuses(): RuntimeStatusView[] {
  return [
    { ...RUNTIME_AVAILABLE },
    { ...RUNTIME_RESPONDED_UNEXPECTEDLY },
    { ...RUNTIME_NOT_DETECTED },
    { ...RUNTIME_UNREACHABLE },
  ];
}

/** The default fixture runtime status for browser development: the available
 * state with a runtime-reported version. Mirrors the single-status shape the
 * Tauri `current_runtime_status` command returns. */
export function fixtureRuntimeStatus(): RuntimeStatusView {
  return { ...RUNTIME_AVAILABLE };
}

// --- Milestone 1D model inventory fixtures ------------------------------------
//
// Clearly artificial model inventories that mirror the Rust `ModelInventoryView`
// wire shape and the controlled text the backend authors via
// `ollama_inventory_text`. The state_label/interpretation/why_it_matters strings
// are copied verbatim from the Rust mapping (Commit 2) so the browser preview
// shows the same text the backend would, and the UI never invents its own
// technical interpretation or infers inventory state from transport kinds. No
// raw errors, URLs, hostnames, ports, status codes, `digest`, or `remote_host`
// appear here; nothing claims install state or health/compat/security/
// performance. Locality is `"undetermined"` for every entry and is not displayed
// per model.
//
// Fixture model names and sizes are clearly artificial (e.g. `example-chat:8b`,
// round-number sizes) — no real installed-model catalogue is copied here.

const INVENTORY_AVAILABLE_ENTRIES: RuntimeModelSummaryView[] = [
  {
    display_name: "example-chat:8b",
    locality: "undetermined",
    size_bytes: 4_000_000_000,
    parameter_size: "8.0B",
    quantization: "Q4_0",
    families: ["llama"],
  },
  {
    display_name: "example-coder:3b",
    locality: "undetermined",
    size_bytes: 2_000_000_000,
    parameter_size: "3.0B",
    quantization: "Q4_K_M",
    families: ["qwen"],
  },
  {
    // An entry with no reported size exercises the null/omitted path
    // (size shown only when the runtime reports one).
    display_name: "example-embed:text",
    locality: "undetermined",
    size_bytes: null,
    parameter_size: null,
    quantization: null,
    families: [],
  },
];

const INVENTORY_AVAILABLE: ModelInventoryView = {
  runtime_type: "ollama",
  state: "available",
  source_availability: "ready",
  models: INVENTORY_AVAILABLE_ENTRIES.map((entry) => ({ ...entry })),
  skipped: 0,
  state_label: "Ollama model catalogue",
  interpretation: "These models are listed by the Ollama runtime.",
  why_it_matters:
    "Each entry shows the model name and the size Ollama reports for it.",
};

const INVENTORY_AVAILABLE_EMPTY: ModelInventoryView = {
  runtime_type: "ollama",
  state: "available",
  source_availability: "ready",
  models: [],
  skipped: 0,
  state_label: "No models in the catalogue",
  interpretation: "Ollama lists no models right now.",
  why_it_matters:
    "This is not an error — the catalogue simply contains no models right now.",
};

const INVENTORY_RESPONDED_UNEXPECTEDLY: ModelInventoryView = {
  runtime_type: "ollama",
  state: "responded_unexpectedly",
  source_availability: "ready",
  models: [],
  skipped: 0,
  state_label: "Ollama responded unexpectedly",
  interpretation:
    "Ollama answered at the local endpoint, but the model list could not be read.",
  why_it_matters:
    "The endpoint was reached, so Ollama is present, but its catalogue response was not usable.",
};

const INVENTORY_NOT_DETECTED: ModelInventoryView = {
  runtime_type: "ollama",
  state: "not_detected",
  source_availability: "not_detected",
  models: [],
  skipped: 0,
  state_label: "Ollama was not detected at the local endpoint",
  interpretation: "Nothing responded at the local Ollama endpoint.",
  why_it_matters:
    "No service answered the local endpoint. This only shows that nothing answered there right now — not whether Ollama is running on this machine.",
};

const INVENTORY_UNREACHABLE: ModelInventoryView = {
  runtime_type: "ollama",
  state: "unreachable",
  source_availability: "unreachable",
  models: [],
  skipped: 0,
  state_label: "Ollama could not be reached",
  interpretation: "Ollama could not be reached at the local endpoint.",
  why_it_matters:
    "The local endpoint did not respond in time. Ollama may be unreachable or too slow to answer right now.",
};

/** All five representative model inventory fixtures, one per inventory state
 * (with `available` split into a populated and an empty case). The browser/
 * preview path uses these; the native path fetches the real inventory over
 * Tauri IPC. Static and artificial — no real Ollama is contacted. */
export function fixtureModelInventories(): ModelInventoryView[] {
  return [
    {
      ...INVENTORY_AVAILABLE,
      models: INVENTORY_AVAILABLE.models.map((m) => ({
        ...m,
        families: [...m.families],
      })),
    },
    { ...INVENTORY_AVAILABLE_EMPTY },
    { ...INVENTORY_RESPONDED_UNEXPECTEDLY },
    { ...INVENTORY_NOT_DETECTED },
    { ...INVENTORY_UNREACHABLE },
  ];
}

/** The default fixture model inventory for browser development: the available
 * state with a representative populated catalogue. Mirrors the single-inventory
 * shape the Tauri `current_model_inventory` command returns. */
export function fixtureModelInventory(): ModelInventoryView {
  return {
    ...INVENTORY_AVAILABLE,
    models: INVENTORY_AVAILABLE.models.map((m) => ({
      ...m,
      families: [...m.families],
    })),
  };
}

/** The empty-catalogue fixture: `available` with no models (a valid empty
 * catalogue is not an error). Used to exercise the empty-state UI. */
export function fixtureEmptyModelInventory(): ModelInventoryView {
  return { ...INVENTORY_AVAILABLE_EMPTY };
}

// --- Milestone 1E loaded-models fixtures --------------------------------------
//
// Clearly artificial loaded-model sets that mirror the Rust `LoadedModelSetView`
// wire shape and the controlled text the backend authors via
// `ollama_loaded_models_text`. The state_label/interpretation/why_it_matters
// strings are copied verbatim from the Rust mapping (Commit 2) so the browser
// preview shows the same text the backend would, and the UI never invents its
// own technical interpretation or infers loaded-model state from transport
// kinds. No raw errors, URLs, hostnames, ports, status codes, `digest`, or
// `remote_host` appear here; nothing claims install state or health/compat/
// security/performance. `expires_at` is carried internally but is NOT displayed
// (no countdown, no date/time dependency). No aggregate total is computed.
//
// Distinct from the 1D catalogue fixtures: 1E answers "what is running now?",
// not "what is in the catalogue?", so the fixture model names and counts differ.
// Names are clearly artificial (`example-*`), sizes are round numbers, and
// `context_length` values are artificial counts — no real loaded-model state is
// copied here.

const LOADED_AVAILABLE_ENTRIES: LoadedModelSummaryView[] = [
  {
    display_name: "example-runner:8b",
    reported_size_bytes: 5_000_000_000,
    reported_vram_bytes: 4_000_000_000,
    reported_context_length: 8192,
    // Carried internally, NOT displayed by the UI.
    expires_at: "2024-01-01T00:00:00.000Z",
  },
  {
    display_name: "example-assistant:3b",
    reported_size_bytes: 2_000_000_000,
    reported_vram_bytes: 1_500_000_000,
    reported_context_length: 4096,
    expires_at: "2024-01-01T00:00:00.000Z",
  },
  {
    // A loaded entry with no reported size/VRAM/context exercises the explicit
    // not-reported presentation path.
    display_name: "example-tiny:text",
    reported_size_bytes: null,
    reported_vram_bytes: null,
    reported_context_length: null,
    expires_at: null,
  },
];

const LOADED_RESOURCE_TEXT = {
  resource_interpretation:
    "Loaded size is metadata reported by Ollama, not an exact model-weight, RAM, or disk allocation. VRAM size is also reported by Ollama and is not an independent measurement of physical VRAM use or capacity.",
  resource_qualification:
    "Configured context is a provider-reported count. A larger configured context can require more memory, but AI Engine Room does not convert it to bytes. KV-cache bytes and runtime overhead are not separately reported, and compute placement remains unknown.",
} as const;

const LOADED_AVAILABLE: LoadedModelSetView = {
  ...LOADED_RESOURCE_TEXT,
  runtime_type: "ollama",
  state: "available",
  source_availability: "ready",
  models: LOADED_AVAILABLE_ENTRIES.map((entry) => ({ ...entry })),
  skipped: 0,
  state_label: "Currently loaded in Ollama",
  interpretation: "Ollama reports these models as currently loaded.",
  why_it_matters:
    "Each entry shows the model name and the loaded size Ollama reports for it.",
};

const LOADED_AVAILABLE_EMPTY: LoadedModelSetView = {
  ...LOADED_RESOURCE_TEXT,
  runtime_type: "ollama",
  state: "available",
  source_availability: "ready",
  models: [],
  skipped: 0,
  state_label: "No models currently running in Ollama",
  interpretation: "Ollama reports no models as loaded right now.",
  why_it_matters:
    "This is not an error — it means no models are currently running in Ollama.",
};

const LOADED_RESPONDED_UNEXPECTEDLY: LoadedModelSetView = {
  ...LOADED_RESOURCE_TEXT,
  runtime_type: "ollama",
  state: "responded_unexpectedly",
  source_availability: "ready",
  models: [],
  skipped: 0,
  state_label: "Ollama responded unexpectedly",
  interpretation:
    "Ollama answered at the local endpoint, but the loaded-model list could not be read.",
  why_it_matters:
    "The endpoint was reached, so Ollama is present, but its loaded-model response was not usable.",
};

const LOADED_NOT_DETECTED: LoadedModelSetView = {
  ...LOADED_RESOURCE_TEXT,
  runtime_type: "ollama",
  state: "not_detected",
  source_availability: "not_detected",
  models: [],
  skipped: 0,
  state_label: "Ollama was not detected at the local endpoint",
  interpretation: "Nothing responded at the local Ollama endpoint.",
  why_it_matters:
    "No service answered the local endpoint. This only shows that nothing answered there right now — not whether Ollama is running on this machine.",
};

const LOADED_UNREACHABLE: LoadedModelSetView = {
  ...LOADED_RESOURCE_TEXT,
  runtime_type: "ollama",
  state: "unreachable",
  source_availability: "unreachable",
  models: [],
  skipped: 0,
  state_label: "Ollama could not be reached",
  interpretation: "Ollama could not be reached at the local endpoint.",
  why_it_matters:
    "The local endpoint did not respond in time. Ollama may be unreachable or too slow to answer right now.",
};

/** All five representative loaded-model fixtures, one per loaded-model state
 * (with `available` split into a populated and an empty case). The browser/
 * preview path uses these; the native path fetches the real loaded-model set
 * over Tauri IPC. Static and artificial — no real Ollama is contacted. */
export function fixtureLoadedModelsSets(): LoadedModelSetView[] {
  return [
    {
      ...LOADED_AVAILABLE,
      models: LOADED_AVAILABLE.models.map((m) => ({ ...m })),
    },
    { ...LOADED_AVAILABLE_EMPTY },
    { ...LOADED_RESPONDED_UNEXPECTEDLY },
    { ...LOADED_NOT_DETECTED },
    { ...LOADED_UNREACHABLE },
  ];
}

/** The default fixture loaded-model set for browser development: the available
 * state with a representative populated set. Mirrors the single-set shape the
 * Tauri `current_loaded_models` command returns. */
export function fixtureLoadedModels(): LoadedModelSetView {
  return {
    ...LOADED_AVAILABLE,
    models: LOADED_AVAILABLE.models.map((m) => ({ ...m })),
  };
}

/** The empty loaded-model fixture: `available` with no models (a valid empty
 * loaded-model list is not an error). Used to exercise the empty-state UI. */
export function fixtureEmptyLoadedModels(): LoadedModelSetView {
  return { ...LOADED_AVAILABLE_EMPTY };
}

// --- Milestone 1F resource-context fixtures -----------------------------------
//
// Clearly artificial resource-context compositions that mirror the Rust
// `ResourceContextView` wire shape and the controlled text the backend authors
// via `resource_context_text`. The interpretation/why_it_matters strings are
// copied verbatim from the Rust mapping (Commit 1) so the browser preview shows
// the same text the backend would, and the UI never invents its own technical
// interpretation, combines values, computes ratios, or picks the variant. No
// raw errors, URLs, hostnames, ports, status codes, `digest`, or `remote_host`
// appear here; nothing claims fit/health/headroom/sufficient/insufficient/
// performance/fast/slow or recommends load/unload. The context-length line is
// qualitative only (no per-token/per-memory figure, no "better", no performance
// claim). No aggregate total is computed and no model name is carried.
//
// Two fixtures cover the two ends of the variant range: variant 1 (both
// available) and variant 4 (both unavailable). Variants 2 and 3 are produced by
// the backend from real views; the browser path only needs a representative
// explanatory note, so variant 1 is the default and variant 4 exercises the
// both-unavailable block.

const RESOURCE_CONTEXT_AVAILABLE: ResourceContextView = {
  interpretation:
    "These values come from different sources. Available memory is the operating system's platform-native observation of memory available to the whole system; operating systems can define it differently. Loaded size and VRAM size are values Ollama reports for each running model. Engine Room does not treat them as an exact account of RAM or VRAM use.",
  why_it_matters:
    "They should not be added to or subtracted from available memory to estimate how much memory a model is using. Available memory already reflects current system state, and the model sizes are runtime-reported values, not independently measured consumption. A larger configured context can require more memory.",
  concepts: [
    {
      concept: "System memory",
      state_label: "Reported evidence available",
      interpretation:
        "The operating system supplied a whole-system available-memory observation.",
    },
    {
      concept: "Model weights",
      state_label: "Not separately reported",
      interpretation:
        "The current evidence does not isolate model-weight allocation from other provider or runtime memory.",
    },
    {
      concept: "Loaded size",
      state_label: "Reported evidence available",
      interpretation:
        "Ollama supplied a loaded-size value for at least one running model. It is provider metadata, not exact RAM use, disk use, or model-weight allocation.",
    },
    {
      concept: "Configured context",
      state_label: "Reported evidence available",
      interpretation:
        "Ollama supplied a configured-context count for at least one running model. AI Engine Room does not convert context length to bytes.",
    },
    {
      concept: "KV cache",
      state_label: "Not separately reported",
      interpretation:
        "The current provider evidence does not expose KV-cache bytes, so AI Engine Room does not calculate or estimate them.",
    },
    {
      concept: "Runtime overhead",
      state_label: "Not separately reported",
      interpretation:
        "The current provider evidence does not isolate runtime-overhead bytes.",
    },
    {
      concept: "VRAM",
      state_label: "Reported evidence available",
      interpretation:
        "Ollama supplied a VRAM-size value for at least one running model. It is not an independent measurement of physical VRAM use or capacity.",
    },
    {
      concept: "Compute placement",
      state_label: "Unknown",
      interpretation:
        "The current evidence does not establish CPU, GPU, split, or offloaded execution placement.",
    },
  ],
};

const RESOURCE_CONTEXT_BOTH_UNAVAILABLE: ResourceContextView = {
  interpretation:
    "Neither the available-memory reading nor the running-model information is available right now.",
  why_it_matters:
    "There is nothing to place alongside until at least one of them is available.",
  concepts: [
    {
      concept: "System memory",
      state_label: "Not reported",
      interpretation:
        "A usable whole-system available-memory observation is not available.",
    },
    {
      concept: "Model weights",
      state_label: "Not separately reported",
      interpretation:
        "The current evidence does not isolate model-weight allocation from other provider or runtime memory.",
    },
    {
      concept: "Loaded size",
      state_label: "Not reported",
      interpretation:
        "Ollama did not supply a loaded-size value for a running model in this observation.",
    },
    {
      concept: "Configured context",
      state_label: "Not reported",
      interpretation:
        "Ollama did not supply a configured-context count for a running model in this observation.",
    },
    {
      concept: "KV cache",
      state_label: "Not separately reported",
      interpretation:
        "The current provider evidence does not expose KV-cache bytes, so AI Engine Room does not calculate or estimate them.",
    },
    {
      concept: "Runtime overhead",
      state_label: "Not separately reported",
      interpretation:
        "The current provider evidence does not isolate runtime-overhead bytes.",
    },
    {
      concept: "VRAM",
      state_label: "Not reported",
      interpretation:
        "Ollama did not supply a VRAM-size value for a running model in this observation; physical VRAM capacity is not acquired here.",
    },
    {
      concept: "Compute placement",
      state_label: "Unknown",
      interpretation:
        "The current evidence does not establish CPU, GPU, split, or offloaded execution placement.",
    },
  ],
};

/** The default fixture resource-context composition for browser development:
 * the variant-1 (both-available) strings, including the approved qualitative
 * context-length line. Mirrors the single-composition shape the Tauri
 * `compose_resource_context` command returns when both sources are available.
 * Static and artificial — no real machine memory value or real loaded-model
 * state is contacted. */
export function fixtureResourceContext(): ResourceContextView {
  return { ...RESOURCE_CONTEXT_AVAILABLE };
}

/** The both-unavailable fixture resource-context composition (variant 4):
 * neither source is available. Used to exercise the both-unavailable block. No
 * context-length line appears in this variant. */
export function fixtureResourceContextUnavailable(): ResourceContextView {
  return { ...RESOURCE_CONTEXT_BOTH_UNAVAILABLE };
}

// --- Milestone 1G observed-inference fixtures + controlled text --------------
//
// Clearly artificial observed-inference results that mirror the Rust
// `InferenceObservationView` wire shape and the controlled text the backend
// authors via `ollama_inference_text`. The interpretation/why_it_matters/
// limitations strings are copied verbatim from the Rust mapping (Commit 1) so
// the browser preview shows the same text the backend would, and the UI never
// invents its own technical interpretation or infers locality/benchmark/
// quality from the counts. No raw errors, URLs, hostnames, ports, status codes,
// **generated response text**, or transport kinds appear here; nothing claims
// "local"/"verified"/install/health/security/performance. `execution_boundary`
// is `"undetermined"` for every fixture (the only value in 1G). Durations are
// carried in nanoseconds (Ollama's unit); the UI renders them as seconds.
//
// The §4 disclosure copy and the §8 rate label/explanation are mirrored verbatim
// from the Rust constants (`DISCLOSURE_*`, `GENERATION_RATE_*`,
// `EXECUTION_LOCATION_NOT_DETERMINED`) so the browser dialog shows the same
// wording the backend owns, and the frontend tests pin the wording.

/** Disclosure dialog title. Mirrors the Rust `DISCLOSURE_TITLE`. */
export const DISCLOSURE_TITLE = "Run an observed inference?";

/** Disclosure dialog body paragraphs, in display order. Mirrors the Rust
 * `DISCLOSURE_BODY` verbatim (1G plan §4). */
export const DISCLOSURE_BODY: readonly string[] = [
  "Engine Room will send a fixed synthetic diagnostic prompt to the selected Ollama model.",
  "Engine Room cannot determine from Ollama's current API whether this model will run on this computer or through Ollama Cloud.",
  "The diagnostic prompt contains no user files, repository content, machine information, or other user-provided data.",
  "This will perform one real inference and may load the model and use CPU, GPU, and memory resources. Ollama may keep the model loaded afterward according to its current configuration.",
  "The result is one observed run, not a general benchmark or model-quality score. This authorization applies to one run only; another run requires another authorization.",
];

/** Primary disclosure action label. Terminology: "Run observation". */
export const DISCLOSURE_PRIMARY_ACTION = "Run observation";

/** Secondary disclosure action label. There is intentionally no
 * "Don't ask again" action. */
export const DISCLOSURE_SECONDARY_ACTION = "Cancel";

/** Controlled label for the Engine-Room-calculated generation rate. Not an
 * Ollama-reported figure. */
export const GENERATION_RATE_LABEL = "Observed generation rate";

/** Controlled explanation of how the generation rate was calculated. */
export const GENERATION_RATE_EXPLANATION =
  "Calculated from the output-token count and generation duration Ollama reported for this run.";

/** The controlled "execution location not determined" limitation sentence,
 * carried in the `limitations` field of every observation. */
export const EXECUTION_LOCATION_NOT_DETERMINED =
  "Execution location not determined.";

const INF_NO_MODEL: InferenceObservationView = {
  state: "no_model_selected",
  model: null,
  diagnostic_profile: null,
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation: "No model is selected for an observed inference.",
  why_it_matters: "Select a model from the catalogue to run an observation.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_BLOCKED: InferenceObservationView = {
  state: "blocked_by_execution_boundary",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation:
    "Engine Room will not run an observation without explicit authorization.",
  why_it_matters:
    "Engine Room cannot determine whether the model will run on this computer or through Ollama Cloud, so each observation requires you to authorize that one run.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_RUNNING: InferenceObservationView = {
  state: "running",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation: "One observed inference is in progress.",
  why_it_matters:
    "Engine Room issues at most one observation request. Controls are disabled until it finishes.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_COMPLETED: InferenceObservationView = {
  state: "completed",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: 12,
  eval_count: 8,
  load_duration_ns: 500_000_000,
  prompt_eval_duration_ns: 300_000_000,
  eval_duration_ns: 1_000_000_000,
  total_duration_ns: 1_800_000_000,
  generation_tokens_per_second: 8.0,
  interpretation:
    "One observed inference completed. The counts and durations are values Ollama reported for this run.",
  why_it_matters:
    "This is a single observed run, not a general benchmark or model-quality score.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_NOT_DETECTED: InferenceObservationView = {
  state: "not_detected",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation: "Ollama was not detected, so the observation could not run.",
  why_it_matters:
    "Nothing answered the Ollama endpoint. This only shows that nothing answered right now.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_UNREACHABLE: InferenceObservationView = {
  state: "unreachable",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation:
    "Ollama could not be reached, so the observation could not run.",
  why_it_matters:
    "The Ollama endpoint did not respond in time. It may be unreachable or too slow to answer right now.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_TIMED_OUT: InferenceObservationView = {
  state: "timed_out",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation: "The observation did not finish within the time limit.",
  why_it_matters:
    "The inference request took longer than the allowed time. No retry is attempted automatically.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_RESPONDED_UNEXPECTEDLY: InferenceObservationView = {
  state: "responded_unexpectedly",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation:
    "Ollama responded in a way Engine Room could not use for an observation.",
  why_it_matters:
    "The endpoint answered, but the response did not include the expected completion information.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

const INF_MODEL_UNAVAILABLE: InferenceObservationView = {
  state: "model_unavailable",
  model: "example-runner:8b",
  diagnostic_profile: "diagnostic-basic-v1",
  execution_boundary: "undetermined",
  prompt_eval_count: null,
  eval_count: null,
  load_duration_ns: null,
  prompt_eval_duration_ns: null,
  eval_duration_ns: null,
  total_duration_ns: null,
  generation_tokens_per_second: null,
  interpretation: "The selected model was not found in the Ollama catalogue.",
  why_it_matters:
    "Engine Room only runs an observation for a model Ollama currently lists. Select a listed model.",
  limitations: EXECUTION_LOCATION_NOT_DETERMINED,
};

/** All nine representative observed-inference fixtures, one per result state.
 * The browser/preview path uses these; the native path fetches the real
 * observation over Tauri IPC. Static and artificial — no real Ollama is
 * contacted and no generated response text is carried. */
export function fixtureInferenceObservations(): InferenceObservationView[] {
  return [
    { ...INF_NO_MODEL },
    { ...INF_BLOCKED },
    { ...INF_RUNNING },
    { ...INF_COMPLETED },
    { ...INF_NOT_DETECTED },
    { ...INF_UNREACHABLE },
    { ...INF_TIMED_OUT },
    { ...INF_RESPONDED_UNEXPECTEDLY },
    { ...INF_MODEL_UNAVAILABLE },
  ];
}

/** The default fixture observed-inference result for browser development: the
 * completed state with artificial counts/durations and an Engine-Room-
 * calculated rate. Mirrors the single-result shape the Tauri
 * `run_inference_observation` command returns. Static and artificial — no real
 * Ollama is contacted and no generated response text is carried. */
export function fixtureInferenceObservation(): InferenceObservationView {
  return { ...INF_COMPLETED };
}
