<script lang="ts">
  import { onMount } from "svelte";
  import type { DataSource } from "./lib/datasource";
  import type {
    LoadedModelSetView,
    ModelInventoryView,
    ResourceContextView,
    RuntimeStatusView,
    SnapshotView,
    InferenceObservationView,
    LlamaCppSnapshotView,
    LmStudioSnapshotView,
    DiagnosticFindingView,
    MachineContextView,
    ReportSaveResult,
  } from "./lib/types";
  import {
    appendCompletedObservation,
    pruneSelectedObservationIds,
    type ObservationHistoryItem,
  } from "./lib/observationHistory";
  import {
    appendAvailableMemoryObservation,
    type AvailableMemoryHistoryItem,
  } from "./lib/availableMemoryHistory";
  import {
    appendDiagnosticObservation,
    createDiagnosticObservationDraft,
    diagnosticDeltas,
    diagnosisInput,
    type DiagnosticDeltaResult,
    type DiagnosticObservationBundle,
  } from "./lib/diagnosticObservation";
  import { createDataSource } from "./lib/datasource";
  import {
    createNativeReportClipboard,
    type ReportClipboard,
  } from "./lib/reportClipboard";
  import AppShell, { type Workspace } from "./lib/components/AppShell.svelte";
  import OverviewDashboard from "./lib/components/OverviewDashboard.svelte";
  import ModelsWorkspace from "./lib/components/ModelsWorkspace.svelte";
  import ObservedInference from "./lib/components/ObservedInference.svelte";
  import RecentObservations from "./lib/components/RecentObservations.svelte";
  import ReportWorkspace from "./lib/components/ReportWorkspace.svelte";
  import DiagnoseWorkspace from "./lib/components/DiagnoseWorkspace.svelte";

  type ReportClipboardState =
    "browser-unavailable" | "native-ready" | "native-init-failed";

  let {
    dataSource,
    reportClipboard: suppliedReportClipboard,
  }: {
    dataSource?: DataSource;
    reportClipboard?: ReportClipboard | null;
  } = $props();
  let reportClipboard = $state<ReportClipboard | null>(null);
  let reportClipboardState = $state<ReportClipboardState>(
    "browser-unavailable",
  );
  let snapshot = $state<SnapshotView | null>(null);
  let machineContext = $state<MachineContextView | null>(null);
  let reportPreview = $state("");
  let reportGeneration = $state("");
  let runtimeStatus = $state<RuntimeStatusView | null>(null);
  let modelInventory = $state<ModelInventoryView | null>(null);
  let loadedModels = $state<LoadedModelSetView | null>(null);
  let lmStudio = $state<LmStudioSnapshotView | null>(null);
  let llamaCpp = $state<LlamaCppSnapshotView | null>(null);
  let resourceContext = $state<ResourceContextView | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(true);
  let activeWorkspace = $state<Workspace>("overview");
  let currentObservation = $state<InferenceObservationView | null>(null);
  let recentObservations = $state<ObservationHistoryItem[]>([]);
  let nextObservationId = $state(1);
  let selectedObservationIds = $state<number[]>([]);
  let retainedObservationAnnouncement = $state("");
  let availableMemoryHistory = $state<AvailableMemoryHistoryItem[]>([]);
  let nextAvailableMemoryObservationId = $state(1);
  let diagnosticHistory = $state<DiagnosticObservationBundle[]>([]);
  let nextDiagnosticObservationId = $state(1);
  let diagnosticChanges = $state<DiagnosticDeltaResult>({
    state: "no_previous",
    changes: [],
  });
  let diagnosticFindings = $state<DiagnosticFindingView[]>([]);
  let diagnosisState = $state<"available" | "invalid_input" | "unavailable">(
    "unavailable",
  );
  let refreshPending = $state(false);

  function retainCompletedObservation(observation: InferenceObservationView) {
    const observationId = nextObservationId;
    const result = appendCompletedObservation(
      recentObservations,
      observation,
      nextObservationId,
    );
    recentObservations = result.history;
    nextObservationId = result.nextObservationId;
    if (result.retained)
      retainedObservationAnnouncement = `Observation ${observationId} added to recent observations.`;
    selectedObservationIds = pruneSelectedObservationIds(
      selectedObservationIds,
      recentObservations,
    );
  }
  function clearRecentObservations() {
    recentObservations = [];
    selectedObservationIds = [];
    retainedObservationAnnouncement = "";
  }
  async function acquire<T>(operation: () => Promise<T>): Promise<T | null> {
    try {
      return await operation();
    } catch {
      return null;
    }
  }
  async function acquireSnapshot(
    operation: () => Promise<SnapshotView>,
  ): Promise<{ snapshot: SnapshotView | null; rejected: boolean }> {
    try {
      return { snapshot: await operation(), rejected: false };
    } catch {
      return { snapshot: null, rejected: true };
    }
  }
  async function refresh(ds: DataSource) {
    if (refreshPending) return;
    refreshPending = true;
    loading = true;
    error = null;
    try {
      const [snapshotResult, machine, preview, rtStatus, inventory, loaded, lm, llama] =
        await Promise.all([
          acquireSnapshot(() => ds.currentSnapshot()),
          acquire(() => ds.machineContext()),
          acquire(() => ds.reportPreview()),
          acquire(() => ds.runtimeStatus()),
          acquire(() => ds.modelInventory()),
          acquire(() => ds.loadedModels()),
          ds.lmStudioSnapshot
            ? acquire(() => ds.lmStudioSnapshot!())
            : Promise.resolve(null),
          ds.llamaCppSnapshot
            ? acquire(() => ds.llamaCppSnapshot!())
            : Promise.resolve(null),
        ]);
      const snap = snapshotResult.snapshot;
      const appended = appendAvailableMemoryObservation(
        availableMemoryHistory,
        snap,
        nextAvailableMemoryObservationId,
        snapshotResult.rejected ? "snapshot_rejected" : "snapshot_unavailable",
      );
      availableMemoryHistory = appended.history;
      nextAvailableMemoryObservationId = appended.nextObservationId;
      snapshot = snap;
      machineContext = machine;
      reportPreview =
        typeof preview === "string" ? preview : (preview?.text ?? "");
      reportGeneration =
        typeof preview === "string" ? "" : (preview?.generation ?? "");
      runtimeStatus = rtStatus;
      modelInventory = inventory;
      loadedModels = loaded;
      lmStudio = lm;
      llamaCpp = llama;
      resourceContext =
        snap && loaded
          ? await acquire(() => ds.composeResourceContext(snap, loaded))
          : null;

      const draft = createDiagnosticObservationDraft(
        snap,
        snapshotResult.rejected,
        rtStatus,
        inventory,
        loaded,
        lm,
        llama,
      );
      const currentBundle: DiagnosticObservationBundle = {
        observation_id: nextDiagnosticObservationId,
        ...draft,
      };
      const priorBundle = diagnosticHistory.at(-1) ?? null;
      const changes = diagnosticDeltas(priorBundle, currentBundle);
      const diagnosis = await acquire(() =>
        ds.diagnoseObservation(diagnosisInput(currentBundle, changes)),
      );
      const diagnosticAppend = appendDiagnosticObservation(
        diagnosticHistory,
        draft,
        nextDiagnosticObservationId,
      );
      diagnosticHistory = diagnosticAppend.history;
      nextDiagnosticObservationId = diagnosticAppend.nextObservationId;
      diagnosticChanges = changes;
      diagnosisState = diagnosis?.state ?? "unavailable";
      diagnosticFindings = diagnosis?.findings ?? [];

      if (
        !snapshot &&
        !machineContext &&
        !runtimeStatus &&
        !modelInventory &&
        !loadedModels &&
        !lmStudio &&
        !llamaCpp
      ) {
        error = "Could not load metric data.";
      }
    } catch {
      error = "Could not load metric data.";
    } finally {
      loading = false;
      refreshPending = false;
    }
  }
  onMount(async () => {
    let ds = dataSource;
    if (!ds) {
      try {
        ds = await createDataSource();
      } catch {
        error = "Could not load metric data.";
        loading = false;
        return;
      }
      dataSource = ds;
    }
    if (ds.isMock) {
      reportClipboard = null;
      reportClipboardState = "browser-unavailable";
    } else if (suppliedReportClipboard !== undefined) {
      reportClipboard = suppliedReportClipboard;
      reportClipboardState = suppliedReportClipboard
        ? "native-ready"
        : "native-init-failed";
    } else {
      try {
        reportClipboard = await createNativeReportClipboard();
        reportClipboardState = "native-ready";
      } catch {
        reportClipboard = null;
        reportClipboardState = "native-init-failed";
      }
    }
    await refresh(ds);
  });
  async function handleRefresh() {
    if (dataSource) await refresh(dataSource);
  }
