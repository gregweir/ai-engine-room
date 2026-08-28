// Data-source boundary: one interface used everywhere, so the frontend does not
// know whether its data comes from Tauri IPC or browser-development fixtures.
//
// Detection uses the supported, first-party `isTauri()` from
// `@tauri-apps/api/core` (documented at v2.tauri.app), never a brittle
// undocumented global. In the native Tauri app the IPC path is selected and
// only Stage 8 DTOs are consumed. In ordinary browser development the
// fixture/mock path is selected — it never calls Tauri `invoke`, never touches
// Ollama or real system telemetry, and never infers localhost services. In a
// production browser build with no Tauri and no dev mode, there is no data
// source and the factory throws rather than guessing.

import { isTauri } from "@tauri-apps/api/core";
import type {
  DiagnosisResultView,
  DiagnosticInput,
  InferenceObservationRequest,
  InferenceObservationView,
  LoadedModelSetView,
  LlamaCppSnapshotView,
  LmStudioSnapshotView,
  MachineContextView,
  ModelInventoryView,
  ResourceContextView,
  RuntimeStatusView,
  SnapshotView,
} from "../types";

/** A source of metric data for the UI. Implementations hide whether the data
 * comes from Tauri IPC or browser-development fixtures. `isMock` lets the UI
 * honestly identify fixture/preview data so it is never mistaken for real
 * system information. */
export interface DataSource {
  readonly isMock: boolean;
  currentSnapshot(): Promise<SnapshotView>;
  machineContext(): Promise<MachineContextView>;
  reportPreview(): Promise<string>;
  /** The current detection status of the supported AI runtime (Ollama in 1C):
   * controlled availability, the verbatim version when present, and
   * pre-computed controlled text. Runtime metadata, not a metric. */
  runtimeStatus(): Promise<RuntimeStatusView>;
  /** The current read-only model inventory for the supported AI runtime
   * (Ollama in 1D): the catalogue the runtime reports, projected to a
   * controlled DTO with pre-computed inventory text. Runtime metadata, not a
   * metric. Read-only — no loading, generation, pull, or delete. */
  modelInventory(): Promise<ModelInventoryView>;
  /** The currently loaded (runtime-resident) models for the supported AI
   * runtime (Ollama in 1E): the read-only set the runtime reports, projected to
   * a controlled DTO with pre-computed loaded-model text. Distinct from the 1D
   * catalogue. Runtime metadata, not a metric. Read-only — no loading,
   * unloading, generation, pull, or delete. `expires_at` is carried but not
   * displayed. */
  loadedModels(): Promise<LoadedModelSetView>;
  /** One read-only LM Studio native-v1 snapshot. */
  lmStudioSnapshot?(): Promise<LmStudioSnapshotView>;
  /** One passive traditional single-model llama-server snapshot. GET-only;
   * never performs inference or provider/model management. */
  llamaCppSnapshot?(): Promise<LlamaCppSnapshotView>;
  /** Compose the resource-context explanation from the views the UI already
   * acquired (Milestone 1F): the available-memory side from `currentSnapshot`
   * (the single `/proc/meminfo` acquisition) and the loaded-model side from
   * `loadedModels` (the single `/api/ps` acquisition). PURE — performs no I/O of
   * its own; it only composes the controlled explanation so technical
   * interpretation stays server-side, not in Svelte. No aggregate, ratio, fit,
   * or health claim; no raw endpoint/host/digest. Distinct from the snapshot and
   * loaded-model surfaces, which it reads from but does not re-fetch. */
  composeResourceContext(
    snapshot: SnapshotView,
    loaded: LoadedModelSetView,
  ): Promise<ResourceContextView>;
  /** Pure deterministic diagnosis of one bounded bundle of views already
   * acquired by the frontend. The native command performs no I/O, inference,
   * provider action, or mutation. */
  diagnoseObservation(input: DiagnosticInput): Promise<DiagnosisResultView>;
  /** Run one observed inference against the supported AI runtime (Ollama in
   * 1G): a single non-streaming `POST /api/generate` carrying a fixed synthetic
   * diagnostic prompt to the explicitly selected model. Per-run authorization
   * is required — `execution_boundary_acknowledged === false` blocks the run
   * with no POST. The backend validates the model, checks the acknowledgement,
   * confirms the model is currently listed in the catalogue, enforces a
   * single-run guard, then issues at most one POST (no retry). The generated
   * response text is discarded; only the controlled DTO with Ollama-reported
   * counts/durations, an Engine-Room-calculated rate, and pre-computed
   * controlled text is returned. The execution boundary is always
   * `"undetermined"`; Engine Room never claims the model ran locally. NOT
   * called automatically (not on mount/refresh/selection) — only on an explicit
   * authorized user action. */
  runInferenceObservation(
    request: InferenceObservationRequest,
  ): Promise<InferenceObservationView>;
}

/** Which data source applies to the current environment. Pure and testable
 * without touching the real environment. */
export type DataSourceKind = "tauri" | "mock" | "none";

/** Decide the data source kind from explicit boolean inputs. `tauri` wins: in
 * the native webview the IPC path is authoritative even during dev. */
export function pickDataSourceKind(
  isTauriEnv: boolean,
  dev: boolean,
): DataSourceKind {
  if (isTauriEnv) return "tauri";
  if (dev) return "mock";
  return "none";
}

/** Construct the data source for an explicit environment. Pure with respect to
 * the booleans it is given, so the full switch — including the throw — is
 * testable without touching the real environment. */
export async function createDataSourceFor(
  isTauriEnv: boolean,
  dev: boolean,
): Promise<DataSource> {
  switch (pickDataSourceKind(isTauriEnv, dev)) {
    case "tauri": {
      const { TauriDataSource } = await import("./tauri");
      return new TauriDataSource();
    }
    case "mock": {
      const { MockDataSource } = await import("./mock");
      return new MockDataSource();
    }
    case "none":
      throw new Error(
        "No data source available: not running in Tauri and not in development mode.",
      );
  }
}

/** Construct the data source for the current environment.
 *
 * - Tauri native webview: `TauriDataSource` over narrow registered commands.
 * - Browser development: `MockDataSource` with clearly artificial fixtures
 *   (dynamically imported so fixtures are tree-shaken from production).
 * - Production browser with no Tauri: throws — there is no data source.
 */
export async function createDataSource(): Promise<DataSource> {
  return createDataSourceFor(isTauri(), import.meta.env.DEV);
}
