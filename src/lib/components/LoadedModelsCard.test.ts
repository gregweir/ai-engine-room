import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import { axe } from "vitest-axe";
import LoadedModelsCard from "./LoadedModelsCard.svelte";
import type { LoadedModelSetView, LoadedModelSummaryView } from "../types";
import {
  fixtureEmptyLoadedModels,
  fixtureLoadedModels,
  fixtureLoadedModelsSets,
} from "../fixtures/states";

function entry(
  overrides: Partial<LoadedModelSummaryView>,
): LoadedModelSummaryView {
  return {
    display_name: "example-runner:8b",
    reported_size_bytes: 5_000_000_000,
    reported_vram_bytes: 4_000_000_000,
    reported_context_length: 8192,
    // Carried internally, NOT displayed.
    expires_at: "2024-01-01T00:00:00.000Z",
    ...overrides,
  };
}

function loaded(overrides: Partial<LoadedModelSetView>): LoadedModelSetView {
  return {
    runtime_type: "ollama",
    state: "available",
    source_availability: "ready",
    models: [entry({})],
    skipped: 0,
    state_label: "Currently loaded in Ollama",
    interpretation: "Ollama reports these models as currently loaded.",
    why_it_matters:
      "Each entry shows the model name and the loaded size Ollama reports for it.",
    resource_interpretation:
      "Loaded size is metadata reported by Ollama, not an exact model-weight, RAM, or disk allocation. VRAM size is also reported by Ollama and is not an independent measurement of physical VRAM use or capacity.",
    resource_qualification:
      "Configured context is a provider-reported count. A larger configured context can require more memory, but AI Engine Room does not convert it to bytes. KV-cache bytes and runtime overhead are not separately reported, and compute placement remains unknown.",
    ...overrides,
  };
}

describe("LoadedModelsCard", () => {
  it("renders the runtime name and controlled state label verbatim", () => {
    const { getByText } = render(LoadedModelsCard, {
      props: { loaded: loaded({}) },
    });
    expect(getByText("Ollama")).toBeVisible();
    expect(getByText("Currently loaded in Ollama")).toBeVisible();
  });

  it("renders the controlled interpretation and why-it-matters verbatim", () => {
    const set = loaded({});
    const { getByText } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    expect(getByText(set.interpretation)).toBeVisible();
    expect(getByText(set.why_it_matters)).toBeVisible();
  });

  it("renders one row per loaded model with the reported loaded size formatted", () => {
    const set = loaded({
      models: [
        entry({
          display_name: "example-runner:8b",
          reported_size_bytes: 5_000_000_000,
        }),
        entry({
          display_name: "example-assistant:3b",
          reported_size_bytes: 2_000_000_000,
        }),
      ],
    });
    const { getByText, getAllByText } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    expect(getByText("example-runner:8b")).toBeVisible();
    expect(getByText("example-assistant:3b")).toBeVisible();
    // The reported loaded size is formatted via formatBytes (base-1000 units).
    expect(getByText("5.00 GB")).toBeVisible();
    expect(getAllByText("Loaded size reported by Ollama").length).toBeGreaterThan(0);
  });

  it("renders the reported VRAM size and context length when present", () => {
    const set = loaded({
      models: [
        entry({
          display_name: "example-runner:8b",
          reported_vram_bytes: 4_000_000_000,
          reported_context_length: 8192,
        }),
      ],
    });
    const { getByText } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    expect(getByText("VRAM size reported by Ollama")).toBeVisible();
    expect(getByText("4.00 GB")).toBeVisible();
    expect(getByText("Configured context reported by Ollama")).toBeVisible();
    expect(getByText("8,192")).toBeVisible();
  });

  it("renders explicit unavailable states for fields Ollama did not report", () => {
    const set = loaded({
      models: [
        entry({
          display_name: "bare:1b",
          reported_size_bytes: null,
          reported_vram_bytes: null,
          reported_context_length: null,
        }),
      ],
    });
    const { getByText, getAllByText, queryByText } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    expect(getByText("bare:1b")).toBeVisible();
    expect(getAllByText("Not reported by Ollama")).toHaveLength(3);
    expect(queryByText(/0 B/)).toBeNull();
  });

  it("renders the Rust-controlled resource interpretation and qualification", () => {
    const set = loaded({});
    const { getByText } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    expect(getByText(set.resource_interpretation)).toBeVisible();
    expect(getByText(set.resource_qualification)).toBeVisible();
  });

  it("renders the empty-state message for a valid empty loaded list (not an error)", () => {
    const empty = fixtureEmptyLoadedModels();
    const { getByText, queryByRole } = render(LoadedModelsCard, {
      props: { loaded: empty },
    });
    expect(getByText("No models currently running in Ollama")).toBeVisible();
    // A valid empty loaded list shows no model rows.
    expect(queryByRole("list")).toBeNull();
  });

  it("renders each loaded-model state from the fixtures with backend wording", () => {
    for (const set of fixtureLoadedModelsSets()) {
      const { getByText, unmount } = render(LoadedModelsCard, {
        props: { loaded: set },
      });
      expect(getByText(set.state_label)).toBeVisible();
      expect(getByText(set.interpretation)).toBeVisible();
      expect(getByText(set.why_it_matters)).toBeVisible();
      unmount();
    }
  });

  it("renders no model-action controls (no load/unload/delete/pull)", () => {
    const { container } = render(LoadedModelsCard, {
      props: { loaded: loaded({}) },
    });
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(0);
    expect(container.textContent ?? "").not.toMatch(
      /\b(load|unload|pull|delete|launch|install|remove|generate|benchmark)\b/i,
    );
  });

  it("does not display expires_at (no countdown, no date/time)", () => {
    const set = loaded({
      models: [entry({ expires_at: "2024-01-01T00:00:00.000Z" })],
    });
    const { container } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    const text = container.textContent ?? "";
    // expires_at is carried internally but never rendered.
    expect(text).not.toContain("2024");
    expect(text).not.toContain("expires");
    expect(text).not.toContain("Expire");
    expect(text).not.toMatch(/countdown|remaining/i);
  });

  it("never renders raw transport details or endpoint URLs in any state", () => {
    for (const set of fixtureLoadedModelsSets()) {
      const { container, unmount } = render(LoadedModelsCard, {
        props: { loaded: set },
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
    for (const set of fixtureLoadedModelsSets()) {
      const { container, unmount } = render(LoadedModelsCard, {
        props: { loaded: set },
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
    const set = loaded({
      models: [
        entry({ display_name: "a:1b", reported_size_bytes: 5_000_000_000 }),
        entry({ display_name: "b:1b", reported_size_bytes: 2_000_000_000 }),
      ],
    });
    const { container } = render(LoadedModelsCard, {
      props: { loaded: set },
    });
    const text = container.textContent ?? "";
    // Per-model sizes are shown; no combined/total figure is computed.
    expect(text).not.toContain("7.00 GB");
    expect(text).not.toMatch(/total/i);
  });

  it("does not display a per-model locality badge", () => {
    const { container } = render(LoadedModelsCard, {
      props: { loaded: fixtureLoadedModels() },
    });
    const text = (container.textContent ?? "").toLowerCase();
    expect(text).not.toContain("locality");
    expect(text).not.toContain("undetermined");
    expect(text).not.toContain("local model");
  });

  it("has no automated axe violations (colour-contrast disabled)", async () => {
    const { container } = render(LoadedModelsCard, {
      props: { loaded: loaded({}) },
    });
    await new Promise((r) => setTimeout(r, 0));
    const results = await axe(container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});
