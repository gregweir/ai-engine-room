<script lang="ts">
  import type {
    InferenceObservationView,
    LoadedModelSetView,
    ModelInventoryView,
    ResourceContextView,
    RuntimeStatusView,
    SnapshotEntryView,
    SnapshotView,
    LmStudioSnapshotView,
    LlamaCppSnapshotView,
    MachineContextView,
  } from "../types";
  import type { ObservationHistoryItem } from "../observationHistory";
  import type { AvailableMemoryHistoryItem } from "../availableMemoryHistory";
  import { AVAILABLE_MEMORY_DISPLAY_NAME } from "../availableMemoryHistory";
  import { formatBytesFriendly, formatExactBytes } from "../formatBytes";
  import AvailableMemoryHistory from "./AvailableMemoryHistory.svelte";
  import EvidenceLabel from "./EvidenceLabel.svelte";
  import ResourceContext from "./ResourceContext.svelte";

  let {
    snapshot,
    machineContext,
    runtimeStatus,
    modelInventory,
    loadedModels,
    resourceContext,
    currentObservation,
    recentObservations,
    lmStudio,
    llamaCpp,
    availableMemoryHistory,
  }: {
    snapshot: SnapshotView | null;
    machineContext: MachineContextView | null;
    runtimeStatus: RuntimeStatusView | null;
    modelInventory: ModelInventoryView | null;
    loadedModels: LoadedModelSetView | null;
    resourceContext: ResourceContextView | null;
    currentObservation: InferenceObservationView | null;
    recentObservations: ObservationHistoryItem[];
    lmStudio: LmStudioSnapshotView | null;
    llamaCpp: LlamaCppSnapshotView | null;
    availableMemoryHistory: AvailableMemoryHistoryItem[];
  } = $props();

  let availableMemory = $derived(
    snapshot?.entries.find(
      (entry) => entry.display_name === AVAILABLE_MEMORY_DISPLAY_NAME,
    ),
  );
  let totalMemory = $derived(
    snapshot?.entries.find((entry) => entry.display_name === "Total memory"),
  );

  function exactValue(entry: SnapshotEntryView): string | null {
    return entry.outcome.kind === "ok"
      ? formatExactBytes(entry.outcome.value)
      : null;
  }

  function friendlyValue(entry: SnapshotEntryView): string | null {
    if (entry.outcome.kind !== "ok") return null;
    const formatted = formatBytesFriendly(entry.outcome.value);
    if (formatted === null) return null;
    return exactValue(entry) === null ? `≈${formatted}` : formatted;
  }
</script>

<section class="workspace-content" aria-labelledby="overview-heading">
  <h2 id="overview-heading">Overview</h2>
  <p class="workspace-intro">
    Current state at a glance from the existing data snapshot.
  </p>

  <div class="summary-grid">
    <article class="summary-card">
      <EvidenceLabel label="Provider-reported state" />
      <h3>Ollama runtime</h3>
      {#if runtimeStatus}
        <p class="summary-primary">{runtimeStatus.status_label}</p>
        {#if runtimeStatus.version}<p>
            Version {runtimeStatus.version} (reported by the runtime)
          </p>{/if}
      {:else}<p class="summary-unavailable">Unavailable</p>{/if}
    </article>
    <article class="summary-card">
      <EvidenceLabel label="Provider-reported state" />
      <h3>LM Studio runtime</h3>
      {#if lmStudio}
        <p class="summary-primary">{lmStudio.state_label}</p>
        <p>{lmStudio.interpretation}</p>
      {:else}<p class="summary-unavailable">Unavailable</p>{/if}
    </article>
    <article class="summary-card">
      <EvidenceLabel label="Provider-reported state" />
      <h3>llama.cpp runtime</h3>
      {#if llamaCpp}
        <p class="summary-primary">{llamaCpp.state_label}</p>
        <p>{llamaCpp.interpretation}</p>
        {#if llamaCpp.state === "ready" && llamaCpp.served_model}
          <p>Served model: {llamaCpp.served_model.model_id}</p>
        {/if}
      {:else}<p class="summary-unavailable">Unavailable</p>{/if}
    </article>
    <article class="summary-card">
      <EvidenceLabel label="OS observation" />
      <h3>Available memory</h3>
      {#if availableMemory && friendlyValue(availableMemory)}
        <p class="summary-primary">
          {exactValue(availableMemory) ?? "Exact value not representable"}
        </p>
        <p>{friendlyValue(availableMemory)}</p>
        <p class="summary-source">Reported by the operating system</p>
      {:else}<p class="summary-unavailable">
          Unavailable — no value reported
        </p>{/if}
    </article>
    <article class="summary-card">
      <EvidenceLabel label="OS observation" />
      <h3>Total memory</h3>
      {#if totalMemory && friendlyValue(totalMemory)}
        <p class="summary-primary">
          {exactValue(totalMemory) ?? "Exact value not representable"}
        </p>
        <p>{friendlyValue(totalMemory)}</p>
        <p class="summary-source">Reported by the operating system</p>
        {#if totalMemory.deeper}<p class="summary-note">{totalMemory.deeper}</p>{/if}
      {:else}<p class="summary-unavailable">
          Unavailable — no value reported
        </p>{/if}
    </article>
    <article class="summary-card">
      <EvidenceLabel label="OS observation" />
      <h3>Native CPU architecture</h3>
      {#if machineContext?.state === "available" && machineContext.native_cpu_architecture}
        <p class="summary-primary">{machineContext.native_cpu_architecture}</p>
        <p>{machineContext.interpretation}</p>
        <p class="summary-note">{machineContext.qualification}</p>
      {:else}<p class="summary-unavailable">
          Unavailable — no value reported
        </p>{/if}
    </article>
    <article class="summary-card">
      <h3>Catalogue models reported</h3>
      {#if modelInventory?.state === "available"}
        <p class="summary-count">{modelInventory.models.length}</p>
      {:else}<p class="summary-unavailable">Unavailable</p>{/if}
    </article>
    <article class="summary-card">
      <h3>Currently loaded models</h3>
      {#if loadedModels?.state === "available"}
        <p class="summary-count">{loadedModels.models.length}</p>
        <p class="summary-note">
          Loaded state is not proof of local execution.
        </p>
      {:else}<p class="summary-unavailable">Unavailable</p>{/if}
    </article>
    <article class="summary-card summary-observation">
      <EvidenceLabel label="Observed inference" />
      <h3>Current observed inference state</h3>
      {#if currentObservation}
        <p class="summary-primary">{currentObservation.interpretation}</p>
        <p>Execution boundary: {currentObservation.execution_boundary}</p>
      {:else}<p class="summary-primary">No observation run this session</p>{/if}
      <p class="summary-note">
        {recentObservations.length === 0
          ? "None yet"
          : `${recentObservations.length} retained this session`}
      </p>
    </article>
  </div>

  <AvailableMemoryHistory history={availableMemoryHistory} />

  {#if resourceContext && snapshot}<ResourceContext
      context={resourceContext}
      entries={snapshot.entries}
    />{/if}
</section>
