/// <reference types="node" />
import { describe, expect, it, vi } from "vitest";
import { axe } from "vitest-axe";
import { render, screen, within } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { readFileSync } from "node:fs";
import App from "./App.svelte";
import type { DataSource } from "./lib/datasource";
import type { ReportClipboard } from "./lib/reportClipboard";
import type {
  LoadedModelSetView,
  LlamaCppSnapshotView,
  LmStudioSnapshotView,
  ModelInventoryView,
  MachineContextView,
  RuntimeStatusView,
  SnapshotView,
} from "./lib/types";
import {
  fixtureInferenceObservation,
  fixtureLoadedModels,
  fixtureModelInventory,
  fixtureReportPreview,
  fixtureResourceContext,
  fixtureRuntimeStatus,
  fixtureSnapshot,
} from "./lib/fixtures/states";

function llamaCppFixture(
  state: LlamaCppSnapshotView["state"] = "ready",
): LlamaCppSnapshotView {
  const ready = state === "ready";
  return {
    provider: "llama_cpp",
    state,
    served_model: ready ? { model_id: "artificial-leaf-server" } : null,
    api_scope: "same_machine_loopback",
    compute_location: "not_independently_verified",
    state_label:
      state === "authentication_required"
        ? "Authentication required"
        : state[0]!.toUpperCase() + state.slice(1),
    interpretation: ready
      ? "Artificial llama-server reports ready with one safe served-model ID."
      : `Controlled ${state} llama.cpp state.`,
    why_it_matters: ready
      ? "Loopback availability does not independently verify compute placement."
      : "Controlled passive provider state.",
  };
}

function lmStudioFixture(
  state: LmStudioSnapshotView["state"] = "available",
): LmStudioSnapshotView {
  const available = state === "available";
  return {
    provider: "lm_studio",
    state,
    models: available
      ? [
          {
            model_id: "artificial/matrix-model",
            display_name: "Artificial Matrix Model",
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
            loaded_instances: [],
          },
        ]
      : [],
    skipped: 0,
    state_label: available ? "Available" : "Not detected",
    interpretation: available
      ? "Artificial LM Studio matrix fixture is available."
      : "Artificial LM Studio matrix fixture is not detected.",
    why_it_matters: "Controlled artificial provider state.",
    resource_interpretation: "Controlled provider resource interpretation.",
    resource_qualification: "Controlled provider resource qualification.",
  };
}

function ollamaRuntimeFixture(available: boolean): RuntimeStatusView {
  if (available) return fixtureRuntimeStatus();
  return {
    ...fixtureRuntimeStatus(),
    source_availability: "not_detected",
    version: null,
    version_reported_by: null,
    status_label: "Ollama not detected",
    interpretation: "Artificial Ollama matrix fixture is not detected.",
    why_it_matters: "Controlled artificial provider state.",
  };
}

function ollamaInventoryFixture(available: boolean): ModelInventoryView {
  if (available) return fixtureModelInventory();
  return {
    ...fixtureModelInventory(),
    state: "not_detected",
    source_availability: "not_detected",
    models: [],
    skipped: 0,
    state_label: "Ollama not detected",
    interpretation: "Artificial Ollama catalogue is not detected.",
    why_it_matters: "Controlled artificial provider state.",
  };
}

function ollamaLoadedFixture(available: boolean): LoadedModelSetView {
  if (available) return fixtureLoadedModels();
  return {
    ...fixtureLoadedModels(),
    state: "not_detected",
    source_availability: "not_detected",
    models: [],
    skipped: 0,
    state_label: "Ollama not detected",
    interpretation: "Artificial Ollama loaded state is not detected.",
    why_it_matters: "Controlled artificial provider state.",
  };
}

const { createNativeReportClipboard } = vi.hoisted(() => ({
  createNativeReportClipboard: vi.fn(),
}));

vi.mock("./lib/reportClipboard", () => ({ createNativeReportClipboard }));

const cssText = readFileSync("src/app.css", "utf8");
function makeDataSource(
  snapshot: SnapshotView = fixtureSnapshot(),
  runtimeStatus: RuntimeStatusView = fixtureRuntimeStatus(),
  modelInventory: ModelInventoryView = fixtureModelInventory(),
  loadedModels: LoadedModelSetView = fixtureLoadedModels(),
  llamaCpp: LlamaCppSnapshotView = llamaCppFixture(),
  lmStudio: LmStudioSnapshotView | null = null,
): DataSource {
  return {
    isMock: true,
    currentSnapshot: vi.fn(async () => snapshot),
    machineContext: vi.fn(async () => ({
      state: "available" as const,
      native_cpu_architecture: "x86_64" as const,
      interpretation: "Artificial native CPU architecture.",
      why_it_matters: "Artificial compatibility context.",
      qualification: "Does not establish performance or compute placement.",
    })),
    reportPreview: vi.fn(async () => ({
      text: fixtureReportPreview(),
      generation: "0000000000000001",
    })),
    runtimeStatus: vi.fn(async () => runtimeStatus),
    modelInventory: vi.fn(async () => modelInventory),
    loadedModels: vi.fn(async () => loadedModels),
    llamaCppSnapshot: vi.fn(async () => llamaCpp),
    ...(lmStudio ? { lmStudioSnapshot: vi.fn(async () => lmStudio) } : {}),
    composeResourceContext: vi.fn(async () => fixtureResourceContext()),
    diagnoseObservation: vi.fn(async () => ({
      state: "available" as const,
      findings: [],
    })),
    runInferenceObservation: vi.fn(async () => fixtureInferenceObservation()),
  };
}
async function loadedView(
  ds = makeDataSource(),
  reportClipboard?: ReportClipboard | null,
) {
  const view = render(App, { props: { dataSource: ds, reportClipboard } });
  await view.findByRole("heading", { name: "Overview", level: 2 });
  return { view, ds, user: userEvent.setup() };
}
async function activate(
  user: ReturnType<typeof userEvent.setup>,
  name: string,
) {
  await user.click(screen.getByRole("tab", { name }));
}
async function runObservation(user: ReturnType<typeof userEvent.setup>) {
  await activate(user, "Observed inference");
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Model" }),
    `ollama:${fixtureModelInventory().models[0]!.display_name}`,
  );
  await user.click(screen.getByRole("button", { name: "Run observation" }));
  const dialog = await screen.findByRole("dialog");
  await user.click(
    within(dialog).getByRole("button", { name: "Run observation" }),
  );
}

