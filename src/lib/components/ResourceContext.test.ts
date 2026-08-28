/// <reference types="node" />
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { axe } from "vitest-axe";
import { readFileSync } from "node:fs";
import ResourceContext from "./ResourceContext.svelte";
import type { ResourceContextView } from "../types";
import {
  fixtureLoadedModels,
  fixtureResourceContext,
  fixtureResourceContextUnavailable,
  fixtureSnapshot,
} from "../fixtures/states";
import { MockDataSource } from "../datasource/mock";

// The component source is the reliable source of truth for what ships (the
// jsdom harness does not execute module-level behaviour the way a browser
// does). Used to assert the component performs no polling/auto-refresh.
const componentSource = readFileSync(
  "src/lib/components/ResourceContext.svelte",
  "utf8",
);

const CONTEXT_LENGTH_LINE =
  "A larger configured context can require more memory.";

// Forbidden semantic claims (plan §9). Single words are matched with word
// boundaries so harmless substrings do not false-positive (e.g. "Loaded" does
// not match "load"; "model" does not contain "load"). Phrases and symbols are
// matched as substrings. "available" is allowed (it is the metric's name).
const FORBIDDEN_WORDS = [
  "fit",
  "health",
  "healthy",
  "low",
  "dangerous",
  "sufficient",
  "insufficient",
  "headroom",
  "good",
  "bad",
  "safe",
  "performance",
  "fast",
  "slow",
  "total",
  "recommend",
  "unload",
  "load",
];
const FORBIDDEN_PHRASES = ["used by ollama", "percent of ram"];

function assertNoForbiddenClaim(text: string) {
  const lower = text.toLowerCase();
  for (const word of FORBIDDEN_WORDS) {
    const re = new RegExp(`\\b${word}\\b`, "i");
    expect(lower, `unexpected forbidden word "${word}"`).not.toMatch(re);
  }
  for (const phrase of FORBIDDEN_PHRASES) {
    expect(lower, `unexpected forbidden phrase "${phrase}"`).not.toContain(
      phrase,
    );
  }
  expect(lower).not.toContain("%");
}

function assertNoRawLeakage(text: string) {
  const lower = text.toLowerCase();
  expect(lower).not.toContain("127.0.0.1");
  expect(lower).not.toContain("11434");
  expect(lower).not.toContain("http");
  expect(lower).not.toContain("digest");
  expect(lower).not.toContain("endpoint");
}

function context(overrides: Partial<ResourceContextView>): ResourceContextView {
  return { ...fixtureResourceContext(), ...overrides };
}