</script>

<AppShell
  bind:activeWorkspace
  isMock={dataSource?.isMock ?? false}
  onRefresh={handleRefresh}
  refreshing={refreshPending}
>
  {#if loading}
    <p class="state-message" role="status">Loading metric data…</p>
  {:else if error}
    <p class="state-message" role="alert">{error}</p>
  {:else}
    <div
      id="workspace-panel-overview"
      role="tabpanel"
      aria-labelledby="workspace-tab-overview"
      hidden={activeWorkspace !== "overview"}
    >
      <OverviewDashboard
        {snapshot}
        {machineContext}
        {runtimeStatus}
        {modelInventory}
        {loadedModels}
        {resourceContext}
        {currentObservation}
        {recentObservations}
        {lmStudio}
        {llamaCpp}
        {availableMemoryHistory}
      />
    </div>
    <div
      id="workspace-panel-models"
      role="tabpanel"
      aria-labelledby="workspace-tab-models"
      hidden={activeWorkspace !== "models"}
    >
      <ModelsWorkspace {loadedModels} {modelInventory} {lmStudio} {llamaCpp} />
    </div>
    <div
      id="workspace-panel-observed"
      role="tabpanel"
      aria-labelledby="workspace-tab-observed"
      hidden={activeWorkspace !== "observed"}
    >
      <section
        class="workspace-content"
        aria-labelledby="observed-workspace-heading"
      >
        <h2 id="observed-workspace-heading">Observed inference</h2>
        {#if modelInventory && dataSource}
          <ObservedInference
            inventory={modelInventory}
            {lmStudio}
            {dataSource}
            isActive={activeWorkspace === "observed"}
            onObservation={(value) => (currentObservation = value)}
            onCompletedObservation={retainCompletedObservation}
          />
          <RecentObservations
            history={recentObservations}
            bind:selectedObservationIds
            retainedAnnouncement={retainedObservationAnnouncement}
            onClear={clearRecentObservations}
          />
        {/if}
      </section>
    </div>
    <div
      id="workspace-panel-diagnose"
      role="tabpanel"
      aria-labelledby="workspace-tab-diagnose"
      hidden={activeWorkspace !== "diagnose"}
    >
      <DiagnoseWorkspace
        history={diagnosticHistory}
        changes={diagnosticChanges}
        findings={diagnosticFindings}
        {diagnosisState}
      />
    </div>
    <div
      id="workspace-panel-report"
      role="tabpanel"
      aria-labelledby="workspace-tab-report"
      hidden={activeWorkspace !== "report"}
    >
      <ReportWorkspace
        preview={reportPreview}
        generation={reportGeneration}
        saveReport={!dataSource?.isMock && dataSource?.saveReport
          ? (generation: string): Promise<ReportSaveResult> =>
              dataSource!.saveReport!(generation)
          : null}
        clipboard={reportClipboard}
        clipboardState={reportClipboardState}
      />
    </div>
  {/if}
</AppShell>
