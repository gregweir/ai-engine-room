import { describe, expect, it } from "vitest";
import {
  appendDiagnosticObservation,
  createDiagnosticObservationDraft,
  diagnosticDeltas,
  diagnosisInput,
  DIAGNOSTIC_OBSERVATION_LIMIT,
  sourceCoverage,
  type DiagnosticObservationBundle,
} from "./diagnosticObservation";
import {
  fixtureLoadedModels,
  fixtureModelInventory,
  fixtureRuntimeStatus,
  fixtureSnapshot,
} from "./fixtures/states";
import type {
  LlamaCppSnapshotView,
  LmStudioSnapshotView,
  SnapshotView,
} from "./types";

function lmStudioModels(
  definitions: { modelId: string; instances: string[] }[],
): LmStudioSnapshotView {
  return {
    provider: "lm_studio",
    state: "available",
    models: definitions.map(({ modelId, instances }) => ({
      model_id: modelId,
      display_name: "Artificial LM model",
      model_type: "llm",
      inference_eligible: true,
      publisher: null,
      architecture: null,
      quantization: null,
      bits_per_weight: null,
      size_bytes: null,
      params_string: null,
      max_context_length: null,
      format: null,
      loaded_instances: instances.map((instance_id) => ({
        instance_id,
        context_length: null,
      })),
    })),
    skipped: 0,
    state_label: "Available",
    interpretation: "Controlled artificial state.",
    why_it_matters: "Controlled artificial context.",
    resource_interpretation:
      "Controlled provider resource interpretation.",
    resource_qualification:
      "Controlled provider resource qualification.",
  };
}

function lmStudio(
  modelId = "artificial/lm-model",
  instances: string[] = ["instance-a"],
): LmStudioSnapshotView {
  return lmStudioModels([{ modelId, instances }]);
}

function llamaCpp(modelId = "artificial-served"): LlamaCppSnapshotView {
  return {
    provider: "llama_cpp",
    state: "ready",
    served_model: { model_id: modelId },
    api_scope: "same_machine_loopback",
    compute_location: "not_independently_verified",
    state_label: "Ready",
    interpretation: "Controlled artificial state.",
    why_it_matters: "Controlled artificial context.",
  };
}

function memorySnapshot(value: number): SnapshotView {
  const snapshot = fixtureSnapshot();
  const entry = snapshot.entries.find(
    (candidate) => candidate.display_name === "Available memory",
  )!;
  entry.outcome = {
    kind: "ok",
    value,
    unit: "bytes",
    provenance: "operating_system_reported",
  };
  return snapshot;
}

function bundle(
  id = 1,
  options: {
    memory?: number | null;
    snapshotRejected?: boolean;
    catalogue?: string[] | null;
    loaded?: string[] | null;
    lm?: LmStudioSnapshotView | null;
    llama?: LlamaCppSnapshotView | null;
  } = {},
): DiagnosticObservationBundle {
  const inventory = fixtureModelInventory();
  const loaded = fixtureLoadedModels();
  if (options.catalogue !== undefined && options.catalogue !== null) {
    inventory.models = options.catalogue.map((display_name) => ({
      ...inventory.models[0]!,
      display_name,
    }));
  }
  if (options.loaded !== undefined && options.loaded !== null) {
    loaded.models = options.loaded.map((display_name) => ({
      ...loaded.models[0]!,
      display_name,
    }));
  }
  return {
    observation_id: id,
    ...createDiagnosticObservationDraft(
      options.memory === null ? null : memorySnapshot(options.memory ?? 4096),
      options.snapshotRejected ?? false,
      fixtureRuntimeStatus(),
      options.catalogue === null ? null : inventory,
      options.loaded === null ? null : loaded,
      options.lm === undefined ? lmStudio() : options.lm,
      options.llama === undefined ? llamaCpp() : options.llama,
    ),
  };
}

