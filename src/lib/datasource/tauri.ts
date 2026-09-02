// Native Tauri data source. Consumes only the registered narrow application
// commands over the supported Tauri `invoke` API, returning controlled view
// DTOs. It does not access operating-system or runtime facilities directly.

import { invoke } from "@tauri-apps/api/core";
import type { DataSource } from "./index";
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
  ReportPreviewView,
  ReportSaveResult,
  RuntimeStatusView,
  SnapshotView,
} from "../types";

export class TauriDataSource implements DataSource {
  readonly isMock = false;

  async currentSnapshot(): Promise<SnapshotView> {
    return invoke<SnapshotView>("current_snapshot");
  }

  async machineContext(): Promise<MachineContextView> {
    return invoke<MachineContextView>("current_machine_context");
  }

  async reportPreview(): Promise<ReportPreviewView> {
    return invoke<ReportPreviewView>("report_preview");
  }

  async saveReport(generation: string): Promise<ReportSaveResult> {
    return invoke<ReportSaveResult>("save_report", { generation });
  }

  async runtimeStatus(): Promise<RuntimeStatusView> {
    return invoke<RuntimeStatusView>("current_runtime_status");
  }

  async modelInventory(): Promise<ModelInventoryView> {
    return invoke<ModelInventoryView>("current_model_inventory");
  }

  async loadedModels(): Promise<LoadedModelSetView> {
    return invoke<LoadedModelSetView>("current_loaded_models");
  }

  async lmStudioSnapshot(): Promise<LmStudioSnapshotView> {
    return invoke<LmStudioSnapshotView>("current_lm_studio_snapshot");
  }

  async llamaCppSnapshot(): Promise<LlamaCppSnapshotView> {
    return invoke<LlamaCppSnapshotView>("current_llama_cpp_snapshot");
  }

  async composeResourceContext(
    snapshot: SnapshotView,
    loaded: LoadedModelSetView,
  ): Promise<ResourceContextView> {
    // PURE command: passes the already-acquired views; the command performs no
    // I/O (no /proc/meminfo, no /api/ps). Only the controlled explanation is
    // returned.
    return invoke<ResourceContextView>("compose_resource_context", {
      snapshot,
      loaded,
    });
  }

  async diagnoseObservation(
    input: DiagnosticInput,
  ): Promise<DiagnosisResultView> {
    // PURE command over views already acquired by the caller. It performs no
    // provider/OS call, inference, filesystem access, clipboard action, or
    // mutation.
    return invoke<DiagnosisResultView>("diagnose_observation", { input });
  }

  async runInferenceObservation(
    request: InferenceObservationRequest,
  ): Promise<InferenceObservationView> {
    // One authorized observed-inference run. The backend enforces per-run
    // authorization, catalogue identity, and the single-run guard before
    // issuing at most one POST. The generated response is discarded; only the
    // controlled DTO returns.
    return invoke<InferenceObservationView>("run_inference_observation", {
      request,
    });
  }
}
