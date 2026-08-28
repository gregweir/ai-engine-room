import { describe, expect, it, vi } from "vitest";
import { render, within } from "@testing-library/svelte";
import { axe } from "vitest-axe";
import userEvent from "@testing-library/user-event";
import ObservedInference from "./ObservedInference.svelte";
const ollamaValue = (model: string) => `ollama:${model}`;
import type { DataSource } from "../datasource";
import type { InferenceObservationView, ModelInventoryView } from "../types";
import { MockDataSource } from "../datasource/mock";
import {
  DISCLOSURE_BODY,
  DISCLOSURE_PRIMARY_ACTION,
  DISCLOSURE_SECONDARY_ACTION,
  DISCLOSURE_TITLE,
  EXECUTION_LOCATION_NOT_DETERMINED,
  GENERATION_RATE_EXPLANATION,
  GENERATION_RATE_LABEL,
  fixtureEmptyModelInventory,
  fixtureInferenceObservation,
  fixtureInferenceObservations,
  fixtureModelInventory,
} from "../fixtures/states";

// A minimal DataSource whose only behaviour the component exercises is
// `runInferenceObservation`. The other methods are stubs so the object satisfies
// the `DataSource` interface; they are never called by this component.
function makeDataSource(
  observation: InferenceObservationView = fixtureInferenceObservation(),
): DataSource {
  return {
    isMock: true,
    currentSnapshot: vi.fn(async () => {
      throw new Error("not used");
    }),
    machineContext: vi.fn(async () => {
      throw new Error("not used");
    }),
    reportPreview: vi.fn(async () => ""),
    runtimeStatus: vi.fn(async () => {
      throw new Error("not used");
    }),
    modelInventory: vi.fn(async () => {
      throw new Error("not used");
    }),
    loadedModels: vi.fn(async () => {
      throw new Error("not used");
    }),
    composeResourceContext: vi.fn(async () => {
      throw new Error("not used");
    }),
    diagnoseObservation: vi.fn(async () => ({
      state: "available" as const,
      findings: [],
    })),
    runInferenceObservation: vi.fn(async () => observation),
  };
}

// An unavailable catalogue (not the `available` state) yields no selector.
function unavailableInventory(): ModelInventoryView {
  return {
    ...fixtureModelInventory(),
    state: "not_detected",
    source_availability: "not_detected",
    models: [],
    state_label: "Ollama was not detected at the local endpoint",
    interpretation: "Nothing responded at the local Ollama endpoint.",
    why_it_matters:
      "No service answered the local endpoint. This only shows that nothing answered there right now.",
  };
}