describe("Diagnostic observation history", () => {
  it("retains startup/Refresh drafts oldest-to-newest with monotonic IDs and duplicates", () => {
    let history: DiagnosticObservationBundle[] = [];
    let nextObservationId = 1;
    const draft = createDiagnosticObservationDraft(
      memorySnapshot(0),
      false,
      fixtureRuntimeStatus(),
      fixtureModelInventory(),
      fixtureLoadedModels(),
      lmStudio(),
      llamaCpp(),
    );
    ({ history, nextObservationId } = appendDiagnosticObservation(
      history,
      draft,
      nextObservationId,
    ));
    ({ history, nextObservationId } = appendDiagnosticObservation(
      history,
      draft,
      nextObservationId,
    ));
    expect(history.map((item) => item.observation_id)).toEqual([1, 2]);
    expect(history.map((item) => item.available_memory)).toEqual([
      { kind: "value", bytes: 0, provenance: "operating_system_reported" },
      { kind: "value", bytes: 0, provenance: "operating_system_reported" },
    ]);
    expect(nextObservationId).toBe(3);
  });

  it("retains only the newest 12 without reusing IDs", () => {
    let history: DiagnosticObservationBundle[] = [];
    let nextObservationId = 1;
    for (let index = 0; index < DIAGNOSTIC_OBSERVATION_LIMIT + 3; index++) {
      const result = appendDiagnosticObservation(
        history,
        bundle(0, { memory: index }),
        nextObservationId,
      );
      history = result.history;
      nextObservationId = result.nextObservationId;
    }
    expect(history).toHaveLength(12);
    expect(history[0]!.observation_id).toBe(4);
    expect(history.at(-1)!.observation_id).toBe(15);
    expect(nextObservationId).toBe(16);
  });

  it("fails closed for missing, rejected, unbounded, or path-like source data", () => {
    const rejected = bundle(1, { memory: null, snapshotRejected: true });
    expect(rejected.available_memory).toEqual({
      kind: "unavailable",
      reason: "snapshot_rejected",
    });
    expect(bundle(1, { catalogue: null }).ollama_catalogue.kind).toBe("gap");

    const inventory = fixtureModelInventory();
    inventory.models[0]!.display_name = "/home/owner/private-model";
    const draft = createDiagnosticObservationDraft(
      memorySnapshot(1),
      false,
      fixtureRuntimeStatus(),
      inventory,
      fixtureLoadedModels(),
      lmStudio(),
      llamaCpp(),
    );
    expect(draft.ollama_catalogue).toEqual({ kind: "gap" });
    expect(JSON.stringify(draft)).not.toContain("/home/owner");

    const privateVersion = fixtureRuntimeStatus();
    privateVersion.version = "/home/owner/private-version";
    const versionDraft = createDiagnosticObservationDraft(
      memorySnapshot(1),
      false,
      privateVersion,
      fixtureModelInventory(),
      fixtureLoadedModels(),
      lmStudio(),
      llamaCpp(),
    );
    expect(versionDraft.ollama_runtime).toEqual({
      kind: "observed",
      value: { state: "ready", version: null },
    });
    expect(JSON.stringify(versionDraft)).not.toContain("/home/owner");
  });

  it("retains no timestamps, persistence keys, errors, responses, or endpoints", () => {
    const observation = bundle();
    const text = JSON.stringify(observation);
    expect(text).not.toMatch(
      /snapshot_at|timestamp|raw_error|response|endpoint/,
    );
    expect(text).not.toMatch(/localStorage|sessionStorage|indexedDB/);
  });
});

