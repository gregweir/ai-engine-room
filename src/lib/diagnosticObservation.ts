import {
  eventFromAvailableMemorySnapshot,
  unavailableReasonLabel,
  type AvailableMemoryEvent,
} from "./availableMemoryHistory";
import { formatBytesFriendly, formatExactBytes } from "./formatBytes";
import type {
  DiagnosticInput,
  LoadedModelSetView,
  LlamaCppSnapshotView,
  LmStudioSnapshotView,
  ModelInventoryView,
  RuntimeStatusView,
  SnapshotView,
} from "./types";

export const DIAGNOSTIC_OBSERVATION_LIMIT = 12;
const MAX_DIAGNOSTIC_IDENTITIES = 256;
const MAX_DIAGNOSTIC_INSTANCES = 512;
const MAX_DIAGNOSTIC_ID_CHARS = 256;
const MAX_RUNTIME_VERSION_CHARS = 128;

export type DiagnosticSource<T> =
  { kind: "observed"; value: T } | { kind: "gap" };

export interface DiagnosticOllamaRuntimeObservation {
  state: RuntimeStatusView["source_availability"];
  version: string | null;
}

export interface DiagnosticOllamaModelSetObservation {
  state: ModelInventoryView["state"] | LoadedModelSetView["state"];
  model_ids: string[];
}

export interface DiagnosticLmStudioModelObservation {
  model_id: string;
  inference_eligible: boolean;
  loaded_instance_ids: string[];
}

export interface DiagnosticLmStudioObservation {
  state: LmStudioSnapshotView["state"];
  models: DiagnosticLmStudioModelObservation[];
}

export interface DiagnosticLlamaCppObservation {
  state: LlamaCppSnapshotView["state"];
  served_model_id: string | null;
}

/** One bounded group of already-acquired controlled views. The group records
 * one startup/Refresh invocation; it is not described as an atomic machine
 * snapshot and contains no timestamp or raw acquisition detail. */
export interface DiagnosticObservationBundle {
  observation_id: number;
  available_memory: AvailableMemoryEvent;
  ollama_runtime: DiagnosticSource<DiagnosticOllamaRuntimeObservation>;
  ollama_catalogue: DiagnosticSource<DiagnosticOllamaModelSetObservation>;
  ollama_loaded: DiagnosticSource<DiagnosticOllamaModelSetObservation>;
  lm_studio: DiagnosticSource<DiagnosticLmStudioObservation>;
  llama_cpp: DiagnosticSource<DiagnosticLlamaCppObservation>;
}

export type DiagnosticObservationDraft = Omit<
  DiagnosticObservationBundle,
  "observation_id"
>;

export interface AppendDiagnosticObservationResult {
  history: DiagnosticObservationBundle[];
  nextObservationId: number;
}

export type DiagnosticChangeSource =
  | "available_memory"
  | "ollama_runtime"
  | "ollama_catalogue"
  | "ollama_loaded"
  | "lm_studio"
  | "llama_cpp";

export interface DiagnosticChange {
  code: string;
  source: DiagnosticChangeSource;
  summary: string;
  details: string[];
}

export type DiagnosticDeltaResult =
  | { state: "no_previous"; changes: [] }
  | { state: "no_supported_change"; changes: [] }
  | { state: "changes"; changes: DiagnosticChange[] };

function hasUnsafeText(value: string): boolean {
  return (
    value.trim() !== value ||
    value.length === 0 ||
    [...value].some((character) => {
      const code = character.charCodeAt(0);
      return code <= 31 || code === 127;
    })
  );
}

function looksLikePrivatePath(value: string): boolean {
  return (
    value.startsWith("/") ||
    value.startsWith("~") ||
    value.startsWith("\\\\") ||
    /^[A-Za-z]:[\\/]/.test(value) ||
    /(^|[\\/])\.\.([\\/]|$)/.test(value) ||
    value.includes("://")
  );
}