describe("ObservedInference", () => {
  it("labels LM Studio models, excludes embeddings, and discloses JIT before one request", async () => {
    const user = userEvent.setup();
    const mock = new MockDataSource();
    const lmStudio = await mock.lmStudioSnapshot();
    const run = vi.spyOn(mock, "runInferenceObservation");
    const view = render(ObservedInference, {
      props: {
        inventory: { ...fixtureEmptyModelInventory(), state: "available" },
        lmStudio,
        dataSource: mock,
      },
    });
    const select = view.getByRole("combobox", { name: "Model" });
    expect(select).toHaveTextContent("LM Studio — Artificial Leaf Model");
    expect(select).not.toHaveTextContent("Artificial Embedding");
    await user.selectOptions(select, "lm_studio:artificial/leaf-model");
    expect(run).not.toHaveBeenCalled();
    await user.click(view.getByRole("button", { name: "Run observation" }));
    const dialog = await view.findByRole("dialog");
    expect(dialog).toHaveTextContent("JIT-load");
    expect(run).not.toHaveBeenCalled();
    await user.click(
      within(dialog).getByRole("button", { name: "Run observation" }),
    );
    await vi.waitFor(() => expect(run).toHaveBeenCalledTimes(1));
    expect(run).toHaveBeenCalledWith({
      provider: "lm_studio",
      model: "artificial/leaf-model",
      execution_boundary_acknowledged: true,
      provider_behavior_acknowledged: true,
    });
  });
  it("renders the section heading and an introductory note", () => {
    const { getByRole, getByText } = render(ObservedInference, {
      props: {
        inventory: fixtureModelInventory(),
        dataSource: makeDataSource(),
      },
    });
    expect(getByRole("heading", { name: "Observed inference" })).toBeVisible();
    expect(getByText(/Run one fixed diagnostic observation/)).toBeVisible();
    expect(
      getByText(
        "llama.cpp is passive-only in this phase and is not available for observed inference.",
      ),
    ).toBeVisible();
  });

  it("populates the model selector from the available catalogue only", () => {
    const inventory = fixtureModelInventory();
    const { getByRole, getAllByRole } = render(ObservedInference, {
      props: { inventory, dataSource: makeDataSource() },
    });
    const select = getByRole("combobox", {
      name: "Model",
    }) as HTMLSelectElement;
    // One disabled placeholder option plus one per catalogue model.
    const options = getAllByRole("option");
    expect(options.length).toBe(inventory.models.length + 1);
    for (const model of inventory.models) {
      expect(
        Array.from(select.options).some(
          (o) => o.value === ollamaValue(model.display_name),
        ),
      ).toBe(true);
    }
  });

  it("shows no selector when the catalogue is not available", () => {
    const { queryByRole, getByText } = render(ObservedInference, {
      props: {
        inventory: unavailableInventory(),
        dataSource: makeDataSource(),
      },
    });
    expect(queryByRole("combobox")).toBeNull();
    expect(
      getByText(/model catalogue is not available right now/),
    ).toBeVisible();
  });

  it("shows no selector for a valid empty available catalogue", () => {
    const { queryByRole, getByText } = render(ObservedInference, {
      props: {
        inventory: fixtureEmptyModelInventory(),
        dataSource: makeDataSource(),
      },
    });
    expect(queryByRole("combobox")).toBeNull();
    expect(
      getByText(/model catalogue is not available right now/),
    ).toBeVisible();
  });

  it("disables the Run observation button until a model is selected", () => {
    const { getByRole } = render(ObservedInference, {
      props: {
        inventory: fixtureModelInventory(),
        dataSource: makeDataSource(),
      },
    });
    const run = getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION });
    expect(run).toBeDisabled();
  });

  it("enables the Run observation button after a model is selected", async () => {
    const user = userEvent.setup();
    const inventory = fixtureModelInventory();
    const { getByRole } = render(ObservedInference, {
      props: { inventory, dataSource: makeDataSource() },
    });
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    expect(
      getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    ).toBeEnabled();
  });

  it("opens the disclosure dialog on Run click without running yet", async () => {
    const user = userEvent.setup();
    const ds = makeDataSource();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole } = render(ObservedInference, {
      props: { inventory, dataSource: ds },
    });
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));

    const dialog = await findByRole("dialog");
    expect(dialog).toBeVisible();
    // Opening the disclosure must NOT issue an observation.
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("renders the disclosure copy verbatim with no Don't ask again", async () => {
    const user = userEvent.setup();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole } = render(ObservedInference, {
      props: { inventory, dataSource: makeDataSource() },
    });
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));

    const dialog = await findByRole("dialog");
    const scoped = within(dialog);
    expect(scoped.getByText(DISCLOSURE_TITLE)).toBeVisible();
    for (const paragraph of DISCLOSURE_BODY) {
      expect(scoped.getByText(paragraph)).toBeVisible();
    }
    expect(
      scoped.getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    ).toBeVisible();
    expect(
      scoped.getByRole("button", { name: DISCLOSURE_SECONDARY_ACTION }),
    ).toBeVisible();
    // There is intentionally no "Don't ask again" action or text.
    expect(dialog.textContent ?? "").not.toMatch(/don't ask again/i);
    expect(dialog.textContent ?? "").not.toMatch(/remember/i);
  });

  it("cancel closes the disclosure without running", async () => {
    const user = userEvent.setup();
    const ds = makeDataSource();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, queryByRole } = render(ObservedInference, {
      props: { inventory, dataSource: ds },
    });
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_SECONDARY_ACTION }),
    );
    expect(queryByRole("dialog")).toBeNull();
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("moves focus into the disclosure, contains it, closes on Escape, and restores Run focus", async () => {
    const user = userEvent.setup();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, queryByRole } = render(ObservedInference, {
      props: { inventory, dataSource: makeDataSource() },
    });
    await user.selectOptions(
      getByRole("combobox", { name: "Model" }),
      ollamaValue(inventory.models[0]!.display_name),
    );
    const run = getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION });
    await user.click(run);
    const dialog = await findByRole("dialog");
    const cancel = within(dialog).getByRole("button", {
      name: DISCLOSURE_SECONDARY_ACTION,
    });
    await vi.waitFor(() => expect(cancel).toHaveFocus());
    await user.keyboard("{Shift>}{Tab}{/Shift}");
    expect(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    ).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(queryByRole("dialog")).toBeNull();
    await vi.waitFor(() => expect(run).toHaveFocus());
  });

  it("authorizing runs one observation with acknowledged execution boundary", async () => {
    const user = userEvent.setup();
    const ds = makeDataSource();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole } = render(ObservedInference, {
      props: { inventory, dataSource: ds },
    });
    const select = getByRole("combobox", { name: "Model" });
    const chosen = inventory.models[0]!.display_name;
    await user.selectOptions(select, ollamaValue(chosen));
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );

    expect(ds.runInferenceObservation).toHaveBeenCalledTimes(1);
    expect(ds.runInferenceObservation).toHaveBeenCalledWith({
      provider: "ollama",
      model: chosen,
      execution_boundary_acknowledged: true,
      provider_behavior_acknowledged: false,
    });
  });

  it("emits exactly one valid completed observation and no controlled failure", async () => {
    const user = userEvent.setup();
    const completed = vi.fn();
    const inventory = fixtureModelInventory();
    const ds = makeDataSource(fixtureInferenceObservation());
    const { getByRole, findByRole, unmount } = render(ObservedInference, {
      props: {
        inventory,
        dataSource: ds,
        onCompletedObservation: completed,
      },
    });
    await user.selectOptions(
      getByRole("combobox", { name: "Model" }),
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    await user.click(
      within(await findByRole("dialog")).getByRole("button", {
        name: DISCLOSURE_PRIMARY_ACTION,
      }),
    );
    await vi.waitFor(() => expect(completed).toHaveBeenCalledTimes(1));
    expect(completed).toHaveBeenCalledWith(fixtureInferenceObservation());
    unmount();

    const failed = vi.fn();
    const timedOut = fixtureInferenceObservations().find(
      (candidate) => candidate.state === "timed_out",
    )!;
    const failureDataSource = makeDataSource(timedOut);
    const failedRender = render(ObservedInference, {
      props: {
        inventory,
        dataSource: failureDataSource,
        onCompletedObservation: failed,
      },
    });
    await user.selectOptions(
      failedRender.getByRole("combobox", { name: "Model" }),
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(
      failedRender.getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );
    await user.click(
      within(await failedRender.findByRole("dialog")).getByRole("button", {
        name: DISCLOSURE_PRIMARY_ACTION,
      }),
    );
    await vi.waitFor(() =>
      expect(failureDataSource.runInferenceObservation).toHaveBeenCalledTimes(
        1,
      ),
    );
    expect(failed).not.toHaveBeenCalled();
  });

  it("disables controls while an observation is running", async () => {
    const user = userEvent.setup();
    let resolveRun: (value: InferenceObservationView) => void = () => {};
    const pending = new Promise<InferenceObservationView>((resolve) => {
      resolveRun = resolve;
    });
    const ds = makeDataSource();
    ds.runInferenceObservation = vi.fn(() => pending);
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, findByText } = render(ObservedInference, {
      props: { inventory, dataSource: ds },
    });
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );

    // While running: the selector and trigger are disabled and a status shows.
    expect(await findByText("Running one observation…")).toBeVisible();
    expect(getByRole("combobox", { name: "Model" })).toBeDisabled();
    expect(
      getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    ).toBeDisabled();

    resolveRun(fixtureInferenceObservation());
    await new Promise((r) => setTimeout(r, 0));
  });

  it("renders the completed result with counts, durations as seconds, rate, and undetermined boundary", async () => {
    const user = userEvent.setup();
    const ds = makeDataSource(fixtureInferenceObservation());
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, getByText, container } = render(
      ObservedInference,
      { props: { inventory, dataSource: ds } },
    );
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );
    await new Promise((r) => setTimeout(r, 0));

    // Ollama-reported counts render verbatim.
    expect(getByText("12")).toBeVisible();
    expect(getByText("8")).toBeVisible();
    // Durations render as seconds (ns / 1_000_000_000), not raw nanoseconds.
    expect(getByText("0.500 s")).toBeVisible();
    expect(getByText("0.300 s")).toBeVisible();
    expect(getByText("1.000 s")).toBeVisible();
    expect(getByText("1.800 s")).toBeVisible();
    // Raw nanosecond figures must not appear anywhere in the result.
    expect(container.textContent ?? "").not.toContain("500000000");
    expect(container.textContent ?? "").not.toContain("1000000000");
    // The Engine-Room-calculated rate label, value, and explanation render.
    expect(getByText(GENERATION_RATE_LABEL)).toBeVisible();
    expect(getByText("8.00 tokens/s")).toBeVisible();
    expect(getByText(GENERATION_RATE_EXPLANATION)).toBeVisible();
    // The execution boundary is always "undetermined".
    expect(getByText("undetermined")).toBeVisible();
    // Controlled interpretation/why_it_matters/limitations render verbatim.
    expect(
      getByText(
        "One observed inference completed. The counts and durations are values Ollama reported for this run.",
      ),
    ).toBeVisible();
    expect(
      getByText(
        "This is a single observed run, not a general benchmark or model-quality score.",
      ),
    ).toBeVisible();
    expect(getByText(EXECUTION_LOCATION_NOT_DETERMINED)).toBeVisible();
  });

  it("never displays the generated response text or echoes the fixed prompt", async () => {
    const user = userEvent.setup();
    // A completed fixture whose underlying backend response was discarded; the
    // view DTO carries no `response` field and no generated text.
    const ds = makeDataSource(fixtureInferenceObservation());
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, container } = render(ObservedInference, {
      props: { inventory, dataSource: ds },
    });
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );
    await new Promise((r) => setTimeout(r, 0));

    const text = container.textContent ?? "";
    // The fixed diagnostic prompt is never echoed.
    expect(text).not.toMatch(/leaves appear green/i);
    // No generated-response field is rendered.
    expect(text).not.toMatch(/generated response/i);
    expect(text).not.toContain("SECRET");
  });

  it("renders controlled text for each non-completed result state", async () => {
    const user = userEvent.setup();
    const inventory = fixtureModelInventory();
    const nonCompleted = fixtureInferenceObservations().filter(
      (o) => o.state !== "completed" && o.state !== "no_model_selected",
    );
    for (const observation of nonCompleted) {
      const ds = makeDataSource(observation);
      const { getByRole, findByRole, getByText, queryByText, unmount } = render(
        ObservedInference,
        { props: { inventory, dataSource: ds } },
      );
      const select = getByRole("combobox", { name: "Model" });
      await user.selectOptions(
        select,
        ollamaValue(inventory.models[0]!.display_name),
      );
      await user.click(
        getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
      );
      const dialog = await findByRole("dialog");
      await user.click(
        within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
      );
      await new Promise((r) => setTimeout(r, 0));

      expect(getByText(observation.interpretation)).toBeVisible();
      expect(getByText(observation.why_it_matters)).toBeVisible();
      expect(getByText(observation.limitations)).toBeVisible();
      // The execution boundary is always "undetermined" for every state.
      expect(getByText("undetermined")).toBeVisible();
      // Non-completed states carry no metric rows.
      expect(queryByText("Prompt tokens")).toBeNull();
      expect(queryByText("Generated tokens")).toBeNull();
      expect(queryByText(GENERATION_RATE_LABEL)).toBeNull();
      unmount();
    }
  });

  it("does not run an observation on mount", () => {
    const ds = makeDataSource();
    render(ObservedInference, {
      props: { inventory: fixtureModelInventory(), dataSource: ds },
    });
    expect(ds.runInferenceObservation).not.toHaveBeenCalled();
  });

  it("never renders raw transport details, endpoints, or status codes", async () => {
    const user = userEvent.setup();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, container, unmount } = render(
      ObservedInference,
      { props: { inventory, dataSource: makeDataSource() } },
    );
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );
    await new Promise((r) => setTimeout(r, 0));
    const text = (container.textContent ?? "").toLowerCase();
    expect(text).not.toContain("127.0.0.1");
    expect(text).not.toContain("11434");
    expect(text).not.toContain("http");
    expect(text).not.toContain("/api/");
    expect(text).not.toContain("refus");
    unmount();
  });

  it("never claims the model ran locally or was verified", async () => {
    const user = userEvent.setup();
    const inventory = fixtureModelInventory();
    const { getByRole, findByRole, container, unmount } = render(
      ObservedInference,
      { props: { inventory, dataSource: makeDataSource() } },
    );
    const select = getByRole("combobox", { name: "Model" });
    await user.selectOptions(
      select,
      ollamaValue(inventory.models[0]!.display_name),
    );
    await user.click(getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }));
    const dialog = await findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: DISCLOSURE_PRIMARY_ACTION }),
    );
    await new Promise((r) => setTimeout(r, 0));
    const text = container.textContent ?? "";
    // No locality claim: the model is never asserted to have run on this
    // computer, and no "verified local" language appears. ("undetermined" is
    // the honest execution boundary and is allowed.)
    expect(text).not.toMatch(/\blocal\b/i);
    expect(text).not.toMatch(/locally/i);
    expect(text).not.toMatch(/verified/i);
    expect(text).not.toMatch(/installed/i);
    unmount();
  });

  it("has no automated axe violations (colour-contrast disabled)", async () => {
    const { container } = render(ObservedInference, {
      props: {
        inventory: fixtureModelInventory(),
        dataSource: makeDataSource(),
      },
    });
    await new Promise((r) => setTimeout(r, 0));
    const results = await axe(container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});