describe("Factual diagnostic deltas", () => {
  it("reports numeric memory change but not equality, including successful zero", () => {
    const changed = diagnosticDeltas(bundle(1, { memory: 0 }), bundle(2));
    expect(changed.state).toBe("changes");
    expect(changed.changes.map((change) => change.code)).toContain(
      "available_memory_value_changed",
    );
    expect(JSON.stringify(changed)).toContain("0 B (0 bytes)");
    expect(diagnosticDeltas(bundle(1), bundle(2))).toEqual({
      state: "no_supported_change",
      changes: [],
    });
  });

  it("reports value/gap transitions without converting a gap to zero", () => {
    const toGap = diagnosticDeltas(bundle(1), bundle(2, { memory: null }));
    expect(toGap.changes[0]!.code).toBe("available_memory_value_gap_changed");
    expect(JSON.stringify(toGap)).not.toContain("0 B");
    const toValue = diagnosticDeltas(bundle(1, { memory: null }), bundle(2));
    expect(toValue.changes[0]!.code).toBe("available_memory_value_gap_changed");
  });

  it("keeps unsafe numeric presentation as a nonnumeric controlled gap", () => {
    const unsafe = bundle(2, { memory: Number.MAX_SAFE_INTEGER + 1 });
    expect(unsafe.available_memory.kind).toBe("unavailable");
    expect(diagnosticDeltas(bundle(1), unsafe).changes[0]!.code).toBe(
      "available_memory_value_gap_changed",
    );
  });

  it("reports Ollama state/version and exact comparable set changes", () => {
    const prior = bundle(1, { catalogue: ["same:1b"], loaded: ["same:1b"] });
    const current = bundle(2, {
      catalogue: ["same:1b", "added:2b"],
      loaded: [],
    });
    if (current.ollama_runtime.kind === "observed") {
      current.ollama_runtime.value = {
        state: "unreachable",
        version: "2.0.0",
      };
    }
    const changes = diagnosticDeltas(prior, current).changes;
    expect(changes.map((change) => change.code)).toEqual(
      expect.arrayContaining([
        "ollama_runtime_state_changed",
        "ollama_reported_version_changed",
        "ollama_catalogue_set_changed",
        "ollama_loaded_set_changed",
      ]),
    );
    expect(JSON.stringify(changes)).toContain("Added: added:2b");
    expect(JSON.stringify(changes)).toContain("Removed: same:1b");
  });

  it("blocks false catalogue/loaded additions or removals across source gaps", () => {
    const changes = diagnosticDeltas(
      bundle(1, { catalogue: ["one"], loaded: ["one"] }),
      bundle(2, { catalogue: null, loaded: null }),
    ).changes;
    expect(changes.map((change) => change.code)).toEqual([
      "ollama_catalogue_source_availability_changed",
      "ollama_loaded_source_availability_changed",
    ]);
    expect(JSON.stringify(changes)).not.toMatch(/Added:|Removed:/);

    const restored = diagnosticDeltas(
      bundle(1, { catalogue: null, loaded: null }),
      bundle(2, { catalogue: ["one"], loaded: ["one"] }),
    ).changes;
    expect(restored.map((change) => change.code)).toEqual([
      "ollama_catalogue_source_availability_changed",
      "ollama_loaded_source_availability_changed",
    ]);
    expect(JSON.stringify(restored)).not.toMatch(/Added:|Removed:/);
  });

  it("reports LM Studio catalogue and exact loaded-instance membership changes", () => {
    const changes = diagnosticDeltas(
      bundle(1, { lm: lmStudio("same", ["instance-a"]) }),
      bundle(2, { lm: lmStudio("same", ["instance-b"]) }),
    ).changes;
    expect(changes.map((change) => change.code)).toContain(
      "lm_studio_loaded_instances_changed",
    );
    expect(JSON.stringify(changes)).toContain(
      'Removed: model ID \\"same\\", instance ID \\"instance-a\\"',
    );
    expect(JSON.stringify(changes)).toContain(
      'Added: model ID \\"same\\", instance ID \\"instance-b\\"',
    );
  });

  it("compares LM Studio loaded membership as structured identity despite display-delimiter collisions", () => {
    const prior = bundle(1, {
      lm: lmStudioModels([
        { modelId: "a", instances: ["b — instance c"] },
        { modelId: "a — instance b", instances: [] },
      ]),
    });
    const current = bundle(2, {
      lm: lmStudioModels([
        { modelId: "a", instances: [] },
        { modelId: "a — instance b", instances: ["c"] },
      ]),
    });

    const changes = diagnosticDeltas(prior, current).changes;
    expect(changes.map((change) => change.code)).not.toContain(
      "lm_studio_catalogue_set_changed",
    );
    const membershipChange = changes.find(
      (change) => change.code === "lm_studio_loaded_instances_changed",
    );
    expect(membershipChange).toEqual({
      code: "lm_studio_loaded_instances_changed",
      source: "lm_studio",
      summary:
        "LM Studio's reported loaded-instance membership changed from 1 to 1.",
      details: [
        'Added: model ID "a — instance b", instance ID "c"',
        'Removed: model ID "a", instance ID "b — instance c"',
      ],
    });

    const inverse = diagnosticDeltas(current, prior).changes.find(
      (change) => change.code === "lm_studio_loaded_instances_changed",
    );
    expect(inverse?.details).toEqual([
      'Added: model ID "a", instance ID "b — instance c"',
      'Removed: model ID "a — instance b", instance ID "c"',
    ]);
  });

  it("keeps an LM Studio source gap unknown instead of claiming loaded-membership changes", () => {
    const observed = bundle(1, {
      lm: lmStudioModels([
        { modelId: "a", instances: ["b — instance c"] },
        { modelId: "a — instance b", instances: [] },
      ]),
    });
    const changes = diagnosticDeltas(observed, bundle(2, { lm: null })).changes;
    expect(changes.filter((change) => change.source === "lm_studio")).toEqual([
      {
        code: "lm_studio_source_availability_changed",
        source: "lm_studio",
        summary:
          "LM Studio observation changed from a controlled observation to a source gap.",
        details: [],
      },
    ]);
    expect(JSON.stringify(changes)).not.toMatch(/Added:|Removed:/);
  });

  it("reports llama.cpp state and comparable served-ID changes", () => {
    const stateChanged = bundle(2, { llama: llamaCpp("same") });
    if (stateChanged.llama_cpp.kind === "observed")
      stateChanged.llama_cpp.value.state = "loading";
    expect(
      diagnosticDeltas(bundle(1, { llama: llamaCpp("same") }), stateChanged)
        .changes[0]!.code,
    ).toBe("llama_cpp_state_changed");
    expect(
      diagnosticDeltas(
        bundle(1, { llama: llamaCpp("prior") }),
        bundle(2, { llama: llamaCpp("current") }),
      ).changes.map((change) => change.code),
    ).toContain("llama_cpp_served_model_changed");
  });

  it("keeps same-looking cross-provider identities in separate source changes", () => {
    const changes = diagnosticDeltas(
      bundle(1, { catalogue: ["shared/name"], lm: lmStudio("other") }),
      bundle(2, { catalogue: [], lm: lmStudio("shared/name") }),
    ).changes;
    expect(changes.map((change) => change.source)).toEqual(
      expect.arrayContaining(["ollama_catalogue", "lm_studio"]),
    );
    expect(changes.every((change) => change.source !== "ollama_runtime")).toBe(
      true,
    );
    expect(JSON.stringify(changes)).not.toMatch(/equivalent|same model/i);
  });

  it("distinguishes first observation from no supported change", () => {
    expect(diagnosticDeltas(null, bundle())).toEqual({
      state: "no_previous",
      changes: [],
    });
  });
});

