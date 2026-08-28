import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { axe } from "vitest-axe";
import ModelInventoryCard from "./ModelInventoryCard.svelte";
import type { ModelInventoryView, RuntimeModelSummaryView } from "../types";
import {
  fixtureEmptyModelInventory,
  fixtureModelInventories,
  fixtureModelInventory,
} from "../fixtures/states";

function entry(
  overrides: Partial<RuntimeModelSummaryView>,
): RuntimeModelSummaryView {
  return {
    display_name: "example-chat:8b",
    locality: "undetermined",
    size_bytes: 4_000_000_000,
    parameter_size: "8.0B",
    quantization: "Q4_0",
    families: ["llama"],
    ...overrides,
  };
}

function inventory(overrides: Partial<ModelInventoryView>): ModelInventoryView {
  return {
    runtime_type: "ollama",
    state: "available",
    source_availability: "ready",
    models: [entry({})],
    skipped: 0,
    state_label: "Ollama model catalogue",
    interpretation: "These models are listed by the Ollama runtime.",
    why_it_matters:
      "Each entry shows the model name and the size Ollama reports for it.",
    ...overrides,
  };
}

describe("ModelInventoryCard", () => {
  it("renders the runtime name and controlled state label verbatim", () => {
    const { getByText } = render(ModelInventoryCard, {
      props: { inventory: inventory({}) },
    });
    expect(getByText("Ollama")).toBeVisible();
    expect(getByText("Ollama model catalogue")).toBeVisible();
  });

  it("renders the controlled interpretation and why-it-matters verbatim", () => {
    const inv = inventory({});
    const { getByText } = render(ModelInventoryCard, {
      props: { inventory: inv },
    });
    expect(getByText(inv.interpretation)).toBeVisible();
    expect(getByText(inv.why_it_matters)).toBeVisible();
  });

  it("renders one row per reported model with the reported size formatted", async () => {
    const user = userEvent.setup();
    const inv = inventory({
      models: [
        entry({ display_name: "example-chat:8b", size_bytes: 4_000_000_000 }),
        entry({ display_name: "example-coder:3b", size_bytes: 2_000_000_000 }),
      ],
    });
    const { getByText, getAllByText } = render(ModelInventoryCard, {
      props: { inventory: inv },
    });
    expect(getByText("example-chat:8b")).toBeVisible();
    expect(getByText("example-coder:3b")).toBeVisible();
    await user.click(getByText("example-chat:8b"));
    // The reported size is formatted via formatBytes (binary/decimal units).
    expect(getByText("4.00 GB")).toBeVisible();
    // "Reported size" is the controlled label, not "disk usage" or "local storage".
    expect(getAllByText("Reported size").length).toBeGreaterThan(0);
  });

  it("keeps the API-ordered catalogue in a named, bounded results region below the filter and count", () => {
    const names = [
      "first-model:1b",
      "second-model:2b",
      "publisher/very-long-model-name-that-must-remain-complete-and-wrappable:latest",
    ];
    const { container, getByRole, getByText } = render(ModelInventoryCard, {
      props: {
        inventory: inventory({
          models: names.map((display_name) => entry({ display_name })),
        }),
      },
    });
    const filter = getByRole("searchbox", {
      name: "Filter catalogue by model name",
    });
    const count = getByText("3 of 3 catalogue models shown");
    const results = getByRole("region", { name: "Catalogue model results" });
    const list = getByRole("list");
    const summaries = Array.from(results.querySelectorAll("summary"));

    expect(results).toHaveClass("catalogue-results");
    expect(results).toHaveAttribute("tabindex", "0");
    expect(results).not.toContainElement(filter);
    expect(results).not.toContainElement(count);
    expect(results).toContainElement(list);
    expect(
      filter.compareDocumentPosition(results) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      count.compareDocumentPosition(results) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(results.querySelectorAll("li.model-entry")).toHaveLength(
      names.length,
    );
    for (const summary of summaries) {
      expect(results).toContainElement(summary.closest("li"));
    }
    expect(summaries.map((summary) => summary.textContent)).toEqual(names);
    expect(summaries[2]).toHaveAccessibleName(names[2]);
    expect(container.querySelector(".model-list")).not.toHaveClass("grid");
  });

  it("keeps native disclosures and approved metadata usable inside the results region", async () => {
    const user = userEvent.setup();
    const { getByRole, getByText } = render(ModelInventoryCard, {
      props: { inventory: inventory({}) },
    });
    const results = getByRole("region", { name: "Catalogue model results" });
    const summary = getByText("example-chat:8b", { selector: "summary" });
    const disclosure = summary.closest("details")!;

    expect(results).toContainElement(disclosure);
    expect(disclosure).not.toHaveAttribute("open");
    await user.click(summary);
    expect(disclosure).toHaveAttribute("open");
    expect(getByText("Reported size")).toBeVisible();
    expect(getByText("Parameter size")).toBeVisible();
    expect(getByText("Quantization")).toBeVisible();
    expect(getByText("Family")).toBeVisible();
    await user.click(summary);
    expect(disclosure).not.toHaveAttribute("open");
  });

  it("filters within the results region and keeps no-match feedback there", async () => {
    const user = userEvent.setup();
    const { getByRole, getByText, queryByText } = render(ModelInventoryCard, {
      props: {
        inventory: inventory({
          models: [
            entry({ display_name: "first:1b" }),
            entry({ display_name: "second:2b" }),
          ],
        }),
      },
    });
    const results = getByRole("region", { name: "Catalogue model results" });
    const filter = getByRole("searchbox", {
      name: "Filter catalogue by model name",
    });

    await user.type(filter, "SECOND");
    expect(results).toHaveTextContent("second:2b");
    expect(queryByText("first:1b")).toBeNull();
    expect(getByText("1 of 2 catalogue models shown")).toBeVisible();
    await user.clear(filter);
    await user.type(filter, "missing");
    expect(results).toContainElement(
      getByText("No catalogue model names match this filter."),
    );
    expect(getByText("0 of 2 catalogue models shown")).toBeVisible();
  });

  it("introduces no catalogue actions, pagination, sorting, or grid semantics", () => {
    const { container, queryByRole, queryByText } = render(ModelInventoryCard, {
      props: { inventory: inventory({}) },
    });

    expect(container.querySelectorAll("button, a")).toHaveLength(0);
    expect(queryByRole("grid")).toBeNull();
    expect(queryByRole("columnheader")).toBeNull();
    expect(queryByText(/\b(next|previous|sort|page)\b/i)).toBeNull();
  });

  it("omits the size when the runtime did not report one (null, not zero)", () => {
    const inv = inventory({
      models: [
        entry({
          display_name: "bare:1b",
          size_bytes: null,
          parameter_size: null,
          quantization: null,
          families: [],
        }),
      ],
    });
    const { getByText, queryByText } = render(ModelInventoryCard, {
      props: { inventory: inv },
    });
    expect(getByText("bare:1b")).toBeVisible();
    // No reported-size label and no "0 B" placeholder for a missing size.
    expect(queryByText("Reported size")).toBeNull();
    expect(queryByText("Parameter size")).toBeNull();
    expect(queryByText(/0 B/)).toBeNull();
  });

  it("renders the empty-state message for a valid empty catalogue (not an error)", () => {
    const empty = fixtureEmptyModelInventory();
    const { getByText, queryByRole } = render(ModelInventoryCard, {
      props: { inventory: empty },
    });
    expect(getByText("No models in the catalogue")).toBeVisible();
    // A valid empty catalogue shows no model rows.
    expect(queryByRole("list")).toBeNull();
  });

  it("renders each inventory state from the fixtures with backend wording", () => {
    for (const inv of fixtureModelInventories()) {
      const { getByText, unmount } = render(ModelInventoryCard, {
        props: { inventory: inv },
      });
      expect(getByText(inv.state_label)).toBeVisible();
      expect(getByText(inv.interpretation)).toBeVisible();
      expect(getByText(inv.why_it_matters)).toBeVisible();
      unmount();
    }
  });

  it("renders no model-action controls (no delete/pull/launch)", () => {
    const { container } = render(ModelInventoryCard, {
      props: { inventory: inventory({}) },
    });
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(0);
    // No link-based or aria-labeled actions either.
    expect(container.textContent ?? "").not.toMatch(
      /\b(delete|pull|launch|install|remove|generate)\b/i,
    );
  });

  it("does not render a per-model locality badge", () => {
    const { container } = render(ModelInventoryCard, {
      props: { inventory: fixtureModelInventory() },
    });
    const text = (container.textContent ?? "").toLowerCase();
    // Locality is internal/deferred in 1D; no per-row locality label is shown.
    expect(text).not.toContain("locality");
    expect(text).not.toContain("undetermined");
    expect(text).not.toContain("local model");
  });

  it("never renders raw transport details or endpoint URLs in any state", () => {
    for (const inv of fixtureModelInventories()) {
      const { container, unmount } = render(ModelInventoryCard, {
        props: { inventory: inv },
      });
      const text = (container.textContent ?? "").toLowerCase();
      expect(text).not.toContain("127.0.0.1");
      expect(text).not.toContain("11434");
      expect(text).not.toContain("http");
      expect(text).not.toContain("refus");
      expect(text).not.toContain("digest");
      expect(text).not.toContain("remote_host");
      unmount();
    }
  });

  it("never claims install state or health/compat/security/performance", () => {
    for (const inv of fixtureModelInventories()) {
      const { container, unmount } = render(ModelInventoryCard, {
        props: { inventory: inv },
      });
      const text = (container.textContent ?? "").toLowerCase();
      expect(text).not.toContain("not installed");
      expect(text).not.toContain("health");
      expect(text).not.toContain("compat");
      expect(text).not.toContain("secure");
      expect(text).not.toContain("fast");
      expect(text).not.toContain("perform");
      unmount();
    }
  });

  it("does not compute or display an aggregate size total", () => {
    const inv = inventory({
      models: [
        entry({ display_name: "a:1b", size_bytes: 4_000_000_000 }),
        entry({ display_name: "b:1b", size_bytes: 2_000_000_000 }),
      ],
    });
    const { container } = render(ModelInventoryCard, {
      props: { inventory: inv },
    });
    const text = container.textContent ?? "";
    // Per-model sizes are shown; no combined/total figure is computed.
    expect(text).not.toContain("6.00 GB");
    expect(text).not.toMatch(/total/i);
  });

  it("has no automated axe violations (colour-contrast disabled)", async () => {
    const { container } = render(ModelInventoryCard, {
      props: { inventory: inventory({}) },
    });
    await new Promise((r) => setTimeout(r, 0));
    const results = await axe(container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});