describe("ResourceContext", () => {
  it("renders an accessible 'Resource context' heading", () => {
    const { getByRole } = render(ResourceContext, {
      props: { context: context({}) },
    });
    expect(
      getByRole("heading", { name: "Resource context", level: 2 }),
    ).toBeVisible();
  });

  it("groups both collapsed native disclosures and every metric in source order", async () => {
    const user = userEvent.setup();
    const snapshot = fixtureSnapshot();
    const { container, getByText, getAllByRole } = render(ResourceContext, {
      props: { context: context({}), entries: snapshot.entries },
    });
    const region = container.querySelector(".resource-context")!;
    const interpretation = getByText(
      "How to interpret memory and loaded-model values",
      { selector: "summary" },
    ).closest("details")!;
    const metrics = getByText("Metric details", {
      selector: "summary",
    }).closest("details")!;

    expect(region).toContainElement(interpretation);
    expect(region).toContainElement(metrics);
    expect(interpretation).not.toHaveAttribute("open");
    expect(metrics).not.toHaveAttribute("open");
    await user.click(getByText("Metric details", { selector: "summary" }));
    expect(metrics).toHaveAttribute("open");
    expect(
      getAllByRole("heading", { level: 2 })
        .filter((heading) => heading.classList.contains("metric-name"))
        .map((heading) => heading.textContent),
    ).toEqual(snapshot.entries.map((entry) => entry.display_name));
  });

  it("renders the interpretation and why_it_matters verbatim (variant 1, with the context-length line)", async () => {
    const user = userEvent.setup();
    const c = fixtureResourceContext();
    const { getByText } = render(ResourceContext, {
      props: { context: c },
    });
    await user.click(
      getByText("How to interpret memory and loaded-model values"),
    );
    expect(getByText(c.interpretation)).toBeVisible();
    expect(getByText(c.why_it_matters)).toBeVisible();
    // The approved qualitative context-length line is present in variant 1.
    expect(c.why_it_matters).toContain(CONTEXT_LENGTH_LINE);
  });

  it("renders no numeric values, model names, or context-length numbers in the block", () => {
    const { container } = render(ResourceContext, {
      props: { context: context({}) },
    });
    const text = container.textContent ?? "";
    // Explanatory only: no formatted byte value, no model name, no
    // context-length number appears (the retained cards show the values).
    expect(text).not.toMatch(/\d/);
    expect(text).not.toContain("example-runner");
    expect(text).not.toContain("example-chat");
    expect(text).not.toMatch(/\b8,?192\b/);
  });

  it("renders no charts, sparklines, progress bars, fit badges, or thresholds", () => {
    const { container } = render(ResourceContext, {
      props: { context: context({}) },
    });
    expect(container.querySelector("canvas")).toBeNull();
    expect(container.querySelector("progress")).toBeNull();
    expect(container.querySelector('[role="meter"]')).toBeNull();
    const text = container.textContent ?? "";
    expect(text).not.toMatch(/\b(fit|health|low|sufficient)\b/i);
    expect(text).not.toContain("%");
  });

  it("renders no action controls (no load/unload/refresh/pull/delete/generate)", () => {
    const { container } = render(ResourceContext, {
      props: { context: context({}) },
    });
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(0);
    const text = container.textContent ?? "";
    expect(text).not.toMatch(
      /\b(load|unload|pull|delete|generate|refresh|poll|launch|install|remove|benchmark)\b/i,
    );
  });

  it("has no polling/auto-refresh: no setInterval or onMount; it renders props only", () => {
    // The shipped component source must not poll or mount a polling loop.
    expect(componentSource).not.toMatch(/setInterval/);
    expect(componentSource).not.toMatch(/onMount/);
    // Re-rendering with new props updates the text (props-driven, not polled).
    const { getByText, rerender } = render(ResourceContext, {
      props: { context: fixtureResourceContext() },
    });
    const unavailable = fixtureResourceContextUnavailable();
    rerender({ context: unavailable });
    expect(getByText(unavailable.interpretation)).toBeInTheDocument();
  });

  it("renders the both-unavailable fixture normally (variant 4, no error, no context-length line)", async () => {
    const user = userEvent.setup();
    const c = fixtureResourceContextUnavailable();
    const { getByText, container } = render(ResourceContext, {
      props: { context: c },
    });
    await user.click(
      getByText("How to interpret memory and loaded-model values"),
    );
    expect(getByText(c.interpretation)).toBeVisible();
    expect(getByText(c.why_it_matters)).toBeVisible();
    // No error state, no crash.
    expect(container.textContent ?? "").toContain(
      "Neither the available-memory reading nor the running-model information is available right now.",
    );
    // The context-length line appears only in variant 1, not variant 4.
    expect(container.textContent ?? "").not.toContain(CONTEXT_LENGTH_LINE);
  });

  it("makes no forbidden arithmetic/health/fit claim in any fixture variant", () => {
    for (const c of [
      fixtureResourceContext(),
      fixtureResourceContextUnavailable(),
    ]) {
      const { container, unmount } = render(ResourceContext, {
        props: { context: c },
      });
      assertNoForbiddenClaim(container.textContent ?? "");
      unmount();
    }
  });

  it("leaks no raw transport or endpoint detail in any fixture variant", () => {
    for (const c of [
      fixtureResourceContext(),
      fixtureResourceContextUnavailable(),
    ]) {
      const { container, unmount } = render(ResourceContext, {
        props: { context: c },
      });
      assertNoRawLeakage(container.textContent ?? "");
      unmount();
    }
  });

  it("does not duplicate numeric values from the retained cards", () => {
    // The block is explanatory only: it carries no value/size field, only the
    // controlled text. A value that appears in the loaded-models fixture must
    // not appear here.
    const { container } = render(ResourceContext, {
      props: { context: context({}) },
    });
    const text = container.textContent ?? "";
    expect(text).not.toContain("5.00 GB");
    expect(text).not.toContain("4.00 GB");
  });

  it("mock browser path: MockDataSource.composeResourceContext returns the fixture without contacting a runtime", async () => {
    const mock = new MockDataSource();
    const result = await mock.composeResourceContext(
      fixtureSnapshot(),
      fixtureLoadedModels(),
    );
    expect(result).toEqual(fixtureResourceContext());
  });

  it("has no automated axe violations (colour-contrast disabled)", async () => {
    const { container } = render(ResourceContext, {
      props: { context: context({}), entries: fixtureSnapshot().entries },
    });
    await new Promise((r) => setTimeout(r, 0));
    const results = await axe(container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });

  it("has no automated axe violations for the both-unavailable variant (colour-contrast disabled)", async () => {
    const { container } = render(ResourceContext, {
      props: { context: fixtureResourceContextUnavailable() },
    });
    await new Promise((r) => setTimeout(r, 0));
    const results = await axe(container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});