describe("Diagnosis input and source coverage", () => {
  it("projects only bounded controlled facts and the factual change count", () => {
    const observation = bundle();
    const deltas = diagnosticDeltas(bundle(1, { memory: 0 }), observation);
    const input = diagnosisInput(observation, deltas);
    expect(input.supported_change_count).toBeGreaterThan(0);
    expect(input.ollama_catalogue.kind).toBe("observed");
    const serialized = JSON.stringify(input);
    for (const excluded of [
      '"interpretation"',
      '"why_it_matters"',
      '"display_name"',
      '"size_bytes"',
      '"endpoint"',
      '"at"',
      '"error"',
    ])
      expect(serialized).not.toContain(excluded);
  });

  it("keeps source gaps explicit in input and textual coverage", () => {
    const observation = bundle(1, {
      memory: null,
      catalogue: null,
      loaded: null,
      lm: null,
      llama: null,
    });
    const input = diagnosisInput(
      observation,
      diagnosticDeltas(null, observation),
    );
    expect(input.available_memory).toEqual({ kind: "source_gap" });
    expect(input.ollama_catalogue).toEqual({ kind: "gap" });
    const coverage = sourceCoverage(observation);
    expect(coverage.filter((source) => source.gap)).toHaveLength(5);
    expect(coverage.map((source) => source.state)).toContain("Source gap");
  });
});