function isSafeDiagnosticIdentity(value: string): boolean {
  return (
    value.length <= MAX_DIAGNOSTIC_ID_CHARS &&
    !hasUnsafeText(value) &&
    !looksLikePrivatePath(value)
  );
}

function safeRuntimeVersion(value: string | null): string | null {
  if (value === null) return null;
  return value.length <= MAX_RUNTIME_VERSION_CHARS &&
    !hasUnsafeText(value) &&
    !looksLikePrivatePath(value)
    ? value
    : null;
}

function boundedIds(values: readonly string[]): string[] | null {
  if (values.length > MAX_DIAGNOSTIC_IDENTITIES) return null;
  if (values.some((value) => !isSafeDiagnosticIdentity(value))) return null;
  return [...values].sort((a, b) => a.localeCompare(b, "en"));
}

function ollamaRuntimeSource(
  view: RuntimeStatusView | null,
): DiagnosticSource<DiagnosticOllamaRuntimeObservation> {
  if (view === null || view.runtime_type !== "ollama") return { kind: "gap" };
  return {
    kind: "observed",
    value: {
      state: view.source_availability,
      version: safeRuntimeVersion(view.version),
    },
  };
}

function ollamaModelSetSource(
  view: ModelInventoryView | LoadedModelSetView | null,
): DiagnosticSource<DiagnosticOllamaModelSetObservation> {
  if (view === null || view.runtime_type !== "ollama") return { kind: "gap" };
  const ids = boundedIds(view.models.map((model) => model.display_name));
  if (ids === null) return { kind: "gap" };
  return { kind: "observed", value: { state: view.state, model_ids: ids } };
}

function lmStudioSource(
  view: LmStudioSnapshotView | null,
): DiagnosticSource<DiagnosticLmStudioObservation> {
  if (view === null || view.provider !== "lm_studio") return { kind: "gap" };
  if (view.models.length > MAX_DIAGNOSTIC_IDENTITIES) return { kind: "gap" };
  let instanceCount = 0;
  const models: DiagnosticLmStudioModelObservation[] = [];
  for (const model of view.models) {
    if (!isSafeDiagnosticIdentity(model.model_id)) return { kind: "gap" };
    const loadedInstanceIds = boundedIds(
      model.loaded_instances.map((instance) => instance.instance_id),
    );
    if (loadedInstanceIds === null) return { kind: "gap" };
    instanceCount += loadedInstanceIds.length;
    if (instanceCount > MAX_DIAGNOSTIC_INSTANCES) return { kind: "gap" };
    models.push({
      model_id: model.model_id,
      inference_eligible: model.inference_eligible,
      loaded_instance_ids: loadedInstanceIds,
    });
  }
  models.sort((a, b) => a.model_id.localeCompare(b.model_id, "en"));
  return { kind: "observed", value: { state: view.state, models } };
}

function llamaCppSource(
  view: LlamaCppSnapshotView | null,
): DiagnosticSource<DiagnosticLlamaCppObservation> {
  if (view === null || view.provider !== "llama_cpp") return { kind: "gap" };
  const id = view.served_model?.model_id ?? null;
  if (id !== null && !isSafeDiagnosticIdentity(id)) return { kind: "gap" };
  return {
    kind: "observed",
    value: { state: view.state, served_model_id: id },
  };
}

export function createDiagnosticObservationDraft(
  snapshot: SnapshotView | null,
  snapshotRejected: boolean,
  runtime: RuntimeStatusView | null,
  catalogue: ModelInventoryView | null,
  loaded: LoadedModelSetView | null,
  lmStudio: LmStudioSnapshotView | null,
  llamaCpp: LlamaCppSnapshotView | null,
): DiagnosticObservationDraft {
  return {
    available_memory: eventFromAvailableMemorySnapshot(
      snapshot,
      snapshotRejected ? "snapshot_rejected" : "snapshot_unavailable",
    ),
    ollama_runtime: ollamaRuntimeSource(runtime),
    ollama_catalogue: ollamaModelSetSource(catalogue),
    ollama_loaded: ollamaModelSetSource(loaded),
    lm_studio: lmStudioSource(lmStudio),
    llama_cpp: llamaCppSource(llamaCpp),
  };
}

