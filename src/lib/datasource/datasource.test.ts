import { beforeEach, describe, expect, it, vi } from "vitest";

// Mock the Tauri core module so tests never reach a real IPC layer. isTauri and
// invoke are controlled per test.
vi.mock("@tauri-apps/api/core", () => ({
  isTauri: vi.fn(() => false),
  invoke: vi.fn(),
}));

// Import after the mock is registered.
import { isTauri, invoke } from "@tauri-apps/api/core";
import type {
  DiagnosticInput,
  InferenceObservationView,
  LoadedModelSetView,
  MetricAvailability,
  ModelInventoryView,
  Provenance,
  RuntimeStatusView,
  SourceAvailability,
} from "../types";
import { createDataSourceFor, pickDataSourceKind } from "./index";
import {
  fixtureInferenceObservation,
  fixtureInferenceObservations,
  fixtureLoadedModels,
  fixtureLoadedModelsSets,
  fixtureModelInventory,
  fixtureModelInventories,
  fixtureReportPreview,
  fixtureRuntimeStatus,
  fixtureRuntimeStatuses,
  fixtureSnapshot,
} from "../fixtures/states";

beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(false);
  vi.mocked(invoke).mockReset();
});

function diagnosticInput(): DiagnosticInput {
  return {
    available_memory: { kind: "value" },
    ollama_runtime: { kind: "observed", state: "ready", version: "1.0" },
    ollama_catalogue: {
      kind: "observed",
      state: "available",
      model_ids: ["artificial:one"],
    },
    ollama_loaded: {
      kind: "observed",
      state: "available",
      model_ids: ["artificial:one"],
    },
    lm_studio: { kind: "observed", state: "available", models: [] },
    llama_cpp: {
      kind: "observed",
      state: "ready",
      served_model_id: "artificial-served",
    },
    supported_change_count: 0,
  };
}

describe("pickDataSourceKind", () => {
  it("selects the IPC path in a Tauri environment", () => {
    expect(pickDataSourceKind(true, false)).toBe("tauri");
    // Tauri wins even in dev mode.
    expect(pickDataSourceKind(true, true)).toBe("tauri");
  });

  it("selects fixtures in ordinary browser dev mode", () => {
    expect(pickDataSourceKind(false, true)).toBe("mock");
  });

  it("has no data source in a production browser with no Tauri", () => {
    expect(pickDataSourceKind(false, false)).toBe("none");
  });
});

describe("createDataSourceFor — Tauri (IPC) path", () => {
  it("uses invoke with the narrow command names and returns their payloads", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "current_snapshot") {
        return Promise.resolve({ at: 7, entries: [] });
      }
      if (cmd === "report_preview") {
        return Promise.resolve({
          text: "preview-from-ipc",
          generation: "0000000000000001",
        });
      }
      if (cmd === "save_report") {
        return Promise.resolve("saved");
      }
      return Promise.reject(new Error(`unexpected command: ${cmd}`));
    });

    const ds = await createDataSourceFor(true, false);

    const snap = await ds.currentSnapshot();
    expect(snap.at).toBe(7);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("current_snapshot");

    const preview = await ds.reportPreview();
    expect(preview).toEqual({
      text: "preview-from-ipc",
      generation: "0000000000000001",
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("report_preview");

    await expect(ds.saveReport!("0000000000000001")).resolves.toBe("saved");
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("save_report", {
      generation: "0000000000000001",
    });
  });
});

