import { describe, expect, it, vi } from "vitest";
import { render, within } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { axe } from "vitest-axe";
import RecentObservations from "./RecentObservations.svelte";
import {
  COMPARISON_LIMITATION,
  GENERATED_TOKEN_LIMITATION,
  LOAD_DURATION_LIMITATION,
  PROFILE_MISMATCH,
} from "../observationHistory";
import { fixtureInferenceObservation } from "../fixtures/states";
import type { ObservationHistoryItem } from "../observationHistory";
import type { InferenceObservationView } from "../types";

function item(
  id: number,
  overrides: Partial<InferenceObservationView> = {},
): ObservationHistoryItem {
  return {
    observation_id: id,
    observation: { ...fixtureInferenceObservation(), ...overrides },
  };
}

describe("RecentObservations", () => {
  it("renders the empty session state without actions or inference controls", () => {
    const { getByRole, getByText, queryByRole } = render(RecentObservations, {
      props: { history: [], selectedObservationIds: [], onClear: vi.fn() },
    });
    expect(getByRole("heading", { name: "Recent observations" })).toBeVisible();
    expect(
      getByText(
        "Completed observations from this app session will appear here.",
      ),
    ).toBeVisible();
    expect(getByText(COMPARISON_LIMITATION)).toBeVisible();
    expect(
      queryByRole("button", { name: "Clear recent observations" }),
    ).toBeNull();
    expect(queryByRole("button", { name: /run again/i })).toBeNull();
  });

  it("renders compact rows newest first with source labels and undetermined execution", () => {
    const history = [
      item(3, { model: "artificial-new:3b", eval_count: null }),
      item(2, { model: "artificial-old:1b" }),
    ];
    const { getAllByRole, getByText, getAllByText } = render(
      RecentObservations,
      {
        props: { history, selectedObservationIds: [], onClear: vi.fn() },
      },
    );
    const rows = getAllByRole("listitem");
    expect(rows).toHaveLength(2);
    expect(rows[0]).toHaveTextContent("Observation 3");
    expect(rows[0]).toHaveTextContent("artificial-new:3b");
    expect(rows[1]).toHaveTextContent("Observation 2");
    expect(getAllByText("Ollama-reported generation duration")).toHaveLength(2);
    expect(
      getAllByText("Observed generation rate (Engine Room calculated)"),
    ).toHaveLength(2);
    expect(getAllByText("Execution location not determined")).toHaveLength(2);
    expect(getByText("—")).toHaveAccessibleName("Not reported");
  });

  it("selects two same-profile observations and renders unchanged raw values", async () => {
    const user = userEvent.setup();
    const history = [
      item(2, {
        model: "artificial-b:2b",
        eval_count: 7,
        generation_tokens_per_second: 7.25,
      }),
      item(1, {
        model: "artificial-a:1b",
        eval_count: 5,
        generation_tokens_per_second: 5.5,
      }),
    ];
    const { getByRole, getAllByRole, getByText, getAllByText } = render(
      RecentObservations,
      { props: { history, selectedObservationIds: [], onClear: vi.fn() } },
    );
    const checkboxes = getAllByRole("checkbox");
    await user.click(checkboxes[0]!);
    await user.click(checkboxes[1]!);

    const section = getByRole("heading", {
      name: "Compare observations",
    }).parentElement!;
    const table = within(section).getByRole("table");
    expect(table).toHaveTextContent("artificial-a:1b");
    expect(table).toHaveTextContent("artificial-b:2b");
    expect(table).toHaveTextContent("5.50 tokens/s");
    expect(table).toHaveTextContent("7.25 tokens/s");
    expect(getAllByText(COMPARISON_LIMITATION)).toHaveLength(2);
    expect(getByText(LOAD_DURATION_LIMITATION)).toBeVisible();
    expect(getByText(GENERATED_TOKEN_LIMITATION)).toBeVisible();
    expect(
      getAllByText("Execution location not determined").length,
    ).toBeGreaterThan(1);
    expect(checkboxes[0]).toBeChecked();
    expect(checkboxes[1]).toBeChecked();
    expect(
      getByText("Two observations selected. Comparison is available."),
    ).toHaveAttribute("aria-live", "polite");
    expect(
      getByText(
        "Two observations are selected. Deselect one before selecting another.",
      ),
    ).toBeVisible();
    expect(checkboxes[0]).toHaveAccessibleDescription(
      "Two observations are selected. Deselect one before selecting another.",
    );
  });

  it("keeps mismatched selections but suppresses the table with exact wording", async () => {
    const user = userEvent.setup();
    const history = [
      item(2, { diagnostic_profile: "diagnostic-other-v1" }),
      item(1),
    ];
    const { getAllByRole, getByText, queryByRole } = render(
      RecentObservations,
      { props: { history, selectedObservationIds: [], onClear: vi.fn() } },
    );
    const checkboxes = getAllByRole("checkbox");
    await user.click(checkboxes[0]!);
    await user.click(checkboxes[1]!);
    expect(checkboxes[0]).toBeChecked();
    expect(checkboxes[1]).toBeChecked();
    expect(getByText(PROFILE_MISMATCH)).toBeVisible();
    expect(queryByRole("table")).toBeNull();
    expect(
      getByText(
        "Two observations selected. Direct comparison is unavailable because their diagnostic profiles differ or are unknown.",
      ),
    ).toHaveAttribute("aria-live", "polite");
  });

  it("limits selection to two without running or exposing a rerun action", async () => {
    const user = userEvent.setup();
    const { getAllByRole, queryByRole } = render(RecentObservations, {
      props: {
        history: [item(3), item(2), item(1)],
        selectedObservationIds: [],
        onClear: vi.fn(),
      },
    });
    const checkboxes = getAllByRole("checkbox");
    await user.click(checkboxes[0]!);
    await user.click(checkboxes[1]!);
    expect(checkboxes[2]).toBeDisabled();
    expect(queryByRole("button", { name: /run again/i })).toBeNull();
  });

  it("allows one of two selections to be replaced and derives the replacement comparison", async () => {
    const user = userEvent.setup();
    const history = [
      item(3, { diagnostic_profile: "replacement-profile" }),
      item(2),
      item(1),
    ];
    const { getAllByRole, getByText, queryByRole } = render(
      RecentObservations,
      {
        props: { history, selectedObservationIds: [], onClear: vi.fn() },
      },
    );
    const checkboxes = getAllByRole("checkbox");

    await user.click(checkboxes[1]!);
    await user.click(checkboxes[2]!);
    expect(checkboxes[0]).toBeDisabled();

    await user.click(checkboxes[2]!);
    expect(checkboxes[0]).toBeEnabled();
    await user.click(checkboxes[0]!);

    expect(checkboxes[0]).toBeChecked();
    expect(checkboxes[1]).toBeChecked();
    expect(checkboxes[2]).not.toBeChecked();
    expect(getByText(PROFILE_MISMATCH)).toBeVisible();
    expect(queryByRole("table")).toBeNull();
  });

  it("announces a retained observation without datasource or inference activity", () => {
    const onClear = vi.fn();
    const { getByText } = render(RecentObservations, {
      props: {
        history: [item(4)],
        selectedObservationIds: [],
        retainedAnnouncement: "Observation 4 added to recent observations.",
        onClear,
      },
    });

    expect(
      getByText("Observation 4 added to recent observations."),
    ).toHaveAttribute("aria-live", "polite");
    expect(onClear).not.toHaveBeenCalled();
  });

  it("clear emits only the local clear callback", async () => {
    const user = userEvent.setup();
    const onClear = vi.fn();
    const { getByRole } = render(RecentObservations, {
      props: {
        history: [item(1)],
        selectedObservationIds: [],
        onClear,
      },
    });
    await user.click(
      getByRole("button", { name: "Clear recent observations" }),
    );
    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it("contains no chart, ranking, delta, generated output, or prompt UI", async () => {
    const { container, queryByRole } = render(RecentObservations, {
      props: {
        history: [item(2), item(1)],
        selectedObservationIds: [2, 1],
        onClear: vi.fn(),
      },
    });
    const text = container.textContent ?? "";
    expect(container.querySelector("svg, canvas")).toBeNull();
    expect(queryByRole("button", { name: /run/i })).toBeNull();
    expect(text).not.toMatch(
      /\b(fastest|better|winner|best|improved|regressed)\b/i,
    );
    expect(text).not.toMatch(/\b(delta|average|median|percentile|score)\b/i);
    expect(text).not.toMatch(/\d+\s*%/);
    expect(text).not.toMatch(/leaves appear green/i);
    expect(text).not.toContain("SECRET-GENERATED-OUTPUT");
  });

  it.each([
    { name: "empty", history: [], selected: [] },
    { name: "populated", history: [item(1)], selected: [] },
    { name: "selected", history: [item(2), item(1)], selected: [2, 1] },
    {
      name: "mismatch",
      history: [item(2, { diagnostic_profile: "other" }), item(1)],
      selected: [2, 1],
    },
  ])(
    "has no automated axe violations in $name state",
    async ({ history, selected }) => {
      const { container } = render(RecentObservations, {
        props: {
          history,
          selectedObservationIds: selected,
          onClear: vi.fn(),
        },
      });
      const results = await axe(container, {
        rules: { "color-contrast": { enabled: false } },
      });
      expect(results.violations).toEqual([]);
    },
  );
});
