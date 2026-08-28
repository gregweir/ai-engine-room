/// <reference types="node" />
import { describe, expect, it } from "vitest";
import { axe } from "vitest-axe";
import { render } from "@testing-library/svelte";
import { readFileSync } from "node:fs";
import DiagnoseWorkspace from "./DiagnoseWorkspace.svelte";
import type { DiagnosticObservationBundle } from "../diagnosticObservation";
import type { DiagnosticFindingView } from "../types";

const longModelId =
  "provider-qualified/artificial-model-with-a-deliberately-long-identity:latest";

function observation(
  observation_id = 1,
  gap = false,
): DiagnosticObservationBundle {
  return {
    observation_id,
    available_memory: gap
      ? { kind: "unavailable", reason: "snapshot_rejected" }
      : {
          kind: "value",
          bytes: 0,
          provenance: "operating_system_reported",
        },
    ollama_runtime: gap
      ? { kind: "gap" }
      : { kind: "observed", value: { state: "ready", version: "1.0" } },
    ollama_catalogue: {
      kind: "observed",
      value: { state: "available", model_ids: [longModelId] },
    },
    ollama_loaded: {
      kind: "observed",
      value: { state: "available", model_ids: [] },
    },
    lm_studio: {
      kind: "observed",
      value: { state: "available", models: [] },
    },
    llama_cpp: {
      kind: "observed",
      value: { state: "ready", served_model_id: "artificial-served" },
    },
  };
}

const findings: DiagnosticFindingView[] = [
  {
    code: "ollama_catalogued_not_loaded",
    source: "ollama",
    observation: `${longModelId} is catalogued but was not reported loaded.`,
    meaning:
      "The exact same-provider identity was absent from a valid running-model observation.",
    safe_next_check:
      "Confirm loaded state in Ollama if that distinction matters.",
  },
];

describe("Diagnose workspace", () => {
  it("renders the fifth-workspace hierarchy, scope, coverage, changes, and controlled finding", () => {
    const view = render(DiagnoseWorkspace, {
      props: {
        history: [observation(1), observation(2)],
        changes: {
          state: "changes",
          changes: [
            {
              code: "ollama_catalogue_set_changed",
              source: "ollama_catalogue",
              summary: "Ollama catalogue changed from 0 to 1.",
              details: [`Added: ${longModelId}`],
            },
          ],
        },
        findings,
        diagnosisState: "available",
      },
    });
    expect(
      view.getByRole("heading", { name: "Diagnose", level: 2 }),
    ).toBeVisible();
    expect(view.getByText("Latest observation: 2")).toBeVisible();
    expect(view.getByText(/not continuous monitoring/)).toBeVisible();
    expect(view.getByText(/not an atomic machine snapshot/)).toBeVisible();
    expect(
      view.getByText("Ollama catalogue changed from 0 to 1."),
    ).toBeVisible();
    expect(view.getByText(`Added: ${longModelId}`)).toBeVisible();
    expect(view.getByText("Observation", { selector: "dt" })).toBeVisible();
    expect(view.getByText("Meaning", { selector: "dt" })).toBeVisible();
    expect(view.getByText("Safe next check", { selector: "dt" })).toBeVisible();
    expect(view.getAllByText(new RegExp(longModelId))).toHaveLength(2);
  });

  it("presents source gaps as unknown text and no supported change without numeric fabrication", () => {
    const view = render(DiagnoseWorkspace, {
      props: {
        history: [observation(1, true)],
        changes: { state: "no_supported_change", changes: [] },
        findings: [],
        diagnosisState: "available",
      },
    });
    expect(view.getAllByText("Source gap").length).toBeGreaterThan(0);
    expect(
      view.getByText(/unknown, not an empty model set or numeric zero/),
    ).toBeVisible();
    expect(view.getByText("No supported change was observed.")).toBeVisible();
    expect(view.queryByText("0 B")).toBeNull();
    expect(
      view.getByText(
        "No deterministic finding applies to the latest observation.",
      ),
    ).toBeVisible();
  });

  it("distinguishes first observation and controlled diagnosis failures", () => {
    const view = render(DiagnoseWorkspace, {
      props: {
        history: [observation()],
        changes: { state: "no_previous", changes: [] },
        findings: [],
        diagnosisState: "invalid_input",
      },
    });
    expect(
      view.getByText(/has no previous explicit observation/),
    ).toBeVisible();
    expect(
      view.getByText(/bounded diagnostic input could not be validated/),
    ).toBeVisible();
  });

  it("provides complete text without a graphical-only or animated diagnostic surface", async () => {
    const view = render(DiagnoseWorkspace, {
      props: {
        history: [observation()],
        changes: { state: "no_supported_change", changes: [] },
        findings,
        diagnosisState: "available",
      },
    });
    expect(
      view.container.querySelector("svg, canvas, meter, progress"),
    ).toBeNull();
    expect(
      view.getByText("Diagnostic observation 1", { selector: "summary" }),
    ).toBeVisible();
    expect(
      view.getByText(
        /Sequence numbers show acquisition order, not elapsed time/,
      ),
    ).toBeVisible();
    const results = await axe(view.container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });

  it("keeps prohibited mechanisms and diagnosis claims out of the component contract", () => {
    const source = readFileSync(
      "src/lib/components/DiagnoseWorkspace.svelte",
      "utf8",
    );
    expect(source).not.toMatch(
      /setInterval|setTimeout|localStorage|sessionStorage|indexedDB/,
    );
    expect(source).not.toMatch(/<svg|<canvas|<meter|<progress/);
    expect(source).not.toMatch(
      /health score|severity score|fit score|trend arrow/i,
    );
    expect(source).not.toMatch(/onclick=|onkeydown=/);
  });
});