describe("createDataSourceFor — browser dev (fixture) path", () => {
  it("uses an artificial passive llama.cpp fixture without native or provider access", async () => {
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    const ds = await createDataSourceFor(false, true);
    const llamaCpp = await ds.llamaCppSnapshot!();
    expect(llamaCpp).toMatchObject({
      provider: "llama_cpp",
      state: "ready",
      served_model: { model_id: "artificial-leaf-server" },
      api_scope: "same_machine_loopback",
      compute_location: "not_independently_verified",
    });
    expect(fetch).not.toHaveBeenCalled();
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });

  it("llama.cpp fixture state cannot expand or alter the report projection", async () => {
    const ds = await createDataSourceFor(false, true);
    const before = await ds.reportPreview();
    const llamaCpp = await ds.llamaCppSnapshot!();
    const after = await ds.reportPreview();
    expect(llamaCpp.served_model?.model_id).toBe("artificial-leaf-server");
    expect(after).toEqual(before);
    const text = typeof after === "string" ? after : after.text;
    expect(text).not.toMatch(
      /llama_cpp|artificial-leaf-server|127\.0\.0\.1:8080|same_machine_loopback|not_independently_verified/i,
    );
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("uses artificial LM Studio fixtures and makes no native/provider request", async () => {
    const ds = await createDataSourceFor(false, true);
    const lm = await ds.lmStudioSnapshot!();
    expect(lm.provider).toBe("lm_studio");
    expect(lm.models.some((model) => model.model_type === "embedding")).toBe(
      true,
    );
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("LM Studio fixture state cannot expand or alter the report projection", async () => {
    const ds = await createDataSourceFor(false, true);
    const before = await ds.reportPreview();
    const lm = await ds.lmStudioSnapshot!();
    expect(lm.models.length).toBeGreaterThan(0);
    const after = await ds.reportPreview();
    expect(after).toEqual(before);
    const text = typeof after === "string" ? after : after.text;
    expect(text).not.toMatch(/LM Studio|Artificial Leaf|loaded instance/i);
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });
  it("returns fixture data and never calls invoke", async () => {
    const ds = await createDataSourceFor(false, true);
    expect(ds.saveReport).toBeUndefined();

    const snap = await ds.currentSnapshot();
    expect(snap.entries.length).toBeGreaterThan(0);
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();

    const preview = await ds.reportPreview();
    const text = typeof preview === "string" ? preview : preview.text;
    expect(text).toContain("Memory in use");
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("browser fixture mode cannot fall through to Tauri invoke", async () => {
    const ds = await createDataSourceFor(false, true);
    await ds.currentSnapshot();
    await ds.reportPreview();
    // Even though a Tauri invoke mock exists, the fixture path must never use it.
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("report preview is report-safe: no paths, ids, or raw errors", async () => {
    const ds = await createDataSourceFor(false, true);
    const preview = await ds.reportPreview();
    const text = typeof preview === "string" ? preview : preview.text;
    expect(text).not.toContain("/home/");
    expect(text).not.toContain("@");
    expect(text).not.toContain("os.ram.used");
    expect(text).not.toContain("timed out");
    expect(text).toContain("available");
  });
});

describe("createDataSourceFor — LM Studio native command", () => {
  it("uses only the narrow snapshot command", async () => {
    vi.mocked(invoke).mockResolvedValue({
      provider: "lm_studio",
      state: "not_detected",
      models: [],
      skipped: 0,
      state_label: "Not detected",
      interpretation: "Controlled",
      why_it_matters: "Controlled",
      resource_interpretation: "Controlled",
      resource_qualification: "Controlled",
    });
    const ds = await createDataSourceFor(true, false);
    await ds.lmStudioSnapshot!();
    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "current_lm_studio_snapshot",
    );
  });
});

describe("createDataSourceFor — passive llama.cpp command", () => {
  it("uses only the narrow snapshot command", async () => {
    vi.mocked(invoke).mockResolvedValue({
      provider: "llama_cpp",
      state: "unreachable",
      served_model: null,
      api_scope: "same_machine_loopback",
      compute_location: "not_independently_verified",
      state_label: "Unreachable",
      interpretation: "Controlled",
      why_it_matters: "Controlled",
    });
    const ds = await createDataSourceFor(true, false);
    await ds.llamaCppSnapshot!();
    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "current_llama_cpp_snapshot",
    );
  });
});

describe("createDataSourceFor — pure deterministic diagnosis", () => {
  it("uses only the narrow diagnosis command with the bounded input", async () => {
    const input = diagnosticInput();
    const result = { state: "available" as const, findings: [] };
    vi.mocked(invoke).mockResolvedValue(result);
    const ds = await createDataSourceFor(true, false);
    await expect(ds.diagnoseObservation(input)).resolves.toEqual(result);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("diagnose_observation", {
      input,
    });
  });

  it("browser fixture diagnosis is artificial and never invokes Tauri or fetch", async () => {
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    const ds = await createDataSourceFor(false, true);
    const result = await ds.diagnoseObservation(diagnosticInput());
    expect(result.state).toBe("available");
    expect(result.findings[0]?.observation).toMatch(/Artificial/);
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
    expect(fetch).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });
});

describe("createDataSourceFor — production browser (no source)", () => {
  it("throws rather than guessing a data source", async () => {
    await expect(createDataSourceFor(false, false)).rejects.toThrow(
      /No data source/,
    );
  });
});

describe("fixture shapes", () => {
  it("covers all outcome kinds", async () => {
    const snap = fixtureSnapshot();
    const kinds = new Set(snap.entries.map((e) => e.outcome.kind));
    expect(kinds.has("ok")).toBe(true);
    expect(kinds.has("no_value")).toBe(true);
    expect(kinds.has("failed")).toBe(true);
  });

  it("covers all provenance categories on ok outcomes", () => {
    const snap = fixtureSnapshot();
    const provenance = new Set(
      snap.entries
        .filter((e) => e.outcome.kind === "ok")
        .map((e) => (e.outcome.kind === "ok" ? e.outcome.provenance : null)),
    );
    for (const p of [
      "operating_system_reported",
      "driver_reported",
      "runtime_reported",
      "application_measured",
      "calculated",
      "estimated",
    ] as Provenance[]) {
      expect(provenance.has(p)).toBe(true);
    }
  });

  it("covers all source availability states and keeps metric availability distinct", () => {
    const snap = fixtureSnapshot();
    const sources = new Set(snap.entries.map((e) => e.source_availability));
    for (const s of [
      "ready",
      "not_detected",
      "not_running",
      "unreachable",
      null,
    ] as SourceAvailability[]) {
      expect(sources.has(s)).toBe(true);
    }
    const metrics = new Set(snap.entries.map((e) => e.metric_availability));
    for (const m of [
      "available",
      "not_exposed",
      "transiently_unavailable",
      "not_applicable",
      null,
    ] as MetricAvailability[]) {
      expect(metrics.has(m)).toBe(true);
    }
  });

  it("carries controlled explanation text from the data source, not invented text", () => {
    const snap = fixtureSnapshot();
    for (const e of snap.entries) {
      expect(e.interpretation.length).toBeGreaterThan(0);
      expect(e.why_it_matters.length).toBeGreaterThan(0);
    }
    // A representative failed entry's explanation is a controlled message.
    const failed = snap.entries.find((e) => e.outcome.kind === "failed")!;
    expect(failed.why_it_matters).toBe("Acquiring this metric timed out.");
  });

  it("report preview matches the fixture function output", () => {
    const preview = fixtureReportPreview();
    expect(preview).toContain("AI Engine Room — Observation Report");
    expect(preview).toContain(
      "Memory in use\nStatus: Available\nValue: 8.59 GB",
    );
    expect(preview).toMatch(/\n$/);
  });
});

describe("createDataSourceFor — runtime status", () => {
  it("Tauri path uses invoke with the runtime status command name", async () => {
    const status: RuntimeStatusView = {
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
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "current_runtime_status") return Promise.resolve(status);
      return Promise.reject(new Error(`unexpected command: ${cmd}`));
    });

    const ds = await createDataSourceFor(true, false);
    const result = await ds.runtimeStatus();
    expect(result).toEqual(status);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("current_runtime_status");
  });

  it("browser fixture path returns a runtime status and never calls invoke", async () => {
    const ds = await createDataSourceFor(false, true);
    const status = await ds.runtimeStatus();
    expect(status.runtime_type).toBe("ollama");
    expect(status.source_availability).toBe("ready");
    expect(typeof status.status_label).toBe("string");
    expect(status.status_label.length).toBeGreaterThan(0);
    // The fixture path must never reach Tauri invoke.
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("runtime status fixture exposes all four user-facing states", () => {
    const statuses = fixtureRuntimeStatuses();
    expect(statuses.length).toBe(4);
    const labels = statuses.map((s) => s.status_label);
    expect(labels).toContain("Ollama available");
    expect(labels).toContain("Ollama responded unexpectedly");
    expect(labels).toContain("Ollama was not detected at the local endpoint");
    expect(labels).toContain("Ollama could not be reached");
  });

  it("runtime status fixtures never leak raw transport details or endpoint URLs", () => {
    for (const s of fixtureRuntimeStatuses()) {
      for (const field of [
        s.status_label,
        s.interpretation,
        s.why_it_matters,
      ]) {
        const lower = field.toLowerCase();
        expect(lower).not.toContain("127.0.0.1");
        expect(lower).not.toContain("11434");
        expect(lower).not.toContain("http");
        expect(lower).not.toContain("refus");
        expect(lower).not.toContain("timeout");
      }
    }
  });

  it("runtime status fixtures never claim install state or health/compat/security", () => {
    for (const s of fixtureRuntimeStatuses()) {
      for (const field of [
        s.status_label,
        s.interpretation,
        s.why_it_matters,
      ]) {
        const lower = field.toLowerCase();
        expect(lower).not.toContain("install");
        expect(lower).not.toContain("health");
        expect(lower).not.toContain("compat");
        expect(lower).not.toContain("secure");
        expect(lower).not.toContain("fast");
      }
    }
  });

  it("available fixture carries a runtime-reported version; absent states do not", () => {
    const available = fixtureRuntimeStatuses().find(
      (s) => s.status_label === "Ollama available",
    )!;
    expect(available.version).not.toBeNull();
    expect(available.version_reported_by).toBe("runtime_reported");
    for (const other of fixtureRuntimeStatuses().filter(
      (s) => s.status_label !== "Ollama available",
    )) {
      expect(other.version).toBeNull();
      expect(other.version_reported_by).toBeNull();
    }
  });

  it("default fixture runtime status is the available state", () => {
    const status = fixtureRuntimeStatus();
    expect(status.status_label).toBe("Ollama available");
    expect(status.source_availability).toBe("ready");
  });
});

describe("createDataSourceFor — model inventory", () => {
  it("Tauri path uses invoke with the model inventory command name", async () => {
    const inventory: ModelInventoryView = {
      runtime_type: "ollama",
      state: "available",
      source_availability: "ready",
      models: [
        {
          display_name: "example-chat:8b",
          locality: "undetermined",
          size_bytes: 4_000_000_000,
          parameter_size: "8.0B",
          quantization: "Q4_0",
          families: ["llama"],
        },
      ],
      skipped: 0,
      state_label: "Ollama model catalogue",
      interpretation: "These models are listed by the Ollama runtime.",
      why_it_matters:
        "Each entry shows the model name and the size Ollama reports for it.",
    };
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "current_model_inventory") return Promise.resolve(inventory);
      return Promise.reject(new Error(`unexpected command: ${cmd}`));
    });

    const ds = await createDataSourceFor(true, false);
    const result = await ds.modelInventory();
    expect(result).toEqual(inventory);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("current_model_inventory");
  });

  it("browser fixture path returns a model inventory and never calls invoke", async () => {
    const ds = await createDataSourceFor(false, true);
    const inventory = await ds.modelInventory();
    expect(inventory.runtime_type).toBe("ollama");
    expect(inventory.state).toBe("available");
    expect(inventory.models.length).toBeGreaterThan(0);
    expect(typeof inventory.state_label).toBe("string");
    expect(inventory.state_label.length).toBeGreaterThan(0);
    // The fixture path must never reach Tauri invoke.
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("model inventory fixture exposes all five representative states", () => {
    const inventories = fixtureModelInventories();
    expect(inventories.length).toBe(5);
    const labels = inventories.map((i) => i.state_label);
    expect(labels).toContain("Ollama model catalogue");
    expect(labels).toContain("No models in the catalogue");
    expect(labels).toContain("Ollama responded unexpectedly");
    expect(labels).toContain("Ollama was not detected at the local endpoint");
    expect(labels).toContain("Ollama could not be reached");
  });

  it("model inventory fixtures never leak raw transport details or endpoint URLs", () => {
    for (const inv of fixtureModelInventories()) {
      for (const field of [
        inv.state_label,
        inv.interpretation,
        inv.why_it_matters,
      ]) {
        const lower = field.toLowerCase();
        expect(lower).not.toContain("127.0.0.1");
        expect(lower).not.toContain("11434");
        expect(lower).not.toContain("http");
        expect(lower).not.toContain("refus");
        expect(lower).not.toContain("timeout");
      }
      for (const m of inv.models) {
        // No private/transport detail leaks into a model entry.
        expect(m.display_name.toLowerCase()).not.toContain("127.0.0.1");
        expect(m.display_name.toLowerCase()).not.toContain("http");
      }
    }
  });

  it("model inventory fixtures never claim the runtime is not installed", () => {
    for (const inv of fixtureModelInventories()) {
      for (const field of [
        inv.state_label,
        inv.interpretation,
        inv.why_it_matters,
      ]) {
        expect(field.toLowerCase()).not.toContain("not installed");
      }
    }
  });

  it("every model inventory fixture entry is locality undetermined", () => {
    for (const inv of fixtureModelInventories()) {
      for (const m of inv.models) {
        expect(m.locality).toBe("undetermined");
      }
    }
  });

  it("fixture model names are clearly artificial and sizes are round numbers", () => {
    const inv = fixtureModelInventory();
    for (const m of inv.models) {
      // Artificial fixture names use the `example-*` prefix; no real catalogue.
      expect(m.display_name).toMatch(/^example-/);
    }
  });

  it("default fixture model inventory is the populated available state", () => {
    const inv = fixtureModelInventory();
    expect(inv.state).toBe("available");
    expect(inv.source_availability).toBe("ready");
    expect(inv.models.length).toBeGreaterThan(0);
  });
});

describe("createDataSourceFor — loaded models", () => {
  it("Tauri path uses invoke with the loaded models command name", async () => {
    const loaded: LoadedModelSetView = {
      runtime_type: "ollama",
      state: "available",
      source_availability: "ready",
      models: [
        {
          display_name: "example-runner:8b",
          reported_size_bytes: 5_000_000_000,
          reported_vram_bytes: 4_000_000_000,
          reported_context_length: 8192,
          expires_at: "2024-01-01T00:00:00.000Z",
        },
      ],
      skipped: 0,
      state_label: "Currently loaded in Ollama",
      interpretation: "Ollama reports these models as currently loaded.",
      why_it_matters:
        "Each entry shows the model name and the loaded size Ollama reports for it.",
      resource_interpretation: "Controlled provider resource interpretation.",
      resource_qualification: "Controlled provider resource qualification.",
    };
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "current_loaded_models") return Promise.resolve(loaded);
      return Promise.reject(new Error(`unexpected command: ${cmd}`));
    });

    const ds = await createDataSourceFor(true, false);
    const result = await ds.loadedModels();
    expect(result).toEqual(loaded);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("current_loaded_models");
  });

  it("browser fixture path returns a loaded-model set and never calls invoke", async () => {
    const ds = await createDataSourceFor(false, true);
    const loaded = await ds.loadedModels();
    expect(loaded.runtime_type).toBe("ollama");
    expect(loaded.state).toBe("available");
    expect(loaded.models.length).toBeGreaterThan(0);
    expect(typeof loaded.state_label).toBe("string");
    expect(loaded.state_label.length).toBeGreaterThan(0);
    // The fixture path must never reach Tauri invoke.
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("loaded-model fixture exposes all five representative states", () => {
    const sets = fixtureLoadedModelsSets();
    expect(sets.length).toBe(5);
    const labels = sets.map((s) => s.state_label);
    expect(labels).toContain("Currently loaded in Ollama");
    expect(labels).toContain("No models currently running in Ollama");
    expect(labels).toContain("Ollama responded unexpectedly");
    expect(labels).toContain("Ollama was not detected at the local endpoint");
    expect(labels).toContain("Ollama could not be reached");
  });

  it("loaded-model fixtures never leak raw transport details or endpoint URLs", () => {
    for (const set of fixtureLoadedModelsSets()) {
      for (const field of [
        set.state_label,
        set.interpretation,
        set.why_it_matters,
      ]) {
        const lower = field.toLowerCase();
        expect(lower).not.toContain("127.0.0.1");
        expect(lower).not.toContain("11434");
        expect(lower).not.toContain("http");
        expect(lower).not.toContain("refus");
        expect(lower).not.toContain("timeout");
      }
      for (const m of set.models) {
        // No private/transport detail leaks into a loaded entry.
        expect(m.display_name.toLowerCase()).not.toContain("127.0.0.1");
        expect(m.display_name.toLowerCase()).not.toContain("http");
      }
    }
  });

  it("loaded-model fixtures never claim the runtime is not installed", () => {
    for (const set of fixtureLoadedModelsSets()) {
      for (const field of [
        set.state_label,
        set.interpretation,
        set.why_it_matters,
      ]) {
        expect(field.toLowerCase()).not.toContain("not installed");
      }
    }
  });

  it("fixture loaded-model names are clearly artificial", () => {
    const loaded = fixtureLoadedModels();
    for (const m of loaded.models) {
      // Artificial fixture names use the `example-*` prefix; no real state.
      expect(m.display_name).toMatch(/^example-/);
    }
  });

  it("default fixture loaded-model set is the populated available state", () => {
    const loaded = fixtureLoadedModels();
    expect(loaded.state).toBe("available");
    expect(loaded.source_availability).toBe("ready");
    expect(loaded.models.length).toBeGreaterThan(0);
  });

  it("loaded-model fixtures carry expires_at but the field is not a transport detail", () => {
    // expires_at is a documented, carried string (not a digest/host/endpoint).
    // The fixtures may carry it; this only asserts it never contains private
    // transport detail when present.
    for (const set of fixtureLoadedModelsSets()) {
      for (const m of set.models) {
        if (m.expires_at !== null) {
          const lower = m.expires_at.toLowerCase();
          expect(lower).not.toContain("127.0.0.1");
          expect(lower).not.toContain("11434");
          expect(lower).not.toContain("http");
        }
      }
    }
  });
});

describe("createDataSourceFor — observed inference", () => {
  it("Tauri path uses invoke with the observed-inference command name and request", async () => {
    const observation: InferenceObservationView = fixtureInferenceObservation();
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "run_inference_observation") {
        return Promise.resolve(observation);
      }
      return Promise.reject(new Error(`unexpected command: ${cmd}`));
    });

    const ds = await createDataSourceFor(true, false);
    const request = {
      model: "example-runner:8b",
      execution_boundary_acknowledged: true,
    };
    const result = await ds.runInferenceObservation(request);
    expect(result).toEqual(observation);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "run_inference_observation",
      { request },
    );
  });

  it("browser fixture path returns an observation and never calls invoke", async () => {
    const ds = await createDataSourceFor(false, true);
    const observation = await ds.runInferenceObservation({
      model: "example-runner:8b",
      execution_boundary_acknowledged: true,
    });
    expect(observation.state).toBe("completed");
    expect(observation.execution_boundary).toBe("undetermined");
    // The fixture path must never reach Tauri invoke (no real POST).
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("browser fixture path never issues a generate POST regardless of authorization", async () => {
    const ds = await createDataSourceFor(false, true);
    // Even an acknowledged request must not escape to invoke on the fixture path.
    await ds.runInferenceObservation({
      model: "example-runner:8b",
      execution_boundary_acknowledged: true,
    });
    await ds.runInferenceObservation({
      model: "example-runner:8b",
      execution_boundary_acknowledged: false,
    });
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });
});

describe("observed-inference fixture shapes", () => {
  it("exposes all nine result states", () => {
    const observations = fixtureInferenceObservations();
    expect(observations.length).toBe(9);
    const states = observations.map((o) => o.state);
    expect(states).toContain("no_model_selected");
    expect(states).toContain("blocked_by_execution_boundary");
    expect(states).toContain("running");
    expect(states).toContain("completed");
    expect(states).toContain("not_detected");
    expect(states).toContain("unreachable");
    expect(states).toContain("timed_out");
    expect(states).toContain("responded_unexpectedly");
    expect(states).toContain("model_unavailable");
  });

  it("every fixture carries execution_boundary undetermined", () => {
    for (const o of fixtureInferenceObservations()) {
      expect(o.execution_boundary).toBe("undetermined");
    }
  });

  it("no fixture carries a generated-response field or generated text", () => {
    for (const o of fixtureInferenceObservations()) {
      // The view DTO has no `response` field; assert it is absent.
      expect("response" in o).toBe(false);
      expect("generated" in o).toBe(false);
      // No fixture carries the fixed diagnostic prompt or generated output.
      const json = JSON.stringify(o).toLowerCase();
      expect(json).not.toMatch(/leaves appear green/);
      expect(json).not.toContain("secret");
    }
  });

  it("no fixture leaks raw transport detail, endpoints, or status codes", () => {
    for (const o of fixtureInferenceObservations()) {
      const json = JSON.stringify(o).toLowerCase();
      expect(json).not.toContain("127.0.0.1");
      expect(json).not.toContain("11434");
      expect(json).not.toContain("http");
      expect(json).not.toContain("/api/");
      expect(json).not.toContain("refus");
    }
  });

  it("no fixture claims local, verified, install, or health", () => {
    for (const o of fixtureInferenceObservations()) {
      const json = JSON.stringify(o).toLowerCase();
      expect(json).not.toMatch(/\blocal\b/);
      expect(json).not.toMatch(/verified/);
      expect(json).not.toMatch(/install/);
      expect(json).not.toMatch(/health/);
      expect(json).not.toMatch(/secure/);
    }
  });

  it("fixture model names are clearly artificial", () => {
    for (const o of fixtureInferenceObservations()) {
      if (o.model !== null) {
        expect(o.model).toMatch(/^example-/);
      }
    }
  });

  it("the completed fixture carries counts, nanosecond durations, and a calculated rate", () => {
    const completed = fixtureInferenceObservations().find(
      (o) => o.state === "completed",
    )!;
    expect(completed.prompt_eval_count).toBe(12);
    expect(completed.eval_count).toBe(8);
    expect(completed.eval_duration_ns).toBe(1_000_000_000);
    // The rate is the Engine-Room-calculated eval_count * 1e9 / eval_duration.
    expect(completed.generation_tokens_per_second).toBe(8.0);
  });

  it("non-completed fixtures carry no metric values", () => {
    for (const o of fixtureInferenceObservations().filter(
      (obs) => obs.state !== "completed",
    )) {
      expect(o.prompt_eval_count).toBeNull();
      expect(o.eval_count).toBeNull();
      expect(o.load_duration_ns).toBeNull();
      expect(o.prompt_eval_duration_ns).toBeNull();
      expect(o.eval_duration_ns).toBeNull();
      expect(o.total_duration_ns).toBeNull();
      expect(o.generation_tokens_per_second).toBeNull();
    }
  });

  it("the no-model fixture carries no model and no diagnostic profile", () => {
    const noModel = fixtureInferenceObservations().find(
      (o) => o.state === "no_model_selected",
    )!;
    expect(noModel.model).toBeNull();
    expect(noModel.diagnostic_profile).toBeNull();
  });

  it("every fixture carries controlled interpretation/why_it_matters/limitations text", () => {
    for (const o of fixtureInferenceObservations()) {
      expect(o.interpretation.length).toBeGreaterThan(0);
      expect(o.why_it_matters.length).toBeGreaterThan(0);
      expect(o.limitations.length).toBeGreaterThan(0);
    }
  });

  it("the default fixture is the completed state", () => {
    expect(fixtureInferenceObservation().state).toBe("completed");
  });
});