describe("Milestone 1I dashboard", () => {
  it("defaults to Overview with complete tab associations and exactly one visible panel", async () => {
    const { view } = await loadedView();
    const tabs = view.getAllByRole("tab");
    expect(tabs).toHaveLength(5);
    expect(tabs[0]).toHaveAttribute("aria-selected", "true");
    expect(tabs.filter((tab) => tab.tabIndex === 0)).toHaveLength(1);
    for (const tab of tabs)
      expect(
        document.getElementById(tab.getAttribute("aria-controls")!),
      ).not.toBeNull();
    const panels = Array.from(
      view.container.querySelectorAll<HTMLElement>('[role="tabpanel"]'),
    );
    expect(panels.filter((panel) => !panel.hidden)).toHaveLength(1);
    expect(panels.filter((panel) => panel.hidden)).toHaveLength(4);
  });

  it("uses manual keyboard activation with arrows, Home, End, Enter and Space", async () => {
    const { view, user } = await loadedView();
    const overview = view.getByRole("tab", { name: "Overview" });
    overview.focus();
    await user.keyboard("{ArrowRight}");
    expect(view.getByRole("tab", { name: "Models" })).toHaveFocus();
    expect(overview).toHaveAttribute("aria-selected", "true");
    await user.keyboard("{Enter}");
    expect(view.getByRole("tab", { name: "Models" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await user.keyboard("{End} ");
    expect(view.getByRole("tab", { name: "Report" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await user.keyboard("{Home}");
    expect(overview).toHaveFocus();
  });

  it("navigation makes no datasource or inference calls", async () => {
    const { user, ds } = await loadedView();
    const calls = [
      ds.currentSnapshot,
      ds.reportPreview,
      ds.runtimeStatus,
      ds.modelInventory,
      ds.loadedModels,
      ds.llamaCppSnapshot!,
      ds.composeResourceContext,
      ds.diagnoseObservation,
      ds.runInferenceObservation,
    ] as const;
    const counts = calls.map((call) => vi.mocked(call).mock.calls.length);
    for (const name of [
      "Models",
      "Observed inference",
      "Diagnose",
      "Report",
      "Overview",
    ])
      await activate(user, name);
    calls.forEach((call, index) =>
      expect(vi.mocked(call).mock.calls.length).toBe(counts[index]),
    );
  });

  it("shows passive llama.cpp state and safe served-model semantics without inference eligibility", async () => {
    const { view, user, ds } = await loadedView();
    expect(
      view.getByRole("heading", { name: "llama.cpp runtime" }),
    ).toBeVisible();
    const providerHeadings = [
      view.getByRole("heading", { name: "Ollama runtime" }),
      view.getByRole("heading", { name: "LM Studio runtime" }),
      view.getByRole("heading", { name: "llama.cpp runtime" }),
    ];
    expect(
      providerHeadings[0]!.compareDocumentPosition(providerHeadings[1]!) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      providerHeadings[1]!.compareDocumentPosition(providerHeadings[2]!) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      view.getByText("Served model: artificial-leaf-server"),
    ).toBeVisible();
    await activate(user, "Models");
    expect(
      view.getByRole("heading", { name: "llama.cpp served model" }),
    ).toBeVisible();
    expect(view.getByText("Served-model ID")).toBeVisible();
    expect(view.getByText("artificial-leaf-server")).toBeVisible();
    expect(view.getByText("Same-machine loopback")).toBeVisible();
    expect(view.getByText("Not independently verified")).toBeVisible();
    await activate(user, "Observed inference");
    const options = Array.from(
      view.getByRole("combobox", { name: "Model" }).querySelectorAll("option"),
      (option) => option.textContent ?? "",
    );
    expect(options.join(" ")).not.toMatch(/llama\.cpp|artificial-leaf-server/i);
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it.each(["core", "ollama", "lm_studio", "llama_cpp"] as const)(
    "keeps successful provider and core results visible when %s acquisition rejects",
    async (failedProvider) => {
      const rawError = `private ${failedProvider} IPC failure /home/owner`;
      const ds = makeDataSource(
        fixtureSnapshot(),
        fixtureRuntimeStatus(),
        fixtureModelInventory(),
        fixtureLoadedModels(),
        llamaCppFixture(),
        lmStudioFixture(),
      );
      if (failedProvider === "core") {
        ds.currentSnapshot = vi.fn(async () => {
          throw new Error(rawError);
        });
      } else if (failedProvider === "ollama") {
        ds.runtimeStatus = vi.fn(async () => {
          throw new Error(rawError);
        });
      } else if (failedProvider === "lm_studio") {
        ds.lmStudioSnapshot = vi.fn(async () => {
          throw new Error(rawError);
        });
      } else {
        ds.llamaCppSnapshot = vi.fn(async () => {
          throw new Error(rawError);
        });
      }

      const { view } = await loadedView(ds);
      if (failedProvider !== "core") {
        expect(
          view.getByText("8,589,934,592 bytes", {
            selector: ".summary-primary",
          }),
        ).toBeVisible();
      }
      if (failedProvider !== "ollama") {
        expect(
          view
            .getByRole("heading", { name: "Ollama runtime" })
            .closest("article"),
        ).toHaveTextContent("Ollama available");
      }
      if (failedProvider !== "lm_studio") {
        expect(
          view
            .getByRole("heading", { name: "LM Studio runtime" })
            .closest("article"),
        ).toHaveTextContent("Available");
      }
      if (failedProvider !== "llama_cpp") {
        expect(
          view
            .getByRole("heading", { name: "llama.cpp runtime" })
            .closest("article"),
        ).toHaveTextContent("Ready");
      }
      expect(view.container).not.toHaveTextContent(rawError);
      expect(ds.runInferenceObservation).not.toHaveBeenCalled();
    },
  );

  it.each([
    ["Ollama only", true, false, false],
    ["LM Studio only", false, true, false],
    ["llama.cpp only", false, false, true],
    ["Ollama + LM Studio", true, true, false],
    ["Ollama + llama.cpp", true, false, true],
    ["LM Studio + llama.cpp", false, true, true],
    ["Ollama + LM Studio + llama.cpp", true, true, true],
  ] as const)(
    "keeps provider state independent for %s",
    async (_label, ollamaAvailable, lmStudioAvailable, llamaCppAvailable) => {
      const ds = makeDataSource(
        fixtureSnapshot(),
        ollamaRuntimeFixture(ollamaAvailable),
        ollamaInventoryFixture(ollamaAvailable),
        ollamaLoadedFixture(ollamaAvailable),
        llamaCppFixture(llamaCppAvailable ? "ready" : "unreachable"),
        lmStudioFixture(lmStudioAvailable ? "available" : "not_detected"),
      );
      const { view, user } = await loadedView(ds);

      const ollamaCard = view
        .getByRole("heading", { name: "Ollama runtime" })
        .closest("article")!;
      const lmStudioCard = view
        .getByRole("heading", { name: "LM Studio runtime" })
        .closest("article")!;
      const llamaCppCard = view
        .getByRole("heading", { name: "llama.cpp runtime" })
        .closest("article")!;
      expect(ollamaCard).toHaveTextContent(
        ollamaAvailable ? "Ollama available" : "Ollama not detected",
      );
      expect(lmStudioCard).toHaveTextContent(
        lmStudioAvailable ? "Available" : "Not detected",
      );
      expect(llamaCppCard).toHaveTextContent(
        llamaCppAvailable ? "Ready" : "Unreachable",
      );

      await activate(user, "Models");
      expect(
        view.getByRole("heading", { name: "Currently loaded" }),
      ).toBeVisible();
      expect(view.getByRole("heading", { name: "LM Studio" })).toBeVisible();
      expect(
        view.getByRole("heading", { name: "llama.cpp served model" }),
      ).toBeVisible();
      const lmStudioSection = view
        .getByRole("heading", { name: "LM Studio" })
        .closest("section")!;
      const llamaCppSection = view
        .getByRole("heading", { name: "llama.cpp served model" })
        .closest("section")!;
      if (lmStudioAvailable) {
        expect(
          within(lmStudioSection).getByText(/Artificial Matrix Model/),
        ).toBeVisible();
      } else {
        expect(
          within(lmStudioSection).queryByText(/Artificial Matrix Model/),
        ).toBeNull();
      }
      if (llamaCppAvailable) {
        expect(
          within(llamaCppSection).getByText("artificial-leaf-server"),
        ).toBeVisible();
      } else {
        expect(
          within(llamaCppSection).queryByText("artificial-leaf-server"),
        ).toBeNull();
      }

      await activate(user, "Observed inference");
      const selector = view.queryByRole("combobox", { name: "Model" });
      if (ollamaAvailable || lmStudioAvailable) {
        expect(selector).not.toBeNull();
        expect(selector).toHaveValue("");
        const optionValues = Array.from(
          selector!.querySelectorAll<HTMLOptionElement>("option"),
          (option) => option.value,
        );
        expect(optionValues.some((value) => value.startsWith("ollama:"))).toBe(
          ollamaAvailable,
        );
        expect(
          optionValues.some((value) => value.startsWith("lm_studio:")),
        ).toBe(lmStudioAvailable);
        expect(optionValues.some((value) => value.includes("llama_cpp"))).toBe(
          false,
        );
      } else {
        expect(selector).toBeNull();
        expect(
          view.queryByRole("button", { name: "Run observation" }),
        ).toBeNull();
      }
      expect(ds.runInferenceObservation).not.toHaveBeenCalled();
    },
  );

  it("presents LM Studio resource metadata with controlled qualifications and explicit missing states", async () => {
    const lm = lmStudioFixture();
    lm.models[0]!.loaded_instances = [
      { instance_id: "artificial-instance", context_length: null },
    ];
    const { view, user } = await loadedView(
      makeDataSource(
        fixtureSnapshot(),
        fixtureRuntimeStatus(),
        fixtureModelInventory(),
        fixtureLoadedModels(),
        llamaCppFixture(),
        lm,
      ),
    );
    await activate(user, "Models");
    expect(view.getByText(lm.resource_interpretation)).toBeVisible();
    expect(view.getByText(lm.resource_qualification)).toBeVisible();
    await user.click(
      view.getByText(/Artificial Matrix Model/, { selector: "summary" }),
    );
    expect(
      view.getByText("Catalogue size reported by LM Studio"),
    ).toBeVisible();
    expect(
      view.getByText("Maximum context metadata reported by LM Studio"),
    ).toBeVisible();
    expect(view.getAllByText("Not reported by LM Studio")).toHaveLength(2);
    expect(
      view.getByText(/configured context not reported by LM Studio/),
    ).toBeVisible();
  });

  it("keeps Ollama and LM Studio usable when llama.cpp is incompatible", async () => {
    const unsafeMarker = "/private/artificial-model.gguf";
    const llamaCpp = llamaCppFixture("incompatible");
    llamaCpp.served_model = { model_id: unsafeMarker };
    const ds = makeDataSource(
      fixtureSnapshot(),
      ollamaRuntimeFixture(true),
      ollamaInventoryFixture(true),
      ollamaLoadedFixture(true),
      llamaCpp,
      lmStudioFixture("available"),
    );
    const { view, user } = await loadedView(ds);

    expect(
      view.getByRole("heading", { name: "Ollama runtime" }).closest("article"),
    ).toHaveTextContent("Ollama available");
    expect(
      view
        .getByRole("heading", { name: "LM Studio runtime" })
        .closest("article"),
    ).toHaveTextContent("Available");
    expect(
      view
        .getByRole("heading", { name: "llama.cpp runtime" })
        .closest("article"),
    ).toHaveTextContent("Incompatible");
    expect(view.container).not.toHaveTextContent(unsafeMarker);

    await activate(user, "Observed inference");
    const selector = view.getByRole("combobox", { name: "Model" });
    expect(selector).toHaveValue("");
    const options = Array.from(
      selector.querySelectorAll<HTMLOptionElement>("option"),
      (option) => option.value,
    );
    expect(options.some((value) => value.startsWith("ollama:"))).toBe(true);
    expect(options.some((value) => value.startsWith("lm_studio:"))).toBe(true);
    expect(options.some((value) => value.includes("llama_cpp"))).toBe(false);
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it.each([
    "unreachable",
    "loading",
    "authentication_required",
    "incompatible",
  ] as const)(
    "renders controlled llama.cpp %s state without a model ID",
    async (state) => {
      const marker = "/private/artificial-model.gguf";
      const fixture = llamaCppFixture(state);
      // Deliberately represent an impossible backend DTO to prove the UI's
      // non-ready state gate never renders a path-bearing served-model value.
      fixture.served_model = { model_id: marker };
      fixture.interpretation = `${fixture.interpretation} No returned identity is displayed.`;
      const { view, user } = await loadedView(
        makeDataSource(
          fixtureSnapshot(),
          fixtureRuntimeStatus(),
          fixtureModelInventory(),
          fixtureLoadedModels(),
          fixture,
        ),
      );
      const overviewCard = view
        .getByRole("heading", { name: "llama.cpp runtime" })
        .closest("article")!;
      expect(overviewCard).toHaveTextContent(fixture.interpretation);
      expect(overviewCard).not.toHaveTextContent(marker);
      expect(
        view.queryByText("Served model: artificial-leaf-server"),
      ).toBeNull();
      await activate(user, "Models");
      const modelsSection = view
        .getByRole("heading", { name: "llama.cpp served model" })
        .closest("section")!;
      expect(modelsSection).toHaveTextContent(fixture.state_label);
      expect(modelsSection).not.toHaveTextContent(marker);
      expect(view.queryByText("artificial-leaf-server")).toBeNull();
    },
  );

  it("shows approved Overview values and disclosures without new acquisition", async () => {
    const { view, ds, user } = await loadedView();
    expect(
      view.getByText("8,589,934,592 bytes", { selector: ".summary-primary" }),
    ).toBeVisible();
    expect(
      view
        .getByRole("heading", { name: "Available memory", level: 3 })
        .closest("article"),
    ).toHaveTextContent("8.59 GB");
    expect(
      view.getAllByText("Reported by the operating system", {
        selector: ".summary-source",
      }),
    ).toHaveLength(2);
    expect(
      view
        .getByRole("heading", { name: "Total memory", level: 3 })
        .closest("article"),
    ).toHaveTextContent("17.18 GB");
    expect(
      view
        .getByRole("heading", { name: "Native CPU architecture", level: 3 })
        .closest("article"),
    ).toHaveTextContent("x86_64");
    expect(view.getByText("No observation run this session")).toBeVisible();
    expect(view.getByText(/must not be added or subtracted/)).toBeVisible();
    const resourceContext = view.getByLabelText("Resource context");
    const interpretation = view
      .getByText("How to interpret memory and loaded-model values", {
        selector: "summary",
      })
      .closest("details")!;
    const metricDetails = view
      .getByText("Metric details", { selector: "summary" })
      .closest("details")!;
    expect(resourceContext).toContainElement(interpretation);
    expect(resourceContext).toContainElement(metricDetails);
    expect(interpretation).not.toHaveAttribute("open");
    expect(metricDetails).not.toHaveAttribute("open");
    expect(metricDetails.parentElement).toBe(resourceContext);
    const calls = [
      ds.currentSnapshot,
      ds.machineContext,
      ds.reportPreview,
      ds.runtimeStatus,
      ds.modelInventory,
      ds.loadedModels,
      ds.composeResourceContext,
      ds.runInferenceObservation,
    ] as const;
    const counts = calls.map((call) => vi.mocked(call).mock.calls.length);
    await user.click(
      view.getByText("How to interpret memory and loaded-model values"),
    );
    expect(
      view.getByText(fixtureResourceContext().interpretation),
    ).toBeVisible();
    expect(
      view.getByText(fixtureResourceContext().why_it_matters),
    ).toBeVisible();
    await user.click(
      view.getByText("How to interpret memory and loaded-model values"),
    );
    await user.click(view.getByText("Metric details", { selector: "summary" }));
    expect(
      Array.from(metricDetails.querySelectorAll(".metric-name")).map(
        (heading) => heading.textContent,
      ),
    ).toEqual(fixtureSnapshot().entries.map((entry) => entry.display_name));
    await user.click(view.getByText("Metric details", { selector: "summary" }));
    calls.forEach((call, index) =>
      expect(vi.mocked(call).mock.calls.length).toBe(counts[index]),
    );
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("renders total-memory zero, unavailable, and unsafe values without fabrication", async () => {
    const snapshotWithTotal = (
      outcome: SnapshotView["entries"][number]["outcome"],
    ) => ({
      ...fixtureSnapshot(),
      entries: fixtureSnapshot().entries.map((entry) =>
        entry.display_name === "Total memory" ? { ...entry, outcome } : entry,
      ),
    });

    const zeroView = await loadedView(
      makeDataSource(
        snapshotWithTotal({
          kind: "ok",
          value: 0,
          unit: "bytes",
          provenance: "operating_system_reported",
        }),
      ),
    );
    const zeroCard = zeroView.view
      .getByRole("heading", { name: "Total memory", level: 3 })
      .closest("article")!;
    expect(zeroCard).toHaveTextContent("0 bytes");
    expect(zeroCard).toHaveTextContent("0 B");
    zeroView.view.unmount();

    const unavailableView = await loadedView(
      makeDataSource(snapshotWithTotal({ kind: "no_value" })),
    );
    expect(
      unavailableView.view
        .getByRole("heading", { name: "Total memory", level: 3 })
        .closest("article"),
    ).toHaveTextContent("Unavailable — no value reported");
    unavailableView.view.unmount();

    const unsafeView = await loadedView(
      makeDataSource(
        snapshotWithTotal({
          kind: "ok",
          value: Number.MAX_SAFE_INTEGER + 1,
          unit: "bytes",
          provenance: "operating_system_reported",
        }),
      ),
    );
    const unsafeCard = unsafeView.view
      .getByRole("heading", { name: "Total memory", level: 3 })
      .closest("article")!;
    expect(unsafeCard).toHaveTextContent("Exact value not representable");
    expect(unsafeCard).toHaveTextContent("≈");
    expect(unsafeCard).not.toHaveTextContent(
      `${(Number.MAX_SAFE_INTEGER + 1).toLocaleString("en-US")} bytes`,
    );
  });

  it("renders controlled unknown and failed native-architecture states", async () => {
    const unknown = makeDataSource();
    unknown.machineContext = vi.fn(async (): Promise<MachineContextView> => ({
      state: "available",
      native_cpu_architecture: "unknown",
      interpretation:
        "The operating system reports an unmapped native CPU architecture.",
      why_it_matters: "Architecture provides compatibility context.",
      qualification:
        "Architecture alone does not establish model compatibility, acceleration, performance, or compute placement.",
    }));
    const unknownView = await loadedView(unknown);
    expect(
      unknownView.view
        .getByRole("heading", { name: "Native CPU architecture", level: 3 })
        .closest("article"),
    ).toHaveTextContent("unknown");
    unknownView.view.unmount();

    const failed = makeDataSource();
    failed.machineContext = vi.fn(async (): Promise<MachineContextView> => ({
      state: "failed",
      native_cpu_architecture: null,
      interpretation: "The native CPU architecture is not available right now.",
      why_it_matters: "Architecture provides compatibility context.",
      qualification:
        "Architecture alone does not establish model compatibility, acceleration, performance, or compute placement.",
    }));
    const failedView = await loadedView(failed);
    expect(
      failedView.view
        .getByRole("heading", { name: "Native CPU architecture", level: 3 })
        .closest("article"),
    ).toHaveTextContent("Unavailable — no value reported");
  });

  it("distinguishes unavailable summary states from zero", async () => {
    const snapshot = {
      ...fixtureSnapshot(),
      entries: fixtureSnapshot().entries.map((entry) =>
        entry.display_name === "Available memory"
          ? { ...entry, outcome: { kind: "no_value" as const } }
          : entry,
      ),
    };
    const inventory = {
      ...fixtureModelInventory(),
      state: "unreachable" as const,
      models: [],
    };
    const loaded = {
      ...fixtureLoadedModels(),
      state: "unreachable" as const,
      models: [],
    };
    const { view } = await loadedView(
      makeDataSource(snapshot, fixtureRuntimeStatus(), inventory, loaded),
    );
    expect(view.getAllByText(/Unavailable/).length).toBeGreaterThanOrEqual(3);
    expect(view.queryByText("0")).toBeNull();
  });

  it("places loaded models before a compact, API-ordered filterable catalogue", async () => {
    const { view, user, ds } = await loadedView();
    await activate(user, "Models");
    const loadedHeading = view.getByRole("heading", {
      name: "Currently loaded",
    });
    const catalogueHeading = view.getByRole("heading", {
      name: "Available catalogue",
    });
    expect(
      loadedHeading.compareDocumentPosition(catalogueHeading) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    const summaries = view
      .getAllByText(/^example-(chat|coder|embed)/)
      .filter((node) => node.tagName === "SUMMARY");
    expect(summaries.map((node) => node.textContent)).toEqual(
      fixtureModelInventory().models.map((model) => model.display_name),
    );
    expect(summaries[0]!.closest("details")).not.toHaveAttribute("open");
    const filter = view.getByRole("searchbox", {
      name: "Filter catalogue by model name",
    });
    await user.type(filter, "CODER:3");
    expect(view.getByText("1 of 3 catalogue models shown")).toBeVisible();
    expect(
      view.getByText("example-coder:3b", { selector: "summary" }),
    ).toBeVisible();
    await user.clear(filter);
    await user.type(filter, "zzzz-missing");
    expect(
      view.getByText("No catalogue model names match this filter."),
    ).toBeVisible();
    expect(ds.modelInventory).toHaveBeenCalledTimes(1);
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(1);
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("preserves model, current result, history, comparison and filter across workspace switches", async () => {
    const { view, user, ds } = await loadedView();
    await activate(user, "Models");
    await user.type(
      view.getByRole("searchbox", { name: "Filter catalogue by model name" }),
      "chat",
    );
    await runObservation(user);
    expect(await view.findByText("Observation 1")).toBeVisible();
    await runObservation(user);
    expect(await view.findByText("Observation 2")).toBeVisible();
    const boxes = view.getAllByRole("checkbox");
    await user.click(boxes[0]!);
    await user.click(boxes[1]!);
    await activate(user, "Overview");
    expect(
      view.getByText(fixtureInferenceObservation().interpretation, {
        selector: ".summary-primary",
      }),
    ).toBeVisible();
    expect(view.getByText("2 retained this session")).toBeVisible();
    await activate(user, "Models");
    expect(
      view.getByRole("searchbox", { name: "Filter catalogue by model name" }),
    ).toHaveValue("chat");
    await activate(user, "Observed inference");
    expect(view.getByRole("combobox", { name: "Model" })).toHaveValue(
      `ollama:${fixtureModelInventory().models[0]!.display_name}`,
    );
    expect(
      view.getByRole("heading", { name: "Compare observations" }),
    ).toBeVisible();
    expect(ds.runInferenceObservation).toHaveBeenCalledTimes(2);
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(1);
  });

  it("Report uses the already acquired narrow preview and does not expose dashboard state", async () => {
    const { view, user, ds } = await loadedView();
    await activate(user, "Report");
    expect(
      view.getByText(/intentionally narrower than the full app state/),
    ).toBeVisible();
    expect(view.getByText("Report-safe preview")).toBeVisible();
    const preview = view.container.querySelector(".report-preview-text")!;
    expect(preview).toHaveTextContent("Memory in use");
    expect(preview).not.toHaveTextContent("example-chat:8b");
    expect(preview).not.toHaveTextContent("llama_cpp");
    expect(preview).not.toHaveTextContent("artificial-leaf-server");
    expect(preview).not.toHaveTextContent("127.0.0.1:8080");
    expect(preview).not.toHaveTextContent("same_machine_loopback");
    expect(preview).not.toHaveTextContent("not_independently_verified");
    expect(
      view.getByText("Copy report is available in the native app."),
    ).toBeVisible();
    expect(view.queryByRole("button", { name: "Copy report" })).toBeNull();
    expect(
      view.getByText(/does not automatically send or upload/),
    ).toBeVisible();
    expect(ds.reportPreview).toHaveBeenCalledTimes(1);
  });

  it("keeps diagnostic findings and provider-qualified model IDs out of Report", async () => {
    const ds = makeDataSource();
    ds.diagnoseObservation = vi.fn(async () => ({
      state: "available" as const,
      findings: [
        {
          code: "ollama_catalogued_not_loaded",
          source: "ollama" as const,
          observation:
            "Ollama model artificial-private-correlation:1b is catalogued but not reported loaded.",
          meaning: "Controlled same-provider correlation.",
          safe_next_check: "Review Ollama separately.",
        },
      ],
    }));
    const { view, user } = await loadedView(ds);
    await activate(user, "Report");
    const preview = view.container.querySelector(".report-preview-text")!;
    expect(preview).not.toHaveTextContent("artificial-private-correlation:1b");
    expect(preview.textContent).toBe(fixtureReportPreview());
    await activate(user, "Diagnose");
    expect(view.getByText(/artificial-private-correlation:1b/)).toBeVisible();
    expect(ds.reportPreview).toHaveBeenCalledTimes(1);
  });

  it("keeps Report preview and Copy byte-for-byte unchanged by machine context", async () => {
    const ds: DataSource = { ...makeDataSource(), isMock: false };
    ds.machineContext = vi.fn(async () => ({
      state: "available" as const,
      native_cpu_architecture: "unknown" as const,
      interpretation: "Controlled unknown architecture.",
      why_it_matters: "Compatibility context only.",
      qualification: "No performance or placement claim.",
    }));
    const copyText = vi.fn(async (_text: string) => {});
    const { view, user } = await loadedView(ds, { copyText });
    await activate(user, "Report");
    const preview = view.container.querySelector(".report-preview-text")!;
    expect(preview.textContent).toBe(fixtureReportPreview());
    expect(preview).not.toHaveTextContent("Controlled unknown architecture.");
    await user.click(view.getByRole("button", { name: "Copy report" }));
    expect(copyText).toHaveBeenCalledWith(fixtureReportPreview());
  });

  it("keeps the separately acquired Report value distinct from session history", async () => {
    const ds = makeDataSource();
    ds.reportPreview = vi.fn(async () => ({
      text: [
        "AI Engine Room — Observation Report",
        "",
        "Observation",
        "Available memory",
        "Status: Available",
        "Value: 12.96 GB",
        "Exact value: 12,961,325,056 bytes",
        "Source: Operating system",
        "",
      ].join("\n"),
      generation: "0000000000000002",
    }));
    const { view, user } = await loadedView(ds);
    expect(
      view.getByText("8,589,934,592 bytes", { selector: "dd" }),
    ).toBeVisible();
    await activate(user, "Report");
    const preview = view.container.querySelector(".report-preview-text")!;
    expect(preview).toHaveTextContent("12,961,325,056 bytes");
    expect(preview).not.toHaveTextContent("8,589,934,592 bytes");
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(1);
    expect(ds.reportPreview).toHaveBeenCalledTimes(1);
  });

  it("copies only on explicit native activation without acquisition, inference, or history mutation", async () => {
    const ds: DataSource = { ...makeDataSource(), isMock: false };
    const copyText = vi.fn(async (_text: string) => {});
    const clipboard: ReportClipboard = { copyText };
    const { view, user } = await loadedView(ds, clipboard);
    const calls = [
      ds.currentSnapshot,
      ds.reportPreview,
      ds.runtimeStatus,
      ds.modelInventory,
      ds.loadedModels,
      ds.composeResourceContext,
    ] as const;
    const counts = calls.map((call) => vi.mocked(call).mock.calls.length);

    expect(copyText).not.toHaveBeenCalled();
    await activate(user, "Report");
    expect(copyText).not.toHaveBeenCalled();
    await user.click(view.getByRole("button", { name: "Copy report" }));
    expect(copyText).toHaveBeenCalledTimes(1);
    expect(copyText).toHaveBeenCalledWith(fixtureReportPreview());
    calls.forEach((call, index) =>
      expect(vi.mocked(call).mock.calls.length).toBe(counts[index]),
    );
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
    expect(
      view.getByText(
        "Completed observations from this app session will appear here.",
      ),
    ).toBeInTheDocument();

    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 1")).toBeVisible();
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(1);

    await activate(user, "Overview");
    expect(view.getByText("1 observation; 1 numeric reading")).toBeVisible();
    await activate(user, "Report");
    expect(copyText).toHaveBeenCalledTimes(1);
    await user.click(view.getByRole("button", { name: "Refresh" }));
    expect(copyText).toHaveBeenCalledTimes(1);
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
    await activate(user, "Observed inference");
    expect(
      view.getByText(
        "Completed observations from this app session will appear here.",
      ),
    ).toBeVisible();
  });

  it("passes only the current report generation to the bounded native save command", async () => {
    const ds: DataSource = { ...makeDataSource(), isMock: false };
    ds.saveReport = vi.fn(async () => "saved" as const);
    const { view, user } = await loadedView(ds, {
      copyText: vi.fn(async () => {}),
    });
    await activate(user, "Report");

    await user.click(view.getByRole("button", { name: "Save report…" }));
    expect(ds.saveReport).toHaveBeenCalledTimes(1);
    expect(ds.saveReport).toHaveBeenCalledWith("0000000000000001");
    expect(view.getByRole("status")).toHaveTextContent(
      "Report saved as a plain-text file.",
    );
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("never exposes save through an injected browser/mock data source", async () => {
    const ds = makeDataSource();
    ds.saveReport = vi.fn(async () => "saved" as const);
    const { view, user } = await loadedView(ds);
    await activate(user, "Report");
    expect(view.queryByRole("button", { name: "Save report…" })).toBeNull();
    expect(ds.saveReport).not.toHaveBeenCalled();
  });

  it("classifies native clipboard initialization failure without exposing its raw error", async () => {
    const rawError = "private plugin import failure /home/owner";
    createNativeReportClipboard.mockRejectedValueOnce(new Error(rawError));
    const ds: DataSource = { ...makeDataSource(), isMock: false };
    const { view, user } = await loadedView(ds);

    await activate(user, "Report");
    expect(
      view.getByText(
        "Clipboard export could not be initialized in this native session.",
      ),
    ).toBeVisible();
    expect(
      view.queryByText("Copy report is available in the native app."),
    ).toBeNull();
    expect(view.queryByRole("button", { name: "Copy report" })).toBeNull();
    expect(view.container).not.toHaveTextContent(rawError);
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("Refresh preserves active workspace and never runs inference", async () => {
    const { view, user, ds } = await loadedView();
    await activate(user, "Models");
    await user.click(view.getByRole("button", { name: "Refresh" }));
    expect(view.getByRole("tab", { name: "Models" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(2);
    expect(ds.llamaCppSnapshot).toHaveBeenCalledTimes(2);
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(2);
  });

  it("retains one startup diagnostic observation and navigation adds none", async () => {
    const { view, user, ds } = await loadedView();
    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 1")).toBeVisible();
    expect(
      view.getByText("Diagnostic observation 1", { selector: "summary" }),
    ).toBeVisible();
    await activate(user, "Models");
    await activate(user, "Report");
    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 1")).toBeVisible();
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(1);
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(1);
  });

  it("retains startup and explicit Refresh Available-memory observations only", async () => {
    const { view, user, ds } = await loadedView();
    expect(view.getByText("1 observation; 1 numeric reading")).toBeVisible();
    expect(view.getByText("Observation 1:")).toBeInTheDocument();

    await activate(user, "Report");
    await activate(user, "Models");
    await activate(user, "Overview");
    expect(view.getByText("1 observation; 1 numeric reading")).toBeVisible();

    await user.click(view.getByRole("button", { name: "Refresh" }));
    expect(
      await view.findByText("2 observations; 2 numeric readings"),
    ).toBeVisible();
    expect(view.getByText("Observation 2:")).toBeInTheDocument();
    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 2")).toBeVisible();
    expect(
      view.getByText("Diagnostic observation 2", { selector: "summary" }),
    ).toBeVisible();
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(2);
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("does not append Available-memory history for observed inference", async () => {
    const { view, user, ds } = await loadedView();
    await runObservation(user);
    expect(await view.findByText("Observation 1")).toBeVisible();
    await activate(user, "Overview");
    expect(view.getByText("1 observation; 1 numeric reading")).toBeVisible();
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(1);
    expect(ds.runInferenceObservation).toHaveBeenCalledTimes(1);
    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 1")).toBeVisible();
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(1);
  });

  it("keeps one Refresh bundle together when source promises settle out of order", async () => {
    const ds = makeDataSource();
    const { view, user } = await loadedView(ds);
    let resolveCatalogue!: (value: ModelInventoryView) => void;
    let resolveLoaded!: (value: LoadedModelSetView) => void;
    const cataloguePromise = new Promise<ModelInventoryView>((resolve) => {
      resolveCatalogue = resolve;
    });
    const loadedPromise = new Promise<LoadedModelSetView>((resolve) => {
      resolveLoaded = resolve;
    });
    vi.mocked(ds.modelInventory).mockImplementationOnce(
      async () => cataloguePromise,
    );
    vi.mocked(ds.loadedModels).mockImplementationOnce(
      async () => loadedPromise,
    );

    const refresh = user.click(view.getByRole("button", { name: "Refresh" }));
    const changedLoaded = fixtureLoadedModels();
    changedLoaded.models = [
      { ...changedLoaded.models[0]!, display_name: "refresh-pair:model" },
    ];
    resolveLoaded(changedLoaded);
    const changedCatalogue = fixtureModelInventory();
    changedCatalogue.models = [
      { ...changedCatalogue.models[0]!, display_name: "refresh-pair:model" },
    ];
    resolveCatalogue(changedCatalogue);
    await refresh;
    await view.findByRole("button", { name: "Refresh" });

    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 2")).toBeVisible();
    expect(
      view.getByText(/Ollama catalogue changed from 3 to 1/),
    ).toBeVisible();
    expect(
      view.getByText(/Ollama reported running-model set changed from 3 to 1/),
    ).toBeVisible();
    expect(view.getAllByText(/refresh-pair:model/).length).toBeGreaterThan(0);
  });

  it("turns a rejected provider source into a gap without false model removals", async () => {
    const ds = makeDataSource();
    const { view, user } = await loadedView(ds);
    const rawError = "private catalogue failure /home/owner";
    vi.mocked(ds.modelInventory).mockRejectedValueOnce(new Error(rawError));
    await user.click(view.getByRole("button", { name: "Refresh" }));
    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 2")).toBeVisible();
    expect(
      view.getByText(
        "Ollama catalogue changed from a controlled observation to a source gap.",
      ),
    ).toBeVisible();
    expect(view.container).not.toHaveTextContent(rawError);
    expect(view.container).not.toHaveTextContent("Removed: example-chat:8b");
  });

  it("still appends one bundle when pure diagnosis is unavailable", async () => {
    const ds = makeDataSource();
    const { view, user } = await loadedView(ds);
    vi.mocked(ds.diagnoseObservation).mockRejectedValueOnce(
      new Error("private diagnosis transport detail"),
    );
    await user.click(view.getByRole("button", { name: "Refresh" }));
    await activate(user, "Diagnose");
    expect(view.getByText("Latest observation: 2")).toBeVisible();
    expect(
      view.getByText(
        "Controlled diagnostic findings are unavailable for this observation.",
      ),
    ).toBeVisible();
    expect(view.container).not.toHaveTextContent("private diagnosis");
  });

  it("disables and ignores overlapping Refresh activation", async () => {
    const ds = makeDataSource();
    const initial = await loadedView(ds);
    let resolveSnapshot!: (snapshot: SnapshotView) => void;
    const pendingSnapshot = new Promise<SnapshotView>((resolve) => {
      resolveSnapshot = resolve;
    });
    vi.mocked(ds.currentSnapshot).mockImplementationOnce(
      async () => pendingSnapshot,
    );

    const button = initial.view.getByRole("button", { name: "Refresh" });
    button.focus();
    await initial.user.click(button);
    const pendingButton = initial.view.getByRole("button", {
      name: "Refreshing…",
    });
    expect(pendingButton).toBeDisabled();
    expect(pendingButton).toHaveAttribute("aria-busy", "true");
    expect(pendingButton).toHaveFocus();
    await initial.user.click(pendingButton);
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(2);

    resolveSnapshot(fixtureSnapshot());
    expect(
      await initial.view.findByText("2 observations; 2 numeric readings"),
    ).toBeVisible();
    expect(ds.diagnoseObservation).toHaveBeenCalledTimes(2);
    expect(initial.view.getByRole("button", { name: "Refresh" })).toHaveFocus();
  });

  it("records a rejected snapshot as a nonnumeric event without raw error detail", async () => {
    const ds = makeDataSource();
    const rawError = "private snapshot stack /home/owner";
    ds.currentSnapshot = vi.fn(async () => {
      throw new Error(rawError);
    });
    const { view } = await loadedView(ds);
    expect(view.getByText("1 observation; 0 numeric readings")).toBeVisible();
    expect(
      view.getAllByText(/Snapshot acquisition unavailable/).length,
    ).toBeGreaterThan(0);
    expect(view.container).not.toHaveTextContent(rawError);
    expect(view.container).not.toHaveTextContent("0 B");
  });

  it("remount resets the workspace and Available-memory session history", async () => {
    const first = await loadedView();
    await first.user.click(first.view.getByRole("button", { name: "Refresh" }));
    expect(
      await first.view.findByText("2 observations; 2 numeric readings"),
    ).toBeVisible();
    await activate(first.user, "Report");
    first.view.unmount();
    const second = await loadedView();
    expect(second.view.getByRole("tab", { name: "Overview" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      second.view.getByText("1 observation; 1 numeric reading"),
    ).toBeVisible();
    await activate(second.user, "Diagnose");
    expect(second.view.getByText("Latest observation: 1")).toBeVisible();
  });

  it("has landmarks, no representative axe violations, responsive and reduced-motion hooks", async () => {
    const { view } = await loadedView();
    expect(view.getByRole("banner")).toBeVisible();
    expect(
      view.getByRole("navigation", { name: "Primary workspaces" }),
    ).toBeVisible();
    expect(view.getByRole("main")).toBeVisible();
    const results = await axe(view.container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
    expect(cssText).toMatch(/max-width: 38rem/);
    expect(cssText).toMatch(/prefers-reduced-motion: reduce/);
  });

  it("keeps session observation history free of polling and persistence", () => {
    const appSource = readFileSync("src/App.svelte", "utf8");
    const historySource = readFileSync(
      "src/lib/availableMemoryHistory.ts",
      "utf8",
    );
    const diagnosticSource = readFileSync(
      "src/lib/diagnosticObservation.ts",
      "utf8",
    );
    const combined = `${appSource}\n${historySource}\n${diagnosticSource}`;
    expect(combined).not.toMatch(/setInterval|setTimeout/);
    expect(combined).not.toMatch(
      /localStorage|sessionStorage|indexedDB|persist|filesystem/i,
    );
  });

  it("gives the catalogue results a bounded local vertical-scroll contract", () => {
    const catalogueRule = cssText.match(
      /\.catalogue-results\s*\{([^}]*)\}/,
    )?.[1];

    expect(catalogueRule).toBeDefined();
    expect(catalogueRule).toMatch(/max-height:\s*24rem/);
    expect(catalogueRule).toMatch(/overflow-y:\s*scroll/);
    expect(catalogueRule).toMatch(/overscroll-behavior:\s*contain/);
  });

  it("closes the authorization disclosure and preserves Observed state when leaving the workspace", async () => {
    const { view, user, ds } = await loadedView();
    await runObservation(user);
    expect(await view.findByText("Observation 1")).toBeVisible();

    // Open the disclosure again; the model is still selected from the previous run.
    await user.click(screen.getByRole("button", { name: "Run observation" }));
    expect(screen.queryByRole("dialog")).not.toBeNull();

    // Switch away while the disclosure is open.
    await activate(user, "Models");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByRole("tab", { name: "Models" })).toHaveFocus();

    // Returning preserves the selected model, current result, and history.
    await activate(user, "Observed inference");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByRole("combobox", { name: "Model" })).toHaveValue(
      `ollama:${fixtureModelInventory().models[0]!.display_name}`,
    );
    expect(view.getByText("Observation 1")).toBeVisible();

    // Navigation caused zero inference and zero additional data acquisition.
    expect(ds.runInferenceObservation).toHaveBeenCalledTimes(1);
    expect(ds.currentSnapshot).toHaveBeenCalledTimes(1);
    expect(ds.modelInventory).toHaveBeenCalledTimes(1);
  });
});