export function appendDiagnosticObservation(
  history: readonly DiagnosticObservationBundle[],
  draft: DiagnosticObservationDraft,
  nextObservationId: number,
): AppendDiagnosticObservationResult {
  return {
    history: [
      ...history,
      { ...draft, observation_id: nextObservationId },
    ].slice(-DIAGNOSTIC_OBSERVATION_LIMIT),
    nextObservationId: nextObservationId + 1,
  };
}

function sourceTransition<T>(
  prior: DiagnosticSource<T>,
  current: DiagnosticSource<T>,
  source: DiagnosticChangeSource,
  label: string,
): DiagnosticChange | null {
  if (prior.kind === current.kind) return null;
  return {
    code: `${source}_source_availability_changed`,
    source,
    summary:
      current.kind === "gap"
        ? `${label} changed from a controlled observation to a source gap.`
        : `${label} changed from a source gap to a controlled observation.`,
    details: [],
  };
}

function sortedSet(values: readonly string[]): string[] {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b, "en"));
}

function setDelta(
  prior: readonly string[],
  current: readonly string[],
): { added: string[]; removed: string[] } {
  const before = new Set(prior);
  const after = new Set(current);
  return {
    added: sortedSet(current.filter((value) => !before.has(value))),
    removed: sortedSet(prior.filter((value) => !after.has(value))),
  };
}

function setChange(
  code: string,
  source: DiagnosticChangeSource,
  label: string,
  prior: readonly string[],
  current: readonly string[],
): DiagnosticChange | null {
  const { added, removed } = setDelta(prior, current);
  if (
    added.length === 0 &&
    removed.length === 0 &&
    prior.length === current.length
  )
    return null;
  return {
    code,
    source,
    summary: `${label} changed from ${prior.length} to ${current.length}.`,
    details: [
      ...added.map((id) => `Added: ${id}`),
      ...removed.map((id) => `Removed: ${id}`),
    ],
  };
}

interface LmStudioLoadedMembership {
  model_id: string;
  instance_id: string;
}

