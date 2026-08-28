// Browser-development mock data source. Returns clearly artificial fixtures
// that mirror the Stage 8 view DTO shapes. It does not import Tauri `invoke`,
// does not contact Ollama or any runtime, and does not read real system
// telemetry or infer localhost services — the fixture data is static.

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
  ResourceContextView,
  SnapshotView,
} from "../types";
import {
  fixtureInferenceObservation,
  fixtureLoadedModels,
  fixtureModelInventory,
  fixtureReportPreview,
  fixtureResourceContext,
  fixtureRuntimeStatus,
  fixtureSnapshot,
} from "../fixtures/states";

export class MockDataSource implements DataSource {
  readonly isMock = true;

  async currentSnapshot() {
    return fixtureSnapshot();
  }

  async machineContext(): Promise<MachineContextView> {
    return {
      state: "available",
      native_cpu_architecture: "arm64",
      interpretation:
        "The artificial browser fixture reports the native CPU architecture as arm64.",
      why_it_matters:
        "This artificial value demonstrates compatibility-context presentation.",
      qualification:
        "Artificial architecture does not establish model compatibility, acceleration, performance, or compute placement.",
    };
  }

  async reportPreview() {
    return fixtureReportPreview();
  }

  async runtimeStatus() {
    // Static fixture — never contacts Ollama or any real runtime.
    return fixtureRuntimeStatus();
  }

  async modelInventory() {
    // Static fixture — never contacts Ollama or any real runtime.
    return fixtureModelInventory();
  }

  async loadedModels() {
    // Static fixture — never contacts Ollama or any real runtime.
    return fixtureLoadedModels();
  }

  async lmStudioSnapshot(): Promise<LmStudioSnapshotView> {
    return {
      provider: "lm_studio",
      state: "available",
      models: [
        {
          model_id: "artificial/leaf-model",
          display_name: "Artificial Leaf Model",
          model_type: "llm",
          inference_eligible: true,
          publisher: "Artificial Publisher",
          architecture: "artificial",
          quantization: "Q4",
          bits_per_weight: 4,
          size_bytes: 1_000_000_000,
          params_string: "1B",
          max_context_length: 4096,
          format: "gguf",
          loaded_instances: [
            { instance_id: "artificial-instance", context_length: 2048 },
          ],
        },
        {
          model_id: "artificial/embedding",
          display_name: "Artificial Embedding",
          model_type: "embedding",
          inference_eligible: false,
          publisher: null,
          architecture: null,
          quantization: null,
          bits_per_weight: null,
          size_bytes: null,
          params_string: null,
          max_context_length: null,
          format: null,
          loaded_instances: [],
        },
      ],
      skipped: 0,
      state_label: "Available",
      interpretation:
        "Artificial LM Studio native REST v1 fixture is available.",
      why_it_matters:
        "This browser data is artificial and makes no provider request.",
    };
  }

  async llamaCppSnapshot(): Promise<LlamaCppSnapshotView> {
    return {
      provider: "llama_cpp",
      state: "ready",
      served_model: { model_id: "artificial-leaf-server" },
      api_scope: "same_machine_loopback",
      compute_location: "not_independently_verified",
      state_label: "Ready",
      interpretation:
        "Artificial llama.cpp traditional single-model fixture is ready.",
      why_it_matters:
        "This browser data is artificial and makes no provider request.",
    };
  }

  async composeResourceContext(
    _snapshot: SnapshotView,
    _loaded: LoadedModelSetView,
  ): Promise<ResourceContextView> {
    // Static fixture — never contacts Ollama or any real runtime, and never
    // invokes Tauri. The passed inputs are themselves fixtures in the browser
    // path, so they are ignored; the variant-1 fixture mirrors the backend's
    // controlled text verbatim.
    return fixtureResourceContext();
  }

  async diagnoseObservation(
    input: DiagnosticInput,
  ): Promise<DiagnosisResultView> {
    // Deterministic artificial presentation only. This browser path never
    // invokes Tauri and never performs provider, OS, or network acquisition.
    const findings: DiagnosisResultView["findings"] = [];
    if (
      input.llama_cpp.kind === "observed" &&
      input.llama_cpp.state === "ready"
    ) {
      findings.push({
        code: "llama_cpp_passive_only_ready",
        source: "llama_cpp",
        observation:
          "Artificial llama.cpp fixture passively reports one served model.",
        meaning:
          "This artificial fixture demonstrates the passive-only diagnosis presentation.",
        safe_next_check:
          "Treat this browser state as artificial and use the native app for real observations.",
      });
    }
    if (input.supported_change_count > 0) {
      findings.push({
        code: "explicit_observation_changed",
        source: "observation",
        observation: `${input.supported_change_count} artificial supported factual change${input.supported_change_count === 1 ? "" : "s"} differed from the previous observation.`,
        meaning:
          "The listed artificial fixture facts differed; no cause is inferred.",
        safe_next_check: "Review the affected artificial fixture section.",
      });
    }
    return { state: "available", findings };
  }

  async runInferenceObservation(
    _request: InferenceObservationRequest,
  ): Promise<InferenceObservationView> {
    // Static fixture — NEVER contacts Ollama, never issues a generate POST, and
    // never invokes Tauri. The browser/preview path cannot run a real
    // observation; it returns a fixture result (with `execution_boundary:
    // "undetermined"` and artificial model name) so the UI can be exercised.
    // The request (including the authorization flag) is ignored on this path.
    return fixtureInferenceObservation();
  }
}
