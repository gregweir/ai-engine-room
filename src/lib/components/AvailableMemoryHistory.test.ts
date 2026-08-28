/// <reference types="node" />
import { describe, expect, it } from "vitest";
import { axe } from "vitest-axe";
import { render } from "@testing-library/svelte";
import { readFileSync } from "node:fs";
import type { AvailableMemoryHistoryItem } from "../availableMemoryHistory";
import AvailableMemoryHistory from "./AvailableMemoryHistory.svelte";

const value = (
  observation_id: number,
  bytes: number,
): AvailableMemoryHistoryItem => ({
  observation_id,
  snapshot_at: observation_id,
  event: { kind: "value", bytes, provenance: "operating_system_reported" },
});
const gap = (observation_id: number): AvailableMemoryHistoryItem => ({
  observation_id,
  snapshot_at: null,
  event: { kind: "unavailable", reason: "snapshot_rejected" },
});

describe("AvailableMemoryHistory", () => {
  it("renders one numeric observation with a complete textual equivalent", () => {
    const view = render(AvailableMemoryHistory, {
      props: { history: [value(1, 12_961_325_056)] },
    });
    expect(
      view.getByRole("heading", { name: "Available memory — this session" }),
    ).toBeVisible();
    expect(view.getByText("OS observation")).toBeVisible();
    expect(
      view.getByText(/Spacing shows observation sequence, not time/),
    ).toBeVisible();
    expect(view.getByText("12.96 GB")).toBeVisible();
    expect(view.getByText("12,961,325,056 bytes")).toBeVisible();
    expect(view.getByText("Operating system")).toBeVisible();
    expect(
      view.getByText("One numeric value is shown; no range is implied."),
    ).toBeVisible();
    expect(view.container).not.toHaveTextContent("Scale maximum shown");
    expect(view.container.querySelectorAll("circle")).toHaveLength(1);
    expect(
      view.container.querySelectorAll("rect.available-memory-gap"),
    ).toHaveLength(0);
  });

  it("distinguishes successful zero, repeated values, and a nonnumeric gap", () => {
    const view = render(AvailableMemoryHistory, {
      props: { history: [value(1, 0), value(2, 0), gap(3)] },
    });
    expect(view.getByText("3 observations; 2 numeric readings")).toBeVisible();
    expect(
      view.getByText("Recorded numeric values were identical."),
    ).toBeVisible();
    expect(view.getAllByText(/0 B —/)).toHaveLength(2);
    expect(view.getAllByText(/Snapshot acquisition unavailable/)).toHaveLength(
      2,
    );
    expect(view.container.querySelectorAll("circle")).toHaveLength(2);
    expect(
      view.container.querySelectorAll("rect.available-memory-gap"),
    ).toHaveLength(1);
  });

  it("renders a gap-only history without converting it to zero", () => {
    const view = render(AvailableMemoryHistory, {
      props: { history: [gap(1)] },
    });
    expect(
      view.getByText("No numeric value", { selector: "dd" }),
    ).toBeVisible();
    expect(view.container).not.toHaveTextContent("0 B");
    expect(view.container.querySelector("circle")).toBeNull();
  });

  it("shows an unsafe finite value only as an approximation and not a point", () => {
    const history: AvailableMemoryHistoryItem[] = [
      {
        observation_id: 1,
        snapshot_at: 1,
        event: {
          kind: "unavailable",
          reason: "not_safely_representable",
          approximate_bytes: Number.MAX_SAFE_INTEGER + 1,
        },
      },
    ];
    const view = render(AvailableMemoryHistory, { props: { history } });
    expect(view.getAllByText(/≈9\.01 PB/).length).toBeGreaterThan(0);
    expect(
      view.getByText("Not representable exactly in this presentation"),
    ).toBeVisible();
    expect(view.getByText("Operating system")).toBeVisible();
    expect(view.container.querySelector("circle")).toBeNull();
    expect(
      view.container.querySelector("rect.available-memory-gap"),
    ).not.toBeNull();
  });

  it.each([
    "snapshot_rejected",
    "snapshot_unavailable",
    "missing",
    "unsupported",
    "failed",
    "no_value",
  ] as const)("does not attribute a %s gap to the operating system", (reason) => {
    const history: AvailableMemoryHistoryItem[] = [
      {
        observation_id: 1,
        snapshot_at: null,
        event: { kind: "unavailable", reason },
      },
    ];
    const view = render(AvailableMemoryHistory, { props: { history } });

    expect(view.queryByText("Operating system")).toBeNull();
    expect(view.getByText("No numeric value", { selector: "dd" })).toBeVisible();
  });

  it("uses no connecting path, line, threshold, health judgement, or animation", () => {
    const source = readFileSync(
      "src/lib/components/AvailableMemoryHistory.svelte",
      "utf8",
    );
    expect(source).not.toMatch(/<(?:line|polyline|path|polygon)\b/);
    expect(source).not.toMatch(
      /threshold|trend arrow|healthy|pressure|headroom|model fit/i,
    );
    expect(source).not.toMatch(/animate|transition/);
  });

  it("has no representative axe violations", async () => {
    const view = render(AvailableMemoryHistory, {
      props: { history: [value(1, 1_048_576), gap(2), value(3, 1024)] },
    });
    const results = await axe(view.container, {
      rules: { "color-contrast": { enabled: false } },
    });
    expect(results.violations).toEqual([]);
  });
});