function compareIdentity(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function compareLoadedMembership(
  left: LmStudioLoadedMembership,
  right: LmStudioLoadedMembership,
): number {
  return (
    compareIdentity(left.model_id, right.model_id) ||
    compareIdentity(left.instance_id, right.instance_id)
  );
}

function sameLoadedMembership(
  left: LmStudioLoadedMembership,
  right: LmStudioLoadedMembership,
): boolean {
  return (
    left.model_id === right.model_id && left.instance_id === right.instance_id
  );
}

function uniqueLoadedMemberships(
  values: readonly LmStudioLoadedMembership[],
): LmStudioLoadedMembership[] {
  return values
    .filter(
      (value, index) =>
        values.findIndex((candidate) =>
          sameLoadedMembership(candidate, value),
        ) === index,
    )
    .sort(compareLoadedMembership);
}

function loadedMembershipDelta(
  prior: readonly LmStudioLoadedMembership[],
  current: readonly LmStudioLoadedMembership[],
): {
  added: LmStudioLoadedMembership[];
  removed: LmStudioLoadedMembership[];
} {
  return {
    added: uniqueLoadedMemberships(
      current.filter(
        (value) =>
          !prior.some((candidate) => sameLoadedMembership(candidate, value)),
      ),
    ),
    removed: uniqueLoadedMemberships(
      prior.filter(
        (value) =>
          !current.some((candidate) => sameLoadedMembership(candidate, value)),
      ),
    ),
  };
}

function loadedMembershipDetail(
  disposition: "Added" | "Removed",
  membership: LmStudioLoadedMembership,
): string {
  return `${disposition}: model ID ${JSON.stringify(membership.model_id)}, instance ID ${JSON.stringify(membership.instance_id)}`;
}

function loadedMembershipChange(
  prior: readonly LmStudioLoadedMembership[],
  current: readonly LmStudioLoadedMembership[],
): DiagnosticChange | null {
  const { added, removed } = loadedMembershipDelta(prior, current);
  if (
    added.length === 0 &&
    removed.length === 0 &&
    prior.length === current.length
  )
    return null;
  return {
    code: "lm_studio_loaded_instances_changed",
    source: "lm_studio",
    summary: `LM Studio's reported loaded-instance membership changed from ${prior.length} to ${current.length}.`,
    details: [
      ...added.map((membership) => loadedMembershipDetail("Added", membership)),
      ...removed.map((membership) =>
        loadedMembershipDetail("Removed", membership),
      ),
    ],
  };
}

function memoryText(event: AvailableMemoryEvent): string {
  if (event.kind === "value") {
    return `${formatBytesFriendly(event.bytes)} (${formatExactBytes(event.bytes)})`;
  }
  return `No numeric value — ${unavailableReasonLabel(event.reason)}`;
}

function availableMemoryChanges(
  prior: AvailableMemoryEvent,
  current: AvailableMemoryEvent,
): DiagnosticChange[] {
  if (prior.kind === "value" && current.kind === "value") {
    if (prior.bytes === current.bytes) return [];
    return [
      {
        code: "available_memory_value_changed",
        source: "available_memory",
        summary: "The Available-memory numeric observation changed.",
        details: [
          `Previous: ${memoryText(prior)}`,
          `Current: ${memoryText(current)}`,
        ],
      },
    ];
  }
  if (prior.kind !== current.kind) {
    return [
      {
        code: "available_memory_value_gap_changed",
        source: "available_memory",
        summary:
          current.kind === "value"
            ? "Available memory changed from a controlled gap to a numeric observation."
            : "Available memory changed from a numeric observation to a controlled gap.",
        details: [
          `Previous: ${memoryText(prior)}`,
          `Current: ${memoryText(current)}`,
        ],
      },
    ];
  }
  if (
    prior.kind === "unavailable" &&
    current.kind === "unavailable" &&
    prior.reason !== current.reason
  ) {
    return [
      {
        code: "available_memory_gap_state_changed",
        source: "available_memory",
        summary: "The controlled Available-memory gap state changed.",
        details: [
          `Previous: ${memoryText(prior)}`,
          `Current: ${memoryText(current)}`,
        ],
      },
    ];
  }
  return [];
}

function ollamaRuntimeChanges(
  prior: DiagnosticObservationBundle,
  current: DiagnosticObservationBundle,
): DiagnosticChange[] {
  const transition = sourceTransition(
    prior.ollama_runtime,
    current.ollama_runtime,
    "ollama_runtime",
    "Ollama runtime observation",
  );
  if (transition) return [transition];
  if (
    prior.ollama_runtime.kind === "gap" ||
    current.ollama_runtime.kind === "gap"
  )
    return [];
  const changes: DiagnosticChange[] = [];
  if (prior.ollama_runtime.value.state !== current.ollama_runtime.value.state) {
    changes.push({
      code: "ollama_runtime_state_changed",
      source: "ollama_runtime",
      summary: `Ollama's controlled runtime state changed from ${prior.ollama_runtime.value.state} to ${current.ollama_runtime.value.state}.`,
      details: [],
    });
  }
  const priorVersion = prior.ollama_runtime.value.version;
  const currentVersion = current.ollama_runtime.value.version;
  if (
    priorVersion !== null &&
    currentVersion !== null &&
    priorVersion !== currentVersion
  ) {
    changes.push({
      code: "ollama_reported_version_changed",
      source: "ollama_runtime",
      summary: "Ollama's reported version changed.",
      details: [`Previous: ${priorVersion}`, `Current: ${currentVersion}`],
    });
  }
  return changes;
}

function ollamaModelSetChanges(
  priorSource: DiagnosticSource<DiagnosticOllamaModelSetObservation>,
  currentSource: DiagnosticSource<DiagnosticOllamaModelSetObservation>,
  source: "ollama_catalogue" | "ollama_loaded",
  label: string,
): DiagnosticChange[] {
  const transition = sourceTransition(
    priorSource,
    currentSource,
    source,
    label,
  );
  if (transition) return [transition];
  if (priorSource.kind === "gap" || currentSource.kind === "gap") return [];
  const changes: DiagnosticChange[] = [];
  if (priorSource.value.state !== currentSource.value.state) {
    changes.push({
      code: `${source}_state_changed`,
      source,
      summary: `${label}'s controlled state changed from ${priorSource.value.state} to ${currentSource.value.state}.`,
      details: [],
    });
  }
  if (
    priorSource.value.state === "available" &&
    currentSource.value.state === "available"
  ) {
    const change = setChange(
      `${source}_set_changed`,
      source,
      label,
      priorSource.value.model_ids,
      currentSource.value.model_ids,
    );
    if (change) changes.push(change);
  }
  return changes;
}

function lmStudioChanges(
  prior: DiagnosticObservationBundle,
  current: DiagnosticObservationBundle,
): DiagnosticChange[] {
  const transition = sourceTransition(
    prior.lm_studio,
    current.lm_studio,
    "lm_studio",
    "LM Studio observation",
  );
  if (transition) return [transition];
  if (prior.lm_studio.kind === "gap" || current.lm_studio.kind === "gap")
    return [];
  const changes: DiagnosticChange[] = [];
  if (prior.lm_studio.value.state !== current.lm_studio.value.state) {
    changes.push({
      code: "lm_studio_state_changed",
      source: "lm_studio",
      summary: `LM Studio's controlled state changed from ${prior.lm_studio.value.state} to ${current.lm_studio.value.state}.`,
      details: [],
    });
  }
  if (
    prior.lm_studio.value.state !== "available" ||
    current.lm_studio.value.state !== "available"
  )
    return changes;

  const catalogueChange = setChange(
    "lm_studio_catalogue_set_changed",
    "lm_studio",
    "LM Studio's reported model set",
    prior.lm_studio.value.models.map((model) => model.model_id),
    current.lm_studio.value.models.map((model) => model.model_id),
  );
  if (catalogueChange) changes.push(catalogueChange);

  const loadedMemberships = (models: DiagnosticLmStudioModelObservation[]) =>
    models.flatMap((model) =>
      model.loaded_instance_ids.map((instanceId) => ({
        model_id: model.model_id,
        instance_id: instanceId,
      })),
    );
  const instanceChange = loadedMembershipChange(
    loadedMemberships(prior.lm_studio.value.models),
    loadedMemberships(current.lm_studio.value.models),
  );
  if (instanceChange) changes.push(instanceChange);
  return changes;
}

function llamaCppChanges(
  prior: DiagnosticObservationBundle,
  current: DiagnosticObservationBundle,
): DiagnosticChange[] {
  const transition = sourceTransition(
    prior.llama_cpp,
    current.llama_cpp,
    "llama_cpp",
    "llama.cpp observation",
  );
  if (transition) return [transition];
  if (prior.llama_cpp.kind === "gap" || current.llama_cpp.kind === "gap")
    return [];
  const changes: DiagnosticChange[] = [];
  if (prior.llama_cpp.value.state !== current.llama_cpp.value.state) {
    changes.push({
      code: "llama_cpp_state_changed",
      source: "llama_cpp",
      summary: `llama.cpp's controlled state changed from ${prior.llama_cpp.value.state} to ${current.llama_cpp.value.state}.`,
      details: [],
    });
  }
  const priorId = prior.llama_cpp.value.served_model_id;
  const currentId = current.llama_cpp.value.served_model_id;
  if (
    prior.llama_cpp.value.state === "ready" &&
    current.llama_cpp.value.state === "ready" &&
    priorId !== null &&
    currentId !== null &&
    priorId !== currentId
  ) {
    changes.push({
      code: "llama_cpp_served_model_changed",
      source: "llama_cpp",
      summary: "llama.cpp's safe provider-reported served-model ID changed.",
      details: [`Previous: ${priorId}`, `Current: ${currentId}`],
    });
  }
  return changes;
}

export function diagnosticDeltas(
  prior: DiagnosticObservationBundle | null,
  current: DiagnosticObservationBundle,
): DiagnosticDeltaResult {
  if (prior === null) return { state: "no_previous", changes: [] };
  const changes = [
    ...availableMemoryChanges(prior.available_memory, current.available_memory),
    ...ollamaRuntimeChanges(prior, current),
    ...ollamaModelSetChanges(
      prior.ollama_catalogue,
      current.ollama_catalogue,
      "ollama_catalogue",
      "Ollama catalogue",
    ),
    ...ollamaModelSetChanges(
      prior.ollama_loaded,
      current.ollama_loaded,
      "ollama_loaded",
      "Ollama reported running-model set",
    ),
    ...lmStudioChanges(prior, current),
    ...llamaCppChanges(prior, current),
  ];
  return changes.length === 0
    ? { state: "no_supported_change", changes: [] }
    : { state: "changes", changes };
}

function availableMemoryInput(
  event: AvailableMemoryEvent,
): DiagnosticInput["available_memory"] {
  if (event.kind === "value") return { kind: "value" };
  return event.reason === "snapshot_rejected" ||
    event.reason === "snapshot_unavailable"
    ? { kind: "source_gap" }
    : { kind: "controlled_gap" };
}

export function diagnosisInput(
  bundle: DiagnosticObservationBundle,
  deltas: DiagnosticDeltaResult,
): DiagnosticInput {
  return {
    available_memory: availableMemoryInput(bundle.available_memory),
    ollama_runtime:
      bundle.ollama_runtime.kind === "gap"
        ? { kind: "gap" }
        : { kind: "observed", ...bundle.ollama_runtime.value },
    ollama_catalogue:
      bundle.ollama_catalogue.kind === "gap"
        ? { kind: "gap" }
        : { kind: "observed", ...bundle.ollama_catalogue.value },
    ollama_loaded:
      bundle.ollama_loaded.kind === "gap"
        ? { kind: "gap" }
        : { kind: "observed", ...bundle.ollama_loaded.value },
    lm_studio:
      bundle.lm_studio.kind === "gap"
        ? { kind: "gap" }
        : { kind: "observed", ...bundle.lm_studio.value },
    llama_cpp:
      bundle.llama_cpp.kind === "gap"
        ? { kind: "gap" }
        : { kind: "observed", ...bundle.llama_cpp.value },
    supported_change_count:
      deltas.state === "changes" ? deltas.changes.length : 0,
  };
}

export function sourceCoverage(
  bundle: DiagnosticObservationBundle,
): { source: string; state: string; gap: boolean }[] {
  const memory = bundle.available_memory;
  return [
    {
      source: "Available memory",
      state:
        memory.kind === "value"
          ? "Numeric observation"
          : unavailableReasonLabel(memory.reason),
      gap:
        memory.kind === "unavailable" &&
        (memory.reason === "snapshot_rejected" ||
          memory.reason === "snapshot_unavailable"),
    },
    ...(
      [
        ["Ollama runtime", bundle.ollama_runtime],
        ["Ollama catalogue", bundle.ollama_catalogue],
        ["Ollama loaded state", bundle.ollama_loaded],
        ["LM Studio", bundle.lm_studio],
        ["llama.cpp", bundle.llama_cpp],
      ] as const
    ).map(([source, observation]) => ({
      source,
      state:
        observation.kind === "gap"
          ? "Source gap"
          : `Controlled state: ${observation.value.state}`,
      gap: observation.kind === "gap",
    })),
  ];
}
