import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import { axe } from "vitest-axe";
import RuntimeStatusCard from "./RuntimeStatusCard.svelte";
import type { RuntimeStatusView } from "../types";
import { fixtureRuntimeStatuses } from "../fixtures/states";

function status(overrides: Partial<RuntimeStatusView>): RuntimeStatusView {
  return {
    runtime_type: "ollama",
    source_availability: "ready",
    version: "0.1.32",
    version_reported_by: "runtime_reported",
    status_label: "Ollama available",
    interpretation:
      "Ollama is responding at the local endpoint and reported its version.",
    why_it_matters:
      "Engine Room can read the version Ollama reports for this local runtime.",
    ...overrides,
  };
}

describe("RuntimeStatusCard", () => {
  it("renders the runtime name and controlled status label verbatim", () => {
    const { getByText } = render(RuntimeStatusCard, {
      props: { status: status({}) },
    });
    expect(getByText("Ollama")).toBeVisible();
    expect(getByText("Ollama available")).toBeVisible();
  });

  it("renders the controlled interpretation and why-it-matters verbatim", () => {
    const s = status({});
    const { getByText } = render(RuntimeStatusCard, {
      props: { status: s },
    });
    expect(getByText(s.interpretation)).toBeVisible();
    expect(getByText(s.why_it_matters)).toBeVisible();
  });

  it("shows the version only when one was reported", () => {
    const first = render(RuntimeStatusCard, {
      props: { status: status({ version: "0.1.32" }) },
    });
    expect(first.getByText(/Version 0.1.32/)).toBeVisible();
    expect(first.getByText(/reported by the runtime/)).toBeVisible();
    first.unmount();

    const second = render(RuntimeStatusCard, {
      props: {
        status: status({
          version: null,
          version_reported_by: null,
          status_label: "Ollama responded unexpectedly",
        }),
      },
    });
    expect(second.queryByText(/Version/)).toBeNull();
  });

  it("renders each of the four user-facing states from the fixtures", () => {
    for (const s of fixtureRuntimeStatuses()) {
      const { getByText, unmount } = render(RuntimeStatusCard, {
        props: { status: s },
      });
      expect(getByText(s.status_label)).toBeVisible();
      expect(getByText(s.interpretation)).toBeVisible();
      expect(getByText(s.why_it_matters)).toBeVisible();
      unmount();
    }
  });

  it("never renders raw transport details or endpoint URLs in any state", () => {
    for (const s of fixtureRuntimeStatuses()) {
      const { container, unmount } = render(RuntimeStatusCard, {
        props: { status: s },
      });
      const text = (container.textContent ?? "").toLowerCase();
      expect(text).not.toContain("127.0.0.1");
      expect(text).not.toContain("11434");
      expect(text).not.toContain("http");
      expect(text).not.toContain("refus");
      unmount();
    }
  });

  it("never claims install state or health/compat/security/performance", () => {
    for (const s of fixtureRuntimeStatuses()) {
      const { container, unmount } = render(RuntimeStatusCard, {
        props: { status: s },
      });
      const text = (container.textContent ?? "").toLowerCase();
      expect(text).not.toContain("install");
      expect(text).not.toContain("health");
      expect(text).not.toContain("compat");
      expect(text).not.toContain("secure");
      expect(text).not.toContain("fast");
      unmount();
    }
  });

  it("does not derive status from source_availability: not_detected shows backend label, not a transport word", () => {
    const s = fixtureRuntimeStatuses().find(
      (x) => x.source_availability === "not_detected",
    )!;
    const { getByText, queryByText } = render(RuntimeStatusCard, {
      props: { status: s },
    });
    // Controlled backend wording, verbatim.
    expect(
      getByText("Ollama was not detected at the local endpoint"),
    ).toBeVisible();
    // No transport/errno wording inferred by the frontend.
    expect(queryByText(/refus/i)).toBeNull();
    expect(queryByText(/connection/i)).toBeNull();
  });

  it("has no automated axe violations (colour-contrast disabled)", async () => {
    const { container } = render(RuntimeStatusCard, {
      props: { status: status({}) },
    });
    await new Promise((r) => setTimeout(r, 0));
    const results = await axe(container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});
